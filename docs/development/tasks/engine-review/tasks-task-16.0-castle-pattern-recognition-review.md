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

- [ ] 2.0 Implement Zone-Based Castle Scoring and Exposed-King Penalties
  - [ ] 2.1 Replace binary match-quality gate with graded scoring that accounts for defender coverage zones, pawn chains, hand reinforcements, and shell completeness.
  - [ ] 2.2 Introduce explicit penalties for exposed kings (e.g., missing primary shell, breached pawn wall) and integrate them into tapered evaluation.
  - [ ] 2.3 Extend `KingSafetyConfig` with tunable weights/thresholds for castle bonuses and exposure penalties; add serde + docs.
  - [ ] 2.4 Update `king_safety.rs` to consume new scoring outputs, ensuring legacy placeholder APIs are removed or redirected.
  - [ ] 2.5 Validate scoring gradients with targeted unit tests (partial castles vs. bare kings) and golden-metric snapshots.

- [ ] 3.0 Integrate Castle Telemetry and Remove Duplicate King-Safety APIs
  - [ ] 3.1 Audit `KingSafetyEvaluator` and related consumers to eliminate duplicate placeholder methods returning zero.
  - [ ] 3.2 Extend telemetry structs and logging (`KingSafetyStats`, trace logs) to capture pattern matched, quality, missing defenders, cache hits/misses, and applied penalties.
  - [ ] 3.3 Add debug/trace hooks that can be toggled via config or feature flag for tuning sessions without flooding production logs.
  - [ ] 3.4 Update documentation and developer tooling to reference the single, authoritative castle evaluation API and telemetry contract.
  - [ ] 3.5 Write integration tests asserting telemetry fields populate correctly for intact, partial, and missing castles.

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


