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
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn toggle(&mut self) {
        *self = self.toggled();
    }

    /// get the opposite player
    pub fn toggled(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// A piece belonging to a specific player
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub color: Color,
    pub r#type: PieceType,
}
