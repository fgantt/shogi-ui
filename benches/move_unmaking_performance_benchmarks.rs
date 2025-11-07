//! Performance benchmarks comparing board cloning vs move unmaking
//!
//! These benchmarks measure the performance improvement from using move unmaking
//! instead of board cloning in the search engine.
//!
//! Target improvement: 10-30% speedup

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use shogi_engine::{
    bitboards::{BitboardBoard, MoveInfo},
    moves::MoveGenerator,
    search::SearchEngine,
    time_utils::TimeSource,
    types::{CapturedPieces, Move, Player},
};

fn create_test_position() -> (BitboardBoard, CapturedPieces, Player) {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    (board, captured, player)
}

/// Benchmark: Making and unmaking a single move
fn bench_single_move_unmake(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_move_unmake");
    group.sampling_mode(SamplingMode::Auto);

    let (mut board, mut captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    if !moves.is_empty() {
        let test_move = &moves[0];

        // Benchmark: make_move_with_info + unmake_move
        group.bench_function("make_unmake", |b| {
            b.iter(|| {
                let mut test_board = board.clone();
                let mut test_captured = captured.clone();
                let move_info = test_board.make_move_with_info(test_move);
                if let Some(ref captured_piece) = move_info.captured_piece {
                    test_captured.add_piece(captured_piece.piece_type, player);
                }
                test_board.unmake_move(&move_info);
                if let Some(ref captured_piece) = move_info.captured_piece {
                    test_captured.remove_piece(captured_piece.piece_type, player);
                }
            });
        });

        // Benchmark: clone + make_move (old method)
        group.bench_function("clone_make", |b| {
            b.iter(|| {
                let mut new_board = board.clone();
                let _new_captured = captured.clone();
                new_board.make_move(test_move);
            });
        });
    }

    group.finish();
}

/// Benchmark: Making and unmaking multiple moves sequentially
fn bench_multiple_moves_unmake(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_moves_unmake");
    group.sampling_mode(SamplingMode::Auto);

    let (mut board, mut captured, mut player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    if moves.len() >= 5 {
        let test_moves: Vec<&Move> = moves.iter().take(5).collect();

        // Benchmark: make_unmake method
        group.bench_function("make_unmake_5_moves", |b| {
            b.iter(|| {
                let mut test_board = board.clone();
                let mut move_history: Vec<MoveInfo> = Vec::new();
                let mut current_player = player;
                let mut current_captured = captured.clone();

                for move_ in &test_moves {
                    let move_info = test_board.make_move_with_info(move_);
                    if let Some(ref captured_piece) = move_info.captured_piece {
                        current_captured.add_piece(captured_piece.piece_type, current_player);
                    }
                    move_history.push(move_info);
                    current_player = current_player.opposite();
                }

                // Unmake in reverse
                while let Some(move_info) = move_history.pop() {
                    current_player = current_player.opposite();
                    if let Some(ref captured_piece) = move_info.captured_piece {
                        current_captured.remove_piece(captured_piece.piece_type, current_player);
                    }
                    test_board.unmake_move(&move_info);
                }
            });
        });

        // Benchmark: clone method
        group.bench_function("clone_5_moves", |b| {
            b.iter(|| {
                for _move_ in &test_moves {
                    let _new_board = board.clone();
                    let _new_captured = captured.clone();
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Search with move unmaking vs cloning
fn bench_search_with_unmaking(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance");
    group.sampling_mode(SamplingMode::Auto);

    let (mut board, captured, player) = create_test_position();
    let mut engine = SearchEngine::new(None, 16);

    // Benchmark search at different depths
    for depth in [1, 2, 3] {
        group.bench_with_input(
            BenchmarkId::new("search_with_unmaking", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut test_board = board.clone();
                    let _result = engine.search_at_depth(
                        &mut test_board,
                        &captured,
                        player,
                        depth,
                        1000,
                        -10000,
                        10000,
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Move unmaking for different move types
fn bench_move_types_unmake(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_types_unmake");
    group.sampling_mode(SamplingMode::Auto);

    let (mut board, mut captured, player) = create_test_position();
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured);

    // Normal moves
    let normal_move = moves
        .iter()
        .find(|m| !m.is_capture && !m.is_promotion && m.from.is_some());
    if let Some(move_) = normal_move {
        group.bench_function("normal_move", |b| {
            b.iter(|| {
                let mut test_board = board.clone();
                let move_info = test_board.make_move_with_info(move_);
                test_board.unmake_move(&move_info);
            });
        });
    }

    // Capture moves
    let capture_move = moves.iter().find(|m| m.is_capture);
    if let Some(move_) = capture_move {
        group.bench_function("capture_move", |b| {
            b.iter(|| {
                let mut test_board = board.clone();
                let mut test_captured = captured.clone();
                let move_info = test_board.make_move_with_info(move_);
                if let Some(ref captured_piece) = move_info.captured_piece {
                    test_captured.add_piece(captured_piece.piece_type, player);
                }
                test_board.unmake_move(&move_info);
                if let Some(ref captured_piece) = move_info.captured_piece {
                    test_captured.remove_piece(captured_piece.piece_type, player);
                }
            });
        });
    }

    // Promotion moves
    let promotion_move = moves.iter().find(|m| m.is_promotion);
    if let Some(move_) = promotion_move {
        group.bench_function("promotion_move", |b| {
            b.iter(|| {
                let mut test_board = board.clone();
                let move_info = test_board.make_move_with_info(move_);
                test_board.unmake_move(&move_info);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_move_unmake,
    bench_multiple_moves_unmake,
    bench_search_with_unmaking,
    bench_move_types_unmake
);
criterion_main!(benches);
