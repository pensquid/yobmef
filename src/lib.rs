use chess::{Board, Color, Movement};
use std::io;
use uci::EngineMessage;

pub mod bitboard;
pub mod chess;
pub mod eval;
pub mod movegen;
pub mod uci;

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub eval: i16,            // evaluation for the position
    pub mv: Option<Movement>, // the best move
}

pub struct Engine {
    position: Option<Board>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine { position: None }
    }

    pub fn uci_loop(&mut self) -> io::Result<()> {
        use std::io::BufRead;

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let msg = uci::parse(&line);

            eprintln!("Got: {}", line);
            eprintln!("Parse: {:?}", msg);

            if let Some(msg) = msg {
                self.handle(msg);
            }
        }

        Ok(())
    }

    // TODO: Add prune (currently just minimax)
    fn alphabeta(board: &Board, depth: u16) -> i16 {
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
        // Also, maybe this shoulden't be an associated method with Engine.

        if board.side_to_move == Color::White {
            best = -eval::MATE;

            for mv in moves {
                let score = Self::alphabeta(&board.make_move(&mv), depth - 1);
                if score > best {
                    best = score;
                }
            }
        } else {
            best = eval::MATE;

            for mv in moves {
                let score = Self::alphabeta(&board.make_move(&mv), depth - 1);
                if score < best {
                    best = score;
                }
            }
        }

        best
    }

    // TODO: Transposition table
    // TODO: Quiet search
    // TODO: Iterative deepening (stop when uci 'stop' is sent)
    fn search(board: &Board) -> SearchResult {
        if board.game_over() {
            return SearchResult {
                eval: eval::get_score(board),
                mv: None,
            };
        }

        // (this will be replaced with iterative deepening later)
        let depth = 3;

        let mut moves = movegen::get_legal_moves(board);

        // Sorting is very important for alpha beta search pruning
        moves.sort_by_key(|m| eval::get_score(&board.make_move(m)));
        if board.side_to_move == Color::White {
            moves.reverse()
        };

        let turn = board.side_to_move.num(); // white=1, black=-1
        let white = board.side_to_move == Color::White;
        let better = if white { i16::gt } else { i16::lt };

        let mut best_move = None;
        let mut best_score = -eval::MATE * turn; // start with worst score

        for mv in moves {
            let score = Self::alphabeta(&board.make_move(&mv), depth - 1);
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

    fn go(board: &Board) {
        // let score = eval::get_score(&board);
        let search_result = Self::search(board);

        println!("info score cp {}", search_result.eval);

        // will only fail at unwrap if board is gameover I think
        println!("bestmove {}", search_result.mv.unwrap());
    }

    fn handle(&mut self, msg: uci::EngineMessage) {
        match msg {
            EngineMessage::UCI => {
                println!("id name Yobmef");
                println!("id author PwnSquad");
                println!("uciok");
            }
            EngineMessage::IsReady => println!("readyok"),
            EngineMessage::Quit => std::process::exit(0),

            EngineMessage::Position(board, moves) => {
                let mut board: Board = board.clone();
                for movement in moves {
                    board.make_move_mut(&movement);
                }
                eprintln!("current position:\n{}", board);
                self.position = Some(board);
            }

            EngineMessage::Go(_) => {
                if let Some(board) = &self.position {
                    Self::go(board);
                } else {
                    eprintln!("No position specified");
                }
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests

    #[test]
    fn test_position() {
        let mut engine = Engine::new();
        engine.handle(uci::parse("position startpos").unwrap());
        assert_eq!(engine.position, Some(Board::from_start_pos()));

        engine.handle(uci::parse("position startpos moves e2e4 e7e5").unwrap());
        // NOTE: we assert for en-passant e6, that was failing earlier because lichess
        // does not generate a fen with en-passant unless it is possible to be taken.
        assert_eq!(
            engine.position,
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2")
        );

        let fen = "K1k5/8/8/8/8/8/8/8 w - - 0 1";
        engine.handle(uci::parse(&format!("position fen {}", fen)).unwrap());
        assert_eq!(engine.position, Board::from_fen(fen));
    }

    // TODO: Use macro, move to separate file for search tests,
    //       This is only here so I can implement alphabeta without getting cancer.

    use crate::movegen::gen_moves_once;

    #[test]
    fn knight_fork() {
        gen_moves_once();
        let fen = "8/3k4/1p4r1/8/2N5/8/8/K7 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        let search_result = Engine::search(&board);
        assert_eq!(search_result.mv, Movement::from_notation("c4e5"));
        assert!(search_result.eval > 0);
    }

    #[test]
    fn knight_fork_black() {
        gen_moves_once();
        let fen = "8/3K4/1P4Q1/8/2n5/8/8/k7 b - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        let search_result = Engine::search(&board);
        assert_eq!(search_result.mv, Movement::from_notation("c4e5"));
        assert!(search_result.eval < 0);
    }

    // Does not pass, either I have a bug or vannila minimax is not strong enough
    // to find it. alphabeta should though.
    // #[test]
    // fn mate_3_fishing_pole() {
    //     gen_moves_once();

    //     let fen = "r1b1kb1r/pppp1pp1/2n5/1B2p3/4PPpq/8/PPPP2P1/RNBQNRK1 b kq f3 0 8";
    //     let board = Board::from_fen(fen).unwrap();

    //     let search_result = Engine::search(&board);
    //     assert_eq!(search_result.mv, Movement::from_notation("g4g3"));
    //     assert_eq!(search_result.eval, eval::MATE);
    // }
}
