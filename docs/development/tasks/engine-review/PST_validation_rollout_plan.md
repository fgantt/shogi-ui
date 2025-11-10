# PST Validation & Rollout Plan

## Measurement Checklist

Run these steps before promoting any new PST asset:

1. **Loader integrity**
   - `cargo test pst_loader`
   - `cargo test pst_regression_suite`
2. **Evaluation smoke**
   - `cargo test pst_contribution_increases_as_position_reaches_endgame`
   - `cargo test evaluation::integration::tests::test_pst_only_configuration`
3. **Performance guardrails**
   - `cargo bench --bench evaluation_performance_optimization_benchmarks --features legacy-tests`
   - Record evaluator construction timing (target: ≤ baseline + 1%).
4. **Telemetry review**
   - Enable debug logging (`setoption name debug value on`)
   - Collect `[EvalTelemetry] pst_total` / `pst_avg` snapshots across 20 engine games
   - Export statistics via `EvaluationStatistics::export_json()`

Document results in `docs/tuning/runs/<YYYY-MM-DD>-<tag>.md`.

## Regression Suite

The automated regression harness lives in `tests/pst_regression_suite.rs`. It verifies
that:

- The loader’s default preset produces identical PST contributions to the built-in tables.
- Representative boards (opening/middlegame/endgame) stay bit-for-bit stable across loader and built-in data.

Extend the suite whenever:

- New positional tunings change default tables.
- Additional feature-specific boards need coverage (e.g., drop-heavy positions).

## Rollout Procedure

1. **Prepare artifacts**
   - Export new JSON tables with `cargo run --bin pst_tuning_runner`.
   - Store under `config/pst/experiments/<tag>.json`.
2. **Internal trials**
   - Run validation checklist above.
   - Capture telemetry deltas vs. baseline; ensure net pst contribution shift < ±30 cp unless intentionally targeted.
3. **Staged deployment**
   - Ship experimental tables behind `PSTPreset=Custom` for research builds.
   - Announce availability to evaluation & search teams; ask for A/B match coverage (minimum 1k games).
4. **Production promotion**
   - If accepted, copy JSON to `config/pst/default.json` (or new preset) and regenerate default loader doc examples.
   - Log rollout decision in `docs/tuning/runs/<tag>.md` including match stats and telemetry highlights.
5. **Fallback plan**
   - Keep previous JSON artifacts for at least two release cycles.
   - To revert: `setoption name PSTPreset value Builtin` (or restore prior default JSON).

Following this plan keeps PST adjustments measurable, reproducible, and reversible.

