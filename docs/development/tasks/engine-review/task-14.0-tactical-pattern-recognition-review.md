# Task 14.0: Tactical Pattern Recognition Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** December 2024  
**Status:** Complete

---

## Executive Summary

The tactical pattern recognizer delivers a configurable scaffold for forks, pins, skewers, discovered attacks, knight forks, and back-rank threats, but the current implementation is **structurally naïve and tactically unreliable**. Detection relies on repeated 9×9 board scans, home-grown move generators that ignore blockers, and scoring formulas that frequently reward the side being attacked. Integration feeds raw scores straight into the tapered evaluator without weights, phase scaling, or telemetry. Compared against the mature `ThreatEvaluator` inside `attacks.rs`, this module duplicates functionality with far less accuracy or performance discipline. Overall grade: **D (62/100)** — a promising API that requires substantial correctness, integration, and testing work before it can be trusted in production search.

Key findings:

- ✅ Configuration flags correctly gate each detector; statistics counters fire on every evaluator pass.
- ⚠️ Sliding attacks ignore blockers, leading to persistent false positives in fork, pin, and skewer detection.
- ❌ Pin/skewer scoring adds positive bonuses for self-pins (`detect_pins`) and enemy skewers (`detect_skewers`), inverting intended penalties.
- ❌ No integration weights, telemetry exports, or test coverage on real tactical positions; unit tests only assert non-negative scores on empty boards.

---

## Relevant Files

### Primary Implementation
- `src/evaluation/tactical_patterns.rs` – Tactical pattern recognizer, configuration, statistics, and unit tests.

### Supporting / Integration
- `src/evaluation/integration.rs` – Pipes tactical scores into the tapered evaluation pipeline without weighting.
- `src/evaluation/attacks.rs` – Mature `ThreatEvaluator` used by king safety; illustrates richer detection utilities that tactical patterns currently ignore.

### Testing & Instrumentation
- `tests/*` – No dedicated tactical-pattern fixtures; existing tactical suites exercise search instead of static recognizer.
- `docs/design/implementation/evaluation-optimizations/pattern-recognition/PHASE_2_HIGH_PRIORITY_COMPLETION.md` – Completion claims used as acceptance criteria baseline.

---

## 1. Implementation Review (Task 14.1)

- **Recognizer Structure:** `TacticalPatternRecognizer` wraps `TacticalConfig` toggles and `TacticalStats`, exposing `new`, `with_config`, `evaluate_tactics`, and `reset_stats`. Each detector runs unconditionally inside `evaluate_tactics` when its flag is enabled, ensuring modular gating but forcing repeated full-board scans per call.
- **Move Generation:** Pattern detection relies on bespoke helpers (`get_piece_attacks`, `add_sliding_attacks`, `add_single_attack`) instead of the engine’s move generator or attack tables, resulting in duplicated logic and mismatched rules (no blocker handling, no drops).
- **Statistics:** `TacticalStats` mixes plain counters with `AtomicU64` fields but lacks a snapshot/clone API, making it hard to export aggregated results alongside other telemetry modules.
- **Integration:** Scores are added directly to the tapered total with no component weight or phase scaling (`evaluate_tactics` returns a `TaperedScore` that is summed as-is).

```259:265:src/evaluation/integration.rs
        if self.config.components.tactical_patterns {
            total += self
                .tactical_patterns
                .borrow_mut()
                .evaluate_tactics(board, player);
        }
```

---

## 2. Fork Detection Accuracy (Task 14.2)

`detect_forks` scans every friendly piece and calls `check_piece_for_forks`, which uses `get_piece_attacks` to enumerate targets. The attack generator ignores occupancy, so sliding pieces “see” through friendly blockers and count targets they cannot attack, inflating fork counts and king-fork bonuses.

```188:284:src/evaluation/tactical_patterns.rs
    fn get_piece_attacks(
        &self,
        pos: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Position> {
        let mut attacks = Vec::new();
        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                for dir in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.add_sliding_attacks(&mut attacks, pos, dir);
                }
            }
            // ...
        }
        attacks
    }

    fn add_sliding_attacks(&self, attacks: &mut Vec<Position>, pos: Position, dir: (i8, i8)) {
        let mut row = pos.row as i8 + dir.0;
        let mut col = pos.col as i8 + dir.1;
        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            attacks.push(Position::new(row as u8, col as u8));
            row += dir.0;
            col += dir.1;
        }
    }
```

Consequences:

- Fork detection registers targets hidden behind friendly shields, so `fork_bonus_factor` applies in positions where no fork exists.
- Knight forks miss drop-based threats and rely on the same target enumeration, ignoring capture legality or king exposure.
- Drop-based forks (central to shogi tactics) are invisible because the recognizer never examines `CapturedPieces`.

---

## 3. Pin Detection Logic (Task 14.3)

Pin detection finds lines from the player’s king to enemy sliders but **adds** positive scores when a friendly piece is pinned, effectively rewarding us for being pinned.

```302:378:src/evaluation/tactical_patterns.rs
    fn detect_pins(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.pin_checks += 1;
        let mut mg_score = 0;
        // ...
        mg_score += self.check_pins_in_directions(/* ... */);
        let eg_score = mg_score / 2;
        TaperedScore::new_tapered(mg_score, eg_score)
    }
    fn check_pins_in_directions(/* ... */) -> i32 {
        // ...
        if self.can_pin_along_line(piece.piece_type, dr, dc) {
            if pieces_in_line.len() == 2 && pieces_in_line[0].1.player == player {
                let pinned_value = pieces_in_line[0].1.piece_type.base_value() / 100;
                pin_value += pinned_value * self.config.pin_penalty_factor / 100;
                self.stats.pins_found.fetch_add(1, Ordering::Relaxed);
            }
        }
        // ...
        pin_value
    }
```

Issues:

- `mg_score` stays positive, so the caller gains centipawns when a friendly piece is immobilized; the sign must be flipped.
- No occupancy pruning: once a friendly piece is logged, the loop keeps scanning, so multiple friends before any attacker still register as a “pin,” generating phantom penalties/bonuses.
- No detection for promoted sliders (Tokin acting as gold/rook hybrids) or hand drops creating pins.

---

## 4. Skewer Detection (Task 14.4)

Skewer detection mirrors the pin logic but iterates over **enemy** sliders and accumulates a positive bonus, rewarding situations where our pieces are skewered.

```399:489:src/evaluation/tactical_patterns.rs
    fn detect_skewers(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.skewer_checks += 1;
        let mut mg_score = 0;
        // ...
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        mg_score += self.check_skewers_from_piece(/* ... */);
                    }
                }
            }
        }
        let eg_score = mg_score / 2;
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

Additional gaps:

- `check_skewers_from_piece` never stops when it encounters our blocking piece, so it can count multiple “targets” even when the line is sealed.
- Value calculations use raw base-value differences divided by 10 000, producing tiny, noisy adjustments that can easily wash out or even change sign if the inversion bug is fixed.
- No awareness of long-range promoted rook/bishop bonus moves or lance-based skewers specific to shogi.

---

## 5. Discovered Attack & Back-Rank Threats (Task 14.5)

- `can_create_discovered_attack` only checks for a friendly slider behind the piece and ignores the path toward the target, so any alignment with the opponent king flags as a discovered threat even when intervening pieces block the line.
- Back-rank detection assumes the king is trapped if it sits on the back rank with zero adjacent empty/enemy squares; it does not verify actual attack coverage, rook obstructions, or defensive escorts. `count_back_rank_threats` merely counts rooks on the rank regardless of blocking pieces.

```623:721:src/evaluation/tactical_patterns.rs
    fn detect_back_rank_threats(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        // ...
        let threats = self.count_back_rank_threats(board, king_pos, player);
        if threats > 0 {
            let penalty = threats * self.config.back_rank_threat_penalty;
            return TaperedScore::new_tapered(-penalty, -penalty / 2);
        }
        TaperedScore::default()
    }
```

The recognizer also overlooks promoted-piece reach, hand drops that open lines, and tactical motifs that involve exchange sacrifices — all common in shogi endgames.

---

## 6. Tactical Evaluation Quality (Task 14.6)

- **Scoring & Weighting:** Tactical contributions are summed directly into `TaperedScore` without weights or phase-based scaling; a single fork bonus can outweigh material in quiet positions, and there is no way to tune aggressiveness vs. noise.
- **Telemetry:** `TacticalStats` is not exported through `EvaluationTelemetry`, so search diagnostics cannot track tactical workload or hit rates.
- **Testing:** Module tests instantiate an empty board and only assert non-negative scores, providing zero verification of actual tactical detection accuracy.

```824:896:src/evaluation/tactical_patterns.rs
    #[test]
    fn test_fork_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let score = recognizer.detect_forks(&board, Player::Black);
        assert!(score.mg >= 0);
    }
```

- **Performance:** Each detector scans the entire 9×9 board (often twice), allocating fresh `Vec`s for every piece. There is no reuse of `AttackTables`, bitboards, or the existing `ThreatEvaluator`, so runtime cost grows quickly while accuracy remains low.

---

## 7. Strengths & Weaknesses (Task 14.7)

**Strengths**
- Configurable feature toggles allow incremental rollout or targeted benchmarking.
- Statistics counters capture evaluator usage and detection hits (albeit not yet exported).
- API symmetry with other evaluation components (returns `TaperedScore`, integrates via `IntegratedEvaluator`).

**Weaknesses**
- Sliding attack generation ignores blockers, producing chronic false positives across fork/pin/skewer detectors.
- Sign errors in pin/skewer scoring reward the defender instead of penalizing tactical vulnerabilities.
- No support for hand-piece drops, promoted sliders, or niche shogi motifs (e.g., lance skewers, drop forks).
- Duplicate logic with `ThreatEvaluator` in `attacks.rs`; tactical patterns reinvent core routines with less rigor.
- Unit tests lack real tactical fixtures; integration lacks weights, telemetry, and performance instrumentation.

---

## 8. Improvement Recommendations (Task 14.8)

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| **High** | Reuse `AttackTables`/`ThreatEvaluator` logic (or factor shared helpers) for accurate line-of-sight and blocker-aware attack generation. | Eliminates false positives, aligns tactical detection with king-safety stack, and avoids duplicated move logic. | 6–8 hrs |
| **High** | Fix sign handling for pins/skewers and normalize scoring against centipawn scale (negative penalties for vulnerabilities, positive bonuses for threats we create). | Stops rewarding self-pins, produces interpretable scores, and keeps tactics proportional to other evaluators. | 2–3 hrs |
| **High** | Accept `CapturedPieces`/hand context in `evaluate_tactics` and extend detectors to cover drop-based forks/pins. | Shogi tactics rely heavily on drops; ignoring hands underestimates real tactical pressure. | 5–7 hrs |
| **Medium** | Introduce evaluation weights/phase scaling and export `TacticalStats` through telemetry. | Enables tuning, balances tactical impact against other components, and surfaces runtime costs. | 4–6 hrs |
| **Medium** | Build regression tests with curated tactical FENs (fork, pin, skewer, discovered attack, back-rank trap) and assert detection outcomes. | Provides coverage for real motifs, prevents future regressions, and validates sign/scoring fixes. | 4–6 hrs |
| **Low** | Optimize iteration (precompute friendly/enemy piece lists, reuse buffers) or defer to bitboard scans to cut redundant 9×9 loops. | Reduces overhead once accuracy issues are resolved; keeps evaluator competitive in deep searches. | 3–5 hrs |

---

## 9. Testing & Validation Plan

1. **Unit Tests**
   - Create board fixtures (with and without blockers) to verify fork/pin/skewer detection accuracy, including hand-drop scenarios.
   - Assert sign-correct scoring (penalties negative for self vulnerabilities, bonuses positive for opponent vulnerabilities).
2. **Integration Tests**
   - Wire tactical evaluations into `IntegratedEvaluator` with representative configurations; verify scores track expected tactical swings when weights vary.
   - Compare outputs against the existing `ThreatEvaluator` to ensure consistency for overlapping motifs.
3. **Performance Benchmarks**
   - Measure evaluator runtime on tactical puzzle suites before/after reuse of `AttackTables`; record allocations and CPU time.
   - Track telemetry counters (once exported) across depth-limited searches to understand frequency of each motif.

---

## 10. Conclusion

The tactical pattern recognizer establishes a configurable entry point for high-value tactical motifs, but the current implementation falls short on correctness, shogi fidelity, integration, and validation. Correcting attack-line logic, fixing scoring polarity, incorporating hand pieces, and leveraging the mature `ThreatEvaluator` infrastructure are prerequisites for trustworthy tactical scoring. Once accuracy gaps close, invest in tunable weights, telemetry, and regression suites so tactical evaluation can contribute signal without destabilizing search. Completion of these items will align the module with the requirements outlined in the PRD and prepare it for inclusion in broader evaluation tuning efforts (Meta-Task 20.0).

---


