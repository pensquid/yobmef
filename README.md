## yobmef

A bad UCI chess engine written from complete scratch in Rust

## TODO

- [x] Factor `Square`, `Piece`, `Color`, `Movement` and `Board` to separate files
- [ ] Add UCI tests
- [ ] Finish movegen
  - [x] Pawns
    - [x] Pushing
    - [x] Captures
    - [x] Promotions
    - [x] En passant
  - [ ] Kings
    - [x] Moves
    - [ ] Cannot move into check
  - [ ] If in check, you must get out of check
  - [x] Knights
  - [x] Rooks
  - [x] Bishops
  - [x] Queens
  - [ ] Castling

