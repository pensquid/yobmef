use std::fmt;

use crate::chess::Piece;
use crate::chess::Square;

// Calling it Movement and not Move because "move" is a keyword
#[derive(Clone, PartialEq, Eq)]
pub struct Movement {
    pub from_square: Square,
    pub to_square: Square,
    pub promote: Option<Piece>,
}

impl fmt::Display for Movement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

impl fmt::Debug for Movement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Movement({})", self.to_notation())
    }
}

impl Movement {
    pub fn new(from_square: Square, to_square: Square, promote: Option<Piece>) -> Movement {
        Movement {
            from_square,
            to_square,
            promote,
        }
    }

    pub fn from_notation(lan: &str) -> Option<Movement> {
        let from_square = Square::from_notation(lan.get(0..2)?)?;
        let to_square = Square::from_notation(lan.get(2..4)?)?;
        let promote = lan.chars().nth(4).and_then(Piece::from_char);

        Some(Movement {
            from_square,
            to_square,
            promote,
        })
    }

    pub fn to_notation(&self) -> String {
        let from_notation = self.from_square.to_notation();
        let to_notation = self.to_square.to_notation();

        let mut lan = String::new();
        lan.push_str(&from_notation);
        lan.push_str(&to_notation);

        if let Some(piece) = self.promote {
            lan.push(piece.as_char());
        }

        lan
    }

    pub fn hash(&self) -> u16 {
        ((self.promote.unwrap_or(Piece::Pawn) as u16) << 12)
            | ((self.from_square.0 as u16) << 6)
            | (self.to_square.0 as u16)
    }

    #[inline]
    pub fn vdelta(&self) -> i8 {
        self.to_square.rank() as i8 - self.from_square.rank() as i8
    }

    #[inline]
    pub fn hdelta(&self) -> i8 {
        self.to_square.file() as i8 - self.from_square.file() as i8
    }
}
