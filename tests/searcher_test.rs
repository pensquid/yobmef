use yobmef::{
    chess::{Board, Movement},
    movegen::gen_moves_once,
    search::Searcher,
};

// TODO: Optional parameter for what range the evaluation should be in
// TODO: Remove duplication
// TODO: Test principle variation

macro_rules! test {
    (name: $name:ident, fen: $fen:expr, want: $want:expr,) => {
        #[test]
        fn $name() {
            gen_moves_once();
            let board = Board::from_fen($fen).expect("fen should be valid");
            let mut searcher = Searcher::new();
            let search_result = searcher.search_depth(&board, 5);
            let got = search_result.mv;
            let want = Movement::from_notation($want).unwrap();
            eprintln!("bestmove {} eval {}", got, search_result.eval);
            assert_eq!(want, got, "want {} got {}", want, got);
        }
    };

    (name: $name:ident, fen: $fen:expr, not: $not:expr,) => {
        #[test]
        fn $name() {
            gen_moves_once();
            let board = Board::from_fen($fen).expect("fen should be valid");
            let mut searcher = Searcher::new();
            let search_result = searcher.search_depth(&board, 5);
            let got = search_result.mv;
            let not = Movement::from_notation($not).unwrap();
            eprintln!("bestmove {} eval {}", got, search_result.eval);
            assert_ne!(not, got, "got {} want something else", got);
        }
    };
}

// todo: not test (don't play this move), for mistakes it has made in the past.
test!(
    name: queen_blunder_check,
    fen: "r1b1k2r/pppp1ppp/4p3/8/2nP4/2B3P1/P1P1KP1P/RQ5q b kq - 0 1",
    not: "h1f3",
);

test!(
    name: free_queen,
    fen: "r1b1k2r/pppp1ppp/4p3/8/2nP4/2B2qP1/P1P1KP1P/RQ6 w kq - 1 2",
    want: "e2f3",
);

test!(
    name: mate_1_white,
    fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
    want: "h5f7",
);

test!(
    name: mate_1_black,
    fen: "rnb1k1nr/pppp1ppp/8/2b1p3/2B1P2q/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 5 4",
    want: "h4f2",
);

// Engine did not care how long mate took, so it was not playing
// mate as long as it had a forcing line.
test!(
    name: mate_1_black_does_not_care,
    fen: "2r1kb1r/pp1npppp/8/7K/3q4/7P/P7/8 b k - 0 1",
    want: "c8c5",
);

test!(
    name: free_knight_black,
    fen: "r1bqkbnr/pppp1ppp/2n5/4N3/4P3/8/PPPP1PPP/RNBQKB1R b KQkq - 0 3",
    want: "c6e5",
);

test!(
    name: free_knight_white,
    fen: "r1bqkb1r/pppp1ppp/2n5/1B2p3/4n3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 5",
    want: "c3e4",
);

test!(
    name: lichess_mate_2,
    fen: "8/5Q1p/p2N2p1/5p1k/4p3/4P3/PP1pqPPP/5RK1 b - - 4 40",
    want: "e2f1",
);

test!(
    name: knight_fork_white,
    fen: "8/3k4/1p4r1/8/2N5/8/8/K7 w - - 0 1",
    want: "c4e5",
);

test! (
    name: knight_fork_black,
    fen: "8/3K4/1P4Q1/8/2n5/8/8/k7 b - - 0 1",
    want: "c4e5",
);

test!(
    name: mate_3_fishing_pole,
    fen: "r1b1kb1r/pppp1pp1/2n5/1B2p3/4PPpq/8/PPPP2P1/RNBQNRK1 b kq f3 0 8",
    want: "g4g3",
);

test!(
    name: mate_4_fishing_pole,
    fen: "r1bqkb1r/pppp1pp1/2n5/1B2p3/4P1p1/8/PPPP1PP1/RNBQNRK1 b kq - 1 7",
    want: "d8h4",
);

test!(
    name: win_exchange_in_2, // 3 ply
    fen: "2r3k1/1p3ppp/1qnBb3/2RpPp2/3P4/rP2QN2/5PPP/1R4K1 w - - 0 1",
    want: "c5c6",
);

test!(
    name: trap_queen_2, // 3 ply
    fen: "r1b2r1k/1p2Npbp/p2p2p1/2n5/3N1P2/4B2P/qPQ3P1/2R2RK1 w - - 0 1",
    want: "c1a1",
);

test!(
    name: fastest_mate_regression,
    fen: "6k1/2r1Q1pp/pp2p3/8/3N2R1/6P1/qPP2P1P/5RK1 w - - 0 1",
    want: "e7e8",
);

test!(
    name: save_bishop,
    fen: "r2qkbnr/3bpppp/p1np4/1p6/2p1P3/1BP2N2/PP1P1PPP/RNBQ1RK1 w kq - 0 1",
    want: "b3c2",
);

test!(
    name: win_rook_in_4, // 8 ply
    fen: "2br2k1/4pp1p/6pB/8/8/2q2P2/P1PrQ1PP/1R1R2K1 b - - 0 1",
    want: "c3d4",
);

test!(
    name: useless_king_move,
    fen: "5rk1/p1p1p2p/4bp2/2n5/8/1PqBb1P1/P1PNP2P/1R1QK2R b K - 4 18",
    not: "g8h8",
);

test!(
    name: mate_in_4, // 8 ply
    fen: "2q4k/1p4pp/7r/pP2B3/P3P1P1/1QP2pRn/5P1K/3R4 b - - 0 30",
    want: "h3f4",
);

// test!(
//     name: free_knight_black_2,
//     fen: "r2q1rk1/p1pp1ppp/1p6/n2b4/1P3Q2/5N2/P3PPPP/R3KB1R w KQ - 0 14",
//     want: "b4a5",
// );

test!(
    name: avoid_stalemate_winning,
    fen: "8/8/4Nk2/3KN3/3N4/8/8/8 w - - 4 3",
    not: "d5d6",
);
