use crate::chess::{
    board::{BoardPosition, BoardPositionOffset},
    types::ChessPiece,
};

/// a redundant QOL type useful for gui, for example to show popup screens for promotions, play visual or audio effects
/// if some special moves occur or a check happens or whatever, like knowing what kind of piece got captured or something.
/// This is to spare the poor gui implementators the work of needing to derive these purely from a [`DeltaChessBoard`]
pub struct MoveInfo {
    pub piece: ChessPiece,
    pub from: BoardPosition,
    pub to: BoardPosition,
    pub captured_piece: Option<(ChessPiece, BoardPosition)>,
    pub check_event: Option<CheckEvent>,
    /// if the move is a promotion, contains which piece it promotes to. If implementing some kind of gui for this,
    /// noticing multiple different valid moves to the same square with different promotion options, this might be a
    /// good time to show a popup to let the user decide which piece they want to promote to.
    pub promotion: Option<PromotionPiece>,
    /// if castling occured, this will contain the from-to positions for the rook
    pub castling: Option<(BoardPosition, BoardPosition)>,
    pub en_passant_occured: bool,
}

pub enum CheckEvent {
    Check,
    Checkmate,
    Stalemate,
}

pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
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
