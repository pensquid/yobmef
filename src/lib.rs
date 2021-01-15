use chess::{Board, Color, Movement};
use search::{SearchResult, Searcher};
use std::io;
use uci::EngineMessage;

pub mod bitboard;
pub mod chess;
pub mod eval;
pub mod movegen;
pub mod search;
pub mod uci;

pub struct Engine {
    position: Option<Board>,
    searcher: Searcher,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            position: None,
            searcher: Searcher::new(),
        }
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

    fn go(&self, opts: uci::Go) {
        match &self.position {
            None => {
                eprintln!("No position specified");
            }

            Some(board) => {
                // let score = eval::get_score(&board);
                let search_result = self.searcher.search(&board);

                println!("info score cp {}", search_result.eval);

                // will only fail at unwrap if board is gameover I think
                println!("bestmove {}", search_result.mv.unwrap());
            }
        }
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

            EngineMessage::Go(opts) => self.go(opts),

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

    // search in tests (with fresh searcher)
    fn search(board: &Board) -> SearchResult {
        // For when I implement threading for searcher.
        // use std::time::Duration;

        // let mut searcher = Searcher::new();
        // searcher.start(board);
        // std::thread::sleep(Duration::from_millis(100));

        // return searcher.stop();

        let mut searcher = Searcher::new();
        searcher.search(board)
    }

    #[test]
    fn knight_fork() {
        gen_moves_once();
        let fen = "8/3k4/1p4r1/8/2N5/8/8/K7 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        let search_result = search(&board);
        assert_eq!(search_result.mv, Movement::from_notation("c4e5"));
        assert!(search_result.eval > 0);
    }

    #[test]
    fn knight_fork_black() {
        gen_moves_once();
        let fen = "8/3K4/1P4Q1/8/2n5/8/8/k7 b - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        let search_result = search(&board);
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
