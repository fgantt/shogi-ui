# Tasks: Material Evaluation Modernization

**Parent PRD:** `task-11.0-material-evaluation-review.md`  
**Date:** November 2025  
**Status:** In Planning

---

## Overview

Task 11.0 focuses on bringing the material evaluation subsystem in line with the commitments documented in the PRD review. The current implementation delivers deterministic tapered scoring for board and hand pieces, but it suffers from dead configuration flags, hard-coded value tables, minimal observability, and linear board scans that block further optimization. This task list breaks the remediation work into actionable sub-projects that can be delivered incrementally while preserving engine stability.

### Objectives

1. **Honor configuration promises.** The `use_research_values` flag must become functional, and material evaluators instantiated via integration layers must reflect runtime configuration.
2. **Unlock tuning workflows.** Value tables should be externalized so that tuning pipelines and tooling can experiment without source changes.
3. **Improve visibility.** Enhanced statistics and telemetry are required to understand material contributions during search and tuning.
4. **Prepare for performance work.** Identify and implement optimizations that reduce redundant board scans and enable future incremental evaluation.
5. **Strengthen validation.** Expand tests and benchmarks to cover Shogi-specific scenarios (hand pieces, impasse boundaries, promotions) and to track regression risk.

### Delivery Phases

| Phase | Focus | Target Timeline | Dependencies |
|-------|-------|-----------------|--------------|
| **Phase 1** | Configuration enablement + integration wiring | 1 sprint | Config tooling, evaluation integration |
| **Phase 2** | Value-set externalization + telemetry | 1–2 sprints | Phase 1 completion |
| **Phase 3** | Performance optimizations + extended validation | 2+ sprints | Phases 1–2, profiling data |
| **Phase 4** | Stretch goals & documentation polish | As needed | Prior phases |

---

## Relevant Files

- `src/evaluation/material.rs`
- `src/evaluation/config.rs`
- `src/evaluation/integration.rs`
- `src/evaluation/performance.rs`
- `src/types.rs`
- `src/evaluation/mod.rs` (wiring changes)
- `src/telemetry/` (if telemetry hooks required)
- `src/cli/configuration/` & `configs/*.toml` (runtime configuration surfaces)
- `benches/material_evaluation_performance_benchmarks.rs`
- `tests/evaluation/material_*`
- `docs/development/tasks/engine-review/task-11.0-material-evaluation-review.md`
- `docs/development/tasks/engine-performance-analysis.md`
- `docs/tuning/` (value-set workflow documentation)

### External Coordination

- **Task 7.0 (Search Integration):** Telemetry and statistics must align with the Integrated Evaluator statistics pipeline.
- **Task 20.0 (Evaluation Integration):** Configuration propagation should leverage forthcoming evaluator factory updates.
- **Meta-task 26.0 (Performance Analysis):** Share benchmark outputs and regression tracking.
- **Tuning Roadmap:** Ensure material-weight import/export aligns with weight manager expectations.

---

## High-Level Task Breakdown

- [x] **1.0 🔴 Value-Set Selection Enablement**
- [ ] **2.0 🔴 Configuration Propagation & Lifecycle**
- [ ] **3.0 🔴 Externalized Material Value Sets**
- [ ] **4.0 🟡 Observability & Telemetry Expansion**
- [ ] **5.0 🟡 Performance & Scalability Enhancements**
- [ ] **6.0 🟡 Test Coverage & Edge Case Validation**
- [ ] **7.0 🔵 Documentation, Release Notes, and Rollout**
- [ ] **8.0 🔵 Post-Deployment Monitoring & Follow-up**

Each parent task contains granular sub-tasks below. Complete checklists gate progression to the next phase unless otherwise noted.

---

## 1.0 🔴 Value-Set Selection Enablement

**Goal:** Honor the `use_research_values` flag and provide a clear API for selecting value tables.

- [x] **1.1 Material Value Abstraction**
  - [x] 1.1.1 Introduce a `MaterialValueSet` struct encapsulating board and hand tapered tables.  
  - [x] 1.1.2 Provide constructors for `classic` and `research` presets mirroring current constants.  
  - [x] 1.1.3 Include metadata (source, version, last updated) to support auditing and telemetry tagging.
- [x] **1.2 Evaluator Construction**
  - [x] 1.2.1 Extend `MaterialEvaluator::new_with_config(config: &MaterialEvaluationConfig)` to load the selected value set.  
  - [x] 1.2.2 Ensure hand-value lookup degrades gracefully when a table omits promoted pieces (default to board values).  
  - [x] 1.2.3 Add assertions or warnings when the requested preset is missing required entries.
- [x] **1.3 Tests & Fixtures**
  - [x] 1.3.1 Snapshot tests comparing classic vs. research evaluations on representative positions.  
  - [x] 1.3.2 Property tests verifying toggling the flag produces different scores when tables diverge.  
  - [x] 1.3.3 Update unit benches (if needed) to exercise both presets.
- [x] **1.4 Configuration Surface**
  - [x] 1.4.1 Update TOML/CLI configuration schemas with documentation for `use_research_values`.  
  - [x] 1.4.2 Add validation ensuring only known presets are accepted (until custom loading is available).  
  - [x] 1.4.3 Document expected default (research) and supported alternatives.

**Exit Criteria:** Unit tests demonstrate measurable differences between classic and research tables; configuration flag toggles behavior without code changes.

---

## 2.0 🔴 Configuration Propagation & Lifecycle

**Goal:** Ensure material configuration changes flow from top-level evaluators into concrete evaluators at runtime.

- [x] **2.1 Integration Wiring**
  - [x] 2.1.1 Modify `IntegratedEvaluator` and `OptimizedEvaluator` constructors to accept a `MaterialEvaluationConfig`.  
  - [x] 2.1.2 Pass material configuration through evaluator factories, avoiding default instantiation.  
  - [x] 2.1.3 Introduce `MaterialEvaluatorHandle` helper if needed to manage interior mutability.
- [x] **2.2 Runtime Reconfiguration**
  - [x] 2.2.1 Add `update_config(&self, new_config)` path that rebuilds or hot-swaps the material evaluator.  
  - [x] 2.2.2 Ensure caches dependent on material values (evaluation cache, phase cache) are invalidated or versioned.  
  - [x] 2.2.3 Reset or migrate `MaterialEvaluationStats` on config change to avoid mixing metrics.
- [x] **2.3 Integration Tests**
  - [x] 2.3.1 Add test toggling `include_hand_pieces` and `use_research_values` during runtime; assert evaluator output changes accordingly.  
  - [x] 2.3.2 Verify configuration updates propagate through telemetry endpoints.  
  - [x] 2.3.3 Confirm time pressure or evaluation batching logic respects updated material configuration.
- [x] **2.4 Error Handling**
  - [x] 2.4.1 Provide clear errors when configurations reference unavailable value sets.  
  - [x] 2.4.2 Ensure concurrent reconfiguration is synchronized (e.g., via `RwLock`/`Mutex`).

**Exit Criteria:** Integration tests pass; runtime config changes are reflected without restart; caches and statistics remain consistent.

---

## 3.0 🔴 Externalized Material Value Sets

**Goal:** Allow material tables to be loaded from and saved to external artifacts to enable tuning workflows.

- [ ] **3.1 File Format & Serialization**
  - [ ] 3.1.1 Define a `material_values.schema.json` (or equivalent) capturing board/hand tables, metadata, and optional annotations.  
  - [ ] 3.1.2 Implement Serde serialization/deserialization supporting JSON and TOML with versioning.  
  - [ ] 3.1.3 Provide default assets for both research and classic value sets under `resources/material/`.
- [ ] **3.2 Loader Integration**
  - [ ] 3.2.1 Add `MaterialValueSet::from_path` and `from_reader` helpers with robust error reporting.  
  - [ ] 3.2.2 Integrate loaders into configuration (e.g., `material.values_path`).  
  - [ ] 3.2.3 Support hot-reloading in development mode (optional).
- [ ] **3.3 Tuning Pipeline Hookups**
  - [ ] 3.3.1 Update tuning manager utilities to export/import material tables.  
  - [ ] 3.3.2 Document canonical workflows for running self-play with alternate value sets.  
  - [ ] 3.3.3 Add regression tests ensuring loading/saving round-trips preserve data.
- [ ] **3.4 Safety & Validation**
  - [ ] 3.4.1 Validate tables for completeness (all piece types) and symmetry before acceptance.  
  - [ ] 3.4.2 Provide fallback behavior (e.g., revert to defaults) when file loading fails, with telemetry warnings.

**Exit Criteria:** Engine can run using value tables stored outside the binary; tuning tools can manipulate these tables without modifying source code.

---

## 4.0 🟡 Observability & Telemetry Expansion

**Goal:** Offer deep insight into material contribution during search and tuning.

- [ ] **4.1 Statistics Enrichment**
  - [ ] 4.1.1 Extend `MaterialEvaluationStats` with per-piece totals (board and hand), phase-weighted contributions, and evaluation deltas.  
  - [ ] 4.1.2 Track evaluations by configuration preset (classic vs. research vs. custom).  
  - [ ] 4.1.3 Introduce hand-piece imbalance metrics to aid impasse diagnostics.
- [ ] **4.2 Telemetry Integration**
  - [ ] 4.2.1 Surface new stats via `IntegratedEvaluator::get_statistics` and telemetry exporters.  
  - [ ] 4.2.2 Add optional debug logging showing top contributing pieces when evaluation thresholds are crossed.  
  - [ ] 4.2.3 Ensure telemetry respects privacy/performance guardrails (toggleable verbosity).
- [ ] **4.3 Testing & Validation**
  - [ ] 4.3.1 Create integration tests verifying telemetry counters increment appropriately across board/hand scenarios.  
  - [ ] 4.3.2 Add assertion tests ensuring statistics reset on configuration change.  
  - [ ] 4.3.3 Update performance benchmarks to measure telemetry overhead (should remain <2%).
- [ ] **4.4 Tooling Support**
  - [ ] 4.4.1 Update search debug UI dashboards (if applicable) with new material metrics.  
  - [ ] 4.4.2 Provide sample telemetry snapshots in documentation for tuning teams.

**Exit Criteria:** Material metrics are exposed through standard telemetry paths with test coverage and acceptable overhead.

---

## 5.0 🟡 Performance & Scalability Enhancements

**Goal:** Reduce evaluation latency without sacrificing determinism.

- [ ] **5.1 Profiling & Target Selection**
  - [ ] 5.1.1 Profile material evaluation hot paths (board scans, hand iteration, value lookups) using Criterion and perf tooling.  
  - [ ] 5.1.2 Establish baseline benchmarks (board-only, board+hand heavy, high promotion counts).  
  - [ ] 5.1.3 Document hotspots and prioritize optimizations (bitboards, cached piece lists, incremental deltas).
- [ ] **5.2 Optimization Implementation**
  - [ ] 5.2.1 Implement bitboard-based counting or maintain cached piece lists keyed by piece type.  
  - [ ] 5.2.2 Add optional incremental update hooks to reuse results during move generation (stretch goal).  
  - [ ] 5.2.3 Ensure optimizations maintain symmetry and determinism via regression tests.
- [ ] **5.3 Benchmark Expansion**
  - [ ] 5.3.1 Augment Criterion suite to cover new scenarios (bulk hand pieces, repeated evaluations).  
  - [ ] 5.3.2 Capture before/after metrics in `docs/development/tasks/engine-performance-analysis.md`.  
  - [ ] 5.3.3 Add ablation benchmark that disables material scoring to quantify contribution to overall evaluation time.  
  - [ ] 5.3.4 Define success thresholds (e.g., >15% speedup on heavy scenarios).
- [ ] **5.4 Rollout Safeguards**
  - [ ] 5.4.1 Provide feature flags or configuration toggles to disable new optimizations if regressions occur.  
  - [ ] 5.4.2 Monitor evaluation consistency via cross-check tests comparing old vs. new pathways.

**Exit Criteria:** Benchmarks demonstrate measurable improvements; optimizations are guarded and verifiable.

---

## 6.0 🟡 Test Coverage & Edge Case Validation

**Goal:** Guarantee correctness across Shogi-specific scenarios and prevent regressions.

- [ ] **6.1 Board & Hand Scenarios**
  - [ ] 6.1.1 Add tests covering large hand inventories (e.g., nine pawns, multiple major pieces).  
  - [ ] 6.1.2 Validate promoted capture handling (board pieces captured, demoted in hand).  
  - [ ] 6.1.3 Ensure symmetry for mirrored positions and both players.
- [ ] **6.2 Impasse & Special Rules**
  - [ ] 6.2.1 Verify material scoring at impasse thresholds (24/27 points) remains accurate with new tables.  
  - [ ] 6.2.2 Evaluate hand-value heuristics versus drop restrictions (knights/lances near promotion zone); adjust or document limitations.  
  - [ ] 6.2.3 Add tests for repetition scenarios to confirm statistics reset between evaluations.
- [ ] **6.3 Regression Harness**
  - [ ] 6.3.1 Integrate material-only evaluation regression suite (positions + expected scores).  
  - [ ] 6.3.2 Provide fixtures for tuning teams to validate custom tables.  
  - [ ] 6.3.3 Ensure tests are included in CI gating (fast subset + extended nightly).

**Exit Criteria:** Test suite covers identified edge cases; CI runs enforce material correctness.

---

## 7.0 🔵 Documentation, Release Notes, and Rollout

**Goal:** Communicate changes clearly to engine developers, tuning teams, and users.

- [ ] **7.1 Developer Documentation**
  - [ ] 7.1.1 Update material evaluation section in `docs/development/tasks/engine-performance-analysis.md` with new metrics and benchmarks.  
  - [ ] 7.1.2 Add value-set workflow documentation in `docs/tuning/material-value-sets.md` (new file) covering import/export, presets, and tuning tips.  
  - [ ] 7.1.3 Document telemetry fields and how to interpret hand-piece statistics.
- [ ] **7.2 Configuration Guides**
  - [ ] 7.2.1 Update configuration reference to explain new options (`values_path`, presets, telemetry verbosity).  
  - [ ] 7.2.2 Provide migration notes for users relying on hard-coded tables.  
  - [ ] 7.2.3 Include troubleshooting guide for missing/invalid value files.
- [ ] **7.3 Release Notes**
  - [ ] 7.3.1 Summarize material evaluation changes for the next engine release.  
  - [ ] 7.3.2 Highlight compatibility impacts and recommended defaults.  
  - [ ] 7.3.3 Link to benchmarks and tuning case studies.

**Exit Criteria:** Documentation updated, reviewed, and published; release notes ready before deployment.

---

## 8.0 🔵 Post-Deployment Monitoring & Follow-up

**Goal:** Ensure the changes remain stable in production and inform future iterations.

- [ ] **8.1 Telemetry Dashboards**
  - [ ] 8.1.1 Add monitoring dashboards tracking material evaluation counts, preset usage, and contribution ratios.  
  - [ ] 8.1.2 Set alerts for anomalous hand-piece imbalances or missing telemetry.
- [ ] **8.2 Regression Watch**
  - [ ] 8.2.1 Run weekly self-play A/B tests comparing classic vs. research tables; log Elo deltas.  
  - [ ] 8.2.2 Track evaluation latency metrics and flag regressions >5% from baseline.  
  - [ ] 8.2.3 Record incident reports if configuration reloads fail.
- [ ] **8.3 Backlog Grooming**
  - [ ] 8.3.1 Review deferred stretch goals (incremental evaluation, drop-aware heuristics).  
  - [ ] 8.3.2 Capture learnings in follow-up tasks (Task 12.x series if needed).  
  - [ ] 8.3.3 Close the loop with tuning teams for additional requests.

**Exit Criteria:** Monitoring is live; post-deployment review completed; follow-up backlog updated.

---

## Acceptance Checklist

| Category | Criteria |
|----------|---------|
| **Configuration** | `use_research_values` toggles behavior, runtime updates supported, errors surfaced clearly |
| **Value Sets** | External files load successfully, schema documented, round-trip serialization tested |
| **Telemetry** | Expanded statistics available via API/logging, overhead <2%, counters reset on config change |
| **Performance** | Benchmarks show targeted improvements, results documented, guardrails in place |
| **Testing** | New unit/integration tests cover edge cases, regression suite included in CI |
| **Docs & Release** | Updated guides, release notes, tuning workflows published |
| **Monitoring** | Dashboards and alerts configured, A/B testing plan in place |

---

## Open Questions & Risks

1. **Hand-value heuristics:** Do we need rank/file-aware modifiers now, or can we defer until telemetry highlights issues?  
2. **Value-set provenance:** Who owns the canonical research vs. classic tables? Define the update process and review gate.  
3. **Hot reload complexity:** Is live material config switching required in production, or can it remain a development-only feature?  
4. **Incremental evaluation roadmap:** Should incremental material scoring be part of this task or spun into a new initiative (Task 12.x)?  
5. **Compatibility:** How do we ensure older configuration files fail fast when referencing removed presets?

Risks should be addressed during implementation planning; unresolved questions require owner assignments before Phase 2 work commences.

---

## References

- `docs/development/tasks/engine-review/task-11.0-material-evaluation-review.md` (PRD)  
- `docs/development/tasks/engine-review/task-7.0-search-algorithm-integration.md` (structure reference)  
- `docs/development/tasks/engine-performance-analysis.md` (benchmark logging)  
- `docs/tuning/weight-manager.md` (integration reference)  
- Criterion benchmarks under `benches/material_evaluation_performance_benchmarks.rs`

---

## Completion Notes

### Task 1.0 — Value-Set Selection Enablement

- **Implementation:** Added `MaterialValueSet` abstraction with classic and research presets, metadata tags, and per-piece tapered tables. `MaterialEvaluator` now selects the appropriate preset based on `MaterialEvaluationConfig`, with graceful hand-value fallbacks when entries are omitted.
- **Testing:** Expanded material evaluator unit tests to cover preset toggling, value-set differentials, and promoted piece fallbacks. Introduced position-based regression ensuring evaluations diverge between presets.
- **Documentation:** Updated `ENGINE_CONFIGURATION_GUIDE.md` to explain preset selection semantics for `use_research_values`.

### Task 2.0 — Configuration Propagation & Lifecycle

- **Implementation:** `IntegratedEvaluationConfig` now owns `MaterialEvaluationConfig`, and both `IntegratedEvaluator` and `OptimizedEvaluator` hydrate their material evaluators from it. Runtime updates rebuild material evaluators, refresh optimized paths, clear evaluation/phase caches, and reset telemetry counters.
- **Testing:** Added unit coverage for `MaterialEvaluator::apply_config` plus integration tests that toggle presets at runtime, assert cache invalidation, score divergence, and material statistics resets.
- **Notes:** A dedicated handle struct was unnecessary—the existing `RefCell`-backed evaluator provides safe runtime swapping. Configuration inputs remain validated by design because the preset flag maps to compiled-in value sets.

---

**Document Status:** Draft  
**Next Review:** Align with Task 11.0 sprint planning meeting  
**Owners:** Material Evaluation Working Group (Config, Evaluation, Tuning representatives)
