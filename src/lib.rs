use chess::Board;
use std::io;
use uci::EngineMessage;

pub mod analyze;
pub mod bitboard;
pub mod chess;
pub mod movegen;
pub mod uci;

pub struct Engine {}

impl Engine {
    pub fn new() -> Engine {
        Engine {}
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
                    board.make_move_mut(movement);
                }
                eprintln!("current position:\n{}", board);

                let score = analyze::get_score(board);
                println!("info score cp {}", score);
            }

            _ => {}
        }
    }
}
