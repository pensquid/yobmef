use rand::Rng;

use crate::chess::Square;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitBoard(pub u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Go by rank, printing each row
        for rank_index in 0..8 {
            let rank_index = 7 - rank_index;
            write!(f, "{}", rank_index + 1)?;

            for file_index in 0..8 {
                let sq = Square::new(rank_index, file_index);
                write!(f, " {}", if self.get(sq) { 'X' } else { '.' })?;
            }
            write!(f, "\n")?;
        }

        write!(f, "  a b c d e f g h")
    }
}

impl BitBoard {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn get(&self, sq: Square) -> bool {
        ((self.0 >> sq.0) & 1) != 0
    }

    #[inline]
    pub fn flip(&self, sq: Square) -> BitBoard {
        BitBoard(self.0 ^ (1 << sq.0))
    }

    #[inline]
    pub fn flip_mut(&mut self, sq: Square) {
        self.0 ^= 1 << sq.0;
    }

    pub fn flip_vertical(&self) -> BitBoard {
        let mut board = self.clone();
        board.flip_vertical_mut();
        board
    }

    pub fn flip_vertical_mut(&mut self) {
        // https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#FlipVertically
        // Optimized algorithm with delta swaps
        // As long as it works :D

        const K1: u64 = 0x00FF00FF00FF00FF;
        const K2: u64 = 0x0000FFFF0000FFFF;

        self.0 = ((self.0 >> 8) & K1) | ((self.0 & K1) << 8);
        self.0 = ((self.0 >> 16) & K2) | ((self.0 & K2) << 16);
        self.0 = (self.0 >> 32) | (self.0 << 32);
    }

    pub fn flip_vertical_if(&self, condition: bool) -> BitBoard {
        if condition {
            self.flip_vertical()
        } else {
            *self
        }
    }

    pub fn flip_vertical_if_mut(&mut self, condition: bool) {
        if condition {
            self.flip_vertical_mut()
        }
    }

    #[inline]
    pub fn count_ones(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn random<R: Rng>(rng: &mut R) -> BitBoard {
        BitBoard(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
    }

    #[inline]
    pub fn from_square(sq: Square) -> BitBoard {
        BitBoard(1 << sq.0)
    }

    // Convert this bitboard to a Square.
    // If the bitboard has multiple bits flipped,
    // This function must still return a valid square.
    #[inline]
    fn to_square(&self) -> Square {
        debug_assert!(self.0 != 0, "empty board cannot be made into a square");
        Square(self.0.trailing_zeros() as u8)
    }
}

// Iterate over the bitboard, order is undefined and subject to change.
impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        } else {
            let result = self.to_square();
            *self ^= BitBoard::from_square(result);
            Some(result)
        }
    }
}

macro_rules! impl_op {
    ($op:ident, $fun:ident) => {
        use std::ops::$op;
        impl $op<Self> for BitBoard {
            type Output = Self;

            fn $fun(self, other: Self) -> Self::Output {
                BitBoard(self.0.$fun(other.0))
            }
        }
        impl<'b> $op<&'b Self> for BitBoard {
            type Output = Self;

            fn $fun(self, other: &'b Self) -> Self::Output {
                BitBoard(self.0.$fun(other.0))
            }
        }
        impl $op<u64> for BitBoard {
            type Output = Self;

            fn $fun(self, other: u64) -> Self::Output {
                BitBoard(self.0.$fun(other))
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($op:ident, $fun:ident) => {
        use std::ops::$op;
        impl $op<Self> for BitBoard {
            fn $fun(&mut self, rhs: Self) {
                self.0.$fun(rhs.0);
            }
        }
        impl<'b> $op<&'b Self> for BitBoard {
            fn $fun(&mut self, rhs: &'b Self) {
                &self.0.$fun(rhs.0);
            }
        }
        impl<'a> $op<Self> for &'a mut BitBoard {
            fn $fun(&mut self, rhs: Self) {
                self.0.$fun(rhs.0);
            }
        }
        impl<'a, 'b> $op<&'b Self> for &'a mut BitBoard {
            fn $fun(&mut self, rhs: &'b Self) {
                self.0.$fun(rhs.0);
            }
        }
        impl $op<u64> for BitBoard {
            fn $fun(&mut self, rhs: u64) {
                self.0.$fun(rhs);
            }
        }
        impl<'a> $op<u64> for &'a mut BitBoard {
            fn $fun(&mut self, rhs: u64) {
                self.0.$fun(rhs);
            }
        }
    };
}

impl_op!(BitOr, bitor);
impl_op_assign!(BitOrAssign, bitor_assign);

impl_op!(BitXor, bitxor);
impl_op_assign!(BitXorAssign, bitxor_assign);

impl_op!(BitAnd, bitand);
impl_op_assign!(BitAndAssign, bitand_assign);

impl_op!(Shl, shl);
impl_op_assign!(ShlAssign, shl_assign);

impl_op!(Shr, shr);
impl_op_assign!(ShrAssign, shr_assign);

use std::ops::Mul;
impl Mul<u64> for BitBoard {
    type Output = Self;

    fn mul(self, other: u64) -> Self::Output {
        BitBoard(self.0.overflowing_mul(other).0)
    }
}
impl Mul<BitBoard> for BitBoard {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        BitBoard(self.0.overflowing_mul(other.0).0)
    }
}

use std::ops::Not;
impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_or() {
        let mut b = BitBoard(1);
        assert_eq!(b | BitBoard(2), BitBoard(3));
        b |= BitBoard(2);
        assert_eq!(b, BitBoard(3));
    }

    #[test]
    fn test_bitboard_get_flip() {
        let b = BitBoard::empty();
        assert_eq!(b.get(Square::new(0, 0)), false);

        let b2 = b.flip(Square::new(0, 3));
        eprintln!("b2: {:?}", b2);
        assert_eq!(b2.get(Square::new(0, 3)), true);

        let b3 = b2.flip(Square::new(0, 0));
        eprintln!("b3: {:?}", b3);
        assert_eq!(b3.get(Square::new(0, 0)), true);
        assert_eq!(b3.get(Square::new(0, 3)), true);
    }

    #[test]
    fn test_bitboard_population() {
        let mut b = BitBoard::empty();
        b.0 |= 0b1111;
        assert_eq!(b.count_ones(), 4);
    }

    #[test]
    fn test_flip_vertical() {
        let mut b = BitBoard::empty();

        b.flip_mut(Square::new(0, 0));
        b.flip_mut(Square::new(1, 1));
        b.flip_mut(Square::new(2, 7));
        b.flip_vertical_mut();

        assert!(!b.get(Square::new(0, 0)));
        assert!(b.get(Square::new(7, 0)));
        assert!(b.get(Square::new(6, 1)));
        assert!(b.get(Square::new(5, 7)));
    }

    #[test]
    fn test_iterate() {
        let mut b = BitBoard(0b1011);

        eprintln!("bitboard:\n{}\n", b);

        // Iteration currently goes from top left by right.
        assert_eq!(b.next(), Some(Square(0)));
        assert_eq!(b.next(), Some(Square(1)));
        assert_eq!(b.next(), Some(Square(3)));
        assert_eq!(b.next(), None);

        assert_eq!(b.0, 0);
    }
}
