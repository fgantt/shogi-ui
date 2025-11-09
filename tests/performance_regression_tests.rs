#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::*;
use shogi_engine::types::*;
use std::time::{Duration, Instant};

/// Performance regression tests for tapered evaluation system
/// These tests ensure that performance doesn't degrade over time
/// and that the evaluation system meets performance requirements

#[test]
fn test_evaluation_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Performance regression test: 1000 evaluations should complete in < 1 second
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let duration = start.elapsed();

    // Regression threshold: should complete in < 1 second
    assert!(
        duration < Duration::from_secs(1),
        "Performance regression: 1000 evaluations took {}ms (threshold: 1000ms)",
        duration.as_millis()
    );

    // Average time per evaluation should be < 1ms
    let avg_time = duration.as_micros() / iterations as u128;
    assert!(
        avg_time < 1000,
        "Performance regression: average evaluation time {}μs (threshold: 1000μs)",
        avg_time
    );
}

#[test]
fn test_phase_calculation_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();

    // Phase calculation performance regression test
    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = evaluator.calculate_game_phase(&board, &CapturedPieces::new());
    }

    let duration = start.elapsed();

    // Phase calculation should be fast: 10000 calls in < 200ms
    assert!(
        duration < Duration::from_millis(200),
        "Phase calculation regression: 10000 calls took {}ms (threshold: 200ms)",
        duration.as_millis()
    );

    // Average time per phase calculation should be < 20μs
    let avg_time = duration.as_micros() / iterations as u128;
    assert!(
        avg_time < 20,
        "Phase calculation regression: average time {}μs (threshold: 20μs)",
        avg_time
    );
}

#[test]
fn test_tapered_score_interpolation_performance_regression() {
    // TaperedScore interpolation performance regression test
    let test_score = TaperedScore::new_tapered(100, 200);
    let iterations = 100000;
    let start = Instant::now();

    for phase in 0..iterations {
        let _ = test_score.interpolate((phase % 257) as i32);
    }

    let duration = start.elapsed();

    // Interpolation should be very fast: 100000 calls in < 50ms
    assert!(
        duration < Duration::from_millis(50),
        "Interpolation regression: 100000 calls took {}ms (threshold: 50ms)",
        duration.as_millis()
    );

    // Average time per interpolation should be < 1μs
    let avg_time = duration.as_nanos() / iterations as u128;
    assert!(
        avg_time < 1000,
        "Interpolation regression: average time {}ns (threshold: 1000ns)",
        avg_time
    );
}

#[test]
fn test_memory_usage_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Memory usage regression test
    let iterations = 1000;
    let start = Instant::now();

    // Create many evaluators to test memory usage
    let mut evaluators = Vec::new();
    for _ in 0..10 {
        evaluators.push(PositionEvaluator::new());
    }

    // Perform evaluations
    for _ in 0..iterations {
        for evaluator in &evaluators {
            let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        }
    }

    let duration = start.elapsed();

    // Memory usage should not significantly impact performance
    assert!(
        duration < Duration::from_secs(30),
        "Memory usage regression: 10 evaluators * 1000 evaluations took {}ms (threshold: 30000ms)",
        duration.as_millis()
    );
}

#[test]
fn test_evaluation_consistency_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Consistency performance regression test
    let iterations = 1000;
    let start = Instant::now();

    // Test consistency across multiple calls
    for _ in 0..iterations {
        let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert_eq!(score1, score2, "Evaluation should be consistent");
    }

    let duration = start.elapsed();

    // Consistency checks should not significantly impact performance
    assert!(
        duration < Duration::from_millis(2000),
        "Consistency regression: 1000 consistency checks took {}ms (threshold: 2000ms)",
        duration.as_millis()
    );
}

#[test]
fn test_tapered_score_arithmetic_performance_regression() {
    // TaperedScore arithmetic performance regression test
    let score1 = TaperedScore::new_tapered(100, 200);
    let score2 = TaperedScore::new_tapered(50, 75);
    let iterations = 100000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = score1 + score2;
        let _ = score1 - score2;
        let _ = -score1;
    }

    let duration = start.elapsed();

    // Arithmetic operations should be very fast
    assert!(
        duration < Duration::from_millis(50),
        "Arithmetic regression: 100000 operations took {}ms (threshold: 50ms)",
        duration.as_millis()
    );
}

#[test]
fn test_evaluation_symmetry_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Symmetry performance regression test
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);

        // Both scores should be positive for starting position
        assert!(
            black_score > 0,
            "Black score should be positive: {}",
            black_score
        );
        assert!(
            white_score > 0,
            "White score should be positive: {}",
            white_score
        );
    }

    let duration = start.elapsed();

    // Symmetry checks should not significantly impact performance
    assert!(
        duration < Duration::from_millis(2000),
        "Symmetry regression: 1000 symmetry checks took {}ms (threshold: 2000ms)",
        duration.as_millis()
    );
}

#[test]
fn test_comprehensive_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Comprehensive performance regression test
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Test complete evaluation pipeline
        let game_phase = evaluator.calculate_game_phase(&board, &CapturedPieces::new());
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        // Verify results are reasonable
        assert!(game_phase >= 0 && game_phase <= GAME_PHASE_MAX);
        assert!(score.abs() < 10000);
    }

    let duration = start.elapsed();

    // Comprehensive evaluation should complete in reasonable time
    assert!(
        duration < Duration::from_secs(1),
        "Comprehensive regression: 1000 complete evaluations took {}ms (threshold: 1000ms)",
        duration.as_millis()
    );
}

#[test]
fn test_stress_performance_regression() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Stress test performance regression
    let iterations = 10000;
    let start = Instant::now();

    for i in 0..iterations {
        let player = if i % 2 == 0 {
            Player::Black
        } else {
            Player::White
        };
        let _ = evaluator.evaluate(&board, player, &captured_pieces);
    }

    let duration = start.elapsed();

    // Stress test should complete in reasonable time
    assert!(
        duration < Duration::from_secs(10),
        "Stress regression: 10000 evaluations took {}ms (threshold: 10000ms)",
        duration.as_millis()
    );
}

#[test]
fn test_performance_benchmark_comparison() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Performance benchmark comparison test
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() / iterations as u128;

    // Performance should be consistent with previous benchmarks
    // This test ensures that performance doesn't degrade over time
    assert!(
        avg_time < 1000,
        "Performance benchmark regression: average time {}μs (threshold: 1000μs)",
        avg_time
    );

    // Log performance metrics for monitoring
    println!(
        "Performance benchmark: {}μs per evaluation ({}ms total for {} iterations)",
        avg_time,
        duration.as_millis(),
        iterations
    );
}
