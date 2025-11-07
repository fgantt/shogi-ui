//! Performance benchmarks for piece-square tables
//!
//! This benchmark suite measures the performance of:
//! - Table creation and initialization
//! - Value lookups for all piece types
//! - Table coordinate calculations
//! - Symmetry handling
//! - Memory usage patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::evaluation::piece_square_tables::PieceSquareTables;
use shogi_engine::types::*;

/// Benchmark table creation
fn benchmark_table_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(PieceSquareTables::new());
        });
    });

    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(PieceSquareTables::default());
        });
    });

    group.finish();
}

/// Benchmark value lookups for basic pieces
fn benchmark_basic_piece_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_piece_lookups");

    let tables = PieceSquareTables::new();
    let pos = Position::new(4, 4); // Center square

    let piece_types = [
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::King,
    ];

    for piece_type in piece_types {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", piece_type)),
            &piece_type,
            |b, &pt| {
                b.iter(|| {
                    black_box(tables.get_value(pt, pos, Player::Black));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark value lookups for promoted pieces
fn benchmark_promoted_piece_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("promoted_piece_lookups");

    let tables = PieceSquareTables::new();
    let pos = Position::new(4, 4);

    let piece_types = [
        PieceType::PromotedPawn,
        PieceType::PromotedLance,
        PieceType::PromotedKnight,
        PieceType::PromotedSilver,
        PieceType::PromotedBishop,
        PieceType::PromotedRook,
    ];

    for piece_type in piece_types {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", piece_type)),
            &piece_type,
            |b, &pt| {
                b.iter(|| {
                    black_box(tables.get_value(pt, pos, Player::Black));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark lookups at different positions
fn benchmark_position_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_variations");

    let tables = PieceSquareTables::new();

    let positions = [
        ("center", Position::new(4, 4)),
        ("corner_top_left", Position::new(0, 0)),
        ("corner_top_right", Position::new(0, 8)),
        ("corner_bottom_left", Position::new(8, 0)),
        ("corner_bottom_right", Position::new(8, 8)),
        ("edge_top", Position::new(0, 4)),
        ("edge_bottom", Position::new(8, 4)),
        ("edge_left", Position::new(4, 0)),
        ("edge_right", Position::new(4, 8)),
    ];

    for (name, pos) in positions {
        group.bench_with_input(BenchmarkId::from_parameter(name), &pos, |b, &p| {
            b.iter(|| {
                black_box(tables.get_value(PieceType::Rook, p, Player::Black));
            });
        });
    }

    group.finish();
}

/// Benchmark player symmetry
fn benchmark_symmetry(c: &mut Criterion) {
    let mut group = c.benchmark_group("symmetry");

    let tables = PieceSquareTables::new();
    let pos = Position::new(4, 4);

    group.bench_function("black_player", |b| {
        b.iter(|| {
            black_box(tables.get_value(PieceType::Rook, pos, Player::Black));
        });
    });

    group.bench_function("white_player", |b| {
        b.iter(|| {
            black_box(tables.get_value(PieceType::Rook, pos, Player::White));
        });
    });

    group.bench_function("both_players", |b| {
        b.iter(|| {
            let black = tables.get_value(PieceType::Rook, pos, Player::Black);
            let white = tables.get_value(PieceType::Rook, pos, Player::White);
            black_box((black, white));
        });
    });

    group.finish();
}

/// Benchmark coordinate calculations
fn benchmark_table_coords(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_coords");

    let tables = PieceSquareTables::new();
    let pos = Position::new(4, 4);

    group.bench_function("black_coords", |b| {
        b.iter(|| {
            black_box(tables.get_table_coords(pos, Player::Black));
        });
    });

    group.bench_function("white_coords", |b| {
        b.iter(|| {
            black_box(tables.get_table_coords(pos, Player::White));
        });
    });

    group.finish();
}

/// Benchmark full board evaluation
fn benchmark_full_board(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_board");

    let tables = PieceSquareTables::new();

    group.bench_function("evaluate_all_squares", |b| {
        b.iter(|| {
            let mut total = TaperedScore::default();
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    total += tables.get_value(PieceType::Rook, pos, Player::Black);
                }
            }
            black_box(total);
        });
    });

    group.bench_function("evaluate_all_pieces_all_squares", |b| {
        let piece_types = [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ];

        b.iter(|| {
            let mut total = TaperedScore::default();
            for &pt in &piece_types {
                for row in 0..9 {
                    for col in 0..9 {
                        let pos = Position::new(row, col);
                        total += tables.get_value(pt, pos, Player::Black);
                    }
                }
            }
            black_box(total);
        });
    });

    group.finish();
}

/// Benchmark repeated lookups (cache effects)
fn benchmark_cache_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effects");

    let tables = PieceSquareTables::new();
    let pos = Position::new(4, 4);

    group.bench_function("same_lookup_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(tables.get_value(PieceType::Rook, pos, Player::Black));
            }
        });
    });

    group.bench_function("different_pieces_1000x", |b| {
        let piece_types = [
            PieceType::Pawn,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
        ];

        b.iter(|| {
            for i in 0..1000 {
                let pt = piece_types[i % 4];
                black_box(tables.get_value(pt, pos, Player::Black));
            }
        });
    });

    group.bench_function("different_positions_1000x", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let row = (i % 9) as u8;
                let col = ((i / 9) % 9) as u8;
                let pos = Position::new(row, col);
                black_box(tables.get_value(PieceType::Rook, pos, Player::Black));
            }
        });
    });

    group.finish();
}

/// Benchmark table access patterns
fn benchmark_access_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("access_patterns");

    let tables = PieceSquareTables::new();

    group.bench_function("sequential_rows", |b| {
        b.iter(|| {
            let mut total = TaperedScore::default();
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    total += tables.get_value(PieceType::Rook, pos, Player::Black);
                }
            }
            black_box(total);
        });
    });

    group.bench_function("sequential_cols", |b| {
        b.iter(|| {
            let mut total = TaperedScore::default();
            for col in 0..9 {
                for row in 0..9 {
                    let pos = Position::new(row, col);
                    total += tables.get_value(PieceType::Rook, pos, Player::Black);
                }
            }
            black_box(total);
        });
    });

    group.bench_function("random_access", |b| {
        let positions = [
            Position::new(3, 7),
            Position::new(1, 2),
            Position::new(8, 5),
            Position::new(0, 4),
            Position::new(6, 1),
            Position::new(2, 8),
            Position::new(5, 3),
            Position::new(7, 6),
            Position::new(4, 0),
        ];

        b.iter(|| {
            let mut total = TaperedScore::default();
            for &pos in &positions {
                total += tables.get_value(PieceType::Rook, pos, Player::Black);
            }
            black_box(total);
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("create_many_tables", |b| {
        b.iter(|| {
            let tables: Vec<PieceSquareTables> =
                (0..100).map(|_| PieceSquareTables::new()).collect();
            black_box(tables);
        });
    });

    group.bench_function("clone_table", |b| {
        let tables = PieceSquareTables::new();
        b.iter(|| {
            black_box(tables.clone());
        });
    });

    group.finish();
}

/// Benchmark complete workflow
fn benchmark_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");

    group.bench_function("evaluate_typical_position", |b| {
        let tables = PieceSquareTables::new();

        // Simulate a typical mid-game position
        let pieces = vec![
            (PieceType::Rook, Position::new(0, 7), Player::Black),
            (PieceType::Bishop, Position::new(1, 7), Player::Black),
            (PieceType::Gold, Position::new(0, 5), Player::Black),
            (PieceType::Silver, Position::new(1, 6), Player::Black),
            (PieceType::Pawn, Position::new(6, 7), Player::Black),
            (PieceType::Rook, Position::new(8, 1), Player::White),
            (PieceType::Bishop, Position::new(7, 1), Player::White),
            (PieceType::Gold, Position::new(8, 3), Player::White),
        ];

        b.iter(|| {
            let mut total = TaperedScore::default();
            for (pt, pos, player) in &pieces {
                let score = tables.get_value(*pt, *pos, *player);
                if *player == Player::Black {
                    total += score;
                } else {
                    total -= score;
                }
            }
            black_box(total);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_table_creation,
    benchmark_basic_piece_lookups,
    benchmark_promoted_piece_lookups,
    benchmark_position_variations,
    benchmark_symmetry,
    benchmark_table_coords,
    benchmark_full_board,
    benchmark_cache_effects,
    benchmark_access_patterns,
    benchmark_memory_patterns,
    benchmark_complete_workflow,
);

criterion_main!(benches);
