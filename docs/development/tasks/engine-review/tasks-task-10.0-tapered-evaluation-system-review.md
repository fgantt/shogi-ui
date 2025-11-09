## Relevant Files

- `src/evaluation/tapered_eval.rs` - Coordinates tapered evaluation workflow, phase caching, and integration touchpoints.
- `src/evaluation/phase_transition.rs` - Hosts interpolation kernels (linear, cubic, sigmoid, smoothstep) and validation helpers.
- `src/types.rs` - Defines `TaperedScore`, configuration structs, phase constants, and piece phase weights.
- `src/evaluation/advanced_interpolation.rs` - Experimental spline/multi-phase interpolator implementations pending production integration.
- `src/evaluation/integration.rs` - `IntegratedEvaluator` pipeline that invokes tapered evaluation within broader feature extractors.
- `src/evaluation/performance.rs` - Profiling infrastructure and optimized evaluator path for benchmarking.
- `src/evaluation/config.rs` - Configuration presets referencing tapered evaluation settings.
- `src/evaluation/statistics.rs` - Aggregated evaluation metrics and statistics reporting.
- `tests/evaluation/phase_transition_tests.rs` - Interpolation correctness suite (needs expansion and default CI coverage).
- `tests/evaluation/tapered_eval_integration_tests.rs` - Integration coverage for tapered evaluation (to be created or expanded).
- `docs/development/tasks/engine-review/task-10.0-tapered-evaluation-system-review.md` - Source analysis driving these tasks.

### Notes

- Unit tests should live alongside corresponding modules under `src/evaluation/` whenever possible.
- Integration and regression coverage can target representative shogi positions to capture hand-piece and promotion dynamics.
- Enable or restructure gated tests so CI exercises all interpolation modes without the `legacy-tests` feature flag.
- Use `cargo test --all-features` when validating optional feature paths locally; default CI should rely on `cargo test`.

## Tasks

- [ ] 1.0 Phase Classification Accuracy Enhancements
  - [ ] 1.1 Audit `PIECE_PHASE_VALUES` and add entries for all promoted piece types with shogi-appropriate weights.
  - [ ] 1.2 Extend `calculate_phase_from_material()` to include pieces in hand when computing total phase.
  - [ ] 1.3 Update phase scaling/clamping logic to account for new hand-piece totals and validate range remains `[0, GAME_PHASE_MAX]`.
  - [ ] 1.4 Amend position hashing for phase caching to incorporate captured-piece pools (both hands) to avoid stale cache hits.
  - [ ] 1.5 Add regression tests covering drop-heavy middlegame and promoted-piece scenarios to confirm accurate phase classification.
  - [ ] 1.6 Evaluate replacing the single-entry cache with a small LRU or caller-provided cache hook and benchmark the impact on cache hit rate.
- [ ] 2.0 Interpolation Fidelity Corrections
  - [ ] 2.1 Replace cubic interpolation weighting with symmetric easing (or rename current variant) and document expected behavior.
  - [ ] 2.2 Add mid-phase assertions ensuring cubic weights remain balanced and update docs to match new curve characteristics.
  - [ ] 2.3 Honor `PhaseTransitionConfig.sigmoid_steepness` in `interpolate_sigmoid`, wiring configuration through all call sites.
  - [ ] 2.4 Expand sigmoid unit tests to verify different steepness values alter transition gradients as configured.
  - [ ] 2.5 Refresh interpolation documentation (inline + doc files) to reflect corrected behavior and tuning guidance.
- [ ] 3.0 Advanced Interpolator Productionization
  - [ ] 3.1 Introduce configuration flag (or preset) that enables `AdvancedInterpolator` within `IntegratedEvaluator` for production trials.
  - [ ] 3.2 Refactor advanced interpolation tests out of `legacy-tests` feature gating so they run under default CI.
  - [ ] 3.3 Ensure graceful fallback to standard interpolators when advanced module is disabled or misconfigured.
  - [ ] 3.4 Document usage patterns and configuration steps for advanced interpolators in evaluation config docs.
- [ ] 4.0 Validation & Test Coverage Expansion
  - [ ] 4.1 Build a parameterized test harness exercising all interpolation methods across key phase checkpoints (0, 64, 128, 192, 256).
  - [ ] 4.2 Integrate `PhaseTransition::validate_smooth_transitions()` into default test suite to enforce smoothness constraints automatically.
  - [ ] 4.3 Add integration tests using `IntegratedEvaluator` on representative shogi positions to confirm phase accuracy with new hand/promotion logic.
  - [ ] 4.4 Supplement performance regression tests/benchmarks capturing interpolation cost comparisons (linear vs sigmoid vs smoothstep vs cubic).
- [ ] 5.0 Observability & Instrumentation Surfacing
  - [ ] 5.1 Surface `TaperedEvaluationStats` and `PhaseTransitionStats` via existing search telemetry or debug logging hooks.
  - [ ] 5.2 Implement RAII guard or scoped helper around `PerformanceProfiler` to simplify enabling/disabling during evaluation runs.
  - [ ] 5.3 Wire interpolation statistics into search diagnostic output so tuning sessions can track cache hit rates and interpolation counts.
  - [ ] 5.4 Update documentation/operational guides (e.g., `ENGINE_OPTIONS_EXPOSURE_ANALYSIS.md`, performance notes) with new telemetry usage and interpretation tips.

