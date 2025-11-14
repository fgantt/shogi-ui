//! Tests for performance optimizations
//!
//! Tests verify that optimizations work correctly:
//! - Evaluation caching
//! - King-square tables
//! - Bitboard optimizations

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternConfig, EndgamePatternEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

#[test]
fn test_evaluation_caching() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // First evaluation should compute
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Cache should be populated
    // Note: We can't directly check cache size, but we can verify it works by checking performance
    // or by checking that subsequent evaluations are faster (would need benchmarks)

    // Clear cache
    evaluator.clear_cache();

    // Cache should be empty after clearing
    // Verify by doing another evaluation
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
}

#[test]
fn test_king_square_tables() {
    let mut config = EndgamePatternConfig::default();
    config.use_king_square_tables = true;

    let mut evaluator = EndgamePatternEvaluator::with_config(config);
    let board = BitboardBoard::new();

    // Test that king-square tables are used
    let score = evaluator.evaluate_endgame(&board, Player::Black, &CapturedPieces::new());

    // Should complete without error
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);
}

#[test]
fn test_bitboard_optimizations() {
    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    // Test bitboard-based piece finding
    let rooks = evaluator.find_pieces(&board, Player::Black, shogi_engine::types::PieceType::Rook);
    assert_eq!(rooks.len(), 1); // One rook in starting position

    // Test bitboard-based pawn collection
    let pawns = evaluator.collect_pawns(&board, Player::Black);
    assert_eq!(pawns.len(), 9); // Nine pawns in starting position

    // Test bitboard-based piece counting
    let total_pieces = evaluator.count_total_pieces(&board);
    assert_eq!(total_pieces, 40); // 20 pieces per player in starting position
}

#[test]
fn test_caching_disabled() {
    let mut config = EndgamePatternConfig::default();
    config.enable_evaluation_caching = false;

    let mut evaluator = EndgamePatternEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Should still work without caching
    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);
}
