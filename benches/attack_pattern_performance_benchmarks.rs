use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::attack_patterns::AttackTables;
use shogi_engine::types::{PieceType, Player};

/// Benchmark attack pattern precomputation initialization
fn bench_attack_tables_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("attack_tables_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            let tables = AttackTables::new();
            black_box(tables);
        });
    });

    group.finish();
}

/// Benchmark attack pattern lookup performance
fn bench_attack_pattern_lookup(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("attack_pattern_lookup");

    // Test different piece types
    let piece_types = vec![
        PieceType::King,
        PieceType::Knight,
        PieceType::Gold,
        PieceType::Silver,
        PieceType::PromotedPawn,
        PieceType::PromotedLance,
        PieceType::PromotedKnight,
        PieceType::PromotedSilver,
        PieceType::PromotedBishop,
        PieceType::PromotedRook,
    ];

    for piece_type in piece_types {
        let players = vec![Player::Black, Player::White];

        for player in players {
            let benchmark_id = BenchmarkId::new(format!("{:?}_{:?}", piece_type, player), "lookup");

            group.bench_function(benchmark_id, |b| {
                b.iter(|| {
                    // Test multiple squares to get realistic performance
                    for square in 0..81 {
                        let pattern = tables.get_attack_pattern(square, piece_type, player);
                        black_box(pattern);
                    }
                });
            });
        }
    }

    group.finish();
}

/// Benchmark attack pattern lookup for specific squares
fn bench_specific_square_lookups(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("specific_square_lookups");

    // Test center squares, edge squares, and corner squares
    let test_squares = vec![
        40, // Center (4,4)
        4,  // Top edge (0,4)
        76, // Bottom edge (8,4)
        0,  // Top-left corner (0,0)
        8,  // Top-right corner (0,8)
        72, // Bottom-left corner (8,0)
        80, // Bottom-right corner (8,8)
    ];

    for square in test_squares {
        let benchmark_id = BenchmarkId::new("king_center", square);

        group.bench_function(benchmark_id, |b| {
            b.iter(|| {
                let pattern = tables.get_attack_pattern(square, PieceType::King, Player::Black);
                black_box(pattern);
            });
        });
    }

    group.finish();
}

/// Benchmark is_square_attacked performance
fn bench_is_square_attacked(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("is_square_attacked");

    group.bench_function("king_attacks", |b| {
        b.iter(|| {
            // Test king attacking adjacent squares
            for from_square in 0..81 {
                for to_square in 0..81 {
                    let is_attacked = tables.is_square_attacked(
                        from_square,
                        to_square,
                        PieceType::King,
                        Player::Black,
                    );
                    black_box(is_attacked);
                }
            }
        });
    });

    group.finish();
}

/// Benchmark memory access patterns
fn bench_memory_access_patterns(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("memory_access_patterns");

    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            let mut total_attacks = 0u32;
            for square in 0..81 {
                let pattern = tables.get_attack_pattern(square, PieceType::King, Player::Black);
                total_attacks += pattern.count_ones();
            }
            black_box(total_attacks);
        });
    });

    group.bench_function("random_access", |b| {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut squares: Vec<u8> = (0..81).collect();
        squares.shuffle(&mut rng);

        b.iter(|| {
            let mut total_attacks = 0u32;
            for &square in &squares {
                let pattern = tables.get_attack_pattern(square, PieceType::King, Player::Black);
                total_attacks += pattern.count_ones();
            }
            black_box(total_attacks);
        });
    });

    group.finish();
}

/// Benchmark comparison with traditional raycast method
fn bench_vs_traditional_raycast(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("precomputed_vs_raycast");

    group.bench_function("precomputed_king", |b| {
        b.iter(|| {
            for square in 0..81 {
                let pattern = tables.get_attack_pattern(square, PieceType::King, Player::Black);
                black_box(pattern);
            }
        });
    });

    // Note: This would require implementing a traditional raycast method for comparison
    // For now, we'll just benchmark the precomputed version
    group.finish();
}

/// Benchmark player-dependent pattern mirroring
fn bench_pattern_mirroring(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("pattern_mirroring");

    let player_dependent_pieces = vec![
        PieceType::Knight,
        PieceType::Gold,
        PieceType::Silver,
        PieceType::PromotedPawn,
        PieceType::PromotedLance,
        PieceType::PromotedKnight,
        PieceType::PromotedSilver,
    ];

    for piece_type in player_dependent_pieces {
        let benchmark_id = BenchmarkId::new(format!("{:?}", piece_type), "mirroring");

        group.bench_function(benchmark_id, |b| {
            b.iter(|| {
                for square in 0..81 {
                    // Test both black and white patterns (white requires mirroring)
                    let black_pattern =
                        tables.get_attack_pattern(square, piece_type, Player::Black);
                    let white_pattern =
                        tables.get_attack_pattern(square, piece_type, Player::White);
                    black_box(black_pattern);
                    black_box(white_pattern);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark cache efficiency
fn bench_cache_efficiency(c: &mut Criterion) {
    let tables = AttackTables::new();
    let mut group = c.benchmark_group("cache_efficiency");

    group.bench_function("repeated_lookups", |b| {
        b.iter(|| {
            // Repeated lookups of the same pattern should be very fast due to cache locality
            for _ in 0..1000 {
                let pattern = tables.get_attack_pattern(40, PieceType::King, Player::Black);
                black_box(pattern);
            }
        });
    });

    group.bench_function("mixed_lookups", |b| {
        b.iter(|| {
            // Mixed lookups to test cache behavior
            for i in 0..1000u32 {
                let square = (i * 13) % 81; // Pseudo-random but deterministic
                let piece_type = match i % 4 {
                    0 => PieceType::King,
                    1 => PieceType::Knight,
                    2 => PieceType::Gold,
                    _ => PieceType::Silver,
                };
                let player = if i % 2 == 0 {
                    Player::Black
                } else {
                    Player::White
                };

                let pattern = tables.get_attack_pattern(square as u8, piece_type, player);
                black_box(pattern);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_attack_tables_creation,
    bench_attack_pattern_lookup,
    bench_specific_square_lookups,
    bench_is_square_attacked,
    bench_memory_access_patterns,
    bench_vs_traditional_raycast,
    bench_pattern_mirroring,
    bench_cache_efficiency
);

criterion_main!(benches);
