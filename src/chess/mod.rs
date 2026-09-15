use std::collections::HashSet;

use crate::chess::board::{BoardPosition, ChessBoard};

pub mod board;

#[derive(Clone, Copy)]
pub enum ChessPiece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone)]
pub enum Player {
    White,
    Black,
}

pub struct Move {
    piece: ChessPiece,
    position: BoardPosition,
}
/// a chess game state machine
pub struct ChessGame {
    /// the initial game state
    init_state: GameState,
    /// all moves made since the initial state in chronological order
    moves: Vec<PlayerMove>,

    // --- redundant state for simpler computation ---
    // (TODO maybe make them cache lookups that just store queries from the init_board+init_state+moves instead)
    /// the current state of the game
    state: GameState,
}

#[derive(Clone)]
pub struct GameState {
    board: ChessBoard,
    state: PositionState,
}

impl GameState {
    /// returns if the game has ended, and if so what the result was
    pub fn get_termination(&self) -> Option<GameResult> {
        todo!()
    }
}

// contains only the chess derived results, not actions like giving up or agreeing on draw. This should be handled by a higher level type
// that contains this base game state machine and also implementation specific things
pub enum GameResult {
    CheckMate(Player),
    StaleMate(Player),

    FiftyMoveRule,
    ThreeMoveRule,
}

/// the current position state of a chess game
#[derive(Clone)]
pub struct PositionState {
    player_at_move: Player,
    /// (white, black)
    can_castle: (bool, bool),
    /// a square where en passant is possible (if a pawn moved past it the previous move)
    en_passant_square: Option<BoardPosition>,
    /// the number of halfmoves since the last capture or pawn advance
    halfmove_clock: u8,
    /// the number of full moves. is 1-indexed for some reason
    n_fullmoves: u16,
}

impl Default for PositionState {
    fn default() -> Self {
        Self {
            player_at_move: Player::White,
            can_castle: (true, true),
            en_passant_square: None,
            halfmove_clock: 0,
            n_fullmoves: 1,
        }
    }
}

impl ChessGame {
    pub fn new() -> Self {
        let state = GameState {
            board: ChessBoard::start_position(),
            state: PositionState::default(),
        };
        Self {
            init_state: state.clone(),
            moves: Vec::new(),

            state,
        }
    }

    /// import a chess game from a chess fen (standardized compact chess position format).
    pub fn from_fen(&self, fen: &str) -> Option<Self> {
        todo!()
    }

    /// returns None if the game has ended
    pub fn turn(&self) -> Option<Player> {
        if self.state.get_termination().is_some() {
            None
        } else {
            Some(
                self.moves
                    .last()
                    .map(|r#move| r#move.player)
                    .unwrap_or(Player::White),
            )
        }
    }

    pub fn valid_moves(&self, player: Player) -> HashSet<PlayerMove> {
        todo!()
    }

    /// if the move is invalid this returns a [`MoveError`], else the move is applied
    pub fn try_move(&mut self, r#move: PlayerMove) -> Result<(), MoveError> {
        todo!()
    }
}

pub enum MoveError {
    NotInTurn,
    InvalidMove,
}

pub struct PlayerMove {
    player: Player,
    r#move: Move,
}
