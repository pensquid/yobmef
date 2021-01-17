use crate::chess::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    // Should use proper tryfrom trait or something
    pub fn from_usize(number: usize) -> Option<Piece> {
        match number {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => None,
        }
    }

    // TODO: Remove duplication (you could change as_char without changing from_char!)
    pub fn as_char(&self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }

    // Note: ch must be a lowercase p,n,b,r,q,k
    pub fn from_char(ch: char) -> Option<Piece> {
        match ch {
            'p' => Some(Piece::Pawn),
            'n' => Some(Piece::Knight),
            'b' => Some(Piece::Bishop),
            'r' => Some(Piece::Rook),
            'q' => Some(Piece::Queen),
            'k' => Some(Piece::King),
            _ => None,
        }
    }

    pub fn as_char_color(&self, color: Color) -> char {
        match color {
            Color::White => self.as_char().to_ascii_uppercase(),
            Color::Black => self.as_char(),
        }
    }

    pub fn can_promote_to(&self) -> bool {
        self != &Piece::Pawn && self != &Piece::King
    }
}
