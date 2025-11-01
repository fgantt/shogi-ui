use criterion::{criterion_group, criterion_main, Criterion, Throughput, SamplingMode, BenchmarkId};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{CapturedPieces, Player};
use shogi_engine::moves::MoveGenerator;
use shogi_engine::search::search_engine::{SearchEngine, IterativeDeepening, snapshot_and_reset_metrics};

fn bench_root_search(c: &mut Criterion) {
    // Silence USI info output during benches to avoid measurement distortion
    std::env::set_var("SHOGI_SILENT_BENCH", "1");
    // Aggregate metrics across the whole run and print once at end
    std::env::set_var("SHOGI_AGGREGATE_METRICS", "1");
    let mut group = c.benchmark_group("parallel_root_search");
    group.sampling_mode(SamplingMode::Auto);

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let mg = MoveGenerator::new();
    let legal = mg.generate_legal_moves(&board, player, &captured);
    group.throughput(Throughput::Elements(legal.len() as u64));

    // Test across depths and thread counts
    for &depth in &[3u8, 5u8, 6u8, 7u8, 8u8] {
        for &threads in &[1usize, 2, 4, 8] {
            group.bench_with_input(BenchmarkId::new(format!("depth{}", depth), threads), &threads, |b, &t| {
                b.iter(|| {
                    // New engine per iteration to avoid cross-benchmark state
                    let mut engine = SearchEngine::new(None, 16);
                    // Enable deeper parallelism (YBWC) for benchmark
                    engine.set_ybwc(true, 6);
                    engine.set_ybwc_branch(20);
                    engine.set_ybwc_max_siblings(6);
                    engine.set_ybwc_scaling(6, 4, 2);
                    engine.set_tt_gating(8, 9, 512);
                    let time_limit = match depth { 3 => 600, 5 => 1000, 6 => 1200, 7 => 1500, 8 => 2000, _ => 1000 };
                    let mut id = if t > 1 {
                        IterativeDeepening::new_with_threads(depth, time_limit, None, t)
                    } else {
                        IterativeDeepening::new(depth, time_limit, None)
                    };
                    let _ = id.search(&mut engine, &board, &captured, player);
                });
            });
        }
    }

    group.finish();
    // Snapshot aggregated profiling metrics for this run and write JSON summary
    let m = snapshot_and_reset_metrics();
    let summary = format!(
        "{{\n  \"tag\": \"{}\",\n  \"tt_reads\": {},\n  \"tt_read_ok\": {},\n  \"tt_read_fail\": {},\n  \"tt_writes\": {},\n  \"tt_write_ok\": {},\n  \"tt_write_fail\": {},\n  \"ybwc_batches\": {},\n  \"ybwc_siblings\": {}\n}}\n",
        "criterion_group:parallel_root_search",
        m.tt_try_reads, m.tt_try_read_successes, m.tt_try_read_fails,
        m.tt_try_writes, m.tt_try_write_successes, m.tt_try_write_fails,
        m.ybwc_sibling_batches, m.ybwc_siblings_evaluated
    );
    let out_dir = std::path::Path::new("target/criterion");
    let _ = std::fs::create_dir_all(out_dir);
    let out_path = out_dir.join("metrics-summary.json");
    let _ = std::fs::write(&out_path, summary.as_bytes());
    // Also echo a concise summary line
    println!(
        "metrics summary written: {:?} (tt_reads={}, tt_writes={}, ybwc_batches={}, ybwc_siblings={})",
        out_path, m.tt_try_reads, m.tt_try_writes, m.ybwc_sibling_batches, m.ybwc_siblings_evaluated
    );
}

criterion_group!(benches, bench_root_search);
criterion_main!(benches);


