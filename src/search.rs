use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen;
use std::time::Instant;

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub eval: i16,            // Evaluation for the position
    pub mv: Option<Movement>, // The best move
}

#[derive(Debug)]
pub struct Searcher {
    // Search statistics
    pub nodes: u64,
    pub pruned: u64,
}

// Sorting is very important for alpha beta search pruning
fn get_sorted_moves(board: &Board) -> Vec<Movement> {
    let mut moves = movegen::get_legal_moves(board);
    let legal_move_count = moves.len();

    moves.sort_by_key(|m| eval::get_score(&board.make_move(m), legal_move_count));
    if board.side_to_move == Color::White {
        moves.reverse()
    };
    moves
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            nodes: 0,
            pruned: 0,
        }
    }

    // TODO: Quiet search
    // TODO: This is so fucking slow without a TP table
    pub fn search(&mut self, board: &Board, depth: u16) -> SearchResult {
        self.nodes = 0;
        self.pruned = 0;

        let start = Instant::now();
        let sr = self.alphabeta(board, depth, i16::MIN, i16::MAX);
        let end = Instant::now();
        let took = end - start;
        let nps = (self.nodes as f64 / took.as_secs_f64()) as u64;

        println!(
            "info depth {} score cp {} nodes {} nps {}",
            depth, sr.eval, self.nodes, nps
        );
        sr
    }

    pub fn alphabeta(
        &mut self,
        board: &Board,
        depth: u16,
        mut alpha: i16,
        mut beta: i16,
    ) -> SearchResult {
        self.nodes += 1;

        let moves = get_sorted_moves(board);
        let is_game_over = moves.len() == 0;

        if depth == 0 || is_game_over {
            let mut sr = SearchResult {
                eval: eval::get_score(board, moves.len()),
                mv: None,
            };
            if i16::abs(sr.eval) == eval::MATE {
                sr.eval -= (depth as i16) * board.side_to_move.polarize()
            }
            return sr;
        }

        let mut sr = SearchResult {
            eval: -i16::MAX * board.side_to_move.polarize(),
            mv: None,
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

        sr
    }
}
