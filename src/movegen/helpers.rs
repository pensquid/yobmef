use super::{gen_moves, get_moves};
use crate::{
    bitboard::BitBoard,
    chess::{Board, Movement, Square},
};

pub const NOT_A_FILE: u64 = 0xfefefefefefefefe; // ~0x0101010101010101
pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f; // ~0x8080808080808080

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
