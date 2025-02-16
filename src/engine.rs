use chess::{Board, ChessMove, MoveGen, Color, Square};
use crate::psts::{PAWN_PST, KNIGHT_PST, BISHOP_PST, QUEEN_PST, KING_PST, ROOK_PST};
use std::time::{Instant, Duration};
use rayon::prelude::*;

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

fn piece_square_value(piece: chess::Piece, square: Square, color: Color) -> i32 {
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

/// Generate all legal moves, but now prioritize moves that immediately deliver mate,
/// then captures (via MVV-LVA), then quiet moves that give check.
fn generate_ordered_moves(board: &Board) -> Vec<ChessMove> {
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();

    moves.sort_by_key(|mv| {
        // Create the new board after this move.
        let new_board = board.make_move_new(*mv);
        // If the move delivers checkmate, give it the highest priority.
        if new_board.status() == chess::BoardStatus::Checkmate {
            return -1_000_000; // a very low key => highest priority when sorting in ascending order
        }
        // For capture moves, use MVV-LVA.
        if let Some(victim) = board.piece_on(mv.get_dest()) {
            let attacker = board.piece_on(mv.get_source()).unwrap();
            -(piece_value(victim) - piece_value(attacker))
        } else {
            // For quiet moves that deliver check, give them a bonus.
            if new_board.checkers().popcnt() > 0 {
                -10_000
            } else {
                0 // lowest priority among our ordering
            }
        }
    });

    moves
}

/// Quiescence search (unchanged)
fn quiesce(
    board: &Board,
    mut alpha: i32,
    beta: i32,
    color: i32,
    start_time: Instant,
    time_limit: Duration,
) -> i32 {
    if start_time.elapsed() >= time_limit {
        return color * evaluate_board(board);
    }

    let stand_pat = color * evaluate_board(board);
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut q_moves = Vec::new();
    for mv in MoveGen::new_legal(board) {
        if board.piece_on(mv.get_dest()).is_some() {
            q_moves.push(mv);
        } else {
            let new_board = board.make_move_new(mv);
            if new_board.checkers().popcnt() > 0 {
                q_moves.push(mv);
            }
        }
    }

    // Order the moves: mate moves should have already been prioritized in generate_ordered_moves,
    // but here we use the same MVV-LVA and check bonus for quiet checks.
    q_moves.sort_by_key(|mv| {
        if let Some(victim) = board.piece_on(mv.get_dest()) {
            let attacker = board.piece_on(mv.get_source()).unwrap();
            -(piece_value(victim) - piece_value(attacker))
        } else {
            -10_000
        }
    });

    for mv in q_moves {
        if start_time.elapsed() >= time_limit {
            return alpha;
        }
        let new_board = board.make_move_new(mv);
        let score = -quiesce(&new_board, -beta, -alpha, -color, start_time, time_limit);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

/// Negamax with alpha-beta pruning.
fn negamax(
    board: &Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    color: i32,
    start_time: Instant,
    time_limit: Duration,
) -> i32 {
    if start_time.elapsed() >= time_limit {
        return color * evaluate_board(board);
    }

    match board.status() {
        chess::BoardStatus::Ongoing => {
            if depth == 0 {
                return quiesce(board, alpha, beta, color, start_time, time_limit);
            }
        }
        chess::BoardStatus::Checkmate => {
            return -(MATE_SCORE - depth as i32);
        }
        chess::BoardStatus::Stalemate => {
            return 0;
        }
    }

    let mut best_value = i32::MIN;
    let mut current_alpha = alpha;

    for mv in generate_ordered_moves(board) {
        if start_time.elapsed() >= time_limit {
            break;
        }
        let new_board = board.make_move_new(mv);
        let value = -negamax(&new_board, depth - 1, -beta, -current_alpha, -color, start_time, time_limit);
        if value > best_value {
            best_value = value;
        }
        if value > current_alpha {
            current_alpha = value;
        }
        if current_alpha >= beta {
            break;
        }
    }

    best_value
}

/// Parallelized search for the best move at a given depth.
///
/// This version uses Rayon to evaluate each candidate move from the root in parallel.
/// (Note: early exit on mate detection is not implemented here.)
fn best_move_at_depth(
    board: &Board,
    depth: u32,
    start_time: Instant,
    time_limit: Duration,
) -> Option<(i32, ChessMove)> {
    let color = if board.side_to_move() == Color::White { 1 } else { -1 };

    let alpha = i32::MIN + 1;
    let beta = i32::MAX - 1;

    // Get the ordered moves at the root.
    let moves = generate_ordered_moves(board);

    // Evaluate each move in parallel.
    let results: Vec<(i32, ChessMove)> = moves.par_iter()
        .filter_map(|&mv| {
            // Check time in each thread.
            if start_time.elapsed() >= time_limit {
                None
            } else {
                let new_board = board.make_move_new(mv);
                let value = -negamax(&new_board, depth - 1, -beta, -alpha, -color, start_time, time_limit);
                Some((value, mv))
            }
        })
        .collect();

    // Choose the move with the highest score.
    results.into_iter().max_by_key(|(score, _)| *score)
}

/// Iterative deepening with a 15-second time limit.
pub fn best_move_iterative(board: &Board, max_depth: u32) -> Option<ChessMove> {
    let start_time = Instant::now();
    let time_limit = Duration::from_secs(15);

    let mut best_move_overall = None;
    let mut best_eval_overall = i32::MIN;

    for depth in 1..=max_depth {
        if start_time.elapsed() >= time_limit {
            break;
        }
        if let Some((score, mv)) = best_move_at_depth(board, depth, start_time, time_limit) {
            best_eval_overall = score;
            best_move_overall = Some(mv);
            // If we have found a mate sequence, no need to search deeper.
            if score >= MATE_SCORE - depth as i32 {
                break;
            }
        } else {
            break;
        }
    }

    best_move_overall
}
