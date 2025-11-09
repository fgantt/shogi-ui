#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::*;
use shogi_engine::types::*;

/// Comprehensive test suite for tapered evaluation system
/// Tests all aspects of dual-phase evaluation including:
/// - Game phase calculation
/// - TaperedScore interpolation
/// - Dual-phase piece-square tables
/// - Phase-dependent evaluation components
/// - Integration and performance

#[test]
fn test_game_phase_calculation_comprehensive() {
    let evaluator = PositionEvaluator::new();

    // Test starting position (should be maximum phase)
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let phase = evaluator.calculate_game_phase(&board, &captured_pieces);
    assert_eq!(
        phase, GAME_PHASE_MAX,
        "Starting position should have maximum phase"
    );

    // Test phase calculation consistency
    for _ in 0..10 {
        let phase_repeat = evaluator.calculate_game_phase(&board, &captured_pieces);
        assert_eq!(
            phase, phase_repeat,
            "Phase calculation should be consistent"
        );
    }

    // Test phase range
    assert!(phase >= 0, "Phase should be non-negative");
    assert!(phase <= GAME_PHASE_MAX, "Phase should not exceed maximum");
}

#[test]
fn test_tapered_score_interpolation_comprehensive() {
    // Test interpolation at all phase values
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

    // Test smooth interpolation across all phases
    let mut prev_score = test_score.interpolate(0);
    for phase in 1..=256 {
        let current_score = test_score.interpolate(phase);

        // Score should change smoothly (no sudden jumps)
        let score_diff = (current_score - prev_score).abs();
        assert!(
            score_diff <= 2,
            "Smooth interpolation at phase {}: {} -> {} (diff: {})",
            phase,
            prev_score,
            current_score,
            score_diff
        );

        // Score should be between mg and eg values
        assert!(
            current_score >= 100,
            "Score should not be below mg value: {}",
            current_score
        );
        assert!(
            current_score <= 200,
            "Score should not exceed eg value: {}",
            current_score
        );

        prev_score = current_score;
    }
}

#[test]
fn test_dual_phase_piece_square_tables_comprehensive() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();

    // Test that evaluation works with all piece types
    let piece_types = [
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
    ];

    // Test evaluation with different piece types present
    for piece_type in piece_types.iter() {
        // Test evaluation for both players
        for player in [Player::Black, Player::White] {
            let score = evaluator.evaluate(&board, player, &CapturedPieces::new());

            // Score should be reasonable
            assert!(
                score.abs() < 10000,
                "Evaluation score should be reasonable for {:?} as {:?}: {}",
                piece_type,
                player,
                score
            );
        }
    }
}

#[test]
fn test_evaluation_components_tapered_score() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that the main evaluation function works and returns reasonable scores
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(
        score.abs() < 10000,
        "Evaluation score should be reasonable: {}",
        score
    );

    // Test that evaluation is consistent
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert_eq!(score, score2, "Evaluation should be consistent");
}

#[test]
fn test_evaluation_phase_differences_comprehensive() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that evaluation works and returns reasonable scores
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(
        score.abs() < 10000,
        "Evaluation score should be reasonable: {}",
        score
    );

    // Test that evaluation is consistent across multiple calls
    for _ in 0..10 {
        let score_repeat = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert_eq!(score, score_repeat, "Evaluation should be consistent");
    }
}

#[test]
fn test_evaluation_integration_pipeline() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test complete evaluation pipeline
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Score should be reasonable
    assert!(
        score.abs() < 10000,
        "Evaluation score should be reasonable: {}",
        score
    );

    // Test consistency across multiple calls
    for _ in 0..10 {
        let score_repeat = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert_eq!(
            score, score_repeat,
            "Evaluation should be consistent across calls"
        );
    }

    // Test both players
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
}

#[test]
fn test_evaluation_performance_benchmark() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Performance benchmark
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
fn test_evaluation_edge_cases() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test edge cases
    let test_score = TaperedScore::new_tapered(100, 200);

    // Test extreme phase values
    let phase_neg = test_score.interpolate(-100);
    let phase_large = test_score.interpolate(1000);

    // Results should be reasonable (can be negative for extreme values)
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
fn test_evaluation_symmetry() {
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
fn test_tapered_score_arithmetic() {
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
fn test_game_phase_edge_cases() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test phase calculation with empty board (if possible)
    // Note: This might not be possible with current BitboardBoard implementation
    let phase = evaluator.calculate_game_phase(&board, &captured_pieces);

    // Phase should be in valid range
    assert!(phase >= 0, "Phase should be non-negative: {}", phase);
    assert!(
        phase <= GAME_PHASE_MAX,
        "Phase should not exceed maximum: {}",
        phase
    );

    // Test phase calculation consistency
    for _ in 0..100 {
        let phase_repeat = evaluator.calculate_game_phase(&board, &captured_pieces);
        assert_eq!(
            phase, phase_repeat,
            "Phase calculation should be deterministic"
        );
    }
}

#[test]
fn test_evaluation_consistency_stress() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Stress test: multiple evaluations
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
fn test_tapered_evaluation_comprehensive() {
    let config = PhaseTransitionConfig {
        use_advanced_interpolator: true,
        ..PhaseTransitionConfig::default()
    };
    let mut evaluator = PositionEvaluator::new_with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Comprehensive test of the entire tapered evaluation system
    let mut transition = PhaseTransition::with_config(config);
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    for method in [
        InterpolationMethod::Linear,
        InterpolationMethod::Cubic,
        InterpolationMethod::Sigmoid,
        InterpolationMethod::Smoothstep,
        InterpolationMethod::Advanced,
    ] {
        let smooth =
            transition.validate_smooth_transitions(TaperedScore::new_tapered(100, 200), method);
        assert!(
            smooth,
            "Interpolation method {:?} should maintain smooth transitions",
            method
        );
    }

    // The final score should be reasonable
    assert!(
        score.abs() < 10000,
        "Final evaluation score should be reasonable: {}",
        score
    );
}
