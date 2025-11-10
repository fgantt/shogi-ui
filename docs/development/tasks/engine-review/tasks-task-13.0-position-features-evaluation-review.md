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

- [ ] 1.0 Restore Position Feature Configuration Fidelity
  - [ ] 1.1 Audit every evaluator entry point and short-circuit when the corresponding `PositionFeatureConfig` flag is disabled.
  - [ ] 1.2 Ensure statistics counters increment only when a feature actually runs; avoid skewing metrics when disabled.
  - [ ] 1.3 Update `IntegratedEvaluator::evaluate_standard` to respect sub-feature toggles and propagate per-feature weights if defined.
  - [ ] 1.4 Add regression tests verifying that disabling each feature returns a zero score and leaves stats untouched.
  - [ ] 1.5 Update configuration presets and documentation to reflect working toggles and defaults.

- [ ] 2.0 Refactor Mobility Evaluation for Performance and Hand Pressure
  - [ ] 2.1 Replace per-piece instantiation of `MoveGenerator` with a shared or cached generator per evaluation pass.
  - [ ] 2.2 Cache legal move lists or introduce pseudo-legal counting to avoid O(n²) regeneration per piece.
  - [ ] 2.3 Incorporate hand piece mobility by evaluating drop opportunities (e.g., rook, bishop, pawn drops) with appropriate weights.
  - [ ] 2.4 Rebalance mobility weights and restriction penalties to avoid over-penalizing castle defenders and promoted minors.
  - [ ] 2.5 Add benchmarks measuring evaluation time before and after the mobility refactor; capture results in `engine-performance-analysis.md`.
  - [ ] 2.6 Write unit tests covering mobility scores for on-board pieces and hand drops, including promoted piece scenarios.

- [ ] 3.0 Add Shogi-Specific King Safety and Pawn Structure Heuristics
  - [ ] 3.1 Treat promoted defenders (Tokin, promoted Silver, promoted Knight, etc.) as Gold-equivalent when computing king shields and pawn cover.
  - [ ] 3.2 Incorporate hand piece coverage into king safety scoring (e.g., potential drops that guard adjacent squares or attack the king).
  - [ ] 3.3 Introduce castle pattern recognition (Mino, Yagura, Anaguma) and adjust safety scores accordingly.
  - [ ] 3.4 Update pawn structure evaluation to handle hand-supported chains, illegal double pawns, and shogi-specific advancement scales.
  - [ ] 3.5 Revise passed pawn detection to account for opposing hand drops and promoted blockers.
  - [ ] 3.6 Create test fixtures validating shogi-specific king safety and pawn structure adjustments across common castles and attack patterns.

- [ ] 4.0 Modernize Center Control and Development Signals
  - [ ] 4.1 Replace occupancy-based center scoring with attack map analysis that differentiates active control from passive placement.
  - [ ] 4.2 Extend control heuristics to cover key edge files and castle anchor squares with phase-aware scaling.
  - [ ] 4.3 Add penalties for undeveloped Golds, Silvers, and knights stuck on their starting ranks; include promotion-aware adjustments.
  - [ ] 4.4 Ensure development bonuses decay or reverse when promoted pieces retreat to back ranks without purpose.
  - [ ] 4.5 Provide targeted unit tests comparing center/development scores across standard opening setups and stalled formations.

- [ ] 5.0 Expand Instrumentation, Testing, and Documentation Coverage
  - [ ] 5.1 Surface `PositionFeatureStats` via evaluation telemetry and allow opt-in/opt-out collection through configuration.
  - [ ] 5.2 Migrate critical legacy tests into the default test suite and add coverage for new configuration and hand-piece scenarios.
  - [ ] 5.3 Add integration tests verifying combined effects of king safety, pawn structure, and mobility in representative midgame positions.
  - [ ] 5.4 Update developer documentation with instructions for enabling/disabling features, interpreting telemetry, and running new benchmarks.
  - [ ] 5.5 Introduce shared caching for king locations, pawn collections, and other reusable feature inputs to avoid repeated board scans across evaluators.
  - [ ] 5.6 Establish CI hooks to run the expanded tests and benchmarks (where feasible) to prevent regressions.
  - [ ] 5.7 Track post-refactor evaluation performance in telemetry dashboards and document findings for future tuning.


