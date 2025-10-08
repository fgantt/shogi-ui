# Tapered Evaluation Usage Guide

## Introduction

This guide explains how to use the tapered evaluation system in the Shogi engine. Tapered evaluation provides phase-aware position assessment by interpolating between middlegame and endgame evaluation weights.

## Quick Start

### Basic Usage

```rust
use shogi_engine::evaluation::tapered_eval::TaperedEvaluation;
use shogi_engine::types::{BitboardBoard, TaperedScore, GAME_PHASE_MAX};

// Create a tapered evaluator
let mut evaluator = TaperedEvaluation::new();

// Get the current board state
let board = BitboardBoard::new(); // Starting position

// Calculate game phase (0 = endgame, 256 = opening)
let phase = evaluator.calculate_game_phase(&board);
println!("Current game phase: {}", phase);

// Create a tapered score with different mg/eg values
let score = TaperedScore::new_tapered(100, 200);

// Interpolate based on current phase
let final_score = evaluator.interpolate(score, phase);
println!("Final score: {}", final_score);
```

## Core Concepts

### Game Phase

The game phase represents the stage of the game from opening to endgame:
- **256 (GAME_PHASE_MAX)**: Opening/middlegame (full material)
- **128**: Transition phase
- **0**: Pure endgame (minimal material)

Phase is calculated based on non-pawn, non-king pieces remaining on the board.

### Tapered Score

A `TaperedScore` contains two values:
- **mg**: Middlegame score
- **eg**: Endgame score

The evaluator interpolates between these based on the current game phase.

### Interpolation

Linear interpolation formula:
```
final_score = (mg × phase + eg × (256 - phase)) / 256
```

This provides smooth transitions between game phases.

## Configuration

### Default Configuration

```rust
let evaluator = TaperedEvaluation::new();
// Uses default configuration:
// - Enabled: true
// - Cache: enabled
// - Performance monitoring: disabled
```

### Custom Configuration

```rust
use shogi_engine::types::TaperedEvaluationConfig;

let config = TaperedEvaluationConfig {
    enabled: true,
    cache_game_phase: true,
    use_simd: false,
    memory_pool_size: 1000,
    enable_performance_monitoring: true,
    king_safety: KingSafetyConfig::default(),
};

let mut evaluator = TaperedEvaluation::with_config(config);
```

### Pre-defined Configurations

```rust
// Performance optimized (larger cache, monitoring enabled)
let config = TaperedEvaluationConfig::performance_optimized();
let evaluator = TaperedEvaluation::with_config(config);

// Memory optimized (smaller cache, no monitoring)
let config = TaperedEvaluationConfig::memory_optimized();
let evaluator = TaperedEvaluation::with_config(config);

// Disabled (fallback to simple evaluation)
let config = TaperedEvaluationConfig::disabled();
let evaluator = TaperedEvaluation::with_config(config);
```

## Creating Tapered Scores

### Equal Values (Phase-Independent)

```rust
// Material values are typically the same in all phases
let pawn_value = TaperedScore::new(100);
// mg = 100, eg = 100
```

### Different Values (Phase-Dependent)

```rust
// Rooks are more valuable in endgame
let rook_positional = TaperedScore::new_tapered(10, 30);
// mg = 10, eg = 30

// Advanced pawns are more valuable in endgame
let passed_pawn = TaperedScore::new_tapered(50, 100);
// mg = 50, eg = 100
```

### Using Helper Methods

```rust
let evaluator = TaperedEvaluation::new();

// Create equal score
let score1 = evaluator.create_score(50);

// Create tapered score
let score2 = evaluator.create_tapered_score(100, 200);
```

## Score Arithmetic

Tapered scores support arithmetic operations:

```rust
let score1 = TaperedScore::new_tapered(100, 150);
let score2 = TaperedScore::new_tapered(50, 75);

// Addition
let sum = score1 + score2; // mg: 150, eg: 225

// Subtraction
let diff = score1 - score2; // mg: 50, eg: 75

// Negation
let neg = -score1; // mg: -100, eg: -150

// Multiplication by scalar
let scaled = score1 * 0.5; // mg: 50, eg: 75

// In-place operations
let mut total = TaperedScore::default();
total += score1;
total += score2;
// total: mg: 150, eg: 225
```

## Accumulating Scores

```rust
let mut evaluator = TaperedEvaluation::new();
let board = BitboardBoard::new();

// Accumulate various evaluation terms
let mut total_score = TaperedScore::default();

// Material (phase-independent)
total_score += TaperedScore::new(100);

// Piece position (phase-dependent)
total_score += TaperedScore::new_tapered(15, 10);

// King safety (more important in middlegame)
total_score += TaperedScore::new_tapered(50, 20);

// Pawn structure (more important in endgame)
total_score += TaperedScore::new_tapered(20, 40);

// Mobility (more important in endgame)
total_score += TaperedScore::new_tapered(10, 25);

// Calculate phase and interpolate
let phase = evaluator.calculate_game_phase(&board);
let final_score = evaluator.interpolate(total_score, phase);

println!("Final evaluation: {}", final_score);
```

## Cache Management

### Automatic Caching

When cache is enabled, the evaluator automatically caches phase calculations:

```rust
let mut evaluator = TaperedEvaluation::new();
let board = BitboardBoard::new();

// First call - calculates and caches
let phase1 = evaluator.calculate_game_phase(&board);

// Second call - retrieves from cache
let phase2 = evaluator.calculate_game_phase(&board);

// Check cache statistics
let stats = evaluator.stats();
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
```

### Manual Cache Control

```rust
let mut evaluator = TaperedEvaluation::new();

// Clear cache when board changes significantly
evaluator.clear_cache();

// Reset all statistics
evaluator.reset_stats();
```

## Performance Monitoring

Enable performance monitoring to track evaluation statistics:

```rust
let config = TaperedEvaluationConfig {
    enable_performance_monitoring: true,
    ..Default::default()
};
let mut evaluator = TaperedEvaluation::with_config(config);

// Perform evaluations...
let board = BitboardBoard::new();
for _ in 0..1000 {
    let phase = evaluator.calculate_game_phase(&board);
    let score = TaperedScore::new_tapered(100, 200);
    let _ = evaluator.interpolate(score, phase);
}

// Check statistics
let stats = evaluator.stats();
println!("Phase calculations: {}", stats.phase_calculations);
println!("Cache hits: {}", stats.cache_hits);
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
println!("Interpolations: {}", stats.total_interpolations());
```

## Integration with Existing Evaluator

The new `TaperedEvaluation` struct can be used alongside the existing `PositionEvaluator`:

```rust
use shogi_engine::evaluation::{PositionEvaluator, tapered_eval::TaperedEvaluation};

let pos_evaluator = PositionEvaluator::new();
let mut tapered_evaluator = TaperedEvaluation::new();

let board = BitboardBoard::new();
let player = Player::Black;
let captured_pieces = CapturedPieces::new();

// Use PositionEvaluator's built-in tapered evaluation
let score1 = pos_evaluator.evaluate(&board, player, &captured_pieces);

// Or use TaperedEvaluation for more control
let phase = tapered_evaluator.calculate_game_phase(&board);
// ... accumulate scores ...
let score2 = tapered_evaluator.interpolate(total_score, phase);
```

## Best Practices

### 1. Use Phase-Appropriate Weights

```rust
// Material values - usually phase-independent
let material = TaperedScore::new(value);

// Positional bonuses - often phase-dependent
let positional = TaperedScore::new_tapered(mg_bonus, eg_bonus);

// Guidelines:
// - King safety: higher mg, lower eg
// - Mobility: lower mg, higher eg
// - Pawn advancement: lower mg, higher eg
// - Center control: higher mg, lower eg
// - Development: higher mg, lower eg
```

### 2. Cache When Appropriate

Enable caching for search applications where the same position may be evaluated multiple times:

```rust
let config = TaperedEvaluationConfig {
    cache_game_phase: true, // Enable for search
    ..Default::default()
};
```

Disable caching for single position evaluation or when memory is constrained:

```rust
let config = TaperedEvaluationConfig {
    cache_game_phase: false, // Disable for one-off evaluations
    ..Default::default()
};
```

### 3. Monitor Performance

Use performance monitoring during development and tuning:

```rust
let config = TaperedEvaluationConfig {
    enable_performance_monitoring: true,
    ..Default::default()
};

// Disable in production for best performance
let config = TaperedEvaluationConfig {
    enable_performance_monitoring: false,
    ..Default::default()
};
```

### 4. Smooth Transitions

Ensure evaluation components have smooth transitions:

```rust
// Good: Smooth transition
let smooth = TaperedScore::new_tapered(100, 150);

// Avoid: Large discontinuities
let discontinuous = TaperedScore::new_tapered(100, 500);
```

### 5. Test Phase Boundaries

Always test evaluation at phase boundaries:

```rust
let score = TaperedScore::new_tapered(100, 200);

// Test at boundaries
let opening_score = score.interpolate(256); // Should be 100
let endgame_score = score.interpolate(0);   // Should be 200
let middle_score = score.interpolate(128);  // Should be 150

assert_eq!(opening_score, 100);
assert_eq!(endgame_score, 200);
assert_eq!(middle_score, 150);
```

## Common Patterns

### Pattern 1: Evaluating a Position

```rust
fn evaluate_position(board: &BitboardBoard) -> i32 {
    let mut evaluator = TaperedEvaluation::new();
    let phase = evaluator.calculate_game_phase(board);
    
    let mut total = TaperedScore::default();
    
    // Add evaluation components
    total += evaluate_material(board);
    total += evaluate_position(board);
    total += evaluate_king_safety(board);
    
    evaluator.interpolate(total, phase)
}
```

### Pattern 2: Creating Piece-Square Tables

```rust
struct PieceSquareTables {
    pawn_mg: [[i32; 9]; 9],
    pawn_eg: [[i32; 9]; 9],
}

impl PieceSquareTables {
    fn get_score(&self, pos: Position) -> TaperedScore {
        let row = pos.row as usize;
        let col = pos.col as usize;
        TaperedScore::new_tapered(
            self.pawn_mg[row][col],
            self.pawn_eg[row][col],
        )
    }
}
```

### Pattern 3: Phase-Aware Bonuses

```rust
fn evaluate_passed_pawn(advancement: i32) -> TaperedScore {
    // Linear scaling example
    let mg_bonus = advancement * 10;
    let eg_bonus = advancement * 20; // More valuable in endgame
    TaperedScore::new_tapered(mg_bonus, eg_bonus)
}
```

## Troubleshooting

### Issue: Unexpected Phase Values

**Problem**: Game phase doesn't match expected values

**Solution**: Check piece phase values and board state

```rust
let mut evaluator = TaperedEvaluation::new();
let board = BitboardBoard::new();
let phase = evaluator.calculate_game_phase(&board);

// Starting position should be 256
assert_eq!(phase, 256);

// Debug: Check piece contributions
for piece_type in [Knight, Silver, Gold, Bishop, Rook, Lance] {
    if let Some(value) = evaluator.get_piece_phase_value(piece_type) {
        println!("{:?}: {}", piece_type, value);
    }
}
```

### Issue: Evaluation Discontinuities

**Problem**: Evaluation jumps suddenly between phases

**Solution**: Verify smooth interpolation

```rust
let score = TaperedScore::new_tapered(mg_value, eg_value);

let mut prev = score.interpolate(0);
for phase in 1..=256 {
    let curr = score.interpolate(phase);
    let diff = (curr - prev).abs();
    
    // Difference should be at most 1
    assert!(diff <= 1, "Discontinuity at phase {}: diff = {}", phase, diff);
    prev = curr;
}
```

### Issue: Poor Cache Performance

**Problem**: Low cache hit rate

**Solution**: Check if positions are being reused

```rust
let mut evaluator = TaperedEvaluation::new();
let stats = evaluator.stats();

// Low hit rate indicates positions aren't being reused
if stats.cache_hit_rate() < 0.3 {
    println!("Warning: Low cache hit rate ({:.2}%)", 
             stats.cache_hit_rate() * 100.0);
    println!("Consider disabling cache for this use case");
}
```

## API Reference

### TaperedEvaluation

```rust
impl TaperedEvaluation {
    pub fn new() -> Self;
    pub fn with_config(config: TaperedEvaluationConfig) -> Self;
    pub fn config(&self) -> &TaperedEvaluationConfig;
    pub fn set_config(&mut self, config: TaperedEvaluationConfig);
    pub fn calculate_game_phase(&mut self, board: &BitboardBoard) -> i32;
    pub fn interpolate(&self, score: TaperedScore, phase: i32) -> i32;
    pub fn create_score(&self, value: i32) -> TaperedScore;
    pub fn create_tapered_score(&self, mg: i32, eg: i32) -> TaperedScore;
    pub fn stats(&self) -> &TaperedEvaluationStats;
    pub fn reset_stats(&mut self);
    pub fn clear_cache(&mut self);
}
```

### TaperedScore

```rust
impl TaperedScore {
    pub fn new(value: i32) -> Self;
    pub fn new_tapered(mg: i32, eg: i32) -> Self;
    pub fn interpolate(&self, phase: i32) -> i32;
}

// Implements: Add, Sub, AddAssign, SubAssign, Neg, Mul<f32>
```

### TaperedEvaluationConfig

```rust
impl TaperedEvaluationConfig {
    pub fn new() -> Self;
    pub fn disabled() -> Self;
    pub fn performance_optimized() -> Self;
    pub fn memory_optimized() -> Self;
}
```

## Examples

See the following files for more examples:
- `src/evaluation/tapered_eval.rs` - Unit tests
- `benches/tapered_evaluation_performance_benchmarks.rs` - Performance benchmarks
- `src/evaluation.rs` - Integration with PositionEvaluator

## Further Reading

- [DESIGN_TAPERED_EVALUATION.md](DESIGN_TAPERED_EVALUATION.md) - Design documentation
- [IMPLEMENT_TAPERED_EVALUATION.md](IMPLEMENT_TAPERED_EVALUATION.md) - Implementation guide
- [TASKS_TAPERED_EVALUATION.md](TASKS_TAPERED_EVALUATION.md) - Task list and progress
- [TASK_1_1_COMPLETION_SUMMARY.md](TASK_1_1_COMPLETION_SUMMARY.md) - Completion summary

