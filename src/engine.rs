use chess::{Board, ChessMove, MoveGen, Color};
mod psts;
use psts::{PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST};

const MATE_SCORE: i32 = 100_000;

fn piece_value(piece: chess::Piece) -> i32 {
    match piece {
        chess::Piece::Pawn   => 100,
        chess::Piece::Knight => 320,
        chess::Piece::Bishop => 330,
        chess::Piece::Rook   => 500,
        chess::Piece::Queen  => 900,
        chess::Piece::King   => 20_000,
    }
}


fn piece_square_value(piece: chess::Piece, square: chess::Square, color: Color) -> i32 {
    let idx = square.to_index() as usize;
    let table_index = match color {
        Color::White => idx,
        Color::Black => 63 - idx,
    };

    match piece {
        chess::Piece::Pawn   => PAWN_PST[table_index],
        chess::Piece::Knight => KNIGHT_PST[table_index],
        chess::Piece::Bishop => BISHOP_PST[table_index],
        chess::Piece::Rook   => ROOK_PST[table_index],
        chess::Piece::Queen  => QUEEN_PST[table_index],
        chess::Piece::King   => KING_PST[table_index],
    }
}

fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;
    for sq in chess::ALL_SQUARES {
        if let Some(piece) = board.piece_on(sq) {
            let color_on_sq = board.color_on(sq).unwrap();
            let piece_score = piece_value(piece) + piece_square_value(piece, sq, color_on_sq);
            if color_on_sq == Color::White {
                score += piece_score;
            } else {
                score -= piece_score;
            }
        }
    }
    score
}

/// Generate all legal moves, ordering captures first by MVV–LVA.
fn generate_ordered_moves(board: &Board) -> Vec<ChessMove> {
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();

    moves.sort_by_key(|mv| {
        if let Some(victim) = board.piece_on(mv.get_dest()) {
            let attacker = board.piece_on(mv.get_source()).unwrap();
            let score = piece_value(victim) - piece_value(attacker);
            -score // Higher scores come first.
        } else {
            i32::MIN // Non-captures go last.
        }
    });

    moves
}

/// A simple game state that supports push/pop (make/undo move).
struct GameState {
    board: Board,
    history: Vec<Board>,
}

impl GameState {
    fn new(board: Board) -> Self {
        Self {
            board,
            history: Vec::new(),
        }
    }

    /// Push a move: store the current board on the history stack and update the board.
    fn push(&mut self, mv: ChessMove) {
        self.history.push(self.board);
        self.board = self.board.make_move_new(mv);
    }

    /// Pop the last move, restoring the previous board state.
    fn pop(&mut self) {
        self.board = self.history.pop().expect("No board in history to pop!");
    }
}

/// QUISCENCE SEARCH using the push/pop game state.
fn quiescence(game_state: &mut GameState, mut alpha: i32, beta: i32, color: i32) -> i32 {
    let board = &game_state.board;

    match board.status() {
        chess::BoardStatus::Ongoing => {},
        chess::BoardStatus::Checkmate => return -MATE_SCORE,
        chess::BoardStatus::Stalemate => return 0,
    }

    // Stand-pat evaluation.
    let stand_pat = color * evaluate_board(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    // Generate only capture moves.
    let mut capture_moves: Vec<ChessMove> = MoveGen::new_legal(board)
        .filter(|mv| board.piece_on(mv.get_dest()).is_some())
        .collect();

    capture_moves.sort_by_key(|mv| {
        if let Some(victim) = board.piece_on(mv.get_dest()) {
            let attacker = board.piece_on(mv.get_source()).unwrap();
            let score = piece_value(victim) - piece_value(attacker);
            -score
        } else {
            i32::MIN
        }
    });

    for mv in capture_moves {
        game_state.push(mv);
        let score = -quiescence(game_state, -beta, -alpha, -color);
        game_state.pop();

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

/// NEGAMAX SEARCH using the push/pop mechanism.
fn negamax(game_state: &mut GameState, depth: u32, mut alpha: i32, beta: i32, color: i32) -> i32 {
    let board = &game_state.board;
    match board.status() {
        chess::BoardStatus::Ongoing => {},
        chess::BoardStatus::Checkmate => return -MATE_SCORE,
        chess::BoardStatus::Stalemate => return 0,
    }

    if depth == 0 {
        return quiescence(game_state, alpha, beta, color);
    }

    let mut best_value = i32::MIN;
    let mut current_alpha = alpha;

    for mv in generate_ordered_moves(board) {
        game_state.push(mv);
        let value = -negamax(game_state, depth - 1, -beta, -current_alpha, -color);
        game_state.pop();

        if value > best_value {
            best_value = value;
        }
        if value > current_alpha {
            current_alpha = value;
        }
        if current_alpha >= beta {
            break; // alpha-beta cutoff
        }
    }

    best_value
}

/// At the root, try all moves using push/pop.
fn best_move_at_depth(board: &Board, depth: u32) -> Option<(i32, ChessMove)> {
    let color = if board.side_to_move() == Color::White { 1 } else { -1 };

    let mut best_eval = i32::MIN;
    let mut best_mv = None;

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX - 1;

    // Create a game state for push/pop.
    let mut game_state = GameState::new(board.clone());

    for mv in generate_ordered_moves(&game_state.board) {
        game_state.push(mv);
        let value = -negamax(&mut game_state, depth - 1, -beta, -alpha, -color);
        game_state.pop();

        if value > best_eval {
            best_eval = value;
            best_mv = Some(mv);
        }
        if value > alpha {
            alpha = value;
        }
        if alpha >= beta {
            break;
        }
    }

    best_mv.map(|mv| (best_eval, mv))
}

pub fn best_move_iterative(board: &Board, max_depth: u32) -> Option<ChessMove> {
    let mut best_move_overall = None;

    for depth in 1..=max_depth {
        if let Some((_score, mv)) = best_move_at_depth(board, depth) {
            best_move_overall = Some(mv);
        }
    }

    best_move_overall
}
