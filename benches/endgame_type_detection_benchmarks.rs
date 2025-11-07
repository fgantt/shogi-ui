//! Performance benchmarks for enhanced endgame type detection
//!
//! This benchmark suite measures the performance overhead and effectiveness of
//! enhanced endgame type detection for null move pruning.
//!
//! Expected results:
//! - Enhanced endgame detection should have minimal overhead (<10% increase)
//! - Enhanced detection should improve zugzwang detection accuracy
//! - Performance impact should be acceptable for the added intelligence

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with basic endgame detection
fn create_test_engine_with_basic_endgame() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.enabled = true;
    config.enable_endgame_detection = true;
    config.enable_endgame_type_detection = false; // Basic detection
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Create a test engine with enhanced endgame type detection
fn create_test_engine_with_enhanced_endgame() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.enabled = true;
    config.enable_endgame_detection = true;
    config.enable_endgame_type_detection = true; // Enhanced detection
    config.material_endgame_threshold = 12;
    config.king_activity_threshold = 8;
    config.zugzwang_threshold = 6;
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Benchmark endgame type detection overhead
fn benchmark_endgame_type_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("endgame_type_detection_overhead");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths
    for depth in [3, 4, 5, 6] {
        // Benchmark with basic endgame detection
        group.bench_with_input(
            BenchmarkId::new("basic_endgame", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_basic_endgame();
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

        // Benchmark with enhanced endgame type detection
        group.bench_with_input(
            BenchmarkId::new("enhanced_endgame", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_enhanced_endgame();
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

/// Benchmark endgame type detection effectiveness
fn benchmark_endgame_type_detection_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("endgame_type_detection_effectiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Benchmark with basic endgame detection
    group.bench_function("basic_endgame", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_basic_endgame();
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

    // Benchmark with enhanced endgame type detection
    group.bench_function("enhanced_endgame", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_enhanced_endgame();
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

    group.finish();
}

/// Benchmark endgame type detection with different thresholds
fn benchmark_endgame_type_thresholds(c: &mut Criterion) {
    let mut group = c.benchmark_group("endgame_type_thresholds");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    let threshold_configs = vec![
        ("conservative", 15, 10, 8),
        ("default", 12, 8, 6),
        ("aggressive", 10, 6, 4),
    ];

    for (name, material, king_activity, zugzwang) in threshold_configs {
        group.bench_with_input(
            BenchmarkId::new("thresholds", name),
            &(material, king_activity, zugzwang),
            |b, &(material, king_activity, zugzwang)| {
                b.iter(|| {
                    let mut engine = SearchEngine::new(None, 16);
                    let mut config = NullMoveConfig::default();
                    config.enabled = true;
                    config.enable_endgame_detection = true;
                    config.enable_endgame_type_detection = true;
                    config.material_endgame_threshold = material;
                    config.king_activity_threshold = king_activity;
                    config.zugzwang_threshold = zugzwang;
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

/// Benchmark comprehensive endgame type detection analysis
fn benchmark_comprehensive_endgame_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_endgame_analysis");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test configurations
    let configurations = vec![("basic", false), ("enhanced", true)];

    for (name, enable_enhanced) in configurations {
        group.bench_with_input(
            BenchmarkId::new("configuration", name),
            &enable_enhanced,
            |b, &enable_enhanced| {
                b.iter(|| {
                    let mut engine = if enable_enhanced {
                        create_test_engine_with_enhanced_endgame()
                    } else {
                        create_test_engine_with_basic_endgame()
                    };
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

                    black_box((result, elapsed, nodes, cutoff_rate))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_endgame_type_detection_overhead,
    benchmark_endgame_type_detection_effectiveness,
    benchmark_endgame_type_thresholds,
    benchmark_comprehensive_endgame_analysis
);

criterion_main!(benches);
