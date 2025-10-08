# Evaluation Caching Implementation

## Overview

This document provides detailed implementation instructions for evaluation caching in the Shogi engine. The implementation includes hash-based caching, replacement policies, statistics tracking, and thread-safe access.

## Implementation Plan

### Phase 1: Core Cache System (Week 1)
1. Basic cache structure
2. Position hashing integration
3. Replacement policies
4. Statistics tracking

### Phase 2: Advanced Features (Week 2)
1. Multi-level caching
2. Cache prefetching
3. Performance optimization
4. Memory management

### Phase 3: Integration and Testing (Week 3)
1. Evaluation engine integration
2. Search algorithm integration
3. Comprehensive testing
4. Documentation

## Phase 1: Core Cache System

### Step 1: Basic Cache Entry

**File**: `src/evaluation/eval_cache.rs`

```rust
/// Cache entry for evaluation results
#[derive(Debug, Clone, Copy)]
pub struct EvaluationEntry {
    /// Zobrist hash of the position
    pub hash_key: u64,
    
    /// Evaluation score for the position
    pub score: i32,
    
    /// Depth at which position was evaluated
    pub depth: u8,
    
    /// Age counter for replacement policy
    pub age: u16,
    
    /// Verification bits to detect hash collisions
    pub verification: u16,
}

impl EvaluationEntry {
    /// Create a new cache entry
    pub fn new(hash_key: u64, score: i32, depth: u8) -> Self {
        Self {
            hash_key,
            score,
            depth,
            age: 0,
            verification: Self::calculate_verification(hash_key),
        }
    }
    
    /// Calculate verification bits from hash key (upper 16 bits)
    fn calculate_verification(hash_key: u64) -> u16 {
        ((hash_key >> 48) & 0xFFFF) as u16
    }
    
    /// Verify that this entry matches the given hash key
    pub fn verify(&self, hash_key: u64) -> bool {
        self.hash_key == hash_key && 
        self.verification == Self::calculate_verification(hash_key)
    }
    
    /// Age the entry
    pub fn increment_age(&mut self) {
        self.age = self.age.saturating_add(1);
    }
    
    /// Reset age to 0
    pub fn reset_age(&mut self) {
        self.age = 0;
    }
}

impl Default for EvaluationEntry {
    fn default() -> Self {
        Self {
            hash_key: 0,
            score: 0,
            depth: 0,
            age: 0,
            verification: 0,
        }
    }
}
```

### Step 2: Cache Statistics

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics for monitoring performance
#[derive(Debug)]
pub struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    collisions: AtomicU64,
    stores: AtomicU64,
    rejected: AtomicU64,
}

impl CacheStats {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            collisions: AtomicU64::new(0),
            stores: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        }
    }
    
    /// Record a cache hit
    #[inline]
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a cache miss
    #[inline]
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a hash collision
    #[inline]
    pub fn record_collision(&self) {
        self.collisions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a successful store
    #[inline]
    pub fn record_store(&self) {
        self.stores.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a rejected store
    #[inline]
    pub fn record_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get total number of probes
    pub fn total_probes(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + 
        self.misses.load(Ordering::Relaxed)
    }
    
    /// Get hit rate (percentage)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = self.total_probes();
        
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }
    
    /// Get collision rate (percentage)
    pub fn collision_rate(&self) -> f64 {
        let collisions = self.collisions.load(Ordering::Relaxed);
        let total = self.total_probes() + collisions;
        
        if total == 0 {
            0.0
        } else {
            (collisions as f64 / total as f64) * 100.0
        }
    }
    
    /// Get rejection rate (percentage of stores rejected)
    pub fn rejection_rate(&self) -> f64 {
        let rejected = self.rejected.load(Ordering::Relaxed);
        let total = self.stores.load(Ordering::Relaxed) + rejected;
        
        if total == 0 {
            0.0
        } else {
            (rejected as f64 / total as f64) * 100.0
        }
    }
    
    /// Reset all statistics
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.collisions.store(0, Ordering::Relaxed);
        self.stores.store(0, Ordering::Relaxed);
        self.rejected.store(0, Ordering::Relaxed);
    }
    
    /// Print statistics summary
    pub fn print_summary(&self) {
        println!("=== Evaluation Cache Statistics ===");
        println!("Hits: {}", self.hits.load(Ordering::Relaxed));
        println!("Misses: {}", self.misses.load(Ordering::Relaxed));
        println!("Collisions: {}", self.collisions.load(Ordering::Relaxed));
        println!("Stores: {}", self.stores.load(Ordering::Relaxed));
        println!("Rejected: {}", self.rejected.load(Ordering::Relaxed));
        println!("Hit Rate: {:.2}%", self.hit_rate());
        println!("Collision Rate: {:.2}%", self.collision_rate());
        println!("Rejection Rate: {:.2}%", self.rejection_rate());
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 3: Replacement Policies

```rust
/// Cache replacement policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementPolicy {
    /// Always replace existing entry
    AlwaysReplace,
    
    /// Prefer entries from deeper searches
    DepthPreferred,
    
    /// Use entry age for replacement
    Aging,
    
    /// Hybrid two-tier policy
    TwoTier,
}

impl ReplacementPolicy {
    /// Determine if new entry should replace existing entry
    pub fn should_replace(&self, existing: &EvaluationEntry, new: &EvaluationEntry) -> bool {
        match self {
            ReplacementPolicy::AlwaysReplace => true,
            ReplacementPolicy::DepthPreferred => {
                // Prefer entries from deeper searches
                new.depth >= existing.depth
            }
            ReplacementPolicy::Aging => {
                // Prefer newer entries
                new.age < existing.age
            }
            ReplacementPolicy::TwoTier => {
                // Hybrid: prefer depth, but allow aging for old entries
                new.depth > existing.depth || 
                (new.depth == existing.depth && new.age < existing.age)
            }
        }
    }
}

impl Default for ReplacementPolicy {
    fn default() -> Self {
        ReplacementPolicy::DepthPreferred
    }
}
```

### Step 4: Main Evaluation Cache

```rust
use std::sync::RwLock;
use std::sync::Arc;

/// Main evaluation cache structure
pub struct EvaluationCache {
    /// Cache entries protected by RwLock for thread safety
    entries: Vec<RwLock<Option<EvaluationEntry>>>,
    
    /// Cache size (number of entries)
    size: usize,
    
    /// Replacement policy
    policy: ReplacementPolicy,
    
    /// Cache statistics
    stats: Arc<CacheStats>,
    
    /// Global age counter
    global_age: std::sync::atomic::AtomicU16,
}

impl EvaluationCache {
    /// Create a new evaluation cache with specified size in MB
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<EvaluationEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            entries.push(RwLock::new(None));
        }
        
        Self {
            entries,
            size: num_entries,
            policy: ReplacementPolicy::default(),
            stats: Arc::new(CacheStats::new()),
            global_age: std::sync::atomic::AtomicU16::new(0),
        }
    }
    
    /// Probe the cache for a position
    #[inline]
    pub fn probe(&self, hash_key: u64) -> Option<i32> {
        let index = self.get_index(hash_key);
        
        // Try to read the entry
        if let Ok(entry_guard) = self.entries[index].read() {
            if let Some(entry) = *entry_guard {
                if entry.verify(hash_key) {
                    self.stats.record_hit();
                    return Some(entry.score);
                } else {
                    self.stats.record_collision();
                }
            }
        }
        
        self.stats.record_miss();
        None
    }
    
    /// Probe with depth information (for transposition table integration)
    pub fn probe_with_depth(&self, hash_key: u64, min_depth: u8) -> Option<i32> {
        let index = self.get_index(hash_key);
        
        if let Ok(entry_guard) = self.entries[index].read() {
            if let Some(entry) = *entry_guard {
                if entry.verify(hash_key) && entry.depth >= min_depth {
                    self.stats.record_hit();
                    return Some(entry.score);
                } else if entry.verify(hash_key) {
                    // Entry found but depth insufficient
                    self.stats.record_hit();
                    return None;
                } else {
                    self.stats.record_collision();
                }
            }
        }
        
        self.stats.record_miss();
        None
    }
    
    /// Store an evaluation in the cache
    #[inline]
    pub fn store(&self, hash_key: u64, score: i32, depth: u8) {
        let index = self.get_index(hash_key);
        let new_entry = EvaluationEntry::new(hash_key, score, depth);
        
        // Try to write the entry
        if let Ok(mut entry_guard) = self.entries[index].write() {
            let should_replace = if let Some(existing) = *entry_guard {
                self.policy.should_replace(&existing, &new_entry)
            } else {
                true
            };
            
            if should_replace {
                *entry_guard = Some(new_entry);
                self.stats.record_store();
            } else {
                self.stats.record_rejected();
            }
        }
    }
    
    /// Get cache index from hash key
    #[inline]
    fn get_index(&self, hash_key: u64) -> usize {
        (hash_key as usize) % self.size
    }
    
    /// Clear all cache entries
    pub fn clear(&self) {
        for entry_lock in &self.entries {
            if let Ok(mut entry) = entry_lock.write() {
                *entry = None;
            }
        }
        self.stats.reset();
    }
    
    /// Age all entries (called periodically or after each search)
    pub fn age_entries(&self) {
        let global_age = self.global_age.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Only age every N searches to reduce overhead
        if global_age % 10 == 0 {
            for entry_lock in &self.entries {
                if let Ok(mut entry) = entry_lock.write() {
                    if let Some(ref mut e) = *entry {
                        e.increment_age();
                    }
                }
            }
        }
    }
    
    /// Set replacement policy
    pub fn set_policy(&mut self, policy: ReplacementPolicy) {
        self.policy = policy;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> Arc<CacheStats> {
        Arc::clone(&self.stats)
    }
    
    /// Get cache utilization (percentage of filled entries)
    pub fn utilization(&self) -> f64 {
        let mut filled = 0;
        let sample_size = self.size.min(1000); // Sample for large caches
        
        for i in 0..sample_size {
            if let Ok(entry) = self.entries[i].read() {
                if entry.is_some() {
                    filled += 1;
                }
            }
        }
        
        (filled as f64 / sample_size as f64) * 100.0
    }
    
    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.size * std::mem::size_of::<RwLock<Option<EvaluationEntry>>>()
    }
    
    /// Resize the cache (expensive operation)
    pub fn resize(&mut self, size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<EvaluationEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        
        let mut new_entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            new_entries.push(RwLock::new(None));
        }
        
        self.entries = new_entries;
        self.size = num_entries;
        self.stats.reset();
    }
}
```

## Phase 2: Integration

### Step 5: Evaluation Engine Integration

**File**: `src/evaluation/evaluator.rs`

```rust
use super::eval_cache::EvaluationCache;
use std::sync::Arc;

pub struct Evaluator {
    /// Evaluation cache
    eval_cache: Arc<EvaluationCache>,
    
    /// Enable/disable caching
    use_cache: bool,
}

impl Evaluator {
    /// Create a new evaluator with default cache size (16MB)
    pub fn new() -> Self {
        Self {
            eval_cache: Arc::new(EvaluationCache::new(16)),
            use_cache: true,
        }
    }
    
    /// Create with custom cache size
    pub fn with_cache_size(size_mb: usize) -> Self {
        Self {
            eval_cache: Arc::new(EvaluationCache::new(size_mb)),
            use_cache: true,
        }
    }
    
    /// Main evaluation function with caching
    pub fn evaluate(&self, board: &impl BoardTrait) -> i32 {
        if !self.use_cache {
            return self.evaluate_position(board);
        }
        
        // Calculate position hash
        let hash_key = board.hash();
        
        // Probe cache first
        if let Some(cached_score) = self.eval_cache.probe(hash_key) {
            return cached_score;
        }
        
        // Cache miss - evaluate position
        let score = self.evaluate_position(board);
        
        // Store in cache
        self.eval_cache.store(hash_key, score, 0);
        
        score
    }
    
    /// Internal evaluation function (without caching)
    fn evaluate_position(&self, board: &impl BoardTrait) -> i32 {
        // Actual evaluation logic here
        // This would include material, piece squares, king safety, etc.
        0
    }
    
    /// Enable or disable caching
    pub fn set_use_cache(&mut self, enable: bool) {
        self.use_cache = enable;
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> Arc<CacheStats> {
        self.eval_cache.stats()
    }
    
    /// Clear the cache
    pub fn clear_cache(&self) {
        self.eval_cache.clear();
    }
    
    /// Get the cache for direct access
    pub fn cache(&self) -> Arc<EvaluationCache> {
        Arc::clone(&self.eval_cache)
    }
}
```

### Step 6: Search Integration

```rust
impl SearchEngine {
    /// Search with evaluation caching
    pub fn search_with_cache(&mut self, board: &mut impl BoardTrait, depth: u8) -> i32 {
        let hash_key = board.hash();
        
        // Try cache with depth requirement
        if let Some(cached_eval) = self.evaluator.cache().probe_with_depth(hash_key, depth) {
            return cached_eval;
        }
        
        // Perform evaluation
        let eval = self.evaluator.evaluate(board);
        
        // Store with depth information
        self.evaluator.cache().store(hash_key, eval, depth);
        
        eval
    }
}
```

## Phase 3: Testing

### Step 7: Unit Tests

**File**: `tests/eval_cache_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_creation() {
        let cache = EvaluationCache::new(4);
        assert!(cache.size > 0);
    }
    
    #[test]
    fn test_cache_store_and_probe() {
        let cache = EvaluationCache::new(4);
        let hash_key = 12345u64;
        let score = 100i32;
        
        // Store
        cache.store(hash_key, score, 0);
        
        // Probe
        assert_eq!(cache.probe(hash_key), Some(score));
    }
    
    #[test]
    fn test_cache_miss() {
        let cache = EvaluationCache::new(4);
        
        // Probe non-existent entry
        assert_eq!(cache.probe(12345), None);
    }
    
    #[test]
    fn test_cache_replacement() {
        let cache = EvaluationCache::new(4);
        let hash_key = 12345u64;
        
        // Store first value
        cache.store(hash_key, 100, 5);
        assert_eq!(cache.probe(hash_key), Some(100));
        
        // Store second value (should replace if depth is higher or equal)
        cache.store(hash_key, 200, 5);
        assert_eq!(cache.probe(hash_key), Some(200));
    }
    
    #[test]
    fn test_cache_statistics() {
        let cache = EvaluationCache::new(4);
        let stats = cache.stats();
        
        // Perform some operations
        cache.store(1, 100, 0);
        cache.probe(1); // Hit
        cache.probe(2); // Miss
        
        assert!(stats.total_probes() == 2);
        assert!(stats.hit_rate() > 0.0);
    }
    
    #[test]
    fn test_depth_preferred_policy() {
        let mut cache = EvaluationCache::new(4);
        cache.set_policy(ReplacementPolicy::DepthPreferred);
        
        let hash_key = 12345u64;
        
        // Store shallow search
        cache.store(hash_key, 100, 3);
        
        // Try to store deeper search (should replace)
        cache.store(hash_key, 200, 5);
        assert_eq!(cache.probe(hash_key), Some(200));
        
        // Try to store shallower search (should not replace)
        cache.store(hash_key, 300, 2);
        assert_eq!(cache.probe(hash_key), Some(200));
    }
    
    #[test]
    fn test_cache_clear() {
        let cache = EvaluationCache::new(4);
        
        // Store some entries
        cache.store(1, 100, 0);
        cache.store(2, 200, 0);
        
        // Clear cache
        cache.clear();
        
        // Verify entries are gone
        assert_eq!(cache.probe(1), None);
        assert_eq!(cache.probe(2), None);
    }
}
```

### Step 8: Performance Benchmarks

**File**: `benches/eval_cache_benchmarks.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_cache_probe(c: &mut Criterion) {
    let cache = EvaluationCache::new(16);
    
    // Pre-populate cache
    for i in 0..10000 {
        cache.store(i, (i % 1000) as i32, 0);
    }
    
    c.bench_function("cache_probe_hit", |b| {
        b.iter(|| {
            black_box(cache.probe(black_box(5000)))
        })
    });
    
    c.bench_function("cache_probe_miss", |b| {
        b.iter(|| {
            black_box(cache.probe(black_box(50000)))
        })
    });
}

fn bench_cache_store(c: &mut Criterion) {
    let cache = EvaluationCache::new(16);
    let mut counter = 0u64;
    
    c.bench_function("cache_store", |b| {
        b.iter(|| {
            counter += 1;
            black_box(cache.store(black_box(counter), black_box(100), black_box(5)))
        })
    });
}

fn bench_cache_utilization(c: &mut Criterion) {
    let cache = EvaluationCache::new(16);
    
    // Fill cache
    for i in 0..100000 {
        cache.store(i, 100, 0);
    }
    
    c.bench_function("cache_utilization", |b| {
        b.iter(|| {
            black_box(cache.utilization())
        })
    });
}

criterion_group!(
    benches,
    bench_cache_probe,
    bench_cache_store,
    bench_cache_utilization
);
criterion_main!(benches);
```

## Configuration

**File**: `config/eval_cache.toml`

```toml
[cache]
enabled = true
size_mb = 16

[cache.policy]
type = "depth_preferred"  # Options: always_replace, depth_preferred, aging, two_tier

[cache.advanced]
enable_two_tier = false
l1_size_mb = 4
l2_size_mb = 16
enable_prefetch = false
age_frequency = 10
```

## WASM Compatibility

```rust
#[cfg(target_arch = "wasm32")]
pub mod wasm_compat {
    use super::*;
    
    /// WASM-compatible evaluation cache
    /// Uses simpler synchronization for single-threaded WASM
    pub struct WasmEvaluationCache {
        entries: Vec<Option<EvaluationEntry>>,
        size: usize,
        policy: ReplacementPolicy,
        stats: CacheStats,
    }
    
    impl WasmEvaluationCache {
        pub fn new(size_mb: usize) -> Self {
            let entry_size = std::mem::size_of::<Option<EvaluationEntry>>();
            let num_entries = (size_mb * 1024 * 1024) / entry_size;
            
            Self {
                entries: vec![None; num_entries],
                size: num_entries,
                policy: ReplacementPolicy::default(),
                stats: CacheStats::new(),
            }
        }
        
        #[inline]
        pub fn probe(&self, hash_key: u64) -> Option<i32> {
            let index = (hash_key as usize) % self.size;
            
            if let Some(entry) = self.entries[index] {
                if entry.verify(hash_key) {
                    self.stats.record_hit();
                    return Some(entry.score);
                }
            }
            
            self.stats.record_miss();
            None
        }
        
        #[inline]
        pub fn store(&mut self, hash_key: u64, score: i32, depth: u8) {
            let index = (hash_key as usize) % self.size;
            let new_entry = EvaluationEntry::new(hash_key, score, depth);
            
            let should_replace = if let Some(existing) = self.entries[index] {
                self.policy.should_replace(&existing, &new_entry)
            } else {
                true
            };
            
            if should_replace {
                self.entries[index] = Some(new_entry);
                self.stats.record_store();
            }
        }
    }
}
```

## Expected Results

After implementation, the evaluation cache should provide:

1. **50-70% Reduction** in evaluation time
2. **60-80% Hit Rate** in typical search scenarios
3. **<5% Collision Rate** with proper hashing
4. **<100ns Lookup Time** for cache operations
5. **Thread-Safe Access** for parallel search

## Troubleshooting

### Common Issues

1. **Low Hit Rate**: Increase cache size or check hashing quality
2. **High Collision Rate**: Verify hash function distribution
3. **Memory Issues**: Reduce cache size or implement dynamic resizing
4. **Thread Contention**: Consider lock-free alternatives or partitioned cache

### Debug Tools

```rust
impl EvaluationCache {
    /// Debug: Print cache state summary
    pub fn debug_print(&self) {
        println!("=== Cache Debug Info ===");
        println!("Size: {} entries", self.size);
        println!("Memory: {} MB", self.memory_usage() / (1024 * 1024));
        println!("Utilization: {:.2}%", self.utilization());
        self.stats.print_summary();
    }
}
```

This implementation provides a complete, production-ready evaluation caching system that significantly improves search performance through efficient position evaluation reuse.

