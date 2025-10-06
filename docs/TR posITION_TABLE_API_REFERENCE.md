# Transposition Table API Reference

This document provides comprehensive API documentation for the transposition table system in the Shogi engine.

## Table of Contents

1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Configuration](#configuration)
4. [Thread Safety](#thread-safety)
5. [Performance Optimization](#performance-optimization)
6. [Error Handling](#error-handling)
7. [Statistics and Monitoring](#statistics-and-monitoring)
8. [Examples](#examples)
9. [Best Practices](#best-practices)

## Overview

The transposition table system provides high-performance storage and retrieval of search results for the Shogi engine. It features thread-safe operations, WASM compatibility, and comprehensive performance monitoring.

### Key Features

- **Thread Safety**: Safe for concurrent access across multiple threads
- **WASM Compatibility**: Works in both native and WebAssembly environments
- **Performance Optimized**: Uses atomic operations and cache-line alignment
- **Memory Efficient**: Compact entry storage with configurable size
- **Statistics Tracking**: Comprehensive performance and usage statistics

## Core Components

### ThreadSafeTranspositionTable

The main transposition table implementation.

```rust
pub struct ThreadSafeTranspositionTable {
    // Internal implementation details
}
```

#### Methods

##### `new(config: TranspositionConfig) -> Self`

Creates a new thread-safe transposition table with the specified configuration.

**Parameters:**
- `config`: Configuration for the transposition table

**Returns:**
- New `ThreadSafeTranspositionTable` instance

**Example:**
```rust
let config = TranspositionConfig::default();
let tt = ThreadSafeTranspositionTable::new(config);
```

##### `store(&mut self, entry: TranspositionEntry)`

Stores a transposition entry in the table.

**Parameters:**
- `entry`: The transposition entry to store

**Example:**
```rust
let entry = TranspositionEntry {
    hash_key: 12345,
    depth: 3,
    score: 100,
    flag: TranspositionFlag::Exact,
    best_move: None,
    age: 0,
};
tt.store(entry);
```

##### `probe(&self, hash: u64, depth: u8) -> Option<TranspositionEntry>`

Retrieves a transposition entry from the table.

**Parameters:**
- `hash`: The hash key to look up
- `depth`: The search depth

**Returns:**
- `Some(entry)` if found, `None` otherwise

**Example:**
```rust
if let Some(entry) = tt.probe(12345, 3) {
    println!("Found entry with score: {}", entry.score);
}
```

##### `clear(&mut self)`

Clears all entries from the transposition table.

**Example:**
```rust
tt.clear();
```

##### `size(&self) -> usize`

Returns the current number of entries in the table.

**Returns:**
- Number of entries

##### `get_stats(&self) -> ThreadSafeStatsSnapshot`

Returns comprehensive statistics about the transposition table.

**Returns:**
- Statistics snapshot

**Example:**
```rust
let stats = tt.get_stats();
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
```

### TranspositionEntry

Represents a single entry in the transposition table.

```rust
pub struct TranspositionEntry {
    pub hash_key: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
    pub age: u32,
}
```

#### Fields

- `hash_key`: Unique identifier for the position
- `depth`: Search depth at which this entry was created
- `score`: Evaluation score for the position
- `flag`: Type of score (Exact, LowerBound, UpperBound)
- `best_move`: Best move found for this position
- `age`: Age counter for replacement policies

### TranspositionFlag

Indicates the type of score stored in a transposition entry.

```rust
pub enum TranspositionFlag {
    Exact,      // Exact score (between alpha and beta)
    LowerBound, // Lower bound (score >= beta)
    UpperBound, // Upper bound (score <= alpha)
}
```

## Configuration

### TranspositionConfig

Configuration options for the transposition table.

```rust
pub struct TranspositionConfig {
    pub table_size: usize,
    pub replacement_policy: ReplacementPolicy,
    pub enable_statistics: bool,
    pub enable_cache_line_alignment: bool,
    pub enable_prefetching: bool,
}
```

#### Fields

- `table_size`: Size of the transposition table (must be power of 2)
- `replacement_policy`: Policy for handling hash collisions
- `enable_statistics`: Whether to collect performance statistics
- `enable_cache_line_alignment`: Whether to align entries to cache lines
- `enable_prefetching`: Whether to enable data prefetching

#### Predefined Configurations

##### `default() -> Self`

Creates a default configuration with balanced performance and memory usage.

##### `performance_optimized() -> Self`

Creates a configuration optimized for maximum performance.

##### `memory_optimized() -> Self`

Creates a configuration optimized for minimal memory usage.

### ReplacementPolicy

Defines how to handle hash collisions.

```rust
pub enum ReplacementPolicy {
    AlwaysReplace,        // Always replace existing entries
    DepthPreferred,       // Prefer entries with higher depth
    AgeBased,            // Replace oldest entries
    ExactPreferred,      // Prefer exact scores over bounds
}
```

## Thread Safety

The transposition table is designed for thread-safe operation:

- **Atomic Operations**: All operations use atomic instructions
- **Lock-Free Design**: No mutexes or locks required
- **Concurrent Access**: Multiple threads can safely access the same table
- **WASM Compatibility**: Single-threaded WASM environment supported

### Thread Safety Guarantees

1. **Store Operations**: Thread-safe storage of entries
2. **Probe Operations**: Thread-safe retrieval of entries
3. **Statistics**: Thread-safe collection of performance metrics
4. **Configuration**: Thread-safe configuration access

## Performance Optimization

### Cache Line Alignment

Entries are aligned to cache line boundaries for optimal performance:

```rust
let config = TranspositionConfig {
    enable_cache_line_alignment: true,
    ..TranspositionConfig::performance_optimized()
};
```

### Prefetching

Data prefetching can be enabled for better cache performance:

```rust
let config = TranspositionConfig {
    enable_prefetching: true,
    ..TranspositionConfig::performance_optimized()
};
```

### Table Size Optimization

Choose table size based on available memory and search depth:

- **Small (4K-16K entries)**: 64KB-256KB memory
- **Medium (64K-256K entries)**: 1MB-4MB memory
- **Large (1M+ entries)**: 16MB+ memory

## Error Handling

### Error Types

```rust
pub enum TranspositionError {
    InvalidConfiguration,
    MemoryAllocationFailed,
    HashCollision,
    StatisticsDisabled,
}
```

### Error Recovery

The system implements graceful error recovery:

1. **Configuration Errors**: Fall back to safe defaults
2. **Memory Errors**: Reduce table size or disable features
3. **Hash Collisions**: Use replacement policies
4. **Statistics Errors**: Disable statistics collection

### Error Handling Example

```rust
let result = std::panic::catch_unwind(|| {
    let config = TranspositionConfig::default();
    let mut tt = ThreadSafeTranspositionTable::new(config);
    // Operations...
});

match result {
    Ok(_) => println!("Operation successful"),
    Err(_) => println!("Operation failed, using fallback"),
}
```

## Statistics and Monitoring

### ThreadSafeStatsSnapshot

Comprehensive statistics about transposition table performance.

```rust
pub struct ThreadSafeStatsSnapshot {
    pub total_probes: u64,
    pub total_stores: u64,
    pub hit_rate: f64,
    pub collision_rate: f64,
    pub table_size: usize,
    pub replacement_count: u64,
    pub atomic_operations: u64,
}
```

#### Fields

- `total_probes`: Total number of probe operations
- `total_stores`: Total number of store operations
- `hit_rate`: Percentage of successful probes
- `collision_rate`: Percentage of hash collisions
- `table_size`: Current table size
- `replacement_count`: Number of entries replaced
- `atomic_operations`: Number of atomic operations performed

### Performance Monitoring

Monitor key metrics for performance tuning:

```rust
let stats = tt.get_stats();

// Monitor hit rate (target: > 30%)
if stats.hit_rate < 0.3 {
    println!("Warning: Low hit rate: {:.2}%", stats.hit_rate * 100.0);
}

// Monitor collision rate (target: < 10%)
if stats.collision_rate > 0.1 {
    println!("Warning: High collision rate: {:.2}%", stats.collision_rate * 100.0);
}
```

## Examples

### Basic Usage

```rust
use shogi_engine::search::*;

// Create transposition table
let config = TranspositionConfig::default();
let mut tt = ThreadSafeTranspositionTable::new(config);

// Store an entry
let entry = TranspositionEntry {
    hash_key: 12345,
    depth: 3,
    score: 100,
    flag: TranspositionFlag::Exact,
    best_move: None,
    age: 0,
};
tt.store(entry);

// Retrieve an entry
if let Some(result) = tt.probe(12345, 3) {
    println!("Found entry with score: {}", result.score);
}

// Get statistics
let stats = tt.get_stats();
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
```

### Performance Optimization

```rust
// Use performance-optimized configuration
let config = TranspositionConfig::performance_optimized();
let mut tt = ThreadSafeTranspositionTable::new(config);

// Monitor performance
let stats = tt.get_stats();
if stats.hit_rate < 0.3 {
    // Consider increasing table size
    println!("Low hit rate detected: {:.2}%", stats.hit_rate * 100.0);
}
```

### Error Handling

```rust
let config = TranspositionConfig::default();
let mut tt = ThreadSafeTranspositionTable::new(config);

// Graceful error handling
let entry = TranspositionEntry {
    hash_key: 12345,
    depth: 3,
    score: 100,
    flag: TranspositionFlag::Exact,
    best_move: None,
    age: 0,
};

tt.store(entry);
let result = tt.probe(12345, 3);

match result {
    Some(entry) => println!("Entry found: {}", entry.score),
    None => println!("Entry not found, continuing with search"),
}
```

## Best Practices

### Configuration

1. **Choose appropriate table size** based on available memory
2. **Use predefined configurations** as starting points
3. **Enable statistics** for performance monitoring
4. **Consider memory constraints** in WASM environments

### Performance

1. **Monitor hit rates** and adjust table size accordingly
2. **Use cache line alignment** for better performance
3. **Enable prefetching** for large tables
4. **Choose appropriate replacement policies**

### Memory Management

1. **Estimate memory requirements** before choosing table size
2. **Use memory-optimized configurations** when needed
3. **Monitor memory usage** over time
4. **Consider memory constraints** in different environments

### Thread Safety

1. **No external synchronization required** - the table is thread-safe
2. **Multiple threads can safely access** the same table
3. **Atomic operations ensure consistency**
4. **WASM environments are single-threaded** by design

### Error Handling

1. **Always check return values** from probe operations
2. **Implement fallback strategies** for critical errors
3. **Log errors** for debugging purposes
4. **Gracefully degrade functionality** when possible

### Testing

1. **Test in both native and WASM environments**
2. **Use the comprehensive test suite** for validation
3. **Monitor performance characteristics**
4. **Validate different configurations**

## Migration Guide

### From Basic Transposition Table

If migrating from a basic transposition table implementation:

1. **Replace HashMap with ThreadSafeTranspositionTable**
2. **Update configuration** to use TranspositionConfig
3. **Modify store/retrieve operations** to use new API
4. **Add statistics monitoring** for performance tuning
5. **Test thoroughly** in both native and WASM environments

### API Changes

- `HashMap::insert()` → `ThreadSafeTranspositionTable::store()`
- `HashMap::get()` → `ThreadSafeTranspositionTable::probe()`
- `HashMap::len()` → `ThreadSafeTranspositionTable::size()`
- `HashMap::clear()` → `ThreadSafeTranspositionTable::clear()`

## Troubleshooting

### Common Issues

1. **Low hit rate**: Increase table size or check hash function
2. **High memory usage**: Use memory-optimized configuration
3. **Slow performance**: Use performance-optimized configuration
4. **WASM errors**: Ensure WASM-compatible code paths

### Debugging

1. **Enable statistics** to monitor performance
2. **Check configuration** for invalid parameters
3. **Monitor memory usage** for leaks
4. **Test with different configurations** to isolate issues

### Performance Tuning

1. **Monitor hit rates** and adjust table size
2. **Check collision rates** and verify hash function
3. **Profile operation times** and optimize hot paths
4. **Consider different replacement policies**

## Conclusion

The transposition table system provides a robust, high-performance solution for storing and retrieving search results in the Shogi engine. With its thread-safe design, WASM compatibility, and comprehensive monitoring capabilities, it enables efficient search algorithms while maintaining flexibility and reliability.

For more examples and advanced usage, see the example files in the `examples/` directory.
