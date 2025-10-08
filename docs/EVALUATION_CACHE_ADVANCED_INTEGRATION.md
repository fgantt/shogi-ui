# Evaluation Cache - Advanced Integration Guide

## Overview

This guide covers advanced integration scenarios for the evaluation cache, including integration with other engine components.

## Integration with Transposition Table

The evaluation cache and transposition table serve different purposes and can work together:

### Differences

**Transposition Table:**
- Stores search results (best move, bounds, score)
- Used in search algorithms
- Larger entries (~64-128 bytes)

**Evaluation Cache:**
- Stores static evaluations only
- Used in evaluation function
- Smaller entries (~32 bytes)

### Combined Usage

```rust
// Both can be enabled simultaneously
let mut engine = SearchEngine::new(None, 32); // 32MB transposition table
engine.enable_eval_cache(); // + evaluation cache

// Transposition table: Caches search results
// Evaluation cache: Caches static evaluations
// Both improve performance!
```

**Benefits:**
- Transposition table: Avoids re-searching positions
- Evaluation cache: Avoids re-evaluating positions
- Combined: Maximum performance improvement

## Integration with Opening Book

The cache works seamlessly with opening book:

```rust
let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();
evaluator.enable_opening_book(); // Already integrated

// Cache warms naturally as opening positions are evaluated
// Opening book provides moves, cache speeds up evaluation
```

**Benefits:**
- Opening book: Provides best moves
- Cache: Speeds up evaluation of opening positions
- Warm cache after opening phase

## Integration with Tablebase

```rust
let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();
evaluator.enable_tablebase(); // Already integrated

// Tablebase: Exact endgame solutions
// Cache: Fast evaluation for non-tablebase positions
```

**Strategy:**
- Tablebase checked first (exact)
- Cache checked next (fast)
- Full evaluation last (slow)

## Cache for Analysis Mode

### Deep Analysis Configuration

```rust
fn setup_analysis_cache() -> PositionEvaluator {
    let mut evaluator = PositionEvaluator::new();
    
    // Large cache for deep analysis
    let config = EvaluationCacheConfig::with_size_mb(128);
    evaluator.enable_eval_cache_with_config(config);
    
    // Or multi-level for even better performance
    let ml_config = MultiLevelCacheConfig {
        l1_size: 65536,   // 64K entries (~2MB)
        l2_size: 4194304, // 4M entries (~128MB)
        promotion_threshold: 3,
        ..Default::default()
    };
    evaluator.enable_multi_level_cache_with_config(ml_config);
    
    evaluator
}
```

### Position Variation Analysis

```rust
fn analyze_variations() {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_multi_level_cache();
    
    let base_board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Analyze multiple variations
    // Cache helps with repeated sub-positions
    for variation in variations {
        let mut board = base_board.clone();
        for move_ in variation {
            board.make_move(&move_);
            let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
            println!("Position score: {}", score);
        }
    }
    
    // Check cache effectiveness
    if let Some(stats) = evaluator.get_cache_statistics() {
        println!("Analysis cache stats:\n{}", stats);
    }
}
```

## Parallel Search Integration

### Thread-Safe Cache Usage

The cache is already thread-safe (RwLock-based):

```rust
use std::sync::Arc;
use std::thread;

fn parallel_search_example() {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    // Cache can be used from multiple threads
    // (In practice, each thread would have its own evaluator,
    //  but shared cache is supported)
    
    // Note: Current implementation has one evaluator per SearchEngine
    // For true parallel search, would need Arc<RwLock<EvaluationCache>>
}
```

### Current Architecture (Thread-Safe)

```
SearchEngine (Thread 1)
    └── PositionEvaluator
        └── EvaluationCache (RwLock-protected)

SearchEngine (Thread 2)  
    └── PositionEvaluator
        └── EvaluationCache (RwLock-protected)
```

Each thread has its own cache. For shared cache, would need additional wrapper.

## Cache Synchronization

### Between Engine Instances

Currently each engine has its own cache. To share:

```rust
// Would require architectural change to share cache
// Current design: One cache per PositionEvaluator
// Future enhancement: Shared cache via Arc<RwLock<EvaluationCache>>
```

### Persistence-Based Sharing

```rust
// Engine 1: Save cache
cache.save_to_file_compressed("shared_cache.gz")?;

// Engine 2: Load cache
let shared_cache = EvaluationCache::load_from_file_compressed("shared_cache.gz")?;
```

## Advanced Features

### 1. Cache Warming for Specific Positions

```rust
use shogi_vibe_usi::evaluation::eval_cache::*;

fn warm_specific_positions(positions: &[(BitboardBoard, Player, CapturedPieces)]) {
    let cache = EvaluationCache::new();
    let evaluator = PositionEvaluator::new();
    
    for (board, player, captured) in positions {
        let score = evaluator.evaluate(board, *player, captured);
        cache.store(board, *player, captured, score, 0);
    }
    
    println!("Warmed {} positions", positions.len());
}
```

### 2. Cache Analytics for Optimization

```rust
fn analyze_cache_usage() {
    let cache = EvaluationCache::new();
    
    // Use cache during search...
    
    // Get detailed analytics
    let analytics = cache.get_analytics();
    
    println!("Depth Distribution:");
    for (depth, count) in &analytics.depth_distribution {
        println!("  Depth {}: {} entries ({:.1}%)", 
                 depth, count,
                 (*count as f64 / analytics.depth_distribution.iter()
                    .map(|(_, c)| c).sum::<usize>() as f64) * 100.0);
    }
    
    // Use insights to tune configuration
    if analytics.depth_distribution.iter().any(|(d, _)| *d > 10) {
        println!("Deep searches detected - DepthPreferred policy recommended");
    }
}
```

### 3. Adaptive Cache Management

```rust
fn adaptive_cache_management() {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    let sizer = AdaptiveCacheSizer::new(
        1024 * 64,    // min: 64K entries
        1024 * 1024 * 8, // max: 8M entries
        65.0          // target hit rate
    );
    
    // In game loop
    loop {
        // Use evaluator...
        
        // Periodically check and adjust
        if let Some(cache) = evaluator.get_eval_cache_mut() {
            if let Some(new_size) = sizer.should_resize(cache) {
                println!("Adapting cache size to {} entries", new_size);
                cache.resize(new_size).ok();
            }
        }
    }
}
```

## Integration Patterns

### Pattern 1: Evaluation + Transposition Table

```rust
// Both enabled for maximum performance
let mut engine = SearchEngine::new(None, 32); // TT: 32MB
engine.enable_eval_cache(); // Eval cache: 32MB default

// Total memory: ~64MB
// Best performance: Both caches working together
```

### Pattern 2: Multi-Level Cache Only

```rust
// For memory-constrained systems
let mut engine = SearchEngine::new(None, 16); // TT: 16MB
engine.enable_multi_level_cache(); // L1: 512KB, L2: 32MB

// Smart tiering reduces memory pressure
```

### Pattern 3: Persistent Cache Across Sessions

```rust
// At startup
let cache = match EvaluationCache::load_from_file_compressed("cache.gz") {
    Ok(c) => c,
    Err(_) => EvaluationCache::new(),
};
evaluator.enable_eval_cache_with_config(cache.get_config().clone());

// At shutdown
if let Some(cache) = evaluator.get_eval_cache() {
    cache.save_to_file_compressed("cache.gz").ok();
}
```

## Advanced Use Cases

### Use Case 1: Tournament Play

```rust
// Large cache, persistence between games
let config = EvaluationCacheConfig::with_size_mb(64);
evaluator.enable_eval_cache_with_config(config);

// Keep cache between games in tournament
// Accumulates knowledge across matches
```

### Use Case 2: Position Database Evaluation

```rust
// Evaluate large position database efficiently
let mut evaluator = PositionEvaluator::new();
evaluator.enable_eval_cache();

let cache = EvaluationCache::new();
let mut evaluated_count = 0;

for position in position_database {
    let score = evaluator.evaluate(&position.board, position.player, &position.captured);
    evaluated_count += 1;
    
    if evaluated_count % 1000 == 0 {
        let stats = cache.get_statistics();
        println!("Evaluated {}, Hit rate: {:.2}%", 
                 evaluated_count, stats.hit_rate());
    }
}
```

### Use Case 3: Real-Time Game Analysis

```rust
// Update cache as game progresses
fn analyze_game_realtime(game_moves: &[Move]) {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    let mut board = BitboardBoard::new();
    let mut captured = CapturedPieces::new();
    
    for (i, move_) in game_moves.iter().enumerate() {
        // Evaluate before move
        let score_before = evaluator.evaluate(&board, Player::Black, &captured);
        
        // Make move
        board.make_move(move_);
        
        // Evaluate after move
        let score_after = evaluator.evaluate(&board, Player::White, &captured);
        
        println!("Move {}: Score change {} → {}", i+1, score_before, -score_after);
    }
    
    // Cache accumulates evaluations throughout game
}
```

## Integration Status

### ✅ Currently Integrated:
- Evaluation engine (automatic)
- Search algorithm (automatic)
- Opening book (compatible)
- Tablebase (compatible)
- WASM targets (optimized)

### 🔄 Partially Integrated:
- Parallel search (thread-safe, but separate caches)
- Distributed systems (would need network layer)

### 📋 Future Enhancements:
- Shared cache across threads (Arc<RwLock<Cache>>)
- Network-based distributed cache
- GPU acceleration (if applicable)

## See Also

- `EVALUATION_CACHE_API.md` - API documentation
- `EVALUATION_CACHE_WASM.md` - WASM-specific guide
- `EVALUATION_CACHE_BEST_PRACTICES.md` - Best practices
