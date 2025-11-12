# Task 16.0: Castle Pattern Recognition Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** December 2024  
**Status:** Complete

---

## Executive Summary

The castle recognizer aspires to score Anaguma, Mino, and Yagura structures and feed their protection value into king-safety evaluation, yet the current implementation is **template-based, brittle, and only loosely connected to real shogi castles**. Pattern definitions hard-code a single orientation, ignore promoted pieces and hand reinforcements, and require near-perfect piece placement to pass the aggressive 0.8 quality gate. Castle recognition therefore fires only in toy setups, never penalizes unsafe kings, and offers little protection insight to the broader evaluation stack. Overall grade: **D (62/100)** — promising scaffolding, but it must be rebuilt around flexible pattern geometry, defensive coverage metrics, and telemetry before it can guide search.

Key findings:

- ❌ Pattern definitions encode one-side-only offsets and reject promoted defenders or alternative castle shells, so standard game positions rarely qualify.
- ⚠️ Match scoring multiplies a static `TaperedScore` by match quality ≥0.7, leaving incomplete castles and bare kings indistinguishable (0 cp) for the evaluator.
- ⚠️ Early termination plus a 50-entry hash map tied to a coarse board hash evicts useful results and recomputes frequently, nullifying the advertised cache advantage.
- ❌ King-safety integration never subtracts points for exposed kings and contains a duplicate placeholder API that still returns `0`, making it straightforward to call the wrong entry point.
- ⚠️ Tests exist only behind `legacy-tests` and assert “score ≥ 0” rather than verifying recognition accuracy, coverage, or failure cases.

---

## Relevant Files

### Primary Implementation
- `src/evaluation/castles.rs` – Castle pattern definitions, matching, caching, and scoring.
- `src/evaluation/patterns/anaguma.rs` – Anaguma template (static offsets and weights).
- `src/evaluation/patterns/mino.rs` – Mino template.
- `src/evaluation/patterns/yagura.rs` – Yagura template.

### Integration
- `src/evaluation/king_safety.rs` – Consumes castle scores inside `KingSafetyEvaluator`.
- `src/evaluation/attacks/` – Attack/threat analyzers combined with castle output for overall king safety.

### Testing & Instrumentation
- `tests/king_safety_tests.rs` (feature `legacy-tests`) – Basic castle recognition smoke tests.
- `tests/king_safety_integration_tests.rs` – Performance budget checks (also legacy-gated).

---

## 1. Pattern Definition Review (Task 16.1)

- Castle templates hard-code a single relative coordinate set around the king and cannot mirror to the opposite wing. A right-side Mino or White’s mirrored layout fails because `get_relative_position` flips only rows, not columns.
- Required piece lists omit promoted defenders (e.g., promoted silver in Anaguma) and alternative guards (gold vs. silver swap), so legitimate castles collapse into “partial matches.”
- Optional pawns lack orientation logic; drops or advanced pawns that still shelter the king are treated as missing pieces.

```5:48:src/evaluation/patterns/anaguma.rs
pub fn get_anaguma_castle() -> CastlePattern {
    CastlePattern {
        name: "Anaguma",
        pieces: vec![
            CastlePiece {
                piece_type: PieceType::Gold,
                relative_pos: (-1, 0),
                required: true,
                weight: 10,
            },
            CastlePiece {
                piece_type: PieceType::Silver,
                relative_pos: (-2, 0),
                required: true,
                weight: 9,
            },
            // ...
        ],
        score: TaperedScore::new_tapered(220, 40),
        flexibility: 3,
    }
}
```

- Weights are arbitrary integers (5–10) with no documented calibration and identical across castles regardless of defensive role.
- `flexibility` is set to 2–3 pieces but is applied after required-piece filtering, so omitting a single optional defender often drops quality below 0.7, zeroing the score.

---

## 2. Anaguma Recognition Accuracy (Task 16.2)

- Recognition checks exact piece types; promoted gold/silver, dragons guarding the back rank, or defensive drops are invisible.
- `relative_pos` expects the Silver two ranks forward of the king; practical Anaguma placements (e.g., Silver on `7h`) mismatch due to coordinate assumptions, and mirrored left-Anaguma lines are unsupported.
- The match-quality gate (`>= 0.8`) combined with required Gold/Silver means any deviation—such as a defending knight or pawn drop—fails recognition entirely.
- Tests only assert the returned score is non-negative, so false negatives and false positives remain undetected.

```81:120:src/evaluation/castles.rs
if match_quality >= 0.7 {
    let adjusted_score = self.adjust_score_for_quality(pattern.score, match_quality);
    if match_quality > best_match_quality {
        best_score = adjusted_score;
        best_match_quality = match_quality;
        best_pattern_index = Some(pattern_index);
    }
}
```

- No logging or telemetry reveals why recognition failed, hindering tuning.

---

## 3. Mino Recognition Accuracy (Task 16.3)

- Mino template assumes the king resides on the same file for both colors and that all three pawns sit on a single file. Actual Mino castles stagger pawns and occasionally trade the Silver/Gold columns, so the recognizer under-scores canonical textbook positions.
- Required Gold/Silver placements ignore common transitions (e.g., Silver promoted to dragon) and treat any captured pawn drop or advanced pawn as missing, collapsing match quality.
- Tests rely on `create_mino_castle_board`, a contrived position with exactly the expected offsets; there is no coverage for live games, mirrored layouts, or partial structures.

---

## 4. Yagura Recognition Accuracy (Task 16.4)

- Yagura template includes a Lance three files away from the king, yet ignores the knight on `7g` and the defining pawn chain at `7f/7g`. The template therefore fires even when the king stands in a pseudo-Yagura without the vital pawn wall, and fails when the lance has advanced but the castle shape remains intact.
- Required Gold/Silver offsets mirror Mino’s assumptions, so White-side Yagura or flexible transitions are unsupported.
- There is no scoring bonus for closed pawn files or rear silver walls, so the recognizer conflates Yagura with any arrangement that happens to satisfy the required offsets.

---

## 5. Castle Quality Evaluation (Task 16.5)

- `evaluate_castle_structure` returns the castle’s base `TaperedScore` scaled by match quality. Castles that miss by one optional pawn drop to zero rather than degrading smoothly, and there is no penalty for bare kings or broken structures.

```56:122:src/evaluation/castles.rs
let (matches, required_matches, total_weight) =
    self.analyze_pattern_match(board, player, king_pos, pattern);
// ...
let match_quality =
    self.calculate_match_quality(matches, pattern.pieces.len(), total_weight, pattern);
// Early termination at 0.8 quality and hard 0.7 lower bound
```

- `calculate_match_quality` weights piece count (60%) and matched weights (40%) but does not account for spatial coverage, open files, or attack lanes.
- Cache inserts only when size < 50 using a naive board hash that ignores hands and promoted state, leading to collisions and frequent evictions.
- Duplicate placeholder APIs (`KingSafetyEvaluator::evaluate_castle_structure`) still return `TaperedScore::default()`, so downstream consumers can easily call the wrong function and lose castle scoring entirely.

```267:304:src/evaluation/king_safety.rs
pub fn evaluate_castle_structure(
    &self,
    _board: &BitboardBoard,
    _player: Player,
) -> TaperedScore {
    // TODO: Implement castle pattern recognition
    TaperedScore::default()
}
```

---

## 6. Defense Assessment Accuracy (Task 16.6)

- King-safety evaluation multiplies castle scores by `castle_weight` (0.3 by default) but never subtracts points when no castle is recognized; bare kings and full fortresses can evaluate identically.
- Fast-mode shortcuts inspect only a fixed 3×3 ring of friendly pieces and award static bonuses, ignoring whether those pieces actually shield the king or control entry squares.

```95:154:src/evaluation/king_safety.rs
if let Some(king_pos) = self.find_king_position(board, player) {
    let castle_score = self
        .castle_recognizer
        .evaluate_castle_structure(board, player, king_pos);
    total_score += castle_score * self.config.castle_weight;
}
// ...
score += self.evaluate_basic_attacks(board, player);
```

- There is no linkage between castle quality and attack/threat analyzers; the system cannot tell whether the recognized castle actually blocks threats, nor does it downgrade castles that have already been breached.
- Performance harnesses measure 1 000 evaluations on empty boards; no profiling exists for real positions, so matching hot spots remain unquantified.

---

## 7. Strengths & Weaknesses (Task 16.7)

**Strengths**
- Modular `CastlePattern` representation enables future sharing across castles with different flexibility requirements.
- Integration pathway into `KingSafetyEvaluator` and weighting hooks already exist.
- Cache scaffold and match-quality scaling lay groundwork for incremental improvements once heuristics are corrected.

**Weaknesses**
- Hard-coded templates misrepresent real castles, lack mirroring, and reject promoted/hand defenders.
- Recognition returns zero for partial matches, providing no gradient to tuning systems.
- Hash-based cache ignores hands, promoted state, and king neighborhood and is too small to be effective.
- Tests are gated behind `legacy-tests` and verify only trivial configurations.
- Duplicate placeholder APIs and missing telemetry make failures silent and hard to diagnose.

---

## 8. Improvement Recommendations (Task 16.8)

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| **High** | Redefine castle templates using parametric “anchor offsets” with left/right symmetry, promoted defender allowances, and multiple valid shells per castle. | Restores recognition coverage for real games; prevents false negatives from mirrored or promoted formations. | 8–12 hrs |
| **High** | Replace binary matching with zone-based scoring (defender coverage, pawn chains, drop buffers) and introduce penalties for exposed kings. | Provides smooth gradients for tuning, differentiates partial castles from bare kings. | 10–14 hrs |
| **High** | Remove duplicate placeholder APIs; expose telemetry (quality, missing pieces, cache hit rate) via `KingSafetyStats`. | Prevents silent failures, enables debugging and tuning. | 3–4 hrs |
| **Medium** | Expand cache to store per-king-radius hashes (including hands/promotions) and adopt LRU eviction sized for mid-search workloads. | Reduces recomputation and aligns with match granularity. | 5–7 hrs |
| **Medium** | Add regression fixtures for canonical castles (right/left Anaguma, Snowroof, Yagura variants) plus break scenarios; enforce via default test suite. | Ensures accurate recognition and guards against regressions. | 6–8 hrs |
| **Medium** | Integrate castle output with attack/threat analyzers (e.g., reduce castle score when open files or mating nets detected) and surface combined stats. | Links defensive quality to actual attacks, improving evaluation fidelity. | 7–9 hrs |
| **Low** | Replace magic numbers (weights, thresholds) with configuration in `KingSafetyConfig`, documented defaults, and tuning hooks. | Simplifies experimentation and aligns with PRD configuration goals. | 3–4 hrs |
| **Low** | Profile castle recognition over realistic game traces, recording cache efficacy, match distribution, and time spent per pattern. | Validates performance assumptions before expanding pattern library. | 3 hrs |

---

## 9. Testing & Validation Plan

1. **Unit Tests**
   - Add symmetry and promotion-aware fixtures for each castle; assert recognition quality across mirrored boards and with promoted defenders.
   - Verify match-quality scaling produces graded scores for partial castles and penalizes bare kings.
   - Test cache evictions and hash stability with hands/promoted pieces in play.

2. **Integration Tests**
   - Run `KingSafetyEvaluator` on representative positions (castle intact, partially broken, under heavy attack) and confirm castle weights interact correctly with attack/threat scores.
   - Ensure `evaluate_castle_structure` returns distinct values for left vs. right castles and that telemetry reports missing components.

3. **Performance Benchmarks**
   - Benchmark castle recognition across full-game traces (opening → late game) to track average time, cache hit rate, and pattern distribution.
   - Introduce stress tests with randomized king zones to validate cache sizing and avoid hash collisions.

4. **Telemetry**
   - Emit structured logs (e.g., `trace_log!("CASTLE", { pattern, quality, missing_pieces })`) when castle quality < 0.6 or cache misses spike.
   - Aggregate castle recognition stats alongside king-safety metrics for tuning sessions.

---

## 10. Conclusion

The castle recognition subsystem currently offers only nominal coverage; its rigid templates fail under real shogi conditions, depriving the engine of vital king-safety signals. Refactoring toward symmetry-aware templates, coverage-based scoring, and integrated telemetry will transform the module into a practical defensive evaluator. Until then, search treats fortresses and exposed kings almost identically, limiting both playing strength and tuning leverage.

**Next Steps:** File engineering tickets covering the high-priority recommendations, align changes with the king-safety roadmap, and ensure new regression fixtures become part of the default CI suite rather than legacy-only tests.

---


