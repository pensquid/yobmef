use crate::bitboard::BitBoard;
use crate::chess::{Board, Color, Movement, Piece, Square};
use crate::movegen::helpers::bitboard_to_squares;

static mut PAWN_ATTACKS: [BitBoard; 64] = [BitBoard::empty(); 64];

fn pawn_attacks(square: Square) -> BitBoard {
    unsafe { PAWN_ATTACKS[square.0 as usize] }
}

pub fn gen_pawn_moves() {
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

pub fn get_pawn_moves(board: &Board, moves: &mut Vec<Movement>) {
    let all_pieces = board
        .color_combined_both()
        .flip_vertical_if(board.side_to_move == Color::Black);
    let my_pawns = board
        .pieces(Piece::Pawn)
        .mask(&board.color_combined(board.side_to_move))
        .flip_vertical_if(board.side_to_move == Color::Black);

    let mut their_pieces = board
        .color_combined(board.side_to_move.other())
        .flip_vertical_if(board.side_to_move == Color::Black);
    if let Some(sq) = board.en_passant {
        their_pieces.flip_mut(sq);
    }

    // Please have mercy uli I wrote this at midnight I'm tired
    bitboard_to_squares(&my_pawns)
        .iter()
        .for_each(|from_square| {
            let valid_attacks = pawn_attacks(*from_square).mask(&their_pieces);
            bitboard_to_squares(&valid_attacks)
                .iter()
                .for_each(|to_square| {
                    let from_square =
                        from_square.flip_vertical_if(board.side_to_move == Color::Black);
                    let to_square = to_square.flip_vertical_if(board.side_to_move == Color::Black);

                    if to_square.rank() == 7 {
                        moves.push(Movement::new(from_square, to_square, Some(Piece::Knight)));
                        moves.push(Movement::new(from_square, to_square, Some(Piece::Bishop)));
                        moves.push(Movement::new(from_square, to_square, Some(Piece::Rook)));
                        moves.push(Movement::new(from_square, to_square, Some(Piece::Queen)));
                    } else {
                        moves.push(Movement::new(from_square, to_square, None));
                    }
                });

            if !all_pieces.get(from_square.up(1).unwrap()) {
                let to_square = from_square
                    .up(1)
                    .unwrap()
                    .flip_vertical_if(board.side_to_move == Color::Black);
                let from_square_f =
                    from_square.flip_vertical_if(board.side_to_move == Color::Black);

                if to_square.rank() == 7 {
                    moves.push(Movement::new(from_square_f, to_square, Some(Piece::Knight)));
                    moves.push(Movement::new(from_square_f, to_square, Some(Piece::Bishop)));
                    moves.push(Movement::new(from_square_f, to_square, Some(Piece::Rook)));
                    moves.push(Movement::new(from_square_f, to_square, Some(Piece::Queen)));
                } else {
                    moves.push(Movement::new(from_square_f, to_square, None));
                }

                if from_square.rank() == 1 && !all_pieces.get(from_square.up(2).unwrap()) {
                    let to_square = from_square
                        .up(2)
                        .unwrap()
                        .flip_vertical_if(board.side_to_move == Color::Black);

                    moves.push(Movement::new(from_square_f, to_square, None));
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::helpers::moves_test;

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
}
