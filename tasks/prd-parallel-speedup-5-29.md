# PRD: Parallel Search Speedup (Task 5.29)

## Introduction / Overview
Parallel root and intra-node search have been integrated using Rayon and initial YBWC hooks. Benchmarks currently show limited scaling, especially at deeper depths (7/8). The goal of this PRD is to define clear steps to achieve ≥3× speedup on 4 cores for representative mid/deep searches, with robust measurement and tuning knobs.

## Goals
- Achieve ≥3× throughput speedup on 4 cores (vs. 1 core) on representative search benchmarks.
- Exercise deeper-node parallelism (YBWC) such that ybwc metrics are non-zero and significant under benchmark conditions.
- Reduce contention from the shared transposition table (TT) via gating and buffering without harming correctness.
- Provide configurable, reproducible benchmarks with low noise and summarized metrics.

## User Stories
- As an engine developer, I want configurable benches and telemetry to quickly evaluate scaling changes so I can iterate on YBWC/TT parameters with confidence.
- As a user of the built-in engine, I want the engine to think faster on multi-core machines without impacting move quality.

## Functional Requirements
1. Bench configurability
   1. Support env overrides for bench depths and thread counts.
   2. Support env overrides for YBWC scaling, branch threshold, max siblings, min activation depth.
   3. Support env overrides for TT write gating (exact-only threshold, min store depth, buffer flush threshold).
   4. Gate USI info during benches via `SHOGI_SILENT_BENCH=1` to reduce noise.
2. Metrics & reporting
   1. Aggregate TT/YBWC metrics across a run and write concise JSON to `target/criterion/metrics-summary.json`.
   2. Report ybwc batches and siblings; these must be > 0 for deep-depth runs post-tuning.
   3. Document how to run tuned benches and where to find reports.
3. YBWC improvements
   1. Trigger YBWC beyond root at practical depths (configurable `ybwc_min_depth`).
   2. Cap parallelized siblings and enable dynamic caps based on depth/branching factor.
   3. Reuse per-thread `SearchEngine` instances for sibling evaluation (thread-local pool) to reduce overhead.
4. TT contention reductions
   1. Gate shallow writes (exact-only at/under configured depth value).
   2. Buffer per-thread writes and flush periodically and/or at node boundaries.
   3. Prefer non-blocking `try_read`/`try_write` with fallbacks to avoid stalls.
5. Correctness & stability
   1. Retain existing correctness suites; parallel benches should not break move legality or invariants.
   2. Keep panic handling and fallback paths intact.

## Non-Goals (Out of Scope)
- Changing evaluation function/heuristics to gain speedups.
- GPU offloading or multi-process clustering.
- External, large tablebases; focus remains on built-in micro tablebase and core search.

## Design Considerations
- YBWC overhead must be outweighed by additional parallel work; prefer coarse-grained batching at deeper nodes.
- Keep thresholds configurable to allow per-hardware tuning.
- Minimize lock contention and cross-thread memory traffic; prefer local buffers and fewer writes.

## Technical Considerations
- Rayon thread pool stack size is already increased (8MB) to avoid stack overflows.
- Bench harness now accepts env vars:
  - `SHOGI_BENCH_DEPTHS` (e.g., `7,8`)
  - `SHOGI_BENCH_THREADS` (e.g., `1,4`)
  - `SHOGI_YBWC_SCALING` (e.g., `6,4,2`)
  - `SHOGI_YBWC_BRANCH` (e.g., `20`)
  - `SHOGI_YBWC_MAX_SIBLINGS` (e.g., `6`)
  - `SHOGI_YBWC_MIN_DEPTH` (e.g., `6`)
  - `SHOGI_TT_GATING` (e.g., `8,9,512`)
  - `SHOGI_SILENT_BENCH=1` and `SHOGI_AGGREGATE_METRICS=1`

## Current Results Summary (Baseline)
- Depth 5/6: ~1.5× at 2–8 threads; below the ≥3× target on 4 cores.
- Depth 7/8: ≤1× at 4 threads under current time limits/position; `ybwc_batches=0`, `ybwc_siblings=0` → deep-node parallelism not engaging.

## Open Questions
- Hardware matrix: which representative machines/cores to target beyond the current dev host?
- Acceptable single-thread regression (if any) to achieve better parallel scaling?
- Benchmark positions: should we include positions with higher branching factor or forced sequences to exercise YBWC?

## Success Metrics
- Primary: Speedup ≥3.0× at 4 cores (vs. 1 core) on at least one representative deep benchmark (depth 7/8) with stable variance.
- Secondary: ybwc_batches and ybwc_siblings noticeably > 0; TT write attempts reduced while maintaining strength.
- Stability: No correctness regressions in existing tests; benches reproducible (95% CI not overlapping baseline sufficiently to mask improvement).

## Acceptance Criteria
1. Criterion benches run with env-config produce JSON metrics and HTML reports; docs updated with usage.
2. On the dev machine, a tuned configuration achieves ≥3.0× at 4 cores for a chosen deep benchmark (document exact env and position/depth).
3. Metrics show non-zero YBWC activity at deep nodes.
4. No failing tests; E2E USI and parallel suites pass.

## Proposed Plan / Next Steps
1. Exercise deeper-node parallelism
   - Lower `ybwc_min_depth` (e.g., 4–5), and widen activation via `ybwc_min_branch`.
   - Increase `ybwc_max_siblings` and implement dynamic sibling caps based on depth and branching factor.
2. Reduce contention further
   - Stricter TT exact-only gating threshold at shallow depths.
   - Increase write buffer sizes and flush at strategic points (end of node/sibling batches).
3. Improve benchmark signal
   - Increase bench time limits for depths 7/8 or select positions with higher branching factor.
   - Keep `SHOGI_SILENT_BENCH=1`; ensure metrics aggregation prints once.
4. Iterate thresholds
   - Compare scaling sets: (6,4,2), (5,3,2), (6,4,2) with different branch/sibling caps (e.g., branch 12–24, siblings 6–12).
   - Record results and update `docs/release/PERFORMANCE_SUMMARY.md` and task doc.
5. Finalize
   - Lock a default configuration yielding ≥3× on 4 cores; keep tunables available for per-machine overrides.

## How to Run
- Example (limited, deep-focused):
```
SHOGI_BENCH_DEPTHS=7,8 \
SHOGI_BENCH_THREADS=1,4 \
SHOGI_YBWC_MIN_DEPTH=5 \
SHOGI_YBWC_SCALING=6,4,2 \
SHOGI_YBWC_BRANCH=16 \
SHOGI_YBWC_MAX_SIBLINGS=8 \
SHOGI_TT_GATING=8,10,1024 \
SHOGI_SILENT_BENCH=1 \
cargo bench --bench parallel_search_performance_benchmarks
```
- Reports: `target/criterion/*` and `target/criterion/metrics-summary.json`.
