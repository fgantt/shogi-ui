# Evaluation Caching Design

## Overview

This document outlines the design for implementing evaluation caching in the Shogi engine. Evaluation caching stores previously calculated position evaluations to avoid redundant calculations, significantly improving search performance.

## Current State

The engine currently recalculates position evaluations every time, even for positions that have been evaluated before, leading to unnecessary computational overhead.

## Design Goals

1. **High Hit Rates**: Maximize cache utilization for best performance
2. **Correctness**: Never return incorrect cached evaluations
3. **Memory Efficiency**: Balance cache size with performance gains
4. **Thread Safety**: Support concurrent access from multiple threads
5. **Low Overhead**: Minimal performance cost for cache operations

## Technical Architecture

### 1. Cache Entry Structure

**Purpose**: Store evaluation results with necessary metadata for validation and management.

**Components**:
- Position hash key
- Evaluation score
- Search depth
- Age/timestamp
- Verification bits

**Implementation**:
```rust
/// Cache entry for evaluation results
#[derive(Debug, Clone, Copy)]
pub struct EvaluationEntry {
    /// Zobrist hash of the position
    pub hash_key: u64,
    
    /// Evaluation score for the position
    pub score: i32,
    
    /// Depth at which position was evaluated (for transposition table integration)
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
    
    /// Calculate verification bits from hash key
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
}
```

### 2. Evaluation Cache Structure

**Purpose**: Manage the hash table of evaluation entries with efficient lookup and replacement.

**Technical Details**:
- Fixed-size hash table
- Configurable size (4MB to 64MB typical)
- Multiple replacement policies
- Collision handling
- Thread-safe access

**Implementation**:
```rust
use std::sync::RwLock;

/// Main evaluation cache structure
pub struct EvaluationCache {
    /// Cache entries
    entries: Vec<RwLock<Option<EvaluationEntry>>>,
    
    /// Cache size (number of entries)
    size: usize,
    
    /// Replacement policy
    policy: ReplacementPolicy,
    
    /// Cache statistics
    stats: CacheStats,
    
    /// Global age counter
    global_age: std::sync::atomic::AtomicU16,
}

impl EvaluationCache {
    /// Create a new evaluation cache
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
            policy: ReplacementPolicy::DepthPreferred,
            stats: CacheStats::new(),
            global_age: std::sync::atomic::AtomicU16::new(0),
        }
    }
    
    /// Probe the cache for a position
    pub fn probe(&self, hash_key: u64) -> Option<i32> {
        let index = self.get_index(hash_key);
        
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
    
    /// Store an evaluation in the cache
    pub fn store(&self, hash_key: u64, score: i32, depth: u8) {
        let index = self.get_index(hash_key);
        let new_entry = EvaluationEntry::new(hash_key, score, depth);
        
        if let Ok(mut entry_guard) = self.entries[index].write() {
            let should_replace = if let Some(existing) = *entry_guard {
                self.should_replace(&existing, &new_entry)
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
    fn get_index(&self, hash_key: u64) -> usize {
        (hash_key as usize) % self.size
    }
    
    /// Determine if new entry should replace existing entry
    fn should_replace(&self, existing: &EvaluationEntry, new: &EvaluationEntry) -> bool {
        match self.policy {
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
    
    /// Clear all cache entries
    pub fn clear(&self) {
        for entry_lock in &self.entries {
            if let Ok(mut entry) = entry_lock.write() {
                *entry = None;
            }
        }
        self.stats.reset();
    }
    
    /// Age all entries (called periodically)
    pub fn age_entries(&self) {
        let global_age = self.global_age.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        for entry_lock in &self.entries {
            if let Ok(mut entry) = entry_lock.write() {
                if let Some(ref mut e) = *entry {
                    e.age = global_age;
                }
            }
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    /// Get cache utilization (percentage of filled entries)
    pub fn utilization(&self) -> f64 {
        let mut filled = 0;
        
        for entry_lock in &self.entries {
            if let Ok(entry) = entry_lock.read() {
                if entry.is_some() {
                    filled += 1;
                }
            }
        }
        
        (filled as f64 / self.size as f64) * 100.0
    }
}
```

### 3. Replacement Policies

**Purpose**: Determine which cache entries to replace when the cache is full.

**Implementation**:
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
    /// Get the default replacement policy
    pub fn default() -> Self {
        ReplacementPolicy::DepthPreferred
    }
}
```

### 4. Cache Statistics

**Purpose**: Track cache performance metrics for monitoring and tuning.

**Implementation**:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics
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
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a hash collision
    pub fn record_collision(&self) {
        self.collisions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a successful store
    pub fn record_store(&self) {
        self.stores.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a rejected store
    pub fn record_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get hit rate (percentage)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }
    
    /// Get collision rate (percentage)
    pub fn collision_rate(&self) -> f64 {
        let collisions = self.collisions.load(Ordering::Relaxed);
        let total = self.hits.load(Ordering::Relaxed) + 
                    self.misses.load(Ordering::Relaxed) + 
                    collisions;
        
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
```

### 5. Multi-Level Cache (Optional)

**Purpose**: Implement a two-tier cache system for better hit rates and performance.

**Implementation**:
```rust
/// Two-tier evaluation cache
pub struct TwoTierCache {
    /// L1 cache (small, fast)
    l1_cache: EvaluationCache,
    
    /// L2 cache (large, slower)
    l2_cache: EvaluationCache,
}

impl TwoTierCache {
    /// Create a new two-tier cache
    pub fn new(l1_size_mb: usize, l2_size_mb: usize) -> Self {
        Self {
            l1_cache: EvaluationCache::new(l1_size_mb),
            l2_cache: EvaluationCache::new(l2_size_mb),
        }
    }
    
    /// Probe both cache tiers
    pub fn probe(&self, hash_key: u64) -> Option<i32> {
        // Try L1 first
        if let Some(score) = self.l1_cache.probe(hash_key) {
            return Some(score);
        }
        
        // Try L2
        if let Some(score) = self.l2_cache.probe(hash_key) {
            // Promote to L1
            self.l1_cache.store(hash_key, score, 0);
            return Some(score);
        }
        
        None
    }
    
    /// Store in both tiers
    pub fn store(&self, hash_key: u64, score: i32, depth: u8) {
        self.l1_cache.store(hash_key, score, depth);
        self.l2_cache.store(hash_key, score, depth);
    }
}
```

### 6. Cache Prefetching (Optional)

**Purpose**: Predictively load likely-to-be-needed evaluations into cache.

**Implementation**:
```rust
/// Cache prefetching manager
pub struct CachePrefetcher {
    cache: Arc<EvaluationCache>,
    prefetch_queue: std::sync::Mutex<Vec<u64>>,
}

impl CachePrefetcher {
    pub fn new(cache: Arc<EvaluationCache>) -> Self {
        Self {
            cache,
            prefetch_queue: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    /// Schedule a position for prefetching
    pub fn schedule_prefetch(&self, hash_key: u64) {
        if let Ok(mut queue) = self.prefetch_queue.lock() {
            queue.push(hash_key);
        }
    }
    
    /// Prefetch scheduled positions (can run in background)
    pub fn prefetch_positions(&self, board: &impl BoardTrait) {
        // Implementation depends on move generation and evaluation
        // This is a simplified version
        
        if let Ok(mut queue) = self.prefetch_queue.lock() {
            for hash_key in queue.drain(..) {
                // Check if already in cache
                if self.cache.probe(hash_key).is_none() {
                    // Would evaluate and store here
                    // In practice, this requires board position reconstruction
                }
            }
        }
    }
}
```

## Integration Points

### Evaluation Engine Integration

```rust
impl Evaluator {
    fn evaluate_with_cache(&self, board: &impl BoardTrait) -> i32 {
        let hash_key = self.hash_position(board);
        
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
}
```

### Search Algorithm Integration

```rust
impl SearchEngine {
    fn search_with_cache(&mut self, board: &mut impl BoardTrait, depth: u8) -> i32 {
        let hash_key = self.hash_position(board);
        
        // Try cache before evaluation
        if let Some(cached_eval) = self.eval_cache.probe(hash_key) {
            return cached_eval;
        }
        
        // Perform evaluation
        let eval = self.evaluate(board);
        
        // Store with depth information
        self.eval_cache.store(hash_key, eval, depth);
        
        eval
    }
}
```

## Performance Considerations

### Memory Usage
- **4MB Cache**: ~330,000 entries (12 bytes per entry)
- **16MB Cache**: ~1.3 million entries
- **64MB Cache**: ~5.3 million entries
- **Overhead**: Thread synchronization and statistics

### Computational Complexity
- **Probe**: O(1) average case
- **Store**: O(1) average case
- **Aging**: O(n) where n is number of entries (infrequent)

### Cache Efficiency
- **Hit Rate Target**: 60-80% in typical positions
- **Collision Rate**: <5% with good hashing
- **Lookup Time**: <100ns average

## Testing Strategy

### Unit Tests
1. **Cache Operations**: Test probe and store
2. **Replacement Policies**: Validate each policy
3. **Hash Collisions**: Test collision detection
4. **Thread Safety**: Concurrent access tests
5. **Statistics**: Verify accurate tracking

### Performance Tests
1. **Hit Rate**: Measure cache effectiveness
2. **Lookup Speed**: Benchmark probe operations
3. **Store Speed**: Benchmark store operations
4. **Memory Usage**: Validate memory consumption
5. **Thread Scaling**: Test concurrent performance

## Configuration Options

```rust
pub struct EvalCacheConfig {
    pub size_mb: usize,
    pub policy: ReplacementPolicy,
    pub enable_prefetch: bool,
    pub enable_two_tier: bool,
    pub l1_size_mb: usize,
    pub l2_size_mb: usize,
}

impl Default for EvalCacheConfig {
    fn default() -> Self {
        Self {
            size_mb: 16,
            policy: ReplacementPolicy::DepthPreferred,
            enable_prefetch: false,
            enable_two_tier: false,
            l1_size_mb: 4,
            l2_size_mb: 16,
        }
    }
}
```

## Expected Performance Impact

### Evaluation Performance
- **50-70% Reduction**: In evaluation time
- **60-80% Hit Rate**: In typical search scenarios
- **<5% Collision Rate**: With proper hashing
- **Minimal Overhead**: <100ns per cache operation

### Memory Usage
- **Configurable**: 4MB to 64MB typical
- **Predictable**: Fixed-size allocation
- **Thread-Safe**: RwLock overhead acceptable

## Future Enhancements

1. **Adaptive Sizing**: Automatically adjust cache size based on hit rates
2. **Machine Learning**: Predict best replacement policy
3. **Distributed Caching**: Share cache across instances
4. **Persistent Cache**: Save/load cache to disk

## Conclusion

The evaluation caching design provides an efficient, thread-safe system for caching position evaluations. Key features include:

- Multiple replacement policies for flexibility
- Thread-safe concurrent access
- Comprehensive statistics tracking
- Optional multi-level caching
- Minimal performance overhead

This design enables significant performance improvements (50-70% reduction in evaluation time) while maintaining correctness and memory efficiency.

