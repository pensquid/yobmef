use criterion::{Criterion, black_box, criterion_group, criterion_main};
use yobmef::{chess::Board, eval::get_score_ongoing, movegen::gen_moves_once, search::get_sorted_moves};

fn criterion_benchmark(c: &mut Criterion) {
    let board = Board::from_fen("r2qr1k1/p1p1bppp/2p2n2/8/4N3/2P2P2/PP4PP/R1BQR1K1 b - - 2 14").unwrap();
    gen_moves_once();

    c.bench_function(
        "get_sorted_moves",
        |b| b.iter(|| get_sorted_moves(black_box(&board)))
    );

    c.bench_function(
        "get_score_ongoing",
        |b| b.iter(|| get_score_ongoing(black_box(&board)))
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);