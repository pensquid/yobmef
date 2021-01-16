pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

mod board;
mod castling_side;
mod color;
mod movement;
mod piece;
mod square;

pub use board::*;
pub use castling_side::*;
pub use color::*;
pub use movement::*;
pub use piece::*;
pub use square::*;
