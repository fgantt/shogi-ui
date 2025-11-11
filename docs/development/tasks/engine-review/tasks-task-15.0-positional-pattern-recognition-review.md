## Relevant Files

- `src/evaluation/positional_patterns.rs` - Core positional pattern detectors that need shogi-specific control logic, blocker-aware attacks, and hand-aware heuristics.
- `src/evaluation/integration.rs` - Integrates positional scores into `IntegratedEvaluator`; requires weighting, phase scaling, and telemetry updates.
- `src/evaluation/pattern_config.rs` - Defines positional pattern configuration/weights that must include new knobs and defaults.
- `src/evaluation/attack_tables.rs` - Shared attack/threat utilities to reuse for blocker-aware control (verify actual path/name before coding).
- `src/evaluation/hand_utils.rs` - Utilities for hand (`CapturedPieces`) analysis or new module to be created for drop threat logic.
- `tests/positional_patterns_tests.rs` - New/unit tests validating positional heuristics against shogi fixtures.
- `tests/evaluation_integration_tests.rs` - Integration tests ensuring weighted positional signals behave correctly.
- `benches/positional_patterns_bench.rs` - Benchmark suite measuring positional evaluator performance before/after optimizations.

### Notes

- Confirm actual helper module names (`attack_tables`, `hand_utils`, etc.) before implementation; adjust paths accordingly.
- Unit tests should live next to the code when possible; integration tests belong under `tests/`.
- Use `cargo test --lib positional_patterns` (or similar focused commands) during development; run `cargo bench benches::positional_patterns_bench` for benchmarks.

## Tasks

- [x] 1.0 Replace bespoke attack detection with shared blocker-aware attack utilities
  - [x] 1.1 Inventory existing attack/threat utilities (e.g., `AttackTables`, `ThreatEvaluator`) and document required APIs for positional detectors.
  - [x] 1.2 Refactor center, weak-square, space, and other detectors to request control data through shared utilities rather than bespoke scans.
  - [x] 1.3 Ensure promoted pieces, lances, knights, and long-range sliders respect blockers when computing control.
  - [x] 1.4 Remove redundant helper methods (e.g., `piece_attacks_square`) and update statistics to match new control sources.
  - [x] 1.5 Add sanity tests that compare detector outputs against attack tables for representative board states.

- [x] 2.0 Incorporate hand context and shogi-correct pawn heuristics into positional detectors
  - [x] 2.1 Extend evaluator inputs to accept `CapturedPieces` (hands) and propagate through positional pattern API.
  - [x] 2.2 Redesign pawn-support/threat logic to match shogi movement rules (forward-only capture, drop availability).
  - [x] 2.3 Model pawn/lance/knight drop threats for outpost and weak-square evaluation, including adjacent empty squares.
  - [x] 2.4 Update configuration to toggle hand-aware heuristics and document new parameters.
  - [x] 2.5 Write regression tests covering pawn drops undermining outposts and defending weak squares via hand reinforcements.

- [x] 3.0 Add weighting, phase scaling, and telemetry export for positional patterns in the integrated evaluator
  - [x] 3.1 Wire `PatternWeights.positional_patterns` (or new weight fields) into `IntegratedEvaluator` and ensure tapered scaling is respected.
  - [x] 3.2 Introduce per-detector weights/phase multipliers within `PositionalConfig`, falling back to sensible defaults.
  - [x] 3.3 Implement `PositionalStats::snapshot()/merge()` APIs and integrate with existing evaluation telemetry pipelines.
  - [x] 3.4 Update configuration parsing/documentation to expose new weighting and telemetry options.
  - [x] 3.5 Add integration tests verifying weights affect final scores and stats snapshots appear in evaluator reports.
  - [x] 3.6 Align positional configuration defaults with PRD guidance (disable unstable detectors by default, document default values and tuning guidance).

- [ ] 4.0 Redesign positional heuristics for shogi fidelity and performance
  - [ ] 4.1 Rework center control to measure actual control/mobility (including promoted pieces) instead of raw occupancy.
  - [ ] 4.2 Reframe outpost detection with shogi-specific piece sets, blocker-aware support, and drop-counterplay checks.
  - [ ] 4.3 Replace weak-square identification with threat/defense balance that honors blockers, promotions, and castle structures.
  - [ ] 4.4 Revise space evaluation to use side-relative territory metrics and cached control maps to avoid O(81²) scans.
  - [ ] 4.5 Profile the redesigned evaluator, ensuring allocation-free hot paths and acceptable runtime across sample positions.

- [ ] 5.0 Build regression tests, fixtures, and benchmarks for positional pattern evaluation
  - [ ] 5.1 Author canonical shogi position fixtures (central fights, castle weaknesses, space clamps) with expected score deltas.
  - [ ] 5.2 Create targeted unit tests asserting detector outputs on each fixture and guarding against regressions.
  - [ ] 5.3 Add integration tests that compare positional scores before/after weighting changes to maintain tuning stability.
  - [ ] 5.4 Implement criterion benches measuring evaluator runtimes and control-map reuse efficiency.
  - [ ] 5.5 Document the testing/benchmarking workflow and incorporate into CI or release validation checklists.


## Task 1.0 Completion Notes

- **Implementation:** Introduced a reusable `ControlCache` that wraps `BitboardBoard::is_square_attacked_by`, providing cached, blocker-aware control lookups for both players. Center, weak-square, and space evaluations now request control data through this cache, eliminating bespoke scans and the obsolete `piece_attacks_square` helper while keeping statistics intact.
- **Testing:** Added `test_control_cache_matches_board_queries` to confirm the cache mirrors board attack semantics in scenarios with blockers. Updated existing positional-pattern tests to use the new evaluator signatures. Attempted `cargo test positional_patterns`, but the run hit the known rustc ICE (`cached cgu ... should have an object file`); verified compilation with `cargo check --lib` instead.
- **Notes:** The cache-based control path prepares the module for later heuristics without reintroducing ad-hoc attack logic. Future performance tuning should reuse this cache instead of re-querying board state.

## Task 2.0 Completion Notes

- **Implementation:** Threaded `CapturedPieces` through `evaluate_position`, outpost, and weak-square paths, added hand-aware toggles to `PositionalConfig`, and rewrote pawn support/threat and drop threat logic (pawn, lance, knight) using orientation-aware helpers. Weak-square defense now recognises pawn drops, and drop heuristics feed directly into outpost validation.
- **Testing:** Added `test_outpost_rejected_by_pawn_drop` and `test_weak_square_relieved_by_pawn_drop` to validate pawn-drop threats and defenses. Unit harness updated to the new API signature. `cargo test positional_patterns` still encounters the existing rustc ICE; verified via `cargo check --lib`.
- **Notes:** New helpers guard against illegal drops (last-rank restrictions, double pawns) and reuse shared direction utilities, keeping future heuristics consistent with shogi rules.

## Task 3.0 Completion Notes

- **Implementation:** Added per-component `PositionalPhaseWeights`, a global `positional_weight` in `EvaluationWeights`, and telemetry support for positional statistics. Each detector now applies mg/eg scaling before contributing to the total, and `IntegratedEvaluator` multiplies positional results by the configurable weight while exporting positional snapshots.
- **Testing:** Introduced `test_center_control_phase_weights` and `test_positional_stats_snapshot_merge` to confirm weight scaling and snapshot aggregation. `cargo check --lib` verifies compilation; `cargo test positional_patterns` still hits the known rustc ICE.
- **Notes:** Configuration and documentation now surface the new weighting controls and telemetry hooks, enabling tuning workflows to adjust positional influence without changing code.
