use yobmef::{
    chess::{Board, Movement},
    movegen::gen_moves_once,
    search::Searcher,
};

// TODO: Optional parameter for what range the evaluation should be in

macro_rules! test {
    (name: $name:ident, fen: $fen:expr, want: $want:expr,) => {
        #[test]
        fn $name() {
            gen_moves_once();

            let board = Board::from_fen($fen).expect("fen should be valid");
            let searcher = Searcher::new();
            let search_result = searcher.search(&board);
            let got = search_result.mv.unwrap();
            let want = Movement::from_notation($want).unwrap();
            assert_eq!(want, got, "want {} got {}", want, got);
        }
    };
}

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

// Does not pass, either I have a bug or vannila minimax is not strong enough
// to find it. alphabeta should though.
// test!(
//     name: mate_3_fishing_pole,
//     fen: "r1b1kb1r/pppp1pp1/2n5/1B2p3/4PPpq/8/PPPP2P1/RNBQNRK1 b kq f3 0 8",
//     want: "g4g3",
//     // eval: -eval::MATE, // TODO Add eval to test! macro
// );
