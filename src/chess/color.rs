#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn polarize(&self) -> i16 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}
