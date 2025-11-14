//! Benchmarks for weight validation and phase-dependent scaling in IntegratedEvaluator
//!
//! Measures the performance impact of phase-dependent weight scaling and weight validation.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

fn benchmark_phase_dependent_weight_scaling(c: &mut Criterion) {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Configuration with phase-dependent scaling enabled
    let mut config_enabled = IntegratedEvaluationConfig::default();
    config_enabled.enable_phase_dependent_weights = true;
    let evaluator_enabled = IntegratedEvaluator::with_config(config_enabled);

    // Configuration with phase-dependent scaling disabled
    let mut config_disabled = IntegratedEvaluationConfig::default();
    config_disabled.enable_phase_dependent_weights = false;
    let evaluator_disabled = IntegratedEvaluator::with_config(config_disabled);

    let mut group = c.benchmark_group("phase_dependent_weight_scaling");

    group.bench_function("with_scaling", |b| {
        b.iter(|| {
            black_box(evaluator_enabled.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("without_scaling", |b| {
        b.iter(|| {
            black_box(evaluator_disabled.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

fn benchmark_weight_validation_overhead(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();

    // Set weights to reasonable values
    config.weights.material_weight = 1.0;
    config.weights.position_weight = 1.0;
    config.weights.king_safety_weight = 1.0;
    config.weights.pawn_structure_weight = 1.0;
    config.weights.mobility_weight = 1.0;
    config.weights.center_control_weight = 1.0;
    config.weights.development_weight = 1.0;
    config.weights.tactical_weight = 1.0;
    config.weights.positional_weight = 1.0;
    config.weights.castle_weight = 1.0;

    c.bench_function("cumulative_weight_validation", |b| {
        b.iter(|| black_box(config.validate_cumulative_weights()))
    });
}

fn benchmark_weight_clamping_overhead(c: &mut Criterion) {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Configuration with weights that need clamping
    let mut config = IntegratedEvaluationConfig::default();
    config.weights.tactical_weight = 15.0; // Above max, will be clamped
    config.weights.positional_weight = -5.0; // Below min, will be clamped

    let evaluator = IntegratedEvaluator::with_config(config);

    c.bench_function("weight_clamping_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });
}

fn benchmark_large_contribution_logging_overhead(c: &mut Criterion) {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Configuration with low threshold to trigger logging more often
    let mut config = IntegratedEvaluationConfig::default();
    config.weight_contribution_threshold = 10.0; // Very low threshold
    config.weights.tactical_weight = 5.0; // High weight to trigger logging

    let evaluator = IntegratedEvaluator::with_config(config);

    c.bench_function("large_contribution_logging_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });
}

criterion_group!(
    benches,
    benchmark_phase_dependent_weight_scaling,
    benchmark_weight_validation_overhead,
    benchmark_weight_clamping_overhead,
    benchmark_large_contribution_logging_overhead
);
criterion_main!(benches);
