#[derive(Clone, Copy)]
pub enum ChessPiece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Player {
    White,
    Black,
}

/// A piece of a specific player
#[derive(Clone, Copy)]
pub struct PlayerPiece {
    pub player: Player,
    pub piece: ChessPiece,
}
