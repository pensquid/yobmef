use std::sync::Once;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::bitboard::BitBoard;
use crate::chess::{Board, Square, NUM_COLORS, NUM_PIECES};

const NUM_RANDOMS: usize = NUM_PIECES * NUM_COLORS;
static mut RANDOMS: [[BitBoard; 64]; NUM_RANDOMS] = [[BitBoard::empty(); 64]; NUM_RANDOMS];
// static mut EMPTY_RANDOM: BitBoard = BitBoard::empty();

static START: Once = Once::new();

pub fn init_once() {
    START.call_once(|| {
        let mut rng = StdRng::from_entropy();

        for i in 0..NUM_RANDOMS {
            for j in 0..64 {
                unsafe {
                    RANDOMS[i][j] = BitBoard::random(&mut rng);
                }
            }
        }

        // unsafe {
        //     EMPTY_RANDOM = BitBoard::random(&mut rng);
        // }
    });
}

// TODO: Update incrementally on board
pub fn hash(board: &Board) -> u64 {
    let mut hash = BitBoard(0);

    for i in 0..64 {
        let piece = board.piece_on(Square(i));

        if let Some(piece) = piece {
            let color = board.color_on(Square(i));

            if let Some(color) = color {
                unsafe { hash ^= RANDOMS[(piece as usize) * (color as usize)][i as usize] }
            }
        }
    }

    hash.0
}
