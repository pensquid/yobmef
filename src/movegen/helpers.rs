use super::{gen_moves, get_moves};
use crate::{
    bitboard::BitBoard,
    chess::{Board, Movement, Square},
};

#[allow(dead_code)]
pub fn moves_test(board: &Board, legal: &str, illegal: &str) {
    gen_moves();
    let moves = get_moves(&board);

    for lan in legal.split(' ') {
        if !moves.contains(&Movement::from_notation(lan).unwrap()) {
            eprintln!("{}", board);
            panic!("{} should be legal", lan);
        }
    }

    if illegal != "" {
        for lan in illegal.split(' ') {
            if moves.contains(&Movement::from_notation(lan).unwrap()) {
                eprintln!("{}", board);
                panic!("{} should be illegal", lan);
            }
        }
    }
}

pub fn bitboard_to_squares(bitboard: &BitBoard) -> Vec<Square> {
    (0..64)
        .filter_map(|i| {
            if bitboard.get(Square(i)) {
                Some(Square(i))
            } else {
                None
            }
        })
        .collect::<Vec<Square>>()
}
