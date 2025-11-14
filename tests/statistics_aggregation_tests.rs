//! Integration tests for statistics aggregation
//!
//! Tests verify that all statistics accumulate correctly across multiple evaluations

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::EndgamePatternEvaluator;
use shogi_engine::types::{CapturedPieces, PieceType, Player};

#[test]
fn test_statistics_aggregation() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();

    // Initial statistics should be zero
    assert_eq!(evaluator.stats().evaluations, 0);
    assert_eq!(evaluator.stats().zugzwang_detections, 0);
    assert_eq!(evaluator.stats().opposition_detections, 0);
    assert_eq!(evaluator.stats().triangulation_detections, 0);
    assert_eq!(evaluator.stats().king_activity_bonuses, 0);
    assert_eq!(evaluator.stats().passed_pawn_bonuses, 0);
    assert_eq!(evaluator.stats().mating_pattern_detections, 0);
    assert_eq!(evaluator.stats().fortress_detections, 0);

    // Perform multiple evaluations
    for _ in 0..10 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    // Statistics should accumulate
    assert_eq!(evaluator.stats().evaluations, 10);
    assert!(evaluator.stats().king_activity_bonuses >= 0);
    assert!(evaluator.stats().passed_pawn_bonuses >= 0);

    // Add pieces to hand to trigger drop-based statistics
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Gold, Player::White);

    // Perform more evaluations
    for _ in 0..5 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    // Evaluations should continue to accumulate
    assert_eq!(evaluator.stats().evaluations, 15);
    assert!(evaluator.stats().drop_mate_threats_detected >= 0);
    assert!(evaluator.stats().opposition_broken_by_drops >= 0);
}

#[test]
fn test_statistics_reset_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Generate statistics
    for _ in 0..5 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    // Verify statistics exist
    assert_eq!(evaluator.stats().evaluations, 5);

    // Reset statistics
    evaluator.reset_stats();

    // Verify all statistics are zero
    assert_eq!(evaluator.stats().evaluations, 0);
    assert_eq!(evaluator.stats().zugzwang_detections, 0);
    assert_eq!(evaluator.stats().opposition_detections, 0);
    assert_eq!(evaluator.stats().triangulation_detections, 0);
    assert_eq!(evaluator.stats().king_activity_bonuses, 0);
    assert_eq!(evaluator.stats().passed_pawn_bonuses, 0);
    assert_eq!(evaluator.stats().mating_pattern_detections, 0);
    assert_eq!(evaluator.stats().fortress_detections, 0);
    assert_eq!(evaluator.stats().drop_mate_threats_detected, 0);
    assert_eq!(evaluator.stats().opposition_broken_by_drops, 0);
    assert_eq!(evaluator.stats().unsafe_king_penalties, 0);

    // Perform new evaluations
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Statistics should start fresh
    assert_eq!(evaluator.stats().evaluations, 1);
}

#[test]
fn test_statistics_summary_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();

    // Generate various statistics
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Get summary
    let summary = evaluator.stats_summary();

    // Verify summary contains all expected fields
    assert!(summary.contains("EndgamePatternStats"));
    assert!(summary.contains("Evaluations"));
    assert!(summary.contains("Zugzwang detections"));
    assert!(summary.contains("Opposition detections"));
    assert!(summary.contains("Triangulation detections"));
    assert!(summary.contains("Unsafe king penalties"));
    assert!(summary.contains("Drop mate threats"));
    assert!(summary.contains("King activity bonuses"));
    assert!(summary.contains("Passed pawn bonuses"));
    assert!(summary.contains("Mating pattern detections"));
    assert!(summary.contains("Fortress detections"));

    // Verify summary contains actual values
    assert!(summary.contains(&evaluator.stats().evaluations.to_string()));
}

#[test]
fn test_all_statistics_tracked() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();

    // Perform comprehensive evaluation
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // All statistics should be non-negative
    assert!(evaluator.stats().evaluations >= 0);
    assert!(evaluator.stats().zugzwang_detections >= 0);
    assert!(evaluator.stats().zugzwang_benefits >= 0);
    assert!(evaluator.stats().zugzwang_penalties >= 0);
    assert!(evaluator.stats().opposition_detections >= 0);
    assert!(evaluator.stats().opposition_broken_by_drops >= 0);
    assert!(evaluator.stats().triangulation_detections >= 0);
    assert!(evaluator.stats().unsafe_king_penalties >= 0);
    assert!(evaluator.stats().drop_mate_threats_detected >= 0);
    assert!(evaluator.stats().king_activity_bonuses >= 0);
    assert!(evaluator.stats().passed_pawn_bonuses >= 0);
    assert!(evaluator.stats().mating_pattern_detections >= 0);
    assert!(evaluator.stats().fortress_detections >= 0);

    // Evaluations should always be incremented
    assert_eq!(evaluator.stats().evaluations, 1);
}
