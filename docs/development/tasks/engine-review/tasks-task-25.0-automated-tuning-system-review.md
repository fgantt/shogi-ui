# Tasks: Automated Tuning System Improvements

**Parent PRD:** `task-25.0-automated-tuning-system-review.md`  
**Date:** December 2024  
**Status:** In Progress

---

## Overview

This task list implements the improvements identified in the Automated Tuning System Review (Task 25.0). The improvements address critical gaps in configuration fidelity, data processing completeness, and validation realism to bring the tuning system to production readiness.

## Relevant Files

- `src/tuning/optimizer.rs` - Core optimization algorithms (Adam, LBFGS, Genetic Algorithm) requiring configuration fixes and line search implementation
- `src/tuning/types.rs` - Data structures (TuningConfig, OptimizationMethod, ValidationConfig, PerformanceConfig) requiring parameter additions
- `src/tuning/data_processor.rs` - Game database processing requiring move parsing implementation (KIF/CSA/PGN)
- `src/tuning/feature_extractor.rs` - Feature extraction requiring replacement of simplified heuristics with actual move generation
- `src/tuning/validator.rs` - Validation framework requiring stratified sampling and random seed support
- `src/tuning/performance.rs` - Performance monitoring requiring checkpoint path configuration
- `src/tuning/validator.rs` - Strength testing requiring actual game playing implementation
- `tests/tuning/optimizer_tests.rs` - Unit tests for optimizer fixes (to be created/updated)
- `tests/tuning/data_processor_tests.rs` - Unit tests for move parsing (to be created/updated)
- `tests/tuning/validator_tests.rs` - Unit tests for validation improvements (to be created/updated)
- `benches/tuning/optimizer_benchmarks.rs` - Performance benchmarks for optimizer improvements (to be created/updated)

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests go in the `tests/` directory
- Benchmarks go in the `benches/` directory
- Use `cargo test` to run tests, `cargo bench` to run benchmarks
- The tuning system is implemented as a separate module to avoid impacting the main engine's performance

---

## Tasks

- [ ] 1.0 Fix Optimizer Configuration Issues (High Priority - Est: 5-7 hours)
  - [ ] 1.1 Update `AdamState::new()` to accept `beta1`, `beta2`, and `epsilon` parameters instead of hardcoding values
  - [ ] 1.2 Modify `adam_optimize()` method to extract `beta1`, `beta2`, and `epsilon` from `OptimizationMethod::Adam` configuration (remove `_` prefix)
  - [ ] 1.3 Pass configuration parameters to `AdamState::new()` call in `adam_optimize()` method (line ~681)
  - [ ] 1.4 Add unit test `test_adam_configuration_parameters()` to verify Adam optimizer uses custom beta1, beta2, and epsilon values
  - [ ] 1.5 Add unit test `test_adam_default_parameters()` to verify default values work correctly
  - [ ] 1.6 Add integration test verifying Adam optimizer behavior changes with different parameter configurations
  - [ ] 1.7 Update `OptimizationMethod::Adam` documentation to clarify that all parameters are honored
  - [ ] 1.8 Add benchmark comparing Adam performance with different beta1/beta2 configurations

- [ ] 2.0 Implement LBFGS Line Search (High Priority - Est: 6-8 hours)
  - [ ] 2.1 Research and select line search algorithm (Armijo or Wolfe conditions recommended)
  - [ ] 2.2 Add line search configuration to `OptimizationMethod::LBFGS` enum (e.g., `line_search_type`, `initial_step_size`, `max_line_search_iterations`)
  - [ ] 2.3 Implement `LineSearch` struct with `armijo_search()` and/or `wolfe_search()` methods
  - [ ] 2.4 Replace fixed learning rate in `lbfgs_optimize()` with line search call
  - [ ] 2.5 Update LBFGS update logic to use step size returned from line search
  - [ ] 2.6 Add convergence checks for line search (backtracking, step size bounds)
  - [ ] 2.7 Add unit test `test_lbfgs_line_search_armijo()` to verify Armijo condition satisfaction
  - [ ] 2.8 Add unit test `test_lbfgs_line_search_wolfe()` to verify Wolfe condition satisfaction (if implemented)
  - [ ] 2.9 Add integration test comparing LBFGS with line search vs. fixed learning rate
  - [ ] 2.10 Add benchmark measuring LBFGS convergence speed with proper line search
  - [ ] 2.11 Update LBFGS documentation with line search algorithm details

- [ ] 3.0 Implement Game Format Parsing (High Priority - Est: 20-30 hours)
  - [ ] 3.1 Research KIF format specification and identify required parsing components
  - [ ] 3.2 Implement `parse_kif_move()` method in `DataProcessor` to parse KIF move notation (e.g., "7g7f", "+7776FU")
  - [ ] 3.3 Implement `parse_kif_game()` method to parse complete KIF game files with headers and moves
  - [ ] 3.4 Research CSA format specification and identify required parsing components
  - [ ] 3.5 Implement `parse_csa_move()` method to parse CSA move notation (e.g., "+7776FU", "-3334FU")
  - [ ] 3.6 Implement `parse_csa_game()` method to parse complete CSA game files with headers and moves
  - [ ] 3.7 Research PGN format specification for shogi (or adapt chess PGN parser)
  - [ ] 3.8 Implement `parse_pgn_move()` method to parse PGN move notation (e.g., "7g7f", "P*7e")
  - [ ] 3.9 Implement `parse_pgn_game()` method to parse complete PGN game files with headers and moves
  - [ ] 3.10 Update `load_games_from_file()` to detect file format and route to appropriate parser
  - [ ] 3.11 Add error handling for malformed moves and games (return `Result` types)
  - [ ] 3.12 Add unit test `test_kif_move_parsing()` with various KIF move formats
  - [ ] 3.13 Add unit test `test_csa_move_parsing()` with various CSA move formats
  - [ ] 3.14 Add unit test `test_pgn_move_parsing()` with various PGN move formats
  - [ ] 3.15 Add integration test `test_load_kif_game_file()` with real KIF game file
  - [ ] 3.16 Add integration test `test_load_csa_game_file()` with real CSA game file
  - [ ] 3.17 Add integration test `test_load_pgn_game_file()` with real PGN game file
  - [ ] 3.18 Add test for error handling with malformed game files
  - [ ] 3.19 Update `DataProcessor` documentation with supported formats and parsing details
  - [ ] 3.20 Consider integrating existing shogi format parsers if available (e.g., from shogi-rs crates)

- [ ] 4.0 Improve Feature Extraction Quality (Medium Priority - Est: 8-12 hours)
  - [ ] 4.1 Review `FeatureExtractor` implementation to identify mobility and coordination heuristics
  - [ ] 4.2 Add `BitboardBoard` parameter to mobility feature extraction methods
  - [ ] 4.3 Replace mobility heuristic with actual move generation call (e.g., `board.generate_legal_moves().len()`)
  - [ ] 4.4 Update mobility feature calculation to use actual move count per piece type
  - [ ] 4.5 Replace coordination heuristic with actual move generation analysis
  - [ ] 4.6 Implement coordination feature using actual piece interactions (e.g., pieces defending each other)
  - [ ] 4.7 Update `extract_features()` method signature to accept `&BitboardBoard` if not already present
  - [ ] 4.8 Add unit test `test_mobility_feature_accuracy()` comparing heuristic vs. actual move generation
  - [ ] 4.9 Add unit test `test_coordination_feature_accuracy()` comparing heuristic vs. actual analysis
  - [ ] 4.10 Add integration test verifying feature extraction produces consistent results
  - [ ] 4.11 Add benchmark measuring feature extraction performance impact of move generation
  - [ ] 4.12 Update feature extraction documentation with actual implementation details

- [ ] 5.0 Implement Realistic Validation (Medium Priority - Est: 15-20 hours)
  - [ ] 5.1 Review `StrengthTester` implementation to identify simulation logic
  - [ ] 5.2 Design interface for actual game playing (e.g., `GamePlayer` trait with `play_game()` method)
  - [ ] 5.3 Integrate with engine search interface or USI protocol for game playing
  - [ ] 5.4 Implement `play_game()` method that uses actual engine to play games
  - [ ] 5.5 Replace simulation logic in `StrengthTester::test_strength()` with actual game playing
  - [ ] 5.6 Add game result collection (wins, losses, draws) from actual games
  - [ ] 5.7 Add time control configuration for strength testing games
  - [ ] 5.8 Add error handling for engine communication failures
  - [ ] 5.9 Add unit test `test_strength_tester_actual_games()` with mock engine interface
  - [ ] 5.10 Add integration test `test_strength_tester_real_engine()` with actual engine (if available)
  - [ ] 5.11 Add benchmark measuring strength testing time with actual games vs. simulation
  - [ ] 5.12 Update `StrengthTester` documentation with actual game playing details
  - [ ] 5.13 Consider adding parallel game playing for faster strength testing

- [ ] 6.0 Enhance Validation Framework (Medium Priority - Est: 4-6 hours)
  - [ ] 6.1 Review `ValidationConfig` to identify `stratified` and `random_seed` fields
  - [ ] 6.2 Implement stratified sampling logic in `cross_validate()` method
  - [ ] 6.3 Group positions by result (WhiteWin/BlackWin/Draw) for stratification
  - [ ] 6.4 Distribute stratified groups proportionally across k-folds
  - [ ] 6.5 Add `rand::SeedableRng` usage with `random_seed` from configuration
  - [ ] 6.6 Replace `thread_rng()` calls with seeded RNG in `cross_validate()` and `holdout_validate()`
  - [ ] 6.7 Add unit test `test_stratified_sampling()` verifying proportional distribution across folds
  - [ ] 6.8 Add unit test `test_random_seed_reproducibility()` verifying same seed produces same splits
  - [ ] 6.9 Add unit test `test_stratified_with_imbalanced_data()` with heavily imbalanced result distribution
  - [ ] 6.10 Add integration test comparing stratified vs. non-stratified cross-validation results
  - [ ] 6.11 Update validation documentation with stratified sampling and reproducibility details
  - [ ] 6.12 **Time-Series Cross-Validation** (Future Enhancement - Not in explicit recommendations but identified as gap in Section 5.4)
    - [ ] 6.12.1 Research time-series cross-validation approaches for game sequences
    - [ ] 6.12.2 Design time-series validation method that respects game sequence ordering
    - [ ] 6.12.3 Implement time-series cross-validation option in `ValidationConfig`
    - [ ] 6.12.4 Add unit tests for time-series validation with sequential game data

- [ ] 7.0 Make Genetic Algorithm Configurable (Medium Priority - Est: 3-4 hours)
  - [ ] 7.1 Add `tournament_size: usize` field to `OptimizationMethod::GeneticAlgorithm` enum
  - [ ] 7.2 Add `elite_percentage: f64` field to `OptimizationMethod::GeneticAlgorithm` enum
  - [ ] 7.3 Add `mutation_magnitude: f64` field to `OptimizationMethod::GeneticAlgorithm` enum
  - [ ] 7.4 Add `mutation_bounds: (f64, f64)` field to `OptimizationMethod::GeneticAlgorithm` enum
  - [ ] 7.5 Update `GeneticAlgorithmState::new()` to accept tournament_size, elite_percentage, mutation_magnitude, and mutation_bounds
  - [ ] 7.6 Replace hardcoded tournament size (3) in selection logic with configurable value
  - [ ] 7.7 Replace hardcoded elite size calculation (population_size / 10) with configurable percentage
  - [ ] 7.8 Replace hardcoded mutation magnitude (0.2) with configurable value
  - [ ] 7.9 Replace hardcoded mutation bounds (-10 to 10) with configurable bounds
  - [ ] 7.10 Update `genetic_algorithm_optimize()` to extract and pass new parameters
  - [ ] 7.11 Add default values for new parameters in `OptimizationMethod::default()` for GeneticAlgorithm
  - [ ] 7.12 Add unit test `test_genetic_algorithm_tournament_size()` verifying tournament selection respects configuration
  - [ ] 7.13 Add unit test `test_genetic_algorithm_elite_percentage()` verifying elite preservation respects configuration
  - [ ] 7.14 Add unit test `test_genetic_algorithm_mutation_parameters()` verifying mutation respects magnitude and bounds
  - [ ] 7.15 Update genetic algorithm documentation with configurable parameters

- [ ] 8.0 Add Checkpoint Path Configuration (Medium Priority - Est: 1-2 hours)
  - [ ] 8.1 Add `checkpoint_path: Option<String>` field to `PerformanceConfig` struct
  - [ ] 8.2 Update `PerformanceConfig::default()` to set default checkpoint path (e.g., "checkpoints/")
  - [ ] 8.3 Update `TuningConfig::default()` to use `performance_config.checkpoint_path` instead of hardcoded value
  - [ ] 8.4 Replace hardcoded "checkpoints/" path in `performance.rs` with `PerformanceConfig.checkpoint_path`
  - [ ] 8.5 Add path validation (create directory if it doesn't exist) in checkpoint save logic
  - [ ] 8.6 Update checkpoint load logic to use configured path
  - [ ] 8.7 Add unit test `test_checkpoint_path_configuration()` verifying custom path is used
  - [ ] 8.8 Add unit test `test_checkpoint_path_creation()` verifying directory is created if missing
  - [ ] 8.9 Update checkpoint documentation with path configuration details

- [ ] 9.0 Advanced Tuning Features (Low Priority - Est: 40-52 hours)
  - [ ] 9.1 **Weight Warm-Starting** (Est: 4-6 hours)
    - [ ] 9.1.1 Add `initial_weights_path: Option<String>` field to `TuningConfig`
    - [ ] 9.1.2 Implement `load_initial_weights()` method to deserialize weights from JSON file
    - [ ] 9.1.3 Update optimizer methods to accept and use initial weights if provided
    - [ ] 9.1.4 Add unit test verifying warm-starting loads weights correctly
    - [ ] 9.1.5 Add integration test comparing warm-started vs. random initialization
  - [ ] 9.2 **Constraint Handling** (Est: 8-10 hours)
    - [ ] 9.2.1 Design constraint system (e.g., `WeightConstraint` enum with `Bounds`, `GroupSum`, `Ratio` variants)
    - [ ] 9.2.2 Add `constraints: Vec<WeightConstraint>` field to `TuningConfig`
    - [ ] 9.2.3 Implement constraint projection in optimizer update steps
    - [ ] 9.2.4 Add constraint violation detection and reporting
    - [ ] 9.2.5 Add unit tests for each constraint type
    - [ ] 9.2.6 Add integration test with multiple constraint types
  - [ ] 9.3 **Multi-Objective Optimization** (Est: 12-16 hours)
    - [ ] 9.3.1 Design multi-objective framework (e.g., `Objective` enum with `Accuracy`, `Speed`, `Stability` variants)
    - [ ] 9.3.2 Implement Pareto-optimal solution tracking
    - [ ] 9.3.3 Add `objectives: Vec<Objective>` field to `TuningConfig`
    - [ ] 9.3.4 Modify optimizer to track multiple objectives and Pareto front
    - [ ] 9.3.5 Implement solution selection from Pareto front (e.g., weighted sum, epsilon-constraint)
    - [ ] 9.3.6 Add unit tests for Pareto-optimal solution identification
    - [ ] 9.3.7 Add integration test with multiple objectives
  - [ ] 9.4 **Online/Incremental Learning** (Est: 15-20 hours)
    - [ ] 9.4.1 Design incremental learning interface (e.g., `IncrementalOptimizer` trait)
    - [ ] 9.4.2 Implement incremental weight update methods for each optimizer
    - [ ] 9.4.3 Add `enable_incremental: bool` and `batch_size: usize` fields to `TuningConfig`
    - [ ] 9.4.4 Implement streaming data processing for new game positions
    - [ ] 9.4.5 Add checkpoint/resume support for incremental learning state
    - [ ] 9.4.6 Add unit tests for incremental weight updates
    - [ ] 9.4.7 Add integration test with streaming game data

---

**Phase 2 Complete - Detailed Sub-Tasks Generated**

All parent tasks have been broken down into **actionable sub-tasks**. Each sub-task is specific, testable, and includes:
- Implementation details based on the review analysis
- Testing requirements (unit tests, integration tests, benchmarks)
- Documentation updates where applicable
- Cross-references to specific sections in the review document

**Coverage Verification:**

✅ **Section 8 (Improvement Recommendations):**
- High Priority: Adam configuration fix → Task 1.0
- High Priority: LBFGS line search → Task 2.0
- High Priority: Game format parsing → Task 3.0
- Medium Priority: Feature extraction quality → Task 4.0
- Medium Priority: Realistic validation → Task 5.0
- Medium Priority: Stratified sampling and random seed → Task 6.0
- Medium Priority: Genetic algorithm configurability → Task 7.0
- Medium Priority: Checkpoint path configuration → Task 8.0
- Low Priority: Weight warm-starting → Task 9.1
- Low Priority: Constraint handling → Task 9.2
- Low Priority: Multi-objective optimization → Task 9.3
- Low Priority: Online/incremental learning → Task 9.4

✅ **Section 5.4 (Additional Gaps Identified):**
- Stratified sampling not implemented → Task 6.0 (covered)
- Random seed not applied → Task 6.0 (covered)
- Time-series cross-validation not supported → Task 6.12 (added as future enhancement)

✅ **Section 7 (Weaknesses):**
- All weaknesses from Section 7 are addressed in the corresponding tasks above

**Task Priorities:**
- **Phase 1 (High Priority, 1-2 weeks):** Tasks 1.0, 2.0, 3.0 - Critical configuration and data processing fixes
- **Phase 2 (Medium Priority, 4-6 weeks):** Tasks 4.0, 5.0, 6.0, 7.0, 8.0 - Quality and usability improvements
- **Phase 3 (Low Priority, 3-6 months):** Task 9.0 - Advanced features for future enhancement

**Expected Cumulative Benefits:**
- **Configuration Fidelity:** 100% API contract compliance (Adam parameters honored)
- **Data Processing:** Real-world dataset support via complete format parsing
- **Validation Quality:** Realistic game playing and reproducible experiments
- **Feature Quality:** Accurate features via actual move generation
- **Production Readiness:** All critical gaps addressed for deployment

