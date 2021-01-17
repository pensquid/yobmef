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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::Board;
    use crate::movegen::{gen_moves_once, get_legal_moves};
    use std::collections::HashMap;

    fn test_zobrist_collisions_hashmap(s: &mut HashMap<u64, Board>, depth: u16, board: &Board) {
        if depth == 0 {
            return;
        }

        for mv in get_legal_moves(board) {
            let h = hash(board);
            let previous = s.insert(h, board.clone());
            if let Some(previous) = &previous {
                if previous != board {
                    eprintln!("previous:\n{}\ncurrent:\n{}\n", previous, board);
                    eprintln!(
                        "prev enp: {:?} curr enp {:?}",
                        previous.en_passant, board.en_passant
                    );
                    panic!("hash collision! {}", h);
                }
            }

            test_zobrist_collisions_hashmap(s, depth - 1, &board.make_move(&mv));
        }
    }

    #[test]
    fn test_zobrist_collisions() {
        init_once();
        gen_moves_once();

        let mut tp = HashMap::new();

        // NOTE: Once you get this to pass, increase ply to 5.
        test_zobrist_collisions_hashmap(&mut tp, 4, &Board::from_start_pos());
    }
}
