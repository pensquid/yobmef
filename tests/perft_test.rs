use yobmef::{
    chess::Board,
    movegen::{gen_moves_once, perft},
};

fn test_perft(board: &Board, depth: u16, want: u64) {
    let got = perft(&board, depth);
    assert_eq!(
        got,
        want,
        "perft({}) = {} want {} off by {:+}",
        depth,
        got,
        want,
        (want as i64 - got as i64),
    );
}

// Could be made DRY using a macro

#[test]
fn test_perft_4() {
    gen_moves_once();
    test_perft(&Board::from_start_pos(), 4, 197281);
}

#[test]
fn test_perft_5() {
    gen_moves_once();
    test_perft(&Board::from_start_pos(), 5, 4865609);
}

// Takes ~1m to run. so ignore by default.
#[ignore]
#[test]
fn test_perft_6() {
    gen_moves_once();
    test_perft(&Board::from_start_pos(), 6, 119060324);
}

#[test]
fn test_perft_4_kiwipete() {
    gen_moves_once();
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    test_perft(&board, 4, 4085603);
}

#[test]
fn test_perft_5_kiwipete() {
    gen_moves_once();
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    test_perft(&board, 5, 193690690);
}
