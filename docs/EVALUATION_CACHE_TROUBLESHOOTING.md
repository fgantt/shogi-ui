# Evaluation Cache - Troubleshooting Guide

## Common Issues and Solutions

### Issue 1: Low Hit Rate (<40%)

**Symptoms:**
- Cache hit rate below 40%
- No significant performance improvement
- High miss rate

**Diagnosis:**
```rust
let stats = cache.get_statistics();
println!("Hit rate: {:.2}%", stats.hit_rate());
println!("Misses: {}", stats.misses);
```

**Solutions:**

1. **Increase Cache Size**
   ```rust
   let config = EvaluationCacheConfig::with_size_mb(64);
   evaluator.enable_eval_cache_with_config(config);
   ```

2. **Use Multi-Level Cache**
   ```rust
   evaluator.enable_multi_level_cache();
   ```

3. **Change Replacement Policy**
   ```rust
   cache.update_replacement_policy(ReplacementPolicy::DepthPreferred);
   ```

4. **Enable Cache Warming**
   ```rust
   let warmer = CacheWarmer::new(WarmingStrategy::Opening);
   warmer.warm_cache(&cache, &evaluator);
   ```

### Issue 2: High Collision Rate (>5%)

**Symptoms:**
- Collision rate above 5%
- Verification failures
- Incorrect evaluations

**Diagnosis:**
```rust
let stats = cache.get_statistics();
println!("Collision rate: {:.2}%", stats.collision_rate());
println!("Collisions: {}", stats.collisions);
```

**Solutions:**

1. **Increase Cache Size** (primary solution)
   ```rust
   let current_size = cache.get_config().size;
   cache.resize(current_size * 2)?;
   ```

2. **Enable Verification** (if disabled)
   ```rust
   cache.set_verification_enabled(true);
   ```

3. **Check Hash Quality**
   - Zobrist hashing should have low collisions
   - May indicate hash function issue

### Issue 3: High Memory Usage

**Symptoms:**
- System running out of memory
- Slow performance
- Memory pressure warnings

**Diagnosis:**
```rust
let usage = cache.get_memory_usage();
println!("Memory: {:.2} MB", usage.total_bytes as f64 / (1024.0 * 1024.0));
println!("Utilization: {:.2}%", usage.entry_utilization());
```

**Solutions:**

1. **Reduce Cache Size**
   ```rust
   cache.resize(cache.get_config().size / 2)?;
   ```

2. **Use Compaction**
   ```rust
   if cache.is_under_memory_pressure() {
       cache.compact(); // Remove old entries
   }
   ```

3. **Use Smaller Default Size**
   ```rust
   let config = EvaluationCacheConfig::with_size_mb(8);
   ```

### Issue 4: Cache Not Working

**Symptoms:**
- No performance improvement
- Statistics show 0 hits
- Cache seems inactive

**Diagnosis:**
```rust
// Check if enabled
println!("Cache enabled: {}", evaluator.is_cache_enabled());

// Check statistics
if let Some(stats) = evaluator.get_cache_statistics() {
    println!("{}", stats);
} else {
    println!("No cache statistics available");
}
```

**Solutions:**

1. **Verify Cache is Enabled**
   ```rust
   evaluator.enable_eval_cache();
   assert!(evaluator.is_cache_enabled());
   ```

2. **Check Cache Size**
   ```rust
   if let Some(cache) = evaluator.get_eval_cache() {
       println!("Cache size: {}", cache.get_config().size);
   }
   ```

3. **Verify Statistics are Enabled**
   ```rust
   cache.set_statistics_enabled(true);
   ```

### Issue 5: Slow Performance with Cache

**Symptoms:**
- Cache enabled but performance worse
- High overhead
- Slower than without cache

**Diagnosis:**
```rust
let metrics = cache.get_performance_metrics();
println!("Avg probe time: {}ns", metrics.avg_probe_time_ns);
println!("Filled entries: {}/{}", metrics.filled_entries, metrics.total_capacity);
```

**Solutions:**

1. **Disable Statistics (Production)**
   ```rust
   cache.set_statistics_enabled(false);
   ```

2. **Disable Verification (Production)**
   ```rust
   cache.set_verification_enabled(false);
   ```

3. **Use Simpler Policy**
   ```rust
   cache.update_replacement_policy(ReplacementPolicy::AlwaysReplace);
   ```

## Performance Issues

### Issue 6: High Replacement Rate (>80%)

**Symptoms:**
- Replacement rate above 80%
- Entries constantly replaced
- Low cache effectiveness

**Diagnosis:**
```rust
let stats = cache.get_statistics();
println!("Replacement rate: {:.2}%", stats.replacement_rate());
```

**Solutions:**

1. **Use DepthPreferred Policy**
   ```rust
   cache.update_replacement_policy(ReplacementPolicy::DepthPreferred);
   ```

2. **Increase Cache Size**
   ```rust
   cache.resize(cache.get_config().size * 2)?;
   ```

### Issue 7: Cache Not Persisting Between Sessions

**Symptoms:**
- Cache file not found
- Load fails
- Saved cache not loading

**Solutions:**

1. **Check File Path**
   ```rust
   let path = "eval_cache.gz";
   if std::path::Path::new(path).exists() {
       println!("Cache file exists");
   } else {
       println!("Cache file not found");
   }
   ```

2. **Check Permissions**
   - Ensure write permissions for save
   - Ensure read permissions for load

3. **Validate Version**
   ```rust
   match EvaluationCache::load_from_file_compressed("cache.gz") {
       Ok(cache) => println!("Loaded successfully"),
       Err(e) => println!("Load error: {}", e),
   }
   ```

## Debugging Tips

### Enable Debug Logging

```rust
// Enable statistics for debugging
cache.set_statistics_enabled(true);
cache.set_verification_enabled(true);

// Check stats frequently
let stats = cache.get_statistics();
println!("Probes: {}, Hits: {}, Misses: {}", 
         stats.probes, stats.hits, stats.misses);
```

### Monitor Cache Health

```rust
// Check if cache needs maintenance
if cache.needs_maintenance() {
    println!("Cache maintenance recommended");
    let recommendations = cache.get_performance_recommendations();
    for rec in recommendations {
        println!("  - {}", rec);
    }
}
```

### Validate Correctness

```rust
// Compare with and without cache
let score_cached = evaluator_with_cache.evaluate(&board, player, &captured_pieces);
let score_uncached = evaluator_no_cache.evaluate(&board, player, &captured_pieces);
assert_eq!(score_cached, score_uncached, "Cache correctness issue!");
```

## Getting Help

If you encounter issues not covered here:

1. Check cache statistics with `get_statistics()`
2. Review performance recommendations with `get_performance_recommendations()`
3. Verify cache configuration with `get_config().summary()`
4. Check cache analytics with `get_analytics()`
5. Review integration tests in `tests/eval_cache_integration_tests.rs`

## See Also

- `EVALUATION_CACHE_API.md` - API documentation
- `EVALUATION_CACHE_TUNING_GUIDE.md` - Performance tuning
- `EVALUATION_CACHE_EXAMPLES.md` - Usage examples
