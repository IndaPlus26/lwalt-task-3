# Chess Engine

This is a chess engine library written in Rust.
It aims to be a complete, bug-free implementation of all chess rules.

## Docs
Docs can be found here: [./doc/chess_rs/index.html](./doc/chess_rs/index.html).

## Testing
To test the library, run the following command in this directory
```bash
cargo test
```

## Known limitations/bugs
For castling, the engine doesn't completely implement chess rules fully correctly yet. Everything else however, is correct.

## FAQ

### How do you create a new chess game?
A chess game is represented by the `chess_rs::ChessGame` type.
```rust
use chess_rs::ChessGame;

let game = ChessGame::new();
```

### How are moves represented?
In this engine, a move is uniquely represented by a change in the chessboard. The type for this is `chess_rs::board::DeltaChessBoard`. Alternatively a move can also be uniquely represented in a less elegant way by a combination of the position a piece moves from, the destination of that piece, as well as whether a specific promotion occured. This engine contains both types for `DeltaChessBoard` but also the redundant extra move information types `chess_rs::move::MoveInfo` for potential moves, and `chess_rs::move::ExtendedMoveInfo` for moves that have already been made.

### How do you enumerate valid moves?
Valid moves are calculated lazily and stored in a lookup table for each valid move query per square. To get the valid moves for a certain square, use `ChessGame::valid_moves`, and for all squares (not really recommended because it might bring unnecessary computation), use `ChessGame::all_valid_moves`.

### How do you make a move?
To make a move you need the from square, to square, and optional promotion information uniquely representing the move. This can be aquired from inside the MoveInfo struct from `ChessGame::valid_moves`. To make the move you need to call `ChessGame::move`. If the move you pass in is invalid, it will return an error, else it will return the `ExtendedMoveInfo` of the move you just made, as well as `Option<GameTermination>` in case the move ended the game.

### How are board positions represented?
A board position is represented by the type `chess_rs::board::BoardPosition`, containing the x and y coordinate.

### How do you enumerate the board squares?
`ChessBoard` has a method `ChessBoard::squares` that you can call to enumerate all board positions. For each of them you can call `ChessBoard::get_square` with the position, to get the square contents at that position.
