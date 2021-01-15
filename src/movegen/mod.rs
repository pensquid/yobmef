use crate::bitboard::BitBoard;
use crate::chess::{Board, Movement};

mod helpers;
mod king;
mod knight;
mod magic;
mod magic_utils;
mod pawn;


pub fn gen_moves() {
    pawn::gen_pawn_moves();
    knight::gen_knight_moves();
    king::gen_king_moves();
    magic::gen_all_magics();
}

pub fn get_pseudolegal_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    pawn::get_pawn_moves(board, &mut moves, board.side_to_move);
    knight::get_knight_moves(board, &mut moves, board.side_to_move);
    king::get_king_moves(board, &mut moves, board.side_to_move);
    magic::get_sliding_moves(board, &mut moves, board.side_to_move);
    moves
}

pub fn get_legal_moves(board: &Board) -> Vec<Movement> {
    let pseudolegal = get_pseudolegal_moves(board);

    pseudolegal.into_iter().filter(|mv| {
        let after_move = board.make_move(mv);

        let mut attacks = BitBoard::empty();
        attacks |= pawn::get_pawn_attacks(&after_move, after_move.side_to_move);
        attacks |= knight::get_knight_attacks(&after_move, after_move.side_to_move);
        attacks |= king::get_king_attacks(&after_move, after_move.side_to_move);
        attacks |= magic::get_sliding_attacks(&after_move, after_move.side_to_move);

        let only_our_king = 1 << after_move.king(board.side_to_move).0;
        let is_in_check = (attacks.0 & only_our_king).count_ones() > 0;
        !is_in_check
    }).collect()
}

#[cfg(test)]
mod tests {
    use crate::chess::Board;
    use super::{helpers::assert_moves};

    #[test]
    fn test_move_into_check() {
        let board = Board::from_fen("K7/2k5/8/8/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(&board, "a8a7");

        let board = Board::from_fen("7k/8/5P2/6K1/8/8/8/8 b - - 0 1").unwrap();
        assert_moves(&board, "h8h7 h8g8");
    }

    #[test]
    fn test_move_in_check() {
        let board = Board::from_fen("7K/6b1/6k1/8/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(&board, "h8g8");
    }

    #[test]
    fn test_block_check_knight() {
        let board = Board::from_fen("3K4/8/6k1/3N2b1/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(&board, "d5e7 d5f6 d8d7 d8e8 d8c8 d8c7");
    }

    #[test]
    fn test_block_check_double_pawn_push() {
        let board = Board::from_fen("8/2r5/6k1/6r1/3K2r1/6r1/4P3/8 w - - 0 1").unwrap();
        assert_moves(&board, "e2e4");
    }

    #[test]
    fn test_endgame() {
        let board = Board::from_fen("8/8/2p2k2/5n2/1N1P4/1K6/8/8 w - - 0 1").unwrap();
        assert_moves(
            &board,
            "b3b2 b3a2 b3c2 b3c3 b3a3 b3a4 b3c4 b4c2 b4a2 b4a6 b4c6 b4d3 b4d5 d4d5",
        );
    }
}
