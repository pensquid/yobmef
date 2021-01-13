use crate::bitboard::BitBoard;
use crate::chess::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CastlingSide {
    WhiteKingside = 0,
    WhiteQueenside = 1,
    BlackKingside = 2,
    BlackQueenside = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub pieces: [BitBoard; NUM_PIECES],
    pub color_combined: [BitBoard; NUM_COLORS],
    pub en_passant: Option<Square>,
    pub side_to_move: Color,
    pub castling: u8, // 4 bits needed, from rtl: white kingside, white queenside, black kingside, black queenside
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
    // Replace old piece with new piece, return old piece,
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

    // Same as replace_mut but removes the piece at the square
    pub fn remove_mut(&mut self, square: Square) -> Option<Piece> {
        let old_piece = self.piece_on(square);
        if let Some(old_piece) = old_piece {
            self.pieces[old_piece as usize].flip_mut(square);
            self.color_combined[self.side_to_move.other() as usize].flip_mut(square);
        }

        old_piece
    }

    pub fn assert_valid(&self) {
        let bitboard = self.color_combined_both();

        for sq in 0..64 {
            let sq = Square(sq);

            if bitboard.get(sq) {
                // Multiple pieces on the same square
                let num_on_square: u8 = (0..NUM_PIECES).map(|p| self.pieces[p].get(sq) as u8).sum();
                assert_eq!(num_on_square, 1, "multiple pieces on {}", sq);

                // Multiple ownership
                let num_owners: u8 = (0..NUM_COLORS)
                    .map(|c| self.color_combined[c].get(sq) as u8)
                    .sum();
                assert_eq!(num_owners, 1, "{} owners of square {}", num_owners, sq);
            } else {
                for piece in 0..NUM_PIECES {
                    if self.pieces[piece].get(sq) {
                        panic!(
                            "piece {:?} owns square {} but isn't owned by color",
                            Piece::from_usize(piece),
                            sq
                        );
                    }
                }
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
        } else if self.en_passant == Some(movement.to_square) {
            self.replace_mut(piece, movement.to_square);
            self.remove_mut(if self.side_to_move == Color::White {
                movement.to_square.down(1).unwrap()
            } else {
                movement.to_square.up(1).unwrap()
            });
        } else {
            self.replace_mut(piece, movement.to_square);
        }

        // Remove the piece from it's old position
        self.pieces[piece as usize].flip_mut(movement.from_square);

        // Move the piece in the color grid
        self.color_combined[color as usize].flip_mut(movement.from_square);
        self.color_combined[color as usize].flip_mut(movement.to_square);

        // Store en passant passing square
        let is_double_move = piece == Piece::Pawn
            && if color == Color::White {
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
