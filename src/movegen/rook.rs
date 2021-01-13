use crate::bitboard::BitBoard;
use crate::chess::{Board, Movement, Square};

static mut ROOK_RAYS: [[BitBoard; 64]; 64] = [[BitBoard::empty(); 64]; 64];

// Magic bitboards:
// 1.  Generate rook moves as if no pieces are on the board (ROOK_RAYS)
//
// 2.  Generate all possible masked blockers configurations for every rook square
//     (masked by possible rook moves if no pieces were on the board)
//
// 3.  Use a magic number (found through bruteforce) with the property that
//     multiplying it with masked blockers results in the first N bits
//     being a perfect hash (unique forall masked blockers).
//
// 3.5 blockers = ROOK_RAYS & all_pieces
//  MASK WITH WHITE PIECES SO WE CANT TAKE OURSELVES
//
// 4.  masked blockers = (white & (black & !sides))
//     where sides = (A_FILE | H_FILE | RANK_1 | RANK_8).
//
//     Because if a black piece is on the side, it is not a blocker,
//     But a white piece on the side is.
//     A black piece not on the side is a blocker however,
//     you can still take it.
//
//     Q: How do you determine legal moves from blockers?
//
//
// 5.  ROOK_MOVES[MAGIC_NUM * (ROOK_RAYS[] & )] gives you your legal moves

pub fn gen_rook_moves() {
    //
}

pub fn get_rook_moves(board: &Board, moves: &mut Vec<Movement>) {
    for from_square_index in 0..64 {
        let from_square = Square(from_square_index);
    }
}
