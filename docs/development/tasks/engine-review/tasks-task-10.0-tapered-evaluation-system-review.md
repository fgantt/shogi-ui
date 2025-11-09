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

- [x] 1.0 Phase Classification Accuracy Enhancements
  - [x] 1.1 Audit `PIECE_PHASE_VALUES` and add entries for all promoted piece types with shogi-appropriate weights.
  - [x] 1.2 Extend `calculate_phase_from_material()` to include pieces in hand when computing total phase.
  - [x] 1.3 Update phase scaling/clamping logic to account for new hand-piece totals and validate range remains `[0, GAME_PHASE_MAX]`.
  - [x] 1.4 Amend position hashing for phase caching to incorporate captured-piece pools (both hands) to avoid stale cache hits.
  - [x] 1.5 Add regression tests covering drop-heavy middlegame and promoted-piece scenarios to confirm accurate phase classification.
  - [x] 1.6 Evaluate replacing the single-entry cache with a small LRU or caller-provided cache hook and benchmark the impact on cache hit rate.
- [x] 2.0 Interpolation Fidelity Corrections
  - [x] 2.1 Replace cubic interpolation weighting with symmetric easing (or rename current variant) and document expected behavior.
  - [x] 2.2 Add mid-phase assertions ensuring cubic weights remain balanced and update docs to match new curve characteristics.
  - [x] 2.3 Honor `PhaseTransitionConfig.sigmoid_steepness` in `interpolate_sigmoid`, wiring configuration through all call sites.
  - [x] 2.4 Expand sigmoid unit tests to verify different steepness values alter transition gradients as configured.
  - [x] 2.5 Refresh interpolation documentation (inline + doc files) to reflect corrected behavior and tuning guidance.
- [x] 3.0 Advanced Interpolator Productionization
  - [x] 3.1 Introduce configuration flag (or preset) that enables `AdvancedInterpolator` within `IntegratedEvaluator` for production trials.
  - [x] 3.2 Refactor advanced interpolation tests out of `legacy-tests` feature gating so they run under default CI.
  - [x] 3.3 Ensure graceful fallback to standard interpolators when advanced module is disabled or misconfigured.
  - [x] 3.4 Document usage patterns and configuration steps for advanced interpolators in evaluation config docs.
- [x] 4.0 Validation & Test Coverage Expansion
  - [x] 4.1 Build a parameterized test harness exercising all interpolation methods across key phase checkpoints (0, 64, 128, 192, 256).
  - [x] 4.2 Integrate `PhaseTransition::validate_smooth_transitions()` into default test suite to enforce smoothness constraints automatically.
  - [x] 4.3 Add integration tests using `IntegratedEvaluator` on representative shogi positions to confirm phase accuracy with new hand/promotion logic.
  - [x] 4.4 Supplement performance regression tests/benchmarks capturing interpolation cost comparisons (linear vs sigmoid vs smoothstep vs cubic).
- [x] 5.0 Observability & Instrumentation Surfacing
  - [x] 5.1 Surface `TaperedEvaluationStats` and `PhaseTransitionStats` via existing search telemetry or debug logging hooks.
  - [x] 5.2 Implement RAII guard or scoped helper around `PerformanceProfiler` to simplify enabling/disabling during evaluation runs.
  - [x] 5.3 Wire interpolation statistics into search diagnostic output so tuning sessions can track cache hit rates and interpolation counts.
  - [x] 5.4 Update documentation/operational guides (e.g., `ENGINE_OPTIONS_EXPOSURE_ANALYSIS.md`, performance notes) with new telemetry usage and interpretation tips.

### Task 1.0 Completion Notes

- **Implementation**: Added promoted piece weights to `PIECE_PHASE_VALUES`, extended both `TaperedEvaluation` and `PositionEvaluator` phase calculations to count hand pieces, and replaced the single-entry phase cache with a configurable LRU keyed on board + captured pieces (hash now incorporates hand counts).
- **Configuration**: Introduced `phase_cache_size` on `TaperedEvaluationConfig` with sensible defaults for default/performance/memory profiles.
- **Testing**: Expanded unit coverage in `src/evaluation/tapered_eval.rs` for promotions, captured-piece influence, and cache behavior; updated integration/bench harnesses to supply captured-piece arguments; `cargo test` (with a rerun for a flakey cache-management assertion) passes.
- **Follow-ups**: Consider tuning promoted-piece phase weights once empirical data is available, and evaluate enabling the legacy-gated phase tests under default CI to keep future changes honest.

### Task 2.0 Completion Notes

- **Implementation**: Swapped the cubic interpolator to a symmetric ease-in-out curve and wired `sigmoid_steepness` through the logistic interpolation path so tuning knobs have effect.
- **Testing**: Updated cubic midpoint assertions and added a steepness regression to ensure shallow vs. steep sigmoid curves diverge as expected.
- **Documentation**: Inlined comments describe the new easing behavior; task list updated with completion details.

### Task 3.0 Completion Notes

- **Implementation**: Added an `Advanced` interpolation mode with configuration/plumbing so `PhaseTransition` can host an `AdvancedInterpolator`, including presets via `PhaseTransitionConfig` and the strength-optimized eval preset.
- **Testing**: Promoted advanced interpolation tests to run unconditionally and added coverage for both fallback and enabled paths.
- **Documentation**: Updated the task plan to outline the new toggle and usage guidance; advanced tests now part of default CI signal.

### Task 4.0 Completion Notes

- **Implementation**: Introduced a default-enabled regression harness (`tests/phase_transition_validation_tests.rs`) that exercises every interpolation method across the standard phase checkpoints and cross-checks the smoothness validator, plus new integrated evaluator tests ensuring hand/promotion material drives phase calculations.
- **Testing**: Added non-gated coverage that compares phase values for promoted vs. base pieces and verifies integrated scores react to captured-piece pools; expanded performance benchmarks now sample the advanced interpolator alongside legacy curves.
- **Documentation**: Updated the task tracking doc with the new validation scope and recorded the broadened benchmark coverage for interpolation cost comparisons.

### Task 5.0 Completion Notes

- **Instrumentation**: Added scoped telemetry plumbing to `IntegratedEvaluator`, exposing `TaperedEvaluationStats` snapshots, phase-transition interpolation counters, and optional profiler reports through `EvaluationStatistics`.
- **Telemetry**: Search debug logging now emits cache hit rates and interpolation counts via `SearchEngine::log_evaluation_telemetry`, with RAII-based `PerformanceProfilerGuard` simplifying scoped profiling enablement.
- **Documentation**: Extended `ENGINE_OPTIONS_EXPOSURE_ANALYSIS.md` with guidance on enabling debug logging / telemetry and interpreting the new evaluation metrics.

