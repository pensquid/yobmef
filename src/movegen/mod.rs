use crate::chess::{Board, Movement};

mod helpers;
mod king;
mod knight;
mod pawn;

pub fn gen_moves() {
    pawn::gen_pawn_moves();
    knight::gen_knight_moves();
    king::gen_king_moves();
}

pub fn get_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    pawn::get_pawn_moves(board, &mut moves);
    knight::get_knight_moves(board, &mut moves);
    king::get_king_moves(board, &mut moves);
    moves
}
