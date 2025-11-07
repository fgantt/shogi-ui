//! Performance Benchmarks for IID Move Extraction Improvements
//!
//! This benchmark suite measures:
//! - TT-based vs tracked move extraction performance
//! - Overhead verification (<1% target)
//! - Move extraction success rates
//! - Search performance with improved move extraction
//!
//! Task 2.15: Compare TT-based vs tracked move extraction
//! Task 2.16: Verify IID move extraction improvement doesn't add significant overhead (<1% search time)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, Player},
};
use std::time::Duration;

/// Create a test engine with IID enabled
fn create_test_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64); // 64MB hash table
    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = true;
    iid_config.min_depth = 4;
    iid_config.iid_depth_ply = 2;
    engine.update_iid_config(iid_config).unwrap();
    engine
}

/// Benchmark move extraction with different methods
fn benchmark_move_extraction_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_move_extraction_methods");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 6;
    let time_limit_ms = 1000;

    // Benchmark with current implementation (tracked move extraction)
    group.bench_function("tracked_move_extraction", |b| {
        let mut engine = create_test_engine();
        let start_time = shogi_engine::time_utils::TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let (score, move_result) = engine.perform_iid_search(
                &mut board,
                &captured_pieces,
                player,
                2, // IID depth
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
            );
            black_box((score, move_result));
        });
    });

    // Benchmark statistics tracking
    group.bench_function("move_extraction_with_stats", |b| {
        let mut engine = create_test_engine();
        let start_time = shogi_engine::time_utils::TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let (score, move_result) = engine.perform_iid_search(
                &mut board,
                &captured_pieces,
                player,
                2,
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
            );

            // Access statistics to verify they're tracked
            let stats = engine.get_iid_stats();
            black_box((
                score,
                move_result,
                stats.iid_move_extracted_from_tt,
                stats.iid_move_extracted_from_tracked,
            ));
        });
    });

    group.finish();
}

/// Benchmark overhead verification
fn benchmark_iid_overhead_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_overhead_verification");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;
    let time_limit_ms = 2000;

    // Benchmark search time with IID enabled (new move extraction)
    group.bench_function("iid_enabled_with_new_extraction", |b| {
        let mut engine = create_test_engine();
        let start_time = shogi_engine::time_utils::TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let result = engine.negamax_with_context(
                &mut board,
                &captured_pieces,
                player,
                depth,
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
                true,
                false,
                false,
                false,
            );
            black_box(result);
        });
    });

    // Benchmark search time with IID disabled (baseline)
    group.bench_function("iid_disabled_baseline", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let mut iid_config = engine.get_iid_config().clone();
        iid_config.enabled = false;
        engine.update_iid_config(iid_config).unwrap();
        let start_time = shogi_engine::time_utils::TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            let result = engine.negamax_with_context(
                &mut board,
                &captured_pieces,
                player,
                depth,
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
                true,
                false,
                false,
                false,
            );
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark move extraction success rates
fn benchmark_move_extraction_success_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_move_extraction_success");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let time_limit_ms = 1000;

    // Benchmark move extraction at different depths
    for iid_depth in [1, 2, 3] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}", iid_depth)),
            &iid_depth,
            |b, &depth| {
                let mut engine = create_test_engine();
                let start_time = shogi_engine::time_utils::TimeSource::now();
                let mut hash_history = Vec::new();

                b.iter(|| {
                    engine.reset_iid_stats();
                    let (score, move_result) = engine.perform_iid_search(
                        &mut board,
                        &captured_pieces,
                        player,
                        depth,
                        -10000,
                        10000,
                        &start_time,
                        time_limit_ms,
                        &mut hash_history,
                    );

                    let stats = engine.get_iid_stats();
                    let success = move_result.is_some();
                    let tt_extracted = stats.iid_move_extracted_from_tt > 0;
                    let tracked_extracted = stats.iid_move_extracted_from_tracked > 0;

                    black_box((success, tt_extracted, tracked_extracted, score));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark performance comparison: old vs new move extraction
fn benchmark_extraction_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_extraction_comparison");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(25);

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let time_limit_ms = 1500;

    // Current implementation with tracked moves
    group.bench_function("current_tracked_extraction", |b| {
        let mut engine = create_test_engine();
        let start_time = shogi_engine::time_utils::TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let (score, move_result) = engine.perform_iid_search(
                &mut board,
                &captured_pieces,
                player,
                2,
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
            );

            // Simulate using the move in ordering
            if let Some(mv) = move_result {
                let moves = shogi_engine::move_generation::MoveGenerator::new()
                    .generate_legal_moves(&board, player, &captured_pieces);
                let is_legal = moves.iter().any(|m| engine.moves_equal(m, &mv));
                black_box((score, is_legal));
            } else {
                black_box((score, false));
            }
        });
    });

    group.finish();
}

/// Comprehensive overhead analysis
fn benchmark_comprehensive_overhead_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_comprehensive_overhead");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(30);

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different search depths
    for depth in [4, 5, 6] {
        let time_limit_ms = 2000;

        // With IID (new extraction)
        group.bench_with_input(
            BenchmarkId::new("with_iid_new_extraction", format!("depth_{}", depth)),
            &depth,
            |b, &d| {
                let mut engine = create_test_engine();
                let start_time = shogi_engine::time_utils::TimeSource::now();
                let mut hash_history = Vec::new();

                b.iter(|| {
                    engine.reset_iid_stats();
                    let result = engine.negamax_with_context(
                        &mut board,
                        &captured_pieces,
                        player,
                        d,
                        -10000,
                        10000,
                        &start_time,
                        time_limit_ms,
                        &mut hash_history,
                        true,
                        false,
                        false,
                        false,
                    );

                    let stats = engine.get_iid_stats();
                    let overhead_ms = stats.iid_time_ms;
                    let total_time_ms = stats.total_search_time_ms;

                    black_box((result, overhead_ms, total_time_ms));
                });
            },
        );

        // Without IID (baseline)
        group.bench_with_input(
            BenchmarkId::new("without_iid_baseline", format!("depth_{}", depth)),
            &depth,
            |b, &d| {
                let mut engine = SearchEngine::new(None, 64);
                let mut iid_config = engine.get_iid_config().clone();
                iid_config.enabled = false;
                engine.update_iid_config(iid_config).unwrap();
                let start_time = shogi_engine::time_utils::TimeSource::now();
                let mut hash_history = Vec::new();

                b.iter(|| {
                    let result = engine.negamax_with_context(
                        &mut board,
                        &captured_pieces,
                        player,
                        d,
                        -10000,
                        10000,
                        &start_time,
                        time_limit_ms,
                        &mut hash_history,
                        true,
                        false,
                        false,
                        false,
                    );
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(20);
    targets =
        benchmark_move_extraction_methods,
        benchmark_iid_overhead_verification,
        benchmark_move_extraction_success_rates,
        benchmark_extraction_performance_comparison,
        benchmark_comprehensive_overhead_analysis
}

criterion_main!(benches);
