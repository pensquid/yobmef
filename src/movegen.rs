use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Movement, Piece, Square};

static mut PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::empty(); 64];
// static mut PAWN_PUSHES: [BitBoard; 64] = [BitBoard::empty(); 64];

fn pawn_attacks(square: Square) -> BitBoard {
    unsafe { PAWN_ATTACKS[square.0 as usize] }
}

fn gen_pawn_moves() {
    for from_square_index in 0..64 {
        let from_square = Square(from_square_index);
        let mut pawn_attacks = BitBoard::empty();

        // Attacks
        from_square
            .up(1)
            .and_then(|s| s.left(1))
            .map(|s| pawn_attacks.flip_mut(s));
        from_square
            .up(1)
            .and_then(|s| s.right(1))
            .map(|s| pawn_attacks.flip_mut(s));

        unsafe {
            PAWN_ATTACKS[from_square_index as usize] = pawn_attacks;
        }
    }
}

pub fn gen_moves() {
    gen_pawn_moves();
}
// static mut KING_MOVES: [BitBoard; 64] = [BitBoard::empty(); 64];

// pub fn gen_king_moves() {
//     for from_square_index in 0..64 {
//         let from_square = Square(from_square_index);
//         let mut king_moves = BitBoard::empty();

//         from_square.up(1).map(|s| king_moves.flip_mut(s));
//         from_square
//             .up(1)
//             .and_then(|s| s.left(1))
//             .map(|s| king_moves.flip_mut(s));
//         from_square
//             .up(1)
//             .and_then(|s| s.right(1))
//             .map(|s| king_moves.flip_mut(s));

//         from_square.left(1).map(|s| king_moves.flip_mut(s));
//         from_square.right(1).map(|s| king_moves.flip_mut(s));

//         from_square.down(1).map(|s| king_moves.flip_mut(s));
//         from_square
//             .down(1)
//             .and_then(|s| s.left(1))
//             .map(|s| king_moves.flip_mut(s));
//         from_square
//             .down(1)
//             .and_then(|s| s.right(1))
//             .map(|s| king_moves.flip_mut(s));

//         unsafe {
//             KING_MOVES[from_square_index as usize] = king_moves;
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_gen_king_moves() {
//         gen_king_moves();
//     }
// }

fn bitboard_to_squares(bitboard: &BitBoard) -> Vec<Square> {
    (0..64)
        .filter_map(|i| {
            if bitboard.get(Square(i)) {
                Some(Square(i))
            } else {
                None
            }
        })
        .collect::<Vec<Square>>()
}

pub fn get_pawn_moves(board: &Board, moves: &mut Vec<Movement>) {
    let all_pieces = board
        .color_combined(Color::White)
        .combine(&board.color_combined(Color::Black))
        .flip_vertical();
    let my_pawns = board
        .pieces(Piece::Pawn)
        .mask(&board.color_combined(board.side_to_move))
        .flip_vertical();

    let mut their_pieces = board
        .color_combined(board.side_to_move.other())
        .flip_vertical();
    if let Some(sq) = board.en_passant {
        their_pieces.flip_mut(sq)
    }

    // Please have mercy uli I wrote this at midnight I'm tired
    bitboard_to_squares(&my_pawns)
        .iter()
        .for_each(|from_square| {
            let valid_attacks = pawn_attacks(*from_square).mask(&their_pieces);
            bitboard_to_squares(&valid_attacks).iter().for_each(|s| {
                if s.rank() == 7 {
                    moves.push(Movement::new(
                        from_square.flip_vertical(),
                        s.flip_vertical(),
                        Some(Piece::Knight),
                    ));
                    moves.push(Movement::new(
                        from_square.flip_vertical(),
                        s.flip_vertical(),
                        Some(Piece::Bishop),
                    ));
                    moves.push(Movement::new(
                        from_square.flip_vertical(),
                        s.flip_vertical(),
                        Some(Piece::Rook),
                    ));
                    moves.push(Movement::new(
                        from_square.flip_vertical(),
                        s.flip_vertical(),
                        Some(Piece::Queen),
                    ));
                } else {
                    moves.push(Movement::new(
                        from_square.flip_vertical(),
                        s.flip_vertical(),
                        None,
                    ));
                }
            });

            if !all_pieces.get(from_square.up(1).unwrap()) {
                moves.push(Movement::new(
                    from_square.flip_vertical(),
                    from_square.up(1).unwrap().flip_vertical(),
                    None,
                ));

                if from_square.rank() < 6 && !all_pieces.get(from_square.up(2).unwrap()) {
                    let s = from_square.up(2).unwrap();

                    if s.rank() == 7 {
                        moves.push(Movement::new(
                            from_square.flip_vertical(),
                            s.flip_vertical(),
                            Some(Piece::Knight),
                        ));
                        moves.push(Movement::new(
                            from_square.flip_vertical(),
                            s.flip_vertical(),
                            Some(Piece::Bishop),
                        ));
                        moves.push(Movement::new(
                            from_square.flip_vertical(),
                            s.flip_vertical(),
                            Some(Piece::Rook),
                        ));
                        moves.push(Movement::new(
                            from_square.flip_vertical(),
                            s.flip_vertical(),
                            Some(Piece::Queen),
                        ));
                    } else {
                        moves.push(Movement::new(
                            from_square.flip_vertical(),
                            s.flip_vertical(),
                            None,
                        ));
                    }
                }
            }
        });
}

pub fn get_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    get_pawn_moves(board, &mut moves);
    moves
}