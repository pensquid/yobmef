use crate::chess::{Board, Color};
use crate::movegen::{perft, MoveGen};
use crate::search::Searcher;
use crate::uci;
use crate::uci::EngineMessage;
use std::io;
use std::time::Duration;

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
        for mv in MoveGen::new_legal(&board) {
            let n = perft(&board.make_move(&mv), depth - 1);
            eprintln!("{}: {}", mv, n);
            nodes += n;
        }

        eprintln!("\nNodes searched: {}", nodes);
    }

    fn thinking_time(&self, opts: uci::Go) -> Duration {
        let (our_time, our_increment) = match self.position.side_to_move {
            Color::White => (opts.white_time, opts.white_increment),
            Color::Black => (opts.black_time, opts.black_increment),
        };
        // default to as if we had 10m no inc for correspondence games.
        let our_time = our_time.unwrap_or(600_000);
        let our_increment = our_increment.unwrap_or(0);

        // 80 is just a conservative estimate on the avg game length.
        let time_for_this_move = our_time / 80;

        Duration::from_millis(time_for_this_move + our_increment)
    }

    fn go(&mut self, opts: uci::Go) {
        // For debugging
        if let Some(depth) = opts.perft {
            self.perft(depth);
            return;
        }

        let sr = if let Some(depth) = opts.depth {
            self.searcher.search_depth(&self.position, depth)
        } else {
            let thinking_time = self.thinking_time(opts);
            self.searcher.search_timed(&self.position, thinking_time)
        };

        println!("bestmove {}", sr.mv.unwrap());
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

    #[test]
    fn test_think_time() {
        // reasonable bounds on thinking time

        let engine = Engine::new();
        let mut opts = uci::Go::empty();
        opts.white_time = Some(300_000);
        // black_time: 300_000,
        let t = engine.thinking_time(opts).as_millis();

        // assume you will think between 1s and 10s per move in a 5 minute game
        assert!(1_000 < t && t < 10_000, "1s < t({}s) < 10s", t / 1000);
    }
}
