#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum ChessPiece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn toggle(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

/// A piece belonging to a specific player
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerPiece {
    pub player: Player,
    pub piece: ChessPiece,
}
