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
    // TODO: Transposition table
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
        Searcher {}
    }

    // TODO: Quiet search
    // TODO: Iterative deepening (stop when uci 'stop' is sent)
    pub fn search(&self, board: &Board, depth: u16) -> SearchResult {
        self.alphabeta(board, depth, i16::MIN, i16::MAX)
    }

    pub fn alphabeta(
        &self,
        board: &Board,
        depth: u16,
        mut alpha: i16,
        mut beta: i16,
    ) -> SearchResult {
        let moves = get_sorted_moves(board);
        let is_game_over = moves.len() == 0;

        if depth == 0 || is_game_over {
            let mut sr = SearchResult {
                eval: eval::get_score(board, moves.len()),
                mv: None,
            };
            if i16::abs(sr.eval) == eval::MATE {
                sr.eval += depth as i16;
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
