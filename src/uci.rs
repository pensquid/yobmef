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
    search_moves: Option<Vec<chess::Movement>>,

    white_time: Option<u64>,
    black_time: Option<u64>,
    white_increment: Option<u32>,
    black_increment: Option<u32>,
    moves_to_go: Option<u8>,

    depth: Option<u8>,
    nodes: Option<u64>,
    mate: Option<u8>,

    move_time: Option<u32>,

    variant: GoVariant,
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
    Position(chess::Board, Vec<chess::Movement>),
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
        "position" => match words.next()? {
            // TODO: Clean up the duplication here
            "startpos" => {
                loop {
                    let word = words.next();
                    match word {
                        Some("moves") => break,
                        None => return None,
                        _ => continue,
                    }
                }

                EngineMessage::Position(chess::Board::from_start_pos(), get_moves(words)?)
            }

            "fen" => {
                let mut fen = Vec::new();
                loop {
                    let word = words.next();
                    match word {
                        Some("moves") => break,
                        Some(chunk) => fen.push(chunk),
                        None => return None,
                    }
                }
                let fen = fen.join(" ");

                EngineMessage::Position(chess::Board::from_fen(&fen)?, get_moves(words)?)
            }

            _ => return None,
        },
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
                    "winc" => go.white_increment = u32::from_str(words.next()?).ok(),
                    "binc" => go.black_increment = u32::from_str(words.next()?).ok(),
                    "movestogo" => go.moves_to_go = u8::from_str(words.next()?).ok(),

                    "depth" => go.depth = u8::from_str(words.next()?).ok(),
                    "nodes" => go.nodes = u64::from_str(words.next()?).ok(),
                    "mate" => go.mate = u8::from_str(words.next()?).ok(),

                    "movetime" => go.move_time = u32::from_str(words.next()?).ok(),

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
}
