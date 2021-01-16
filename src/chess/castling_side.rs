use super::{Color, Movement, Square};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    WhiteKingside = 0,
    WhiteQueenside = 1,
    BlackKingside = 2,
    BlackQueenside = 3,
}

impl CastlingSide {
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

    pub fn of_color(color: Color) -> [Self; 2] {
        match color {
            Color::White => [CastlingSide::WhiteKingside, CastlingSide::WhiteQueenside],
            Color::Black => [CastlingSide::BlackKingside, CastlingSide::BlackQueenside],
        }
    }

    pub fn from_rook_square(square: Square) -> Option<Self> {
        match square {
            Square(0) => Some(CastlingSide::WhiteQueenside),
            Square(7) => Some(CastlingSide::WhiteKingside),
            Square(56) => Some(CastlingSide::BlackQueenside),
            Square(63) => Some(CastlingSide::BlackKingside),
            _ => None,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            CastlingSide::WhiteKingside => Color::White,
            CastlingSide::WhiteQueenside => Color::White,
            CastlingSide::BlackKingside => Color::Black,
            CastlingSide::BlackQueenside => Color::Black,
        }
    }

    // PAIN AND SUFFERING
    pub fn get_rook_movement(&self) -> Movement {
        match self {
            CastlingSide::WhiteKingside => Movement::new(Square(7), Square(5), None),
            CastlingSide::WhiteQueenside => Movement::new(Square(0), Square(3), None),
            CastlingSide::BlackKingside => Movement::new(Square(63), Square(61), None),
            CastlingSide::BlackQueenside => Movement::new(Square(56), Square(59), None),
        }
    }
}
