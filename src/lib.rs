//! A chess engine library written in rust

use std::collections::HashMap;
use std::fmt::Write;
use std::{array, iter};

use crate::board::{BoardPosition, BoardPositionOffset, ChessBoard, DeltaChessBoard};
use crate::r#move::{
    CheckEvent, ExtendedMoveInfo, MoveInfo, MovedPiece, PromotionPiece, move_piece,
};
use crate::types::{ChessPiece, Color, PieceType};

pub mod board;
pub mod r#move;
pub mod types;

#[cfg(test)]
mod tests;

/// A state machine for a chess game
///
/// Does not include manual terminations like resigning or offering/accepting draw
pub struct ChessGame {
    /// The initial game state
    _init_state: GameState,
    /// All moves made since the initial state in chronological order
    moves: Vec<(DeltaChessBoard, ExtendedMoveInfo)>,

    // (FEATURE: maybe make all redundant sate cache lookups that just store queries from the init_board+init_state+moves instead)
    // --- redundant state for simpler computation ---
    game_termination: Option<GameTermination>,
    valid_moves: HashMap<DeltaChessBoard, MoveInfo>,

    /// The current state of the game
    state: GameState,
}

/// The state of a chess game in one specific position
#[derive(Clone)]
pub struct GameState {
    board: ChessBoard,
    state: PositionState,
}

/// A type describing all kinds of terminations of a chess game
#[derive(Clone)]
pub enum GameTermination {
    /// Contains the player who won
    CheckMate(Color),
    StaleMate,
    FiftyMoveRule,
}

// TODO: refactor and look over the code in the entire codebase to follow the rule of every function
// always staying on the same abstraction layer
impl GameState {
    /// Get all the unique valid moves at a certain position
    pub fn all_valid_moves<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
        Box::new(self.board.squares().flat_map(|pos| self.valid_moves(pos)))
    }

    /// Get all the unique valid moves for a piece at a certain position, including with check rules
    pub fn valid_moves<'a>(
        &'a self,
        pos: BoardPosition,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
        Box::new(
            self.pseudo_valid_moves(pos)
                .filter(|(board_delta, proposed_move)| {
                    let mut board = self.board.clone();
                    board.update(&board_delta);

                    // validity check
                    // check the opponents next valid moves
                    let state = self.state.updated_from_move(proposed_move);

                    let game_state = GameState { board, state }; // both update board and toggle player
                    for (_, opponent_move) in game_state
                        .board
                        .squares()
                        .flat_map(|position| game_state.pseudo_valid_moves(position))
                    {
                        // if the proposed move is a castling, and if any opponent move touches a square inbetween,
                        // the castling is invalid.
                        if let Some((rook_init_pos, _)) = proposed_move.castling {
                            let castle_len = rook_init_pos.x().abs_diff(proposed_move.from.x());

                            // R###K
                            // ##KR#

                            // maybe for pawns this wont work because they dont capture the same way as they move, TODO: maybe fix thislater
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
                        if let Some((
                            ChessPiece {
                                r#type: PieceType::King,
                                ..
                            },
                            _,
                        )) = opponent_move.captured_piece
                        {
                            return false;
                        }
                    }
                    return true;
                }),
        )
    }

    /// Get all the unique valid moves for a piece at a certain position, not including check rules
    ///
    /// This means it may include invalid moves that enable the king being captured in the next move.
    ///
    /// For check rules included, use [`Self::valid_moves`]
    pub fn pseudo_valid_moves<'a>(
        &'a self,
        position: BoardPosition,
    ) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
        // if the selected piece doesnt exist or belong to the player in turn
        let Some(piece) = self.board.get_square(position) else {
            return Box::new(iter::empty());
        };
        let ChessPiece { color, r#type } = piece;

        if color != self.state.player_at_turn {
            return Box::new(iter::empty());
        }
        match r#type {
            PieceType::Pawn => pseudo_valid_pawn_moves(position, &self.board, &self.state),
            PieceType::Knight => pseudo_valid_knight_moves(position, &self.board),
            PieceType::Bishop => pseudo_valid_bishop_moves(position, &self.board),
            PieceType::Rook => pseudo_valid_rook_moves(position, &self.board, &self.state),
            PieceType::Queen => pseudo_valid_queen_moves(position, &self.board),
            PieceType::King => pseudo_valid_king_moves(position, &self.board, &self.state),
        }
    }
    /// Import a game state from a fen string (standardized compact chess position format).
    ///
    /// This is not implemented and will panic.
    pub fn from_fen(&self, _fen: &str) -> Option<Self> {
        todo!()
    }
    /// Convert the game state to a fen string
    pub fn to_fen(&self) -> Result<String, std::fmt::Error> {
        // board positions / ranks
        let mut ranks: [String; board::SIDE_LENGTH] = array::from_fn(|_| String::new());
        for rank in 0..board::SIDE_LENGTH {
            let mut n_empty_squares = 0;
            for x in 0..board::SIDE_LENGTH {
                match self.board.get_square(
                    BoardPosition::new(x as u8, ((board::SIDE_LENGTH - 1) - rank) as u8).unwrap(),
                ) {
                    Some(piece) => {
                        if n_empty_squares > 0 {
                            ranks[rank as usize].push_str(&n_empty_squares.to_string());
                            n_empty_squares = 0;
                        }
                        let base = match piece.r#type {
                            PieceType::Pawn => 'p',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Rook => 'r',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                        };
                        ranks[rank as usize].push(match piece.color {
                            Color::White => base.to_ascii_uppercase(),
                            Color::Black => base,
                        });
                    }
                    None => {
                        n_empty_squares += 1;
                    }
                }
            }
            if n_empty_squares > 0 {
                ranks[rank as usize].push_str(&n_empty_squares.to_string());
            }
        }

        let mut fen = ranks.join("/");

        // separator
        write!(&mut fen, " ")?;

        // player at turn
        write!(
            &mut fen,
            "{}",
            match self.state.player_at_turn {
                Color::White => 'w',
                Color::Black => 'b',
            }
        )?;

        // separator
        write!(&mut fen, " ")?;

        // castling
        let mut castling_string = String::new();
        if self.state.white_can_castle.1 {
            write!(&mut castling_string, "K")?;
        }
        if self.state.white_can_castle.0 {
            write!(&mut castling_string, "Q")?;
        }
        if self.state.black_can_castle.1 {
            write!(&mut castling_string, "k")?;
        }
        if self.state.black_can_castle.0 {
            write!(&mut castling_string, "q")?;
        }
        if castling_string.is_empty() {
            write!(&mut fen, "-")?;
        } else {
            write!(&mut fen, "{}", castling_string)?;
        }

        // separator
        write!(&mut fen, " ")?;

        //en passant
        match self.state.en_passant_square {
            Some(square) => {
                write!(&mut fen, "{}", square.chess_display())?;
            }
            None => {
                write!(&mut fen, "-")?;
            }
        }

        // separator
        write!(&mut fen, " ")?;

        // half move clock
        write!(&mut fen, "{}", self.state.halfmove_clock)?;

        // separator
        write!(&mut fen, " ")?;

        // n fullmoves
        write!(&mut fen, "{}", self.state.n_fullmoves)?;

        Ok(fen)
    }
}

fn pseudo_valid_moves_from_directions<'a>(
    directions: impl Iterator<Item = BoardPositionOffset> + 'a,
    piece: ChessPiece,
    origin: BoardPosition,
    board: &'a ChessBoard,
) -> impl Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a {
    directions.into_iter().flat_map(move |direction| {
        // follow each direction's path until executing a capture, hitting a friendly team piece, or going outside the board bounds
        let mut path_finished = false;
        (1i8..).map_while(move |i| {
            if path_finished {
                None
            // if inside board bounds, and not hitting a same team piece
            } else if let Some(destination) = origin.add(direction * i)
                && let Ok(MovedPiece {
                    board_delta,
                    replaced_piece,
                }) = move_piece(&board, origin, destination)
                && replaced_piece
                    .is_none_or(|replaced_piece| replaced_piece.color == piece.color.toggled())
            {
                // if capturing a piece, the path will end next time
                if replaced_piece.is_some() {
                    path_finished = true;
                }

                Some((
                    board_delta,
                    MoveInfo {
                        piece,
                        from: origin,
                        to: destination,
                        captured_piece: replaced_piece
                            .map(|captured_piece| (captured_piece, destination)),
                        promotion: None,
                        castling: None,
                        en_passant: false,
                        // TODO: a piece may disable castling if it captured a rook that was on its original square
                        disabled_castling_white: (false, false),
                        disabled_castling_black: (false, false),
                        set_en_passant_square: None,
                    },
                ))
            // else the path has already ended
            } else {
                path_finished = true;
                None
            }
        })
    })
}

fn pseudo_valid_pawn_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
    position_state: &'a PositionState,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    let Some(
        piece @ ChessPiece {
            color,
            r#type: PieceType::Pawn,
        },
    ) = board.get_square(position)
    else {
        panic!("pawn invariant broken");
    };
    let y_sign = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    let promotion_y = match color {
        Color::White => 7,
        Color::Black => 0,
    };
    let home_row_y = match color {
        Color::White => 1,
        Color::Black => 6,
    };
    let capture_offsets = [
        BoardPositionOffset::new(1, y_sign),
        BoardPositionOffset::new(-1, y_sign),
    ];
    // move 1 step forward
    let forward_one = 'f1: {
        let offset = BoardPositionOffset::new(0, y_sign);
        let Some(destination) = position.add(offset) else {
            break 'f1 None;
        };
        // pawns may not capture forward
        let Ok(MovedPiece {
            board_delta,
            replaced_piece: None,
        }) = move_piece(&board, position, destination)
        else {
            break 'f1 None;
        };
        Some((
            board_delta,
            MoveInfo {
                piece,
                from: position,
                to: destination,
                captured_piece: None,
                promotion: None,
                castling: None,
                en_passant: false,
                disabled_castling_white: (false, false),
                disabled_castling_black: (false, false),
                set_en_passant_square: None,
            },
        ))
    };
    // move 2 steps forward
    let forward_two = 'f2: {
        // requires that 1 step forward didnt fail, and the pawn is at the home row
        if forward_one.is_some() && position.y() == home_row_y {
            let offset = BoardPositionOffset::new(0, 2 * y_sign);
            let Some(destination) = position.add(offset) else {
                break 'f2 None;
            };
            // pawns may not capture forward
            let Ok(MovedPiece {
                board_delta,
                replaced_piece: None,
            }) = move_piece(&board, position, destination)
            else {
                break 'f2 None;
            };
            Some((
                board_delta,
                MoveInfo {
                    piece,
                    from: position,
                    to: destination,
                    captured_piece: None,
                    promotion: None,
                    castling: None,
                    en_passant: false,
                    disabled_castling_white: (false, false),
                    disabled_castling_black: (false, false),
                    // sets en passant opportunity on the square before it
                    set_en_passant_square: position.add(BoardPositionOffset::new(0, y_sign)),
                },
            ))
        } else {
            None
        }
    };
    let capture = capture_offsets.into_iter().filter_map(move |offset| {
        let Some(destination) = position.add(offset) else {
            return None;
        };
        match move_piece(&board, position, destination) {
            Ok(MovedPiece {
                board_delta,
                replaced_piece: Some(replaced_piece),
            }) if replaced_piece.color == color.toggled() => Some((
                board_delta,
                MoveInfo {
                    piece,
                    from: position,
                    to: destination,
                    captured_piece: Some((replaced_piece, destination)),
                    promotion: None,
                    castling: None,
                    en_passant: false,
                    // TODO: a piece may disable castling if it captured a rook that was on its original square
                    disabled_castling_white: (false, false),
                    disabled_castling_black: (false, false),
                    set_en_passant_square: None,
                },
            )),
            Ok(MovedPiece {
                mut board_delta,
                replaced_piece: None,
            }) if let Some(square) = position_state.en_passant_square
                && square == destination =>
            {
                let captured_pawn_position = square
                    .add(BoardPositionOffset::new(0, -y_sign))
                    .expect("En passant square invariant broken");
                let captured_pawn = match board.get_square(captured_pawn_position) {
                    Some(
                        captured_pawn @ ChessPiece {
                            color: captured_color,
                            r#type: PieceType::Pawn,
                        },
                    ) if captured_color == color.toggled() => captured_pawn,
                    _ => {
                        panic!("en passant piece invariant broken");
                    }
                };
                board_delta.insert(captured_pawn_position, None);
                Some((
                    board_delta,
                    MoveInfo {
                        piece,
                        from: position,
                        to: destination,
                        captured_piece: Some((captured_pawn, captured_pawn_position)),
                        promotion: None,
                        castling: None,
                        en_passant: true,
                        disabled_castling_white: (false, false),
                        disabled_castling_black: (false, false),
                        set_en_passant_square: None,
                    },
                ))
            }
            _ => None,
        }
    });

    Box::new(
        forward_one
            .into_iter()
            .chain(forward_two)
            .chain(capture)
            // add promotion
            .flat_map(
                move |(board_delta, move_info)| -> Box<
                    dyn Iterator<Item = (DeltaChessBoard, MoveInfo)>,
                > {
                    if move_info.to.y() == promotion_y {
                        Box::new(
                            // multiply options by 4 because of promotion
                            [
                                PromotionPiece::Knight,
                                PromotionPiece::Bishop,
                                PromotionPiece::Rook,
                                PromotionPiece::Queen,
                            ]
                            .into_iter()
                            .map(
                                move |promotion_piece| {
                                    let mut board_delta = board_delta.clone();
                                    board_delta.insert(
                                        move_info.to,
                                        Some(ChessPiece {
                                            color,
                                            r#type: promotion_piece.into(),
                                        }),
                                    );
                                    (
                                        board_delta,
                                        MoveInfo {
                                            promotion: Some(promotion_piece),
                                            ..move_info
                                        },
                                    )
                                },
                            ),
                        )
                    } else {
                        Box::new(iter::once((board_delta, move_info)))
                    }
                },
            ),
    )
}

fn pseudo_valid_knight_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    //  # #
    // #   #
    //   O
    // #   #
    //  # #

    let Some(ChessPiece {
        color,
        r#type: PieceType::Knight,
    }) = board.get_square(position)
    else {
        panic!("knight invariant broken");
    };

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
    Box::new(offsets.into_iter().filter_map(move |offset| {
        if let Some(destination) = position.add(offset)
            && let Ok(MovedPiece {
                board_delta,
                replaced_piece,
            }) = move_piece(&board, position, destination)
            && replaced_piece.is_none_or(|replaced_piece| replaced_piece.color == color.toggled())
        {
            Some((
                board_delta,
                MoveInfo {
                    piece: ChessPiece {
                        color,
                        r#type: PieceType::Knight,
                    },
                    from: position,
                    to: destination,
                    captured_piece: replaced_piece
                        .map(|captured_piece| (captured_piece, destination)),
                    promotion: None,
                    castling: None,
                    en_passant: false,
                    // TODO: a piece may disable castling if it captured a rook that was on its original square
                    disabled_castling_white: (false, false),
                    disabled_castling_black: (false, false),
                    set_en_passant_square: None,
                },
            ))
        } else {
            None
        }
    }))
}

fn pseudo_valid_bishop_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    // #   #
    //  # #
    //   O
    //  # #
    // #   #

    let Some(
        piece @ ChessPiece {
            r#type: PieceType::Bishop,
            ..
        },
    ) = board.get_square(position)
    else {
        panic!("bishop invariant broken");
    };

    let directions = [
        BoardPositionOffset::DIAGONAL_NE,
        BoardPositionOffset::DIAGONAL_NW,
        BoardPositionOffset::DIAGONAL_SW,
        BoardPositionOffset::DIAGONAL_SE,
    ];

    Box::new(pseudo_valid_moves_from_directions(
        directions.into_iter(),
        piece,
        position,
        board,
    ))
}

fn pseudo_valid_rook_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
    position_state: &'a PositionState,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    //   #
    //   #
    // ##O##
    //   #
    //   #

    let Some(
        piece @ ChessPiece {
            color,
            r#type: PieceType::Rook,
        },
    ) = board.get_square(position)
    else {
        panic!("rook invariant broken");
    };

    let directions = [
        BoardPositionOffset::NORTH,
        BoardPositionOffset::SOUTH,
        BoardPositionOffset::WEST,
        BoardPositionOffset::EAST,
    ];
    let can_castle = match color {
        Color::White => position_state.white_can_castle,
        Color::Black => position_state.black_can_castle,
    };
    let castle_row_y = match color {
        Color::White => 0,
        Color::Black => 7,
    };
    // if castling is currently enabled (the rook hasnt moved before) and
    // that rook is currently moving, disable castling on that spot
    let disabled_castling =
        if can_castle.0 && position == BoardPosition::new(0, castle_row_y).unwrap() {
            (true, false)
        } else if can_castle.1 && position == BoardPosition::new(7, castle_row_y).unwrap() {
            (false, true)
        } else {
            (false, false)
        };

    Box::new(
        pseudo_valid_moves_from_directions(directions.into_iter(), piece, position, board)
            // all of the proposed rook moves in this iterator will disable castling
            // if that rook previously was unmoved (as well as the king)
            .map(move |(board_delta, mut move_info)| {
                match color {
                    Color::White => {
                        move_info.disabled_castling_white = disabled_castling;
                    }
                    Color::Black => {
                        move_info.disabled_castling_black = disabled_castling;
                    }
                }
                (board_delta, move_info)
            }),
    )
}
fn pseudo_valid_queen_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    // # # #
    //  ###
    // ##O##
    //  ###
    // # # #

    let Some(
        piece @ ChessPiece {
            r#type: PieceType::Queen,
            ..
        },
    ) = board.get_square(position)
    else {
        panic!("queen invariant broken");
    };

    let directions = [
        BoardPositionOffset::DIAGONAL_NE,
        BoardPositionOffset::DIAGONAL_NW,
        BoardPositionOffset::DIAGONAL_SW,
        BoardPositionOffset::DIAGONAL_SE,
        BoardPositionOffset::NORTH,
        BoardPositionOffset::SOUTH,
        BoardPositionOffset::WEST,
        BoardPositionOffset::EAST,
    ];
    Box::new(pseudo_valid_moves_from_directions(
        directions.into_iter(),
        piece,
        position,
        board,
    ))
}

fn pseudo_valid_king_moves<'a>(
    position: BoardPosition,
    board: &'a ChessBoard,
    position_state: &'a PositionState,
) -> Box<dyn Iterator<Item = (DeltaChessBoard, MoveInfo)> + 'a> {
    //
    //  ###
    //  #O#
    //  ###
    //

    let Some(
        piece @ ChessPiece {
            color,
            r#type: PieceType::King,
        },
    ) = board.get_square(position)
    else {
        panic!("king invariant broken");
    };

    let offsets = [
        BoardPositionOffset::DIAGONAL_NE,
        BoardPositionOffset::DIAGONAL_NW,
        BoardPositionOffset::DIAGONAL_SW,
        BoardPositionOffset::DIAGONAL_SE,
        BoardPositionOffset::NORTH,
        BoardPositionOffset::SOUTH,
        BoardPositionOffset::WEST,
        BoardPositionOffset::EAST,
    ];

    let can_castle = match color {
        Color::White => position_state.white_can_castle,
        Color::Black => position_state.black_can_castle,
    };

    let castling_row_y = match color {
        Color::White => 0,
        Color::Black => 7,
    };
    let castle = |can_castle, old_king_x, new_king_x, old_rook_x, new_rook_x| {
        if can_castle &&
        // the king is at the right position
        position == BoardPosition::new(old_king_x, castling_row_y).unwrap()
        // the rook is at the right position
        && board
            .get_square(BoardPosition::new(old_rook_x, castling_row_y).unwrap())
            .is_some_and(|piece| piece.color == color && piece.r#type == PieceType::Rook)
        // all squares inbetween are empty
        && (old_king_x.min(old_rook_x)+1..old_king_x.max(old_rook_x)).all(|x|
                board
                    .get_square(BoardPosition::new(x, castling_row_y).unwrap())
                    .is_none()
                    )
        {
            let mut board_delta = DeltaChessBoard::new();
            let new_king_position = BoardPosition::new(new_king_x, castling_row_y).unwrap();
            let new_rook_position = BoardPosition::new(new_rook_x, castling_row_y).unwrap();
            let old_rook_position = BoardPosition::new(old_rook_x, castling_row_y).unwrap();

            // move the king
            board_delta.insert(new_king_position, Some(piece));
            // move the rook
            board_delta.insert(
                new_rook_position,
                Some(ChessPiece {
                    color,
                    r#type: PieceType::Rook,
                }),
            );
            // empty where the king and rook used to be
            board_delta.insert(position, None);
            board_delta.insert(old_rook_position, None);
            Some((
                board_delta,
                MoveInfo {
                    piece,
                    from: position,
                    to: new_king_position,
                    captured_piece: None,
                    promotion: None,
                    castling: Some((old_rook_position, new_rook_position)),
                    en_passant: false,
                    disabled_castling_white: if color == Color::White {
                        (true, true)
                    } else {
                        (false, false)
                    },
                    disabled_castling_black: if color == Color::Black {
                        (true, true)
                    } else {
                        (false, false)
                    },
                    set_en_passant_square: None,
                },
            ))
        } else {
            None
        }
    };
    let left_castle = castle(can_castle.0, 4, 2, 0, 3);
    let right_castle = castle(can_castle.1, 4, 6, 7, 5);

    // if castling wasnt already disabled, moving the king or castling will definitely disable it
    let disabled_castling = can_castle;
    Box::new(
        offsets
            .into_iter()
            .filter_map(move |offset| {
                if let Some(destination) = position.add(offset)
                    && let Ok(MovedPiece {
                        board_delta,
                        replaced_piece,
                    }) = move_piece(&board, position, destination)
                    && replaced_piece
                        .is_none_or(|replaced_piece| replaced_piece.color == color.toggled())
                {
                    Some((
                        board_delta,
                        MoveInfo {
                            piece,
                            from: position,
                            to: destination,
                            captured_piece: replaced_piece
                                .map(|captured_piece| (captured_piece, destination)),
                            promotion: None,
                            castling: None,
                            en_passant: false,
                            disabled_castling_white: if color == Color::White {
                                disabled_castling
                            } else {
                                (false, false)
                            },
                            disabled_castling_black: if color == Color::Black {
                                disabled_castling
                            } else {
                                (false, false)
                            },
                            set_en_passant_square: None,
                        },
                    ))
                } else {
                    None
                }
            })
            .chain(left_castle)
            .chain(right_castle),
    )
}

/// The state of a chess game in one specific position, excluding the piece positions/board and containing only metadata
#[derive(Clone)]
pub struct PositionState {
    player_at_turn: Color,
    /// (queen-side, king-side)
    white_can_castle: (bool, bool),
    /// (queen-side, king-side)
    black_can_castle: (bool, bool),
    /// A square where en passant is possible (if a pawn moved past it the previous move)
    en_passant_square: Option<BoardPosition>,
    /// The number of halfmoves since the last capture or pawn advance
    halfmove_clock: u8,
    /// The number of full moves. is 1-indexed for some reason
    n_fullmoves: u16,
}

impl Default for PositionState {
    fn default() -> Self {
        Self {
            player_at_turn: Color::White,
            white_can_castle: (true, true),
            black_can_castle: (true, true),
            en_passant_square: None,
            halfmove_clock: 0,
            n_fullmoves: 1,
        }
    }
}

impl PositionState {
    /// Get an updated version of a position state after a move has occured
    pub fn updated_from_move(&self, move_info: &MoveInfo) -> Self {
        Self {
            white_can_castle: (
                self.white_can_castle.0 && !move_info.disabled_castling_white.0,
                self.white_can_castle.1 && !move_info.disabled_castling_white.1,
            ),
            black_can_castle: (
                self.black_can_castle.0 && !move_info.disabled_castling_black.0,
                self.black_can_castle.1 && !move_info.disabled_castling_black.1,
            ),
            en_passant_square: move_info.set_en_passant_square,
            halfmove_clock: if move_info.piece.r#type == PieceType::Pawn
                || move_info.captured_piece.is_some()
            {
                0
            } else {
                self.halfmove_clock + 1
            },
            n_fullmoves: self.n_fullmoves
                + match self.player_at_turn {
                    Color::White => 0,
                    Color::Black => 1,
                },
            player_at_turn: self.player_at_turn.toggled(),
        }
    }
}

impl ChessGame {
    /// Create a new chess game with default settings and a default starting board position
    pub fn new() -> Self {
        let state = GameState {
            board: ChessBoard::start_position(),
            state: PositionState::default(),
        };
        let valid_moves = state.all_valid_moves().collect();
        Self {
            _init_state: state.clone(),
            moves: Vec::new(),

            game_termination: None,
            valid_moves,
            state,
        }
    }

    /// Get the chess board
    pub fn board(&self) -> &ChessBoard {
        &self.state.board
    }

    /// Get the position state
    pub fn state(&self) -> &PositionState {
        &self.state.state
    }

    /// Get whether the chess game is terminated
    pub fn termination(&self) -> &Option<GameTermination> {
        &self.game_termination
    }

    /// Get all valid moves
    pub fn valid_moves(&self) -> &HashMap<DeltaChessBoard, MoveInfo> {
        &self.valid_moves
    }

    /// Import a chess game from a game state
    pub fn from_game_state(state: GameState) -> Self {
        let valid_moves: HashMap<DeltaChessBoard, _> = state.all_valid_moves().collect();
        // check for check
        let check_state = GameState {
            board: state.board.clone(),
            state: PositionState {
                player_at_turn: state.state.player_at_turn.toggled(),
                en_passant_square: None,
                ..state.state
            },
        };
        // check for check
        let is_check = check_state
            .board
            .squares()
            .flat_map(|square| check_state.pseudo_valid_moves(square))
            .find(|(_, move_info)| {
                move_info
                    .captured_piece
                    .is_some_and(|piece| piece.0.r#type == PieceType::King)
            })
            .is_some(); // update state based on how the move changed it

        let stalemate = if valid_moves.is_empty() { true } else { false };
        let game_termination = if stalemate && is_check {
            Some(GameTermination::CheckMate(
                state.state.player_at_turn.toggled(),
            ))
        } else if stalemate {
            Some(GameTermination::StaleMate)
        } else if state.state.halfmove_clock >= 100 {
            Some(GameTermination::FiftyMoveRule)
        } else {
            None
        };
        Self {
            _init_state: state.clone(),
            moves: Vec::new(),

            game_termination,
            valid_moves,
            state,
        }
    }

    /// Returns the player whos turn it is. If the game has ended this will return None
    pub fn turn(&self) -> Option<Color> {
        if self.game_termination.is_some() {
            None
        } else {
            Some(self.state.state.player_at_turn)
        }
    }

    /// Perform a move on the chess game. If the move is invalid or the game has ended, this will return an error.
    ///
    /// Will otherwise return the information of the move, and whether the game terminated after the move
    pub fn r#move(
        &mut self,
        r#move: DeltaChessBoard,
    ) -> Result<(ExtendedMoveInfo, Option<GameTermination>), MoveError> {
        if self.game_termination.is_some() {
            return Err(MoveError::GameAlreadyEnded);
        }
        let intermediate_move_info = self
            .valid_moves
            .remove(&r#move)
            .ok_or(MoveError::InvalidMove)?;
        let player = intermediate_move_info.piece.color;

        // update board and state
        let mut new_board = self.state.board.clone();
        new_board.update(&r#move);
        let new_position_state = self.state.state.updated_from_move(&intermediate_move_info);

        let check_state = GameState {
            board: new_board.clone(),
            state: PositionState {
                player_at_turn: intermediate_move_info.piece.color,
                en_passant_square: None,
                ..new_position_state
            },
        };
        // check for check
        let is_check = check_state
            .board
            .squares()
            .flat_map(|square| check_state.pseudo_valid_moves(square))
            .find(|(_, move_info)| {
                move_info
                    .captured_piece
                    .is_some_and(|piece| piece.0.r#type == PieceType::King)
            })
            .is_some(); // update state based on how the move changed it

        // update state from move
        self.state = GameState {
            board: new_board,
            state: new_position_state,
        };

        // update valid moves
        self.valid_moves.clear();
        self.valid_moves.extend(self.state.all_valid_moves());

        // check for stalemate/checkmate and get ExtendedMoveInfo
        let stalemate = if self.valid_moves.is_empty() {
            true
        } else {
            false
        };
        let new_move_info = ExtendedMoveInfo::new(
            intermediate_move_info,
            match (is_check, stalemate) {
                (true, true) => Some(CheckEvent::Checkmate),
                (false, true) => Some(CheckEvent::Stalemate),
                (true, false) => Some(CheckEvent::Check),
                (false, false) => None,
            },
        );
        self.moves.push((r#move, new_move_info.clone()));

        let game_termination = if stalemate && is_check {
            Some(GameTermination::CheckMate(player))
        } else if stalemate {
            Some(GameTermination::StaleMate)
        } else if self.state.state.halfmove_clock >= 100 {
            Some(GameTermination::FiftyMoveRule)
        } else {
            None
        };
        self.game_termination = game_termination.clone();
        Ok((new_move_info, game_termination))
    }
}

/// An error type for [`ChessGame::move`].
pub enum MoveError {
    InvalidMove,
    GameAlreadyEnded,
}
