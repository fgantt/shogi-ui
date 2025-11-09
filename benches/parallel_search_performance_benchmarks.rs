// Bench: Parallel root search
//
// Env overrides (optional):
// - SHOGI_BENCH_DEPTHS: comma list of depths (e.g., "7,8")
// - SHOGI_BENCH_THREADS: comma list of thread counts (e.g., "1,4")
// - SHOGI_YBWC_SCALING: shallow,mid,deep divisors (e.g., "6,4,2")
// - SHOGI_YBWC_BRANCH: ybwc_min_branch (e.g., "20")
// - SHOGI_YBWC_MAX_SIBLINGS: max siblings to parallelize (e.g., "6")
// - SHOGI_YBWC_MIN_DEPTH: activation depth (e.g., "6")
// - SHOGI_TT_GATING: exact_only_max_depth,min_store_depth,buffer_flush_threshold (e.g., "8,9,512")
// - SHOGI_SILENT_BENCH: set to "1" to silence USI info output
// - SHOGI_AGGREGATE_METRICS: set to "1" to enable aggregated TT/YBWC metrics
// - SHOGI_BENCH_FEN: custom FEN (board player captured) to override default start position
// - SHOGI_BENCH_TIME_MS: per-iteration time limit override (applies to all depths)
// - SHOGI_BENCH_TIME_MS_7 / SHOGI_BENCH_TIME_MS_8: per-depth overrides when set
//
// Example quick run:
// SHOGI_BENCH_DEPTHS=3 SHOGI_BENCH_THREADS=1,4 SHOGI_SILENT_BENCH=1 \
// cargo bench --bench parallel_search_performance_benchmarks

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::search::search_engine::{
    snapshot_and_reset_metrics, IterativeDeepening, SearchEngine,
};
use shogi_engine::search::ParallelSearchConfig;
use shogi_engine::types::{CapturedPieces, Player};

fn bench_root_search(c: &mut Criterion) {
    // Silence USI info output during benches to avoid measurement distortion
    std::env::set_var("SHOGI_SILENT_BENCH", "1");
    // Aggregate metrics across the whole run and print once at end
    std::env::set_var("SHOGI_AGGREGATE_METRICS", "1");
    let mut group = c.benchmark_group("parallel_root_search");
    group.sampling_mode(SamplingMode::Auto);
    if let Ok(samples) = std::env::var("SHOGI_BENCH_SAMPLES") {
        if let Ok(n) = samples.parse::<usize>() {
            group.sample_size(n);
        }
    }

    let (board, captured, player) = if let Ok(fen) = std::env::var("SHOGI_BENCH_FEN") {
        if !fen.trim().is_empty() {
            if let Ok((b, p, c)) = BitboardBoard::from_fen(&fen) {
                (b, c, p)
            } else {
                (BitboardBoard::new(), CapturedPieces::new(), Player::Black)
            }
        } else {
            (BitboardBoard::new(), CapturedPieces::new(), Player::Black)
        }
    } else {
        (BitboardBoard::new(), CapturedPieces::new(), Player::Black)
    };
    let mg = MoveGenerator::new();
    let legal = mg.generate_legal_moves(&board, player, &captured);
    group.throughput(Throughput::Elements(legal.len() as u64));

    // Optional env overrides for depths and threads
    let depths_env = std::env::var("SHOGI_BENCH_DEPTHS").ok();
    let depths: Vec<u8> = depths_env
        .as_deref()
        .map(|s| {
            s.split(',')
                .filter_map(|p| p.trim().parse::<u8>().ok())
                .collect()
        })
        .filter(|v: &Vec<u8>| !v.is_empty())
        .unwrap_or_else(|| vec![3u8, 5u8, 6u8, 7u8, 8u8]);

    let threads_env = std::env::var("SHOGI_BENCH_THREADS").ok();
    let thread_counts: Vec<usize> = threads_env
        .as_deref()
        .map(|s| {
            s.split(',')
                .filter_map(|p| p.trim().parse::<usize>().ok())
                .collect()
        })
        .filter(|v: &Vec<usize>| !v.is_empty())
        .unwrap_or_else(|| vec![1usize, 2, 4, 8]);

    // Config overrides for YBWC/TT gating
    let ybwc_min_depth: u8 = std::env::var("SHOGI_YBWC_MIN_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let (ybwc_shallow, ybwc_mid, ybwc_deep) = {
        if let Ok(cfg) = std::env::var("SHOGI_YBWC_SCALING") {
            let parts: Vec<_> = cfg.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(a), Ok(b), Ok(c)) = (
                    parts[0].trim().parse::<usize>(),
                    parts[1].trim().parse::<usize>(),
                    parts[2].trim().parse::<usize>(),
                ) {
                    (a, b, c)
                } else {
                    (6, 4, 2)
                }
            } else {
                (6, 4, 2)
            }
        } else {
            (6, 4, 2)
        }
    };
    let ybwc_branch: usize = std::env::var("SHOGI_YBWC_BRANCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let ybwc_max_siblings: usize = std::env::var("SHOGI_YBWC_MAX_SIBLINGS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let (tt_exact_only_max_depth_value, tt_min_store_depth, tt_buffer_flush_threshold) = {
        if let Ok(cfg) = std::env::var("SHOGI_TT_GATING") {
            let parts: Vec<_> = cfg.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(a), Ok(b), Ok(c)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<usize>(),
                ) {
                    (a, b, c)
                } else {
                    (8, 9, 512)
                }
            } else {
                (8, 9, 512)
            }
        } else {
            (8, 9, 512)
        }
    };

    // Test across depths and thread counts
    for &depth in &depths {
        for &threads in &thread_counts {
            group.bench_with_input(
                BenchmarkId::new(format!("depth{}", depth), threads),
                &threads,
                |b, &t| {
                    b.iter(|| {
                        // New engine per iteration to avoid cross-benchmark state
                        let mut engine = SearchEngine::new(None, 16);
                        // Enable deeper parallelism (YBWC) for benchmark
                        engine.set_ybwc(true, ybwc_min_depth);
                        engine.set_ybwc_branch(ybwc_branch);
                        engine.set_ybwc_max_siblings(ybwc_max_siblings);
                        engine.set_ybwc_scaling(ybwc_shallow, ybwc_mid, ybwc_deep);
                        engine.set_tt_gating(
                            tt_exact_only_max_depth_value,
                            tt_min_store_depth,
                            tt_buffer_flush_threshold,
                        );
                        let time_limit = if let Ok(ms) = std::env::var("SHOGI_BENCH_TIME_MS") {
                            ms.parse::<u64>().unwrap_or(1000) as u32
                        } else {
                            // Per-depth env overrides
                            let key = format!("SHOGI_BENCH_TIME_MS_{}", depth);
                            if let Ok(msd) = std::env::var(key) {
                                msd.parse::<u64>().unwrap_or(0) as u32
                            } else {
                                match depth {
                                    3 => 600,
                                    5 => 1000,
                                    6 => 1200,
                                    7 => 1500,
                                    8 => 2000,
                                    _ => 1000,
                                }
                            }
                        };
                        let mut id = if t > 1 {
                            IterativeDeepening::new_with_threads(
                                depth,
                                time_limit,
                                None,
                                t,
                                ParallelSearchConfig::new(t),
                            )
                        } else {
                            IterativeDeepening::new(depth, time_limit, None)
                        };
                        let _ = id.search(&mut engine, &board, &captured, player);
                    });
                },
            );
        }
    }

    group.finish();
    // Snapshot aggregated profiling metrics for this run and write JSON summary
    let m = snapshot_and_reset_metrics();
    let summary = format!(
        "{{\n  \"tag\": \"{}\",\n  \"tt_reads\": {},\n  \"tt_read_ok\": {},\n  \"tt_read_fail\": {},\n  \"tt_writes\": {},\n  \"tt_write_ok\": {},\n  \"tt_write_fail\": {},\n  \"ybwc_batches\": {},\n  \"ybwc_siblings\": {},\n  \"ybwc_trigger_opportunities\": {},\n  \"ybwc_trigger_eligible_depth\": {},\n  \"ybwc_trigger_eligible_branch\": {},\n  \"ybwc_triggered\": {}\n}}\n",
        "criterion_group:parallel_root_search",
        m.tt_try_reads, m.tt_try_read_successes, m.tt_try_read_fails,
        m.tt_try_writes, m.tt_try_write_successes, m.tt_try_write_fails,
        m.ybwc_sibling_batches, m.ybwc_siblings_evaluated,
        m.ybwc_trigger_opportunities, m.ybwc_trigger_eligible_depth, m.ybwc_trigger_eligible_branch, m.ybwc_triggered
    );
    let out_dir = std::path::Path::new("target/criterion");
    let _ = std::fs::create_dir_all(out_dir);
    let out_path = out_dir.join("metrics-summary.json");
    let _ = std::fs::write(&out_path, summary.as_bytes());
    // Also echo a concise summary line
    println!(
        "metrics summary written: {:?} (tt_reads={}, tt_writes={}, ybwc_batches={}, ybwc_siblings={}, ybwc_triggers={}/{}/{}/{})",
        out_path, m.tt_try_reads, m.tt_try_writes, m.ybwc_sibling_batches, m.ybwc_siblings_evaluated,
        m.ybwc_trigger_opportunities, m.ybwc_trigger_eligible_depth, m.ybwc_trigger_eligible_branch, m.ybwc_triggered
    );
}

criterion_group!(benches, bench_root_search);
criterion_main!(benches);
