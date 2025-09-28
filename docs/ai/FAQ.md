# Frequently Asked Questions (FAQ)

Common questions and answers about the automated tuning system.

## Table of Contents

1. [General Questions](#general-questions)
2. [Installation and Setup](#installation-and-setup)
3. [Data and Input](#data-and-input)
4. [Optimization and Algorithms](#optimization-and-algorithms)
5. [Performance and Hardware](#performance-and-hardware)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Usage](#advanced-usage)

## General Questions

### What is automated tuning?

Automated tuning is a process that uses machine learning algorithms to automatically optimize the evaluation function parameters of a chess/shogi engine. Instead of manually adjusting weights, the system learns optimal values from large datasets of games.

### Why use automated tuning?

- **Accuracy**: Data-driven optimization often produces better results than manual tuning
- **Efficiency**: Automates the tedious process of parameter adjustment
- **Objectivity**: Removes human bias from the tuning process
- **Scalability**: Can process large amounts of data quickly
- **Reproducibility**: Results can be replicated and validated

### How does it work?

The system uses Texel's tuning method, which:
1. Extracts features from game positions
2. Uses the actual game results as training labels
3. Optimizes weights to minimize prediction error
4. Validates results using cross-validation

### What algorithms are supported?

- **Adam Optimizer**: Adaptive learning rate (recommended)
- **Gradient Descent**: Classic optimization with momentum
- **LBFGS**: Limited-memory BFGS quasi-Newton method
- **Genetic Algorithm**: Population-based search

### How long does tuning take?

Typical tuning times:
- **Small dataset** (10K positions): 10-30 minutes
- **Medium dataset** (100K positions): 2-8 hours
- **Large dataset** (1M+ positions): 12-48 hours

## Installation and Setup

### What are the system requirements?

**Minimum:**
- Rust 1.70+
- 4GB RAM
- 2 CPU cores
- 10GB free disk space

**Recommended:**
- Rust 1.70+
- 16GB+ RAM
- 8+ CPU cores
- SSD storage
- 50GB+ free disk space

### How do I install the tuning system?

```bash
# Clone repository
git clone <repository-url>
cd shogi-game/worktrees/usi

# Build the tuning system
cargo build --release --bin tuner
```

### Do I need special hardware?

No special hardware is required, but performance scales with:
- **CPU cores**: More cores = faster parallel processing
- **RAM**: More memory = larger datasets
- **Storage**: SSD recommended for checkpoints

### Can I run this on Windows/Mac/Linux?

Yes, the system works on all platforms supported by Rust:
- Windows 10+
- macOS 10.15+
- Linux (most distributions)

## Data and Input

### What game formats are supported?

- **KIF**: Japanese Shogi format
- **CSA**: Computer Shogi Association format
- **PGN**: Portable Game Notation
- **JSON**: Custom structured format

### How much data do I need?

**Minimum**: 10,000 positions
**Recommended**: 100,000+ positions
**Optimal**: 1,000,000+ positions

More data generally leads to better results, but with diminishing returns.

### Where can I get game data?

- **Professional databases**: ShogiDB, KifuDB
- **Online platforms**: 81Dojo, Shogi Wars, Shogi Club 24
- **Tournament games**: Local and national championships
- **Synthetic data**: Generated for testing

### What makes good training data?

- **High player ratings** (>2000 recommended)
- **Balanced results** (not all wins for one side)
- **Quiet positions** (no recent captures)
- **Diverse game phases** (opening, middlegame, endgame)
- **Clean data** (no errors or duplicates)

### How do I prepare my data?

```bash
# Convert to JSON format
./target/release/tuner prepare-data --input games.kif --output games.json

# Filter for quality
./target/release/tuner prepare-data \
  --input games.json \
  --output filtered.json \
  --min-rating 2000 \
  --quiet-only
```

## Optimization and Algorithms

### Which optimizer should I use?

**Adam Optimizer** is recommended for most cases because it:
- Adapts learning rate automatically
- Converges reliably
- Works well with default parameters
- Handles noisy gradients well

### How do I choose learning rate?

**Start with defaults:**
- Adam: 0.01
- Gradient Descent: 0.01
- LBFGS: Automatic
- Genetic Algorithm: Population-based

**Adjust based on convergence:**
- Too slow: Increase learning rate
- Oscillating: Decrease learning rate
- Diverging: Significantly decrease learning rate

### How many iterations do I need?

**Typical ranges:**
- **Small datasets**: 500-2,000 iterations
- **Medium datasets**: 2,000-10,000 iterations
- **Large datasets**: 10,000-50,000 iterations

**Stop when:**
- Error stops decreasing
- Validation error starts increasing (overfitting)
- Time limit reached

### What is cross-validation?

Cross-validation splits your data into multiple folds, trains on some folds, and tests on others. This provides more reliable estimates of performance and helps detect overfitting.

**Common settings:**
- **5-fold**: Standard choice, good balance
- **10-fold**: More reliable but slower
- **3-fold**: Faster but less reliable

### How do I prevent overfitting?

- **Regularization**: Add L2 regularization (0.001-0.01)
- **Early stopping**: Stop when validation error increases
- **More data**: Larger datasets reduce overfitting
- **Cross-validation**: Use proper train/validation splits

## Performance and Hardware

### How can I make tuning faster?

**Hardware optimizations:**
- Use more CPU cores (`--threads 16`)
- Use more RAM (`--memory-limit 16384`)
- Use SSD storage for checkpoints
- Use faster CPU (higher clock speed)

**Algorithm optimizations:**
- Use LBFGS for smooth problems
- Reduce dataset size for testing
- Use smaller batch sizes
- Enable progress monitoring

### How much memory do I need?

**Memory usage scales with dataset size:**
- 10K positions: ~1GB
- 100K positions: ~4GB
- 1M positions: ~16GB
- 10M positions: ~64GB

**Optimization tips:**
- Use `--memory-limit` to control usage
- Process data in chunks
- Use memory-efficient algorithms

### Can I run this on a cluster?

Yes, the system supports distributed processing:
- **Multi-node**: Use MPI or similar
- **Cloud**: AWS, GCP, Azure compatible
- **Container**: Docker support included
- **Batch systems**: SLURM, PBS compatible

### How do I monitor progress?

```bash
# Enable progress bars
./target/release/tuner tune --progress

# Enable verbose logging
./target/release/tuner tune --verbose

# Monitor system resources
htop
iotop
```

## Troubleshooting

### Tuning is not converging

**Possible causes:**
- Learning rate too high/low
- Poor data quality
- Insufficient data
- Wrong optimizer for problem

**Solutions:**
```bash
# Try different learning rate
./target/release/tuner tune --learning-rate 0.001

# Try different optimizer
./target/release/tuner tune --method lbfgs

# Check data quality
./target/release/tuner validate-data --dataset games.json
```

### Getting out of memory errors

**Solutions:**
```bash
# Reduce memory limit
./target/release/tuner tune --memory-limit 4096

# Use smaller batches
./target/release/tuner tune --batch-size 100

# Process data in chunks
./target/release/tuner prepare-data --input games.json --output chunk1.json --max-positions 50000
```

### Validation error is increasing

This indicates overfitting. **Solutions:**
```bash
# Increase regularization
./target/release/tuner tune --regularization 0.01

# Use early stopping
./target/release/tuner tune --early-stopping --patience 100

# Get more data
./target/release/tuner prepare-data --input games.json --output more_data.json --max-positions 1000000
```

### File format errors

**Common issues:**
- Invalid JSON syntax
- Wrong file encoding
- Corrupted files

**Solutions:**
```bash
# Validate JSON
python -m json.tool games.json

# Check file encoding
file games.json

# Convert format
./target/release/tuner prepare-data --input games.kif --output games.json
```

### Slow performance

**Optimization tips:**
```bash
# Use more threads
./target/release/tuner tune --threads 16

# Use faster algorithm
./target/release/tuner tune --method lbfgs

# Reduce dataset size for testing
./target/release/tuner tune --max-positions 50000
```

## Advanced Usage

### How do I customize the evaluation function?

The evaluation function is defined in `src/evaluation.rs`. You can:
- Add new features
- Modify existing features
- Change feature weights
- Add new evaluation terms

### Can I use this for other games?

The framework can be adapted for other games by:
- Modifying the position representation
- Updating the feature extraction
- Changing the game rules
- Adjusting the optimization parameters

### How do I integrate with my engine?

```rust
// Load tuned weights
let mut evaluator = PositionEvaluator::new();
evaluator.load_tuned_weights("weights.json")?;
evaluator.set_use_tuned_weights(true);

// Use in evaluation
let score = evaluator.evaluate_with_tuned_weights(&board, player, &captured_pieces);
```

### Can I resume interrupted tuning?

Yes, use checkpoints:
```bash
# Enable checkpointing
./target/release/tuner tune --checkpoint-frequency 100

# Resume from checkpoint
./target/release/tuner tune --resume-from checkpoint_iter_1000.json
```

### How do I compare different configurations?

```bash
# Test multiple configurations
for config in config1.json config2.json config3.json; do
  ./target/release/tuner tune --config $config --output weights_${config}.json
  ./target/release/tuner validate --weights weights_${config}.json --dataset test.json
done
```

### Can I use this for research?

Yes, the system is designed for research:
- **Reproducible results**: All parameters are logged
- **Statistical validation**: Cross-validation and significance testing
- **Performance metrics**: Detailed performance analysis
- **Extensible**: Easy to add new algorithms or features

### How do I contribute to the project?

1. **Report bugs**: Use GitHub issues
2. **Suggest features**: Use GitHub discussions
3. **Submit code**: Use pull requests
4. **Improve documentation**: Submit documentation improvements
5. **Share results**: Contribute benchmark results

### Where can I get help?

- **Documentation**: Check all guides in `docs/ai/`
- **Examples**: See `OPTIMIZATION_EXAMPLES.md`
- **Community**: GitHub discussions and issues
- **Troubleshooting**: See `TROUBLESHOOTING_GUIDE.md`

### What's the license?

Check the LICENSE file in the repository root for licensing information.

### How often should I retune?

**Retuning frequency depends on:**
- **New data availability**: Retune when you have significant new data
- **Engine changes**: Retune when evaluation function changes
- **Performance degradation**: Retune if engine strength decreases
- **Research needs**: Retune for new experiments

**Typical schedule:**
- **Production**: Monthly or quarterly
- **Research**: As needed for experiments
- **Development**: Weekly during active development

## Next Steps

- [User Guide](USER_GUIDE.md) for detailed usage instructions
- [Optimization Examples](OPTIMIZATION_EXAMPLES.md) for configuration examples
- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for problem resolution
- [Data Preparation Guide](DATA_PREPARATION_GUIDE.md) for data handling
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) for optimization tips
