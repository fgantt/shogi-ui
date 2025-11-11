# Pattern Recognition Usage Examples

**Version**: 1.0  
**Date**: October 8, 2025

## Basic Usage Examples

### Example 1: Simple Evaluation with Patterns

```rust
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{Player, CapturedPieces};

fn main() {
    // Create board and evaluator
    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let mut evaluator = PositionEvaluator::new();
    
    // Evaluate position (patterns automatically included)
    let score = evaluator.evaluate(&mut board, Player::Black, &captured_pieces);
    
    println!("Position evaluation: {} centipawns", score);
    // Includes: material, piece-square tables, pawn structure,
    //           king safety, mobility, tactical patterns, 
    //           positional patterns, and more!
}
```

---

### Example 2: Using Individual Pattern Recognizers

```rust
use shogi_engine::evaluation::tactical_patterns::TacticalPatternRecognizer;
use shogi_engine::evaluation::positional_patterns::PositionalPatternAnalyzer;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::Player;

fn analyze_position() {
    let board = BitboardBoard::new();
    let player = Player::Black;
    
    // Tactical analysis
    let mut tactical = TacticalPatternRecognizer::new();
    let tactical_score = tactical.evaluate_tactics(&board, player);
    println!("Tactical: {}mg / {}eg", tactical_score.mg, tactical_score.eg);
    
    // Positional analysis
    let mut positional = PositionalPatternAnalyzer::new();
    let positional_score = positional.evaluate_position(&board, player, &CapturedPieces::new());
    println!("Positional: {}mg / {}eg", positional_score.mg, positional_score.eg);
}
```

---

### Example 3: Pattern Configuration

```rust
use shogi_engine::evaluation::pattern_config::PatternConfig;

fn configure_patterns() -> Result<(), String> {
    let mut config = PatternConfig::default();
    
    // Enable all patterns
    config.enable_all();
    
    // Adjust weights for aggressive play
    config.set_tactical_patterns_weight(1.5);  // Emphasize tactics
    config.set_king_safety_weight(1.2);        // Emphasize king safety
    config.set_positional_patterns_weight(0.9); // Slightly reduce positional
    
    // Validate configuration
    config.validate()?;
    
    // Save configuration
    let json = config.to_json()?;
    std::fs::write("aggressive_config.json", json)?;
    
    println!("Configuration saved!");
    Ok(())
}
```

---

### Example 4: Custom ComponentFlags

```rust
use shogi_engine::evaluation::integration::{
    IntegratedEvaluator,
    IntegratedEvaluationConfig,
    ComponentFlags,
};

fn create_custom_evaluator() {
    // Custom component selection
    let components = ComponentFlags {
        material: true,
        piece_square_tables: true,
        position_features: true,
        opening_principles: false,
        endgame_patterns: true,
        tactical_patterns: true,    // Enable tactical patterns
        positional_patterns: false, // Disable positional patterns
    };
    
    let mut config = IntegratedEvaluationConfig::default();
    config.components = components;
    
    let evaluator = IntegratedEvaluator::with_config(config);
    
    // Use evaluator
    let score = evaluator.evaluate(&board, player, &captured_pieces);
}
```

---

### Example 5: Pattern Caching

```rust
use shogi_engine::evaluation::pattern_cache::{PatternCache, CachedPatternResult};

fn use_pattern_cache() {
    let mut cache = PatternCache::new(100_000);
    
    // Compute position hash
    let position_hash = compute_position_hash(&board, player);
    
    // Try cache lookup first
    if let Some(cached) = cache.lookup(position_hash) {
        println!("Cache hit!");
        return cached.tactical_score.0 + cached.positional_score.0;
    }
    
    // Cache miss - evaluate patterns
    let tactical = evaluate_tactical_patterns(&board, player);
    let positional = evaluate_positional_patterns(&board, player);
    
    // Store in cache
    let result = CachedPatternResult {
        tactical_score: (tactical.mg, tactical.eg),
        positional_score: (positional.mg, positional.eg),
        endgame_score: (0, 0),
        age: 0,
    };
    cache.store(position_hash, result);
    
    // Monitor cache performance
    println!("Hit rate: {:.1}%", cache.hit_rate() * 100.0);
    println!("Cache usage: {:.1}%", cache.usage_percent());
}
```

---

### Example 6: Search Integration

```rust
use shogi_engine::evaluation::pattern_search_integration::PatternSearchIntegrator;

fn enhanced_move_ordering(moves: &[Move], board: &BitboardBoard, player: Player) -> Vec<Move> {
    let mut integrator = PatternSearchIntegrator::new();
    
    // Order moves by pattern value
    let ordered = integrator.order_moves_by_patterns(board, moves, player);
    
    // Extract just the moves (sorted by score)
    ordered.into_iter().map(|(mv, _score)| mv).collect()
}

fn should_prune_node(
    board: &BitboardBoard,
    player: Player,
    depth: u8,
    alpha: i32,
    beta: i32
) -> bool {
    let mut integrator = PatternSearchIntegrator::new();
    
    // Check if we should prune based on patterns
    integrator.should_prune_by_patterns(board, player, depth, alpha, beta)
}
```

---

### Example 7: Dynamic Pattern Selection

```rust
use shogi_engine::evaluation::pattern_advanced::AdvancedPatternSystem;

fn adaptive_evaluation(board: &BitboardBoard, game_phase: u8, player: Player) {
    let mut system = AdvancedPatternSystem::new();
    
    // Select patterns based on game phase
    let selection = system.select_patterns(board, game_phase);
    
    if game_phase > 192 {
        // Opening - emphasizes positional patterns
        println!("Opening phase: Positional weight = {}", selection.weights[6]);
    } else if game_phase > 64 {
        // Middlegame - balanced
        println!("Middlegame: Balanced weights");
    } else {
        // Endgame - emphasizes endgame patterns
        println!("Endgame phase: Endgame weight = {}", selection.weights[7]);
    }
}
```

---

### Example 8: Pattern Analytics

```rust
use shogi_engine::evaluation::pattern_advanced::PatternAnalytics;

fn track_pattern_usage() {
    let mut analytics = PatternAnalytics::new();
    
    // Record pattern occurrences during analysis
    analytics.record_pattern("fork", 120);
    analytics.record_pattern("fork", 150);
    analytics.record_pattern("pin", 80);
    analytics.record_pattern("outpost", 60);
    
    // Get statistics
    println!("Fork frequency: {}", analytics.get_frequency("fork"));
    println!("Fork average value: {:.1}", analytics.get_average_value("fork"));
    
    let stats = analytics.get_stats();
    println!("Total patterns detected: {}", stats.total_patterns);
    println!("Unique pattern types: {}", stats.unique_patterns);
}
```

---

### Example 9: Comprehensive Testing

```rust
use shogi_engine::evaluation::pattern_comprehensive_tests::PatternTestSuite;

fn run_full_test_suite() {
    let mut suite = PatternTestSuite::new();
    
    // Run all test categories
    let success = suite.run_all_tests();
    
    // Print detailed summary
    suite.print_summary();
    
    // Get results
    let results = suite.results();
    println!("\nOverall Results:");
    println!("  Total tests: {}", results.total_run());
    println!("  Passed: {}", results.total_passed());
    println!("  Pass rate: {:.1}%", results.pass_rate() * 100.0);
    
    if success {
        println!("\n✅ All pattern tests passed!");
    }
}
```

---

### Example 10: Complete Integration Example

```rust
use shogi_engine::evaluation::{
    PositionEvaluator,
    pattern_config::PatternConfig,
    pattern_cache::PatternCache,
    pattern_search_integration::PatternSearchIntegrator,
};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{Player, CapturedPieces, Move};

struct EnhancedEngine {
    evaluator: PositionEvaluator,
    pattern_cache: PatternCache,
    search_integrator: PatternSearchIntegrator,
}

impl EnhancedEngine {
    fn new() -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            pattern_cache: PatternCache::new(100_000),
            search_integrator: PatternSearchIntegrator::new(),
        }
    }
    
    fn evaluate_position(&mut self, board: &mut BitboardBoard, player: Player) -> i32 {
        let captured_pieces = CapturedPieces::new();
        
        // Evaluation automatically includes patterns
        self.evaluator.evaluate(board, player, &captured_pieces)
    }
    
    fn order_moves(&mut self, moves: &[Move], board: &BitboardBoard, player: Player) -> Vec<Move> {
        // Order moves using pattern recognition
        let ordered = self.search_integrator
            .order_moves_by_patterns(board, moves, player);
        
        ordered.into_iter().map(|(mv, _)| mv).collect()
    }
    
    fn should_prune(&mut self, board: &BitboardBoard, player: Player, depth: u8, alpha: i32, beta: i32) -> bool {
        // Use pattern-based pruning
        self.search_integrator.should_prune_by_patterns(board, player, depth, alpha, beta)
    }
}

fn main() {
    let mut engine = EnhancedEngine::new();
    let mut board = BitboardBoard::new();
    
    // Evaluate
    let score = engine.evaluate_position(&mut board, Player::Black);
    println!("Position score: {}", score);
    
    // Order moves
    let moves = vec![/* generated moves */];
    let ordered_moves = engine.order_moves(&moves, &board, Player::Black);
    println!("Ordered {} moves", ordered_moves.len());
}
```

---

## Best Practices

1. **Use PositionEvaluator** - Patterns are automatically included
2. **Enable caching** - 90% speedup on cache hits
3. **Configure for your needs** - Use ComponentFlags to enable/disable features
4. **Monitor performance** - Check statistics and cache hit rates
5. **Validate configurations** - Always call `config.validate()`
6. **Use appropriate weights** - Tune for your playing style
7. **Test thoroughly** - Use PatternTestSuite

---

## Next Steps

- See `PATTERN_TUNING_GUIDE.md` for weight optimization
- See `PATTERN_BEST_PRACTICES.md` for advanced techniques
- See `PATTERN_TROUBLESHOOTING.md` for common issues

**Usage Examples Complete** ✅
