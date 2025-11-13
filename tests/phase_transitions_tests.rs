//! Tests for Phase Transitions and Documentation (Task 17.0 - Task 6.0)

use shogi_engine::evaluation::config::PhaseBoundaryConfig;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{BitboardBoard, CapturedPieces, Player};

#[test]
fn test_gradual_phase_out_endgame() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.endgame_patterns = true;
    config.enable_gradual_phase_transitions = true;
    
    // Set custom fade boundaries for testing
    config.phase_boundaries.endgame_fade_start = 80;
    config.phase_boundaries.endgame_fade_end = 64;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();
    
    // Test fade factor calculation
    let fade_factor_64 = evaluator.config.phase_boundaries.calculate_endgame_fade_factor(64);
    assert_eq!(fade_factor_64, 1.0, "At fade_end, fade factor should be 1.0");
    
    let fade_factor_80 = evaluator.config.phase_boundaries.calculate_endgame_fade_factor(80);
    assert_eq!(fade_factor_80, 0.0, "At fade_start, fade factor should be 0.0");
    
    let fade_factor_72 = evaluator.config.phase_boundaries.calculate_endgame_fade_factor(72);
    assert!(fade_factor_72 > 0.0 && fade_factor_72 < 1.0, 
            "Between fade_end and fade_start, fade factor should be between 0.0 and 1.0");
    
    // Verify fade is linear
    let fade_factor_68 = evaluator.config.phase_boundaries.calculate_endgame_fade_factor(68);
    assert!((fade_factor_68 - 0.5).abs() < 0.1, 
            "At midpoint (68), fade factor should be approximately 0.5");
}

#[test]
fn test_gradual_phase_out_opening() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.opening_principles = true;
    config.enable_gradual_phase_transitions = true;
    
    // Set custom fade boundaries for testing
    config.phase_boundaries.opening_fade_start = 192;
    config.phase_boundaries.opening_fade_end = 160;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    
    // Test fade factor calculation
    let fade_factor_192 = evaluator.config.phase_boundaries.calculate_opening_fade_factor(192);
    assert_eq!(fade_factor_192, 1.0, "At fade_start, fade factor should be 1.0");
    
    let fade_factor_160 = evaluator.config.phase_boundaries.calculate_opening_fade_factor(160);
    assert_eq!(fade_factor_160, 0.0, "At fade_end, fade factor should be 0.0");
    
    let fade_factor_176 = evaluator.config.phase_boundaries.calculate_opening_fade_factor(176);
    assert!(fade_factor_176 > 0.0 && fade_factor_176 < 1.0, 
            "Between fade_start and fade_end, fade factor should be between 0.0 and 1.0");
    
    // Verify fade is linear
    let fade_factor_176_expected = (176.0 - 160.0) / (192.0 - 160.0);
    assert!((fade_factor_176 - fade_factor_176_expected).abs() < 0.01, 
            "Fade factor should be linear");
}

#[test]
fn test_configurable_phase_boundaries() {
    let mut config = IntegratedEvaluationConfig::default();
    
    // Set custom phase boundaries
    config.phase_boundaries.opening_threshold = 200;
    config.phase_boundaries.endgame_threshold = 60;
    config.phase_boundaries.opening_fade_start = 200;
    config.phase_boundaries.opening_fade_end = 170;
    config.phase_boundaries.endgame_fade_start = 70;
    config.phase_boundaries.endgame_fade_end = 60;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    
    // Verify boundaries are set correctly
    assert_eq!(evaluator.config.phase_boundaries.opening_threshold, 200);
    assert_eq!(evaluator.config.phase_boundaries.endgame_threshold, 60);
    assert_eq!(evaluator.config.phase_boundaries.opening_fade_start, 200);
    assert_eq!(evaluator.config.phase_boundaries.opening_fade_end, 170);
    assert_eq!(evaluator.config.phase_boundaries.endgame_fade_start, 70);
    assert_eq!(evaluator.config.phase_boundaries.endgame_fade_end, 60);
}

#[test]
fn test_phase_transition_smoothness() {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.endgame_patterns = true;
    config.components.opening_principles = true;
    config.enable_gradual_phase_transitions = true;
    config.collect_position_feature_stats = true;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();
    
    // Evaluate at different phases to check smoothness
    // Note: We can't easily control phase in a test, but we can verify the fade factors
    // are calculated correctly for smooth transitions
    
    let phases = vec![64, 68, 72, 76, 80];
    let mut previous_fade = 1.0;
    
    for phase in phases {
        let fade = evaluator.config.phase_boundaries.calculate_endgame_fade_factor(phase);
        assert!(fade <= previous_fade, 
                "Fade factor should decrease as phase increases (smooth transition)");
        previous_fade = fade;
    }
    
    // Test opening fade (should decrease as phase decreases)
    let opening_phases = vec![192, 180, 170, 165, 160];
    let mut previous_opening_fade = 1.0;
    
    for phase in opening_phases {
        let fade = evaluator.config.phase_boundaries.calculate_opening_fade_factor(phase);
        assert!(fade <= previous_opening_fade, 
                "Opening fade factor should decrease as phase decreases (smooth transition)");
        previous_opening_fade = fade;
    }
}

#[test]
fn test_phase_boundary_defaults() {
    let config = PhaseBoundaryConfig::default();
    
    // Verify default values match documentation
    assert_eq!(config.opening_threshold, 192);
    assert_eq!(config.endgame_threshold, 64);
    assert_eq!(config.opening_fade_start, 192);
    assert_eq!(config.opening_fade_end, 160);
    assert_eq!(config.endgame_fade_start, 80);
    assert_eq!(config.endgame_fade_end, 64);
}

#[test]
fn test_fade_factor_edge_cases() {
    let config = PhaseBoundaryConfig::default();
    
    // Test edge cases
    let fade_below_endgame = config.calculate_endgame_fade_factor(50);
    assert_eq!(fade_below_endgame, 1.0, "Below fade_end should be 1.0");
    
    let fade_above_endgame = config.calculate_endgame_fade_factor(100);
    assert_eq!(fade_above_endgame, 0.0, "Above fade_start should be 0.0");
    
    let fade_below_opening = config.calculate_opening_fade_factor(150);
    assert_eq!(fade_below_opening, 0.0, "Below fade_end should be 0.0");
    
    let fade_above_opening = config.calculate_opening_fade_factor(200);
    assert_eq!(fade_above_opening, 1.0, "Above fade_start should be 1.0");
}

#[test]
fn test_abrupt_vs_gradual_transitions() {
    // Test that gradual transitions produce different results than abrupt
    let mut config_abrupt = IntegratedEvaluationConfig::default();
    config_abrupt.components.endgame_patterns = true;
    config_abrupt.enable_gradual_phase_transitions = false;
    
    let mut config_gradual = IntegratedEvaluationConfig::default();
    config_gradual.components.endgame_patterns = true;
    config_gradual.enable_gradual_phase_transitions = true;
    
    // Both should work, but gradual should have smoother transitions
    // (We can't easily test this without controlling phase, but we verify both compile and run)
    let evaluator_abrupt = IntegratedEvaluator::with_config(config_abrupt);
    let evaluator_gradual = IntegratedEvaluator::with_config(config_gradual);
    
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();
    
    // Both should evaluate without panicking
    let _score_abrupt = evaluator_abrupt.evaluate(&board, Player::Black, &captured);
    let _score_gradual = evaluator_gradual.evaluate(&board, Player::Black, &captured);
}

