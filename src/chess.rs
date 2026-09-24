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
    /// Get all the unique valid moves for a piece, including with check rules
    pub fn valid_moves<'a>(
        &'a self,
        pos: BoardPosition,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, IntermediateMoveInfo)> + 'a> {
        Box::new(
            self.quasi_valid_moves(pos)
                .filter(|(board_delta, proposed_move)| {
                    let mut board = self.board.clone();
                    board.update(&board_delta);

                    // validitiy check
                    // check the opponents next valid moves
                    let mut state = self.state.clone();
                    state.player_at_turn.toggle();
                    let board_squares = board.squares();
                    let game_state = GameState { board, state }; // both update board and toggle player
                    for (_, opponent_move) in board_squares
                        .iter()
                        .flat_map(|position| game_state.quasi_valid_moves(*position))
                    {
                        // if the proposed move is a castling, and if any opponent move touches a square inbetween,
                        // the castling is invalid.
                        if let Some((rook_init_pos, _)) = proposed_move.castling {
                            let castle_len = rook_init_pos.x().abs_diff(proposed_move.from.x());

                            // R###K
                            // ##KR#

                            if opponent_move.to.y() == rook_init_pos.y()
                                // if within 2 squares of the king's starting position (meaning the king passes through it)
                                && opponent_move.to.x().abs_diff(proposed_move.from.x()) <= 2
                                // and within castle_len squares from the rook
                                && opponent_move.to.x().abs_diff(rook_init_pos.x()) <= castle_len
                            {
                                return false;
                            }
                        }

                        // if an opponent move captures your king, the proposed move is invalid and filtered out
                        if let Some((ChessPiece::King, _)) = opponent_move.captured_piece {
                            return false;
                        }
                    }
                    return true;
                }),
        )
    }

    /// Get the unique valid moves for a piece on the board, only according to the piece's move rules, and no check rules.
    /// This means it may include invalid moves that enable the king being captured in the next move.
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
                let can_castle = match player {
                    Player::White => self.state.white_can_castle,
                    Player::Black => self.state.black_can_castle,
                };
                // if the rook move will disable castling.
                // means if castling is currently enabled (the rook hasnt moved before) and
                // that rook is currently moving, disable castling on that spot
                let disabled_castling = if can_castle.0
                    && position
                        == BoardPosition::new(
                            0,
                            match player {
                                Player::White => 0,
                                Player::Black => 7,
                            },
                        )
                        .unwrap()
                {
                    (Some(()), None)
                } else if can_castle.1
                    && position
                        == BoardPosition::new(
                            7,
                            match player {
                                Player::White => 0,
                                Player::Black => 7,
                            },
                        )
                        .unwrap()
                {
                    (None, Some(()))
                } else {
                    (None, None)
                };

                return Box::new(
                    directions
                        .into_iter()
                        .flat_map(move |direction| {
                            Path::new(direction, PathLength::Infinite).moves(&self.board, position)
                        })
                        // all of the proposed rook moves in this iterator will disable castling
                        // if that rook previously was unmoved (as well as the king)
                        .map(move |(board_delta, mut move_info)| {
                            move_info.disabled_castling = disabled_castling;
                            (board_delta, move_info)
                        }),
                );
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

                let can_castle = match player {
                    Player::White => self.state.white_can_castle,
                    Player::Black => self.state.black_can_castle,
                };
                // TODO fix this taking up so much code for a simple castling implementation
                // if castling is enabled on the left side, make sure all squares inbetween are empty
                let left_castle = if can_castle.0
                    // all squares inbetween are empty
                    && (1..position.x()).all(|x| {
                        self.board
                            .get_square(BoardPosition::new(x, position.y()).unwrap())
                            .is_none()
                    }) {
                    let mut board_delta = DeltaChessBoard::new();
                    let new_king_position =
                        BoardPosition::new(position.x() - 2, position.y()).unwrap();
                    let new_rook_position =
                        BoardPosition::new(position.x() - 1, position.y()).unwrap();
                    let old_rook_position = BoardPosition::new(0, position.y()).unwrap();

                    // move the king
                    board_delta.insert(
                        new_king_position,
                        Some(PlayerPiece {
                            player,
                            piece: ChessPiece::King,
                        }),
                    );
                    // move the rook
                    board_delta.insert(
                        new_rook_position,
                        Some(PlayerPiece {
                            player,
                            piece: ChessPiece::Rook,
                        }),
                    );
                    // empty where the king and rook used to be
                    board_delta.insert(position, None);
                    board_delta.insert(old_rook_position, None);
                    Some((
                        board_delta,
                        IntermediateMoveInfo {
                            player,
                            piece,
                            from: position,
                            to: new_king_position,
                            captured_piece: None,
                            promotion: None,
                            castling: Some((old_rook_position, new_rook_position)),
                            en_passant: false,
                            disabled_castling: (Some(()), Some(())),
                            set_en_passant_square: None,
                        },
                    ))
                } else {
                    None
                };

                let right_castle = if can_castle.1
                    // all squares inbetween are empty
                    && (position.x()+1..7).all(|x| {
                        self.board
                            .get_square(BoardPosition::new(x, position.y()).unwrap())
                            .is_none()
                    }) {
                    let mut board_delta = DeltaChessBoard::new();
                    let new_king_position =
                        BoardPosition::new(position.x() + 2, position.y()).unwrap();
                    let new_rook_position =
                        BoardPosition::new(position.x() + 1, position.y()).unwrap();
                    let old_rook_position = BoardPosition::new(7, position.y()).unwrap();

                    // move the king
                    board_delta.insert(
                        new_king_position,
                        Some(PlayerPiece {
                            player,
                            piece: ChessPiece::King,
                        }),
                    );
                    // move the rook
                    board_delta.insert(
                        new_rook_position,
                        Some(PlayerPiece {
                            player,
                            piece: ChessPiece::Rook,
                        }),
                    );
                    // empty where the king and rook used to be
                    board_delta.insert(position, None);
                    board_delta.insert(old_rook_position, None);
                    Some((
                        board_delta,
                        IntermediateMoveInfo {
                            player,
                            piece,
                            from: position,
                            to: new_king_position,
                            captured_piece: None,
                            promotion: None,
                            castling: Some((old_rook_position, new_rook_position)),
                            en_passant: false,
                            disabled_castling: (Some(()), Some(())),
                            set_en_passant_square: None,
                        },
                    ))
                } else {
                    None
                };

                // if castling wasnt already disabled, moving the king or castling will definitely disable it
                let disabled_castling = match can_castle {
                    (true, true) => (Some(()), Some(())),
                    (true, false) => (Some(()), None),
                    (false, true) => (None, Some(())),
                    (false, false) => (None, None),
                };

                return Box::new(
                    offsets
                        .into_iter()
                        .filter_map(move |offset| {
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
                                        disabled_castling,
                                        set_en_passant_square: None,
                                    },
                                ))
                            } else {
                                None
                            }
                        })
                        .chain(left_castle)
                        .chain(right_castle),
                );

                // add castling
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
