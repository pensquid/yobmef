use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Movement, Piece, Square};

// SHITTY SHIT HERE, JUST FOR EXPERIMENTATION, NOT FOR USE IN FINAL PROGRAM
// Inspiration from:
// https://www.chess.com/article/view/the-evaluation-of-material-imbalances-by-im-larry-kaufman
// https://www.chessprogramming.org/Simplified_Evaluation_Function

#[rustfmt::skip]
const PAWN_VALUE_TABLE: [i16; 64] = [
    0,  0,  0,   0,   0,   0,  0,  0,
    5,  10, 10, -20, -20,  10, 10, 5,
    5, -5, -10,  0,   0,  -10, -5,  5,
    0,  0,  0,   20,  20,  0,   0,  0,
    5,  5,  10,  25,  25,  10,  5,  5,
    10, 10, 20,  30,  30,  20,  10, 10,
    50, 50, 50,  50,  50,  50,  50, 50,
    0,  0,  0,   0,   0,   0,   0,  0,
];

#[rustfmt::skip]
const KNIGHT_VALUE_TABLE: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,  0,   5,   5,   0,  -20, -40,
    -30,  5,   10,  15,  15,  10,  5,  -30,
    -30,  0,   15,  20,  20,  15,  0,  -30,
    -30,  5,   15,  20,  20,  15,  5,  -30,
    -30,  0,   10,  15,  15,  10,  0,  -30,
    -40, -20,  0,   0,   0,   0,  -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_VALUE_TABLE: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,  5,   0,   0,   0,   0,   5,  -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,  0,   10,  10,  10,  10,  0,  -10,
    -10,  5,   5,   10,  10,  5,   5,  -10,
    -10,  0,   5,   10,  10,  5,   0,  -10,
    -10,  0,   0,   0,   0,   0,   0,  -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_VALUE_TABLE: [i16; 64] = [
    -5,    0,   0,   5,  5,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
    -5,    0,   0,   0,  0,   0,   0,  -5,
     5,    10,  10,  10, 10,  10,  10,  5,
     0,    0,   0,   0,  0,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_VALUE_TABLE: [i16; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10,  0,   5,   0,  0,  0,   0,  -10,
    -10,  5,   5,   5,  5,  5,   0,  -10,
     0,   0,   5,   5,  5,  5,   0,  -5,
    -5,   0,   5,   5,  5,  5,   0,  -5,
    -10,  0,   5,   5,  5,  5,   0,  -10,
    -10,  0,   0,   0,  0,  0,   0,  -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_VALUE_TABLE_MIDDLEGAME: [i16; 64] = [
     20,  50,  40,    0,   0,  10,  50,   20,
     20,  20,   0,    0,   0,   0,  20,   20,
    -10, -20, -20,  -20, -20, -20, -20,  -10,
    -20, -30, -30,  -40, -40, -30, -30,  -20,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
    -30, -40, -40,  -50, -50, -40, -40,  -30,
];

#[rustfmt::skip]
const ATTACK_VALUE_TABLE: [i16; 64] = [
    0,   0,    0,   0,   0,   0,   0,   0,
    0,   10,   0,   0,   0,   0,   10,  0,
    0,   20,   50,  40,  40,  50,  20,  0,
    0,   26,   30,  80,  80,  30,  26,  0,
    0,   26,   30,  80,  80,  30,  26,  0,
    0,   20,   50,  40,  40,  50,  20,  0,
    0,   10,   0,   0,   0,   0,   10,  0,
    0,   0,    0,   0,   0,   0,   0,   0,
];

// Not i16::MAX, because we use i16::MAX as infinity, ie.
// we want best move updated from None -> Some(mv) even if
// the best move still results in our demise.
pub const MATE: i16 = 10000;

#[inline]
fn multiply_table(bitboard: &BitBoard, table: [i16; 64], square_value: i16) -> i16 {
    (0..64)
        .map(|i| {
            let exists = bitboard.get(Square(i)) as i16;
            let offset = table[i as usize];
            let offset_value = square_value + offset;

            exists * offset_value
        })
        .sum()
}

#[inline]
fn get_piece_value(piece: Piece) -> i16 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 975,
        _ => 0,
    }
}

#[inline]
fn get_piece_score(board: &Board, color: Color, piece: Piece) -> i16 {
    let value = get_piece_value(piece);
    let table = match piece {
        Piece::Pawn => PAWN_VALUE_TABLE,
        Piece::Knight => KNIGHT_VALUE_TABLE,
        Piece::Bishop => BISHOP_VALUE_TABLE,
        Piece::Rook => ROOK_VALUE_TABLE,
        Piece::Queen => QUEEN_VALUE_TABLE,
        Piece::King => KING_VALUE_TABLE_MIDDLEGAME,
    };

    let bitboard = board.pieces(piece);
    let mut bitboard = *bitboard & *board.color_combined(color);
    if color == Color::Black {
        bitboard.flip_vertical_mut()
    };

    multiply_table(&bitboard, table, value as i16)
}

#[inline]
fn get_piece_score_for_color(board: &Board, color: Color) -> i16 {
    let mut score = 0;
    score += get_piece_score(board, color, Piece::Pawn);
    score += get_piece_score(board, color, Piece::Knight);
    score += get_piece_score(board, color, Piece::Bishop);
    score += get_piece_score(board, color, Piece::Rook);
    score += get_piece_score(board, color, Piece::Queen);
    score += get_piece_score(board, color, Piece::King);
    score
}

pub fn get_score_ongoing(board: &Board) -> i16 {
    let mut score = 0;

    score += get_piece_score_for_color(board, Color::White)
        - get_piece_score_for_color(board, Color::Black);
    score += 30 * board.side_to_move.polarize(); // Side to move gets inherent advantage

    // This should probably account for how *many* pieces are attacking squares
    let white_attacking = board.attacked(Color::White);
    let black_attacking = board.attacked(Color::Black);
    score += multiply_table(&white_attacking, ATTACK_VALUE_TABLE, 0);
    score -= multiply_table(&black_attacking, ATTACK_VALUE_TABLE, 0);

    score
}

pub fn get_score(board: &Board, game_over: bool) -> i16 {
    // NOTE: Make sure eval is never more then MATE when it is checkmate,
    // Otherwise the engine will delay mate to capture pieces.
    if game_over {
        if board.in_check() {
            MATE * board.side_to_move.other().polarize()
        } else {
            0
        }
    } else {
        get_score_ongoing(board)
    }
}

// How promising is the move, on the board? this returns
// higher for more promise, lower for less. (relative to maker of mv)
pub fn get_promise(board: &Board, mv: &Movement) -> i16 {
    // let after_move = board.make_move(mv);
    let mut p = 0; // board.side_to_move.polarize() * get_score_ongoing(&after_move);

    let moved_piece = board.piece_on(mv.from_square).unwrap();

    // most valuable victim, least valuable aggressor
    if let Some(captured) = board.piece_on(mv.to_square) {
        p += get_piece_value(captured);
        p -= get_piece_value(moved_piece) / 100;
    }

    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chess::Movement, movegen::MoveGen};

    #[test]
    fn test_get_promise() {
        let b =
            Board::from_fen("r1b2rk1/ppppnppp/2n2q2/2b1P3/3N4/2P1B3/PP3PPP/RN1QKB1R w KQ - 1 2")
                .unwrap();
        let d4c4 = Movement::from_notation("d4c6").unwrap();
        let f1e2 = Movement::from_notation("f1e2").unwrap();
        assert!(get_promise(&b, &d4c4) > get_promise(&b, &f1e2));

        let pawn_takes_queen = Movement::from_notation("e5f6").unwrap();
        assert!(get_promise(&b, &pawn_takes_queen) > get_promise(&b, &d4c4));
    }

    #[test]
    fn test_get_score_e2e4() {
        let mut b = Board::from_start_pos();
        b.make_move_mut(&Movement::from_notation("e2e4").expect("e2e4 move is valid"));

        let score = get_score(&b, MoveGen::new_legal(&b).count() == 0);
        eprintln!("score: {}", score);
        assert!(score > 0); // White should have the advantage
    }

    #[test]
    fn test_get_score_mate_for_black() {
        let b =
            Board::from_fen("r1b1kb1r/pppp1pp1/2n5/1B2p3/4PP2/6p1/PPPP2Pq/RNBQNRK1 w kq f3 0 8")
                .unwrap();
        let score = get_score(&b, MoveGen::new_legal(&b).count() == 0);

        eprintln!("board:\n{}", b);
        eprintln!("score (white in checkmate) = {}", score);
        assert_eq!(score, -MATE);
    }

    #[test]
    fn test_get_score_mate_for_white() {
        let b = Board::from_fen("k1R5/8/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        let score = get_score(&b, MoveGen::new_legal(&b).count() == 0);

        eprintln!("board:\n{}", b);
        eprintln!("score (black in checkmate) = {}", score);
        assert_eq!(score, MATE);
    }

    #[test]
    fn test_get_score_castle() {
        let mut b =
            Board::from_fen("rnbqkb1r/ppp2ppp/3p1n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4")
                .unwrap();
        let score_1 = get_score(&b, MoveGen::new_legal(&b).count() == 0);
        b.make_move_mut(&Movement::from_notation("e1g1").unwrap());
        let score_2 = get_score(&b, MoveGen::new_legal(&b).count() == 0);
        println!("{} should be > {}", score_2, score_1);
        assert!(score_2 > score_1);
    }
}
