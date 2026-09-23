use crate::chess::{
    board::{BoardPosition, BoardPositionOffset, DeltaChessBoard},
    types::ChessPiece,
};

/// a redundant type describing a chess move, useful for visual
pub struct Move {
    pub piece: ChessPiece,
    pub from: BoardPosition,
    pub to: BoardPosition,
    pub change: DeltaChessBoard,
}

/// a path for a piece on a chess board, consisting of an offset with constant step size and direction
pub struct Path {
    offset: BoardPositionOffset,
    length: PathLength,
    r#type: PathType,
}

impl Path {
    pub fn new(offset: BoardPositionOffset, length: PathLength, r#type: PathType) -> Self {
        Self {
            offset,
            length,
            r#type,
        }
    }

    pub fn iter(&self) -> PathIter {
        PathIter {
            path: &self,
            current_offset: BoardPositionOffset::ZERO,
            current_iteration: 0,
        }
    }
}

/// an iterator for a [`Path`] that returns the position offsets of the path in order
pub struct PathIter<'a> {
    path: &'a Path,
    current_offset: BoardPositionOffset,
    current_iteration: u8,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = BoardPositionOffset;

    fn next(&mut self) -> Option<Self::Item> {
        if let PathLength::Fixed(len) = self.path.length
            && len == self.current_iteration
        {
            return None;
        }

        self.current_iteration += 1;
        self.current_offset += self.path.offset;
        Some(self.current_offset)
    }
}

/// whether the piece can capture enemy pieces on the path or if blocked by them
pub enum PathType {
    Capture,
    Block,
}

/// the length of the path
pub enum PathLength {
    Fixed(u8),
    Infinite,
}
