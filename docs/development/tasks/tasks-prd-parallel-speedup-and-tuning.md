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

- [x] 1.0 Bench configurability and controls
  - [x] 1.1 Document all bench env overrides in PRD and bench file header.
  - [x] 1.2 Add a usage snippet to `docs/release/PERFORMANCE_SUMMARY.md` for quick runs.
  - [x] 1.3 Run a limited bench (depths 7/8; threads 1/4) to validate env parsing.
  - [x] 1.4 Verify `target/criterion/metrics-summary.json` exists and includes TT/YBWC fields.
  - [x] 1.5 Commit documentation updates.

- [ ] 2.0 YBWC activation and sibling parallelism
  - [x] 2.1 Lower/tune `ybwc_min_depth` and widen activation via `ybwc_min_branch`.
  - [x] 2.2 Implement/tune dynamic sibling caps by depth/branching factor.
  - [x] 2.3 Ensure TLS engine pool reuse is engaged for siblings; check allocations.
  - [x] 2.4 Expose sibling cap/scaling via env in benches and validate propagation.
  - [x] 2.5 Re-run deep benches and confirm `ybwc_batches > 0` and `ybwc_siblings > 0`.

- [ ] 3.0 TT contention reduction
  - [x] 3.1 Tighten exact-only gating threshold at shallow depths (configurable).
  - [x] 3.2 Increase per-thread TT buffer flush threshold for batching.
  - [x] 3.3 Audit `try_read`/`try_write` coverage in probes/stores; minimize blocking.
  - [x] 3.4 Capture/store contention metrics in the aggregated JSON (if available) and review.

- [ ] 4.0 Benchmark signal and positions
  - [x] 4.1 Add high-branching positions (or enable a dataset switch) for benches.
  - [x] 4.2 Add env override for per-depth time limits and increase for 7/8.
  - [x] 4.3 Re-run limited sweeps (7/8; 1/4 threads) and capture reports.
  - [x] 4.4 Confirm YBWC metrics non-zero and collect speedup numbers.

- [ ] 5.0 Tuning and validation to ≥3× @ 4 cores
  - [x] 5.1 Sweep YBWC scaling/branch/sibling caps and TT gating; record grid and results.
  - [x] 5.2 Select a configuration achieving ≥3× @ 4 cores (or document gap and follow-ups).
  - [ ] 5.3 Update defaults (safe, conservative) in code or presets if warranted.
  - [x] 5.4 Update `docs/release/PERFORMANCE_SUMMARY.md` and PRD with final results.
  - [ ] 5.5 Re-run correctness and E2E USI tests; commit final changes.


