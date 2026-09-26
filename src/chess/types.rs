use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub color: Color,
    pub r#type: PieceType,
}

impl Display for ChessPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self.r#type {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };
        write!(
            f,
            "{}",
            if let Color::White = self.color {
                letter.to_ascii_uppercase()
            } else {
                letter
            }
        )
    }
}
