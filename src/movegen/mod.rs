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

pub fn get_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    pawn::get_pawn_moves(board, &mut moves);
    knight::get_knight_moves(board, &mut moves);
    king::get_king_moves(board, &mut moves);
    magic::get_sliding_moves(board, &mut moves);
    moves
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn vec_moves(moves_str: &str) -> Vec<Movement> {
        let mut moves = Vec::new();
        for lan in moves_str.split(' ') {
            moves.push(Movement::from_notation(lan).unwrap())
        }
        moves
    }

    fn hash(mv: &Movement) -> u64 {
        let mut hasher = DefaultHasher::new();
        mv.hash(&mut hasher);
        hasher.finish()
    }

    fn assert_moves(board: &Board, moves: &str) {
        let mut want_moves = vec_moves(moves);
        let mut got_moves = get_moves(board);
        want_moves.sort_by_key(|m| hash(m));
        got_moves.sort_by_key(|m| hash(m));

        if want_moves != got_moves {
            eprintln!("{:?} to move\n{}", board.side_to_move, board);
            eprintln!("got legal: {}", moves_to_str(&got_moves));
            eprintln!("want legal: {}", moves_to_str(&want_moves));
            panic!("move vectors don't match");
        }
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
