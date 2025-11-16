# Engine Improvement Roadmap

Date: December 2024  
Source: Task 30.0 Prioritization and Roadmapping

---

## Overview

Phased roadmap translating prioritized recommendations into scheduled workstreams and milestones. Incorporates dependencies, critical path, and deferred items from Task 27.0. Dates are indicative and use effort bands from the recommendations document.

---

## Roadmap Structure

- Phases grouped by milestones/quarters
- Streams: Search, Evaluation, Pattern Recognition, Infrastructure (TT, Parallel), Tooling & Tests
- Each item: Priority, Effort, Owner (TBD), Dependencies, Target Window

---

## Phase 1: Immediate Wins (Quick Wins)

- Window: Month 1
- Goals: Improve search efficiency and stability without risky changes; seed telemetry for future work.

1) Re-tune LMR aggressiveness and aspiration windows  
   - Priority: High | Effort: S-M | Owner: TBD  
   - Deps: None  
   - Target: Weeks 1–2  
   - Exit: -5–10% nodes to depth; ≤2% re-search frequency

2) Killer/history normalization and aging  
   - Priority: Medium | Effort: S | Owner: TBD  
   - Deps: None  
   - Target: Week 2  
   - Exit: Improved ordering accuracy in phase-shift suites

3) Time management buffers → confidence-based allocation  
   - Priority: High | Effort: M | Owner: TBD  
   - Deps: Telemetry counter for fail-high/volatility  
   - Target: Weeks 2–4  
   - Exit: +0.3–0.7 ply at same time; ≤1.0% time losses

4) Telemetry hardening for search/eval stats (enables later work)  
   - Priority: High | Effort: S | Owner: TBD  
   - Deps: Existing debug hooks  
   - Target: Week 1  
   - Exit: Metrics available to CI dashboards

---

## Phase 2: Core Engine Enhancements

- Window: Months 2–3
- Goals: Reduce node count, improve reuse via TT, stabilize quiescence behavior.

1) TT: Age-aware, depth-preferred replacement with cluster probing  
   - Priority: High | Effort: L | Owner: TBD  
   - Deps: Thread-safe table integration  
   - Target: Month 2  
   - Exit: +3–7% hit rate; -4–8% nodes

2) Quiescence: SEE-based noisy move gating; delta/futility threshold recalibration  
   - Priority: High | Effort: M | Owner: TBD  
   - Deps: SEE utility; eval alignment  
   - Target: Month 2  
   - Exit: -8–15% qnodes with equal/better tactical accuracy

3) PST/tapered retune for promoted pieces and hand material  
   - Priority: High | Effort: L | Owner: TBD  
   - Deps: Phase/telemetry fixes from Phase 1  
   - Target: Month 3  
   - Exit: +10–20 Elo in internal gauntlet; smoother phase curves

---

## Phase 3: Parallel Scaling and Stability

- Window: Months 3–4
- Goals: Recover scaling at 8–16 threads; reduce contention.

1) YBWC split heuristics refinement  
   - Priority: High | Effort: M | Owner: TBD  
   - Deps: TT updates complete  
   - Target: Early Month 3  
   - Exit: +10–20% speedup at 8–16 threads

2) TT partitioning/sharding for reduced contention  
   - Priority: High | Effort: M-L | Owner: TBD  
   - Deps: Replacement policy finalized  
   - Target: Month 4  
   - Exit: Lower lock contention; stable nps across threads

---

## Phase 4: Advanced Features and Research

- Items from the Research Backlog and postponed Task 27.0 implementations, gated behind prerequisites and data-driven validation.

- Window: Months 5–8 (gated)
- Goals: Explore high-upside changes behind flags; productionize only with sustained gains.

1) Adaptive pruning via volatility estimators (flagged)  
   - Priority: Research | Effort: L | Owner: TBD  
   - Deps: Robust telemetry; safeguards  
   - Exit: ≥5% nodes reduction without tactical regressions

2) ProbCut/Multi-Cut trials (flagged)  
   - Priority: Research | Effort: L | Owner: TBD  
   - Deps: Extensive test harnesses  
   - Exit: Demonstrated speedups on curated suites

3) NNUE/hybrid evaluation exploration (flagged)  
   - Priority: Research | Effort: XL | Owner: TBD  
   - Deps: Data pipeline; inference integration  
   - Exit: ≥30–50 Elo on internal gauntlet at comparable time controls

---

## Dependencies and Critical Path

- Phase 1 telemetry enables safe tuning in later phases.  
- TT replacement and probing land before parallel scaling changes.  
- Quiescence SEE gating lands prior to aggressive aspiration tightening.  
- PST/tapered retune follows phase/telemetry fixes to avoid rework.

---

## Tracking and Updates

- This roadmap will be updated after Task 31.0 documentation consolidation and as metrics/benchmarks evolve.


