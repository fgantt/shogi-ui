# Migration Guide

Guide for migrating from manual to automated evaluation tuning.

## Table of Contents

1. [Migration Overview](#migration-overview)
2. [Pre-Migration Checklist](#pre-migration-checklist)
3. [Step-by-Step Migration](#step-by-step-migration)
4. [Data Migration](#data-migration)
5. [Configuration Migration](#configuration-migration)
6. [Testing Migration](#testing-migration)
7. [Rollback Procedures](#rollback-procedures)

## Migration Overview

### Benefits of Migration

- **Automated Optimization**: No more manual parameter adjustment
- **Data-Driven Results**: Optimization based on real game data
- **Reproducible Process**: Consistent results across runs
- **Performance Monitoring**: Built-in performance tracking
- **Validation Framework**: Comprehensive testing and validation

### Migration Path

1. **Preparation**: Gather existing evaluation data and configurations
2. **Data Collection**: Collect game databases for training
3. **System Setup**: Install and configure automated tuning system
4. **Initial Tuning**: Run first automated tuning session
5. **Validation**: Compare results with manual tuning
6. **Integration**: Integrate tuned weights with engine
7. **Testing**: Comprehensive testing of new evaluation
8. **Deployment**: Deploy automated tuning in production

## Pre-Migration Checklist

### Current System Assessment

- [ ] **Document Current Evaluation Function**
  - List all evaluation terms and weights
  - Document manual tuning process
  - Record performance benchmarks
  - Identify critical evaluation features

- [ ] **Collect Existing Data**
  - Manual tuning logs and results
  - Performance test results
  - Game databases used for testing
  - Engine strength measurements

- [ ] **System Requirements**
  - Verify hardware meets requirements
  - Install required dependencies
  - Prepare storage for datasets
  - Set up monitoring systems

### Data Preparation

- [ ] **Game Database Collection**
  - Professional games (recommended: 100K+ positions)
  - High-quality amateur games (rating > 2000)
  - Diverse time controls and game phases
  - Clean, validated game data

- [ ] **Data Format Conversion**
  - Convert existing data to supported formats
  - Validate data integrity
  - Remove duplicates and invalid games
  - Create train/validation/test splits

## Step-by-Step Migration

### Phase 1: System Setup

#### 1.1 Install Automated Tuning System

```bash
# Build the tuning system
cargo build --release --bin tuner

# Verify installation
./target/release/tuner --help
```

#### 1.2 Prepare Data

```bash
# Convert existing data to JSON format
./target/release/tuner prepare-data \
  --input existing_games.kif \
  --output converted_games.json \
  --format json

# Validate converted data
./target/release/tuner validate-data --dataset converted_games.json
```

#### 1.3 Create Baseline

```bash
# Create baseline weights from current evaluation
./target/release/tuner extract-weights \
  --evaluation-function current_eval.rs \
  --output baseline_weights.json
```

### Phase 2: Initial Tuning

#### 2.1 Run Initial Tuning

```bash
# Run automated tuning with conservative settings
./target/release/tuner tune \
  --dataset converted_games.json \
  --output initial_weights.json \
  --method adam \
  --iterations 1000 \
  --k-fold 3 \
  --progress
```

#### 2.2 Validate Results

```bash
# Validate tuned weights
./target/release/tuner validate \
  --weights initial_weights.json \
  --dataset converted_games.json \
  --k-fold 3

# Compare with baseline
./target/release/tuner benchmark \
  --weights baseline_weights.json \
  --weights2 initial_weights.json \
  --games 1000
```

### Phase 3: Integration

#### 3.1 Integrate with Engine

```rust
// Add to engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    // ... existing fields ...
    
    pub evaluation: EvaluationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub use_tuned_weights: bool,
    pub weights_path: Option<PathBuf>,
    pub fallback_enabled: bool,
}

// Load tuned weights in engine initialization
impl Engine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        let mut evaluator = PositionEvaluator::new();
        
        if config.evaluation.use_tuned_weights {
            if let Some(weights_path) = &config.evaluation.weights_path {
                evaluator.load_tuned_weights(weights_path)?;
                evaluator.set_use_tuned_weights(true);
            }
        }
        
        Ok(Engine { evaluator, config })
    }
}
```

#### 3.2 Add CLI Options

```rust
#[derive(Parser)]
struct Cli {
    // ... existing arguments ...
    
    /// Use tuned weights for evaluation
    #[arg(long)]
    tuned_weights: bool,
    
    /// Path to tuned weights file
    #[arg(long)]
    weights_path: Option<PathBuf>,
}
```

### Phase 4: Testing and Validation

#### 4.1 Performance Testing

```bash
# Test engine strength with tuned weights
./target/release/engine \
  --tuned-weights \
  --weights-path initial_weights.json \
  --benchmark \
  --games 10000

# Compare with baseline
./target/release/engine \
  --benchmark \
  --games 10000
```

#### 4.2 Regression Testing

```bash
# Run comprehensive test suite
cargo test --release

# Test specific evaluation scenarios
./target/release/tuner test-evaluation \
  --weights initial_weights.json \
  --test-positions test_positions.json
```

## Data Migration

### Converting Existing Data

#### Manual Tuning Data

```bash
# Convert manual tuning logs to structured format
./target/release/tuner convert-manual-data \
  --input manual_tuning_logs.txt \
  --output manual_data.json \
  --format json
```

#### Evaluation Parameters

```bash
# Extract current evaluation parameters
./target/release/tuner extract-parameters \
  --evaluation-file src/evaluation.rs \
  --output current_parameters.json
```

### Data Validation

```bash
# Validate migrated data
./target/release/tuner validate-migration \
  --original-data original_games.kif \
  --migrated-data converted_games.json \
  --check-consistency
```

## Configuration Migration

### Engine Configuration

#### Before (Manual Tuning)

```rust
// Manual evaluation parameters
pub const PAWN_VALUE: i32 = 100;
pub const LANCE_VALUE: i32 = 300;
pub const KNIGHT_VALUE: i32 = 400;
pub const SILVER_VALUE: i32 = 500;
pub const GOLD_VALUE: i32 = 600;
pub const BISHOP_VALUE: i32 = 900;
pub const ROOK_VALUE: i32 = 1000;

// Manual positional weights
pub const PAWN_PST: [[i32; 9]; 9] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    // ... manual piece-square table
];
```

#### After (Automated Tuning)

```rust
// Configuration-based evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub weights_path: Option<PathBuf>,
    pub use_tuned_weights: bool,
    pub fallback_enabled: bool,
}

impl EvaluationConfig {
    pub fn default() -> Self {
        Self {
            weights_path: None,
            use_tuned_weights: false,
            fallback_enabled: true,
        }
    }
}
```

### Tuning Configuration

```json
{
  "evaluation": {
    "use_tuned_weights": true,
    "weights_path": "tuned_weights.json",
    "fallback_enabled": true
  },
  "tuning": {
    "enabled": true,
    "dataset_path": "training_data.json",
    "method": "adam",
    "iterations": 5000,
    "k_fold": 5
  }
}
```

## Testing Migration

### Validation Tests

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn test_manual_vs_automated_weights() {
        let manual_weights = load_manual_weights();
        let automated_weights = load_automated_weights();
        
        // Test on same positions
        let test_positions = load_test_positions();
        
        for position in test_positions {
            let manual_score = evaluate_with_weights(&position, &manual_weights);
            let automated_score = evaluate_with_weights(&position, &automated_weights);
            
            // Scores should be reasonably close
            let difference = (manual_score - automated_score).abs();
            assert!(difference < 100, "Score difference too large: {}", difference);
        }
    }
    
    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = PositionEvaluator::new();
        evaluator.load_tuned_weights("tuned_weights.json").unwrap();
        
        // Test multiple evaluations of same position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let scores: Vec<i32> = (0..100).map(|_| {
            evaluator.evaluate_with_tuned_weights(&board, Player::Black, &captured_pieces)
        }).collect();
        
        // All scores should be identical
        let first_score = scores[0];
        for &score in &scores[1..] {
            assert_eq!(score, first_score, "Inconsistent evaluation scores");
        }
    }
}
```

### Performance Tests

```rust
#[test]
fn test_performance_comparison() {
    let mut manual_engine = Engine::new_with_manual_evaluation();
    let mut tuned_engine = Engine::new_with_tuned_evaluation();
    
    let test_positions = load_performance_test_positions();
    
    // Measure evaluation speed
    let manual_time = measure_evaluation_time(&mut manual_engine, &test_positions);
    let tuned_time = measure_evaluation_time(&mut tuned_engine, &test_positions);
    
    // Tuned evaluation should not be significantly slower
    let slowdown = tuned_time.as_secs_f64() / manual_time.as_secs_f64();
    assert!(slowdown < 2.0, "Tuned evaluation too slow: {:.2}x slower", slowdown);
}

fn measure_evaluation_time(engine: &mut Engine, positions: &[TestPosition]) -> Duration {
    let start = Instant::now();
    
    for position in positions {
        engine.evaluate_position(&position.board, position.player, &position.captured_pieces);
    }
    
    start.elapsed()
}
```

## Rollback Procedures

### Emergency Rollback

```bash
# Disable tuned weights immediately
./target/release/engine --no-tuned-weights

# Or modify configuration
cp config_backup.json engine_config.json
```

### Gradual Rollback

```rust
impl Engine {
    pub fn rollback_to_manual_evaluation(&mut self) {
        self.evaluator.set_use_tuned_weights(false);
        log::info!("Rolled back to manual evaluation");
    }
    
    pub fn enable_tuned_evaluation(&mut self) -> Result<(), EngineError> {
        if self.evaluator.get_weight_manager().has_weights() {
            self.evaluator.set_use_tuned_weights(true);
            log::info!("Enabled tuned evaluation");
            Ok(())
        } else {
            Err(EngineError::NoWeightsLoaded)
        }
    }
}
```

### Data Backup

```bash
# Backup current weights
cp tuned_weights.json tuned_weights_backup.json

# Backup configuration
cp engine_config.json engine_config_backup.json

# Backup evaluation code
git tag manual-evaluation-backup
git push origin manual-evaluation-backup
```

## Migration Checklist

### Pre-Migration

- [ ] Document current evaluation system
- [ ] Collect existing performance data
- [ ] Prepare game databases
- [ ] Set up automated tuning system
- [ ] Create baseline measurements

### Migration Process

- [ ] Convert existing data to supported formats
- [ ] Run initial automated tuning
- [ ] Validate tuning results
- [ ] Integrate tuned weights with engine
- [ ] Test engine with tuned evaluation
- [ ] Compare performance with baseline

### Post-Migration

- [ ] Deploy automated tuning system
- [ ] Set up continuous tuning pipeline
- [ ] Monitor performance metrics
- [ ] Document new processes
- [ ] Train team on new system

### Validation

- [ ] Engine strength maintained or improved
- [ ] Evaluation speed acceptable
- [ ] No regression in test suite
- [ ] Automated tuning produces consistent results
- [ ] Fallback mechanisms work correctly

## Troubleshooting

### Common Issues

1. **Tuned weights perform worse than manual**
   - Check data quality
   - Verify feature extraction
   - Increase training iterations
   - Try different optimization methods

2. **Integration errors**
   - Verify weight file format
   - Check feature count compatibility
   - Enable fallback mechanisms
   - Validate input data

3. **Performance degradation**
   - Profile evaluation code
   - Optimize feature extraction
   - Use caching mechanisms
   - Consider parallel processing

### Support Resources

- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md)
- [User Guide](USER_GUIDE.md)
- [API Documentation](API_DOCUMENTATION.md)
- [FAQ](FAQ.md)

## Next Steps

After successful migration:

1. **Set up continuous tuning**: Automate regular retuning with new data
2. **Monitor performance**: Track engine strength and evaluation speed
3. **Expand features**: Add new evaluation terms and features
4. **Optimize pipeline**: Improve data processing and tuning efficiency
5. **Document processes**: Maintain documentation for team knowledge transfer

## Next Steps

- [User Guide](USER_GUIDE.md) for detailed usage instructions
- [Integration Guide](INTEGRATION_GUIDE.md) for engine integration
- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for problem resolution
