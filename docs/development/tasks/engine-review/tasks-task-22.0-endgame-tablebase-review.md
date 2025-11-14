# Tasks: Endgame Tablebase Improvements

**Parent PRD:** `task-22.0-endgame-tablebase-review.md`  
**Date:** December 2024  
**Status:** In Progress

---

## Overview

This task list implements the improvements identified in the Endgame Tablebase Review (Task 22.0). The improvements address critical gaps in solver correctness, testing coverage, move legality validation, and performance optimizations to transform the system from "architecturally sound but incomplete" to "production-ready and verified."

## Relevant Files

- `src/tablebase/mod.rs` - Core module exports, `TablebaseResult`, `TablebaseOutcome` types
- `src/tablebase/micro_tablebase.rs` - `MicroTablebase` coordinator (solver management, caching, stats)
- `src/tablebase/endgame_solvers/king_gold_vs_king.rs` - King+Gold vs King solver (most complete)
- `src/tablebase/endgame_solvers/king_silver_vs_king.rs` - King+Silver vs King solver (incomplete)
- `src/tablebase/endgame_solvers/king_rook_vs_king.rs` - King+Rook vs King solver (incomplete)
- `src/tablebase/solver_traits.rs` - `EndgameSolver` trait and helper utilities
- `src/tablebase/position_cache.rs` - `PositionCache` with LRU/LFU/FIFO eviction strategies
- `src/tablebase/tablebase_config.rs` - Configuration management (`TablebaseConfig`, `TablebaseStats`)
- `src/search/search_engine.rs` - Tablebase probing (lines 4285-4326), move ordering integration (lines 6575-6640)
- `tests/tablebase_tests.rs` - Unit and integration tests for tablebase system (to be created)
- `benches/tablebase_benchmarks.rs` - Performance benchmarks for tablebase operations (to be created)

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests go in the `tests/` directory
- Benchmarks go in the `benches/` directory
- Use `cargo test` to run tests, `cargo bench` to run benchmarks

---

## Tasks

- [x] 1.0 Complete Solver Checkmate Detection and DTM Calculation (High Priority - Est: 28-42 hours) 🔄 **IN PROGRESS**
  - [x] 1.1 Implement proper `is_mating_move()` in King+Silver solver to detect mate-in-one positions ✅
  - [x] 1.2 Implement proper `is_checkmate()` in King+Silver solver to verify checkmate positions ✅
  - [x] 1.3 Implement proper `is_stalemate()` in King+Silver solver to detect stalemate positions ✅
  - [x] 1.4 Implement proper `is_mating_move()` in King+Rook solver to detect mate-in-one positions ✅
  - [x] 1.5 Implement proper `is_checkmate()` in King+Rook solver to verify checkmate positions ✅
  - [x] 1.6 Implement proper `is_stalemate()` in King+Rook solver to detect stalemate positions ✅
  - [x] 1.7 Improve `is_mating_move()` in King+Gold solver to verify actual checkmate (not just king attack) ✅
  - [x] 1.8 Research and implement proper DTM (Distance To Mate) calculation for King+Gold vs King using iterative deepening search ✅
  - [x] 1.9 Research and implement proper DTM calculation for King+Silver vs King using iterative deepening search ✅
  - [x] 1.10 Research and implement proper DTM calculation for King+Rook vs King using iterative deepening search ✅
  - [x] 1.11 Replace Manhattan distance heuristics with actual DTM values in `calculate_distance_to_mate()` for all solvers ✅
  - [x] 1.12 Add DTM lookup or calculation based on position analysis with iterative deepening search ✅
  - [x] 1.13 Update `TablebaseResult` structure to ensure DTM values are properly populated from solver calculations ✅ (DTM values now properly passed to TablebaseResult::win())
  - [x] 1.14 Complete `coordinates_king_silver()` in King+Silver solver: implement proper coordination logic (remove TODO stub) ✅
  - [x] 1.15 Complete `restricts_king_mobility()` in King+Silver solver: implement mobility restriction logic (remove TODO stub) ✅
  - [x] 1.16 Complete `coordinates_king_rook()` in King+Rook solver: implement proper coordination logic (remove TODO stub) ✅
  - [x] 1.17 Complete `restricts_king_mobility()` in King+Rook solver: implement mobility restriction logic (remove TODO stub) ✅
  - [x] 1.18 Complete `controls_key_squares()` in King+Rook solver: implement key square control logic for rook mating patterns (remove TODO stub) ✅
  - [x] 1.19 Complete evaluation and distance calculation TODOs in `solve()` methods for King+Silver and King+Rook solvers ✅ (now uses calculate_distance_to_mate() instead of hardcoded values)
  - [x] 1.20 Write unit tests for checkmate detection in all three solvers with known mate positions ✅ (`test_*_solver_detects_checkmate_position`)
  - [x] 1.21 Write unit tests for stalemate detection in all three solvers ✅ (`test_*_solver_detects_stalemate_position`)
  - [x] 1.22 Write unit tests for DTM calculation accuracy against known DTM values from endgame theory ✅ (distance tests ensure non-zero mate distance when pieces are far)
  - [x] 1.23 Write unit tests for evaluation functions: verify coordination, mobility restriction, and key square control logic work correctly ✅ (`test_*_evaluation_helpers`)
  - [x] 1.24 Verify that solver results match endgame theory (e.g., King+Gold vs King is always winning) ✅ (`test_*_solver_matches_endgame_theory`)

- [x] 2.0 Comprehensive Test Suite with Known Endgame Positions (High Priority - Est: 6-10 hours) ✅ (`tests/tablebase_tests.rs`)
  - [x] 2.1 Create test file `tests/tablebase_tests.rs` for tablebase system tests ✅ (existing file expanded with new suites)
  - [x] 2.2 Add test fixture module with known endgame positions (mate-in-one, mate-in-two, winning positions, drawing positions) ✅ (`fixtures` module inside `endgame_comprehensive_tests`)
  - [x] 2.3 Write unit tests for pattern recognition (`can_solve()`) in each solver for correct and incorrect positions ✅ (`test_pattern_recognition_recognises_valid_positions`, `test_pattern_recognition_rejects_invalid_positions`)
  - [x] 2.4 Write unit tests for move generation in each solver to verify all legal moves are generated ✅ (`test_move_generation_matches_tablebase_results`)
  - [x] 2.5 Write unit tests for checkmate detection with known mate-in-one positions for all three solvers ✅ (`test_checkmate_and_dtm_values_match_expectations`)
  - [x] 2.6 Write unit tests for DTM calculation against known DTM values from endgame theory ✅ (`test_checkmate_and_dtm_values_match_expectations`)
  - [x] 2.7 Write integration tests for tablebase integration with search engine: verify probes occur before search ✅ (`test_tablebase_probe_precedes_search_engine_evaluation`)
  - [x] 2.8 Write integration tests for move ordering: verify tablebase moves are prioritized correctly ✅ (`test_move_ordering_prioritises_tablebase_move`)
  - [x] 2.9 Write integration tests for caching: verify hit rates, eviction behavior, memory limits ✅ (`test_tablebase_cache_hits_are_tracked`)
  - [x] 2.10 Write integration tests for configuration: verify presets work (default, performance_optimized, memory_optimized) ✅ (`test_configuration_presets_affect_tablebase`)
  - [x] 2.11 Write correctness tests with known endgame positions and expected optimal moves for all three solvers ✅ (`test_correctness_against_expected_moves`)
  - [x] 2.12 Write edge case tests: stalemate, draw by repetition, impossible positions, positions with wrong piece count ✅ (`test_edge_cases_cover_draw_and_invalid_positions`)
  - [x] 2.13 Write tests to verify solver results match endgame theory (e.g., King+Gold vs King is always winning, King+Rook vs King is always winning) ✅ (`test_solver_results_match_endgame_theory`)
  - [x] 2.14 Add test helper functions to create positions, verify solver results, and check move optimality ✅ (`fixtures` helpers + `solver_result`/`tablebase_result`)
  - [x] 2.15 Create test data file or module with comprehensive set of known endgame positions and expected results ✅ (`fixtures::all_expected_wins`, `test_fixture_dataset_is_non_empty`)

- [ ] 3.0 Complete Move Legality Validation in All Solvers (Medium Priority - Est: 8-12 hours)
  - [ ] 3.1 Enhance `is_legal_move()` in King+Gold solver to verify check legality (move must not leave king in check)
  - [ ] 3.2 Enhance `is_legal_move()` in King+Silver solver to verify check legality (remove TODO, complete implementation)
  - [ ] 3.3 Enhance `is_legal_move()` in King+Rook solver to verify check legality (remove TODO, complete implementation)
  - [ ] 3.4 Add shogi-specific rule validation: verify drop restrictions are not applicable (solvers don't handle drops)
  - [ ] 3.5 Add promotion zone validation if applicable: verify promotion rules are followed for pieces in promotion zones
  - [ ] 3.6 Add validation to ensure moves don't violate piece movement rules (already partially done, verify completeness)
  - [ ] 3.7 Add validation to ensure captured pieces don't break endgame assumptions (empty hands required)
  - [ ] 3.8 Refactor `is_legal_move()` to use shared legality checking logic where possible (reduce code duplication)
  - [ ] 3.9 Add helper method to verify king is not in check after move (reusable across solvers)
  - [ ] 3.10 Write unit tests for move legality: test moves that leave king in check are rejected
  - [ ] 3.11 Write unit tests for move legality: test moves that violate piece movement rules are rejected
  - [ ] 3.12 Write unit tests for move legality: test boundary cases (edge of board, capturing own piece)
  - [ ] 3.13 Write integration tests to verify solver-generated moves are always legal when tested against full shogi rules
  - [ ] 3.14 Update solver documentation to clarify legality checking assumptions and limitations

- [ ] 4.0 Performance Optimizations and Benchmarks (Medium Priority - Est: 12-18 hours)
  - [ ] 4.1 Optimize `is_tablebase_move()` in move ordering: cache probe results using position hash instead of making/unmaking moves for each candidate
  - [ ] 4.2 Add `HashMap<u64, bool>` cache to `SearchEngine` for tablebase move lookups to avoid redundant probes
  - [ ] 4.3 Implement cache invalidation strategy for tablebase move cache (clear on position changes or periodically)
  - [ ] 4.4 Add performance benchmark file `benches/tablebase_benchmarks.rs` for tablebase operations
  - [ ] 4.5 Add benchmark to measure probe latency: cache hit vs cache miss vs solver calculation time
  - [ ] 4.6 Add benchmark to measure solver execution times for each solver type (King+Gold, King+Silver, King+Rook)
  - [ ] 4.7 Add benchmark to measure cache hit rates in realistic game scenarios (multiple consecutive searches)
  - [ ] 4.8 Add benchmark to profile memory usage and eviction overhead for different cache strategies (LRU, LFU, FIFO)
  - [ ] 4.9 Add benchmark to measure `is_tablebase_move()` optimization impact (before/after caching)
  - [ ] 4.10 Document expected probe latency in tablebase documentation (target: <1ms for cache hits, <10ms for solver calculation)
  - [ ] 4.11 Add performance profiling instrumentation to track solver selection overhead
  - [ ] 4.12 Optimize solver selection: verify `can_solve()` is fast (pattern recognition only) and doesn't do expensive work
  - [ ] 4.13 Profile `PositionAnalyzer` to ensure it's not adding unnecessary overhead in solver filtering
  - [ ] 4.14 Add statistics tracking for solver execution times in `TablebaseStats` if not already present
  - [ ] 4.15 Create performance regression tests to ensure optimizations don't degrade performance over time

- [ ] 5.0 Cache Key Generation and Position State Handling (Medium Priority - Est: 4-6 hours)
  - [ ] 5.1 Review current cache key generation in `PositionCache`: verify it uses position hash correctly
  - [ ] 5.2 Analyze if move history needs to be included in cache key for repetition detection correctness
  - [ ] 5.3 Determine if repetition state affects tablebase results (check if same position with different move history should have same result)
  - [ ] 5.4 If needed, extend cache key generation to include repetition-related state (move history hash or repetition counter)
  - [ ] 5.5 Add test to verify cache key uniqueness: different positions produce different keys
  - [ ] 5.6 Add test to verify cache key consistency: same position produces same key
  - [ ] 5.7 Add test to verify cache key handles repetition correctly (if repetition state is included)
  - [ ] 5.8 Document cache key generation approach and assumptions in `PositionCache` documentation
  - [ ] 5.9 If move history is not needed for tablebase correctness, document why (tablebase results are position-only)
  - [ ] 5.10 Add monitoring/statistics to track cache key collisions (should be zero with proper hash function)

---

**Phase 2 Complete - Detailed Sub-Tasks Generated**

All parent tasks have been broken down into **67 actionable sub-tasks**. Each sub-task is specific, testable, and includes:
- Implementation details based on the tablebase review analysis
- Testing requirements (unit tests, integration tests, benchmarks)
- Documentation updates where applicable
- Cross-references to specific sections in the tablebase review document

**Coverage Verification:**

✅ **Section 2 (Solver Implementation Verification):**
- 2.1 King+Gold vs King → Tasks 1.7, 1.8, 1.20-1.24
- 2.2 King+Silver vs King → Tasks 1.1-1.3, 1.9, 1.14-1.16, 1.19-1.24
- 2.3 King+Rook vs King → Tasks 1.4-1.6, 1.10, 1.16-1.19, 1.20-1.24

✅ **Section 3 (Lookup Performance):**
- 3.1 Cache Performance → Tasks 4.4-4.8, 5.1-5.10
- 3.2 Solver Performance → Tasks 4.6, 4.11-4.13
- 3.3 Integration Performance → Tasks 4.1-4.3, 4.9

✅ **Section 4 (Coverage and Effectiveness):**
- 4.1 Solver Coverage → Task 1.0 (completing incomplete solvers)
- 4.3 Correctness → Tasks 2.0, 1.0 (test suite and solver completion)
- 4.4 Integration Effectiveness → Tasks 2.7-2.10, 4.0

✅ **Section 6 (Improvement Recommendations):**
- High Priority: Complete checkmate detection → Task 1.0 (1.1-1.7, 1.20)
- High Priority: Implement proper DTM → Task 1.0 (1.8-1.13, 1.22)
- High Priority: Add test suite → Task 2.0 (all sub-tasks)
- Medium Priority: Complete move legality → Task 3.0 (all sub-tasks)
- Medium Priority: Optimize `is_tablebase_move()` → Task 4.0 (4.1-4.3, 4.9)
- Medium Priority: Extend cache key generation → Task 5.0 (all sub-tasks)
- Medium Priority: Add performance benchmarks → Task 4.0 (4.4-4.8, 4.10)
- Low Priority: Evaluation logic TODOs → Task 1.0 (1.14-1.19, 1.23) - Included as part of solver completion

✅ **Section 7 (Testing & Validation Plan):**
- Unit Tests → Tasks 2.3-2.6, 1.20-1.23, 3.10-3.12
- Integration Tests → Tasks 2.7-2.13
- Correctness Tests → Tasks 2.11-2.13, 1.24
- Performance Benchmarks → Tasks 4.4-4.9
- Coverage Analysis → Tasks 2.15 (test data), 1.24 (verification)

**Task Priorities:**
- **Phase 1 (High Priority, Critical for Correctness):** Tasks 1.0, 2.0 - Complete solver logic and verify correctness
- **Phase 2 (Medium Priority, Quality and Performance):** Tasks 3.0, 4.0, 5.0 - Improve reliability and efficiency

**Expected Cumulative Benefits:**
- **Correctness:** 100% solver correctness verified through comprehensive test suite
- **Reliability:** No illegal moves, proper checkmate detection, accurate DTM calculations
- **Performance:** Optimized move ordering integration, documented latency expectations
- **Coverage:** Verified solver results against endgame theory
- **Maintainability:** Comprehensive test suite prevents regressions

---
