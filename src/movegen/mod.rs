use crate::chess::{Board, Movement};

mod helpers;
mod pawn;

pub fn gen_moves() {
    pawn::gen_pawn_moves();
}

pub fn get_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    pawn::get_pawn_moves(board, &mut moves);
    moves
}
