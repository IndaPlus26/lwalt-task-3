use crate::chess::{
    board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard},
    types::{ChessPiece, Player},
};

/// a redundant QOL type useful for gui, for example to show popup screens for promotions, play visual or audio effects
/// if some special moves occur or a check happens or whatever, like knowing what kind of piece got captured or something.
/// This is to spare the poor gui implementators the work of needing to derive these purely from a board diff
pub struct MoveInfo {
    pub player: Player,
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
    pub en_passant: bool,
}

/// same as [`MoveInfo`] but not containing information about check/checkmate/stalemate. Only for internal use
/// representating the transition stage before a real [`MoveInfo`] is created
pub struct IntermediateMoveInfo {
    pub player: Player,
    pub piece: ChessPiece,
    pub from: BoardPosition,
    pub to: BoardPosition,
    pub captured_piece: Option<(ChessPiece, BoardPosition)>,
    pub promotion: Option<PromotionPiece>,
    pub castling: Option<(BoardPosition, BoardPosition)>,
    pub en_passant: bool,
}

pub enum CheckEvent {
    Check,
    Checkmate,
    Stalemate,
}

impl MoveInfo {
    pub fn is_termination(&self) -> bool {
        self.check_event
            .as_ref()
            .is_some_and(|event| event.is_termination())
    }
}
impl CheckEvent {
    pub fn is_termination(&self) -> bool {
        match self {
            CheckEvent::Check => false,
            CheckEvent::Checkmate => true,
            CheckEvent::Stalemate => true,
        }
    }
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

/// If a piece was captured or nothing happneed
pub enum MoveType {
    Captured(ChessPiece),
    Moved,
}
pub enum MoveError {
    /// if the piece landed on another piece of the same team
    FriendlyFire,
    /// if the piece doesn't exist
    InvalidPiece,
}
// reason i made this take in the entire chess board is bc i want to fetch the piece directly from there,
// to avoid redundant state that may contain errors or be out of sync. so i thought i might as well keep
// track of the captured status here as well, otherwise i'd have derived that from a DeltaBoard alone
/// Move (or teleport) a piece from an origin to an offset.
/// Only accounts for direct teleport moves, aka a piece swapping position for the one at the specified offset
pub fn move_piece(
    board: &ChessBoard,
    origin: BoardPosition,
    destination: BoardPosition,
) -> Result<(DeltaChessBoard, MoveType), MoveError> {
    let Some(piece) = board.get_square(origin) else {
        return Err(MoveError::InvalidPiece);
    };
    let mut delta = DeltaChessBoard::new();
    // make origin square empty
    delta.insert(origin, None);
    // put piece at destination square
    delta.insert(destination, Some(piece));
    Ok((
        delta,
        // check if existing piece previously on destination square
        match board.get_square(destination) {
            Some(existing_piece) => {
                if existing_piece.player == piece.player {
                    return Err(MoveError::FriendlyFire);
                }
                MoveType::Captured(existing_piece.piece)
            }
            None => MoveType::Moved,
        },
    ))
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
