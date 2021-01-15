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
    pub fn search(&self, board: &Board) -> SearchResult {
        let moves = get_sorted_moves(board);
        let is_game_over = moves.len() == 0;
        if is_game_over {
            return SearchResult {
                eval: eval::get_score(board, moves.len()),
                mv: None,
            };
        }

        // (This will be replaced with iterative deepening later)
        let depth = 3;

        let turn = board.side_to_move.polarize();
        let white = board.side_to_move == Color::White;
        let better = if white { i16::gt } else { i16::lt };

        let mut best_move = None;
        let mut best_score = -eval::MATE * turn; // start with worst score

        for mv in moves {
            let score = self.alphabeta(&board.make_move(&mv), depth - 1);
            if better(&score, &best_score) || best_move.is_none() {
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
        let moves = get_sorted_moves(board);
        let is_game_over = moves.len() == 0;

        if depth == 0 || is_game_over {
            return eval::get_score(board, moves.len());
        }

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
