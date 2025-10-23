# Move Ordering Troubleshooting Guide

## Common Issues and Solutions

### Issue 1: "time not implemented on this platform" Error

**Error Message:**
```
panicked at library/std/src/sys/pal/wasm/../unsupported/time.rs:13:9:
time not implemented on this platform
```

**Cause:** Code is using `std::time::Instant` which is not supported in WASM environments.

**Solution:** This has been fixed in the codebase. The move ordering system now uses `TimeSource` from `crate::time_utils`, which is WASM-compatible.

**Verification:**
```rust
// Correct usage (already implemented)
use crate::time_utils::TimeSource;

let start = TimeSource::now();
// ... operations ...
let elapsed_ms = start.elapsed_ms();
```

### Issue 2: Index Out of Bounds Error

**Error Message:**
```
index out of bounds: the len is 9 but the index is 76
```

**Cause:** Incorrect indexing using `Position::to_u8()` which returns 0-80, on an array expecting 0-8.

**Solution:** Use `Position::row` and `Position::col` directly for indexing:

```rust
// Incorrect (causes error)
let idx = position.to_u8() as usize; // Returns 0-80
history_table[idx][...] // Error if idx >= 9

// Correct
let row_idx = position.row as usize; // Returns 0-8
let col_idx = position.col as usize; // Returns 0-8
history_table[row_idx][col_idx] // Always valid
```

**Fixed in:** `update_history_from_tt()` and `update_history_from_cutoff()` methods.

### Issue 3: Method Not Found Errors

**Error Message:**
```
error[E0599]: no method named `integrate_with_transposition_table` found
```

**Cause:** Methods defined outside the `impl MoveOrdering` block.

**Solution:** Ensure all methods are inside the implementation block. Check that:

```rust
impl MoveOrdering {
    // All methods must be here
    pub fn integrate_with_transposition_table(...) { ... }
    // ...
} // <-- Methods after this line won't be found
```

### Issue 4: Compilation Warnings for Unused Fields

**Warning Message:**
```
warning: fields `simple_history_table` and `pv_moves` are never read
```

**Cause:** Fields added but not yet fully utilized.

**Solution:** These warnings are expected during development. They can be suppressed with:

```rust
#[allow(dead_code)]
simple_history_table: [[i32; 9]; 9],
```

Or the fields will be used when the corresponding features are fully integrated.

### Issue 5: Poor Search Performance

**Symptoms:**
- Search is slower than expected
- Many nodes searched
- Poor move ordering

**Diagnosis:**

```rust
let stats = orderer.get_stats();

println!("Diagnostics:");
println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate);
println!("  PV hit rate: {:.2}%", stats.pv_move_hit_rate);
println!("  Killer hit rate: {:.2}%", stats.killer_move_hit_rate);
println!("  History hit rate: {:.2}%", stats.history_hit_rate);

// All hit rates should ideally be > 30%
```

**Solutions:**

1. **Low cache hit rate (< 30%)**:
   ```rust
   let mut config = orderer.get_config().clone();
   config.cache_config.max_cache_size = 200000; // Increase
   orderer.set_config(config);
   ```

2. **Low PV hit rate (< 20%)**:
   ```rust
   // Ensure PV moves are being updated
   orderer.update_pv_move(&board, &captured_pieces, player, depth, best_move, score);
   
   // Increase PV weight
   let mut config = orderer.get_config().clone();
   config.weights.pv_move_weight = 12000;
   orderer.set_config(config);
   ```

3. **Low killer hit rate (< 15%)**:
   ```rust
   // Ensure killer moves are being added at beta cutoffs
   if score >= beta {
       orderer.add_killer_move(move_.clone());
   }
   
   // Increase max killer moves
   let mut config = orderer.get_config().clone();
   config.heuristic_config.max_killer_moves = 3;
   orderer.set_config(config);
   ```

4. **Low history hit rate (< 20%)**:
   ```rust
   // Ensure history is being updated
   orderer.update_history(&move_, success, depth);
   
   // Reduce aging frequency
   let mut config = orderer.get_config().clone();
   config.heuristic_config.history_aging_frequency = 200;
   orderer.set_config(config);
   ```

### Issue 6: High Memory Usage

**Symptoms:**
- Memory usage > 50 MB
- Out of memory errors
- Performance degradation

**Diagnosis:**

```rust
let memory = orderer.get_current_memory_usage();
let peak = orderer.get_peak_memory_usage();

println!("Memory Analysis:");
println!("  Current: {:.2} MB", memory as f64 / 1_048_576.0);
println!("  Peak: {:.2} MB", peak as f64 / 1_048_576.0);

let stats = orderer.get_stats();
println!("  Cache entries: {}", stats.cache_hits + stats.cache_misses);
println!("  SEE cache size: {}", stats.see_calculations);
```

**Solutions:**

1. **Reduce cache sizes**:
   ```rust
   let mut config = orderer.get_config().clone();
   config.cache_config.max_cache_size = 50000;
   config.cache_config.max_see_cache_size = 25000;
   orderer.set_config(config);
   ```

2. **Clear caches periodically**:
   ```rust
   if search_count % 100 == 0 {
       orderer.clear_caches();
   }
   ```

3. **Disable SEE**:
   ```rust
   let mut config = orderer.get_config().clone();
   config.heuristic_config.enable_see = false;
   orderer.set_config(config);
   ```

### Issue 7: Incorrect Move Ordering

**Symptoms:**
- Bad moves tried before good moves
- Captures not prioritized
- PV moves ignored

**Diagnosis:**

```rust
let weights = orderer.get_weights();
println!("Current Weights:");
println!("  PV: {}", weights.pv_move_weight);
println!("  Killer: {}", weights.killer_move_weight);
println!("  Capture: {}", weights.capture_weight);
println!("  History: {}", weights.history_weight);

// Weights should be in descending order of importance
assert!(weights.pv_move_weight > weights.killer_move_weight);
assert!(weights.killer_move_weight > weights.capture_weight);
```

**Solutions:**

1. **Check weight configuration**:
   ```rust
   let mut config = orderer.get_config().clone();
   config.weights.pv_move_weight = 10000;
   config.weights.killer_move_weight = 5000;
   config.weights.capture_weight = 1000;
   config.weights.history_weight = 100;
   orderer.set_config(config);
   ```

2. **Verify heuristics are enabled**:
   ```rust
   let config = orderer.get_config();
   assert!(config.heuristic_config.enable_pv_moves, "PV moves should be enabled");
   assert!(config.heuristic_config.enable_killer_moves, "Killer moves should be enabled");
   ```

3. **Check that moves are being scored correctly**:
   ```rust
   for move_ in &moves {
       match orderer.score_move(move_) {
           Ok(score) => println!("Move {} score: {}", move_.to_usi_string(), score),
           Err(e) => eprintln!("Error scoring move: {}", e),
       }
   }
   ```

### Issue 8: Configuration Validation Failures

**Error Message:**
```
Configuration error: Invalid weight configuration
```

**Cause:** Invalid configuration values.

**Solution:**

```rust
// Validate configuration before using
let config = MoveOrderingConfig::default();

// Ensure weights are reasonable
assert!(config.weights.pv_move_weight > 0);
assert!(config.weights.pv_move_weight < 1_000_000);

// Ensure cache sizes are reasonable
assert!(config.cache_config.max_cache_size > 0);
assert!(config.cache_config.max_cache_size < 10_000_000);

// Use validated configuration
let orderer = MoveOrdering::with_config(config);
```

## Debugging Techniques

### Enable Debug Output

```rust
// Use debug-optimized configuration for more detailed output
let config = MoveOrderingConfig::debug_optimized();
let mut orderer = MoveOrdering::with_config(config);

// Check specific move scoring
let move_ = Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, Player::Black, false);
match orderer.score_move(&move_) {
    Ok(score) => {
        println!("Move {} scored: {}", move_.to_usi_string(), score);
        
        // Check which heuristics contributed
        if orderer.is_killer_move(&move_) {
            println!("  - Is killer move");
        }
        let history = orderer.get_history_value(&move_);
        if history > 0 {
            println!("  - History value: {}", history);
        }
    },
    Err(e) => eprintln!("Error scoring: {}", e),
}
```

### Check Heuristic Updates

```rust
// Verify heuristics are being updated
let initial_stats = orderer.get_stats();

// Perform operations
orderer.add_killer_move(move_.clone());
orderer.update_history(&move_, true, 3);

let updated_stats = orderer.get_stats();

// Check if stats changed
assert!(updated_stats.killer_moves_stored > initial_stats.killer_moves_stored, 
    "Killer moves should be stored");
assert!(updated_stats.history_updates > initial_stats.history_updates,
    "History should be updated");
```

### Trace Move Ordering Decisions

```rust
let moves = vec![
    Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, Player::Black, false),
    Move::new_move(Position::new(8, 1), Position::new(7, 1), PieceType::Lance, Player::Black, false),
];

println!("Tracing move ordering decisions:");
for move_ in &moves {
    let score = orderer.score_move(move_).unwrap_or(0);
    let is_killer = orderer.is_killer_move(move_);
    let history = orderer.get_history_value(move_);
    
    println!("\nMove: {}", move_.to_usi_string());
    println!("  Total score: {}", score);
    println!("  Is killer: {}", is_killer);
    println!("  History value: {}", history);
    println!("  Is capture: {}", move_.is_capture);
    println!("  Is promotion: {}", move_.is_promotion);
}
```

## Performance Troubleshooting

### Slow Ordering Performance

**Problem:** Ordering takes too long (> 100μs per set of moves)

**Checklist:**
- [ ] Check cache hit rate (should be > 50%)
- [ ] Check if SEE is enabled (can be expensive)
- [ ] Verify cache size is appropriate
- [ ] Check number of moves being ordered

**Solution Steps:**

1. Profile the operation:
   ```rust
   use shogi_engine::time_utils::TimeSource;
   
   let start = TimeSource::now();
   let _ = orderer.order_moves(&moves);
   let elapsed_ms = start.elapsed_ms();
   
   println!("Ordering took: {}ms", elapsed_ms);
   ```

2. Check cache effectiveness:
   ```rust
   let stats = orderer.get_stats();
   if stats.cache_hit_rate < 50.0 {
       println!("Low cache hit rate: {:.2}%", stats.cache_hit_rate);
       // Increase cache size
   }
   ```

3. Disable expensive features:
   ```rust
   let mut config = orderer.get_config().clone();
   config.heuristic_config.enable_see = false;
   orderer.set_config(config);
   ```

### Memory Leaks

**Problem:** Memory usage grows continuously

**Detection:**

```rust
let initial_memory = orderer.get_current_memory_usage();

// Perform many operations
for _ in 0..1000 {
    let _ = orderer.order_moves(&moves);
}

let final_memory = orderer.get_current_memory_usage();
let growth = final_memory as f64 / initial_memory as f64;

if growth > 2.0 {
    println!("Warning: Memory grew {:.1}x", growth);
    println!("Possible memory leak!");
}
```

**Solution:**

```rust
// Clear caches periodically to prevent unbounded growth
if iteration_count % 1000 == 0 {
    orderer.clear_caches();
}

// Monitor memory tracker
let tracker_stats = orderer.get_memory_tracker_mut();
// Check for leaks via tracker
```

## Error Messages and Solutions

### MoveOrderingError::InvalidMove

**Error:** "Invalid move: <details>"

**Causes:**
- Move with invalid from/to positions
- Move with null positions when required

**Solution:**
```rust
// Validate moves before ordering
for move_ in &moves {
    if let Some(from) = move_.from {
        assert!(from.is_valid(), "Invalid from position");
    }
    assert!(move_.to.is_valid(), "Invalid to position");
}
```

### MoveOrderingError::CacheError

**Error:** "Cache error: <details>"

**Causes:**
- Cache corruption
- Cache size limit exceeded

**Solution:**
```rust
// Clear and rebuild cache
orderer.clear_caches();

// Reduce cache size if needed
let mut config = orderer.get_config().clone();
config.cache_config.max_cache_size = 50000;
orderer.set_config(config);
```

### MoveOrderingError::SEEError

**Error:** "SEE calculation error: <details>"

**Causes:**
- Invalid board position
- Corrupted captured pieces state

**Solution:**
```rust
// Disable SEE if encountering errors
let mut config = orderer.get_config().clone();
config.heuristic_config.enable_see = false;
orderer.set_config(config);

// Or handle SEE errors gracefully
match orderer.calculate_see(&move_, &board, &captured_pieces, player) {
    Ok(see_value) => println!("SEE: {}", see_value),
    Err(_) => {
        // Fall back to basic scoring
        let score = orderer.score_move(&move_).unwrap_or(0);
        println!("Using basic score: {}", score);
    }
}
```

## Debugging Checklist

When experiencing issues, check these items in order:

### 1. Configuration
- [ ] Configuration is valid
- [ ] Weights are positive and reasonable (< 1,000,000)
- [ ] Cache sizes are positive and reasonable (< 10,000,000)
- [ ] Heuristics that are needed are enabled

### 2. Integration
- [ ] Heuristics are being updated during search
- [ ] Killer moves added at beta cutoffs
- [ ] PV moves updated when best move found
- [ ] History updated for all moves tried

### 3. Statistics
- [ ] Statistics show moves are being ordered
- [ ] Cache hit rates are reasonable (> 30%)
- [ ] Memory usage is stable (not growing unbounded)

### 4. Environment
- [ ] WASM: Using TimeSource, not std::time::Instant
- [ ] WASM: Array indices use row/col, not to_u8()
- [ ] Dependencies are compatible

## Getting Help

If issues persist:

1. **Check existing tests**: Run the comprehensive test suite
   ```bash
   cargo test --lib move_ordering
   ```

2. **Enable debug output**: Use debug-optimized configuration
   ```rust
   let config = MoveOrderingConfig::debug_optimized();
   ```

3. **Collect diagnostics**:
   ```rust
   let stats = orderer.get_stats();
   let pv_stats = orderer.get_pv_stats();
   let killer_stats = orderer.get_killer_move_stats();
   let history_stats = orderer.get_history_stats();
   let tt_stats = orderer.get_tt_integration_stats();
   
   // Export all statistics for analysis
   println!("Full Diagnostics:");
   println!("{:#?}", stats);
   ```

4. **Create minimal reproduction**:
   ```rust
   let mut orderer = MoveOrdering::new();
   let move_ = Move::new_move(
       Position::new(6, 4), 
       Position::new(5, 4), 
       PieceType::Pawn, 
       Player::Black, 
       false
   );
   
   match orderer.order_moves(&vec![move_]) {
       Ok(_) => println!("Success"),
       Err(e) => println!("Error: {}", e),
   }
   ```

## FAQ

**Q: Why is move ordering slow on first search?**

A: Caches are empty on first search. Performance improves significantly after cache warming. Use `order_moves_with_all_heuristics()` which populates caches.

**Q: Should I clear caches between searches?**

A: Clear killer moves and root PV between different game searches, but keep history table and score caches for better performance across searches of the same game.

**Q: What's the difference between `order_moves()` and `order_moves_with_all_heuristics()`?**

A: `order_moves()` uses cached data and basic scoring. `order_moves_with_all_heuristics()` uses position-specific information for more accurate ordering but requires board state.

**Q: How often should I age the history table?**

A: Every 100-200 searches is typical. Monitor `get_history_stats()` - if history hit rate drops below 20%, age less frequently.

**Q: Can I use move ordering in WASM?**

A: Yes! The system is fully WASM-compatible. Ensure you're using the latest version with TimeSource and correct array indexing.

**Q: How do I integrate with my search engine?**

A: See the `SearchEngine` integration in `src/search/search_engine.rs`. The key points are:
1. Create `MoveOrdering` instance in your engine
2. Call `initialize_advanced_move_orderer()` at search start
3. Use `order_moves_with_all_heuristics()` in search
4. Update heuristics (PV, killer, history) as you search

**Q: What if I get duplicate enum errors?**

A: Ensure `MoveOrderingError` and related types are defined only once. Check that there are no duplicate definitions in the file.

**Q: Why are my tests failing with "method not found"?**

A: Methods must be inside `impl MoveOrdering { }` block. Check that implementation block hasn't been accidentally closed early.



