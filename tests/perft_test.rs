use yobmef::{
    chess::Board,
    movegen::{gen_moves_once, perft},
};

fn test_perft(depth: u16, want: u64) {
    gen_moves_once();
    let got = perft(&Board::from_start_pos(), depth);
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
fn test_perft_1() {
    test_perft(1, 20);
}

#[test]
fn test_perft_2() {
    test_perft(2, 400);
}

#[test]
fn test_perft_3() {
    test_perft(3, 8902);
}

#[test]
fn test_perft_4() {
    test_perft(4, 197281);
}

#[test]
fn test_perft_5() {
    test_perft(5, 4865609);
}

// Takes ~1m to run. also fails, so ignore by default.
#[ignore]
#[test]
fn test_perft_6() {
    test_perft(6, 119060324);
}
