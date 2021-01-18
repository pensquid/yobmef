use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Piece, Square};
use crate::movegen::get_attacked_squares;

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
     20,  200, 180,  0,   0,   10,  180,  20,
     20,  20,  0,    0,   0,   0,   20,   20,
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
fn get_piece_score(board: &Board, color: Color, piece: Piece) -> i16 {
    let value = match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 975,
        _ => 0,
    };
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

pub fn get_score(board: &Board) -> i16 {
    // Check for captured king first (we are a king capture engine)
    // TODO: Clean this up, maybe add methods on board.
    let kings = *board.pieces(Piece::King);
    if kings & board.color_combined(Color::Black) == BitBoard::empty() {
        return MATE;
    } else if kings & board.color_combined(Color::White) == BitBoard::empty() {
        return -MATE;
    }

    let mut score = 0;

    score += get_piece_score_for_color(board, Color::White)
        - get_piece_score_for_color(board, Color::Black);
    score += 30 * board.side_to_move.polarize(); // Side to move gets inherent advantage

    // This should probably account for how *many* pieces are attacking squares
    let white_attacking = get_attacked_squares(board, Color::White);
    let black_attacking = get_attacked_squares(board, Color::Black);
    score += multiply_table(&white_attacking, ATTACK_VALUE_TABLE, 0);
    score -= multiply_table(&black_attacking, ATTACK_VALUE_TABLE, 0);

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::Movement;
    use crate::movegen::gen_moves_once;

    #[test]
    fn test_get_score_e2e4() {
        gen_moves_once();

        let mut b = Board::from_start_pos();
        b.make_move_mut(&Movement::from_notation("e2e4").expect("e2e4 move is valid"));

        let score = get_score(&b);
        eprintln!("score: {}", score);
        assert!(score > 0); // White should have the advantage
    }

    #[test]
    fn test_get_score_castle() {
        gen_moves_once();

        let mut b =
            Board::from_fen("rnbqkb1r/ppp2ppp/3p1n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4")
                .unwrap();
        let score_1 = get_score(&b);
        b.make_move_mut(&Movement::from_notation("e1g1").unwrap());
        let score_2 = get_score(&b);
        println!("{} should be > {}", score_2, score_1);
        assert!(score_2 > score_1);
    }
}
