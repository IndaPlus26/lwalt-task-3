use crate::chess::{
    board::{BoardPosition, ChessBoard, DeltaChessBoard},
    types::{ChessPiece, PieceType},
};

// check conditions for a move: If you make the move, and then make another move of the same player, you could capture the king
// stalemate conditions: If you make a move, and for every proposed move of the other player, you can capture the king in the next move
// checkmate conditions: Both check and stalemate at the same time
//
// check conditions for a move will in compute operations the number valid moves for the same player after, O(n)
// stalemate conditions for a move will require O(n²) because one needs to calculate all valid moves for the other player
// and for each of them the valid moves for the same player
// what does this mean practically? the most legal moves for a single player possible is around 220, average is around 40 pseudo legal
// moves. This means on average stalemate checks can cost up to 40^2=1600 operations. There are most likely good ways to lower
// this dramatically with dynamic programming to cache operation results that can be reused.
//
// for every single pseudo legal move the entire chess board will be cloned, meaning 128 bytes being cloned, meaning for 1600
// operations this goes up to 200kb being cloned checking if a single move causes a stalemate, and for 40 proposed moves this
// means 8mb (!) of cloning every single turn. Im therefore choosing to not evaluate stalemate/checkmate for each move, only check,
// and i'll let the stalemate condition be decided by if the move after has no legal moves, which is only O(n) and managable.
//
// In fact, we could even make the check check only happen once a move is made, theres not really any good reason to calculate it
// before for every potential move.
//
/// a redundant QOL type useful for gui, for example to show popup screens for promotions, play visual or audio effects
/// if some special moves occur or a check happens or whatever, like knowing what kind of piece got captured or something.
/// This is to spare the poor gui implementators the work of needing to derive these purely from a board diff
#[derive(Clone)]
pub struct ExtendedMoveInfo {
    /// the piece that was moved
    pub piece: ChessPiece,
    /// where from the piece was moved
    pub from: BoardPosition,
    /// where the piece was moved to
    pub to: BoardPosition,
    /// if the move captures a piece, and on which position that piece used to stand
    pub captured_piece: Option<(ChessPiece, BoardPosition)>,
    /// if the move is a promotion, contains which piece it promotes to. If implementing some kind of gui for this,
    /// noticing multiple different valid moves to the same square with different promotion options, this might be a
    /// good time to show a popup to let the user decide which piece they want to promote to.
    pub promotion: Option<PromotionPiece>,
    /// if castling occured, this contains the from-to positions for the rook
    pub castling: Option<(BoardPosition, BoardPosition)>,
    /// if the move executed was en passant
    pub en_passant: bool,
    /// if the move disabled castling on any of the castling sides of the current player who executed the move.
    /// this being None does not imply castling is enabled, just that no change occured during this move
    pub disabled_castling: (Option<()>, Option<()>),
    /// if the move was a 2 step pawn move and enabled a potential en passant opportunity on the specified square behind it
    pub set_en_passant_square: Option<BoardPosition>,
    /// the check event of the move
    pub check_event: Option<CheckEvent>,
}

#[derive(Clone)]
pub enum CheckEvent {
    Check,
    Stalemate,
    Checkmate,
}

/// Same as [`ExtendedMoveInfo`] but not containing information about check. Only for internal use
/// representating the transition stage before a real [`ExtendedMoveInfo`] is created
#[derive(Clone)]
pub struct MoveInfo {
    /// the piece that was moved
    pub piece: ChessPiece,
    /// where from the piece was moved
    pub from: BoardPosition,
    /// where the piece was moved to
    pub to: BoardPosition,
    /// if the move captures a piece, and on which position that piece used to stand
    pub captured_piece: Option<(ChessPiece, BoardPosition)>,
    /// if the move is a promotion, contains which piece it promotes to. If implementing some kind of gui for this,
    /// noticing multiple different valid moves to the same square with different promotion options, this might be a
    /// good time to show a popup to let the user decide which piece they want to promote to.
    pub promotion: Option<PromotionPiece>,
    /// if castling occured, this contains the from-to positions for the rook
    pub castling: Option<(BoardPosition, BoardPosition)>,
    /// if the move executed was en passant
    pub en_passant: bool,
    /// if the move disabled castling on any of the castling sides of the current player who executed the move.
    /// this being None does not imply castling is enabled, just that no change occured during this move
    pub disabled_castling: (Option<()>, Option<()>),
    /// if the move was a 2 step pawn move and enabled a potential en passant opportunity on the specified square behind it
    pub set_en_passant_square: Option<BoardPosition>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl From<PromotionPiece> for PieceType {
    fn from(value: PromotionPiece) -> Self {
        match value {
            PromotionPiece::Queen => PieceType::Queen,
            PromotionPiece::Rook => PieceType::Rook,
            PromotionPiece::Bishop => PieceType::Bishop,
            PromotionPiece::Knight => PieceType::Knight,
        }
    }
}

impl ExtendedMoveInfo {
    pub fn new(move_info: MoveInfo, check_event: Option<CheckEvent>) -> Self {
        Self {
            piece: move_info.piece,
            from: move_info.from,
            to: move_info.to,
            captured_piece: move_info.captured_piece,
            promotion: move_info.promotion,
            castling: move_info.castling,
            en_passant: move_info.en_passant,
            disabled_castling: move_info.disabled_castling,
            set_en_passant_square: move_info.set_en_passant_square,
            check_event,
        }
    }
}

/// No piece at the specified origin position exists
pub struct PieceDoesNotExist;

/// A type representing a piece having moved from one square to another
pub struct MovedPiece {
    // /// The piece which moved
    // pub moved_piece: ChessPiece,
    /// The delta chess board that represents the move
    pub board_delta: DeltaChessBoard,
    /// Optionally the piece it replaced on the destination square
    pub replaced_piece: Option<ChessPiece>,
}

/// Get a representation of a piece that has moved (or teleported) from an origin square to a destination.
/// Only accounts for direct teleport moves, aka a piece teleporting to the specified destination
/// Will not account for color of the piece replaced, and a check for capture or if the move is valid must
/// be done outside this function
pub fn move_piece(
    board: &ChessBoard,
    origin: BoardPosition,
    destination: BoardPosition,
) -> Result<MovedPiece, PieceDoesNotExist> {
    let Some(piece) = board.get_square(origin) else {
        return Err(PieceDoesNotExist);
    };
    let mut board_delta = DeltaChessBoard::new();
    // make origin square empty
    board_delta.insert(origin, None);
    // put piece at destination square
    board_delta.insert(destination, Some(piece));
    Ok(MovedPiece {
        board_delta,
        replaced_piece: board.get_square(destination),
    })
}
