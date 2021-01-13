pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

mod board;
mod color;
mod movement;
mod piece;
mod square;

pub use board::*;
pub use color::*;
pub use movement::*;
pub use piece::*;
pub use square::*;

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
        b.make_move_mut(movement).unwrap();
        b.assert_valid();

        assert_eq!(b.piece_on(movement.to_square), Some(Piece::Pawn));
        assert_eq!(b.piece_on(movement.from_square), None);
    }

    #[test]
    fn test_valid_after_capture() {
        let mut b =
            Board::from_fen("rnbqkbnr/ppp2ppp/8/3P4/8/2Np4/PP2PPPP/R1BQKBNR w KQkq - 0 1").unwrap();

        let movement = &Movement::from_notation("e2d3").unwrap();
        b.make_move_mut(movement).unwrap();

        b.assert_valid();
    }
}
