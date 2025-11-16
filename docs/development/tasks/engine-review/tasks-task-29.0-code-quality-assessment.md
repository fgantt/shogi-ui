# Tasks: Code Quality Assessment

**Parent PRD:** `task-29.0-code-quality-assessment.md`  
**Date:** November 2025  
**Status:** In Progress

---

## Overview

This task list captures the implementation work derived from the Code Quality Assessment. The scope focuses on maintainability, documentation, API clarity across Rust/TypeScript boundaries, and CI visibility for tests/coverage. This document currently lists only the high-level parent tasks. Sub-tasks and relevant files will be generated after confirmation.

---

## Relevant Files

- `src/` - Core engine Rust code; primary targets for maintainability and documentation improvements.
- `src/evaluation/` - Evaluators likely to be split into `extractors/` and `aggregators/`.
- `src/search/` - Public-facing structures and stats separation; rustdoc targets.
- `src/bitboards/`, `src/tablebase/`, `src/opening_book/` - Public API docs and module index references.
- `src/types.rs` - Public types, configuration structs, statistics; `Debug` derives, docs, decoupling config/stats.
- `src/utils/` (Rust) - Target location to consolidate shared helpers/traits.
- `src/components/`, `src/utils/` (TS) - Add JSDoc and cross-language API mapping notes.
- `src/types.ts` - Cross-language type mapping surface; add JSDoc and mapping references.
- `docs/architecture/` - Add "Engine Module Index" page; add cross-language API mapping doc.
- `docs/development/` - Document coverage workflow and CI details.
- `Cargo.toml`, `Cargo.lock` - Coverage tooling integration notes (e.g., grcov/tarpaulin).
- `package.json` - TS coverage/JSDoc scripts if applicable.
- `scripts/` - Add or update scripts for coverage runs and scheduled CI jobs.
- `tests/`, `benches/` - Ensure visibility in CI and document which suites are default vs. scheduled.

### Notes

- Unit tests should be located near their modules in Rust where feasible, with integration tests under `tests/`.
- Keep configuration and statistics as distinct types to reduce coupling; prefer explicit wiring at integration boundaries.
- JSDoc for TS and rustdoc for Rust should be aligned; link cross-language mappings from both sides.

---

## Tasks

- [x] 1.0 Rust utilities consolidation
  - [x] 1.1 Inventory duplicated helper traits/types (timing, telemetry, small utils) across `src/**`
  - [x] 1.2 Propose a `src/utils/` Rust module structure (modules, naming, ownership)
  - [x] 1.3 Create `src/utils/` and move/consolidate helpers with stable public surfaces
  - [x] 1.4 Replace call sites to use consolidated helpers; remove dead duplicates
  - [x] 1.5 Add rustdoc for consolidated helpers (purpose, invariants, usage examples)
  - [x] 1.6 Add unit tests for helpers where missing; ensure no behavior regressions
  - [x] 1.7 Update docs in `docs/development/` to reference consolidated utilities

- [ ] 2.0 Debug-ability improvements for public types
  - [ ] 2.1 Identify externally-consumed structs/enums lacking `#[derive(Debug)]` in `src/**`
  - [ ] 2.2 Add `Debug` derives where appropriate; avoid leaking sensitive data
  - [ ] 2.3 Ensure display/log formatting is coherent; add `Display` impls if useful
  - [ ] 2.4 Add/Update tests that rely on debug printing in integration/telemetry
  - [ ] 2.5 Document debug expectations in rustdoc for key public types

- [ ] 3.0 Evaluators modularization (extractors vs. aggregators)
  - [ ] 3.1 Identify evaluators that combine feature extraction and scoring in one module
  - [ ] 3.2 Design split into `extractors/` (feature extraction) and `aggregators/` (scoring/weights)
  - [ ] 3.3 Create submodule directories and move code with minimal public API disruption
  - [ ] 3.4 Introduce thin integration layer re-exporting stable surfaces
  - [ ] 3.5 Add rustdoc to each submodule explaining responsibilities and invariants
  - [ ] 3.6 Update tests to reflect new module paths; keep test names stable for CI reporting
  - [ ] 3.7 Document the new structure in `docs/architecture/` and link from module index

- [ ] 4.0 Configuration vs. statistics separation
  - [ ] 4.1 Audit configuration structs that also carry runtime statistics
  - [ ] 4.2 Create parallel `...Config` and `...Stats` types where coupled
  - [ ] 4.3 Refactor call sites to accept config separately and return/update stats explicitly
  - [ ] 4.4 Ensure serialization/deserialization boundaries are clear (config only)
  - [ ] 4.5 Add rustdoc clarifying ownership, lifecycle, and threading considerations
  - [ ] 4.6 Add tests to validate config immutability and stats updates

- [ ] 5.0 Documentation improvements (rustdoc, module index, TS JSDoc, cross-language mapping)
  - [ ] 5.1 Target 100% rustdoc coverage for public items in `src/` (prioritize integration surfaces)
  - [ ] 5.2 Create `docs/architecture/ENGINE_MODULE_INDEX.md` with one-paragraph per major module
  - [ ] 5.3 Add cross-language API mapping doc linking `src/types.rs` ↔ `src/types.ts`
  - [ ] 5.4 Annotate TS utilities and types with JSDoc; reference Rust equivalents
  - [ ] 5.5 Ensure `cargo doc` builds clean; fix warnings and broken intra-doc links
  - [ ] 5.6 Add a doc section in `docs/development/` describing documentation conventions

- [ ] 6.0 Test coverage and CI visibility enhancements
  - [ ] 6.1 Identify critical tests currently behind non-default features
  - [ ] 6.2 Move feasible critical tests to default CI; document trade-offs for heavy suites
  - [ ] 6.3 Create scheduled CI job (nightly/weekly) for extended test/bench suites
  - [ ] 6.4 Integrate Rust coverage (e.g., grcov/tarpaulin) and publish summary artifacts/badges
  - [ ] 6.5 Integrate TS/Jest/Vitest coverage (if applicable) with thresholds and artifacts
  - [ ] 6.6 Add scripts in `scripts/` to run coverage locally; document usage in `docs/development/`
  - [ ] 6.7 Ensure CI surfaces line/branch coverage deltas and highlights regressions

- [ ] 7.0 Overgrown integration modules → submodules and re-exports
  - [ ] 7.1 Identify “god” modules accumulating helpers/re-exports (integration-heavy files across `src/**`)
  - [ ] 7.2 Propose submodule layout per target (e.g., `integration/`, `helpers/`, `interfaces/`) with ownership
  - [ ] 7.3 Extract internal helpers into dedicated submodules; keep stable public surfaces
  - [ ] 7.4 Add re-exports at the original module root to avoid breaking external imports
  - [ ] 7.5 Add module-level rustdoc overviews describing responsibilities and boundaries
  - [ ] 7.6 Update internal imports; remove dead code and redundant re-exports
  - [ ] 7.7 Optional: Add a lightweight CI check or script to flag modules exceeding size/complexity thresholds
  - [ ] 7.8 Document the restructuring in `docs/architecture/ENGINE_MODULE_INDEX.md`

---

Ready to generate detailed sub-tasks and the Relevant Files section. Reply with "Go" to proceed.



### Task 1.0 Completion Notes

- Implementation: Replaced all remaining usages of `crate::debug_utils::debug_log` with `crate::utils::telemetry::debug_log` across the Rust codebase, aligning call sites with the consolidated utilities surface. Updated direct `use crate::debug_utils::debug_log;` imports to `use crate::utils::telemetry::debug_log;` where applicable. Macro-based fast logging (e.g., `debug_log_fast!`) remains under `crate::debug_utils` for feature-gated, zero-overhead compilation as intended.
- Utilities Surface: Confirmed `src/utils/telemetry.rs` re-exports `debug_log`, `trace_log`, `is_debug_enabled`, and `set_debug_enabled` from `crate::debug_utils` and provides a lazy-format helper (`tracef`). This keeps a stable, centralized path for telemetry without forcing churn at call sites in the future.
- Documentation: Rustdoc already present on `src/utils/telemetry.rs`; callers should favor `crate::utils::telemetry` for debug/trace logging and toggling. No external behavior changes; only import paths updated for consistency with the utilities consolidation goals.
- Testing/Build: The change is path-only; no functional logic altered. Macro locations unchanged to preserve feature gating (`verbose-debug`). No additional configuration is required.
