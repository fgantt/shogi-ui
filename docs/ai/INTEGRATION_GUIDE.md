# Integration Guide

Complete guide for integrating tuned weights with the Shogi engine.

## Table of Contents

1. [Basic Integration](#basic-integration)
2. [Weight Loading](#weight-loading)
3. [Evaluation Integration](#evaluation-integration)
4. [Performance Optimization](#performance-optimization)
5. [Fallback Mechanisms](#fallback-mechanisms)
6. [Configuration Management](#configuration-management)
7. [Monitoring and Debugging](#monitoring-and-debugging)
8. [Best Practices](#best-practices)

## Basic Integration

### Adding Weight Support to Engine

The tuning system integrates with the existing evaluation system through the `PositionEvaluator` struct.

```rust
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::weights::WeightManager;
use shogi_engine::types::{BitboardBoard, Player, CapturedPieces};

// Create evaluator with weight support
let mut evaluator = PositionEvaluator::new();

// Load tuned weights
evaluator.load_tuned_weights("tuned_weights.json")?;

// Enable tuned weights
evaluator.set_use_tuned_weights(true);

// Use in evaluation
let score = evaluator.evaluate_with_tuned_weights(&board, player, &captured_pieces);
```

### Engine Configuration

Add weight configuration to your engine's configuration system:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    // ... existing fields ...
    
    // Weight configuration
    pub weights: WeightConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub fallback_enabled: bool,
    pub performance_monitoring: bool,
}
```

### Command Line Integration

Add weight options to your engine's CLI:

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
    
    /// Disable fallback to default evaluation
    #[arg(long)]
    no_fallback: bool,
}
```

## Weight Loading

### Loading Weights at Startup

```rust
impl Engine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        let mut evaluator = PositionEvaluator::new();
        
        // Load weights if enabled
        if config.weights.enabled {
            if let Some(weights_path) = &config.weights.path {
                match evaluator.load_tuned_weights(weights_path) {
                    Ok(_) => {
                        evaluator.set_use_tuned_weights(true);
                        println!("Loaded tuned weights from: {:?}", weights_path);
                    }
                    Err(e) => {
                        if config.weights.fallback_enabled {
                            println!("Warning: Failed to load weights: {}. Using default evaluation.", e);
                            evaluator.set_use_tuned_weights(false);
                        } else {
                            return Err(EngineError::WeightLoadingFailed(e));
                        }
                    }
                }
            } else {
                return Err(EngineError::NoWeightPath);
            }
        }
        
        Ok(Engine { evaluator, config })
    }
}
```

### Dynamic Weight Loading

```rust
impl Engine {
    pub fn reload_weights(&mut self, path: &Path) -> Result<(), EngineError> {
        match self.evaluator.load_tuned_weights(path) {
            Ok(_) => {
                self.evaluator.set_use_tuned_weights(true);
                println!("Successfully reloaded weights from: {:?}", path);
                Ok(())
            }
            Err(e) => {
                println!("Failed to reload weights: {}", e);
                Err(EngineError::WeightLoadingFailed(e))
            }
        }
    }
    
    pub fn disable_weights(&mut self) {
        self.evaluator.set_use_tuned_weights(false);
        println!("Disabled tuned weights, using default evaluation");
    }
    
    pub fn enable_weights(&mut self) -> Result<(), EngineError> {
        if self.evaluator.get_weight_manager().has_weights() {
            self.evaluator.set_use_tuned_weights(true);
            println!("Enabled tuned weights");
            Ok(())
        } else {
            Err(EngineError::NoWeightsLoaded)
        }
    }
}
```

### Weight Validation

```rust
impl Engine {
    pub fn validate_weights(&self) -> Result<WeightValidationResult, EngineError> {
        let weight_manager = self.evaluator.get_weight_manager();
        
        if !weight_manager.has_weights() {
            return Ok(WeightValidationResult::NoWeights);
        }
        
        // Test weight application
        let test_features = vec![0.0; 2000];
        match weight_manager.apply_weights(&test_features, 0) {
            Ok(_) => Ok(WeightValidationResult::Valid),
            Err(e) => Ok(WeightValidationResult::Invalid(e.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum WeightValidationResult {
    Valid,
    NoWeights,
    Invalid(String),
}
```

## Evaluation Integration

### Using Tuned Evaluation

```rust
impl Engine {
    pub fn evaluate_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        // Use tuned evaluation if available
        if self.evaluator.is_using_tuned_weights() {
            self.evaluator.evaluate_with_tuned_weights(board, player, captured_pieces)
        } else {
            // Fallback to default evaluation
            self.evaluator.evaluate(board, player, captured_pieces)
        }
    }
}
```

### Feature Extraction Integration

```rust
impl Engine {
    pub fn get_position_features(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<f64> {
        self.evaluator.get_evaluation_features(board, player, captured_pieces)
    }
    
    pub fn analyze_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> PositionAnalysis {
        let features = self.get_position_features(board, player, captured_pieces);
        let score = self.evaluate_position(board, player, captured_pieces);
        
        PositionAnalysis {
            score,
            features,
            using_tuned_weights: self.evaluator.is_using_tuned_weights(),
        }
    }
}

#[derive(Debug)]
pub struct PositionAnalysis {
    pub score: i32,
    pub features: Vec<f64>,
    pub using_tuned_weights: bool,
}
```

### Search Integration

```rust
impl SearchEngine {
    pub fn evaluate_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        // Use tuned evaluation in search
        self.position_evaluator.evaluate_with_tuned_weights(board, player, captured_pieces)
    }
    
    pub fn quiescence_search(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces, alpha: i32, beta: i32, depth: u8) -> i32 {
        // Use tuned evaluation at leaf nodes
        if depth <= 0 {
            return self.evaluate_position(board, player, captured_pieces);
        }
        
        // ... rest of quiescence search implementation
    }
}
```

## Performance Optimization

### Caching and Memoization

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct PositionHash {
    board_hash: u64,
    player: Player,
    captured_pieces_hash: u64,
}

impl Hash for PositionHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.board_hash.hash(state);
        self.player.hash(state);
        self.captured_pieces_hash.hash(state);
    }
}

impl Engine {
    pub fn new_with_cache(config: EngineConfig, cache_size: usize) -> Result<Self, EngineError> {
        let mut engine = Engine::new(config)?;
        engine.evaluation_cache = Some(HashMap::with_capacity(cache_size));
        Ok(engine)
    }
    
    pub fn evaluate_position_cached(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        if let Some(ref mut cache) = self.evaluation_cache {
            let position_hash = self.create_position_hash(board, player, captured_pieces);
            
            if let Some(&cached_score) = cache.get(&position_hash) {
                return cached_score;
            }
            
            let score = self.evaluate_position(board, player, captured_pieces);
            cache.insert(position_hash, score);
            
            // Limit cache size
            if cache.len() > self.max_cache_size {
                self.prune_cache();
            }
            
            score
        } else {
            self.evaluate_position(board, player, captured_pieces)
        }
    }
    
    fn create_position_hash(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> PositionHash {
        PositionHash {
            board_hash: board.hash(),
            player,
            captured_pieces_hash: captured_pieces.hash(),
        }
    }
    
    fn prune_cache(&mut self) {
        if let Some(ref mut cache) = self.evaluation_cache {
            if cache.len() > self.max_cache_size {
                // Remove oldest entries (simple LRU approximation)
                let keys_to_remove: Vec<_> = cache.keys().take(cache.len() / 2).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }
    }
}
```

### Parallel Evaluation

```rust
use rayon::prelude::*;

impl Engine {
    pub fn evaluate_positions_parallel(&mut self, positions: &[(BitboardBoard, Player, CapturedPieces)]) -> Vec<i32> {
        positions.par_iter().map(|(board, player, captured_pieces)| {
            self.evaluate_position(board, *player, captured_pieces)
        }).collect()
    }
    
    pub fn analyze_positions_parallel(&mut self, positions: &[(BitboardBoard, Player, CapturedPieces)]) -> Vec<PositionAnalysis> {
        positions.par_iter().map(|(board, player, captured_pieces)| {
            let features = self.get_position_features(board, *player, captured_pieces);
            let score = self.evaluate_position(board, *player, captured_pieces);
            
            PositionAnalysis {
                score,
                features,
                using_tuned_weights: self.evaluator.is_using_tuned_weights(),
            }
        }).collect()
    }
}
```

## Fallback Mechanisms

### Automatic Fallback

```rust
impl Engine {
    pub fn evaluate_with_fallback(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        if self.evaluator.is_using_tuned_weights() {
            match self.evaluator.evaluate_with_tuned_weights(board, player, captured_pieces) {
                Ok(score) => score,
                Err(e) => {
                    println!("Warning: Tuned evaluation failed: {}. Falling back to default evaluation.", e);
                    self.evaluator.set_use_tuned_weights(false);
                    self.evaluator.evaluate(board, player, captured_pieces)
                }
            }
        } else {
            self.evaluator.evaluate(board, player, captured_pieces)
        }
    }
}
```

### Graceful Degradation

```rust
impl Engine {
    pub fn evaluate_with_graceful_degradation(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        // Try tuned evaluation first
        if self.evaluator.is_using_tuned_weights() {
            match self.evaluator.evaluate_with_tuned_weights(board, player, captured_pieces) {
                Ok(score) => return score,
                Err(WeightError::FeatureCountMismatch { .. }) => {
                    println!("Warning: Feature count mismatch. Using partial tuned evaluation.");
                    return self.evaluate_partial_tuned(board, player, captured_pieces);
                }
                Err(e) => {
                    println!("Warning: Tuned evaluation failed: {}. Falling back to default.", e);
                    self.evaluator.set_use_tuned_weights(false);
                }
            }
        }
        
        // Fallback to default evaluation
        self.evaluator.evaluate(board, player, captured_pieces)
    }
    
    fn evaluate_partial_tuned(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        let features = self.evaluator.get_evaluation_features(board, player, captured_pieces);
        let weight_manager = self.evaluator.get_weight_manager();
        
        // Use available weights and fallback for missing features
        let mut score = 0.0;
        let weights = weight_manager.get_weights();
        
        for (i, &feature) in features.iter().enumerate() {
            if i < weights.len() {
                score += feature * weights[i];
            } else {
                // Fallback to default evaluation for missing weights
                score += feature * self.get_default_weight(i);
            }
        }
        
        score as i32
    }
    
    fn get_default_weight(&self, feature_index: usize) -> f64 {
        // Return default weights for features not covered by tuned weights
        match feature_index {
            0..=15 => 1.0,  // Material features
            16..=815 => 0.5, // Positional features
            _ => 0.1,        // Other features
        }
    }
}
```

## Configuration Management

### Configuration Files

#### Engine Configuration

```json
{
  "search": {
    "depth": 15,
    "time_limit": 5000
  },
  "evaluation": {
    "weights": {
      "enabled": true,
      "path": "tuned_weights.json",
      "fallback_enabled": true,
      "performance_monitoring": true
    }
  },
  "logging": {
    "level": "info",
    "file": "engine.log"
  }
}
```

#### Weight Configuration

```json
{
  "weights": {
    "enabled": true,
    "path": "tuned_weights.json",
    "fallback_enabled": true,
    "performance_monitoring": true,
    "cache_size": 10000,
    "validation_enabled": true
  }
}
```

### Dynamic Configuration

```rust
impl Engine {
    pub fn update_config(&mut self, config: EngineConfig) -> Result<(), EngineError> {
        // Update weight configuration
        if config.weights.enabled != self.config.weights.enabled {
            if config.weights.enabled {
                self.enable_weights()?;
            } else {
                self.disable_weights();
            }
        }
        
        if let Some(new_path) = &config.weights.path {
            if Some(new_path) != self.config.weights.path.as_ref() {
                self.reload_weights(new_path)?;
            }
        }
        
        self.config = config;
        Ok(())
    }
    
    pub fn get_weight_status(&self) -> WeightStatus {
        let weight_manager = self.evaluator.get_weight_manager();
        
        WeightStatus {
            enabled: self.evaluator.is_using_tuned_weights(),
            loaded: weight_manager.has_weights(),
            stats: weight_manager.get_stats().clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeightStatus {
    pub enabled: bool,
    pub loaded: bool,
    pub stats: PerformanceStats,
}
```

## Monitoring and Debugging

### Performance Monitoring

```rust
impl Engine {
    pub fn get_evaluation_stats(&self) -> EvaluationStats {
        let weight_manager = self.evaluator.get_weight_manager();
        let stats = weight_manager.get_stats();
        
        EvaluationStats {
            total_evaluations: stats.evaluation_calls,
            tuned_evaluations: if self.evaluator.is_using_tuned_weights() { stats.evaluation_calls } else { 0 },
            default_evaluations: if self.evaluator.is_using_tuned_weights() { 0 } else { stats.evaluation_calls },
            average_time_us: stats.avg_evaluation_time_us,
            cache_hit_rate: stats.cache_hit_rate,
            weight_load_time: stats.weight_load_time,
        }
    }
    
    pub fn reset_evaluation_stats(&mut self) {
        let weight_manager = self.evaluator.get_weight_manager_mut();
        weight_manager.reset_stats();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationStats {
    pub total_evaluations: u64,
    pub tuned_evaluations: u64,
    pub default_evaluations: u64,
    pub average_time_us: f64,
    pub cache_hit_rate: f64,
    pub weight_load_time: Option<Duration>,
}
```

### Debug Information

```rust
impl Engine {
    pub fn debug_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> DebugInfo {
        let features = self.get_position_features(board, player, captured_pieces);
        let score = self.evaluate_position(board, player, captured_pieces);
        let weight_manager = self.evaluator.get_weight_manager();
        
        DebugInfo {
            position_fen: board.to_fen(),
            features: features,
            score: score,
            using_tuned_weights: self.evaluator.is_using_tuned_weights(),
            weight_info: if weight_manager.has_weights() {
                Some(WeightInfo {
                    feature_count: weight_manager.get_weights().len(),
                    enabled: self.evaluator.is_using_tuned_weights(),
                })
            } else {
                None
            },
            evaluation_time_us: 0, // Would be measured in real implementation
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub position_fen: String,
    pub features: Vec<f64>,
    pub score: i32,
    pub using_tuned_weights: bool,
    pub weight_info: Option<WeightInfo>,
    pub evaluation_time_us: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeightInfo {
    pub feature_count: usize,
    pub enabled: bool,
}
```

## Best Practices

### Error Handling

```rust
impl Engine {
    pub fn safe_evaluate(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Result<i32, EvaluationError> {
        // Validate inputs
        if features.len() != NUM_EVAL_FEATURES {
            return Err(EvaluationError::InvalidFeatureCount(features.len()));
        }
        
        // Attempt tuned evaluation
        if self.evaluator.is_using_tuned_weights() {
            match self.evaluator.evaluate_with_tuned_weights(board, player, captured_pieces) {
                Ok(score) => {
                    // Validate score
                    if score.is_finite() && score.abs() < 10000 {
                        return Ok(score);
                    } else {
                        return Err(EvaluationError::InvalidScore(score));
                    }
                }
                Err(e) => {
                    // Log error and fallback
                    log::warn!("Tuned evaluation failed: {}", e);
                    if self.config.weights.fallback_enabled {
                        return self.safe_default_evaluate(board, player, captured_pieces);
                    } else {
                        return Err(EvaluationError::WeightError(e));
                    }
                }
            }
        }
        
        // Default evaluation
        self.safe_default_evaluate(board, player, captured_pieces)
    }
    
    fn safe_default_evaluate(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Result<i32, EvaluationError> {
        let score = self.evaluator.evaluate(board, player, captured_pieces);
        
        if score.is_finite() && score.abs() < 10000 {
            Ok(score)
        } else {
            Err(EvaluationError::InvalidScore(score))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    #[error("Invalid feature count: {0}")]
    InvalidFeatureCount(usize),
    #[error("Invalid score: {0}")]
    InvalidScore(i32),
    #[error("Weight error: {0}")]
    WeightError(#[from] WeightError),
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),
}
```

### Testing Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weight_loading() {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        
        // Test loading valid weights
        assert!(engine.reload_weights("test_weights.json").is_ok());
        assert!(engine.evaluator.is_using_tuned_weights());
        
        // Test loading invalid weights
        assert!(engine.reload_weights("invalid_weights.json").is_err());
        assert!(!engine.evaluator.is_using_tuned_weights());
    }
    
    #[test]
    fn test_fallback_mechanism() {
        let config = EngineConfig {
            weights: WeightConfig {
                enabled: true,
                path: Some("nonexistent_weights.json".into()),
                fallback_enabled: true,
                performance_monitoring: false,
            },
            ..Default::default()
        };
        
        let mut engine = Engine::new(config).unwrap();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Should fallback to default evaluation
        let score = engine.evaluate_with_fallback(&board, Player::Black, &captured_pieces);
        assert!(score.is_finite());
    }
    
    #[test]
    fn test_performance_monitoring() {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Perform multiple evaluations
        for _ in 0..100 {
            engine.evaluate_position(&board, Player::Black, &captured_pieces);
        }
        
        let stats = engine.get_evaluation_stats();
        assert_eq!(stats.total_evaluations, 100);
        assert!(stats.average_time_us > 0.0);
    }
}
```

### Memory Management

```rust
impl Engine {
    pub fn optimize_memory_usage(&mut self) {
        // Clear evaluation cache if it's too large
        if let Some(ref mut cache) = self.evaluation_cache {
            if cache.len() > self.max_cache_size {
                cache.clear();
            }
        }
        
        // Reset weight manager statistics
        if self.config.weights.performance_monitoring {
            let weight_manager = self.evaluator.get_weight_manager_mut();
            weight_manager.reset_stats();
        }
    }
    
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let mut total_memory = 0;
        
        // Cache memory usage
        if let Some(ref cache) = self.evaluation_cache {
            total_memory += cache.len() * std::mem::size_of::<(PositionHash, i32)>();
        }
        
        // Weight memory usage
        let weight_manager = self.evaluator.get_weight_manager();
        if weight_manager.has_weights() {
            total_memory += weight_manager.get_weights().len() * std::mem::size_of::<f64>();
        }
        
        MemoryUsage {
            total_bytes: total_memory,
            cache_bytes: if let Some(ref cache) = self.evaluation_cache {
                cache.len() * std::mem::size_of::<(PositionHash, i32)>()
            } else {
                0
            },
            weights_bytes: if weight_manager.has_weights() {
                weight_manager.get_weights().len() * std::mem::size_of::<f64>()
            } else {
                0
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_bytes: usize,
    pub cache_bytes: usize,
    pub weights_bytes: usize,
}
```

## Next Steps

- [API Documentation](API_DOCUMENTATION.md) for detailed API reference
- [Code Examples](CODE_EXAMPLES.md) for extending the system
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) for optimization tips
- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for problem resolution
