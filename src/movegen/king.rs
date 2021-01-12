static mut KING_MOVES: [BitBoard; 64] = [BitBoard::empty(); 64];

pub fn gen_king_moves() {
    for from_square_index in 0..64 {
        let from_square = Square(from_square_index);
        let mut king_moves = BitBoard::empty();

        from_square.up(1).map(|s| king_moves.flip_mut(s));
        from_square
            .up(1)
            .and_then(|s| s.left(1))
            .map(|s| king_moves.flip_mut(s));
        from_square
            .up(1)
            .and_then(|s| s.right(1))
            .map(|s| king_moves.flip_mut(s));

        from_square.left(1).map(|s| king_moves.flip_mut(s));
        from_square.right(1).map(|s| king_moves.flip_mut(s));

        from_square.down(1).map(|s| king_moves.flip_mut(s));
        from_square
            .down(1)
            .and_then(|s| s.left(1))
            .map(|s| king_moves.flip_mut(s));
        from_square
            .down(1)
            .and_then(|s| s.right(1))
            .map(|s| king_moves.flip_mut(s));

        unsafe {
            KING_MOVES[from_square_index as usize] = king_moves;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_king_moves() {
        gen_king_moves();
    }
}
