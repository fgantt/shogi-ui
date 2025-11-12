## Relevant Files

- `src/evaluation/castles.rs` - Core recognizer entry point; hosts pattern catalog, match analysis, score scaling, and cache management.
- `src/evaluation/patterns/` - Individual castle templates (`anaguma.rs`, `mino.rs`, `yagura.rs`) implemented with parametric, symmetry-aware pattern sources.
- `src/evaluation/castle_geometry.rs` - Shared helpers for anchor offsets, defender classes, and mirror handling.
- `src/evaluation/king_safety.rs` - Consumes castle scores, applies weights, and exposes king-safety telemetry; duplicates placeholder API that must be retired.
- `src/evaluation/attacks/` - Threat evaluation modules that will integrate castle coverage penalties.
- `src/config/king_safety_config.rs` (or equivalent) - Configuration surface for castle weights, penalties, and thresholds.
- `tests/king_safety_tests.rs` & `tests/king_safety_integration_tests.rs` - Legacy suites to be migrated/expanded with real fixtures and partial/broken castles.
- `tests/castle_pattern_tests.rs` (new) - Unit tests covering symmetry, promotion-aware recognition, and cache behavior.
- `benches/castle_recognition_bench.rs` (new) - Benchmarks for pattern matching, cache hit rate, and telemetry overhead.
- `docs/development/tasks/engine-review/fixtures/castle-pattern-schema.md` - Developer guide for the castle pattern schema and extension workflow.

### Notes

- Capture canonical board fixtures (right/left Anaguma, Mino variants, Yagura stages, Snowroof, damaged castles) under version control for reuse across tests and benchmarks.
- Prefer declarative pattern definitions (e.g., JSON/TOML + codegen or static arrays) if it improves readability and configurability; ensure load cost is acceptable.
- Telemetry should flow through an extended `KingSafetyStats` structure so downstream consumers can track castle quality, missing defenders, cache hits, and penalties.
- Coordinate systems must respect player perspective; reuse existing helpers (e.g., `Player::forward_dir`, `mirror_file`) to avoid sign errors.
- When introducing penalties, ensure evaluation remains tapered and integrates with existing king-safety blending logic.

## Tasks

- [x] 1.0 Rebuild Castle Pattern Definitions with Symmetry and Variant Support
  - [x] 1.1 Design a parametric pattern model supporting anchor squares, optional shells, defender classes (e.g., GoldFamily), and left/right mirroring.
  - [x] 1.2 Refactor existing Anaguma/Mino/Yagura templates into the new model, including promoted defenders, drop buffers, and multi-stage shells.
  - [x] 1.3 Create reusable helpers (`castle_geometry.rs`) for relative coordinate transforms, mirroring, and piece-class matching.
  - [x] 1.4 Backfill unit tests validating recognition across mirrored boards, promoted defenders, and variant-specific fixtures.
  - [x] 1.5 Document pattern schema and configuration defaults in developer docs, highlighting extension process for future castles.

- [x] 2.0 Implement Zone-Based Castle Scoring and Exposed-King Penalties
  - [x] 2.1 Replace binary match-quality gate with graded scoring that accounts for defender coverage zones, pawn chains, hand reinforcements, and shell completeness.
  - [x] 2.2 Introduce explicit penalties for exposed kings (e.g., missing primary shell, breached pawn wall) and integrate them into tapered evaluation.
  - [x] 2.3 Extend `KingSafetyConfig` with tunable weights/thresholds for castle bonuses and exposure penalties; add serde + docs.
  - [x] 2.4 Update `king_safety.rs` to consume new scoring outputs, ensuring legacy placeholder APIs are removed or redirected.
  - [x] 2.5 Validate scoring gradients with targeted unit tests (partial castles vs. bare kings) and golden-metric snapshots.

- [x] 3.0 Integrate Castle Telemetry and Remove Duplicate King-Safety APIs
  - [x] 3.1 Audit `KingSafetyEvaluator` and related consumers to eliminate duplicate placeholder methods returning zero.
  - [x] 3.2 Extend telemetry structs and logging (`KingSafetyStats`, trace logs) to capture pattern matched, quality, missing defenders, cache hits/misses, and applied penalties.
  - [x] 3.3 Add debug/trace hooks that can be toggled via config or feature flag for tuning sessions without flooding production logs.
  - [x] 3.4 Update documentation and developer tooling to reference the single, authoritative castle evaluation API and telemetry contract.
  - [x] 3.5 Write integration tests asserting telemetry fields populate correctly for intact, partial, and missing castles.

- [ ] 4.0 Redesign Castle Recognition Cache and Hashing Strategy
  - [ ] 4.1 Define new cache key incorporating king square, local neighborhood hash, hand pieces, and promotion state; add tests covering collision scenarios.
  - [ ] 4.2 Implement configurable LRU cache sized for mid-search workloads, with metrics for hit/miss, evictions, and reuse.
  - [ ] 4.3 Ensure cache respects symmetry (mirrored positions share results where valid) without cross-color leakage.
  - [ ] 4.4 Add instrumentation to record cache effectiveness in benchmarks and expose stats through telemetry.
  - [ ] 4.5 Benchmark cache behavior on recorded game traces to validate sizing assumptions and adjust defaults.

- [ ] 5.0 Expand Castle Regression Coverage, Benchmarks, and CI Integration
  - [ ] 5.1 Build fixture library (possibly under `tests/fixtures/castles/`) with canonical, mirrored, partial, broken, and attacked castle states.
  - [ ] 5.2 Migrate legacy castle tests behind `legacy-tests` into default test suite with assertions on quality scores, penalties, and telemetry.
  - [ ] 5.3 Add integration tests covering castle/attack interactions—ensure penalties respond to open files or mating nets flagged by threat analyzers.
  - [ ] 5.4 Create benchmark suite measuring castle recognition throughput, cache hit rate, and telemetry overhead across opening/middlegame/endgame traces.
  - [ ] 5.5 Update CI configuration to run new tests/benchmarks (or representative subset) and document commands for local verification.

## Completion Notes

### Task 1.0 — Rebuild Castle Pattern Definitions with Symmetry and Variant Support

- **Implementation:** Introduced `castle_geometry.rs` with reusable defender families (`GOLD_FAMILY`, `SILVER_FAMILY`, etc.), `RelativeOffset` mirroring, and descriptor helpers. Refactored `Anaguma`, `Mino`, and `Yagura` pattern modules to publish left/right base and advanced variants that recognise promoted defenders and alternate pawn shells. Updated `CastleRecognizer` to operate on pattern variants, cache best matches, and expose helper constructors for future templates.
- **Testing:** Added symmetry and promotion coverage tests directly in `src/evaluation/castles.rs` along with pattern-specific variant assertions, ensuring recognition succeeds for mirrored boards, promoted silvers, and pawn-wall configurations. Supplemented fixtures with targeted recognition scenarios to guard against regressions.
- **Documentation:** Authored `docs/development/tasks/engine-review/fixtures/castle-pattern-schema.md`, detailing the new schema, defender classes, mirroring workflow, and checklist for adding future castles.

### Task 2.0 — Implement Zone-Based Castle Scoring and Exposed-King Penalties

- **Implementation:** `CastleRecognizer::evaluate_castle` now tracks coverage, shield, buffer, and infiltration metrics via `ZoneMetrics`, returning a rich `CastleEvaluation`. `KingSafetyEvaluator` combines these ratios with configurable weights (`KingSafetyConfig`) to award coverage bonuses, apply missing-defender penalties, and scale exposed-king / infiltration penalties. Pattern descriptors leverage `CastlePieceRole` so primary, secondary, shield, and buffer components feed into the scoring pipeline.
- **Testing:** Expanded king-safety integration tests ensure full, partial, and bare castles produce the expected score ordering while infiltration tests verify enemy pieces inside the king ring lower the result. Castle recognizer unit tests assert quality ratios, missing defender counts, and infiltration detection, covering symmetry and promotion scenarios. Additional regression fixtures guard against future scoring regressions.
- **Documentation:** Updated the castle pattern schema guide to describe zone metrics, defender roles, and the new `KingSafetyConfig` weights. Task documentation now reflects graded scoring, exposure penalties, and telemetry output for tuning teams.

### Task 3.0 — Integrate Castle Telemetry and Remove Duplicate King-Safety APIs

- **Implementation:** Removed duplicate placeholder methods `evaluate_attacks` and `evaluate_threats` from `KingSafetyEvaluator` (actual implementations exist in `AttackAnalyzer` and `ThreatEvaluator`). Introduced `KingSafetyStats` and `KingSafetyStatsSnapshot` structs to track evaluations, castle matches, missing defenders, cache hits/misses, and applied penalties. Added `set_debug_logging` method and trace logging hooks that emit detailed castle evaluation information when enabled. Integrated telemetry into `EvaluationStatistics` and `EvaluationTelemetry` structures, with `record_king_safety_stats` and accessor methods.
- **Testing:** Added comprehensive integration tests (`test_king_safety_stats_track_evaluations`, `test_king_safety_stats_track_castle_matches`, `test_king_safety_stats_track_missing_defenders`, `test_king_safety_stats_track_penalties`, `test_king_safety_stats_track_cache_hits`, `test_king_safety_stats_reset`, `test_king_safety_debug_logging_toggle`) that verify telemetry fields populate correctly for intact, partial, and missing castles, confirming statistics tracking works as expected.
- **Documentation:** The single, authoritative castle evaluation API is `CastleRecognizer::evaluate_castle`, which returns a rich `CastleEvaluation` struct. `KingSafetyEvaluator` consumes this API and exposes telemetry through `stats()` and `reset_stats()` methods. Debug logging can be enabled via `set_debug_logging(true)` for tuning sessions without flooding production logs.


