use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::str::FromStr;
use yobmef::chess::{Board, Movement};
use yobmef::movegen::{gen_moves_once, perft, MoveGen};

#[derive(Debug, PartialEq, Eq)]
struct PerftResult {
    mv: Movement,
    nodes: u64,
}

type PerftResults = Vec<PerftResult>;

fn engine_perft(path: &str, board: &Board, depth: u16) -> Result<PerftResults, Box<dyn Error>> {
    let mut proc = Command::new(path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;

    // TODO: Proper error handling with Option
    let mut stdin = proc.stdin.take().unwrap();
    let stdout = BufReader::new(proc.stdout.take().unwrap());

    writeln!(stdin, "position startpos fen {}", board.to_fen())?;
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

    for mv in MoveGen::new_legal(&board) {
        let nodes = perft(&board.make_move(&mv), depth - 1);
        res.push(PerftResult { mv, nodes });
    }

    res
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
fn perft_drill(board: Board, depth: u16) {
    eprintln!("stockfish perft({})", depth);
    let mut sf = engine_perft("stockfish", &board, depth).expect("stockfish failed to start");
    sort(&mut sf);

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

            perft_drill(board.make_move(&r_sf.mv), depth - 1);

            // TODO: Ask "would you like to continue searching for diffs?"
            // and continue if they say yes.
            std::process::exit(1);
        }
    }
}

fn main() {
    gen_moves_once();

    eprintln!("testing startpos");
    perft_drill(Board::from_start_pos(), 5);

    eprintln!("\ntesting the KiwiPete position");
    perft_drill(
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap(),
        5, // only 5 bc its so deeeep
    );
}
