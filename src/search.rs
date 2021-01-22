use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen::MoveGen;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchResult {
    pub eval: i16,    // Evaluation for the position
    pub mv: Movement, // The best move

    // Depth of this evaluation, with respect to the root node.
    pub depth: i16,
}

// A handle on a running search.
pub struct SearcherHandle {
    // stop the search?
    stop: Arc<AtomicBool>,

    jh: JoinHandle<(Searcher, SearchResult)>,
}

impl SearcherHandle {
    // Stop the thread, then wait for it to exit.
    pub fn stop_join(self) -> (Searcher, SearchResult) {
        self.stop();
        // only fails if searcher thread panics, which it shoulden't
        self.jh.join().unwrap()
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct Searcher {
    // Transposition table
    // TODO: Store PV and use as move guesses for a/b search
    pub tp: HashMap<Board, SearchResult>,
    pub tp_max_len: usize,

    // Search statistics
    pub nodes: u64, // including qs!
    pub cached: u64,

    // Used so I don't pass fucking everything as a parameter to alphabeta
    pub start_depth: i16, // start depth of this ID iteration
}

// Sorting is very important for alpha beta search pruning
pub fn sort_by_promise(board: &Board, moves: &mut Vec<Movement>) {
    let is_game_over = moves.len() == 0;

    moves.sort_by_cached_key(|m| eval::get_score(&board.make_move(m), is_game_over));
    if board.side_to_move == Color::White {
        moves.reverse()
    };
}

// TODO: Move this to movement?
fn moves_to_str(moves: &Vec<Movement>) -> String {
    moves
        .iter()
        .map(|mv| mv.to_notation())
        .collect::<Vec<String>>()
        .join(" ")
}

impl Searcher {
    pub fn new() -> Self {
        let mut s = Searcher {
            nodes: 0,
            cached: 0,
            tp: HashMap::new(),
            tp_max_len: 0,
            start_depth: 0,
        };

        // default to a 64mb hashtable (small)
        s.set_hash_size(64);

        return s;
    }

    pub fn set_hash_size(&mut self, mb: usize) {
        use std::mem;
        self.tp_max_len = (1024 * 1024 * mb) / mem::size_of::<Board>();
    }

    pub fn start_search(mut self, board: Board) -> SearcherHandle {
        // FIXME: Ugly and redundant.
        let stop_th = Arc::new(AtomicBool::new(false));
        let stop = stop_th.clone();

        let jh = thread::spawn(move || {
            let sr = self.search(&board, |_| stop_th.load(Ordering::Relaxed));
            return (self, sr);
        });

        SearcherHandle { jh, stop }
    }

    pub fn search_depth(&mut self, board: &Board, depth: i16) -> SearchResult {
        self.search(board, |sr| sr.depth >= depth)
    }

    pub fn search_timed(&mut self, board: &Board, thinking_time: Duration) -> SearchResult {
        let start = Instant::now();
        self.search(board, |_sr| {
            return start.elapsed() > thinking_time;
        })
    }

    pub fn search<F>(&mut self, board: &Board, quit: F) -> SearchResult
    where
        F: Fn(&SearchResult) -> bool,
    {
        self.reset_stats();

        // so we don't use infinite memory
        // TODO: Better replacement strategies
        // https://www.chessprogramming.org/Transposition_Table#Replacement_Strategies
        // NOTE: Tests rely on TP being available after search to verify PV.
        // FIXME: max size can be exeeded during iterative deepening or alphabeta!
        if self.tp.len() > self.tp_max_len {
            eprintln!("hash clear, len {} max {}", self.tp.len(), self.tp_max_len);
            self.tp.clear();
        }

        let mut depth = 1;
        let start = Instant::now();
        loop {
            self.start_depth = depth;

            let score = self.alphabeta(board, depth, i16::MIN, i16::MAX);
            let nps = (self.nodes as f64 / start.elapsed().as_secs_f64()) as u64;
            let pv = self.get_pv(board);

            println!(
                "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                depth,
                score,
                self.nodes,
                nps,
                start.elapsed().as_millis(),
                moves_to_str(&pv),
            );

            let sr = SearchResult {
                eval: score,
                mv: pv.get(0).expect("pv empty, no legal moves?").clone(),
                depth: depth,
            };

            // Bound ply because of possible recursion limit in endgames.
            if quit(&sr) || depth >= 1000 {
                println!("bestmove {}", sr.mv);
                return sr;
            }
            depth += 1;
        }
    }

    // TODO: Perhaps keep pv state and update from alphabeta?
    // Need to see how stockfish does it.
    // NOTE: If we aren't careful, transpositions will cause an infinite loop.
    fn get_pv(&self, board: &Board) -> Vec<Movement> {
        use std::collections::HashSet;

        let mut moves = Vec::new();
        let mut curr = board.clone();
        let mut seen = HashSet::new();

        while let Some(mv) = self.get_pv_next(&curr) {
            curr.make_move_mut(&mv);
            if seen.contains(&curr) {
                // eprintln!("transposition!\n{}\nlastmove: {}", curr, mv);
                break;
            }
            seen.insert(curr.clone());
            moves.push(mv.clone());
        }

        moves
    }

    fn reset_stats(&mut self) {
        self.nodes = 0;
        self.cached = 0;
    }

    // Get the next PV move
    // NOTE: This assumes the TP will always hold the deepest search for a given board.
    // TODO: Remove this function?
    fn get_pv_next(&self, board: &Board) -> Option<&Movement> {
        Some(&self.tp.get(board)?.mv)
    }

    // alphabeta search in a negamax framework.
    // 'alpha' is always our best score,
    // 'beta'
    pub fn alphabeta(
        &mut self,
        board: &Board,
        mut depth: i16,
        mut alpha: i16,
        mut beta: i16,
    ) -> i16 {
        self.nodes += 1;

        if let Some(sr) = self.tp.get(board) {
            if sr.depth >= depth {
                self.cached += 1;
                return sr.eval;
            }

            // TODO: Use sr as guess for the best move,
            // even if the depth is not sufficent to return it immediately.
        }

        let mut moves: Vec<Movement> = MoveGen::new_legal(board).collect();
        let is_game_over = moves.len() == 0;

        // NOTE: We don't store the static eval in the TP table, because we aren't whores.
        if is_game_over {
            // Easier to inline instead of calling `eval::get_score`
            // and then have to check if it returned eval::MATE.
            let score = if board.in_check() {
                (depth + eval::MATE) * board.side_to_move.other().polarize()
            } else {
                0
            };

            return score;
        }

        // So simple, yet so effective!
        if board.in_check() {
            depth += 1;
        }

        if depth < 0 {
            // Quiet search!
            let score = eval::get_score(board, is_game_over);

            // It is our move, so if the static score is already better then
            // Our previous best score, we can just return the static eval.
            // TODO: Refactor into a negamax framework to cleanup this code.
            // FIXME: If we're in zugzwang, then this will prematurely prune.
            match board.side_to_move {
                Color::White => {
                    if score >= alpha {
                        return score;
                    }
                }
                Color::Black => {
                    if score <= beta {
                        return score;
                    }
                }
            }

            moves.retain(|mv| board.is_capture(&mv));

            if moves.len() == 0 {
                // End of QS, no captures remain
                return score;
            }
        }

        sort_by_promise(board, &mut moves);

        let mut score = -i16::MAX * board.side_to_move.polarize();
        let mut best_move = moves[0].clone(); // moves len > 0 else gameover and return

        // This is ugly, normally I would use higher order functions
        // but this is easier to follow.
        // TODO: Fix inconsitent usage of 'score' and 'eval'

        if board.side_to_move == Color::White {
            for mv in moves {
                let mv_score = self.alphabeta(&board.make_move(&mv), depth - 1, alpha, beta);
                if mv_score > score {
                    score = mv_score;
                    best_move = mv;
                }

                alpha = i16::max(alpha, score);
                if beta <= alpha {
                    break;
                }
            }
        } else {
            for mv in moves {
                let mv_score = self.alphabeta(&board.make_move(&mv), depth - 1, alpha, beta);
                if mv_score < score {
                    score = mv_score;
                    best_move = mv;
                }

                beta = i16::min(beta, score);
                if beta <= alpha {
                    break;
                }
            }
        }

        // Will always be deepest search of this position, since
        // if there was a deeper search already, we would have returned it.
        self.tp.insert(
            board.clone(),
            SearchResult {
                eval: score,
                depth: depth,
                mv: best_move,
            },
        );
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_sort_by_promise_mate() {
        let board =
            Board::from_fen("rn1qkbnr/ppp2ppp/3p4/4p2Q/2B1P1b1/8/PPPP1PPP/RNB1K1NR w KQkq - 2 4")
                .unwrap();

        let mut moves = MoveGen::new_legal(&board).collect();
        sort_by_promise(&board, &mut moves);

        assert_eq!(moves[0], Movement::from_notation("h5f7").unwrap());
    }

    #[test]
    fn test_pv_deepest_startpos() {
        let depth = 4;

        // TODO: Cleanup this ugly test.

        // The PV should always contain the deepest search for a node.
        // Even when transposition's occur.
        let mut s = Searcher::new();
        let mut board = Board::from_start_pos();
        let sr = s.search_depth(&board, depth);
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(&sr, &sr_tp);

        board.make_move_mut(&sr_tp.mv);
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(sr_tp.depth, depth - 1);

        board.make_move_mut(&sr_tp.mv);
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(sr_tp.depth, depth - 2);
    }

    #[test]
    fn test_pv_deepest_mate2() {
        let mut s = Searcher::new();
        let board = Board::from_fen("8/p4p1k/3p1P2/1p1br3/3p4/1Pr5/P6K/8 b - - 0 1").unwrap();
        let sr = s.search_depth(&board, 5);
        let sr_tp = s.tp.get(&board).unwrap();
        assert_eq!(&sr, sr_tp);
    }

    // The principled variation should always be legal.
    // Including when there is a forced mate, and depth exceeds it.

    #[test]
    fn test_pv_legal_mate_1_white() {
        let board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
                .unwrap();
        eprintln!("board:\n{}", board);

        let mut s = Searcher::new();
        s.search_depth(&board, 4);
        let pv = s.get_pv(&board);
        assert_eq!(moves_to_str(&pv), "h5f7");
    }

    #[test]
    fn test_pv_legal_mate_3_black() {
        let board = Board::from_fen("8/p4p1k/3p1P2/1p1br3/3p4/1Pr5/P6K/8 b - - 0 1").unwrap();
        eprintln!("board:\n{}", board);

        let mut s = Searcher::new();
        s.search_depth(&board, 5);
        let pv = s.get_pv(&board);
        assert_eq!(moves_to_str(&pv), "e5e2 h2g1 c3c1");
    }
}
