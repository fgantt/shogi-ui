#![cfg(feature = "hierarchical-tt")]

//! Benchmarks comparing flat vs. hierarchical transposition tables (Task 9.9)
//!
//! This suite evaluates end-to-end store+probe throughput across two table
//! implementations:
//!   * Baseline `ThreadSafeTranspositionTable`
//!   * New `HierarchicalTranspositionTable` (L1 + compressed L2)
//!
//! The workload simulates a typical search pattern:
//!   1. Insert a batch of mixed-depth entries with varied bounds and ages
//!   2. Probe the same hashes at their recorded depths (mostly hits)
//!   3. Repeat the sequence every Criterion iteration to gather timings
//!
//! In addition to Criterion metrics, we emit a single reference measurement so
//! the documentation can report concrete numbers without parsing the benchmark
//! output.

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use shogi_engine::search::thread_safe_table::ThreadSafeEntry;
use shogi_engine::search::{
    CompressedTranspositionTableConfig, HierarchicalTranspositionConfig,
    HierarchicalTranspositionTable, HitLevel, ThreadSafeTranspositionTable, TranspositionConfig,
};
use shogi_engine::types::{EntrySource, TranspositionEntry, TranspositionFlag};
use std::time::{Duration, Instant};

const WORKLOAD_SIZE: usize = 4096;

fn generate_entries(count: usize) -> Vec<TranspositionEntry> {
    let mut entries = Vec::with_capacity(count);
    let mut hash = 0x9E37_79B9_7F4A_7C15u64;

    for i in 0..count {
        hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1);

        let depth = (1 + (i % 8)) as u8;
        let flag = match i % 3 {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            _ => TranspositionFlag::UpperBound,
        };
        let age = (i % 32) as u32;
        let score = ((i as i32) % 200) - 100;

        entries.push(TranspositionEntry::new(
            score,
            depth,
            flag,
            None,
            hash,
            age,
            EntrySource::MainSearch,
        ));
    }

    entries
}

fn generate_probe_workload(entries: &[TranspositionEntry]) -> Vec<(u64, u8)> {
    let mut probes = Vec::with_capacity(entries.len());
    for entry in entries {
        probes.push((entry.hash_key, entry.depth));
    }
    probes
}

fn make_flat_config() -> TranspositionConfig {
    let mut config = TranspositionConfig::debug_config();
    config.table_size = 1 << 15; // 32K entries
    config.bucket_count = 64;
    config.enable_statistics = true;
    config.clear_between_games = false;
    config
}

fn make_hierarchical_config() -> HierarchicalTranspositionConfig {
    HierarchicalTranspositionConfig::default()
        .with_statistics_enabled(true)
        .with_l1_table_size(1 << 10) // 1K entries (forces L2 usage under workload)
        .with_l2_config(
            CompressedTranspositionTableConfig::default()
                .with_max_entries(1 << 16) // 65K logical entries
                .with_segment_count(128)
                .with_target_compression_ratio(0.5)
                .with_max_maintenance_backlog(1 << 16),
        )
        .with_promotion_depth(6)
        .with_demotion_age(2)
}

fn store_and_probe_flat(
    entries: &[TranspositionEntry],
    probes: &[(u64, u8)],
    config: &TranspositionConfig,
) -> usize {
    let mut table = ThreadSafeTranspositionTable::new(config.clone());

    for entry in entries {
        table.store(entry.clone());
    }

    let mut hits = 0usize;
    for (hash, depth) in probes {
        if table.probe(*hash, *depth).is_some() {
            hits += 1;
        }
    }

    hits
}

fn store_and_probe_hierarchical(
    entries: &[TranspositionEntry],
    probes: &[(u64, u8)],
    config: &HierarchicalTranspositionConfig,
) -> (usize, HierarchicalTranspositionTable) {
    let mut table = HierarchicalTranspositionTable::new(config.clone());

    for entry in entries {
        table.store(entry.clone());
    }

    let mut hits = 0usize;
    for (hash, depth) in probes {
        if let Some((_entry, level)) = table.probe(*hash, *depth) {
            if matches!(level, HitLevel::L1 | HitLevel::L2) {
                hits += 1;
            }
        }
    }

    (hits, table)
}

fn benchmark_hierarchical_vs_flat(c: &mut Criterion) {
    let entries = generate_entries(WORKLOAD_SIZE);
    let probes = generate_probe_workload(&entries);
    let flat_config = make_flat_config();
    let hierarchical_config = make_hierarchical_config();

    let mut group = c.benchmark_group("tt_hierarchical_vs_flat");
    group.measurement_time(Duration::from_secs(4));
    group.sample_size(12);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(WORKLOAD_SIZE as u64));

    group.bench_function(BenchmarkId::new("flat_store_probe", WORKLOAD_SIZE), |b| {
        b.iter(|| {
            let hits = store_and_probe_flat(black_box(&entries), black_box(&probes), &flat_config);
            black_box(hits)
        })
    });

    group.bench_function(
        BenchmarkId::new("hierarchical_store_probe", WORKLOAD_SIZE),
        |b| {
            b.iter(|| {
                let (hits, _table) = store_and_probe_hierarchical(
                    black_box(&entries),
                    black_box(&probes),
                    &hierarchical_config,
                );
                black_box(hits)
            })
        },
    );

    group.finish();

    log_reference_measurement(&entries, &probes, &flat_config, &hierarchical_config);
}

fn log_reference_measurement(
    entries: &[TranspositionEntry],
    probes: &[(u64, u8)],
    flat_config: &TranspositionConfig,
    hierarchical_config: &HierarchicalTranspositionConfig,
) {
    let start_flat = Instant::now();
    let hits_flat = store_and_probe_flat(entries, probes, flat_config);
    let flat_duration = start_flat.elapsed();

    let start_hier = Instant::now();
    let (hits_hier, hier_table) =
        store_and_probe_hierarchical(entries, probes, hierarchical_config);
    let hier_duration = start_hier.elapsed();
    let hier_snapshot = hier_table.snapshot();

    println!(
        "[TT Baseline vs Hierarchical] workload={} flat_time_ms={:.2} hier_time_ms={:.2} flat_hits={} hier_hits={} l1_hits={} l2_hits={} promotions={} demotions={}",
        entries.len(),
        flat_duration.as_secs_f64() * 1000.0,
        hier_duration.as_secs_f64() * 1000.0,
        hits_flat,
        hits_hier,
        hier_snapshot.stats.l1_hits,
        hier_snapshot.stats.l2_hits,
        hier_snapshot.stats.promotions,
        hier_snapshot.stats.demotions,
    );

    log_memory_vs_hit_rate(entries, probes);
}

fn log_memory_vs_hit_rate(entries: &[TranspositionEntry], probes: &[(u64, u8)]) {
    const THREAD_SAFE_ENTRY_BYTES: u64 = std::mem::size_of::<ThreadSafeEntry>() as u64;
    let l1_sizes = [1 << 9, 1 << 10, 1 << 11, 1 << 12];
    let promotion_depths = [4u8, 6u8];
    let demotion_ages = [2u32, 4u32];

    let base_l2_config = CompressedTranspositionTableConfig::default()
        .with_max_entries(1 << 16)
        .with_segment_count(128)
        .with_target_compression_ratio(0.5)
        .with_max_maintenance_backlog(1 << 16);

    for &l1_size in &l1_sizes {
        for &promotion_depth in &promotion_depths {
            for &demotion_age in &demotion_ages {
                let config = HierarchicalTranspositionConfig::default()
                    .with_statistics_enabled(true)
                    .with_l1_table_size(l1_size)
                    .with_l2_config(base_l2_config.clone())
                    .with_promotion_depth(promotion_depth)
                    .with_demotion_age(demotion_age);

                let (hits, table) = store_and_probe_hierarchical(entries, probes, &config);
                let snapshot = table.snapshot();
                let l2_stats = snapshot.l2_stats;
                let l1_hits = snapshot.stats.l1_hits;
                let l2_hits = snapshot.stats.l2_hits;
                let total_hits = hits as u64;
                let hit_rate = if probes.is_empty() {
                    0.0
                } else {
                    (hits as f64 / probes.len() as f64) * 100.0
                };

                let l1_capacity = l1_size.next_power_of_two() as u64;
                let l1_bytes = l1_capacity * THREAD_SAFE_ENTRY_BYTES;
                let l2_bytes = l2_stats.physical_bytes;
                let total_bytes = l1_bytes + l2_bytes;

                println!(
                    "[TT Memory vs Hit Rate] l1_entries={} promotion_depth={} demotion_age={} total_hits={} hit_rate={:.2}% l1_hits={} l2_hits={} l1_mem_mb={:.2} l2_mem_mb={:.2} total_mem_mb={:.2}",
                    l1_capacity,
                    promotion_depth,
                    demotion_age,
                    total_hits,
                    hit_rate,
                    l1_hits,
                    l2_hits,
                    l1_bytes as f64 / 1_048_576.0,
                    l2_bytes as f64 / 1_048_576.0,
                    total_bytes as f64 / 1_048_576.0
                );
            }
        }
    }
}

criterion_group!(benches, benchmark_hierarchical_vs_flat);
criterion_main!(benches);
