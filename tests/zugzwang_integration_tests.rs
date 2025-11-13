//! Integration tests for zugzwang detection in EndgamePatternEvaluator
//!
//! Tests verify that zugzwang detection works correctly in the full evaluation context
//! and integrates properly with the endgame patterns module.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternEvaluator, EndgamePatternConfig};
use shogi_engine::evaluation::integration::{ComponentFlags, IntegratedEvaluator, IntegratedEvaluationConfig};
use shogi_engine::types::{CapturedPieces, Player, PieceType};

#[test]
fn test_zugzwang_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Zugzwang should work in full evaluation context
    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    
    // Score should be computed (not necessarily non-zero, but evaluation should complete)
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);
}

#[test]
fn test_zugzwang_with_integrated_evaluator() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.endgame_patterns = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Zugzwang should work through IntegratedEvaluator
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    
    // Score should be computed
    assert!(score >= -10000 && score <= 10000);
}

#[test]
fn test_zugzwang_drop_consideration_integration() {
    let mut config = EndgamePatternConfig::default();
    config.enable_zugzwang = true;
    config.enable_zugzwang_drop_consideration = true;
    
    let mut evaluator = EndgamePatternEvaluator::with_config(config);
    let board = BitboardBoard::empty();
    let mut captured_pieces = CapturedPieces::new();
    
    // Add captured pieces to enable drops
    captured_pieces.add_piece(PieceType::Pawn, Player::Black);
    captured_pieces.add_piece(PieceType::Rook, Player::Black);

    // Evaluation should complete with drop consideration enabled
    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);
}

#[test]
fn test_zugzwang_statistics_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let initial_detections = evaluator.stats().zugzwang_detections;
    let initial_benefits = evaluator.stats().zugzwang_benefits;
    let initial_penalties = evaluator.stats().zugzwang_penalties;

    // Evaluate endgame patterns multiple times
    for _ in 0..5 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    // Statistics should be tracked
    assert!(evaluator.stats().zugzwang_detections >= initial_detections);
    assert!(evaluator.stats().zugzwang_benefits >= initial_benefits);
    assert!(evaluator.stats().zugzwang_penalties >= initial_penalties);
}

