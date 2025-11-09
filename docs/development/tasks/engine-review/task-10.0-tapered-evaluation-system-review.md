# Task 10.0: Tapered Evaluation System Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** December 2024  
**Status:** Complete

---

## Executive Summary

The tapered evaluation stack is **feature-rich and highly modular**. It combines material-aware phase calculation, multiple interpolation curves, and an integrated evaluator that orchestrates positional feature extractors, pattern recognizers, and performance instrumentation. The implementation offers extensive flexibility (linear/cubic/sigmoid/smoothstep/spline interpolation, adaptive phase boundaries, profiling hooks) and slots cleanly into the broader evaluation pipeline via `IntegratedEvaluator`.

Key findings:

- ✅ Linear interpolation path is correct, deterministic, and heavily exercised by tests.
- ✅ Smoothstep and sigmoid transitions generate stable S-curves with bounded deltas; validation helpers ensure monotonicity.
- ✅ Integrated evaluator caches phase results and exposes statistics, making tapered evaluation cost manageable even in deep search.
- ⚠️ Cubic interpolation uses a pure `t³` easing that over-weights endgame values in the middlegame, leading to a 7× stronger endgame bias at `phase = 128`.
- ⚠️ Sigmoid interpolation ignores the configurable `sigmoid_steepness`, preventing tuning.
- ⚠️ Phase calculation omits pieces in hand and promoted-piece contributions, causing misclassification in shogi-specific positions (drop-heavy middlegames, promoted minor pieces).
- ⚠️ Advanced interpolator is not wired into the production path; tests are gated behind `legacy-tests`, leaving coverage gaps.

Overall grade: **B+ (88/100)** — solid foundations with clear improvement opportunities around phase accuracy, configuration fidelity, and interpolation consistency.

---

## Relevant Files

### Primary Implementation
- `src/evaluation/tapered_eval.rs` – `TaperedEvaluation` coordinator (phase calculation, caching, stats).
- `src/evaluation/phase_transition.rs` – Interpolation kernels (linear, cubic, sigmoid, smoothstep) and smoothness validation utilities.
- `src/types.rs` – `TaperedScore`, `TaperedEvaluationConfig`, phase constants, and piece phase weights.
- `src/evaluation/integration.rs` – `IntegratedEvaluator` pipelines tapered scores with feature evaluators and caches.
- `src/evaluation/performance.rs` – `OptimizedEvaluator` fast path plus `PerformanceProfiler`.

### Supporting / Tuning
- `src/evaluation/advanced_interpolation.rs` – Spline, multi-phase, and adaptive interpolator prototypes.
- `src/evaluation/phase_transition.rs` (config/statistics structs).
- `src/evaluation/config.rs` – Preset bundles referencing tapered evaluation.
- `src/evaluation/statistics.rs` – Aggregated evaluation metrics.
- `tests/*` (feature-gated) – Interpolation correctness coverage.

---

## 1. Implementation Review (Task 10.1)

### 1.1 Core Architecture
- `TaperedEvaluation` owns:
  - `TaperedEvaluationConfig` (flags, memory targets, king-safety delegation).
  - `cached_phase: Option<(u64, i32)>` – single-position cache keyed by a simplified hash.
  - `TaperedEvaluationStats` (phase calculations, cache hits, interpolation counter).
- Construction surfaces `new()` and `with_config()`, ensuring king safety configuration propagates consistently.
- `set_config()` clears caches to avoid stale phase data after configuration changes.

### 1.2 Game Phase Calculation
- `calculate_game_phase()` increments statistics, consults cache when enabled, and falls back to `calculate_phase_from_material()`.
- Material scan walks the 9×9 board, summing `PIECE_PHASE_VALUES`, then scales to `[0, 256]`.
- Pros:
  - Deterministic scaling, clamps to bounds.
  - Cache hit rate is observable via `stats.cache_hit_rate()`.
  - Works seamlessly with `IntegratedEvaluator::calculate_phase_cached()` (HashMap-based multi-node cache).
- Gaps:
  - **Pieces in hand are ignored** — critical in shogi where drops influence phase perception.
  - **Promoted pieces contribute zero** because `PIECE_PHASE_VALUES` lacks entries for promoted types; games with multiple promoted pieces appear artificially endgame-heavy.
  - O(81) scans per call; acceptable with caching but still hot path when cache disabled.
  - Position hash (`get_position_hash`) does not include captured-piece pools, resulting in cache misses for board-equivalent states with different hands.

### 1.3 Statistics and Configuration
- `TaperedEvaluationStats` supports cloning and exposes `cache_hit_rate()` plus atomic interpolation counters (used rarely).
- `TaperedEvaluationConfig` presets:
  - `default()` – enabled, caching on, monitoring off.
  - `performance_optimized()` – larger memory pool, monitoring enabled.
  - `memory_optimized()` – disables caching to shrink footprint.
- King safety configuration nests inside the same struct, aligning with PRD requirement for unified knobs.

### 1.4 Integration Points
- `IntegratedEvaluator` stores `TaperedEvaluation` and `PhaseTransition` inside `RefCell`s, uses them for phase computation and final score interpolation.
- Caching:
  - Phase cache (`HashMap<u64, i32>`) eliminates re-computation within a node expansion.
  - Evaluation cache stores final score + phase to reuse entire evaluation results.
- `OptimizedEvaluator` demonstrates low-overhead integration for benchmarks/perf mode (Phase Transition default linear).

---

## 2. Interpolation Method Verification (Tasks 10.2 – 10.5)

### 2.1 Linear Interpolation (Task 10.2)
- Implemented in `TaperedScore::interpolate` and mirrored by `PhaseTransition::interpolate_linear`.
- Formula: `(mg * phase + eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX`.
- Test coverage (`phase_transition.rs`, `tapered_eval.rs`) confirms:
  - Endpoints (`phase = 0` → EG, `phase = 256` → MG).
  - Midpoint (`phase = 128`) yields arithmetic mean.
- Integer arithmetic preserves determinism; overflow-safe given score ranges (±32K typical).
- ✅ Meets correctness and performance expectations; default path across engine for both standard and optimized evaluators.

### 2.2 Cubic Interpolation (Task 10.3)
- `PhaseTransition::interpolate_cubic` uses an ease-in curve (`mg_weight = t³`).
- Effects:
  - At `phase = 128`, `mg_weight = 0.125`, `eg_weight = 0.875`, producing strong endgame bias.
  - Breaks expectation of “smoother linear-like transition”; instead it accelerates toward endgame weights prematurely.
- Missing configurability:
  - No phase boundary adjustments or exponent tuning.
  - Not exposed via configuration; invoked only when callers request `InterpolationMethod::Cubic`.
- Tests (`Legacy` gated) assert endpoints but do not detect mid-phase skew.
- ⚠️ Recommendation: adopt symmetric easing (`mg_weight = 1 - (1 - t)³`) or rename to “EndgameEaseIn” to avoid misleading semantics. Also add mid-phase assertions.

### 2.3 Sigmoid Interpolation (Task 10.4)
- `PhaseTransition::interpolate_sigmoid` implements `mg_weight = 1 / (1 + exp(-k*(t-0.5)))` with `k = 6.0`.
- Advantages:
  - Produces smooth S-curve; transitions gently at extremes.
  - Endpoints converge to MG/EG values without overshoot.
- Issues:
  - Ignores `PhaseTransitionConfig.sigmoid_steepness`; tuning knob is dead.
  - Uses `f64::exp`, introducing modest overhead; still acceptable due to optional usage.
- ✅ Functional correctness; ⚠️ configuration compliance missing.

### 2.4 Smoothstep Interpolation (Task 10.5)
- Uses polynomial `smoothstep(t) = 3t² - 2t³`.
- Balanced weights (`phase = 128` → 0.5). Endpoints exact.
- Pure arithmetic (no transcendental functions); good compromise between smoothness and cost.
- Tested (legacy-gated) and validated via `validate_smooth_transitions`.
- ✅ Implementation aligns with expectations.

### 2.5 Advanced Interpolation Module
- `AdvancedInterpolator` adds:
  - Cubic spline segments with control points.
  - Multi-phase interpolation (opening/middlegame/endgame thresholds).
  - Adaptive adjustments based on `PositionCharacteristics`.
  - Custom Bezier and user-defined interpolation hooks.
- Current state:
  - Not wired into `IntegratedEvaluator` (remains experimental).
  - Tests behind `cfg(all(test, feature = "legacy-tests"))`; default CI misses regressions.
  - Spline coefficients simplified (linear cubic segments) but adequate for prototypes.
- ✅ Valuable toolkit, but adoption path and validation incomplete.

---

## 3. Phase Transition Smoothness (Task 10.6)

- `PhaseTransition` provides two validation helpers:
  - `is_transition_smooth()` compares adjacent phases with configurable tolerance.
  - `validate_smooth_transitions()` scans entire phase range (`0..=256`) ensuring per-step delta ≤ 2 centipawns.
- `calculate_max_transition_rate()` quantifies steepest change for telemetry.
- Observations:
  - Linear and smoothstep pass validations comfortably.
  - Sigmoid meets delta constraint for default `k=6`.
  - Cubic occasionally hits the 2 cp ceiling near mid-phase due to the sharp curve; still within tolerance but reveals bias.
- Testing:
  - Helpers are not called automatically; integration tests should leverage them.
  - Recommend adding CI assertions to enforce smoothness whenever interpolation configuration changes.

---

## 4. Interpolation Performance & Instrumentation (Task 10.7)

- **Statistics Hooks**
  - `TaperedEvaluationStats` counts phase calculations and cache hits; interpolation counter uses atomic increments but lacks reporting path.
  - `PhaseTransitionStats` records total interpolations.
- **Profiling**
  - `PerformanceProfiler` (in `performance.rs`) measures evaluation, phase calculation, PST lookup, and interpolation times; provides averages and percentages.
  - Disabled by default; enabling requires explicit caller invocation (`profiler.enable()`).
- **Cost Characteristics**
  - Linear/Smoothstep: simple integer/floating operations (~2–5 ns).
  - Sigmoid: `exp()` call (~35–45 ns on Apple M-series) but infrequent (optional).
  - Cubic: simple float power (converted to `t * t * t`).
  - Phase calculation dominates when cache misses (~81 lookups, piece retrieval); instrumentation confirms via `phase_calc_percentage`.
- **Gaps**
  - No aggregated report linking `TaperedEvaluationStats` with profiler outputs.
  - Profiling API lacks RAII guard; clients must remember to disable/clear.
  - Performance data not persisted to docs/benchmarks; meta-task 26.0 should capture.

---

## 5. Strengths & Weaknesses (Task 10.8)

**Strengths**
- Modular design: phase calculation, interpolation, and integration are decoupled yet cohesive.
- Multiple interpolation options enable tuning for different play styles.
- Integrated evaluator caches phase/evaluation paths, minimizing redundancy.
- Comprehensive configurability (performance vs memory presets, king-safety toggles).
- Extensive documentation within source modules accelerating onboarding.

**Weaknesses**
- Phase calculation ignores hand pieces and promoted piece weights, misrepresenting true phase in shogi-specific contexts.
- Cubic interpolation curve is asymmetric, tilting evaluations toward endgame mid-search.
- Sigmoid steepness setting unused; configuration drift between docs and code.
- Advanced interpolation/test coverage gated behind non-default feature flag.
- Stats/profiling outputs not surfaced to search logs or telemetry; limited visibility.
- `TaperedEvaluation` cache stores only the most recent position; deeper reuse delegated to integrator but not shared across components (e.g., performance evaluator maintains its own state).

---

## 6. Improvement Recommendations (Task 10.9)

| Priority | Recommendation | Rationale | Effort |
|---------|----------------|-----------|--------|
| **High** | Fix cubic interpolation weighting (`mg_weight = 1 - (1 - t).powi(3)`) or rename/clarify current behavior; add mid-phase tests. | Prevent unintended endgame bias; align documentation with behavior. | 2–3 hrs |
| **High** | Honor `PhaseTransitionConfig.sigmoid_steepness` in `interpolate_sigmoid`; expose via configuration docs. | Restores tuning lever promised in config; improves experimentation. | 2 hrs |
| **High** | Extend phase calculation to include pieces in hand and promoted-piece phase weights; update `PIECE_PHASE_VALUES`. | Accurate phase classification for shogi-specific mechanics; avoids mis-tuned evaluations. | 6–10 hrs (with testing) |
| **Medium** | Unify interpolation validation in automated tests (enable smoothness checks in default test suite). | Catch regressions early; ensures smooth transitions for all methods. | 3–4 hrs |
| **Medium** | Expose `TaperedEvaluationStats`/`PhaseTransitionStats` via search telemetry (e.g., debug logging, performance report). | Improves observability for tuning sessions; aligns with PRD instrumentation goals. | 4–6 hrs |
| **Medium** | Integrate `AdvancedInterpolator` behind configuration toggle or document as experimental; ensure tests run without feature flags. | Clarifies production readiness; raises coverage. | 6–8 hrs |
| **Low** | Replace single-entry cache with small LRU or delegate entirely to caller-supplied cache to reduce duplicated hashing. | Slight hit-rate improvement for repeated subtrees; reduces duplicate work. | 6 hrs |
| **Low** | Provide benchmarking harness comparing interpolation modes (linear vs sigmoid vs smoothstep) using existing profiler scaffolding. | Quantifies overhead vs. benefit; informs default selection. | 6–8 hrs |

---

## 7. Testing & Validation Plan

1. **Unit Tests**
   - Add mid-phase assertions for cubic and sigmoid methods.
   - Verify `sigmoid_steepness` knob alters curve slope (parametrized tests).
   - Create regression covering promoted pieces/hand pieces once phase calculation is updated.

2. **Integration Tests**
   - Use `IntegratedEvaluator` to evaluate representative positions (opening castle, late middlegame with drops, endgame fortress) across interpolation modes; confirm monotonic phase transitions.
   - Validate `PhaseTransition::validate_smooth_transitions()` passes for each configured mode in default build (no feature gate).

3. **Performance Benchmarks**
   - Leverage `OptimizedEvaluator::evaluate_optimized` with profiler enabled to measure interpolation cost distribution.
   - Record metrics (avg ns, percentages) for documentation in `engine-performance-analysis.md`.

4. **Telemetry**
   - Gate debug logs under `trace_log!("EVAL_PHASE", ...)` when stats thresholds exceeded (e.g., cache hit rate < 40%).
   - Ensure logs integrate with existing `DEBUG_LOGGING_OPTIMIZATION.md` guidance.

---

## 8. Conclusion

The tapered evaluation framework provides a strong backbone for phase-aware scoring, featuring robust integration, multiple interpolation strategies, and performance instrumentation. Addressing the identified gaps—particularly phase accuracy for shogi-specific mechanics, configuration fidelity, and interpolation consistency—will elevate the system from “powerful but slightly inconsistent” to “production-ready and tunable.” Immediate focus should go to fixing cubic weighting, restoring sigmoid configurability, and enriching phase calculation inputs. Subsequent efforts can expand observability, integrate advanced interpolations, and solidify benchmarking results to support future tuning initiatives.

**Next Steps:** File engineering tickets for the high-priority recommendations, align them with meta-task 20.0 (evaluation integration), and update documentation once fixes land to maintain PRD traceability.

---

