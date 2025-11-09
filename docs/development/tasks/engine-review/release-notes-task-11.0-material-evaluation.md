# Release Notes — Task 11.0 Material Evaluation Modernization

## Summary
- Functional configuration surface: `use_research_values`, `values_path`, and `enable_fast_loop` now shape runtime behaviour across evaluators.
- Material values externalized with schema-backed JSON/TOML assets, custom loaders, and regression harnesses.
- Telemetry enriched with per-piece contributions, preset usage counters, and parity cross-checks for optimized traversal.
- Material telemetry monitoring plan documented in `docs/monitoring/material-evaluation-monitoring.md`, covering dashboards, alerts, and incident workflow.

## Key Changes
- New `MaterialValueSet` abstraction with built-in `research` and `classic` presets plus on-disk schema under `resources/material/`.
- `MaterialEvaluator` supports opt-in fast loop traversal (`enable_fast_loop`) and incremental deltas (`MaterialDelta`).
- `IntegratedEvaluator` and `OptimizedEvaluator` honour `MaterialEvaluationConfig` updates at initialization and runtime.
- Benchmark suite (`material_evaluation_performance_benchmarks.rs`) expanded to cover heavy board/hand scenarios, ablation, and legacy vs. fast loop comparisons.
- Documentation updated: configuration guide, performance analysis, value-set guides, and tuning workflows.
- Regression tests added (`material_edge_case_tests`, `material_regression_tests`) to ensure determinism across presets and edge cases.

## Migration Guidance
- Existing configs continue to default to the research preset with hand pieces enabled. No changes required to preserve previous behaviour.
- Replace source-level table edits with external files referenced via `values_path`.
- Enable `enable_fast_loop` only after running `cargo test --features material_fast_loop material_delta` to verify parity.

## Verification Checklist
- `cargo test material_edge_case_tests material_regression_tests`
- `cargo test --features material_fast_loop material_delta material_stats`
- `cargo bench --bench material_evaluation_performance_benchmarks --features "legacy-tests,material_fast_loop"`

## Rollout Notes
- Keep fast loop disabled by default in production until monitoring dashboards confirm stability.
- Capture telemetry snapshots (material contributions, preset usage) during staged rollout for comparison with baseline runs.
- Coordinate with tuning teams to publish updated value-set workflows and archive previous preset versions.
