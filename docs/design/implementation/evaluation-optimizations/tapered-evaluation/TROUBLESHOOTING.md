# Tapered Evaluation - Troubleshooting Guide

## Common Issues and Solutions

### Performance Issues

#### Issue: Evaluation is slower than expected

**Symptoms**:
- Evaluation taking >2000ns
- Search is slow
- Low nodes per second

**Diagnosis**:
```rust
evaluator.enable_statistics();
// Run evaluations...
let report = evaluator.get_statistics().generate_report();
println!("{}", report);
```

**Solutions**:

1. **Check if caching is enabled**:
```rust
let cache_stats = evaluator.cache_stats();
if !cache_stats.phase_cache_enabled {
    // Enable caching
    let mut config = evaluator.config().clone();
    config.enable_phase_cache = true;
    config.enable_eval_cache = true;
    evaluator.set_config(config);
}
```

2. **Use minimal components**:
```rust
let mut config = evaluator.config().clone();
config.components = ComponentFlags::minimal();
evaluator.set_config(config);
```

3. **Disable statistics in production**:
```rust
evaluator.disable_statistics();
```

#### Issue: Cache hit rate is low

**Symptoms**:
- Eval cache size keeps growing
- No performance improvement from caching

**Diagnosis**:
```rust
let stats = evaluator.cache_stats();
println!("Cache size: {} / {}", stats.eval_cache_size, config.max_cache_size);
```

**Solutions**:

1. **Increase cache size**:
```rust
let mut config = evaluator.config().clone();
config.max_cache_size = 50000;
evaluator.set_config(config);
```

2. **Clear cache between games** (not during search):
```rust
// After game ends
evaluator.clear_caches();
```

### Accuracy Issues

#### Issue: Evaluation scores seem wrong

**Symptoms**:
- Starting position not near 0
- Material advantage not reflected
- Scores outside expected range

**Diagnosis**:
```rust
let board = BitboardBoard::new();
let score = evaluator.evaluate(&board, Player::Black, &CapturedPieces::new());
println!("Starting position score: {}", score);

if score.abs() > 100 {
    println!("WARNING: Starting position should be near 0!");
}
```

**Solutions**:

1. **Check if integrated evaluator is enabled**:
```rust
if !evaluator.is_using_integrated_evaluator() {
    evaluator.enable_integrated_evaluator();
}
```

2. **Verify component configuration**:
```rust
let config = evaluator.config();
println!("Material: {}", config.components.material);
println!("PST: {}", config.components.piece_square_tables);
```

3. **Test individual components**:
```rust
// Test material only
let mut config = IntegratedEvaluationConfig::default();
config.components = ComponentFlags::all_disabled();
config.components.material = true;

let mut eval = IntegratedEvaluator::with_config(config);
let score = eval.evaluate(&board, player, &captured);
println!("Material only: {}", score);
```

#### Issue: Evaluation inconsistent

**Symptoms**:
- Same position gives different scores
- Random variations in evaluation

**Diagnosis**:
```rust
let score1 = evaluator.evaluate(&board, player, &captured);
evaluator.clear_caches();
let score2 = evaluator.evaluate(&board, player, &captured);

if score1 != score2 {
    println!("ERROR: Inconsistent evaluation!");
    println!("  First: {}", score1);
    println!("  Second: {}", score2);
}
```

**Solutions**:

1. **This should never happen** - File a bug if it does
2. **Check for uninitialized state**
3. **Verify board state isn't mutated during evaluation**

### Integration Issues

#### Issue: SearchEngine not using tapered evaluation

**Symptoms**:
- No performance improvement
- Search behaves like before

**Diagnosis**:
```rust
let search_engine = SearchEngine::new(None, 64);

// Check if evaluator has integrated evaluator
// (Access through internal field if possible, or test behavior)
```

**Solution**:

The integration is automatic! Verify by:
```rust
// SearchEngine.evaluator is PositionEvaluator
// PositionEvaluator.integrated_evaluator is Some(IntegratedEvaluator)
// PositionEvaluator.use_integrated_eval is true

// All evaluations automatically use tapered system
```

#### Issue: TaperedSearchEnhancer not accessible

**Symptoms**:
- Can't call `get_tapered_search_enhancer()`
- Compilation error

**Solution**:
```rust
// Use public accessor methods
let enhancer = search_engine.get_tapered_search_enhancer();  // Immutable
let enhancer_mut = search_engine.get_tapered_search_enhancer_mut();  // Mutable
```

### Compilation Issues

#### Issue: Borrow checker errors with RefCell

**Symptoms**:
- "Cannot borrow as mutable" errors
- "Already borrowed" panics

**Solution**:

The API is designed to avoid this. If you encounter it:

```rust
// DON'T hold references across calls
let stats = evaluator.get_statistics();  // Returns clone, not reference
// ... use stats ...
```

### Memory Issues

#### Issue: Memory usage growing unbounded

**Symptoms**:
- Increasing memory over time
- Cache growing too large

**Diagnosis**:
```rust
let stats = evaluator.cache_stats();
println!("Phase cache: {} entries (~{}KB)", 
    stats.phase_cache_size,
    stats.phase_cache_size * 16 / 1024);
println!("Eval cache: {} entries (~{}KB)",
    stats.eval_cache_size,
    stats.eval_cache_size * 32 / 1024);
```

**Solutions**:

1. **Set cache limits**:
```rust
config.max_cache_size = 5000;  // Smaller limit
```

2. **Clear caches periodically**:
```rust
if nodes % 50000 == 0 {
    evaluator.clear_caches();
}
```

3. **Clear between games**:
```rust
// After each game
evaluator.clear_caches();
```

## Debugging Techniques

### 1. Enable Debug Logging

```rust
evaluator.enable_statistics();

// Run problematic scenario
evaluator.evaluate(&problem_board, player, &captured);

// Examine statistics
let report = evaluator.get_statistics().generate_report();
println!("{}", report);
```

### 2. Component Isolation

```rust
// Test each component individually
let components = vec![
    "material",
    "piece_square_tables",
    "position_features",
    "opening_principles",
    "endgame_patterns",
];

for comp in &components {
    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::all_disabled();
    
    // Enable one component
    match comp.as_ref() {
        "material" => config.components.material = true,
        "piece_square_tables" => config.components.piece_square_tables = true,
        // ... etc
        _ => {}
    }
    
    let mut eval = IntegratedEvaluator::with_config(config);
    let score = eval.evaluate(&board, player, &captured);
    println!("{}: {}", comp, score);
}
```

### 3. Compare Against Legacy

```rust
// Tapered evaluation
let evaluator = PositionEvaluator::new();
let tapered_score = evaluator.evaluate(&board, player, &captured);

// Legacy evaluation
let mut evaluator_legacy = PositionEvaluator::new();
evaluator_legacy.disable_integrated_evaluator();
let legacy_score = evaluator_legacy.evaluate(&board, player, &captured);

println!("Tapered: {}", tapered_score);
println!("Legacy: {}", legacy_score);
println!("Difference: {}", tapered_score - legacy_score);
```

## FAQs

### Q: Why is my evaluation different from before?

**A**: The tapered evaluation system is more sophisticated and accurate. Differences are expected and normal. Validate that:
- Starting position is near 0
- Material advantages are reflected correctly
- Scores are in reasonable range

### Q: Can I disable tapered evaluation?

**A**: Yes, for backward compatibility:
```rust
evaluator.disable_integrated_evaluator();
```

### Q: How do I know if tapered evaluation is active?

**A**: Check the flag:
```rust
if evaluator.is_using_integrated_evaluator() {
    println!("Tapered evaluation is active!");
} else {
    println!("Using legacy evaluation");
}
```

### Q: What's the memory overhead?

**A**: With default settings:
- Evaluator: ~1KB
- Phase cache (10K entries): ~160KB
- Eval cache (10K entries): ~320KB
- **Total: ~481KB**

### Q: Is it thread-safe?

**A**: No. Create separate evaluator instances per thread.

### Q: Can I use it in WASM?

**A**: Yes! The system compiles to WASM. Reduce cache sizes for browser environments.

## Getting Help

1. **Check documentation**:
   - API_DOCUMENTATION.md
   - USAGE_EXAMPLES.md
   - BEST_PRACTICES.md
   - TUNING_GUIDE.md

2. **Run diagnostics**:
```rust
// Enable statistics
evaluator.enable_statistics();

// Run your scenario
// ...

// Get report
let report = evaluator.get_statistics().generate_report();
println!("{}", report);
```

3. **Test with minimal configuration**:
```rust
config.components = ComponentFlags::minimal();
// If this works, enable components one by one to find issue
```

---

*Troubleshooting Guide Version: 1.0*
*Generated: October 8, 2025*

