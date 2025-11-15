//! Tests for Shogi-specific hash handling with known positions
//!
//! This module provides comprehensive tests using well-known Shogi positions
//! to validate the correctness of hash handling for all Shogi-specific features.

use crate::bitboards::BitboardBoard;
use crate::search::shogi_hash::{ShogiHashHandler, ShogiMoveValidator};
use crate::search::RepetitionState;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Piece, PieceType, Player, Position};

/// Known Shogi positions for testing
pub struct ShogiPositionTests;

impl ShogiPositionTests {
    /// Test the starting position hash
    pub fn test_starting_position() -> bool {
        let handler = ShogiHashHandler::new_default();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let hash = handler.get_position_hash(&board, Player::Black, &captured_pieces);

        // Starting position should have a non-zero hash
        hash != 0
    }

    /// Test hash after a simple pawn move
    pub fn test_pawn_move_hash() -> bool {
        let handler = ShogiHashHandler::new_default();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Get initial hash
        let initial_hash = handler.get_position_hash(&board, Player::Black, &captured_pieces);

        // Make a pawn move
        let pawn_move = Move::new_move(
            Position::new(6, 4), // Black pawn from 7g
            Position::new(5, 4), // Black pawn to 6g
            PieceType::Pawn,
            Player::Black,
            false, // no promotion
        );

        let updated_hash = handler.update_hash_for_normal_move(
            initial_hash,
            &pawn_move,
            &board,
            &captured_pieces,
            &captured_pieces,
        );

        // Hash should change after the move
        initial_hash != updated_hash
    }

    /// Test hash for a drop move
    pub fn test_drop_move_hash() -> bool {
        let handler = ShogiHashHandler::new_default();
        let _board = BitboardBoard::new();
        let mut captured_before = CapturedPieces::new();
        let captured_after = CapturedPieces::new();

        // Add a pawn to hand
        captured_before.add_piece(PieceType::Pawn, Player::Black);

        // Create a drop move
        let drop_move = Move::new_drop(PieceType::Pawn, Position::new(4, 4), Player::Black);

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_drop_move(
            initial_hash,
            &drop_move,
            &captured_before,
            &captured_after,
        );

        // Hash should change due to the drop move
        initial_hash != updated_hash
    }

    /// Test hash for a capture move
    pub fn test_capture_move_hash() -> bool {
        let handler = ShogiHashHandler::new_default();
        let mut board = BitboardBoard::new();
        let captured_before = CapturedPieces::new();
        let mut captured_after = CapturedPieces::new();

        // Set up a capture scenario
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::White),
            Position::new(5, 5),
        );

        // Create a capture move
        let capture_move = Move {
            from: Some(Position::new(6, 5)),
            to: Position::new(5, 5),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: true,
            captured_piece: Some(Piece::new(PieceType::Pawn, Player::White)),
            gives_check: false,
            is_recapture: false,
        };

        // Add captured piece to hand
        captured_after.add_piece(PieceType::Pawn, Player::Black);

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_capture_move(
            initial_hash,
            &capture_move,
            &board,
            &captured_before,
            &captured_after,
        );

        // Hash should change due to the capture move
        initial_hash != updated_hash
    }

    /// Test hash for a promotion move
    pub fn test_promotion_move_hash() -> bool {
        let handler = ShogiHashHandler::new_default();
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Set up a promotion scenario
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(1, 1),
        );

        // Create a promotion move
        let promotion_move = Move::new_move(
            Position::new(1, 1),
            Position::new(0, 1),
            PieceType::Pawn,
            Player::Black,
            true, // is_promotion
        );

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_promotion_move(
            initial_hash,
            &promotion_move,
            &board,
            &captured_pieces,
            &captured_pieces,
        );

        // Hash should change due to the promotion move
        initial_hash != updated_hash
    }

    /// Test repetition detection
    pub fn test_repetition_detection() -> bool {
        let mut handler = ShogiHashHandler::new_default();
        let hash1 = 0x1111111111111111;
        let hash2 = 0x2222222222222222;
        let hash3 = 0x1111111111111111; // Same as hash1

        // Add positions to history
        handler.add_position_to_history(hash1);
        handler.add_position_to_history(hash2);
        handler.add_position_to_history(hash3);

        // Check repetition states
        let state_hash1 = handler.get_repetition_state_for_hash(hash1);
        let state_hash2 = handler.get_repetition_state_for_hash(hash2);

        // hash1 should be two-fold, hash2 should be none
        state_hash1 == RepetitionState::TwoFold && state_hash2 == RepetitionState::None
    }

    /// Test four-fold repetition (draw)
    pub fn test_four_fold_repetition() -> bool {
        let mut handler = ShogiHashHandler::new_default();
        let hash = 0x1111111111111111;

        // Add the same position 4 times
        for _ in 0..4 {
            handler.add_position_to_history(hash);
        }

        // Should be four-fold repetition (draw)
        let state = handler.get_repetition_state_for_hash(hash);
        state == RepetitionState::FourFold && handler.is_repetition(hash)
    }

    /// Test hand piece counting in hash
    pub fn test_hand_piece_counting() -> bool {
        let handler = ShogiHashHandler::new_default();
        let mut captured_before = CapturedPieces::new();
        let mut captured_after = CapturedPieces::new();

        // Add pieces to hand
        captured_before.add_piece(PieceType::Pawn, Player::Black);
        captured_before.add_piece(PieceType::Pawn, Player::Black);
        captured_after.add_piece(PieceType::Pawn, Player::Black);

        let initial_hash = 0x1234567890ABCDEF;

        // Test adding a piece to hand
        let hash_after_add = handler.update_hand_piece_hash(
            initial_hash,
            PieceType::Pawn,
            Player::Black,
            &captured_before,
            &captured_after,
        );

        // Test removing a piece from hand
        let hash_after_remove = handler.update_hand_piece_hash(
            hash_after_add,
            PieceType::Pawn,
            Player::Black,
            &captured_after,
            &captured_before,
        );

        // Hash should change when hand pieces change
        initial_hash != hash_after_add && hash_after_add != hash_after_remove
    }

    /// Test move validation for drop moves
    pub fn test_drop_move_validation() -> bool {
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a pawn to hand
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);

        // Valid drop move
        let valid_drop = Move::new_drop(PieceType::Pawn, Position::new(4, 4), Player::Black);
        let is_valid =
            ShogiMoveValidator::validate_drop_move(&valid_drop, &board, &captured_pieces);

        // Invalid drop move (no piece in hand)
        let captured_empty = CapturedPieces::new();
        let is_invalid =
            ShogiMoveValidator::validate_drop_move(&valid_drop, &board, &captured_empty);

        is_valid && !is_invalid
    }

    /// Test move validation for capture moves
    pub fn test_capture_move_validation() -> bool {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Set up capture scenario
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::White),
            Position::new(5, 5),
        );

        // Valid capture move
        let valid_capture = Move {
            from: Some(Position::new(6, 5)),
            to: Position::new(5, 5),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: true,
            captured_piece: Some(Piece::new(PieceType::Pawn, Player::White)),
            gives_check: false,
            is_recapture: false,
        };

        let is_valid =
            ShogiMoveValidator::validate_capture_move(&valid_capture, &board, &captured_pieces);

        // Invalid capture move (not a capture)
        let invalid_capture = Move {
            from: Some(Position::new(6, 5)),
            to: Position::new(5, 5),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: false, // Not a capture
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let is_invalid =
            ShogiMoveValidator::validate_capture_move(&invalid_capture, &board, &captured_pieces);

        is_valid && !is_invalid
    }

    /// Test move validation for promotion moves
    pub fn test_promotion_move_validation() -> bool {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Valid promotion move (from promotion zone)
        let valid_promotion = Move::new_move(
            Position::new(1, 1), // In Black's promotion zone
            Position::new(0, 1),
            PieceType::Pawn,
            Player::Black,
            true, // is_promotion
        );

        let is_valid =
            ShogiMoveValidator::validate_promotion_move(&valid_promotion, &board, &captured_pieces);

        // Invalid promotion move (not a promotion)
        let invalid_promotion = Move::new_move(
            Position::new(1, 1),
            Position::new(0, 1),
            PieceType::Pawn,
            Player::Black,
            false, // Not a promotion
        );

        let is_invalid = ShogiMoveValidator::validate_promotion_move(
            &invalid_promotion,
            &board,
            &captured_pieces,
        );

        is_valid && !is_invalid
    }

    /// Test hash uniqueness for different positions
    pub fn test_hash_uniqueness() -> bool {
        let handler = ShogiHashHandler::new_default();
        let board1 = BitboardBoard::new();
        let board2 = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Same position should have same hash
        let hash1_black = handler.get_position_hash(&board1, Player::Black, &captured_pieces);
        let hash2_black = handler.get_position_hash(&board2, Player::Black, &captured_pieces);

        // Different side to move should have different hash
        let hash1_white = handler.get_position_hash(&board1, Player::White, &captured_pieces);

        // Same position, same side should have same hash
        let same_hash = hash1_black == hash2_black;

        // Different side should have different hash
        let different_hash = hash1_black != hash1_white;

        same_hash && different_hash
    }

    /// Run all position tests
    pub fn run_all_tests() -> (usize, usize) {
        let mut passed = 0;
        let total = 12;

        // Test 1: Starting Position
        if Self::test_starting_position() {
            println!("✅ Starting Position: PASSED");
            passed += 1;
        } else {
            println!("❌ Starting Position: FAILED");
        }

        // Test 2: Pawn Move Hash
        if Self::test_pawn_move_hash() {
            println!("✅ Pawn Move Hash: PASSED");
            passed += 1;
        } else {
            println!("❌ Pawn Move Hash: FAILED");
        }

        // Test 3: Drop Move Hash
        if Self::test_drop_move_hash() {
            println!("✅ Drop Move Hash: PASSED");
            passed += 1;
        } else {
            println!("❌ Drop Move Hash: FAILED");
        }

        // Test 4: Capture Move Hash
        if Self::test_capture_move_hash() {
            println!("✅ Capture Move Hash: PASSED");
            passed += 1;
        } else {
            println!("❌ Capture Move Hash: FAILED");
        }

        // Test 5: Promotion Move Hash
        if Self::test_promotion_move_hash() {
            println!("✅ Promotion Move Hash: PASSED");
            passed += 1;
        } else {
            println!("❌ Promotion Move Hash: FAILED");
        }

        // Test 6: Repetition Detection
        if Self::test_repetition_detection() {
            println!("✅ Repetition Detection: PASSED");
            passed += 1;
        } else {
            println!("❌ Repetition Detection: FAILED");
        }

        // Test 7: Four-fold Repetition
        if Self::test_four_fold_repetition() {
            println!("✅ Four-fold Repetition: PASSED");
            passed += 1;
        } else {
            println!("❌ Four-fold Repetition: FAILED");
        }

        // Test 8: Hand Piece Counting
        if Self::test_hand_piece_counting() {
            println!("✅ Hand Piece Counting: PASSED");
            passed += 1;
        } else {
            println!("❌ Hand Piece Counting: FAILED");
        }

        // Test 9: Drop Move Validation
        if Self::test_drop_move_validation() {
            println!("✅ Drop Move Validation: PASSED");
            passed += 1;
        } else {
            println!("❌ Drop Move Validation: FAILED");
        }

        // Test 10: Capture Move Validation
        if Self::test_capture_move_validation() {
            println!("✅ Capture Move Validation: PASSED");
            passed += 1;
        } else {
            println!("❌ Capture Move Validation: FAILED");
        }

        // Test 11: Promotion Move Validation
        if Self::test_promotion_move_validation() {
            println!("✅ Promotion Move Validation: PASSED");
            passed += 1;
        } else {
            println!("❌ Promotion Move Validation: FAILED");
        }

        // Test 12: Hash Uniqueness
        if Self::test_hash_uniqueness() {
            println!("✅ Hash Uniqueness: PASSED");
            passed += 1;
        } else {
            println!("❌ Hash Uniqueness: FAILED");
        }

        println!("\nTest Results: {}/{} tests passed", passed, total);
        (passed, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_shogi_position_tests() {
        let (passed, total) = ShogiPositionTests::run_all_tests();
        assert_eq!(passed, total, "All Shogi position tests should pass");
    }

    #[test]
    fn test_starting_position_hash() {
        assert!(ShogiPositionTests::test_starting_position());
    }

    #[test]
    fn test_pawn_move_hash() {
        assert!(ShogiPositionTests::test_pawn_move_hash());
    }

    #[test]
    fn test_drop_move_hash() {
        assert!(ShogiPositionTests::test_drop_move_hash());
    }

    #[test]
    fn test_capture_move_hash() {
        assert!(ShogiPositionTests::test_capture_move_hash());
    }

    #[test]
    fn test_promotion_move_hash() {
        assert!(ShogiPositionTests::test_promotion_move_hash());
    }

    #[test]
    fn test_repetition_detection() {
        assert!(ShogiPositionTests::test_repetition_detection());
    }

    #[test]
    fn test_four_fold_repetition() {
        assert!(ShogiPositionTests::test_four_fold_repetition());
    }

    #[test]
    fn test_hand_piece_counting() {
        assert!(ShogiPositionTests::test_hand_piece_counting());
    }

    #[test]
    fn test_drop_move_validation() {
        assert!(ShogiPositionTests::test_drop_move_validation());
    }

    #[test]
    fn test_capture_move_validation() {
        assert!(ShogiPositionTests::test_capture_move_validation());
    }

    #[test]
    fn test_promotion_move_validation() {
        assert!(ShogiPositionTests::test_promotion_move_validation());
    }

    #[test]
    fn test_hash_uniqueness() {
        assert!(ShogiPositionTests::test_hash_uniqueness());
    }
}
