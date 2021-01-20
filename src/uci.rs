use chess::Board;

use crate::chess;
use std::str::{FromStr, Split};

#[derive(Debug, PartialEq, Eq)]
pub enum GoVariant {
    Vanilla,
    Infinite,
    Ponder,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Go {
    pub search_moves: Option<Vec<chess::Movement>>,

    pub white_time: Option<u64>,
    pub black_time: Option<u64>,
    pub white_increment: Option<u64>,
    pub black_increment: Option<u64>,
    pub moves_to_go: Option<u8>,

    pub depth: Option<i16>,
    pub nodes: Option<u64>,
    pub mate: Option<u8>,

    pub move_time: Option<u32>,
    pub perft: Option<u16>,

    pub variant: GoVariant,
}

impl Go {
    pub fn empty() -> Self {
        Go {
            search_moves: None,

            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,

            depth: None,
            nodes: None,
            mate: None,

            perft: None,
            move_time: None,
            variant: GoVariant::Vanilla,
        }
    }

    pub fn variant(v: GoVariant) -> Self {
        let mut go = Self::empty();
        go.variant = v;
        go
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineMessage {
    UCI,
    Debug(bool),
    IsReady,

    UCINewGame,
    Position(Board, Vec<chess::Movement>),
    Go(Go),

    Stop,
    PonderHit,
    Quit,

    DontMissTheShredderChessAnnualBarbeque, // Very important 10/10
}

fn get_moves(mut words: Split<char>) -> Option<Vec<chess::Movement>> {
    let mut moves = Vec::new();

    while let Some(word) = words.next() {
        moves.push(chess::Movement::from_notation(word)?);
    }

    Some(moves)
}

pub fn parse(s: &str) -> Option<EngineMessage> {
    let mut words = s.split(' ');

    Some(match words.next()? {
        "uci" => EngineMessage::UCI,
        "debug" => match words.next()? {
            "true" => EngineMessage::Debug(true),
            "false" => EngineMessage::Debug(false),
            _ => return None,
        },
        "isready" => EngineMessage::IsReady,

        "ucinewgame" => EngineMessage::UCINewGame,
        "position" => {
            let board;
            let mut moves = Vec::new();

            match words.next()? {
                "startpos" => {
                    board = Board::from_start_pos();

                    if let Some("moves") = words.next() {
                        moves = get_moves(words)?;
                    }
                }

                "fen" => {
                    let mut fen = Vec::new();

                    loop {
                        let word = words.next();
                        match word {
                            Some("moves") => break,
                            Some(chunk) => fen.push(chunk),
                            _ => break,
                        }
                    }
                    let fen = fen.join(" ");
                    board = Board::from_fen(&fen)?;
                    moves = get_moves(words)?;
                }

                _ => return None,
            }

            EngineMessage::Position(board, moves)
        }
        "go" => {
            let mut go = Go::empty();

            while let Some(word) = words.next() {
                match word {
                    "ponder" => go.variant = GoVariant::Ponder,
                    "infinite" => go.variant = GoVariant::Infinite,

                    "searchmoves" => {
                        let mut moves = Vec::new();
                        while let Some(word) = words.next() {
                            moves.push(chess::Movement::from_notation(word)?);
                        }

                        go.search_moves = Some(moves);
                        break;
                    }

                    "wtime" => go.white_time = u64::from_str(words.next()?).ok(),
                    "btime" => go.black_time = u64::from_str(words.next()?).ok(),
                    "winc" => go.white_increment = u64::from_str(words.next()?).ok(),
                    "binc" => go.black_increment = u64::from_str(words.next()?).ok(),
                    "movestogo" => go.moves_to_go = u8::from_str(words.next()?).ok(),

                    "depth" => go.depth = i16::from_str(words.next()?).ok(),
                    "nodes" => go.nodes = u64::from_str(words.next()?).ok(),
                    "mate" => go.mate = u8::from_str(words.next()?).ok(),

                    "movetime" => go.move_time = u32::from_str(words.next()?).ok(),

                    // For debugging
                    "perft" => go.perft = u16::from_str(words.next()?).ok(),

                    _ => (),
                }
            }

            EngineMessage::Go(go)
        }

        "stop" => EngineMessage::Stop,
        "ponderhit" => EngineMessage::PonderHit,
        "quit" => EngineMessage::Quit,

        "uwu" => EngineMessage::DontMissTheShredderChessAnnualBarbeque,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use chess::Movement;

    use super::*;

    #[test]
    fn test_parse_go() {
        assert_eq!(parse("go"), Some(EngineMessage::Go(Go::empty())));
    }

    #[test]
    fn test_parse_go_infinite() {
        assert_eq!(
            parse("go infinite"),
            Some(EngineMessage::Go(Go::variant(GoVariant::Infinite)))
        );
    }

    #[test]
    fn test_parse_go_ponder() {
        assert_eq!(
            parse("go ponder"),
            Some(EngineMessage::Go(Go::variant(GoVariant::Ponder)))
        );
    }

    #[test]
    fn test_uci() {
        assert_eq!(parse("uci"), Some(EngineMessage::UCI))
    }

    #[test]
    fn test_position() {
        assert_eq!(
            parse("position startpos"),
            Some(EngineMessage::Position(Board::from_start_pos(), Vec::new()))
        );

        assert_eq!(
            parse("position startpos moves e2e4"),
            Some(EngineMessage::Position(
                Board::from_start_pos(),
                vec![Movement::from_notation("e2e4").unwrap()]
            ))
        );

        assert_eq!(parse("position"), None);

        assert_eq!(
            parse("position fen 2k5/2r5/8/3K4/8/8/8/8 b - - 0 1 moves c7c2 d5e5"),
            Some(EngineMessage::Position(
                Board::from_fen("2k5/2r5/8/3K4/8/8/8/8 b - - 0 1").unwrap(),
                vec![
                    Movement::from_notation("c7c2").unwrap(),
                    Movement::from_notation("d5e5").unwrap()
                ],
            )),
        );
    }
}
