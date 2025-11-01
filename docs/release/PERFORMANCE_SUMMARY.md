# Parallel Search Performance Summary

This document summarizes benchmark results and how to run/interpret them for the parallel search implementation.

## How to Run

1. Ensure release build:
   - `cargo build --release`
2. Run the Criterion benchmarks for parallel search:
   - `cargo bench --bench parallel_search_performance_benchmarks`

### Quick, configurable runs (env overrides)

- Silence USI output and aggregate metrics:
  - `SHOGI_SILENT_BENCH=1 SHOGI_AGGREGATE_METRICS=1`
- Limit depths/threads for faster runs (examples):
  - `SHOGI_BENCH_DEPTHS=3`
  - `SHOGI_BENCH_THREADS=1,4`
- Tune YBWC/TT in benches:
  - `SHOGI_YBWC_SCALING=6,4,2 SHOGI_YBWC_BRANCH=20 SHOGI_YBWC_MAX_SIBLINGS=6 SHOGI_YBWC_MIN_DEPTH=6`
  - `SHOGI_TT_GATING=8,9,512`
  - `SHOGI_BENCH_SAMPLES=10` (reduce sample size for faster sweeps)

Example quick command:
```
SHOGI_BENCH_DEPTHS=3 SHOGI_BENCH_THREADS=1,4 SHOGI_SILENT_BENCH=1 \
cargo bench --bench parallel_search_performance_benchmarks
```

Metrics JSON: `target/criterion/metrics-summary.json` (written at the end of the bench group).

## Where to View Reports

Criterion stores HTML reports under `target/criterion/`.

- Depth 5: `target/criterion/parallel_root_search/depth5/{1,2,4,8}/report/index.html`
- Depth 6: `target/criterion/parallel_root_search/depth6/{1,2,4,8}/report/index.html`
- Depth 7: `target/criterion/parallel_root_search/depth7/{1,2,4,8}/report/index.html`
- Depth 8: `target/criterion/parallel_root_search/depth8/{1,2,4,8}/report/index.html`

The JSON estimates (for automation) are in `.../base/estimates.json` in each directory.

## What to Look For

- Prefer Mean for overall time comparisons; use Median as a robust alternative when outliers are present.
- Throughput (elem/s) is inversely related to time; higher is better. We benchmark per-root search, so comparing mean times is straightforward.
- Compare 1 thread to 2/4/8 threads to understand scaling.

## Latest Results (mean time, lower is better)

- Automation note: pulled from `target/criterion/parallel_root_search/depth{5,6}/{1,2,4,8}/new/estimates.json` (mean.point_estimate)

Depth 5 (s) after TT write gating/buffering:
- 1 thread: 1.440
- 2 threads: 1.464 (speedup 0.98×)
- 4 threads: 1.833 (speedup 0.79×)
- 8 threads: 1.203 (speedup 1.20×)

Depth 6 (s) after TT write gating/buffering:
- 1 thread: 1.445
- 2 threads: 1.457 (speedup 0.99×)
- 4 threads: 1.686 (speedup 0.86×)
- 8 threads: 1.196 (speedup 1.21×)

Depth 7 (s) with tightened TT gating and YBWC thresholds:
- 1 thread: 1.947
- 2 threads: 2.560 (speedup 0.76×)
- 4 threads: 2.222 (speedup 0.88×)
- 8 threads: 1.728 (speedup 1.13×)

Depth 8 (s) with tightened TT gating and YBWC thresholds:
- 1 thread: 2.287
- 2 threads: 3.608 (speedup 0.63×)
- 4 threads: 2.302 (speedup 0.99×)
- 8 threads: 2.696 (speedup 0.85×)

## Notes and Next Steps

- Shared transposition table (reads + writes) across workers improves reuse and PV consistency.
- Depth 5/6 runs showed limited speedup (best ≈1.21× at 8 threads). At depths 7/8 with stricter gating, current configuration regresses on 2–4 threads and is only modestly positive at 8 threads; likely dominated by overhead (including verbose USI info output during benches) and insufficient deep parallel granularity.
- Next steps to hit ≥3× on 4 cores:
  - Gate shared TT writes (write-back only for exact or deep entries); buffer writes per-thread and flush periodically
  - Reduce shared TT lock scope; prefer try_read/try_write + skip on contention
  - Increase task granularity: parallelize deeper siblings (YBWC cut nodes) not just root; tune with_min_len per depth
  - Reuse/arena allocate per-thread buffers to minimize alloc traffic during make/undo paths
 - Suppress USI info output during Criterion runs to avoid measurement distortion; add a silent mode in benches
 - Raise YBWC depth threshold and refine sibling caps dynamically based on branching factor



