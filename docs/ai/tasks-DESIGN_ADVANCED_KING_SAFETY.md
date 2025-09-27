## Relevant Files

- `src/evaluation.rs` - Main evaluation module that contains the current `evaluate_king_safety` function and `PositionEvaluator` struct.
- `src/evaluation/king_safety.rs` - New main king safety evaluation module containing `KingSafetyEvaluator` struct.
- `src/evaluation/castles.rs` - Castle pattern recognition module with `CastleRecognizer` and `CastlePattern` types.
- `src/evaluation/attacks.rs` - Attack analysis module with `AttackAnalyzer` and attack evaluation logic.
- `src/evaluation/patterns/mino.rs` - Mino castle pattern definitions and recognition logic.
- `src/evaluation/patterns/anaguma.rs` - Anaguma castle pattern definitions and recognition logic.
- `src/evaluation/patterns/yagura.rs` - Yagura castle pattern definitions and recognition logic.
- `src/evaluation/patterns/common.rs` - Common attack patterns and tactical threat detection.
- `src/types.rs` - Type definitions that may need extension for king safety configuration.
- `tests/king_safety_tests.rs` - Unit tests for king safety evaluation components.
- `tests/king_safety_integration_tests.rs` - Integration tests for king safety evaluation.
- `tests/king_safety_performance_tests.rs` - Performance tests for king safety evaluation.

### Notes

- Unit tests should be placed in the `tests/` directory following the existing pattern (e.g., `king_safety_tests.rs`).
- Use `cargo test king_safety` to run king safety specific tests.
- The new evaluation modules will be created under `src/evaluation/` directory.
- Configuration will be integrated into the existing `TaperedEvaluationConfig` structure.

## Tasks

- [x] 1.0 Create Core Infrastructure and Module Structure
  - [x] 1.1 Create `src/evaluation/` directory structure
  - [x] 1.2 Define `KingSafetyConfig` struct in `src/types.rs`
  - [x] 1.3 Create `src/evaluation/king_safety.rs` with `KingSafetyEvaluator` struct
  - [x] 1.4 Add `KingSafetyConfig` to `TaperedEvaluationConfig` in `src/types.rs`
  - [x] 1.5 Create `src/evaluation/patterns/` directory for castle patterns
  - [x] 1.6 Update `src/lib.rs` to include new evaluation modules
  - [x] 1.7 Create basic module structure with placeholder implementations

- [x] 2.0 Implement Castle Pattern Recognition System
  - [x] 2.1 Create `src/evaluation/castles.rs` with `CastleRecognizer` and `CastlePattern` types
  - [x] 2.2 Define `CastlePiece` struct with relative position and weight fields
  - [x] 2.3 Implement `src/evaluation/patterns/mino.rs` with Mino castle pattern definition
  - [x] 2.4 Implement `src/evaluation/patterns/anaguma.rs` with Anaguma castle pattern definition
  - [x] 2.5 Implement `src/evaluation/patterns/yagura.rs` with Yagura castle pattern definition
  - [x] 2.6 Create `src/evaluation/patterns/common.rs` for shared pattern utilities
  - [x] 2.7 Implement pattern matching algorithm in `CastleRecognizer`
  - [x] 2.8 Add flexibility scoring for incomplete castle patterns
  - [x] 2.9 Implement castle recognition for both Black and White players

- [x] 3.0 Implement Attack Analysis and Threat Evaluation
  - [x] 3.1 Create `src/evaluation/attacks.rs` with `AttackAnalyzer` struct
  - [x] 3.2 Define `AttackZone` struct with center position and radius
  - [x] 3.3 Implement `AttackTables` with pre-computed attack bitboards
  - [x] 3.4 Create `AttackEvaluation` struct to hold attack analysis results
  - [x] 3.5 Implement attack zone generation around king position
  - [x] 3.6 Add piece-specific attack value calculations
  - [x] 3.7 Implement attack coordination analysis (rook-bishop, double attacks)
  - [x] 3.8 Create `ThreatEvaluator` struct for tactical threat detection
  - [x] 3.9 Define `TacticalPattern` and `TacticalType` enums
  - [x] 3.10 Implement common tactical pattern detectors (pins, skewers, forks)
  - [x] 3.11 Add threat scoring system with phase-aware adjustments

- [x] 4.0 Integrate Advanced King Safety with Existing Evaluation
  - [x] 4.1 Modify `PositionEvaluator` in `src/evaluation.rs` to use `KingSafetyEvaluator`
  - [x] 4.2 Update `evaluate_king_safety` function to call new advanced evaluation
  - [x] 4.3 Add configuration options to enable/disable advanced king safety
  - [x] 4.4 Implement fallback to basic king safety when advanced is disabled
  - [x] 4.5 Add phase-aware evaluation adjustments
  - [x] 4.6 Integrate castle, attack, and threat scores into final evaluation
  - [x] 4.7 Update `SearchEngine` to use new king safety evaluation
  - [x] 4.8 Add performance mode for fast evaluation in deep search nodes

- [x] 5.0 Add Comprehensive Testing and Validation
  - [x] 5.1 Create `tests/king_safety_tests.rs` with unit tests for all components
  - [x] 5.2 Add tests for Mino, Anaguma, and Yagura castle recognition
  - [x] 5.3 Create tests for attack analysis and coordination detection
  - [x] 5.4 Add tests for tactical threat detection (pins, skewers, forks)
  - [x] 5.5 Create `tests/king_safety_integration_tests.rs` for end-to-end testing
  - [x] 5.6 Add integration tests with real game positions
  - [x] 5.7 Create `tests/king_safety_performance_tests.rs` for performance validation
  - [x] 5.8 Add tests for configuration options and fallback behavior
  - [x] 5.9 Create test positions for castle recognition accuracy validation
  - [x] 5.10 Add tests for phase-aware evaluation adjustments

- [x] 6.0 Performance Optimization and Tuning
  - [x] 6.1 Implement fast evaluation mode for deep search nodes
  - [x] 6.2 Add attack table pre-computation and caching
  - [x] 6.3 Optimize castle pattern matching with early termination
  - [x] 6.4 Add performance benchmarking and profiling
  - [x] 6.5 Implement tuning parameters for castle scores and attack weights
  - [x] 6.6 Add configuration presets for different performance levels
  - [x] 6.7 Optimize memory usage for attack tables and pattern storage
  - [x] 6.8 Add performance regression tests
  - [x] 6.9 Implement incremental evaluation updates where possible
  - [x] 6.10 Add performance monitoring and statistics collection
