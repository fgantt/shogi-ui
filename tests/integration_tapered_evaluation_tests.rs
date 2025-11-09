#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::*;
use shogi_engine::types::*;

/// Comprehensive integration tests for tapered evaluation system
/// Tests the complete evaluation pipeline with various game positions
/// and verifies that the tapered evaluation works correctly in practice

#[test]
fn test_integration_starting_position() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test starting position evaluation
    let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);

    // Both scores should be positive (evaluation from player's perspective)
    assert!(
        black_score > 0,
        "Black starting position should be positive: {}",
        black_score
    );
    assert!(
        white_score > 0,
        "White starting position should be positive: {}",
        white_score
    );

    // Scores should be similar (not opposite) since evaluation is from player's perspective
    let score_diff = (black_score - white_score).abs();
    assert!(
        score_diff < 100,
        "Black and White scores should be similar: {} vs {}",
        black_score,
        white_score
    );

    // Game phase should be maximum for starting position
    let game_phase = evaluator.calculate_game_phase(&board, &captured_pieces);
    assert_eq!(
        game_phase, GAME_PHASE_MAX,
        "Starting position should have maximum phase: {}",
        game_phase
    );
}

#[test]
fn test_integration_evaluation_consistency() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test evaluation consistency across multiple calls
    let mut scores = Vec::new();
    for _ in 0..100 {
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        scores.push(score);
    }

    // All scores should be identical
    let first_score = scores[0];
    for (i, score) in scores.iter().enumerate() {
        assert_eq!(
            *score, first_score,
            "Score {} should match first score: {} vs {}",
            i, score, first_score
        );
    }
}

#[test]
fn test_integration_phase_calculation_consistency() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test phase calculation consistency
    let mut phases = Vec::new();
    for _ in 0..100 {
        let phase = evaluator.calculate_game_phase(&board, &captured_pieces);
        phases.push(phase);
    }

    // All phases should be identical
    let first_phase = phases[0];
    for (i, phase) in phases.iter().enumerate() {
        assert_eq!(
            *phase, first_phase,
            "Phase {} should match first phase: {} vs {}",
            i, phase, first_phase
        );
    }
}

#[test]
fn test_integration_tapered_score_arithmetic() {
    // Test TaperedScore arithmetic operations
    let score1 = TaperedScore::new_tapered(100, 200);
    let score2 = TaperedScore::new_tapered(50, 75);

    // Test addition
    let sum = score1 + score2;
    assert_eq!(sum.mg, 150);
    assert_eq!(sum.eg, 275);

    // Test subtraction
    let diff = score1 - score2;
    assert_eq!(diff.mg, 50);
    assert_eq!(diff.eg, 125);

    // Test negation
    let neg = -score1;
    assert_eq!(neg.mg, -100);
    assert_eq!(neg.eg, -200);

    // Test addition assignment
    let mut score3 = TaperedScore::new_tapered(10, 20);
    score3 += score2;
    assert_eq!(score3.mg, 60);
    assert_eq!(score3.eg, 95);

    // Test subtraction assignment
    score3 -= score1;
    assert_eq!(score3.mg, -40);
    assert_eq!(score3.eg, -105);
}

#[test]
fn test_integration_interpolation_accuracy() {
    // Test interpolation accuracy across all phase values
    let test_score = TaperedScore::new_tapered(100, 200);

    // Test exact boundaries
    assert_eq!(
        test_score.interpolate(0),
        200,
        "Phase 0 should return pure eg value"
    );
    assert_eq!(
        test_score.interpolate(256),
        100,
        "Phase 256 should return pure mg value"
    );

    // Test midpoint
    let midpoint = test_score.interpolate(128);
    assert!(
        midpoint >= 140 && midpoint <= 160,
        "Phase 128 should be approximately halfway: {}",
        midpoint
    );

    // Test smooth interpolation
    let mut prev_score = test_score.interpolate(0);
    for phase in 1..=256 {
        let current_score = test_score.interpolate(phase);

        // Score should change smoothly
        let score_diff = (current_score - prev_score).abs();
        assert!(
            score_diff <= 2,
            "Smooth interpolation at phase {}: {} -> {} (diff: {})",
            phase,
            prev_score,
            current_score,
            score_diff
        );

        prev_score = current_score;
    }
}

#[test]
fn test_integration_evaluation_performance() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Performance test
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() / iterations as u128;

    // Should complete 1000 evaluations in reasonable time
    assert!(
        duration.as_millis() < 1000,
        "1000 evaluations should complete in < 1 second: {}ms",
        duration.as_millis()
    );

    // Average time per evaluation should be reasonable
    assert!(
        avg_time < 1000,
        "Average evaluation time should be < 1ms: {}μs",
        avg_time
    );
}

#[test]
fn test_integration_evaluation_symmetry() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test evaluation symmetry
    let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);

    // Both scores should be positive for starting position
    assert!(
        black_score > 0,
        "Black evaluation should be positive: {}",
        black_score
    );
    assert!(
        white_score > 0,
        "White evaluation should be positive: {}",
        white_score
    );

    // Scores should be similar (not opposite) since evaluation is from player's perspective
    let score_diff = (black_score - white_score).abs();
    assert!(
        score_diff < 100,
        "Black and White scores should be similar: {} vs {}",
        black_score,
        white_score
    );
}

#[test]
fn test_integration_edge_cases() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test edge cases
    let test_score = TaperedScore::new_tapered(100, 200);

    // Test extreme phase values
    let phase_neg = test_score.interpolate(-100);
    let phase_large = test_score.interpolate(1000);

    // Results should be reasonable
    assert!(
        phase_neg >= -1000,
        "Negative phase should produce reasonable result: {}",
        phase_neg
    );
    assert!(
        phase_large >= -1000,
        "Large phase should produce reasonable result: {}",
        phase_large
    );

    // Test zero TaperedScore
    let zero_score = TaperedScore::default();
    assert_eq!(
        zero_score.interpolate(128),
        0,
        "Zero score should interpolate to zero"
    );

    // Test negative TaperedScore
    let neg_score = TaperedScore::new_tapered(-100, -200);
    let neg_result = neg_score.interpolate(128);
    assert!(
        neg_result < 0,
        "Negative score should interpolate to negative: {}",
        neg_result
    );
}

#[test]
fn test_integration_comprehensive_evaluation() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Comprehensive test of the entire tapered evaluation system
    let game_phase = evaluator.calculate_game_phase(&board, &captured_pieces);
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Verify all components work together
    assert!(
        game_phase >= 0 && game_phase <= GAME_PHASE_MAX,
        "Game phase should be valid: {}",
        game_phase
    );
    assert!(
        score.abs() < 10000,
        "Final score should be reasonable: {}",
        score
    );

    // Test that evaluation is consistent
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert_eq!(score, score2, "Evaluation should be consistent");
}

#[test]
fn test_integration_stress_test() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Stress test: multiple evaluations with different players
    let mut scores = Vec::new();
    for _ in 0..50 {
        let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);
        scores.push((black_score, white_score));
    }

    // All scores should be consistent
    let first_black = scores[0].0;
    let first_white = scores[0].1;

    for (i, (black_score, white_score)) in scores.iter().enumerate() {
        assert_eq!(
            *black_score, first_black,
            "Black score {} should match first: {} vs {}",
            i, black_score, first_black
        );
        assert_eq!(
            *white_score, first_white,
            "White score {} should match first: {} vs {}",
            i, white_score, first_white
        );
    }
}

#[test]
fn test_integration_phase_boundaries() {
    // Test evaluation at phase boundaries
    let test_score = TaperedScore::new_tapered(100, 200);

    // Test exact phase boundaries
    let phase_0_score = test_score.interpolate(0);
    let phase_1_score = test_score.interpolate(1);
    let phase_255_score = test_score.interpolate(255);
    let phase_256_score = test_score.interpolate(256);

    // Verify boundary behavior
    assert_eq!(phase_0_score, 200, "Phase 0 should be pure eg");
    assert_eq!(phase_256_score, 100, "Phase 256 should be pure mg");

    // Verify smooth transition at boundaries
    let diff_0_1 = (phase_1_score - phase_0_score).abs();
    let diff_255_256 = (phase_256_score - phase_255_score).abs();

    assert!(
        diff_0_1 <= 1,
        "Smooth transition at phase 0-1: {}",
        diff_0_1
    );
    assert!(
        diff_255_256 <= 1,
        "Smooth transition at phase 255-256: {}",
        diff_255_256
    );
}

#[test]
fn test_integration_evaluation_quality() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that evaluation produces reasonable scores
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be in reasonable range
    assert!(
        score > 0,
        "Evaluation should be positive for starting position: {}",
        score
    );
    assert!(score < 1000, "Evaluation should not be too high: {}", score);

    // Test that evaluation is deterministic
    for _ in 0..10 {
        let score_repeat = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert_eq!(score, score_repeat, "Evaluation should be deterministic");
    }
}
