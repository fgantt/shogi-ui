use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::tuning::optimizer::Optimizer;
use shogi_engine::tuning::types::{OptimizationMethod, TrainingPosition};
use shogi_engine::types::{NUM_EVAL_FEATURES, Player};

fn generate_test_positions(count: usize) -> Vec<TrainingPosition> {
    (0..count)
        .map(|i| {
            let mut features = vec![0.0; NUM_EVAL_FEATURES];
            features[0] = (i as f64) * 0.1;
            features[1] = ((i * 2) as f64) * 0.1;
            features[2] = ((i * 3) as f64) * 0.1;
            TrainingPosition::new(
                features,
                if i % 2 == 0 { 1.0 } else { 0.0 },
                128,
                false,
                i as u32,
                Player::White,
            )
        })
        .collect()
}

fn benchmark_adam_default_parameters(c: &mut Criterion) {
    let positions = generate_test_positions(100);
    let optimizer = Optimizer::new(OptimizationMethod::Adam {
        learning_rate: 0.001,
        beta1: 0.9,
        beta2: 0.999,
        epsilon: 1e-8,
    });

    c.bench_function("adam_default_parameters", |b| {
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });
}

fn benchmark_adam_high_beta1(c: &mut Criterion) {
    let positions = generate_test_positions(100);
    let optimizer = Optimizer::new(OptimizationMethod::Adam {
        learning_rate: 0.001,
        beta1: 0.95, // Higher momentum
        beta2: 0.999,
        epsilon: 1e-8,
    });

    c.bench_function("adam_high_beta1", |b| {
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });
}

fn benchmark_adam_low_beta2(c: &mut Criterion) {
    let positions = generate_test_positions(100);
    let optimizer = Optimizer::new(OptimizationMethod::Adam {
        learning_rate: 0.001,
        beta1: 0.9,
        beta2: 0.99, // Lower second moment decay
        epsilon: 1e-8,
    });

    c.bench_function("adam_low_beta2", |b| {
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });
}

fn benchmark_adam_low_epsilon(c: &mut Criterion) {
    let positions = generate_test_positions(100);
    let optimizer = Optimizer::new(OptimizationMethod::Adam {
        learning_rate: 0.001,
        beta1: 0.9,
        beta2: 0.999,
        epsilon: 1e-10, // Lower epsilon
    });

    c.bench_function("adam_low_epsilon", |b| {
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });
}

fn benchmark_adam_parameter_comparison(c: &mut Criterion) {
    let positions = generate_test_positions(100);

    let mut group = c.benchmark_group("adam_parameter_comparison");
    group.bench_function("default", |b| {
        let optimizer = Optimizer::new(OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.bench_function("high_beta1", |b| {
        let optimizer = Optimizer::new(OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.95,
            beta2: 0.999,
            epsilon: 1e-8,
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.bench_function("low_beta2", |b| {
        let optimizer = Optimizer::new(OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.99,
            epsilon: 1e-8,
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_adam_default_parameters,
    benchmark_adam_high_beta1,
    benchmark_adam_low_beta2,
    benchmark_adam_low_epsilon,
    benchmark_adam_parameter_comparison
);
criterion_main!(benches);

