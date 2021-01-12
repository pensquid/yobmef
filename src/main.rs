use yobmef::*;

fn main() {
    movegen::gen_moves();

    let mut engine = Engine::new();
    if let Err(e) = engine.uci_loop() {
        eprintln!("{}", e);
    }
}
