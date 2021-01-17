use super::{Color, Movement, Square};
use crate::bitboard::BitBoard;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    WhiteKingside = 0,
    WhiteQueenside = 1,
    BlackKingside = 2,
    BlackQueenside = 3,
}

impl CastlingSide {
    #[inline]
    pub fn from_movement(movement: &Movement) -> Option<Self> {
        match movement {
            Movement {
                from_square: Square(4),
                to_square: Square(6),
                promote: None,
            } => Some(Self::WhiteKingside),
            Movement {
                from_square: Square(4),
                to_square: Square(2),
                promote: None,
            } => Some(Self::WhiteQueenside),

            Movement {
                from_square: Square(60),
                to_square: Square(62),
                promote: None,
            } => Some(Self::BlackKingside),
            Movement {
                from_square: Square(60),
                to_square: Square(58),
                promote: None,
            } => Some(Self::BlackQueenside),

            _ => None,
        }
    }

    #[inline]
    pub fn of_color(color: Color) -> [Self; 2] {
        match color {
            Color::White => [CastlingSide::WhiteKingside, CastlingSide::WhiteQueenside],
            Color::Black => [CastlingSide::BlackKingside, CastlingSide::BlackQueenside],
        }
    }

    #[inline]
    pub fn from_rook_square(square: Square) -> Option<Self> {
        match square {
            Square(0) => Some(CastlingSide::WhiteQueenside),
            Square(7) => Some(CastlingSide::WhiteKingside),
            Square(56) => Some(CastlingSide::BlackQueenside),
            Square(63) => Some(CastlingSide::BlackKingside),
            _ => None,
        }
    }

    #[inline]
    pub fn color(&self) -> Color {
        match self {
            CastlingSide::WhiteKingside => Color::White,
            CastlingSide::WhiteQueenside => Color::White,
            CastlingSide::BlackKingside => Color::Black,
            CastlingSide::BlackQueenside => Color::Black,
        }
    }

    // PAIN AND SUFFERING
    #[inline]
    pub fn get_rook_movement(&self) -> Movement {
        match self {
            CastlingSide::WhiteKingside => Movement::new(Square(7), Square(5), None),
            CastlingSide::WhiteQueenside => Movement::new(Square(0), Square(3), None),
            CastlingSide::BlackKingside => Movement::new(Square(63), Square(61), None),
            CastlingSide::BlackQueenside => Movement::new(Square(56), Square(59), None),
        }
    }

    #[inline]
    pub fn get_king_movement(&self) -> Movement {
        match self {
            CastlingSide::WhiteKingside => Movement::new(Square(4), Square(6), None),
            CastlingSide::WhiteQueenside => Movement::new(Square(4), Square(2), None),
            CastlingSide::BlackKingside => Movement::new(Square(60), Square(62), None),
            CastlingSide::BlackQueenside => Movement::new(Square(60), Square(58), None),
        }
    }

    #[inline]
    pub fn get_castling_middle(&self) -> BitBoard {
        match self {
            CastlingSide::WhiteKingside => BitBoard(0x60),
            CastlingSide::WhiteQueenside => BitBoard(0xe),
            CastlingSide::BlackKingside => BitBoard(0x6000000000000000),
            CastlingSide::BlackQueenside => BitBoard(0xe00000000000000),
        }
    }

    #[inline]
    pub fn get_castling_not_attacked(&self) -> BitBoard {
        match self {
            CastlingSide::WhiteKingside => BitBoard(0x70),
            CastlingSide::WhiteQueenside => BitBoard(0x1c),
            CastlingSide::BlackKingside => BitBoard(0x7000000000000000),
            CastlingSide::BlackQueenside => BitBoard(0x1c00000000000000),
        }
    }
}
