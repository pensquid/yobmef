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

        // so we don't use infinite memory
        // TODO: Maintain TP between searches, via replacement strategies.
        // https://www.chessprogramming.org/Transposition_Table#Replacement_Strategies
        self.tp.clear();

        deepest.unwrap() // safe because we always run alphabeta at least once.
    }

    // TODO: Perhaps keep pv state and update from alphabeta?
    // Need to see how stockfish does it.
    fn get_pv(&self, board: &Board) -> Vec<Movement> {
        let mut moves = Vec::new();
        let mut curr = board.clone();
        while let Some(sr) = self.tp.get(&curr) {
            if let Some(mv) = &sr.mv {
                curr.make_move_mut(&mv);
                moves.push(mv.clone());
            } else {
                break;
            }
        }
        moves
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
}
