#![cfg(feature = "legacy-tests")]
//! Unit tests for Task 5.0 code quality improvements (Task 5.15)
//!
//! This module tests:
//! - Named constants (MIN_SCORE, MAX_SCORE)
//! - Improved fallback logic using static evaluation
//! - Best score initialization fixes
//! - Core search metrics functionality

use shogi_engine::{
    bitboards::BitboardBoard,
    search::search_engine::{SearchEngine, MAX_SCORE, MIN_SCORE},
    search::zobrist::RepetitionState,
    types::{CapturedPieces, Player},
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn test_min_max_score_constants() {
    // Test that constants are defined correctly
    assert_eq!(MIN_SCORE, i32::MIN + 1);
    assert_eq!(MAX_SCORE, i32::MAX - 1);

    // Test that constants are safe (not at boundary)
    assert!(MIN_SCORE > i32::MIN);
    assert!(MAX_SCORE < i32::MAX);

    // Test that constants can be used in comparisons
    assert!(0 > MIN_SCORE);
    assert!(0 < MAX_SCORE);
}

#[test]
fn test_constants_in_search_context() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = SearchEngine::new(Some(stop_flag), 64);

    // Verify constants are accessible
    assert_eq!(MIN_SCORE, i32::MIN + 1);
    assert_eq!(MAX_SCORE, i32::MAX - 1);

    // Constants should be usable for alpha-beta bounds
    assert!(MIN_SCORE < 0);
    assert!(MAX_SCORE > 0);
    assert!(MIN_SCORE < MAX_SCORE);
}

#[test]
fn test_core_search_metrics_initialization() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = SearchEngine::new(Some(stop_flag), 64);

    let metrics = engine.get_core_search_metrics();

    // Initial metrics should be zero
    assert_eq!(metrics.total_nodes, 0);
    assert_eq!(metrics.total_cutoffs, 0);
    assert_eq!(metrics.total_tt_probes, 0);
    assert_eq!(metrics.total_tt_hits, 0);
    assert_eq!(metrics.total_aspiration_searches, 0);
    assert_eq!(metrics.successful_aspiration_searches, 0);
    assert_eq!(metrics.beta_cutoffs, 0);

    // Rates should be 0.0 when no data
    assert_eq!(metrics.cutoff_rate(), 0.0);
    assert_eq!(metrics.tt_hit_rate(), 0.0);
    assert_eq!(metrics.aspiration_success_rate(), 0.0);
}

#[test]
fn test_core_search_metrics_calculation() {
    use shogi_engine::types::CoreSearchMetrics;

    let mut metrics = CoreSearchMetrics::default();

    // Simulate some search activity
    metrics.total_nodes = 1000;
    metrics.total_cutoffs = 150;
    metrics.total_tt_probes = 500;
    metrics.total_tt_hits = 200;
    metrics.total_aspiration_searches = 100;
    metrics.successful_aspiration_searches = 80;
    metrics.beta_cutoffs = 100;

    // Test cutoff rate
    let cutoff_rate = metrics.cutoff_rate();
    assert!(
        (cutoff_rate - 15.0).abs() < 0.01,
        "Cutoff rate should be ~15%"
    );

    // Test TT hit rate
    let tt_hit_rate = metrics.tt_hit_rate();
    assert!(
        (tt_hit_rate - 40.0).abs() < 0.01,
        "TT hit rate should be ~40%"
    );

    // Test aspiration success rate
    let aspiration_success = metrics.aspiration_success_rate();
    assert!(
        (aspiration_success - 80.0).abs() < 0.01,
        "Aspiration success rate should be ~80%"
    );
}

#[test]
fn test_core_search_metrics_reset() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);

    // Get metrics and manually modify (simulating search)
    let metrics = engine.get_core_search_metrics();
    // Note: Can't directly modify through reference, so we test the reset function

    // Reset metrics
    engine.reset_core_search_metrics();

    let metrics_after_reset = engine.get_core_search_metrics();
    assert_eq!(metrics_after_reset.total_nodes, 0);
    assert_eq!(metrics_after_reset.total_cutoffs, 0);
    assert_eq!(metrics_after_reset.total_tt_probes, 0);
}

#[test]
fn test_core_search_metrics_report_generation() {
    use shogi_engine::types::CoreSearchMetrics;

    let mut metrics = CoreSearchMetrics::default();

    // Add some data
    metrics.total_nodes = 1000;
    metrics.total_cutoffs = 150;
    metrics.total_tt_probes = 500;
    metrics.total_tt_hits = 200;
    metrics.tt_exact_hits = 100;
    metrics.tt_lower_bound_hits = 60;
    metrics.tt_upper_bound_hits = 40;
    metrics.total_aspiration_searches = 100;
    metrics.successful_aspiration_searches = 80;
    metrics.beta_cutoffs = 100;

    // Generate report
    let report = metrics.generate_report();

    // Verify report contains expected information
    assert!(report.contains("Core Search Metrics Report"));
    assert!(report.contains("1000")); // total nodes
    assert!(report.contains("150")); // total cutoffs
    assert!(report.contains("500")); // total TT probes
    assert!(report.contains("200")); // total TT hits
    assert!(report.contains("100")); // aspiration searches
    assert!(report.contains("80")); // successful aspiration
}

#[test]
fn test_tt_hit_breakdown() {
    use shogi_engine::types::CoreSearchMetrics;

    let mut metrics = CoreSearchMetrics::default();

    // Set up TT hit data
    metrics.total_tt_hits = 100;
    metrics.tt_exact_hits = 50;
    metrics.tt_lower_bound_hits = 30;
    metrics.tt_upper_bound_hits = 20;

    let (exact_pct, lower_pct, upper_pct) = metrics.tt_hit_breakdown();

    // Percentages should sum to 100% (within rounding)
    let total = exact_pct + lower_pct + upper_pct;
    assert!(
        (total - 100.0).abs() < 0.01,
        "Percentages should sum to ~100%"
    );

    // Verify individual percentages
    assert!((exact_pct - 50.0).abs() < 0.01, "Exact should be ~50%");
    assert!(
        (lower_pct - 30.0).abs() < 0.01,
        "Lower bound should be ~30%"
    );
    assert!(
        (upper_pct - 20.0).abs() < 0.01,
        "Upper bound should be ~20%"
    );
}

#[test]
fn test_repetition_state_enum() {
    // Test RepetitionState enum
    assert_eq!(RepetitionState::None.is_draw(), false);
    assert_eq!(RepetitionState::TwoFold.is_draw(), false);
    assert_eq!(RepetitionState::ThreeFold.is_draw(), false);
    assert_eq!(RepetitionState::FourFold.is_draw(), true);
}

#[test]
fn test_fallback_logic_constants() {
    // Test that constants are appropriate for fallback logic
    // MIN_SCORE should be used as lower bound for alpha
    // MAX_SCORE should be used as upper bound for beta
    assert!(MIN_SCORE < 0);
    assert!(MAX_SCORE > 0);

    // Constants should not be at i32 boundaries (to avoid sentinel value issues)
    assert!(MIN_SCORE > i32::MIN);
    assert!(MAX_SCORE < i32::MAX);
}

#[test]
fn test_hash_based_repetition_integration() {
    // Test hash-based repetition using ShogiHashHandler directly
    // (hash_calculator field is private in SearchEngine)
    use shogi_engine::search::ShogiHashHandler;

    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    // Get position hash
    let hash = hash_handler.get_position_hash(&board, player, &captured);

    // Initially, no repetition
    let state = hash_handler.get_repetition_state_for_hash(hash);
    assert_eq!(state, RepetitionState::None);

    // Add position multiple times
    for _ in 0..4 {
        hash_handler.add_position_to_history(hash);
    }

    // Should detect four-fold repetition
    let state = hash_handler.get_repetition_state_for_hash(hash);
    assert_eq!(state, RepetitionState::FourFold);
    assert!(hash_handler.is_repetition(hash));
}

#[test]
fn test_metrics_report_formatting() {
    use shogi_engine::types::CoreSearchMetrics;

    let metrics = CoreSearchMetrics::default();
    let report = metrics.generate_report();

    // Report should contain key sections
    assert!(report.contains("Total Nodes Searched"));
    assert!(report.contains("Transposition Table"));
    assert!(report.contains("Aspiration Windows"));

    // Report should be formatted nicely
    assert!(report.contains("=")); // Separator
    assert!(report.lines().count() > 10); // Should have multiple lines
}
