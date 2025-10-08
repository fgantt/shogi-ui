# Tapered Evaluation - Tuning Guide

## Overview

This guide explains how to use the automated tuning system to optimize evaluation weights for maximum playing strength.

## Prerequisites

- Game database with results (KIF, CSA, or PGN format)
- At least 10,000 positions for training
- 2,000 positions for validation

## Quick Start

### Step 1: Prepare Training Data

```rust
use shogi_engine::evaluation::tuning::*;

fn prepare_training_data(game_database: &str) -> Vec<TuningPosition> {
    let mut positions = Vec::new();
    
    // Load games from database
    for game in load_games(game_database) {
        for position in extract_positions(&game) {
            positions.push(TuningPosition {
                material_score: evaluate_material(&position),
                position_score: evaluate_position(&position),
                king_safety_score: evaluate_king_safety(&position),
                pawn_structure_score: evaluate_pawns(&position),
                mobility_score: evaluate_mobility(&position),
                center_control_score: evaluate_center(&position),
                development_score: evaluate_development(&position),
                result: game.result_for_player(position.player),
            });
        }
    }
    
    positions
}
```

### Step 2: Create Tuner

```rust
let mut tuner = TaperedEvaluationTuner::new();

// Or with custom configuration
let config = TuningConfig {
    method: OptimizationMethod::GradientDescent,
    learning_rate: 0.001,
    max_iterations: 1000,
    convergence_threshold: 0.0001,
};

let mut tuner = TaperedEvaluationTuner::with_config(config);
```

### Step 3: Add Training Data

```rust
let positions = prepare_training_data("games.db");

tuner.add_training_data(positions);

// Split into training (80%) and validation (20%)
tuner.split_data(0.2);
```

### Step 4: Run Optimization

```rust
let results = tuner.optimize()?;

println!("Optimization Results:");
println!("  Iterations: {}", results.iterations);
println!("  Training error: {:.6}", results.training_error);
println!("  Validation error: {:.6}", results.validation_error);
println!("  Duration: {:?}", results.duration);
```

### Step 5: Extract Optimized Weights

```rust
let weights = results.optimized_weights;

println!("\nOptimized Weights:");
println!("  Material: {:.3}", weights.material_weight);
println!("  Position: {:.3}", weights.position_weight);
println!("  King Safety: {:.3}", weights.king_safety_weight);
println!("  Pawn Structure: {:.3}", weights.pawn_structure_weight);
println!("  Mobility: {:.3}", weights.mobility_weight);
println!("  Center Control: {:.3}", weights.center_control_weight);
println!("  Development: {:.3}", weights.development_weight);
```

## Optimization Methods

### 1. Gradient Descent (Default, Fast)

Best for: Quick tuning with good initialization

```rust
let config = TuningConfig {
    method: OptimizationMethod::GradientDescent,
    learning_rate: 0.001,
    max_iterations: 1000,
    convergence_threshold: 0.0001,
};
```

**Characteristics**:
- Fast convergence (100-500 iterations)
- Requires good initial weights
- May find local minima
- Recommended for: Fine-tuning existing weights

### 2. Genetic Algorithm (Robust, Slower)

Best for: Global optimization, avoiding local minima

```rust
let config = TuningConfig {
    method: OptimizationMethod::GeneticAlgorithm,
    learning_rate: 0.0,  // Not used
    max_iterations: 200,
    convergence_threshold: 0.0,
};
```

**Characteristics**:
- Explores larger solution space
- More robust to initialization
- Slower convergence (50-200 generations)
- Recommended for: Initial weight discovery

### 3. Cross-Validation (Stable, Most Robust)

Best for: Preventing overfitting, production weights

```rust
let config = TuningConfig {
    method: OptimizationMethod::CrossValidation,
    learning_rate: 0.001,
    max_iterations: 500,
    convergence_threshold: 0.0001,
};
```

**Characteristics**:
- K-fold validation (K=5)
- Best generalization
- Prevents overfitting
- Recommended for: Final production weights

## Best Practices

### 1. Data Preparation

**Quality over Quantity**:
- Use games from strong players (>1800 rating)
- Filter out blitz/bullet games (too random)
- Remove duplicate positions
- Balance win/loss/draw ratios

**Position Selection**:
- Skip first 10 moves (opening theory)
- Use positions from move 10-60
- Avoid obvious winning/losing positions
- Include diverse position types

### 2. Training Process

**Start with Gradient Descent**:
```rust
// Quick tuning run
let config = TuningConfig {
    method: OptimizationMethod::GradientDescent,
    learning_rate: 0.001,
    max_iterations: 500,
    convergence_threshold: 0.0001,
};
```

**Validate with Cross-Validation**:
```rust
// Final weights
let config = TuningConfig {
    method: OptimizationMethod::CrossValidation,
    learning_rate: 0.0005,
    max_iterations: 1000,
    convergence_threshold: 0.00001,
};
```

### 3. Hyperparameter Tuning

**Learning Rate**:
- Too high: Unstable, oscillation
- Too low: Slow convergence
- Recommended: Start at 0.001, reduce if unstable

**Iterations**:
- Gradient Descent: 500-2000
- Genetic Algorithm: 100-500
- Cross-Validation: 500-1000

### 4. Validation

**Monitor Errors**:
```rust
let results = tuner.optimize()?;

// Check for overfitting
if results.validation_error > results.training_error * 1.5 {
    println!("Warning: Possible overfitting!");
    println!("  Training: {:.6}", results.training_error);
    println!("  Validation: {:.6}", results.validation_error);
}
```

**Compare Against Baseline**:
```rust
// Test optimized weights vs default
let baseline_error = test_with_weights(EvaluationWeights::default());
let tuned_error = test_with_weights(results.optimized_weights);

println!("Improvement: {:.2}%", 
    (baseline_error - tuned_error) / baseline_error * 100.0);
```

## Common Issues

### Issue: Training Error Not Decreasing

**Causes**:
- Learning rate too high or too low
- Bad initialization
- Insufficient data

**Solutions**:
```rust
// Try lower learning rate
config.learning_rate = 0.0001;

// Try genetic algorithm
config.method = OptimizationMethod::GeneticAlgorithm;

// Add more training data
```

### Issue: Validation Error Higher Than Training

**Cause**: Overfitting

**Solutions**:
```rust
// Use cross-validation
config.method = OptimizationMethod::CrossValidation;

// Reduce iterations
config.max_iterations = 500;

// Add more validation data
tuner.split_data(0.3);  // 30% validation
```

### Issue: Weights Stuck at Extremes

**Cause**: Optimization hitting bounds

**Solution**:
- Weights are automatically clamped to reasonable ranges
- Check if clamping is too restrictive
- Consider using genetic algorithm for better exploration

## Advanced Tuning Techniques

### Ensemble Methods

```rust
// Run multiple optimizations
let methods = [
    OptimizationMethod::GradientDescent,
    OptimizationMethod::GeneticAlgorithm,
    OptimizationMethod::CrossValidation,
];

let mut all_weights = Vec::new();

for method in &methods {
    let config = TuningConfig {
        method: *method,
        learning_rate: 0.001,
        max_iterations: 500,
        convergence_threshold: 0.0001,
    };
    
    let mut tuner = TaperedEvaluationTuner::with_config(config);
    tuner.add_training_data(positions.clone());
    
    let results = tuner.optimize()?;
    all_weights.push(results.optimized_weights);
}

// Average the weights
let final_weights = average_weights(&all_weights);
```

### Progressive Tuning

```rust
// Stage 1: Material only
let mut config = tuning_config.clone();
// ... tune material weight only ...

// Stage 2: Add positional
// ... tune material + position weights ...

// Stage 3: Add all features
// ... tune all weights ...
```

## Expected Results

### Typical Weight Ranges

| Weight | Default | Typical Range | Best Practices |
|---|---|---|---|
| Material | 1.0 | 0.8-1.5 | Keep close to 1.0 |
| Position | 0.8 | 0.5-1.2 | Depends on PST quality |
| King Safety | 1.2 | 0.8-2.0 | Higher in tactical games |
| Pawn Structure | 0.6 | 0.3-1.0 | Lower than material |
| Mobility | 0.5 | 0.3-0.8 | Important in middlegame |
| Center Control | 0.7 | 0.4-1.0 | Higher in opening |
| Development | 0.5 | 0.3-0.8 | Important in opening |

### Performance Improvements

Expected improvements after tuning:
- **Evaluation accuracy**: 10-20% better prediction
- **Playing strength**: +50-150 Elo
- **Win rate**: +5-10% against baseline

## Saving and Loading Weights

The tuned weights are automatically integrated with the configuration system:

```rust
use shogi_engine::evaluation::config::TaperedEvalConfig;

// After tuning, create config with optimized weights
let mut config = TaperedEvalConfig::default();
config.weights = results.optimized_weights;

// Save to file
config.save_to_file("tuned_weights.json")?;

// Load later
let loaded_config = TaperedEvalConfig::load_from_file("tuned_weights.json")?;
```

## Conclusion

The automated tuning system provides:
- 3 optimization algorithms
- Flexible configuration
- Validation support
- Easy integration

Follow this guide to:
1. Prepare quality training data
2. Choose appropriate optimization method
3. Configure hyperparameters
4. Run optimization
5. Validate results
6. Deploy optimized weights

Expected outcome: +50-150 Elo improvement with properly tuned weights!

---

*Guide Version: 1.0*
*Generated: October 8, 2025*

