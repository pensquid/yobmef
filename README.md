# yobmef

A bad UCI chess engine written from complete scratch in Rust

## TODO

- [x] Factor `Square`, `Piece`, `Color`, `Movement` and `Board` to separate files
- [ ] Add UCI tests
- [ ] Optimize pruning
- [ ] Transposition tables
  - [x] Zobrist hashing
- [x] Finish movegen
  - [x] Pawns
    - [x] Pushing
    - [x] Captures
    - [x] Promotions
    - [x] En passant
  - [x] Kings
    - [x] Moves
    - [x] Cannot move into check
  - [x] If in check, you must get out of check
  - [x] Knights
  - [x] Rooks
  - [x] Bishops
  - [x] Queens
  - [x] Castling
