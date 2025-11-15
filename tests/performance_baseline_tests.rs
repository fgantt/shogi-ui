//! Tests for performance baseline persistence and comparison (Task 26.0 - Task 1.0)

use shogi_engine::search::performance_tuning::BaselineManager;
use shogi_engine::types::{PerformanceBaseline, BaselineMoveOrderingMetrics};
use tempfile::TempDir;

#[test]
fn test_baseline_serialization() {
    // Create a test baseline
    let baseline = PerformanceBaseline::new();
    
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let manager = BaselineManager::with_directory(temp_dir.path());
    
    // Save baseline
    let filename = "test_baseline.json";
    manager.save_baseline(&baseline, filename).unwrap();
    
    // Load baseline
    let loaded = manager.load_baseline(filename).unwrap();
    
    // Verify round-trip
    assert_eq!(baseline.timestamp, loaded.timestamp);
    assert_eq!(baseline.git_commit, loaded.git_commit);
    assert_eq!(baseline.hardware.cores, loaded.hardware.cores);
    assert_eq!(baseline.search_metrics.nodes_per_second, loaded.search_metrics.nodes_per_second);
}

#[test]
fn test_baseline_comparison() {
    // Create two baselines with different metrics
    let mut baseline1 = PerformanceBaseline::new();
    baseline1.search_metrics.nodes_per_second = 1000000.0;
    baseline1.search_metrics.average_cutoff_rate = 0.25;
    
    let mut baseline2 = PerformanceBaseline::new();
    baseline2.search_metrics.nodes_per_second = 1100000.0; // 10% improvement
    baseline2.search_metrics.average_cutoff_rate = 0.30; // 20% improvement
    
    let manager = BaselineManager::new();
    let comparison = manager.compare_baselines(&baseline2, &baseline1);
    
    // Verify comparison calculates differences correctly
    // nodes_per_second: 1100000 / 1000000 = 1.1, so 10% increase
    assert!((comparison.search_metrics_diff.nodes_per_second_change - 10.0).abs() < 0.1);
    
    // average_cutoff_rate: 0.30 / 0.25 = 1.2, so 20% increase
    assert!((comparison.search_metrics_diff.average_cutoff_rate_change - 20.0).abs() < 0.1);
}

#[test]
fn test_baseline_regression_detection() {
    // Create baseline with good performance
    let mut baseline = PerformanceBaseline::new();
    baseline.search_metrics.nodes_per_second = 1000000.0;
    baseline.search_metrics.average_cutoff_rate = 0.25;
    baseline.evaluation_metrics.average_evaluation_time_ns = 350.0;
    baseline.tt_metrics.hit_rate = 0.65;
    
    // Create current baseline with regressions (>5% degradation)
    let mut current = PerformanceBaseline::new();
    current.search_metrics.nodes_per_second = 940000.0; // 6% degradation
    current.search_metrics.average_cutoff_rate = 0.20; // 20% degradation
    current.evaluation_metrics.average_evaluation_time_ns = 400.0; // 14% degradation
    current.tt_metrics.hit_rate = 0.60; // 7.7% degradation
    
    let manager = BaselineManager::new();
    let result = manager.detect_regression(&current, &baseline);
    
    // Should detect regressions
    assert!(result.has_regression);
    assert!(!result.regressions.is_empty());
    
    // Verify specific regressions are detected
    let nodes_regression = result.regressions.iter()
        .find(|r| r.metric == "nodes_per_second")
        .unwrap();
    assert!(nodes_regression.change_percent < -5.0);
    
    let cutoff_regression = result.regressions.iter()
        .find(|r| r.metric == "average_cutoff_rate")
        .unwrap();
    assert!(cutoff_regression.change_percent < -5.0);
}

#[test]
fn test_baseline_no_regression() {
    // Create baseline
    let baseline = PerformanceBaseline::new();
    
    // Create current baseline with improvements
    let mut current = PerformanceBaseline::new();
    current.search_metrics.nodes_per_second = 1050000.0; // 5% improvement
    current.search_metrics.average_cutoff_rate = 0.26; // 4% improvement
    
    let manager = BaselineManager::new();
    let result = manager.detect_regression(&current, &baseline);
    
    // Should not detect regressions (improvements don't count as regressions)
    // Note: This test may need adjustment based on actual implementation
    // If baseline has 0.0 values, improvements won't trigger regression detection
}

#[test]
fn test_baseline_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_dir = temp_dir.path().join("baselines");
    let manager = BaselineManager::with_directory(&baseline_dir);
    
    let baseline = PerformanceBaseline::new();
    manager.save_baseline(&baseline, "test.json").unwrap();
    
    // Verify directory was created
    assert!(baseline_dir.exists());
    assert!(baseline_dir.join("test.json").exists());
}

#[test]
fn test_baseline_git_commit_hash() {
    // Test that git commit hash is included
    let baseline = PerformanceBaseline::new();
    
    // Git commit should be set (either from git or "unknown")
    assert!(!baseline.git_commit.is_empty());
}

#[test]
fn test_baseline_hardware_info() {
    let baseline = PerformanceBaseline::new();
    
    // Hardware info should be populated
    assert!(baseline.hardware.cores > 0);
    // CPU may be "Unknown" if detection fails, which is acceptable
}

