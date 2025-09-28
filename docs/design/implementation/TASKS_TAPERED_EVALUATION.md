# Tasks: Tapered Evaluation System Implementation

## Relevant Files

- `src/types.rs` - Contains core types and will need TaperedScore struct and game phase constants
- `src/evaluation.rs` - Main evaluation system that needs complete refactoring for tapered evaluation
- `src/search.rs` - Search engine that calls evaluation functions, may need updates for new evaluation interface
- `src/bitboards.rs` - Board representation, may need side_to_move() method for evaluation
- `tests/tapered_evaluation_tests.rs` - New comprehensive test file for tapered evaluation functionality
- `tests/evaluation_tests.rs` - New unit tests for individual evaluation components
- `tests/performance_tests.rs` - Performance benchmarks for tapered evaluation system

### Notes

- Unit tests should be placed alongside the code files they are testing (e.g., `evaluation.rs` and `evaluation_tests.rs` in the same directory)
- Use `cargo test [optional/path/to/test/file]` to run tests. Running without a path executes all tests found by the Cargo configuration
- The implementation will require careful refactoring to maintain backward compatibility during the transition

## Tasks

- [x] 1.0 Implement Core TaperedScore Infrastructure
  - [x] 1.1 Add TaperedScore struct to `src/types.rs` with mg and eg fields (both i32)
  - [x] 1.2 Implement TaperedScore constructor methods: `new(value)`, `new_tapered(mg, eg)`, and `default()`
  - [x] 1.3 Add interpolation method `interpolate(phase)` that blends mg and eg based on game phase
  - [x] 1.4 Implement arithmetic operators: `Add`, `Sub`, `AddAssign`, and `Neg` for TaperedScore
  - [x] 1.5 Add game phase constants: `GAME_PHASE_MAX = 256` and `PIECE_PHASE_VALUES` array
  - [x] 1.6 Create unit tests for TaperedScore operations and interpolation accuracy
  - [x] 1.7 Add debug formatting and serialization support for TaperedScore

- [x] 2.0 Add Game Phase Calculation System
  - [x] 2.1 Implement `calculate_game_phase()` method in PositionEvaluator that counts pieces on board
  - [x] 2.2 Add `get_piece_phase_value()` helper method to map piece types to phase values
  - [x] 2.3 Create phase value mapping: Knight=1, Silver=1, Gold=2, Bishop=2, Rook=3, Lance=1
  - [x] 2.4 Scale phase calculation to 0-256 range (0=endgame, 256=opening)
  - [x] 2.5 Add unit tests for game phase calculation with different board positions
  - [x] 2.6 Test phase calculation with starting position (should be max phase)
  - [x] 2.7 Test phase calculation with endgame positions (should be low phase)
  - [x] 2.8 Add performance test to ensure phase calculation is O(1) complexity

- [x] 3.0 Refactor Piece-Square Tables for Dual-Phase Evaluation
  - [x] 3.1 Create separate mg and eg tables for each piece type in PieceSquareTables struct
  - [x] 3.2 Add new table fields: `pawn_table_mg`, `pawn_table_eg`, `lance_table_mg`, `lance_table_eg`, etc.
  - [x] 3.3 Update `new()` constructor to initialize both mg and eg tables
  - [x] 3.4 Modify `get_value()` method to return TaperedScore instead of i32
  - [x] 3.5 Add `get_tables()` helper method to return both mg and eg table references
  - [x] 3.6 Create initialization functions for all mg tables (copy existing values initially)
  - [x] 3.7 Create initialization functions for all eg tables with endgame-optimized values
  - [x] 3.8 Update table coordinate calculation to work with both mg and eg tables
  - [x] 3.9 Add unit tests for dual-phase table lookups and value retrieval

- [x] 4.0 Update Evaluation Components to Return TaperedScore
  - [x] 4.1 Refactor `evaluate_material_and_position()` to return TaperedScore
  - [x] 4.2 Update material evaluation to use TaperedScore::new() for constant values
  - [x] 4.3 Update positional evaluation to use new dual-phase piece-square tables
  - [x] 4.4 Refactor `evaluate_pawn_structure()` to return TaperedScore with phase-dependent weights
  - [x] 4.5 Make pawn advancement more valuable in endgame (eg weight higher than mg)
  - [x] 4.6 Make pawn chains more valuable in endgame than middlegame
  - [x] 4.7 Refactor `evaluate_king_safety()` to return TaperedScore with phase-dependent weights
  - [x] 4.8 Make king safety more important in middlegame (mg weight higher than eg)
  - [x] 4.9 Refactor `evaluate_mobility()` to return TaperedScore
  - [x] 4.10 Refactor `evaluate_piece_coordination()` to return TaperedScore
  - [x] 4.11 Refactor `evaluate_center_control()` to return TaperedScore
  - [x] 4.12 Refactor `evaluate_development()` to return TaperedScore
  - [x] 4.13 Add unit tests for each evaluation component's phase-dependent behavior

- [x] 5.0 Implement Main Evaluation Function with Phase Interpolation
  - [x] 5.1 Update main `evaluate()` function to calculate game phase first
  - [x] 5.2 Change function to accumulate TaperedScore instead of i32
  - [x] 5.3 Add tempo bonus as TaperedScore::new(10) for consistency
  - [x] 5.4 Implement final score interpolation using `total_score.interpolate(game_phase)`
  - [x] 5.5 Update player perspective logic to work with interpolated final score
  - [x] 5.6 Add debug logging for game phase, mg score, eg score, and final score
  - [x] 5.7 Ensure backward compatibility by maintaining same function signature
  - [x] 5.8 Add comprehensive integration tests for complete evaluation pipeline
  - [x] 5.9 Test evaluation consistency across multiple calls with same position
  - [x] 5.10 Test evaluation symmetry (Black vs White should return opposite scores)

- [x] 6.0 Create Comprehensive Test Suite
  - [x] 6.1 Create `tests/tapered_evaluation_tests.rs` with comprehensive test coverage
  - [x] 6.2 Add tests for game phase calculation with various board positions
  - [x] 6.3 Add tests for TaperedScore interpolation at different phase values
  - [x] 6.4 Add tests for phase-dependent evaluation behavior (king safety, pawn advancement)
  - [x] 6.5 Add tests for evaluation consistency and symmetry
  - [x] 6.6 Add tests for piece-square table dual-phase lookups
  - [x] 6.7 Create `tests/evaluation_tests.rs` for individual component testing
  - [x] 6.8 Add unit tests for each evaluation function's TaperedScore return values
  - [x] 6.9 Add edge case tests (empty board, single piece positions, etc.)
  - [x] 6.10 Add regression tests to ensure no performance degradation

- [x] 7.0 Performance Optimization and Validation
  - [x] 7.1 Create `tests/performance_tests.rs` for benchmarking tapered evaluation
  - [x] 7.2 Add performance benchmarks comparing old vs new evaluation system
  - [x] 7.3 Measure memory usage impact of dual-phase tables
  - [x] 7.4 Optimize game phase calculation to be called once per search node
  - [x] 7.5 Add performance tests for 1000+ evaluation calls to ensure reasonable speed
  - [x] 7.6 Validate that evaluation performance meets search engine requirements
  - [x] 7.7 Add memory usage validation tests
  - [x] 7.8 Create performance regression tests to prevent future slowdowns
  - [x] 7.9 Document performance characteristics and optimization strategies
  - [x] 7.10 Add configuration options for enabling/disabling tapered evaluation
