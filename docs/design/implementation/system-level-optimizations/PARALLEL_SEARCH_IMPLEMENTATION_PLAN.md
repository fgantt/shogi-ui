# Parallel Search Implementation Plan

## Overview

This document outlines the implementation plan for parallel search optimization in the Shogi engine. Parallel search can provide linear speedup with the number of CPU cores available, potentially achieving 4-8x performance improvement on modern multi-core systems.

## Current State

- **Implementation Status**: Single-threaded search
- **Performance Bottleneck**: Search algorithm cannot utilize multiple CPU cores
- **Impact**: Search depth limited by single-core performance
- **Missed Opportunity**: Modern CPUs have 4-16 cores that remain unused

## Objectives

1. Implement parallel search algorithms to utilize multiple CPU cores
2. Achieve near-linear speedup with number of available cores
3. Maintain search correctness and stability
4. Minimize thread synchronization overhead
5. Provide configurable thread count for different hardware

## Technical Approach

### Parallel Search Algorithms

#### 1. Principal Variation Splitting (PVS)
- Split search at root level among multiple threads
- Each thread searches a subset of moves
- Synchronize results to find best move
- Best for root-level parallelization

#### 2. Young Brothers Wait Concept (YBWC)
- Parallel search with work-stealing
- Threads wait for first ("oldest") move to complete
- Parallel search of remaining ("younger") moves
- Good load balancing with synchronization

#### 3. Lazy SMP (Simplified Parallel Search)
- Multiple threads with shared transposition table
- Each thread performs independent search
- Threads share information through TT
- Simplest implementation with good scalability

### Recommended Implementation: Lazy SMP + PVS Hybrid

```rust
use std::sync::{Arc, Mutex, RwLock};
use rayon::prelude::*;

pub struct ParallelSearchEngine {
    // Shared transposition table (thread-safe)
    transposition_table: Arc<RwLock<TranspositionTable>>,
    
    // Thread-local search context
    thread_pool: rayon::ThreadPool,
    
    // Search configuration
    max_threads: usize,
    
    // Search statistics per thread
    thread_stats: Arc<Mutex<Vec<SearchStats>>>,
}

impl ParallelSearchEngine {
    pub fn new(max_threads: usize) -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(max_threads)
            .build()
            .expect("Failed to create thread pool");
            
        Self {
            transposition_table: Arc::new(RwLock::new(TranspositionTable::new())),
            thread_pool,
            max_threads,
            thread_stats: Arc::new(Mutex::new(vec![SearchStats::default(); max_threads])),
        }
    }
    
    pub fn parallel_search(
        &mut self,
        board: &BitboardBoard,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> SearchResult {
        let moves = self.generate_and_order_moves(board);
        
        if moves.is_empty() {
            return SearchResult::new(None, self.evaluate_terminal(board));
        }
        
        // Search first move sequentially (PV move)
        let first_move = moves[0];
        let mut best_score = self.search_move(board, first_move, depth - 1, -beta, -alpha);
        let mut best_move = first_move;
        
        // Parallel search remaining moves
        let remaining_moves = &moves[1..];
        
        // Use thread pool for parallel search
        let results: Vec<(Move, i32)> = self.thread_pool.install(|| {
            remaining_moves
                .par_iter()
                .map(|&mv| {
                    let score = self.search_move_parallel(board, mv, depth - 1, -beta, -alpha);
                    (mv, score)
                })
                .collect()
        });
        
        // Find best move from parallel results
        for (mv, score) in results {
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }
        
        SearchResult::new(Some(best_move), best_score)
    }
    
    fn search_move_parallel(
        &self,
        board: &BitboardBoard,
        mv: Move,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        // Clone board for thread-local search
        let mut board_copy = board.clone();
        board_copy.make_move(mv);
        
        // Probe transposition table (read lock)
        if let Some(entry) = self.probe_tt(board_copy.hash()) {
            if entry.depth >= depth {
                return entry.score;
            }
        }
        
        // Perform search
        let score = -self.negamax_parallel(&board_copy, depth, -beta, -alpha);
        
        // Store in transposition table (write lock)
        self.store_tt(board_copy.hash(), depth, score, mv);
        
        score
    }
    
    fn negamax_parallel(
        &self,
        board: &BitboardBoard,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        // Check for terminal conditions
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }
        
        // Probe TT
        if let Some(entry) = self.probe_tt(board.hash()) {
            if entry.depth >= depth {
                return entry.score;
            }
        }
        
        // Generate and search moves
        let moves = self.generate_and_order_moves(board);
        let mut best_score = i32::MIN;
        
        for mv in moves {
            let score = -self.search_move_parallel(board, mv, depth - 1, -beta, -alpha);
            best_score = best_score.max(score);
            
            if score >= beta {
                return beta; // Beta cutoff
            }
        }
        
        best_score
    }
    
    fn probe_tt(&self, hash: u64) -> Option<TranspositionEntry> {
        let tt = self.transposition_table.read().unwrap();
        tt.probe(hash)
    }
    
    fn store_tt(&self, hash: u64, depth: u8, score: i32, best_move: Move) {
        let mut tt = self.transposition_table.write().unwrap();
        tt.store(hash, depth, score, best_move);
    }
}
```

## Thread Safety Considerations

### Shared Data Structures

1. **Transposition Table**
   - Use `RwLock` for concurrent read/write access
   - Multiple readers, single writer pattern
   - Lock-free alternatives: atomic operations for simple updates

2. **Search Statistics**
   - Thread-local statistics to avoid contention
   - Aggregate at end of search
   - Use `Mutex` for final aggregation

3. **Board State**
   - Clone board for each thread
   - No shared mutable state during search
   - Copy-on-write for memory efficiency

### Synchronization Strategies

```rust
// Lock-free transposition table entry update
use std::sync::atomic::{AtomicU64, Ordering};

pub struct LockFreeTTEntry {
    data: AtomicU64,  // Packed: hash(32) + depth(8) + score(16) + flags(8)
}

impl LockFreeTTEntry {
    pub fn store(&self, hash: u32, depth: u8, score: i16, flags: u8) {
        let packed = ((hash as u64) << 32)
            | ((depth as u64) << 24)
            | ((score as u64 & 0xFFFF) << 8)
            | (flags as u64);
        
        self.data.store(packed, Ordering::Release);
    }
    
    pub fn load(&self) -> Option<(u32, u8, i16, u8)> {
        let packed = self.data.load(Ordering::Acquire);
        
        let hash = (packed >> 32) as u32;
        let depth = ((packed >> 24) & 0xFF) as u8;
        let score = ((packed >> 8) & 0xFFFF) as i16;
        let flags = (packed & 0xFF) as u8;
        
        Some((hash, depth, score, flags))
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Add `rayon` dependency to `Cargo.toml`
- [ ] Implement thread pool configuration
- [ ] Add thread-safe transposition table with `RwLock`
- [ ] Create parallel search scaffolding

### Phase 2: Basic Parallel Search (Week 2)
- [ ] Implement root-level move parallelization
- [ ] Add board cloning for thread-local search
- [ ] Implement result aggregation
- [ ] Basic testing with 2-4 threads

### Phase 3: Lazy SMP Integration (Week 3)
- [ ] Implement independent thread search
- [ ] Add TT sharing between threads
- [ ] Implement thread-local search statistics
- [ ] Test scalability with 4-8 threads

### Phase 4: Optimizations (Week 4)
- [ ] Implement lock-free TT operations where possible
- [ ] Add work-stealing for better load balancing
- [ ] Optimize thread synchronization overhead
- [ ] Benchmark parallel speedup

### Phase 5: Advanced Features (Week 5)
- [ ] Implement split point management
- [ ] Add dynamic thread allocation
- [ ] Implement YBWC work-stealing
- [ ] Add thread affinity optimization

### Phase 6: Integration & Testing (Week 6)
- [ ] Integration with existing search engine
- [ ] Performance benchmarking across core counts
- [ ] Correctness validation
- [ ] Thread safety testing with ThreadSanitizer

## Performance Targets

### Speedup Goals (vs Single-threaded)

| Cores | Expected Speedup | Efficiency |
|-------|-----------------|------------|
| 2     | 1.8x           | 90%        |
| 4     | 3.2x           | 80%        |
| 8     | 5.6x           | 70%        |
| 16    | 9.6x           | 60%        |

### Benchmark Metrics

1. **Nodes per Second (NPS)**
   - Measure total nodes searched per second
   - Compare single vs multi-threaded NPS
   - Target: Linear scaling up to 8 cores

2. **Search Depth**
   - Measure achievable depth in fixed time
   - Compare single vs multi-threaded depth
   - Target: +2 ply with 4 cores, +3 ply with 8 cores

3. **Move Quality**
   - Ensure parallel search finds same best moves
   - Compare tactical puzzle solving rates
   - Target: 100% correctness

4. **Overhead**
   - Measure synchronization overhead
   - Track lock contention
   - Target: <10% overhead with 8 threads

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parallel_search_correctness() {
        let board = BitboardBoard::from_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
        
        // Single-threaded search
        let mut single = SearchEngine::new();
        let single_result = single.search(&board, 5);
        
        // Multi-threaded search
        let mut parallel = ParallelSearchEngine::new(4);
        let parallel_result = parallel.parallel_search(&board, 5, i32::MIN, i32::MAX);
        
        // Should find same best move
        assert_eq!(single_result.best_move, parallel_result.best_move);
    }
    
    #[test]
    fn test_parallel_speedup() {
        let board = BitboardBoard::from_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
        
        let start = std::time::Instant::now();
        let mut single = SearchEngine::new();
        single.search(&board, 6);
        let single_time = start.elapsed();
        
        let start = std::time::Instant::now();
        let mut parallel = ParallelSearchEngine::new(4);
        parallel.parallel_search(&board, 6, i32::MIN, i32::MAX);
        let parallel_time = start.elapsed();
        
        let speedup = single_time.as_secs_f64() / parallel_time.as_secs_f64();
        
        // Should achieve at least 2.5x speedup with 4 threads
        assert!(speedup >= 2.5, "Speedup: {:.2}x", speedup);
    }
    
    #[test]
    fn test_thread_safety() {
        let board = BitboardBoard::from_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
        let engine = Arc::new(Mutex::new(ParallelSearchEngine::new(8)));
        
        // Run multiple searches concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let engine = Arc::clone(&engine);
                let board = board.clone();
                std::thread::spawn(move || {
                    let mut eng = engine.lock().unwrap();
                    eng.parallel_search(&board, 5, i32::MIN, i32::MAX)
                })
            })
            .collect();
        
        // All threads should complete without panic
        for handle in handles {
            assert!(handle.join().is_ok());
        }
    }
}
```

### Integration Tests

1. **Tactical Puzzle Suite**
   - Solve 100+ tactical puzzles
   - Compare single vs multi-threaded results
   - Verify move quality maintained

2. **Endgame Test Suite**
   - Test endgame positions
   - Verify mate finding with parallel search
   - Check for search instabilities

3. **Stress Testing**
   - Long-running searches (1-5 minutes)
   - Memory leak detection
   - Thread starvation checks

## Dependencies

### Crate Dependencies

```toml
[dependencies]
rayon = "1.8"  # Parallel iterators and thread pool
crossbeam = "0.8"  # Lock-free data structures
parking_lot = "0.12"  # Faster RwLock implementation

[dev-dependencies]
criterion = "0.5"  # Performance benchmarking
```

### Internal Dependencies

- Transposition table implementation
- Move generation system
- Evaluation function (must be thread-safe)
- Position hashing (Zobrist)

## Benchmarking

### Performance Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_parallel_search(c: &mut Criterion) {
    let board = BitboardBoard::from_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
    
    let mut group = c.benchmark_group("parallel_search");
    
    for threads in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                let mut engine = ParallelSearchEngine::new(threads);
                b.iter(|| {
                    engine.parallel_search(
                        black_box(&board),
                        black_box(6),
                        i32::MIN,
                        i32::MAX,
                    )
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_parallel_search);
criterion_main!(benches);
```

## Configuration

### Runtime Configuration

```rust
pub struct ParallelSearchConfig {
    /// Number of threads to use (0 = auto-detect)
    pub num_threads: usize,
    
    /// Minimum depth for parallel search activation
    pub min_depth_parallel: u8,
    
    /// Split point depth threshold
    pub split_depth: u8,
    
    /// Thread affinity (pin threads to cores)
    pub thread_affinity: bool,
    
    /// TT lock granularity
    pub tt_lock_granularity: usize,
}

impl Default for ParallelSearchConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            min_depth_parallel: 4,
            split_depth: 3,
            thread_affinity: true,
            tt_lock_granularity: 8192,
        }
    }
}
```

## Risk Mitigation

### Known Risks

1. **Race Conditions**
   - Risk: Data races in shared structures
   - Mitigation: Thorough thread safety review, use ThreadSanitizer
   - Testing: Concurrent stress tests

2. **Performance Overhead**
   - Risk: Lock contention reduces speedup
   - Mitigation: Lock-free algorithms, fine-grained locking
   - Testing: Profile with perf, measure lock contention

3. **Search Instability**
   - Risk: Non-deterministic search behavior
   - Mitigation: Careful move ordering, deterministic TT
   - Testing: Repeated search verification

4. **Memory Usage**
   - Risk: Per-thread data increases memory
   - Mitigation: Shared read-only data, efficient cloning
   - Testing: Memory profiling with valgrind

## Success Criteria

- [ ] Achieve >3x speedup with 4 cores
- [ ] Achieve >5x speedup with 8 cores
- [ ] Maintain 100% move correctness in tactical tests
- [ ] Zero thread safety issues (ThreadSanitizer clean)
- [ ] <10% synchronization overhead
- [ ] Stable search behavior (deterministic results)
- [ ] No memory leaks in long-running searches

## Future Enhancements

1. **NUMA Optimization**
   - Thread and memory affinity for NUMA systems
   - Per-NUMA-node transposition tables

2. **GPU Acceleration**
   - Offload evaluation to GPU
   - Parallel move generation on GPU

3. **Distributed Search**
   - Network-based parallel search
   - Multi-machine search coordination

4. **Adaptive Threading**
   - Dynamic thread count based on position
   - Resource-aware scheduling

## References

- [Lazy SMP in Stockfish](https://www.chessprogramming.org/Lazy_SMP)
- [YBWC Algorithm](https://www.chessprogramming.org/Young_Brothers_Wait_Concept)
- [Parallel Alpha-Beta](https://www.chessprogramming.org/Parallel_Search)
- [Rayon Documentation](https://docs.rs/rayon/latest/rayon/)

