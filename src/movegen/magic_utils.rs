use rand::Rng;

use crate::bitboard::BitBoard;
use crate::chess::{Piece, Square};

use super::helpers::NOT_EDGES;

const ROOK_BITS: usize = 12;
const BISHOP_BITS: usize = 9;

// Just some math magic involving powers of two,
// don't try to understand it
pub const NUM_MOVES: usize = (64 * (1 << ROOK_BITS)) + (64 * (1 << BISHOP_BITS));

fn get_bishop_rays(from_sq: Square) -> BitBoard {
    let mut rays = BitBoard::empty();

    for to_sq_index in 0..64 {
        let to_sq = Square(to_sq_index);

        let abs_relative_rank = (to_sq.rank() as i8 - from_sq.rank() as i8).abs();
        let abs_relative_file = (to_sq.file() as i8 - from_sq.file() as i8).abs();
        let is_diagonal = abs_relative_rank == abs_relative_file;

        if is_diagonal && from_sq != to_sq {
            rays.flip_mut(to_sq);
        }
    }

    rays
}

fn get_rook_rays(from_sq: Square) -> BitBoard {
    let mut rays = BitBoard::empty();

    for to_sq_index in 0..64 {
        let to_sq = Square(to_sq_index);
        let is_same_rank = from_sq.rank() == to_sq.rank();
        let is_same_file = from_sq.file() == to_sq.file();

        if (is_same_rank || is_same_file) && from_sq != to_sq {
            rays.flip_mut(to_sq);
        }
    }

    rays
}

pub fn get_rays(sq: Square, piece: Piece) -> BitBoard {
    if piece == Piece::Rook {
        get_rook_rays(sq)
    } else {
        get_bishop_rays(sq)
    }
}

pub fn get_occupancy_mask(sq: Square, piece: Piece) -> BitBoard {
    if piece == Piece::Bishop {
        get_rays(sq, piece) & NOT_EDGES
    } else {
        // We can't just use NOT_EDGES for rooks because rooks can be on
        // the edges and that would remove the entire ray(s) so we need to
        // mask out only the tips
        let mut tip_mask = BitBoard::empty();

        for mask_sq_index in 0..64 {
            let mask_sq = Square(mask_sq_index);

            let is_same_rank = sq.rank() == mask_sq.rank();
            let is_edge_file = mask_sq.file() == 0 || mask_sq.file() == 7;
            let is_rank_tip = is_same_rank && is_edge_file;

            let is_same_file = sq.file() == mask_sq.file();
            let is_edge_rank = mask_sq.rank() == 0 || mask_sq.rank() == 7;
            let is_file_tip = is_same_file && is_edge_rank;

            if is_rank_tip || is_file_tip {
                tip_mask.flip_mut(mask_sq);
            }
        }

        get_rays(sq, piece) & !tip_mask
    }
}

pub fn random_bitboard<R: Rng>(rng: &mut R) -> BitBoard {
    BitBoard(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
}

// These directions functions get a list of functions which take squares
// and return directions for the rooks/bishops to move in as a utility for
// generating answers, basically taken entirely directly from the chess crate
// because an iterable is more elegant than traditional approaches using msb
// and lsb and we don't need too much speed anyways

fn rook_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn left(sq: Square) -> Option<Square> {
        sq.left(1)
    }
    fn right(sq: Square) -> Option<Square> {
        sq.right(1)
    }
    fn up(sq: Square) -> Option<Square> {
        sq.up(1)
    }
    fn down(sq: Square) -> Option<Square> {
        sq.down(1)
    }

    vec![left, right, up, down]
}

fn bishop_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn nw(sq: Square) -> Option<Square> {
        sq.left(1).map_or(None, |s| s.up(1))
    }
    fn ne(sq: Square) -> Option<Square> {
        sq.right(1).map_or(None, |s| s.up(1))
    }
    fn sw(sq: Square) -> Option<Square> {
        sq.left(1).map_or(None, |s| s.down(1))
    }
    fn se(sq: Square) -> Option<Square> {
        sq.right(1).map_or(None, |s| s.down(1))
    }

    vec![nw, ne, sw, se]
}

// I stole the terminology of questions and answers from the chess crate,
// questions are the possibilities for occupancy used as lookups in the hash
// and answers are the moves accounting for the blocking

fn get_questions(occupancy_mask: BitBoard) -> Vec<BitBoard> {
    let mut result = Vec::new();

    let mut one_bits = Vec::new();
    for sq_index in 0..64 {
        let sq = Square(sq_index);
        if occupancy_mask.get(sq) { one_bits.push(sq); }
    }

    for i in 0..(1 << occupancy_mask.count_ones()) {
        let mut current = BitBoard::empty();

        for (j, sq) in one_bits.iter().enumerate() {
            if (i >> j) & 1 == 1 {
                current.flip_mut(*sq);
            }
        }

        result.push(current);
    }

    result
}

pub fn get_questions_and_answers(sq: Square, piece: Piece) -> (Vec<BitBoard>, Vec<BitBoard>) {
    let mask = get_occupancy_mask(sq, piece);
    let questions = get_questions(mask);

    let mut answers = Vec::new();

    let directions = if piece == Piece::Bishop {
        bishop_directions()
    } else {
        rook_directions()
    };

    for question in questions.iter() {
        let mut answer = BitBoard::empty();

        // Iterate over the directions and flip bits in the answer bitboard
        // until encountering a blocked square
        for get_movement in directions.iter() {
            let mut next = get_movement(sq);

            while next != None {
                answer ^= BitBoard(1 << next.unwrap().0);

                if (BitBoard(1 << next.unwrap().0) & question) != BitBoard::empty() {
                    // Stop the loop if blocked
                    break;
                }

                next = get_movement(next.unwrap());
            }
        }
        answers.push(answer);
    }

    (questions, answers)
}

#[cfg(test)]
mod tests {
    use crate::movegen::helpers::bitboard_test;
    use super::*;

    #[test]
    fn test_get_bishop_rays() {
        let rays = get_bishop_rays(Square::from_notation("d5").unwrap());
        bitboard_test(&rays, "c6 a8 b3 g2 e6", "b4 h5 d5");
    }
    
    #[test]
    fn test_get_rook_rays() {
        let rays = get_rook_rays(Square::from_notation("a6").unwrap());
        bitboard_test(&rays, "a8 a3 d6 h6", "a6 b5");
    }

    #[test]
    fn test_get_bishop_occupancy_mask() {
        let rays = get_occupancy_mask(Square::from_notation("b3").unwrap(), Piece::Bishop);
        bitboard_test(&rays, "c2 d5", "a2 g8 d1 a4 b3");
    }

    #[test]
    fn test_get_rook_occupancy_mask() {
        let rays = get_occupancy_mask(Square::from_notation("c5").unwrap(), Piece::Rook);
        bitboard_test(&rays, "f5 c2", "c1 a5 c5");
    }

    #[test]
    fn test_get_rook_occupancy_mask_corner() {
        let rays = get_occupancy_mask(Square::from_notation("a8").unwrap(), Piece::Rook);
        bitboard_test(&rays, "d8 g8 a5 a2", "h8 a8 a1");
    }

    #[test]
    fn test_get_questions_and_answers() {
        let sq = Square::from_notation("a7").unwrap();
        let piece = Piece::Rook;

        let (questions, answers) = get_questions_and_answers(sq, piece);

        assert_eq!(questions.len().count_ones(), 1);
        assert_eq!(questions.len(), answers.len());

        assert_eq!(
            answers.iter().fold(BitBoard::empty(), |b, n| b | n),
            get_rays(sq, piece)
        );
    }
}