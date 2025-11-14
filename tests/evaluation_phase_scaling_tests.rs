//! Tests for phase-dependent weight scaling (Task 20.0 - Task 3.0)

use shogi_engine::evaluation::config::{
    EvaluationWeights, PhaseScalingConfig, PhaseScalingCurve, TaperedEvalConfig,
};

/// Test that phase-dependent weights are enabled by default (Task 20.0 - Task 3.13)
#[test]
fn test_phase_scaling_enabled_by_default() {
    let config = TaperedEvalConfig::default();
    assert!(
        config.enable_phase_dependent_weights,
        "Phase-dependent weights should be enabled by default"
    );
}

/// Test expanded phase scaling for development, mobility, and pawn_structure weights (Task 20.0 - Task 3.14)
#[test]
fn test_expanded_phase_scaling() {
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = true;

    // Use Step curve for exact values at phase boundaries
    let mut scaling_config = PhaseScalingConfig::default();
    scaling_config.scaling_curve = PhaseScalingCurve::Step;
    config.phase_scaling_config = Some(scaling_config);

    // Test opening phase (phase >= 192) - use phase 256 for exact values
    let mut weights = EvaluationWeights::default();
    let original_development = weights.development_weight;
    let original_mobility = weights.mobility_weight;
    let original_pawn_structure = weights.pawn_structure_weight;

    config.apply_phase_scaling(&mut weights, 256);

    // In opening: development should be 1.2x, mobility 1.0x, pawn_structure 0.8x
    assert!(
        (weights.development_weight - original_development * 1.2).abs() < 0.01,
        "Development weight should be 1.2x in opening, got {:.2}",
        weights.development_weight / original_development
    );
    assert!(
        (weights.mobility_weight - original_mobility * 1.0).abs() < 0.01,
        "Mobility weight should be 1.0x in opening, got {:.2}",
        weights.mobility_weight / original_mobility
    );
    assert!(
        (weights.pawn_structure_weight - original_pawn_structure * 0.8).abs() < 0.01,
        "Pawn structure weight should be 0.8x in opening, got {:.2}",
        weights.pawn_structure_weight / original_pawn_structure
    );

    // Test middlegame phase (64 <= phase < 192)
    let mut weights = EvaluationWeights::default();
    let original_development = weights.development_weight;
    let original_mobility = weights.mobility_weight;
    let original_pawn_structure = weights.pawn_structure_weight;

    config.apply_phase_scaling(&mut weights, 128);

    // In middlegame: development 1.0x, mobility 1.1x, pawn_structure 1.0x
    assert!(
        (weights.development_weight - original_development * 1.0).abs() < 0.01,
        "Development weight should be 1.0x in middlegame, got {:.2}",
        weights.development_weight / original_development
    );
    assert!(
        (weights.mobility_weight - original_mobility * 1.1).abs() < 0.01,
        "Mobility weight should be 1.1x in middlegame, got {:.2}",
        weights.mobility_weight / original_mobility
    );
    assert!(
        (weights.pawn_structure_weight - original_pawn_structure * 1.0).abs() < 0.01,
        "Pawn structure weight should be 1.0x in middlegame, got {:.2}",
        weights.pawn_structure_weight / original_pawn_structure
    );

    // Test endgame phase (phase < 64)
    let mut weights = EvaluationWeights::default();
    let original_development = weights.development_weight;
    let original_mobility = weights.mobility_weight;
    let original_pawn_structure = weights.pawn_structure_weight;

    config.apply_phase_scaling(&mut weights, 32);

    // In endgame: development 0.6x, mobility 0.7x, pawn_structure 1.2x
    assert!(
        (weights.development_weight - original_development * 0.6).abs() < 0.01,
        "Development weight should be 0.6x in endgame, got {:.2}",
        weights.development_weight / original_development
    );
    assert!(
        (weights.mobility_weight - original_mobility * 0.7).abs() < 0.01,
        "Mobility weight should be 0.7x in endgame, got {:.2}",
        weights.mobility_weight / original_mobility
    );
    assert!(
        (weights.pawn_structure_weight - original_pawn_structure * 1.2).abs() < 0.01,
        "Pawn structure weight should be 1.2x in endgame, got {:.2}",
        weights.pawn_structure_weight / original_pawn_structure
    );
}

/// Test scaling curves (Linear, Sigmoid, Step) (Task 20.0 - Task 3.15)
#[test]
fn test_scaling_curves() {
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = true;

    // Test Step curve - should have discrete jumps at boundaries
    let mut scaling_config = PhaseScalingConfig::default();
    scaling_config.scaling_curve = PhaseScalingCurve::Step;
    scaling_config.development = (2.0, 1.0, 0.5); // Distinct values for testing
    config.phase_scaling_config = Some(scaling_config.clone());

    let mut weights = EvaluationWeights::default();
    let original_development = weights.development_weight;

    // Exactly at opening boundary (phase 192)
    config.apply_phase_scaling(&mut weights, 192);
    assert!(
        (weights.development_weight - original_development * 2.0).abs() < 0.01,
        "Step curve: development should be 2.0x at phase 192"
    );

    // Exactly at middlegame (phase 128)
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 128);
    assert!(
        (weights.development_weight - original_development * 1.0).abs() < 0.01,
        "Step curve: development should be 1.0x at phase 128"
    );

    // Exactly at endgame (phase 32)
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 32);
    assert!(
        (weights.development_weight - original_development * 0.5).abs() < 0.01,
        "Step curve: development should be 0.5x at phase 32"
    );

    // Test Linear curve - should have smooth interpolation
    scaling_config.scaling_curve = PhaseScalingCurve::Linear;
    config.phase_scaling_config = Some(scaling_config.clone());

    // At midpoint between opening and middlegame (phase 224)
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 224);
    // Should interpolate between 2.0 (opening) and 1.0 (middlegame)
    // Phase 224 is normalized to 0.875, which is in the opening->middlegame transition
    // t = (0.875 - 0.75) / 0.25 = 0.5
    // So scale = 2.0 * (1 - 0.5) + 1.0 * 0.5 = 1.5
    let expected_scale = 1.5;
    assert!(
        (weights.development_weight - original_development * expected_scale).abs() < 0.05,
        "Linear curve: development should interpolate at phase 224, got {:.2}",
        weights.development_weight / original_development
    );

    // Test Sigmoid curve - should have smooth S-curve transitions
    scaling_config.scaling_curve = PhaseScalingCurve::Sigmoid;
    config.phase_scaling_config = Some(scaling_config);

    // At midpoint between opening and middlegame (phase 224)
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 224);
    // Sigmoid should produce a different (smoother) interpolation than linear
    // The exact value depends on the sigmoid function, but should be between
    // opening and middlegame values
    let scale = weights.development_weight / original_development;
    assert!(
        scale > 1.0 && scale < 2.0,
        "Sigmoid curve: development should be between 1.0x and 2.0x at phase 224, got {:.2}",
        scale
    );
}

/// Test phase scaling impact on evaluation (Task 20.0 - Task 3.16)
#[test]
fn test_phase_scaling_impact() {
    let mut config = TaperedEvalConfig::default();

    // Get weights with scaling enabled
    config.enable_phase_dependent_weights = true;
    let mut weights_scaled = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights_scaled, 200); // Opening phase

    // Get weights with scaling disabled
    config.enable_phase_dependent_weights = false;
    let mut weights_unscaled = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights_unscaled, 200);

    // Weights should differ when scaling is enabled vs disabled
    assert!(
        (weights_scaled.development_weight - weights_unscaled.development_weight).abs() > 0.01,
        "Development weight should differ between scaled and unscaled in opening"
    );
    assert!(
        (weights_scaled.mobility_weight - weights_unscaled.mobility_weight).abs() < 0.01,
        "Mobility weight should be same (1.0x) in opening"
    );
    assert!(
        (weights_scaled.pawn_structure_weight - weights_unscaled.pawn_structure_weight).abs()
            > 0.01,
        "Pawn structure weight should differ between scaled and unscaled in opening"
    );

    // Test middlegame
    config.enable_phase_dependent_weights = true;
    let mut weights_scaled_mg = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights_scaled_mg, 128);

    assert!(
        (weights_scaled_mg.mobility_weight - weights_unscaled.mobility_weight).abs() > 0.01,
        "Mobility weight should differ between scaled and unscaled in middlegame"
    );

    // Test endgame
    let mut weights_scaled_eg = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights_scaled_eg, 32);

    assert!(
        (weights_scaled_eg.development_weight - weights_unscaled.development_weight).abs() > 0.01,
        "Development weight should differ between scaled and unscaled in endgame"
    );
    assert!(
        (weights_scaled_eg.pawn_structure_weight - weights_unscaled.pawn_structure_weight).abs()
            > 0.01,
        "Pawn structure weight should differ between scaled and unscaled in endgame"
    );
}

/// Test that custom phase scaling config works
#[test]
fn test_custom_phase_scaling_config() {
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = true;

    // Create custom scaling config
    let mut custom_config = PhaseScalingConfig::default();
    custom_config.development = (3.0, 2.0, 1.0); // Custom values
    custom_config.scaling_curve = PhaseScalingCurve::Step;
    config.phase_scaling_config = Some(custom_config);

    let mut weights = EvaluationWeights::default();
    let original_development = weights.development_weight;

    config.apply_phase_scaling(&mut weights, 200); // Opening

    assert!(
        (weights.development_weight - original_development * 3.0).abs() < 0.01,
        "Custom config: development should be 3.0x in opening"
    );
}

/// Test that tactical and positional weights still scale correctly
#[test]
fn test_tactical_positional_scaling() {
    let mut config = TaperedEvalConfig::default();
    config.enable_phase_dependent_weights = true;

    // Use Step curve for exact values at phase boundaries
    let mut scaling_config = PhaseScalingConfig::default();
    scaling_config.scaling_curve = PhaseScalingCurve::Step;
    config.phase_scaling_config = Some(scaling_config);

    let mut weights = EvaluationWeights::default();
    let original_tactical = weights.tactical_weight;
    let original_positional = weights.positional_weight;

    // Opening phase: tactical 1.0x, positional 1.0x - use phase 256 for exact values
    config.apply_phase_scaling(&mut weights, 256);
    assert!(
        (weights.tactical_weight - original_tactical * 1.0).abs() < 0.01,
        "Tactical weight should be 1.0x in opening"
    );
    assert!(
        (weights.positional_weight - original_positional * 1.0).abs() < 0.01,
        "Positional weight should be 1.0x in opening"
    );

    // Middlegame phase: tactical 1.2x, positional 0.9x
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 128);
    assert!(
        (weights.tactical_weight - original_tactical * 1.2).abs() < 0.01,
        "Tactical weight should be 1.2x in middlegame"
    );
    assert!(
        (weights.positional_weight - original_positional * 0.9).abs() < 0.01,
        "Positional weight should be 0.9x in middlegame"
    );

    // Endgame phase: tactical 0.8x, positional 1.2x
    let mut weights = EvaluationWeights::default();
    config.apply_phase_scaling(&mut weights, 32);
    assert!(
        (weights.tactical_weight - original_tactical * 0.8).abs() < 0.01,
        "Tactical weight should be 0.8x in endgame"
    );
    assert!(
        (weights.positional_weight - original_positional * 1.2).abs() < 0.01,
        "Positional weight should be 1.2x in endgame"
    );
}
