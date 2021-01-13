use super::{gen_moves, get_moves};
use crate::chess::{Board, Movement};

pub const NOT_A_FILE: u64 = 0xfefefefefefefefe; // ~0x0101010101010101
pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f; // ~0x8080808080808080

#[allow(dead_code)]
pub fn moves_test(board: &Board, legal: &str, illegal: &str) {
    gen_moves();
    let moves = get_moves(&board);

    let mut legal_str = moves
        .iter()
        .map(|mv| mv.to_notation())
        .collect::<Vec<String>>()
        .join(", ");
    if legal_str == "" {
        legal_str = "<none>".to_string();
    }

    eprintln!("{}", board);
    for lan in legal.split(' ') {
        if !moves.contains(&Movement::from_notation(lan).unwrap()) {
            panic!("{} should be legal, legal moves: {}", lan, legal_str);
        }
    }

    if illegal != "" {
        for lan in illegal.split(' ') {
            if moves.contains(&Movement::from_notation(lan).unwrap()) {
                panic!("{} should be illegal, legal moves: {}", lan, legal_str);
            }
        }
    }
}

// pub fn bitboard_to_squares(bitboard: &BitBoard) -> Vec<Square> {
//     (0..64)
//         .filter_map(|i| {
//             if bitboard.get(Square(i)) {
//                 Some(Square(i))
//             } else {
//                 None
//             }
//         })
//         .collect::<Vec<Square>>()
// }
