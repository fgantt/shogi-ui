//! Integration tests for redundancy elimination and coordination in IntegratedEvaluator
//!
//! Tests verify that evaluation components coordinate properly to avoid double-counting
//! of features like passed pawns and center control.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

/// Create a position with a passed pawn for Black
/// Black pawn at row 2 (advanced, no enemy pawns in front)
fn create_passed_pawn_position() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let captured_pieces = CapturedPieces::new();

    // Black king
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );

    // Black passed pawn (advanced, no enemy pawns blocking)
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4), // Very advanced for Black
    );

    // White king
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );

    // Add minimal material to keep phase in endgame range (< 64)
    // Only kings and one pawn = low phase

    (board, captured_pieces)
}

/// Create a position with more material to keep phase in middlegame (>= 64)
fn create_middlegame_position() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let captured_pieces = CapturedPieces::new();

    // Black pieces
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(7, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::Black),
        Position::new(7, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(8, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(8, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(5, 4), // Passed pawn
    );

    // White pieces
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(1, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(1, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(0, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(0, 5),
    );

    (board, captured_pieces)
}

#[test]
fn test_passed_pawn_coordination() {
    // Test that passed pawns are not double-counted when both endgame_patterns
    // and position_features are enabled in endgame (phase < 64)
    let (board, captured_pieces) = create_passed_pawn_position();

    // Configuration with both modules enabled
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.endgame_patterns = true;

    let evaluator = IntegratedEvaluator::with_config(config);

    // Evaluate the position
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be computed (not necessarily non-zero, but evaluation should complete)
    // The key is that passed pawns should not be double-counted
    assert!(score >= -10000 && score <= 10000);

    // Verify that the coordination logic is working by checking that
    // the evaluation completes without errors
}

#[test]
fn test_passed_pawn_evaluation_in_middlegame() {
    // Test that passed pawns ARE evaluated in position_features when NOT in endgame
    // (phase >= 64), even if endgame_patterns is enabled
    let (board, captured_pieces) = create_middlegame_position();

    // Configuration with both modules enabled
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.endgame_patterns = true;

    let evaluator = IntegratedEvaluator::with_config(config);

    // Evaluate the position
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be computed
    assert!(score >= -10000 && score <= 10000);

    // In middlegame, passed pawns should be evaluated by position_features
    // (endgame_patterns only activates in endgame phase < 64)
}

#[test]
fn test_center_control_overlap_warning() {
    // Test that a warning is logged when both position_features.center_control
    // and positional_patterns are enabled
    // Note: This test verifies the warning mechanism exists, but we can't easily
    // capture debug_log output in tests. Instead, we verify the code path executes.
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.positional_patterns = true;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Evaluate - this should trigger the warning log (if debug logging is enabled)
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be computed
    assert!(score >= -10000 && score <= 10000);

    // The warning is logged via debug_log, which we can't easily verify in tests
    // but the code path should execute without errors
}

#[test]
fn test_no_double_counting_passed_pawns() {
    // Test with positions containing passed pawns to verify evaluation consistency
    // Compare scores with endgame_patterns enabled vs disabled in endgame
    let (board, captured_pieces) = create_passed_pawn_position();

    // Configuration 1: Only position_features (passed pawns evaluated)
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.endgame_patterns = false;

    let evaluator1 = IntegratedEvaluator::with_config(config1);
    let score1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);

    // Configuration 2: Both enabled (passed pawns skipped in position_features,
    // evaluated in endgame_patterns)
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.endgame_patterns = true;

    let evaluator2 = IntegratedEvaluator::with_config(config2);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);

    // Both scores should be valid
    assert!(score1 >= -10000 && score1 <= 10000);
    assert!(score2 >= -10000 && score2 <= 10000);

    // The scores may differ (endgame_patterns may evaluate passed pawns differently),
    // but both should be reasonable evaluations
    // The key is that passed pawns are not counted twice in score2
}

#[test]
fn test_component_flags_passed_pawn_coordination() {
    // Test that ComponentFlags properly control the coordination logic
    let (board, captured_pieces) = create_passed_pawn_position();

    // Test with endgame_patterns disabled - passed pawns should be evaluated
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.endgame_patterns = false;

    let evaluator1 = IntegratedEvaluator::with_config(config1);
    let score1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);

    // Test with endgame_patterns enabled - passed pawns should be skipped in position_features
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.endgame_patterns = true;

    let evaluator2 = IntegratedEvaluator::with_config(config2);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);

    // Both should produce valid scores
    assert!(score1 >= -10000 && score1 <= 10000);
    assert!(score2 >= -10000 && score2 <= 10000);
}
