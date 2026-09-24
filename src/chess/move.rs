use crate::chess::{
    board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard},
    types::{ChessPiece, Player, PlayerPiece},
};

// check conditions for a move: If you make the move, and then make another move of the same player, you could capture the king
// stalemate conditions: If you make a move, and for every proposed move of the other player, you can capture the king in the next move
// checkmate conditions: Both check and stalemate at the same time
//
// check conditions for a move will in compute operations the number valid moves for the same player after, O(n)
// stalemate conditions for a move will require O(n²) because one needs to calculate all valid moves for the other player
// and for each of them the valid moves for the same player
// what does this mean practically? the most legal moves for a single player possible is around 220, average is around 40 quasi legal
// moves. This means on average stalemate checks can cost up to 40^2=1600 operations. There are most likely good ways to lower
// this dramatically with dynamic programming to cache operation results that can be reused.
//
// for every single quasi legal move the entire chess board will be cloned, meaning 128 bytes being cloned, meaning for 1600
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
pub struct MoveInfo {
    pub player: Player,
    pub piece: ChessPiece,
    pub from: BoardPosition,
    pub to: BoardPosition,
    pub captured_piece: Option<(ChessPiece, BoardPosition)>,
    pub check: bool,
    /// if the move is a promotion, contains which piece it promotes to. If implementing some kind of gui for this,
    /// noticing multiple different valid moves to the same square with different promotion options, this might be a
    /// good time to show a popup to let the user decide which piece they want to promote to.
    pub promotion: Option<PromotionPiece>,
    /// if castling occured, this will contain the from-to positions for the rook
    pub castling: Option<(BoardPosition, BoardPosition)>,
    pub en_passant: bool,
}

/// Same as [`MoveInfo`] but not containing information about check. Only for internal use
/// representating the transition stage before a real [`MoveInfo`] is created
pub struct IntermediateMoveInfo {
    /// the player who executed the move
    pub player: Player,
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

pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
}

/// a path for a piece on a chess board, consisting of an offset with constant step size and direction
#[derive(Copy, Clone)]
pub struct Path {
    offset: BoardPositionOffset,
    length: PathLength,
}

/// An invalid move state
pub enum MoveError {
    /// if the piece landed on another piece of the same team
    FriendlyFire,
    /// if the piece doesn't exist
    InvalidPiece,
}

pub struct MovedPiece {
    pub board_delta: DeltaChessBoard,
    pub captured_piece: Option<ChessPiece>,
    pub player: Player,
    pub piece: ChessPiece,
}

// reason i made this take in the entire chess board is bc i want to fetch the piece directly from there,
// to avoid redundant state that may contain errors or be out of sync. so i thought i might as well keep
// track of the captured status here as well, otherwise i'd have derived that from a DeltaBoard alone
/// Move (or teleport) a piece from an origin to a destination.
/// Only accounts for direct teleport moves, aka a piece swapping position for the one at the specified offset
/// Returns Ok if the move was valid, as well as the delta chess board and optionally if it captured a piece there
pub fn move_piece(
    board: &ChessBoard,
    origin: BoardPosition,
    destination: BoardPosition,
) -> Result<MovedPiece, MoveError> {
    let Some(player_piece) = board.get_square(origin) else {
        return Err(MoveError::InvalidPiece);
    };
    let mut board_delta = DeltaChessBoard::new();
    // make origin square empty
    board_delta.insert(origin, None);
    // put piece at destination square
    board_delta.insert(destination, Some(player_piece));
    Ok(MovedPiece {
        board_delta,
        captured_piece: match board.get_square(destination) {
            Some(existing_piece) => {
                if existing_piece.player == player_piece.player {
                    return Err(MoveError::FriendlyFire);
                }
                Some(existing_piece.piece)
            }
            None => None,
        },
        player: player_piece.player,
        piece: player_piece.piece,
    })
}

impl Path {
    pub fn new(offset: BoardPositionOffset, length: PathLength) -> Self {
        Self { offset, length }
    }

    pub fn into_iter(self) -> PathIter {
        PathIter {
            path: self,
            current_offset: BoardPositionOffset::ZERO,
            current_iteration: 0,
        }
    }

    pub fn moves<'a>(
        self,
        board: &'a ChessBoard,
        position: BoardPosition,
    ) -> impl Iterator<Item = (DeltaChessBoard, IntermediateMoveInfo)> + 'a {
        self.into_iter().map_while(move |offset| {
            let Some(destination) = position.add(offset) else {
                return None; // if the position is out of bounds end the path
            };
            let Ok(MovedPiece {
                board_delta,
                captured_piece,
                player,
                piece,
            }) = move_piece(&board, position, destination)
            else {
                return None; // if the poisition is occupied by a same team piece, end the path
            };
            // else return the move
            Some((
                board_delta,
                IntermediateMoveInfo {
                    player,
                    piece,
                    from: position,
                    to: destination,
                    captured_piece: captured_piece
                        .map(|captured_piece| (captured_piece, destination)),
                    promotion: None,
                    castling: None,
                    en_passant: false,
                    disabled_castling: (None, None),
                    set_en_passant_square: None,
                },
            ))
        })
    }
}

/// an iterator for a [`Path`] that returns the position offsets of the path in order
pub struct PathIter {
    path: Path,
    current_offset: BoardPositionOffset,
    current_iteration: u8,
}

impl Iterator for PathIter {
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

/// the length of the path
#[derive(Copy, Clone)]
pub enum PathLength {
    Fixed(u8),
    Infinite,
}
