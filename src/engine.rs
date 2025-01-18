// engine.rs
use chess::{
    Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square, ALL_SQUARES
};

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000, // Very high so we don't lose the king 'cheaply'
    }
}

/*
// Explanation of PST indexing:
   - The chess crate's Square::to_index() goes from 0..=63 in row-major order:
     Rank1 (A1..H1) => 0..7
     Rank2 (A2..H2) => 8..15
     ...
     Rank8 (A8..H8) => 56..63

   - We'll define PSTs from White's perspective so that
     PST[index_of_A1..H1] is the bottom row (Rank1).
   - For Black, we'll mirror by doing 63 - square_index.
*/

// Pawn PST (white perspective)
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

fn piece_square_value(piece: Piece, square: Square, color: Color) -> i32 {
    // Convert square to index 0..63
    let idx = square.to_index() as usize;
    
    // For black, we flip the index to mirror (63 - idx)
    let table_index = match color {
        Color::White => idx,
        Color::Black => 63 - idx,
    };

    match piece {
        Piece::Pawn => PAWN_PST[table_index],
        Piece::Knight => KNIGHT_PST[table_index],
        Piece::Bishop => BISHOP_PST[table_index],
        Piece::Rook => ROOK_PST[table_index],
        Piece::Queen => QUEEN_PST[table_index],
        Piece::King => KING_PST[table_index],
    }
}

fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;

    for square in ALL_SQUARES {
        if let Some(piece) = board.piece_on(square) {
            let color_on_square = board.color_on(square).unwrap();

            // Add (for white) or subtract (for black)
            let piece_score = piece_value(piece) + piece_square_value(piece, square, color_on_square);
            if color_on_square == Color::White {
                score += piece_score;
            } else {
                score -= piece_score;
            }
        }
    }

    score
}

fn negamax(board: &Board, depth: u32, mut alpha: i32, beta: i32, color: i32) -> i32 {
    match board.status() {
        BoardStatus::Ongoing => {
            if depth == 0 {
                return color * evaluate_board(board);
            }
        }
        BoardStatus::Checkmate => {
            // Side to move has been checkmated => big negative
            return -10_000;
        }
        BoardStatus::Stalemate => {
            // A draw
            return 0;
        }
    }

    let mut best_value = i32::MIN;
    let mut current_alpha = alpha;

    for mv in MoveGen::new_legal(board) {
        let new_board = board.make_move_new(mv);
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

pub fn best_move(board: &Board, depth: u32) -> Option<ChessMove> {
    let color = if board.side_to_move() == Color::White { 1 } else { -1 };

    let mut best_mv = None;
    let mut best_eval = i32::MIN;
    let mut alpha = i32::MIN + 1; //add 1 to  avoid overflow
    let beta = i32::MAX - 1;      //subtract 1  avoid overflow

    for mv in MoveGen::new_legal(board) {
        let new_board = board.make_move_new(mv);

        let value = -negamax(&new_board, depth - 1, -beta, -alpha, -color);
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

    best_mv
}