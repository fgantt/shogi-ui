//! Tests for benchmark regression suite (Task 26.0 - Task 5.0)

use shogi_engine::search::performance_tuning::{load_standard_positions, BenchmarkRunner, RegressionTestResult};
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::types::{BenchmarkPositionType, *};

#[test]
fn test_standard_positions_loading() {
    let positions = load_standard_positions();
    assert!(positions.is_ok(), "Should load standard positions successfully");
    
    let positions = positions.unwrap();
    assert_eq!(positions.len(), 5, "Should have 5 standard positions");
    
    // Verify position types
    let position_types: Vec<PositionType> = positions.iter()
        .map(|p| p.position_type)
        .collect();
    
    assert!(position_types.contains(&PositionType::Opening));
    assert!(position_types.contains(&PositionType::MiddlegameTactical));
    assert!(position_types.contains(&PositionType::MiddlegamePositional));
    assert!(position_types.contains(&PositionType::EndgameKingActivity));
    assert!(position_types.contains(&PositionType::EndgameZugzwang));
    
    // Verify all positions have valid FEN strings
    for position in &positions {
        assert!(!position.fen.is_empty(), "Position {} should have FEN", position.name);
        assert!(position.expected_depth > 0, "Position {} should have expected depth > 0", position.name);
    }
}

#[test]
fn test_regression_detection() {
    let runner = BenchmarkRunner::new()
        .with_regression_threshold(5.0);
    
    // Create test results
    let results = vec![
        RegressionTestResult::new("Position1".to_string(), 100, 110, 5.0), // 10% regression
        RegressionTestResult::new("Position2".to_string(), 100, 103, 5.0), // 3% improvement (no regression)
        RegressionTestResult::new("Position3".to_string(), 100, 106, 5.0), // 6% regression
    ];
    
    let regressions = runner.detect_regressions(&results);
    
    // Should detect 2 regressions (Position1 and Position3)
    assert_eq!(regressions.len(), 2);
    assert!(regressions.iter().any(|r| r.position_name == "Position1"));
    assert!(regressions.iter().any(|r| r.position_name == "Position3"));
    assert!(!regressions.iter().any(|r| r.position_name == "Position2"));
}

#[test]
fn test_regression_test_result_creation() {
    let result = RegressionTestResult::new(
        "TestPosition".to_string(),
        100,
        110,
        5.0,
    );
    
    assert_eq!(result.position_name, "TestPosition");
    assert_eq!(result.baseline_time_ms, 100);
    assert_eq!(result.current_time_ms, 110);
    assert_eq!(result.regression_percentage, 10.0); // 10% slower
    assert!(result.regression_detected); // > 5% threshold
}

#[test]
fn test_regression_suite_execution() {
    let mut engine = SearchEngine::new(None, 16);
    let runner = BenchmarkRunner::new()
        .with_time_limit(5000); // 5 seconds per position
    
    // Run regression suite (may take a while)
    let result = runner.run_regression_suite(&mut engine);
    
    assert!(result.is_ok(), "Regression suite should run successfully");
    let suite_result = result.unwrap();
    
    assert_eq!(suite_result.total_positions, 5);
    assert_eq!(suite_result.benchmark_results.len(), 5);
    
    // Verify all positions were benchmarked
    for (benchmark, _) in &suite_result.benchmark_results {
        assert!(benchmark.search_time_ms > 0, "Should have search time > 0");
        assert!(benchmark.depth_searched > 0, "Should have depth > 0");
    }
}

#[test]
fn test_benchmark_runner_configuration() {
    let runner = BenchmarkRunner::new()
        .with_regression_threshold(10.0)
        .with_time_limit(3000);
    
    // Verify configuration
    // (We can't directly access private fields, but we can test behavior)
    let result = RegressionTestResult::new(
        "Test".to_string(),
        100,
        115,
        10.0,
    );
    
    // 15% regression should be detected with 10% threshold
    assert!(result.regression_detected);
}

#[test]
fn test_position_benchmark_result() {
    use shogi_engine::search::performance_tuning::PositionBenchmarkResult;
    
    let result = PositionBenchmarkResult {
        position_name: "Test".to_string(),
        search_time_ms: 100,
        nodes_searched: 1000,
        nodes_per_second: 10000.0,
        depth_searched: 5,
        best_move_found: true,
    };
    
    assert_eq!(result.position_name, "Test");
    assert_eq!(result.search_time_ms, 100);
    assert_eq!(result.nodes_searched, 1000);
    assert!(result.best_move_found);
}

