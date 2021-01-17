use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen;
use std::collections::HashMap;
use std::time::Instant;

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
}

// Sorting is very important for alpha beta search pruning
pub fn sort_by_promise(board: &Board, moves: &mut Vec<Movement>) {
    let legal_move_count = moves.len();

    moves.sort_by_key(|m| eval::get_score(&board.make_move(m), legal_move_count));
    if board.side_to_move == Color::White {
        moves.reverse()
    };
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            nodes: 0,
            pruned: 0,

            tp: HashMap::new(),
        }
    }

    // TODO: Quiet search
    pub fn search(&mut self, board: &Board, depth: u16) -> SearchResult {
        self.nodes = 0;
        self.pruned = 0;

        let start = Instant::now();
        let sr = self.alphabeta(board, depth, 0, i16::MIN, i16::MAX);
        let end = Instant::now();
        let took = end - start;
        let nps = (self.nodes as f64 / took.as_secs_f64()) as u64;

        println!(
            "info depth {} score cp {} nodes {} nps {}",
            depth, sr.eval, self.nodes, nps
        );

        // so we don't use infinite memory
        // TODO: Maintain TP between searches, via replacement strategies.
        // https://www.chessprogramming.org/Transposition_Table#Replacement_Strategies
        self.tp.clear();

        sr
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

        // FIXME: fastest mate tests don't like this
        if let Some(sr) = self.tp.get(board) {
            if sr.depth >= depth_to_go {
                return sr.clone();
            }

            // TODO: Use sr as guess for the best move,
            // even if the depth is not sufficent to return it immediately.
        }

        let mut moves = movegen::get_legal_moves(board);
        let is_game_over = moves.len() == 0;

        if depth >= max_depth || is_game_over {
            let mut sr = SearchResult {
                eval: eval::get_score(board, moves.len()),
                mv: None,
                depth: 0,
            };
            if i16::abs(sr.eval) == eval::MATE {
                sr.eval -= (depth as i16) * board.side_to_move.polarize()
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
