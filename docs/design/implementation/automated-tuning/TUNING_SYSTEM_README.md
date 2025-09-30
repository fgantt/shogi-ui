# Shogi Engine Automated Tuning System

A comprehensive automated evaluation tuning system for the Shogi engine using Texel's tuning method and advanced optimization algorithms.

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Game database files (KIF, CSA, PGN, or JSON format)
- At least 4GB RAM for large datasets
- Multi-core CPU recommended for parallel processing

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd shogi-game/worktrees/usi

# Build the tuning system
cargo build --release --bin tuner
```

### Basic Usage

```bash
# Run automated tuning with default settings
./target/release/tuner tune --dataset games.json --output weights.json

# Use Adam optimizer with custom parameters
./target/release/tuner tune \
  --dataset games.json \
  --output weights.json \
  --method adam \
  --iterations 1000 \
  --learning-rate 0.01

# Validate existing weights
./target/release/tuner validate --weights weights.json --dataset test_games.json
```

### Example: Complete Tuning Workflow

```bash
# 1. Prepare your game database (see Data Preparation Guide)
./target/release/tuner prepare-data --input raw_games.json --output processed_games.json

# 2. Run tuning with cross-validation
./target/release/tuner tune \
  --dataset processed_games.json \
  --output tuned_weights.json \
  --method adam \
  --k-fold 5 \
  --iterations 5000 \
  --checkpoint-frequency 100

# 3. Validate the results
./target/release/tuner validate \
  --weights tuned_weights.json \
  --dataset processed_games.json

# 4. Test engine strength improvement
./target/release/tuner benchmark \
  --weights tuned_weights.json \
  --games 1000
```

## Features

### 🚀 **Optimization Algorithms**
- **Gradient Descent** - Classic optimization with momentum
- **Adam** - Adaptive learning rate optimization (recommended)
- **LBFGS** - Quasi-Newton method for smooth convergence
- **Genetic Algorithm** - Population-based search for complex landscapes

### 📊 **Validation & Testing**
- K-fold cross-validation
- Holdout validation with train/test splits
- Engine strength testing with ELO calculations
- Overfitting detection and prevention

### 🔧 **Performance Monitoring**
- Real-time progress reporting with ETA
- Memory usage tracking
- Checkpoint/resume functionality
- Comprehensive performance metrics
- Statistical analysis tools

### 📁 **Data Processing**
- Support for KIF, CSA, PGN, and JSON formats
- Position filtering (rating, move number, quiet positions)
- Automatic deduplication
- Binary serialization for fast loading

## Command Line Interface

### Main Commands

```bash
tuner tune          # Run automated tuning
tuner validate      # Validate weights or tuning results
tuner benchmark     # Performance benchmarking
tuner generate      # Generate synthetic data for testing
tuner prepare-data  # Process raw game databases
```

### Tuning Options

```bash
--dataset <path>           # Input game database
--output <path>            # Output weights file
--method <algorithm>       # Optimization method (adam, gradient, lbfgs, genetic)
--iterations <number>      # Maximum iterations
--learning-rate <float>    # Learning rate (method-dependent)
--k-fold <number>          # Cross-validation folds
--test-split <float>       # Test set proportion (0.0-1.0)
--regularization <float>   # L2 regularization strength
--min-rating <number>      # Minimum player rating filter
--quiet-threshold <number> # Quiet position threshold (moves)
--progress                # Show progress bars
--checkpoint-frequency <n> # Save checkpoint every N iterations
```

## Configuration Examples

### Quick Tuning (Fast Results)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output quick_weights.json \
  --method adam \
  --iterations 1000 \
  --learning-rate 0.01 \
  --progress
```

### High-Quality Tuning (Best Results)
```bash
./target/release/tuner tune \
  --dataset games.json \
  --output best_weights.json \
  --method adam \
  --iterations 10000 \
  --learning-rate 0.005 \
  --k-fold 5 \
  --regularization 0.001 \
  --min-rating 2000 \
  --quiet-threshold 3 \
  --checkpoint-frequency 500 \
  --progress
```

### Research/Development
```bash
./target/release/tuner tune \
  --dataset research_games.json \
  --output research_weights.json \
  --method genetic \
  --iterations 50000 \
  --population-size 100 \
  --mutation-rate 0.1 \
  --crossover-rate 0.8 \
  --validation-split 0.2 \
  --progress
```

## File Formats

### Supported Input Formats

- **KIF** - Japanese Shogi format
- **CSA** - Computer Shogi Association format  
- **PGN** - Portable Game Notation
- **JSON** - Custom structured format

### Weight File Format

```json
{
  "header": {
    "version": "1.0",
    "magic_number": "SHOGI_WEIGHTS",
    "feature_count": 2000,
    "checksum": "abc123...",
    "tuning_method": "Adam",
    "validation_error": 0.0234,
    "training_positions": 150000,
    "timestamp": 1703123456
  },
  "weights": [0.1, 0.2, ...] // 2000 weight values
}
```

## Performance Guidelines

### Hardware Recommendations
- **CPU**: 8+ cores for parallel processing
- **RAM**: 8GB+ for datasets >100k positions
- **Storage**: SSD recommended for checkpoint files
- **Time**: 2-24 hours depending on dataset size and iterations

### Dataset Size Guidelines
- **Minimum**: 10,000 positions for basic tuning
- **Recommended**: 100,000+ positions for high-quality results
- **Maximum**: 1M+ positions (requires significant RAM)

### Optimization Tips
- Start with Adam optimizer for best results
- Use checkpointing for long runs (>1000 iterations)
- Enable progress bars to monitor convergence
- Validate results with holdout data

## Next Steps

1. **Data Preparation**: See [Data Preparation Guide](DATA_PREPARATION_GUIDE.md)
2. **Advanced Configuration**: See [User Guide](USER_GUIDE.md)
3. **Troubleshooting**: See [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md)
4. **API Documentation**: See [API Documentation](API_DOCUMENTATION.md)

## Support

- **Documentation**: Complete guides available in `docs/user/guides/` and `docs/design/algorithms/`
- **Examples**: See `examples/` directory for sample configurations
- **Issues**: Report problems via GitHub issues
- **FAQ**: Common questions answered in [FAQ](FAQ.md)

---

*For detailed information, see the complete documentation in the `docs/user/` and `docs/design/` directories.*
