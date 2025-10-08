# Pattern Recognition Best Practices

**Version**: 1.0  
**Date**: October 8, 2025

## Best Practices for Pattern Recognition

### 1. Use Default Configuration

✅ **DO**: Use default PositionEvaluator
```rust
let mut evaluator = PositionEvaluator::new();
// Patterns enabled and balanced
```

❌ **DON'T**: Disable patterns unnecessarily
```rust
config.components.tactical_patterns = false;  // Usually not needed
```

### 2. Enable Caching

✅ **DO**: Let pattern cache work automatically
```rust
let evaluator = PositionEvaluator::new();
// Pattern cache active by default (100K entries)
```

✅ **DO**: Monitor cache performance
```rust
// Check hit rates periodically to ensure effective caching
```

### 3. Configure for Playing Style

✅ **DO**: Adjust weights for style
```rust
// Aggressive
config.set_tactical_patterns_weight(1.5);

// Positional
config.set_positional_patterns_weight(1.4);

// Defensive
config.set_king_safety_weight(1.6);
```

### 4. Validate Configurations

✅ **DO**: Always validate
```rust
match config.validate() {
    Ok(()) => { /* use config */ },
    Err(e) => eprintln!("Invalid config: {}", e),
}
```

❌ **DON'T**: Use invalid weights
```rust
config.set_king_safety_weight(-1.0);  // Invalid!
config.set_tactical_patterns_weight(15.0);  // Too large!
```

### 5. Use Appropriate Patterns for Game Phase

✅ **DO**: Trust phase-aware evaluation
```rust
// Pattern weights automatically adjusted by game phase
let score = evaluator.evaluate(&board, player, &captured_pieces);
```

✅ **DO**: Use dynamic selection for analysis
```rust
let system = AdvancedPatternSystem::new();
let patterns = system.select_patterns(&board, game_phase);
```

### 6. Performance Optimization

✅ **DO**: Use compact storage for memory-constrained environments
```rust
use CompactPatternStorage;  // 64 bytes vs 24 bytes unaligned
```

✅ **DO**: Enable pattern caching
```rust
// Automatic 90% speedup on cache hits
```

❌ **DON'T**: Disable caching unless necessary
```rust
// Only disable if memory is extremely limited
```

### 7. WASM Deployment

✅ **DO**: Use WASM-optimized configuration
```rust
let components = WasmEvaluationOptimizer::get_wasm_components(true);
// Patterns disabled by default for size
```

✅ **DO**: Enable patterns if binary size allows
```rust
config.components.tactical_patterns = true;  // +50KB acceptable?
```

### 8. Testing and Validation

✅ **DO**: Run comprehensive tests
```rust
let mut suite = PatternTestSuite::new();
suite.run_all_tests();
```

✅ **DO**: Validate against known positions
```rust
// Test with professional games
// Verify pattern detection accuracy
```

### 9. Error Handling

✅ **DO**: Handle configuration errors
```rust
match PatternConfig::from_json(&json) {
    Ok(config) => { /* use */ },
    Err(e) => { /* handle error */ },
}
```

✅ **DO**: Provide fallbacks
```rust
let score = pattern_result.unwrap_or(TaperedScore::default());
```

### 10. Documentation

✅ **DO**: Document custom configurations
```rust
// Document why custom weights are used
let mut config = PatternConfig::default();
config.set_tactical_patterns_weight(1.5);  // Aggressive style
```

---

## Performance Best Practices

### 1. Caching Strategy

**Best**: Use default cache size (100K entries)
```rust
// Provides good hit rate (60-80%) without excessive memory
```

**Good**: Adjust for memory constraints
```rust
let cache = PatternCache::new(50_000);  // Smaller cache
```

**Avoid**: Too small cache
```rust
let cache = PatternCache::new(100);  // Poor hit rate
```

### 2. Pattern Selection

**Best**: Enable all relevant patterns
```rust
ComponentFlags::all_enabled()  // Best accuracy
```

**Good**: Disable non-critical patterns
```rust
components.opening_principles = false;  // Save computation
```

**Avoid**: Disabling core patterns
```rust
components.tactical_patterns = false;  // Loses tactical awareness!
```

### 3. Memory Management

**Best**: Use compact storage
```rust
CompactPatternStorage  // 64-byte cache-line aligned
```

**Good**: Monitor memory usage
```rust
println!("Cache usage: {:.1}%", cache.usage_percent());
```

---

## Integration Best Practices

### 1. Evaluation Integration

✅ **DO**: Use integrated evaluator
```rust
// IntegratedEvaluator is used by default
let evaluator = PositionEvaluator::new();
```

✅ **DO**: Configure components appropriately
```rust
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::all_enabled();
```

### 2. Search Integration

✅ **DO**: Use pattern-based move ordering
```rust
let ordered = integrator.order_moves_by_patterns(&board, &moves, player);
// Better move ordering = more efficient search
```

✅ **DO**: Apply pattern-based pruning
```rust
if integrator.should_prune_by_patterns(&board, player, depth, alpha, beta) {
    return; // Prune safely
}
```

---

## Common Pitfalls to Avoid

### ❌ Pitfall 1: Disabling Patterns Without Testing

**Problem**: Disabling patterns without understanding impact

**Solution**: Test with and without patterns, measure accuracy

### ❌ Pitfall 2: Invalid Weight Ranges

**Problem**: Using weights outside 0.0-10.0 range

**Solution**: Always validate configurations

### ❌ Pitfall 3: Ignoring Game Phase

**Problem**: Using same pattern weights for all game phases

**Solution**: Use phase-aware evaluation (automatic with IntegratedEvaluator)

### ❌ Pitfall 4: Forgetting WASM Size Impact

**Problem**: Enabling all patterns in WASM without checking binary size

**Solution**: Use WASM-optimized defaults, enable patterns selectively

### ❌ Pitfall 5: Not Monitoring Cache Performance

**Problem**: Poor cache configuration leading to low hit rates

**Solution**: Monitor hit rates, adjust cache size as needed

---

## Recommended Configurations

### Balanced (Default)

```rust
PatternConfig::default()
// All weights: 1.0
// All patterns enabled
// Suitable for most use cases
```

### Aggressive

```rust
let mut config = PatternConfig::default();
config.set_tactical_patterns_weight(1.5);
config.set_piece_coordination_weight(1.3);
config.set_mobility_weight(1.2);
```

### Positional

```rust
let mut config = PatternConfig::default();
config.set_positional_patterns_weight(1.4);
config.set_pawn_structure_weight(1.3);
config.set_center_control_weight(1.2);
```

### Defensive

```rust
let mut config = PatternConfig::default();
config.set_king_safety_weight(1.6);
config.set_fortress_patterns_weight(1.4);
config.set_weak_square_weight(1.3);
```

### Fast (Performance)

```rust
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags {
    material: true,
    piece_square_tables: true,
    position_features: true,
    opening_principles: false,  // Disable for speed
    endgame_patterns: false,    // Disable for speed
    tactical_patterns: true,    // Keep tactical
    positional_patterns: false, // Disable for speed
};
```

---

## Monitoring and Debugging

### Check Pattern Detection

```rust
let mut recognizer = TacticalPatternRecognizer::new();
let score = recognizer.evaluate_tactics(&board, player);

let stats = recognizer.stats();
if stats.forks_found.load(Ordering::Relaxed) > 0 {
    println!("Fork detected!");
}
```

### Monitor Cache Performance

```rust
let cache_hit_rate = cache.hit_rate();
if cache_hit_rate < 0.5 {
    println!("⚠️ Low cache hit rate: {:.1}%", cache_hit_rate * 100.0);
    // Consider increasing cache size
}
```

### Track Pattern Statistics

```rust
let stats = analyzer.stats();
println!("Center control checks: {}", stats.center_control_checks);
println!("Outposts found: {}", stats.outposts_found);
```

---

## Summary of Best Practices

1. ✅ Use default configuration (patterns enabled)
2. ✅ Enable pattern caching (automatic)
3. ✅ Validate all configurations
4. ✅ Adjust weights for playing style
5. ✅ Monitor cache performance
6. ✅ Use WASM-optimized config for WASM
7. ✅ Test thoroughly with comprehensive suite
8. ✅ Document custom configurations
9. ✅ Use phase-aware evaluation
10. ✅ Profile performance when optimizing

---

**Best Practices Guide Complete** ✅
