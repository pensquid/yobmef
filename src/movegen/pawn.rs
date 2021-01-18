use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Movement, Piece, Square};

use super::helpers::{NOT_A_FILE, NOT_H_FILE};

// 48 because we don't need the top or bottom rows for pawns
static mut PAWN_ATTACKS: [[BitBoard; 48]; 64] = [[BitBoard::empty(); 48]; 64];
static mut PAWN_PUSHES: [[BitBoard; 48]; 64] = [[BitBoard::empty(); 48]; 64];
static mut PAWN_DBL_PUSHES: [[BitBoard; 48]; 64] = [[BitBoard::empty(); 48]; 64];

pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_ATTACKS[color as usize][(square.0 - 8) as usize] }
}
fn pawn_pushes(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_PUSHES[color as usize][(square.0 - 8) as usize] }
}
fn pawn_dbl_pushes(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_DBL_PUSHES[color as usize][(square.0 - 8) as usize] }
}

pub fn get_pawn_attacks(board: &Board, color: Color) -> BitBoard {
    let mut attacked = BitBoard::empty();

    let pawns = *board.pieces(Piece::Pawn) & *board.color_combined(color);
    // NOTE: tempting the gods with this optimization
    for sq_index in 0..48 {
        let sq = Square(sq_index + 8);
        if pawns.get(sq) {
            attacked |= pawn_attacks(sq, color);
        }
    }

    attacked
}

pub fn gen_pawn_moves() {
    for from_sq_index in 0..48 {
        let from_sq = Square(from_sq_index + 8);
        let only_from_sq = 1 << (from_sq_index + 8);

        // Even a fucking gradeschooler would then know
        let white_pawn_attacks =
            BitBoard(((only_from_sq << 9) & NOT_A_FILE) | ((only_from_sq << 7) & NOT_H_FILE));
        let black_pawn_attacks =
            BitBoard(((only_from_sq >> 9) & NOT_H_FILE) | ((only_from_sq >> 7) & NOT_A_FILE));

        let white_pawn_pushes = BitBoard(only_from_sq << 8);
        let black_pawn_pushes = BitBoard(only_from_sq >> 8);
        if from_sq.rank() == 1 {
            let white_dbl_pawn_pushes = BitBoard(only_from_sq << 16);
            unsafe {
                PAWN_DBL_PUSHES[Color::White as usize][from_sq_index as usize] =
                    white_dbl_pawn_pushes;
            }
        }

        if from_sq.rank() == 6 {
            let black_dbl_pawn_pushes = BitBoard(only_from_sq >> 16);
            unsafe {
                PAWN_DBL_PUSHES[Color::Black as usize][from_sq_index as usize] =
                    black_dbl_pawn_pushes;
            }
        }

        unsafe {
            PAWN_ATTACKS[Color::White as usize][from_sq_index as usize] = white_pawn_attacks;
            PAWN_ATTACKS[Color::Black as usize][from_sq_index as usize] = black_pawn_attacks;
            PAWN_PUSHES[Color::White as usize][from_sq_index as usize] = white_pawn_pushes;
            PAWN_PUSHES[Color::Black as usize][from_sq_index as usize] = black_pawn_pushes;
        }
    }
}

pub fn get_pawn_moves(board: &Board, moves: &mut Vec<Movement>, color: Color) {
    // We need bitwise not because we want the mask to cancel when
    // a piece *IS* there, not when it isn't
    let pushes_mask = !board.combined();
    let my_pawns = *board.pieces(Piece::Pawn) & *board.color_combined(color);

    let mut their_pieces = board.color_combined(color.other()).clone();
    if let Some(sq) = board.en_passant {
        their_pieces.flip_mut(sq);
    }

    let promotion_rank = match color {
        Color::White => 7,
        Color::Black => 0,
    };

    for from_sq in my_pawns {
        let mut moves_bitboard = BitBoard::empty();

        // Attacks
        moves_bitboard |= pawn_attacks(from_sq, color) & their_pieces;

        // Single pushes
        let mut pushes = pawn_pushes(from_sq, color);
        pushes &= pushes_mask;
        moves_bitboard |= pushes;

        // Double pushes
        let mut dbl_pushes = pawn_dbl_pushes(from_sq, color);
        dbl_pushes &= pushes_mask;
        dbl_pushes &= if color == Color::White {
            pushes_mask << 8 // up
        } else {
            pushes_mask >> 8 // down
        };
        moves_bitboard |= dbl_pushes;

        // Add all the moves
        for to_sq in moves_bitboard {
            if to_sq.rank() == promotion_rank {
                moves.push(Movement::new(from_sq, to_sq, Some(Piece::Bishop)));
                moves.push(Movement::new(from_sq, to_sq, Some(Piece::Rook)));
                moves.push(Movement::new(from_sq, to_sq, Some(Piece::Knight)));
                moves.push(Movement::new(from_sq, to_sq, Some(Piece::Queen)));
            } else {
                moves.push(Movement::new(from_sq, to_sq, None));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::helpers::moves_test;

    #[test]
    fn test_take_pawn_h_file() {
        let mut board = Board::from_fen("8/8/7p/6P1/8/k7/8/K7 w - - 0 1").unwrap();
        moves_test(&board, "g5h6", "h6g5 g5f6");
        board.side_to_move = Color::Black;
        moves_test(&board, "h6g5", "g6h5");
    }

    #[test]
    fn test_take_own_pawn() {
        let board = Board::from_fen("8/8/5P1p/6P1/8/k7/8/K7 w - - 0 1").unwrap();
        moves_test(&board, "g5h6", "g5f6");
    }

    #[test]
    fn test_take_pawn_a_file() {
        let mut board = Board::from_fen("8/8/p7/1P6/8/k7/8/K7 w - - 0 1").unwrap();
        moves_test(&board, "b5a6", "a6b5");
        board.side_to_move = Color::Black;
        moves_test(&board, "a6b5", "b5a6");
    }

    #[test]
    fn test_get_pawn_moves_startpos() {
        let mut board = Board::from_start_pos();

        moves_test(&board, "e2e4 d2d3", "e2d3 e2e5 e7e5 d7d6");

        board.side_to_move = Color::Black;
        moves_test(&board, "e7e5 d7d6", "e2e5 e2e4 d2d3");
    }

    #[test]
    fn test_get_pawn_moves_endgame() {
        let board = Board::from_fen("8/3k1p2/1R1p2P1/8/2P1N3/2Q1K3/8/8 w - - 0 1").unwrap();

        moves_test(&board, "c4c5 g6g7 g6f7", "f7f6 g6h7");
    }

    #[test]
    fn test_white_pawn_promote() {
        let board = Board::from_fen("4r1b1/5P2/8/8/8/k7/8/K7 w - - 0 1").unwrap();
        moves_test(
            &board,
            "f7e8q f7e8r f7f8q f7f8r f7g8q f7g8r",
            "f7g8k f7f8k f7f8p",
        );
    }

    #[test]
    fn test_black_pawn_promote() {
        let board = Board::from_fen("8/8/8/8/8/k7/6p1/K4N1R b - - 0 1").unwrap();
        moves_test(&board, "g2f1q g2f1r g2g1q g2h1r", "g2f1p g2h1k");
    }

    #[test]
    fn test_get_pawn_en_passant() {
        let board =
            Board::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
                .unwrap();

        moves_test(&board, "e5f6 e5e6", "e5d6");
    }

    #[test]
    fn test_en_passant_capture() {
        let mut board = Board::from_fen("k1K5/8/8/8/3PPp2/8/8/8 b - e3 0 1").unwrap();
        moves_test(&board, "f4e3 f4f3", "d4d5");

        board.make_move_mut(&Movement::from_notation("f4e3").unwrap());
        board.assert_valid();
        moves_test(&board, "d4d5", "e4e5");
    }
}
