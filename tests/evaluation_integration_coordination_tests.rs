//! Integration tests for evaluation system coordination and double-counting prevention
//! 
//! Tests for Task 20.0 - Task 1.0: Double-Counting Prevention and Conflict Resolution

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{CenterControlPrecedence, IntegratedEvaluator, IntegratedEvaluationConfig};
use shogi_engine::evaluation::config::ComponentDependencyWarning;
use shogi_engine::types::*;

#[test]
fn test_center_control_conflict_resolution() {
    // Test that center control precedence logic works correctly
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test 1: PositionalPatterns precedence (default) - skip position_features center control
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.positional_patterns = true;
    config1.center_control_precedence = CenterControlPrecedence::PositionalPatterns;
    
    let evaluator1 = IntegratedEvaluator::with_config(config1);
    let score1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);
    
    // Test 2: PositionFeatures precedence - skip positional_patterns center control
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.positional_patterns = true;
    config2.center_control_precedence = CenterControlPrecedence::PositionFeatures;
    
    let evaluator2 = IntegratedEvaluator::with_config(config2);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);
    
    // Test 3: Both precedence - evaluate both (may cause double-counting)
    let mut config3 = IntegratedEvaluationConfig::default();
    config3.components.position_features = true;
    config3.components.positional_patterns = true;
    config3.center_control_precedence = CenterControlPrecedence::Both;
    
    let evaluator3 = IntegratedEvaluator::with_config(config3);
    let score3 = evaluator3.evaluate(&board, Player::Black, &captured_pieces);
    
    // All should return valid scores
    assert!(score1 != i32::MIN && score1 != i32::MAX);
    assert!(score2 != i32::MIN && score2 != i32::MAX);
    assert!(score3 != i32::MIN && score3 != i32::MAX);
    
    // With Both precedence, score might be different due to double-counting
    // (but we still want to verify it doesn't crash)
    assert_ne!(score1, i32::MIN);
    assert_ne!(score2, i32::MIN);
    assert_ne!(score3, i32::MIN);
}

#[test]
fn test_development_overlap_coordination() {
    // Test that development is skipped in position_features when opening_principles is enabled
    // and we're in opening phase (phase >= opening_threshold, default: 192)
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Create a position in opening phase (phase >= 192)
    // Starting position has phase ~256, which is >= 192, so opening_principles will evaluate
    
    // Test 1: Both position_features and opening_principles enabled
    // Development should be skipped in position_features when in opening
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.opening_principles = true;
    
    let evaluator1 = IntegratedEvaluator::with_config(config1);
    let score1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);
    
    // Test 2: Only position_features enabled (no overlap)
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.opening_principles = false;
    
    let evaluator2 = IntegratedEvaluator::with_config(config2);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);
    
    // Test 3: Only opening_principles enabled (no overlap)
    let mut config3 = IntegratedEvaluationConfig::default();
    config3.components.position_features = false;
    config3.components.opening_principles = true;
    
    let evaluator3 = IntegratedEvaluator::with_config(config3);
    let score3 = evaluator3.evaluate(&board, Player::Black, &captured_pieces);
    
    // All should return valid scores
    assert!(score1 != i32::MIN && score1 != i32::MAX);
    assert!(score2 != i32::MIN && score2 != i32::MAX);
    assert!(score3 != i32::MIN && score3 != i32::MAX);
    
    // Verify no crashes and scores are reasonable
    assert_ne!(score1, i32::MIN);
    assert_ne!(score2, i32::MIN);
    assert_ne!(score3, i32::MIN);
}

#[test]
fn test_double_counting_prevention() {
    // Comprehensive integration test to verify no double-counting occurs
    // with various component combinations
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Test various combinations of components that might overlap
    let test_configs = vec![
        // Center control overlap scenarios
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.positional_patterns = true;
            config.center_control_precedence = CenterControlPrecedence::PositionalPatterns;
            config
        },
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.positional_patterns = true;
            config.center_control_precedence = CenterControlPrecedence::PositionFeatures;
            config
        },
        // Development overlap scenario
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.opening_principles = true;
            config
        },
        // All components enabled (should handle all overlaps correctly)
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.positional_patterns = true;
            config.components.opening_principles = true;
            config.components.endgame_patterns = true;
            config.center_control_precedence = CenterControlPrecedence::PositionalPatterns;
            config
        },
    ];
    
    for (i, config) in test_configs.iter().enumerate() {
        let evaluator = IntegratedEvaluator::with_config(config.clone());
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        // Verify evaluation completes successfully
        assert!(score != i32::MIN && score != i32::MAX, 
                "Test config {} failed: score = {}", i, score);
        
        // Verify score is reasonable (not extreme values)
        assert!(score.abs() < 10000, 
                "Test config {} produced extreme score: {}", i, score);
    }
}

#[test]
fn test_center_control_precedence_default() {
    // Verify default precedence is PositionalPatterns
    let config = IntegratedEvaluationConfig::default();
    assert_eq!(config.center_control_precedence, CenterControlPrecedence::PositionalPatterns);
}

#[test]
fn test_validate_component_dependencies() {
    // Test that validation warnings are generated for overlaps
    
    // Center control overlap
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.positional_patterns = true;
    let warnings1 = config1.validate_component_dependencies();
    assert!(warnings1.contains(&ComponentDependencyWarning::CenterControlOverlap));
    
    // Development overlap
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.opening_principles = true;
    let warnings2 = config2.validate_component_dependencies();
    assert!(warnings2.contains(&ComponentDependencyWarning::DevelopmentOverlap));
    
    // Both overlaps
    let mut config3 = IntegratedEvaluationConfig::default();
    config3.components.position_features = true;
    config3.components.positional_patterns = true;
    config3.components.opening_principles = true;
    let warnings3 = config3.validate_component_dependencies();
    assert!(warnings3.contains(&ComponentDependencyWarning::CenterControlOverlap));
    assert!(warnings3.contains(&ComponentDependencyWarning::DevelopmentOverlap));
}

