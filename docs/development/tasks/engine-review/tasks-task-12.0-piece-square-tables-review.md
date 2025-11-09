# Tasks: Piece-Square Tables Modernization

**Parent PRD:** `task-12.0-piece-square-tables-review.md`  
**Date:** November 2025  
**Status:** Not Started

---

## Overview

This task list translates the recommendations from Task 12.0 into actionable engineering work. The goals are to eliminate configuration drift, harden test coverage, introduce configurable/tunable tables, and expand telemetry so PST impact is observable across evaluation and search pipelines.

## Relevant Files

- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/piece_square_tables.rs` - Primary PST implementation
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/integration.rs` - Integrated evaluator using PST lookups
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation.rs` - Legacy evaluator containing duplicate PST definitions
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/src/evaluation/performance.rs` - Performance profiler hooks
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/benches/evaluation_performance_optimization_benchmarks.rs` - Existing PST micro-benchmarks
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/docs/design/implementation/evaluation-optimizations/tapered-evaluation/` - Supporting documentation for tapered evaluation
- `/Users/fgantt/projects/vibe/shogi-game/shogi-ui/config/` (new) - Proposed location for tunable PST artifacts

### Notes

- Unit tests should live alongside the modules they cover (e.g., `src/evaluation/piece_square_tables.rs`)
- Integration tests belong under `tests/`
- Benchmarks go in `benches/`
- Loader configuration assets should be version-controlled under `config/` with documentation in `docs/`

---

## Tasks

- [ ] 1.0 Unify Piece-Square Table Implementations (High Priority — Est: 4-6 hours)
  - [ ] 1.1 Remove legacy `PieceSquareTables` struct from `src/evaluation.rs` and redirect all usages to `evaluation::piece_square_tables::PieceSquareTables`
  - [ ] 1.2 Update fallback evaluator paths (feature flags, tests, cache probes) to construct the shared PST module
  - [ ] 1.3 Add regression tests to ensure legacy evaluator paths produce identical scores to the integrated evaluator for a representative corpus (opening, middlegame, late-game positions)
  - [ ] 1.4 Verify bench harnesses and profiling hooks use the unified PST implementation
  - [ ] 1.5 Document migration notes in `docs/development/tasks/engine-review/task-12.0-piece-square-tables-review.md` (Section 12.1 reference)

- [ ] 2.0 Promote PST Tests to Default CI (High Priority — Est: 2-3 hours)
  - [ ] 2.1 Remove the `legacy-tests` feature gate around PST unit tests and ensure they run under `cargo test`
  - [ ] 2.2 Expand unit tests to cover promoted pieces, white/black symmetry, and taper consistency (mg vs. eg totals)
  - [ ] 2.3 Add integration tests verifying PST scoring across phase transitions within `tests/evaluation_pipeline_tests.rs`
  - [ ] 2.4 Update CI configuration/docs to highlight PST coverage and expected runtime deltas (Section 12.4 reference)

- [ ] 3.0 Introduce Configurable PST Loader (High Priority — Est: 8-12 hours)
  - [ ] 3.1 Design a serializable PST schema (JSON or TOML) supporting piece type, phase, and symmetry metadata
  - [ ] 3.2 Implement loader module (`src/evaluation/pst_loader.rs`) that deserializes external tables into `PieceSquareTables`
  - [ ] 3.3 Add CLI flag and config plumbing to select PST presets at runtime (`EngineOptions` / `MaterialValueSet` integration)
  - [ ] 3.4 Provide default schema files under `config/pst/default.json` (matching current baked-in values)
  - [ ] 3.5 Write unit tests for deserialization, validation (board dimensions, symmetry), and round-trip comparisons
  - [ ] 3.6 Update documentation with loader usage, override instructions, and safety guidelines (Section 12.4 reference)

- [ ] 4.0 Expand PST Telemetry & Observability (Medium Priority — Est: 5-7 hours)
  - [ ] 4.1 Extend evaluation telemetry structures (`EvaluationStatistics`, `EvaluationTelemetry`) to record PST midgame/endgame contributions and per-piece aggregates
  - [ ] 4.2 Surface PST contribution metrics in search debug logs and optional profiler output (`PerformanceProfiler::record_pst_score`)
  - [ ] 4.3 Update self-play/regression logging to capture PST deltas for before/after comparisons
  - [ ] 4.4 Add integration tests asserting telemetry values align with evaluation results for known positions
  - [ ] 4.5 Refresh documentation (`DEBUG_LOGGING_OPTIMIZATION.md`) with PST telemetry guidance (Section 12.6 reference)

- [ ] 5.0 Establish Tuning Methodology & Data Pipeline (Medium Priority — Est: 6-10 hours)
  - [ ] 5.1 Create `docs/evaluation/pst-tuning-methodology.md` outlining data sources, tuning workflows, and validation criteria
  - [ ] 5.2 Add scripted pipeline (e.g., `scripts/pst_tuning_runner.rs`) to ingest tuner outputs and emit loader-compatible files
  - [ ] 5.3 Define baseline experiments (self-play, expert positions) and success metrics for PST adjustments
  - [ ] 5.4 Capture sample tuning results and integrate them into the documentation as references
  - [ ] 5.5 Coordinate with material evaluation team to ensure PST changes align with material weights (Section 12.4 & Coordination Considerations)

- [ ] 6.0 Optimize PST Construction & Memory Usage (Low Priority — Est: 3-4 hours)
  - [ ] 6.1 Evaluate `OnceLock`/`lazy_static` adoption to share PST arrays across evaluator instances
  - [ ] 6.2 Benchmark current vs. shared PST initialization using Criterion (extend existing `pst_evaluation` group)
  - [ ] 6.3 If sharing is adopted, ensure thread safety and update tests/benchmarks to cover multi-instantiation scenarios
  - [ ] 6.4 Document memory/performance findings and rationale for chosen strategy (Section 12.5 reference)

- [ ] 7.0 Validation & Rollout Plan (Cross-Cutting — Est: 2-3 hours)
  - [ ] 7.1 Update measurement checklist (Section 12.7) to include unit, integration, benchmarking, and telemetry verification steps
  - [ ] 7.2 Add regression suite covering representative positions to guard against PST drift during future tuning
  - [ ] 7.3 Coordinate rollout with search/evaluation teams, ensuring feature flag strategies or staged deployment if loader introduces runtime variability

---

## Alignment & Coverage

| PRD Section | Addressed By | Notes |
|-------------|--------------|-------|
| 12.1 Table Architecture | Task 1.0 | Eliminates duplicate implementations, guarantees consistent consumption paths |
| 12.2 Piece Coverage & Consistency | Tasks 1.0, 2.0 | Adds regression tests covering promoted pieces and symmetry guarantees |
| 12.3 Phase-Specific Value Quality | Tasks 2.0, 5.0 | Ensures tests and tuning workflows validate taper integrity and heuristics |
| 12.4 Tuning & Maintainability | Tasks 2.0, 3.0, 5.0 | Loader, documentation, and CI coverage establish maintainable tuning pipeline |
| 12.5 Performance & Memory Traits | Task 6.0 | Benchmarks and memory-sharing evaluation keep PST footprint optimized |
| 12.6 Evaluation Contribution & Observability | Task 4.0 | Telemetry upgrades expose PST contribution in logs, metrics, and profiler |
| Measurement & Validation Plan | Task 7.0 | Consolidates test/benchmark requirements and rollout coordination |

---

## Success Criteria

- Unified PST implementation yields identical scores across all evaluation paths within ±1 cp tolerance
- PST tests execute in default CI with promoted-piece coverage and phase consistency assertions
- Loader supports drop-in replacement of PST values without code changes, with validated schema and documentation
- Telemetry surfaces PST contributions in evaluation logs, profiler snapshots, and regression dashboards
- Tuning methodology produces reproducible PST variants with documented validation results
- Performance benchmarks demonstrate <= 1% overhead increase after loader integration, with memory usage documented

Meeting these criteria confirms Task 12.0’s recommendations are fully implemented and production-ready.

---

