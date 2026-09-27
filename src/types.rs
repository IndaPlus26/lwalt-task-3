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

impl TryFrom<char> for ChessPiece {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'P' => Self {
                color: Color::White,
                r#type: PieceType::Pawn,
            },
            'p' => Self {
                color: Color::Black,
                r#type: PieceType::Pawn,
            },
            'N' => Self {
                color: Color::White,
                r#type: PieceType::Knight,
            },
            'n' => Self {
                color: Color::Black,
                r#type: PieceType::Knight,
            },
            'B' => Self {
                color: Color::White,
                r#type: PieceType::Bishop,
            },
            'b' => Self {
                color: Color::Black,
                r#type: PieceType::Bishop,
            },
            'R' => Self {
                color: Color::White,
                r#type: PieceType::Rook,
            },
            'r' => Self {
                color: Color::Black,
                r#type: PieceType::Rook,
            },
            'Q' => Self {
                color: Color::White,
                r#type: PieceType::Queen,
            },
            'q' => Self {
                color: Color::Black,
                r#type: PieceType::Queen,
            },
            'K' => Self {
                color: Color::White,
                r#type: PieceType::King,
            },
            'k' => Self {
                color: Color::Black,
                r#type: PieceType::King,
            },
            _ => {
                return Err(());
            }
        })
    }
}
