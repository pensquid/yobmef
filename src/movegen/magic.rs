use rand::rngs::StdRng;
use rand::SeedableRng;

use super::magic_utils::{
    get_occupancy_mask, get_questions_and_answers, get_rays, random_bitboard, NUM_MOVES,
};

use crate::chess::Square;
use crate::{bitboard::BitBoard, chess::Piece};

static mut MOVES: [BitBoard; NUM_MOVES] = [BitBoard::empty(); NUM_MOVES];
static mut ROOK_MAGICS: [MagicSquare; 64] = [MagicSquare::empty(); 64];
static mut BISHOP_MAGICS: [MagicSquare; 64] = [MagicSquare::empty(); 64];

// For storing some info for gen_single_magic
static mut MOVE_RAYS: [BitBoard; NUM_MOVES] = [BitBoard::empty(); NUM_MOVES];

#[derive(Debug, Clone, Copy)]
pub struct MagicSquare {
    number: BitBoard,
    occupancy_mask: BitBoard,
    offset: u32,
    right_shift: u8,
    empty: bool,
}

impl MagicSquare {
    fn new(
        number: BitBoard,
        occupancy_mask: BitBoard,
        offset: u32,
        right_shift: u8,
    ) -> MagicSquare {
        MagicSquare {
            number,
            occupancy_mask,
            offset,
            right_shift,
            empty: false,
        }
    }

    const fn empty() -> MagicSquare {
        MagicSquare {
            number: BitBoard::empty(),
            occupancy_mask: BitBoard::empty(),
            offset: 0,
            right_shift: 0,
            empty: true,
        }
    }

    pub fn lookup_hash(&self, occupancy: &BitBoard) -> BitBoard {
        if self.empty {
            panic!(
                "tried to lookup hash on an empty magic square, did you forget to run gen_moves?"
            );
        }
        
        let raw_hash = self.number * (self.occupancy_mask & occupancy);
        let shifted_hash = (raw_hash.0 as usize) >> (self.right_shift as usize);
        let hash = (self.offset as usize) + shifted_hash;
        
        eprintln!("self: {:?}", self);
        eprintln!("hash: {}", hash);

        unsafe { *MOVES.get_unchecked(hash) }
    }
}

// TODO: Go through, fully re-comprehend, and refactor this BS
fn gen_single_magic(from_sq: Square, piece: Piece, cur_offset: usize) -> usize {
    let (questions, answers) = get_questions_and_answers(from_sq, piece);
    let occupancy_mask = get_occupancy_mask(from_sq, piece);

    let mut new_offset = cur_offset;

    for i in 0..cur_offset {
        let mut found = true;

        for j in 0..answers.len() {
            unsafe {
                if MOVE_RAYS[i + j] & get_rays(from_sq, piece) != BitBoard::empty() {
                    found = false;
                    break;
                }
            }
        }

        if found {
            new_offset = i;
            break;
        }
    }

    let mut new_magic = MagicSquare::new(
        BitBoard::empty(),
        occupancy_mask,
        new_offset as u32,
        (questions.len().leading_zeros() + 1) as u8,
    );

    let mut done = false;
    let mut rng = StdRng::seed_from_u64(0xDEADBEEF12345678);

    while !done {
        let magic_bitboard = random_bitboard(&mut rng);

        // DEAR GOD
        if (occupancy_mask * magic_bitboard).count_ones() < 6 {
            continue;
        }

        // AAAAAAAAAAAAAAAA
        let mut new_answers = vec![BitBoard::empty(); questions.len()];
        done = true;
        for i in 0..questions.len() {
            let j = ((magic_bitboard * questions[i]).0 >> (new_magic.right_shift as u64)) as usize;
            if new_answers[j] == BitBoard::empty() || new_answers[j] == answers[i] {
                new_answers[j] = answers[i];
            } else {
                done = false;
                break;
            }
        }
        if done {
            new_magic.number = magic_bitboard;
        }
    }

    unsafe {
        if piece == Piece::Rook {
            ROOK_MAGICS[from_sq.0 as usize] = new_magic;
        } else {
            BISHOP_MAGICS[from_sq.0 as usize] = new_magic;
        }

        for i in 0..questions.len() {
            let j: BitBoard = (new_magic.number * questions[i]) >> (new_magic.right_shift as u64);
            MOVES[(new_magic.offset as usize) + (j.0 as usize)] |= answers[i];
            MOVE_RAYS[(new_magic.offset as usize) + (j.0 as usize)] |= get_rays(from_sq, piece);
        }
    }

    if new_offset + questions.len() < cur_offset {
        new_offset = cur_offset;
    } else {
        new_offset += questions.len();
    }

    new_offset
}

pub fn gen_all_magics() {
    let mut cur_offset = 0;

    for sq_index in 0..64 {
        cur_offset = gen_single_magic(Square(sq_index), Piece::Bishop, cur_offset);
        cur_offset = gen_single_magic(Square(sq_index), Piece::Rook, cur_offset);
    }
}

pub fn get_sliding_moves(sq: Square, piece: Piece, occupancy: &BitBoard) -> BitBoard {
    unsafe {
        if piece == Piece::Rook {
            let magic = ROOK_MAGICS[sq.0 as usize];
            magic.lookup_hash(occupancy)
        } else if piece == Piece::Bishop {
            let magic = BISHOP_MAGICS[sq.0 as usize];
            magic.lookup_hash(occupancy)
        } else {
            panic!("{:?} is not a sliding piece", piece);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::movegen::helpers::bitboard_test;
    use super::*;

    #[test]
    fn test_rook_move_lookup() {
        gen_all_magics();
        let sq = Square::from_notation("d5").unwrap();

        let mut occupancy = BitBoard::empty();
        occupancy.flip_mut(Square::from_notation("d3").unwrap());
        occupancy.flip_mut(Square::from_notation("h5").unwrap());

        let moves = get_sliding_moves(sq, Piece::Rook, &occupancy);
        bitboard_test(&moves, "f5 h5 d3 b5", "d2 d1 g3");
    }

    #[test]
    fn test_bishop_move_lookup() {
        gen_all_magics();
        let sq = Square::from_notation("g3").unwrap();

        let mut occupancy = BitBoard::empty();
        occupancy.flip_mut(Square::from_notation("d1").unwrap());
        occupancy.flip_mut(Square::from_notation("e6").unwrap());

        let moves = get_sliding_moves(sq, Piece::Rook, &occupancy);
        bitboard_test(&moves, "a2 d5 d1 e6", "g8 b3 f3");
    }
}