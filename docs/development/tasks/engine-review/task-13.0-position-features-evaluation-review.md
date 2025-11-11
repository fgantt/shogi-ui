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
- ✅ Mobility evaluation now shares a single `MoveGenerator` per pass, aggregates legal moves once, and avoids the prior O(n²) blow-up observed in profiling (see Task 2.0 notes).
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

Updates:
- Pawn chains now consider diagonal and forward-support links, with hand-supported gaps filled by both pawn and gold drops when legal.
- Advancement scoring uses shogi-specific phase tables that reward penetration into the enemy camp while tapering values near home ranks.
- Isolation and passed-pawn heuristics factor in opponent hand drops (pawns, golds, silvers, lances, knights) and on-board promoted blockers.
- Illegal doubled pawns receive immediate and severe penalties to reflect shogi rules while still detecting multi-file pressure.

---

## 4. Mobility Analysis (Task 13.4)

```539:586:src/evaluation/position_features.rs
    pub fn evaluate_mobility(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        if !self.config.enable_mobility {
            return TaperedScore::default();
        }

        self.stats.mobility_evals += 1;

        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        let mut piece_stats = vec![PieceMobilityStats::default(); MOBILITY_BOARD_AREA];
        let mut drop_stats = [DropMobilityStats::default(); PieceType::COUNT];

        for mv in &legal_moves {
            if let Some(from) = mv.from {
                let stats = &mut piece_stats[index_for_position(from)];
                stats.total_moves += 1;
                if self.is_central_square(mv.to) {
                    stats.central_moves += 1;
                }
                if mv.is_capture {
                    stats.attack_moves += 1;
                }
            } else {
                let stats = &mut drop_stats[mv.piece_type.as_index()];
                stats.total_moves += 1;
                if self.is_central_square(mv.to) {
                    stats.central_moves += 1;
                }
            }
        }

        let mut total = TaperedScore::default();
        // Per-piece accumulation (players' pieces)
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        total += self.evaluate_piece_mobility_from_stats(
                            piece.piece_type,
                            &piece_stats[index_for_position(pos)],
                        );
                    }
                }
            }
        }
        // Hand (drop) mobility
        for piece_type in ALL_PIECE_TYPES.iter().copied() {
            total += self.evaluate_drop_mobility(piece_type, &drop_stats[piece_type.as_index()]);
        }

        total
    }
```

Findings:
- **Performance:** The evaluator now generates legal moves once per pass, caches per-square stats, and records drop mobility; the previous O(pieces²) pattern is eliminated. Local benchmarking shows ~19–20× speedup against the reconstructed naive loop (see Task 2.0 notes).
- **Hand pressure:** Drop moves are counted with dedicated weights/bonuses, so mobility now acknowledges shogi hand threats instead of ignoring `from == None` moves.
- **Scoring:** Rebalanced weights and penalties reduce over-punishment of castle defenders and promoted minors while keeping central/attack bonuses intact.

Impact: mobility is no longer the dominant runtime hotspot and produces shogi-aware heuristics for both on-board and hand pieces.

---

## 5. Center Control & Development (Tasks 13.5 & 13.6)

Updates:
- Center control now builds attack maps using the shared `MoveGenerator`, compares player vs opponent control over the 3×3 core, 5×5 band, edge files, and a set of castle anchor squares, and grants additional credit when gold-equivalent defenders secure those anchors.  
- Development scoring penalises home-rank Golds/Silvers/Knights, rewards forward advancement proportionally, and applies retreat penalties when promoted pieces drift back into the rear three ranks.  
- New regression tests (`tests/center_control_development_tests.rs`) cover attack-map superiority, edge pressure rewards, penalties for undeveloped knights, and retreating promoted defenders.

Impact: center and development signals now distinguish active control from passive occupancy and deliver shogi-aware feedback for both early development and midgame castle formation.

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

**Update (Nov 2025):** Position feature toggles are now enforced in `position_features.rs`, and `IntegratedEvaluator` applies per-feature weights and respects the configuration during integration. Regression tests covering disabled features and weight propagation run in the default test suite.

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
