# Task 15.0: Positional Pattern Recognition Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** December 2024  
**Status:** Complete

---

## Executive Summary

The positional pattern recognizer aspires to cover center control, outposts, weak squares, activity, space, and tempo, but it currently reflects **chess-centric heuristics that mis-handle core shogi mechanics**. Every detector operates on naive board scans that ignore blockers, hands, and promoted pieces; several concepts (pawn support, pawn threats) are directly imported from chess and therefore mis-evaluate shogi positions. Integration contributes raw scores without weights, telemetry, or phase awareness, making the signal noisy and hard to tune. Overall grade: **D+ (66/100)** — useful scaffolding, but accuracy, performance, and shogi fidelity must be overhauled before the module can influence search.

Key findings:

- ⚠️ All detectors assume on-board context only; hands (`CapturedPieces`) are ignored despite their decisive impact on positional pressure.
- ❌ Center/outpost logic counts occupying pieces instead of *control* and models pawn support/attack diagonally, contradicting shogi movement rules.
- ❌ Weak-square and space evaluation reuse blocker-agnostic attack checks, flagging phantom control and missing lance/dragon/bishop influence.
- ⚠️ Integration bypasses component weights and telemetry; stats lack snapshots, so positional metrics never surface in evaluator reports.

---

## Relevant Files

### Primary Implementation
- `src/evaluation/positional_patterns.rs` – Positional pattern analyzer, configuration, and tests.

### Supporting / Integration
- `src/evaluation/integration.rs` – Invokes positional analyzer without weighting (`IntegratedEvaluator` pipeline).
- `src/evaluation/pattern_config.rs` – Pattern component toggles and weights (positional weight defined but unused).

### Testing & Instrumentation
- No dedicated positional fixtures or benchmarks; unit tests only cover construction and statistics counters.

---

## 1. Implementation Review (Task 15.1)

- **Analyzer structure:** `PositionalPatternAnalyzer` owns a `PositionalConfig` of feature toggles and scalar bonuses plus a `PositionalStats` counter set. Evaluation runs every enabled detector sequentially, accumulating tapered scores without intermediate weighting or normalization.
- **Shogi awareness:** Evaluation API accepts only `board` and `player`; it never inspects `CapturedPieces`, so drop threats/support and hand-based defenses are invisible.
- **Statistics:** `PositionalStats` is a plain POD struct with u64 counters but no snapshot/merge helpers, so integration cannot expose stats alongside other telemetry modules.
- **Performance:** Each detector performs nested 9×9 scans and allocates temporary `Vec`s; no reuse of bitboards, attack tables, or incremental caches that exist elsewhere in the evaluation stack.

---

## 2. Center Control Evaluation (Task 15.2)

- **Occupancy vs. control:** The core center bonus checks only pieces *occupying* 3×3 / 5×5 regions and assigns fixed centipawn values, ignoring actual attack coverage or mobility.

```143:178:src/evaluation/positional_patterns.rs
        for (row, col) in center_squares {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                let value = self.get_center_piece_value(piece.piece_type, true);
                if piece.player == player {
                    mg_score += value.0;
                    eg_score += value.1;
                } else {
                    mg_score -= value.0;
                    eg_score -= value.1;
                }
            }
        }
        // ...
        let pawn_control = self.count_pawn_center_control(board, player, &center_squares);
```

- **Chess-based assumptions:** `count_pawn_center_control` only rewards pawns already seated on the center squares, not those controlling them from adjacent files or via drops. `get_center_piece_value` ignores promoted pieces and sets identical multipliers for core and extended regions.
- **No blocker awareness:** Sliding pieces receive full credit even if allied pieces block their influence; standstill pieces in prison-like formations still get positive scores.
- **Phase tuning absent:** Middlegame/endgame splits are hard-coded but identical scaling (dividing by 2) regardless of configuration or phase transition data, yielding stagnant adjustments.

---

## 3. Outpost Detection (Task 15.3)

- **Misapplied pawn logic:** `is_outpost` checks for pawn support on diagonals behind the piece and threats on diagonals ahead—patterns valid in chess but illegal in shogi, where pawns move and capture straight forward.

```248:285:src/evaluation/positional_patterns.rs
        let has_pawn_support = self.has_pawn_support(board, pos, player);
        let enemy_pawn_threat = self.is_under_enemy_pawn_threat(board, pos, player);
        let is_good_piece = matches!(
            piece_type,
            PieceType::Knight | PieceType::Silver | PieceType::Gold
        );
        has_pawn_support && !enemy_pawn_threat && is_good_piece
```

- **Drops ignored:** Outposts can be undermined instantly by pawn drops, but the detector neither considers hands nor checks empty adjacent squares where drops would occur.
- **Piece coverage gaps:** Lances, promoted minors, and rooks never qualify as “good pieces,” so quintessential shogi outposts (e.g., promoted silver on 5e) are undervalued.
- **Depth bonus inflation:** `get_outpost_value` adds linear depth bonuses that can exceed material value without considering board state or opponent counterplay.

---

## 4. Weak Square Identification (Task 15.4)

- **File scan heuristic:** `can_be_defended_by_pawn` scans entire files for friendly pawns regardless of promotion status or intervening pieces. It treats any pawn on the file as a valid defender, even if it sits far away or is blocked.

```424:449:src/evaluation/positional_patterns.rs
        for dc in pawn_files {
            // ...
            for row in 0..9 {
                let check_pos = Position::new(row, file as u8);
                if let Some(piece) = board.get_piece(check_pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        return true;
                    }
                }
            }
        }
```

- **Attack detection ignores blockers:** `is_controlled_by_opponent` walks all enemy pieces and calls `piece_attacks_square`, which disregards occupancy for sliders. Squares shielded by friendly pieces can still be flagged as weak because enemy rooks “attack” through blockers.

```452:507:src/evaluation/positional_patterns.rs
        if self.piece_attacks_square(
            board,
            check_pos,
            pos,
            piece.piece_type,
            opponent,
        ) {
            return true;
        }
```

- **Shogi motif gaps:** Lance pressure, promoted rook/bishop moves, and knight drops are absent. Key squares list only central files around a default king location, ignoring castle geometries (Anaguma, Mino, Yagura) that relocate the king.

---

## 5. Space Advantage Calculation (Task 15.5)

- **Quadratic scans:** `count_controlled_squares` iterates every board square and invokes the same blocker-free `piece_attacks_square` routine, resulting in O(81²) lookups and pervasive false positives.

```583:596:src/evaluation/positional_patterns.rs
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if self.is_controlled_by_opponent(board, pos, player) {
                    count += 1;
                }
            }
        }
```

- **Lack of scaling:** The raw difference in “controlled” squares is multiplied by a fixed constant (2 mg / 0.66 eg) regardless of phase or board context, and negative values are unbounded—positions with mis-counted coverage can swing hundreds of centipawns.
- **No territory modeling:** Space in shogi depends on ranks/files relative to the player, open files, and pawn structure. Current detection ignores side-specific territory and piece drops, so space bonuses fire in closed fortress positions where no expansion exists.

---

## 6. Positional Understanding Quality (Task 15.6)

- **Unweighted integration:** `IntegratedEvaluator` sums positional scores directly into the tapered total, unlike tactical patterns which honor `weights.tactical_weight`. There is no knob to dampen noisy signals or disable subcomponents independently.

```274:279:src/evaluation/integration.rs
        if self.config.components.positional_patterns {
            total += self
                .positional_patterns
                .borrow_mut()
                .evaluate_position(board, player);
        }
```

- **Stats unusable:** `PositionalStats` provides no snapshot/clone helpers, so integration cannot export evaluator usage; telemetry dashboards never display positional workloads.
- **Testing gaps:** Unit tests assert only that scores are non-negative or counters increment; no fixtures validate center control, outposts, or weak squares on real positions. Regression risk is high.
- **Instrumentation missing:** No profiler hooks, caching, or early exits exist. Each evaluation re-scans the full board even if intermediate results (e.g., key squares) could be cached or derived from other evaluators (king safety, mobility).

---

## 7. Strengths & Weaknesses (Task 15.7)

**Strengths**
- Modular configuration toggles allow selective enabling for experiments.
- Returns `TaperedScore`, aligning with the rest of the evaluation pipeline.
- Stats counters establish a foundation for future telemetry once snapshot/export support lands.

**Weaknesses**
- Chess-specific heuristics (diagonal pawn logic, occupancy-based center control) mis-evaluate shogi positions.
- Attack detection ignores blockers, promoted moves, and hand drops, creating pervasive false positives.
- Integration lacks weighting, telemetry, and phase coordination; positional scores cannot be tuned or observed.
- Performance is quadratic in board size with repeated allocations and no reuse of existing attack tables or caches.
- Tests cover only construction and counter increments, providing zero assurance of correctness.

---

## 8. Improvement Recommendations (Task 15.8)

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| **High** | Replace bespoke attack logic with shared `AttackTables`/`ThreatEvaluator` utilities, including blocker handling and promoted move sets. | Eliminates false positives across center, weak-square, and space detectors; aligns with engine-wide attack semantics. | 6–9 hrs |
| **High** | Incorporate hand (`CapturedPieces`) context into evaluation (drop threats, pawn reinforcements) and adjust pawn heuristics to shogi movement rules. | Restores shogi fidelity for outposts, weak squares, and center control; prevents chess-only assumptions. | 8–12 hrs |
| **High** | Introduce weighting/phase scaling in `IntegratedEvaluator` (reuse `PatternWeights.positional_patterns`) and add stats snapshot/export APIs. | Enables tuning and telemetry, prevents positional noise from dominating evaluations. | 4–6 hrs |
| **Medium** | Redesign center/outpost heuristics to measure *control* (attacked squares, mobility, occupation quality) and cover promoted pieces/lances. | Delivers meaningful positional insight, especially in castle vs. central fights. | 6–8 hrs |
| **Medium** | Optimize scans using bitboards or cached feature maps; avoid repeated 81×81 loops per detector. | Reduces evaluator overhead and keeps positional patterns viable in deep searches. | 5–7 hrs |
| **Medium** | Build regression tests with canonical shogi scenarios (central pawn storms, castle weaknesses, space clamps) and assert expected score deltas. | Ensures future changes maintain accuracy; gives confidence when integrating with tuning pipelines. | 4–6 hrs |
| **Low** | Align configuration defaults with PRD expectations (e.g., disable unstable detectors until tuned, document bonuses). | Clarifies production readiness and reduces surprises during deployment. | 2–3 hrs |

---

## 9. Testing & Validation Plan

1. **Unit Tests**
   - Create board + hand fixtures covering center control (piece vs. drop control), legitimate outposts, and weak squares behind pawn gaps; assert signed score changes.
   - Verify blocker-aware attack computation by contrasting positions with defenders interposed vs. open files.
2. **Integration Tests**
   - Enable positional patterns with different weights and confirm `IntegratedEvaluator` honors scaling and telemetry snapshots.
   - Compare positional outputs against threat/mobility evaluators on shared positions to ensure consistency.
3. **Performance Benchmarks**
   - Measure evaluator runtime before/after attack-table integration on tactical and strategic suites; record allocations and cache hit rates.
4. **Telemetry**
   - Export positional stats snapshots and plumb them into existing evaluation diagnostics so tuning runs can monitor feature activation frequency.

---

## 10. Conclusion

The positional pattern recognizer supplies a configurable skeleton for high-level positional heuristics, but its chess-derived assumptions and lack of shogi-specific context render current outputs unreliable. Correctness must be addressed first—integrating existing attack infrastructure, honoring hand/drop dynamics, and revising heuristics to evaluate *control* rather than mere occupancy. Once fidelity improves, add weighting, telemetry, and targeted tests so positional scores become a trustworthy, tunable signal within the broader tapered evaluation system.

---


