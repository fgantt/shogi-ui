//! Benchmarks for Phase Transitions (Task 17.0 - Task 6.0)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::evaluation::config::PhaseBoundaryConfig;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{BitboardBoard, CapturedPieces, Player};

fn benchmark_abrupt_vs_gradual_phase_transitions(c: &mut Criterion) {
    let mut config_abrupt = IntegratedEvaluationConfig::default();
    config_abrupt.components.endgame_patterns = true;
    config_abrupt.components.opening_principles = true;
    config_abrupt.enable_gradual_phase_transitions = false;

    let mut config_gradual = IntegratedEvaluationConfig::default();
    config_gradual.components.endgame_patterns = true;
    config_gradual.components.opening_principles = true;
    config_gradual.enable_gradual_phase_transitions = true;

    let evaluator_abrupt = IntegratedEvaluator::with_config(config_abrupt);
    let evaluator_gradual = IntegratedEvaluator::with_config(config_gradual);

    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();

    c.bench_function("evaluation_abrupt_transitions", |b| {
        b.iter(|| {
            let _score = evaluator_abrupt.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });

    c.bench_function("evaluation_gradual_transitions", |b| {
        b.iter(|| {
            let _score = evaluator_gradual.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });
}

fn benchmark_fade_factor_calculation(c: &mut Criterion) {
    let config = PhaseBoundaryConfig::default();

    c.bench_function("calculate_endgame_fade_factor", |b| {
        b.iter(|| {
            for phase in 64..=80 {
                let _fade = config.calculate_endgame_fade_factor(black_box(phase));
            }
        });
    });

    c.bench_function("calculate_opening_fade_factor", |b| {
        b.iter(|| {
            for phase in 160..=192 {
                let _fade = config.calculate_opening_fade_factor(black_box(phase));
            }
        });
    });
}

fn benchmark_phase_boundary_configuration(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();
    config.phase_boundaries.opening_threshold = 200;
    config.phase_boundaries.endgame_threshold = 60;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();

    c.bench_function("evaluation_with_custom_phase_boundaries", |b| {
        b.iter(|| {
            let _score = evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });
}

criterion_group!(
    benches,
    benchmark_abrupt_vs_gradual_phase_transitions,
    benchmark_fade_factor_calculation,
    benchmark_phase_boundary_configuration
);
criterion_main!(benches);
