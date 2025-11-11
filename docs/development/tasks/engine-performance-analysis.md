# Engine Performance Analysis — Material Evaluation (Task 11.0)

**Run Date:** November 9, 2025  
**Command:** `cargo bench --bench material_evaluation_performance_benchmarks --features "legacy-tests,material_fast_loop" -- --sample-size 20`

## Profiling Highlights

- **Hot paths identified:** board scans, hand aggregation, and preset lookups dominated runtime in legacy mode (O(81) traversal per evaluation).
- **Scenario coverage:** Added Criterion groups for heavy board setups, hand-heavy inventories, promoted mixes, and repetitive evaluation loops to mirror real search workloads.
- **Ablation control:** Introduced a scan-only baseline to quantify loop overhead independent of value lookups.

## Key Measurements (ns unless noted)

| Scenario | Legacy Loop | Fast Loop | Delta |
|----------|-------------|-----------|-------|
| Board heavy (default config) | 233 ± 4 | 41.4 ± 0.4 | **-82%**
| Hand heavy | 239 ± 0.5 | 41.4 ± 0.4 | **-83%**
| Promoted mix | 231 ± 0.2 | 41.4 ± 0.4 | **-82%**
| Configurations — Fast Loop toggle | 286.6 (default) | 35.3 (fast preset) | **-88%**
| Ablation (scan only) | 230.8 | 273.9 | +19% (control)

> **Interpretation:** The popcount-based fast loop is ~6.1–7.5× faster across complex scenarios. The ablation benchmark shows the raw scan loop is more expensive than value application, confirming traversal was the primary bottleneck.

## Incremental Update Hooks

- Added `MaterialDelta` and `MaterialEvaluator::apply_delta` to reuse existing scores for make/unmake pipelines.
- Statistics now accept deltas, preserving per-piece contributions and preset usage counters when incremental paths are used.

## Guardrails

- New config knob `enable_fast_loop` keeps the optimization opt-in; default builds retain the legacy traversal.
- Cross-check test (`cargo test --features material_fast_loop material_delta`) compares legacy vs. fast loop scores to prevent parity regressions.

## Next Steps

1. Integrate fast loop toggle into performance-optimized presets (done in code; awaiting rollout validation).
2. Capture nightly CI benchmarks to monitor regression risk once the optimization ships by default.
3. Extend ablation coverage to include hybrid incremental paths once the make/unmake integration lands.

## Telemetry Interpretation

Material telemetry snapshots now surface the following fields:

- `board_contributions` / `hand_contributions`: per-piece tapered aggregates. Track which pieces dominate evaluations when comparing presets.
- `hand_balance`: net advantage from pieces in hand; large swings indicate impending drops.
- `phase_weighted_total`: cumulative score after phase interpolation, useful when correlating with search scores.
- `preset_usage`: counts of `research`, `classic`, and `custom` tables evaluated during the run. Useful for spotting mixed configurations.

Access via `EvaluationStatistics::telemetry()` or the integrated evaluator debug logs.

## Regression Harness

- Edge-case coverage lives in `tests/material_edge_case_tests.rs` (large hands, promoted captures, impasse thresholds, stats resets).
- Deterministic score expectations are codified in `tests/material_regression_tests.rs`.
- Run both suites with `cargo test material_edge_case_tests material_regression_tests` after modifying tables or configuration defaults.

# Engine Performance Analysis — Mobility Evaluation (Task 13.0)

**Run Date:** November 10, 2025  
**Command:** `cargo test mobility_benchmark_snapshot -- --ignored --nocapture`

## Snapshot Metrics

| Scenario | Iterations | Elapsed | Notes |
|----------|------------|---------|-------|
| Cached evaluator (shared move generator & reuse) | 5,000 | 2.26 s | Refactored path (`PositionFeatureEvaluator::evaluate_mobility`) |
| Naive evaluator (per-piece move generator) | 5,000 | 44.37 s | Reconstructed legacy loop for comparison |

> **Speedup:** ~19.7× faster than the naive baseline on an empty board, primarily from eliminating repeated move generation and redundant central/attack scans.

## Observations

- The cached path keeps a single `MoveGenerator` per evaluation and aggregates per-square mobility in linear time, which removes the quadratic blow-up seen in presence of many pieces.  
- Drop mobility evaluation adds negligible overhead on positions without captured pieces; in drop-heavy scenarios it reuses the same aggregated move data.
- Statistics counters continue to track evaluation passes identically between the implementations, preserving telemetry fidelity.
- The performance gap widens further on full game states where the naive version instantiates `MoveGenerator` dozens of times per side; the cached approach scales with the number of legal moves instead of the number of pieces squared.

## Verification

- Functional parity covered by `tests/mobility_evaluation_tests.rs` (`mobility_includes_drop_opportunities`, `mobility_attack_moves_award_bonus`, `mobility_handles_promoted_pieces`).  
- Benchmark harness lives in `tests/mobility_benchmark.rs` (`#[ignore]`d to avoid CI noise but runs locally with the command above).
