use crate::bitboard::BitBoard;
use std::fmt;

pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (rank, file) = self.to_notation();
        write!(f, "{}{}", file, rank)
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

    pub fn to_notation(&self) -> (char, char) {
        ((self.rank() + b'1') as char, (self.file() + b'a') as char)
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

// Calling it Movement and not Move because "move" is a keyword
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    from_square: Square,
    to_square: Square,
    promote: Option<Piece>,
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
        let from_square = Square::from_notation(&lan.get(0..2)?)?;
        let to_square = Square::from_notation(&lan.get(2..4)?)?;
        let promote = lan.chars().nth(4).and_then(|ch| Piece::from_char(ch));

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
        lan.push(from_notation.1);
        lan.push(from_notation.0);
        lan.push(to_notation.1);
        lan.push(to_notation.0);

        if let Some(piece) = self.promote {
            lan.push(piece.as_char());
        }

        lan
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

    // TODO: Finish later
    // pub fn as_char_fancy(&self, color: Color) -> char {
    //     match color {
    //         Color::White => {
    //             match self {
    //                 Piece::Pawn => '♙',
    //                 Piece::Knight => '♘',
    //             }
    //         }
    //     }
    // }

    pub fn can_promote_to(&self) -> bool {
        self != &Piece::Pawn && self != &Piece::King
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    color_combined: [BitBoard; NUM_COLORS],
    pub en_passant: Option<Square>,
    pub side_to_move: Color,
    castling: u8, // 4 bits needed, from rtl: white kingside, white queenside, black kingside, black queenside
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Go by rank, printing each row
        for rank_index in 0..8 {
            let rank_index = 7 - rank_index;
            write!(f, "{}", rank_index + 1)?;

            for file_index in 0..8 {
                let square = Square::new(rank_index, file_index);

                let character = match self.piece_on(square) {
                    Some(piece) => piece.as_char_color(self.color_on(square).unwrap()),
                    None => '.',
                };

                write!(f, " {}", character)?;
            }
            write!(f, "\n")?;
        }

        write!(f, "  a b c d e f g h")
    }
}

impl Board {
    // replace old piece with new piece, return old piece
    // if the board is invalid, get ready for some fun debugging
    pub fn replace_mut(&mut self, piece: Piece, square: Square) -> Option<Piece> {
        let old_piece = self.piece_on(square);
        if let Some(old_piece) = old_piece {
            self.pieces[old_piece as usize].flip_mut(square);
            self.color_combined[self.side_to_move.other() as usize].flip_mut(square);
        }

        self.pieces[piece as usize].flip_mut(square);

        old_piece
    }

    pub fn assert_valid(&self) {
        let bitboard = self.color_combined_both();

        for sq in 0..64 {
            let sq = Square(sq);

            if bitboard.get(sq) {
                // multiple pieces on the same square
                let num_on_square: u8 = (0..NUM_PIECES).map(|p| self.pieces[p].get(sq) as u8).sum();
                assert_eq!(num_on_square, 1, "multiple pieces on {}", sq);

                // multiple ownership
                let num_owners: u8 = (0..NUM_COLORS)
                    .map(|c| self.color_combined[c].get(sq) as u8)
                    .sum();
                assert_eq!(num_owners, 1, "{} owners of square {}", num_owners, sq);
            }
        }

        // TODO: check for pawns on the 0th or 7th row

        // Check number of kings (1 each)
        let kings = self.pieces(Piece::King);
        let num_white_kings = kings.mask(self.color_combined(Color::White)).sum();
        assert_eq!(num_white_kings, 1, "{} white kings", num_white_kings);

        let num_black_kings = kings.mask(self.color_combined(Color::Black)).sum();
        assert_eq!(num_black_kings, 1, "{} black kings", num_black_kings);
    }

    pub fn empty() -> Board {
        Board {
            pieces: [BitBoard(0); NUM_PIECES],
            color_combined: [BitBoard(0); NUM_COLORS],
            en_passant: None,
            castling: 0b1111,
            side_to_move: Color::White,
        }
    }

    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        for piece in 0..NUM_PIECES {
            if self.pieces[piece].get(square) {
                return Some(Piece::from_usize(piece).unwrap());
            }
        }
        return None;
    }

    pub fn color_on(&self, square: Square) -> Option<Color> {
        if self.color_combined(Color::White).get(square) {
            Some(Color::White)
        } else if self.color_combined(Color::Black).get(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn pieces(&self, piece: Piece) -> &BitBoard {
        &self.pieces[piece as usize]
    }

    pub fn color_combined(&self, color: Color) -> &BitBoard {
        &self.color_combined[color as usize]
    }

    pub fn color_combined_both(&self) -> BitBoard {
        self.color_combined(Color::White)
            .merge(self.color_combined(Color::Black))
    }

    pub fn from_fen(s: &str) -> Option<Board> {
        let mut board = Board::empty();

        let mut fen_split = s.split(' ');
        let mut board_split = fen_split.next()?.split('/');

        let mut rank_index = 8;
        while let Some(rank) = board_split.next() {
            rank_index -= 1;
            let mut file_index: u8 = 0;

            for piece_char in rank.chars() {
                if piece_char.is_numeric() {
                    file_index += piece_char.to_digit(10)? as u8;
                } else {
                    let piece = Piece::from_char(piece_char.to_ascii_lowercase())?;
                    let color = if piece_char.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let square = Square::new(rank_index, file_index);

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

        let en_passant = fen_split.next()?;
        if en_passant.len() == 2 {
            board.en_passant = Square::from_notation(&en_passant[0..2]);
        }

        Some(board)
    }

    pub fn from_start_pos() -> Board {
        Board::from_fen(STARTING_FEN).unwrap()
    }

    pub fn set_castling(&self, side: CastlingSide, can_castle: bool) -> Board {
        let mut board = self.clone();
        board.set_castling_mut(side, can_castle);
        board
    }

    pub fn set_castling_mut(&mut self, side: CastlingSide, can_castle: bool) {
        let side_bit = side as u8;
        if can_castle {
            self.castling |= 1 << side_bit;
        } else {
            self.castling &= !(1 << side_bit);
        }
    }

    pub fn make_move(&self, movement: &Movement) -> Board {
        let mut board = self.clone();
        board.make_move_mut(&movement);
        board
    }

    // TODO: Real error messages, not Option<T>
    pub fn make_move_mut(&mut self, movement: &Movement) -> Option<()> {
        let color = self.color_on(movement.from_square).unwrap();

        if self.color_combined(color).get(movement.to_square) {
            return None;
        }

        // Find the piece type
        let piece = self.piece_on(movement.from_square)?;

        // Move to the destination or promote
        if let Some(promoted_piece) = movement.promote {
            if !promoted_piece.can_promote_to() {
                return None;
            }
            self.replace_mut(promoted_piece, movement.to_square);
        } else {
            self.replace_mut(piece, movement.to_square);
        }

        // Remove the piece from it's old position
        self.pieces[piece as usize].flip_mut(movement.from_square);

        // Move the piece in the color grid
        self.color_combined[color as usize].flip_mut(movement.from_square);
        self.color_combined[color as usize].flip_mut(movement.to_square);

        // Store en passant passing square
        let is_double_move = if color == Color::White {
            movement.to_square.rank() - movement.from_square.rank() == 2
        } else {
            movement.from_square.rank() - movement.to_square.rank() == 2
        };

        if piece == Piece::Pawn && is_double_move {
            let passing_square = if color == Color::White {
                movement.to_square.down(1).unwrap()
            } else {
                movement.to_square.up(1).unwrap()
            };
            self.en_passant = Some(passing_square)
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

    fn sq(s: &str) -> Square {
        Square::from_notation(s).unwrap()
    }

    #[test]
    fn test_color_on() {
        let b = Board::from_start_pos();
        assert_eq!(b.color_on(sq("e2")), Some(Color::White));
        assert_eq!(b.color_on(sq("e7")), Some(Color::Black));
        assert_eq!(b.color_on(sq("d8")), Some(Color::Black));
        assert_eq!(b.color_on(sq("e3")), None);
    }

    #[test]
    fn test_piece_on() {
        let b = Board::from_start_pos();
        assert_eq!(b.piece_on(sq("e2")), Some(Piece::Pawn));
        assert_eq!(b.piece_on(sq("e7")), Some(Piece::Pawn));
        assert_eq!(b.piece_on(sq("d8")), Some(Piece::Queen));
        assert_eq!(b.piece_on(sq("a8")), Some(Piece::Rook));
        assert_eq!(b.piece_on(sq("a1")), Some(Piece::Rook));
        assert_eq!(b.piece_on(sq("e3")), None);
    }

    #[test]
    fn test_from_fen_starting() {
        let b = Board::from_start_pos();

        assert!(b.en_passant.is_none());
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::White);

        assert!(b.pieces(Piece::Rook).get(Square::new(0, 0)));
        assert!(b.pieces(Piece::Rook).get(Square::new(0, 7)));
        assert!(!b.pieces(Piece::Rook).get(Square::new(0, 1)));
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
        eprintln!("board:\n{}", b);

        // Just moved a pawn forward 2, so en_passant
        assert_eq!(b.en_passant, Some(Square::new(2, 4)));
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::Black);

        assert!(b.pieces(Piece::Pawn).get(Square::new(3, 4)));
    }

    #[test]
    fn test_fen_endgame() {
        let b =
            Board::from_fen("8/3k1p2/1R1p2P1/8/2P1N3/2Q1K3/8/8 w - - 0 1").expect("fen is valid");

        assert_eq!(b.en_passant, None);
        assert_eq!(b.castling, 0b0000);
        assert_eq!(b.side_to_move, Color::White);

        assert_eq!(b.piece_on(sq("e3")), Some(Piece::King));
        assert_eq!(b.piece_on(sq("f7")), Some(Piece::Pawn));
    }

    #[test]
    fn test_from_fen_invalid() {
        assert!(Board::from_fen("").is_none());
    }

    #[test]
    fn test_make_move_e2e4() {
        let mut b = Board::from_start_pos();

        assert!(b.en_passant.is_none());
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::White);

        b.make_move_mut(&Movement::from_notation("e2e4").expect("movement is valid"));
        b.assert_valid();

        // Just moved a pawn forward 2, so en_passant
        assert_eq!(b.en_passant, Some(Square::new(2, 4)));
        assert_eq!(b.castling, 0b1111);
        assert_eq!(b.side_to_move, Color::Black);

        assert!(b.pieces(Piece::Pawn).get(Square::new(3, 4)));
    }

    #[test]
    fn test_make_move_promote() {
        // Very common and realistic board position 11/10
        let mut b = Board::from_fen("1nbqkbnr/rP1ppppp/p1p5/8/8/8/1PPPPPPP/RNBQKBNR w KQk - 1 5")
            .expect("before promotion fen is valid");

        b.make_move_mut(&Movement::from_notation("b7c8q").expect("movement is valid"));
        b.assert_valid();

        let b7 = Square::new(6, 1);
        let c8 = Square::new(7, 2);

        assert!(!b.pieces(Piece::Pawn).get(b7));
        assert!(!b.pieces(Piece::Pawn).get(c8));
        assert!(b.pieces(Piece::Queen).get(c8));
    }

    #[test]
    fn test_make_move_capture() {
        let mut b = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
            .expect("fen is valid");
        eprintln!("{}", b);

        let movement = &Movement::from_notation("e4d5").unwrap();
        b.make_move_mut(movement).unwrap();
        b.assert_valid();

        assert_eq!(b.piece_on(movement.to_square), Some(Piece::Pawn));
        assert_eq!(b.piece_on(movement.from_square), None);
    }

    #[test]
    fn test_valid_after_capture() {
        let mut b = Board::from_fen("rnbqkbnr/ppp2ppp/8/3P4/8/2Np4/PP2PPPP/R1BQKBNR w KQkq - 0 1")
            .expect("fen is valid");

        let movement = &Movement::from_notation("e2d3").unwrap();
        b.make_move_mut(movement).unwrap();

        b.assert_valid();
    }
}
