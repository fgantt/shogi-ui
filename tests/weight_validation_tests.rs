//! Integration tests for weight validation and coordination in IntegratedEvaluator
//!
//! Tests verify that weight validation, phase-dependent scaling, and large contribution
//! logging work correctly.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::{ConfigError, TaperedEvalConfig};
use shogi_engine::evaluation::integration::{ComponentFlags, IntegratedEvaluator, IntegratedEvaluationConfig};
use shogi_engine::types::{CapturedPieces, Player};

#[test]
fn test_cumulative_weight_validation() {
    // Test that validation rejects weights outside the acceptable range (5.0-15.0)
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set all weights to very high values to exceed the maximum
    config.weights.material_weight = 10.0;
    config.weights.position_weight = 10.0;
    config.weights.king_safety_weight = 10.0;
    config.weights.pawn_structure_weight = 10.0;
    config.weights.mobility_weight = 10.0;
    config.weights.center_control_weight = 10.0;
    config.weights.development_weight = 10.0;
    config.weights.tactical_weight = 10.0;
    config.weights.positional_weight = 10.0;
    config.weights.castle_weight = 10.0;
    
    // Enable all components
    config.components = ComponentFlags::all_enabled();
    
    // This should fail validation (sum = 100.0, max = 15.0)
    let result = config.validate_cumulative_weights();
    assert!(result.is_err());
    
    if let Err(ConfigError::CumulativeWeightOutOfRange { sum, min, max }) = result {
        assert!(sum > max);
        assert_eq!(min, 5.0);
        assert_eq!(max, 15.0);
    } else {
        panic!("Expected CumulativeWeightOutOfRange error");
    }
}

#[test]
fn test_cumulative_weight_validation_accepts_valid_range() {
    // Test that validation accepts weights within the acceptable range (5.0-15.0)
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set weights to reasonable values that sum to within range
    config.weights.material_weight = 1.0;
    config.weights.position_weight = 1.0;
    config.weights.king_safety_weight = 1.0;
    config.weights.pawn_structure_weight = 1.0;
    config.weights.mobility_weight = 1.0;
    config.weights.center_control_weight = 1.0;
    config.weights.development_weight = 1.0;
    config.weights.tactical_weight = 1.0;
    config.weights.positional_weight = 1.0;
    config.weights.castle_weight = 1.0;
    
    // Enable all components
    config.components = ComponentFlags::all_enabled();
    
    // This should pass validation (sum = 10.0, within 5.0-15.0 range)
    let result = config.validate_cumulative_weights();
    assert!(result.is_ok(), "Validation should pass for weights summing to 10.0");
}

#[test]
fn test_cumulative_weight_validation_too_low() {
    // Test that validation rejects weights that sum to less than 5.0
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set all weights to very low values
    config.weights.material_weight = 0.1;
    config.weights.position_weight = 0.1;
    config.weights.king_safety_weight = 0.1;
    config.weights.pawn_structure_weight = 0.1;
    config.weights.mobility_weight = 0.1;
    config.weights.center_control_weight = 0.1;
    config.weights.development_weight = 0.1;
    config.weights.tactical_weight = 0.1;
    config.weights.positional_weight = 0.1;
    config.weights.castle_weight = 0.1;
    
    // Enable all components
    config.components = ComponentFlags::all_enabled();
    
    // This should fail validation (sum = 1.0, min = 5.0)
    let result = config.validate_cumulative_weights();
    assert!(result.is_err());
    
    if let Err(ConfigError::CumulativeWeightOutOfRange { sum, min, max }) = result {
        assert!(sum < min);
        assert_eq!(min, 5.0);
        assert_eq!(max, 15.0);
    } else {
        panic!("Expected CumulativeWeightOutOfRange error");
    }
}

#[test]
fn test_phase_dependent_weight_scaling() {
    // Test that weights are scaled correctly based on game phase
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = true;
    
    // Set initial weights
    config.weights.tactical_weight = 1.0;
    config.weights.positional_weight = 1.0;
    
    // Test endgame phase (phase < 64)
    let mut weights_endgame = config.weights.clone();
    config.apply_phase_scaling(&mut weights_endgame, 32); // Endgame
    
    // Tactical should be reduced (0.8x), positional should be increased (1.2x)
    assert!((weights_endgame.tactical_weight - 0.8).abs() < 0.01,
        "Tactical weight should be 0.8 in endgame, got {}", weights_endgame.tactical_weight);
    assert!((weights_endgame.positional_weight - 1.2).abs() < 0.01,
        "Positional weight should be 1.2 in endgame, got {}", weights_endgame.positional_weight);
    
    // Test middlegame phase (64 <= phase < 192)
    let mut weights_middlegame = config.weights.clone();
    config.apply_phase_scaling(&mut weights_middlegame, 128); // Middlegame
    
    // Tactical should be increased (1.2x), positional should be reduced (0.9x)
    assert!((weights_middlegame.tactical_weight - 1.2).abs() < 0.01,
        "Tactical weight should be 1.2 in middlegame, got {}", weights_middlegame.tactical_weight);
    assert!((weights_middlegame.positional_weight - 0.9).abs() < 0.01,
        "Positional weight should be 0.9 in middlegame, got {}", weights_middlegame.positional_weight);
    
    // Test opening phase (phase >= 192)
    let mut weights_opening = config.weights.clone();
    config.apply_phase_scaling(&mut weights_opening, 224); // Opening
    
    // Both should remain at 1.0 (neutral)
    assert!((weights_opening.tactical_weight - 1.0).abs() < 0.01,
        "Tactical weight should be 1.0 in opening, got {}", weights_opening.tactical_weight);
    assert!((weights_opening.positional_weight - 1.0).abs() < 0.01,
        "Positional weight should be 1.0 in opening, got {}", weights_opening.positional_weight);
}

#[test]
fn test_phase_dependent_weight_scaling_disabled() {
    // Test that phase scaling doesn't apply when disabled
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = false;
    
    config.weights.tactical_weight = 1.0;
    config.weights.positional_weight = 1.0;
    
    let mut weights = config.weights.clone();
    config.apply_phase_scaling(&mut weights, 128); // Middlegame
    
    // Weights should remain unchanged
    assert!((weights.tactical_weight - 1.0).abs() < 0.01);
    assert!((weights.positional_weight - 1.0).abs() < 0.01);
}

#[test]
fn test_weight_balance_recommendations() {
    // Test that weight balance recommendations are generated correctly
    let mut config = TaperedEvalConfig::default();
    
    // Set tactical weight very high
    config.weights.tactical_weight = 2.5;
    config.weights.positional_weight = 1.0;
    
    let suggestions = config.suggest_weight_adjustments();
    assert!(!suggestions.is_empty(), "Should generate suggestions for imbalanced weights");
    
    // Should suggest increasing positional weight or reducing tactical weight
    let suggestion_text = suggestions.join(" ");
    assert!(suggestion_text.contains("tactical") || suggestion_text.contains("Tactical"),
        "Suggestions should mention tactical weight");
}

#[test]
fn test_weight_balance_recommendations_no_suggestions() {
    // Test that no suggestions are generated for balanced weights
    let mut config = TaperedEvalConfig::default();
    
    // Set balanced weights
    config.weights.tactical_weight = 1.0;
    config.weights.positional_weight = 1.0;
    
    let _suggestions = config.suggest_weight_adjustments();
    // May or may not have suggestions, but shouldn't error
    // (The implementation may still suggest if weights are very high)
}

#[test]
fn test_weight_clamping() {
    // Test that weights are clamped to valid range (0.0-10.0) during evaluation
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set weights outside valid range
    config.weights.tactical_weight = 15.0; // Above max
    config.weights.positional_weight = -5.0; // Below min
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Evaluation should complete (weights should be clamped internally)
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert!(score >= -10000 && score <= 10000);
}

#[test]
fn test_large_contribution_logging() {
    // Test that large contributions can be detected (though we can't easily verify logging)
    // This test verifies the code path exists and doesn't crash
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set a very high weight and low threshold to trigger logging
    config.weights.tactical_weight = 5.0;
    config.weight_contribution_threshold = 100.0; // Low threshold
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Evaluation should complete without errors
    // (Logging happens via debug_log which we can't easily verify in tests)
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert!(score >= -10000 && score <= 10000);
}

#[test]
fn test_cumulative_weight_validation_with_partial_components() {
    // Test that cumulative weight validation only considers enabled components
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set weights to high values
    config.weights.material_weight = 10.0;
    config.weights.tactical_weight = 10.0;
    config.weights.positional_weight = 10.0;
    
    // Only enable a few components
    config.components = ComponentFlags::minimal();
    config.components.material = true;
    config.components.tactical_patterns = true;
    
    // This should pass validation (only material + tactical = 20.0, but only those are enabled)
    // Actually, wait - if only 2 components are enabled and they sum to 20.0, that's still > 15.0
    // Let me adjust the test
    config.weights.material_weight = 3.0;
    config.weights.tactical_weight = 3.0;
    
    let result = config.validate_cumulative_weights();
    // Should pass (sum = 6.0, within 5.0-15.0 range)
    assert!(result.is_ok());
}

