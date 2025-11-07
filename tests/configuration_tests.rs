#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::*;
use shogi_engine::types::*;

/// Tests for tapered evaluation configuration options
/// Tests the ability to enable/disable tapered evaluation and configure performance options

#[test]
fn test_default_configuration() {
    let evaluator = PositionEvaluator::new();
    let config = evaluator.get_config();

    // Default configuration should have tapered evaluation enabled
    assert!(
        config.enabled,
        "Default configuration should have tapered evaluation enabled"
    );
    assert!(
        config.cache_game_phase,
        "Default configuration should cache game phase"
    );
    assert!(
        !config.use_simd,
        "Default configuration should not use SIMD"
    );
    assert_eq!(
        config.memory_pool_size, 1000,
        "Default memory pool size should be 1000"
    );
    assert!(
        !config.enable_performance_monitoring,
        "Default should not enable performance monitoring"
    );
}

#[test]
fn test_disabled_configuration() {
    let config = TaperedEvaluationConfig::disabled();
    let evaluator = PositionEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // With disabled configuration, evaluation should still work but use simple evaluation
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(score.abs() < 10000, "Score should be reasonable: {}", score);

    // Configuration should be disabled
    let config = evaluator.get_config();
    assert!(!config.enabled, "Configuration should be disabled");
}

#[test]
fn test_performance_optimized_configuration() {
    let config = TaperedEvaluationConfig::performance_optimized();
    let evaluator = PositionEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Performance optimized configuration should work
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(score.abs() < 10000, "Score should be reasonable: {}", score);

    // Configuration should be performance optimized
    let config = evaluator.get_config();
    assert!(config.enabled, "Configuration should be enabled");
    assert!(
        config.cache_game_phase,
        "Configuration should cache game phase"
    );
    assert_eq!(
        config.memory_pool_size, 2000,
        "Memory pool size should be 2000"
    );
    assert!(
        config.enable_performance_monitoring,
        "Performance monitoring should be enabled"
    );
}

#[test]
fn test_memory_optimized_configuration() {
    let config = TaperedEvaluationConfig::memory_optimized();
    let evaluator = PositionEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Memory optimized configuration should work
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(score.abs() < 10000, "Score should be reasonable: {}", score);

    // Configuration should be memory optimized
    let config = evaluator.get_config();
    assert!(config.enabled, "Configuration should be enabled");
    assert!(
        !config.cache_game_phase,
        "Configuration should not cache game phase"
    );
    assert_eq!(
        config.memory_pool_size, 100,
        "Memory pool size should be 100"
    );
    assert!(
        !config.enable_performance_monitoring,
        "Performance monitoring should be disabled"
    );
}

#[test]
fn test_configuration_update() {
    let mut evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with default configuration
    let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Update configuration to disabled
    let disabled_config = TaperedEvaluationConfig::disabled();
    evaluator.set_config(disabled_config);

    // Test with disabled configuration
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Both scores should be reasonable
    assert!(
        score1.abs() < 10000,
        "Score1 should be reasonable: {}",
        score1
    );
    assert!(
        score2.abs() < 10000,
        "Score2 should be reasonable: {}",
        score2
    );

    // Configuration should be updated
    let config = evaluator.get_config();
    assert!(
        !config.enabled,
        "Configuration should be disabled after update"
    );
}

#[test]
fn test_custom_configuration() {
    let custom_config = TaperedEvaluationConfig {
        enabled: true,
        cache_game_phase: false,
        use_simd: false,
        memory_pool_size: 500,
        enable_performance_monitoring: true,
    };

    let evaluator = PositionEvaluator::with_config(custom_config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Custom configuration should work
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(score.abs() < 10000, "Score should be reasonable: {}", score);

    // Configuration should match custom values
    let config = evaluator.get_config();
    assert!(config.enabled, "Configuration should be enabled");
    assert!(
        !config.cache_game_phase,
        "Configuration should not cache game phase"
    );
    assert_eq!(
        config.memory_pool_size, 500,
        "Memory pool size should be 500"
    );
    assert!(
        config.enable_performance_monitoring,
        "Performance monitoring should be enabled"
    );
}

#[test]
fn test_configuration_consistency() {
    let config = TaperedEvaluationConfig::new();
    let evaluator1 = PositionEvaluator::with_config(config.clone());
    let evaluator2 = PositionEvaluator::with_config(config);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Both evaluators should produce consistent results
    let score1 = evaluator1.evaluate(&board, Player::Black, &captured_pieces);
    let score2 = evaluator2.evaluate(&board, Player::Black, &captured_pieces);

    assert_eq!(
        score1, score2,
        "Scores should be consistent: {} vs {}",
        score1, score2
    );
}

#[test]
fn test_configuration_performance_impact() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with enabled configuration
    let enabled_config = TaperedEvaluationConfig::performance_optimized();
    let enabled_evaluator = PositionEvaluator::with_config(enabled_config);

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = enabled_evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }
    let enabled_duration = start.elapsed();

    // Test with disabled configuration
    let disabled_config = TaperedEvaluationConfig::disabled();
    let disabled_evaluator = PositionEvaluator::with_config(disabled_config);

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = disabled_evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }
    let disabled_duration = start.elapsed();

    // Both should complete in reasonable time
    assert!(
        enabled_duration.as_millis() < 1000,
        "Enabled evaluation should be fast: {}ms",
        enabled_duration.as_millis()
    );
    assert!(
        disabled_duration.as_millis() < 1000,
        "Disabled evaluation should be fast: {}ms",
        disabled_duration.as_millis()
    );
}

#[test]
fn test_configuration_validation() {
    // Test that all configuration presets work
    let presets = [
        TaperedEvaluationConfig::new(),
        TaperedEvaluationConfig::disabled(),
        TaperedEvaluationConfig::performance_optimized(),
        TaperedEvaluationConfig::memory_optimized(),
    ];

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for (i, config) in presets.iter().enumerate() {
        let evaluator = PositionEvaluator::with_config(config.clone());
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        // All configurations should produce reasonable scores
        assert!(
            score.abs() < 10000,
            "Configuration {} should produce reasonable score: {}",
            i,
            score
        );
    }
}
