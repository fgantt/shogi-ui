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

## Current Results (mean times)

Depth 5:
- 1 thread: 1.427 s
- 2 threads: 0.919 s (≈1.55×)
- 4 threads: 0.946 s (≈1.51×)
- 8 threads: 0.917 s (≈1.56×)

Depth 6:
- 1 thread: 1.405 s
- 2 threads: 0.918 s (≈1.53×)
- 4 threads: 0.946 s (≈1.49×)
- 8 threads: 1.213 s mean (median ≈0.922 s → ≈1.52×); mean impacted by outliers

## Notes and Next Steps

- Shared transposition table (reads + writes) across workers improved reuse and PV consistency, yielding ~1.5× speedups at deeper depths.
- Remaining overhead limits scaling >1.5×; additional gains will likely come from deeper parallelization (beyond root), reducing synchronization/lock contention, and further memory reuse.



