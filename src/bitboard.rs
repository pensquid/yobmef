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

            let s = if self.get(n) { "1" } else { "0" };
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
    pub fn new() -> Self {
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
        let b = BitBoard::new();
        assert_eq!(b.get(0), false);

        let b2 = b.flip(3);
        eprintln!("b2: {:?}", b2);
        assert_eq!(b2.get(3), true);

        let b3 = b2.flip(0);
        eprintln!("b3: {:?}", b3);
        assert_eq!(b3.get(0), true);
        assert_eq!(b3.get(3), true);
    }

    #[test]
    fn test_bitboard_population() {
        let mut b = BitBoard::new();
        b.0 |= 0b1111;
        assert_eq!(b.population(), 4);
    }

    #[test]
    fn test_bitboard_display() {
        let mut b = BitBoard::new();
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
}
