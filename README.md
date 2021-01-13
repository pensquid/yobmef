## yobmef

A chess engine in rust


## TODO

- [ ] Refactor make_move_mut to return a `Result`
- [ ] Factor `Square`, `Piece`, `Color`, `Movement` and `Board` to separate files
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
  - [ ] Rooks
  - [ ] Bishops
  - [ ] Queens
  - [ ] Castling

