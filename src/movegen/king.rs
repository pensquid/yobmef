use super::helpers::*;
use crate::bitboard::*;
use crate::chess::*;

static mut KING_MOVES: [BitBoard; 64] = [BitBoard::empty(); 64];

fn king_moves(square: Square) -> BitBoard {
    unsafe { KING_MOVES[square.0 as usize] }
}

pub fn gen_king_moves() {
    for from_square_index in 0..64 {
        let mut king_moves: u64 = 0;
        let only_square = 1 << from_square_index;

        king_moves |= (only_square << 8) & !RANK_1; // up
        king_moves |= (only_square << 9) & !(RANK_1 | A_FILE); // up-right
        king_moves |= (only_square << 7) & !(RANK_1 | H_FILE); // up-left

        king_moves |= (only_square >> 8) & !RANK_8; // down
        king_moves |= (only_square >> 7) & !(RANK_8 | A_FILE); // down-right
        king_moves |= (only_square >> 9) & !(RANK_8 | H_FILE); // down-left

        king_moves |= (only_square >> 1) & !H_FILE; // left
        king_moves |= (only_square << 1) & !A_FILE; // right

        unsafe {
            KING_MOVES[from_square_index as usize] = BitBoard(king_moves);
        }
    }
}

pub fn get_king_moves(board: &Board, moves: &mut Vec<Movement>) {
    let our_pieces = board.color_combined(board.side_to_move);
    let his_pieces = board.color_combined(board.side_to_move.other());
    let king = board.pieces(Piece::King).mask(our_pieces);
    let pieces_mask = our_pieces.merge(his_pieces).not();

    // TODO: extract this loop into a helper (common pattern)
    for from_square_index in 0..64 {
        let from_square = Square(from_square_index);
        let only_square = 1 << from_square_index;
        if (king.0 & only_square) == 0 {
            continue;
        }

        // TODO Handle check (mask with attacked squares)
        // eprintln!("king moves at {}\n{}", from_square, king_moves(from_square));
        let moves_bitboard = king_moves(from_square).mask(&pieces_mask);

        for to_square_index in 0..64 {
            let to_square = Square(to_square_index);
            if !moves_bitboard.get(to_square) {
                continue;
            }

            let movement = Movement::new(from_square, to_square, None);
            moves.push(movement);
        }

        // if we have more then one king; we've got bigger problems
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::helpers::moves_test;

    #[test]
    fn test_gen_king_moves() {
        let board = Board::from_fen("8/8/4k3/7n/7K/6N1/8/8 w - - 0 1").unwrap();
        moves_test(&board, "h4g5 h4g4 h4h3", "h4h5 h4g3 h4a5");
    }

    #[test]
    fn test_gen_king_moves_topright() {
        let board = Board::from_fen("7K/8/4k3/4n3/8/6N1/8/8 w - - 0 1").unwrap();
        assert_eq!(board.side_to_move, Color::White);
        moves_test(&board, "h8g7 h8h7 h8g8", "h8a1 h8a8 h8h1");
    }

    #[test]
    fn test_gen_king_moves_black() {
        let board = Board::from_fen("7K/8/4k3/4n3/8/6N1/8/8 b - - 0 1").unwrap();
        assert_eq!(board.side_to_move, Color::Black);
        moves_test(&board, "e6f6 e6f7 e6e7 e6d6 e6d7 e6d5", "e6e5");
    }

    #[test]
    fn test_gen_king_moves_topleft() {
        let board = Board::from_fen("K7/8/4k3/4n3/8/6N1/8/8 w - - 0 1").unwrap();
        moves_test(&board, "a8b8 a8a7 a8b7", "a8h1 a8h8 a8a1");
    }

    #[test]
    fn test_gen_king_moves_bottomright() {
        let board = Board::from_fen("8/8/4k3/4n3/8/6N1/8/7K w - - 0 1").unwrap();
        moves_test(&board, "h1h2 h1g1 h1g2", "h1a1 h1h8 h1a8");
    }
}
