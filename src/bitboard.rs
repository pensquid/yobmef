use crate::chess::Square;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct BitBoard(pub u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for n in 0..64 {
            if n % 8 != 0 {
                write!(f, " ")?;
            }

            let s = if self.get(Square(n)) { "1" } else { "0" };
            write!(f, "{}", s)?;

            if n % 8 == 7 {
                write!(f, "\n")?;
            }
        }

        write!(f, "")
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

    #[inline]
    pub fn mask(&self, mask: &BitBoard) -> BitBoard {
        BitBoard(self.0 & mask.0)
    }

    #[inline]
    pub fn mask_mut(&mut self, mask: &BitBoard) {
        self.0 &= mask.0;
    }

    #[inline]
    pub fn combine(&self, with: &BitBoard) -> BitBoard {
        BitBoard(self.0 | with.0)
    }

    #[inline]
    pub fn combine_mut(&mut self, with: &BitBoard) {
        self.0 |= with.0;
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

    #[inline]
    pub fn population(&mut self) -> u8 {
        // https://www.chessprogramming.org/Population_Count

        (0..64).map(|i| self.get(Square(i)) as u8).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(b.population(), 4);
    }

    #[test]
    fn test_bitboard_display() {
        let mut b = BitBoard::empty();
        b.flip_mut(Square(0));
        b.flip_mut(Square(7));
        b.flip_mut(Square(63));
        assert_eq!(
            format!("{}", b),
            "\
      1 0 0 0 0 0 0 1\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 0\n\
      0 0 0 0 0 0 0 1\n\
    "
        );
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
}
