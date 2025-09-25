# Task List: Null Move Pruning Implementation

## Relevant Files

- `src/types.rs` - Contains type definitions and will need NullMoveConfig and NullMoveStats structures added
- `src/search.rs` - Main search engine implementation, needs NMP integration in negamax function and SearchEngine struct
- `tests/null_move_tests.rs` - Unit tests for null move pruning functionality (to be created)
- `tests/performance_benchmarks.rs` - Performance benchmarks for NMP (to be extended)
- `tests/comprehensive_tests.rs` - Integration tests for NMP with existing search infrastructure (to be extended)

### Notes

- Unit tests should be placed alongside the code files they are testing
- Use `cargo test` to run all tests, or `cargo test null_move` to run NMP-specific tests
- Performance benchmarks will be added to existing benchmark files
- Configuration validation follows the same patterns as existing QuiescenceConfig

## Tasks

- [ ] 1.0 Add Null Move Pruning Data Structures
  - [ ] 1.1 Add NullMoveConfig struct to src/types.rs with fields: enabled, min_depth, reduction_factor, max_pieces_threshold, enable_dynamic_reduction, enable_endgame_detection
  - [ ] 1.2 Add NullMoveStats struct to src/types.rs with fields: attempts, cutoffs, depth_reductions, disabled_in_check, disabled_endgame
  - [ ] 1.3 Implement Default trait for NullMoveStats with all fields initialized to 0
  - [ ] 1.4 Add Debug, Clone, Serialize, Deserialize derives to both structs following existing patterns
  - [ ] 1.5 Add validation methods to NullMoveConfig following QuiescenceConfig patterns
- [ ] 2.0 Implement Core Null Move Pruning Logic
  - [ ] 2.1 Add should_attempt_null_move method to SearchEngine with checks for enabled, depth, check status, and endgame detection
  - [ ] 2.2 Implement count_pieces_on_board method to count pieces on the board for endgame detection
  - [ ] 2.3 Add perform_null_move_search method that calculates reduction factor and performs zero-width window search
  - [ ] 2.4 Implement dynamic reduction factor calculation (R = 2 + depth/6) with fallback to static factor
  - [ ] 2.5 Add null move pruning logic to negamax function after transposition table lookup but before move generation
- [ ] 3.0 Integrate NMP with Search Engine Architecture
  - [ ] 3.1 Add null_move_config and null_move_stats fields to SearchEngine struct
  - [ ] 3.2 Update SearchEngine::new and SearchEngine::new_with_config constructors to initialize NMP fields
  - [ ] 3.3 Modify negamax function signature to accept can_null_move: bool parameter
  - [ ] 3.4 Update all negamax call sites in search_at_depth to pass can_null_move: true
  - [ ] 3.5 Update all recursive negamax calls to pass can_null_move: true for regular moves, false for null moves
  - [ ] 3.6 Ensure NMP is not used in quiescence search by maintaining existing quiescence logic
- [ ] 4.0 Add Configuration Management and Validation
  - [ ] 4.1 Implement NullMoveConfig::validate method with checks for min_depth > 0, reduction_factor > 0, max_pieces_threshold > 0
  - [ ] 4.2 Add NullMoveConfig::new_validated method that clamps values to safe ranges
  - [ ] 4.3 Add SearchEngine::new_null_move_config method that returns default configuration
  - [ ] 4.4 Implement SearchEngine::update_null_move_config method with validation and error handling
  - [ ] 4.5 Add SearchEngine::get_null_move_config method to retrieve current configuration
  - [ ] 4.6 Add SearchEngine::reset_null_move_stats method to clear statistics
- [ ] 5.0 Implement Statistics Collection and Monitoring
  - [ ] 5.1 Add statistics tracking in should_attempt_null_move for disabled_in_check and disabled_endgame counters
  - [ ] 5.2 Add statistics tracking in perform_null_move_search for attempts, depth_reductions, and cutoffs
  - [ ] 5.3 Implement SearchEngine::get_null_move_stats method to return reference to statistics
  - [ ] 5.4 Add debug logging support with log_null_move_attempt method for debugging NMP behavior
  - [ ] 5.5 Add performance monitoring methods to calculate cutoff rates and average reduction factors
  - [ ] 5.6 Implement statistics reset functionality for clean testing and benchmarking
- [ ] 6.0 Create Comprehensive Test Suite
  - [ ] 6.1 Create tests/null_move_tests.rs with basic NMP functionality tests
  - [ ] 6.2 Add test_null_move_basic_functionality test that verifies NMP can perform null move searches
  - [ ] 6.3 Add test_null_move_disabled_in_check test that ensures NMP is disabled when in check
  - [ ] 6.4 Add test_null_move_endgame_detection test that verifies endgame detection logic
  - [ ] 6.5 Add test_null_move_configuration_validation test for config validation methods
  - [ ] 6.6 Add test_null_move_statistics_tracking test to verify statistics collection
  - [ ] 6.7 Add test_null_move_integration_with_negamax test for full integration testing
  - [ ] 6.8 Add test_null_move_performance_improvement test to verify speed gains
- [ ] 7.0 Add Performance Benchmarks and Validation
  - [ ] 7.1 Extend tests/performance_benchmarks.rs with NMP benchmark methods
  - [ ] 7.2 Add benchmark_null_move_performance method to SearchEngine for comparing with/without NMP
  - [ ] 7.3 Implement benchmark_null_move_nodes_per_second test to measure search speed improvement
  - [ ] 7.4 Add benchmark_null_move_depth_improvement test to measure depth gains for same time
  - [ ] 7.5 Create benchmark_null_move_memory_usage test to ensure minimal memory overhead
  - [ ] 7.6 Add benchmark_null_move_playing_strength test to verify tactical accuracy is maintained
- [ ] 8.0 Implement Safety Mechanisms and Risk Mitigation
  - [ ] 8.1 Add is_safe_for_null_move method with additional safety checks beyond basic conditions
  - [ ] 8.2 Implement is_late_endgame method to detect late endgame positions where zugzwang is common
  - [ ] 8.3 Add count_major_pieces method to help with endgame detection
  - [ ] 8.4 Implement conservative default configuration to minimize false pruning risk
  - [ ] 8.5 Add fallback mechanism to disable NMP if issues are detected during search
  - [ ] 8.6 Create test suite for zugzwang positions to ensure NMP doesn't prune critical lines
  - [ ] 8.7 Add validation tests for tactical positions to ensure no important sequences are missed
