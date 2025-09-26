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

- [ ] 1.0 Implement Core TaperedScore Infrastructure
  - [ ] 1.1 Add TaperedScore struct to `src/types.rs` with mg and eg fields (both i32)
  - [ ] 1.2 Implement TaperedScore constructor methods: `new(value)`, `new_tapered(mg, eg)`, and `default()`
  - [ ] 1.3 Add interpolation method `interpolate(phase)` that blends mg and eg based on game phase
  - [ ] 1.4 Implement arithmetic operators: `Add`, `Sub`, `AddAssign`, and `Neg` for TaperedScore
  - [ ] 1.5 Add game phase constants: `GAME_PHASE_MAX = 256` and `PIECE_PHASE_VALUES` array
  - [ ] 1.6 Create unit tests for TaperedScore operations and interpolation accuracy
  - [ ] 1.7 Add debug formatting and serialization support for TaperedScore

- [ ] 2.0 Add Game Phase Calculation System
  - [ ] 2.1 Implement `calculate_game_phase()` method in PositionEvaluator that counts pieces on board
  - [ ] 2.2 Add `get_piece_phase_value()` helper method to map piece types to phase values
  - [ ] 2.3 Create phase value mapping: Knight=1, Silver=1, Gold=2, Bishop=2, Rook=3, Lance=1
  - [ ] 2.4 Scale phase calculation to 0-256 range (0=endgame, 256=opening)
  - [ ] 2.5 Add unit tests for game phase calculation with different board positions
  - [ ] 2.6 Test phase calculation with starting position (should be max phase)
  - [ ] 2.7 Test phase calculation with endgame positions (should be low phase)
  - [ ] 2.8 Add performance test to ensure phase calculation is O(1) complexity

- [ ] 3.0 Refactor Piece-Square Tables for Dual-Phase Evaluation
  - [ ] 3.1 Create separate mg and eg tables for each piece type in PieceSquareTables struct
  - [ ] 3.2 Add new table fields: `pawn_table_mg`, `pawn_table_eg`, `lance_table_mg`, `lance_table_eg`, etc.
  - [ ] 3.3 Update `new()` constructor to initialize both mg and eg tables
  - [ ] 3.4 Modify `get_value()` method to return TaperedScore instead of i32
  - [ ] 3.5 Add `get_tables()` helper method to return both mg and eg table references
  - [ ] 3.6 Create initialization functions for all mg tables (copy existing values initially)
  - [ ] 3.7 Create initialization functions for all eg tables with endgame-optimized values
  - [ ] 3.8 Update table coordinate calculation to work with both mg and eg tables
  - [ ] 3.9 Add unit tests for dual-phase table lookups and value retrieval

- [ ] 4.0 Update Evaluation Components to Return TaperedScore
  - [ ] 4.1 Refactor `evaluate_material_and_position()` to return TaperedScore
  - [ ] 4.2 Update material evaluation to use TaperedScore::new() for constant values
  - [ ] 4.3 Update positional evaluation to use new dual-phase piece-square tables
  - [ ] 4.4 Refactor `evaluate_pawn_structure()` to return TaperedScore with phase-dependent weights
  - [ ] 4.5 Make pawn advancement more valuable in endgame (eg weight higher than mg)
  - [ ] 4.6 Make pawn chains more valuable in endgame than middlegame
  - [ ] 4.7 Refactor `evaluate_king_safety()` to return TaperedScore with phase-dependent weights
  - [ ] 4.8 Make king safety more important in middlegame (mg weight higher than eg)
  - [ ] 4.9 Refactor `evaluate_mobility()` to return TaperedScore
  - [ ] 4.10 Refactor `evaluate_piece_coordination()` to return TaperedScore
  - [ ] 4.11 Refactor `evaluate_center_control()` to return TaperedScore
  - [ ] 4.12 Refactor `evaluate_development()` to return TaperedScore
  - [ ] 4.13 Add unit tests for each evaluation component's phase-dependent behavior

- [ ] 5.0 Implement Main Evaluation Function with Phase Interpolation
  - [ ] 5.1 Update main `evaluate()` function to calculate game phase first
  - [ ] 5.2 Change function to accumulate TaperedScore instead of i32
  - [ ] 5.3 Add tempo bonus as TaperedScore::new(10) for consistency
  - [ ] 5.4 Implement final score interpolation using `total_score.interpolate(game_phase)`
  - [ ] 5.5 Update player perspective logic to work with interpolated final score
  - [ ] 5.6 Add debug logging for game phase, mg score, eg score, and final score
  - [ ] 5.7 Ensure backward compatibility by maintaining same function signature
  - [ ] 5.8 Add comprehensive integration tests for complete evaluation pipeline
  - [ ] 5.9 Test evaluation consistency across multiple calls with same position
  - [ ] 5.10 Test evaluation symmetry (Black vs White should return opposite scores)

- [ ] 6.0 Create Comprehensive Test Suite
  - [ ] 6.1 Create `tests/tapered_evaluation_tests.rs` with comprehensive test coverage
  - [ ] 6.2 Add tests for game phase calculation with various board positions
  - [ ] 6.3 Add tests for TaperedScore interpolation at different phase values
  - [ ] 6.4 Add tests for phase-dependent evaluation behavior (king safety, pawn advancement)
  - [ ] 6.5 Add tests for evaluation consistency and symmetry
  - [ ] 6.6 Add tests for piece-square table dual-phase lookups
  - [ ] 6.7 Create `tests/evaluation_tests.rs` for individual component testing
  - [ ] 6.8 Add unit tests for each evaluation function's TaperedScore return values
  - [ ] 6.9 Add edge case tests (empty board, single piece positions, etc.)
  - [ ] 6.10 Add regression tests to ensure no performance degradation

- [ ] 7.0 Performance Optimization and Validation
  - [ ] 7.1 Create `tests/performance_tests.rs` for benchmarking tapered evaluation
  - [ ] 7.2 Add performance benchmarks comparing old vs new evaluation system
  - [ ] 7.3 Measure memory usage impact of dual-phase tables
  - [ ] 7.4 Optimize game phase calculation to be called once per search node
  - [ ] 7.5 Add performance tests for 1000+ evaluation calls to ensure reasonable speed
  - [ ] 7.6 Validate that evaluation performance meets search engine requirements
  - [ ] 7.7 Add memory usage validation tests
  - [ ] 7.8 Create performance regression tests to prevent future slowdowns
  - [ ] 7.9 Document performance characteristics and optimization strategies
  - [ ] 7.10 Add configuration options for enabling/disabling tapered evaluation
