//! Performance benchmarks for mate threat detection
//!
//! This benchmark suite measures the performance overhead and effectiveness of
//! mate threat detection for null move pruning.
//!
//! Expected results:
//! - Mate threat detection should have minimal overhead when enabled
//! - Mate threat detection should improve NMP safety in winning positions
//! - Performance impact should be acceptable (<10% overhead)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with mate threat detection enabled
fn create_test_engine_with_mate_threat_detection() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.enabled = true;
    config.enable_mate_threat_detection = true;
    config.mate_threat_margin = 500;
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Create a test engine without mate threat detection
fn create_test_engine_without_mate_threat_detection() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.enabled = true;
    config.enable_mate_threat_detection = false;
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Benchmark mate threat detection overhead
fn benchmark_mate_threat_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("mate_threat_detection_overhead");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths
    for depth in [3, 4, 5, 6] {
        // Benchmark with mate threat detection disabled
        group.bench_with_input(
            BenchmarkId::new("without_mate_threat", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_without_mate_threat_detection();
                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();
                    black_box((result, stats))
                });
            },
        );

        // Benchmark with mate threat detection enabled
        group.bench_with_input(
            BenchmarkId::new("with_mate_threat", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_mate_threat_detection();
                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();
                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mate threat detection effectiveness
fn benchmark_mate_threat_detection_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("mate_threat_detection_effectiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Benchmark with mate threat detection disabled
    group.bench_function("without_mate_threat", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_without_mate_threat_detection();
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

            black_box((result, elapsed, nodes, cutoff_rate))
        });
    });

    // Benchmark with mate threat detection enabled
    group.bench_function("with_mate_threat", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_mate_threat_detection();
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
            let mate_threat_rate = stats.mate_threat_detection_rate();

            black_box((result, elapsed, nodes, cutoff_rate, mate_threat_rate))
        });
    });

    group.finish();
}

/// Benchmark mate threat detection with different margins
fn benchmark_mate_threat_margin_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("mate_threat_margin_comparison");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    let margins = vec![100, 300, 500, 1000];

    for margin in margins {
        group.bench_with_input(
            BenchmarkId::new("mate_threat_margin", margin),
            &margin,
            |b, &margin| {
                b.iter(|| {
                    let mut engine = SearchEngine::new(None, 16);
                    let mut config = NullMoveConfig::default();
                    config.enabled = true;
                    config.enable_mate_threat_detection = true;
                    config.mate_threat_margin = margin;
                    engine.update_null_move_config(config).unwrap();
                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();
                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mate threat detection integration with verification search
fn benchmark_mate_threat_with_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("mate_threat_with_verification");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Benchmark with both mate threat and verification enabled
    group.bench_function("mate_threat_and_verification", |b| {
        b.iter(|| {
            let mut engine = SearchEngine::new(None, 16);
            let mut config = NullMoveConfig::default();
            config.enabled = true;
            config.enable_mate_threat_detection = true;
            config.mate_threat_margin = 500;
            config.verification_margin = 200;
            engine.update_null_move_config(config).unwrap();
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
            let mate_threat_rate = stats.mate_threat_detection_rate();
            let verification_rate = stats.verification_cutoff_rate();

            black_box((result, elapsed, nodes, mate_threat_rate, verification_rate))
        });
    });

    group.finish();
}

/// Benchmark comprehensive mate threat detection analysis
fn benchmark_comprehensive_mate_threat_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_mate_threat_analysis");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test configurations
    let configurations = vec![("no_mate_threat", false), ("with_mate_threat", true)];

    for (name, enable_mate_threat) in configurations {
        group.bench_with_input(
            BenchmarkId::new("configuration", name),
            &enable_mate_threat,
            |b, &enable_mate_threat| {
                b.iter(|| {
                    let mut engine = SearchEngine::new(None, 16);
                    let mut config = NullMoveConfig::default();
                    config.enabled = true;
                    config.enable_mate_threat_detection = enable_mate_threat;
                    config.mate_threat_margin = if enable_mate_threat { 500 } else { 500 };
                    engine.update_null_move_config(config).unwrap();
                    engine.reset_null_move_stats();

                    let start = std::time::Instant::now();
                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        5, // Fixed depth for comparison
                        1000,
                    );
                    let elapsed = start.elapsed();

                    let stats = engine.get_null_move_stats().clone();
                    let nodes = engine.get_nodes_searched();
                    let cutoff_rate = stats.cutoff_rate();
                    let mate_threat_rate = stats.mate_threat_detection_rate();

                    black_box((result, elapsed, nodes, cutoff_rate, mate_threat_rate))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_mate_threat_detection_overhead,
    benchmark_mate_threat_detection_effectiveness,
    benchmark_mate_threat_margin_comparison,
    benchmark_mate_threat_with_verification,
    benchmark_comprehensive_mate_threat_analysis
);

criterion_main!(benches);
