use std::collections::HashMap;

use crate::{
    ChessGame, GameState, PositionState,
    board::{BoardPosition, ChessBoard, DeltaChessBoard},
    r#move::{MoveInfo, PromotionPiece},
    types::{ChessPiece, Color, PieceType},
};

#[test]
fn valid_moves_capture() {
    use crate::board::board_init::*;
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
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_pawn)).unwrap();
        let move_info = MoveInfo {
            piece: white_pawn,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let pawn_advance_2 = {
        let origin = BoardPosition::new(1, 1).unwrap();
        let destination = BoardPosition::new(1, 3).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_pawn)).unwrap();
        let move_info = MoveInfo {
            piece: white_pawn,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: Some(BoardPosition::new(1, 2).unwrap()),
        };
        (delta_board, move_info)
    };
    let knight_advance_1 = {
        let origin = BoardPosition::new(7, 7).unwrap();
        let destination = BoardPosition::new(5, 6).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_knight)).unwrap();
        let move_info = MoveInfo {
            piece: white_knight,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let knight_advance_2 = {
        let origin = BoardPosition::new(7, 7).unwrap();
        let destination = BoardPosition::new(6, 5).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_knight)).unwrap();
        let move_info = MoveInfo {
            piece: white_knight,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
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
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(black_bishop)).unwrap();
        let move_info = MoveInfo {
            piece: black_bishop,
            from: origin,
            to: destination,
            captured_piece: Some((white_pawn, destination)),
            promotion: None,
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let expected_valid_moves = HashMap::from([bishop_capture]);
    assert_eq!(game.valid_moves, expected_valid_moves);
}

#[test]
fn valid_moves_castling() {
    use crate::board::board_init::*;
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
        let origin = BoardPosition::new(4, 0).unwrap();
        let destination = BoardPosition::new(2, 0).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_king)).unwrap();
        let rook_origin = BoardPosition::new(0, 0).unwrap();
        let rook_destination = BoardPosition::new(3, 0).unwrap();

        delta_board.insert(rook_origin, None).unwrap();
        delta_board
            .insert(rook_destination, Some(white_rook))
            .unwrap();
        let move_info = MoveInfo {
            piece: white_king,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: Some((rook_origin, rook_destination)),
            en_passant: false,
            disabled_castling_white: (true, true),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    };
    let castling_right = {
        let origin = BoardPosition::new(4, 0).unwrap();
        let destination = BoardPosition::new(6, 0).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board.insert(destination, Some(white_king)).unwrap();
        let rook_origin = BoardPosition::new(7, 0).unwrap();
        let rook_destination = BoardPosition::new(5, 0).unwrap();

        delta_board.insert(rook_origin, None).unwrap();
        delta_board
            .insert(rook_destination, Some(white_rook))
            .unwrap();
        let move_info = MoveInfo {
            piece: white_king,
            from: origin,
            to: destination,
            captured_piece: None,
            promotion: None,
            castling: Some((rook_origin, rook_destination)),
            en_passant: false,
            disabled_castling_white: (true, true),
            disabled_castling_black: (false, false),
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

#[test]
fn chess_board_rotation() {
    use crate::board::board_init::*;

    //    [r n b q k b n r]
    //    [p p p p p p p p]
    //    [e e e e e e e e]
    //    [e e e e e e e e]
    //    [e e e e e e e e]
    //    [e e e e e e e e]
    //    [P P P P P P P P]
    //    [R N B Q K B N R]
    //    =>
    //    [R P e e e e p r]
    //    [N P e e e e p n]
    //    [B P e e e e p b]
    //    [Q P e e e e p q]
    //    [K P e e e e p k]
    //    [B P e e e e p b]
    //    [N P e e e e p n]
    //    [R P e e e e p r]
    let board = ChessBoard::from_rotated([
        [BR, BN, BB, BQ, BK, BB, BN, BR],
        [BP, BP, BP, BP, BP, BP, BP, BP],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [WP, WP, WP, WP, WP, WP, WP, WP],
        [WR, WN, WB, WQ, WK, WB, WN, WR],
    ]);
    let result = [
        [WR, WP, EE, EE, EE, EE, BP, BR],
        [WN, WP, EE, EE, EE, EE, BP, BN],
        [WB, WP, EE, EE, EE, EE, BP, BB],
        [WQ, WP, EE, EE, EE, EE, BP, BQ],
        [WK, WP, EE, EE, EE, EE, BP, BK],
        [WB, WP, EE, EE, EE, EE, BP, BB],
        [WN, WP, EE, EE, EE, EE, BP, BN],
        [WR, WP, EE, EE, EE, EE, BP, BR],
    ];

    // println!("{board}");
    assert_eq!(board.inner(), &result);
}
#[test]
fn promotion() {
    use crate::board::board_init::*;
    let board = ChessBoard::from_rotated([
        [EE, EE, BR, BR, EE, EE, EE, EE],
        [EE, EE, EE, WP, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
    ]);
    let state = PositionState::default();
    let game_state = GameState { state, board };
    let mut game = ChessGame::from_game_state(game_state);
    let white_pawn = ChessPiece {
        color: Color::White,
        r#type: PieceType::Pawn,
    };
    let black_rook = ChessPiece {
        color: Color::Black,
        r#type: PieceType::Rook,
    };

    assert_eq!(game.valid_moves.len(), 4);

    for (delta_board, move_info) in [
        PromotionPiece::Knight,
        PromotionPiece::Bishop,
        PromotionPiece::Rook,
        PromotionPiece::Queen,
    ]
    .into_iter()
    .map(|prom_piece| {
        let origin = BoardPosition::new(3, 6).unwrap();
        let destination = BoardPosition::new(2, 7).unwrap();
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(origin, None).unwrap();
        delta_board
            .insert(
                destination,
                Some(ChessPiece {
                    color: Color::White,
                    r#type: prom_piece.into(),
                }),
            )
            .unwrap();
        let move_info = MoveInfo {
            piece: white_pawn,
            from: origin,
            to: destination,
            captured_piece: Some((black_rook, destination)),
            promotion: Some(prom_piece),
            castling: None,
            en_passant: false,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };
        (delta_board, move_info)
    }) {
        assert!(
            game.valid_moves
                .remove(&delta_board)
                .is_some_and(|mov_info| mov_info == move_info)
        );
    }
}
#[test]
fn en_passant() {
    use crate::board::board_init::*;
    let board = ChessBoard::from_rotated([
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, BP, WP, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
        [EE, EE, EE, EE, EE, EE, EE, EE],
    ]);
    let state = PositionState {
        player_at_turn: Color::Black,
        en_passant_square: Some(BoardPosition::new(3, 2).unwrap()),
        ..Default::default()
    };
    let game_state = GameState { state, board };
    let mut game = ChessGame::from_game_state(game_state);
    let white_pawn = ChessPiece {
        color: Color::White,
        r#type: PieceType::Pawn,
    };
    let black_pawn = ChessPiece {
        color: Color::Black,
        r#type: PieceType::Pawn,
    };
    let black_pawn_from = BoardPosition::new(2, 3).unwrap();
    let black_pawn_to = BoardPosition::new(3, 2).unwrap();
    let white_pawn_pos = BoardPosition::new(3, 3).unwrap();

    let en_passant = {
        let mut delta_board = DeltaChessBoard::new();
        delta_board.insert(white_pawn_pos, None).unwrap();
        delta_board.insert(black_pawn_to, Some(black_pawn)).unwrap();
        delta_board.insert(black_pawn_from, None).unwrap();

        let move_info = MoveInfo {
            piece: black_pawn,
            from: black_pawn_from,
            to: black_pawn_to,
            captured_piece: Some((white_pawn, white_pawn_pos)),
            promotion: None,
            castling: None,
            en_passant: true,
            disabled_castling_white: (false, false),
            disabled_castling_black: (false, false),
            set_en_passant_square: None,
        };

        (delta_board, move_info)
    };
    assert!(
        game.valid_moves
            .remove(&en_passant.0)
            .is_some_and(|mov_info| mov_info == en_passant.1)
    );
}
