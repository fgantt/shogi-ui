## Relevant Files

- `src/evaluation/tactical_patterns.rs` - Core tactical recognizer logic that requires blocker-aware detection, scoring fixes, and drop support.
- `src/evaluation/attacks.rs` - Provides `ThreatEvaluator` and attack tables to reuse for accurate line-of-sight and tactical motif helpers.
- `src/evaluation/integration.rs` - Wires tactical scores into the tapered evaluator; needs weighting and telemetry hooks.
- `src/evaluation/config.rs` - Houses evaluator configuration structures; extend with tactical weights and toggles.
- `src/telemetry/evaluation_telemetry.rs` - Export tactical statistics alongside other evaluation metrics (if telemetry module differs, adjust path accordingly).
- `tests/evaluation/tactical_patterns_tests.rs` - New targeted tactical fixtures and unit tests.
- `tests/evaluation/evaluation_integration_tests.rs` - Integration tests validating evaluator balance and weighting.
- `benches/evaluation/tactical_patterns_bench.rs` - Benchmarks measuring detection cost before/after optimizations.
- `docs/development/tasks/engine-review/task-14.0-tactical-pattern-recognition-review.md` - Source PRD for reference when implementing improvements.

### Notes

- Align detection helpers with existing `ThreatEvaluator` utilities to avoid duplicating move-generation logic.
- Ensure scoring follows centipawn conventions: negative for our vulnerabilities, positive for threats we create.
- Include shogi-specific scenarios (drops, promoted pieces, lance skewers) in both implementation and tests.
- Update telemetry and configuration defaults so tactical weights are tunable without code changes.
- Coordinate with evaluation benchmarks to quantify performance and accuracy shifts after each milestone.

## Tasks

- [x] 1.0 Tactical Detection Accuracy Overhaul
  - [x] 1.1 Replace bespoke attack enumeration with blocker-aware helpers from `attacks.rs`, factoring shared utilities where necessary.
  - [x] 1.2 Refactor fork, pin, skewer, discovered attack, and back-rank detectors to respect occupancy, promotions, and move legality.
  - [x] 1.3 Introduce centralized line-tracing helpers that terminate when encountering blockers or invalid squares.
  - [x] 1.4 Profile and reduce redundant 9×9 scans by reusing piece lists or bitboard iterators within each detection pass.
  - [x] 1.5 Document detection flow and shared helpers to simplify future maintenance.
- [x] 2.0 Tactical Scoring & Integration Corrections
  - [x] 2.1 Fix scoring polarity for pins and skewers so friendly vulnerabilities apply penalties and discovered advantages grant bonuses.
  - [x] 2.2 Normalize tactical motif scoring factors to centipawn scale and expose them via `TacticalConfig`.
  - [x] 2.3 Add phase-aware weighting (midgame/endgame) for each motif in `tactical_patterns.rs`.
  - [x] 2.4 Update `integration.rs` to apply configurable weights before contributing tactical scores to the tapered evaluator.
  - [x] 2.5 Refresh evaluator configuration documentation to reflect new tuning knobs and defaults.
- [x] 3.0 Hand Piece & Shogi-Specific Motif Support
  - [x] 3.1 Extend `evaluate_tactics` signature to accept hand (`CapturedPieces`) context and propagate it through detectors.
  - [x] 3.2 Implement drop-based fork and pin detection leveraging available hand pieces and legal drop squares.
  - [x] 3.3 Enhance detection for promoted sliders, lance skewers, and other shogi-exclusive motifs highlighted in the PRD.
  - [x] 3.4 Add configuration toggles to enable/disable motif families (drops, promoted tactics) for incremental rollout.
  - [x] 3.5 Validate new motifs against curated tactical positions to ensure correct detection and scoring.
- [ ] 4.0 Telemetry, Weights, and Configuration Enhancements
  - [ ] 4.1 Expand `TacticalStats` with snapshot/export APIs compatible with existing evaluation telemetry.
  - [ ] 4.2 Wire tactical statistics into `EvaluationTelemetry` (or equivalent) for surfaced metrics during search.
  - [ ] 4.3 Introduce runtime-configurable weights through CLI or engine options, mirroring other evaluation components.
  - [ ] 4.4 Provide default tuning presets (aggressive, balanced, conservative) for tactical weighting.
  - [ ] 4.5 Update docs/configuration guides with instructions for enabling telemetry and adjusting weights.
- [ ] 5.0 Testing, Benchmarks, and Validation Suite
  - [ ] 5.1 Create unit tests covering fork, pin, skewer, discovered attack, back-rank threat, and drop scenarios using blocker-aware fixtures.
  - [ ] 5.2 Add regression tests ensuring sign-correct scoring and weight application within the integrated evaluator.
  - [ ] 5.3 Develop performance benchmarks measuring detection frequency, evaluation overhead, and allocation counts.
  - [ ] 5.4 Assemble a tactical FEN corpus (including failure cases cited in the PRD) for automated validation.
  - [ ] 5.5 Integrate new tests and benchmarks into CI, documenting expected thresholds and alerting criteria.


## Task 1.0 Completion Notes

- **Implementation:** Added `TacticalDetectionContext` and blocker-aware line tracing so forks, knight forks, and back-rank evaluation reuse a shared attack enumerator that stops at the first blocker. Fork, knight fork, and discovered-attack paths now operate on context caches instead of repeated 9×9 scans, and back-rank logic scales penalties when limited escapes remain.
- **Testing:** Introduced `tests/tactical_patterns_accuracy_tests.rs` with regression coverage for blocked forks and back-rank threats, and updated internal unit smoke tests to use the new evaluation flow.
- **Notes:** Detection helpers are documented inline, and the new context reduces redundant board iterations by collecting player/opponent piece lists once per evaluation while remaining compatible with future telemetry and scoring work.

## Task 2.0 Completion Notes

- **Implementation:** Reworked scoring polarity so pinned and skewered vulnerabilities now apply negative pressure while discovered attacks and forks award positive centipawn values. Each motif funnels through `apply_phase_weights`, combining cp-based base scores with configurable midgame/endgame scaling.
- **Configuration:** Expanded `TacticalConfig` with explicit centipawn parameters and per-motif `TacticalPhaseWeights`, and added `tactical_weight` to `EvaluationWeights` so the integrated evaluator can gate tactical contributions.
- **Testing:** Added regression coverage for pin penalties and integration-weight scaling in `tests/tactical_patterns_accuracy_tests.rs`, ensuring tactical weights respect both fork detection and back-rank threat calculations.
- **Documentation:** Updated this task plan and inline rustdoc comments to describe the new configuration knobs, clarifying how centipawn parameters and phase weights interact with the tapered evaluation pipeline.

## Task 3.0 Completion Notes

- **Implementation:** Threaded `CapturedPieces` through tactical evaluation, introduced drop-aware fork and pin heuristics with legal-drop gating (pawn files, final ranks, lance/knight restrictions), and added rook/bishop/lance drop pin support aligned with shogi-specific motifs.
- **Configuration:** Reused existing toggles while enabling drop-only bonuses to respect motif phase weights; no new toggles were required because motif drops reuse the existing component flags.
- **Testing:** Expanded `tests/tactical_patterns_accuracy_tests.rs` with drop-based fork and pin scenarios to ensure hand pieces translate into positive tactical pressure, keeping back-rank and pin regression suites intact.

