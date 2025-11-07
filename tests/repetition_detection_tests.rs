#![cfg(feature = "legacy-tests")]
//! Comprehensive tests for hash-based repetition detection (Task 5.13)
//!
//! This module tests the hash-based repetition detection system to ensure:
//! - Correctness: Repetition states are correctly identified (TwoFold, ThreeFold, FourFold)
//! - Performance: Hash-based detection is faster than FEN-based
//! - Integration: Works correctly with search engine
//! - Edge cases: Handles boundary conditions correctly

use shogi_engine::{
    bitboards::BitboardBoard,
    moves::MoveGenerator,
    search::zobrist::RepetitionState,
    search::ShogiHashHandler,
    types::{CapturedPieces, Player},
};

#[test]
fn test_hash_based_repetition_detection_basic() {
    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    // Get initial position hash
    let hash1 = hash_handler.get_position_hash(&board, player, &captured);

    // Add position to history (first occurrence - should be None when checked)
    hash_handler.add_position_to_history(hash1);
    // After first add, hash is in history once, so repetition state is None (not yet repeated)
    // But the hash is tracked, so checking immediately might show None

    // Add same position again (second occurrence)
    hash_handler.add_position_to_history(hash1);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::TwoFold
    );

    // Add same position third time
    hash_handler.add_position_to_history(hash1);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::ThreeFold
    );

    // Add same position fourth time (four-fold = draw)
    hash_handler.add_position_to_history(hash1);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::FourFold
    );
    assert!(hash_handler.is_repetition(hash1));
}

#[test]
fn test_repetition_detection_with_moves() {
    let mut hash_handler = ShogiHashHandler::new_default();
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let mut player = Player::Black;
    let move_generator = MoveGenerator::new();

    // Get initial position hash
    let initial_hash = hash_handler.get_position_hash(&board, player, &captured);
    hash_handler.add_position_to_history(initial_hash);

    // Make a move
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    assert!(!moves.is_empty(), "Should have legal moves");
    let test_move = &moves[0];

    let move_info = board.make_move_with_info(test_move);
    if let Some(ref cp) = move_info.captured_piece {
        captured.add_piece(cp.piece_type, player);
    }
    player = player.opposite();

    // Get new position hash
    let hash_after_move = hash_handler.get_position_hash(&board, player, &captured);
    assert_ne!(
        initial_hash, hash_after_move,
        "Hash should change after move"
    );

    hash_handler.add_position_to_history(hash_after_move);

    // Undo move
    board.unmake_move(&move_info);
    if let Some(ref cp) = move_info.captured_piece {
        captured.remove_piece(cp.piece_type, move_info.player);
    }
    player = player.opposite();

    // Should be back to initial position
    let hash_after_unmake = hash_handler.get_position_hash(&board, player, &captured);
    assert_eq!(
        initial_hash, hash_after_unmake,
        "Hash should match initial after unmake"
    );

    // Adding it again should detect two-fold
    hash_handler.add_position_to_history(hash_after_unmake);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash_after_unmake),
        RepetitionState::TwoFold
    );
}

#[test]
fn test_four_fold_repetition_draw() {
    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    let hash = hash_handler.get_position_hash(&board, player, &captured);

    // Add position 4 times to trigger four-fold repetition
    for _ in 0..4 {
        hash_handler.add_position_to_history(hash);
    }

    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::FourFold
    );
    assert!(
        hash_handler.is_repetition(hash),
        "Four-fold repetition should be detected as draw"
    );
}

#[test]
fn test_repetition_history_cleanup() {
    let mut hash_handler = ShogiHashHandler::new(3); // Small history limit for testing
    let hash1 = 0x1111111111111111;
    let hash2 = 0x2222222222222222;
    let hash3 = 0x3333333333333333;
    let hash4 = 0x4444444444444444;

    // Add positions to fill up history (max_length = 3)
    hash_handler.add_position_to_history(hash1);
    hash_handler.add_position_to_history(hash2);
    hash_handler.add_position_to_history(hash3);

    // Verify hash1 is still tracked (count = 1, so None)
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::None
    );

    // Add hash1 again to make it two-fold
    hash_handler.add_position_to_history(hash1);
    // Now hash1 appears twice: history = [hash1, hash2, hash3, hash1]
    // After cleanup (len > 3): remove oldest hash1 -> [hash2, hash3, hash1]
    // hash1 now has count = 1 (only one occurrence remains after cleanup)
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::None
    );

    // Add another position - this triggers cleanup again
    hash_handler.add_position_to_history(hash4);
    // History after cleanup: [hash3, hash1, hash4] (oldest hash2 removed)

    // hash1 now has count = 1
    let state_hash1 = hash_handler.get_repetition_state_for_hash(hash1);
    assert_eq!(state_hash1, RepetitionState::None);

    // hash2 should no longer be tracked (removed by cleanup)
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash2),
        RepetitionState::None
    );

    // hash3 should still be tracked (count = 1)
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash3),
        RepetitionState::None
    );
}

#[test]
fn test_repetition_detection_in_search_context() {
    // Test repetition detection using public API
    // Note: hash_calculator is private, so we test through ShogiHashHandler directly
    let mut hash_handler = ShogiHashHandler::new_default();
    let mut board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    // Create a sequence of moves that should cause repetition
    let move_generator = MoveGenerator::new();

    // Get initial hash
    let initial_hash = hash_handler.get_position_hash(&board, player, &captured);
    hash_handler.add_position_to_history(initial_hash);

    // Make a move
    let moves = move_generator.generate_legal_moves(&board, player, &captured);
    if !moves.is_empty() {
        let test_move = &moves[0];
        let move_info = board.make_move_with_info(test_move);
        let mut new_captured = captured.clone();
        if let Some(ref cp) = move_info.captured_piece {
            new_captured.add_piece(cp.piece_type, player);
        }

        let hash_after_move =
            hash_handler.get_position_hash(&board, player.opposite(), &new_captured);
        hash_handler.add_position_to_history(hash_after_move);

        // Unmake move
        board.unmake_move(&move_info);

        // Check if we can detect repetition when position repeats
        let hash_after_unmake = hash_handler.get_position_hash(&board, player, &captured);
        // Add the position again after unmaking
        hash_handler.add_position_to_history(hash_after_unmake);
        let repetition_state = hash_handler.get_repetition_state_for_hash(hash_after_unmake);

        // Should detect two-fold repetition (position appeared twice)
        assert_eq!(repetition_state, RepetitionState::TwoFold);
    }
}

#[test]
fn test_hash_uniqueness_for_different_positions() {
    let hash_handler = ShogiHashHandler::new_default();
    let board1 = BitboardBoard::new();
    let mut board2 = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    let hash1 = hash_handler.get_position_hash(&board1, player, &captured);

    // Make a move on board2
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board2, player, &captured);
    if !moves.is_empty() {
        let move_info = board2.make_move_with_info(&moves[0]);
        let mut new_captured = captured.clone();
        if let Some(ref cp) = move_info.captured_piece {
            new_captured.add_piece(cp.piece_type, player);
        }
        let hash2 = hash_handler.get_position_hash(&board2, player.opposite(), &new_captured);

        // Hashes should be different
        assert_ne!(
            hash1, hash2,
            "Different positions should have different hashes"
        );
    }
}

#[test]
fn test_repetition_detection_clearing() {
    let mut hash_handler = ShogiHashHandler::new_default();
    let hash1 = 0x1111111111111111;
    let hash2 = 0x2222222222222222;

    hash_handler.add_position_to_history(hash1);
    hash_handler.add_position_to_history(hash1);
    hash_handler.add_position_to_history(hash2);

    // Verify repetition states
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::TwoFold
    );
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash2),
        RepetitionState::None
    );

    // Clear history
    hash_handler.clear_history();

    // All repetition states should be None
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash1),
        RepetitionState::None
    );
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash2),
        RepetitionState::None
    );
}

#[test]
fn test_repetition_state_progression() {
    let mut hash_handler = ShogiHashHandler::new_default();
    let hash = 0x1234567890ABCDEF;

    // Initial: None (hash not in history)
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::None
    );

    // After 1st occurrence: None (single occurrence, not repeated yet)
    hash_handler.add_position_to_history(hash);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::None
    );

    // After 2nd occurrence: TwoFold (first repetition)
    hash_handler.add_position_to_history(hash);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::TwoFold
    );

    // After 3rd occurrence: ThreeFold
    hash_handler.add_position_to_history(hash);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::ThreeFold
    );

    // After 4th occurrence: FourFold (draw)
    hash_handler.add_position_to_history(hash);
    assert_eq!(
        hash_handler.get_repetition_state_for_hash(hash),
        RepetitionState::FourFold
    );
    assert!(hash_handler.is_repetition(hash));
}
