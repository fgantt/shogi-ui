//! Performance benchmarks for endgame detection optimization
//!
//! This benchmark suite measures the performance improvement from optimizing
//! piece counting to use bitboard popcount instead of iterating through all squares.
//!
//! Expected results:
//! - Optimized counting should be 50-80% faster than iterative counting
//! - Endgame detection overhead should be minimal (<1% of search time)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with endgame detection enabled
fn create_test_engine_with_endgame_detection() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.enabled = true;
    config.enable_endgame_detection = true;
    config.max_pieces_threshold = 12;
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Benchmark endgame detection overhead with optimized piece counting
fn benchmark_endgame_detection_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("endgame_detection_performance");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("search_with_endgame_detection", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_endgame_detection();

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

/// Benchmark piece counting directly using bitboard vs iteration
fn benchmark_piece_counting_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_counting_methods");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    let board = BitboardBoard::new();

    // Benchmark bitboard popcount method (optimized)
    group.bench_function("bitboard_popcount", |b| {
        b.iter(|| {
            let occupied = board.get_occupied_bitboard();
            black_box(occupied.count_ones() as u8)
        });
    });

    // Benchmark iterative method (old approach)
    group.bench_function("iterative_counting", |b| {
        b.iter(|| {
            let mut count = 0u8;
            for row in 0..9 {
                for col in 0..9 {
                    let pos = shogi_engine::types::Position::new(row, col);
                    if board.is_square_occupied(pos) {
                        count += 1;
                    }
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

/// Benchmark endgame detection overhead in search context
fn benchmark_endgame_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("endgame_detection_overhead");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Benchmark with endgame detection enabled
    group.bench_function("with_endgame_detection", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_endgame_detection();
            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats();
            black_box(stats.disabled_endgame)
        });
    });

    // Benchmark with endgame detection disabled for comparison
    group.bench_function("without_endgame_detection", |b| {
        b.iter(|| {
            let mut engine = SearchEngine::new(None, 16);
            let mut config = NullMoveConfig::default();
            config.enabled = true;
            config.enable_endgame_detection = false;
            engine.update_null_move_config(config).unwrap();
            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats();
            black_box(stats.disabled_endgame)
        });
    });

    group.finish();
}

/// Benchmark piece counting at different board states
fn benchmark_piece_counting_different_states(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_counting_different_states");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Test with initial position (40 pieces)
    let board_full = BitboardBoard::new();

    group.bench_function("full_board_40_pieces", |b| {
        b.iter(|| {
            let occupied = board_full.get_occupied_bitboard();
            black_box(occupied.count_ones() as u8)
        });
    });

    // Note: Creating sparse boards for testing would require board manipulation
    // For now, we test with the full board which represents the common case
    // The optimization benefit will be even greater for sparse boards

    group.finish();
}

/// Benchmark search performance with optimized endgame detection
fn benchmark_search_performance_with_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance_with_optimization");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths to see cumulative effect
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("search_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_endgame_detection();
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

    group.finish();
}

criterion_group!(
    benches,
    benchmark_endgame_detection_performance,
    benchmark_piece_counting_methods,
    benchmark_endgame_detection_overhead,
    benchmark_piece_counting_different_states,
    benchmark_search_performance_with_optimization
);

criterion_main!(benches);
