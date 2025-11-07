#![cfg(feature = "legacy-tests")]
//! Comprehensive tests for move unmaking functionality
//!
//! This module tests the move unmaking system to ensure that:
//! - All move types can be unmade correctly (normal, capture, promotion, drop)
//! - Board state is fully restored after unmaking
//! - Multiple consecutive moves can be made and unmade
//! - Edge cases are handled correctly

use shogi_engine::{
    bitboards::{BitboardBoard, MoveInfo},
    moves::MoveGenerator,
    types::{CapturedPieces, Move, PieceType, Player, Position},
};

fn board_fen_eq(
    board1: &BitboardBoard,
    board2: &BitboardBoard,
    captured1: &CapturedPieces,
    captured2: &CapturedPieces,
    player: Player,
) -> bool {
    board1.to_fen(player, captured1) == board2.to_fen(player, captured2)
}

#[test]
fn test_unmake_normal_move() {
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let player = Player::Black;

    // Get initial state
    let initial_fen = board.to_fen(player, &captured);

    // Find a legal move
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    assert!(!moves.is_empty(), "Should have legal moves");

    let test_move = &moves[0];

    // Make move with info
    let move_info = board.make_move_with_info(test_move);
    if let Some(ref captured_piece) = move_info.captured_piece {
        captured.add_piece(captured_piece.piece_type, player);
    }

    // Verify board changed
    let after_fen = board.to_fen(player, &captured);
    assert_ne!(initial_fen, after_fen, "Board should change after move");

    // Unmake move
    if let Some(ref captured_piece) = move_info.captured_piece {
        captured.remove_piece(captured_piece.piece_type, player);
    }
    board.unmake_move(&move_info);

    // Verify board restored
    let restored_fen = board.to_fen(player, &captured);
    assert_eq!(
        initial_fen, restored_fen,
        "Board should be restored after unmaking move"
    );
}

#[test]
fn test_unmake_capture_move() {
    // Create a position where a capture is possible
    // For simplicity, use initial position and find a capture move
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let player = Player::Black;

    let initial_fen = board.to_fen(player, &captured);
    let initial_captured_count = captured.count(PieceType::Pawn, Player::Black)
        + captured.count(PieceType::Lance, Player::Black)
        + captured.count(PieceType::Knight, Player::Black);

    // Find a capture move
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    let capture_move = moves.iter().find(|m| m.is_capture);

    if let Some(test_move) = capture_move {
        // Make move with info
        let move_info = board.make_move_with_info(test_move);
        let captured_before = captured.clone();

        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.add_piece(captured_piece.piece_type, player);
        }

        // Verify capture occurred
        let after_captured_count = captured.count(PieceType::Pawn, Player::Black)
            + captured.count(PieceType::Lance, Player::Black)
            + captured.count(PieceType::Knight, Player::Black);

        // Unmake move
        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.remove_piece(captured_piece.piece_type, player);
        }
        board.unmake_move(&move_info);

        // Verify board and captured pieces restored
        let restored_fen = board.to_fen(player, &captured);
        assert_eq!(
            initial_fen, restored_fen,
            "Board should be restored after unmaking capture"
        );
        assert_eq!(
            captured_before.black.len(),
            captured.black.len(),
            "Captured pieces should be restored"
        );
    }
}

#[test]
fn test_unmake_promotion_move() {
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let player = Player::Black;

    // Find a promotion move
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    let promotion_move = moves.iter().find(|m| m.is_promotion);

    if let Some(test_move) = promotion_move {
        let initial_fen = board.to_fen(player, &captured);

        // Make move with info
        let move_info = board.make_move_with_info(test_move);
        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.add_piece(captured_piece.piece_type, player);
        }

        // Verify board changed
        let after_fen = board.to_fen(player, &captured);
        assert_ne!(
            initial_fen, after_fen,
            "Board should change after promotion"
        );

        // Unmake move
        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.remove_piece(captured_piece.piece_type, player);
        }
        board.unmake_move(&move_info);

        // Verify board restored
        let restored_fen = board.to_fen(player, &captured);
        assert_eq!(
            initial_fen, restored_fen,
            "Board should be restored after unmaking promotion"
        );

        // Verify original piece type is restored
        if let Some(from) = move_info.from {
            if let Some(piece) = board.get_piece(from) {
                assert_eq!(
                    piece.piece_type, move_info.original_piece_type,
                    "Original piece type should be restored"
                );
            }
        }
    }
}

#[test]
fn test_unmake_drop_move() {
    // Setup: get pieces in hand first
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let player = Player::Black;

    // Add a piece to hand (simulate a capture)
    captured.add_piece(PieceType::Pawn, player);

    let initial_fen = board.to_fen(player, &captured);
    let initial_captured_count = captured.count(PieceType::Pawn, player);

    // Find a drop move
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    let drop_move = moves.iter().find(|m| m.from.is_none());

    if let Some(test_move) = drop_move {
        // Make move with info
        let move_info = board.make_move_with_info(test_move);
        captured.remove_piece(PieceType::Pawn, player);

        // Verify board changed
        let after_fen = board.to_fen(player, &captured);
        assert_ne!(initial_fen, after_fen, "Board should change after drop");

        // Verify piece was removed from hand
        let after_captured_count = captured.count(PieceType::Pawn, player);
        assert_eq!(
            after_captured_count,
            initial_captured_count - 1,
            "Piece should be removed from hand"
        );

        // Unmake move
        board.unmake_move(&move_info);
        captured.add_piece(PieceType::Pawn, player);

        // Verify board restored
        let restored_fen = board.to_fen(player, &captured);
        assert_eq!(
            initial_fen, restored_fen,
            "Board should be restored after unmaking drop"
        );

        // Verify piece returned to hand
        let restored_captured_count = captured.count(PieceType::Pawn, player);
        assert_eq!(
            restored_captured_count, initial_captured_count,
            "Piece should be returned to hand"
        );
    }
}

#[test]
fn test_multiple_moves_unmake() {
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let mut player = Player::Black;

    let initial_fen = board.to_fen(player, &captured);
    let mut move_history: Vec<MoveInfo> = Vec::new();

    // Make 5 moves
    let move_generator = MoveGenerator::new();
    for _ in 0..5 {
        let moves = move_generator.generate_legal_moves(&board, player, &captured);
        if moves.is_empty() {
            break;
        }

        let test_move = &moves[0];
        let move_info = board.make_move_with_info(test_move);

        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.add_piece(captured_piece.piece_type, player);
        }

        move_history.push(move_info);
        player = player.opposite();
    }

    // Unmake all moves in reverse order
    while let Some(move_info) = move_history.pop() {
        player = player.opposite();

        if let Some(ref captured_piece) = move_info.captured_piece {
            captured.remove_piece(captured_piece.piece_type, player);
        }

        board.unmake_move(&move_info);
    }

    // Verify board restored to initial state
    let restored_fen = board.to_fen(Player::Black, &captured);
    assert_eq!(
        initial_fen, restored_fen,
        "Board should be fully restored after unmaking all moves"
    );
}

#[test]
fn test_unmake_with_captured_pieces_tracking() {
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let player = Player::Black;

    let initial_captured_black = captured.black.clone();
    let initial_captured_white = captured.white.clone();

    // Make several moves that involve captures
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    let mut move_history: Vec<MoveInfo> = Vec::new();
    let mut current_player = player;

    for move_ in moves.iter().take(10) {
        if move_.is_capture {
            let move_info = board.make_move_with_info(move_);

            if let Some(ref captured_piece) = move_info.captured_piece {
                captured.add_piece(captured_piece.piece_type, current_player);
            }

            move_history.push(move_info);
            current_player = current_player.opposite();

            // Unmake immediately to test
            if let Some(move_info) = move_history.pop() {
                current_player = current_player.opposite();

                if let Some(ref captured_piece) = move_info.captured_piece {
                    captured.remove_piece(captured_piece.piece_type, current_player);
                }

                board.unmake_move(&move_info);
                break;
            }
        }
    }

    // Verify captured pieces restored
    assert_eq!(
        captured.black, initial_captured_black,
        "Black captured pieces should be restored"
    );
    assert_eq!(
        captured.white, initial_captured_white,
        "White captured pieces should be restored"
    );
}

#[test]
fn test_move_info_structure() {
    let mut board = BitboardBoard::new();
    let player = Player::Black;
    let captured = CapturedPieces::new();

    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    assert!(!moves.is_empty(), "Should have legal moves");

    let test_move = &moves[0];
    let move_info = board.make_move_with_info(test_move);

    // Verify MoveInfo structure is correct
    assert_eq!(
        move_info.to, test_move.to,
        "MoveInfo should store correct 'to' position"
    );
    assert_eq!(
        move_info.player, test_move.player,
        "MoveInfo should store correct player"
    );
    assert_eq!(
        move_info.from, test_move.from,
        "MoveInfo should store correct 'from' position"
    );
    assert_eq!(
        move_info.was_promotion, test_move.is_promotion,
        "MoveInfo should store promotion status"
    );

    // Unmake and verify
    board.unmake_move(&move_info);
}

#[test]
fn test_unmake_in_search_context() {
    use shogi_engine::search::SearchEngine;
    use shogi_engine::time_utils::TimeSource;

    let mut engine = SearchEngine::new(None, 16);
    let mut board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    let initial_fen = board.to_fen(player, &captured);

    // Perform a shallow search
    let start_time = TimeSource::now();
    let result = engine.search_at_depth(&mut board, &captured, player, 2, 1000, -10000, 10000);

    // Verify board is still in original state after search
    let after_fen = board.to_fen(player, &captured);
    assert_eq!(
        initial_fen, after_fen,
        "Board should be unchanged after search (move unmaking should restore state)"
    );

    // Result should exist
    assert!(result.is_some(), "Search should return a result");
}
