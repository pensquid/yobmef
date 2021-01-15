use crate::chess::{Board, Color, Movement};
use crate::eval;
use crate::movegen;
use std::thread;

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub eval: i16,            // evaluation for the position
    pub mv: Option<Movement>, // the best move
}

#[derive(Debug)]
pub struct Searcher {
    // TODO: Transposition table
}

// Sorting is very important for alpha beta search pruning
fn get_sorted_moves(board: &Board) -> Vec<Movement> {
    let mut moves = movegen::get_legal_moves(board);

    moves.sort_by_key(|m| eval::get_score(&board.make_move(m)));
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
    pub fn search(&self, board: &Board) -> SearchResult {
        if board.game_over() {
            return SearchResult {
                eval: eval::get_score(board),
                mv: None,
            };
        }

        // (this will be replaced with iterative deepening later)
        let depth = 3;

        let turn = board.side_to_move.num(); // white=1, black=-1
        let white = board.side_to_move == Color::White;
        let better = if white { i16::gt } else { i16::lt };

        let mut best_move = None;
        let mut best_score = -eval::MATE * turn; // start with worst score

        let moves = get_sorted_moves(board);
        for mv in moves {
            let score = self.alphabeta(&board.make_move(&mv), depth - 1);
            if better(&score, &best_score) || best_move.is_none() {
                // eprintln!(
                //     "{}({:?}) better then {}({:?}) for {:?}",
                //     score, mv, best_score, best_move, board.side_to_move
                // );
                best_score = score;
                best_move = Some(mv);
            }
        }

        SearchResult {
            eval: best_score,
            mv: best_move,
        }
    }

    // TODO: Add prune (currently just minimax)
    pub fn alphabeta(&self, board: &Board, depth: u16) -> i16 {
        if depth == 0 || board.game_over() {
            if board.game_over() {
                eprintln!("game over {:?}\n{}", board.status(), board);
            }
            return eval::get_score(board);
        }

        let moves = movegen::get_legal_moves(board); // TODO: Sort improves pruning

        let mut best;

        // This is ugly, normally I would use higher order functions
        // but once you add alphabeta pruning its hard to abstract.
        // Also, maybe this shouldn't be an associated method with Engine.

        if board.side_to_move == Color::White {
            best = -eval::MATE;

            for mv in moves {
                let score = self.alphabeta(&board.make_move(&mv), depth - 1);
                if score > best {
                    best = score;
                }
            }
        } else {
            best = eval::MATE;

            for mv in moves {
                let score = self.alphabeta(&board.make_move(&mv), depth - 1);
                if score < best {
                    best = score;
                }
            }
        }

        best
    }
}
