# Move Ordering Performance Tuning Guide

## Overview

This guide provides detailed information on optimizing move ordering performance for different scenarios and constraints.

## Table of Contents

- [Performance Metrics](#performance-metrics)
- [Tuning Strategies](#tuning-strategies)
- [Configuration Profiles](#configuration-profiles)
- [Optimization Techniques](#optimization-techniques)
- [Profiling and Monitoring](#profiling-and-monitoring)
- [Common Performance Issues](#common-performance-issues)

## Performance Metrics

### Key Performance Indicators

1. **Ordering Time**: Average time to order a set of moves
   - Target: < 10μs for cached moves, < 100μs for uncached
   - Monitor: `stats.avg_ordering_time_us`

2. **Cache Hit Rate**: Percentage of moves found in cache
   - Target: > 70%
   - Monitor: `stats.cache_hit_rate`

3. **PV Move Hit Rate**: Percentage of PV moves successfully used
   - Target: 30-50%
   - Monitor via `get_pv_stats()`

4. **Killer Move Hit Rate**: Percentage of killer moves successfully used
   - Target: 20-40%
   - Monitor via `get_killer_move_stats()`

5. **Memory Usage**: Total memory consumed by move ordering
   - Target: < 10 MB for typical use
   - Monitor: `get_current_memory_usage()`

### Measuring Performance

```rust
let stats = orderer.get_stats();

println!("Performance Metrics:");
println!("  Avg ordering time: {:.2}μs", stats.avg_ordering_time_us);
println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate);
println!("  Memory usage: {:.2} MB", stats.memory_usage_bytes as f64 / 1_048_576.0);

// Calculate moves per second
let moves_per_second = if stats.avg_ordering_time_us > 0.0 {
    1_000_000.0 / stats.avg_ordering_time_us
} else {
    0.0
};
println!("  Moves per second: {:.0}", moves_per_second);
```

## Tuning Strategies

### Strategy 1: Maximize Search Speed

**Goal**: Order moves as quickly as possible

```rust
let mut config = MoveOrderingConfig::default();

// Use large caches for high hit rates
config.cache_config.max_cache_size = 500000;
config.cache_config.max_see_cache_size = 250000;
config.cache_config.enable_dynamic_sizing = true;

// Enable all fast heuristics
config.heuristic_config.enable_pv_moves = true;
config.heuristic_config.enable_killer_moves = true;
config.heuristic_config.enable_history_heuristic = true;

// Disable expensive SEE for speed
config.heuristic_config.enable_see = false;

// High weights for effective heuristics
config.weights.pv_move_weight = 10000;
config.weights.killer_move_weight = 5000;
config.weights.history_weight = 200;
```

### Strategy 2: Maximize Search Quality

**Goal**: Find the best move ordering, even if slower

```rust
let mut config = MoveOrderingConfig::default();

// Enable all heuristics including expensive ones
config.heuristic_config.enable_pv_moves = true;
config.heuristic_config.enable_killer_moves = true;
config.heuristic_config.enable_history_heuristic = true;
config.heuristic_config.enable_see = true;  // Enable SEE

// Balanced weights
config.weights.pv_move_weight = 10000;
config.weights.killer_move_weight = 5000;
config.weights.see_weight = 800;
config.weights.capture_weight = 1500;
config.weights.tactical_weight = 1200;

// More killer moves per depth
config.heuristic_config.max_killer_moves = 3;
```

### Strategy 3: Minimize Memory Usage

**Goal**: Use minimal memory while maintaining effectiveness

```rust
let mut config = MoveOrderingConfig::default();

// Small cache sizes
config.cache_config.max_cache_size = 10000;
config.cache_config.max_see_cache_size = 5000;
config.cache_config.enable_dynamic_sizing = false;

// Disable expensive features
config.heuristic_config.enable_see = false;

// Fewer killer moves
config.heuristic_config.max_killer_moves = 1;

// More frequent history aging to limit growth
config.heuristic_config.history_aging_frequency = 50;
```

### Strategy 4: Balanced (Recommended)

**Goal**: Good performance with reasonable resource usage

```rust
let config = MoveOrderingConfig::performance_optimized();
// This provides a balanced configuration suitable for most use cases
```

## Configuration Profiles

### Profile: Blitz Games (Fast Time Controls)

```rust
let mut config = MoveOrderingConfig::default();

// Medium cache sizes
config.cache_config.max_cache_size = 100000;
config.cache_config.max_see_cache_size = 50000;

// Prioritize speed
config.heuristic_config.enable_see = false;
config.weights.pv_move_weight = 10000;
config.weights.killer_move_weight = 6000;
config.weights.history_weight = 300;
```

### Profile: Long Games (Slow Time Controls)

```rust
let mut config = MoveOrderingConfig::default();

// Large cache sizes
config.cache_config.max_cache_size = 500000;
config.cache_config.max_see_cache_size = 250000;

// Enable all features
config.heuristic_config.enable_see = true;
config.heuristic_config.max_killer_moves = 3;

// Balanced weights favoring accuracy
config.weights.see_weight = 1000;
config.weights.tactical_weight = 1500;
```

### Profile: Analysis Mode

```rust
let mut config = MoveOrderingConfig::default();

// Very large caches
config.cache_config.max_cache_size = 1000000;
config.cache_config.max_see_cache_size = 500000;

// Enable everything for best quality
config.heuristic_config.enable_pv_moves = true;
config.heuristic_config.enable_killer_moves = true;
config.heuristic_config.enable_history_heuristic = true;
config.heuristic_config.enable_see = true;
config.heuristic_config.max_killer_moves = 4;
```

### Profile: Mobile/WASM

```rust
let mut config = MoveOrderingConfig::default();

// Minimal memory usage
config.cache_config.max_cache_size = 5000;
config.cache_config.max_see_cache_size = 2000;
config.cache_config.enable_dynamic_sizing = false;

// Disable expensive features
config.heuristic_config.enable_see = false;
config.heuristic_config.max_killer_moves = 1;
```

## Optimization Techniques

### 1. Cache Optimization

**Monitor cache effectiveness:**

```rust
let stats = orderer.get_stats();
let hit_rate = stats.cache_hit_rate;

if hit_rate < 50.0 {
    // Cache too small - increase size
    let mut config = orderer.get_config().clone();
    config.cache_config.max_cache_size *= 2;
    orderer.set_config(config);
} else if hit_rate > 95.0 {
    // Cache may be too large - could reduce
    let mut config = orderer.get_config().clone();
    config.cache_config.max_cache_size = (config.cache_config.max_cache_size * 3) / 4;
    orderer.set_config(config);
}
```

### 2. Weight Tuning

**Adjust weights based on heuristic effectiveness:**

```rust
let pv_stats = orderer.get_pv_stats();
let killer_stats = orderer.get_killer_move_stats();
let history_stats = orderer.get_history_stats();

let mut config = orderer.get_config().clone();

// If PV hit rate is high, increase weight
if pv_stats.2 > 40.0 {
    config.weights.pv_move_weight = 12000;
}

// If killer hit rate is low, decrease weight
if killer_stats.2 < 15.0 {
    config.weights.killer_move_weight = 3000;
}

// If history hit rate is high, increase weight
if history_stats.2 > 50.0 {
    config.weights.history_weight = 400;
}

orderer.set_config(config);
```

### 3. Memory Management

**Monitor and manage memory usage:**

```rust
const MAX_MEMORY_BYTES: usize = 10 * 1024 * 1024; // 10 MB

let current_memory = orderer.get_current_memory_usage();

if current_memory > MAX_MEMORY_BYTES {
    println!("Memory usage high: {} bytes, clearing caches", current_memory);
    orderer.clear_caches();
}

// After clearing
let new_memory = orderer.get_current_memory_usage();
println!("Memory after clearing: {} bytes", new_memory);
```

### 4. Adaptive Configuration

**Automatically adjust based on game phase:**

```rust
// Detect game phase
let move_count = 40; // Example
let material_balance = 100; // Example
let tactical_complexity = 0.5; // Example

orderer.update_game_phase(move_count, material_balance, tactical_complexity);

// Configuration automatically adjusts for:
// - Opening: Development and center control
// - Middlegame: Tactical and balanced play
// - Endgame: King safety and promotion
```

## Profiling and Monitoring

### Real-Time Performance Monitoring

```rust
use shogi_engine::time_utils::TimeSource;

let start = TimeSource::now();

// Perform operations
for _ in 0..1000 {
    let _ = orderer.order_moves(&moves);
}

let elapsed_ms = start.elapsed_ms();
let moves_per_ms = (1000.0 * moves.len() as f64) / elapsed_ms as f64;

println!("Performance: {:.0} moves/ms", moves_per_ms);
println!("Time per ordering: {:.2}μs", (elapsed_ms as f64 * 1000.0) / 1000.0);
```

### Statistics Export

```rust
// Get comprehensive statistics
let stats = orderer.get_stats();

// Export to JSON (if serde feature enabled)
let json = serde_json::to_string_pretty(&stats).unwrap();
std::fs::write("move_ordering_stats.json", json).unwrap();
```

### Hot Path Profiling

```rust
let stats = orderer.get_stats();
let hot_path = &stats.hot_path_stats;

println!("Hot Path Statistics:");
println!("  score_move calls: {}", hot_path.score_move_calls);
println!("  cache lookups: {}", hot_path.cache_lookups);
println!("  hash calculations: {}", hot_path.hash_calculations);
println!("  Time in score_move: {}μs", hot_path.score_move_time_us);
println!("  Time in cache ops: {}μs", hot_path.cache_time_us);
println!("  Time in hash ops: {}μs", hot_path.hash_time_us);
```

## Common Performance Issues

### Issue 1: Slow Move Ordering

**Problem**: Ordering takes > 100μs per move consistently

**Diagnosis:**
```rust
let stats = orderer.get_stats();
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate);
println!("SEE calculations: {}", stats.see_calculations);
println!("Avg SEE time: {:.2}μs", stats.avg_see_calculation_time_us);
```

**Solutions:**
1. If cache hit rate < 50%: Increase cache size
2. If SEE time is high: Disable SEE or increase SEE cache
3. If many cache misses: Pre-warm cache for common positions

### Issue 2: Memory Growth

**Problem**: Memory usage increases over time

**Diagnosis:**
```rust
let current = orderer.get_current_memory_usage();
let peak = orderer.get_peak_memory_usage();
println!("Current: {} bytes, Peak: {} bytes", current, peak);

if peak > current * 2 {
    println!("Warning: Significant memory fluctuation");
}
```

**Solutions:**
1. Clear caches periodically:
   ```rust
   if search_count % 100 == 0 {
       orderer.clear_caches();
   }
   ```

2. Use memory-optimized configuration
3. Reduce cache sizes

### Issue 3: Low Heuristic Hit Rates

**Problem**: PV/Killer/History hit rates < 20%

**Diagnosis:**
```rust
let pv_stats = orderer.get_pv_stats();
let killer_stats = orderer.get_killer_move_stats();
let history_stats = orderer.get_history_stats();

println!("PV hit rate: {:.2}%", pv_stats.2);
println!("Killer hit rate: {:.2}%", killer_stats.2);
println!("History hit rate: {:.2}%", history_stats.2);
```

**Solutions:**
1. Ensure heuristics are being updated:
   ```rust
   // After finding good move
   orderer.update_pv_move(&board, &captured_pieces, player, depth, move_, score);
   orderer.add_killer_move(move_.clone());
   orderer.update_history(&move_, true, depth);
   ```

2. Increase max killer moves:
   ```rust
   config.heuristic_config.max_killer_moves = 3;
   ```

3. Reduce history aging frequency:
   ```rust
   config.heuristic_config.history_aging_frequency = 200;
   ```

## Performance Benchmarks

### Expected Performance Levels

| Configuration | Ordering Time | Cache Hit Rate | Memory Usage |
|--------------|---------------|----------------|--------------|
| Performance Optimized | 5-10μs | 80-90% | 8-12 MB |
| Memory Optimized | 20-50μs | 60-70% | 1-2 MB |
| Debug Optimized | 50-100μs | 50-60% | 5-8 MB |

### Benchmark Your Configuration

```rust
use shogi_engine::time_utils::TimeSource;

fn benchmark_move_ordering(orderer: &mut MoveOrdering, moves: &[Move]) {
    let iterations = 1000;
    let start = TimeSource::now();
    
    for _ in 0..iterations {
        let _ = orderer.order_moves(moves);
    }
    
    let elapsed_ms = start.elapsed_ms();
    let avg_us = (elapsed_ms as f64 * 1000.0) / iterations as f64;
    
    println!("Benchmark Results:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {}ms", elapsed_ms);
    println!("  Average per ordering: {:.2}μs", avg_us);
    
    let stats = orderer.get_stats();
    println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate);
}
```

## Best Practices for Performance

### 1. Pre-warm Caches

```rust
// Before starting search, order common moves to warm cache
let opening_moves = generate_common_opening_moves();
for move_set in opening_moves {
    let _ = orderer.order_moves(&move_set);
}
```

### 2. Batch Operations

```rust
// Instead of ordering one move at a time
// Order all moves together for better cache utilization
let all_ordered = orderer.order_moves(&all_moves)?;
```

### 3. Clear Stale Data

```rust
// Clear between games
orderer.clear_killer_moves();
orderer.clear_caches();

// Clear PV at root
orderer.clear_pv_move(0);
```

### 4. Monitor and Adapt

```rust
// Periodic performance check
if iterations % 1000 == 0 {
    let stats = orderer.get_stats();
    
    if stats.cache_hit_rate < 60.0 {
        println!("Warning: Low cache hit rate {:.2}%", stats.cache_hit_rate);
        // Consider increasing cache size
    }
    
    if stats.avg_ordering_time_us > 100.0 {
        println!("Warning: Slow ordering {:.2}μs", stats.avg_ordering_time_us);
        // Consider disabling expensive heuristics
    }
}
```

### 5. Use Appropriate Methods

```rust
// Root node: Use full ordering with all heuristics
let root_ordered = orderer.order_moves_with_all_heuristics(
    &moves, &board, &captured_pieces, player, depth
);

// Internal nodes: Use cached ordering when possible
let internal_ordered = orderer.order_moves(&moves)?;

// Leaf nodes: Minimal ordering
let leaf_ordered = moves.to_vec(); // Or simple ordering
```

## Advanced Optimization

### Dynamic Weight Adjustment

```rust
// Enable advanced features
orderer.set_advanced_features_enabled(true);

// Automatically adjust weights based on performance
orderer.adjust_weights_dynamically();

// Check adjusted weights
let weights = orderer.get_weights();
println!("Adjusted weights:");
println!("  PV: {}", weights.pv_move_weight);
println!("  Killer: {}", weights.killer_move_weight);
println!("  History: {}", weights.history_weight);
```

### Position-Specific Strategies

```rust
// Update game phase for automatic strategy adjustment
orderer.update_game_phase(move_count, material_balance, tactical_complexity);

// Different strategies apply automatically:
// - Opening: Development-focused
// - Middlegame: Tactical-focused  
// - Endgame: King-safety-focused
```

### Cache-Friendly Patterns

```rust
// Use consistent depth values for better cache reuse
// Instead of random depths, use structured iterative deepening
for depth in 1..=10 {
    orderer.set_current_depth(depth);
    let ordered = orderer.order_moves_with_all_heuristics(
        &moves, &board, &captured_pieces, player, depth
    );
    // ...
}
```

## Profiling Tools

### Built-in Profiling

```rust
// Hot path statistics show where time is spent
let stats = orderer.get_stats();
let hot_path = &stats.hot_path_stats;

let total_time = hot_path.score_move_time_us + 
                 hot_path.cache_time_us + 
                 hot_path.hash_time_us;

println!("Time distribution:");
println!("  Scoring: {:.1}%", (hot_path.score_move_time_us as f64 / total_time as f64) * 100.0);
println!("  Caching: {:.1}%", (hot_path.cache_time_us as f64 / total_time as f64) * 100.0);
println!("  Hashing: {:.1}%", (hot_path.hash_time_us as f64 / total_time as f64) * 100.0);
```

### External Profiling

For deeper profiling, use external tools:

```bash
# Flame graph profiling (non-WASM)
cargo install flamegraph
cargo flamegraph --example move_ordering_performance

# Criterion benchmarks
cargo bench --bench move_ordering_benchmarks
```

## Recommendations Summary

| Scenario | Cache Size | SEE | Max Killer | History Aging |
|----------|-----------|-----|------------|---------------|
| Fast Search | Large (500k) | No | 2 | 200 |
| Quality Search | Large (500k) | Yes | 3 | 150 |
| Low Memory | Small (10k) | No | 1 | 50 |
| Blitz Games | Medium (100k) | No | 2 | 100 |
| Analysis | Very Large (1M) | Yes | 4 | 300 |
| WASM/Mobile | Small (5k) | No | 1 | 50 |

## Conclusion

Performance tuning move ordering involves balancing:
- **Speed vs Quality**: Fast ordering vs accurate ordering
- **Memory vs Performance**: Cache size vs memory usage
- **Simplicity vs Features**: Basic heuristics vs advanced features

Start with `MoveOrderingConfig::performance_optimized()` and adjust based on your specific needs and constraints.

For most competitive play, the default performance-optimized configuration provides excellent results. Only tune further if you have specific requirements or constraints.



