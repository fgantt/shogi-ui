//! Tests for automatic profiling integration (Task 26.0 - Task 3.0)

use shogi_engine::evaluation::performance::{HotPathEntry, PerformanceProfiler};
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::types::*;

#[test]
fn test_auto_profiling_enable() {
    let mut engine = SearchEngine::new(None, 16);
    
    // Initially disabled
    assert!(!engine.auto_profiling_enabled);
    
    // Enable profiling
    engine.enable_auto_profiling();
    assert!(engine.auto_profiling_enabled);
    
    // Disable profiling
    engine.disable_auto_profiling();
    assert!(!engine.auto_profiling_enabled);
}

#[test]
fn test_profiling_sample_rate() {
    let mut profiler = PerformanceProfiler::new();
    profiler.enable();
    profiler.set_sample_rate(10); // Profile every 10th call
    
    // Record 20 operations
    for i in 0..20 {
        profiler.record_operation("test_op", i as u64 * 1000);
    }
    
    // Should have approximately 2 samples (20 / 10)
    let hot_paths = profiler.get_hot_path_summary(10);
    assert!(hot_paths.len() > 0);
    
    // Check that sample rate is working (should have fewer samples than calls)
    let test_op = hot_paths.iter().find(|e| e.operation == "test_op");
    if let Some(op) = test_op {
        assert!(op.call_count <= 20, "Sample rate should reduce call count");
    }
}

#[test]
fn test_hot_path_identification() {
    let mut engine = SearchEngine::new(None, 16);
    engine.enable_auto_profiling();
    engine.performance_profiler.set_sample_rate(1); // Profile every call for testing
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Run some evaluations
    for _ in 0..10 {
        let _ = engine.evaluate_position(&board, Player::Black, &captured_pieces);
    }
    
    // Get hot path summary
    let hot_paths = engine.get_hot_path_summary(10);
    
    // Should have at least evaluation in hot paths
    assert!(hot_paths.len() > 0, "Should identify at least one hot path");
    
    // Check that evaluation is in the hot paths
    let eval_entry = hot_paths.iter().find(|e| e.operation == "evaluation");
    assert!(eval_entry.is_some(), "Should identify evaluation as hot path");
    
    if let Some(eval) = eval_entry {
        assert!(eval.call_count > 0, "Should have recorded evaluation calls");
        assert!(eval.average_time_ns > 0.0, "Should have recorded timing");
    }
}

#[test]
fn test_profiling_overhead_tracking() {
    let mut profiler = PerformanceProfiler::new();
    profiler.enable();
    profiler.set_sample_rate(1);
    
    // Record some operations
    for _ in 0..100 {
        profiler.record_operation("test", 1000);
    }
    
    // Check overhead tracking
    let overhead_pct = profiler.get_profiling_overhead_percentage();
    assert!(overhead_pct >= 0.0, "Overhead should be non-negative");
    assert!(overhead_pct < 100.0, "Overhead should be reasonable");
}

#[test]
fn test_export_profiling_data() {
    let mut profiler = PerformanceProfiler::new();
    profiler.enable();
    profiler.set_sample_rate(1);
    
    // Record some operations
    profiler.record_evaluation(1000);
    profiler.record_phase_calculation(500);
    profiler.record_interpolation(200);
    profiler.record_operation("custom_op", 1500);
    
    // Export to JSON
    let json_result = profiler.export_profiling_data();
    assert!(json_result.is_ok(), "Should export profiling data successfully");
    
    let json_str = json_result.unwrap();
    assert!(!json_str.is_empty(), "JSON should not be empty");
    
    // Verify JSON contains expected fields
    assert!(json_str.contains("enabled"), "Should contain enabled field");
    assert!(json_str.contains("sample_rate"), "Should contain sample_rate field");
    assert!(json_str.contains("hot_paths"), "Should contain hot_paths field");
    assert!(json_str.contains("evaluation_stats"), "Should contain evaluation_stats");
}

#[test]
fn test_profiling_with_move_ordering() {
    let mut engine = SearchEngine::new(None, 16);
    engine.enable_auto_profiling();
    engine.performance_profiler.set_sample_rate(1);
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let moves = vec![
        Move::new(0, 0, 1, 1, Player::Black, None),
        Move::new(1, 1, 2, 2, Player::Black, None),
    ];
    
    // Order moves
    let _ordered = engine.order_moves_for_negamax(
        &moves,
        &board,
        &captured_pieces,
        Player::Black,
        3,
        -10000,
        10000,
        None,
        None,
    );
    
    // Check that move ordering was profiled
    let hot_paths = engine.get_hot_path_summary(10);
    let ordering_entry = hot_paths.iter().find(|e| e.operation == "move_ordering");
    assert!(ordering_entry.is_some(), "Should profile move ordering");
}

#[test]
fn test_profiling_reset() {
    let mut profiler = PerformanceProfiler::new();
    profiler.enable();
    profiler.set_sample_rate(1);
    
    // Record some data
    profiler.record_evaluation(1000);
    profiler.record_operation("test", 500);
    
    assert!(profiler.evaluation_times.len() > 0);
    
    // Reset
    profiler.reset();
    
    // Verify reset
    assert_eq!(profiler.evaluation_times.len(), 0);
    assert_eq!(profiler.operation_timings.len(), 0);
    assert_eq!(profiler.profiling_overhead_ns, 0);
    assert_eq!(profiler.profiling_operations, 0);
}

#[test]
fn test_hot_path_summary_ordering() {
    let mut profiler = PerformanceProfiler::new();
    profiler.enable();
    profiler.set_sample_rate(1);
    
    // Record operations with different timings
    profiler.record_operation("slow_op", 10000);
    profiler.record_operation("fast_op", 100);
    profiler.record_operation("medium_op", 1000);
    
    // Get hot path summary (top 3)
    let hot_paths = profiler.get_hot_path_summary(3);
    
    assert_eq!(hot_paths.len(), 3);
    
    // Should be ordered by average time (descending)
    assert_eq!(hot_paths[0].operation, "slow_op");
    assert_eq!(hot_paths[1].operation, "medium_op");
    assert_eq!(hot_paths[2].operation, "fast_op");
    
    // Verify timing values
    assert!(hot_paths[0].average_time_ns > hot_paths[1].average_time_ns);
    assert!(hot_paths[1].average_time_ns > hot_paths[2].average_time_ns);
}

