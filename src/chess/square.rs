use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Square({})", self.to_notation())
    }
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Square {
        Square((rank * 8) + file)
    }

    pub fn from_notation(s: &str) -> Option<Square> {
        let mut chars = s.chars();
        let file = chars.next()? as u8;
        let rank = chars.next()? as u8;

        if rank < b'1' || rank > b'8' {
            return None;
        }
        if file < b'a' || file > b'h' {
            return None;
        }

        let rank_index = rank - b'1';
        let file_index = file - b'a';

        Some(Square::new(rank_index, file_index))
    }

    pub fn to_notation(&self) -> String {
        let mut s = String::new();
        s.push((self.file() + b'a') as char);
        s.push((self.rank() + b'1') as char);
        s
    }

    pub fn rank(&self) -> u8 {
        self.0 / 8
    }
    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    pub fn up(&self, ranks: u8) -> Option<Square> {
        if self.rank() + ranks > 7 {
            return None;
        }
        Some(Square::new(self.rank() + ranks, self.file()))
    }
    pub fn down(&self, ranks: u8) -> Option<Square> {
        if ranks > self.rank() {
            return None;
        }
        Some(Square::new(self.rank() - ranks, self.file()))
    }
    pub fn left(&self, files: u8) -> Option<Square> {
        if files > self.file() {
            return None;
        }
        Some(Square::new(self.rank(), self.file() - files))
    }
    pub fn right(&self, files: u8) -> Option<Square> {
        if self.file() + files > 7 {
            return None;
        }
        Some(Square::new(self.rank(), self.file() + files))
    }

    pub fn flip_vertical(&self) -> Square {
        Square::new(7 - self.rank(), self.file())
    }

    pub fn flip_vertical_if(&self, condition: bool) -> Square {
        if condition {
            self.flip_vertical()
        } else {
            *self
        }
    }
}
