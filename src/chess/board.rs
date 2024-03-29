use crate::bitboard::BitBoard;
use crate::chess::*;
use crate::movegen;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub pieces: [BitBoard; NUM_PIECES],
    pub color_combined: [BitBoard; NUM_COLORS],
    pub en_passant: Option<Square>,
    pub side_to_move: Color,
    pub castling: u8, // 4 bits needed, from rtl: white kingside, white queenside, black kingside, black queenside
    pub attacked: [BitBoard; NUM_COLORS], // Colors white attacks, Colors black attacks.
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "side to move: {:?}", self.side_to_move)?;

        // Go by rank, printing each row
        for rank_index in 0..8 {
            let rank_index = 7 - rank_index;
            write!(f, "{}", rank_index + 1)?;

            for file_index in 0..8 {
                let sq = Square::new(rank_index, file_index);

                let character = match self.piece_on(sq) {
                    Some(piece) => piece.as_char_color(self.color_on(sq).unwrap()),
                    None => '.',
                };

                write!(f, " {}", character)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "  a b c d e f g h")?;
        writeln!(f, "fen: {}", self.to_fen())
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

    pub fn in_check(&self) -> bool {
        let attacked = self.attacked(self.side_to_move.other());

        let our_pieces = self.color_combined(self.side_to_move);
        let our_king = *self.pieces(Piece::King) & our_pieces;
        (our_king & attacked).0 != 0
    }

    pub fn is_capture(&self, mv: &Movement) -> bool {
        let to_square_bb = BitBoard::from_square(mv.to_square);
        let enemy_pieces = self.color_combined(self.side_to_move.other());
        to_square_bb & enemy_pieces != BitBoard::empty()
    }

    // TODO: Needed?
    pub fn other_side(&self) -> Self {
        let mut board = self.clone();
        board.side_to_move = board.side_to_move.other();
        board
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
        let bitboard = self.combined();

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
        let num_white_kings = (*kings & *self.color_combined(Color::White)).count_ones();
        assert_eq!(num_white_kings, 1, "{} white kings", num_white_kings);

        let num_black_kings = (*kings & *self.color_combined(Color::Black)).count_ones();
        assert_eq!(num_black_kings, 1, "{} black kings", num_black_kings);
    }

    pub fn empty() -> Board {
        Board {
            pieces: [BitBoard(0); NUM_PIECES],
            color_combined: [BitBoard(0); NUM_COLORS],
            en_passant: None,
            castling: 0b1111,
            side_to_move: Color::White,
            attacked: [BitBoard(0); NUM_COLORS],
        }
    }

    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        for piece in 0..NUM_PIECES {
            if self.pieces[piece].get(square) {
                return Some(Piece::from_usize(piece).unwrap());
            }
        }
        None
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

    pub fn combined(&self) -> BitBoard {
        *self.color_combined(Color::White) | *self.color_combined(Color::Black)
    }

    pub fn attacked(&self, color: Color) -> BitBoard {
        self.attacked[color as usize]
    }

    pub fn update_attackers(&mut self) {
        self.attacked[Color::White as usize] = movegen::get_attacked_squares(self, Color::White);
        self.attacked[Color::Black as usize] = movegen::get_attacked_squares(self, Color::Black);
    }

    pub fn from_fen(s: &str) -> Option<Board> {
        // In release mode, checking a sync.Once all the time is cringe
        if cfg!(test) {
            movegen::gen_moves_once();
        }

        let mut board = Board::empty();

        let mut fen_split = s.split(' ');
        let board_split = fen_split.next()?.split('/');

        let mut rank_index = 8;
        for rank in board_split {
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
                    let sq = Square::new(rank_index, file_index);

                    board.pieces[piece as usize].flip_mut(sq);
                    board.color_combined[color as usize].flip_mut(sq);
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

        board.update_attackers();
        Some(board)
    }

    // TODO: Clean up, this is awful code.
    pub fn to_fen(&self) -> String {
        let mut buf = String::new();

        for rank in 0..8 {
            // Fen starts at the top (black side)
            let rank = 7 - rank;

            let mut since_last_piece = 0;
            for file in 0..8 {
                let sq = Square::new(rank, file);
                if let Some(piece) = self.piece_on(sq) {
                    if since_last_piece != 0 {
                        buf.push_str(&since_last_piece.to_string());
                        since_last_piece = 0;
                    }
                    buf.push(piece.as_char_color(self.color_on(sq).unwrap()));
                } else {
                    since_last_piece += 1;
                }
            }
            if since_last_piece != 0 {
                buf.push_str(&since_last_piece.to_string());
            }

            buf.push('/');
        }
        buf.pop();
        buf.push(' ');
        buf.push(self.side_to_move.as_char());
        buf.push(' ');

        // tfw tenary is too verbose for you
        macro_rules! kk {
            ($side:ident, $character:expr) => {
                if self.can_castle_unchecked(CastlingSide::$side) {
                    buf.push($character);
                }
            };
        }
        if self.castling != 0 {
            kk!(WhiteKingside, 'K');
            kk!(WhiteQueenside, 'Q');
            kk!(BlackKingside, 'k');
            kk!(BlackQueenside, 'q');
        } else {
            buf.push('-');
        }

        // En-pasasnt
        buf.push(' ');
        buf.push_str(
            &self
                .en_passant
                .map(|sq| sq.to_notation())
                .unwrap_or("-".to_string()),
        );

        // TODO: Halfmove clock, Fullmove number
        buf.push_str(" 0 1");

        buf
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

    // Checks for castling privileges but doesn't check square occupancy
    pub fn can_castle_unchecked(&self, side: CastlingSide) -> bool {
        let side_bit = side as u8;
        (self.castling >> side_bit) & 1 == 1
    }

    pub fn make_move(&self, movement: &Movement) -> Board {
        let mut board = self.clone();
        board.make_move_mut(movement);
        board
    }

    // This function WILL break if passed invalid moves
    pub fn make_move_mut(&mut self, movement: &Movement) {
        // TODO: Clean up using math instead of tenary conditionals.

        let color = self
            .color_on(movement.from_square)
            .expect("no color on square");

        // Find the piece type
        let piece = self
            .piece_on(movement.from_square)
            .expect("no piece on square");

        // Piece specific logic
        match piece {
            Piece::King => {
                let castling = CastlingSide::from_movement(movement);
                if let Some(castling) = castling {
                    debug_assert!(
                        self.can_castle_unchecked(castling),
                        "tried to castle ({:?}) but cannot castle",
                        castling,
                    );
                    self.make_move_mut(&castling.get_rook_movement());

                    // self.make_move_mut changed the side to move, change it back!
                    // TODO: We should really inline this, or inline a call to a helper.
                    // Recursion here is not ideal for maintainability.
                    self.side_to_move = self.side_to_move.other();
                }

                // No matter what king move, we can no longer castle.
                // TODO: Optimize using bitwise operations
                CastlingSide::of_color(color)
                    .iter()
                    .for_each(|side| self.set_castling_mut(*side, false));
            }

            Piece::Rook => {
                // You can no longer castle on this side after moving your rook.
                if let Some(side) = CastlingSide::from_rook_square(movement.from_square) {
                    self.set_castling_mut(side, false)
                }
            }

            Piece::Pawn => {
                if self.en_passant == Some(movement.to_square) {
                    // Remove the captured pawn
                    self.remove_mut(if self.side_to_move == Color::White {
                        movement.to_square.down(1).unwrap()
                    } else {
                        movement.to_square.up(1).unwrap()
                    });
                }
            }

            _ => {}
        }

        // Store en passant passing square
        let is_double_move = piece == Piece::Pawn && i8::abs(movement.vdelta()) == 2;

        if is_double_move {
            let passing_square = if color == Color::White {
                movement.to_square.down(1).unwrap()
            } else {
                movement.to_square.up(1).unwrap()
            };
            self.en_passant = Some(passing_square)
        } else {
            // If a move is made which does not create an en_passant square,
            // we remove the en_passant square (en_passant is valid for one move only)
            self.en_passant = None;
        }

        // NOTE: Not checking rank is ok! this function is undefined for invalid moves. <o/
        if let Some(promotion) = movement.promote {
            self.replace_mut(promotion, movement.to_square);
        } else {
            self.replace_mut(piece, movement.to_square);
        }

        // Piece independent logic

        // Remove the piece from its old position
        self.pieces[piece as usize].flip_mut(movement.from_square);

        // Move the piece in the color grid
        self.color_combined[color as usize].flip_mut(movement.from_square);
        self.color_combined[color as usize].flip_mut(movement.to_square);

        // Switch side to move
        self.side_to_move = self.side_to_move.other();

        // Update attackers (todo: inline for speed)
        self.update_attackers();
    }

    // TODO: Test
    pub fn king(&self, color: Color) -> Square {
        let king_bb = self.pieces[Piece::King as usize] & self.color_combined[color as usize];

        if king_bb.0.trailing_zeros() >= 64 {
            debug_assert!(king_bb.0.trailing_zeros() < 64);
        }

        Square(king_bb.0.trailing_zeros() as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::from_notation(s).unwrap()
    }

    #[test]
    fn test_get_king_square() {
        let board = Board::from_fen("8/5k2/8/2K5/8/8/8/8 w - - 0 1").unwrap();
        let white_king = Square::from_notation("c5").unwrap();
        let black_king = Square::from_notation("f7").unwrap();
        assert_eq!(board.king(Color::White), white_king);
        assert_eq!(board.king(Color::Black), black_king);
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
        let b = Board::from_fen("8/3k1p2/1R1p2P1/8/2P1N3/2Q1K3/8/8 w - - 0 1").unwrap();

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
            .unwrap();
        eprintln!("{}", b);

        let movement = &Movement::from_notation("e4d5").unwrap();
        b.make_move_mut(movement);
        b.assert_valid();

        assert_eq!(b.piece_on(movement.to_square), Some(Piece::Pawn));
        assert_eq!(b.piece_on(movement.from_square), None);
    }

    #[test]
    fn test_valid_after_capture() {
        let mut b =
            Board::from_fen("rnbqkbnr/ppp2ppp/8/3P4/8/2Np4/PP2PPPP/R1BQKBNR w KQkq - 0 1").unwrap();

        let movement = &Movement::from_notation("e2d3").unwrap();
        b.make_move_mut(movement);

        b.assert_valid();
    }

    #[test]
    fn test_is_in_check() {
        let board = Board::from_fen("k1R5/8/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(board.in_check(), "black should be in check");
    }

    #[test]
    fn test_make_move_castle() {
        let mut board =
            Board::from_fen("rnbqk1nr/ppp2ppp/3b4/3p4/8/3B1N2/PPPP1PPP/RNBQK2R w KQkq - 2 5")
                .unwrap();
        board.make_move_mut(&Movement::from_notation("e1g1").unwrap());

        assert_eq!(board.piece_on(Square::from_notation("e1").unwrap()), None);
        assert_eq!(
            board.piece_on(Square::from_notation("f1").unwrap()),
            Some(Piece::Rook)
        );
        assert_eq!(
            board.piece_on(Square::from_notation("g1").unwrap()),
            Some(Piece::King)
        );
        assert_eq!(board.piece_on(Square::from_notation("h1").unwrap()), None);

        assert!(!board.can_castle_unchecked(CastlingSide::WhiteKingside));
        assert!(!board.can_castle_unchecked(CastlingSide::WhiteQueenside));
    }

    #[test]
    fn test_make_move_remove_castling() {
        let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

        board.make_move_mut(&Movement::from_notation("a1a2").unwrap());
        assert!(!board.can_castle_unchecked(CastlingSide::WhiteQueenside));
        assert!(board.can_castle_unchecked(CastlingSide::WhiteKingside));

        board.make_move_mut(&Movement::from_notation("h8h7").unwrap());
        assert!(!board.can_castle_unchecked(CastlingSide::BlackKingside));
        assert!(board.can_castle_unchecked(CastlingSide::BlackQueenside));

        board.make_move_mut(&Movement::from_notation("e1f2").unwrap());
        assert!(!board.can_castle_unchecked(CastlingSide::WhiteKingside));
        assert!(!board.can_castle_unchecked(CastlingSide::WhiteQueenside));

        board.make_move_mut(&Movement::from_notation("e8d7").unwrap());
        assert!(!board.can_castle_unchecked(CastlingSide::BlackKingside));
        assert!(!board.can_castle_unchecked(CastlingSide::BlackQueenside));
    }

    #[test]
    fn test_make_move_bishop_en_passant() {
        let mut board = Board::from_start_pos();

        board.make_move_mut(&Movement::from_notation("e2e4").unwrap());
        board.make_move_mut(&Movement::from_notation("a7a5").unwrap());
        board.make_move_mut(&Movement::from_notation("f1a6").unwrap());

        let on_a5 = board.piece_on(Square::from_notation("a5").unwrap());
        let on_a6 = board.piece_on(Square::from_notation("a6").unwrap());

        assert_eq!(on_a5, Some(Piece::Pawn));
        assert_eq!(on_a6, Some(Piece::Bishop));
    }

    #[test]
    fn test_make_move_en_passant_cleared() {
        // en-passant should be cleared every move.
        let mut board = Board::from_start_pos();
        board.make_move_mut(&Movement::from_notation("e2e4").unwrap());
        assert_eq!(board.en_passant, Some(Square::from_notation("e3").unwrap()));

        // Knight move is a better test if the pawn function clears en-passant,
        // it would hide on another pawn move.
        board.make_move_mut(&Movement::from_notation("g8f6").unwrap());
        assert_eq!(board.en_passant, None);
    }

    #[test]
    fn test_to_fen_startpos() {
        let board = Board::from_start_pos();
        assert_eq!(board.to_fen(), STARTING_FEN);
    }

    #[test]
    fn test_to_fen_e2e4() {
        let mut board = Board::from_start_pos();
        board.make_move_mut(&Movement::from_notation("e2e4").unwrap());
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    macro_rules! test_to_fen {
        ($name:ident, $fen:expr) => {
            #[test]
            fn $name() {
                let fen = $fen;
                let board = Board::from_fen(fen).unwrap();
                assert_eq!(board.to_fen(), fen);
            }
        };
    }
    test_to_fen!(
        black_castling,
        "r1bqkb1r/1ppn2pp/p3p3/5p2/P1BPn3/4PN2/1PQ2PPP/RNB2RK1 w kq - 0 1"
    );

    test_to_fen!(
        en_passant,
        "rnbqkbnr/ppp3pp/4p3/3pPp2/8/1P6/P1PP1PPP/RNBQKBNR w KQkq f6 0 1"
    );

    test_to_fen!(
        no_castling,
        "1rbq1rk1/ppbn1pp1/4p2p/1P1pP3/3P2P1/PQN1BN2/1K3P1P/3R3R w - - 0 1"
    );
}
