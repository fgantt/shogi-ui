# Evaluation Cache - Performance Tuning Guide

## Overview

This guide helps you optimize the evaluation cache for maximum performance in your specific use case.

## Performance Targets

### Baseline Targets
- **Cache Probe Time**: <50ns
- **Cache Store Time**: <80ns
- **Hit Rate**: 60%+ 
- **Collision Rate**: <5%
- **Evaluation Time Reduction**: 50-70%

### How to Measure

```rust
let stats = cache.get_statistics();
let metrics = cache.get_performance_metrics();

println!("Hit Rate: {:.2}%", stats.hit_rate());
println!("Collision Rate: {:.2}%", stats.collision_rate());
println!("Utilization: {:.2}%", metrics.memory_utilization());
```

## Tuning Parameters

### 1. Cache Size

**Small (4-8MB)**: Quick games, memory-constrained
```rust
let config = EvaluationCacheConfig::with_size_mb(8);
```

**Medium (16-32MB)**: Normal games, good balance
```rust
let config = EvaluationCacheConfig::with_size_mb(32);
```

**Large (64-128MB)**: Long games, analysis mode
```rust
let config = EvaluationCacheConfig::with_size_mb(128);
```

**How to Choose:**
- Estimate unique positions in search: ~10K-100K
- Cache size should be 2-5x unique positions
- Larger cache = higher hit rate, more memory

### 2. Replacement Policy

**DepthPreferred** (Recommended for search):
```rust
replacement_policy: ReplacementPolicy::DepthPreferred
```
- Keeps evaluations from deeper searches
- Best for iterative deepening
- Good for negamax/alpha-beta

**AgingBased** (For analysis):
```rust
replacement_policy: ReplacementPolicy::AgingBased
```
- Keeps recent evaluations
- Good for position exploration
- Good for analysis mode

**AlwaysReplace** (Simplest):
```rust
replacement_policy: ReplacementPolicy::AlwaysReplace
```
- Fastest (no decision logic)
- Good for testing
- Not recommended for serious search

### 3. Statistics and Verification

**Development Mode:**
```rust
enable_statistics: true,
enable_verification: true,
```
- Track performance
- Catch issues early
- Slight overhead (~5-10ns)

**Production Mode:**
```rust
enable_statistics: false,
enable_verification: false,
```
- Maximum speed
- Minimal overhead
- Use after testing

## Optimization Strategies

### Strategy 1: Maximize Hit Rate

**Goal**: Get hit rate above 70%

**Steps:**
1. Increase cache size
2. Use DepthPreferred policy
3. Enable multi-level cache
4. Use cache warming

**Implementation:**
```rust
let config = MultiLevelCacheConfig {
    l1_size: 32768,   // 32K entries
    l2_size: 2097152, // 2M entries
    l1_policy: ReplacementPolicy::AlwaysReplace,
    l2_policy: ReplacementPolicy::DepthPreferred,
    promotion_threshold: 2,
    ..Default::default()
};
evaluator.enable_multi_level_cache_with_config(config);
```

### Strategy 2: Minimize Memory

**Goal**: Use <16MB while maintaining >50% hit rate

**Steps:**
1. Use 8-16MB cache
2. Enable compaction
3. Use adaptive sizing
4. Disable verification in production

**Implementation:**
```rust
let config = EvaluationCacheConfig {
    size: 262144, // 256K entries (~8MB)
    replacement_policy: ReplacementPolicy::AgingBased,
    enable_statistics: false,
    enable_verification: false,
};
evaluator.enable_eval_cache_with_config(config);

// Periodic compaction
if cache.is_under_memory_pressure() {
    cache.compact();
}
```

### Strategy 3: Maximize Speed

**Goal**: Minimize cache overhead (<30ns total)

**Steps:**
1. Disable statistics
2. Disable verification
3. Use AlwaysReplace policy
4. Use moderate cache size

**Implementation:**
```rust
let config = EvaluationCacheConfig {
    size: 1048576, // 1M entries (still good size)
    replacement_policy: ReplacementPolicy::AlwaysReplace,
    enable_statistics: false,
    enable_verification: false,
};
evaluator.enable_eval_cache_with_config(config);
```

## Workload-Specific Tuning

### Tuning for Opening Play

**Characteristics:**
- Many repeated positions
- Common patterns
- High hit rate potential

**Optimal Configuration:**
```rust
// Medium cache with warming
let config = EvaluationCacheConfig::with_size_mb(16);
evaluator.enable_eval_cache_with_config(config);

// Warm with opening positions
let warmer = CacheWarmer::new(WarmingStrategy::Opening);
warmer.warm_cache(&cache, &evaluator);
```

**Expected**: 70-90% hit rate

### Tuning for Middle Game

**Characteristics:**
- Many unique positions
- Complex tactics
- Moderate hit rate

**Optimal Configuration:**
```rust
// Large cache with depth-preferred
let config = EvaluationCacheConfig::with_size_mb(64);
config.replacement_policy = ReplacementPolicy::DepthPreferred;
evaluator.enable_eval_cache_with_config(config);
```

**Expected**: 50-70% hit rate

### Tuning for Endgame

**Characteristics:**
- Fewer pieces
- More unique positions
- Lower hit rate

**Optimal Configuration:**
```rust
// Smaller cache, aging-based
let config = EvaluationCacheConfig::with_size_mb(16);
config.replacement_policy = ReplacementPolicy::AgingBased;
evaluator.enable_eval_cache_with_config(config);
```

**Expected**: 40-60% hit rate

### Tuning for Analysis Mode

**Characteristics:**
- Deep searches
- Position variations
- Memory available

**Optimal Configuration:**
```rust
// Large multi-level cache
let config = MultiLevelCacheConfig {
    l1_size: 65536,   // 64K entries (~2MB)
    l2_size: 4194304, // 4M entries (~128MB)
    l1_policy: ReplacementPolicy::AlwaysReplace,
    l2_policy: ReplacementPolicy::DepthPreferred,
    promotion_threshold: 3,
    enable_statistics: true,
    enable_verification: true,
};
evaluator.enable_multi_level_cache_with_config(config);
```

**Expected**: 70-85% hit rate

## Measuring Performance

### Benchmark Script

```rust
fn benchmark_cache_performance() {
    let mut evaluator_with_cache = PositionEvaluator::new();
    evaluator_with_cache.enable_eval_cache();
    
    let mut evaluator_without = PositionEvaluator::new();
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Warm cache
    let _ = evaluator_with_cache.evaluate(&board, Player::Black, &captured_pieces);
    
    // Benchmark cached
    let iterations = 10000;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = evaluator_with_cache.evaluate(&board, Player::Black, &captured_pieces);
    }
    let time_cached = start.elapsed();
    
    // Benchmark uncached
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = evaluator_without.evaluate(&board, Player::Black, &captured_pieces);
    }
    let time_uncached = start.elapsed();
    
    println!("Cached: {:?} ({:.2}ns avg)", time_cached, 
             time_cached.as_nanos() as f64 / iterations as f64);
    println!("Uncached: {:?} ({:.2}ns avg)", time_uncached,
             time_uncached.as_nanos() as f64 / iterations as f64);
    println!("Speedup: {:.2}x", 
             time_uncached.as_nanos() as f64 / time_cached.as_nanos() as f64);
}
```

## Tuning Checklist

- [ ] Measure baseline performance without cache
- [ ] Enable cache with default settings
- [ ] Measure performance with cache
- [ ] Check hit rate (target: >60%)
- [ ] Check collision rate (target: <5%)
- [ ] Adjust cache size based on hit rate
- [ ] Select optimal replacement policy
- [ ] Test with real workload
- [ ] Disable stats/verification for production
- [ ] Validate performance improvement (target: 50%+)

## Advanced Tuning

### Adaptive Sizing

```rust
let sizer = AdaptiveCacheSizer::new(
    1024 * 256,   // min: 256K entries (~8MB)
    1024 * 1024 * 16, // max: 16M entries (~512MB)
    65.0          // target hit rate
);

// Check periodically (e.g., every 10 seconds)
if let Some(new_size) = sizer.should_resize(&cache) {
    cache.resize(new_size)?;
}
```

### Multi-Level Tuning

```rust
// Tune L1 for hot positions
// Tune L2 for general storage
let config = MultiLevelCacheConfig {
    l1_size: 16384,    // Smaller, faster
    l2_size: 1048576,  // Larger, comprehensive
    promotion_threshold: 2, // Lower = more aggressive promotion
    ..Default::default()
};
```

## See Also

- `EVALUATION_CACHE_API.md` - Full API reference
- `EVALUATION_CACHE_BEST_PRACTICES.md` - Best practices
- `EVALUATION_CACHE_EXAMPLES.md` - Usage examples
