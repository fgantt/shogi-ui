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
