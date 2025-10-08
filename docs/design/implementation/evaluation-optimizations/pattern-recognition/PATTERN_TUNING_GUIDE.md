# Pattern Recognition Tuning Guide

**Version**: 1.0  
**Date**: October 8, 2025

## Overview

This guide covers tuning pattern recognition weights and parameters to optimize evaluation accuracy and playing strength.

## Default Weights

All pattern types start with weight = 1.0:

| Pattern Type | Default Weight | Impact |
|--------------|----------------|--------|
| Piece-Square Tables | 1.0 | Positional placement |
| Pawn Structure | 1.0 | Pawn evaluation |
| King Safety | 1.0 | King protection |
| Piece Coordination | 1.0 | Piece cooperation |
| Mobility | 1.0 | Piece movement |
| Tactical Patterns | 1.0 | Forks, pins, skewers |
| Positional Patterns | 1.0 | Center, outposts, space |
| Endgame Patterns | 1.0 | Endgame evaluation |

## Tuning Strategy

### Step 1: Baseline Measurement

```rust
// Measure current performance
let mut evaluator = PositionEvaluator::new();
let test_positions = load_test_positions();

for position in test_positions {
    let score = evaluator.evaluate(&position.board, position.player, &captured);
    let error = (score - position.expected_score).abs();
    total_error += error;
}

println!("Baseline error: {}", total_error / test_positions.len());
```

### Step 2: Adjust Weights

```rust
let mut config = PatternConfig::default();

// Try different weights
for tactical_weight in [0.8, 1.0, 1.2, 1.5] {
    config.set_tactical_patterns_weight(tactical_weight);
    
    // Test and measure error
    let error = test_with_config(&config);
    println!("Tactical weight {}: error = {}", tactical_weight, error);
}
```

### Step 3: Fine-Tune

```rust
// Best weight found: 1.2
config.set_tactical_patterns_weight(1.2);

// Now tune related weights
config.set_piece_coordination_weight(1.1);
config.set_mobility_weight(1.05);
```

### Step 4: Validate

```rust
// Test with independent positions
let validation_error = validate_with_config(&config);
if validation_error < baseline_error {
    println!("✅ Tuning improved evaluation by {:.1}%",
        (baseline_error - validation_error) / baseline_error * 100.0);
}
```

---

## Weight Guidelines

### Tactical Patterns Weight

**Range**: 0.8 - 1.8

**Recommendations**:
- **0.8-0.9**: Positional style (reduce tactical emphasis)
- **1.0**: Balanced (default)
- **1.2-1.5**: Aggressive style (emphasize tactics)
- **1.6-1.8**: Ultra-aggressive (may over-value tactics)

### Positional Patterns Weight

**Range**: 0.8 - 1.6

**Recommendations**:
- **0.8**: Fast play (reduce positional calculation)
- **1.0**: Balanced (default)
- **1.2-1.4**: Positional style
- **1.5-1.6**: Strategic play (long-term planning)

### King Safety Weight

**Range**: 0.9 - 2.0

**Recommendations**:
- **0.9**: Aggressive (accept some risk)
- **1.0**: Balanced (default)
- **1.3-1.6**: Defensive style
- **1.7-2.0**: Very defensive (prioritize safety)

### Endgame Patterns Weight

**Range**: 0.8 - 1.8

**Recommendations**:
- **0.8**: Middlegame focused
- **1.0**: Balanced (default)
- **1.2-1.5**: Endgame specialist
- **1.6-1.8**: Pure endgame focus

---

## Style-Specific Tuning

### Aggressive Style

```rust
let mut config = PatternConfig::default();
config.set_tactical_patterns_weight(1.5);      // Emphasize tactics
config.set_piece_coordination_weight(1.3);     // Value coordination
config.set_mobility_weight(1.2);               // Value activity
config.set_king_safety_weight(0.9);            // Accept some risk
config.set_positional_patterns_weight(0.8);    // Reduce positional
```

**Expected**: More aggressive play, tactical complications

### Positional Style

```rust
let mut config = PatternConfig::default();
config.set_positional_patterns_weight(1.4);    // Emphasize positional
config.set_pawn_structure_weight(1.3);         // Value pawn structure
config.set_center_control_weight(1.2);         // Control center
config.set_tactical_patterns_weight(0.9);      // Reduce tactics
config.set_mobility_weight(1.1);               // Value mobility
```

**Expected**: Strategic play, strong positional foundation

### Defensive Style

```rust
let mut config = PatternConfig::default();
config.set_king_safety_weight(1.6);            // Prioritize king safety
config.set_fortress_patterns_weight(1.4);      // Value defense
config.set_weak_square_weight(1.3);            // Avoid weaknesses
config.set_tactical_patterns_weight(1.1);      // Keep tactical awareness
config.set_mobility_weight(0.9);               // Less emphasis on mobility
```

**Expected**: Solid defense, fewer risks

### Endgame Specialist

```rust
let mut config = PatternConfig::default();
config.set_endgame_patterns_weight(1.5);       // Emphasize endgame
config.set_pawn_structure_weight(1.3);         // Pawn endgames
config.set_mobility_weight(1.2);               // King mobility
config.set_positional_patterns_weight(0.8);    // Reduce positional
```

**Expected**: Strong endgame play, accurate endgame evaluation

---

## Automated Tuning

### Using Machine Learning (Advanced)

```rust
use shogi_engine::evaluation::pattern_advanced::AdvancedPatternSystem;

let mut system = AdvancedPatternSystem::new();
system.ml_config.enabled = true;
system.ml_config.learning_rate = 0.01;
system.ml_config.iterations = 1000;

// Prepare training data
let training_data = load_professional_games();

// Optimize weights
let optimized_weights = system.optimize_weights(&training_data);

// Apply optimized weights
for (i, weight) in optimized_weights.iter().enumerate() {
    println!("Pattern {}: weight = {:.3}", i, weight);
}
```

---

## Performance Tuning

### Cache Size Tuning

**Too Small** (<10K entries):
- Low hit rate (<40%)
- Frequent evictions
- Poor performance gain

**Optimal** (50K-200K entries):
- Good hit rate (60-80%)
- Balanced memory usage
- 90% speedup on hits

**Too Large** (>500K entries):
- Diminishing returns
- High memory usage
- Cache management overhead

**Recommendation**: Start with 100K, adjust based on hit rate

### Component Selection Tuning

```rust
// For fastest evaluation
let components = ComponentFlags {
    material: true,
    piece_square_tables: true,
    position_features: true,
    opening_principles: false,  // Disable
    endgame_patterns: false,    // Disable
    tactical_patterns: true,    // Keep
    positional_patterns: false, // Disable
};
// ~50% faster, ~10% less accurate
```

---

## Validation

### Test Against Benchmark Positions

```rust
let benchmark_positions = vec![
    (fen1, expected_score1),
    (fen2, expected_score2),
    // ...
];

let mut total_error = 0;
for (fen, expected) in benchmark_positions {
    let board = parse_fen(fen);
    let score = evaluator.evaluate(&board, player, &captured);
    let error = (score - expected).abs();
    total_error += error;
}

let avg_error = total_error / benchmark_positions.len();
println!("Average error: {} centipawns", avg_error);
```

### Cross-Validation

```rust
// Split data into training and validation sets
let (training, validation) = split_positions(all_positions, 0.8);

// Tune on training set
let tuned_weights = tune_weights(&training);

// Validate on validation set
let validation_error = test_weights(&validation, &tuned_weights);

println!("Validation error: {}", validation_error);
```

---

## Common Tuning Patterns

### Pattern 1: Overvaluing Tactics

**Symptom**: Engine makes overly aggressive moves

**Fix**:
```rust
config.set_tactical_patterns_weight(0.8);  // Reduce tactical emphasis
config.set_king_safety_weight(1.2);        // Increase safety
```

### Pattern 2: Undervaluing Position

**Symptom**: Engine accepts poor pawn structure, weak squares

**Fix**:
```rust
config.set_pawn_structure_weight(1.3);
config.set_positional_patterns_weight(1.2);
```

### Pattern 3: Weak Endgame

**Symptom**: Poor endgame conversion, missing winning lines

**Fix**:
```rust
config.set_endgame_patterns_weight(1.4);
config.set_pawn_structure_weight(1.3);  // Pawn endgames
```

---

## Summary

1. **Start with defaults** (weight = 1.0 for all)
2. **Measure baseline** performance
3. **Adjust weights** systematically
4. **Test thoroughly** with positions
5. **Validate** with independent data
6. **Document** your configuration
7. **Monitor** performance impact

**Tuning Guide Complete** ✅
