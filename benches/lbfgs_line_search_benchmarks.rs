use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::tuning::optimizer::Optimizer;
use shogi_engine::tuning::types::{LineSearchType, OptimizationMethod, TrainingPosition};
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

fn benchmark_lbfgs_with_armijo_line_search(c: &mut Criterion) {
    let positions = generate_test_positions(100);
    let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
        memory_size: 10,
        max_iterations: 50,
        line_search_type: LineSearchType::Armijo,
        initial_step_size: 1.0,
        max_line_search_iterations: 20,
        armijo_constant: 0.0001,
        step_size_reduction: 0.5,
    });

    c.bench_function("lbfgs_armijo_line_search", |b| {
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });
}

fn benchmark_lbfgs_convergence_speed(c: &mut Criterion) {
    let positions = generate_test_positions(100);

    let mut group = c.benchmark_group("lbfgs_convergence");
    group.bench_function("with_armijo_line_search", |b| {
        let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 50,
            line_search_type: LineSearchType::Armijo,
            initial_step_size: 1.0,
            max_line_search_iterations: 20,
            armijo_constant: 0.0001,
            step_size_reduction: 0.5,
        });
        b.iter(|| {
            let result = black_box(optimizer.optimize(black_box(&positions)));
            // Measure convergence quality
            if let Ok(res) = result {
                black_box(res.iterations);
                black_box(res.final_error);
            }
        });
    });

    group.bench_function("permissive_line_search", |b| {
        // Effectively fixed step size with very permissive parameters
        let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 50,
            line_search_type: LineSearchType::Armijo,
            initial_step_size: 10.0,
            max_line_search_iterations: 1,
            armijo_constant: 0.00001,
            step_size_reduction: 0.9,
        });
        b.iter(|| {
            let result = black_box(optimizer.optimize(black_box(&positions)));
            if let Ok(res) = result {
                black_box(res.iterations);
                black_box(res.final_error);
            }
        });
    });

    group.finish();
}

fn benchmark_lbfgs_line_search_parameters(c: &mut Criterion) {
    let positions = generate_test_positions(100);

    let mut group = c.benchmark_group("lbfgs_line_search_params");
    
    group.bench_function("default_armijo", |b| {
        let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 50,
            line_search_type: LineSearchType::Armijo,
            initial_step_size: 1.0,
            max_line_search_iterations: 20,
            armijo_constant: 0.0001,
            step_size_reduction: 0.5,
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.bench_function("strict_armijo", |b| {
        // Stricter Armijo condition
        let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 50,
            line_search_type: LineSearchType::Armijo,
            initial_step_size: 1.0,
            max_line_search_iterations: 20,
            armijo_constant: 0.001, // Stricter
            step_size_reduction: 0.5,
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.bench_function("aggressive_backtracking", |b| {
        // More aggressive step size reduction
        let optimizer = Optimizer::new(OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 50,
            line_search_type: LineSearchType::Armijo,
            initial_step_size: 1.0,
            max_line_search_iterations: 20,
            armijo_constant: 0.0001,
            step_size_reduction: 0.25, // More aggressive
        });
        b.iter(|| {
            let _result = black_box(optimizer.optimize(black_box(&positions)));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_lbfgs_with_armijo_line_search,
    benchmark_lbfgs_convergence_speed,
    benchmark_lbfgs_line_search_parameters
);
criterion_main!(benches);

