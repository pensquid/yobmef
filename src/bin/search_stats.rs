use std::time::Instant;
use yobmef::chess::Board;
use yobmef::movegen::gen_moves_once;
use yobmef::search::Searcher;

fn main() {
    gen_moves_once();

    let board = Board::from_start_pos();

    for depth in 3..7 {
        eprintln!("depth: {}", depth);
        let mut searcher = Searcher::new();

        let start = Instant::now();
        searcher.search_depth(&board, depth);
        let took = Instant::now() - start;

        eprintln!(
            "cached {} pruned {} took {}ms\n",
            searcher.cached,
            searcher.pruned,
            took.as_millis()
        );
    }
}
