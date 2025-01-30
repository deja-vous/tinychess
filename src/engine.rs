use chess::{Board, ChessMove, MoveGen, Color};

const MATE_SCORE: i32 = 100_000;

fn piece_value(piece: chess::Piece) -> i32 {
    match piece {
        chess::Piece::Pawn => 100,
        chess::Piece::Knight => 320,
        chess::Piece::Bishop => 330,
        chess::Piece::Rook => 500,
        chess::Piece::Queen => 900,
        chess::Piece::King => 20_000,
    }
}

static PAWN_PST: [i32; 64] = [
    // RANK 1 (A1..H1)
     0,   0,   0,   0,   0,   0,   0,   0,
    // RANK 2 (A2..H2)
     5,  10,  10, -20, -20,  10,  10,   5,
    // RANK 3 (A3..H3)
     5,  -5, -10,   0,   5, -10,  -5,   5,
    // RANK 4 (A4..H4)
     0,   0,   0,  20,  20,   0,   0,   0,
    // RANK 5 (A5..H5)
     5,   5,  10,  25,  25,  10,   5,   5,
    // RANK 6 (A6..H6)
    10,  10,  20,  30,  30,  20,  10,  10,
    // RANK 7 (A7..H7)
    50,  50,  50,  50,  50,  50,  50,  50,
    // RANK 8 (A8..H8)
     0,   0,   0,   0,   0,   0,   0,   0,
];

// Knight PST
static KNIGHT_PST: [i32; 64] = [
    // RANK 1
    -50, -40, -30, -30, -30, -30, -40, -50,
    // RANK 2
    -40, -20,   0,   0,   0,   0, -20, -40,
    // RANK 3
    -30,   0,  10,  15,  15,  5,   0, -30,
    // RANK 4
    -30,   5,  15,  20,  20,  15,   5, -30,
    // RANK 5
    -30,   0,  15,  20,  20,  15,   0, -30,
    // RANK 6
    -30,   5,  10,  15,  15,  10,   5, -30,
    // RANK 7
    -40, -20,   0,   5,   5,   0, -20, -40,
    // RANK 8
    -50, -40, -30, -30, -30, -30, -40, -50,
];

// Bishop PST
static BISHOP_PST: [i32; 64] = [
    // RANK 1
    -20, -10, -10, -10, -10, -10, -10, -20,
    // RANK 2
    -10,   5,   0,   0,   0,   0,   5, -10,
    // RANK 3
    -10,  10,  10,  10,  10,  10,  10, -10,
    // RANK 4
    -10,   0,  10,  10,  10,  10,   0, -10,
    // RANK 5
    -10,   5,   5,  10,  10,   5,   5, -10,
    // RANK 6
    -10,   0,   5,  10,  10,   5,   0, -10,
    // RANK 7
    -10,   0,   0,   0,   0,   0,   0, -10,
    // RANK 8
    -20, -10, -10, -10, -10, -10, -10, -20,
];

// Rook PST
static ROOK_PST: [i32; 64] = [
    // RANK 1
     0,   0,   0,   5,   5,   0,   0,   0,
    // RANK 2
    -5,   0,   0,   0,   0,   0,   0,  -5,
    // RANK 3
    -5,   0,   0,   0,   0,   0,   0,  -5,
    // RANK 4
    -5,   0,   0,   0,   0,   0,   0,  -5,
    // RANK 5
    -5,   0,   0,   0,   0,   0,   0,  -5,
    // RANK 6
    -5,   0,   0,   0,   0,   0,   0,  -5,
    // RANK 7
     5,  10,  10,  10,  10,  10,  10,   5,
    // RANK 8
     0,   0,   0,   5,   5,   0,   0,   0,
];

// Queen PST
static QUEEN_PST: [i32; 64] = [
    // RANK 1
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    // RANK 2
    -10,   0,   5,   0,   0,   0,   0, -10,
    // RANK 3
    -10,   5,   5,   5,   5,   5,   0, -10,
    // RANK 4
     -5,   0,   5,   5,   5,   5,   0,  -5,
    // RANK 5
      0,   0,   5,   5,   5,   5,   0,  -5,
    // RANK 6
    -10,   5,   5,   5,   5,   5,   0, -10,
    // RANK 7
    -10,   0,   5,   0,   0,   0,   0, -10,
    // RANK 8
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

// King PST (middlegame-ish)
static KING_PST: [i32; 64] = [
    // RANK 1
    -30, -40, -40, -50, -50, -40, -40, -30,
    // RANK 2
    -30, -40, -40, -50, -50, -40, -40, -30,
    // RANK 3
    -30, -40, -40, -50, -50, -40, -40, -30,
    // RANK 4
    -30, -40, -40, -50, -50, -40, -40, -30,
    // RANK 5
    -20, -30, -30, -40, -40, -30, -30, -20,
    // RANK 6
     20,  20,   0,   0,   0,   0,  20,  20,
    // RANK 7
     30,  30,  20,   0,   0,  20,  30,  30,
    // RANK 8
     50,  50,  30,   0,   0,  30,  50,  50,
];

fn piece_square_value(piece: chess::Piece, square: chess::Square, color: Color) -> i32 {
    let idx = square.to_index() as usize;
    let table_index = match color {
        Color::White => idx,
        Color::Black => 63 - idx,
    };

    match piece {
        chess::Piece::Pawn => PAWN_PST[table_index],
        chess::Piece::Knight => KNIGHT_PST[table_index],
        chess::Piece::Bishop => BISHOP_PST[table_index],
        chess::Piece::Rook => ROOK_PST[table_index],
        chess::Piece::Queen => QUEEN_PST[table_index],
        chess::Piece::King => KING_PST[table_index],
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

/// Generate all legal moves, ordered so that captures come first,
/// sorted by MVV-LVA (Most Valuable Victim - Least Valuable Attacker).
fn generate_ordered_moves(board: &Board) -> Vec<ChessMove> {
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();

    moves.sort_by_key(|mv| {
        // If the move is a capture, let's compute MVV-LVA
        if let Some(victim) = board.piece_on(mv.get_dest()) {
            let attacker = board.piece_on(mv.get_source()).unwrap();
            let score = piece_value(victim) - piece_value(attacker);
            
            -(score)
        } else {
            // Non-captures go last; use a very large negative so they come after any capture
            i32::MIN
        }
    });

    moves
}


fn negamax(board: &Board, depth: u32, mut alpha: i32, beta: i32, color: i32) -> i32 {
    match board.status() {
        chess::BoardStatus::Ongoing => {
            if depth == 0 {
                return color * evaluate_board(board);
            }
        }
        chess::BoardStatus::Checkmate => {
           
            return -MATE_SCORE;
        }
        chess::BoardStatus::Stalemate => {
            return 0; 
        }
    }

    let mut best_value = i32::MIN;
    let mut current_alpha = alpha;

    // Use ordered moves for searching
    for mv in generate_ordered_moves(board) {
        let new_board = board.make_move_new(mv);
        // Negamax recursion
        let value = -negamax(&new_board, depth - 1, -beta, -current_alpha, -color);

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


fn best_move_at_depth(board: &Board, depth: u32) -> Option<(i32, ChessMove)> {
    let color = if board.side_to_move() == Color::White { 1 } else { -1 };

    let mut best_eval = i32::MIN;
    let mut best_mv = None;


    let mut alpha = i32::MIN + 1; 
    let beta = i32::MAX - 1;      

    // Use ordered moves for the root
    for mv in generate_ordered_moves(board) {
        let new_board = board.make_move_new(mv);

        // Negamax call with "depth-1" because we're using one ply already for this move
        let value = -negamax(&new_board, depth - 1, -beta, -alpha, -color);

        if value > best_eval {
            best_eval = value;
            best_mv = Some(mv);
        }
        if value > alpha {
            alpha = value;
        }
        if alpha >= beta {
            // alpha-beta cutoff
            break;
        }
    }

    best_mv.map(|mv| (best_eval, mv))
}


pub fn best_move_iterative(board: &Board, max_depth: u32) -> Option<ChessMove> {
    let mut best_eval_overall = i32::MIN;
    let mut best_move_overall = None;

    for depth in 1..=max_depth {
        // Attempt a full search at this depth
        if let Some((score, mv)) = best_move_at_depth(board, depth) {
            best_eval_overall = score;
            best_move_overall = Some(mv);
        }
    }

    best_move_overall
}
