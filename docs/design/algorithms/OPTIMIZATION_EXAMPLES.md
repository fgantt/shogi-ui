# Optimization Examples and Configurations

Comprehensive examples for different optimization scenarios and use cases.

## Table of Contents

1. [Quick Start Examples](#quick-start-examples)
2. [Optimization Method Examples](#optimization-method-examples)
3. [Data Size Examples](#data-size-examples)
4. [Performance Scenarios](#performance-scenarios)
5. [Research Examples](#research-examples)
6. [Production Examples](#production-examples)

## Quick Start Examples

### Minimal Example - 5 Minutes
```bash
# Generate synthetic data and tune quickly
./target/release/tuner generate --output quick_test.json --positions 1000
./target/release/tuner tune \
  --dataset quick_test.json \
  --output quick_weights.json \
  --method adam \
  --iterations 100 \
  --progress
```

### Basic Example - 30 Minutes
```bash
# Use small real dataset
./target/release/tuner tune \
  --dataset small_games.json \
  --output basic_weights.json \
  --method adam \
  --iterations 500 \
  --k-fold 3 \
  --progress
```

### Standard Example - 2 Hours
```bash
# Standard quality tuning
./target/release/tuner tune \
  --dataset standard_games.json \
  --output standard_weights.json \
  --method adam \
  --iterations 2000 \
  --k-fold 5 \
  --learning-rate 0.01 \
  --regularization 0.001 \
  --progress
```

## Optimization Method Examples

### Gradient Descent Examples

#### Basic Gradient Descent
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_gradient.json \
  --method gradient \
  --learning-rate 0.01 \
  --iterations 2000
```

#### Gradient Descent with Momentum
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_gradient_momentum.json \
  --method gradient \
  --learning-rate 0.01 \
  --momentum 0.9 \
  --iterations 3000
```

#### High Learning Rate Gradient Descent
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_gradient_fast.json \
  --method gradient \
  --learning-rate 0.05 \
  --iterations 1000
```

### Adam Optimizer Examples

#### Standard Adam (Recommended)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_adam_standard.json \
  --method adam \
  --learning-rate 0.01 \
  --beta1 0.9 \
  --beta2 0.999 \
  --epsilon 1e-8 \
  --iterations 2000
```

#### Conservative Adam (Stable)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_adam_conservative.json \
  --method adam \
  --learning-rate 0.005 \
  --beta1 0.95 \
  --beta2 0.999 \
  --epsilon 1e-9 \
  --iterations 4000
```

#### Aggressive Adam (Fast)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_adam_aggressive.json \
  --method adam \
  --learning-rate 0.02 \
  --beta1 0.9 \
  --beta2 0.99 \
  --epsilon 1e-7 \
  --iterations 1000
```

### LBFGS Examples

#### Standard LBFGS
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_lbfgs_standard.json \
  --method lbfgs \
  --memory 10 \
  --tolerance 1e-5 \
  --iterations 500
```

#### High Memory LBFGS
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_lbfgs_high_memory.json \
  --method lbfgs \
  --memory 20 \
  --tolerance 1e-6 \
  --iterations 300
```

#### Low Tolerance LBFGS
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_lbfgs_precise.json \
  --method lbfgs \
  --memory 15 \
  --tolerance 1e-7 \
  --iterations 1000
```

### Genetic Algorithm Examples

#### Standard Genetic Algorithm
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_genetic_standard.json \
  --method genetic \
  --population-size 100 \
  --mutation-rate 0.1 \
  --crossover-rate 0.8 \
  --generations 500
```

#### Large Population Genetic Algorithm
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_genetic_large.json \
  --method genetic \
  --population-size 200 \
  --mutation-rate 0.15 \
  --crossover-rate 0.75 \
  --generations 300
```

#### High Mutation Genetic Algorithm
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_genetic_explore.json \
  --method genetic \
  --population-size 150 \
  --mutation-rate 0.2 \
  --crossover-rate 0.7 \
  --generations 400
```

## Data Size Examples

### Small Dataset (10K positions)
```bash
./target/release/tuner tune \
  --dataset small_games.json \
  --output weights_small.json \
  --method adam \
  --iterations 500 \
  --k-fold 3 \
  --learning-rate 0.01 \
  --memory-limit 2048
```

### Medium Dataset (100K positions)
```bash
./target/release/tuner tune \
  --dataset medium_games.json \
  --output weights_medium.json \
  --method adam \
  --iterations 2000 \
  --k-fold 5 \
  --learning-rate 0.01 \
  --regularization 0.001 \
  --memory-limit 8192 \
  --threads 8
```

### Large Dataset (1M positions)
```bash
./target/release/tuner tune \
  --dataset large_games.json \
  --output weights_large.json \
  --method adam \
  --iterations 5000 \
  --k-fold 10 \
  --learning-rate 0.005 \
  --regularization 0.0001 \
  --memory-limit 16384 \
  --threads 16 \
  --checkpoint-frequency 100
```

### Massive Dataset (10M positions)
```bash
./target/release/tuner tune \
  --dataset massive_games.json \
  --output weights_massive.json \
  --method adam \
  --iterations 10000 \
  --k-fold 20 \
  --learning-rate 0.003 \
  --regularization 0.0001 \
  --memory-limit 32768 \
  --threads 32 \
  --checkpoint-frequency 50 \
  --batch-size 1000
```

## Performance Scenarios

### Memory-Constrained System (4GB RAM)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_low_memory.json \
  --method adam \
  --iterations 3000 \
  --memory-limit 3072 \
  --threads 2 \
  --batch-size 500 \
  --checkpoint-frequency 200
```

### CPU-Constrained System (2 cores)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_low_cpu.json \
  --method lbfgs \
  --iterations 1000 \
  --threads 2 \
  --memory 5
```

### High-Performance System (32 cores, 64GB RAM)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_high_perf.json \
  --method adam \
  --iterations 10000 \
  --threads 32 \
  --memory-limit 57344 \
  --batch-size 2000 \
  --checkpoint-frequency 100
```

### Storage-Constrained System (Limited Disk Space)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output weights_low_storage.json \
  --method adam \
  --iterations 5000 \
  --checkpoint-frequency 1000 \
  --compress-checkpoints \
  --cleanup-temp-files
```

## Research Examples

### Algorithm Comparison Study
```bash
# Test all algorithms on same dataset
for method in gradient adam lbfgs genetic; do
  ./target/release/tuner tune \
    --dataset research_games.json \
    --output weights_${method}_research.json \
    --method $method \
    --iterations 5000 \
    --k-fold 10 \
    --progress
done

# Compare results
for method in gradient adam lbfgs genetic; do
  ./target/release/tuner validate \
    --weights weights_${method}_research.json \
    --dataset research_games.json \
    --k-fold 10
done
```

### Hyperparameter Sensitivity Study
```bash
# Test different learning rates
for lr in 0.001 0.005 0.01 0.02 0.05; do
  ./target/release/tuner tune \
    --dataset research_games.json \
    --output weights_adam_lr_${lr}.json \
    --method adam \
    --learning-rate $lr \
    --iterations 3000 \
    --k-fold 5
done
```

### Regularization Study
```bash
# Test different regularization strengths
for reg in 0.0001 0.001 0.01 0.1; do
  ./target/release/tuner tune \
    --dataset research_games.json \
    --output weights_adam_reg_${reg}.json \
    --method adam \
    --regularization $reg \
    --iterations 3000 \
    --k-fold 5
done
```

### Data Size Impact Study
```bash
# Test with different dataset sizes
for size in 10000 50000 100000 500000 1000000; do
  ./target/release/tuner prepare-data \
    --input full_games.json \
    --output subset_${size}.json \
    --max-positions $size
  
  ./target/release/tuner tune \
    --dataset subset_${size}.json \
    --output weights_size_${size}.json \
    --method adam \
    --iterations 2000 \
    --k-fold 5
done
```

## Production Examples

### High-Quality Production Tuning
```bash
# Production-grade tuning with extensive validation
./target/release/tuner tune \
  --dataset production_games.json \
  --output production_weights.json \
  --method adam \
  --iterations 20000 \
  --learning-rate 0.005 \
  --beta1 0.95 \
  --beta2 0.999 \
  --epsilon 1e-9 \
  --k-fold 10 \
  --regularization 0.0001 \
  --min-rating 2200 \
  --quiet-threshold 4 \
  --checkpoint-frequency 500 \
  --threads 16 \
  --memory-limit 32768 \
  --progress \
  --verbose

# Comprehensive validation
./target/release/tuner validate \
  --weights production_weights.json \
  --dataset production_games.json \
  --k-fold 10 \
  --games 10000

# Performance benchmarking
./target/release/tuner benchmark \
  --weights production_weights.json \
  --games 50000 \
  --time-control 120
```

### Incremental Tuning (Update Existing Weights)
```bash
# Start from existing weights
./target/release/tuner tune \
  --dataset new_games.json \
  --output updated_weights.json \
  --method adam \
  --iterations 5000 \
  --learning-rate 0.003 \
  --initial-weights existing_weights.json \
  --warm-start
```

### A/B Testing Setup
```bash
# Tune two versions for comparison
./target/release/tuner tune \
  --dataset games_v1.json \
  --output weights_v1.json \
  --method adam \
  --iterations 10000

./target/release/tuner tune \
  --dataset games_v2.json \
  --output weights_v2.json \
  --method adam \
  --iterations 10000

# Compare performance
./target/release/tuner benchmark \
  --weights weights_v1.json \
  --games 10000 \
  --output results_v1.json

./target/release/tuner benchmark \
  --weights weights_v2.json \
  --games 10000 \
  --output results_v2.json
```

### Continuous Integration Tuning
```bash
# Automated tuning for CI/CD pipeline
./target/release/tuner tune \
  --dataset ci_games.json \
  --output ci_weights.json \
  --method adam \
  --iterations 1000 \
  --k-fold 3 \
  --time-limit 3600 \
  --checkpoint-frequency 100 \
  --fail-on-regression \
  --min-improvement 0.01
```

## Specialized Scenarios

### Opening Book Tuning
```bash
# Focus on opening positions
./target/release/tuner tune \
  --dataset opening_games.json \
  --output opening_weights.json \
  --method adam \
  --iterations 3000 \
  --min-move-number 1 \
  --max-move-number 20 \
  --opening-weight 1.0 \
  --middlegame-weight 0.0 \
  --endgame-weight 0.0
```

### Endgame Tuning
```bash
# Focus on endgame positions
./target/release/tuner tune \
  --dataset endgame_games.json \
  --output endgame_weights.json \
  --method adam \
  --iterations 3000 \
  --min-move-number 80 \
  --endgame-weight 1.0 \
  --opening-weight 0.0 \
  --middlegame-weight 0.0
```

### Tactical Position Tuning
```bash
# Focus on tactical positions (non-quiet)
./target/release/tuner tune \
  --dataset tactical_games.json \
  --output tactical_weights.json \
  --method adam \
  --iterations 3000 \
  --quiet-threshold 0 \
  --tactical-weight 1.0
```

### Positional Position Tuning
```bash
# Focus on quiet positional play
./target/release/tuner tune \
  --dataset positional_games.json \
  --output positional_weights.json \
  --method adam \
  --iterations 3000 \
  --quiet-threshold 5 \
  --positional-weight 1.0
```

## Performance Optimization Examples

### Fast Convergence Setup
```bash
# Optimize for speed
./target/release/tuner tune \
  --dataset games.json \
  --output weights_fast.json \
  --method adam \
  --learning-rate 0.02 \
  --iterations 500 \
  --k-fold 3 \
  --threads 16 \
  --memory-limit 16384 \
  --batch-size 1000
```

### Maximum Quality Setup
```bash
# Optimize for quality
./target/release/tuner tune \
  --dataset games.json \
  --output weights_quality.json \
  --method adam \
  --learning-rate 0.003 \
  --iterations 50000 \
  --k-fold 20 \
  --regularization 0.00001 \
  --threads 32 \
  --memory-limit 65536 \
  --batch-size 500 \
  --checkpoint-frequency 100
```

### Balanced Setup
```bash
# Balance speed and quality
./target/release/tuner tune \
  --dataset games.json \
  --output weights_balanced.json \
  --method adam \
  --learning-rate 0.01 \
  --iterations 5000 \
  --k-fold 10 \
  --regularization 0.001 \
  --threads 16 \
  --memory-limit 32768 \
  --batch-size 1000 \
  --checkpoint-frequency 200
```

## Configuration Files

### JSON Configuration Example
```json
{
  "dataset": "games.json",
  "output": "weights.json",
  "method": "adam",
  "iterations": 5000,
  "learning_rate": 0.01,
  "beta1": 0.9,
  "beta2": 0.999,
  "epsilon": 1e-8,
  "k_fold": 5,
  "regularization": 0.001,
  "min_rating": 2000,
  "quiet_threshold": 3,
  "threads": 16,
  "memory_limit": 32768,
  "checkpoint_frequency": 100,
  "progress": true,
  "verbose": true
}
```

### YAML Configuration Example
```yaml
dataset: games.json
output: weights.json
method: adam
iterations: 5000
learning_rate: 0.01
beta1: 0.9
beta2: 0.999
epsilon: 1e-8
k_fold: 5
regularization: 0.001
min_rating: 2000
quiet_threshold: 3
threads: 16
memory_limit: 32768
checkpoint_frequency: 100
progress: true
verbose: true
```

## Best Practices Summary

1. **Start Simple**: Begin with Adam optimizer and default parameters
2. **Use Cross-Validation**: Always use k-fold validation for reliable results
3. **Monitor Progress**: Enable progress bars and checkpoints
4. **Validate Results**: Test on holdout data before deployment
5. **Consider Resources**: Match configuration to available hardware
6. **Document Settings**: Keep track of successful configurations
7. **Iterate Gradually**: Make small changes and measure impact

## Next Steps

- [User Guide](USER_GUIDE.md) for detailed command reference
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) for optimization tips
- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for common issues
