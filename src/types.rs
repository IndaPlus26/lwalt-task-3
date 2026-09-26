use std::fmt::Display;

/// The type of a chess piece
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// The color of a chess piece, also representing a player
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Toggle the color, aka make it the opposite color
    pub fn toggle(&mut self) {
        *self = self.toggled();
    }

    /// Get the toggled color, aka opposite color
    pub fn toggled(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// A chess piece with a color and a type
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
