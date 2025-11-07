//! Performance benchmarks for configuration system
//!
//! This benchmark suite measures the performance of:
//! - Configuration creation
//! - Configuration validation
//! - Runtime weight updates
//! - Configuration serialization/deserialization

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::evaluation::config::TaperedEvalConfig;

/// Benchmark configuration creation
fn benchmark_config_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::new());
        });
    });

    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::default());
        });
    });

    group.bench_function("disabled", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::disabled());
        });
    });

    group.bench_function("performance_optimized", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::performance_optimized());
        });
    });

    group.bench_function("strength_optimized", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::strength_optimized());
        });
    });

    group.bench_function("memory_optimized", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::memory_optimized());
        });
    });

    group.finish();
}

/// Benchmark configuration validation
fn benchmark_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    let config = TaperedEvalConfig::default();

    group.bench_function("validate", |b| {
        b.iter(|| {
            black_box(config.validate());
        });
    });

    group.bench_function("validate_all_presets", |b| {
        b.iter(|| {
            black_box(TaperedEvalConfig::default().validate());
            black_box(TaperedEvalConfig::performance_optimized().validate());
            black_box(TaperedEvalConfig::strength_optimized().validate());
            black_box(TaperedEvalConfig::memory_optimized().validate());
        });
    });

    group.finish();
}

/// Benchmark runtime weight updates
fn benchmark_weight_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("weight_updates");

    group.bench_function("update_single_weight", |b| {
        let mut config = TaperedEvalConfig::default();
        b.iter(|| {
            black_box(config.update_weight("material", 1.5));
        });
    });

    group.bench_function("update_all_weights", |b| {
        let mut config = TaperedEvalConfig::default();
        b.iter(|| {
            config.update_weight("material", 1.2).ok();
            config.update_weight("position", 0.9).ok();
            config.update_weight("king_safety", 1.1).ok();
            config.update_weight("pawn_structure", 0.85).ok();
            config.update_weight("mobility", 0.65).ok();
            config.update_weight("center_control", 0.75).ok();
            config.update_weight("development", 0.55).ok();
            black_box(&config);
        });
    });

    group.bench_function("get_weight", |b| {
        let config = TaperedEvalConfig::default();
        b.iter(|| {
            black_box(config.get_weight("material"));
        });
    });

    group.finish();
}

/// Benchmark feature toggles
fn benchmark_feature_toggles(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_toggles");

    group.bench_function("toggle_single_feature", |b| {
        let mut config = TaperedEvalConfig::default();
        b.iter(|| {
            config.set_feature_enabled("mobility", false);
            black_box(&config);
        });
    });

    group.bench_function("toggle_all_features", |b| {
        let mut config = TaperedEvalConfig::default();
        b.iter(|| {
            config.set_feature_enabled("king_safety", false);
            config.set_feature_enabled("pawn_structure", false);
            config.set_feature_enabled("mobility", false);
            config.set_feature_enabled("center_control", false);
            config.set_feature_enabled("development", false);
            black_box(&config);
        });
    });

    group.finish();
}

/// Benchmark configuration queries
fn benchmark_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");

    let config = TaperedEvalConfig::default();

    group.bench_function("list_weights", |b| {
        b.iter(|| {
            black_box(config.list_weights());
        });
    });

    group.bench_function("multiple_get_weight", |b| {
        b.iter(|| {
            for name in &["material", "position", "king_safety", "mobility"] {
                black_box(config.get_weight(name));
            }
        });
    });

    group.finish();
}

/// Benchmark serialization
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let config = TaperedEvalConfig::default();

    group.bench_function("serialize_json", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&config));
        });
    });

    group.bench_function("serialize_json_pretty", |b| {
        b.iter(|| {
            black_box(serde_json::to_string_pretty(&config));
        });
    });

    let json = serde_json::to_string(&config).unwrap();

    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<TaperedEvalConfig>(&json));
        });
    });

    group.finish();
}

/// Benchmark configuration cloning
fn benchmark_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloning");

    let config = TaperedEvalConfig::default();

    group.bench_function("clone", |b| {
        b.iter(|| {
            black_box(config.clone());
        });
    });

    group.bench_function("clone_many", |b| {
        b.iter(|| {
            let configs: Vec<TaperedEvalConfig> = (0..100).map(|_| config.clone()).collect();
            black_box(configs);
        });
    });

    group.finish();
}

/// Benchmark complete workflows
fn benchmark_workflows(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflows");

    group.bench_function("create_validate_update", |b| {
        b.iter(|| {
            let mut config = TaperedEvalConfig::default();
            config.validate().ok();
            config.update_weight("material", 1.2).ok();
            config.validate().ok();
            black_box(config);
        });
    });

    group.bench_function("serialize_deserialize_validate", |b| {
        let config = TaperedEvalConfig::default();
        b.iter(|| {
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: TaperedEvalConfig = serde_json::from_str(&json).unwrap();
            deserialized.validate().ok();
            black_box(deserialized);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_config_creation,
    benchmark_validation,
    benchmark_weight_updates,
    benchmark_feature_toggles,
    benchmark_queries,
    benchmark_serialization,
    benchmark_cloning,
    benchmark_workflows,
);

criterion_main!(benches);
