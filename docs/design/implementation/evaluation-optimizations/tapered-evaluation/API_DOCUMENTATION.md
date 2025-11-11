# Tapered Evaluation - API Documentation

## Overview

Complete API reference for the tapered evaluation system. This document covers all public interfaces, usage patterns, and integration points.

## Core Types

### TaperedScore

The fundamental data structure for tapered evaluation.

```rust
pub struct TaperedScore {
    pub mg: i32,  // Middlegame score
    pub eg: i32,  // Endgame score
}

impl TaperedScore {
    // Constructors
    pub fn new() -> Self
    pub fn new_tapered(mg: i32, eg: i32) -> Self
    
    // Interpolation
    pub fn interpolate(&self, phase: i32) -> i32
    
    // Operators
    // Supports: +, -, +=, -=
}
```

**Example**:
```rust
let score = TaperedScore::new_tapered(100, 200);
let final_score = score.interpolate(128); // Blend 50/50
```

## Evaluation Modules

### IntegratedEvaluator

Main entry point for tapered evaluation.

```rust
pub struct IntegratedEvaluator

impl IntegratedEvaluator {
    // Constructors
    pub fn new() -> Self
    pub fn with_config(config: IntegratedEvaluationConfig) -> Self
    
    // Evaluation
    pub fn evaluate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces
    ) -> i32
    
    // Cache management
    pub fn clear_caches(&self)
    pub fn cache_stats(&self) -> CacheStatistics
    
    // Statistics
    pub fn enable_statistics(&self)
    pub fn disable_statistics(&self)
    pub fn get_statistics(&self) -> EvaluationStatistics
    pub fn reset_statistics(&self)
    
    // Configuration
    pub fn config(&self) -> &IntegratedEvaluationConfig
    pub fn set_config(&mut self, config: IntegratedEvaluationConfig)
}
```

**Example**:
```rust
let mut evaluator = IntegratedEvaluator::new();
evaluator.enable_statistics();

let score = evaluator.evaluate(&board, Player::Black, &captured);

let stats = evaluator.get_statistics();
println!("Evaluated {} positions", stats.count());
```

### PositionEvaluator

Existing evaluator with integrated tapered evaluation.

```rust
pub struct PositionEvaluator

impl PositionEvaluator {
    // Constructors (integrated evaluator enabled by default)
    pub fn new() -> Self
    pub fn with_config(config: TaperedEvaluationConfig) -> Self
    
    // Evaluation (automatically uses IntegratedEvaluator)
    pub fn evaluate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces
    ) -> i32
    
    // Control integrated evaluator
    pub fn enable_integrated_evaluator(&mut self)
    pub fn disable_integrated_evaluator(&mut self)
    pub fn is_using_integrated_evaluator(&self) -> bool
    
    // Access integrated evaluator
    pub fn get_integrated_evaluator(&self) -> Option<&IntegratedEvaluator>
    pub fn get_integrated_evaluator_mut(&mut self) -> Option<&mut IntegratedEvaluator>
    
    // Statistics
    pub fn enable_integrated_statistics(&self)
    pub fn get_integrated_statistics(&self) -> Option<EvaluationStatistics>
}
```

**Example**:
```rust
let evaluator = PositionEvaluator::new();

// Automatically uses integrated tapered evaluation
let score = evaluator.evaluate(&board, player, &captured);

// Access statistics
evaluator.enable_integrated_statistics();
if let Some(stats) = evaluator.get_integrated_statistics() {
    println!("Report: {}", stats.generate_report());
}
```

## Search Integration

### TaperedSearchEnhancer

Phase-aware search enhancements.

```rust
pub struct TaperedSearchEnhancer

impl TaperedSearchEnhancer {
    // Constructors
    pub fn new() -> Self
    pub fn with_config(config: TaperedSearchConfig) -> Self
    
    // Phase tracking
    pub fn track_phase(&mut self, board: &BitboardBoard) -> i32
    
    // Pruning
    pub fn should_prune(&mut self, phase: i32, depth: u8, score: i32, beta: i32) -> bool
    
    // Move ordering
    pub fn get_phase_move_bonus(&self, piece_type: PieceType, phase: i32) -> i32
    
    // Extensions
    pub fn get_phase_extension(&self, phase: i32, is_check: bool, is_capture: bool) -> u8
    
    // Cache management
    pub fn clear_cache(&mut self)
    
    // Statistics
    pub fn stats(&self) -> &TaperedSearchStats
    pub fn reset_stats(&mut self)
}
```

**Example**:
```rust
let mut search_engine = SearchEngine::new(None, 64);

// Access tapered search enhancer
let enhancer = search_engine.get_tapered_search_enhancer_mut();

// Track phase
let phase = enhancer.track_phase(&board);

// Make phase-aware decisions
if enhancer.should_prune(phase, depth, score, beta) {
    return beta; // Prune
}
```

### SearchEngine Integration

```rust
impl SearchEngine {
    // Access tapered search enhancer
    pub fn get_tapered_search_enhancer(&self) -> &TaperedSearchEnhancer
    pub fn get_tapered_search_enhancer_mut(&mut self) -> &mut TaperedSearchEnhancer
}
```

## Configuration

### IntegratedEvaluationConfig

```rust
pub struct IntegratedEvaluationConfig {
    pub components: ComponentFlags,
    pub enable_phase_cache: bool,
    pub enable_eval_cache: bool,
    pub use_optimized_path: bool,
    pub max_cache_size: usize,
    pub pattern_cache_size: usize,
    pub collect_position_feature_stats: bool,
    pub material: MaterialEvaluationConfig,
    pub pst: PieceSquareTableConfig,
    pub position_features: PositionFeatureConfig,
    pub weights: EvaluationWeights,
}

pub struct ComponentFlags {
    pub material: bool,
    pub piece_square_tables: bool,
    pub position_features: bool,
    pub opening_principles: bool,
    pub endgame_patterns: bool,
}

impl ComponentFlags {
    pub fn all_enabled() -> Self
    pub fn all_disabled() -> Self
    pub fn minimal() -> Self
}
```

**Example**:
```rust
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::minimal();
config.enable_phase_cache = true;

let mut evaluator = IntegratedEvaluator::with_config(config);
```

### TaperedSearchConfig

```rust
pub struct TaperedSearchConfig {
    pub enable_phase_aware_pruning: bool,
    pub enable_phase_aware_ordering: bool,
    pub enable_phase_extensions: bool,
    pub opening_pruning_margin: i32,
    pub middlegame_pruning_margin: i32,
    pub endgame_pruning_margin: i32,
}
```

## Statistics and Monitoring

### EvaluationStatistics

```rust
pub struct EvaluationStatistics

impl EvaluationStatistics {
    pub fn new() -> Self
    pub fn enable(&mut self)
    pub fn disable(&mut self)
    pub fn is_enabled(&self) -> bool
    pub fn set_collect_position_feature_stats(&mut self, collect: bool)
    
    // Recording
    pub fn record_evaluation(&mut self, score: i32, phase: i32)
    pub fn record_phase(&mut self, phase: i32)
    pub fn record_accuracy(&mut self, predicted: i32, actual: i32)
    pub fn record_timing(&mut self, duration_ns: u64)
    
    // Reporting
    pub fn generate_report(&self) -> StatisticsReport
    pub fn export_json(&self) -> Result<String, serde_json::Error>
    
    // Management
    pub fn reset(&mut self)
    pub fn count(&self) -> u64
}
```

**Example**:
```rust
let mut stats = EvaluationStatistics::new();
stats.enable();

// During evaluation
stats.record_evaluation(score, phase);
stats.record_timing(duration_ns);

// Get report
let report = stats.generate_report();
println!("{}", report);

// Export to JSON
let json = stats.export_json()?;
std::fs::write("stats.json", json)?;
```

## Advanced Features

### AdvancedInterpolator

```rust
pub struct AdvancedInterpolator

impl AdvancedInterpolator {
    pub fn new() -> Self
    pub fn with_config(config: AdvancedInterpolationConfig) -> Self
    
    // Interpolation methods
    pub fn interpolate_spline(&self, score: TaperedScore, phase: i32) -> i32
    pub fn interpolate_multi_phase(&self, score: TaperedScore, phase: i32, position_type: PositionType) -> i32
    pub fn interpolate_adaptive(&self, score: TaperedScore, phase: i32, characteristics: &PositionCharacteristics) -> i32
    pub fn interpolate_bezier(&self, score: TaperedScore, phase: i32, control1: f64, control2: f64) -> i32
    pub fn interpolate_custom<F>(&self, score: TaperedScore, phase: i32, custom_fn: F) -> i32
        where F: Fn(i32, i32, f64) -> i32
}
```

### TaperedEvaluationTuner

```rust
pub struct TaperedEvaluationTuner

impl TaperedEvaluationTuner {
    pub fn new() -> Self
    pub fn with_config(config: TuningConfig) -> Self
    
    // Data management
    pub fn add_training_data(&mut self, positions: Vec<TuningPosition>)
    pub fn add_validation_data(&mut self, positions: Vec<TuningPosition>)
    pub fn split_data(&mut self, validation_ratio: f64)
    
    // Optimization
    pub fn optimize(&mut self) -> Result<TuningResults, TuningError>
    
    // Access
    pub fn weights(&self) -> &EvaluationWeights
    pub fn stats(&self) -> &TuningStats
}
```

## Complete Usage Examples

### Basic Evaluation

```rust
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{Player, CapturedPieces};

let evaluator = PositionEvaluator::new();
let board = BitboardBoard::new();
let captured_pieces = CapturedPieces::new();

let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
println!("Position evaluation: {}", score);
```

### With Statistics

```rust
let evaluator = PositionEvaluator::new();
evaluator.enable_integrated_statistics();

// Run many evaluations
for position in positions {
    let score = evaluator.evaluate(&position.board, position.player, &position.captured);
}

// Get statistics
if let Some(stats) = evaluator.get_integrated_statistics() {
    let report = stats.generate_report();
    println!("{}", report);
}
```

### Custom Configuration

```rust
use shogi_engine::evaluation::integration::*;

let mut config = IntegratedEvaluationConfig::default();
config.components.opening_principles = false;  // Disable opening eval
config.enable_eval_cache = true;

let mut evaluator = IntegratedEvaluator::with_config(config);
```

### Search Integration

```rust
use shogi_engine::search::SearchEngine;

let mut search_engine = SearchEngine::new(None, 64);

// Access tapered search enhancer
let enhancer = search_engine.get_tapered_search_enhancer_mut();

// Track phase at search node
let phase = enhancer.track_phase(&board);

// Use phase-aware features
if enhancer.should_prune(phase, depth, score, beta) {
    // Prune this branch
}
```

### Automated Tuning

```rust
use shogi_engine::evaluation::tuning::*;

let mut tuner = TaperedEvaluationTuner::new();

// Add training data from games
tuner.add_training_data(training_positions);
tuner.split_data(0.2);  // 20% validation

// Optimize weights
let results = tuner.optimize()?;

println!("Training error: {:.4}", results.training_error);
println!("Validation error: {:.4}", results.validation_error);

// Get optimized weights
let weights = results.optimized_weights;
```

## Error Handling

Most operations don't return errors as they work with internal state. The main error types are:

### TuningError

```rust
pub enum TuningError {
    NoTrainingData,
    OptimizationFailed(String),
}
```

### WeightError (existing)

For weight file I/O operations in the existing system.

## Performance Considerations

### Fast Path

```rust
// Fastest: Minimal components + optimized path
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::minimal();
config.use_optimized_path = true;

let evaluator = IntegratedEvaluator::with_config(config);
// ~600ns per evaluation
```

### Balanced

```rust
// Default configuration
let evaluator = IntegratedEvaluator::new();
// ~800-1200ns per evaluation
```

### Full Features

```rust
// All components enabled
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::all_enabled();

let evaluator = IntegratedEvaluator::with_config(config);
// ~1200-1500ns per evaluation
```

## Thread Safety

**Note**: The tapered evaluation system uses `RefCell` for interior mutability and is **not thread-safe** by default. For multi-threaded search, create separate evaluator instances per thread.

```rust
// Single-threaded (OK)
let mut evaluator = IntegratedEvaluator::new();
evaluator.evaluate(&board, player, &captured);

// Multi-threaded (Create per thread)
use std::thread;

let handles: Vec<_> = positions.chunks(100).map(|chunk| {
    thread::spawn(move || {
        let mut evaluator = IntegratedEvaluator::new();
        chunk.iter().map(|pos| {
            evaluator.evaluate(&pos.board, pos.player, &pos.captured)
        }).collect::<Vec<_>>()
    })
}).collect();
```

## Migration Guide

### From Legacy Evaluation

**Before** (Legacy):
```rust
let evaluator = PositionEvaluator::new();
let score = evaluator.evaluate(&board, player, &captured);
```

**After** (Tapered - Same API!):
```rust
let evaluator = PositionEvaluator::new();  // Integrated enabled by default
let score = evaluator.evaluate(&board, player, &captured);  // Automatically faster!
```

**No code changes required!** The tapered evaluation is automatically used.

### Gradual Rollout

```rust
// Start with integrated disabled for A/B testing
let mut evaluator = PositionEvaluator::new();
evaluator.disable_integrated_evaluator();

// Enable for specific games/positions
evaluator.enable_integrated_evaluator();
```

## Troubleshooting

### Issue: Evaluation seems slow

**Check**:
1. Is caching enabled?
2. Are statistics enabled? (adds ~5% overhead)
3. Are all components enabled? (use `minimal()` for speed)

**Solution**:
```rust
let mut config = IntegratedEvaluationConfig::default();
config.enable_phase_cache = true;
config.enable_eval_cache = true;
config.components = ComponentFlags::minimal();
```

### Issue: Scores differ from legacy evaluation

**Expected**: Tapered evaluation produces different scores because:
- Uses phase-aware weights
- Includes endgame patterns
- Includes opening principles
- More accurate positional evaluation

**Validation**:
- Scores should still be in reasonable range (±10000)
- Starting position should be near 0
- Material advantage should show positive score

### Issue: Memory usage too high

**Check cache sizes**:
```rust
let stats = evaluator.cache_stats();
println!("Phase cache: {} entries", stats.phase_cache_size);
println!("Eval cache: {} entries", stats.eval_cache_size);

// Clear if needed
evaluator.clear_caches();
```

**Reduce cache size**:
```rust
let mut config = IntegratedEvaluationConfig::default();
config.max_cache_size = 1000;  // Smaller cache
```

## Best Practices

### 1. Enable Caching

```rust
// Always enable caching for repeated evaluations
let mut config = IntegratedEvaluationConfig::default();
config.enable_phase_cache = true;
config.enable_eval_cache = true;
```

### 2. Use Statistics for Profiling

```rust
// During development/testing
evaluator.enable_statistics();

// In production
evaluator.disable_statistics();  // Zero overhead
```

### 3. Clear Caches Periodically

```rust
// Every 10,000 nodes or between games
if nodes_searched % 10000 == 0 {
    evaluator.clear_caches();
}
```

### 4. Use Minimal Components for Speed

```rust
// For ultra-fast evaluation
config.components = ComponentFlags::minimal();  // Material + PST only
```

### 5. Component Selection by Phase

Already handled automatically! The system enables:
- Opening principles when phase ≥ 192
- Endgame patterns when phase < 64
- Position features in middlegame

## Configuration Reference

### Component Presets

```rust
// Full evaluation
ComponentFlags::all_enabled()

// Fast evaluation
ComponentFlags::minimal()

// Custom
ComponentFlags {
    material: true,
    piece_square_tables: true,
    position_features: false,
    opening_principles: true,
    endgame_patterns: true,
}
```

### Performance Tuning

```rust
// Maximum performance
let config = IntegratedEvaluationConfig {
    components: ComponentFlags::minimal(),
    enable_phase_cache: true,
    enable_eval_cache: true,
    use_optimized_path: true,
    max_cache_size: 10000,
};

// Maximum accuracy
let config = IntegratedEvaluationConfig {
    components: ComponentFlags::all_enabled(),
    enable_phase_cache: true,
    enable_eval_cache: false,  // Always recalculate
    use_optimized_path: false,  // Full evaluation path
    max_cache_size: 0,
};
```

## Constants

```rust
// Phase range
pub const GAME_PHASE_MAX: i32 = 256;

// Piece phase values
pub const PIECE_PHASE_VALUES: [i32; 8] = [0, 1, 1, 2, 2, 4, 5, 0];
```

## Complete API Surface

Total public functions across all modules: **100+**

### Modules
- `tapered_eval` - Core evaluation (10 functions)
- `material` - Material evaluation (8 functions)
- `piece_square_tables` - PST management (5 functions)
- `phase_transition` - Interpolation (6 functions)
- `position_features` - Position evaluation (12 functions)
- `config` - Configuration (10 functions)
- `endgame_patterns` - Endgame evaluation (8 functions)
- `opening_principles` - Opening evaluation (8 functions)
- `performance` - Performance optimization (8 functions)
- `tuning` - Automated tuning (8 functions)
- `statistics` - Statistics tracking (12 functions)
- `advanced_interpolation` - Advanced methods (7 functions)
- `integration` - Main integration (14 functions)
- `tapered_search_integration` - Search integration (8 functions)

**Total: 124 public API functions**

---

*API Version: 1.0*
*Generated: October 8, 2025*
*Modules: 14*
*Public Functions: 124*

