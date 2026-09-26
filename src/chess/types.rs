#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceType {
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
    pub fn toggle(&mut self) {
        *self = self.toggled();
    }

    /// get the opposite player
    pub fn toggled(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

/// A piece belonging to a specific player
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub color: Player,
    pub r#type: PieceType,
}
