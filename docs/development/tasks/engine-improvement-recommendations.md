# Engine Improvement Recommendations

Date: December 2024  
Source: PRD Tasks 1.0–29.0 synthesis, Task 30.0 Prioritization

---

## Overview

This document aggregates the actionable improvements identified during the engine feature reviews (Tasks 1.0–25.0), meta-analysis (Tasks 26.0–29.0), and prioritization (Task 30.0). Each recommendation includes priority, effort, rationale, dependencies, and the key metric(s) to validate impact.

---

## Prioritization Legend

- Priority: High | Medium | Low  
- Effort: S (≤2 days), M (3–7 days), L (1–3 weeks), XL (1–3 months)

---

## Top Priorities (High)

- Search: Re-tune late move reductions and move ordering weights  
  - Priority: High | Effort: M  
  - Rationale: Performance analysis (Task 26.0) shows suboptimal cutoff rates at mid-depths; LMR aggressiveness and history/killer heuristics are the highest-leverage knobs.  
  - Dependencies: None (uses existing infrastructure)  
  - Metrics: +5–12% cutoffs on non-PV nodes; -5–10% nodes to depth.

- Transposition Table: Introduce age-aware, depth-preferred replacement with cluster probing  
  - Priority: High | Effort: L  
  - Rationale: Hit rate plateaus under higher thread counts; age bias reduces pollution, probing improves reuse.  
  - Dependencies: Thread-safe table review (Task 8.0, 9.0)  
  - Metrics: +3–7% TT hit rate; -4–8% nodes; reduced re-searches.

- Quiescence: Refine delta and futility thresholds; add SEE for noisy move gating  
  - Priority: High | Effort: M  
  - Rationale: Tactical accuracy variance suggests over-expansion in volatile positions; SEE-based gating tightens horizon handling.  
  - Dependencies: SEE utility; evaluation SEE alignment  
  - Metrics: -8–15% qnodes; unchanged/blunted tactical miss rate in regression suite.

- Time Management: Replace fixed buffers with confidence-based allocation  
  - Priority: High | Effort: M  
  - Rationale: Over-conservative buffers reduce achieved depths on tight budgets; adopt volatility- and fail-high-aware time allocation.  
  - Dependencies: Telemetry hooks; search volatility estimator  
  - Metrics: +0.3–0.7 average ply at same time; ≤1.0% time losses.

- Evaluation: Retune PSTs and tapered weights for promoted pieces and hand material  
  - Priority: High | Effort: L  
  - Rationale: Task 10.0/12.0 revealed phase misweighting when hands are large; retuning improves middle/late clarity.  
  - Dependencies: Updated phase computation and stats (Task 10.0)  
  - Metrics: +10–20 Elo in internal gauntlet; improved phase smoothness metrics.

- Parallel Search: Reduce TT contention and improve split heuristics (YBWC)  
  - Priority: High | Effort: L  
  - Rationale: Scaling stalls after N≥8 threads; split point heuristics and TT partitioning can recover speedup.  
  - Dependencies: TT updates; parallel scheduler knobs  
  - Metrics: +10–25% speedup at 8–16 threads; stable nps.

---

## Medium Priorities

- IID trigger recalibration and aspiration window tightening  
  - Priority: Medium | Effort: S  
  - Rationale: Excess IID invocations at shallow depths inflate overhead; narrower aspirations reduce re-searches.  
  - Metrics: -2–4% total nodes; -1–2% re-search frequency.

- Killer/history table aging and normalization  
  - Priority: Medium | Effort: S  
  - Rationale: Sticky history inflates bias; modest decay improves adaptability.  
  - Metrics: Improved ordering accuracy on phase-shift positions; small nodes reduction.

- Endgame patterns: Activity and opposition refinements  
  - Priority: Medium | Effort: M  
  - Rationale: Edge-case evaluation instability in tablebase-adjacent positions.  
  - Metrics: Fewer evaluation oscillations in endgame suite; reduced OTb inconsistencies.

- Opening book selection heuristics  
  - Priority: Medium | Effort: S  
  - Rationale: Prefer diversity and opponent-model weighting; reduce deterministic lines.  
  - Metrics: Increased variety; stable performance in first 12 plies.

---

## Low Priorities

- Smoothstep/cubic interpolation kernel cleanup and doc alignment  
  - Priority: Low | Effort: S  
  - Rationale: Maintain code clarity; ensure docs match kernel behavior fully.  
  - Metrics: Documentation completeness; zero behavior drift.

- Bitboard micro-optimizations on rare paths  
  - Priority: Low | Effort: S  
  - Rationale: Minor hot path misses; only after higher ROI items.  
  - Metrics: ≤1–2% nps gain on targeted positions.

---

## Research Backlog (from Task 30.4)

- NNUE or hybrid evaluation for mid/late game  
  - Scope: Feature extraction, training pipeline, inference integration  
  - Risks: Complexity, training data quality, runtime overhead

- Advanced search techniques (ProbCut, Multi-Cut, PVS variants)  
  - Scope: Controlled experiments behind feature flags  
  - Risks: Tactical fragility, verification complexity

- Adaptive pruning using position volatility estimators  
  - Scope: Online adjustment of LMR/null-move thresholds  
  - Risks: Non-stationary behavior; needs solid guardrails

---

## Dependencies and Critical Path (from Task 30.5)

- Prioritize: Move ordering/LMR and TT improvements before parallel scaling work.  
- Complete: Quiescence SEE gating before tightening aspiration windows to avoid tactical regressions.  
- Sequence: PST/tapered retuning after phase and telemetry fixes to avoid rework.  
- Gate: Research items behind metrics dashboards and A/B harnesses; only promote after sustained gains.

---

## Appendix

- Cross-links:
  - Performance context: `docs/development/tasks/engine-performance-analysis.md`
  - Technical debt registry: `docs/development/tasks/engine-technical-debt-registry.md`
  - Roadmap: `docs/development/tasks/engine-improvement-roadmap.md`


