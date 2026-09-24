use std::ops::{Add, AddAssign, Sub};

use crate::chess::types::{Player, PlayerPiece};

/// the side length of a chess board
pub const SIDE_LENGTH: usize = 8;

/// A chess board. Does not encode any rules, but is freely changable
/// Stores the squares such that indexing becomes coordinates, meaning the first array is the first vertical row to the left,
/// and goes upwards
#[derive(Clone)]
pub struct ChessBoard(pub [[ChessBoardSquare; SIDE_LENGTH]; SIDE_LENGTH]);

/// A zero-indexed position on the chess board
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardPosition {
    x: u8,
    y: u8,
}

/// a type representing an offset in a board position
#[derive(Copy, Clone)]
pub struct BoardPositionOffset {
    dx: i8,
    dy: i8,
}

/// A square on a chess board
pub type ChessBoardSquare = Option<PlayerPiece>;

/// A compact type of storing a change of a [`ChessBoard`], storing in only 17 bytes what would otherwise take 128
#[derive(Eq, PartialEq, Hash)]
pub struct DeltaChessBoard {
    // the max number of squares that can be changed by a single move is 4 (castling)
    store: [(BoardPosition, ChessBoardSquare); 4],
    len: u8,
}

impl BoardPosition {
    /// returns None if the coordinates are out of bounds
    pub fn new(x: u8, y: u8) -> Option<Self> {
        if x >= SIDE_LENGTH as u8 || y >= SIDE_LENGTH as u8 {
            None
        } else {
            Some(Self { x, y })
        }
    }
    /// creates a board position from one-indexed coordinates, and returns None if out of bounds
    pub fn from_one_indexed(x: u8, y: u8) -> Option<Self> {
        Self::new(x.checked_sub(1)?, y.checked_sub(1)?)
    }
    /// converts the board position from zero-indexed to one-indexed coordinates
    pub fn to_one_indexed(&self) -> (u8, u8) {
        (self.x + 1, self.y + 1)
    }
    /// add a board offset to the board position, returning None if it results in an invalid state
    pub fn add(self, offset: BoardPositionOffset) -> Option<Self> {
        let x = self.x as i8 + offset.dx;
        let y = self.y as i8 + offset.dy;
        if x < 0 || y < 0 || x >= SIDE_LENGTH as i8 || y >= SIDE_LENGTH as i8 {
            None
        } else {
            Some(Self {
                x: x as u8,
                y: y as u8,
            })
        }
    }
}

impl BoardPositionOffset {
    pub const ZERO: Self = Self { dx: 0, dy: 0 };
    pub const fn new(dx: i8, dy: i8) -> Self {
        Self { dx, dy }
    }

    pub const DIAGONAL_NE: Self = Self::new(1, 1);
    pub const DIAGONAL_NW: Self = Self::new(-1, 1);
    pub const DIAGONAL_SE: Self = Self::new(1, -1);
    pub const DIAGONAL_SW: Self = Self::new(-1, -1);
    pub const FORWARD: Self = Self::new(0, 1);
    pub const BACKWARD: Self = Self::new(0, -1);
    pub const RIGHT: Self = Self::new(1, 0);
    pub const LEFT: Self = Self::new(-1, 0);

    /// flip the x coordinate of the offset
    pub const fn mirror_x(&self) -> Self {
        Self {
            dx: -self.dx,
            dy: self.dy,
        }
    }
    /// flip the y coordinate of the offset
    pub const fn mirror_y(&self) -> Self {
        Self {
            dx: self.dx,
            dy: -self.dy,
        }
    }
    /// rotate the offset 90 degrees right
    pub const fn rotate_right(&self) -> Self {
        Self {
            dx: self.dy,
            dy: -self.dx,
        }
    }
    /// rotate the offset 90 degrees left
    pub const fn rotate_left(&self) -> Self {
        Self {
            dx: -self.dy,
            dy: self.dx,
        }
    }
    /// rotate the offset 180 degrees
    pub const fn rotate_180(&self) -> Self {
        Self {
            dx: -self.dx,
            dy: -self.dy,
        }
    }
}

impl ChessBoard {
    /// an empty chess board with no pieces on it
    pub const EMPTY: Self = Self([[None; SIDE_LENGTH]; SIDE_LENGTH]);

    /// create a chess board with the standard chess starting position
    pub const fn start_position() -> Self {
        use board_init::*;
        Self::from_rotated([
            [BR, BN, BB, BQ, BK, BB, BN, BR],
            [BP, BP, BP, BP, BP, BP, BP, BP],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [WP, WP, WP, WP, WP, WP, WP, WP],
            [WR, WN, WB, WQ, WK, WB, WN, WR],
        ])
    }

    /// construct a chess board from an array of arrays, and rotate it so it matches the
    /// right indexing. Useful when defining constant chess boards in code
    pub const fn from_rotated(mut rotated: [[ChessBoardSquare; SIDE_LENGTH]; SIDE_LENGTH]) -> Self {
        // diagonal starting top left until the middle
        let mut n_y = 0;
        while n_y < SIDE_LENGTH / 2 {
            // index of the row
            let mut x = 0 + n_y;
            while x < SIDE_LENGTH - 1 - n_y {
                let item = rotated[n_y][x];
                rotated[n_y][x] = rotated[SIDE_LENGTH - 1 - n_y][x];
                rotated[SIDE_LENGTH - 1 - n_y][x] =
                    rotated[SIDE_LENGTH - 1 - n_y][SIDE_LENGTH - 1 - x];
                rotated[SIDE_LENGTH - 1 - n_y][SIDE_LENGTH - 1 - x] =
                    rotated[n_y][SIDE_LENGTH - 1 - x];
                rotated[n_y][SIDE_LENGTH - 1 - x] = item;
                x += 1
            }

            n_y += 1;
        }
        Self(rotated)
    }

    /// get the square at the specified position
    pub fn get_square(&self, position: BoardPosition) -> ChessBoardSquare {
        self.0[position.x as usize][position.y as usize]
    }

    /// get all the possible board positions
    // TODO: make this an iterator instead
    pub fn squares(&self) -> Vec<BoardPosition> {
        let mut squares = Vec::with_capacity(SIDE_LENGTH.pow(2));
        for x in 0..SIDE_LENGTH {
            for y in 0..SIDE_LENGTH {
                squares.push(BoardPosition::new(x as u8, y as u8).unwrap());
            }
        }
        squares
    }

    /// get all pieces of the specified player on the board
    pub fn get_player_pieces(&self, player: Player) -> Vec<(&PlayerPiece, BoardPosition)> {
        let mut pieces = Vec::new();
        for (x, column) in self.0.iter().enumerate() {
            for (y, piece) in column.iter().enumerate() {
                if let Some(piece) = piece
                    && piece.player == player
                {
                    pieces.push((piece, BoardPosition::new(x as u8, y as u8).unwrap()))
                }
            }
        }
        pieces
    }

    /// update the chess board from a [`DeltaChessBoard`] describing the changes
    pub fn update(&mut self, delta: &DeltaChessBoard) {
        for (pos, square) in delta.iter() {
            self.0[pos.x as usize][pos.y as usize] = square;
        }
    }
}

impl DeltaChessBoard {
    pub fn new() -> Self {
        Self {
            store: [(BoardPosition::default(), None); 4],
            len: 0,
        }
    }

    pub fn insert(&mut self, position: BoardPosition, square: ChessBoardSquare) {
        if self.len == 4 {
            // you shouldnt run into this but if you do its most likely because you tried modding chess
            // to support moves that affect more than 4 positions of a chessboard per move, in which case
            // you must increase the size of DeltaChessBoard's storage, since it depends on the invariant
            // that no move affects more than 4 squares at once (castling)
            panic!("DeltaChessBoard out of bounds! Read comment above panic for more info!");
        }
        // check if position already exists in O(n^2) time (for n inserts)
        for (pos, _) in self.iter() {
            if pos == position {
                // shouldnt happen if there arent any bugs
                panic!("Uniqueness invariant in DeltaChessBoard failed.");
            }
        }
        self.store[self.len as usize] = (position, square);
        self.len += 1;
    }

    pub fn iter(&self) -> DeltaChessBoardIter {
        DeltaChessBoardIter {
            delta_board: &self,
            current_index: 0,
        }
    }
}

pub struct DeltaChessBoardIter<'a> {
    delta_board: &'a DeltaChessBoard,
    current_index: u8,
}

impl<'a> Iterator for DeltaChessBoardIter<'a> {
    type Item = (BoardPosition, ChessBoardSquare);

    fn next(&mut self) -> Option<Self::Item> {
        if self.delta_board.len == self.current_index {
            return None;
        }

        let index = self.current_index as usize;
        self.current_index += 1;
        Some(self.delta_board.store[index])
    }
}

impl AddAssign for BoardPositionOffset {
    fn add_assign(&mut self, rhs: Self) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}
impl Add for BoardPositionOffset {
    type Output = BoardPositionOffset;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}
impl Sub for BoardPositionOffset {
    type Output = BoardPositionOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
        }
    }
}
impl Sub for BoardPosition {
    type Output = BoardPositionOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            dx: self.x as i8 - rhs.x as i8,
            dy: self.y as i8 - rhs.y as i8,
        }
    }
}
impl From<BoardPosition> for BoardPositionOffset {
    fn from(value: BoardPosition) -> Self {
        Self {
            dx: value.x as i8,
            dy: value.y as i8,
        }
    }
}
impl Default for BoardPosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// shorthand piece constants to make defining hardcoded board positions in code simpler
mod board_init {
    use crate::chess::types::ChessPiece;

    use super::*;

    /// An empty square
    pub const EE: ChessBoardSquare = None;

    pub const BK: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::King,
    });
    pub const WK: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::King,
    });
    pub const BQ: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::Queen,
    });
    pub const WQ: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::Queen,
    });
    pub const BR: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::Rook,
    });
    pub const WR: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::Rook,
    });
    pub const BB: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::Bishop,
    });
    pub const WB: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::Bishop,
    });
    pub const BN: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::Knight,
    });
    pub const WN: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::Knight,
    });
    pub const BP: ChessBoardSquare = Some(PlayerPiece {
        player: Player::Black,
        piece: ChessPiece::Pawn,
    });
    pub const WP: ChessBoardSquare = Some(PlayerPiece {
        player: Player::White,
        piece: ChessPiece::Pawn,
    });
}
