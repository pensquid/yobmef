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

fn get_pawn_moves(board: &Board, moves: &mut Vec<Movement>) {
    let all_pieces = board
        .color_combined(Color::White)
        .combine(&board.color_combined(Color::Black))
        .orient(board.side_to_move);
    let my_pawns = board
        .pieces(Piece::Pawn)
        .mask(&board.color_combined(board.side_to_move))
        .orient(board.side_to_move);

    let mut their_pieces = board
        .color_combined(board.side_to_move.other())
        .orient(board.side_to_move);
    if let Some(sq) = board.en_passant {
        their_pieces.flip_mut(sq);
    }

    // Please have mercy uli I wrote this at midnight I'm tired
    bitboard_to_squares(&my_pawns)
        .iter()
        .for_each(|from_square| {
            let valid_attacks = pawn_attacks(*from_square).mask(&their_pieces);
            bitboard_to_squares(&valid_attacks).iter().for_each(|s| {
                if s.rank() == 7 {
                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        Some(Piece::Knight),
                    ));
                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        Some(Piece::Bishop),
                    ));
                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        Some(Piece::Rook),
                    ));
                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        Some(Piece::Queen),
                    ));
                } else {
                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        None,
                    ));
                }
            });

            if !all_pieces.get(from_square.up(1).unwrap()) {
                moves.push(Movement::new(
                    from_square.orient(board.side_to_move),
                    from_square.up(1).unwrap().orient(board.side_to_move),
                    None,
                ));

                if from_square.rank() == 1 && !all_pieces.get(from_square.up(2).unwrap()) {
                    let s = from_square.up(2).unwrap();

                    moves.push(Movement::new(
                        from_square.orient(board.side_to_move),
                        s.orient(board.side_to_move),
                        None,
                    ));
                }
            }
        });
}

pub fn get_moves(board: &Board) -> Vec<Movement> {
    let mut moves = Vec::new();
    get_pawn_moves(board, &mut moves);
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(lan: &str) -> Movement {
        Movement::from_notation(lan).unwrap()
    }

    // TODO: pretty-print board on error
    fn moves_test(board: &Board, legal: &str, illegal: &str) {
        let moves = get_moves(&board);

        for lan in legal.split(' ') {
            assert!(
                moves.contains(&mv(lan)),
                format!("{} should be legal, but it isn't", lan),
            );
        }

        for lan in illegal.split(' ') {
            assert!(
                !moves.contains(&mv(lan)),
                format!("{} should not be legal, but it is", lan),
            );
        }
    }

    #[test]
    fn test_get_pawn_moves_startpos() {
        let mut board = Board::from_start_pos();

        moves_test(&board, "e2e4 d2d3", "e2d3 e2e5 e7e5 d7d6");

        board.side_to_move = Color::Black;
        moves_test(&board, "e7e5 d7d6", "e2e5 e2e4 d2d3");
    }

    #[test]
    fn test_get_pawn_moves_endgame() {
        let board = Board::from_fen("8/3k1p2/1R1p2P1/8/2P1N3/2Q1K3/8/8 w - - 0 1").unwrap();

        moves_test(&board, "c4c5 g6g7 g6f7", "f7f6 g6h7");
    }

    // #[test]
    // fn test_get_king_moves_endgame() {
    //     let board = Board::from_fen("8/3k1p2/1R1p4/6P1/2P1N3/2Q1K3/8/8 w - - 0 1").unwrap();
    //     moves_test(&board, "e3e2 e3f3 e3f2", "e3e4 e3f5");
    // }
}
