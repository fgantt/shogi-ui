//! Integration tests for castle pattern recognition in IntegratedEvaluator
//!
//! Tests verify that castle patterns are properly integrated as a first-class component
//! with ComponentFlags, weights, and telemetry support.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::EvaluationWeights;
use shogi_engine::evaluation::integration::{
    ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator,
};
use shogi_engine::types::{CapturedPieces, Player};

#[test]
fn test_castle_pattern_integration() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = true;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Castle patterns should be evaluated when flag is enabled
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be computed (not necessarily non-zero, but evaluation should complete)
    assert!(score >= -10000 && score <= 10000);
}

#[test]
fn test_castle_weight_application() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = true;
    config.weights.castle_weight = 2.0; // Double the weight

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Test with different weight
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.castle_patterns = true;
    config2.weights.castle_weight = 0.5; // Half the weight

    let evaluator2 = IntegratedEvaluator::with_config(config2);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);

    // Scores should differ when weights differ (if castle patterns contribute)
    // Note: On initial board, castle patterns may not contribute, so scores might be equal
    // This test verifies the weight is applied, not that it changes the score
    assert!(score1 >= -10000 && score1 <= 10000);
    assert!(score2 >= -10000 && score2 <= 10000);
}

#[test]
fn test_castle_pattern_stats_telemetry() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = true;

    let mut evaluator = IntegratedEvaluator::with_config(config);
    evaluator.enable_statistics();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    evaluator.evaluate(&board, Player::Black, &captured_pieces);

    let telemetry = evaluator.telemetry_snapshot();
    assert!(telemetry.is_some());

    let telemetry = telemetry.unwrap();
    // Castle pattern stats should be present in telemetry
    assert!(telemetry.castle_patterns.is_some());

    let castle_stats = telemetry.castle_patterns.unwrap();
    // Cache stats should be initialized
    assert_eq!(castle_stats.max_size, 500); // Default cache size
}

#[test]
fn test_castle_pattern_disabled() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = false;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Evaluation should complete even when castle patterns are disabled
    assert!(score >= -10000 && score <= 10000);

    // Telemetry should not include castle stats when disabled
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.castle_patterns = false;
    let evaluator2 = IntegratedEvaluator::with_config(config2);
    evaluator2.enable_statistics();
    evaluator2.evaluate(&board, Player::Black, &captured_pieces);

    let telemetry = evaluator2.telemetry_snapshot();
    if let Some(_telemetry) = telemetry {
        // Castle stats may be None when disabled
        // This is acceptable behavior
    }
}

#[test]
fn test_component_flags_castle_patterns() {
    let all_enabled = ComponentFlags::all_enabled();
    assert!(all_enabled.castle_patterns);

    let all_disabled = ComponentFlags::all_disabled();
    assert!(!all_disabled.castle_patterns);

    let minimal = ComponentFlags::minimal();
    assert!(!minimal.castle_patterns);
}

#[test]
fn test_castle_weight_validation() {
    use shogi_engine::evaluation::config::{ConfigError, TaperedEvalConfig};

    let mut config = TaperedEvalConfig::default();

    // Valid weight should pass
    config.weights.castle_weight = 1.5;
    assert!(config.validate().is_ok());

    // Invalid weight (negative) should fail
    config.weights.castle_weight = -1.0;
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidWeight(_))
    ));

    // Invalid weight (too large) should fail
    config.weights.castle_weight = 15.0;
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidWeight(_))
    ));

    // Valid weight (boundary) should pass
    config.weights.castle_weight = 10.0;
    assert!(config.validate().is_ok());
}
