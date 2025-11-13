# Tasks: Opening Principles Review Improvements

**Parent PRD:** `task-19.0-opening-principles-review.md`  
**Date:** December 2024  
**Status:** In Progress

---

## Overview

This task list implements the improvements identified in the Opening Principles Review (Task 19.0). The improvements address critical gaps in piece coordination evaluation, opening book integration, move count parameter bug, and performance optimizations to elevate the opening principles module from "good coverage of individual principles" to "comprehensive opening evaluation with shogi-specific coordination."

## Relevant Files

- `src/evaluation/opening_principles.rs` - Main opening principles evaluator with all five evaluation components
- `src/evaluation/integration.rs` - Integration into `IntegratedEvaluator` with phase-aware gating (line 446: move_count hardcoded to 0)
- `src/types.rs` - `TaperedScore`, `Position`, `PieceType`, `Player`, `OpeningPrincipleConfig`, `OpeningPrincipleStats` types
- `src/opening_book.rs` - Opening book implementation (JSON format, position lookup, move selection)
- `src/search/move_ordering.rs` - Opening book integration with move ordering (lines 6838-6886)
- `src/lib.rs` - Opening book lookup in search (lines 529-540)
- `src/evaluation/advanced_integration.rs` - Advanced integration with opening book (lines 76-85)
- `src/evaluation/config.rs` - Evaluation configuration presets
- `src/evaluation/statistics.rs` - Evaluation statistics aggregation
- `tests/evaluation/opening_principles_tests.rs` - Unit tests for opening principles (to be created/enhanced)
- `tests/evaluation/integration_tests.rs` - Integration tests for opening principles + opening book (to be created)
- `benches/opening_principles_performance_benchmarks.rs` - Performance benchmarks (exists, to be enhanced)

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests go in the `tests/` directory
- Benchmarks go in the `benches/` directory
- Use `cargo test` to run tests, `cargo bench` to run benchmarks

---

## Tasks

- [ ] 1.0 Fix Move Count Parameter Bug (High Priority - Est: 1-2 hours)
  - [ ] 1.1 Investigate how move_count is tracked in the evaluation/search context (check Position type, game state, or search depth)
  - [ ] 1.2 Determine the best way to calculate or pass move_count to `IntegratedEvaluator::evaluate()` method
  - [ ] 1.3 Update `IntegratedEvaluator::evaluate()` signature to accept move_count parameter (or calculate from position/context)
  - [ ] 1.4 Replace hardcoded `0` with actual move_count in `integration.rs` line 446: `.evaluate_opening(board, player, move_count)`
  - [ ] 1.5 Verify move_count is correctly passed through all evaluation call sites
  - [ ] 1.6 Write unit test `test_move_count_parameter_fix` to verify tempo bonuses apply when move_count <= 10
  - [ ] 1.7 Write integration test evaluating positions at move 5, 10, 15 to verify tempo/development tracking works correctly
  - [ ] 1.8 Add regression test to prevent move_count from being hardcoded to 0 again

- [ ] 2.0 Add Piece Coordination Evaluation (High Priority - Est: 8-12 hours)
  - [ ] 2.1 Add `enable_piece_coordination: bool` toggle to `OpeningPrincipleConfig` in `types.rs`
  - [ ] 2.2 Create `evaluate_piece_coordination()` method in `OpeningPrincipleEvaluator`
  - [ ] 2.3 Implement rook-lance battery detection (same file, both pieces developed, rook supports lance)
  - [ ] 2.4 Implement bishop-lance combination detection (same diagonal, both pieces developed)
  - [ ] 2.5 Implement gold-silver defensive coordination evaluation (golds and silvers near king forming defensive structure)
  - [ ] 2.6 Implement rook-bishop coordination evaluation (attacking combinations, both pieces developed)
  - [ ] 2.7 Add piece synergy bonuses (developed pieces supporting each other, e.g., rook supporting developed silver)
  - [ ] 2.8 Integrate `evaluate_piece_coordination()` into `evaluate_opening()` method with config toggle
  - [ ] 2.9 Return `TaperedScore` with appropriate MG/EG weighting (MG emphasis, EG = MG / 4)
  - [ ] 2.10 Write unit tests for rook-lance batteries (verify detection and scoring)
  - [ ] 2.11 Write unit tests for bishop-lance combinations (verify detection and scoring)
  - [ ] 2.12 Write unit tests for gold-silver coordination (verify defensive structure recognition)
  - [ ] 2.13 Write integration test with all coordination types present to verify combined scoring
  - [ ] 2.14 Add benchmark to measure coordination evaluation overhead

- [ ] 3.0 Integrate Opening Principles with Opening Book (High Priority - Est: 6-10 hours)
  - [ ] 3.1 Create `evaluate_book_move_quality()` method in `OpeningPrincipleEvaluator` that scores a move using opening principles
  - [ ] 3.2 Modify method to accept a `Move` parameter and evaluate the position after making the move
  - [ ] 3.3 Return a quality score (TaperedScore or i32) indicating how well the move aligns with opening principles
  - [ ] 3.4 Update `opening_book.rs` or `move_ordering.rs` to call `evaluate_book_move_quality()` when multiple book moves are available
  - [ ] 3.5 Implement book move prioritization: sort book moves by opening principles score (highest first)
  - [ ] 3.6 Add book move validation: warn (via debug log) if book move violates opening principles (e.g., early king move, undeveloped major piece)
  - [ ] 3.7 Integrate opening principles into move ordering when book moves are present (use principles to break ties)
  - [ ] 3.8 Add statistics tracking: `book_moves_evaluated`, `book_moves_prioritized`, `book_moves_validated`, `book_move_quality_scores`
  - [ ] 3.9 Update `OpeningPrincipleStats` structure to include book integration statistics
  - [ ] 3.10 Write unit test `test_book_move_quality_evaluation` to verify book moves are scored correctly
  - [ ] 3.11 Write unit test `test_book_move_prioritization` to verify multiple book moves are sorted by quality
  - [ ] 3.12 Write integration test `test_opening_book_principles_coordination` to verify book + principles work together
  - [ ] 3.13 Add debug logging for book move quality scores when both book and principles are enabled
  - [ ] 3.14 Update documentation explaining book-principles integration

- [ ] 4.0 Performance Optimizations and Statistics (Medium Priority - Est: 14-18 hours)
  - [ ] 4.1 **Optimize Board Scans** - Replace O(81) `find_pieces()` scans with bitboard operations
  - [ ] 4.2 Investigate bitboard operations for piece finding (check existing bitboard infrastructure)
  - [ ] 4.3 Refactor `find_pieces()` to use bitboard lookups instead of iterating over all 81 squares
  - [ ] 4.4 Update `evaluate_major_piece_development()` to use optimized piece finding
  - [ ] 4.5 Update `evaluate_minor_piece_development()` to use optimized piece finding
  - [ ] 4.6 Update `evaluate_center_control_opening()` to use optimized piece finding
  - [ ] 4.7 Update `evaluate_opening_penalties()` to use optimized piece finding
  - [ ] 4.8 Add benchmark comparing O(81) scans vs. bitboard operations (measure evaluation overhead reduction)
  - [ ] 4.9 **Add Per-Component Statistics** - Enhance `OpeningPrincipleStats` structure
  - [ ] 4.10 Add per-component score tracking: `development_score`, `center_control_score`, `castle_formation_score`, `tempo_score`, `penalties_score`
  - [ ] 4.11 Add per-component evaluation counts: `development_evaluations`, `center_control_evaluations`, etc.
  - [ ] 4.12 Update each component evaluation method to track its contribution to total score
  - [ ] 4.13 Add `get_component_statistics()` method to return per-component breakdown
  - [ ] 4.14 Integrate statistics with `EvaluationStatistics` aggregation in `statistics.rs`
  - [ ] 4.15 Write unit tests verifying per-component statistics are tracked correctly
  - [ ] 4.16 Add benchmark measuring statistics tracking overhead
  - [ ] 4.17 **Add Center Control via Piece Attacks** - Enhance center control evaluation
  - [ ] 4.18 Create `evaluate_center_control_via_attacks()` method that evaluates center control from piece attacks (not just occupied squares)
  - [ ] 4.19 Integrate attack-based center control into `evaluate_center_control_opening()` (combine with existing occupied-square evaluation)
  - [ ] 4.20 Add configuration toggle `enable_attack_based_center_control` in `OpeningPrincipleConfig`
  - [ ] 4.21 Write unit tests for attack-based center control evaluation
  - [ ] 4.22 Add benchmark comparing center control evaluation with/without attack-based assessment
  - [ ] 4.23 **Integrate with PositionFeatureEvaluator** - Avoid redundancy with center control maps (Section 3.4)
  - [ ] 4.24 Investigate `PositionFeatureEvaluator` center control maps to identify overlap with opening principles center control
  - [ ] 4.25 Refactor to share center control evaluation logic or use PositionFeatureEvaluator results when available
  - [ ] 4.26 Add configuration option to use PositionFeatureEvaluator center control instead of duplicate evaluation

- [ ] 5.0 Advanced Features and Enhancements (Low Priority - Est: 20-26 hours)
  - [ ] 5.1 **Add Drop Pressure Evaluation** - Evaluate center control via potential drops (shogi-specific)
  - [ ] 5.2 Create `evaluate_drop_pressure_on_center()` method that evaluates center control via potential piece drops
  - [ ] 5.3 Check captured pieces to determine which pieces can be dropped
  - [ ] 5.4 Evaluate center squares that could be controlled via drops (bishop, rook, silver, gold, knight drops)
  - [ ] 5.5 Integrate drop pressure evaluation into `evaluate_center_control_opening()` with config toggle
  - [ ] 5.6 Add configuration toggle `enable_drop_pressure_evaluation` in `OpeningPrincipleConfig`
  - [ ] 5.7 Write unit tests for drop pressure evaluation (verify correct calculation with various captured piece combinations)
  - [ ] 5.8 **Add Move History Tracking** - Track repeated piece moves in opening
  - [ ] 5.9 Add `move_history: Vec<Move>` field to `OpeningPrincipleEvaluator` (or pass as parameter)
  - [ ] 5.10 Implement detection of repeated piece moves (same piece moved multiple times in opening)
  - [ ] 5.11 Add penalty in `evaluate_opening_penalties()` for moving same piece multiple times (addresses TODO in code)
  - [ ] 5.12 Update `evaluate_opening()` signature to accept move history (or extract from position/context)
  - [ ] 5.13 Write unit tests for move history tracking and repeated move penalties
  - [ ] 5.14 **Add Telemetry Integration** - Log opening principles impact on move selection
  - [ ] 5.15 Add telemetry field `opening_principles_influenced_move: bool` to track when principles influence best move
  - [ ] 5.16 Add debug logging when opening principles component contributions exceed threshold (e.g., > 100cp)
  - [ ] 5.17 Log opening book move quality scores when book + principles are both enabled
  - [ ] 5.18 Integrate telemetry with existing `DEBUG_LOGGING_OPTIMIZATION.md` guidance
  - [ ] 5.19 Add telemetry statistics: `moves_influenced_by_development`, `moves_influenced_by_center_control`, etc.
  - [ ] 5.20 Write integration test verifying telemetry logs are generated correctly
  - [ ] 5.21 Update documentation with telemetry usage guide
  - [ ] 5.22 **A/B Testing Framework Support** - Enable A/B testing for opening strength improvement (Section 6.4)
  - [ ] 5.23 Design A/B testing interface that uses per-component statistics (from Task 4.9-4.16) to compare opening principles configurations
  - [ ] 5.24 Add configuration presets for A/B testing (e.g., with/without piece coordination, different center control weights)
  - [ ] 5.25 Document A/B testing methodology using per-component statistics to measure opening strength improvement

---

**Phase 2 Complete - Detailed Sub-Tasks Generated**

All parent tasks have been broken down into **77 actionable sub-tasks**. Each sub-task is specific, testable, and includes:
- Implementation details based on the opening principles review
- Testing requirements (unit tests, integration tests, benchmarks)
- Statistics tracking for monitoring effectiveness
- Documentation updates where applicable
- Cross-references to specific sections in the review document

**Coverage Verification:**

✅ **Section 8 (Improvement Recommendations):**
- High Priority: Move count fix → Task 1.0 (8 sub-tasks)
- High Priority: Piece coordination → Task 2.0 (14 sub-tasks)
- High Priority: Opening book integration → Task 3.0 (14 sub-tasks)
- Medium Priority: Board scan optimization → Task 4.1-4.8 (8 sub-tasks)
- Medium Priority: Per-component statistics → Task 4.9-4.16 (8 sub-tasks)
- Medium Priority: Center control via attacks → Task 4.17-4.22 (6 sub-tasks)
- Medium Priority: Integration with PositionFeatureEvaluator → Task 4.23-4.26 (4 sub-tasks, addresses Section 3.4 gap)
- Low Priority: Drop pressure evaluation → Task 5.1-5.7 (7 sub-tasks)
- Low Priority: Move history tracking → Task 5.8-5.13 (6 sub-tasks)
- Low Priority: Telemetry integration → Task 5.14-5.21 (8 sub-tasks)
- Low Priority: A/B testing framework support → Task 5.22-5.25 (4 sub-tasks, addresses Section 6.4 measurement gap)

✅ **Section 9 (Testing & Validation Plan):**
- Unit Tests → Tasks 1.6-1.7, 2.10-2.13, 3.10-3.11, 4.15, 4.21, 5.7, 5.13
- Integration Tests → Tasks 1.7, 2.13, 3.12, 5.20
- Performance Benchmarks → Tasks 2.14, 4.8, 4.16, 4.22
- Telemetry → Tasks 5.14-5.20

**Task Priorities:**
- **Phase 1 (High Priority, 1-2 weeks):** Tasks 1.0, 2.0, 3.0 - Critical bug fixes and missing features
- **Phase 2 (Medium Priority, 2-3 weeks):** Task 4.0 - Performance and observability improvements
- **Phase 3 (Low Priority, 3-4 weeks):** Task 5.0 - Advanced features and enhancements

**Expected Cumulative Benefits:**
- **Bug Fix:** Tempo bonuses now apply correctly (Task 1.0)
- **Feature Completeness:** Piece coordination evaluation adds critical shogi opening concept (Task 2.0)
- **Integration:** Opening book and principles work together for better move selection (Task 3.0)
- **Performance:** 20-40% evaluation overhead reduction from bitboard optimizations (Task 4.0)
- **Observability:** Per-component statistics enable tuning and A/B testing (Task 4.0)
- **Advanced Features:** Drop pressure, move history, and telemetry enhance evaluation quality (Task 5.0)

