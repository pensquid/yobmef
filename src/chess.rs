use crate::bitboard::BitBoard;
use std::fmt;

pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn position_to_square(rank: char, file: char) -> Option<u8> {
    let rank = rank as u8;
    let file = file as u8;

    if rank < b'1' || rank > b'8' { return None; }
    if file < b'a' || file > b'h' { return None; }

    let rank_index = rank - b'1';
    let file_index = file - b'a';
    let square = (rank_index * 8) + file_index;

    Some(square)
}

// Calling it Movement and not Move because "move" is a keyword
#[derive(Debug)]
pub struct Movement {
    from_square: u32,
    to_square: u32,
    promote: Option<Piece>,
}

impl Movement {
    pub fn from_notation(lan: &str) -> Option<Movement> {
        // Cursed code incoming
        // TODO: Write tests
        let mut lan = lan.chars();

        let from_file = lan.next()?;
        let from_rank = lan.next()?;
        let from_square = position_to_square(from_rank, from_file)? as u32;

        let to_file = lan.next()?;
        let to_rank = lan.next()?;
        let to_square = position_to_square(to_rank, to_file)? as u32;

        let mut promote = None;
        if let Some(ch) = lan.next() {
            promote = Piece::from_char(ch);
        }

        Some(Movement {
            from_square,
            to_square,
            promote,
        })
    }
}

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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    WhiteKingside = 0,
    WhiteQueenside = 1,
    BlackKingside = 2,
    BlackQueenside = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    color_combined: [BitBoard; NUM_COLORS],
    en_passant: Option<u8>,
    castling: u8, // 4 bits needed, from rtl: white kingside, white queenside, black kingside, black queenside
    side_to_move: Color,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board = [[' '; 8]; 8];

        for rank_index in 0..8 {
            for file_index in 0..8 {
                let square = (rank_index * 8) + file_index;

                if self.color_combined[Color::White as usize].get(square as u32) {
                    board[7 - rank_index][file_index] = 'w';
                } else if self.color_combined[Color::Black as usize].get(square as u32) {
                    board[7 - rank_index][file_index] = 'b';
                }
            }
        }

        let s = board
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", s)
    }
}

impl Board {
    pub fn empty() -> Board {
        Board {
            pieces: [BitBoard(0); NUM_PIECES],
            color_combined: [BitBoard(0); NUM_COLORS],
            en_passant: None,
            castling: 0b1111,
            side_to_move: Color::White,
        }
    }

    pub fn from_fen(s: &str) -> Option<Board> {
        let mut board = Board::empty();

        let mut fen_split = s.split(' ');
        let mut board_split = fen_split.next()?.split('/');

        let mut rank_index = 8;
        while let Some(rank) = board_split.next() {
            rank_index -= 1;
            let mut file_index = 0;

            for piece_char in rank.chars() {
                if piece_char.is_numeric() {
                    file_index += piece_char.to_digit(10)?;
                } else {
                    let piece = Piece::from_char(piece_char.to_ascii_lowercase())?;
                    let color = if piece_char.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let square = (8 * rank_index) + file_index;

                    board.pieces[piece as usize].flip_mut(square);
                    board.color_combined[color as usize].flip_mut(square);
                    file_index += 1;
                }
            }
        }

        board.side_to_move = if fen_split.next()? == "w" {
            Color::White
        } else {
            Color::Black
        };

        let castling_string = fen_split.next()?;
        board.set_castling_mut(CastlingSide::WhiteKingside, castling_string.contains('K'));
        board.set_castling_mut(CastlingSide::WhiteQueenside, castling_string.contains('Q'));
        board.set_castling_mut(CastlingSide::BlackKingside, castling_string.contains('k'));
        board.set_castling_mut(CastlingSide::BlackQueenside, castling_string.contains('q'));

        let en_passant = fen_split.next()?.as_bytes();
        if en_passant.len() == 2 {
            board.en_passant = position_to_square(en_passant[1] as char, en_passant[0] as char);
        }

        Some(board)
    }

    pub fn from_start_pos() -> Board {
        Board::from_fen(STARTING_FEN).unwrap()
    }

    pub fn set_castling_mut(&mut self, side: CastlingSide, can_castle: bool) {
        let side_bit = side as u8;
        if can_castle {
            self.castling |= 1 << side_bit;
        } else {
            self.castling &= !(1 << side_bit);
        }
    }

    pub fn make_move_mut(&mut self, movement: Movement) -> Option<()> {
        // Find the color
        // Who needs to handle edge cases anyways
        let is_white = self.color_combined[Color::White as usize].get(movement.from_square);
        let color = if is_white { Color::White } else { Color::Black };

        if self.color_combined[color as usize].get(movement.to_square) { return None; }

        // Find the piece type
        let piece = self.pieces.iter().position(|b| b.get(movement.from_square))?;
        let piece = Piece::from_usize(piece).unwrap();

        // Move to the destination or promote
        if let Some(promoted_piece) = movement.promote {
            if !promoted_piece.can_promote_to() { return None; }
            self.pieces[promoted_piece as usize].flip_mut(movement.to_square);
        } else {
            self.pieces[piece as usize].flip_mut(movement.to_square);
        }

        // Remove the piece
        self.pieces[piece as usize].flip_mut(movement.from_square);

        // Move the piece in the color grid
        self.color_combined[color as usize].flip_mut(movement.from_square);
        self.color_combined[color as usize].flip_mut(movement.to_square);

        if (piece == Piece::Pawn) && (movement.to_square - movement.from_square == 16) {
            // En passant
            self.en_passant = Some((movement.to_square - 8) as u8);
        } else {
            self.en_passant = None;
        }

        // Switch side to move
        self.side_to_move = self.side_to_move.other();

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fen_starting() {
        let b = Board::from_fen(STARTING_FEN).expect("starting fen is valid");

        assert!(b.en_passant.is_none());
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::White);

        assert!(b.pieces[Piece::Rook as usize].get(0));
        assert!(b.pieces[Piece::Rook as usize].get(7));
        assert!(!b.pieces[Piece::Rook as usize].get(1));
    }

    #[test]
    fn test_from_fen_castling() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1")
            .expect("castling fen is valid");

        assert!(b.en_passant.is_none());
        assert_eq!(b.castling, 0b1001);
        assert_eq!(b.side_to_move, Color::White);
    }

    #[test]
    fn test_from_fen_e2e4() {
        let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
            .expect("e2e4 fen is valid");
        let e3 = (2 * 8) + 4;
        let e4 = (3 * 8) + 4;

        eprintln!("board:\n{}", b);

        // Just moved a pawn forward 2, so en_passant
        assert_eq!(b.en_passant, Some(e3));
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::Black);

        assert!(b.pieces[Piece::Pawn as usize].get(e4));
    }

    #[test]
    fn test_from_fen_invalid() {
        assert!(Board::from_fen("").is_none());
    }

    #[test]
    fn test_make_move_e2e4() {
        let mut b = Board::from_fen(STARTING_FEN).expect("starting fen is valid");

        assert!(b.en_passant.is_none());
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::White);

        b.make_move_mut(Movement::from_notation("e2e4").expect("movement is valid"));

        let e3 = (2 * 8) + 4;
        let e4 = (3 * 8) + 4;

        // Just moved a pawn forward 2, so en_passant
        assert_eq!(b.en_passant, Some(e3));
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::Black);

        assert!(b.pieces[Piece::Pawn as usize].get(e4));
    }

    #[test]
    fn test_make_move_promote() {
        // Very common and realistic board position 11/10
        let mut b = Board::from_fen("1nbqkbnr/rP1ppppp/p1p5/8/8/8/1PPPPPPP/RNBQKBNR w KQk - 1 5").expect("before promotion fen is valid");
        
        b.make_move_mut(Movement::from_notation("b7c8q").expect("movement is valid"));

        let b7 = (6 * 8) + 1;
        let c8 = (7 * 8) + 2;

        assert!(!b.pieces[Piece::Pawn as usize].get(b7));
        assert!(!b.pieces[Piece::Pawn as usize].get(c8));
        assert!(b.pieces[Piece::Queen as usize].get(c8));
    }
}
