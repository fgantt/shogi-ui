//! Tests for evaluation weight balance automation and validation
//!
//! Tests for Task 20.0 - Task 2.0: Weight Balance Automation and Validation

use shogi_engine::evaluation::config::{
    ComponentFlagsForValidation, TaperedEvalConfig, WeightPreset,
};
use shogi_engine::evaluation::statistics::EvaluationTelemetry;

#[test]
fn test_automatic_weight_validation() {
    // Test that validation is called during weight updates when auto_validate_weights is enabled
    let mut config = TaperedEvalConfig::default();
    config.auto_validate_weights = true;

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: true,
        tactical_patterns: true,
        positional_patterns: true,
        castle_patterns: true,
    };

    // Valid weight update should succeed
    assert!(config
        .update_weight("material", 1.5, Some(&components))
        .is_ok());

    // Invalid cumulative weight should fail
    // Set weights too high to exceed max cumulative weight
    config.weights.material_weight = 20.0;
    assert!(config
        .update_weight("position", 1.0, Some(&components))
        .is_err());
}

#[test]
fn test_weight_range_warnings() {
    // Test that warnings are returned for out-of-range weights
    let mut config = TaperedEvalConfig::default();

    // Set a weight outside recommended range
    config.weights.material_weight = 2.0; // Above recommended max of 1.2
    config.weights.mobility_weight = 0.2; // Below recommended min of 0.4

    let warnings = config.check_weight_ranges();

    // Should have warnings for material and mobility
    assert!(warnings.len() >= 2);
    assert!(warnings.iter().any(|(name, _, _, _)| *name == "material"));
    assert!(warnings.iter().any(|(name, _, _, _)| *name == "mobility"));
}

#[test]
fn test_weight_normalization() {
    // Test that normalization maintains ratios while fixing cumulative sum
    let mut config = TaperedEvalConfig::default();

    // Set weights that sum to too much (20.0, above max of 15.0)
    config.weights.material_weight = 4.0;
    config.weights.position_weight = 4.0;
    config.weights.king_safety_weight = 4.0;
    config.weights.tactical_weight = 4.0;
    config.weights.positional_weight = 4.0;

    // Store original ratios
    let material = config.weights.material_weight;
    let position = config.weights.position_weight;
    let original_ratio = material / position;

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: false, // Disabled to simplify test
        tactical_patterns: true,
        positional_patterns: true,
        castle_patterns: false,
    };

    // Normalize weights
    config.weights.normalize_weights(&components);

    // Verify ratios are maintained
    let new_ratio = config.weights.material_weight / config.weights.position_weight;
    assert!(
        (original_ratio - new_ratio).abs() < 0.001,
        "Ratios should be maintained: original={}, new={}",
        original_ratio,
        new_ratio
    );

    // Verify sum is now in range
    let sum = config.weights.material_weight
        + config.weights.position_weight
        + config.weights.tactical_weight
        + config.weights.positional_weight;
    assert!(
        sum >= 5.0 && sum <= 15.0,
        "Sum should be in range [5.0, 15.0]: sum={}",
        sum
    );
}

#[test]
fn test_weight_presets() {
    // Test that all presets set weights correctly
    let mut config = TaperedEvalConfig::default();

    // Test Balanced preset (default)
    config.balanced_preset();
    assert_eq!(config.weights.material_weight, 1.0);
    assert_eq!(config.weights.tactical_weight, 1.0);
    assert_eq!(config.weights.positional_weight, 1.0);

    // Test Aggressive preset
    config.aggressive_preset();
    assert_eq!(config.weights.tactical_weight, 1.5); // Higher than default
    assert!(config.weights.mobility_weight > 0.6); // Higher than default

    // Test Positional preset
    config.positional_preset();
    assert_eq!(config.weights.positional_weight, 1.5); // Higher than default
    assert!(config.weights.pawn_structure_weight > 0.8); // Higher than default

    // Test Defensive preset
    config.defensive_preset();
    assert_eq!(config.weights.king_safety_weight, 1.5); // Higher than default
    assert!(config.weights.castle_weight > 1.0); // Higher than default
}

#[test]
fn test_weight_preset_enum() {
    // Test that WeightPreset enum works correctly
    let mut weights = shogi_engine::evaluation::config::EvaluationWeights::default();

    weights.apply_preset(WeightPreset::Aggressive);
    assert_eq!(weights.tactical_weight, 1.5);

    weights.apply_preset(WeightPreset::Positional);
    assert_eq!(weights.positional_weight, 1.5);

    weights.apply_preset(WeightPreset::Defensive);
    assert_eq!(weights.king_safety_weight, 1.5);

    weights.apply_preset(WeightPreset::Balanced);
    assert_eq!(weights.material_weight, 1.0);
    assert_eq!(weights.tactical_weight, 1.0);
}

#[test]
fn test_telemetry_driven_recommendations() {
    // Test that recommendations are generated from telemetry data
    let config = TaperedEvalConfig::default();

    // Create mock telemetry with imbalanced contributions
    let mut telemetry = EvaluationTelemetry::default();
    telemetry
        .weight_contributions
        .insert("material".to_string(), 0.30); // Too high (target: 0.15)
    telemetry
        .weight_contributions
        .insert("tactical_patterns".to_string(), 0.05); // Too low (target: 0.15)

    let recommendations = config.analyze_telemetry_for_recommendations(&telemetry, None);

    // Should have recommendations for material and tactical_patterns
    assert!(recommendations.len() >= 2);

    // Material should be recommended to decrease
    let material_rec = recommendations
        .iter()
        .find(|(name, _, _, _)| name == "material");
    assert!(material_rec.is_some());
    if let Some((_, current, target, change)) = material_rec {
        assert!(
            *current > *target,
            "Material contribution should be above target"
        );
        assert!(*change < 0.0, "Material weight should be decreased");
    }

    // Tactical should be recommended to increase
    let tactical_rec = recommendations
        .iter()
        .find(|(name, _, _, _)| name == "tactical_patterns");
    assert!(tactical_rec.is_some());
    if let Some((_, current, target, change)) = tactical_rec {
        assert!(
            *current < *target,
            "Tactical contribution should be below target"
        );
        assert!(*change > 0.0, "Tactical weight should be increased");
    }
}

#[test]
fn test_auto_balance_weights() {
    // Test that auto_balance_weights adjusts weights based on telemetry
    let mut config = TaperedEvalConfig::default();

    // Create mock telemetry with imbalanced contributions
    let mut telemetry = EvaluationTelemetry::default();
    telemetry
        .weight_contributions
        .insert("material".to_string(), 0.30); // Too high
    telemetry
        .weight_contributions
        .insert("tactical_patterns".to_string(), 0.05); // Too low

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: true,
        tactical_patterns: true,
        positional_patterns: true,
        castle_patterns: true,
    };

    let initial_material = config.weights.material_weight;
    let initial_tactical = config.weights.tactical_weight;

    // Auto balance with learning rate 0.1
    let adjusted = config.auto_balance_weights(&telemetry, &components, None, 0.1);

    // Should have adjusted at least 2 weights
    assert!(adjusted >= 2);

    // Material weight should be decreased
    assert!(config.weights.material_weight < initial_material);

    // Tactical weight should be increased
    assert!(config.weights.tactical_weight > initial_tactical);
}

#[test]
fn test_auto_normalize_weights() {
    // Test that auto_normalize_weights normalizes when enabled
    let mut config = TaperedEvalConfig::default();
    config.auto_normalize_weights = true;

    // Set weights that sum to too much
    config.weights.material_weight = 10.0;
    config.weights.position_weight = 10.0;

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: false,
        tactical_patterns: false,
        positional_patterns: false,
        castle_patterns: false,
    };

    // Update a weight - should trigger normalization
    assert!(config
        .update_weight("material", 10.0, Some(&components))
        .is_ok());

    // Verify weights were normalized
    let sum = config.weights.material_weight + config.weights.position_weight;
    assert!(
        sum >= 5.0 && sum <= 15.0,
        "Weights should be normalized to range [5.0, 15.0]: sum={}",
        sum
    );
}

#[test]
fn test_auto_validate_weights_enabled() {
    // Test that validation happens when auto_validate_weights is enabled
    let mut config = TaperedEvalConfig::default();
    config.auto_validate_weights = true;

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: true,
        tactical_patterns: true,
        positional_patterns: true,
        castle_patterns: true,
    };

    // Set weights that will exceed cumulative max when we update
    config.weights.material_weight = 10.0;
    config.weights.position_weight = 10.0;
    config.weights.tactical_weight = 10.0;

    // Updating another weight should fail validation
    assert!(config
        .update_weight("positional", 10.0, Some(&components))
        .is_err());
}

#[test]
fn test_auto_validate_weights_disabled() {
    // Test that validation is skipped when auto_validate_weights is disabled
    let mut config = TaperedEvalConfig::default();
    config.auto_validate_weights = false;

    let components = ComponentFlagsForValidation {
        material: true,
        piece_square_tables: true,
        position_features: true,
        tactical_patterns: true,
        positional_patterns: true,
        castle_patterns: true,
    };

    // Set weights that would exceed cumulative max
    config.weights.material_weight = 10.0;
    config.weights.position_weight = 10.0;
    config.weights.tactical_weight = 10.0;

    // Update should succeed when validation is disabled
    assert!(config
        .update_weight("positional", 10.0, Some(&components))
        .is_ok());
}

#[test]
fn test_recommended_ranges() {
    // Test that recommended ranges match documented values
    let config = TaperedEvalConfig::default();

    // Default weights should be within recommended ranges
    let warnings = config.check_weight_ranges();

    // Default config should have no warnings (all weights are in range)
    assert!(
        warnings.is_empty(),
        "Default config should have no range warnings. Found: {:?}",
        warnings
    );
}

#[test]
fn test_weight_update_without_components() {
    // Test backward compatibility - update_weight without components
    let mut config = TaperedEvalConfig::default();

    // Should work without components (validation skipped)
    assert!(config.update_weight("material", 1.5, None).is_ok());
    assert_eq!(config.weights.material_weight, 1.5);
}
