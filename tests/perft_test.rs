use yobmef::{
    chess::{Board, Movement},
    movegen::{gen_moves_once, get_legal_moves},
};

fn perft(board: &Board, depth: u16) -> u64 {
    if depth == 0 {
        return 1;
    } else {
        let mut n = 0;
        for mv in get_legal_moves(board) {
            n += perft(&board.make_move(&mv), depth - 1);
        }
        return n;
    }
}

#[test]
fn test_perft() {
    gen_moves_once();

    // taken from stockfish 'go perft n' as a reference, also
    // the python-chess module perft agrees.
    assert_eq!(perft(&Board::from_start_pos(), 3), 8902);
    assert_eq!(perft(&Board::from_start_pos(), 4), 197281);
    // TODO: Uncomment once perft(4) is correct
    // assert_eq!(perft(&Board::from_start_pos(), 5), 4865609);
}
