use yobmef::*;
mod chess;

fn main() {
  let mut engine = Engine::new();
  if let Err(e) = engine.uci_loop() {
    eprintln!("{}", e);
  }
}