# Pattern Recognition Troubleshooting Guide

**Version**: 1.0  
**Date**: October 8, 2025

## Common Issues and Solutions

### Issue 1: Patterns Not Detected

**Symptoms**:
- Tactical opportunities not recognized
- Evaluation seems inaccurate
- Pattern statistics show zero detections

**Diagnosis**:
```rust
// Check if patterns are enabled
if evaluator.is_using_integrated_evaluator() {
    println!("✅ Integrated evaluator active");
} else {
    println!("❌ Using legacy evaluation");
}

// Check ComponentFlags
let config = evaluator.get_integrated_evaluator()
    .unwrap().config.components;
println!("Tactical patterns: {}", config.tactical_patterns);
println!("Positional patterns: {}", config.positional_patterns);
```

**Solutions**:
1. Ensure `use_integrated_eval = true` (default)
2. Check ComponentFlags are enabled
3. Verify patterns not disabled in configuration

---

### Issue 2: Compilation Errors

**Error**: `cannot find type PatternCache`

**Solution**:
```rust
// Add import
use crate::evaluation::pattern_cache::PatternCache;
```

**Error**: `no method named 'evaluate_tactics'`

**Solution**:
```rust
// Ensure tactical_patterns module is imported
use crate::evaluation::tactical_patterns::TacticalPatternRecognizer;
```

---

### Issue 3: Poor Cache Performance

**Symptoms**:
- Low cache hit rate (<40%)
- Evaluation not significantly faster with cache

**Diagnosis**:
```rust
println!("Hit rate: {:.1}%", cache.hit_rate() * 100.0);
println!("Cache size: {} / {}", cache.size(), cache.max_size());
```

**Solutions**:
1. Increase cache size: `cache.resize(200_000)`
2. Check if positions are too varied (low repetition)
3. Verify cache is being used (not cleared too frequently)

---

### Issue 4: Unexpected Evaluation Scores

**Symptoms**:
- Scores much higher/lower than expected
- Evaluation inconsistent

**Diagnosis**:
```rust
// Check individual components
let tactical = recognizer.evaluate_tactics(&board, player);
let positional = analyzer.evaluate_position(&board, player, &CapturedPieces::new());

println!("Tactical: {}mg / {}eg", tactical.mg, tactical.eg);
println!("Positional: {}mg / {}eg", positional.mg, positional.eg);
```

**Solutions**:
1. Verify position is as expected (print board)
2. Check if pattern weights are appropriate
3. Ensure phase interpolation is working correctly
4. Validate against known positions

---

### Issue 5: WASM Build Fails

**Error**: Binary size too large

**Solution**:
```rust
// Use WASM-optimized configuration (patterns disabled by default)
#[cfg(target_arch = "wasm32")]
let components = WasmEvaluationOptimizer::get_wasm_components(true);
// Patterns disabled for size optimization
```

**Error**: Performance too slow in browser

**Solution**:
```rust
// Reduce cache sizes for WASM
config.pattern_cache_size = 10_000;  // Smaller cache
config.max_cache_size = 5_000;
```

---

### Issue 6: Memory Usage Too High

**Symptoms**:
- High memory consumption
- Out of memory errors

**Diagnosis**:
```rust
println!("Cache usage: {}", cache.size() * 40);  // ~40 bytes per entry
println!("Pattern cache: {} entries", cache.size());
```

**Solutions**:
1. Reduce cache size: `PatternCache::new(50_000)`
2. Use CompactPatternStorage: 15 bytes data (vs 24 unoptimized)
3. Clear cache periodically: `cache.clear()`
4. Use minimal ComponentFlags

---

### Issue 7: Pattern Weight Validation Errors

**Error**: "Weight 'X' cannot be negative"

**Solution**:
```rust
// Ensure weights are in range [0.0, 10.0]
config.set_tactical_patterns_weight(1.5);  // ✅ Valid
// Not: config.set_tactical_patterns_weight(-1.0);  // ❌ Invalid
```

**Error**: "At least one pattern type must be enabled"

**Solution**:
```rust
// Enable at least one pattern
config.enable_all();  // Or enable specific patterns
config.patterns.tactical_patterns = true;
```

---

### Issue 8: Integration Not Working

**Symptoms**:
- Patterns don't seem to affect evaluation
- No performance improvement

**Diagnosis**:
```rust
// Verify IntegratedEvaluator is active
if !evaluator.is_using_integrated_evaluator() {
    println!("❌ Integrated evaluator not active!");
    evaluator.enable_integrated_evaluator();
}
```

**Solution**:
See `INTEGRATION_VERIFICATION_REPORT.md` for complete verification steps.

---

### Issue 9: Test Failures

**Error**: Pattern tests failing

**Solutions**:
1. Run individual test categories
2. Check test position setup
3. Verify expected vs actual patterns
4. Use `cargo test --lib pattern` to isolate

---

### Issue 10: Performance Regression

**Symptoms**:
- Evaluation slower after enabling patterns
- Search taking longer

**Diagnosis**:
```rust
use std::time::Instant;

let start = Instant::now();
let score = evaluator.evaluate(&board, player, &captured_pieces);
let elapsed = start.elapsed();

if elapsed.as_micros() > 1000 {
    println!("⚠️ Evaluation slow: {:?}", elapsed);
}
```

**Solutions**:
1. Enable caching if not already active
2. Use optimized path: `config.use_optimized_path = true`
3. Disable non-critical patterns
4. Profile to identify bottlenecks

---

## Debugging Tips

### Enable Debug Logging

```rust
// Compile with debug assertions
#[cfg(debug_assertions)]
{
    println!("Pattern detection: {:?}", score);
}
```

### Use Test Suite for Validation

```rust
let mut suite = PatternTestSuite::new();
suite.run_accuracy_tests();
suite.print_summary();
```

### Verify Individual Patterns

```rust
// Test one pattern type at a time
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::all_disabled();
config.components.tactical_patterns = true;  // Test only tactical

let evaluator = IntegratedEvaluator::with_config(config);
```

---

## Getting Help

1. **Check Documentation**: Review API docs and examples
2. **Run Tests**: Use PatternTestSuite to validate
3. **Check Integration**: See INTEGRATION_VERIFICATION_REPORT.md
4. **Profile Performance**: Use benchmarks to identify issues
5. **Validate Configuration**: Always call `config.validate()`

---

**Troubleshooting Guide Complete** ✅
