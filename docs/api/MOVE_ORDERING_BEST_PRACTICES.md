# Move Ordering Best Practices

## Overview

This guide provides best practices for using the move ordering system effectively in your Shogi engine.

## Core Principles

### 1. Always Order Moves

**Do:**
```rust
// Order moves before searching
let ordered_moves = orderer.order_moves_with_all_heuristics(
    &moves, &board, &captured_pieces, player, depth
);

for move_ in ordered_moves {
    let score = search(move_, ...);
    // ...
}
```

**Don't:**
```rust
// Never search unordered moves at root or important nodes
for move_ in moves {
    let score = search(move_, ...);
    // ...
}
```

### 2. Update Heuristics Consistently

**Do:**
```rust
// Always update heuristics based on search results
if score >= beta {
    // Beta cutoff - this move was good
    orderer.add_killer_move(move_.clone());
    orderer.update_history(&move_, true, depth);
} else if score > alpha {
    // New best move
    orderer.update_pv_move(&board, &captured_pieces, player, depth, move_, score);
    orderer.update_history(&move_, true, depth);
} else {
    // Move didn't improve score
    orderer.update_history(&move_, false, depth);
}
```

**Don't:**
```rust
// Don't forget to update heuristics - they're useless without updates
if score >= beta {
    return score; // Missing: orderer.add_killer_move(move_);
}
```

### 3. Clear State Between Games

**Do:**
```rust
// Before starting a new game
orderer.clear_killer_moves();
orderer.clear_pv_move(0);
// Keep history table for learning across games

// Between different positions in analysis
orderer.clear_caches();
```

**Don't:**
```rust
// Don't carry over killer moves from previous games
// They're position-specific and will hurt performance
```

### 4. Use Appropriate Ordering Methods

**Do:**
```rust
// Root node: Use full ordering with all heuristics
let root_ordered = orderer.order_moves_with_all_heuristics(
    &moves, &board, &captured_pieces, player, depth
);

// Internal nodes with cached data: Use basic ordering
let internal_ordered = orderer.order_moves(&moves)?;

// Quiescence search: Order captures by SEE
let q_ordered = moves.iter()
    .filter(|m| m.is_capture)
    .cloned()
    .collect::<Vec<_>>();
```

**Don't:**
```rust
// Don't use expensive ordering in quiescence search
let q_ordered = orderer.order_moves_with_all_heuristics(
    &captures, &board, &captured_pieces, player, depth
); // Too expensive for qsearch!
```

### 5. Integrate with Transposition Table

**Do:**
```rust
// Always check transposition table first
let hash = hash_calculator.get_position_hash(&board, player, &captured_pieces);

if let Some(tt_entry) = tt.probe(hash, depth) {
    // Use TT data to improve move ordering
    let _ = orderer.integrate_with_transposition_table(
        Some(&tt_entry),
        &board,
        &captured_pieces,
        player,
        depth
    );
    
    // Try TT move first if available
    if let Some(tt_move) = tt_entry.best_move {
        // Search this move first
    }
}
```

**Don't:**
```rust
// Don't ignore transposition table data - it's valuable for ordering
if let Some(tt_entry) = tt.probe(hash, depth) {
    // Missing: integrate with move ordering
    return tt_entry.score; // Lost opportunity to learn!
}
```

## Configuration Best Practices

### 1. Start with Defaults

```rust
// Start with performance-optimized defaults
let config = MoveOrderingConfig::performance_optimized();
let mut orderer = MoveOrdering::with_config(config);

// Monitor performance
let stats = orderer.get_stats();

// Only tune if needed
if stats.cache_hit_rate < 60.0 {
    // Then adjust configuration
}
```

### 2. Tune Incrementally

```rust
// Don't change everything at once
let mut config = orderer.get_config().clone();

// Change one parameter at a time
config.cache_config.max_cache_size = 200000;
orderer.set_config(config.clone());

// Test and measure
benchmark_performance(&mut orderer);

// If better, keep change; otherwise, revert
```

### 3. Document Custom Configurations

```rust
// Document why you chose specific values
let mut config = MoveOrderingConfig::default();

// Reduced for mobile devices with limited memory
config.cache_config.max_cache_size = 10000;

// Disabled SEE because of performance requirements
config.heuristic_config.enable_see = false;

// Higher PV weight because hit rate was 45%
config.weights.pv_move_weight = 12000;
```

## Integration Best Practices

### 1. Search Engine Integration

```rust
pub struct MySearchEngine {
    move_orderer: MoveOrdering,
    // ... other fields
}

impl MySearchEngine {
    pub fn search(&mut self, board: &BitboardBoard, depth: u8) -> Option<Move> {
        // Initialize move ordering for this search
        self.move_orderer.set_current_depth(depth);
        
        // Integrate with transposition table
        if let Some(tt_entry) = self.get_tt_entry(board) {
            let _ = self.move_orderer.integrate_with_transposition_table(
                Some(&tt_entry),
                board,
                &self.captured_pieces,
                self.current_player,
                depth
            );
        }
        
        // Generate and order moves
        let moves = self.generate_moves(board);
        let ordered_moves = self.move_orderer.order_moves_with_all_heuristics(
            &moves,
            board,
            &self.captured_pieces,
            self.current_player,
            depth
        );
        
        // Search ordered moves
        self.search_moves(&ordered_moves, depth)
    }
}
```

### 2. Iterative Deepening Integration

```rust
pub fn iterative_deepening_search(&mut self, max_depth: u8) -> Option<Move> {
    let mut best_move = None;
    
    for depth in 1..=max_depth {
        // Set current depth
        self.orderer.set_current_depth(depth);
        
        // Search at this depth
        if let Some(move_) = self.search_at_depth(depth) {
            best_move = Some(move_.clone());
            
            // Update PV for next depth
            self.orderer.update_pv_move(
                &self.board,
                &self.captured_pieces,
                self.player,
                depth,
                move_,
                self.last_score
            );
        }
        
        // Age history periodically
        if depth % 2 == 0 {
            self.orderer.age_history_table();
        }
    }
    
    best_move
}
```

### 3. Negamax Integration

```rust
fn negamax(&mut self, depth: u8, alpha: i32, beta: i32) -> i32 {
    // ... check terminal conditions ...
    
    // Generate and order moves
    let moves = self.generate_legal_moves();
    let ordered_moves = self.orderer.order_moves_with_all_heuristics(
        &moves,
        &self.board,
        &self.captured_pieces,
        self.current_player,
        depth
    );
    
    let mut best_score = alpha;
    let mut best_move = None;
    
    for move_ in ordered_moves {
        self.make_move(&move_);
        let score = -self.negamax(depth - 1, -beta, -best_score);
        self.unmake_move();
        
        if score > best_score {
            best_score = score;
            best_move = Some(move_.clone());
            
            if score >= beta {
                // Beta cutoff - update heuristics
                self.orderer.add_killer_move(move_);
                self.orderer.update_history(&move_, true, depth);
                return beta;
            }
        }
    }
    
    // Update PV if we found a best move
    if let Some(mv) = best_move {
        self.orderer.update_pv_move(
            &self.board,
            &self.captured_pieces,
            self.current_player,
            depth,
            mv,
            best_score
        );
    }
    
    best_score
}
```

## Statistics and Monitoring Best Practices

### 1. Regular Monitoring

```rust
// Monitor every N searches
const MONITORING_INTERVAL: u64 = 1000;

if search_count % MONITORING_INTERVAL == 0 {
    let stats = orderer.get_stats();
    
    println!("Performance Check:");
    println!("  Ordering time: {:.2}μs", stats.avg_ordering_time_us);
    println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate);
    println!("  Memory: {:.2} MB", stats.memory_usage_bytes as f64 / 1_048_576.0);
    
    // Alert on issues
    if stats.cache_hit_rate < 50.0 {
        println!("  WARNING: Low cache hit rate!");
    }
    if stats.avg_ordering_time_us > 100.0 {
        println!("  WARNING: Slow ordering!");
    }
}
```

### 2. Export Statistics for Analysis

```rust
// At end of long analysis or tournament
let stats = orderer.get_stats();

// Export to JSON for later analysis
let json = serde_json::to_string_pretty(&stats).unwrap();
std::fs::write("move_ordering_stats.json", json).unwrap();

// Or log key metrics
eprintln!("Final stats: ordered={}, hit_rate={:.2}%, avg_time={:.2}μs",
    stats.total_moves_ordered,
    stats.cache_hit_rate,
    stats.avg_ordering_time_us
);
```

### 3. Compare Configurations

```rust
fn compare_configurations() {
    let configs = vec![
        ("Default", MoveOrderingConfig::default()),
        ("Performance", MoveOrderingConfig::performance_optimized()),
        ("Debug", MoveOrderingConfig::debug_optimized()),
    ];
    
    for (name, config) in configs {
        let mut orderer = MoveOrdering::with_config(config);
        
        // Run benchmark
        benchmark(&mut orderer);
        
        let stats = orderer.get_stats();
        println!("{} config:", name);
        println!("  Avg time: {:.2}μs", stats.avg_ordering_time_us);
        println!("  Hit rate: {:.2}%", stats.cache_hit_rate);
        println!("  Memory: {} KB", stats.memory_usage_bytes / 1024);
        println!();
    }
}
```

## Memory Management Best Practices

### 1. Monitor Memory Usage

```rust
// Check memory regularly
let current = orderer.get_current_memory_usage();
let peak = orderer.get_peak_memory_usage();

const MAX_ALLOWED_MEMORY: usize = 20 * 1024 * 1024; // 20 MB

if current > MAX_ALLOWED_MEMORY {
    println!("Memory limit exceeded, clearing caches");
    orderer.clear_caches();
}
```

### 2. Clear Caches Strategically

```rust
// Don't clear too often (wastes cache benefits)
// Don't clear too rarely (memory grows)
// Sweet spot: every 100-1000 searches

const CACHE_CLEAR_INTERVAL: u64 = 500;

if search_count % CACHE_CLEAR_INTERVAL == 0 {
    orderer.clear_caches();
}
```

### 3. Use Memory Pool Efficiently

```rust
// The system has built-in memory pooling
// Just ensure you're not holding references to pooled objects

// Good: Consume ordered moves immediately
let ordered = orderer.order_moves(&moves)?;
for move_ in ordered {
    process_move(&move_);
}
// ordered is dropped, memory returned to pool

// Bad: Holding multiple ordered vectors
let ordered1 = orderer.order_moves(&moves1)?;
let ordered2 = orderer.order_moves(&moves2)?;
let ordered3 = orderer.order_moves(&moves3)?;
// All held simultaneously - prevents pool reuse
```

## Heuristic Update Best Practices

### 1. PV Move Updates

```rust
// Update PV when you find a new best move
if score > alpha {
    alpha = score;
    best_move = Some(move_.clone());
    
    // Update PV immediately
    orderer.update_pv_move(
        &board,
        &captured_pieces,
        player,
        depth,
        move_,
        score
    );
}
```

### 2. Killer Move Updates

```rust
// Add killer moves at beta cutoffs for quiet moves
if score >= beta && !move_.is_capture {
    orderer.add_killer_move(move_.clone());
}

// Don't add captures as killer moves - they're ordered by SEE/MVV-LVA anyway
```

### 3. History Heuristic Updates

```rust
// Update for both successes and failures
if score >= beta {
    orderer.update_history(&move_, true, depth);
} else if score <= alpha {
    orderer.update_history(&move_, false, depth);
}

// Update with depth-based bonus for deeper searches
// This is automatic in the implementation
```

### 4. Transposition Table Integration

```rust
// At start of search node, integrate TT data
let hash = hash_calc.get_position_hash(&board, player, &captured_pieces);

if let Some(tt_entry) = tt.probe(hash, depth) {
    // Integrate before ordering moves
    let _ = orderer.integrate_with_transposition_table(
        Some(&tt_entry),
        &board,
        &captured_pieces,
        player,
        depth
    );
}

// Now order moves - will benefit from TT data
let ordered = orderer.order_moves_with_all_heuristics(...);
```

## Advanced Techniques

### 1. Adaptive Weight Adjustment

```rust
// Monitor heuristic effectiveness
let pv_stats = orderer.get_pv_stats();
let killer_stats = orderer.get_killer_move_stats();

// Adjust weights based on hit rates
let mut config = orderer.get_config().clone();

if pv_stats.2 > 40.0 {
    // PV very effective - increase weight
    config.weights.pv_move_weight += 1000;
}

if killer_stats.2 < 20.0 {
    // Killer not effective - decrease weight
    config.weights.killer_move_weight -= 500;
}

orderer.set_config(config);
```

### 2. Position-Type Specific Ordering

```rust
// Different positions benefit from different orderings

// Tactical positions: Prioritize captures and checks
if is_tactical_position {
    let mut config = orderer.get_config().clone();
    config.weights.capture_weight = 2000;
    config.weights.tactical_weight = 1500;
    config.weights.see_weight = 1000;
    orderer.set_config(config);
}

// Quiet positions: Prioritize positional moves
if is_quiet_position {
    let mut config = orderer.get_config().clone();
    config.weights.position_value_weight = 600;
    config.weights.center_control_weight = 400;
    orderer.set_config(config);
}
```

### 3. Multi-PV Search

```rust
// For multi-PV analysis, track multiple principal variations
let mut best_moves = Vec::new();

for pv_number in 0..num_pvs {
    // Order moves
    let ordered = orderer.order_moves_with_all_heuristics(...);
    
    // Exclude already found PV moves
    let filtered: Vec<_> = ordered.iter()
        .filter(|m| !best_moves.contains(m))
        .cloned()
        .collect();
    
    if let Some(best) = search_moves(&filtered) {
        best_moves.push(best.clone());
        // Update PV for this line
        orderer.update_pv_move(&board, &captured_pieces, player, depth, best, score);
    }
}
```

## Anti-Patterns to Avoid

### ❌ Don't: Order Moves Multiple Times

```rust
// Bad: Ordering same moves repeatedly
for depth in 1..=5 {
    let ordered = orderer.order_moves(&moves)?; // Same moves each time!
    // ...
}

// Good: Order once, or order with depth-specific information
for depth in 1..=5 {
    orderer.set_current_depth(depth);
    let ordered = orderer.order_moves_with_all_heuristics(
        &moves, &board, &captured_pieces, player, depth
    );
    // ...
}
```

### ❌ Don't: Ignore Statistics

```rust
// Bad: Never checking if ordering is effective
loop {
    let ordered = orderer.order_moves(&moves)?;
    search(ordered);
}

// Good: Monitor and adjust
if iteration_count % 1000 == 0 {
    let stats = orderer.get_stats();
    if stats.cache_hit_rate < 50.0 {
        // Adjust configuration
    }
}
```

### ❌ Don't: Use Global State

```rust
// Bad: Sharing move orderer across threads without synchronization
static mut GLOBAL_ORDERER: Option<MoveOrdering> = None;

// Good: Each search thread has its own orderer
let orderer = MoveOrdering::new();
// Use local orderer in this search
```

### ❌ Don't: Forget to Age History

```rust
// Bad: History table grows unbounded
// (History values can overflow after many updates)

// Good: Age periodically
if search_count % 100 == 0 {
    orderer.age_history_table();
}
```

## Testing Best Practices

### 1. Test Different Configurations

```rust
#[test]
fn test_various_configurations() {
    let configs = vec![
        MoveOrderingConfig::default(),
        MoveOrderingConfig::performance_optimized(),
        MoveOrderingConfig::debug_optimized(),
    ];
    
    for config in configs {
        let mut orderer = MoveOrdering::with_config(config);
        let result = orderer.order_moves(&test_moves);
        assert!(result.is_ok());
    }
}
```

### 2. Test Edge Cases

```rust
#[test]
fn test_edge_cases() {
    let mut orderer = MoveOrdering::new();
    
    // Empty moves
    assert!(orderer.order_moves(&vec![]).is_ok());
    
    // Single move
    assert!(orderer.order_moves(&vec![test_move]).is_ok());
    
    // Many moves
    let many_moves = vec![test_move; 1000];
    assert!(orderer.order_moves(&many_moves).is_ok());
}
```

### 3. Test Performance Regression

```rust
#[test]
fn test_performance_regression() {
    let mut orderer = MoveOrdering::new();
    let moves = create_test_moves(50);
    
    // Benchmark
    let start = TimeSource::now();
    for _ in 0..100 {
        let _ = orderer.order_moves(&moves);
    }
    let elapsed = start.elapsed_ms();
    
    // Ensure performance hasn't regressed
    assert!(elapsed < 100, "Performance regression: took {}ms", elapsed);
}
```

## Summary Checklist

Before deploying your move ordering integration, verify:

- [ ] Moves are ordered before searching
- [ ] Heuristics are updated based on search results
- [ ] Transposition table data is integrated
- [ ] State is cleared between games
- [ ] Memory usage is monitored
- [ ] Statistics are checked periodically
- [ ] Configuration is appropriate for your use case
- [ ] Tests cover your specific usage patterns
- [ ] Performance meets your requirements
- [ ] Error handling is robust

## Quick Reference

### Essential Do's

1. ✅ Order moves before searching
2. ✅ Update heuristics during search
3. ✅ Clear killer moves between games
4. ✅ Age history table periodically
5. ✅ Monitor cache hit rates
6. ✅ Integrate with transposition table
7. ✅ Use appropriate ordering method for each node type

### Essential Don'ts

1. ❌ Don't search unordered moves (at important nodes)
2. ❌ Don't forget to update heuristics
3. ❌ Don't carry killer moves across games
4. ❌ Don't ignore statistics
5. ❌ Don't use same configuration for all scenarios
6. ❌ Don't let memory grow unbounded
7. ❌ Don't order in quiescence search leaves

## Conclusion

Following these best practices will help you:
- Achieve optimal search performance
- Avoid common pitfalls
- Maintain stable and predictable behavior
- Make effective use of heuristics
- Keep resource usage under control

Start with the recommended defaults and tune based on your specific needs and measurements!



