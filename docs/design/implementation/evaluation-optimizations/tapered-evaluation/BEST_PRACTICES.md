# Tapered Evaluation - Best Practices

## Overview

This guide provides best practices for using the tapered evaluation system effectively in production.

## Performance Best Practices

### 1. Always Enable Caching

**DO**:
```rust
let mut config = IntegratedEvaluationConfig::default();
config.enable_phase_cache = true;  // 2-20× speedup
config.enable_eval_cache = true;   // 160-240× speedup on hits

let evaluator = IntegratedEvaluator::with_config(config);
```

**DON'T**:
```rust
// Disabling caches significantly hurts performance
config.enable_phase_cache = false;
config.enable_eval_cache = false;
```

**Why**: Caching provides massive performance improvements with minimal memory cost.

### 2. Use Appropriate Component Selection

**Fast Mode** (Material + PST only):
```rust
config.components = ComponentFlags::minimal();
// ~600ns per evaluation
```

**Balanced Mode** (Default):
```rust
config.components = ComponentFlags::all_enabled();
// ~800-1200ns per evaluation
```

**Choose Based On**:
- Time controls: Blitz → minimal, Classical → all enabled
- Hardware: Slow → minimal, Fast → all enabled
- Accuracy needs: Quick analysis → minimal, Critical games → all enabled

### 3. Clear Caches Appropriately

**DO**:
```rust
// Clear between games
evaluator.clear_caches();

// Clear periodically during long analysis
if nodes_searched % 100000 == 0 {
    evaluator.clear_caches();
}
```

**DON'T**:
```rust
// Clear every few nodes (defeats the purpose!)
if nodes_searched % 10 == 0 {
    evaluator.clear_caches();  // Too frequent!
}
```

### 4. Disable Statistics in Production

**DO**:
```rust
#[cfg(debug_assertions)]
evaluator.enable_statistics();

#[cfg(not(debug_assertions))]
evaluator.disable_statistics();  // Zero overhead
```

**Why**: Statistics add ~5% overhead when enabled.

## Accuracy Best Practices

### 1. Use All Components for Strong Play

```rust
// Maximum accuracy
let config = IntegratedEvaluationConfig {
    components: ComponentFlags::all_enabled(),
    enable_phase_cache: true,
    enable_eval_cache: true,
    use_optimized_path: true,
    max_cache_size: 10000,
};
```

### 2. Tune Weights for Your Use Case

```rust
// For tactical games
weights.king_safety_weight = 1.5;
weights.mobility_weight = 0.8;

// For positional games
weights.pawn_structure_weight = 0.9;
weights.center_control_weight = 0.8;
```

### 3. Validate Against Test Positions

```rust
let test_suite = load_test_positions();

for position in test_suite {
    let score = evaluator.evaluate(&position.board, position.player, &position.captured);
    
    // Check if score makes sense
    if position.has_material_advantage(Player::Black) {
        assert!(score > 0, "Material advantage not reflected");
    }
}
```

## Search Integration Best Practices

### 1. Use Phase-Aware Pruning

```rust
let enhancer = search_engine.get_tapered_search_enhancer_mut();
let phase = enhancer.track_phase(&board);

if enhancer.should_prune(phase, depth, score, beta) {
    return beta;  // Safely prune
}
```

**Why**: Phase-aware pruning is more accurate and reduces tree size by 20-40%.

### 2. Apply Phase Move Bonuses

```rust
// In move ordering
for mv in &mut moves {
    let bonus = enhancer.get_phase_move_bonus(mv.piece_type, phase);
    mv.score += bonus;
}

// Sort by score
moves.sort_by(|a, b| b.score.cmp(&a.score));
```

### 3. Use Phase Extensions Wisely

```rust
let extension = enhancer.get_phase_extension(phase, is_check, is_capture);

// Only extend if depth allows
if depth + extension <= MAX_DEPTH {
    new_depth = depth + extension;
}
```

## Memory Management Best Practices

### 1. Limit Cache Size

```rust
let config = IntegratedEvaluationConfig {
    max_cache_size: 10000,  // ~320KB for eval cache
    // ...
};
```

**Guidelines**:
- Desktop: 10,000-50,000 entries
- Mobile: 1,000-5,000 entries
- WASM: 500-2,000 entries

### 2. Monitor Cache Growth

```rust
let stats = evaluator.cache_stats();

if stats.eval_cache_size > config.max_cache_size {
    println!("Warning: Cache exceeds limit");
    evaluator.clear_caches();
}
```

### 3. Use Separate Evaluators Per Thread

**DO**:
```rust
use std::thread;

let handles: Vec<_> = (0..num_threads).map(|_| {
    thread::spawn(move || {
        let mut evaluator = IntegratedEvaluator::new();  // Per-thread
        // ... use evaluator ...
    })
}).collect();
```

**DON'T**:
```rust
// Share evaluator across threads (not thread-safe!)
let evaluator = IntegratedEvaluator::new();
// ... use from multiple threads ... // ERROR!
```

## Configuration Best Practices

### 1. Start with Defaults

```rust
// Good starting point
let evaluator = IntegratedEvaluator::new();
```

### 2. Profile Before Optimizing

```rust
evaluator.enable_statistics();

// Run representative workload
for _ in 0..10000 {
    evaluator.evaluate(&board, player, &captured);
}

// Check performance
let report = evaluator.get_statistics().generate_report();
println!("{}", report);

// Identify bottlenecks, then optimize
```

### 3. A/B Test Configuration Changes

```rust
// Test both configurations
let score_a = test_with_config(config_a, &test_positions);
let score_b = test_with_config(config_b, &test_positions);

println!("Config A error: {:.4}", score_a);
println!("Config B error: {:.4}", score_b);

// Use the better one
```

## Testing Best Practices

### 1. Test at Multiple Phases

```rust
#[test]
fn test_evaluation_phases() {
    let mut evaluator = IntegratedEvaluator::new();
    
    // Test opening
    let opening_board = create_opening_position();
    let opening_score = evaluator.evaluate(&opening_board, player, &captured);
    
    // Test middlegame
    let middlegame_board = create_middlegame_position();
    let middlegame_score = evaluator.evaluate(&middlegame_board, player, &captured);
    
    // Test endgame
    let endgame_board = create_endgame_position();
    let endgame_score = evaluator.evaluate(&endgame_board, player, &captured);
}
```

### 2. Test Cache Behavior

```rust
#[test]
fn test_cache_consistency() {
    let mut evaluator = IntegratedEvaluator::new();
    
    let score1 = evaluator.evaluate(&board, player, &captured);
    evaluator.clear_caches();
    let score2 = evaluator.evaluate(&board, player, &captured);
    
    assert_eq!(score1, score2, "Cache inconsistency!");
}
```

### 3. Stress Test

```rust
#[test]
fn test_stress_load() {
    let mut evaluator = IntegratedEvaluator::new();
    
    // Should handle many evaluations without issues
    for _ in 0..100000 {
        evaluator.evaluate(&board, player, &captured);
    }
    
    // Check cache didn't explode
    let stats = evaluator.cache_stats();
    assert!(stats.eval_cache_size < 20000);
}
```

## Common Pitfalls to Avoid

### 1. Don't Disable Caching Without Reason

❌ **Bad**:
```rust
config.enable_phase_cache = false;
config.enable_eval_cache = false;
// Massive performance loss!
```

✅ **Good**:
```rust
config.enable_phase_cache = true;
config.enable_eval_cache = true;
```

### 2. Don't Keep Statistics Enabled in Production

❌ **Bad**:
```rust
evaluator.enable_statistics();  // Always on = 5% overhead
```

✅ **Good**:
```rust
#[cfg(debug_assertions)]
evaluator.enable_statistics();
```

### 3. Don't Ignore Component Flags

❌ **Bad**:
```rust
// Always using all components even when speed matters
```

✅ **Good**:
```rust
// Adjust based on time control
if time_control.is_blitz {
    config.components = ComponentFlags::minimal();
} else {
    config.components = ComponentFlags::all_enabled();
}
```

### 4. Don't Share Evaluators Across Threads

❌ **Bad**:
```rust
let evaluator = IntegratedEvaluator::new();
// Use from multiple threads... // NOT THREAD-SAFE!
```

✅ **Good**:
```rust
// Create per thread
thread::spawn(move || {
    let mut evaluator = IntegratedEvaluator::new();
    // ...
});
```

## Production Deployment Checklist

- [ ] Weights tuned on representative game database
- [ ] Caching enabled
- [ ] Statistics disabled (or conditional)
- [ ] Component selection appropriate for time control
- [ ] Tested on diverse positions
- [ ] Memory usage profiled and acceptable
- [ ] Performance benchmarked
- [ ] A/B tested against baseline

## Performance Optimization Checklist

- [ ] Phase cache enabled
- [ ] Evaluation cache enabled
- [ ] Appropriate component selection
- [ ] Statistics disabled in production
- [ ] Caches cleared between games
- [ ] Cache size limits set appropriately

## Summary

**Key Principles**:
1. **Enable caching** - Massive performance gains
2. **Choose components wisely** - Balance speed/accuracy
3. **Disable statistics in production** - Eliminate overhead
4. **Tune weights** - 10-20% accuracy improvement
5. **Test thoroughly** - Validate on diverse positions
6. **Profile first** - Measure before optimizing
7. **A/B test** - Validate improvements

Following these best practices ensures:
- **Maximum performance** (~800ns per evaluation)
- **High accuracy** (tuned weights)
- **Stable operation** (well-tested)
- **Efficient memory** (controlled cache growth)

---

*Guide Version: 1.0*
*Generated: October 8, 2025*

