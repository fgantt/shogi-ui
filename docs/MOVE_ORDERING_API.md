# Move Ordering API Documentation

## Overview

The Move Ordering system provides sophisticated move prioritization for the Shogi engine, significantly improving search efficiency through various heuristics including PV moves, killer moves, history heuristic, and Static Exchange Evaluation (SEE).

## Table of Contents

- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [API Reference](#api-reference)
- [Configuration](#configuration)
- [Integration](#integration)
- [Performance Tuning](#performance-tuning)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Basic Usage

```rust
use shogi_engine::search::move_ordering::MoveOrdering;
use shogi_engine::types::{Move, Player};
use shogi_engine::bitboards::BitboardBoard;

// Create a move orderer with default configuration
let mut orderer = MoveOrdering::new();

// Generate moves (from your move generator)
let moves = vec![/* your moves */];

// Order the moves
let ordered_moves = orderer.order_moves(&moves)?;

// Use ordered_moves in your search algorithm
for move_ in ordered_moves {
    // Search this move...
}
```

### Advanced Usage with All Heuristics

```rust
let mut orderer = MoveOrdering::new();
let board = BitboardBoard::new();
let captured_pieces = CapturedPieces::new();
let player = Player::Black;
let depth = 5;

// Order moves using all heuristics (PV, killer, history, SEE)
let ordered_moves = orderer.order_moves_with_all_heuristics(
    &moves, 
    &board, 
    &captured_pieces, 
    player, 
    depth
);

// After finding a good move, update heuristics
orderer.update_history(&best_move, true, depth);
orderer.add_killer_move(best_move.clone());
```

## Core Concepts

### 1. Principal Variation (PV) Moves

PV moves are the best moves from previous searches. They receive the highest priority in move ordering.

```rust
// Update PV move after finding a good move
orderer.update_pv_move(&board, &captured_pieces, player, depth, best_move, score);

// Retrieve PV move for current position
if let Some(pv_move) = orderer.get_pv_move(&board, &captured_pieces, player, depth) {
    // Prioritize this move
}
```

### 2. Killer Moves

Killer moves are quiet moves that caused beta cutoffs at the same depth.

```rust
// Add a killer move
orderer.add_killer_move(move_);

// Check if a move is a killer move
if orderer.is_killer_move(&move_) {
    // Prioritize this move
}

// Clear killer moves when starting a new search
orderer.clear_killer_moves();
```

### 3. History Heuristic

The history heuristic tracks moves that have been successful across different positions.

```rust
// Update history after a move succeeds or fails
orderer.update_history(&move_, success, depth);

// Get history score for a move
let history_score = orderer.get_history_value(&move_);

// Age history table periodically
orderer.age_history_table();
```

### 4. Static Exchange Evaluation (SEE)

SEE evaluates the material outcome of capture sequences.

```rust
// Calculate SEE for a capture move
let see_value = orderer.calculate_see(&move_, &board, &captured_pieces, player)?;

// Score a move using SEE
let see_score = orderer.score_see_move(&move_, &board, &captured_pieces, player)?;
```

### 5. Transposition Table Integration

Move ordering integrates with the transposition table to leverage previous search results.

```rust
// Integrate with transposition table entry
if let Some(tt_entry) = transposition_table.probe(hash, depth) {
    orderer.integrate_with_transposition_table(
        Some(&tt_entry), 
        &board, 
        &captured_pieces, 
        player, 
        depth
    )?;
}

// Get PV move from transposition table
if let Some(pv_move) = orderer.get_pv_move_from_tt(Some(&tt_entry)) {
    // Use this move first
}
```

## API Reference

### Core Methods

#### `new() -> Self`

Creates a new move orderer with default configuration.

```rust
let orderer = MoveOrdering::new();
```

#### `with_config(config: MoveOrderingConfig) -> Self`

Creates a new move orderer with custom configuration.

```rust
let config = MoveOrderingConfig::performance_optimized();
let orderer = MoveOrdering::with_config(config);
```

#### `order_moves(&mut self, moves: &[Move]) -> MoveOrderingResult<Vec<Move>>`

Orders moves using all configured heuristics.

**Parameters:**
- `moves`: Slice of moves to order

**Returns:**
- `Ok(Vec<Move>)`: Ordered moves
- `Err(MoveOrderingError)`: Error if ordering fails

#### `order_moves_with_all_heuristics(&mut self, moves: &[Move], board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move>`

Orders moves using all heuristics including position-specific data.

**Parameters:**
- `moves`: Slice of moves to order
- `board`: Current board position
- `captured_pieces`: Captured pieces state
- `player`: Current player
- `depth`: Current search depth

**Returns:**
- Ordered vector of moves

### PV Move Methods

#### `update_pv_move(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, move_: Move, score: i32)`

Updates the PV move for a specific position and depth.

#### `get_pv_move(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Option<Move>`

Retrieves the PV move for the current position.

#### `clear_pv_move(&mut self, depth: u8)`

Clears the PV move at a specific depth.

### Killer Move Methods

#### `add_killer_move(&mut self, move_: Move)`

Adds a move to the killer move table.

#### `is_killer_move(&self, move_: &Move) -> bool`

Checks if a move is a killer move at the current depth.

#### `clear_killer_moves(&mut self)`

Clears all killer moves.

### History Heuristic Methods

#### `update_history(&mut self, move_: &Move, success: bool, depth: u8)`

Updates the history table based on move success.

#### `get_history_value(&self, move_: &Move) -> u32`

Gets the history score for a move.

#### `age_history_table(&mut self)`

Reduces all history values by a factor (aging).

### SEE Methods

#### `calculate_see(&mut self, move_: &Move, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> MoveOrderingResult<i32>`

Calculates the Static Exchange Evaluation for a move.

#### `score_see_move(&mut self, move_: &Move, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> MoveOrderingResult<i32>`

Scores a move using SEE with caching.

### Statistics Methods

#### `get_stats(&self) -> &OrderingStats`

Returns comprehensive statistics about move ordering performance.

#### `get_pv_stats(&self) -> (u64, u64, f64, u64, u64)`

Returns PV move statistics: `(hits, misses, hit_rate, stored, cache_size)`.

#### `get_killer_move_stats(&self) -> (u64, u64, f64, u64)`

Returns killer move statistics: `(hits, misses, hit_rate, stored)`.

#### `get_history_stats(&self) -> (u64, u64, f64, u64, u64)`

Returns history heuristic statistics: `(hits, misses, hit_rate, updates, aging_ops)`.

#### `get_tt_integration_stats(&self) -> TTIntegrationStats`

Returns transposition table integration statistics.

### Configuration Methods

#### `set_config(&mut self, config: MoveOrderingConfig)`

Updates the configuration at runtime.

#### `get_config(&self) -> &MoveOrderingConfig`

Retrieves the current configuration.

#### `get_weights(&self) -> &OrderingWeights`

Gets the current heuristic weights.

### Memory Management Methods

#### `get_current_memory_usage(&self) -> usize`

Returns current memory usage in bytes.

#### `get_peak_memory_usage(&self) -> usize`

Returns peak memory usage in bytes.

#### `clear_caches(&mut self)`

Clears all caches to free memory.

## Configuration

### Configuration Options

```rust
use shogi_engine::search::move_ordering::MoveOrderingConfig;

let mut config = MoveOrderingConfig::default();

// Adjust heuristic weights
config.weights.pv_move_weight = 10000;      // Highest priority
config.weights.killer_move_weight = 5000;   // High priority
config.weights.history_weight = 100;        // Moderate priority
config.weights.capture_weight = 1000;       // High for tactical
config.weights.promotion_weight = 800;      // Good value
config.weights.tactical_weight = 600;       // Tactical moves
config.weights.see_weight = 400;            // SEE evaluation

// Cache configuration
config.cache_config.max_cache_size = 100000;
config.cache_config.max_see_cache_size = 50000;
config.cache_config.enable_dynamic_sizing = true;

// Heuristic configuration
config.heuristic_config.enable_pv_moves = true;
config.heuristic_config.enable_killer_moves = true;
config.heuristic_config.enable_history_heuristic = true;
config.heuristic_config.enable_see = true;
config.heuristic_config.max_killer_moves = 2;
config.heuristic_config.history_aging_frequency = 100;
```

### Preset Configurations

#### Performance Optimized

```rust
let config = MoveOrderingConfig::performance_optimized();
let orderer = MoveOrdering::with_config(config);
```

Best for competitive play with maximum search efficiency.

#### Debug Optimized

```rust
let config = MoveOrderingConfig::debug_optimized();
let orderer = MoveOrdering::with_config(config);
```

Optimized for debugging with additional statistics tracking.

#### Memory Optimized

```rust
let mut config = MoveOrderingConfig::default();
config.cache_config.max_cache_size = 10000;
config.cache_config.max_see_cache_size = 5000;
let orderer = MoveOrdering::with_config(config);
```

Minimizes memory usage for resource-constrained environments.

## Integration

### Integration with Search Engine

```rust
use shogi_engine::search::search_engine::SearchEngine;

// The SearchEngine automatically integrates move ordering
let mut engine = SearchEngine::new(None, 64); // 64 MB hash table

// Move ordering is automatically used during search
let best_move = engine.search_at_depth(
    &board, 
    &captured_pieces, 
    player, 
    depth, 
    time_limit_ms,
    alpha,
    beta
);
```

### Integration with Transposition Table

```rust
// Inside your search function
let hash = hash_calculator.get_position_hash(&board, player, &captured_pieces);

// Check transposition table
if let Some(tt_entry) = transposition_table.probe(hash, depth) {
    // Integrate TT data with move ordering
    orderer.integrate_with_transposition_table(
        Some(&tt_entry),
        &board,
        &captured_pieces,
        player,
        depth
    )?;
}

// Order moves (will use TT data)
let ordered_moves = orderer.order_moves_with_all_heuristics(
    &moves,
    &board,
    &captured_pieces,
    player,
    depth
);
```

### Updating Heuristics During Search

```rust
// After trying a move in your search
if score >= beta {
    // Beta cutoff - update killer move
    orderer.add_killer_move(move_.clone());
    orderer.update_history(&move_, true, depth);
} else if score > alpha {
    // New best move - update PV
    orderer.update_pv_move(&board, &captured_pieces, player, depth, move_, score);
    orderer.update_history(&move_, true, depth);
} else {
    // Move failed - reduce history score
    orderer.update_history(&move_, false, depth);
}
```

## Performance Tuning

### Monitoring Performance

```rust
// Get comprehensive statistics
let stats = orderer.get_stats();
println!("Total moves ordered: {}", stats.total_moves_ordered);
println!("Average ordering time: {:.2}μs", stats.avg_ordering_time_us);
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate);
println!("PV move hit rate: {:.2}%", stats.pv_move_hit_rate);
println!("Killer move hit rate: {:.2}%", stats.killer_move_hit_rate);
println!("History hit rate: {:.2}%", stats.history_hit_rate);

// Get memory usage
let memory = orderer.get_current_memory_usage();
println!("Current memory usage: {} bytes", memory);
```

### Optimizing for Different Game Phases

```rust
// Opening phase - prioritize development and center control
let mut config = MoveOrderingConfig::default();
config.weights.development_weight = 400;
config.weights.center_control_weight = 300;

// Middlegame - balanced weights
config.weights.tactical_weight = 600;
config.weights.capture_weight = 1000;

// Endgame - prioritize king safety and promotion
config.weights.king_safety_weight = 800;
config.weights.promotion_weight = 900;
```

### Tuning for Performance

1. **Adjust Cache Sizes**: Larger caches improve hit rates but use more memory
   ```rust
   config.cache_config.max_cache_size = 200000; // Increase for more caching
   ```

2. **Enable Dynamic Sizing**: Automatically adjust cache sizes
   ```rust
   config.cache_config.enable_dynamic_sizing = true;
   ```

3. **Tune Heuristic Weights**: Higher weights = higher priority
   ```rust
   config.weights.pv_move_weight = 10000;  // Always try PV move first
   ```

4. **Disable Expensive Heuristics**: For faster but less accurate ordering
   ```rust
   config.heuristic_config.enable_see = false; // SEE can be expensive
   ```

## Best Practices

### 1. Always Clear Between Searches

```rust
// Before starting a new search
orderer.clear_killer_moves();
orderer.clear_pv_move(0); // Clear root PV
```

### 2. Age History Periodically

```rust
// Every N searches or after N moves
if search_count % 100 == 0 {
    orderer.age_history_table();
}
```

### 3. Use Appropriate Ordering Method

```rust
// For root node - use all heuristics
let ordered = orderer.order_moves_with_all_heuristics(&moves, &board, &captured_pieces, player, depth);

// For leaf nodes - use faster basic ordering
let ordered = orderer.order_moves(&moves)?;

// For quiescence search - order captures only
let ordered = orderer.order_moves_with_see(&moves, &board, &captured_pieces, player)?;
```

### 4. Monitor and Adjust Weights

```rust
// Check which heuristics are performing well
let pv_stats = orderer.get_pv_stats();
let killer_stats = orderer.get_killer_move_stats();
let history_stats = orderer.get_history_stats();

// Adjust weights based on hit rates
if pv_stats.2 > 90.0 {  // 90% hit rate
    // PV moves are very effective, keep high weight
}
```

### 5. Integrate with Transposition Table

```rust
// Always integrate TT data when available
if let Some(tt_entry) = tt.probe(hash, depth) {
    let _ = orderer.integrate_with_transposition_table(
        Some(&tt_entry),
        &board,
        &captured_pieces,
        player,
        depth
    );
}
```

## Troubleshooting

### Issue: Poor Move Ordering Performance

**Symptoms:**
- Low cache hit rates (< 30%)
- High average ordering time (> 100μs per move)
- Poor search performance

**Solutions:**
1. Check if caches are too small:
   ```rust
   let stats = orderer.get_stats();
   if stats.cache_hit_rate < 30.0 {
       config.cache_config.max_cache_size *= 2;
   }
   ```

2. Verify heuristics are enabled:
   ```rust
   let config = orderer.get_config();
   assert!(config.heuristic_config.enable_pv_moves);
   assert!(config.heuristic_config.enable_killer_moves);
   ```

3. Check that heuristics are being updated:
   ```rust
   let stats = orderer.get_stats();
   println!("History updates: {}", stats.history_updates);
   println!("Killer moves stored: {}", stats.killer_moves_stored);
   ```

### Issue: High Memory Usage

**Symptoms:**
- Memory usage growing over time
- Out of memory errors

**Solutions:**
1. Clear caches periodically:
   ```rust
   orderer.clear_caches();
   ```

2. Reduce cache sizes:
   ```rust
   config.cache_config.max_cache_size = 50000;
   config.cache_config.max_see_cache_size = 25000;
   ```

3. Monitor memory usage:
   ```rust
   let memory = orderer.get_current_memory_usage();
   if memory > MAX_ALLOWED_MEMORY {
       orderer.clear_caches();
   }
   ```

### Issue: Incorrect Move Ordering

**Symptoms:**
- Bad moves ordered before good moves
- Search explores poor lines first

**Solutions:**
1. Check weight configuration:
   ```rust
   let weights = orderer.get_weights();
   assert!(weights.pv_move_weight > weights.killer_move_weight);
   assert!(weights.capture_weight > weights.quiet_weight);
   ```

2. Verify heuristics are being updated correctly:
   ```rust
   // After a beta cutoff
   orderer.add_killer_move(cutoff_move.clone());
   
   // After finding best move
   orderer.update_pv_move(&board, &captured_pieces, player, depth, best_move, score);
   ```

3. Check for configuration errors:
   ```rust
   match orderer.validate_config() {
       Ok(_) => println!("Configuration valid"),
       Err(e) => println!("Configuration error: {}", e),
   }
   ```

### Issue: WASM Runtime Errors

**Symptoms:**
- "time not implemented on this platform" errors
- Index out of bounds errors

**Solutions:**
1. The timing issue has been fixed by using `TimeSource` from `time_utils.rs`
2. For index errors, ensure position indices are valid:
   ```rust
   // Always use Position::row and Position::col for indexing
   let from_idx = from.row as usize;
   let to_idx = to.row as usize;
   ```

## Configuration Examples

### Example 1: Aggressive Tactical Play

```rust
let mut config = MoveOrderingConfig::default();
config.weights.capture_weight = 2000;      // High priority for captures
config.weights.tactical_weight = 1500;     // Prioritize tactics
config.weights.see_weight = 1000;          // Use SEE heavily
config.weights.killer_move_weight = 3000;  // Learn from cutoffs
config.heuristic_config.enable_see = true;
```

### Example 2: Positional Play

```rust
let mut config = MoveOrderingConfig::default();
config.weights.position_value_weight = 500;    // Value position
config.weights.center_control_weight = 400;    // Control center
config.weights.development_weight = 350;       // Develop pieces
config.weights.king_safety_weight = 600;       // Protect king
config.weights.quiet_weight = 200;             // Consider quiet moves
```

### Example 3: Memory Constrained

```rust
let mut config = MoveOrderingConfig::default();
config.cache_config.max_cache_size = 10000;      // Small cache
config.cache_config.max_see_cache_size = 5000;   // Small SEE cache
config.cache_config.enable_dynamic_sizing = false; // Fixed size
config.heuristic_config.enable_see = false;       // Disable SEE to save memory
```

### Example 4: Maximum Performance

```rust
let mut config = MoveOrderingConfig::performance_optimized();
config.cache_config.max_cache_size = 500000;     // Large cache
config.cache_config.max_see_cache_size = 250000; // Large SEE cache
config.cache_config.enable_dynamic_sizing = true;
config.heuristic_config.enable_pv_moves = true;
config.heuristic_config.enable_killer_moves = true;
config.heuristic_config.enable_history_heuristic = true;
config.heuristic_config.enable_see = true;
config.heuristic_config.max_killer_moves = 3;    // More killer moves
```

## Advanced Features

### Position-Specific Strategies

The move orderer can adapt its strategy based on game phase:

```rust
// The orderer automatically detects game phase
orderer.update_game_phase(move_count, material_balance, tactical_complexity);

// Different strategies are applied automatically
// - Opening: Emphasize development and center control
// - Middlegame: Balanced tactical and positional play
// - Endgame: Focus on king safety and promotion
```

### Dynamic Weight Adjustment

Automatically adjust weights based on performance:

```rust
// Enable dynamic weights
orderer.set_advanced_features_enabled(true);

// Weights will automatically adjust based on heuristic effectiveness
orderer.adjust_weights_dynamically();
```

### Error Handling and Recovery

The system includes robust error handling:

```rust
match orderer.order_moves(&moves) {
    Ok(ordered) => {
        // Use ordered moves
    },
    Err(MoveOrderingError::InvalidMove(msg)) => {
        // Handle invalid move
        eprintln!("Invalid move: {}", msg);
    },
    Err(MoveOrderingError::CacheError(msg)) => {
        // Cache error - clear and retry
        orderer.clear_caches();
        orderer.order_moves(&moves)?
    },
    Err(e) => {
        // Other errors
        eprintln!("Ordering error: {}", e);
    }
}
```

## Performance Characteristics

### Time Complexity

- Basic ordering: O(n log n) where n = number of moves
- With caching: O(1) for cache hits, O(n log n) for cache misses
- SEE calculation: O(k) where k = number of attackers/defenders

### Space Complexity

- Move score cache: O(c) where c = cache size
- Killer moves: O(d × k) where d = max depth, k = killer moves per depth
- History table: O(81 × 81) for position-based history
- PV moves: O(d) where d = max depth

### Expected Performance

- Ordering 50 moves: < 10μs (with cache hits)
- Ordering 50 moves: < 100μs (with cache misses)
- 1000 orderings: < 100ms
- Cache hit rate: 70-90% (typical)
- PV hit rate: 30-50% (typical)
- Killer hit rate: 20-40% (typical)

## Examples

See the `/examples` directory for complete working examples:

- `move_ordering_basic.rs` - Basic usage examples
- `move_ordering_advanced.rs` - Advanced integration examples
- `move_ordering_performance.rs` - Performance tuning examples

## Version History

- v0.1.0: Initial implementation with basic move ordering
- v0.2.0: Added PV moves, killer moves, and history heuristic
- v0.3.0: Added SEE and transposition table integration
- v0.4.0: Added advanced features and comprehensive testing

## License

This documentation is part of the Shogi Engine project.



