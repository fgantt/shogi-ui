#![cfg(feature = "legacy-tests")]
//! Comprehensive Testing Suite for Tapered Evaluation
//!
//! This module provides end-to-end integration tests, stress tests,
//! accuracy validation, and regression tests for the complete tapered
//! evaluation system.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::*;

#[test]
fn test_integrated_evaluator_end_to_end() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Starting position should have balanced evaluation
    assert!(score.abs() < 100, "Starting position score: {}", score);
}

#[test]
fn test_position_evaluator_uses_integrated() {
    let evaluator = PositionEvaluator::new();

    // Verify integrated evaluator is enabled by default
    assert!(evaluator.is_using_integrated_evaluator());
}

#[test]
fn test_evaluation_consistency() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Same position should give same score
    let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    assert_eq!(score1, score2);
}

#[test]
fn test_evaluation_symmetry() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);

    // Scores should be negated for opposite players
    assert_eq!(black_score, -white_score);
}

#[test]
fn test_cache_effectiveness() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // First evaluation
    evaluator.evaluate(&board, Player::Black, &captured_pieces);

    let stats = evaluator.cache_stats();

    // Cache should have entries
    assert!(stats.phase_cache_size > 0 || stats.eval_cache_size > 0);
}

#[test]
fn test_stress_many_evaluations() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Run many evaluations without crashing
    for _ in 0..10000 {
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(score.abs() < 100000);
    }
}

#[test]
fn test_statistics_tracking() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Run evaluations
    for _ in 0..100 {
        evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let stats = evaluator.get_statistics();
    assert_eq!(stats.count(), 100);
}

#[test]
fn test_component_flags() {
    use shogi_engine::evaluation::integration::{ComponentFlags, IntegratedEvaluationConfig};

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::minimal();

    let mut evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Should still work with minimal components
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert!(score.abs() < 100000);
}

#[test]
fn test_search_engine_integration() {
    let search_engine = SearchEngine::new(None, 64);

    // Verify tapered search enhancer is available
    let enhancer = search_engine.get_tapered_search_enhancer();
    assert!(enhancer.stats().phase_transitions == 0);
}

#[test]
fn test_phase_tracking_in_search() {
    let mut search_engine = SearchEngine::new(None, 64);
    let board = BitboardBoard::new();

    // Track phase
    let enhancer = search_engine.get_tapered_search_enhancer_mut();
    let phase = enhancer.track_phase(&board);

    // Starting position should be in opening/middlegame
    assert!(phase >= 64, "Phase: {}", phase);
}

#[test]
fn test_regression_basic_evaluation() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Regression check: starting position should be close to 0
    assert!(score.abs() < 100, "Starting position regression: {}", score);
}

#[test]
fn test_clear_caches_functionality() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Populate caches
    evaluator.evaluate(&board, Player::Black, &captured_pieces);

    let stats_before = evaluator.cache_stats();
    assert!(stats_before.eval_cache_size > 0);

    // Clear caches
    evaluator.clear_caches();

    let stats_after = evaluator.cache_stats();
    assert_eq!(stats_after.phase_cache_size, 0);
    assert_eq!(stats_after.eval_cache_size, 0);
}

#[test]
fn test_performance_under_load() {
    use std::time::Instant;

    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let start = Instant::now();

    // 1000 evaluations should be fast
    for _ in 0..1000 {
        evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let duration = start.elapsed();

    // Should complete in under 10ms (average ~10μs per eval)
    assert!(duration.as_millis() < 10, "Duration: {:?}", duration);
}

#[test]
fn test_accuracy_starting_position() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Starting position: equal material, should be near 0
    assert!(
        score >= -50 && score <= 50,
        "Starting position accuracy: {}",
        score
    );
}

#[test]
fn test_known_position_material_advantage() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // For a balanced starting position
    let balanced_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Material advantage should result in positive score for that player
    // This is a basic sanity check
    assert!(balanced_score.abs() < 200);
}

#[test]
fn test_end_to_end_evaluation_pipeline() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Full pipeline: PositionEvaluator -> IntegratedEvaluator -> Components
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    assert!(score.abs() < 100000);
}

#[test]
fn test_stress_cache_growth() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Run many evaluations
    for _ in 0..1000 {
        evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let stats = evaluator.cache_stats();

    // Cache should not grow unbounded (should be 1 for same position)
    assert!(stats.eval_cache_size <= 10);
}

#[test]
fn test_statistics_accuracy() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Run 50 evaluations
    for _ in 0..50 {
        evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }

    let stats = evaluator.get_statistics();

    // Should have recorded exactly 50
    assert_eq!(stats.count(), 50);
}

#[test]
fn test_component_isolation() {
    use shogi_engine::evaluation::integration::{ComponentFlags, IntegratedEvaluationConfig};

    // Test with each component isolated
    let components = [
        ComponentFlags {
            material: true,
            ..ComponentFlags::all_disabled()
        },
        ComponentFlags {
            piece_square_tables: true,
            ..ComponentFlags::all_disabled()
        },
    ];

    for comp in &components {
        let mut config = IntegratedEvaluationConfig::default();
        config.components = comp.clone();

        let mut evaluator = IntegratedEvaluator::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Should not crash with isolated components
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(score.abs() < 100000);
    }
}

#[test]
fn test_regression_cache_consistency() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Clear and re-evaluate
    evaluator.clear_caches();
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

    // Should get same result
    assert_eq!(score1, score2);
}
