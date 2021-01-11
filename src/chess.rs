use crate::bitboard::BitBoard;

const NUM_COLORS: usize = 2;

#[derive(Debug)]
struct Board {
    pieces: [BitBoard; NUM_COLORS],
}
