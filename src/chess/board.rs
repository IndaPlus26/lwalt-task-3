use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, Sub},
};

use crate::chess::types::{ChessPiece, Color, PieceType};

/// the side length of a chess board
pub const SIDE_LENGTH: usize = 8;

/// A chess board. Does not encode any rules, but is freely changable
/// Stores the squares such that indexing becomes coordinates, meaning the first array is the first vertical row to the left,
/// and goes upwards
#[derive(Clone)]
pub struct ChessBoard(pub [[ChessBoardSquare; SIDE_LENGTH]; SIDE_LENGTH]);

/// A zero-indexed position on the chess board
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
pub type ChessBoardSquare = Option<ChessPiece>;

/// A compact type of storing a change of a [`ChessBoard`], storing in only 17 bytes what would otherwise take 128
#[derive(Clone, Eq, PartialEq, Hash)]
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
    /// add a board offset to the board position, returning None if it lands out of bounds
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

    /// get the x coordinate
    pub fn x(&self) -> u8 {
        self.x
    }
    /// get the y coordinate
    pub fn y(&self) -> u8 {
        self.y
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
    pub const NORTH: Self = Self::new(0, 1);
    pub const SOUTH: Self = Self::new(0, -1);
    pub const EAST: Self = Self::new(1, 0);
    pub const WEST: Self = Self::new(-1, 0);

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
    pub fn squares(&self) -> impl Iterator<Item = BoardPosition> {
        (0..SIDE_LENGTH).flat_map(|x| {
            (0..SIDE_LENGTH).map(move |y| BoardPosition::new(x as u8, y as u8).unwrap())
        })
    }

    /// get all pieces of the specified color on the board
    pub fn get_player_pieces(&self, color: Color) -> Vec<(&ChessPiece, BoardPosition)> {
        let mut pieces = Vec::new();
        for (x, column) in self.0.iter().enumerate() {
            for (y, piece) in column.iter().enumerate() {
                if let Some(piece) = piece
                    && piece.color == color
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

    /// overwrites the square if it already exists in the [`DeltaChessBoard`]
    pub fn insert(&mut self, position: BoardPosition, square: ChessBoardSquare) {
        match (&mut self.store[..self.len as usize])
            .binary_search_by_key(&position, |(pos, _)| *pos)
        {
            // if the position already exists, overwrite it. this doesn't increase the length
            Ok(square_index) => {
                self.store[square_index].1 = square;
            }
            // else try insert it
            Err(insert_index) => {
                if self.len == 4 {
                    // you shouldnt run into this but if you do its most likely because you tried modding chess
                    // to support moves that affect more than 4 positions of a chessboard per move, in which case
                    // you must increase the size of DeltaChessBoard's storage, since it depends on the invariant
                    // that no move affects more than 4 squares at once (castling)
                    panic!(
                        "DeltaChessBoard out of bounds! Read comment above panic for more info!"
                    );
                }
                // push all the elements one step and insert
                self.store[..self.len as usize].copy_within(insert_index.., insert_index + 1);
                self.store[insert_index] = (position, square);
                self.len += 1;
            }
        }
    }

    pub fn iter<'a>(&'a self) -> DeltaChessBoardIter<'a> {
        DeltaChessBoardIter {
            delta_board: &self,
            current_index: 0,
        }
    }
    pub fn iter_mut<'a>(&'a mut self) -> DeltaChessBoardIterMut<'a> {
        DeltaChessBoardIterMut(self.store[0..self.len as usize].iter_mut())
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
pub struct DeltaChessBoardIterMut<'a>(std::slice::IterMut<'a, (BoardPosition, ChessBoardSquare)>);

impl<'a> Iterator for DeltaChessBoardIterMut<'a> {
    type Item = &'a mut (BoardPosition, ChessBoardSquare);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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

impl Mul<i8> for BoardPositionOffset {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        Self {
            dx: self.dx * rhs,
            dy: self.dx * rhs,
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

impl Display for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (SIDE_LENGTH - 1)..=0 {
            for x in 0..SIDE_LENGTH {
                write!(
                    f,
                    "{}",
                    match self.0[x][y] {
                        None => '#',
                        Some(piece) => match piece {
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::Pawn,
                            } => 'P',
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::Knight,
                            } => 'N',
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::Bishop,
                            } => 'B',
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::Rook,
                            } => 'R',
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::Queen,
                            } => 'Q',
                            ChessPiece {
                                color: Color::White,
                                r#type: PieceType::King,
                            } => 'K',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::Pawn,
                            } => 'p',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::Knight,
                            } => 'n',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::Bishop,
                            } => 'b',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::Rook,
                            } => 'r',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::Queen,
                            } => 'q',
                            ChessPiece {
                                color: Color::Black,
                                r#type: PieceType::King,
                            } => 'k',
                        },
                    }
                )?;
            }
            write!(f, "\n")?;
        }
        Ok(())
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
    use crate::chess::types::PieceType;

    use super::*;

    /// An empty square
    pub const EE: ChessBoardSquare = None;

    pub const BK: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::King,
    });
    pub const WK: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::King,
    });
    pub const BQ: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::Queen,
    });
    pub const WQ: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::Queen,
    });
    pub const BR: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::Rook,
    });
    pub const WR: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::Rook,
    });
    pub const BB: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::Bishop,
    });
    pub const WB: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::Bishop,
    });
    pub const BN: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::Knight,
    });
    pub const WN: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::Knight,
    });
    pub const BP: ChessBoardSquare = Some(ChessPiece {
        color: Color::Black,
        r#type: PieceType::Pawn,
    });
    pub const WP: ChessBoardSquare = Some(ChessPiece {
        color: Color::White,
        r#type: PieceType::Pawn,
    });
}
