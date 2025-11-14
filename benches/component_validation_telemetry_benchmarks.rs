//! Benchmarks for Component Validation and Telemetry (Task 17.0 - Task 5.0)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{BitboardBoard, CapturedPieces, Player};

fn benchmark_telemetry_collection_overhead(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.components.tactical_patterns = true;
    config.components.positional_patterns = true;
    config.components.castle_patterns = true;
    config.collect_position_feature_stats = true;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();

    c.bench_function("telemetry_collection_enabled", |b| {
        b.iter(|| {
            let _score = evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
            let _telemetry = evaluator.telemetry_snapshot();
        });
    });

    // Compare with telemetry disabled
    let mut config_no_telemetry = IntegratedEvaluationConfig::default();
    config_no_telemetry.components.material = true;
    config_no_telemetry.components.tactical_patterns = true;
    config_no_telemetry.components.positional_patterns = true;
    config_no_telemetry.components.castle_patterns = true;
    config_no_telemetry.collect_position_feature_stats = false;

    let evaluator_no_telemetry = IntegratedEvaluator::with_config(config_no_telemetry);

    c.bench_function("telemetry_collection_disabled", |b| {
        b.iter(|| {
            let _score = evaluator_no_telemetry.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });
}

fn benchmark_component_validation_overhead(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.components.tactical_patterns = true;
    config.enable_component_validation = false; // Validation disabled

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();

    c.bench_function("component_validation_disabled", |b| {
        b.iter(|| {
            let _score = evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });

    // Compare with validation enabled
    let mut config_validation = IntegratedEvaluationConfig::default();
    config_validation.components.material = true;
    config_validation.components.tactical_patterns = true;
    config_validation.enable_component_validation = true; // Validation enabled

    let evaluator_validation = IntegratedEvaluator::with_config(config_validation);

    c.bench_function("component_validation_enabled", |b| {
        b.iter(|| {
            let _score = evaluator_validation.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
        });
    });
}

fn benchmark_weight_contributions_calculation(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.material = true;
    config.components.piece_square_tables = true;
    config.components.tactical_patterns = true;
    config.components.positional_patterns = true;
    config.components.castle_patterns = true;
    config.collect_position_feature_stats = true;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::default();
    let captured = CapturedPieces::default();

    c.bench_function("weight_contributions_calculation", |b| {
        b.iter(|| {
            let _score = evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured),
            );
            let telemetry = evaluator.telemetry_snapshot();
            let _contributions = telemetry.map(|t| t.weight_contributions);
        });
    });
}

criterion_group!(
    benches,
    benchmark_telemetry_collection_overhead,
    benchmark_component_validation_overhead,
    benchmark_weight_contributions_calculation
);
criterion_main!(benches);
