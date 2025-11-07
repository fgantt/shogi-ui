//! Performance benchmarks for dynamic reduction formula scaling
//!
//! This benchmark suite measures the performance and effectiveness of different
//! reduction formulas for null move pruning:
//! - Static: Always uses base reduction factor
//! - Linear: R = base + depth / 6 (integer division, creates steps)
//! - Smooth: R = base + (depth / 6.0).round() (floating-point with rounding)
//!
//! Metrics measured:
//! - Search time
//! - Nodes searched
//! - NMP cutoff rates
//! - Average reduction factor applied
//!
//! Expected results:
//! - Smooth formula should provide more consistent scaling than Linear
//! - All formulas should maintain NMP effectiveness

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, DynamicReductionFormula, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with specific reduction formula
fn create_test_engine_with_formula(formula: DynamicReductionFormula) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = NullMoveConfig::default();
    config.dynamic_reduction_formula = formula;
    config.enable_dynamic_reduction = match formula {
        DynamicReductionFormula::Static => false,
        _ => true,
    };
    engine.update_null_move_config(config).unwrap();
    engine
}

/// Benchmark reduction formula calculations directly
fn benchmark_reduction_formula_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduction_formula_calculations");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(1000);

    let base_reduction = 2;

    // Benchmark Static formula
    group.bench_function("static_formula", |b| {
        b.iter(|| {
            for depth in 3..=18 {
                black_box(
                    DynamicReductionFormula::Static.calculate_reduction(depth, base_reduction),
                );
            }
        });
    });

    // Benchmark Linear formula
    group.bench_function("linear_formula", |b| {
        b.iter(|| {
            for depth in 3..=18 {
                black_box(
                    DynamicReductionFormula::Linear.calculate_reduction(depth, base_reduction),
                );
            }
        });
    });

    // Benchmark Smooth formula
    group.bench_function("smooth_formula", |b| {
        b.iter(|| {
            for depth in 3..=18 {
                black_box(
                    DynamicReductionFormula::Smooth.calculate_reduction(depth, base_reduction),
                );
            }
        });
    });

    group.finish();
}

/// Benchmark search performance with different reduction formulas
fn benchmark_search_performance_with_formulas(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance_with_formulas");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths
    for depth in [3, 4, 5, 6, 12] {
        // Benchmark with Static formula
        group.bench_with_input(BenchmarkId::new("static", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine_with_formula(DynamicReductionFormula::Static);
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
        });

        // Benchmark with Linear formula
        group.bench_with_input(BenchmarkId::new("linear", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine_with_formula(DynamicReductionFormula::Linear);
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
        });

        // Benchmark with Smooth formula
        group.bench_with_input(BenchmarkId::new("smooth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine_with_formula(DynamicReductionFormula::Smooth);
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
        });
    }

    group.finish();
}

/// Benchmark reduction formula effectiveness comparison
fn benchmark_reduction_formula_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduction_formula_effectiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    let formulas = vec![
        ("static", DynamicReductionFormula::Static),
        ("linear", DynamicReductionFormula::Linear),
        ("smooth", DynamicReductionFormula::Smooth),
    ];

    for (name, formula) in formulas {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut engine = create_test_engine_with_formula(formula);
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
                let avg_reduction = stats.average_reduction_factor();

                black_box((result, elapsed, nodes, cutoff_rate, avg_reduction))
            });
        });
    }

    group.finish();
}

/// Benchmark reduction values at different depths for comparison
fn benchmark_reduction_values_by_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduction_values_by_depth");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    let base_reduction = 2;
    let depths = vec![3, 4, 5, 6, 9, 12, 18];

    for depth in depths {
        // Static formula
        group.bench_with_input(BenchmarkId::new("static", depth), &depth, |b, &depth| {
            b.iter(|| {
                black_box(
                    DynamicReductionFormula::Static.calculate_reduction(depth, base_reduction),
                )
            });
        });

        // Linear formula
        group.bench_with_input(BenchmarkId::new("linear", depth), &depth, |b, &depth| {
            b.iter(|| {
                black_box(
                    DynamicReductionFormula::Linear.calculate_reduction(depth, base_reduction),
                )
            });
        });

        // Smooth formula
        group.bench_with_input(BenchmarkId::new("smooth", depth), &depth, |b, &depth| {
            b.iter(|| {
                black_box(
                    DynamicReductionFormula::Smooth.calculate_reduction(depth, base_reduction),
                )
            });
        });
    }

    group.finish();
}

/// Benchmark comprehensive reduction formula analysis
fn benchmark_comprehensive_reduction_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_reduction_analysis");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test configurations
    let configurations = vec![
        ("static", DynamicReductionFormula::Static),
        ("linear", DynamicReductionFormula::Linear),
        ("smooth", DynamicReductionFormula::Smooth),
    ];

    for (name, formula) in configurations {
        group.bench_with_input(
            BenchmarkId::new("formula", name),
            &formula,
            |b, &formula| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_formula(formula);
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
                    let avg_reduction = stats.average_reduction_factor();

                    black_box((result, elapsed, nodes, cutoff_rate, avg_reduction))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_reduction_formula_calculations,
    benchmark_search_performance_with_formulas,
    benchmark_reduction_formula_effectiveness,
    benchmark_reduction_values_by_depth,
    benchmark_comprehensive_reduction_analysis
);

criterion_main!(benches);
