//! Tests for benchmark result aggregation and reporting (Task 26.0 - Task 2.0)

use shogi_engine::search::performance_tuning::{BenchmarkAggregator, BenchmarkReport};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_benchmark_aggregation() {
    // Create a test aggregator
    let temp_dir = TempDir::new().unwrap();
    let aggregator = BenchmarkAggregator::with_directory(temp_dir.path());

    // Create mock criterion directory structure
    let criterion_dir = temp_dir.path().join("criterion");
    let benchmark_dir = criterion_dir.join("test_benchmark");
    let id_dir = benchmark_dir.join("test_id");
    let base_dir = id_dir.join("base");
    fs::create_dir_all(&base_dir).unwrap();

    // Create a mock estimates.json file
    let estimates_json = r#"
    {
        "mean": {
            "point_estimate": 1000000.0,
            "standard_error": 50000.0,
            "confidence_interval": {
                "confidence_level": 0.95,
                "lower_bound": 900000.0,
                "upper_bound": 1100000.0
            }
        },
        "throughput": {
            "per_second": {
                "point_estimate": 1000.0
            }
        }
    }
    "#;
    fs::write(base_dir.join("estimates.json"), estimates_json).unwrap();

    // Aggregate results
    let reports = aggregator.aggregate_criterion_results(&criterion_dir).unwrap();

    // Verify aggregation
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].benchmark_name, "test_benchmark");
    assert!((reports[0].mean_time_ns - 1000000.0).abs() < 0.1);
    assert!((reports[0].std_dev_ns - 50000.0).abs() < 0.1);
    assert!((reports[0].throughput_ops_per_sec - 1000.0).abs() < 0.1);
}

#[test]
fn test_report_generation() {
    let aggregator = BenchmarkAggregator::new();

    // Create test reports
    let reports = vec![
        BenchmarkReport {
            benchmark_name: "bench1".to_string(),
            mean_time_ns: 1000000.0,
            std_dev_ns: 50000.0,
            throughput_ops_per_sec: 1000.0,
            samples: 100,
            baseline_comparison: None,
        },
        BenchmarkReport {
            benchmark_name: "bench2".to_string(),
            mean_time_ns: 2000000.0,
            std_dev_ns: 100000.0,
            throughput_ops_per_sec: 500.0,
            samples: 100,
            baseline_comparison: None,
        },
    ];

    // Generate aggregated report
    let aggregated = aggregator.generate_benchmark_report(&reports);

    // Verify report structure
    assert_eq!(aggregated.benchmarks.len(), 2);
    assert_eq!(aggregated.summary.total_benchmarks, 2);
    assert!((aggregated.summary.average_mean_time_ns - 1500000.0).abs() < 0.1);
    assert!((aggregated.summary.total_throughput_ops_per_sec - 1500.0).abs() < 0.1);
    assert_eq!(aggregated.summary.regressions_detected, 0);
}

#[test]
fn test_report_export_json() {
    let temp_dir = TempDir::new().unwrap();
    let aggregator = BenchmarkAggregator::with_directory(temp_dir.path());

    let reports = vec![BenchmarkReport {
        benchmark_name: "test_bench".to_string(),
        mean_time_ns: 1000000.0,
        std_dev_ns: 50000.0,
        throughput_ops_per_sec: 1000.0,
        samples: 100,
        baseline_comparison: None,
    }];

    let aggregated = aggregator.generate_benchmark_report(&reports);
    aggregator.export_report_to_json(&aggregated, "test_report.json").unwrap();

    // Verify file was created
    let report_file = temp_dir.path().join("test_report.json");
    assert!(report_file.exists());

    // Verify JSON is valid
    let content = fs::read_to_string(&report_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["summary"]["total_benchmarks"], 1);
}

#[test]
fn test_report_export_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let aggregator = BenchmarkAggregator::with_directory(temp_dir.path());

    let reports = vec![BenchmarkReport {
        benchmark_name: "test_bench".to_string(),
        mean_time_ns: 1000000.0,
        std_dev_ns: 50000.0,
        throughput_ops_per_sec: 1000.0,
        samples: 100,
        baseline_comparison: None,
    }];

    let aggregated = aggregator.generate_benchmark_report(&reports);
    aggregator.export_report_to_markdown(&aggregated, "test_report.md").unwrap();

    // Verify file was created
    let report_file = temp_dir.path().join("test_report.md");
    assert!(report_file.exists());

    // Verify markdown content
    let content = fs::read_to_string(&report_file).unwrap();
    assert!(content.contains("# Benchmark Report"));
    assert!(content.contains("test_bench"));
    assert!(content.contains("Summary"));
}

#[test]
fn test_benchmark_baseline_comparison() {
    // Create a test baseline file
    let temp_dir = TempDir::new().unwrap();
    let baseline_file = temp_dir.path().join("baseline.json");

    // Create a minimal baseline JSON
    let baseline_json = r#"
    {
        "timestamp": "2024-01-01T00:00:00Z",
        "git_commit": "test",
        "hardware": {
            "cpu": "Test CPU",
            "cores": 4,
            "ram_gb": 8
        },
        "search_metrics": {
            "nodes_per_second": 1000000.0,
            "average_cutoff_rate": 0.25,
            "average_cutoff_index": 1.5
        },
        "evaluation_metrics": {
            "average_evaluation_time_ns": 350.0,
            "cache_hit_rate": 0.72,
            "phase_calc_time_ns": 120.0
        },
        "tt_metrics": {
            "hit_rate": 0.65,
            "exact_entry_rate": 0.32,
            "occupancy_rate": 0.52
        },
        "move_ordering_metrics": {
            "average_cutoff_index": 1.4,
            "pv_hit_rate": 0.45,
            "killer_hit_rate": 0.22,
            "cache_hit_rate": 0.58
        },
        "parallel_search_metrics": {
            "speedup_4_cores": 3.5,
            "speedup_8_cores": 6.2,
            "efficiency_4_cores": 0.88,
            "efficiency_8_cores": 0.78
        },
        "memory_metrics": {
            "tt_memory_mb": 16.0,
            "cache_memory_mb": 4.0,
            "peak_memory_mb": 32.0
        }
    }
    "#;
    fs::write(&baseline_file, baseline_json).unwrap();

    // Create a benchmark report
    let report = BenchmarkReport {
        benchmark_name: "test_bench".to_string(),
        mean_time_ns: 1100000.0, // 10% slower than baseline
        std_dev_ns: 50000.0,
        throughput_ops_per_sec: 900.0,
        samples: 100,
        baseline_comparison: None,
    };

    // Compare with baseline (5% threshold)
    let comparison = report.compare_with_baseline(&baseline_file, 5.0).unwrap();

    // Verify comparison detects regression (>5% threshold)
    assert!(comparison.has_regression);
    assert!(comparison.change_percent > 5.0);
}

#[test]
fn test_full_benchmark_pipeline() {
    // This is a simplified integration test
    // In a real scenario, we'd run actual benchmarks
    let temp_dir = TempDir::new().unwrap();
    let aggregator = BenchmarkAggregator::with_directory(temp_dir.path());

    // Create mock reports
    let reports = vec![
        BenchmarkReport {
            benchmark_name: "bench1".to_string(),
            mean_time_ns: 1000000.0,
            std_dev_ns: 50000.0,
            throughput_ops_per_sec: 1000.0,
            samples: 100,
            baseline_comparison: None,
        },
    ];

    // Generate report
    let aggregated = aggregator.generate_benchmark_report(&reports);

    // Export to both formats
    aggregator.export_report_to_json(&aggregated, "pipeline_test.json").unwrap();
    aggregator.export_report_to_markdown(&aggregated, "pipeline_test.md").unwrap();

    // Verify both files exist
    assert!(temp_dir.path().join("pipeline_test.json").exists());
    assert!(temp_dir.path().join("pipeline_test.md").exists());
}

#[test]
fn test_environment_variable_baseline_path() {
    // Test that BENCHMARK_BASELINE_PATH is respected
    let test_path = "/test/baseline.json";
    std::env::set_var("BENCHMARK_BASELINE_PATH", test_path);

    let aggregator = BenchmarkAggregator::new();
    assert_eq!(
        aggregator.baseline_path.as_ref().map(|p| p.to_str().unwrap()),
        Some(test_path)
    );

    // Clean up
    std::env::remove_var("BENCHMARK_BASELINE_PATH");
}

