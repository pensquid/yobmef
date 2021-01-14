#![allow(dead_code)]
use super::{gen_moves, get_moves};
use crate::{
    bitboard::BitBoard,
    chess::{Board, Movement, Square},
};
use std::sync::Once;

// <3 kognise
// We could inline shifts of the different files but this is more readable
pub const A_FILE: u64 = 0x101010101010101;
pub const B_FILE: u64 = A_FILE << 1;
pub const C_FILE: u64 = A_FILE << 2;
pub const D_FILE: u64 = A_FILE << 3;
pub const E_FILE: u64 = A_FILE << 4;
pub const F_FILE: u64 = A_FILE << 5;
pub const G_FILE: u64 = A_FILE << 6;
pub const H_FILE: u64 = A_FILE << 7;

pub const RANK_1: u64 = 0x0000000000000ff;
pub const RANK_2: u64 = RANK_1 << 8 * 1;
pub const RANK_3: u64 = RANK_1 << 8 * 2;
pub const RANK_4: u64 = RANK_1 << 8 * 3;
pub const RANK_5: u64 = RANK_1 << 8 * 4;
pub const RANK_6: u64 = RANK_1 << 8 * 5;
pub const RANK_7: u64 = RANK_1 << 8 * 6;
pub const RANK_8: u64 = RANK_1 << 8 * 7;

pub const NOT_A_FILE: u64 = !A_FILE;
pub const NOT_H_FILE: u64 = !H_FILE;
pub const NOT_AB_FILE: u64 = !(A_FILE | B_FILE);
pub const NOT_GH_FILE: u64 = !(G_FILE | H_FILE);

pub const NOT_EDGES: u64 = !(A_FILE | H_FILE | RANK_1 | RANK_8);

pub fn moves_to_str(moves: &Vec<Movement>) -> String {
    let s = moves
        .iter()
        .map(|mv| mv.to_notation())
        .collect::<Vec<String>>()
        .join(", ");
    if s == "" {
        "<none>".to_string()
    } else {
        s
    }
}

static START: Once = Once::new();

pub fn moves_test(board: &Board, legal: &str, illegal: &str) {
    START.call_once(|| {
        gen_moves();
    });
    let moves = get_moves(&board);

    let legal_str = moves_to_str(&moves);

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

pub fn bitboard_test(board: &BitBoard, included: &str, excluded: &str) {
    let mut squares = Vec::new();
    for sq_index in 0..64 {
        let sq = Square(sq_index);
        if board.get(sq) {
            squares.push(sq);
        }
    }

    for coord in included.split(' ') {
        if !squares.contains(&Square::from_notation(coord).unwrap()) {
            eprintln!("{}", board);
            panic!("{} should be included in bitboard", coord);
        }
    }

    if excluded != "" {
        for coord in excluded.split(' ') {
            if squares.contains(&Square::from_notation(coord).unwrap()) {
                eprintln!("{}", board);
                panic!("{} should be excluded in bitboard", coord);
            }
        }
    }
}
