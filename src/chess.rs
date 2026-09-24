use std::iter;

use crate::chess::board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard};
use crate::chess::r#move::{
    IntermediateMoveInfo, MoveInfo, MovedPiece, Path, PathLength, move_piece,
};
use crate::chess::types::{ChessPiece, Player, PlayerPiece};

pub mod board;
pub mod r#move;
#[cfg(test)]
mod tests;
pub mod types;

/// A chess game state machine
/// Does not include manual terminations like resigning or offering/accepting draw
pub struct ChessGame {
    /// the initial game state
    init_state: GameState,
    /// all moves made since the initial state in chronological order
    moves: Vec<(DeltaChessBoard, MoveInfo)>,

    // (FEATURE: maybe make all redundant sate cache lookups that just store queries from the init_board+init_state+moves instead)
    // --- redundant state for simpler computation ---
    /// this doesn't mean the game termination can't be checkmate, to check that, it is required to check if the last
    /// move was a check or not. Just to check, don't forget to check for the check.
    is_stalemate: bool,

    /// the current state of the game
    state: GameState,
}

#[derive(Clone)]
pub struct GameState {
    board: ChessBoard,
    state: PositionState,
}

// TODO: actually for this entire codebase, refactor the code to follow the rule of every function always staying on
// the same abstraction layer
impl GameState {
    /// Get all the valid moves for a piece, including with check rules
    /// The output iterator guarantees a uniqueness invariant
    pub fn valid_moves<'a>(
        &'a self,
        pos: BoardPosition,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, IntermediateMoveInfo)> + 'a> {
        Box::new(self.quasi_valid_moves(pos).filter(|(board_delta, _)| {
            let mut board = self.board.clone();
            board.update(&board_delta);

            // validitiy check
            // check the opponents next valid moves
            let mut state = self.state.clone();
            state.player_at_turn.toggle();
            let board_squares = board.squares();
            let game_state = GameState { board, state }; // both update board and toggle player
            for (_, intermediate_move_info) in board_squares
                .iter()
                .flat_map(|position| game_state.quasi_valid_moves(*position))
            {
                // if one of them captures your king, the proposed move is invalid and filtered out
                if let Some((ChessPiece::King, _)) = intermediate_move_info.captured_piece {
                    return false;
                }
            }
            return true;
        }))
    }

    // OPTIMIZATION: this could instead be a lazily computed iterator, which would need not needing to store the entire Vec, since
    // basically all use cases only require sequiental access. Could be useful in cases of early return, like if you find a move that
    // captured the king you dont need to check further.
    /// Get the valid moves for a piece on the board, only according to the piece's move rules, and no check rules.
    /// This means it may include invalid moves that enable the king being captured in the next move.
    ///
    /// The output iterator has a uniqueness invariant
    pub fn quasi_valid_moves<'a>(
        &'a self,
        position: BoardPosition,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, IntermediateMoveInfo)> + 'a> {
        // if the selected piece doesnt exist or belong to the player in turn
        let Some(PlayerPiece { player, piece }) = self.board.get_square(position) else {
            return Box::new(iter::empty());
        };
        if player != self.state.player_at_turn {
            return Box::new(iter::empty());
        }
        match piece {
            // make this in the case of promotion return one move for every promotion variant, such that only the DeltaChessBoard differs
            ChessPiece::Pawn => {
                // add diagonal capture and en passant and initial 2 jump and promotion
                todo!()
            }
            ChessPiece::Knight => {
                //  # #
                // #   #
                //   O
                // #   #
                //  # #
                let offsets = [
                    BoardPositionOffset::new(1, 2),
                    BoardPositionOffset::new(1, -2),
                    BoardPositionOffset::new(-1, 2),
                    BoardPositionOffset::new(-1, -2),
                    BoardPositionOffset::new(2, 1),
                    BoardPositionOffset::new(2, -1),
                    BoardPositionOffset::new(-2, 1),
                    BoardPositionOffset::new(-2, -1),
                ];
                return Box::new(offsets.into_iter().filter_map(move |offset| {
                    if let Some(destination) = position.add(offset)
                        && let Ok(MovedPiece {
                            board_delta,
                            captured_piece,
                            player,
                            piece,
                        }) = move_piece(&self.board, position, destination)
                    {
                        Some((
                            board_delta,
                            IntermediateMoveInfo {
                                player,
                                piece,
                                from: position,
                                to: destination,
                                captured_piece: captured_piece
                                    .map(|captured_piece| (captured_piece, destination)),
                                promotion: None,
                                castling: None,
                                en_passant: false,
                                disabled_castling: (None, None),
                                set_en_passant_square: None,
                            },
                        ))
                    } else {
                        None
                    }
                }));
            }
            ChessPiece::Bishop => {
                // #   #
                //  # #
                //   O
                //  # #
                // #   #

                let directions = [
                    BoardPositionOffset::DIAGONAL_NE,
                    BoardPositionOffset::DIAGONAL_NW,
                    BoardPositionOffset::DIAGONAL_SW,
                    BoardPositionOffset::DIAGONAL_SE,
                ];
                return Box::new(directions.into_iter().flat_map(move |direction| {
                    Path::new(direction, PathLength::Infinite).moves(&self.board, position)
                }));
            }
            ChessPiece::Rook => {
                //   #
                //   #
                // ##O##
                //   #
                //   #

                let directions = [
                    BoardPositionOffset::FORWARD,
                    BoardPositionOffset::BACKWARD,
                    BoardPositionOffset::LEFT,
                    BoardPositionOffset::RIGHT,
                ];
                return Box::new(directions.into_iter().flat_map(move |direction| {
                    Path::new(direction, PathLength::Infinite).moves(&self.board, position)
                }));
            }
            ChessPiece::Queen => {
                // # # #
                //  ###
                // ##O##
                //  ###
                // # # #

                let directions = [
                    BoardPositionOffset::DIAGONAL_NE,
                    BoardPositionOffset::DIAGONAL_NW,
                    BoardPositionOffset::DIAGONAL_SW,
                    BoardPositionOffset::DIAGONAL_SE,
                    BoardPositionOffset::FORWARD,
                    BoardPositionOffset::BACKWARD,
                    BoardPositionOffset::LEFT,
                    BoardPositionOffset::RIGHT,
                ];
                return Box::new(directions.into_iter().flat_map(move |direction| {
                    Path::new(direction, PathLength::Infinite).moves(&self.board, position)
                }));
            }
            ChessPiece::King => {
                //
                //  ###
                //  #O#
                //  ###
                //

                let offsets = [
                    BoardPositionOffset::DIAGONAL_NE,
                    BoardPositionOffset::DIAGONAL_NW,
                    BoardPositionOffset::DIAGONAL_SW,
                    BoardPositionOffset::DIAGONAL_SE,
                    BoardPositionOffset::FORWARD,
                    BoardPositionOffset::BACKWARD,
                    BoardPositionOffset::LEFT,
                    BoardPositionOffset::RIGHT,
                ];
                // for offset in offsets {
                //     if let Some(destination) = position.add(offset)
                //         && let Ok((delta_board, r#type)) =
                //             move_piece(&self.board, position, destination)
                //     {
                //         moves.insert(
                //             delta_board,
                //             IntermediateMoveInfo {
                //                 player,
                //                 piece,
                //                 from: position,
                //                 to: destination,
                //                 captured_piece: match r#type {
                //                     MoveType::Captured(chess_piece) => {
                //                         Some((chess_piece, destination))
                //                     }
                //                     MoveType::Moved => None,
                //                 },
                //                 promotion: None,
                //                 castling: None,
                //                 en_passant: false,
                //             },
                //         );
                //     }
                // }

                // add castling
                todo!()
            }
        }
    }
}

// pub fn moves_from_path(board: )

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
    /// (king-side, queen-side)
    white_can_castle: (bool, bool),
    black_can_castle: (bool, bool),
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
            white_can_castle: (true, true),
            black_can_castle: (true, true),
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
            is_stalemate: false,

            state,
        }
    }

    /// import a chess game from a chess fen (standardized compact chess position format).
    pub fn from_fen(&self, fen: &str) -> Option<Self> {
        todo!()
    }

    /// returns None if the game has ended
    pub fn turn(&self) -> Option<Player> {
        if self.is_stalemate {
            None
        } else {
            Some(
                self.moves
                    .last()
                    .map(|r#move| r#move.1.player.toggled())
                    .unwrap_or(Player::White),
            )
        }
    }

    // TODO: make the move instead take an index of valid moves
    // /// if the move is invalid this returns a [`MoveError`], else the move is applied
    // pub fn try_move(&mut self, r#move: PlayerMove) -> Result<(), MoveError> {
    //     todo!()
    // }
}
