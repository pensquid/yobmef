use crate::bitboard::BitBoard;
use std::fmt;

pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, PartialEq, Eq)]
pub enum Color { Black, White }

#[derive(Debug, PartialEq, Eq)]
pub enum Piece { Rook, Knight, Bishop, Queen, King, Pawn }

impl Piece {
    // TODO: Use proper trait (tryfrom?)
    // (written explicitly so nobody screws with the order, ruining it.)
    pub fn as_usize(&self) -> usize {
        match self {
            Piece::Rook => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Queen => 3,
            Piece::King => 4,
            Piece::Pawn => 5,
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Piece::Rook => 'r',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Queen => 'q',
            Piece::King => 'k',
            Piece::Pawn => 'p'
        }
    }

    pub fn as_char_color(&self, color: Color) -> char {
        match color {
            Color::White => self.as_char(),
            Color::Black => self.as_char().to_ascii_uppercase(),
        }
    }
}

#[derive(Debug)]
pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    color_combined: [BitBoard; NUM_COLORS],
    side_to_move: Color,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = [[' '; 8]; 8];

        // TODO (psudocode)
        //  loop over squares covered in color_combined[Color::White]
        //    and set board[col][row] = piece.as_char_color(White)
        //
        //  loop over squares covered in color_combined[Color::Black]
        //    and set board[col][row] = piece.as_char_color(Black)

        let s = board.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<String>();
        write!(f, "{}", s)
    }
}


impl Board {
    pub fn empty() -> Board {
        Board {
            pieces: [BitBoard(0); NUM_PIECES],
            color_combined: [BitBoard(0); NUM_COLORS],
            side_to_move: Color::White,
        }
    }

    pub fn from_fen(s: &str) -> Option<Board> {
        let mut split = s.split(' ');
        eprintln!("{:?}", split.next());
        Some(Board::empty())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fen() {
        let b = Board::from_fen(STARTING_FEN);
        eprintln!("BOARD: '{}'", b.unwrap());
        // logs only showed when test fails
        // assert_eq!(2+2, 5);
    }
}
