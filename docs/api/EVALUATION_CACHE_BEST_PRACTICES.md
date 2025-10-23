# Evaluation Cache - Best Practices Guide

## Core Principles

### 1. Enable Cache for Search Engines

✅ **DO**: Enable cache for any search depth > 3
```rust
let mut engine = SearchEngine::new(None, 16);
engine.enable_eval_cache();
```

❌ **DON'T**: Run deep searches without cache
```rust
// Slow for deep search!
let engine = SearchEngine::new(None, 16);
// cache not enabled
```

### 2. Use Appropriate Cache Size

✅ **DO**: Size cache based on workload
```rust
// For typical games: 16-32MB
let config = EvaluationCacheConfig::with_size_mb(32);
```

❌ **DON'T**: Use tiny cache for deep search
```rust
// Too small! Will have poor hit rate
let config = EvaluationCacheConfig {
    size: 1024, // Only 1K entries (~32KB)
    ...
};
```

### 3. Choose Right Replacement Policy

✅ **DO**: Use DepthPreferred for search
```rust
replacement_policy: ReplacementPolicy::DepthPreferred
```

❌ **DON'T**: Use AlwaysReplace for serious search
```rust
// Not optimal for iterative deepening
replacement_policy: ReplacementPolicy::AlwaysReplace
```

### 4. Monitor Performance

✅ **DO**: Check statistics periodically
```rust
if let Some(stats) = engine.get_eval_cache_statistics() {
    println!("{}", stats);
}
```

❌ **DON'T**: Enable cache blindly without monitoring
```rust
engine.enable_eval_cache();
// Never check if it's working
```

### 5. Disable Stats in Production

✅ **DO**: Disable for maximum speed
```rust
let config = EvaluationCacheConfig {
    enable_statistics: false,
    enable_verification: false,
    ...
};
```

❌ **DON'T**: Leave stats enabled in production
```rust
// Slight overhead in hot path
enable_statistics: true,  // Only for development
```

## Configuration Best Practices

### Development Configuration

```rust
let config = EvaluationCacheConfig {
    size: 1048576,
    replacement_policy: ReplacementPolicy::DepthPreferred,
    enable_statistics: true,   // Monitor performance
    enable_verification: true, // Catch bugs early
};
```

### Production Configuration

```rust
let config = EvaluationCacheConfig {
    size: 1048576,
    replacement_policy: ReplacementPolicy::DepthPreferred,
    enable_statistics: false,  // Maximum speed
    enable_verification: false, // Maximum speed
};
```

## Usage Patterns

### Pattern 1: Game Play

```rust
// Setup once
let mut engine = SearchEngine::new(None, 32);
engine.enable_eval_cache();

// Use for entire game
loop {
    let result = engine.search_at_depth(...);
    // Cache accumulates knowledge
}

// Optional: Clear between games
engine.clear_eval_cache();
```

### Pattern 2: Position Analysis

```rust
// Use multi-level cache for deep analysis
engine.enable_multi_level_cache();

// Analyze position at multiple depths
for depth in 1..=20 {
    let result = engine.search_at_depth(..., depth, ...);
    // Cache helps with deeper iterations
}
```

### Pattern 3: Training/Tuning

```rust
// Use persistence to save computed evaluations
engine.enable_eval_cache();

// Evaluate many positions
for position in training_set {
    let score = evaluator.evaluate(...);
}

// Save for next session
cache.save_to_file_compressed("training_cache.gz")?;
```

## Memory Management Best Practices

### Practice 1: Set Appropriate Size

```rust
// Calculate based on available memory
let available_mb = 100; // System-dependent
let cache_mb = (available_mb / 4).min(64); // Use up to 25%, max 64MB
let config = EvaluationCacheConfig::with_size_mb(cache_mb);
```

### Practice 2: Monitor Utilization

```rust
let usage = cache.get_memory_usage();
if usage.entry_utilization() > 90.0 {
    println!("Cache nearly full - consider increasing size");
}
```

### Practice 3: Use Compaction

```rust
// Periodic compaction (every 1000 moves, etc.)
if move_count % 1000 == 0 {
    cache.compact();
}
```

## Multi-Level Cache Best Practices

### When to Use Multi-Level

✅ **Use When:**
- Cache size > 32MB
- Clear hot/cold position patterns
- Want to optimize L1 hit rate
- Have memory available

```rust
evaluator.enable_multi_level_cache();
```

❌ **Don't Use When:**
- Cache size < 16MB (overhead not worth it)
- Memory constrained
- Simple workloads

### Tuning Multi-Level

**Aggressive Promotion** (more L1 hits):
```rust
promotion_threshold: 1, // Promote after 1 L2 hit
```

**Conservative Promotion** (save L1 space):
```rust
promotion_threshold: 3, // Promote after 3 L2 hits
```

## Cache Warming Best Practices

### When to Warm Cache

✅ **Warm When:**
- At application startup
- Before important games
- Known opening positions

```rust
let warmer = CacheWarmer::new(WarmingStrategy::Opening);
warmer.warm_cache(&cache, &evaluator);
```

❌ **Don't Warm When:**
- Memory limited
- Positions are unpredictable
- Quick games

## Integration Best Practices

### Practice 1: Enable Early

```rust
// Enable cache before any evaluation
let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();

// Now use for everything
let score = evaluator.evaluate(...);
```

### Practice 2: Consistent Configuration

```rust
// Save configuration for consistency
config.save_to_file("cache_config.json")?;

// Load in all instances
let config = EvaluationCacheConfig::load_from_file("cache_config.json")?;
```

### Practice 3: Clear Between Games (Optional)

```rust
// If you want fresh cache each game
engine.clear_eval_cache();
```

**Note**: Usually not necessary - cache naturally adapts

## Performance Optimization Checklist

### Initial Setup
- [x] Enable cache with appropriate size
- [x] Choose right replacement policy
- [x] Monitor statistics initially

### During Development
- [x] Track hit rate (target: >60%)
- [x] Monitor collision rate (target: <5%)
- [x] Validate correctness (compare with/without)
- [x] Adjust size based on hit rate

### Before Production
- [x] Disable statistics
- [x] Disable verification
- [x] Test performance improvement
- [x] Validate memory usage
- [x] Document configuration

### In Production
- [x] Monitor hit rate periodically
- [x] Use adaptive sizing if needed
- [x] Handle memory pressure
- [x] Consider cache persistence

## Common Mistakes

### Mistake 1: Cache Too Small

❌ **Wrong**:
```rust
let config = EvaluationCacheConfig {
    size: 1024, // Only 32KB - too small!
    ...
};
```

✅ **Correct**:
```rust
let config = EvaluationCacheConfig::with_size_mb(16); // At least 16MB
```

### Mistake 2: Wrong Policy for Workload

❌ **Wrong**:
```rust
// AlwaysReplace in iterative deepening search
replacement_policy: ReplacementPolicy::AlwaysReplace
```

✅ **Correct**:
```rust
// DepthPreferred keeps deeper evaluations
replacement_policy: ReplacementPolicy::DepthPreferred
```

### Mistake 3: Not Monitoring Performance

❌ **Wrong**:
```rust
engine.enable_eval_cache();
// Never check if it helps
```

✅ **Correct**:
```rust
engine.enable_eval_cache();
// Monitor and adjust
if let Some(stats) = engine.get_eval_cache_statistics() {
    println!("{}", stats);
}
```

### Mistake 4: Ignoring Recommendations

❌ **Wrong**:
```rust
// Ignore performance recommendations
let _ = cache.get_performance_recommendations();
```

✅ **Correct**:
```rust
let recommendations = cache.get_performance_recommendations();
for rec in recommendations {
    println!("Recommendation: {}", rec);
    // Act on recommendations
}
```

## Summary of Best Practices

### Configuration
- ✅ Use 16-64MB for normal games
- ✅ Use DepthPreferred policy for search
- ✅ Enable statistics during development
- ✅ Disable statistics in production

### Usage
- ✅ Enable cache early
- ✅ Monitor hit rate (target: >60%)
- ✅ Use multi-level for large caches
- ✅ Consider cache warming

### Performance
- ✅ Measure before and after
- ✅ Target 50-70% evaluation time reduction
- ✅ Check collision rate (<5%)
- ✅ Validate correctness

### Memory
- ✅ Size appropriately for system
- ✅ Monitor utilization
- ✅ Use compaction if needed
- ✅ Consider adaptive sizing

### Integration
- ✅ Use automatic integration (no algorithm changes)
- ✅ Access via evaluator/engine methods
- ✅ Clear cache between games if needed
- ✅ Persist cache for long sessions

## See Also

- `EVALUATION_CACHE_API.md` - API documentation
- `EVALUATION_CACHE_TUNING_GUIDE.md` - Performance tuning
- `EVALUATION_CACHE_TROUBLESHOOTING.md` - Common issues
