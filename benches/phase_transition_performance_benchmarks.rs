//! Performance benchmarks for phase transition smoothing
//!
//! This benchmark suite measures the performance of:
//! - Different interpolation methods
//! - Phase transition smoothness validation
//! - Transition rate calculations
//! - Configuration variations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::evaluation::phase_transition::{InterpolationMethod, PhaseTransition};
use shogi_engine::types::*;

/// Benchmark interpolation methods
fn benchmark_interpolation_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpolation_methods");

    let score = TaperedScore::new_tapered(100, 200);
    let phase = 128; // Mid-game

    group.bench_function("linear", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, phase, InterpolationMethod::Linear));
        });
    });

    group.bench_function("cubic", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, phase, InterpolationMethod::Cubic));
        });
    });

    group.bench_function("sigmoid", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, phase, InterpolationMethod::Sigmoid));
        });
    });

    group.bench_function("smoothstep", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, phase, InterpolationMethod::Smoothstep));
        });
    });

    group.finish();
}

/// Benchmark interpolation at different phases
fn benchmark_phase_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase_variations");

    let score = TaperedScore::new_tapered(100, 200);
    let phases = [0, 64, 128, 192, 256];

    for phase in phases {
        group.bench_with_input(BenchmarkId::from_parameter(phase), &phase, |b, &p| {
            let mut transition = PhaseTransition::new();
            b.iter(|| {
                black_box(transition.interpolate(score, p, InterpolationMethod::Linear));
            });
        });
    }

    group.finish();
}

/// Benchmark all phases sweep
fn benchmark_all_phases_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_phases_sweep");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("linear_all_phases", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            for phase in 0..=256 {
                black_box(transition.interpolate(score, phase, InterpolationMethod::Linear));
            }
        });
    });

    group.bench_function("cubic_all_phases", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            for phase in 0..=256 {
                black_box(transition.interpolate(score, phase, InterpolationMethod::Cubic));
            }
        });
    });

    group.finish();
}

/// Benchmark transition validation
fn benchmark_transition_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transition_validation");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("validate_smooth_transitions_linear", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.validate_smooth_transitions(score, InterpolationMethod::Linear));
        });
    });

    group.bench_function("validate_smooth_transitions_cubic", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.validate_smooth_transitions(score, InterpolationMethod::Cubic));
        });
    });

    group.bench_function("is_transition_smooth", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.is_transition_smooth(
                score,
                127,
                128,
                InterpolationMethod::Linear,
            ));
        });
    });

    group.finish();
}

/// Benchmark transition rate calculations
fn benchmark_transition_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("transition_rates");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("calculate_max_rate_linear", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.calculate_max_transition_rate(score, InterpolationMethod::Linear));
        });
    });

    group.bench_function("calculate_max_rate_cubic", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.calculate_max_transition_rate(score, InterpolationMethod::Cubic));
        });
    });

    group.finish();
}

/// Benchmark different score ranges
fn benchmark_score_ranges(c: &mut Criterion) {
    let mut group = c.benchmark_group("score_ranges");

    let scores = [
        ("small_diff", TaperedScore::new_tapered(100, 110)),
        ("medium_diff", TaperedScore::new_tapered(100, 200)),
        ("large_diff", TaperedScore::new_tapered(0, 1000)),
        ("negative", TaperedScore::new_tapered(-100, 100)),
    ];

    for (name, score) in scores {
        group.bench_with_input(BenchmarkId::from_parameter(name), &score, |b, &s| {
            let mut transition = PhaseTransition::new();
            b.iter(|| {
                black_box(transition.interpolate(s, 128, InterpolationMethod::Linear));
            });
        });
    }

    group.finish();
}

/// Benchmark phase clamping
fn benchmark_phase_clamping(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase_clamping");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("no_clamp_needed", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, 128, InterpolationMethod::Linear));
        });
    });

    group.bench_function("clamp_negative", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, -10, InterpolationMethod::Linear));
        });
    });

    group.bench_function("clamp_too_large", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, 300, InterpolationMethod::Linear));
        });
    });

    group.finish();
}

/// Benchmark statistics tracking overhead
fn benchmark_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("with_stats", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, 128, InterpolationMethod::Linear));
            black_box(transition.stats());
        });
    });

    group.bench_function("many_interpolations", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            for phase in 0..=256 {
                transition.interpolate(score, phase, InterpolationMethod::Linear);
            }
            black_box(transition.stats());
        });
    });

    group.finish();
}

/// Benchmark default vs explicit method
fn benchmark_default_vs_explicit(c: &mut Criterion) {
    let mut group = c.benchmark_group("default_vs_explicit");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("default_method", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate_default(score, 128));
        });
    });

    group.bench_function("explicit_linear", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            black_box(transition.interpolate(score, 128, InterpolationMethod::Linear));
        });
    });

    group.finish();
}

/// Benchmark multiple scores
fn benchmark_multiple_scores(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_scores");

    let scores = vec![
        TaperedScore::new_tapered(100, 150),
        TaperedScore::new_tapered(200, 250),
        TaperedScore::new_tapered(50, 100),
        TaperedScore::new_tapered(300, 400),
    ];

    group.bench_function("interpolate_4_scores", |b| {
        let mut transition = PhaseTransition::new();
        b.iter(|| {
            let mut total = 0;
            for score in &scores {
                total += transition.interpolate(*score, 128, InterpolationMethod::Linear);
            }
            black_box(total);
        });
    });

    group.bench_function("interpolate_many_scores", |b| {
        let mut transition = PhaseTransition::new();
        let many_scores: Vec<TaperedScore> = (0..100)
            .map(|i| TaperedScore::new_tapered(i * 10, (i + 1) * 10))
            .collect();

        b.iter(|| {
            let mut total = 0;
            for score in &many_scores {
                total += transition.interpolate(*score, 128, InterpolationMethod::Linear);
            }
            black_box(total);
        });
    });

    group.finish();
}

/// Benchmark configuration variations
fn benchmark_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let score = TaperedScore::new_tapered(100, 200);

    let configs = vec![
        ("default", PhaseTransitionConfig::default()),
        (
            "with_boundaries",
            PhaseTransitionConfig {
                use_phase_boundaries: true,
                ..Default::default()
            },
        ),
        (
            "advanced_enabled",
            PhaseTransitionConfig {
                use_advanced_interpolator: true,
                ..PhaseTransitionConfig::default()
            },
        ),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &config, |b, cfg| {
            let mut transition = PhaseTransition::with_config(cfg.clone());
            b.iter(|| {
                black_box(transition.interpolate(score, 128, transition.default_method));
            });
        });
    }

    group.finish();
}

/// Benchmark complete workflow
fn benchmark_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");

    group.bench_function("typical_evaluation", |b| {
        let mut transition = PhaseTransition::new();

        // Simulate typical evaluation with multiple scores
        let material = TaperedScore::new_tapered(500, 520);
        let position = TaperedScore::new_tapered(150, 180);
        let king_safety = TaperedScore::new_tapered(80, 40);
        let mobility = TaperedScore::new_tapered(30, 60);

        b.iter(|| {
            let phase = 128; // Mid-game

            let mut total = 0;
            total += transition.interpolate(material, phase, InterpolationMethod::Linear);
            total += transition.interpolate(position, phase, InterpolationMethod::Linear);
            total += transition.interpolate(king_safety, phase, InterpolationMethod::Linear);
            total += transition.interpolate(mobility, phase, InterpolationMethod::Linear);

            black_box(total);
        });
    });

    group.bench_function("evaluate_all_phases", |b| {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);

        b.iter(|| {
            let results: Vec<i32> = (0..=256)
                .map(|phase| transition.interpolate(score, phase, InterpolationMethod::Linear))
                .collect();
            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_interpolation_methods,
    benchmark_phase_variations,
    benchmark_all_phases_sweep,
    benchmark_transition_validation,
    benchmark_transition_rates,
    benchmark_score_ranges,
    benchmark_phase_clamping,
    benchmark_statistics,
    benchmark_default_vs_explicit,
    benchmark_multiple_scores,
    benchmark_configurations,
    benchmark_complete_workflow,
);

criterion_main!(benches);
