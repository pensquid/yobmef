use chess::{Board, Color, Movement};
use std::io;
use uci::EngineMessage;

pub mod analyze;
pub mod bitboard;
pub mod chess;
pub mod movegen;
pub mod uci;

pub struct Engine {
    position: Option<Board>,
    best_move: Option<Movement>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            position: None,
            best_move: None,
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

    fn handle(&mut self, msg: uci::EngineMessage) {
        match msg {
            EngineMessage::UCI => {
                println!("id name Yobmef");
                println!("id author PwnSquad");
                println!("uciok");
            }
            EngineMessage::IsReady => println!("readyok"),
            EngineMessage::Quit => std::process::exit(0),

            EngineMessage::Stop => {
                eprintln!("ASdasd {:?}", self.best_move);
                if let Some(best_move) = &self.best_move {
                    println!("bestmove {}", best_move.to_notation());
                    self.best_move = None;
                }
            }

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
                    let score = analyze::get_score(&board);
                    println!("info score cp {}", score);

                    let moves = movegen::get_moves(&board);
                    let mut sorted_moves = Vec::new();
                    moves.iter().for_each(|m| sorted_moves.push(m));
                    sorted_moves.sort_by_key(|m| analyze::get_score(&board.make_move(m)));
                    if board.side_to_move == Color::White {
                        sorted_moves.reverse();
                    }

                    let best_move = (*sorted_moves.get(0).unwrap()).clone();
                    println!("info pv {}", best_move.to_notation());
                    self.best_move = Some(best_move);
                }
            }

            _ => {}
        }
    }
}
