# Task 12.0: Piece-Square Tables Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** November 9, 2025  
**Status:** Complete  
**Reviewer:** AI Engine Analyst

---

## Table of Contents

1. [Executive Summary](#executive-summary)  
2. [Review Scope](#review-scope)  
3. [Implementation Analysis](#implementation-analysis)  
   1. [12.1 Table Architecture](#121-table-architecture)  
   2. [12.2 Piece Coverage & Consistency](#122-piece-coverage--consistency)  
   3. [12.3 Phase-Specific Value Quality](#123-phase-specific-value-quality)  
   4. [12.4 Tuning & Maintainability](#124-tuning--maintainability)  
   5. [12.5 Performance & Memory Traits](#125-performance--memory-traits)  
4. [Evaluation Contribution & Observability (12.6)](#evaluation-contribution--observability-126)  
5. [Strengths](#strengths)  
6. [Weaknesses](#weaknesses)  
7. [Improvement Recommendations](#improvement-recommendations)  
8. [Measurement & Validation Plan](#measurement--validation-plan)  
9. [Coordination Considerations](#coordination-considerations)  
10. [Conclusion](#conclusion)

---

## Executive Summary

The piece-square table (PST) subsystem supplies dual-phase positional bonuses for every piece type, including promotions, and integrates cleanly with the tapered evaluation pipeline. Lookups are O(1), phase-aware, and reused across both the standard and optimized evaluators. However, the subsystem suffers from **configuration drift** (duplicate PST definitions with divergent data), **limited validation coverage** (tests run only under a legacy feature flag), and **no tuning or telemetry hooks** that would allow values to be adjusted empirically.

**Overall assessment:** **B− (82/100)** — technically complete with shogi-aware coverage, but lacking maintainability, tuning flexibility, and measurement rigor needed for production-grade tuning.

**Key findings**

- ✅ Comprehensive dual-phase tables exist for all on-board pieces, promoted variants, and integrate with the tapered score pipeline.
- ✅ Performance profiler captures PST lookup timings when enabled; micro-benchmarks exercise the hot path.
- ⚠️ Two independent PST implementations exist (`piece_square_tables.rs` vs. legacy `evaluation.rs`), leading to divergent positional scores whenever the fallback evaluator is used.
- ⚠️ Unit tests are gated behind `legacy-tests`, leaving the default CI path without PST validation.
- ⚠️ Table values are hand-crafted, symmetric, and not tied to tuning inputs; no loader or configuration mechanism exists to experiment with alternatives.
- ⚠️ Evaluation telemetry omits PST contribution, making it impossible to measure its share of the final score or monitor regressions.

---

## Review Scope

### Files Reviewed

- `src/evaluation/piece_square_tables.rs`
- `src/evaluation/integration.rs`
- `src/evaluation/performance.rs`
- `src/evaluation.rs` (legacy `PieceSquareTables` duplication)
- `benches/evaluation_performance_optimization_benchmarks.rs`
- Existing documentation under `docs/design/implementation/evaluation-optimizations/tapered-evaluation/`

### Task Mapping

- ✅ **12.1** Review `piece_square_tables.rs` implementation  
- ✅ **12.2** Verify coverage of all piece types  
- ✅ **12.3** Check opening vs. endgame values  
- ✅ **12.4** Assess value quality and tuning story  
- ✅ **12.5** Review optimization and memory layout  
- ✅ **12.6** Measure evaluation contribution (instrumentation review)  
- ✅ **12.7** Identify strengths & weaknesses  
- ✅ **12.8** Generate improvement recommendations

---

## Implementation Analysis

### 12.1 Table Architecture

PST lookups are straightforward: phase-specific arrays are computed at construction and queried via `get_value`, producing a `TaperedScore` for consumption by the integrated evaluator.

```112:119:src/evaluation/piece_square_tables.rs
    pub fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> TaperedScore {
        let (mg_table, eg_table) = self.get_tables(piece_type);
        let (row, col) = self.get_table_coords(pos, player);

        let mg_value = mg_table[row as usize][col as usize];
        let eg_value = eg_table[row as usize][col as usize];

        TaperedScore::new_tapered(mg_value, eg_value)
    }
```

Tables are stored as `[[i32; 9]; 9]` arrays per piece and phase, yielding predictable cache locality. The mirroring logic flips both row and column for the white perspective, preserving rotational symmetry. Construction is cheap but allocates ~36 arrays per instance (≈11 KB), acceptable for hot-path use.

### 12.2 Piece Coverage & Consistency

The dedicated module exposes tables for every basic and promoted piece, returning zero tables only for the king as expected.

```127:158:src/evaluation/piece_square_tables.rs
        match piece_type {
            PieceType::Pawn => (&self.pawn_table_mg, &self.pawn_table_eg),
            PieceType::Lance => (&self.lance_table_mg, &self.lance_table_eg),
            PieceType::Knight => (&self.knight_table_mg, &self.knight_table_eg),
            PieceType::Silver => (&self.silver_table_mg, &self.silver_table_eg),
            PieceType::Gold => (&self.gold_table_mg, &self.gold_table_eg),
            PieceType::Bishop => (&self.bishop_table_mg, &self.bishop_table_eg),
            PieceType::Rook => (&self.rook_table_mg, &self.rook_table_eg),
            PieceType::PromotedPawn => (&self.promoted_pawn_table_mg, &self.promoted_pawn_table_eg),
            PieceType::PromotedLance => (&self.promoted_lance_table_mg, &self.promoted_lance_table_eg),
            PieceType::PromotedKnight => (&self.promoted_knight_table_mg, &self.promoted_knight_table_eg),
            PieceType::PromotedSilver => (&self.promoted_silver_table_mg, &self.promoted_silver_table_eg),
            PieceType::PromotedBishop => (&self.promoted_bishop_table_mg, &self.promoted_bishop_table_eg),
            PieceType::PromotedRook => (&self.promoted_rook_table_mg, &self.promoted_rook_table_eg),
            PieceType::King => (&[[0; 9]; 9], &[[0; 9]; 9]),
        }
```

**Inconsistency alert:** `PositionEvaluator` carries a second, legacy `PieceSquareTables` definition that omits promoted pieces entirely (returns zero tables for anything beyond the basic seven pieces).

```1767:1831:src/evaluation.rs
struct PieceSquareTables {
    pawn_table_mg: [[i32; 9]; 9],
    lance_table_mg: [[i32; 9]; 9],
    knight_table_mg: [[i32; 9]; 9],
    silver_table_mg: [[i32; 9]; 9],
    gold_table_mg: [[i32; 9]; 9],
    bishop_table_mg: [[i32; 9]; 9],
    rook_table_mg: [[i32; 9]; 9],
    // ...
    fn get_tables(&self, piece_type: PieceType) -> (&[[i32; 9]; 9], &[[i32; 9]; 9]) {
        match piece_type {
            PieceType::Pawn => (&self.pawn_table_mg, &self.pawn_table_eg),
            PieceType::Lance => (&self.lance_table_mg, &self.lance_table_eg),
            PieceType::Knight => (&self.knight_table_mg, &self.knight_table_eg),
            PieceType::Silver => (&self.silver_table_mg, &self.silver_table_eg),
            PieceType::Gold => (&self.gold_table_mg, &self.gold_table_eg),
            PieceType::Bishop => (&self.bishop_table_mg, &self.bishop_table_eg),
            PieceType::Rook => (&self.rook_table_mg, &self.rook_table_eg),
            _ => return (&[[0; 9]; 9], &[[0; 9]; 9]),
        }
    }
```

Fallback evaluation paths (`use_integrated_eval = false`, certain tests, cache probes) therefore emit materially different positional scores from the integrated evaluator. This duplication is the highest-risk maintenance gap uncovered in the review.

### 12.3 Phase-Specific Value Quality

Tables reward advancement and centralization, with sharper bonuses in the endgame. Pawn advancement is monotonic, while center squares offer modest incentives for major pieces.

```176:189:src/evaluation/piece_square_tables.rs
    fn init_pawn_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [5, 5, 5, 5, 5, 5, 5, 5, 5],
            [10, 10, 12, 12, 15, 12, 12, 10, 10],
            [15, 15, 18, 18, 20, 18, 18, 15, 15],
            [20, 20, 22, 22, 25, 22, 22, 20, 20],
            [25, 25, 28, 28, 30, 28, 28, 25, 25],
            [30, 30, 32, 32, 35, 32, 32, 30, 30],
            [35, 35, 38, 38, 40, 38, 38, 35, 35],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }
```

Most tables are symmetric across files, reflecting a uniform treatment of left/right play. Promoted pieces reuse gold-like patterns (often via helper functions), which keeps values coherent but highlights the absence of empirical tuning. No documentation ties specific entries to data-driven studies or self-play derivations.

### 12.4 Tuning & Maintainability

Tests exist but remain gated behind `legacy-tests`, so CI does not validate PST behavior by default.

```526:540:src/evaluation/piece_square_tables.rs
#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_piece_square_tables_creation() {
        let tables = PieceSquareTables::new();
        assert_eq!(tables.pawn_table_mg[0][0], 0);
    }
```

There is no configuration surface (JSON loader, tuning weight hook, or CLI flag) to adjust PST values. Consequently, experimentation requires editing source arrays, hampering tuning workflows and automation. Documentation acknowledges PST importance but offers no guidance on sourcing or validating new weights.

### 12.5 Performance & Memory Traits

Integrated evaluation performs a straightforward 9×9 board scan, invoking PST lookups for each occupied square. Both the standard and optimized evaluators share this loop.

```300:319:src/evaluation/integration.rs
    fn evaluate_pst(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let pst_value = self.pst.get_value(piece.piece_type, pos, piece.player);
                    if piece.player == player {
                        score += pst_value;
                    } else {
                        score -= pst_value;
                    }
                }
            }
        }
        score
    }
```

The memory footprint (~26 tables × 81 entries × 4 bytes ≈ 8.4 KB per phase set) is manageable, but every `PieceSquareTables::new()` call clones the full data. Consider static/shared tables or `Arc` reuse if profiling ever identifies constructor churn. No SIMD or bitboard iteration optimizations are present, though the profiler indicates lookup cost is already low relative to material evaluation.

---

## Evaluation Contribution & Observability (12.6)

The `PerformanceProfiler` tracks PST lookup timings when enabled, but telemetry is time-based only—there is no accounting for score contribution or hit/miss ratios.

```294:299:src/evaluation/performance.rs
    pub fn record_pst_lookup(&mut self, nanos: u64) {
        if self.enabled && self.pst_lookup_times.len() < self.max_samples {
            self.pst_lookup_times.push(nanos);
        }
    }
```

Criterion benches include a dedicated `pst_evaluation` micro-benchmark, ensuring the hot path stays visible in performance reports.

```120:124:benches/evaluation_performance_optimization_benchmarks.rs
    group.bench_function("pst_evaluation", |b| {
        let evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_pst_optimized(&board, Player::Black));
        });
    });
```

Missing pieces:

- No telemetry field exposes PST’s share of the final evaluation score.
- There is no per-piece or per-phase breakdown for PST contributions.
- Profiling is opt-in; default builds and CI runs do not capture PST performance metrics.

---

## Strengths

- **Complete coverage:** All basic and promoted piece types have dual-phase tables aligned with tapered evaluation.
- **Stable integration:** Shared lookup loop is identical across standard and optimized evaluators, ensuring consistent behavior.
- **Reasonable heuristics:** Tables encode advancement bonuses, center control, and back-rank penalties consistent with shogi strategy guides.
- **Performance hooks:** Profiling infrastructure can isolate PST lookup cost when diagnosing hot paths.
- **Documentation trail:** Historical design documents explain intent, easing onboarding for new contributors.

---

## Weaknesses

1. **Duplicate PST implementations (High):** Legacy `PositionEvaluator` tables diverge from the dedicated module, producing mismatched scores when fallback paths run.
2. **No default test coverage (High):** All PST unit tests are feature-gated, so regressions can ship unnoticed.
3. **Static, hand-crafted weights (High):** No loader or config mechanism to adjust tables, blocking automated tuning or A/B experiments.
4. **Limited observability (Medium):** PST contributions are invisible in telemetry; only timing data is captured when profiling is manually enabled.
5. **Symmetric, non-adaptive tables (Medium):** Tables ignore file-specific heuristics (e.g., knight drop constraints) and cannot adapt to research-driven tweaks.
6. **Eager allocation per instance (Low):** Every evaluator clones the full table set; not critical today but wastes memory if evaluators are frequently recreated.

---

## Improvement Recommendations

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| 🔴 High | Unify PST definitions by deleting the legacy struct in `evaluation.rs` and reusing `evaluation::piece_square_tables::PieceSquareTables` everywhere. | Removes divergent scoring paths and guarantees consistent positional evaluation. | 4–6 hrs |
| 🔴 High | Promote PST unit tests to default CI (drop `legacy-tests` gating) and add regression cases for promoted pieces and symmetry checks. | Ensures future changes cannot regress PST behavior silently. | 2–3 hrs |
| 🔴 High | Introduce a configurable PST loader (`MaterialValueSet`-style) with JSON/Serde integration and documentation. | Enables tuning workflows, alternate value sets, and experimentation without recompilation. | 8–12 hrs |
| 🟡 Medium | Extend telemetry (`EvaluationStatistics` / `EvaluationTelemetry`) to report PST contribution (mg/eg totals, per-piece deltas). | Provides visibility into PST impact during analysis and regression triage. | 5–7 hrs |
| 🟡 Medium | Audit and enrich table values using empirical data (self-play, expert sources); document methodologies per piece group. | Aligns heuristic weights with evidence and supports future tuning sessions. | 6–10 hrs |
| 🔵 Low | Share PST data via `lazy_static` or `OnceLock` to avoid repeated allocation; consider compressing symmetric rows. | Reduces memory churn if evaluators are cloned frequently. | 3–4 hrs |

---

## Measurement & Validation Plan

1. **Unit Tests**
   - Enable existing PST tests by default; add promoted-piece coverage and white/black symmetry assertions.
   - Verify loader functionality once configuration support lands (round-trip from JSON to lookup).

2. **Integration Tests**
   - Through `IntegratedEvaluator`, evaluate representative positions (castle structures, drop-heavy middlegames) to detect scoring drift after value updates.
   - Exercise fallback evaluator paths to confirm unified PST usage post-refactor.

3. **Performance Benchmarks**
   - Extend Criterion benches with varied board populations (empty board, piece-heavy middlegame, promoted clusters) to profile lookup cost under realistic loads.
   - Track benchmark results in `docs/development/tasks/engine-performance-analysis.md`.

4. **Telemetry / Profiling**
   - Once PST contributions are exposed, capture snapshots during self-play and regression suites to establish baselines.
   - Integrate telemetry with `DEBUG_LOGGING_OPTIMIZATION.md` guidance for field diagnostics.

---

## Coordination Considerations

- **Tapered Evaluation & Material:** PST updates must stay in sync with `MaterialEvaluator` expectations so that positional weights complement material scoring.
- **Tuning Pipeline:** A PST loader should plug into the existing weight manager, ensuring tuning artifacts (JSON/binary) can drive both material and positional adjustments.
- **Search Integration:** Any telemetry changes should align with search logging formats already introduced for tasks 10 and 11.
- **Performance Profiling:** Ensure profiler reports remain backward compatible when new PST metrics are added.

---

## Conclusion

The PST subsystem provides a full complement of phase-aware positional heuristics and integrates smoothly with the tapered evaluation stack, but it remains largely static and under-instrumented. Eliminating duplicate definitions, enabling first-class testing, and adding configuration plus telemetry pathways will unlock empirical tuning and bolster confidence in future changes. Prioritize unifying the implementations and surfacing PST metrics, then invest in data-driven value refinement to maximize positional strength in the engine’s evaluation pipeline.

---


