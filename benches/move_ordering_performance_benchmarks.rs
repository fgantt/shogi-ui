//! Performance benchmarks for move ordering optimization (Task 6.1, 6.7)
//!
//! This benchmark measures:
//! - Move ordering overhead at different depths
//! - Cache effectiveness for move ordering results
//! - TT integration impact on move ordering
//! - Comparison of different move ordering strategies

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    moves::MoveGenerator,
    search::search_engine::SearchEngine,
    std::sync::atomic::AtomicBool,
    std::sync::Arc,
    types::{CapturedPieces, Player},
};

fn create_test_position() -> (BitboardBoard, CapturedPieces, Player) {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    (board, captured, player)
}

/// Benchmark: Move ordering overhead at different depths
fn bench_move_ordering_overhead_by_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_overhead_by_depth");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);
    let (board, captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    if !moves.is_empty() {
        for depth in [1, 3, 5, 7].iter() {
            group.bench_with_input(
                BenchmarkId::new("order_moves", depth),
                depth,
                |b, &depth| {
                    b.iter(|| {
                        let mut test_board = board.clone();
                        let test_captured = captured.clone();
                        let sorted = engine.order_moves_for_negamax(
                            black_box(&moves),
                            black_box(&test_board),
                            black_box(&test_captured),
                            black_box(player),
                            black_box(*depth),
                            black_box(0),
                            black_box(0),
                        );
                        black_box(sorted);
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Move ordering with vs without caching
fn bench_move_ordering_with_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_caching");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);
    let (board, captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    if !moves.is_empty() {
        // Benchmark: Move ordering (first call - no cache)
        group.bench_function("first_call_no_cache", |b| {
            b.iter(|| {
                let mut test_engine = SearchEngine::new(None, 64);
                let test_board = board.clone();
                let test_captured = captured.clone();
                let sorted = test_engine.order_moves_for_negamax(
                    black_box(&moves),
                    black_box(&test_board),
                    black_box(&test_captured),
                    black_box(player),
                    black_box(3),
                    black_box(0),
                    black_box(0),
                );
                black_box(sorted);
            });
        });

        // Benchmark: Move ordering (subsequent calls - with cache)
        group.bench_function("subsequent_calls_with_cache", |b| {
            // Warm up cache
            engine.order_moves_for_negamax(&moves, &board, &captured, player, 3, 0, 0);

            b.iter(|| {
                let test_board = board.clone();
                let test_captured = captured.clone();
                let sorted = engine.order_moves_for_negamax(
                    black_box(&moves),
                    black_box(&test_board),
                    black_box(&test_captured),
                    black_box(player),
                    black_box(3),
                    black_box(0),
                    black_box(0),
                );
                black_box(sorted);
            });
        });
    }

    group.finish();
}

/// Benchmark: Move ordering with TT integration
fn bench_move_ordering_with_tt(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_tt_integration");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);
    let (board, captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    if !moves.is_empty() {
        // Benchmark: Move ordering without TT (empty TT)
        group.bench_function("without_tt", |b| {
            let mut test_engine = SearchEngine::new(None, 64);
            b.iter(|| {
                let test_board = board.clone();
                let test_captured = captured.clone();
                let sorted = test_engine.order_moves_for_negamax(
                    black_box(&moves),
                    black_box(&test_board),
                    black_box(&test_captured),
                    black_box(player),
                    black_box(3),
                    black_box(0),
                    black_box(0),
                );
                black_box(sorted);
            });
        });

        // Benchmark: Move ordering with TT (populated TT)
        group.bench_function("with_tt", |b| {
            // Populate TT by doing a search
            let mut test_board = board.clone();
            let test_captured = captured.clone();
            let _ = engine.find_best_move(&mut test_board, &test_captured, player, 3, 1000);

            b.iter(|| {
                let test_board = board.clone();
                let test_captured = captured.clone();
                let sorted = engine.order_moves_for_negamax(
                    black_box(&moves),
                    black_box(&test_board),
                    black_box(&test_captured),
                    black_box(player),
                    black_box(3),
                    black_box(0),
                    black_box(0),
                );
                black_box(sorted);
            });
        });
    }

    group.finish();
}

/// Benchmark: Move ordering effectiveness (cutoff rate)
fn bench_move_ordering_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_effectiveness");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);
    let (board, captured, player) = create_test_position();

    // Benchmark: Search with move ordering
    group.bench_function("with_ordering", |b| {
        b.iter(|| {
            let mut test_board = board.clone();
            let test_captured = captured.clone();
            let _ = engine.find_best_move(
                black_box(&mut test_board),
                black_box(&test_captured),
                black_box(player),
                black_box(3),
                black_box(1000),
            );
        });
    });

    group.finish();
}

/// Benchmark: Move ordering for different move counts
fn bench_move_ordering_by_move_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_by_move_count");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut engine = SearchEngine::new(Some(stop_flag), 64);
    let (board, captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let all_moves = move_generator.generate_legal_moves(&board, player, &captured);

    if !all_moves.is_empty() {
        for move_count in [5, 10, 20, 30].iter() {
            let moves: Vec<_> = all_moves
                .iter()
                .take(*move_count.min(&all_moves.len()))
                .cloned()
                .collect();

            group.bench_with_input(
                BenchmarkId::new("order_moves", move_count),
                move_count,
                |b, _| {
                    b.iter(|| {
                        let test_board = board.clone();
                        let test_captured = captured.clone();
                        let sorted = engine.order_moves_for_negamax(
                            black_box(&moves),
                            black_box(&test_board),
                            black_box(&test_captured),
                            black_box(player),
                            black_box(3),
                            black_box(0),
                            black_box(0),
                        );
                        black_box(sorted);
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_move_ordering_overhead_by_depth,
    bench_move_ordering_with_caching,
    bench_move_ordering_with_tt,
    bench_move_ordering_effectiveness,
    bench_move_ordering_by_move_count
);
criterion_main!(benches);
