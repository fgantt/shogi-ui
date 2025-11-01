## Relevant Files

- `benches/parallel_search_performance_benchmarks.rs` - Criterion benches and env-config overrides for depths/threads/YBWC/TT.
- `src/search/parallel_search.rs` - ParallelSearchEngine, work-stealing/YBWC activation points and metrics.
- `src/search/search_engine.rs` - Iterative deepening integration, YBWC/TT gating knobs and counters.
- `docs/release/PERFORMANCE_SUMMARY.md` - Performance results and notes; to be updated with new runs.
- `docs/development/tasks/tasks-prd-parallel-search.md` - Parallel search task tracker; cross-reference for 5.29.
- `tests/parallel_*` and `tests/usi_e2e_tests.rs` - Guardrails for correctness and USI behavior during tuning.

### Notes

- Use `SHOGI_SILENT_BENCH=1` and `SHOGI_AGGREGATE_METRICS=1` during benches to minimize noise and capture metrics.
- Metrics summary JSON is written to `target/criterion/metrics-summary.json` by the bench harness.

## Tasks

- [ ] 1.0 Bench configurability and controls
  - Ensure env overrides (depths/threads/YBWC/TT) are documented and stable across runs.
  - Verify silent bench mode and aggregated metrics output are functioning.

- [ ] 2.0 YBWC activation and sibling parallelism
  - Lower/parameterize `ybwc_min_depth` and widen activation conditions (min branch).
  - Implement/tune dynamic sibling caps; validate reuse of per-thread engine pool.
  - Confirm YBWC metrics (batches/siblings) become non-zero on deep runs.

- [ ] 3.0 TT contention reduction
  - Tighten exact-only gating at shallow depths; increase per-thread buffer thresholds.
  - Audit `try_read`/`try_write` usage to minimize stalls; measure lock/sync counters.

- [ ] 4.0 Benchmark signal and positions
  - Add or select positions with higher branching factor; adjust bench time limits at depths 7/8.
  - Re-run limited sweeps (e.g., depths 7/8; threads 1/4) and confirm YBWC activity.

- [ ] 5.0 Tuning and validation to ≥3× @ 4 cores
  - Sweep YBWC scaling/branch/sibling caps and TT gating; record best configs.
  - Update `docs/release/PERFORMANCE_SUMMARY.md` and PRD with results and chosen defaults.
  - Verify no correctness regressions; finalize default thresholds.


