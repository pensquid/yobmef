#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    #[inline]
    pub fn polarize(&self) -> i16 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    #[inline]
    pub fn as_char(&self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}
