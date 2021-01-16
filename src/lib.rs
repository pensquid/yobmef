use chess::Board;
use search::Searcher;
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
                let search_result = self.searcher.search(&board, opts.depth.unwrap_or(4));

                println!("info score cp {}", search_result.eval);
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
}
