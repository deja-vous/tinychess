//interfacing best moves to lichess bot, ignore and create your own GUI if you dont want to use lichess
mod engine;
mod psts;
use std::time::Duration;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use engine::best_move_iterative;
use futures_util::TryStreamExt;
use reqwest::{header::USER_AGENT, Client};
use serde::Deserialize;
use std::{env, error::Error, str::FromStr, sync::Arc};
use tokio::{spawn, sync::Mutex};
use tokio_stream::{Stream, StreamExt};
use tokio_util::{
    codec::{FramedRead, LinesCodec, LinesCodecError},
    io::StreamReader,
};

// Update these structs to parse color info from Lichess
#[derive(Debug, Deserialize)]
struct Player {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum IncomingEvent {
    #[serde(rename = "challenge")]
    Challenge { challenge: Challenge },
    #[serde(rename = "gameStart")]
    GameStart { game: Game },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct Challenge {
    id: String,
}

#[derive(Debug, Deserialize)]
struct Game {
    id: String,
}

// Expanded GameEvent to capture player info
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum GameEvent {
    #[serde(rename = "gameFull")]
    GameFull {
        id: String,
        white: Player,
        black: Player,
        state: GameState,
    },
    #[serde(rename = "gameState")]
    GameState(GameState),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, Clone)]
struct GameState {
    moves: String,
    status: Option<String>,
}

// Track game state locally, including which color we play
struct GameTracker {
    board: Board,
    my_color: Color,
    game_id: String,
}

// Replace with your actual bot's username on Lichess:
const MY_BOT_USERNAME: &str = "insert-your-bot-username-here";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("LICHESS_BOT_TOKEN")
        .expect("Please set LICHESS_BOT_TOKEN in your environment!");

    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let event_stream_url = "https://lichess.org/api/stream/event";
    let mut event_stream = stream_endpoint(&client, &token, event_stream_url).await?;

    println!("Bot is now listening for events...");

    // For concurrency safety if you want to track multiple games
    let active_games = Arc::new(Mutex::new(Vec::new()));

    while let Some(Ok(line_str)) = event_stream.next().await {
        // Ignore keep-alive lines or empty lines
        if line_str.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<IncomingEvent>(&line_str) {
            Ok(IncomingEvent::Challenge { challenge }) => {
                println!("Challenge received: {:?}... Accepting...", challenge.id);
                accept_challenge(&client, &token, &challenge.id).await?;
            }
            Ok(IncomingEvent::GameStart { game }) => {
                println!("Game started: {:?}", game.id);
                let game_id = game.id.clone();
                let client_clone = client.clone();
                let token_clone = token.clone();
                let active_games_ref = Arc::clone(&active_games);

                spawn(async move {
                    if let Err(err) =
                        handle_game(&client_clone, &token_clone, &game_id, active_games_ref).await
                    {
                        eprintln!("Error in game {game_id}: {err}");
                    }
                });
            }
            Ok(IncomingEvent::Other) => {}
            Err(e) => eprintln!("Error parsing event stream: {e}. Raw line = {line_str}"),
        }
    }

    Ok(())
}

async fn handle_game(
    client: &Client,
    token: &str,
    game_id: &str,
    active_games_ref: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn Error>> {
    // Add this game to "active games"
    {
        let mut ag = active_games_ref.lock().await;
        ag.push(game_id.to_string());
    }

    let url = format!("https://lichess.org/api/bot/game/stream/{game_id}");
    let mut stream = stream_endpoint(client, token, &url).await?;

    // We'll maintain our local tracker, which includes board + color
    let mut tracker: Option<GameTracker> = None;

    while let Some(Ok(line_str)) = stream.next().await {
        if line_str.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<GameEvent>(&line_str) {
            // This event gives us the full game state, including which side is which
            Ok(GameEvent::GameFull { id, white, black, state }) => {
                println!("Received GameFull for game {id}");

                // Figure out if we are White or Black
                let my_color = if white.id == MY_BOT_USERNAME {
                    Color::White
                } else {
                    Color::Black
                };

                // Create a default board, then apply all moves so far
                let mut board = Board::default();
                apply_uci_moves(&mut board, &state.moves);

                // Store into our local tracker
                tracker = Some(GameTracker {
                    board,
                    my_color,
                    game_id: id,
                });

                // If it's our turn, try a move
                if let Some(t) = &mut tracker {
                    if t.board.side_to_move() == t.my_color
                        && t.board.status() == BoardStatus::Ongoing
                    {
                        try_play_move(client, token, t).await?;
                    }
                }
            }

            // This event updates us with new moves in the game
            Ok(GameEvent::GameState(state)) => {
                if let Some(t) = &mut tracker {
                    // Rebuild board from scratch
                    let mut new_board = Board::default();
                    apply_uci_moves(&mut new_board, &state.moves);
                    t.board = new_board;

                    // Check if game ended
                    if let Some(status) = &state.status {
                        if status != "started" {
                            println!("Game {} ended with status {}", t.game_id, status);
                            break;
                        }
                    }

                    // If it's our turn, try a move
                    if t.board.side_to_move() == t.my_color
                        && t.board.status() == BoardStatus::Ongoing
                    {
                        try_play_move(client, token, t).await?;
                    }
                }
            }
            Ok(GameEvent::Other) => {}
            Err(e) => eprintln!("Error parsing game event: {e}. Raw line = {line_str}"),
        }
    }

    // Remove this game from "active games"
    {
        let mut ag = active_games_ref.lock().await;
        ag.retain(|id| id != game_id);
    }

    Ok(())
}


async fn try_play_move(
    client: &Client,
    token: &str,
    tracker: &mut GameTracker,
) -> Result<(), Box<dyn Error>> {
    if tracker.board.status() == BoardStatus::Ongoing {
        if let Some(chosen_move) = best_move_iterative(&tracker.board, 5) {
            let uci = format_move_as_uci(chosen_move);
            let url = format!(
                "https://lichess.org/api/bot/game/{}/move/{}",
                tracker.game_id, uci
            );

            //  Add delay here to prevent rate limiting 
            tokio::time::sleep(Duration::from_millis(100)).await;

            println!("Playing move {uci} for game {}", tracker.game_id);
            let resp = client.post(url).bearer_auth(token).send().await?;

            if resp.status().is_success() {
                tracker.board = tracker.board.make_move_new(chosen_move);
            } else {
                eprintln!(
                    "Move {uci} was rejected for game {}: {}",
                    tracker.game_id,
                    resp.text().await?
                );
            }
        }
    }
    Ok(())
}

async fn accept_challenge(client: &Client, token: &str, challenge_id: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://lichess.org/api/challenge/{challenge_id}/accept");
    let resp = client.post(&url).bearer_auth(token).send().await?;
    if !resp.status().is_success() {
        eprintln!(
            "Failed to accept challenge {challenge_id}: {}",
            resp.text().await?
        );
    }
    Ok(())
}

async fn stream_endpoint(
    client: &Client,
    token: &str,
    url: &str,
) -> Result<impl Stream<Item = Result<String, std::io::Error>>, Box<dyn Error>> {
    let resp = client.get(url).bearer_auth(token).send().await?.error_for_status()?;
    let byte_stream = resp.bytes_stream().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
    let line_stream = FramedRead::new(StreamReader::new(byte_stream), LinesCodec::new())
        .map(|res: Result<String, LinesCodecError>| {
            res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
    Ok(line_stream)
}

fn apply_uci_moves(board: &mut Board, moves_str: &str) {
    if moves_str.is_empty() {
        return;
    }
    for mv in moves_str.split_whitespace() {
        if let Ok(chess_move) = parse_uci_move(board, mv) {
            *board = board.make_move_new(chess_move);
        }
    }
}

fn parse_uci_move(board: &Board, uci: &str) -> Result<ChessMove, ()> {
    if uci.len() < 4 {
        return Err(());
    }

    let (src_str, dst_str) = (&uci[0..2], &uci[2..4]);
    let promo_char = uci.chars().nth(4);

    let src_sq = Square::from_str(src_str).map_err(|_| ())?;
    let dst_sq = Square::from_str(dst_str).map_err(|_| ())?;
    let promotion_piece = match promo_char {
        Some('q') => Some(Piece::Queen),
        Some('r') => Some(Piece::Rook),
        Some('b') => Some(Piece::Bishop),
        Some('n') => Some(Piece::Knight),
        None => None,
        _ => return Err(()),
    };

    for legal_mv in MoveGen::new_legal(board) {
        if legal_mv.get_source() == src_sq
            && legal_mv.get_dest() == dst_sq
            && legal_mv.get_promotion() == promotion_piece
        {
            return Ok(legal_mv);
        }
    }
    Err(())
}

fn format_move_as_uci(chess_move: ChessMove) -> String {
    format!(
        "{}{}{}",
        chess_move.get_source(),
        chess_move.get_dest(),
        match chess_move.get_promotion() {
            Some(Piece::Knight) => "n",
            Some(Piece::Bishop) => "b",
            Some(Piece::Rook) => "r",
            Some(Piece::Queen) => "q",
            _ => "",
        }
    )
}
