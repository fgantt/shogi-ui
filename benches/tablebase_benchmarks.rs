// Benchmarks for tablebase probes and move ordering (Task 4.0 suite)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::tablebase::MicroTablebase;
use shogi_engine::types::{CapturedPieces, Move, Piece, PieceType, Player, Position};

fn build_gold_position() -> (BitboardBoard, CapturedPieces, Player) {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(2, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(1, 3),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    (board, CapturedPieces::new(), Player::Black)
}

fn build_silver_position() -> (BitboardBoard, CapturedPieces, Player) {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(2, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(1, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 3),
    );
    (board, CapturedPieces::new(), Player::Black)
}

fn build_rook_position() -> (BitboardBoard, CapturedPieces, Player) {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(2, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(0, 2),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    (board, CapturedPieces::new(), Player::Black)
}

fn build_non_tablebase_position() -> (BitboardBoard, CapturedPieces, Player) {
    (BitboardBoard::new(), CapturedPieces::new(), Player::Black)
}

fn benchmark_tablebase_probe_cache_hit(c: &mut Criterion) {
    let mut tablebase = MicroTablebase::new();
    let (board, captured, player) = build_gold_position();
    let _ = tablebase.probe(&board, player, &captured);

    c.bench_function("tablebase_probe_cache_hit", |b| {
        b.iter(|| black_box(tablebase.probe(black_box(&board), player, black_box(&captured))))
    });
}

fn benchmark_tablebase_probe_cache_miss(c: &mut Criterion) {
    let mut tablebase = MicroTablebase::new();
    let (board, captured, player) = build_non_tablebase_position();

    c.bench_function("tablebase_probe_cache_miss", |b| {
        b.iter(|| black_box(tablebase.probe(black_box(&board), player, black_box(&captured))))
    });
}

fn benchmark_solver_execution_times(c: &mut Criterion) {
    let mut tablebase = MicroTablebase::new();
    let scenarios = [
        ("gold_solver", build_gold_position()),
        ("silver_solver", build_silver_position()),
        ("rook_solver", build_rook_position()),
    ];

    let mut group = c.benchmark_group("tablebase_solver_execution");
    for (name, (board, captured, player)) in scenarios {
        group.bench_function(BenchmarkId::new("probe", name), |b| {
            b.iter(|| black_box(tablebase.probe(black_box(&board), player, black_box(&captured))))
        });
    }
    group.finish();
}

fn benchmark_tablebase_move_cache(c: &mut Criterion) {
    let (board, _, _) = build_gold_position();
    let mut engine = SearchEngine::new(None, 4);
    let mut board_template = board.clone();

    let winning_move = Move::new_move(
        Position::new(1, 3),
        Position::new(0, 4),
        PieceType::Gold,
        Player::Black,
        false,
    );
    let alt_move = Move::new_move(
        Position::new(2, 4),
        Position::new(2, 3),
        PieceType::King,
        Player::Black,
        false,
    );
    let moves = vec![winning_move, alt_move];

    c.bench_function("tablebase_move_cache", |b| {
        b.iter(|| {
            let mut local_board = board_template.clone();
            black_box(engine.sort_moves_with_pruning_awareness(
                &moves,
                &mut local_board,
                None,
                None,
                None,
                None,
            ))
        })
    });
}

criterion_group!(
    tablebase_benches,
    benchmark_tablebase_probe_cache_hit,
    benchmark_tablebase_probe_cache_miss,
    benchmark_solver_execution_times,
    benchmark_tablebase_move_cache
);
criterion_main!(tablebase_benches);
