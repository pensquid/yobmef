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
    pub depth: u16,
}

#[derive(Debug)]
pub struct Searcher {
    // Transposition table
    // TODO: Store PV and use as move guesses for a/b search
    pub tp: HashMap<Board, SearchResult>,

    // Search statistics
    pub nodes: u64,
    pub pruned: u64,
    pub cached: u64,
}

// Sorting is very important for alpha beta search pruning
pub fn sort_by_promise(board: &Board, moves: &mut Vec<Movement>) {
    let legal_move_count = moves.len();

    moves.sort_by_cached_key(|m| eval::get_score(&board.make_move(m), legal_move_count));
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
            pruned: 0,
            cached: 0,

            tp: HashMap::new(),
        }
    }

    fn reset_stats(&mut self) {
        self.nodes = 0;
        self.pruned = 0;
        self.cached = 0;
    }

    pub fn search_depth(&mut self, board: &Board, depth: u16) -> SearchResult {
        self.search(board, |sr| sr.depth >= depth)
    }

    // TODO: Quiet search
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
            let ab_start = Instant::now();
            let sr = self.alphabeta(board, depth, 0, i16::MIN, i16::MAX);
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
        max_depth: u16,
        depth: u16,
        mut alpha: i16,
        mut beta: i16,
    ) -> SearchResult {
        self.nodes += 1;

        let depth_to_go = max_depth - depth;

        if let Some(sr) = self.tp.get(board) {
            if sr.depth >= depth_to_go {
                self.cached += 1;
                return sr.clone();
            }

            // TODO: Use sr as guess for the best move,
            // even if the depth is not sufficent to return it immediately.
        }

        let mut moves: Vec<Movement> = MoveGen::new_legal(board).collect();
        let is_game_over = moves.len() == 0;

        if depth >= max_depth || is_game_over {
            let mut sr = SearchResult {
                eval: eval::get_score(board, moves.len()),
                mv: None,
                depth: 0,
            };
            if i16::abs(sr.eval) == eval::MATE {
                // if side_to_move = White:
                //   black won, so we add depth to make black prefer shorter mates.
                // ditto for if side_to_move = Black.
                sr.eval += (depth as i16) * board.side_to_move.polarize()
            }
            // We don't store the static eval in the TP table, because
            // looking it up would likely take longer then re-computing it!
            return sr;
        }

        sort_by_promise(board, &mut moves);

        let mut sr = SearchResult {
            eval: -i16::MAX * board.side_to_move.polarize(),
            mv: None,
            depth: depth_to_go,
        };

        // This is ugly, normally I would use higher order functions
        // but this is easier to follow.
        // TODO: Fix inconsitent usage of 'score' and 'eval'

        if board.side_to_move == Color::White {
            for mv in moves {
                let score =
                    self.alphabeta(&board.make_move(&mv), max_depth, depth + 1, alpha, beta);
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
                let score =
                    self.alphabeta(&board.make_move(&mv), max_depth, depth + 1, alpha, beta);

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
