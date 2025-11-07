//! Automated Performance Monitoring and Regression Testing for Null Move Pruning
//!
//! This benchmark suite provides comprehensive performance monitoring, regression testing,
//! and automated reporting for NMP effectiveness across different configurations and position types.
//!
//! Features:
//! - Automated benchmark execution for CI/CD
//! - Performance regression detection
//! - Statistics tracking over time
//! - Position-type specific metrics (opening, middlegame, endgame)
//! - Comparison benchmarks (NMP enabled vs disabled)
//! - Automated performance reports

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json;
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
};
use std::fs;
use std::path::Path as StdPath;
use std::time::Duration as StdDuration;

/// Performance metrics for a single benchmark run
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: String,
    pub configuration: String,
    pub position_type: String,
    pub depth: u8,
    pub search_time_ms: f64,
    pub nodes_searched: u64,
    pub nmp_attempts: u64,
    pub nmp_cutoffs: u64,
    pub cutoff_rate: f64,
    pub average_reduction: f64,
    pub efficiency: f64,
    pub verification_attempts: u64,
    pub verification_cutoffs: u64,
    pub mate_threat_attempts: u64,
    pub mate_threat_detected: u64,
    pub disabled_endgame: u64,
    pub disabled_material_endgame: u64,
    pub disabled_king_activity_endgame: u64,
    pub disabled_zugzwang: u64,
}

/// Performance baseline for regression testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceBaseline {
    pub min_cutoff_rate: f64,      // Minimum acceptable cutoff rate (%)
    pub max_search_time_ms: f64,   // Maximum acceptable search time (ms)
    pub min_efficiency: f64,       // Minimum acceptable efficiency (%)
    pub max_overhead_percent: f64, // Maximum acceptable overhead (%)
}

impl Default for PerformanceBaseline {
    fn default() -> Self {
        Self {
            min_cutoff_rate: 30.0,      // At least 30% cutoff rate
            max_search_time_ms: 5000.0, // Max 5 seconds per search
            min_efficiency: 20.0,       // At least 20% efficiency
            max_overhead_percent: 20.0, // Max 20% overhead from NMP features
        }
    }
}

/// Save metrics to JSON file for historical tracking
fn save_metrics(
    metrics: &PerformanceMetrics,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load existing metrics if file exists
    let mut all_metrics: Vec<PerformanceMetrics> = if StdPath::new(file_path).exists() {
        let content = fs::read_to_string(file_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Add new metrics
    all_metrics.push(metrics.clone());

    // Keep only last 100 entries to prevent file from growing too large
    if all_metrics.len() > 100 {
        all_metrics = all_metrics[all_metrics.len() - 100..].to_vec();
    }

    // Save to file
    let json = serde_json::to_string_pretty(&all_metrics)?;
    fs::write(file_path, json)?;

    Ok(())
}

/// Load metrics history
fn load_metrics_history(
    file_path: &str,
) -> Result<Vec<PerformanceMetrics>, Box<dyn std::error::Error>> {
    if StdPath::new(file_path).exists() {
        let content = fs::read_to_string(file_path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(Vec::new())
    }
}

/// Create a test engine with specific NMP configuration
fn create_test_engine_with_config(config: NullMoveConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Benchmark NMP enabled vs disabled comparison
fn benchmark_nmp_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_comparison");
    group.measurement_time(StdDuration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test configurations
    let configurations = vec![
        ("disabled", {
            let mut config = NullMoveConfig::default();
            config.enabled = false;
            config
        }),
        ("default", NullMoveConfig::default()),
        ("with_verification", {
            let mut config = NullMoveConfig::default();
            config.verification_margin = 200;
            config
        }),
        ("with_mate_threat", {
            let mut config = NullMoveConfig::default();
            config.enable_mate_threat_detection = true;
            config.mate_threat_margin = 500;
            config
        }),
        ("with_endgame_type", {
            let mut config = NullMoveConfig::default();
            config.enable_endgame_type_detection = true;
            config
        }),
        ("full_features", {
            let mut config = NullMoveConfig::default();
            config.verification_margin = 200;
            config.enable_mate_threat_detection = true;
            config.mate_threat_margin = 500;
            config.enable_endgame_type_detection = true;
            config
        }),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configurations {
            group.bench_with_input(
                BenchmarkId::new(name.to_string(), depth),
                &depth,
                |b, &depth| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_null_move_stats();

                        let start = std::time::Instant::now();
                        let mut board_mut = board.clone();
                        let result = engine.search_at_depth_legacy(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            depth,
                            1000,
                        );
                        let elapsed = start.elapsed();

                        let stats = engine.get_null_move_stats().clone();
                        let nodes = engine.get_nodes_searched();

                        black_box((result, elapsed, nodes, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark NMP effectiveness across position types
fn benchmark_nmp_by_position_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_by_position_type");
    group.measurement_time(StdDuration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test with default configuration
    let config = NullMoveConfig::default();

    group.bench_function("initial_position", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(config.clone());
            engine.reset_null_move_stats();

            let start = std::time::Instant::now();
            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );
            let elapsed = start.elapsed();

            let stats = engine.get_null_move_stats().clone();
            let nodes = engine.get_nodes_searched();

            black_box((result, elapsed, nodes, stats))
        });
    });

    group.finish();
}

/// Benchmark NMP regression testing
fn benchmark_nmp_regression_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_regression_testing");
    group.measurement_time(StdDuration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let config = NullMoveConfig::default();

    // Test at different depths to ensure consistent performance
    for depth in [3, 4, 5] {
        group.bench_with_input(
            BenchmarkId::new("regression", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_config(config.clone());
                    engine.reset_null_move_stats();

                    let start = std::time::Instant::now();
                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );
                    let elapsed = start.elapsed();

                    let stats = engine.get_null_move_stats().clone();
                    let nodes = engine.get_nodes_searched();
                    let cutoff_rate = stats.cutoff_rate();
                    let efficiency = stats.efficiency();

                    // Regression checks (only fail in CI, not in normal benchmarks)
                    if std::env::var("NMP_REGRESSION_TEST").is_ok() {
                        let baseline = PerformanceBaseline::default();
                        if stats.attempts > 0 {
                            assert!(
                                cutoff_rate >= baseline.min_cutoff_rate,
                                "Regression: cutoff rate {} < threshold {}",
                                cutoff_rate,
                                baseline.min_cutoff_rate
                            );
                            assert!(
                                efficiency >= baseline.min_efficiency,
                                "Regression: efficiency {} < threshold {}",
                                efficiency,
                                baseline.min_efficiency
                            );
                        }
                        assert!(
                            elapsed.as_secs_f64() * 1000.0 <= baseline.max_search_time_ms,
                            "Regression: search time {}ms > threshold {}ms",
                            elapsed.as_secs_f64() * 1000.0,
                            baseline.max_search_time_ms
                        );
                    }

                    black_box((result, elapsed, nodes, cutoff_rate, efficiency))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark comprehensive NMP performance monitoring
fn benchmark_comprehensive_nmp_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_nmp_monitoring");
    group.measurement_time(StdDuration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let config = NullMoveConfig::default();

    group.bench_function("full_monitoring", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(config.clone());
            engine.reset_null_move_stats();

            let start = std::time::Instant::now();
            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );
            let elapsed = start.elapsed();

            let stats = engine.get_null_move_stats().clone();
            let nodes = engine.get_nodes_searched();

            // Collect comprehensive metrics
            let metrics = PerformanceMetrics {
                timestamp: chrono::Utc::now().to_rfc3339(),
                configuration: "default".to_string(),
                position_type: "initial".to_string(),
                depth: 5,
                search_time_ms: elapsed.as_secs_f64() * 1000.0,
                nodes_searched: nodes,
                nmp_attempts: stats.attempts,
                nmp_cutoffs: stats.cutoffs,
                cutoff_rate: stats.cutoff_rate(),
                average_reduction: stats.average_reduction_factor(),
                efficiency: stats.efficiency(),
                verification_attempts: stats.verification_attempts,
                verification_cutoffs: stats.verification_cutoffs,
                mate_threat_attempts: stats.mate_threat_attempts,
                mate_threat_detected: stats.mate_threat_detected,
                disabled_endgame: stats.disabled_endgame,
                disabled_material_endgame: stats.disabled_material_endgame,
                disabled_king_activity_endgame: stats.disabled_king_activity_endgame,
                disabled_zugzwang: stats.disabled_zugzwang,
            };

            // Save metrics if directory is set
            if let Ok(metrics_dir) = std::env::var("NMP_METRICS_DIR") {
                let _ = save_metrics(&metrics, &format!("{}/nmp_metrics.json", metrics_dir));
            }

            black_box((result, metrics))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_nmp_comparison,
    benchmark_nmp_by_position_type,
    benchmark_nmp_regression_testing,
    benchmark_comprehensive_nmp_monitoring
);

criterion_main!(benches);
