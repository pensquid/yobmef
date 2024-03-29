use crate::chess::{Board, Movement};
use crate::{bitboard::BitBoard, chess::Color};
use std::sync::Once;

mod helpers;
mod king;
mod knight;
mod magic;
mod magic_utils;
mod pawn;

static START: Once = Once::new();

pub fn gen_moves_once() {
    START.call_once(|| {
        gen_moves();
    });
}

fn gen_moves() {
    pawn::gen_pawn_moves();
    knight::gen_knight_moves();
    king::gen_king_moves();
    magic::gen_all_magics();
}

pub struct MoveGen {
    pseudolegal: Vec<Movement>,
    index: usize,
    board: Board,
    iterator_mask: BitBoard,
}

impl MoveGen {
    pub fn new_legal(board: &Board) -> MoveGen {
        let pseudolegal = get_pseudolegal_moves(board);
        MoveGen {
            pseudolegal,
            index: 0,
            iterator_mask: !BitBoard::empty(),

            // TODO: Owning the board should not be needed if you just
            // generate moves that don't leave you in check.
            board: board.clone(),
        }
    }

    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;
    }
}

impl Iterator for MoveGen {
    type Item = Movement;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.pseudolegal.len() {
            let mv = &self.pseudolegal[self.index];
            self.index += 1;

            // Is this move covered by the iterator mask?
            if BitBoard::from_square(mv.to_square) & self.iterator_mask == BitBoard::empty() {
                continue;
            }

            // After the move, are we in check?

            let after_move = self.board.make_move(mv);
            let attacks = after_move.attacked(after_move.side_to_move);

            let only_our_king = 1 << after_move.king(self.board.side_to_move).0;
            let is_in_check = (attacks.0 & only_our_king).count_ones() > 0;
            if !is_in_check {
                return Some(mv.clone());
            }
        }

        None
    }
}

pub fn get_pseudolegal_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    pawn::get_pawn_moves(board, &mut moves, board.side_to_move);
    knight::get_knight_moves(board, &mut moves, board.side_to_move);
    king::get_king_moves(board, &mut moves, board.side_to_move);
    magic::get_sliding_moves(board, &mut moves, board.side_to_move);
    moves
}

pub fn get_attacked_squares(board: &Board, color: Color) -> BitBoard {
    let mut attacks = BitBoard::empty();
    attacks |= pawn::get_pawn_attacks(board, color);
    attacks |= knight::get_knight_attacks(board, color);
    attacks |= king::get_king_attacks(board, color);
    attacks |= magic::get_sliding_attacks(board, color);
    attacks
}

// For debugging, used in tests and for a debug command 'go perft depth'
pub fn perft(board: &Board, depth: u16) -> u64 {
    if depth == 1 {
        MoveGen::new_legal(board).count() as u64
    } else {
        let mut n = 0;
        for mv in MoveGen::new_legal(board) {
            n += perft(&board.make_move(&mv), depth - 1);
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::assert_moves;
    use super::*;
    use crate::chess::Board;

    #[test]
    fn test_move_into_check() {
        let board = Board::from_fen("K7/2k5/8/8/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(&board, MoveGen::new_legal(&board).collect(), "a8a7");

        let board = Board::from_fen("7k/8/5P2/6K1/8/8/8/8 b - - 0 1").unwrap();
        assert_moves(&board, MoveGen::new_legal(&board).collect(), "h8h7 h8g8");
    }

    #[test]
    fn test_move_in_check() {
        let board = Board::from_fen("7K/6b1/6k1/8/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(&board, MoveGen::new_legal(&board).collect(), "h8g8");
    }

    #[test]
    fn test_block_check_knight() {
        let board = Board::from_fen("3K4/8/6k1/3N2b1/8/8/8/8 w - - 0 1").unwrap();
        assert_moves(
            &board,
            MoveGen::new_legal(&board).collect(),
            "d5e7 d5f6 d8d7 d8e8 d8c8 d8c7",
        );
    }

    #[test]
    fn test_block_check_double_pawn_push() {
        let board = Board::from_fen("8/2r5/6k1/6r1/3K2r1/6r1/4P3/8 w - - 0 1").unwrap();
        assert_moves(&board, MoveGen::new_legal(&board).collect(), "e2e4");
    }

    #[test]
    fn test_endgame() {
        let board = Board::from_fen("8/8/2p2k2/5n2/1N1P4/1K6/8/8 w - - 0 1").unwrap();
        assert_moves(
            &board,
            MoveGen::new_legal(&board).collect(),
            "b3b2 b3a2 b3c2 b3c3 b3a3 b3a4 b3c4 b4c2 b4a2 b4a6 b4c6 b4d3 b4d5 d4d5",
        );
    }

    #[test]
    fn test_iter_attacked() {
        let board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
                .unwrap();

        let mut legal_iter = MoveGen::new_legal(&board);
        let attacked = board.attacked(board.side_to_move);
        let enemy_pieces = board.color_combined(board.side_to_move.other());
        legal_iter.set_iterator_mask(attacked & enemy_pieces);
        assert_moves(&board, legal_iter.collect(), "c4f7 h5e5 h5f7 h5h7");
    }
}
