use super::helpers::*;
use crate::bitboard::*;
use crate::chess::*;

static mut KING_MOVES: [BitBoard; 64] = [BitBoard::empty(); 64];

fn king_moves(square: Square) -> BitBoard {
    unsafe { KING_MOVES[square.0 as usize] }
}

pub fn gen_king_moves() {
    for from_sq_index in 0..64 {
        let mut king_moves: u64 = 0;
        let only_from_sq = 1 << from_sq_index;

        king_moves |= (only_from_sq << 8) & !RANK_1; // Up
        king_moves |= (only_from_sq << 9) & !(RANK_1 | A_FILE); // Up-right
        king_moves |= (only_from_sq << 7) & !(RANK_1 | H_FILE); // Up-left

        king_moves |= (only_from_sq >> 8) & !RANK_8; // Down
        king_moves |= (only_from_sq >> 7) & !(RANK_8 | A_FILE); // Down-right
        king_moves |= (only_from_sq >> 9) & !(RANK_8 | H_FILE); // Down-left

        king_moves |= (only_from_sq >> 1) & !H_FILE; // Left
        king_moves |= (only_from_sq << 1) & !A_FILE; // Right

        unsafe {
            KING_MOVES[from_sq_index as usize] = BitBoard(king_moves);
        }
    }
}

pub fn get_king_attacks(board: &Board, color: Color) -> BitBoard {
    let mut attacks = BitBoard::empty();

    let our_pieces = *board.color_combined(color);
    let king = *board.pieces(Piece::King) & our_pieces;

    for from_sq_index in 0..64 {
        let from_sq = Square(from_sq_index);
        let only_from_sq = 1 << from_sq_index;
        if (king.0 & only_from_sq) == 0 {
            continue;
        }

        attacks |= king_moves(from_sq);

        // If we have more then one king; we've got bigger problems
        break;
    }

    attacks
}

pub fn get_king_moves(board: &Board, moves: &mut Vec<Movement>, color: Color) {
    let our_pieces = *board.color_combined(color);
    let king = *board.pieces(Piece::King) & our_pieces;

    let king_sq_index = king.0.trailing_zeros();
    let king_sq = Square(king_sq_index as u8);

    let moves_bitboard = king_moves(king_sq) & !our_pieces;

    for to_sq_index in 0..64 {
        let to_sq = Square(to_sq_index);
        if !moves_bitboard.get(to_sq) {
            continue;
        }

        let movement = Movement::new(king_sq, to_sq, None);
        moves.push(movement);
    }

    if !board.in_check() {
        let attacks = super::get_attacked_squares(board, color.other());
        let our_rooks = *board.pieces(Piece::Rook) & our_pieces;

        CastlingSide::of_color(color).iter().for_each(|side| {
            if !board.can_castle_unchecked(*side) {
                return;
            }
            
            let middle = side.get_castling_middle();
            let attacked = (attacks & middle).count_ones() > 0;
            let blocked = (our_pieces & middle).count_ones() > 0;

            // This is really bad and horrible and needs optimization
            let king_movement = side.get_king_movement();
            let king_placed = king_movement.from_square == king_sq;

            let rook_movement = side.get_rook_movement();
            let rook_placed = our_rooks.get(rook_movement.from_square);

            if !blocked && !attacked && king_placed && rook_placed {
                moves.push(side.get_king_movement());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::helpers::moves_test;

    #[test]
    fn test_gen_king_moves() {
        let board = Board::from_fen("8/8/4k3/7n/7K/6N1/8/8 w - - 0 1").unwrap();
        moves_test(&board, "h4g5 h4g4 h4h3", "h4g3 h4a5");
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

    #[test]
    fn test_king_takes_queen() {
        let board =
            Board::from_fen("r1b1k2r/pppp1ppp/4p3/8/2nP4/2B2qP1/P1P1KP1P/RQ6 w kq - 1 2").unwrap();
        moves_test(&board, "e2f3", "");
    }

    #[test]
    fn test_king_castling_unblocked() {
        let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        moves_test(&board, "e1c1 e1g1", "");
        board.side_to_move = Color::Black;
        moves_test(&board, "e8c8 e8g8", "");
    }

    #[test]
    fn test_king_castling_blocked() {
        let mut board = Board::from_fen("r2pk2r/8/8/8/8/8/8/R3K1PR w KQkq - 0 1").unwrap();
        moves_test(&board, "e1c1", "e1g1");
        board.side_to_move = Color::Black;
        moves_test(&board, "e8g8", "e8c8");
    }

    #[test]
    fn test_king_castling_through_check() {
        let mut board = Board::from_fen("r3k2r/8/8/5R2/5b2/8/8/R3K2R w KQkq - 0 1").unwrap();
        moves_test(&board, "e1g1", "e1c1");
        board.side_to_move = Color::Black;
        moves_test(&board, "e8c8", "e8g8");
    }
}
