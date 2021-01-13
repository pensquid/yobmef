use crate::chess::{Board, Color, Movement, Piece, Square};
use crate::{bitboard::BitBoard, chess::NUM_PIECES};

use super::helpers::{NOT_A_FILE, NOT_H_FILE};

// 48 because we don't need the top or bottom rows for pawns
static mut PAWN_ATTACKS: [[BitBoard; 48]; NUM_PIECES] = [[BitBoard::empty(); 48]; NUM_PIECES];
static mut PAWN_PUSHES: [[BitBoard; 48]; NUM_PIECES] = [[BitBoard::empty(); 48]; NUM_PIECES];
static mut PAWN_DBL_PUSHES: [[BitBoard; 48]; NUM_PIECES] = [[BitBoard::empty(); 48]; NUM_PIECES];

fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_ATTACKS[color as usize][(square.0 - 8) as usize] }
}
fn pawn_pushes(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_PUSHES[color as usize][(square.0 - 8) as usize] }
}
fn pawn_dbl_pushes(square: Square, color: Color) -> BitBoard {
    unsafe { PAWN_DBL_PUSHES[color as usize][(square.0 - 8) as usize] }
}

pub fn gen_pawn_moves() {
    for from_square_index in 0..48 {
        let square = Square(from_square_index + 8);
        let only_square = 1 << (from_square_index + 8);

        // Even a fucking gradeschooler would then know
        let white_pawn_attacks =
            BitBoard(((only_square << 9) & NOT_A_FILE) | ((only_square << 7) & NOT_H_FILE));
        let black_pawn_attacks =
            BitBoard(((only_square >> 9) & NOT_H_FILE) | ((only_square >> 7) & NOT_A_FILE));

        let white_pawn_pushes = BitBoard(only_square << 8);
        let black_pawn_pushes = BitBoard(only_square >> 8);
        if square.rank() == 1 {
            let white_dbl_pawn_pushes = BitBoard(only_square << 16);
            unsafe {
                PAWN_DBL_PUSHES[Color::White as usize][from_square_index as usize] =
                    white_dbl_pawn_pushes;
            }
        }

        if square.rank() == 6 {
            let black_dbl_pawn_pushes = BitBoard(only_square >> 16);
            unsafe {
                PAWN_DBL_PUSHES[Color::Black as usize][from_square_index as usize] =
                    black_dbl_pawn_pushes;
            }
        }

        unsafe {
            PAWN_ATTACKS[Color::White as usize][from_square_index as usize] = white_pawn_attacks;
            PAWN_ATTACKS[Color::Black as usize][from_square_index as usize] = black_pawn_attacks;
            PAWN_PUSHES[Color::White as usize][from_square_index as usize] = white_pawn_pushes;
            PAWN_PUSHES[Color::Black as usize][from_square_index as usize] = black_pawn_pushes;
        }
    }
}

pub fn get_pawn_moves(board: &Board, moves: &mut Vec<Movement>) {
    // we're so fucking dumb kognise
    // we need bitwise not because we want the mask to cancel when
    // a piece *IS* there, not when it isen't
    let all_pieces = board.color_combined_both().not();
    let my_pawns = board
        .pieces(Piece::Pawn)
        .mask(&board.color_combined(board.side_to_move));

    let mut their_pieces = board.color_combined(board.side_to_move.other()).clone();
    if let Some(sq) = board.en_passant {
        their_pieces.flip_mut(sq);
    }

    for from_square_index in 0..48 {
        let from_square = Square(from_square_index + 8);
        if !my_pawns.get(from_square) {
            continue;
        }

        let mut moves_bitboard = BitBoard::empty();

        // Attacks
        moves_bitboard.merge_mut(&pawn_attacks(from_square, board.side_to_move));
        moves_bitboard.mask_mut(&their_pieces);

        // Single pushes
        let mut pushes = pawn_pushes(from_square, board.side_to_move).clone();
        pushes.mask_mut(&all_pieces);
        moves_bitboard.merge_mut(&pushes);

        // Double pushes
        let mut dbl_pushes = pawn_dbl_pushes(from_square, board.side_to_move).clone();
        dbl_pushes.mask_mut(&all_pieces);
        dbl_pushes.mask_mut(
            &(if board.side_to_move == Color::White {
                BitBoard(all_pieces.0 << 8)
            } else {
                BitBoard(all_pieces.0 >> 8)
            }),
        );
        moves_bitboard.merge_mut(&dbl_pushes);

        // Add all the moves
        for to_square_index in 0..48 {
            let to_square = Square(to_square_index + 8);
            if !moves_bitboard.get(to_square) {
                continue;
            }
            moves.push(Movement::new(from_square, to_square, None));
        }
    }

    // TODO: Add promotion and optimize
    // bitboard_to_squares(&my_pawns)
    //     .iter()
    //     .for_each(|from_square| {
    //         let valid_attacks = pawn_attacks(*from_square, board.side_to_move).mask(&their_pieces);

    //         bitboard_to_squares(&valid_attacks)
    //             .iter()
    //             .for_each(|to_square| {
    //                 moves.push(Movement::new(*from_square, *to_square, None));
    //             });

    //         let up_square = match board.side_to_move {
    //             Color::White => from_square.up(1),
    //             Color::Black => from_square.down(1),
    //         }.unwrap();
    //         if !all_pieces.get(up_square) {
    //             moves.push(Movement::new(*from_square, up_square, None));

    //             let up_two_square = match board.side_to_move {
    //                 Color::White => from_square.up(2),
    //                 Color::Black => from_square.down(2),
    //             }.unwrap();
    //             let needed_two_rank = match board.side_to_move {
    //                 Color::White => 1,
    //                 Color::Black => 6,
    //             };
    //             if from_square.rank() == needed_two_rank && !all_pieces.get(up_two_square) {
    //                 moves.push(Movement::new(*from_square, up_two_square, None));
    //             }
    //         }
    //     });
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
}
