use crate::chess::Board;
use crate::movegen::{get_legal_moves, perft};
use crate::search::Searcher;
use crate::uci;
use crate::uci::EngineMessage;
use std::io;

pub struct Engine {
    position: Board,
    searcher: Searcher,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            position: Board::from_start_pos(),
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

    fn perft(&self, depth: u16) {
        let board = &self.position;

        let mut nodes = 0;
        for mv in get_legal_moves(&board) {
            let n = perft(&board.make_move(&mv), depth - 1);
            eprintln!("{}: {}", mv, n);
            nodes += n;
        }

        eprintln!("\nNodes searched: {}", nodes);
    }

    fn go(&mut self, opts: uci::Go) {
        // for debugging
        if let Some(depth) = opts.perft {
            self.perft(depth);
            return;
        }

        let search_result = self
            .searcher
            .search(&self.position, opts.depth.unwrap_or(4));

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
                self.position = board;
            }

            EngineMessage::Go(opts) => self.go(opts),

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let mut engine = Engine::new();
        engine.handle(uci::parse("position startpos").unwrap());
        assert_eq!(engine.position, Board::from_start_pos());

        engine.handle(uci::parse("position startpos moves e2e4 e7e5").unwrap());
        // NOTE: we assert for en-passant e6, that was failing earlier because lichess
        // does not generate a fen with en-passant unless it is possible to be taken.
        assert_eq!(
            engine.position,
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2")
                .unwrap()
        );

        let fen = "K1k5/8/8/8/8/8/8/8 w - - 0 1";
        engine.handle(uci::parse(&format!("position fen {}", fen)).unwrap());
        assert_eq!(engine.position, Board::from_fen(fen).unwrap());
    }
}
