use yobmef::{
    chess::Board,
    movegen::{gen_moves_once, perft},
};

#[test]
fn test_perft() {
    gen_moves_once();

    // taken from stockfish 'go perft n' as a reference, also
    // the python-chess module perft agrees.
    assert_eq!(perft(&Board::from_start_pos(), 3), 8902);
    // assert_eq!(perft(&Board::from_start_pos(), 4), 197281);
    // TODO: Uncomment once perft(4) is correct
    // assert_eq!(perft(&Board::from_start_pos(), 5), 4865609);
}
