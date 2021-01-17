use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::str::FromStr;
use yobmef::chess::{Board, Movement};
use yobmef::movegen::{gen_moves_once, get_legal_moves, perft};

#[derive(Debug, PartialEq, Eq)]
struct PerftResult {
    mv: Movement,
    nodes: u64,
}

type PerftResults = Vec<PerftResult>;

fn engine_perft(path: &str, moves: &str, depth: u16) -> Result<PerftResults, Box<dyn Error>> {
    let mut proc = Command::new(path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;

    // TODO: Proper error handling with Option
    let mut stdin = proc.stdin.take().unwrap();
    let stdout = BufReader::new(proc.stdout.take().unwrap());

    // NOTE: This is relying on the engine defaulting to startpos
    if moves != "" {
        writeln!(stdin, "position startpos moves {}", moves)?;
    }
    writeln!(stdin, "go perft {}", depth)?;

    let mut res = Vec::new();

    for line in stdout.lines().skip(1) {
        let line = line?;
        match line.trim().split(' ').collect::<Vec<&str>>()[..] {
            [mv, nodes] => {
                // eprintln!("mv: {} nodes: {}", mv, nodes);
                res.push(PerftResult {
                    mv: Movement::from_notation(&mv[0..4]).unwrap(),
                    nodes: u64::from_str(nodes)?,
                });
            }

            _ => {
                break;
            }
        }
    }

    writeln!(stdin, "quit")?;

    // TODO: Return Err instead of assert, then panic.
    let status = proc.wait()?;
    assert!(status.success(), "exit: {:?}", status);

    Ok(res)
}

// Technically I could reuse engine_perft for this, since yobmef
// prints the same way as stockfish. But that would be very brittle.
fn yobmef_perft(board: &Board, depth: u16) -> PerftResults {
    let mut res = Vec::new();

    for mv in get_legal_moves(&board) {
        let nodes = perft(&board.make_move(&mv), depth - 1);
        res.push(PerftResult { mv, nodes });
    }

    res
}

// TODO: Use uci parser instead of inlining this duplication
fn board_from_moves(moves: &str) -> Board {
    let mut board = Board::from_start_pos();
    if moves == "" {
        return board;
    }

    for mv in moves
        .trim()
        .split(' ')
        .map(|mv| Movement::from_notation(mv).unwrap())
    {
        board.make_move_mut(&mv).unwrap()
    }
    board
}

// not using get_sorted_moves from engine, because many evals in the opening
// can be equal, so we might not get the same ordering.
fn sort(res: &mut PerftResults) {
    res.sort_by_key(|r| r.mv.to_notation());
}

fn join_moves(res: &PerftResults) -> String {
    res.iter()
        .map(|r| r.mv.to_notation())
        .collect::<Vec<String>>()
        .join(" ")
}

// CURSED, OH SO CURSED

// TODO: Change moves to a Vec<Movement> and convert back to string
// in engine_perft
fn perft_drill(mut moves: String, depth: u16) {
    eprintln!("\nmoves: '{}'", moves.trim());
    eprintln!("stockfish perft({})", depth);
    let mut sf = engine_perft("stockfish", &moves, depth).expect("stockfish failed to start");
    sort(&mut sf);

    let board = board_from_moves(&moves);
    eprintln!("yobmef perft({})", depth);
    let mut ym = yobmef_perft(&board, depth);
    sort(&mut ym);

    // TODO: Instead of a Vec<PerftResult> perhaps have a struct with
    // a Vec<Movement> and Vec<u64>
    if depth == 1 {
        eprintln!("{}\n", board);

        // TODO: Add fancy diff
        eprintln!("stockfish legal: {}", join_moves(&sf));
        eprintln!("yobmef legal:    {}", join_moves(&ym));

        return;
    }

    for (r_sf, r_ym) in sf.iter().zip(ym) {
        if r_sf.nodes != r_ym.nodes {
            eprintln!("DIFF {} sf {} yobmef {}", r_sf.mv, r_sf.nodes, r_ym.nodes);

            let mv_str = r_sf.mv.to_notation();
            // cursed but ok as long as board_from_moves calls trim()
            moves.push_str(" ");
            moves.push_str(&mv_str);

            perft_drill(moves, depth - 1);

            // TODO: Ask "would you like to continue searching for diffs?"
            // and continue if they say yes.
            std::process::exit(1);
        }
    }
}

fn main() {
    gen_moves_once();

    perft_drill("".to_owned(), 6);
}
