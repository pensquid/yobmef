use yobmef::*;

fn main() {
    movegen::gen_moves_once();

    let mut engine = Engine::new();
    if let Err(e) = engine.uci_loop() {
        eprintln!("{}", e);
    }
}
