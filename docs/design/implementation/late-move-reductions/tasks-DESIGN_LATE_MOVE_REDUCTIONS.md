## Relevant Files

- `src/types.rs` - Contains LMRConfig and LMRStats structures, following existing config patterns like QuiescenceConfig and NullMoveConfig.
- `src/search.rs` - Main search engine implementation where LMR logic will be integrated into the negamax function.
- `tests/lmr_tests.rs` - Unit tests for LMR functionality, following existing test patterns in the tests/ directory.
- `tests/lmr_integration_tests.rs` - Integration tests for LMR with existing search features.
- `tests/lmr_performance_tests.rs` - Performance benchmarks and regression tests for LMR.

### Notes

- Unit tests should be placed in the `tests/` directory following the existing pattern (e.g., `null_move_tests.rs`, `quiescence_tests.rs`).
- Use `cargo test [test_name]` to run specific tests. Running `cargo test` executes all tests.
- Configuration structures follow the established pattern with validation, default implementations, and summary methods.

## Tasks

- [x] 1.0 Create LMR Configuration and Statistics Structures
  - [x] 1.1 Add LMRConfig struct to types.rs with all configuration parameters (enabled, min_depth, min_move_index, base_reduction, max_reduction, enable_dynamic_reduction, enable_adaptive_reduction, enable_extended_exemptions)
  - [x] 1.2 Implement Default trait for LMRConfig with conservative default values
  - [x] 1.3 Add validate() method to LMRConfig following existing validation patterns from QuiescenceConfig and NullMoveConfig
  - [x] 1.4 Add new_validated() method for automatic parameter clamping
  - [x] 1.5 Add summary() method for configuration reporting
  - [x] 1.6 Add LMRStats struct with all performance tracking fields (moves_considered, reductions_applied, researches_triggered, cutoffs_after_reduction, cutoffs_after_research, total_depth_saved, average_reduction)
  - [x] 1.7 Implement Default trait for LMRStats
  - [x] 1.8 Add helper methods to LMRStats (research_rate, efficiency, reset)
  - [x] 1.9 Add Serialize and Deserialize derives to LMRConfig for configuration persistence

- [x] 2.0 Integrate LMR into SearchEngine
  - [x] 2.1 Add lmr_config: LMRConfig field to SearchEngine struct
  - [x] 2.2 Add lmr_stats: LMRStats field to SearchEngine struct
  - [x] 2.3 Update SearchEngine::new() to initialize LMR fields with default values
  - [x] 2.4 Update SearchEngine::new_with_config() to accept LMR configuration
  - [x] 2.5 Add update_lmr_config() method to SearchEngine with validation
  - [x] 2.6 Add get_lmr_config() method to SearchEngine
  - [x] 2.7 Add get_lmr_stats() method to SearchEngine
  - [x] 2.8 Add reset_lmr_stats() method to SearchEngine
  - [x] 2.9 Update SearchEngine::clear() to reset LMR statistics

- [x] 3.0 Implement Core LMR Logic
  - [x] 3.1 Modify negamax move loop to track move_index and call search_move_with_lmr()
  - [x] 3.2 Implement search_move_with_lmr() method with full LMR decision logic
  - [x] 3.3 Implement should_apply_lmr() method with depth, move index, and exemption checks
  - [x] 3.4 Implement is_move_exempt_from_lmr() method with basic and extended exemption rules
  - [x] 3.5 Implement calculate_reduction() method with static and dynamic reduction logic
  - [x] 3.6 Implement apply_adaptive_reduction() method for position-based adjustments
  - [x] 3.7 Add proper statistics tracking throughout LMR decision process
  - [x] 3.8 Ensure LMR works correctly with existing alpha-beta pruning
  - [x] 3.9 Add debug logging for LMR decisions (optional, behind debug flag)

- [x] 4.0 Add LMR Helper Methods
  - [x] 4.1 Implement is_killer_move() method to check against killer move table
  - [x] 4.2 Implement is_transposition_table_move() method (placeholder for TT best move tracking)
  - [x] 4.3 Implement is_escape_move() method (placeholder for threat detection)
  - [x] 4.4 Implement is_tactical_position() method for position characteristic detection
  - [x] 4.5 Implement is_quiet_position() method for position characteristic detection
  - [x] 4.6 Implement is_center_move() method to check if move targets center squares
  - [x] 4.7 Add position evaluation helpers for adaptive reduction decisions
  - [x] 4.8 Add move classification helpers for exemption rule decisions
  - [x] 4.9 Add performance monitoring helpers for LMR effectiveness

- [x] 5.0 Create Comprehensive Test Suite
  - [x] 5.1 Create tests/lmr_tests.rs with unit tests for LMRConfig validation
  - [x] 5.2 Add unit tests for LMRStats calculation methods
  - [x] 5.3 Add unit tests for move exemption rules (captures, promotions, checks, killer moves)
  - [x] 5.4 Add unit tests for reduction calculation (static and dynamic)
  - [x] 5.5 Add unit tests for adaptive reduction logic
  - [x] 5.6 Create tests/lmr_integration_tests.rs with integration tests
  - [x] 5.7 Add integration tests for LMR with null move pruning
  - [x] 5.8 Add integration tests for LMR with quiescence search
  - [x] 5.9 Add integration tests for LMR with transposition table
  - [x] 5.10 Add integration tests for LMR re-search behavior
  - [x] 5.11 Create tests/lmr_performance_tests.rs with performance benchmarks
  - [x] 5.12 Add performance tests comparing NPS with/without LMR
  - [x] 5.13 Add performance tests for different position types (tactical vs quiet)
  - [x] 5.14 Add regression tests to ensure LMR doesn't weaken tactical play
  - [x] 5.15 Add stress tests with high move counts and deep searches

- [x] 6.0 Performance Optimization and Tuning
  - [x] 6.1 Profile LMR overhead and optimize hot paths
  - [x] 6.2 Tune default LMR parameters based on performance testing
  - [x] 6.3 Optimize move exemption checks for minimal overhead
  - [x] 6.4 Fine-tune reduction calculation formulas
  - [x] 6.5 Add adaptive parameter adjustment based on position characteristics
  - [x] 6.6 Implement LMR effectiveness monitoring and auto-tuning
  - [x] 6.7 Add performance metrics reporting for LMR statistics
  - [x] 6.8 Optimize memory usage for LMR configuration and statistics
  - [x] 6.9 Add configuration presets for different playing styles (aggressive, conservative, balanced)
  - [x] 6.10 Validate LMR performance across different hardware configurations
