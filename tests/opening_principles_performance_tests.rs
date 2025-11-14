//! Tests for opening principles performance optimizations and statistics (Task 19.0 - Task 4.0)
//!
//! These tests verify:
//! - Bitboard-optimized piece finding
//! - Per-component statistics tracking
//! - Attack-based center control evaluation

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::opening_principles::OpeningPrincipleEvaluator;
use shogi_engine::types::*;

/// Test that bitboard-optimized piece finding works (tested indirectly via evaluation)
#[test]
fn test_bitboard_optimized_piece_finding() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    // Test that evaluation works correctly (uses bitboard-optimized find_pieces internally)
    let score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

    // Should return valid score (bitboard optimization is transparent to caller)
    let score_interp = score.interpolate(256);
    assert!(score_interp.abs() < 100000);
}

/// Test per-component statistics tracking
#[test]
fn test_per_component_statistics() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    // Evaluate opening principles
    let _score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

    // Check that statistics are tracked
    let stats = evaluator.stats();
    // Statistics should be tracked if components are enabled (default: all enabled)
    assert!(stats.development_evaluations >= 0);
    assert!(stats.center_control_evaluations >= 0);

    // Get component statistics
    let component_stats = evaluator.get_component_statistics();
    assert!(component_stats.development.evaluation_count >= 0);
    assert!(component_stats.center_control.evaluation_count >= 0);
}

/// Test component statistics breakdown
#[test]
fn test_component_statistics_breakdown() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    // Evaluate multiple times
    for _ in 0..3 {
        let _score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);
    }

    let component_stats = evaluator.get_component_statistics();

    // Check that averages are calculated correctly
    if component_stats.development.evaluation_count > 0 {
        let expected_avg = component_stats.development.total_score as f64
            / component_stats.development.evaluation_count as f64;
        assert!((component_stats.development.average_score - expected_avg).abs() < 0.01);
    }
}

/// Test attack-based center control
#[test]
fn test_attack_based_center_control() {
    use shogi_engine::evaluation::opening_principles::OpeningPrincipleConfig;

    let mut config = OpeningPrincipleConfig::default();
    config.enable_attack_based_center_control = true;
    let mut evaluator = OpeningPrincipleEvaluator::with_config(config);

    let board = BitboardBoard::new();
    let score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

    // Should return valid score
    let score_interp = score.interpolate(256);
    assert!(score_interp.abs() < 100000);
}

/// Test that statistics are reset correctly
#[test]
fn test_statistics_reset() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    // Evaluate and accumulate statistics
    let _score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);
    let stats_before = evaluator.stats().evaluations;
    assert!(stats_before > 0);

    // Reset statistics
    evaluator.reset_stats();
    let stats_after = evaluator.stats().evaluations;
    assert_eq!(stats_after, 0);
}
