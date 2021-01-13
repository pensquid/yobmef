use super::{gen_moves, get_moves};
use crate::chess::{Board, Movement};

pub const NOT_A_FILE: u64 = 0xfefefefefefefefe;
pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
pub const NOT_AB_FILE: u64 = 0xfcfcfcfcfcfcfcfc;
pub const NOT_GH_FILE: u64 = 0x3f3f3f3f3f3f3f3f;

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

    for lan in legal.split(' ') {
        if !moves.contains(&Movement::from_notation(lan).unwrap()) {
            eprintln!("{}", board);
            panic!("{} should be legal, legal moves: {}", lan, legal_str);
        }
    }

    if illegal != "" {
        for lan in illegal.split(' ') {
            if moves.contains(&Movement::from_notation(lan).unwrap()) {
                eprintln!("{}", board);
                panic!("{} should be illegal, legal moves: {}", lan, legal_str);
            }
        }
    }
}
