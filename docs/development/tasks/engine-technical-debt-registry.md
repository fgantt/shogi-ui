# Engine Technical Debt Registry

Date: December 2024  
Source: Task 28.0 Technical Debt Documentation, Task 29.0 Code Quality Assessment, Task 30.0 Prioritization

---

## Overview

Catalog of identified technical debt items with priority, severity, refactoring approach, and cross-references to affected modules. Aligned with the prioritization outcomes from Task 30.0.

---

## Registry Fields

- Area/Module  
- Description  
- Impact (Maintainability/Performance/Correctness/Complexity)  
- Priority (High/Medium/Low)  
- Effort (S/M/L/XL)  
- Dependencies/Blocks  
- Notes/Refactoring Plan  

---

## Entries

- Bitboards: Scattered micro-optimizations across rare code paths  
  - Impact: Maintainability/Performance  
  - Priority: Low | Effort: S  
  - Deps/Blocks: None  
  - Notes/Plan: Consolidate helpers, remove duplicate bit-scan utilities, favor intrinsics only on hot paths validated by perf.

- Search: Overlapping responsibility between `search_engine.rs` and `quiescence_search.rs`  
  - Impact: Maintainability/Complexity  
  - Priority: Medium | Effort: M  
  - Deps/Blocks: Quiescence gating redesign  
  - Notes/Plan: Extract shared node accounting and bound logic into a small utility; clarify ownership of stand-pat and SEE calls.

- Move Ordering: History/killer tables lack decay/normalization  
  - Impact: Correctness/Performance  
  - Priority: Medium | Effort: S  
  - Deps/Blocks: Telemetry counters for ordering accuracy  
  - Notes/Plan: Introduce aging, clamp ranges, expose metrics to tuning tools.

- Transposition Table: Replacement policy monolith and unclear aging semantics  
  - Impact: Performance/Maintainability  
  - Priority: High | Effort: L  
  - Deps/Blocks: Parallel table integration  
  - Notes/Plan: Implement age-aware, depth-preferred replacement; add cluster probing; document invariants and entry lifecycle.

- Parallel Search (YBWC): Split heuristics hard-coded; limited instrumentation  
  - Impact: Performance/Complexity  
  - Priority: High | Effort: M  
  - Deps/Blocks: TT contention improvements  
  - Notes/Plan: Parameterize split thresholds; add counters for idle time, steal success rate; experiment with lightweight task hints.

- Evaluation: PST and tapered weights out-of-sync with promoted/hand material  
  - Impact: Correctness/Performance  
  - Priority: High | Effort: L  
  - Deps/Blocks: Phase telemetry stabilization  
  - Notes/Plan: Retune with updated phase model; expand tests for hand-heavy middlegames and promoted pieces.

- Time Management: Fixed safety buffers not tied to volatility/fail-highs  
  - Impact: Performance  
  - Priority: Medium | Effort: M  
  - Deps/Blocks: Telemetry on volatility/fail-high streaks  
  - Notes/Plan: Replace with confidence-based allocation; backtest to avoid time forfeits.

- Testing/CI: Legacy-gated tests hiding coverage gaps  
  - Impact: Correctness/Maintainability  
  - Priority: Medium | Effort: S  
  - Deps/Blocks: Test runtime budget on CI  
  - Notes/Plan: Ungate core suites; mark truly slow tests as nightly; add sampling-based property tests where feasible.

- Documentation: Interpolation kernels docs drift from implementation  
  - Impact: Maintainability  
  - Priority: Low | Effort: S  
  - Deps/Blocks: None  
  - Notes/Plan: Align docstrings and guides with production kernels; include visual references for tuning.

---

## Cross-References

- Recommendations: `docs/development/tasks/engine-improvement-recommendations.md`  
- Roadmap: `docs/development/tasks/engine-improvement-roadmap.md`  
- Performance Analysis: `docs/development/tasks/engine-performance-analysis.md`  


