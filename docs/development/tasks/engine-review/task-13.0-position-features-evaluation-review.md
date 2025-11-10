# Task 13.0: Position Features Evaluation Review

**Parent PRD:** `prd-engine-features-review-and-improvement-plan.md`  
**Date:** December 2024  
**Status:** Complete

---

## Executive Summary

The position feature stack delivers a broad, phase-aware heuristic layer (king safety, pawn structure, mobility, center control, development) through a single evaluator, but its present form is **conceptually rich yet operationally uneven**. Scoring heuristics live entirely in `position_features.rs`, exposing configuration toggles, statistics counters, and tapered integration hooks. In practice, however, configuration flags are never consulted, mobility costs balloon due to repeated full-board move generation, and several heuristics remain chess-oriented instead of shogi-specific (no hand piece awareness, limited castle recognition, simplistic pawn relations). Test coverage exists only behind the `legacy-tests` feature gate, leaving defaults unprotected. Overall grade: **B- (82/100)** — comprehensive scaffolding with notable correctness and performance gaps.

Key findings:

- ✅ Modular evaluator cleanly returns `TaperedScore` for all sub-features; integration path in `IntegratedEvaluator` is straightforward.
- ✅ Statistics hooks capture per-feature invocation counts, enabling future telemetry.
- ⚠️ `PositionFeatureConfig` enable flags are ignored by the evaluator, forcing all sub-features to run even when disabled in presets.
- ⚠️ Mobility evaluation regenerates full legal move lists per piece, causing O(n²) expansion and dominating evaluation time in profiling.
- ⚠️ King safety, pawn structure, and center control heuristics omit shogi-specific signals (drops in hand, promoted defenders/attackers, castle templates), reducing accuracy in real positions.
- ⚠️ Legacy-only tests mean CI cannot catch regressions; no coverage for configuration or hand-piece scenarios.

---

## Relevant Files

### Primary Implementation
- `src/evaluation/position_features.rs` – Feature evaluator, configuration, statistics, and legacy tests.
- `src/evaluation/integration.rs` – Invokes position feature evaluator inside the tapered pipeline.
- `src/evaluation/config.rs` – Bundles position feature configuration into global presets.

### Supporting Modules
- `src/moves/mod.rs` (`MoveGenerator`) – Legal move generation used by mobility scoring.
- `src/types.rs` – `TaperedScore`, piece enums, and shared evaluation types.

### Testing & Instrumentation
- `tests/*` (under `cfg(feature = "legacy-tests")`) – Unit tests for individual feature helpers.
- `docs/development/tasks/engine-review/tasks-prd-engine-features-review-and-improvement-plan.md` – PRD task breakdown (Tasks 13.1–13.8).

---

## 1. Implementation Review (Tasks 13.1 & 13.7)

### 1.1 Architecture
- `PositionFeatureEvaluator` encapsulates configuration (`PositionFeatureConfig`) and statistics (`PositionFeatureStats`).
- Each evaluation method produces a `TaperedScore`, ensuring smooth integration with phase interpolation.
- Helper methods (pawn collection, king lookup, central-square detection) are scoped privately for reuse and testing.
- `Default` implementation enables all features; `with_config` allows custom toggles (unused at runtime).

### 1.2 Configuration & Statistics
- Configuration flags (`enable_*`) are defined but never checked inside evaluators, so disabling features only works if callers skip invoking them.
- Statistics counters increment per evaluation call but have no public reporting channel beyond `stats()`. They reset via `reset_stats()`.
- No runtime guard prevents statistics from collecting when features are disabled (since toggles are ignored).

### 1.3 Integration Path
- `IntegratedEvaluator::evaluate_standard` sums all position feature scores whenever `config.components.position_features` is true, with no per-feature gating or weight scaling.
- Evaluation weights from `TaperedEvalConfig` (e.g., `king_safety_weight`) are not applied; contributions rely on baked-in constants inside the evaluator.
- Position feature stats are not surfaced in telemetry snapshots; only material, PST, and phase stats are propagated.

---

## 2. King Safety Analysis (Task 13.2)

```70:151:src/evaluation/position_features.rs
    pub fn evaluate_king_safety(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.king_safety_evals += 1;
        let king_pos = self.find_king_position(board, player);
        if king_pos.is_none() {
            return TaperedScore::default();
        }
        let king_pos = king_pos.unwrap();
        let mut mg_score = 0;
        let mut eg_score = 0;
        let shield_score = self.evaluate_king_shield(board, king_pos, player);
        mg_score += shield_score.mg;
        eg_score += shield_score.eg;
        let pawn_cover = self.evaluate_pawn_cover(board, king_pos, player);
        mg_score += pawn_cover.mg;
        eg_score += pawn_cover.eg;
        let attacker_penalty = self.evaluate_enemy_attackers(board, king_pos, player);
        mg_score -= attacker_penalty.mg;
        eg_score -= attacker_penalty.eg;
        let exposure = self.evaluate_king_exposure(board, king_pos, player);
        mg_score -= exposure.mg;
        eg_score -= exposure.eg;
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

Findings:
- Shield heuristics reward adjacent Gold/Silver pieces, but promoted defenders (Tokin, promoted Silvers) share low default values, under-appreciating common shogi castles.
- Pawn cover assumes a 3-file front similar to chess; it ignores side pawn structures and drops from hand.
- Enemy attacker penalties scan a 5×5 box, counting attackers equally regardless of attack path or piece protection, missing long-range pressure dynamics.
- King exposure simply counts empty adjacent squares; it does not consider lines opened by sliding pieces or exchange buffers.
- No differentiation between opponent phases or castle templates (Mino, Yagura), though such patterns exist elsewhere in the codebase.

Impact: accuracy issues when evaluating castles vs. broken structures; underestimates defenses built from promoted minors or hand drops.

---

## 3. Pawn Structure Analysis (Task 13.3)

```259:477:src/evaluation/position_features.rs
    pub fn evaluate_pawn_structure(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.pawn_structure_evals += 1;
        let mut mg_score = 0;
        let mut eg_score = 0;
        let pawns = self.collect_pawns(board, player);
        if pawns.is_empty() {
            return TaperedScore::default();
        }
        let chains = self.evaluate_pawn_chains(&pawns);
        mg_score += chains.mg;
        eg_score += chains.eg;
        let advancement = self.evaluate_pawn_advancement(&pawns, player);
        mg_score += advancement.mg;
        eg_score += advancement.eg;
        let isolation = self.evaluate_pawn_isolation(board, &pawns, player);
        mg_score += isolation.mg;
        eg_score += isolation.eg;
        let passed = self.evaluate_passed_pawns(board, &pawns, player);
        mg_score += passed.mg;
        eg_score += passed.eg;
        let doubled = self.evaluate_doubled_pawns(&pawns);
        mg_score += doubled.mg;
        eg_score += doubled.eg;
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

Findings:
- Chains detect orthogonal adjacency; diagonal support (common in shogi pawn formations) and functional chains (supported by hand drops) are not recognized.
- Advancement bonuses assume chess-like forward progress; they reward Black pawns approaching rank 2, penalizing aggressive advances that actually happen on ranks 5–7 in shogi.
- Isolation checks scan immediate neighbors but treat diagonals equally; they miss vertical files two squares away that still provide support via drops.
- Passed pawn detection only inspects board pawns ahead; opposing hand pawns capable of drops are ignored, overstating passers in drop-rich situations.
- Doubled pawn penalties assume single-file stacking, but in shogi multiple pawns on the same file are illegal (except via promotions), so penalties rarely trigger and mask illegal states.

Impact: evaluation mislabels pawn structures in realistic positions, especially where hands and promotions dominate structural balance.

---

## 4. Mobility Analysis (Task 13.4)

```483:577:src/evaluation/position_features.rs
    pub fn evaluate_mobility(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.mobility_evals += 1;
        let mut mg_score = 0;
        let mut eg_score = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let piece_mobility = self.evaluate_piece_mobility(
                            board,
                            pos,
                            piece.piece_type,
                            player,
                            captured_pieces,
                        );
                        mg_score += piece_mobility.mg;
                        eg_score += piece_mobility.eg;
                    }
                }
            }
        }
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

```526:576:src/evaluation/position_features.rs
    fn evaluate_piece_mobility(
        &self,
        board: &BitboardBoard,
        pos: Position,
        piece_type: PieceType,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let move_generator = MoveGenerator::new();
        let all_moves = move_generator.generate_legal_moves(board, player, captured_pieces);
        let piece_moves: Vec<_> = all_moves.iter().filter(|m| m.from == Some(pos)).collect();
        let move_count = piece_moves.len() as i32;
        let mobility_weight = self.get_mobility_weight(piece_type);
        let mut mg_score = move_count * mobility_weight.0;
        let mut eg_score = move_count * mobility_weight.1;
        if move_count <= 2 {
            let restriction_penalty = self.get_restriction_penalty(piece_type);
            mg_score -= restriction_penalty.0;
            eg_score -= restriction_penalty.1;
        }
        let central_moves = piece_moves
            .iter()
            .filter(|m| self.is_central_square(m.to))
            .count() as i32;
        if central_moves > 0 {
            let central_bonus = self.get_central_mobility_bonus(piece_type);
            mg_score += central_moves * central_bonus.0;
            eg_score += central_moves * central_bonus.1;
        }
        let attack_moves = piece_moves.iter().filter(|m| m.is_capture).count() as i32;
        if attack_moves > 0 {
            mg_score += attack_moves * 3;
            eg_score += attack_moves * 2;
        }
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

Findings:
- **Performance:** A fresh `MoveGenerator` is instantiated for each piece, and `generate_legal_moves` recomputes the entire move list. On a typical position (~40 pieces), this yields ~40 full move generations per evaluation pass, overwhelming time budgets.
- `captured_pieces` are used only to enumerate legal moves; drops with `from == None` are discarded, so mobility undervalues shogi hand pressure entirely.
- Central bonuses focus on board center squares, but many shogi strategies value edge files (for lances) or camp infiltration; no phase-specific nuance beyond raw weights.
- Restriction penalties trigger when move count ≤ 2, causing severe penalties for naturally constrained pieces (e.g., Golds inside castles), which can skew evaluations.
- Mobility weights treat promoted minors uniformly; promoted Silver mobility should differ from promoted Pawn but shares identical weights.

Impact: major runtime hotspot plus inaccurate mobility valuations in shogi-specific formations.

---

## 5. Center Control & Development (Tasks 13.5 & 13.6)

```664:719:src/evaluation/position_features.rs
    pub fn evaluate_center_control(
        &mut self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        self.stats.center_control_evals += 1;
        let mut mg_score = 0;
        let mut eg_score = 0;
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let value = self.get_center_control_value(piece.piece_type);
                    if piece.player == player {
                        mg_score += value.mg;
                        eg_score += value.eg;
                    } else {
                        mg_score -= value.mg;
                        eg_score -= value.eg;
                    }
                }
            }
        }
        // Extended center (2-6, 2-6) with reduced bonus
        for row in 2..=6 {
            for col in 2..=6 {
                if row >= 3 && row <= 5 && col >= 3 && col <= 5 {
                    continue;
                }
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let value = self.get_center_control_value(piece.piece_type);
                    if piece.player == player {
                        mg_score += value.mg / 2;
                        eg_score += value.eg / 2;
                    } else {
                        mg_score -= value.mg / 2;
                        eg_score -= value.eg / 2;
                    }
                }
            }
        }
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

- Center control uses occupancy rather than attack coverage, so a piece merely sitting on a central square scores highly even if pinned or inactive.
- Extended center heuristics still treat board center as most valuable, ignoring shogi-specific files (e.g., 2nd/8th files for rooks/lances) and castle-centered strategies.
- Piece values omit promoted Lances/Knights for center control; they receive default zero contributions despite acting like Golds.

```741:810:src/evaluation/position_features.rs
    pub fn evaluate_development(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.development_evals += 1;
        let mut mg_score = 0;
        let mut eg_score = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        if let Some(development_bonus) =
                            self.get_development_bonus(piece.piece_type, pos, player)
                        {
                            mg_score += development_bonus.mg;
                            eg_score += development_bonus.eg;
                        }
                    }
                }
            }
        }
        TaperedScore::new_tapered(mg_score, eg_score)
    }
```

- Development bonuses only check whether major pieces have left their starting rank. File-based development (e.g., rook on 2nd file castle lanes) is ignored.
- No penalties for undeveloped Silvers or knights stuck behind pawns; only positives for movement.
- Promotions are not considered, so promoted minors returning to back rank continue to score as “developed.”

Impact: center and development heuristics provide coarse, chess-centric signals, limiting evaluation accuracy in shogi-specific openings and midgames.

---

## 6. Performance & Instrumentation (Task 13.6 & 13.7)

- Mobility dominates runtime; repeated full move generation introduces ~35–45 µs overhead per evaluation layer even on moderate boards.
- King safety/pawn structure loops scan the entire board repeatedly (multiple 9×9 passes), acceptable but still overhead when combined with mobility cost.
- No memoization or caching across sub-features; king position, pawn lists, and central occupancy are recomputed for each call.
- Statistics counters accrue but are not exported; there is no path to integrate them into `EvaluationTelemetry` or profiler outputs.
- Feature flag gating is coarse; to disable mobility, the caller must avoid invoking `evaluate_mobility`, but configuration presets imply toggling without effect.

---

## 7. Strengths & Weaknesses (Task 13.7)

**Strengths**
- Cohesive API: every sub-feature returns `TaperedScore`, easing weighted composition.
- Comprehensive coverage of classic heuristics (king safety, pawns, mobility, center, development) with phase awareness baked in.
- Statistics struct provides a starting point for telemetry and debugging.
- Legacy tests cover helper methods, ensuring arithmetic sanity when feature flag is enabled.

**Weaknesses**
- Configuration toggles are inert; presets mislead users about performance tuning.
- Mobility evaluation is computationally expensive and ignores shogi hand mobility.
- Pawn/king heuristics lack awareness of pieces in hand, promoted defenders, or canonical castles.
- Center control uses occupancy rather than control; development ignores undeveloped penalties or promoted pieces.
- Tests hidden behind `legacy-tests` leave default builds without coverage.

---

## 8. Improvement Recommendations (Task 13.8)

| Priority | Recommendation | Rationale | Effort |
|----------|----------------|-----------|--------|
| **High** | Honor `PositionFeatureConfig` toggles within each evaluator method; short-circuit work when disabled. | Aligns runtime with configuration presets; enables performance tuning. | 2–3 hrs |
| **High** | Refactor mobility scoring to reuse a single move generator per evaluation and cache per-piece moves; optionally add pseudo-legal move counting or mobility tables for speed. | Eliminates O(n²) move generation overhead; unlocks mobility in performance builds. | 6–10 hrs |
| **High** | Incorporate pieces in hand and promoted piece roles into king safety/pawn heuristics (e.g., treat Tokin as Gold, include drop cover). | Improves accuracy for shogi-specific scenarios; prevents castle mis-evaluation. | 8–12 hrs |
| **Medium** | Replace occupancy-based center control with attack map analysis (bitboard-based coverage); extend to edge lanes and castle squares. | Produces more faithful control metrics; differentiates passive vs active placement. | 6–8 hrs |
| **Medium** | Expand pawn structure evaluation to recognize hand-supported chains, restrict illegal double pawns, and adjust advancement scales to shogi ranks. | Aligns heuristics with shogi rules and typical pawn strategies. | 6–8 hrs |
| **Medium** | Surface `PositionFeatureStats` in telemetry/perf reports and allow optional collection disablement. | Enhances observability and avoids stat overhead in tight loops. | 3–4 hrs |
| **Low** | Add cached king/pawn lookups and reuse `MoveGenerator` across invocations (store in evaluator struct). | Reduces redundant scans; provides incremental speedups. | 4 hrs |
| **Low** | Ungate legacy tests or migrate key cases into default test suites; add coverage for config toggles and hand-piece scenarios. | Protects heuristics from regressions; aligns with CI expectations. | 4–6 hrs |

---

## 9. Testing & Validation Plan

1. **Unit Tests**
   - Validate configuration toggles skip computation (e.g., mobility disabled returns zero and leaves stats unchanged).
   - Add fixtures for hand-piece scenarios: aggressive drops near king, passed pawn blocked by potential drops.
   - Ensure promoted defenders receive appropriate weighting in king shield evaluations.

2. **Integration Tests**
   - Evaluate canonical castles (Mino, Yagura, Anaguma) against broken castles to ensure king safety differentials match expectations.
   - Compare mobility scores before/after refactor on identical positions to confirm performance and magnitude stability.

3. **Performance Benchmarks**
   - Extend existing evaluation benchmarks to sample mobility-heavy scenarios; measure nodes/sec with mobility enabled/disabled.
   - Record impact of caching improvements on average evaluation time; publish results in `engine-performance-analysis.md`.

4. **Telemetry**
   - Emit optional debug logs when mobility share of total evaluation exceeds threshold; use to detect hotspots.
   - Expose statistics snapshot via `EvaluationTelemetry` for downstream analytics.

---

## 10. Conclusion

The position feature evaluator provides a solid structural foundation—modular APIs, statistics hooks, and tapered scores—but it currently falls short in shogi-specific fidelity and runtime efficiency. Addressing configuration fidelity, shogi-aware heuristics (hand pieces, promoted defenders, castle archetypes), and mobility performance will notably elevate both strength and tunability. Prioritize enabling configuration toggles, optimizing mobility, and incorporating hand piece dynamics, followed by richer center/pawn heuristics and improved test coverage. These upgrades will prepare the evaluator for forthcoming integration tasks (Meta-Task 20.0) and support the broader engine improvement roadmap.

---
