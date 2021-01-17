use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen;

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub eval: i16,            // Evaluation for the position
    pub mv: Option<Movement>, // The best move
}

#[derive(Debug)]
pub struct Searcher {
    // Search statistics
    pub nodes: u64,
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
        Searcher { nodes: 0 }
    }

    // TODO: Quiet search
    // TODO: This is so fucking slow without a TP table
    pub fn search(&mut self, board: &Board, start_depth: u16) -> SearchResult {
        let mut deepest = None;
        let mut depth = start_depth;
        // TODO: Normally you would do time control not node count.
        while self.nodes < 10000 || deepest.is_none() {
            self.nodes = 0;
            let sr = self.alphabeta(board, depth, i16::MIN, i16::MAX);
            println!(
                "info depth {} score cp {} nodes {}",
                depth, sr.eval, self.nodes
            );
            deepest = Some(sr);
            depth += 1;
        }

        return deepest.unwrap();
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
                    break;
                }
            }
        }

        sr
    }
}
