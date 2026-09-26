# Chess Engine

This is a blazingly fast chess engine library written in Rust.
It aims to be a complete, bug-free implementation of all chess rules.

## Docs
Docs can be found here: [./doc/chess_rs/index.html](./doc/chess_rs/index.html).

## Testing
To test the library, run the following command in this directory
```bash
cargo test
```

## FAQ

### How do you create a new chess game?
A chess game is represented by the `chess_rs::ChessGame` type.
```rust
use chess_rs::ChessGame;

let game = ChessGame::new();
```

### How are moves represented?
In this engine, a move is uniquely represented by a change in the chessboard. The type for this is `chess_rs::board::DeltaChessBoard`. However, every move will also come with redundant derived information that might be convenient to use when building on top of the engine. The types for this are `chess_rs::move::MoveInfo` for potential moves, and `chess_rs::move::ExtendedMoveInfo` for moves that have already been made.

### How do you enumerate valid moves?
Valid moves are calculated on `ChessGame` creation, as well as every time a new move is made. To get a full list of valid moves, run `ChessGame::valid_moves` on an instance of `ChessGame`, which gives you a HashMap, mapping a valid move's `DeltaChessBoard` to the `MoveInfo`. This doesn't calculate anything, and calling this is free in terms of performance.

### How do you make a move?
To make a move you need a `DeltaChessBoard` representing the move. This can be aquired from enumerating `ChessGame::valid_moves`. If you have one, you can call `ChessGame::move` with it, and it will make the move and run all calculations. If the move you pass in is invalid, it will return an error, else it will return the `ExtendedMoveInfo` of the move you just made, as well as `Option<GameTermination>` in case the move ended the game.

### How are board positions represented?
A board position is represented by the type `chess_rs::board::BoardPosition`, containing the x and y coordinate.

### How do you enumerate the board squares?
`ChessBoard` has a method `ChessBoard::squares` that you can call to enumerate all board positions. For each of them you can call `ChessBoard::get_square` with the position, to get the square contents at that position.
