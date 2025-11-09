//! Performance benchmarks for material evaluation system
//!
//! This benchmark suite measures the performance of:
//! - Material evaluation for various positions
//! - Hand piece evaluation
//! - Material balance calculation
//! - Piece value lookups
//! - Material counting operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::material::{MaterialEvaluationConfig, MaterialEvaluator};
use shogi_engine::types::*;

fn scenario_board_heavy() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    for (piece_type, player, row, col) in [
        (PieceType::Rook, Player::Black, 4, 4),
        (PieceType::Bishop, Player::Black, 4, 5),
        (PieceType::Silver, Player::Black, 6, 3),
        (PieceType::Gold, Player::Black, 7, 4),
        (PieceType::Rook, Player::White, 3, 4),
        (PieceType::Bishop, Player::White, 2, 5),
        (PieceType::Silver, Player::White, 1, 3),
        (PieceType::Gold, Player::White, 0, 4),
        (PieceType::PromotedPawn, Player::Black, 5, 4),
        (PieceType::PromotedPawn, Player::White, 3, 5),
    ] {
        board.place_piece(Piece::new(piece_type, player), Position::new(row, col));
    }
    (board, CapturedPieces::new())
}

fn scenario_hand_heavy() -> (BitboardBoard, CapturedPieces) {
    let (board, mut captured) = scenario_board_heavy();
    for piece in [
        (PieceType::Pawn, Player::Black),
        (PieceType::Pawn, Player::Black),
        (PieceType::Silver, Player::Black),
        (PieceType::Knight, Player::Black),
        (PieceType::Pawn, Player::White),
        (PieceType::Gold, Player::White),
        (PieceType::Rook, Player::White),
    ] {
        captured.add_piece(piece.0, piece.1);
    }
    (board, captured)
}

fn scenario_promoted() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    for (piece_type, player, row, col) in [
        (PieceType::PromotedLance, Player::Black, 5, 2),
        (PieceType::PromotedKnight, Player::Black, 5, 6),
        (PieceType::PromotedSilver, Player::Black, 6, 5),
        (PieceType::PromotedRook, Player::Black, 4, 4),
        (PieceType::PromotedBishop, Player::Black, 6, 4),
        (PieceType::PromotedLance, Player::White, 3, 2),
        (PieceType::PromotedKnight, Player::White, 3, 6),
        (PieceType::PromotedSilver, Player::White, 2, 5),
        (PieceType::PromotedRook, Player::White, 4, 5),
        (PieceType::PromotedBishop, Player::White, 2, 4),
    ] {
        board.place_piece(Piece::new(piece_type, player), Position::new(row, col));
    }
    (board, CapturedPieces::new())
}

fn baseline_board_scan(board: &BitboardBoard) -> i32 {
    let mut total = 0;
    for row in 0..9 {
        for col in 0..9 {
            if board.get_piece(Position::new(row, col)).is_some() {
                total += 1;
            }
        }
    }
    total
}

/// Benchmark material evaluator creation
fn benchmark_evaluator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator_creation");

    group.bench_function("new_default", |b| {
        b.iter(|| {
            black_box(MaterialEvaluator::new());
        });
    });

    group.bench_function("with_config", |b| {
        let config = MaterialEvaluationConfig::default();
        b.iter(|| {
            black_box(MaterialEvaluator::with_config(config.clone()));
        });
    });

    group.finish();
}

/// Benchmark piece value lookups
fn benchmark_piece_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_values");

    let evaluator = MaterialEvaluator::new();

    group.bench_function("get_pawn_value", |b| {
        b.iter(|| {
            black_box(evaluator.get_piece_value(PieceType::Pawn));
        });
    });

    group.bench_function("get_rook_value", |b| {
        b.iter(|| {
            black_box(evaluator.get_piece_value(PieceType::Rook));
        });
    });

    group.bench_function("get_promoted_rook_value", |b| {
        b.iter(|| {
            black_box(evaluator.get_piece_value(PieceType::PromotedRook));
        });
    });

    group.bench_function("get_hand_pawn_value", |b| {
        b.iter(|| {
            black_box(evaluator.get_hand_piece_value(PieceType::Pawn));
        });
    });

    // Benchmark all piece types
    for piece_type in [
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::King,
    ] {
        group.bench_with_input(
            BenchmarkId::new("all_pieces", format!("{:?}", piece_type)),
            &piece_type,
            |b, &pt| {
                b.iter(|| {
                    black_box(evaluator.get_piece_value(pt));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark material evaluation
fn benchmark_material_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_evaluation");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("evaluate_starting_position", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("evaluate_both_players", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
            let white = evaluator.evaluate_material(&board, Player::White, &captured_pieces);
            black_box((black, white));
        });
    });

    group.finish();
}

/// Benchmark material evaluation with hand pieces
fn benchmark_hand_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hand_evaluation");

    let board = BitboardBoard::new();

    // Create various captured piece scenarios
    let captured_empty = CapturedPieces::new();

    let mut captured_one_pawn = CapturedPieces::new();
    captured_one_pawn.add_piece(PieceType::Pawn, Player::Black);

    let mut captured_multiple = CapturedPieces::new();
    captured_multiple.add_piece(PieceType::Pawn, Player::Black);
    captured_multiple.add_piece(PieceType::Silver, Player::Black);
    captured_multiple.add_piece(PieceType::Rook, Player::Black);

    group.bench_function("no_captures", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_empty));
        });
    });

    group.bench_function("one_capture", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_one_pawn));
        });
    });

    group.bench_function("multiple_captures", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_multiple));
        });
    });

    group.finish();
}

fn benchmark_material_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_scenarios");

    let (board_heavy, captured_none) = scenario_board_heavy();
    group.bench_function("board_heavy_default", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board_heavy, Player::Black, &captured_none));
        });
    });

    let (board_hand, captured_hand) = scenario_hand_heavy();
    group.bench_function("hand_heavy", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board_hand, Player::Black, &captured_hand));
        });
    });

    let (board_promoted, captured_promoted) = scenario_promoted();
    group.bench_function("promoted_mix", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(
                &board_promoted,
                Player::Black,
                &captured_promoted,
            ));
        });
    });

    group.finish();
}

/// Benchmark material balance calculation
fn benchmark_material_balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_balance");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("calculate_balance", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.calculate_material_balance(
                &board,
                &captured_pieces,
                Player::Black,
            ));
        });
    });

    group.finish();
}

/// Benchmark material counting
fn benchmark_material_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_counting");

    let evaluator = MaterialEvaluator::new();
    let board = BitboardBoard::new();

    group.bench_function("count_total_material", |b| {
        b.iter(|| {
            black_box(evaluator.count_total_material(&board));
        });
    });

    group.bench_function("count_pawns", |b| {
        b.iter(|| {
            black_box(evaluator.count_material_by_type(&board, PieceType::Pawn, Player::Black));
        });
    });

    group.bench_function("count_rooks", |b| {
        b.iter(|| {
            black_box(evaluator.count_material_by_type(&board, PieceType::Rook, Player::Black));
        });
    });

    // Benchmark counting all piece types
    for piece_type in [
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
    ] {
        group.bench_with_input(
            BenchmarkId::new("count_by_type", format!("{:?}", piece_type)),
            &piece_type,
            |b, &pt| {
                b.iter(|| {
                    black_box(evaluator.count_material_by_type(&board, pt, Player::Black));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_ablation(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_ablation");
    let (board, captured) = scenario_board_heavy();

    group.bench_function("full_material", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured));
        });
    });

    group.bench_function("baseline_scan_only", |b| {
        b.iter(|| {
            black_box(baseline_board_scan(&board));
        });
    });

    group.finish();
}

/// Benchmark configuration variations
fn benchmark_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Pawn, Player::Black);

    let configs = vec![
        ("default", MaterialEvaluationConfig::default()),
        (
            "no_hand_pieces",
            MaterialEvaluationConfig {
                include_hand_pieces: false,
                ..MaterialEvaluationConfig::default()
            },
        ),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &config, |b, config| {
            let mut evaluator = MaterialEvaluator::with_config(config.clone());
            b.iter(|| {
                black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
            });
        });
    }

    #[cfg(feature = "material_fast_loop")]
    {
        group.bench_function("fast_loop", |b| {
            let mut config = MaterialEvaluationConfig::default();
            config.enable_fast_loop = true;
            let mut evaluator = MaterialEvaluator::with_config(config);
            b.iter(|| {
                black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
            });
        });
    }

    group.finish();
}

#[cfg(feature = "material_fast_loop")]
fn benchmark_fast_loop_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_fast_loop");
    let (board, captured) = scenario_hand_heavy();

    group.bench_function("legacy_loop", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured));
        });
    });

    group.bench_function("fast_loop_enabled", |b| {
        let mut config = MaterialEvaluationConfig::default();
        config.enable_fast_loop = true;
        let mut evaluator = MaterialEvaluator::with_config(config);
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured));
        });
    });

    group.finish();
}

#[cfg(not(feature = "material_fast_loop"))]
fn benchmark_fast_loop_comparison(_: &mut Criterion) {}

/// Benchmark complete evaluation workflow
fn benchmark_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("full_evaluation", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            // Evaluate material
            let material = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

            // Calculate balance
            let balance =
                evaluator.calculate_material_balance(&board, &captured_pieces, Player::Black);

            // Count total material
            let total = evaluator.count_total_material(&board);

            black_box((material, balance, total));
        });
    });

    group.bench_function("repeated_evaluations", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
            }
        });
    });

    group.finish();
}

/// Benchmark statistics tracking
fn benchmark_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("with_stats_tracking", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
            black_box(evaluator.stats());
        });
    });

    group.bench_function("stats_overhead", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            for _ in 0..1000 {
                evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
            }
            black_box(evaluator.stats());
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("create_many_evaluators", |b| {
        b.iter(|| {
            let evaluators: Vec<MaterialEvaluator> =
                (0..100).map(|_| MaterialEvaluator::new()).collect();
            black_box(evaluators);
        });
    });

    group.bench_function("evaluate_many_positions", |b| {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        b.iter(|| {
            let scores: Vec<TaperedScore> = (0..100)
                .map(|_| evaluator.evaluate_material(&board, Player::Black, &captured_pieces))
                .collect();
            black_box(scores);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evaluator_creation,
    benchmark_piece_values,
    benchmark_material_evaluation,
    benchmark_hand_evaluation,
    benchmark_material_scenarios,
    benchmark_material_balance,
    benchmark_material_counting,
    benchmark_ablation,
    benchmark_configurations,
    benchmark_fast_loop_comparison,
    benchmark_complete_workflow,
    benchmark_statistics,
    benchmark_memory_patterns,
);

criterion_main!(benches);
