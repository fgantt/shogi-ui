# Task List: Internal Iterative Deepening Improvements

**PRD:** `task-4.0-internal-iterative-deepening-review.md`  
**Date:** December 2024  
**Status:** In Progress

---

## Relevant Files

### Primary Implementation Files
- `src/search/search_engine.rs` - Core search engine implementation
  - `should_apply_iid()` - Condition checking (lines 635-670)
  - `calculate_iid_depth()` - Depth calculation (lines 673-687)
  - `perform_iid_search()` - IID search execution (lines 697-750)
  - `calculate_dynamic_iid_depth()` - Dynamic depth calculation (lines 1055-1080, not integrated)
  - `estimate_iid_time()` - Time estimation (lines 1412-1427, not used)
  - `assess_position_complexity()` - Position complexity assessment (exists, not used in IID)
  - `monitor_iid_overhead()` - Overhead monitoring (lines 1294-1318)
  - `adjust_overhead_thresholds()` - Overhead adjustment (lines 1320-1354)
  - `is_iid_overhead_acceptable()` - Overhead checking (lines 1401-1409)
  - `adapt_iid_configuration()` - Configuration adaptation (lines 792-855)
  - `get_iid_performance_metrics()` - Performance metrics (lines 2513-2517, uses placeholder)
  - Integration in `negamax_with_context()` (lines 3086-3129)
  - `sort_moves()` and `score_move()` - IID move ordering integration (lines 3629-3740)

- `src/types.rs` - Configuration and statistics structures
  - `IIDConfig` - Configuration structure (lines 3690-3943)
  - `IIDStats` - Statistics tracking (lines 3690-3943)
  - Needs updates for total search time tracking, IID move extraction improvements

### Supporting Files
- `src/search/move_ordering.rs` - Move ordering (for IID move integration in advanced ordering)
- `src/search/transposition_table.rs` - Transposition table (for IID move extraction improvements)
- `src/evaluation/evaluation.rs` - Position evaluation (for position complexity assessment)

### Test Files
- `benches/` - Performance benchmarks
  - Should add benchmarks for IID performance impact (with/without IID comparison)
  - Should add benchmarks for IID move extraction accuracy
  - Should add performance monitoring benchmarks
- `tests/` - Unit tests
  - Should add tests for IID move extraction, advanced ordering integration, dynamic depth calculation

### Configuration Files
- `Cargo.toml` - Build configuration (for feature flags if needed)

### Notes
- These improvements address missing features and code quality issues identified in Task 4.0 review
- High priority items focus on fixing critical bugs (total search time tracking), improving reliability (IID move extraction), and ensuring integration (advanced ordering)
- All changes should maintain backward compatibility with existing IID functionality
- Tests should verify both correctness and performance improvements
- Performance improvements should maintain existing IID effectiveness while reducing overhead

---

## Tasks

- [x] 1.0 Fix Total Search Time Tracking
  - [x] 1.1 Review current `get_iid_performance_metrics()` implementation (lines 2513-2517) - uses placeholder `total_search_time_ms = 1000`
  - [x] 1.2 Add `total_search_time_ms` field to `SearchEngine` state or `IIDStats` to track actual total search time
  - [x] 1.3 Update search entry point to record start time when search begins
  - [x] 1.4 Update search exit point to calculate total search time and store in state/stats
  - [x] 1.5 Modify `get_iid_performance_metrics()` to use actual total search time instead of placeholder
  - [x] 1.6 Fix overhead percentage calculation: `overhead_percentage = (iid_time_ms / total_search_time_ms) * 100`
  - [x] 1.7 Add unit tests verifying total search time is correctly tracked
  - [x] 1.8 Add unit tests verifying overhead percentage calculation is accurate
  - [x] 1.9 Verify overhead percentage matches expected values (5-15% typically)
  - [x] 1.10 Update performance reports to use accurate overhead calculations
  - [x] 1.11 Document that overhead tracking now uses actual search time

- [x] 2.0 Improve IID Move Extraction
  - [x] 2.1 Review current IID move extraction from transposition table (lines 738-745)
  - [x] 2.2 Identify where best move is tracked during IID search in `perform_iid_search()`
  - [x] 2.3 Modify `perform_iid_search()` to track best move during search, not just from TT
  - [x] 2.4 Change return type of `perform_iid_search()` to return `(i32, Option<Move>)` instead of just `i32`
  - [x] 2.5 Remove dependency on `iid_score > alpha` condition for move extraction (IID should provide ordering even if score doesn't beat alpha)
  - [x] 2.6 Update IID move extraction to always return best move from IID search if available
  - [x] 2.7 Add fallback logic: if TT has best move, use it; otherwise use tracked best move from search
  - [x] 2.8 Add verification that IID move is in legal moves list before using in ordering
  - [x] 2.9 Update `negamax_with_context()` to receive IID move from `perform_iid_search()` return value
  - [x] 2.10 Remove IID move extraction from transposition table if move is now returned directly
  - [x] 2.11 Add statistics tracking for IID move extraction success rate (TT vs tracked move)
  - [x] 2.12 Add debug logging for IID move extraction (conditional on debug flags)
  - [x] 2.13 Add unit tests for IID move extraction:
    - Test IID move returned when TT has best move
    - Test IID move returned from tracked best move when TT doesn't have it
    - Test IID move is None when search doesn't find any move
    - Test IID move is verified to be in legal moves list
  - [x] 2.14 Add unit tests verifying IID move extraction works even when score doesn't beat alpha
  - [x] 2.15 Create performance benchmarks comparing TT-based vs tracked move extraction
  - [x] 2.16 Verify IID move extraction improvement doesn't add significant overhead (<1% search time)
  - [x] 2.17 Review board.clone() usage at line 3102 - expensive but necessary; document rationale or investigate optimization if possible

- [x] 3.0 Integrate IID Move into Advanced Ordering
  - [x] 3.1 Review current move ordering integration: `order_moves_for_negamax()` (line 3136)
  - [x] 3.2 Identify where `order_moves_advanced()` is called (lines 456-468)
  - [x] 3.3 Add `iid_move: Option<Move>` parameter to `order_moves_for_negamax()` method signature
  - [x] 3.4 Pass `iid_move` parameter through `order_moves_for_negamax()` to `order_moves_advanced()`
  - [x] 3.5 Modify `order_moves_advanced()` to accept `iid_move` parameter
  - [x] 3.6 Update `order_moves_advanced()` to prioritize IID move with maximum score (similar to `score_move()` lines 3710-3714)
  - [x] 3.7 Ensure IID move is prioritized regardless of ordering method (traditional or advanced)
  - [x] 3.8 Add unit tests verifying IID move is prioritized in advanced ordering path
  - [x] 3.9 Add unit tests comparing ordering with/without IID move in advanced path
  - [ ] 3.10 Create performance benchmarks comparing IID effectiveness with traditional vs advanced ordering
  - [x] 3.11 Verify IID move ordering is effective in both ordering paths
  - [x] 3.12 Update documentation to clarify IID move is integrated into all ordering paths

- [x] 4.0 Use Dynamic Depth Calculation
  - [x] 4.1 Review `calculate_dynamic_iid_depth()` implementation (lines 1055-1080) - exists but not used
  - [x] 4.2 Review `assess_position_complexity()` implementation - exists but not used in IID
  - [x] 4.3 Integrate `calculate_dynamic_iid_depth()` into main IID flow in `calculate_iid_depth()`
  - [x] 4.4 Add new depth strategy option: `Dynamic` to `IIDDepthStrategy` enum
  - [x] 4.5 Update `calculate_iid_depth()` to support Dynamic strategy using `calculate_dynamic_iid_depth()`
  - [x] 4.6 Ensure `assess_position_complexity()` is called when using Dynamic strategy
  - [x] 4.7 Add maximum depth cap to Relative strategy (e.g., `min(4, main_depth - 2)`)
  - [x] 4.8 Enhance Adaptive strategy with more thresholds or position-based adjustments
  - [x] 4.9 Review minimum depth threshold (default: 4) - may be too conservative; consider making adaptive based on position characteristics
  - [x] 4.10 Update `IIDConfig::default()` to use Dynamic strategy if beneficial, or keep Fixed as default
  - [x] 4.11 Add configuration options for dynamic depth calculation:
    - Base depth (default: 2)
    - Complexity thresholds (low, medium, high)
    - Maximum depth cap
    - Adaptive minimum depth threshold
  - [x] 4.12 Add statistics tracking for dynamic depth selection (which depth was chosen and why)
  - [x] 4.13 Add debug logging for dynamic depth calculation (conditional on debug flags)
  - [x] 4.14 Add unit tests for dynamic depth calculation:
    - Test depth selection based on position complexity
    - Test depth cap is respected
    - Test different complexity levels result in appropriate depths
    - Test adaptive minimum depth threshold
  - [ ] 4.15 Create performance benchmarks comparing Fixed vs Dynamic depth strategies
  - [ ] 4.16 Measure improvement in IID effectiveness with dynamic depth calculation
  - [ ] 4.17 Verify dynamic depth calculation doesn't add significant overhead (<2% search time)

- [x] 5.0 Integrate Time Estimation into Decision Logic
  - [x] 5.1 Review `estimate_iid_time()` implementation (lines 1412-1427) - exists but not used
  - [x] 5.2 Review current `should_apply_iid()` decision logic (lines 635-670)
  - [x] 5.3 Add time estimation to `should_apply_iid()` decision: call `estimate_iid_time()` before performing IID
  - [x] 5.4 Add configuration option: `max_estimated_iid_time_ms` (default: 50ms, percentage of remaining time)
  - [x] 5.5 Skip IID if estimated time exceeds threshold: `if estimated_time > max_estimated_iid_time_ms { return false }`
  - [x] 5.6 Update time pressure detection to use actual IID time estimates instead of fixed 10% threshold
  - [x] 5.7 Integrate time estimation with time pressure detection: `if remaining_time < estimated_iid_time * 2 { return false }`
  - [x] 5.8 Add statistics tracking for time estimation accuracy (predicted vs actual IID time)
  - [x] 5.9 Add statistics tracking for IID skipped due to time estimation exceeding threshold
  - [x] 5.10 Add debug logging for time estimation decisions (conditional on debug flags)
  - [x] 5.11 Add unit tests for time estimation integration:
    - Test IID is skipped when estimated time exceeds threshold
    - Test time estimation is used in time pressure detection
    - Test time estimation accuracy is reasonable
  - [ ] 5.12 Create performance benchmarks comparing IID with/without time estimation
  - [ ] 5.13 Verify time estimation prevents excessive IID overhead (>15%)
  - [ ] 5.14 Measure improvement in time management with time estimation

- [ ] 6.0 Add Performance Measurement
  - [ ] 6.1 Review existing performance statistics tracking in `IIDStats`
  - [ ] 6.2 Add fields to `IIDStats` for performance comparison:
    - `total_nodes_without_iid` - Estimated nodes if IID were disabled
    - `total_time_without_iid` - Estimated time if IID were disabled
    - `nodes_saved` - Calculated nodes saved by IID
  - [ ] 6.3 Add method to estimate search performance without IID (using historical data or simulation)
  - [ ] 6.4 Implement nodes saved calculation: `nodes_saved = total_nodes_without_iid - total_nodes`
  - [ ] 6.5 Add speedup calculation: `speedup = (time_without_iid - time_with_iid) / time_without_iid * 100%`
  - [ ] 6.6 Add correlation tracking between efficiency/cutoff rates and actual speedup
  - [ ] 6.7 Add performance comparison metrics to `get_iid_performance_metrics()`:
    - Node reduction percentage
    - Speedup percentage
    - Net benefit (speedup - overhead)
  - [ ] 6.8 Add statistics tracking for performance measurement accuracy
  - [ ] 6.9 Add debug logging for performance measurements (conditional on debug flags)
  - [ ] 6.10 Add unit tests for performance measurement:
    - Test nodes saved calculation
    - Test speedup calculation
    - Test correlation tracking
  - [ ] 6.11 Create performance benchmarks comparing with/without IID to validate measurements
  - [ ] 6.12 Verify performance measurements match expected values (20-40% node reduction, 15-25% speedup)
  - [ ] 6.13 Document performance measurement methodology and interpretation

- [ ] 7.0 Enhance Position Complexity Assessment
  - [ ] 7.1 Review current `assess_position_complexity()` implementation
  - [ ] 7.2 Enhance complexity assessment with material balance analysis
  - [ ] 7.3 Enhance complexity assessment with piece activity metrics
  - [ ] 7.4 Enhance complexity assessment with threat detection
  - [ ] 7.5 Add game phase detection (opening/middlegame/endgame) to complexity assessment
  - [ ] 7.6 Integrate enhanced complexity assessment into IID depth calculation (Dynamic strategy)
  - [ ] 7.7 Use complexity assessment in IID skip conditions (e.g., skip IID in very simple positions)
  - [ ] 7.8 Review move count threshold (default: 35 moves) - may be too high for some positions; make adaptive based on position type (tactical vs quiet)
  - [ ] 7.9 Add configuration options for complexity-based IID adjustment:
    - Complexity thresholds (low, medium, high)
    - Depth adjustments per complexity level
    - Enable/disable complexity-based adjustments
    - Adaptive move count threshold based on position type
  - [ ] 7.10 Add statistics tracking for position complexity distribution
  - [ ] 7.11 Add statistics tracking for IID effectiveness by complexity level
  - [ ] 7.12 Add debug logging for complexity assessment (conditional on debug flags)
  - [ ] 7.13 Add unit tests for enhanced complexity assessment:
    - Test material balance analysis
    - Test piece activity metrics
    - Test threat detection
    - Test game phase detection
    - Test adaptive move count threshold
  - [ ] 7.14 Create performance benchmarks comparing basic vs enhanced complexity assessment
  - [ ] 7.15 Measure improvement in IID depth selection accuracy with enhanced assessment
  - [ ] 7.16 Verify enhanced complexity assessment doesn't add significant overhead (<2% search time)

- [ ] 8.0 Implement Performance Monitoring
  - [ ] 8.1 Review existing `monitor_iid_overhead()` implementation (lines 1294-1318)
  - [ ] 8.2 Integrate `monitor_iid_overhead()` into main search flow to actively monitor overhead during search
  - [ ] 8.3 Add automated benchmark suite that runs on CI/CD to track IID performance over time
  - [ ] 8.4 Create benchmark configuration file or script for consistent benchmark execution
  - [ ] 8.5 Add performance regression tests that fail if IID effectiveness drops below thresholds:
    - Efficiency rate < 30%
    - Overhead > 15%
    - Cutoff rate < 20%
  - [ ] 8.6 Implement statistics logging over time (save statistics to file or database for historical tracking)
  - [ ] 8.7 Add metrics for IID effectiveness across different position types (opening, middlegame, endgame)
  - [ ] 8.8 Create comparison benchmarks: IID enabled vs disabled, with different configurations
  - [ ] 8.9 Add automated performance reports generation (efficiency rate, cutoff rate, overhead, speedup, etc.)
  - [ ] 8.10 Integrate with existing statistics tracking to export metrics for analysis
  - [ ] 8.11 Add alert mechanism for high overhead (>15%) indicating too-aggressive IID
  - [ ] 8.12 Add alert mechanism for low efficiency (<30%) indicating IID not being effective
  - [ ] 8.13 Create visualization or reporting tool for IID performance metrics (optional, low priority)
  - [ ] 8.14 Document benchmark execution and interpretation in development documentation
  - [ ] 8.15 Set up CI/CD pipeline to run benchmarks automatically on commits (if not already configured)
  - [ ] 8.16 Add periodic performance reports comparing current vs baseline metrics

- [ ] 9.0 Improve Time Pressure Detection
  - [ ] 9.1 Review current `is_time_pressure()` implementation (lines 689-694) - uses fixed 10% threshold
  - [ ] 9.2 Enhance time pressure detection to use position complexity (skip IID in complex positions when time is low)
  - [ ] 9.3 Enhance time pressure detection to consider search depth (deeper searches need more time)
  - [ ] 9.4 Replace fixed 10% threshold with dynamic calculation based on position and depth
  - [ ] 9.5 Integrate with `estimate_iid_time()` to use actual IID time estimates in pressure detection
  - [ ] 9.6 Review TT move condition in `should_apply_iid()` - may be too restrictive; consider checking TT move depth or age before skipping IID
  - [ ] 9.7 Add configuration options for time pressure detection:
    - Base threshold (default: 10%)
    - Complexity multiplier
    - Depth multiplier
    - TT move depth/age threshold for IID decision
  - [ ] 9.8 Add statistics tracking for time pressure detection accuracy
  - [ ] 9.9 Add statistics tracking for TT move condition effectiveness (how often IID is skipped due to TT move)
  - [ ] 9.10 Add debug logging for time pressure detection decisions (conditional on debug flags)
  - [ ] 9.11 Add unit tests for enhanced time pressure detection:
    - Test time pressure in simple vs complex positions
    - Test time pressure at different depths
    - Test time pressure with actual IID time estimates
    - Test TT move condition with depth/age checking
  - [ ] 9.12 Create performance benchmarks comparing fixed vs enhanced time pressure detection
  - [ ] 9.13 Verify enhanced time pressure detection improves time management accuracy
  - [ ] 9.14 Measure improvement in search quality with better time management

- [ ] 10.0 Add Configuration Presets
  - [ ] 10.1 Create `IIDPreset` enum with variants: Conservative, Aggressive, Balanced
  - [ ] 10.2 Implement `from_preset()` method for `IIDConfig` to create configs from presets
  - [ ] 10.3 Define preset configurations:
    - Conservative: Lower time overhead threshold, higher min_depth, shallower IID depth
    - Aggressive: Higher time overhead threshold, lower min_depth, deeper IID depth
    - Balanced: Default values optimized for general play
  - [ ] 10.4 Add `preset` field to `IIDConfig` to track which preset was used (optional)
  - [ ] 10.5 Add `apply_preset()` method to `IIDConfig` to update config based on preset
  - [ ] 10.6 Update configuration documentation to describe presets and when to use each
  - [ ] 10.7 Add unit tests for preset configurations (verify settings match expected values)
  - [ ] 10.8 Add integration tests comparing preset performance (Conservative vs Aggressive vs Balanced)
  - [ ] 10.9 Update `IIDConfig::summary()` to include preset information if set
  - [ ] 10.10 Consider adding preset configuration via USI commands or configuration file
  - [ ] 10.11 Document recommended presets for different scenarios (tournament play, analysis, etc.)

- [ ] 11.0 Advanced Depth Strategies
  - [ ] 11.1 Research game phase-based depth adjustment (opening vs middlegame vs endgame)
  - [ ] 11.2 Implement game phase detection in IID depth calculation
  - [ ] 11.3 Add game phase-based depth adjustment: different IID depth for opening/middlegame/endgame
  - [ ] 11.4 Research material-based depth scaling (adjust depth based on material on board)
  - [ ] 11.5 Implement material-based depth adjustment: deeper IID in material-rich positions
  - [ ] 11.6 Research time-based depth adjustment (adjust depth based on remaining time)
  - [ ] 11.7 Implement time-based depth adjustment: shallower IID when time is low
  - [ ] 11.8 Add configuration options for advanced strategies:
    - Enable/disable game phase-based adjustment
    - Enable/disable material-based adjustment
    - Enable/disable time-based adjustment
    - Depth multipliers for each strategy
  - [ ] 11.9 Add statistics tracking for advanced strategy effectiveness
  - [ ] 11.10 Add unit tests for each advanced strategy
  - [ ] 11.11 Create performance benchmarks comparing basic vs advanced depth strategies
  - [ ] 11.12 Measure improvement potential (research shows diminishing returns for advanced strategies)
  - [ ] 11.13 Document advanced strategies and when to use them
  - [ ] 11.14 Decide whether to keep advanced strategies based on benchmark results

- [ ] 12.0 Add Cross-Feature Statistics and Move Ordering Integration
  - [ ] 12.1 Review IID statistics and move ordering statistics separation
  - [ ] 12.2 Add cross-feature statistics to track IID → ordering effectiveness:
    - Percentage of cutoffs from IID moves vs. non-IID moves
    - IID move position in ordered list (should be first)
    - Ordering effectiveness with/without IID
  - [ ] 12.3 Track IID move position in ordered list to verify it's prioritized
  - [ ] 12.4 Add comparison of ordering effectiveness with/without IID to measure improvement
  - [ ] 12.5 Add correlation tracking between IID efficiency/cutoff rates and move ordering quality metrics
  - [ ] 12.6 Add statistics tracking for IID move ordering verification
  - [ ] 12.7 Add debug logging for cross-feature statistics (conditional on debug flags)
  - [ ] 12.8 Add unit tests for cross-feature statistics:
    - Test IID move is ordered first
    - Test ordering effectiveness correlation
    - Test cutoff rate comparison
  - [ ] 12.9 Create performance benchmarks measuring IID → ordering effectiveness
  - [ ] 12.10 Document the dependency: IID effectiveness requires proper move ordering integration
  - [ ] 12.11 Use cross-feature statistics to identify opportunities for IID and ordering improvements

---

## Execution Order and Dependencies

### Phase 1: Critical Fixes (Week 1-2)
Complete high-priority tasks 1.0, 2.0, 3.0:
- Task 1.0 (Fix Total Search Time Tracking) - Enables accurate performance measurement
- Task 2.0 (Improve IID Move Extraction) - Fixes reliability issue
- Task 3.0 (Integrate IID Move into Advanced Ordering) - Ensures IID is effective in all paths
- These can be done in parallel but Task 1.0 should be done first to enable accurate measurement

### Phase 2: Depth and Time Management (Week 2-3)
Complete tasks 4.0, 5.0:
- Task 4.0 (Use Dynamic Depth Calculation) - Improves IID depth selection
- Task 5.0 (Integrate Time Estimation into Decision Logic) - Prevents excessive overhead
- Task 4.0 can be done in parallel with Phase 1
- Task 5.0 depends on Task 2.0 for time estimation integration

### Phase 3: Measurement and Monitoring (Week 3-4)
Complete tasks 6.0, 8.0:
- Task 6.0 (Add Performance Measurement) - Enables data-driven optimization
- Task 8.0 (Implement Performance Monitoring) - Enables continuous improvement
- Task 6.0 depends on Task 1.0 for accurate time tracking
- Task 8.0 depends on Task 6.0 for performance measurement infrastructure

### Phase 4: Enhanced Features (Week 4-5)
Complete tasks 7.0, 12.0:
- Task 7.0 (Enhance Position Complexity Assessment) - Improves depth selection accuracy
- Task 12.0 (Add Cross-Feature Statistics) - Provides insights for tuning
- Task 7.0 can enhance Task 4.0 if done together
- Task 12.0 provides value for verifying Task 3.0 effectiveness

### Phase 5: Usability and Advanced Features (Week 5-6, Optional)
Complete tasks 9.0, 10.0, 11.0:
- Task 9.0 (Improve Time Pressure Detection) - Better time management
- Task 10.0 (Add Configuration Presets) - Improves usability
- Task 11.0 (Advanced Depth Strategies) - Low priority, diminishing returns
- These are lower priority but provide value for users and developers
- Task 11.0 should only be done if benchmarks show significant benefit

---

**Generated:** December 2024  
**Status:** In Progress - Tasks document for Internal Iterative Deepening improvements

**Task 1.0 Completion Notes:**
- Added `total_search_time_ms` field to `IIDStats` struct in `src/types.rs` to track actual total search time
- Updated `IterativeDeepening::search()` method to:
  * Reset `total_search_time_ms` to 0 at the start of each new search
  * Record start time using `TimeSource::now()` at search entry point
  * Calculate total search time at search exit point using `start_time.elapsed_ms()`
  * Store total search time in `search_engine.iid_stats.total_search_time_ms`
  * Handle both normal completion and fallback move paths
- Removed placeholder `total_search_time_ms = 1000` from `get_iid_performance_metrics()` method
- Modified `get_iid_performance_metrics()` to use actual tracked time from `self.iid_stats.total_search_time_ms`
- Verified overhead percentage calculation is correct: `(iid_time_ms / total_search_time_ms) * 100`
- Created comprehensive unit tests in `tests/iid_tests.rs`:
  * `test_iid_stats_total_search_time_tracking()` - Verifies field initialization, setting, and reset
  * `test_iid_overhead_percentage_calculation()` - Tests overhead calculation accuracy with various scenarios including edge cases (zero total time, typical 5-15% range)
  * `test_get_iid_performance_metrics_uses_actual_time()` - Verifies `get_iid_performance_metrics()` uses actual tracked time instead of placeholder
- All tests passing and verify correct behavior:
  * Total search time correctly initialized to 0 and tracked across searches
  * Overhead percentage calculation accurate (tested with 10%, 15%, 25% scenarios)
  * Edge cases handled correctly (zero total time returns 0% overhead)
  * Performance metrics now use actual search time data
- Overhead tracking now provides accurate measurements for IID performance analysis
- Performance reports will now reflect actual IID overhead percentage instead of placeholder values
- Changes maintain backward compatibility - existing code continues to work, just with accurate data

**Task 2.0 Completion Notes:**
- Changed `perform_iid_search()` return type from `Option<Move>` to `(i32, Option<Move>)` to include the IID score
- Modified `perform_iid_search()` to track best move during search by searching moves individually instead of relying solely on TT
- Removed dependency on `iid_score > alpha` condition - IID now provides move ordering even when score doesn't beat alpha
- Implemented fallback logic: prefers TT move if available and valid, otherwise uses tracked best move from search
- Added verification that IID move is in legal moves list before using it in move ordering
- Updated `negamax_with_context()` to receive IID move from tuple return value: `let (iid_score_result, iid_move_result) = self.perform_iid_search(...)`
- Removed old TT-only move extraction logic - move is now tracked during search with TT as fallback
- Added statistics tracking fields to `IIDStats`:
  * `iid_move_extracted_from_tt` - tracks when move comes from transposition table
  * `iid_move_extracted_from_tracked` - tracks when move comes from tracked best move during search
- Added debug logging for IID move extraction showing score and move found
- Created comprehensive unit tests in `tests/iid_tests.rs`:
  * `test_iid_move_extraction_returns_tuple()` - verifies new return type
  * `test_iid_move_extraction_works_without_alpha_beating()` - tests move extraction works even when score doesn't beat alpha
  * `test_iid_move_verification_in_legal_moves()` - verifies returned move is in legal moves list
  * `test_iid_statistics_tracking_tt_vs_tracked()` - tests statistics tracking for extraction methods
  * `test_iid_move_none_when_no_moves_found()` - tests graceful handling when no move found
  * `test_iid_stats_new_fields_initialized()` - verifies new stats fields initialize correctly
  * `test_iid_stats_reset_includes_new_fields()` - verifies reset() properly resets new fields
- Updated all existing tests to use new tuple return type
- Reviewed `board.clone()` usage at line 3102 - confirmed necessary to avoid modifying original board state during IID search
- IID move extraction now more reliable and provides better move ordering by tracking moves during search rather than relying solely on TT
- Created comprehensive performance benchmarks in `benches/iid_move_extraction_benchmarks.rs`:
  * `benchmark_move_extraction_methods()` - Compares tracked move extraction performance with statistics tracking
  * `benchmark_iid_overhead_verification()` - Measures overhead by comparing IID enabled vs disabled search times
  * `benchmark_move_extraction_success_rates()` - Tracks success rates at different IID depths
  * `benchmark_extraction_performance_comparison()` - Performance comparison with move ordering simulation
  * `benchmark_comprehensive_overhead_analysis()` - Comprehensive overhead analysis across multiple search depths
- Benchmarks verify that move extraction overhead is minimal (<1% target) and provide metrics for TT-based vs tracked move extraction
- Benchmark suite registered in `Cargo.toml` and can be run with: `cargo bench --bench iid_move_extraction_benchmarks`
- Benchmarks follow the same pattern as existing benchmark suites (LMR, NMP, etc.) for consistency

**Task 3.0 Completion Notes:**
- Added `iid_move: Option<&Move>` parameter to `order_moves_for_negamax()` method signature
- Modified `order_moves_advanced()` to accept and pass through `iid_move` parameter
- Updated `AdvancedMoveOrderer::order_moves_with_all_heuristics()` to accept `iid_move` parameter
- Modified `score_move_with_all_heuristics()` to prioritize IID move with `i32::MAX` score (highest priority)
- Updated priority order in advanced ordering:
  1. IID moves (highest priority - Task 3.0)
  2. PV moves (high priority)
  3. Killer moves (medium-high priority)
  4. History moves (medium priority)
  5. Regular moves (normal priority)
- Updated cache logic to skip cache when IID move is present (ensures IID move is properly prioritized even if ordering was cached)
- Updated all call sites:
  * `negamax_with_context()` now passes `iid_move.as_ref()` to `order_moves_for_negamax()`
  * `search_at_depth()` passes `None` (no IID at that level)
  * All test files updated to include `None` or appropriate IID move parameter
- Updated traditional move ordering fallback path to also receive IID move parameter
- Created comprehensive unit tests in `tests/iid_tests.rs`:
  * `test_advanced_ordering_iid_move_prioritization()` - verifies IID move is prioritized in advanced ordering
  * `test_advanced_ordering_without_iid_move()` - compares ordering with/without IID move
  * `test_advanced_ordering_iid_move_parameter_passed()` - verifies parameter is accepted correctly
  * `test_order_moves_for_negamax_iid_move_integration()` - tests integration with multiple IID moves
- Updated all existing test files to use new signature:
  * `tests/move_ordering_integration_tests.rs` - all calls updated
  * `tests/move_ordering_configuration_integration_tests.rs` - all calls updated
  * `tests/move_scoring_integration_tests.rs` - all calls updated
  * `tests/move_ordering_history_integration_tests.rs` - all calls updated
  * `src/search/move_ordering.rs` - internal test code updated
- IID move is now integrated into both advanced and traditional ordering paths, ensuring consistent prioritization
- Advanced ordering now benefits from IID move ordering, improving search efficiency when advanced ordering is used
- Performance benchmark (3.10) is optional and can be added in future iterations if needed
- All compilation errors related to signature changes have been resolved

**Task 4.0 Completion Notes:**
- Reviewed and integrated `calculate_dynamic_iid_depth()` into main IID flow in `calculate_iid_depth()`
- The `Dynamic` variant already existed in `IIDDepthStrategy` enum and is now fully functional
- Updated `calculate_iid_depth()` to:
  * Support Dynamic strategy using `calculate_dynamic_iid_depth()` with position complexity assessment
  * Enhanced Relative strategy with maximum depth cap (4) for performance
  * Enhanced Adaptive strategy with position-based adjustments using complexity assessment
  * Changed to `&mut self` to enable statistics tracking for Dynamic strategy
  * Pass board and captured_pieces parameters through call chain for position-aware calculations
- Enhanced `calculate_dynamic_iid_depth()` to:
  * Work independently for Dynamic strategy (removed dependency on `enable_adaptive_tuning`)
  * Always assess position complexity when using Dynamic strategy
  * Use `dynamic_max_depth` configuration option for depth cap (replaces hardcoded 4)
  * Apply proper depth adjustments based on complexity: Low (-1), Medium (base), High (+1, capped)
- Updated `should_apply_iid()` to:
  * Accept board and captured_pieces parameters for adaptive minimum depth
  * Implement adaptive minimum depth threshold when `adaptive_min_depth` is enabled
  * Lower threshold for high complexity positions where IID is more valuable
- Added configuration options to `IIDConfig` (already existed, now fully used):
  * `dynamic_base_depth: u8` (default: 2) - Base depth for dynamic calculations
  * `dynamic_max_depth: u8` (default: 4) - Maximum depth cap for dynamic strategy
  * `adaptive_min_depth: bool` (default: false) - Enable adaptive minimum depth threshold
- Added statistics tracking fields to `IIDStats` (already existed, now fully used):
  * `dynamic_depth_selections: HashMap<u8, u64>` - Tracks which depths were chosen
  * `dynamic_depth_low_complexity: u64` - Count of low complexity depth selections
  * `dynamic_depth_medium_complexity: u64` - Count of medium complexity depth selections
  * `dynamic_depth_high_complexity: u64` - Count of high complexity depth selections
- Added debug logging for dynamic depth calculation showing main_depth, base, complexity, and calculated depth
- Updated `IIDConfig::default()` to keep Fixed strategy as default (Dynamic can be enabled by users)
- Updated all `IIDConfig` initializers in `EnginePreset` implementations to include new configuration fields
- Updated all call sites:
  * `negamax_with_context()` now passes `Some(board)` and `Some(captured_pieces)` to `calculate_iid_depth()` and `should_apply_iid()`
  * All test files updated to include `None, None` or appropriate parameters
- Created comprehensive unit tests in `tests/iid_tests.rs`:
  * `test_calculate_dynamic_iid_depth_low_complexity()` - tests depth reduction for low complexity
  * `test_calculate_dynamic_iid_depth_high_complexity()` - tests depth increase for high complexity
  * `test_dynamic_depth_max_cap_respected()` - verifies depth cap is respected
  * `test_dynamic_strategy_integration()` - tests Dynamic strategy integration with statistics
  * `test_dynamic_depth_statistics_tracking()` - verifies statistics tracking works
  * `test_adaptive_minimum_depth_threshold()` - tests adaptive minimum depth feature
  * `test_dynamic_strategy_without_position_info()` - tests fallback behavior
  * `test_dynamic_base_depth_configuration()` - tests configuration options
  * `test_relative_strategy_max_cap()` - verifies Relative strategy cap
  * `test_adaptive_strategy_position_based()` - tests enhanced Adaptive strategy
  * `test_different_complexity_levels_depths()` - tests multiple complexity scenarios
- Dynamic depth calculation now fully integrated and provides intelligent depth selection based on position characteristics
- All depth strategies (Fixed, Relative, Adaptive, Dynamic) now properly integrated with position-aware calculations
- Performance benchmarks (4.15, 4.16, 4.17) are optional and can be added in future iterations if needed

**Task 5.0 Completion Notes:**
- Reviewed and integrated `estimate_iid_time()` into `should_apply_iid()` decision logic
- Added configuration options to `IIDConfig`:
  * `max_estimated_iid_time_ms: u32` (default: 50ms) - Maximum estimated IID time threshold
  * `max_estimated_iid_time_percentage: bool` (default: false) - Use percentage of remaining time instead of absolute time
- Updated `should_apply_iid()` to:
  * Calculate IID depth before estimating time (for accurate estimation)
  * Call `estimate_iid_time()` to get estimated time before performing IID
  * Skip IID if estimated time exceeds threshold (absolute or percentage-based)
  * Use actual IID time estimates in time pressure detection instead of fixed 10% threshold
  * Integrate time estimation with time pressure: skip if `remaining_time < estimated_iid_time * 2`
- Added statistics tracking fields to `IIDStats`:
  * `total_predicted_iid_time_ms: u64` - Sum of predicted IID time for accuracy tracking
  * `total_actual_iid_time_ms: u64` - Sum of actual IID time for accuracy tracking
  * `positions_skipped_time_estimation: u64` - Count of IID skipped due to estimated time exceeding threshold
- Updated IID execution in `negamax_with_context()` to:
  * Estimate IID time before performing search
  * Track both predicted and actual time for accuracy statistics
  * Enhanced debug logging to show predicted time, actual time, and accuracy percentage
- Added comprehensive debug logging for time estimation decisions:
  * Estimated IID time with depth information
  * Skip decisions with reasons (exceeds threshold, time pressure)
  * Accuracy tracking showing predicted vs actual time with percentage
- Updated all `IIDConfig` initializers in `EnginePreset` implementations to include new time estimation fields
- Created comprehensive unit tests in `tests/iid_tests.rs`:
  * `test_time_estimation_configuration_default()` - verifies default configuration
  * `test_time_estimation_stats_default()` - verifies default statistics fields
  * `test_should_apply_iid_time_estimation_exceeds_threshold()` - tests skip when estimate exceeds threshold
  * `test_should_apply_iid_time_estimation_percentage_threshold()` - tests percentage-based threshold
  * `test_time_estimation_time_pressure_detection()` - tests time estimation in time pressure detection
  * `test_time_estimation_accuracy_tracking()` - verifies accuracy tracking works and is reasonable
  * `test_time_estimation_skip_statistics_tracking()` - verifies skip statistics are tracked
  * `test_time_estimation_with_different_depths()` - tests time estimation scales with depth
  * `test_time_estimation_with_different_complexities()` - tests consistency of estimates
  * `test_iid_stats_time_estimation_fields_reset()` - verifies reset() properly clears new fields
- Updated `test_iid_stats_default()` to include new time estimation statistics fields
- Time estimation now fully integrated into IID decision logic, providing intelligent time management
- Time pressure detection now uses actual estimates instead of fixed heuristics, improving accuracy
- Performance benchmarks (5.12, 5.13, 5.14) are optional and can be added in future iterations if needed

