use crate::chess::{ChessPiece, Player};

/// A chess board. Does not encode any rules, but is freely changable
#[derive(Clone)]
pub struct ChessBoard(pub [[Option<ChessBoardPiece>; 8]; 8]);

/// A zero-indexed position on the chess board
#[derive(Clone)]
pub struct BoardPosition {
    x: u8,
    y: u8,
}

#[derive(Clone, Copy)]
pub struct ChessBoardPiece {
    player: Player,
    piece: ChessPiece,
}

impl BoardPosition {
    /// returns None if the coordinates are not valid
    pub fn new(x: u8, y: u8) -> Option<Self> {
        if x > 7 && y > 7 {
            None
        } else {
            Some(Self { x, y })
        }
    }
    pub fn x(&self) -> u8 {
        self.x
    }
    pub fn y(&self) -> u8 {
        self.y
    }
}

impl ChessBoard {
    /// an empty chess board with no pieces on it
    pub const EMPTY: Self = Self([[None; 8]; 8]);

    /// create a chess board with the standard chess starting position
    pub const fn start_position() -> Self {
        use board_init::*;
        Self([
            [BR, BN, BB, BQ, BK, BB, BN, BR],
            [BP, BP, BP, BP, BP, BP, BP, BP],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [EE, EE, EE, EE, EE, EE, EE, EE],
            [WP, WP, WP, WP, WP, WP, WP, WP],
            [WR, WN, WB, WQ, WK, WB, WN, WR],
        ])
    }
}

/// shorthand piece constants to make defining hardcoded board positions in code simpler
mod board_init {
    use super::*;

    /// An empty square
    pub const EE: Option<ChessBoardPiece> = None;

    pub const BK: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::King,
    });
    pub const WK: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::King,
    });
    pub const BQ: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::Queen,
    });
    pub const WQ: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::Queen,
    });
    pub const BR: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::Rook,
    });
    pub const WR: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::Rook,
    });
    pub const BB: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::Bishop,
    });
    pub const WB: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::Bishop,
    });
    pub const BN: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::Knight,
    });
    pub const WN: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::Knight,
    });
    pub const BP: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::Black,
        piece: ChessPiece::Pawn,
    });
    pub const WP: Option<ChessBoardPiece> = Some(ChessBoardPiece {
        player: Player::White,
        piece: ChessPiece::Pawn,
    });
}
