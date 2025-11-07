//! Performance benchmarks for hash-based vs FEN-based repetition detection (Task 5.14)
//!
//! This benchmark compares:
//! - Hash-based repetition detection (current implementation)
//! - FEN-based repetition detection (old implementation)
//! - Performance impact of repetition detection in search context

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    moves::MoveGenerator,
    search::ShogiHashHandler,
    types::{CapturedPieces, Player},
};

/// Simulate FEN-based repetition detection (old approach)
fn check_fen_repetition(fen: &str, history: &[String]) -> bool {
    history.contains(&fen.to_string())
}

/// Simulate hash-based repetition detection (current approach)
fn check_hash_repetition(hash: u64, hash_handler: &ShogiHashHandler) -> bool {
    hash_handler.get_repetition_state_for_hash(hash).is_draw()
}

fn bench_hash_vs_fen_repetition_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("repetition_detection");

    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    // Prepare test data
    let hash = hash_handler.get_position_hash(&board, player, &captured);
    let fen = board.to_fen(player, &captured);

    // Setup hash history
    for _ in 0..4 {
        hash_handler.add_position_to_history(hash);
    }

    // Setup FEN history
    let mut fen_history: Vec<String> = Vec::new();
    for _ in 0..4 {
        fen_history.push(fen.clone());
    }

    // Benchmark hash-based detection
    group.bench_function("hash_based", |b| {
        b.iter(|| {
            black_box(check_hash_repetition(black_box(hash), &hash_handler));
        });
    });

    // Benchmark FEN-based detection
    group.bench_function("fen_based", |b| {
        b.iter(|| {
            black_box(check_fen_repetition(black_box(&fen), &fen_history));
        });
    });

    group.finish();
}

fn bench_repetition_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("repetition_detection_overhead");

    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    let hash = hash_handler.get_position_hash(&board, player, &captured);

    // Benchmark with no repetition (common case)
    group.bench_function("no_repetition", |b| {
        b.iter(|| {
            let repetition_state = hash_handler.get_repetition_state_for_hash(black_box(hash));
            black_box(repetition_state);
        });
    });

    // Benchmark with repetition (rare case)
    hash_handler.add_position_to_history(hash);
    hash_handler.add_position_to_history(hash);
    hash_handler.add_position_to_history(hash);
    hash_handler.add_position_to_history(hash);

    group.bench_function("with_repetition", |b| {
        b.iter(|| {
            let repetition_state = hash_handler.get_repetition_state_for_hash(black_box(hash));
            black_box(repetition_state);
        });
    });

    group.finish();
}

fn bench_hash_calculation_vs_fen_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_vs_fen_generation");

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let hash_handler = ShogiHashHandler::new_default();

    // Benchmark hash calculation
    group.bench_function("hash_calculation", |b| {
        b.iter(|| {
            let hash = hash_handler.get_position_hash(
                black_box(&board),
                black_box(player),
                black_box(&captured),
            );
            black_box(hash);
        });
    });

    // Benchmark FEN generation
    group.bench_function("fen_generation", |b| {
        b.iter(|| {
            let fen = black_box(&board).to_fen(black_box(player), black_box(&captured));
            black_box(fen);
        });
    });

    group.finish();
}

fn bench_history_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_management");

    let mut hash_handler = ShogiHashHandler::new_default();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;

    let hash = hash_handler.get_position_hash(&board, player, &captured);
    let fen = board.to_fen(player, &captured);

    // Benchmark hash history operations
    group.bench_function("hash_history_add", |b| {
        b.iter(|| {
            hash_handler.add_position_to_history(black_box(hash));
        });
    });

    group.bench_function("hash_history_check", |b| {
        hash_handler.add_position_to_history(hash);
        b.iter(|| {
            let state = hash_handler.get_repetition_state_for_hash(black_box(hash));
            black_box(state);
        });
    });

    // Benchmark FEN history operations
    let mut fen_history: Vec<String> = Vec::new();
    group.bench_function("fen_history_add", |b| {
        b.iter(|| {
            fen_history.push(black_box(fen.clone()));
        });
    });

    group.bench_function("fen_history_check", |b| {
        fen_history.push(fen.clone());
        b.iter(|| {
            let contains = fen_history.contains(&black_box(fen.clone()));
            black_box(contains);
        });
    });

    group.finish();
}

fn bench_repetition_detection_in_search_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("repetition_in_search");

    let mut hash_handler = ShogiHashHandler::new_default();
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    let mut player = Player::Black;
    let move_generator = MoveGenerator::new();

    // Create a sequence of positions
    let mut hashes = Vec::new();
    let mut fens = Vec::new();

    for _ in 0..10 {
        let hash = hash_handler.get_position_hash(&board, player, &captured);
        let fen = board.to_fen(player, &captured);
        hashes.push(hash);
        fens.push(fen);

        // Make a move
        let moves = move_generator.generate_legal_moves(&board, player, &captured);
        if moves.is_empty() {
            break;
        }
        let move_info = board.make_move_with_info(&moves[0]);
        if let Some(ref cp) = move_info.captured_piece {
            captured.add_piece(cp.piece_type, player);
        }
        player = player.opposite();
    }

    // Benchmark hash-based repetition checking in search loop
    group.bench_function("hash_based_search_loop", |b| {
        b.iter(|| {
            for hash in hashes.iter() {
                hash_handler.add_position_to_history(*hash);
                let state = hash_handler.get_repetition_state_for_hash(*hash);
                black_box(state);
            }
        });
    });

    // Benchmark FEN-based repetition checking in search loop
    let mut fen_history: Vec<String> = Vec::new();
    group.bench_function("fen_based_search_loop", |b| {
        b.iter(|| {
            for fen in fens.iter() {
                fen_history.push(fen.clone());
                let contains = fen_history.contains(fen);
                black_box(contains);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hash_vs_fen_repetition_detection,
    bench_repetition_detection_overhead,
    bench_hash_calculation_vs_fen_generation,
    bench_history_management,
    bench_repetition_detection_in_search_context
);
criterion_main!(benches);
