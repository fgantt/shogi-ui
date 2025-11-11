# Pattern Recognition API Documentation

**Version**: 1.0  
**Date**: October 8, 2025

## Overview

The Pattern Recognition system provides comprehensive tactical, positional, and endgame pattern detection for the Shogi engine. This document covers the complete API for all pattern recognition modules.

---

## Module Overview

| Module | Purpose | Lines | Tests |
|--------|---------|-------|-------|
| `piece_square_tables` | Piece positioning evaluation | 729 | 18 |
| `position_features` | Pawn structure, king safety, mobility | 936 | 24 |
| `pattern_config` | Configuration system | 748 | 18 |
| `tactical_patterns` | Fork, pin, skewer detection | 819 | 8 |
| `positional_patterns` | Center, outpost, space analysis | 574 | 5 |
| `endgame_patterns` | Zugzwang, opposition, fortress | Enhanced | Existing |
| `pattern_cache` | Result caching with LRU | 461 | 16 |
| `pattern_optimization` | Fast detection, memory optimization | 471 | 9 |
| `pattern_advanced` | ML, dynamic selection, analytics | 487 | 14 |
| `pattern_search_integration` | Move ordering, pruning | 323 | 8 |
| `pattern_comprehensive_tests` | Test suite | 289 | 9 |

---

## Core Pattern Recognition APIs

### Tactical Patterns

```rust
use crate::evaluation::tactical_patterns::TacticalPatternRecognizer;

// Create recognizer
let mut recognizer = TacticalPatternRecognizer::new();

// Evaluate all tactical patterns
let score = recognizer.evaluate_tactics(&board, player);
// Returns: TaperedScore { mg, eg }

// Access statistics
let stats = recognizer.stats();
println!("Forks found: {}", stats.forks_found.load(Ordering::Relaxed));
println!("Pins found: {}", stats.pins_found.load(Ordering::Relaxed));
```

**Detects**:
- Forks (double attacks)
- Pins (immobile pieces)
- Skewers (through-piece attacks)
- Discovered attacks
- Knight forks
- Back rank threats

---

### Positional Patterns

```rust
use crate::evaluation::positional_patterns::PositionalPatternAnalyzer;

// Create analyzer
let mut analyzer = PositionalPatternAnalyzer::new();

// Evaluate all positional patterns
let score = analyzer.evaluate_position(&board, player, &CapturedPieces::new());
// Returns: TaperedScore { mg, eg }

// Access statistics
let stats = analyzer.stats();
println!("Outposts found: {}", stats.outposts_found);
println!("Weak squares found: {}", stats.weak_squares_found);
```

**Evaluates**:
- Center control (3x3 + 5x5)
- Outpost detection
- Weak square identification
- Piece activity
- Space advantage
- Tempo

---

### Endgame Patterns

```rust
use crate::evaluation::endgame_patterns::EndgamePatternEvaluator;

// Create evaluator
let mut evaluator = EndgamePatternEvaluator::new();

// Evaluate endgame patterns
let score = evaluator.evaluate_endgame(&board, player, &captured_pieces);
// Returns: TaperedScore { mg, eg }
```

**Detects**:
- King activity
- Passed pawns
- Mate patterns
- Zugzwang
- Opposition
- Triangulation
- Piece vs pawns
- Fortress patterns

---

### Pattern Configuration

```rust
use crate::evaluation::pattern_config::PatternConfig;

// Create default configuration
let mut config = PatternConfig::default();

// Enable all patterns
config.enable_all();

// Adjust weights
config.set_king_safety_weight(1.5);
config.set_tactical_patterns_weight(1.2);

// Validate configuration
config.validate()?;

// Save to JSON
let json = config.to_json()?;
std::fs::write("pattern_config.json", json)?;

// Load from JSON
let loaded_config = PatternConfig::from_json(&json)?;

// Runtime update
active_config.update_from(&loaded_config)?;
```

---

### Pattern Caching

```rust
use crate::evaluation::pattern_cache::{PatternCache, CachedPatternResult};

// Create cache
let mut cache = PatternCache::new(100_000);

// Store result
let result = CachedPatternResult {
    tactical_score: (50, 30),
    positional_score: (40, 25),
    endgame_score: (20, 35),
    age: 0,
};
cache.store(position_hash, result);

// Lookup result
if let Some(cached) = cache.lookup(position_hash) {
    // Use cached result (90% faster!)
    return cached.tactical_score;
}

// Monitor performance
println!("Hit rate: {:.1}%", cache.hit_rate() * 100.0);
println!("Usage: {:.1}%", cache.usage_percent());

// Manage cache
cache.resize(50_000);
cache.clear();
cache.reset_stats();
```

---

### Search Integration

```rust
use crate::evaluation::pattern_search_integration::PatternSearchIntegrator;

// Create integrator
let mut integrator = PatternSearchIntegrator::new();

// Order moves by patterns
let ordered_moves = integrator.order_moves_by_patterns(&board, &moves, player);
for (mv, score) in ordered_moves {
    println!("Move: {:?}, Pattern Score: {}", mv, score);
}

// Check pruning
if integrator.should_prune_by_patterns(&board, player, depth, alpha, beta) {
    return; // Prune this branch
}

// Quiescence evaluation
let qs_score = integrator.evaluate_in_quiescence(&board, player);
```

---

## Integrated Evaluation API

### Using PositionEvaluator (Recommended)

```rust
use crate::evaluation::PositionEvaluator;

// Create evaluator (patterns enabled by default)
let mut evaluator = PositionEvaluator::new();

// Evaluate position (includes all patterns automatically)
let score = evaluator.evaluate(&mut board, player, &captured_pieces);

// Check if integrated evaluator is active
if evaluator.is_using_integrated_evaluator() {
    println!("Using integrated evaluation with patterns");
}

// Access integrated evaluator
if let Some(integrated) = evaluator.get_integrated_evaluator() {
    // Access pattern components if needed
}
```

### Using IntegratedEvaluator Directly

```rust
use crate::evaluation::integration::IntegratedEvaluator;

// Create evaluator
let evaluator = IntegratedEvaluator::new();

// Evaluate with all patterns
let score = evaluator.evaluate(&board, player, &captured_pieces);

// Get statistics
evaluator.enable_statistics();
let stats = evaluator.get_statistics();
```

---

## Configuration Examples

### Enable/Disable Specific Patterns

```rust
use crate::evaluation::integration::{IntegratedEvaluationConfig, ComponentFlags};

let mut config = IntegratedEvaluationConfig::default();

// Disable tactical patterns
config.components.tactical_patterns = false;

// Enable only core patterns
config.components = ComponentFlags::minimal();

// Custom configuration
config.components = ComponentFlags {
    material: true,
    piece_square_tables: true,
    position_features: true,
    opening_principles: false,
    endgame_patterns: true,
    tactical_patterns: true,
    positional_patterns: false,
};

let evaluator = IntegratedEvaluator::with_config(config);
```

### WASM-Optimized Configuration

```rust
use crate::evaluation::wasm_compatibility::WasmEvaluationOptimizer;

// Get WASM-optimized components
let components = WasmEvaluationOptimizer::get_wasm_components(true);

let mut config = IntegratedEvaluationConfig::default();
config.components = components;

// Patterns disabled by default in WASM for size
// Enable if needed:
config.components.tactical_patterns = true;
```

---

## Performance APIs

### Optimized Pattern Detection

```rust
use crate::evaluation::pattern_optimization::OptimizedPatternDetector;

let mut detector = OptimizedPatternDetector::new();

// Fast detection
let result = detector.detect_patterns_fast(&board, player);

if result.has_fork {
    // Handle fork quickly
}

// Performance monitoring
println!("Average time: {}ns", detector.avg_time_ns());
```

### Compact Storage

```rust
use crate::evaluation::pattern_optimization::CompactPatternStorage;

let mut storage = CompactPatternStorage::new();

// Set pattern flags (bit-packed)
storage.set_pattern(0);  // Fork
storage.set_pattern(2);  // Outpost

// Store scores (i16 for memory efficiency)
storage.set_tactical_score(150, 90);

// Check patterns
if storage.has_pattern(0) {
    let (mg, eg) = storage.get_tactical_score();
    println!("Tactical: {}mg / {}eg", mg, eg);
}
```

---

## Advanced Features APIs

### Dynamic Pattern Selection

```rust
use crate::evaluation::pattern_advanced::AdvancedPatternSystem;

let system = AdvancedPatternSystem::new();

// Select patterns by game phase
let patterns = system.select_patterns(&board, game_phase);

if patterns.enable_tactical {
    // Use tactical patterns
}

// Get phase-specific weights
let weights = patterns.weights;  // Vec<f32> with 8 weights
```

### Pattern Explanation

```rust
// Get human-readable pattern explanations
let explanations = system.explain_patterns(&board, player);

for explanation in explanations {
    println!("{}: {} (+{})",
        explanation.pattern_name,
        explanation.description,
        explanation.value
    );
}
```

### Pattern Analytics

```rust
let analytics = system.get_analytics();

// Record patterns
analytics.record_pattern("fork", 100);
analytics.record_pattern("outpost", 60);

// Get statistics
let frequency = analytics.get_frequency("fork");
let avg_value = analytics.get_average_value("fork");

let stats = analytics.get_stats();
println!("Total patterns: {}", stats.total_patterns);
```

---

## Testing APIs

### Comprehensive Test Suite

```rust
use crate::evaluation::pattern_comprehensive_tests::PatternTestSuite;

let mut suite = PatternTestSuite::new();

// Run all tests
if suite.run_all_tests() {
    println!("All pattern tests passed!");
}

// Run specific test categories
suite.run_unit_tests();
suite.run_integration_tests();
suite.run_performance_tests();
suite.run_accuracy_tests();
suite.run_regression_tests();

// Print summary
suite.print_summary();

// Get results
let results = suite.results();
println!("Pass rate: {:.1}%", results.pass_rate() * 100.0);
```

---

## Performance Considerations

### Caching for Performance

**Cache Hit Performance**: ~90% faster than recomputation

```rust
// Enable pattern cache (default: 100K entries)
let mut evaluator = IntegratedEvaluator::new();
// Pattern cache automatically used

// Monitor cache performance
// (cache monitoring methods would be added to IntegratedEvaluator)
```

### Memory Optimization

**Compact Storage**: 15 bytes data + 64-byte alignment

```rust
// Use CompactPatternStorage for memory-constrained environments
let storage = CompactPatternStorage::new();
// Size: 64 bytes (cache-line aligned)
// vs unoptimized: 24 bytes (unaligned)
```

---

## Error Handling

All pattern recognition methods return `TaperedScore` or safe values:

```rust
// No panics - safe defaults
let score = recognizer.evaluate_tactics(&board, player);
// Returns: TaperedScore::default() if no patterns found

// Configuration validation
match config.validate() {
    Ok(()) => println!("Configuration valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

---

## Common Patterns

### Basic Evaluation with Patterns

```rust
// Most common usage - automatic pattern evaluation
let mut evaluator = PositionEvaluator::new();
let score = evaluator.evaluate(&mut board, player, &captured_pieces);
// Patterns automatically included in score
```

### Custom Pattern Weights

```rust
let mut config = PatternConfig::default();
config.set_tactical_patterns_weight(1.5);  // Emphasize tactics
config.set_positional_patterns_weight(0.8);  // Reduce positional
```

### Performance Profiling

```rust
use std::time::Instant;

let start = Instant::now();
let score = evaluator.evaluate(&board, player, &captured_pieces);
let elapsed = start.elapsed();

println!("Evaluation time: {:?}", elapsed);
// Typical: <1ms (uncached), ~50μs (cached)
```

---

## See Also

- `PATTERN_RECOGNITION_GUIDE.md` - User guide
- `PATTERN_TUNING_GUIDE.md` - Tuning and optimization
- `PATTERN_BEST_PRACTICES.md` - Best practices
- `PATTERN_TROUBLESHOOTING.md` - Common issues and solutions
- `INTEGRATION_VERIFICATION_REPORT.md` - Integration verification

---

**API Documentation Complete** ✅

For usage examples, see `PATTERN_RECOGNITION_USAGE_EXAMPLES.md`.
