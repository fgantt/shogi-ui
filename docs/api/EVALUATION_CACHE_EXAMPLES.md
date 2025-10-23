# Evaluation Cache - Usage Examples

## Basic Examples

### Example 1: Enable and Use Cache

```rust
use shogi_vibe_usi::evaluation::PositionEvaluator;
use shogi_vibe_usi::bitboards::BitboardBoard;
use shogi_vibe_usi::types::*;

fn main() {
    // Create evaluator
    let mut evaluator = PositionEvaluator::new();
    
    // Enable cache with default settings
    evaluator.enable_eval_cache();
    
    // Create position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Evaluate (cache is automatic)
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    println!("Evaluation: {}", score);
    
    // Second evaluation will hit cache
    let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    assert_eq!(score, score2); // Same result, but faster!
}
```

### Example 2: Custom Cache Configuration

```rust
use shogi_vibe_usi::evaluation::eval_cache::*;

fn setup_custom_cache() {
    let mut evaluator = PositionEvaluator::new();
    
    // Create custom configuration (16MB, depth-preferred)
    let config = EvaluationCacheConfig {
        size: 524288, // 512K entries (~16MB)
        replacement_policy: ReplacementPolicy::DepthPreferred,
        enable_statistics: true,
        enable_verification: true,
    };
    
    evaluator.enable_eval_cache_with_config(config);
    
    println!("Cache enabled with {} entries", config.size);
}
```

### Example 3: Multi-Level Cache

```rust
fn setup_multi_level_cache() {
    let mut evaluator = PositionEvaluator::new();
    
    // Enable multi-level cache (L1 + L2)
    evaluator.enable_multi_level_cache();
    
    // Or with custom configuration
    let config = MultiLevelCacheConfig {
        l1_size: 8192,   // 8K entries (~256KB)
        l2_size: 262144, // 256K entries (~8MB)
        l1_policy: ReplacementPolicy::AlwaysReplace,
        l2_policy: ReplacementPolicy::DepthPreferred,
        enable_statistics: true,
        enable_verification: true,
        promotion_threshold: 2, // Promote after 2 L2 hits
    };
    
    evaluator.enable_multi_level_cache_with_config(config);
}
```

## Search Engine Examples

### Example 4: Cache in Search

```rust
use shogi_vibe_usi::search::SearchEngine;

fn search_with_cache() {
    let mut engine = SearchEngine::new(None, 16);
    
    // Enable cache
    engine.enable_eval_cache();
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Search automatically uses cache
    let result = engine.search_at_depth(
        &board,
        &captured_pieces,
        Player::Black,
        6,          // depth
        10000,      // time limit ms
        -10000,     // alpha
        10000       // beta
    );
    
    if let Some((best_move, score)) = result {
        println!("Best move: {} (score: {})", best_move.to_usi_string(), score);
    }
    
    // Check cache statistics
    if let Some(stats) = engine.get_eval_cache_statistics() {
        println!("\nCache Stats:\n{}", stats);
    }
}
```

### Example 5: Monitor Cache During Search

```rust
fn search_with_monitoring() {
    let mut engine = SearchEngine::new(None, 16);
    engine.enable_eval_cache();
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Perform search
    for depth in 1..=8 {
        let result = engine.search_at_depth(&board, &captured_pieces, Player::Black,
                                            depth, 5000, -10000, 10000);
        
        // Print cache statistics after each depth
        if let Some(stats) = engine.get_eval_cache_statistics() {
            println!("Depth {}: {}", depth, stats);
        }
    }
}
```

## Advanced Examples

### Example 6: Cache Persistence

```rust
use std::path::Path;

fn save_and_load_cache() -> Result<(), String> {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    // Use cache during game...
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    for _ in 0..1000 {
        let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }
    
    // Save cache at end of session
    if let Some(cache) = evaluator.get_eval_cache() {
        cache.save_to_file_compressed("eval_cache.gz")?;
        println!("Cache saved");
    }
    
    // Load cache at start of next session
    let loaded_cache = EvaluationCache::load_from_file_compressed("eval_cache.gz")?;
    let config = loaded_cache.get_config().clone();
    
    let mut new_evaluator = PositionEvaluator::new();
    new_evaluator.enable_eval_cache_with_config(config);
    
    println!("Cache loaded");
    Ok(())
}
```

### Example 7: Memory Management

```rust
fn manage_cache_memory() {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    // Periodically check memory
    if let Some(cache) = evaluator.get_eval_cache_mut() {
        let usage = cache.get_memory_usage();
        
        println!("Cache utilization: {:.2}%", usage.entry_utilization());
        
        // Check for pressure
        if cache.is_under_memory_pressure() {
            println!("Memory pressure detected, compacting...");
            cache.compact();
        }
        
        // Get size suggestion
        let suggested = cache.suggest_cache_size();
        if suggested != cache.get_config().size {
            println!("Suggested size: {} entries", suggested);
        }
    }
}
```

### Example 8: Adaptive Cache Sizing

```rust
fn adaptive_cache_example() {
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    let sizer = AdaptiveCacheSizer::new(
        1024,         // min: 1K entries
        16*1024*1024, // max: 16M entries
        60.0          // target: 60% hit rate
    );
    
    // In game loop or periodically
    if let Some(cache) = evaluator.get_eval_cache_mut() {
        if let Some(new_size) = sizer.should_resize(&cache) {
            println!("Resizing cache from {} to {} entries",
                     cache.get_config().size, new_size);
            cache.resize(new_size).expect("Resize failed");
        }
    }
}
```

### Example 9: Cache Warming

```rust
fn warm_cache_example() {
    use shogi_vibe_usi::evaluation::eval_cache::*;
    
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache();
    
    if let Some(cache) = evaluator.get_eval_cache() {
        // Create warmer with opening strategy
        let warmer = CacheWarmer::new(WarmingStrategy::Opening);
        
        // Warm the cache
        warmer.warm_cache(&cache, &evaluator);
        
        println!("Warmed {} positions", warmer.get_warmed_count());
    }
}
```

### Example 10: Cache Analytics

```rust
fn analyze_cache_usage() {
    let cache = EvaluationCache::new();
    
    // Use cache...
    
    // Get analytics
    let analytics = cache.get_analytics();
    
    println!("Depth distribution:");
    for (depth, count) in &analytics.depth_distribution {
        println!("  Depth {}: {} entries", depth, count);
    }
    
    println!("\nAge distribution:");
    for (age, count) in &analytics.age_distribution {
        println!("  Age {}: {} entries", age, count);
    }
    
    // Export as JSON
    let json = cache.export_analytics_json().expect("Export failed");
    std::fs::write("cache_analytics.json", json).expect("Write failed");
}
```

## Integration Examples

### Example 11: Full Game with Cache

```rust
fn play_game_with_cache() {
    let mut engine = SearchEngine::new(None, 32);
    engine.enable_multi_level_cache();
    
    let mut board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    let mut player = Player::Black;
    
    // Game loop
    for move_num in 1..=100 {
        // Search for best move
        let result = engine.search_at_depth(
            &board,
            &captured_pieces,
            player,
            6,
            5000,
            -10000,
            10000
        );
        
        if let Some((best_move, score)) = result {
            println!("Move {}: {} (score: {})", move_num, best_move.to_usi_string(), score);
            
            // Make move
            if let Some(captured) = board.make_move(&best_move) {
                captured_pieces.add_piece(captured.piece_type, player);
            }
            
            player = player.opposite();
        } else {
            break; // No legal moves
        }
        
        // Print cache stats every 10 moves
        if move_num % 10 == 0 {
            if let Some(stats) = engine.get_eval_cache_statistics() {
                println!("\n{}\n", stats);
            }
        }
    }
    
    // Save cache for next game
    if let Some(cache) = engine.get_evaluator().get_eval_cache() {
        cache.save_to_file_compressed("game_cache.gz").expect("Save failed");
    }
}
```

### Example 12: Benchmarking Cache

```rust
fn benchmark_cache_impact() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Without cache
    let mut evaluator_no_cache = PositionEvaluator::new();
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = evaluator_no_cache.evaluate(&board, Player::Black, &captured_pieces);
    }
    let time_no_cache = start.elapsed();
    
    // With cache
    let mut evaluator_with_cache = PositionEvaluator::new();
    evaluator_with_cache.enable_eval_cache();
    
    // Warm cache
    let _ = evaluator_with_cache.evaluate(&board, Player::Black, &captured_pieces);
    
    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = evaluator_with_cache.evaluate(&board, Player::Black, &captured_pieces);
    }
    let time_with_cache = start.elapsed();
    
    println!("Without cache: {:?}", time_no_cache);
    println!("With cache: {:?}", time_with_cache);
    println!("Speedup: {:.2}x", time_no_cache.as_nanos() as f64 / time_with_cache.as_nanos() as f64);
}
```

### Example 13: Configuration from File

```rust
fn load_config_from_file() -> Result<(), String> {
    // Save configuration
    let config = EvaluationCacheConfig {
        size: 262144,
        replacement_policy: ReplacementPolicy::DepthPreferred,
        enable_statistics: true,
        enable_verification: false,
    };
    config.save_to_file("cache_config.json")?;
    
    // Load configuration
    let loaded_config = EvaluationCacheConfig::load_from_file("cache_config.json")?;
    
    let mut evaluator = PositionEvaluator::new();
    evaluator.enable_eval_cache_with_config(loaded_config);
    
    Ok(())
}
```

## Troubleshooting Examples

### Example 14: Diagnose Low Hit Rate

```rust
fn diagnose_low_hit_rate() {
    let cache = EvaluationCache::new();
    
    // Use cache...
    
    let stats = cache.get_statistics();
    let hit_rate = stats.hit_rate();
    
    if hit_rate < 40.0 {
        println!("Low hit rate: {:.2}%", hit_rate);
        
        // Get recommendations
        let recommendations = cache.get_performance_recommendations();
        for rec in recommendations {
            println!("  - {}", rec);
        }
        
        // Try solutions
        if stats.probes > 1000 {
            println!("Consider: Increase cache size or use multi-level cache");
        }
    }
}
```

### Example 15: Export Statistics

```rust
fn export_cache_statistics() -> Result<(), String> {
    let cache = EvaluationCache::new();
    
    // Use cache...
    
    let stats = cache.get_statistics();
    
    // Export as JSON
    let json = stats.export_json()?;
    std::fs::write("cache_stats.json", json)?;
    
    // Export as CSV
    let csv = stats.export_csv();
    std::fs::write("cache_stats.csv", csv)?;
    
    // Get human-readable summary
    println!("{}", stats.summary());
    
    Ok(())
}
```

## See Also

- `EVALUATION_CACHE_API.md` - Full API documentation
- `EVALUATION_CACHE_TUNING_GUIDE.md` - Performance tuning
- `EVALUATION_CACHE_TROUBLESHOOTING.md` - Common issues
