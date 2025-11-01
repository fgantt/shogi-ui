# Parallel Search Performance Summary

This document summarizes benchmark results and how to run/interpret them for the parallel search implementation.

## How to Run

1. Ensure release build:
   - `cargo build --release`
2. Run the Criterion benchmarks for parallel search:
   - `cargo bench --bench parallel_search_performance_benchmarks`

## Where to View Reports

Criterion stores HTML reports under `target/criterion/`.

- Depth 5: `target/criterion/parallel_root_search/depth5/{1,2,4,8}/report/index.html`
- Depth 6: `target/criterion/parallel_root_search/depth6/{1,2,4,8}/report/index.html`

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

## Notes and Next Steps

- Shared transposition table (reads + writes) across workers improves reuse and PV consistency.
- This run shows limited speedup (best ≈1.21× at 8 threads). TT write gating alone isn’t sufficient; contention and work granularity still dominate. Deeper parallelism and reduced shared writes should help.
- Next steps to hit ≥3× on 4 cores:
  - Gate shared TT writes (write-back only for exact or deep entries); buffer writes per-thread and flush periodically
  - Reduce shared TT lock scope; prefer try_read/try_write + skip on contention
  - Increase task granularity: parallelize deeper siblings (YBWC cut nodes) not just root; tune with_min_len per depth
  - Reuse/arena allocate per-thread buffers to minimize alloc traffic during make/undo paths



