use std::collections::HashSet;

use crate::chess::board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard};
use crate::chess::r#move::{Path, PathLength, PathType};
use crate::chess::types::{ChessPiece, Move, Player, PlayerPiece};

pub mod board;
pub mod r#move;
pub mod types;

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

// TODO: a chess piece's action will be defined by a function that takes in the current game state and returns a set
// of squares it can move to, and for each of those also optionally additional effects. for example the en passant move will have the additional
// effect of capturing the pawn, or the castling move will have the additional effect of moving the rook along with the king, or the promotion
// move will have the additional effect of promoting to some piece (last one is questionable)
//
// if a move happens the game will check if the move captured a piece, if it was a castling or king move (to disable the castling available flag),
// if it was a promotion, if it was a pawn moving 2 steps forward (enables en passant), etc

impl GameState {
    /// returns if the game has ended, and if so what the result was
    pub fn get_termination(&self) -> Option<GameResult> {
        todo!()
    }

    pub fn is_check(&self) -> Option<Player> {
        todo!()
    }

    pub fn valid_moves(&self) -> HashSet<PlayerMove> {
        let player = self.state.player_at_turn;

        // TODO: check if player in check (meaning they are forced to move into a position where they're not in check)

        for (piece, position) in self.board.get_player_pieces(&player) {
            // check all available moves and return them
        }

        todo!()
    }
}

/// get the valid moves for a piece on the board
pub fn valid_moves(
    piece: &PlayerPiece,
    pos: BoardPosition,
    state: &GameState,
) -> HashSet<Move, DeltaChessBoard> {
    let mut moves = HashSet::new();
    match piece {
        ChessPiece::Pawn => {
            moves.insert(Path::new(
                BoardPositionOffset::FORWARD,
                PathLength::Fixed(2),
                PathType::Block,
            ));
            // add diagonal capture and en passant
        }
        ChessPiece::Knight => {
            let offset = BoardPositionOffset::new(1, 2);
            offset.rotate_right();
            offset.rotate_right();
            offset.rotate_right();
            offset.mirror_x();
            offset.rotate_right();
            offset.rotate_right();
            offset.rotate_right();
        }
        ChessPiece::Bishop => todo!(),
        ChessPiece::Rook => todo!(),
        ChessPiece::Queen => todo!(),
        ChessPiece::King => todo!(),
    }
    moves
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
    player_at_turn: Player,
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
            player_at_turn: Player::White,
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
