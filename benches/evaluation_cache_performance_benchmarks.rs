use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_vibe_usi::bitboards::BitboardBoard;
use shogi_vibe_usi::evaluation::eval_cache::*;
use shogi_vibe_usi::types::*;

/// Benchmark basic cache operations
fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    let cache = EvaluationCache::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("probe_miss", |b| {
        b.iter(|| black_box(cache.probe(&board, Player::Black, &captured_pieces)))
    });

    // Store a value first
    cache.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("probe_hit", |b| {
        b.iter(|| black_box(cache.probe(&board, Player::Black, &captured_pieces)))
    });

    group.bench_function("store", |b| {
        b.iter(|| cache.store(&board, Player::Black, &captured_pieces, black_box(150), 5))
    });

    group.finish();
}

/// Benchmark different cache sizes
fn bench_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_sizes");

    let sizes_mb = vec![4, 8, 16, 32, 64];

    for size_mb in sizes_mb {
        let config = EvaluationCacheConfig::with_size_mb(size_mb);
        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Pre-populate cache
        cache.store(&board, Player::Black, &captured_pieces, 150, 5);

        group.bench_with_input(BenchmarkId::new("probe", size_mb), &size_mb, |b, _| {
            b.iter(|| black_box(cache.probe(&board, Player::Black, &captured_pieces)))
        });

        group.bench_with_input(BenchmarkId::new("store", size_mb), &size_mb, |b, _| {
            b.iter(|| cache.store(&board, Player::Black, &captured_pieces, black_box(150), 5))
        });
    }

    group.finish();
}

/// Benchmark different replacement policies
fn bench_replacement_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("replacement_policies");

    let policies = vec![
        ReplacementPolicy::AlwaysReplace,
        ReplacementPolicy::DepthPreferred,
        ReplacementPolicy::AgingBased,
    ];

    for policy in policies {
        let mut config = EvaluationCacheConfig::default();
        config.replacement_policy = policy;
        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let policy_name = format!("{:?}", policy);

        // Pre-populate cache
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        group.bench_with_input(
            BenchmarkId::new("store_replace", &policy_name),
            &policy_name,
            |b, _| {
                b.iter(|| cache.store(&board, Player::Black, &captured_pieces, black_box(150), 6))
            },
        );
    }

    group.finish();
}

/// Benchmark cache under different load patterns
fn bench_cache_load_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_load_patterns");

    let cache = EvaluationCache::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Sequential access pattern
    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            for i in 0..100 {
                let score = i * 10;
                cache.store(&board, Player::Black, &captured_pieces, score, 5);
                let _ = black_box(cache.probe(&board, Player::Black, &captured_pieces));
            }
        })
    });

    // Random access pattern (simulated with different depths)
    group.bench_function("random_access", |b| {
        b.iter(|| {
            for depth in 1..=100 {
                cache.store(
                    &board,
                    Player::Black,
                    &captured_pieces,
                    depth as i32 * 10,
                    depth as u8 % 10,
                );
                let _ = black_box(cache.probe(&board, Player::Black, &captured_pieces));
            }
        })
    });

    group.finish();
}

/// Benchmark cache statistics overhead
fn bench_statistics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_overhead");

    // With statistics enabled
    let mut config_with_stats = EvaluationCacheConfig::default();
    config_with_stats.enable_statistics = true;
    let cache_with_stats = EvaluationCache::with_config(config_with_stats);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    cache_with_stats.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("probe_with_stats", |b| {
        b.iter(|| black_box(cache_with_stats.probe(&board, Player::Black, &captured_pieces)))
    });

    // Without statistics
    let mut config_no_stats = EvaluationCacheConfig::default();
    config_no_stats.enable_statistics = false;
    let cache_no_stats = EvaluationCache::with_config(config_no_stats);

    cache_no_stats.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("probe_without_stats", |b| {
        b.iter(|| black_box(cache_no_stats.probe(&board, Player::Black, &captured_pieces)))
    });

    group.finish();
}

/// Benchmark verification overhead
fn bench_verification_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_overhead");

    // With verification enabled
    let mut config_with_verify = EvaluationCacheConfig::default();
    config_with_verify.enable_verification = true;
    let cache_with_verify = EvaluationCache::with_config(config_with_verify);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    cache_with_verify.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("probe_with_verify", |b| {
        b.iter(|| black_box(cache_with_verify.probe(&board, Player::Black, &captured_pieces)))
    });

    // Without verification
    let mut config_no_verify = EvaluationCacheConfig::default();
    config_no_verify.enable_verification = false;
    let cache_no_verify = EvaluationCache::with_config(config_no_verify);

    cache_no_verify.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("probe_without_verify", |b| {
        b.iter(|| black_box(cache_no_verify.probe(&board, Player::Black, &captured_pieces)))
    });

    group.finish();
}

/// Benchmark cache clear operation
fn bench_cache_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_clear");

    let sizes_mb = vec![4, 8, 16, 32];

    for size_mb in sizes_mb {
        let config = EvaluationCacheConfig::with_size_mb(size_mb);
        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Pre-populate cache
        for i in 0..100 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        }

        group.bench_with_input(BenchmarkId::new("clear", size_mb), &size_mb, |b, _| {
            b.iter(|| cache.clear())
        });
    }

    group.finish();
}

/// Benchmark cache statistics retrieval
fn bench_get_statistics(c: &mut Criterion) {
    let cache = EvaluationCache::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Perform some operations
    for i in 0..100 {
        cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        let _ = cache.probe(&board, Player::Black, &captured_pieces);
    }

    c.bench_function("get_statistics", |b| {
        b.iter(|| black_box(cache.get_statistics()))
    });
}

/// Benchmark concurrent access patterns (stress test)
fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");

    let cache = EvaluationCache::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("mixed_read_write", |b| {
        b.iter(|| {
            // Simulate mixed read/write workload
            for i in 0..50 {
                cache.store(
                    &board,
                    Player::Black,
                    &captured_pieces,
                    i * 10,
                    (i % 10) as u8,
                );
            }
            for _ in 0..50 {
                let _ = black_box(cache.probe(&board, Player::Black, &captured_pieces));
            }
        })
    });

    group.finish();
}

/// Benchmark hit rate under different scenarios
fn bench_hit_rate_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_rate_scenarios");

    let cache = EvaluationCache::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Scenario 1: High hit rate (same position repeatedly)
    cache.store(&board, Player::Black, &captured_pieces, 150, 5);

    group.bench_function("high_hit_rate", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(cache.probe(&board, Player::Black, &captured_pieces));
            }
        })
    });

    // Scenario 2: Low hit rate (always missing)
    group.bench_function("low_hit_rate", |b| {
        b.iter(|| {
            for i in 0..100 {
                // Different player each time to force misses
                let player = if i % 2 == 0 {
                    Player::Black
                } else {
                    Player::White
                };
                cache.clear(); // Clear to force miss
                let _ = black_box(cache.probe(&board, player, &captured_pieces));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_operations,
    bench_cache_sizes,
    bench_replacement_policies,
    bench_cache_load_patterns,
    bench_statistics_overhead,
    bench_verification_overhead,
    bench_cache_clear,
    bench_get_statistics,
    bench_concurrent_access,
    bench_hit_rate_scenarios,
);
criterion_main!(benches);
