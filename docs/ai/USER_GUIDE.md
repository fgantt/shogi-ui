# Automated Tuning System User Guide

Complete guide for using the Shogi engine automated tuning system.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Command Reference](#command-reference)
3. [Configuration Options](#configuration-options)
4. [Workflow Examples](#workflow-examples)
5. [Advanced Usage](#advanced-usage)
6. [Integration with Engine](#integration-with-engine)

## Getting Started

### System Requirements

- **Rust**: Version 1.70 or higher
- **Memory**: Minimum 4GB, 8GB+ recommended for large datasets
- **CPU**: Multi-core processor recommended for parallel processing
- **Storage**: SSD recommended for checkpoint files

### Installation

```bash
# Build the tuning system
cargo build --release --bin tuner

# The executable will be available at:
# ./target/release/tuner
```

### First Run

```bash
# Test the installation
./target/release/tuner --help

# Run with sample data (if available)
./target/release/tuner tune --dataset sample_games.json --output my_weights.json
```

## Command Reference

### Main Commands

#### `tune` - Run Automated Tuning

The primary command for optimizing evaluation weights.

```bash
tuner tune [OPTIONS] --dataset <PATH> --output <PATH>
```

**Required Arguments:**
- `--dataset <PATH>`: Path to game database file
- `--output <PATH>`: Path for output weights file

**Optimization Options:**
- `--method <ALGORITHM>`: Optimization algorithm
  - `gradient`: Gradient descent with momentum
  - `adam`: Adam optimizer (recommended)
  - `lbfgs`: Limited-memory BFGS
  - `genetic`: Genetic algorithm
- `--iterations <NUMBER>`: Maximum iterations (default: 1000)
- `--learning-rate <FLOAT>`: Learning rate (default: 0.01)

**Validation Options:**
- `--k-fold <NUMBER>`: Cross-validation folds (default: 5)
- `--test-split <FLOAT>`: Test set proportion 0.0-1.0 (default: 0.2)
- `--validation-split <FLOAT>`: Validation set proportion (default: 0.1)

**Regularization:**
- `--regularization <FLOAT>`: L2 regularization strength (default: 0.001)
- `--early-stopping`: Enable early stopping (default: enabled)

**Data Filtering:**
- `--min-rating <NUMBER>`: Minimum player rating (default: 1500)
- `--max-rating <NUMBER>`: Maximum player rating (default: none)
- `--quiet-threshold <NUMBER>`: Quiet position threshold in moves (default: 3)
- `--min-move-number <NUMBER>`: Minimum move number (default: 10)

**Performance:**
- `--threads <NUMBER>`: Number of threads (default: auto)
- `--checkpoint-frequency <NUMBER>`: Checkpoint every N iterations (default: 100)
- `--memory-limit <MB>`: Memory limit in MB (default: 8192)

**Output:**
- `--progress`: Show progress bars
- `--verbose`: Verbose logging
- `--log-level <LEVEL>`: Log level (off, error, warn, info, debug, trace)

#### `validate` - Validate Weights

Test the quality of tuned weights.

```bash
tuner validate [OPTIONS] --weights <PATH> --dataset <PATH>
```

**Required Arguments:**
- `--weights <PATH>`: Path to weights file
- `--dataset <PATH>`: Path to validation dataset

**Validation Options:**
- `--k-fold <NUMBER>`: Cross-validation folds (default: 5)
- `--test-split <FLOAT>`: Test set proportion (default: 0.2)
- `--games <NUMBER>`: Number of games for strength testing (default: 1000)

#### `benchmark` - Performance Benchmarking

Measure engine strength improvement.

```bash
tuner benchmark [OPTIONS] --weights <PATH>
```

**Required Arguments:**
- `--weights <PATH>`: Path to weights file

**Benchmark Options:**
- `--games <NUMBER>`: Number of test games (default: 1000)
- `--time-control <SECONDS>`: Time control per game (default: 30)
- `--depth <NUMBER>`: Search depth (default: auto)

#### `generate` - Generate Synthetic Data

Create synthetic training data for testing.

```bash
tuner generate [OPTIONS] --output <PATH> --positions <NUMBER>
```

**Required Arguments:**
- `--output <PATH>`: Output file path
- `--positions <NUMBER>`: Number of positions to generate

#### `prepare-data` - Process Game Databases

Convert and filter game databases.

```bash
tuner prepare-data [OPTIONS] --input <PATH> --output <PATH>
```

**Required Arguments:**
- `--input <PATH>`: Input game database
- `--output <PATH>`: Output processed database

**Processing Options:**
- `--format <FORMAT>`: Output format (json, binary)
- `--filter-rating`: Apply rating filters
- `--deduplicate`: Remove duplicate positions
- `--quiet-only`: Keep only quiet positions

## Configuration Options

### Optimization Methods

#### Gradient Descent
```bash
--method gradient --learning-rate 0.01 --momentum 0.9
```
- Best for: Smooth optimization landscapes
- Pros: Simple, predictable convergence
- Cons: Can be slow, sensitive to learning rate

#### Adam Optimizer (Recommended)
```bash
--method adam --learning-rate 0.01 --beta1 0.9 --beta2 0.999 --epsilon 1e-8
```
- Best for: Most scenarios
- Pros: Adaptive learning rate, robust convergence
- Cons: More memory usage

#### LBFGS
```bash
--method lbfgs --memory 10 --tolerance 1e-5
```
- Best for: Smooth, well-conditioned problems
- Pros: Fast convergence, no learning rate tuning
- Cons: Requires smooth gradients

#### Genetic Algorithm
```bash
--method genetic --population-size 100 --mutation-rate 0.1 --crossover-rate 0.8
```
- Best for: Complex, non-convex optimization
- Pros: Global search, handles discontinuities
- Cons: Slower convergence, more parameters

### Data Filtering

#### Rating Filters
```bash
--min-rating 2000 --max-rating 2500
```
Filter games by player rating to focus on high-quality positions.

#### Position Filters
```bash
--quiet-threshold 3 --min-move-number 20 --max-move-number 80
```
Focus on quiet, middlegame positions for better training.

#### Game Phase Filters
```bash
--opening-weight 0.2 --middlegame-weight 0.6 --endgame-weight 0.2
```
Balance training data across game phases.

## Workflow Examples

### Basic Tuning Workflow

```bash
# 1. Prepare data
./target/release/tuner prepare-data \
  --input raw_games.json \
  --output processed_games.json \
  --filter-rating \
  --deduplicate

# 2. Run tuning
./target/release/tuner tune \
  --dataset processed_games.json \
  --output weights.json \
  --method adam \
  --iterations 2000 \
  --k-fold 5 \
  --progress

# 3. Validate results
./target/release/tuner validate \
  --weights weights.json \
  --dataset processed_games.json \
  --k-fold 5

# 4. Benchmark improvement
./target/release/tuner benchmark \
  --weights weights.json \
  --games 2000
```

### High-Quality Tuning Workflow

```bash
# 1. High-quality data preparation
./target/release/tuner prepare-data \
  --input master_games.json \
  --output master_processed.json \
  --filter-rating \
  --deduplicate \
  --quiet-only \
  --min-move-number 15

# 2. Extensive tuning with validation
./target/release/tuner tune \
  --dataset master_processed.json \
  --output master_weights.json \
  --method adam \
  --iterations 10000 \
  --learning-rate 0.005 \
  --k-fold 10 \
  --regularization 0.0001 \
  --checkpoint-frequency 500 \
  --min-rating 2200 \
  --quiet-threshold 4 \
  --progress

# 3. Comprehensive validation
./target/release/tuner validate \
  --weights master_weights.json \
  --dataset master_processed.json \
  --k-fold 10 \
  --games 5000

# 4. Performance benchmarking
./target/release/tuner benchmark \
  --weights master_weights.json \
  --games 10000 \
  --time-control 60
```

### Research/Development Workflow

```bash
# 1. Generate synthetic data for testing
./target/release/tuner generate \
  --output synthetic_data.json \
  --positions 50000

# 2. Test different algorithms
for method in gradient adam lbfgs genetic; do
  ./target/release/tuner tune \
    --dataset synthetic_data.json \
    --output weights_${method}.json \
    --method $method \
    --iterations 5000 \
    --k-fold 5 \
    --progress
done

# 3. Compare results
for method in gradient adam lbfgs genetic; do
  ./target/release/tuner validate \
    --weights weights_${method}.json \
    --dataset synthetic_data.json
done
```

## Advanced Usage

### Custom Optimization Parameters

```bash
# Fine-tuned Adam parameters
./target/release/tuner tune \
  --dataset games.json \
  --output weights.json \
  --method adam \
  --learning-rate 0.003 \
  --beta1 0.95 \
  --beta2 0.999 \
  --epsilon 1e-9 \
  --iterations 15000
```

### Memory-Efficient Processing

```bash
# For large datasets with limited memory
./target/release/tuner tune \
  --dataset large_games.json \
  --output weights.json \
  --memory-limit 4096 \
  --threads 4 \
  --checkpoint-frequency 50
```

### Parallel Processing

```bash
# Use all CPU cores
./target/release/tuner tune \
  --dataset games.json \
  --output weights.json \
  --threads 0 \  # 0 = auto-detect
  --progress
```

### Checkpoint and Resume

```bash
# Long-running tuning with checkpoints
./target/release/tuner tune \
  --dataset games.json \
  --output weights.json \
  --iterations 50000 \
  --checkpoint-frequency 1000

# If interrupted, resume from checkpoint
./target/release/tuner tune \
  --dataset games.json \
  --output weights.json \
  --resume-from checkpoint_iter_10000.json
```

## Integration with Engine

### Loading Tuned Weights

```rust
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::weights::WeightManager;

// Create evaluator with tuned weights
let mut evaluator = PositionEvaluator::new();
evaluator.load_tuned_weights("tuned_weights.json")?;
evaluator.set_use_tuned_weights(true);

// Use in evaluation
let score = evaluator.evaluate_with_tuned_weights(&board, player, &captured_pieces);
```

### Performance Monitoring

```rust
use shogi_engine::weights::WeightManager;

let weight_manager = evaluator.get_weight_manager();
let stats = weight_manager.get_stats();

println!("Evaluation calls: {}", stats.evaluation_calls);
println!("Average time per evaluation: {}μs", stats.avg_evaluation_time_us);
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
```

### Fallback Configuration

```rust
// Enable automatic fallback to default evaluation
evaluator.set_use_tuned_weights(true);

// The engine will automatically fall back to default evaluation
// if tuned weights fail to load or are invalid
```

## Best Practices

### Data Quality
1. Use high-quality games (rating > 2000)
2. Filter for quiet positions (no recent captures)
3. Balance opening/middlegame/endgame positions
4. Remove duplicate positions

### Optimization Settings
1. Start with Adam optimizer
2. Use cross-validation for reliable results
3. Enable early stopping to prevent overfitting
4. Use checkpointing for long runs

### Performance
1. Use SSD storage for checkpoint files
2. Allocate sufficient memory (8GB+ for large datasets)
3. Use parallel processing for faster execution
4. Monitor convergence with progress bars

### Validation
1. Always validate tuned weights
2. Use holdout data for final testing
3. Compare against baseline performance
4. Test on different position types

## Troubleshooting

See [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for common issues and solutions.

## Next Steps

- [Data Preparation Guide](DATA_PREPARATION_GUIDE.md)
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md)
- [API Documentation](API_DOCUMENTATION.md)
- [FAQ](FAQ.md)
