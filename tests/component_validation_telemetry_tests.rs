//! Tests for Component Validation and Telemetry (Task 17.0 - Task 5.0)

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::ComponentDependencyWarning;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

#[test]
fn test_component_dependency_validation() {
    // Test center control overlap warning
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.positional_patterns = true;
    
    let warnings = config.validate_component_dependencies();
    assert!(warnings.contains(&ComponentDependencyWarning::CenterControlOverlap));
    
    // Test no warnings when only one is enabled
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.positional_patterns = false;
    
    let warnings2 = config2.validate_component_dependencies();
    assert!(!warnings2.contains(&ComponentDependencyWarning::CenterControlOverlap));
}

#[test]
fn test_validate_config_with_warnings() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.positional_patterns = true;
    
    // Should not error, but return warnings
    match config.validate() {
        Ok(warnings) => {
            assert!(warnings.contains(&ComponentDependencyWarning::CenterControlOverlap));
        }
        Err(e) => panic!("Validation should not error, but got: {:?}", e),
    }
}

#[test]
fn test_castle_stats_in_telemetry() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate a position
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // Get telemetry
    let telemetry = evaluator.telemetry_snapshot();
    assert!(telemetry.is_some());
    
    let telemetry = telemetry.unwrap();
    // Castle stats should be present when castle_patterns is enabled
    assert!(telemetry.castle_patterns.is_some());
}

#[test]
fn test_weight_contributions_tracking() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.components.tactical_patterns = true;
    config.components.positional_patterns = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate a position
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // Get telemetry
    let telemetry = evaluator.telemetry_snapshot();
    assert!(telemetry.is_some());
    
    let telemetry = telemetry.unwrap();
    // Weight contributions should be tracked
    assert!(!telemetry.weight_contributions.is_empty());
    
    // Check that enabled components have contributions
    if telemetry.weight_contributions.contains_key("material") {
        let contrib = telemetry.weight_contributions.get("material").unwrap();
        assert!(*contrib >= 0.0 && *contrib <= 1.0); // Should be a percentage
    }
}

#[test]
fn test_large_contribution_logging() {
    // This test verifies the code path exists - actual logging would require
    // a position that produces large contributions, which is hard to set up
    let mut config = IntegratedEvaluationConfig::default();
    config.components.tactical_patterns = true;
    config.large_contribution_threshold = 0.5; // 50% threshold for testing
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate a position - logging code path should execute
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // If we got here without panicking, the logging code path exists
    // (actual logging verification would require capturing debug output)
}

#[test]
fn test_all_pattern_stats_aggregated() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.tactical_patterns = true;
    config.components.positional_patterns = true;
    config.components.castle_patterns = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate a position
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // Get telemetry
    let telemetry = evaluator.telemetry_snapshot();
    assert!(telemetry.is_some());
    
    let telemetry = telemetry.unwrap();
    
    // Verify all pattern stats are present
    assert!(telemetry.tactical.is_some(), "Tactical stats should be present");
    assert!(telemetry.positional.is_some(), "Positional stats should be present");
    assert!(telemetry.castle_patterns.is_some(), "Castle stats should be present");
}

#[test]
fn test_component_zero_score_validation() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.enable_component_validation = true; // Enable validation mode
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate - validation code path should execute
    // (zero score warnings would be logged if material produced zero, which is unlikely)
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // If we got here, the validation code path exists
}

#[test]
fn test_endgame_patterns_phase_warning() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.endgame_patterns = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate in opening phase (phase >= 64)
    // Warning should be logged (we can't easily capture it, but code path should execute)
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // If we got here, the phase warning code path exists
}

#[test]
fn test_weight_contributions_percentage_calculation() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.components.piece_square_tables = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluate a position
    let _score = evaluator.evaluate(&board, Player::Black, &captured);
    
    // Get telemetry
    let telemetry = evaluator.telemetry_snapshot();
    assert!(telemetry.is_some());
    
    let telemetry = telemetry.unwrap();
    
    // Verify contributions are percentages (0.0-1.0)
    for (component, contrib) in &telemetry.weight_contributions {
        assert!(
            *contrib >= 0.0 && *contrib <= 1.0,
            "Component {} contribution {} should be between 0.0 and 1.0",
            component,
            contrib
        );
    }
}

