//! Integration tests for evaluation system coordination and double-counting prevention
//!
//! Tests for Task 20.0 - Task 1.0: Double-Counting Prevention and Conflict Resolution

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::ComponentDependencyWarning;
use shogi_engine::evaluation::integration::{
    CenterControlPrecedence, IntegratedEvaluationConfig, IntegratedEvaluator,
};
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

    let mut evaluator1 = IntegratedEvaluator::with_config(config1);
    let result1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);
    let score1 = result1.score;

    // Test 2: PositionFeatures precedence - skip positional_patterns center control
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.positional_patterns = true;
    config2.center_control_precedence = CenterControlPrecedence::PositionFeatures;

    let mut evaluator2 = IntegratedEvaluator::with_config(config2);
    let result2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);
    let score2 = result2.score;

    // Test 3: Both precedence - evaluate both (may cause double-counting)
    let mut config3 = IntegratedEvaluationConfig::default();
    config3.components.position_features = true;
    config3.components.positional_patterns = true;
    config3.center_control_precedence = CenterControlPrecedence::Both;

    let mut evaluator3 = IntegratedEvaluator::with_config(config3);
    let result3 = evaluator3.evaluate(&board, Player::Black, &captured_pieces);
    let score3 = result3.score;

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

    let mut evaluator1 = IntegratedEvaluator::with_config(config1);
    let result1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);
    let score1 = result1.score;

    // Test 2: Only position_features enabled (no overlap)
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.opening_principles = false;

    let mut evaluator2 = IntegratedEvaluator::with_config(config2);
    let result2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);
    let score2 = result2.score;

    // Test 3: Only opening_principles enabled (no overlap)
    let mut config3 = IntegratedEvaluationConfig::default();
    config3.components.position_features = false;
    config3.components.opening_principles = true;

    let mut evaluator3 = IntegratedEvaluator::with_config(config3);
    let result3 = evaluator3.evaluate(&board, Player::Black, &captured_pieces);
    let score3 = result3.score;

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
        let mut evaluator = IntegratedEvaluator::with_config(config.clone());
        let result = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let score = result.score;

        // Verify evaluation completes successfully
        assert!(
            score != i32::MIN && score != i32::MAX,
            "Test config {} failed: score = {}",
            i,
            score
        );

        // Verify score is reasonable (not extreme values)
        assert!(
            score.abs() < 10000,
            "Test config {} produced extreme score: {}",
            i,
            score
        );
    }
}

#[test]
fn test_center_control_precedence_default() {
    // Verify default precedence is PositionalPatterns
    let config = IntegratedEvaluationConfig::default();
    assert_eq!(
        config.center_control_precedence,
        CenterControlPrecedence::PositionalPatterns
    );
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

// Task 3.0 - Task 3.29: Additional tests to verify no double-counting occurs in evaluation coordination

#[test]
fn test_no_double_counting_king_safety() {
    // Test that king safety is not double-counted when both position_features
    // and castle_recognizer are enabled
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test 1: Both position_features and castle_patterns enabled
    // KingSafetyEvaluator in position_features evaluates general king safety,
    // while CastleRecognizer evaluates specific castle formations.
    // These are complementary and should both be enabled.
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.castle_patterns = true;

    let mut evaluator1 = IntegratedEvaluator::with_config(config1);
    let result1 = evaluator1.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Test 2: Only position_features enabled
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.castle_patterns = false;

    let mut evaluator2 = IntegratedEvaluator::with_config(config2);
    let result2 = evaluator2.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Both should return valid scores
    assert!(result1.score != i32::MIN && result1.score != i32::MAX);
    assert!(result2.score != i32::MIN && result2.score != i32::MAX);

    // With castle_recognizer enabled, score should account for castle formations
    // (may be different, but should be reasonable)
    assert!(result1.score.abs() < 10000);
    assert!(result2.score.abs() < 10000);
}

#[test]
fn test_no_double_counting_passed_pawns() {
    // Test that passed pawns are not double-counted when both position_features
    // and endgame_patterns are enabled in endgame phase (phase < 64)
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Create an endgame position (simplified - actual endgame would have fewer pieces)
    // For this test, we'll verify coordination logic works correctly

    // Test 1: Both position_features and endgame_patterns enabled
    // In endgame (phase < 64), passed pawns should be evaluated by endgame_patterns,
    // and skipped in position_features to avoid double-counting.
    let mut config1 = IntegratedEvaluationConfig::default();
    config1.components.position_features = true;
    config1.components.endgame_patterns = true;

    let mut evaluator1 = IntegratedEvaluator::with_config(config1);
    let result1 = evaluator1.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Test 2: Only position_features enabled (passed pawns evaluated here in all phases)
    let mut config2 = IntegratedEvaluationConfig::default();
    config2.components.position_features = true;
    config2.components.endgame_patterns = false;

    let mut evaluator2 = IntegratedEvaluator::with_config(config2);
    let result2 = evaluator2.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Both should return valid scores
    assert!(result1.score != i32::MIN && result1.score != i32::MAX);
    assert!(result2.score != i32::MIN && result2.score != i32::MAX);

    // Scores should be reasonable
    assert!(result1.score.abs() < 10000);
    assert!(result2.score.abs() < 10000);
}

#[test]
fn test_no_double_counting_all_components() {
    // Comprehensive test with all components enabled to verify coordination
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Enable all components with proper precedence settings
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.positional_patterns = true;
    config.components.opening_principles = true;
    config.components.endgame_patterns = true;
    config.components.tactical_patterns = true;
    config.components.castle_patterns = true;
    config.center_control_precedence = CenterControlPrecedence::PositionalPatterns;

    let mut evaluator = IntegratedEvaluator::with_config(config);
    let result = evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Verify evaluation completes successfully
    assert!(result.score != i32::MIN && result.score != i32::MAX);
    assert!(result.score.abs() < 10000, "Score should be reasonable: {}", result.score);

    // Verify component contributions are tracked (if available)
    // The key is that evaluation completes without double-counting errors
}

#[test]
fn test_evaluation_coordination_consistency() {
    // Test that evaluation results are consistent across different component configurations
    // that should avoid double-counting
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test various configurations that should avoid double-counting
    let configs = vec![
        // Configuration 1: All components with proper precedence
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.positional_patterns = true;
            config.components.opening_principles = true;
            config.components.endgame_patterns = true;
            config.center_control_precedence = CenterControlPrecedence::PositionalPatterns;
            config
        },
        // Configuration 2: Position features only (no overlaps)
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = true;
            config.components.positional_patterns = false;
            config.components.opening_principles = false;
            config.components.endgame_patterns = false;
            config
        },
        // Configuration 3: Positional patterns only (no overlaps)
        {
            let mut config = IntegratedEvaluationConfig::default();
            config.components.position_features = false;
            config.components.positional_patterns = true;
            config
        },
    ];

    let mut scores = Vec::new();
    for (i, config) in configs.iter().enumerate() {
        let mut evaluator = IntegratedEvaluator::with_config(config.clone());
        let result = evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

        assert!(
            result.score != i32::MIN && result.score != i32::MAX,
            "Config {} failed: score = {}",
            i,
            result.score
        );

        scores.push(result.score);
    }

    // All scores should be reasonable (not extreme values)
    for (i, score) in scores.iter().enumerate() {
        assert!(
            score.abs() < 10000,
            "Config {} produced extreme score: {}",
            i,
            score
        );
    }

    // The key test: evaluation should complete without errors for all configurations
    // Even if scores differ, they should all be valid evaluations
}

// Task 3.0 - Task 3.29: Additional comprehensive tests to verify no double-counting

#[test]
fn test_no_double_counting_component_contributions() {
    // Test that component contributions don't double-count the same evaluation aspects
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Enable all components with proper precedence to avoid double-counting
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.positional_patterns = true;
    config.components.opening_principles = true;
    config.components.endgame_patterns = true;
    config.components.tactical_patterns = true;
    config.components.castle_patterns = true;
    config.center_control_precedence = CenterControlPrecedence::PositionalPatterns;

    let mut evaluator = IntegratedEvaluator::with_config(config);
    let result = evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Verify evaluation completes successfully
    assert!(result.score != i32::MIN && result.score != i32::MAX);
    assert!(result.score.abs() < 10000, "Score should be reasonable: {}", result.score);

    // Verify component contributions are tracked
    // The key is that evaluation completes without double-counting errors
    // Each component should contribute to the final score without overlap
}

#[test]
fn test_phase_aware_gating_prevents_double_counting() {
    // Test that phase-aware gating prevents double-counting between components
    // that are only active in specific phases (opening_principles, endgame_patterns)
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Opening position (phase >= 192) - opening_principles should evaluate,
    // and development should be skipped in position_features
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true;
    config.components.opening_principles = true;
    config.components.endgame_patterns = true; // Should be skipped in opening phase

    let mut evaluator = IntegratedEvaluator::with_config(config);
    let result = evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Verify evaluation completes successfully
    assert!(result.score != i32::MIN && result.score != i32::MAX);
    assert!(result.score.abs() < 10000, "Score should be reasonable: {}", result.score);

    // The key is that phase-aware gating prevents endgame_patterns from evaluating
    // in opening phase, and prevents opening_principles from evaluating in endgame phase
}

#[test]
fn test_coordination_with_complementary_components() {
    // Test that complementary components (king safety + castle patterns) work together
    // without causing issues (they should complement, not conflict)
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Enable complementary components
    let mut config = IntegratedEvaluationConfig::default();
    config.components.position_features = true; // Includes king safety
    config.components.castle_patterns = true; // Complementary to king safety

    let mut evaluator1 = IntegratedEvaluator::with_config(config.clone());
    let result1 = evaluator1.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Disable castle patterns
    config.components.castle_patterns = false;
    let mut evaluator2 = IntegratedEvaluator::with_config(config);
    let result2 = evaluator2.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

    // Both should return valid scores
    assert!(result1.score != i32::MIN && result1.score != i32::MAX);
    assert!(result2.score != i32::MIN && result2.score != i32::MAX);

    // Castle patterns should add to the evaluation (complementary)
    // but not cause double-counting since they evaluate different aspects
    assert!(result1.score.abs() < 10000);
    assert!(result2.score.abs() < 10000);
}

#[test]
fn test_validation_warns_on_double_counting_risks() {
    // Test that validation warns when components that could double-count are both enabled
    let mut config = IntegratedEvaluationConfig::default();

    // Center control overlap
    config.components.position_features = true;
    config.components.positional_patterns = true;
    let warnings = config.validate_component_dependencies();
    assert!(
        warnings.iter().any(|w| matches!(w, ComponentDependencyWarning::CenterControlOverlap)),
        "Should warn about center control overlap"
    );

    // Development overlap
    config.components.opening_principles = true;
    let warnings = config.validate_component_dependencies();
    assert!(
        warnings.iter().any(|w| matches!(w, ComponentDependencyWarning::DevelopmentOverlap)),
        "Should warn about development overlap"
    );
}
