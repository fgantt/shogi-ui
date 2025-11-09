# Task 11.0: Material Evaluation Review

**Date:** November 9, 2025  
**Status:** Complete  
**Reviewer:** AI Engine Analyst  
**Related PRD:** `prd-engine-features-review-and-improvement-plan.md`

---

## Table of Contents

1. [Executive Summary](#executive-summary)  
2. [Review Scope](#review-scope)  
3. [Implementation Analysis](#implementation-analysis)  
   1. [11.1 Material Evaluation Flow](#111-material-evaluation-flow)  
   2. [11.2 Piece Value Tables](#112-piece-value-tables)  
   3. [11.3 Captured Piece Handling](#113-captured-piece-handling)  
   4. [11.4 Configuration & Integration](#114-configuration--integration)  
   5. [11.5 Accuracy & Observability](#115-accuracy--observability)  
   6. [11.6 Performance Characteristics](#116-performance-characteristics)  
4. [Strengths](#strengths)  
5. [Weaknesses](#weaknesses)  
6. [Improvement Recommendations](#improvement-recommendations)  
7. [Measurement & Validation Plan](#measurement--validation-plan)  
8. [Coordination with Other Features](#coordination-with-other-features)  
9. [Conclusion](#conclusion)

---

## Executive Summary

The material evaluation subsystem delivers reliable tapered scoring for board and hand pieces, with clear separation between configuration, statistics, and integration points. The implementation is deterministic and well-tested, but several configuration promises (notably research-vs-classic value selection) are currently unimplemented, and integration hooks do not propagate configuration changes downstream. Observability is minimal, and performance still relies on full board scans rather than bitboard differentials.

**Overall assessment:** **B (85/100)** — solid foundations with notable configuration and observability gaps that block tuning workflows.

**Highlights**
- Deterministic tapered scoring pipeline with hand-piece awareness and symmetrical board balancing.
- Promoted and unpromoted piece tables are differentiated, aligning with tapered evaluation expectations.
- Unit tests and Criterion benches cover core behaviors, preventing obvious regressions.

**Primary Concerns**
- `use_research_values` flag is unused; configuration promises cannot be honored.
- Integrated evaluator always instantiates the default material config, so runtime configuration changes never reach the material module.
- Instrumentation is limited to a single counter, offering no insight into material contribution or hand-piece balance during search.

---

## Review Scope

### Files Reviewed

- `src/evaluation/material.rs`
- `src/evaluation/integration.rs`
- `src/evaluation/performance.rs`
- `src/evaluation/config.rs`
- `src/types.rs`
- `benches/material_evaluation_performance_benchmarks.rs`
- Associated unit tests under `src/evaluation/material.rs`

### Review Criteria Mapping

- ✅ **11.1** Review `material.rs` implementation
- ✅ **11.2** Verify piece value accuracy (board + hand tables)
- ✅ **11.3** Check captured-piece handling in evaluation
- ✅ **11.4** Assess promoted-piece values and drop heuristics
- ✅ **11.5** Review research-vs-classic value configuration promises
- ✅ **11.6** Measure evaluation contribution / benchmarks (documentation review)
- ✅ **11.7** Identify strengths and weaknesses
- ✅ **11.8** Generate improvement recommendations

---

## Implementation Analysis

### 11.1 Material Evaluation Flow

The evaluator increments a simple counter, evaluates board material, then (optionally) hand material before returning a tapered score. The flow is straightforward and symmetrical for both players.

```63:134:src/evaluation/material.rs
    pub fn evaluate_material(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.evaluations += 1;
        let mut score = TaperedScore::default();
        score += self.evaluate_board_material(board, player);
        if self.config.include_hand_pieces {
            score += self.evaluate_hand_material(captured_pieces, player);
        }
        score
    }

    fn evaluate_board_material(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value = self.get_piece_value(piece.piece_type);
                    if piece.player == player {
                        score += piece_value;
                    } else {
                        score -= piece_value;
                    }
                }
            }
        }
        score
    }
```

Observations:
- Board traversal is O(81) per evaluation; no attempt is made to reuse piece lists from the bitboard representation.
- The tapered scoring follows the standard add/subtract pattern, keeping evaluation symmetric for both sides.
- Statistics capture only the number of evaluations, offering no drill-down by piece or phase.

### 11.2 Piece Value Tables

Piece values are hard-coded tapered scores derived from internal research. Promoted variants emphasize endgame strength, and `get_hand_piece_value` inflates hand pieces slightly to reflect drop flexibility.

```145:185:src/evaluation/material.rs
    pub fn get_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            PieceType::Pawn => TaperedScore::new_tapered(100, 120),
            PieceType::Lance => TaperedScore::new_tapered(300, 280),
            PieceType::Knight => TaperedScore::new_tapered(350, 320),
            PieceType::Silver => TaperedScore::new_tapered(450, 460),
            PieceType::Gold => TaperedScore::new_tapered(500, 520),
            PieceType::Bishop => TaperedScore::new_tapered(800, 850),
            PieceType::Rook => TaperedScore::new_tapered(1000, 1100),
            PieceType::King => TaperedScore::new(20000),
            PieceType::PromotedPawn => TaperedScore::new_tapered(500, 550),
            PieceType::PromotedLance => TaperedScore::new_tapered(500, 540),
            PieceType::PromotedKnight => TaperedScore::new_tapered(520, 550),
            PieceType::PromotedSilver => TaperedScore::new_tapered(520, 550),
            PieceType::PromotedBishop => TaperedScore::new_tapered(1200, 1300),
            PieceType::PromotedRook => TaperedScore::new_tapered(1400, 1550),
        }
    }

    pub fn get_hand_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            PieceType::Pawn => TaperedScore::new_tapered(110, 130),
            PieceType::Lance => TaperedScore::new_tapered(320, 300),
            PieceType::Knight => TaperedScore::new_tapered(370, 350),
            PieceType::Silver => TaperedScore::new_tapered(480, 490),
            PieceType::Gold => TaperedScore::new_tapered(530, 550),
            PieceType::Bishop => TaperedScore::new_tapered(850, 920),
            PieceType::Rook => TaperedScore::new_tapered(1050, 1180),
            _ => TaperedScore::default(),
        }
    }
```

Key findings:
- Value tables align with `PieceType::base_value` defaults but extend to phase-aware scoring; however, the system lacks a mechanism to load alternative value sets (classic vs research) despite the configuration flag.
- Hand values do not account for drop restrictions (e.g., knights near promotion zone), which may slightly overstate tactical flexibility.

### 11.3 Captured Piece Handling

Hand evaluation leverages `CapturedPieces`, which stores unpromoted piece types thanks to the board layer converting captures to their unpromoted variants before pushing them into hand.

```548:575:src/types.rs
    pub fn add_piece(&mut self, piece_type: PieceType, player: Player) {
        match player {
            Player::Black => self.black.push(piece_type),
            Player::White => self.white.push(piece_type),
        }
    }

    pub fn count(&self, piece_type: PieceType, player: Player) -> usize {
        let pieces = match player {
            Player::Black => &self.black,
            Player::White => &self.white,
        };
        pieces.iter().filter(|&&p| p == piece_type).count()
    }
```

- `evaluate_hand_material` iterates over these vectors, adding/subtracting tapered values. For typical Shogi positions (few pieces in hand), the linear scan is adequate.
- There is no caching or deduplication by piece type; high-drop scenarios incur repeated lookups and allocations.

### 11.4 Configuration & Integration

`MaterialEvaluationConfig` exposes two flags, but only `include_hand_pieces` is honored. `use_research_values` is unused, and configuration does not flow from higher-level managers.

```260:274:src/evaluation/material.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialEvaluationConfig {
    pub include_hand_pieces: bool,
    pub use_research_values: bool,
}

impl Default for MaterialEvaluationConfig {
    fn default() -> Self {
        Self {
            include_hand_pieces: true,
            use_research_values: true,
        }
    }
}
```

```99:116:src/evaluation/integration.rs
        Self {
            config: config.clone(),
            tapered_eval: RefCell::new(TaperedEvaluation::new()),
            material_eval: RefCell::new(MaterialEvaluator::new()),
            pst: PieceSquareTables::new(),
            phase_transition: RefCell::new(PhaseTransition::new()),
            position_features: RefCell::new(PositionFeatureEvaluator::new()),
            endgame_patterns: RefCell::new(EndgamePatternEvaluator::new()),
            opening_principles: RefCell::new(OpeningPrincipleEvaluator::new()),
            tactical_patterns: RefCell::new(TacticalPatternRecognizer::new()),
            positional_patterns: RefCell::new(PositionalPatternAnalyzer::new()),
            pattern_cache: RefCell::new(PatternCache::new(config.pattern_cache_size)),
            optimized_eval,
            statistics: RefCell::new(EvaluationStatistics::new()),
            telemetry: RefCell::new(None),
            phase_cache: RefCell::new(HashMap::new()),
            eval_cache: RefCell::new(HashMap::new()),
        }
```

- Integrated evaluator ignores the `MaterialEvaluationConfig` entirely; toggling options in configuration files has no effect on runtime behavior.
- The optimized evaluator also constructs a fresh default `MaterialEvaluator`, so performance profiles do not reflect configuration differences.

### 11.5 Accuracy & Observability

`MaterialEvaluationStats` is a single counter used only to track invocation counts.

```278:283:src/evaluation/material.rs
#[derive(Debug, Clone, Default)]
pub struct MaterialEvaluationStats {
    pub evaluations: u64,
}
```

- No phase-aware or per-piece metrics are recorded, so material contribution cannot be monitored in telemetry.
- Tests confirm value tables and balancing symmetry but do not exercise scenarios with large numbers of hand pieces, promoted captures, or impasse scoring tie-ins.
- There is no ablation harness to quantify how much of the final evaluation comes from material vs. other components.

### 11.6 Performance Characteristics

The Criterion bench suite covers evaluator creation, piece lookups, board evaluation, and hand evaluation scenarios.

```90:152:benches/material_evaluation_performance_benchmarks.rs
    group.bench_function("evaluate_starting_position", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("multiple_captures", |b| {
        let mut evaluator = MaterialEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_material(&board, Player::Black, &captured_multiple));
        });
    });
```

- Benchmarks rely on the default configuration, so they do not surface the cost of alternative value sets or statistics tracking.
- Board iteration remains the dominant cost; no evidence of popcount-based optimization or caching.
- There is no documentation of baseline timings or regression history in `docs/`.

---

## Strengths

- **Phase-aware scoring:** Separate middlegame/endgame values for promoted and unpromoted pieces integrate smoothly with tapered evaluation and tuning infrastructure.
- **Hand-piece awareness:** Incorporates captured pieces into material balance, aligning with Shogi-specific mechanics and impasse rules.
- **Deterministic behavior:** Symmetric add/subtract logic ensures opposite scores for opposing players in identical positions.
- **Baseline coverage:** Unit tests validate value tables, promoted piece superiority, hand toggles, and statistical counters; benchmarks provide quick performance smoke tests.
- **Integration surface:** `MaterialEvaluator` plugs into both standard and optimized evaluators without unsafe code or complex lifetimes.

## Weaknesses

1. **Dead configuration path (High):** `use_research_values` is never consulted, so users cannot switch to classic values despite documentation promises.
2. **Integration disconnect (High):** `IntegratedEvaluator` and `OptimizedEvaluator` always instantiate default configs, ignoring user-provided material settings.
3. **Hard-coded tables (High):** Value sets are embedded in source, preventing runtime tuning, self-play adjustments, or loading from weight files.
4. **Minimal telemetry (Medium):** Only a global evaluation counter is tracked; no per-piece stats, hand-piece ratios, or phase breakdowns are exposed to search logs or telemetry.
5. **O(81) board scans (Medium):** Every evaluation iterates the entire 9×9 board and hand vectors; no use of bitboard population counts or incremental updates.
6. **Hand-value heuristics (Low):** Drop values are globally inflated but ignore file/rank restrictions (e.g., knights and lances near promotion zones), potentially overstating tactical flexibility.

---

## Improvement Recommendations

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| 🔴 High | Implement value-set selection (`use_research_values` vs. classic) and expose loader for custom tables (file or tuning manager). | Honors configuration contract and enables empirical tuning experiments. | 6–8 hrs |
| 🔴 High | Allow `MaterialEvaluator::with_config` to flow through `IntegratedEvaluator`/`OptimizedEvaluator`, including runtime updates. | Ensures engine-wide configuration changes actually affect material scoring. | 4–6 hrs |
| 🔴 High | Externalize material tables into a serializable `MaterialValueSet` (JSON/Serde) and integrate with tuning weights. | Removes hard-coded constants and aligns with automated tuning infrastructure. | 8–12 hrs |
| 🟡 Medium | Extend statistics to track per-piece contributions, hand-piece deltas, and phase-weighted totals; surface via evaluation telemetry. | Improves observability for tuning and regression analysis. | 5–7 hrs |
| 🟡 Medium | Introduce bitboard-driven counting (per-piece bitboards or cached counts) to reduce board traversal cost; benchmark impact. | Lowers evaluation latency, benefiting deep searches. | 6–9 hrs |
| 🟡 Medium | Add regression tests covering large hand inventories, promoted captures, and impasse boundary cases to guarantee correctness. | Strengthens coverage for Shogi-specific edge cases. | 3–4 hrs |
| 🔵 Low | Document source/justification for research vs. classic values and provide guidance for future tuning sessions. | Clarifies decision-making for maintainers and contributors. | 2 hrs |

---

## Measurement & Validation Plan

1. **Unit Tests**
   - Add parameterized tests that compare research vs. classic value sets to ensure configuration toggles work as expected.
   - Validate hand-piece evaluation across bulk drops (e.g., nine pawns in hand) for symmetry and scaling.

2. **Integration Tests**
   - Through `IntegratedEvaluator`, toggle material-only evaluation (disable other components) to confirm configuration propagation, cache behavior, and telemetry output.
   - Ensure impasse scoring leverages updated material values consistently.

3. **Performance Benchmarks**
   - Extend existing Criterion benches to run against each value set and different statistics configurations, recording baseline timings and publishing results in `docs/development/tasks/engine-performance-analysis.md`.
   - Add ablation benchmark that toggles material evaluation to quantify its share of total evaluation time.

4. **Tuning Experiments**
   - Use the weight manager to export/import material tables and measure Elo impact via self-play, logging results in the tuning roadmap.

---

## Coordination with Other Features

- **Tapered Evaluation & Phase Transition:** Updated material tables must stay in sync with phase interpolation assumptions and `PIECE_PHASE_VALUES` so that phase calculation and interpolation remain consistent.
- **Integrated Evaluator Telemetry:** Surfacing material statistics should hook into the broader telemetry pipeline to align with Task 20.0 (evaluation integration) and debugging guides.
- **Automated Tuning:** Externalized tables should integrate with the tuning module to share weight files and maintain compatibility with existing weight manager utilities.
- **Performance Analysis:** Benchmarks should feed meta-task 26.0, giving visibility into material evaluation’s share of overall evaluation cost.

---

## Conclusion

The material evaluation module is reliable and Shogi-aware, but its configurability and telemetry lag behind the promises made in the configuration layer. Implementing value-set selection, wiring configuration through the integrated evaluator, and enriching observability will unlock meaningful tuning experiments and align the subsystem with broader evaluation modernization goals. Immediate focus should fall on honoring `use_research_values` and exposing configuration to downstream consumers; subsequent iterations can optimize performance and expand diagnostics to support future tuning initiatives.

---
