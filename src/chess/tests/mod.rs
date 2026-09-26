use std::collections::HashMap;

use crate::chess::{
    ChessGame, GameState, PositionState,
    board::{BoardPosition, ChessBoard, DeltaChessBoard},
    r#move::MoveInfo,
    types::{ChessPiece, Color, PieceType},
};

#[test]
fn valid_moves_capture() {
    use crate::chess::board::board_init::*;
    let board = ChessBoard::from_rotated([
        [EE, EE, EE, EE, EE, EE, EE, WN],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, WP, EE, EE, EE, EE, EE, EE],
        [BB, EE, EE, EE, EE, EE, EE, EE],
    ]);
    // if it were white's turn
    let state = PositionState {
        player_at_turn: Color::White,
        ..Default::default()
    };
    let game_state = GameState {
        board: board.clone(),
        state,
    };
    let game = ChessGame::from_game_state(game_state);
    let white_pawn = ChessPiece {
        color: Color::White,
        r#type: PieceType::Pawn,
    };
    let white_knight = ChessPiece {
        color: Color::White,
        r#type: PieceType::Knight,
    };
    let pawn_advance_1 = {
        let origin = BoardPosition::new(1, 1).unwrap();
        let destination = BoardPosition::new(1, 2).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_pawn));
        let move_info = MoveInfo {
            piece: white_pawn,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling: (None, None),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let pawn_advance_2 = {
        let origin = BoardPosition::new(1, 1).unwrap();
        let destination = BoardPosition::new(1, 3).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_pawn));
        let move_info = MoveInfo {
            piece: white_pawn,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling: (None, None),
            set_en_passant_square: Some(BoardPosition::new(1, 2).unwrap()),
        };
        (delta_board, move_info)
    };
    let knight_advance_1 = {
        let origin = BoardPosition::new(7, 7).unwrap();
        let destination = BoardPosition::new(5, 6).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_knight));
        let move_info = MoveInfo {
            piece: white_knight,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling: (None, None),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let knight_advance_2 = {
        let origin = BoardPosition::new(7, 7).unwrap();
        let destination = BoardPosition::new(6, 5).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_knight));
        let move_info = MoveInfo {
            piece: white_knight,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling: (None, None),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let expected_valid_moves = HashMap::from([
        pawn_advance_1,
        pawn_advance_2,
        knight_advance_1,
        knight_advance_2,
    ]);
    assert_eq!(game.valid_moves, expected_valid_moves);

    // if it were black's turn
    let state = PositionState {
        player_at_turn: Color::Black,
        ..Default::default()
    };
    let game_state = GameState { state, board };
    let game = ChessGame::from_game_state(game_state);
    let black_bishop = ChessPiece {
        color: Color::Black,
        r#type: PieceType::Bishop,
    };
    let bishop_capture = {
        let origin = BoardPosition::new(0, 0).unwrap();
        let destination = BoardPosition::new(1, 1).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(black_bishop));
        let move_info = MoveInfo {
            piece: black_bishop,
            from: origin,
            to: destination,
            captured_piece: Some((white_pawn, destination)),
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling: (None, None),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let expected_valid_moves = HashMap::from([bishop_capture]);
    assert_eq!(game.valid_moves, expected_valid_moves);
}

#[test]
fn valid_moves_castling() {
    use crate::chess::board::board_init::*;
    let board = ChessBoard::from_rotated([
        [BR, BN, BB, BQ, BK, BB, BN, BR],
        [BP, BP, BP, BP, BP, BP, BP, BP],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [WP, WP, WP, WP, WP, WP, WP, WP],
        [WR, EE, EE, EE, WK, EE, EE, WR],
    ]);
    let state = PositionState::default();
    let game_state = GameState { state, board };
    let mut game = ChessGame::from_game_state(game_state);
    let white_king = ChessPiece {
        color: Color::White,
        r#type: PieceType::King,
    };
    let white_rook = ChessPiece {
        color: Color::White,
        r#type: PieceType::Rook,
    };
    let castling_left = {
        let origin = BoardPosition::new(0, 4).unwrap();
        let destination = BoardPosition::new(0, 2).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_king));
        let rook_origin = BoardPosition::new(0, 0).unwrap();
        let rook_destination = BoardPosition::new(0, 3).unwrap();

        delta_board.insert(rook_origin, None);
        delta_board.insert(rook_destination, Some(white_rook));
        let move_info = MoveInfo {
            piece: white_king,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: Some((rook_origin, rook_destination)),
            en_passant: false,
            disabled_castling: (Some(()), Some(())),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let castling_right = {
        let origin = BoardPosition::new(0, 4).unwrap();
        let destination = BoardPosition::new(0, 6).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None);
        delta_board.insert(destination, Some(white_king));
        let rook_origin = BoardPosition::new(0, 7).unwrap();
        let rook_destination = BoardPosition::new(0, 5).unwrap();

        delta_board.insert(rook_origin, None);
        delta_board.insert(rook_destination, Some(white_rook));
        let move_info = MoveInfo {
            piece: white_king,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: Some((rook_origin, rook_destination)),
            en_passant: false,
            disabled_castling: (Some(()), Some(())),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    // contains left castling
    assert!(
        game.valid_moves
            .remove(&castling_left.0)
            .is_some_and(|move_info| move_info == castling_left.1)
    );
    // contains right castling
    assert!(
        game.valid_moves
            .remove(&castling_right.0)
            .is_some_and(|move_info| move_info == castling_right.1)
    );
}
