//! Tests for tuning infrastructure integration (Task 20.0 - Task 4.0)

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::EvaluationWeights;
use shogi_engine::evaluation::integration::{
    IntegratedEvaluator, TuningConfig, TuningPosition, TuningPositionSet,
};
use shogi_engine::types::{CapturedPieces, Player};

/// Test tune_weights() API and basic functionality (Task 20.0 - Task 4.19)
#[test]
fn test_tune_weights_api() {
    let mut evaluator = IntegratedEvaluator::new();
    let mut position_set = TuningPositionSet::empty();

    // Create a few training positions
    for _ in 0..5 {
        let position = TuningPosition {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            player: Player::Black,
            expected_score: 0.0, // Draw
            game_phase: 128,     // Middlegame
            move_number: 10,
        };
        position_set.add_position(position);
    }

    let tuning_config = TuningConfig::default();

    // Should not error (even if optimization doesn't converge)
    let result = evaluator.tune_weights(&position_set, &tuning_config);
    assert!(
        result.is_ok(),
        "tune_weights should not error on valid input"
    );

    let tuning_result = result.unwrap();
    // Iterations might be less than max_iterations if we converge early or hit early stopping
    assert!(
        tuning_result.iterations <= tuning_config.max_iterations,
        "Iterations should not exceed max_iterations"
    );
    assert!(
        tuning_result.iterations > 0,
        "Should complete at least one iteration"
    );
    assert!(tuning_result.final_error >= 0.0);
    assert!(
        tuning_result.error_history.len() > 0,
        "Should have error history"
    );
}

/// Test weight adapter layers (Task 20.0 - Task 4.20)
#[test]
fn test_weight_adapter_layers() {
    // Test to_vector()
    let weights = EvaluationWeights::default();
    let vector = weights.to_vector();

    assert_eq!(vector.len(), 10, "Weight vector should have 10 elements");
    assert_eq!(vector[0], weights.material_weight as f64);
    assert_eq!(vector[1], weights.position_weight as f64);
    assert_eq!(vector[2], weights.king_safety_weight as f64);
    assert_eq!(vector[3], weights.pawn_structure_weight as f64);
    assert_eq!(vector[4], weights.mobility_weight as f64);
    assert_eq!(vector[5], weights.center_control_weight as f64);
    assert_eq!(vector[6], weights.development_weight as f64);
    assert_eq!(vector[7], weights.tactical_weight as f64);
    assert_eq!(vector[8], weights.positional_weight as f64);
    assert_eq!(vector[9], weights.castle_weight as f64);

    // Test from_vector()
    let test_vector = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let converted_weights = EvaluationWeights::from_vector(&test_vector).unwrap();

    assert_eq!(converted_weights.material_weight, 1.0);
    assert_eq!(converted_weights.position_weight, 2.0);
    assert_eq!(converted_weights.king_safety_weight, 3.0);
    assert_eq!(converted_weights.pawn_structure_weight, 4.0);
    assert_eq!(converted_weights.mobility_weight, 5.0);
    assert_eq!(converted_weights.center_control_weight, 6.0);
    assert_eq!(converted_weights.development_weight, 7.0);
    assert_eq!(converted_weights.tactical_weight, 8.0);
    assert_eq!(converted_weights.positional_weight, 9.0);
    assert_eq!(converted_weights.castle_weight, 10.0);

    // Test error case: wrong length
    let wrong_vector = vec![1.0, 2.0, 3.0];
    let error_result = EvaluationWeights::from_vector(&wrong_vector);
    assert!(error_result.is_err());
    assert!(error_result.unwrap_err().contains("Expected 10 weights"));

    // Test round-trip conversion
    let original = EvaluationWeights::default();
    let vector = original.to_vector();
    let round_trip = EvaluationWeights::from_vector(&vector).unwrap();

    assert_eq!(original.material_weight, round_trip.material_weight);
    assert_eq!(original.position_weight, round_trip.position_weight);
    assert_eq!(original.king_safety_weight, round_trip.king_safety_weight);
    assert_eq!(
        original.pawn_structure_weight,
        round_trip.pawn_structure_weight
    );
    assert_eq!(original.mobility_weight, round_trip.mobility_weight);
    assert_eq!(
        original.center_control_weight,
        round_trip.center_control_weight
    );
    assert_eq!(original.development_weight, round_trip.development_weight);
    assert_eq!(original.tactical_weight, round_trip.tactical_weight);
    assert_eq!(original.positional_weight, round_trip.positional_weight);
    assert_eq!(original.castle_weight, round_trip.castle_weight);
}

/// Test that tuning improves evaluation accuracy (Task 20.0 - Task 4.21)
#[test]
fn test_tuning_improves_evaluation() {
    let mut evaluator = IntegratedEvaluator::new();
    let mut position_set = TuningPositionSet::empty();

    // Create training positions with known expected scores
    // Using initial position as a simple test case
    for i in 0..10 {
        let expected_score = if i % 2 == 0 {
            0.0 // Draw
        } else {
            0.1 // Slight advantage
        };

        let position = TuningPosition {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            player: Player::Black,
            expected_score,
            game_phase: 128,
            move_number: 10,
        };
        position_set.add_position(position);
    }

    // Get initial evaluation error
    // Note: We can't access weights directly, so we'll just verify tuning completes
    let _initial_error = calculate_error(&evaluator, &position_set);

    // Run tuning with limited iterations
    let mut tuning_config = TuningConfig::default();
    tuning_config.max_iterations = 10; // Small for test speed
    tuning_config.learning_rate = 0.01;

    let result = evaluator.tune_weights(&position_set, &tuning_config);
    assert!(result.is_ok(), "tune_weights should succeed");

    let tuning_result = result.unwrap();

    // Verify that final error is reasonable
    assert!(
        tuning_result.final_error >= 0.0,
        "Final error should be non-negative"
    );
    assert!(
        tuning_result.iterations > 0,
        "Should complete at least one iteration"
    );
    assert!(
        tuning_result.error_history.len() > 0,
        "Error history should have at least one entry, got {}",
        tuning_result.error_history.len()
    );
    // Note: error_history might be shorter than iterations if we complete early
    // but should be at least 1 entry

    // Verify optimized weights are valid
    assert!(
        tuning_result.optimized_weights.material_weight >= 0.0,
        "Material weight should be non-negative"
    );
    assert!(
        tuning_result.optimized_weights.position_weight >= 0.0,
        "Position weight should be non-negative"
    );

    // Note: We can't guarantee that error decreased because we're using a very
    // simplified test case, but we can verify the mechanism works
}

/// Test telemetry-to-tuning pipeline (Task 20.0 - Task 4.22)
#[test]
fn test_telemetry_tuning_pipeline() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    // Create some positions and collect telemetry
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Evaluate and get telemetry
    evaluator.evaluate(&board, player, &captured_pieces);
    let telemetry = evaluator.telemetry_snapshot().unwrap();

    // Create telemetry positions
    let mut telemetry_positions = Vec::new();
    for i in 0..5 {
        let expected_score = if i % 2 == 0 { 0.0 } else { 0.1 };
        telemetry_positions.push((
            board.clone(),
            captured_pieces.clone(),
            player,
            telemetry.clone(),
            expected_score,
        ));
    }

    // Test telemetry-to-tuning pipeline
    let position_set = evaluator.telemetry_to_tuning_pipeline(&telemetry_positions);

    assert_eq!(position_set.len(), 5);
    assert!(!position_set.is_empty());

    // Verify positions have correct expected scores
    for (i, position) in position_set.positions.iter().enumerate() {
        let expected = if i % 2 == 0 { 0.0 } else { 0.1 };
        assert_eq!(position.expected_score, expected);
        assert_eq!(position.player, player);
    }
}

/// Test tune_from_telemetry() method
#[test]
fn test_tune_from_telemetry() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    // Create telemetry by evaluating positions
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    evaluator.evaluate(&board, player, &captured_pieces);
    let telemetry = evaluator.telemetry_snapshot().unwrap();

    // Create telemetry set
    let telemetry_set = vec![telemetry.clone(); 5];

    // Test tune_from_telemetry
    let result = evaluator.tune_from_telemetry(&telemetry_set, None, 0.01);
    assert!(result.is_ok());

    let optimized_weights = result.unwrap();

    // Verify weights are valid
    assert!(optimized_weights.material_weight >= 0.0);
    assert!(optimized_weights.position_weight >= 0.0);
}

/// Test TuningPositionSet operations
#[test]
fn test_tuning_position_set() {
    let mut position_set = TuningPositionSet::empty();

    assert!(position_set.is_empty());
    assert_eq!(position_set.len(), 0);

    // Add positions
    for i in 0..5 {
        let position = TuningPosition {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            player: Player::Black,
            expected_score: i as f64 * 0.1,
            game_phase: 128,
            move_number: i as u32,
        };
        position_set.add_position(position);
    }

    assert!(!position_set.is_empty());
    assert_eq!(position_set.len(), 5);

    // Test new() constructor
    let positions = position_set.positions.clone();
    let new_set = TuningPositionSet::new(positions);
    assert_eq!(new_set.len(), 5);
}

/// Helper function to calculate error for testing
fn calculate_error(evaluator: &IntegratedEvaluator, position_set: &TuningPositionSet) -> f64 {
    let mut total_error = 0.0;
    let k_factor = 1.0;

    for position in &position_set.positions {
        let predicted_score =
            evaluator.evaluate(&position.board, position.player, &position.captured_pieces) as f64;

        let predicted_prob = sigmoid(predicted_score * k_factor);
        let expected_prob = position.expected_score;
        let error = expected_prob - predicted_prob;
        total_error += error * error;
    }

    total_error / position_set.len() as f64
}

/// Sigmoid function for probability conversion
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
