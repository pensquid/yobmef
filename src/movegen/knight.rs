use crate::chess::{Board, Movement, Square, Piece};
use crate::bitboard::BitBoard;
use super::helpers::{NOT_AB_FILE, NOT_A_FILE, NOT_GH_FILE, NOT_H_FILE};

static mut KNIGHT_MOVES: [BitBoard; 64] = [BitBoard::empty(); 64];

fn knight_moves(square: Square) -> BitBoard {
    unsafe { KNIGHT_MOVES[square.0 as usize] }
}

pub fn gen_knight_moves() {
    for from_square_index in 0..64 {
        let only_square = 1 << from_square_index;

        let mut knight_moves: u64 = 0;
        knight_moves |= (only_square << 17) & NOT_A_FILE;
        knight_moves |= (only_square << 10) & NOT_AB_FILE;
        knight_moves |= (only_square >> 6) & NOT_AB_FILE;
        knight_moves |= (only_square >> 15) & NOT_A_FILE;
        knight_moves |= (only_square << 15) & NOT_H_FILE;
        knight_moves |= (only_square << 6) & NOT_GH_FILE;
        knight_moves |= (only_square >> 10) & NOT_GH_FILE;
        knight_moves |= (only_square >> 17) & NOT_H_FILE;

        unsafe {
            KNIGHT_MOVES[from_square_index as usize] = BitBoard(knight_moves);
        }
    }
}

pub fn get_knight_moves(board: &Board, moves: &mut Vec<Movement>) {
    let my_pieces = board.color_combined(board.side_to_move);
    let my_knights = board.pieces(Piece::Knight).mask(my_pieces);
    let move_locations = my_pieces.not();

    for from_square_index in 0..64 {
        let from_square = Square(from_square_index);
        if !my_knights.get(from_square) { continue; }
        let moves_bitboard = knight_moves(from_square).mask(&move_locations);

        for to_square_index in 0..64 {
            let to_square = Square(to_square_index);
            if !moves_bitboard.get(to_square) { continue; }

            moves.push(Movement::new(from_square, to_square, None));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::Color;
    use crate::movegen::helpers::moves_test;

    #[test]
    fn test_get_knight_moves_startpos() {
        let mut board = Board::from_start_pos();

        moves_test(&board, "g1f3 g1h3 b1a3 b1c3", "g8f6 b8a6 b1d2");

        board.side_to_move = Color::Black;
        moves_test(&board, "g8f6 g8h6 b8a6 b8c6", "g1f3 b1a3 b8d7");
    }

    #[test]
    fn test_get_knight_moves_other_directions() {
        let mut board = Board::from_fen("b6k/7K/8/2N5/4n3/8/8/4B3 w - - 0 1").expect("fen is valid");

        moves_test(&board, "c5b7 c5a6 c5a4 c5b3 c5d3 c5e4 c5e6 c5d7", "");

        board.side_to_move = Color::Black;
        moves_test(&board, "e4d6 e4c5 e4c3 e4d2 e4f2 e4g3 e4g5 e4f6", "");
    }

    #[test]
    fn test_get_knight_moves_edges() {
        let board = Board::from_fen("8/8/2k3K1/8/8/8/N6N/8 w - - 0 1").expect("fen is valid");
        moves_test(&board, "h2g4 a2c3", "h2g8 h2a4 a2g3");
    }
}
