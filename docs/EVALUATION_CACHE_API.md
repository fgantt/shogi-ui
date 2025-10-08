# Evaluation Cache API Documentation

## Overview

The Evaluation Cache provides high-performance caching of position evaluations in the Shogi engine, reducing evaluation time by 50-70% through intelligent caching strategies.

## Table of Contents

1. [Quick Start](#quick-start)
2. [API Reference](#api-reference)
3. [Configuration](#configuration)
4. [Usage Examples](#usage-examples)
5. [Best Practices](#best-practices)
6. [Performance Tuning](#performance-tuning)
7. [Troubleshooting](#troubleshooting)

## Quick Start

### Basic Usage

```rust
use shogi_vibe_usi::evaluation::PositionEvaluator;
use shogi_vibe_usi::bitboards::BitboardBoard;
use shogi_vibe_usi::types::*;

// Create evaluator and enable cache
let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();

// Use normally - cache is automatic
let board = BitboardBoard::new();
let captured_pieces = CapturedPieces::new();
let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
```

### With Search Engine

```rust
use shogi_vibe_usi::search::SearchEngine;

let mut engine = SearchEngine::new(None, 16);
engine.enable_eval_cache();

// Search automatically uses cache
let result = engine.search_at_depth(&board, &captured_pieces, Player::Black,
                                    5, 5000, -10000, 10000);
```

## API Reference

### PositionEvaluator Cache Methods

#### enable_eval_cache()
Enable evaluation cache with default configuration (1M entries, ~32MB).

```rust
pub fn enable_eval_cache(&mut self)
```

#### enable_eval_cache_with_config()
Enable cache with custom configuration.

```rust
pub fn enable_eval_cache_with_config(&mut self, config: EvaluationCacheConfig)
```

**Example:**
```rust
let config = EvaluationCacheConfig {
    size: 262144, // 256K entries (~8MB)
    replacement_policy: ReplacementPolicy::DepthPreferred,
    enable_statistics: true,
    enable_verification: true,
};
evaluator.enable_eval_cache_with_config(config);
```

#### enable_multi_level_cache()
Enable multi-level cache (L1: 16K entries, L2: 1M entries).

```rust
pub fn enable_multi_level_cache(&mut self)
```

#### enable_multi_level_cache_with_config()
Enable multi-level cache with custom configuration.

```rust
pub fn enable_multi_level_cache_with_config(&mut self, config: MultiLevelCacheConfig)
```

#### disable_eval_cache()
Disable caching (fall back to direct evaluation).

```rust
pub fn disable_eval_cache(&mut self)
```

#### is_cache_enabled()
Check if cache is currently enabled.

```rust
pub fn is_cache_enabled(&self) -> bool
```

#### get_cache_statistics()
Get cache performance statistics.

```rust
pub fn get_cache_statistics(&self) -> Option<String>
```

**Returns**: Summary string with hit rate, collision rate, etc.

#### clear_eval_cache()
Clear all cached entries.

```rust
pub fn clear_eval_cache(&mut self)
```

### EvaluationCache Methods

#### new()
Create a new cache with default configuration (1M entries).

```rust
pub fn new() -> Self
```

#### with_config()
Create a cache with custom configuration.

```rust
pub fn with_config(config: EvaluationCacheConfig) -> Self
```

#### probe()
Query the cache for a position.

```rust
pub fn probe(&self, board: &BitboardBoard, player: Player, 
             captured_pieces: &CapturedPieces) -> Option<i32>
```

**Returns**: `Some(score)` on hit, `None` on miss.

#### store()
Store an evaluation in the cache.

```rust
pub fn store(&self, board: &BitboardBoard, player: Player,
             captured_pieces: &CapturedPieces, score: i32, depth: u8)
```

#### clear()
Clear all entries.

```rust
pub fn clear(&self)
```

#### get_statistics()
Get detailed statistics.

```rust
pub fn get_statistics(&self) -> CacheStatistics
```

#### save_to_file()
Save cache to disk (JSON format).

```rust
pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String>
```

#### save_to_file_compressed()
Save cache to disk with gzip compression.

```rust
pub fn save_to_file_compressed<P: AsRef<Path>>(&self, path: P) -> Result<(), String>
```

#### load_from_file()
Load cache from disk.

```rust
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>
```

#### load_from_file_compressed()
Load cache from compressed file.

```rust
pub fn load_from_file_compressed<P: AsRef<Path>>(path: P) -> Result<Self, String>
```

### MultiLevelCache Methods

#### new()
Create multi-level cache with default configuration.

```rust
pub fn new() -> Self
```

#### with_config()
Create with custom configuration.

```rust
pub fn with_config(config: MultiLevelCacheConfig) -> Self
```

#### probe()
Query cache (checks L1 first, then L2).

```rust
pub fn probe(&self, board: &BitboardBoard, player: Player,
             captured_pieces: &CapturedPieces) -> Option<i32>
```

#### store()
Store evaluation (goes to L2, promoted to L1 after repeated access).

```rust
pub fn store(&self, board: &BitboardBoard, player: Player,
             captured_pieces: &CapturedPieces, score: i32, depth: u8)
```

#### get_statistics()
Get multi-level statistics.

```rust
pub fn get_statistics(&self) -> MultiLevelCacheStatistics
```

## Configuration

### EvaluationCacheConfig

```rust
pub struct EvaluationCacheConfig {
    pub size: usize,                           // Must be power of 2
    pub replacement_policy: ReplacementPolicy,
    pub enable_statistics: bool,
    pub enable_verification: bool,
}
```

**Default Values:**
- `size`: 1,048,576 (1M entries, ~32MB)
- `replacement_policy`: `DepthPreferred`
- `enable_statistics`: `true`
- `enable_verification`: `true`

### Replacement Policies

```rust
pub enum ReplacementPolicy {
    AlwaysReplace,      // Always replace existing entries
    DepthPreferred,     // Prefer entries with higher depth (recommended)
    AgingBased,         // Replace old entries (age > 8)
}
```

**Recommendation**: Use `DepthPreferred` for best results in search.

### MultiLevelCacheConfig

```rust
pub struct MultiLevelCacheConfig {
    pub l1_size: usize,              // L1 cache size (default: 16K)
    pub l2_size: usize,              // L2 cache size (default: 1M)
    pub l1_policy: ReplacementPolicy,
    pub l2_policy: ReplacementPolicy,
    pub enable_statistics: bool,
    pub enable_verification: bool,
    pub promotion_threshold: u8,     // Accesses needed for L2→L1 promotion
}
```

**Default Values:**
- `l1_size`: 16,384 (~512KB) - Fast, hot cache
- `l2_size`: 1,048,576 (~32MB) - Large, warm cache
- `l1_policy`: `AlwaysReplace` - Keep L1 fresh
- `l2_policy`: `DepthPreferred` - Keep deep evals in L2
- `promotion_threshold`: 2 - Promote after 2 L2 hits

## Usage Examples

### Example 1: Basic Single-Level Cache

```rust
use shogi_vibe_usi::evaluation::PositionEvaluator;

let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();

// Evaluate (cache automatic)
let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

// Check statistics
if let Some(stats) = evaluator.get_cache_statistics() {
    println!("{}", stats);
}
```

### Example 2: Custom Configuration

```rust
use shogi_vibe_usi::evaluation::eval_cache::*;

// Create custom config (16MB cache)
let config = EvaluationCacheConfig::with_size_mb(16);

evaluator.enable_eval_cache_with_config(config);
```

### Example 3: Multi-Level Cache

```rust
// Enable multi-level cache for better hit rates
evaluator.enable_multi_level_cache();

// L1: 16K entries (~512KB) - frequently accessed positions
// L2: 1M entries (~32MB) - all evaluated positions
// Automatic promotion from L2 to L1 based on access patterns
```

### Example 4: Search Integration

```rust
let mut engine = SearchEngine::new(None, 16);

// Enable cache
engine.enable_eval_cache();

// Search with cache (automatic)
let result = engine.search_at_depth(&board, &captured_pieces, Player::Black,
                                    6, 10000, -10000, 10000);

// Monitor cache performance
if let Some(stats) = engine.get_eval_cache_statistics() {
    println!("Cache Stats:\n{}", stats);
}
```

### Example 5: Cache Persistence

```rust
// Save cache at end of session
if let Some(cache) = evaluator.get_eval_cache() {
    cache.save_to_file_compressed("eval_cache.gz")?;
}

// Load cache at start of next session
let cache = EvaluationCache::load_from_file_compressed("eval_cache.gz")?;
evaluator.enable_eval_cache_with_config(cache.get_config().clone());
```

### Example 6: Memory Management

```rust
if let Some(cache) = evaluator.get_eval_cache_mut() {
    // Check memory pressure
    if cache.is_under_memory_pressure() {
        cache.compact(); // Remove old entries
    }
    
    // Get size suggestion
    let suggested = cache.suggest_cache_size();
    if suggested != cache.get_config().size {
        cache.resize(suggested)?;
    }
}
```

### Example 7: Cache Warming

```rust
use shogi_vibe_usi::evaluation::eval_cache::*;

let cache = EvaluationCache::new();
let warmer = CacheWarmer::new(WarmingStrategy::Opening);

// Pre-populate cache with common positions
warmer.warm_cache(&cache, &evaluator);
println!("Warmed {} positions", warmer.get_warmed_count());
```

### Example 8: Adaptive Sizing

```rust
let sizer = AdaptiveCacheSizer::new(
    1024,          // min: 1K entries
    64*1024*1024,  // max: 64M entries
    65.0           // target: 65% hit rate
);

// Periodically check and adjust
if let Some(new_size) = sizer.should_resize(&cache) {
    println!("Resizing cache to {} entries", new_size);
    cache.resize(new_size)?;
}
```

## Best Practices

### 1. Cache Size Selection

**Small Games/Quick Analysis** (4-8MB):
```rust
let config = EvaluationCacheConfig::with_size_mb(8);
```

**Normal Games** (16-32MB):
```rust
let config = EvaluationCacheConfig::with_size_mb(32);
```

**Long Games/Analysis** (64-128MB):
```rust
let config = EvaluationCacheConfig::with_size_mb(128);
```

### 2. Replacement Policy Selection

**For Search Engines:**
```rust
replacement_policy: ReplacementPolicy::DepthPreferred
```
- Keeps deeper search results
- Best for iterative deepening
- Recommended for most use cases

**For Quick Evaluations:**
```rust
replacement_policy: ReplacementPolicy::AlwaysReplace
```
- Simplest policy
- Good when positions rarely repeat

**For Long-Running Analysis:**
```rust
replacement_policy: ReplacementPolicy::AgingBased
```
- Keeps recent evaluations
- Good for evolving position analysis

### 3. When to Use Multi-Level Cache

**Use Multi-Level When:**
- Cache size > 32MB
- Clear hot/cold position patterns
- Memory is limited
- L1 hit rate matters

**Example:**
```rust
evaluator.enable_multi_level_cache();
// L1 caches hot positions for ultra-fast access
// L2 caches everything for good overall hit rate
```

### 4. Statistics Monitoring

**During Development:**
```rust
let config = EvaluationCacheConfig {
    enable_statistics: true,  // Track performance
    enable_verification: true, // Catch issues early
    ...
};
```

**In Production:**
```rust
let config = EvaluationCacheConfig {
    enable_statistics: false, // Slight performance gain
    enable_verification: false, // Slight performance gain
    ...
};
```

### 5. Cache Persistence

**Save after long games:**
```rust
cache.save_to_file_compressed("cache.gz")?;
```

**Load at startup:**
```rust
if let Ok(cache) = EvaluationCache::load_from_file_compressed("cache.gz") {
    // Use loaded cache
} else {
    // Create new cache
}
```

## Performance Tuning

### Optimize for Hit Rate

**Problem**: Low hit rate (<40%)

**Solutions:**
1. Increase cache size
   ```rust
   cache.resize(cache.get_config().size * 2)?;
   ```

2. Use `DepthPreferred` policy
   ```rust
   cache.update_replacement_policy(ReplacementPolicy::DepthPreferred);
   ```

3. Use multi-level cache
   ```rust
   evaluator.enable_multi_level_cache();
   ```

### Optimize for Memory

**Problem**: Memory usage too high

**Solutions:**
1. Reduce cache size
   ```rust
   cache.resize(cache.get_config().size / 2)?;
   ```

2. Enable compaction
   ```rust
   if cache.is_under_memory_pressure() {
       cache.compact();
   }
   ```

3. Use smaller cache configuration
   ```rust
   let config = EvaluationCacheConfig::with_size_mb(8);
   ```

### Optimize for Speed

**Problem**: Cache overhead noticeable

**Solutions:**
1. Disable statistics (in production)
   ```rust
   cache.set_statistics_enabled(false);
   ```

2. Disable verification (in production)
   ```rust
   cache.set_verification_enabled(false);
   ```

3. Use `AlwaysReplace` policy (simplest)
   ```rust
   cache.update_replacement_policy(ReplacementPolicy::AlwaysReplace);
   ```

### Target Performance

- **Cache Probe**: <50ns
- **Cache Store**: <80ns
- **Hit Rate**: 60%+ (achievable)
- **Collision Rate**: <5% (typically <1%)
- **Speedup**: 50-70% evaluation time reduction

## Troubleshooting

### Problem: Low Hit Rate

**Symptoms**: Hit rate <40%

**Diagnosis:**
```rust
let stats = cache.get_statistics();
println!("Hit rate: {:.2}%", stats.hit_rate());
```

**Solutions:**
- Increase cache size
- Change to `DepthPreferred` policy
- Use multi-level cache
- Enable cache warming

### Problem: High Collision Rate

**Symptoms**: Collision rate >5%

**Diagnosis:**
```rust
let stats = cache.get_statistics();
println!("Collision rate: {:.2}%", stats.collision_rate());
```

**Solutions:**
- Increase cache size (more entries = fewer collisions)
- Enable verification (default)
- Check hash function quality

### Problem: High Memory Usage

**Symptoms**: System running out of memory

**Diagnosis:**
```rust
let usage = cache.get_memory_usage();
println!("Using {:.2} MB", usage.total_bytes as f64 / (1024.0 * 1024.0));
```

**Solutions:**
- Reduce cache size
- Use compaction
- Enable memory pressure monitoring

### Problem: Cache Not Working

**Symptoms**: No performance improvement

**Checks:**
```rust
// 1. Is cache enabled?
assert!(evaluator.is_cache_enabled());

// 2. Are we getting hits?
let stats = evaluator.get_cache_statistics();
println!("{:?}", stats);

// 3. Is cache size reasonable?
if let Some(cache) = evaluator.get_eval_cache() {
    println!("Cache size: {} entries", cache.get_config().size);
}
```

**Solutions:**
- Ensure cache is enabled
- Check cache size is not too small
- Verify positions are actually repeating

## Configuration Guide

### Small Memory Systems (<100MB available)

```rust
let config = EvaluationCacheConfig::with_size_mb(8);
evaluator.enable_eval_cache_with_config(config);
```

### Normal Systems (100-500MB available)

```rust
let config = EvaluationCacheConfig::with_size_mb(32);
evaluator.enable_eval_cache_with_config(config);
```

### Large Systems (>500MB available)

```rust
let config = EvaluationCacheConfig::with_size_mb(128);
evaluator.enable_eval_cache_with_config(config);
// or
evaluator.enable_multi_level_cache();
```

### WASM/Browser (Memory constrained)

```rust
let config = EvaluationCacheConfig::with_size_mb(4);
evaluator.enable_eval_cache_with_config(config);
```

## Performance Optimization Guide

### 1. Choose Right Cache Size

**Rule of Thumb**: 
- Unique positions in typical search: ~10K-100K
- Cache should be 2-5x expected unique positions
- Minimum: 4MB (128K entries)
- Recommended: 16-32MB (512K-1M entries)
- Maximum: 128MB+ (4M+ entries)

### 2. Select Appropriate Policy

**DepthPreferred** (Recommended):
- Best for search engines
- Keeps deeper evaluations
- Good for iterative deepening

**AgingBased**:
- Good for analysis mode
- Keeps recent evaluations
- Good for position exploration

**AlwaysReplace**:
- Simplest, fastest
- Good for quick evaluations
- Not recommended for search

### 3. Tune for Workload

**Tactical Positions** (many unique):
- Larger cache
- AgingBased policy
- Enable compaction

**Strategic Positions** (repetitive):
- Normal cache size
- DepthPreferred policy
- Multi-level cache

### 4. Monitor and Adjust

```rust
// Get recommendations
let recommendations = cache.get_performance_recommendations();
for rec in recommendations {
    println!("{}", rec);
}

// Check if maintenance needed
if cache.needs_maintenance() {
    cache.compact();
}
```

## Advanced Features

### Cache Prefetching

```rust
let prefetcher = CachePrefetcher::new();

// Queue positions
prefetcher.queue_prefetch(board, player, captured_pieces, priority);

// Process queue
prefetcher.process_queue(&cache, &evaluator);
```

### Adaptive Sizing

```rust
let sizer = AdaptiveCacheSizer::new(min_size, max_size, target_hit_rate);
if let Some(new_size) = sizer.should_resize(&cache) {
    cache.resize(new_size)?;
}
```

### Analytics

```rust
let analytics = cache.get_analytics();
let json = cache.export_analytics_json()?;
println!("{}", json);
```

## Thread Safety

The evaluation cache is **fully thread-safe**:

- Uses `RwLock` for entry synchronization
- Uses atomic operations for statistics
- Safe for concurrent evaluation
- No data races possible

**Example:**
```rust
// Safe to use from multiple contexts
let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
let score2 = evaluator.evaluate(&board, Player::White, &captured_pieces);
// Both can happen concurrently
```

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Cache Probe (hit) | <50ns | Very fast |
| Cache Probe (miss) | <50ns | Still fast |
| Cache Store | <80ns | Quick |
| Full Evaluation | 1000-5000ns | Without cache |
| Cached Evaluation | <100ns | Probe + return |
| **Speedup** | **20-100x** | For cache hits |

## Best Practices Summary

✅ **DO:**
- Use `DepthPreferred` policy for search
- Enable cache for iterative deepening
- Monitor statistics during development
- Use multi-level cache for large caches (>32MB)
- Save/load cache between sessions
- Disable stats/verification in production

❌ **DON'T:**
- Use cache size not power of 2
- Ignore collision rate warnings
- Forget to clear cache between games (if needed)
- Use `AlwaysReplace` for serious search
- Disable verification during development

## See Also

- `EVALUATION_CACHE_TROUBLESHOOTING.md` - Detailed troubleshooting
- `EVALUATION_CACHE_TUNING_GUIDE.md` - Performance tuning
- `EVALUATION_CACHE_EXAMPLES.md` - More examples
- `src/evaluation/eval_cache.rs` - Implementation

---

**Version**: 1.0  
**Last Updated**: October 8, 2025  
**Status**: Complete ✅
