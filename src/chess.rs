use std::collections::{HashMap, HashSet};

use crate::chess::board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard};
use crate::chess::r#move::{
    IntermediateMoveInfo, MoveInfo, MoveType, Path, PathLength, PathType, move_piece,
};
use crate::chess::types::{ChessPiece, Player, PlayerPiece};

pub mod board;
pub mod r#move;
pub mod types;

/// a chess game state machine
pub struct ChessGame {
    /// the initial game state
    init_state: GameState,
    /// all moves made since the initial state in chronological order
    moves: Vec<(DeltaChessBoard, MoveInfo)>,

    // --- redundant state for simpler computation ---
    /// this doesn't mean the game termination can't be checkmate, to check that, it is required to check if the last
    /// move was a check or not.
    is_stalemate: bool,

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
    /// get all the valid moves for a piece, including with check rules
    pub fn valid_moves(&self, pos: BoardPosition) -> HashMap<DeltaChessBoard, MoveInfo> {
        let other_player = self.state.player_at_turn.toggled();
        let mut proposed_moves = self.quasi_valid_moves(pos);
        let mut valid_moves = HashMap::with_capacity(proposed_moves.len());

        'move_loop: for (proposed_move_delta, _) in proposed_moves {
            let mut board = self.board.clone();
            board.update(&proposed_move_delta);

            // validitiy check
            // check the opponents next valid moves
            let mut state = self.state.clone();
            state.player_at_turn.toggle();
            let game_state = GameState { board, state }; // both update board and toggle player
            for (_, intermediate_move_info) in board
                .get_player_pieces(other_player)
                .iter()
                .flat_map(|(_, piece_pos)| game_state.quasi_valid_moves(*piece_pos).into_iter())
            {
                // if one of them captures your king, the move is invalid and not included
                if let Some((ChessPiece::King, _)) = intermediate_move_info.captured_piece {
                    continue 'move_loop;
                }
            }
            // check check
            // check your next valid moves like it were your turn twice in a row
            let mut state = self.state.clone();
            let game_state = GameState { board, state }; // only update board, same player still at turn
            for (_, intermediate_move_info) in board
                .get_player_pieces()
                .iter()
                .flat_map(|(_, piece_pos)| game_state.quasi_valid_moves(*piece_pos).into_iter())
            {
                // check if a king was captured somewhere, if so the move is a check
                if let Some((ChessPiece::King, _)) = intermediate_move_info.captured_piece {
                    return false;
                }
            }
            return true;
        }

        proposed_moves
    }

    // OPTIMIZATION: this could instead be a lazily computed iterator, which would need not needing to store the entire HashMap, since
    // basically all use cases only require sequiental access. Could be useful in cases of early return, like if you find a move that
    // captured the king you dont need to check further.
    /// Get the valid moves for a piece on the board, only according to the piece's move rules, and no check rules.
    /// This means it may include invalid moves that enable the king being captured in the next move.
    pub fn quasi_valid_moves(
        &self,
        position: BoardPosition,
    ) -> HashMap<DeltaChessBoard, IntermediateMoveInfo> {
        let mut moves = HashMap::new();
        // if the selected piece doesnt exist or belong to the player in turn
        let Some(PlayerPiece { player, piece }) = self.board.get_square(position) else {
            return moves;
        };
        if player != self.state.player_at_turn {
            return moves;
        }
        match piece {
            // make this in the case of promotion return one move for every promotion variant, such that only the DeltaChessBoard differs
            ChessPiece::Pawn => {
                // add diagonal capture and en passant and initial 2 jump
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

                for offset in offsets {
                    if let Some(destination) = position.add(offset)
                        && let Ok((delta_board, r#type)) =
                            move_piece(&self.board, position, destination)
                    {
                        moves.insert(
                            delta_board,
                            IntermediateMoveInfo {
                                player,
                                piece,
                                from: position,
                                to: destination,
                                captured_piece: match r#type {
                                    MoveType::Captured(chess_piece) => {
                                        Some((chess_piece, destination))
                                    }
                                    MoveType::Moved => None,
                                },
                                promotion: None,
                                castling: None,
                                en_passant: false,
                            },
                        );
                    }
                }
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
                for direction in directions {
                    'paths: for offset in
                        Path::new(direction, PathLength::Infinite, PathType::Capture).iter()
                    {
                    }
                }
                todo!()
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
                for direction in directions {
                    let path = Path::new(direction, PathLength::Infinite, PathType::Capture);
                }
                todo!()
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
                todo!()
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
                todo!()
            }
        }
        moves
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
        if self
            .moves
            .last()
            .is_some_and(|r#move| r#move.1.is_termination())
        {
            None
        } else {
            Some(
                self.moves
                    .last()
                    .map(|r#move| r#move.1.player)
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

pub enum MoveError {
    NotInTurn,
    InvalidMove,
}

// pub struct PlayerMove {
//     player: Player,
//     r#move: Move,
// }
