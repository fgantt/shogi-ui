## Relevant Files

- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/position_features.rs` - Core position feature evaluator, configuration flags, and statistics counters.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/integration.rs` - Integrates position feature scores into the tapered evaluation pipeline.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/config.rs` - Declares configuration presets and feature gating options.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/moves/mod.rs` - Move generation utilities required for mobility analysis.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/types.rs` - Shared evaluation types, `TaperedScore`, captured piece structures, and telemetry structs.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/telemetry.rs` - (Add or update) Expose position feature statistics in engine telemetry.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/tests/position_features_tests.rs` - (Add) Unit tests for configuration gating and heuristic scoring edge cases.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/tests/evaluation_integration_tests.rs` - (Add) Integration tests covering castles, mobility, and feature toggles.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/benches/evaluation_benchmarks.rs` - (Update) Benchmarks to measure mobility and evaluation throughput.
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/docs/development/engine-performance-analysis.md` - (Update) Document performance impact and measurement methodology.

### Notes

- Unit and integration tests should run without requiring the `legacy-tests` feature gate.
- Prefer reusing existing move generator instances or caches to avoid repeated allocations during evaluation.
- Capture telemetry snapshots after major refactors to validate runtime statistics before deployment.

## Tasks

- [x] 1.0 Restore Position Feature Configuration Fidelity
  - [x] 1.1 Audit every evaluator entry point and short-circuit when the corresponding `PositionFeatureConfig` flag is disabled.
  - [x] 1.2 Ensure statistics counters increment only when a feature actually runs; avoid skewing metrics when disabled.
  - [x] 1.3 Update `IntegratedEvaluator::evaluate_standard` to respect sub-feature toggles and propagate per-feature weights if defined.
- [x] 1.4 Add regression tests verifying that disabling each feature returns a zero score and leaves stats untouched.
- [x] 1.5 Update configuration presets and documentation to reflect working toggles and defaults.

- [x] 2.0 Refactor Mobility Evaluation for Performance and Hand Pressure
  - [x] 2.1 Replace per-piece instantiation of `MoveGenerator` with a shared or cached generator per evaluation pass.
  - [x] 2.2 Cache legal move lists or introduce pseudo-legal counting to avoid O(n²) regeneration per piece.
  - [x] 2.3 Incorporate hand piece mobility by evaluating drop opportunities (e.g., rook, bishop, pawn drops) with appropriate weights.
  - [x] 2.4 Rebalance mobility weights and restriction penalties to avoid over-penalizing castle defenders and promoted minors.
  - [x] 2.5 Add benchmarks measuring evaluation time before and after the mobility refactor; capture results in `engine-performance-analysis.md`.
  - [x] 2.6 Write unit tests covering mobility scores for on-board pieces and hand drops, including promoted piece scenarios.

- [x] 3.0 Add Shogi-Specific King Safety and Pawn Structure Heuristics
  - [x] 3.1 Treat promoted defenders (Tokin, promoted Silver, promoted Knight, etc.) as Gold-equivalent when computing king shields and pawn cover.
  - [x] 3.2 Incorporate hand piece coverage into king safety scoring (e.g., potential drops that guard adjacent squares or attack the king).
  - [x] 3.3 Introduce castle pattern recognition (Mino, Yagura, Anaguma) and adjust safety scores accordingly.
  - [x] 3.4 Update pawn structure evaluation to handle hand-supported chains, illegal double pawns, and shogi-specific advancement scales.
  - [x] 3.5 Revise passed pawn detection to account for opposing hand drops and promoted blockers.
  - [x] 3.6 Create test fixtures validating shogi-specific king safety and pawn structure adjustments across common castles and attack patterns.

- [x] 4.0 Modernize Center Control and Development Signals
  - [x] 4.1 Replace occupancy-based center scoring with attack map analysis that differentiates active control from passive placement.
  - [x] 4.2 Extend control heuristics to cover key edge files and castle anchor squares with phase-aware scaling.
  - [x] 4.3 Add penalties for undeveloped Golds, Silvers, and knights stuck on their starting ranks; include promotion-aware adjustments.
  - [x] 4.4 Ensure development bonuses decay or reverse when promoted pieces retreat to back ranks without purpose.
  - [x] 4.5 Provide targeted unit tests comparing center/development scores across standard opening setups and stalled formations.

- [ ] 5.0 Expand Instrumentation, Testing, and Documentation Coverage
  - [ ] 5.1 Surface `PositionFeatureStats` via evaluation telemetry and allow opt-in/opt-out collection through configuration.
  - [ ] 5.2 Migrate critical legacy tests into the default test suite and add coverage for new configuration and hand-piece scenarios.
  - [ ] 5.3 Add integration tests verifying combined effects of king safety, pawn structure, and mobility in representative midgame positions.
  - [ ] 5.4 Update developer documentation with instructions for enabling/disabling features, interpreting telemetry, and running new benchmarks.
  - [ ] 5.5 Introduce shared caching for king locations, pawn collections, and other reusable feature inputs to avoid repeated board scans across evaluators.
  - [ ] 5.6 Establish CI hooks to run the expanded tests and benchmarks (where feasible) to prevent regressions.
  - [ ] 5.7 Track post-refactor evaluation performance in telemetry dashboards and document findings for future tuning.

## Task 1.0 Completion Notes

- **Implementation:** `src/evaluation/position_features.rs` now short-circuits every public evaluator when the corresponding `PositionFeatureConfig` flag is disabled and only increments statistics for executed features. `PositionFeatureEvaluator` exposes `set_config`, and `IntegratedEvaluator` (via `IntegratedEvaluationConfig`) propagates both per-feature toggles and `EvaluationWeights`, multiplying each `TaperedScore` contribution accordingly.
- **Testing:** Added `tests/position_feature_config_tests.rs` covering disabled-feature scoring/stat counters and weight propagation through the integrated evaluator (`cargo test position_feature_config_tests`). A full `cargo test` run currently surfaces pre-existing failures in `evaluation::advanced_interpolation::tests::test_bezier_endpoints`, `evaluation::config::tests::test_strength_optimized`, and `evaluation::material::tests::test_material_preset_usage_tracking`; new tests pass.
- **Documentation:** Updated `docs/development/tasks/engine-review/task-13.0-position-features-evaluation-review.md` (Section 8) to note the restored configuration fidelity and integration behavior.

## Task 2.0 Completion Notes

- **Implementation:** `PositionFeatureEvaluator` now keeps a shared `MoveGenerator` and aggregates mobility (including drop opportunities) in a single pass over legal moves. Updated weighting tables reduce penalties on castle defenders/promoted minors and boost hand pressure signals. Integration in `src/evaluation/integration.rs` reuses the cached evaluator while multiplying per-feature weights.  
- **Performance:** Local benchmark (`cargo test mobility_benchmark_snapshot -- --ignored --nocapture`) shows the cached evaluator completing 5,000 mobility evaluations in 2.26 s versus 44.37 s for a reconstructed naive loop (~19.7× speedup). Results logged in `docs/development/tasks/engine-performance-analysis.md`.  
- **Testing:** Added `tests/mobility_evaluation_tests.rs` to cover drop mobility, attack bonuses, and promoted piece behaviour. Benchmark harness lives in `tests/mobility_benchmark.rs` for repeatable perf measurements. Existing suites continue to pass aside from known unrelated legacy failures.

## Task 3.0 Completion Notes

- **Implementation:** King-safety heuristics now treat promoted defenders as gold-equivalents, recognise castle guard placements with shared move resources, and account for both friendly and enemy drop pressure (pawns, golds, silvers, lances, knights). Pawn-structure scoring rewards hand-supported chains, applies shogi-oriented advancement tables, and reduces passed-pawn value when the opponent can interpose drops or promoted blockers.  
- **Testing:** Extended `tests/king_safety_and_pawn_structure_tests.rs` with coverage for tokins near the king, gold-in-hand chain completion, and enemy knight drop threats. Existing suites continue to run under the default feature set.  
- **Documentation:** Updated `docs/development/tasks/engine-review/task-13.0-position-features-evaluation-review.md` to reflect the new heuristics and mitigated issues identified in the review.
- **Performance:** Eliminated per-evaluation `HashSet` allocations and now reuse stack-allocated bitsets for hand-supported chains; Criterion (`cargo bench --bench position_features_performance_benchmarks --features "legacy-tests" king_safety pawn_structure`) reports ~0.64–0.89 µs per side, matching pre-refactor throughput.

## Task 4.0 Completion Notes

- **Implementation:** Center control now builds attack maps via the shared `MoveGenerator`, compares control strength on core, extended, and edge files, and recognises castle anchor squares with gold-equivalent bonuses. Development scoring penalises home-rank Golds/Silvers/Knights, rewards forward deployment, and applies retreat penalties to promoted pieces that fall back into the rear ranks.  
- **Testing:** Added `tests/center_control_development_tests.rs` validating attack-map scoring, edge pressure, undeveloped knights, and retreating promoted defenders.  
- **Documentation:** Refreshed `task-13.0-position-features-evaluation-review.md` to capture the new approach for center/development heuristics.



