use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen::MoveGen;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchResult {
    pub eval: i16,            // Evaluation for the position
    pub mv: Option<Movement>, // The best move

    // Depth of this evaluation, with respect to the root node.
    pub depth: i16,
}

#[derive(Debug)]
pub struct Searcher {
    // Transposition table
    // TODO: Store PV and use as move guesses for a/b search
    pub tp: HashMap<Board, SearchResult>,

    // Search statistics
    pub nodes: u64, // excluding qs!
    pub qs_nodes: u64,
    pub pruned: u64,
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
fn moves_to_str(moves: Vec<Movement>) -> String {
    moves
        .iter()
        .map(|mv| mv.to_notation())
        .collect::<Vec<String>>()
        .join(" ")
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            nodes: 0,
            qs_nodes: 0,
            pruned: 0,
            cached: 0,
            tp: HashMap::new(),

            start_depth: 0,
        }
    }

    fn reset_stats(&mut self) {
        self.nodes = 0;
        self.pruned = 0;
        self.qs_nodes = 0;
        self.cached = 0;
    }

    pub fn search_depth(&mut self, board: &Board, depth: i16) -> SearchResult {
        self.search(board, |sr| sr.depth >= depth)
    }

    pub fn search_timed(&mut self, board: &Board, thinking_time: Duration) -> SearchResult {
        let start = Instant::now();
        self.search(board, |_sr| start.elapsed() > thinking_time)
    }

    pub fn search<F>(&mut self, board: &Board, quit: F) -> SearchResult
    where
        F: Fn(SearchResult) -> bool,
    {
        self.reset_stats();

        // so we don't use infinite memory
        // TODO: Maintain TP between searches, via replacement strategies.
        // https://www.chessprogramming.org/Transposition_Table#Replacement_Strategies
        // NOTE: Tests rely on TP being available after search to verify PV.
        self.tp.clear();

        let mut deepest = None;

        // Bound ply because of possible recursion limit in endgames.
        for depth in 1..1000 {
            self.start_depth = depth;

            let ab_start = Instant::now();
            let sr = self.alphabeta(board, depth, i16::MIN, i16::MAX);
            let nps = (self.nodes as f64 / ab_start.elapsed().as_secs_f64()) as u64;
            let pv = self.get_pv(board);

            println!(
                "info depth {} score cp {} nodes {} nps {} pv {}",
                depth,
                sr.eval,
                self.nodes,
                nps,
                moves_to_str(pv),
            );
            deepest = Some(sr.clone());

            if quit(sr) {
                break;
            }
        }

        deepest.unwrap() // safe because we always run alphabeta at least once.
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

    // Get the next PV move
    // NOTE: This assumes the TP will always hold the deepest search for a given board.
    fn get_pv_next(&self, board: &Board) -> Option<&Movement> {
        let sr = self.tp.get(board)?;
        sr.mv.as_ref()
    }

    pub fn alphabeta(
        &mut self,
        board: &Board,
        mut depth: i16,
        mut alpha: i16,
        mut beta: i16,
    ) -> SearchResult {
        if depth < 0 {
            self.qs_nodes += 1;
        } else {
            self.nodes += 1;
        }

        if let Some(sr) = self.tp.get(board) {
            if sr.depth >= depth {
                self.cached += 1;
                return sr.clone();
            }

            // TODO: Use sr as guess for the best move,
            // even if the depth is not sufficent to return it immediately.
        }

        let mut moves: Vec<Movement> = MoveGen::new_legal(board).collect();
        let is_game_over = moves.len() == 0;

        // NOTE: We don't store the static eval in the TP table, because we aren't whores.
        if is_game_over {
            return SearchResult {
                // The bigger depth to go, the better
                eval: (depth + eval::MATE) * board.side_to_move.other().polarize(),
                mv: None,
                depth: 0,
            };
        }

        // So simple, yet so effective!
        if board.in_check() {
            depth += 1;
        }

        if depth < 0 {
            // Quiet search!
            let score = eval::get_score(board, is_game_over);
            let sr = SearchResult {
                eval: score,
                mv: None,
                depth: 0,
            };

            // It is our move, so if the static score is already better then
            // Our previous best score, we can just return the static eval.
            // TODO: Refactor into a negamax framework to cleanup this code.
            // FIXME: If we're in zugzwang, then this will prematurely prune.
            match board.side_to_move {
                Color::White => {
                    if score >= alpha {
                        return sr;
                    }
                }
                Color::Black => {
                    if score <= beta {
                        return sr;
                    }
                }
            }

            moves.retain(|mv| board.is_capture(&mv));

            if moves.len() == 0 {
                // End of QS, no captures remain
                return SearchResult {
                    eval: eval::get_score(board, is_game_over),
                    mv: None,
                    depth: 0,
                };
            }
        }

        sort_by_promise(board, &mut moves);

        let mut sr = SearchResult {
            eval: -i16::MAX * board.side_to_move.polarize(),
            mv: None,
            depth: i16::max(depth, 0), // avoid negative depth in QS
        };

        // This is ugly, normally I would use higher order functions
        // but this is easier to follow.
        // TODO: Fix inconsitent usage of 'score' and 'eval'

        if board.side_to_move == Color::White {
            for mv in moves {
                let score = self.alphabeta(&board.make_move(&mv), depth - 1, alpha, beta);
                if score.eval > sr.eval {
                    sr.eval = score.eval;
                    sr.mv = Some(mv);
                }

                alpha = i16::max(alpha, score.eval);
                if beta <= alpha {
                    self.pruned += 1;
                    break;
                }
            }
        } else {
            for mv in moves {
                let score = self.alphabeta(&board.make_move(&mv), depth - 1, alpha, beta);

                if score.eval < sr.eval {
                    sr.eval = score.eval;
                    sr.mv = Some(mv);
                }

                beta = i16::min(beta, score.eval);
                if beta <= alpha {
                    self.pruned += 1;
                    break;
                }
            }
        }

        self.tp.insert(board.clone(), sr.clone());
        sr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen;
    use crate::movegen::helpers::assert_moves;

    #[ignore]
    #[test]
    fn test_sort_by_promise_mate() {
        movegen::gen_moves_once();

        let board =
            Board::from_fen("rn1qkbnr/ppp2ppp/3p4/4p2Q/2B1P1b1/8/PPPP1PPP/RNB1K1NR w KQkq - 2 4")
                .unwrap();

        let mut moves = MoveGen::new_legal(&board).collect();
        sort_by_promise(&board, &mut moves);

        assert_eq!(moves[0], Movement::from_notation("h5f7").unwrap());
    }

    #[test]
    fn test_pv_deepest_startpos() {
        movegen::gen_moves_once();
        let depth = 4;

        // TODO: Cleanup this ugly test.

        // The PV should always contain the deepest search for a node.
        // Even when transposition's occur.
        let mut s = Searcher::new();
        let mut board = Board::from_start_pos();
        let sr = s.search_depth(&board, depth);
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(&sr, &sr_tp);

        board.make_move_mut(&sr_tp.mv.unwrap());
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(sr_tp.depth, depth - 1);

        board.make_move_mut(&sr_tp.mv.unwrap());
        let sr_tp = s.tp.get(&board).unwrap().clone();
        assert_eq!(sr_tp.depth, depth - 2);
    }

    #[test]
    fn test_pv_deepest_mate2() {
        movegen::gen_moves_once();

        let mut s = Searcher::new();
        let board = Board::from_fen("8/p4p1k/3p1P2/1p1br3/3p4/1Pr5/P6K/8 b - - 0 1").unwrap();
        let sr = s.search_depth(&board, 5);
        let sr_tp = s.tp.get(&board).unwrap();
        assert_eq!(&sr, sr_tp);
    }

    #[test]
    fn test_pv_legal_mate() {
        movegen::gen_moves_once();

        // The principled variation should always be legal.
        // Including when there is a forced mate, and depth exceeds it.

        let board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
                .unwrap();

        let mut s = Searcher::new();
        s.search_depth(&board, 4);
        let pv = s.get_pv(&board);

        assert_moves(&board, pv, "h5f7");

        let board = Board::from_fen("8/p4p1k/3p1P2/1p1br3/3p4/1Pr5/P6K/8 b - - 0 1").unwrap();
        // Reusing searcher is fine, it's what the engine would do
        // depth 5 so after mate it has a chance to fuckup with an illegal move.
        s.search_depth(&board, 5);
        let pv = s.get_pv(&board);
        assert_moves(&board, pv, "e5e2 h2g1 c3c1");
    }
}
