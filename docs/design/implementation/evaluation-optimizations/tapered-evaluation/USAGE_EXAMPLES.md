# Tapered Evaluation - Usage Examples

## Quick Start

### 1. Basic Evaluation

```rust
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{Player, CapturedPieces};

fn main() {
    // Create evaluator (tapered evaluation enabled by default)
    let evaluator = PositionEvaluator::new();
    
    // Create a position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Evaluate
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    println!("Position score: {}", score);
}
```

### 2. With Statistics Tracking

```rust
use shogi_engine::evaluation::PositionEvaluator;

fn main() {
    let evaluator = PositionEvaluator::new();
    
    // Enable statistics
    evaluator.enable_integrated_statistics();
    
    // Run many evaluations
    for position in load_positions() {
        let score = evaluator.evaluate(&position.board, position.player, &position.captured);
        println!("Score: {}", score);
    }
    
    // Get statistics report
    if let Some(stats) = evaluator.get_integrated_statistics() {
        let report = stats.generate_report();
        println!("\n{}", report);
        
        // Export to JSON
        if let Ok(json) = stats.export_json() {
            std::fs::write("eval_stats.json", json).unwrap();
        }
    }
}
```

### 3. Custom Configuration

```rust
use shogi_engine::evaluation::integration::*;

fn main() {
    // Create custom configuration
    let mut config = IntegratedEvaluationConfig::default();
    
    // Use minimal components for speed
    config.components = ComponentFlags::minimal();
    
    // Enable caching
    config.enable_phase_cache = true;
    config.enable_eval_cache = true;
    
    // Create evaluator
    let mut evaluator = IntegratedEvaluator::with_config(config);
    
    // Use it
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
}
```

## Advanced Examples

### 4. Performance Profiling

```rust
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use std::time::Instant;

fn main() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..10000 {
        evaluator.evaluate(&board, Player::Black, &captured_pieces);
    }
    let duration = start.elapsed();
    
    println!("10,000 evaluations in {:?}", duration);
    println!("Average: {:.2} μs per evaluation", duration.as_micros() as f64 / 10000.0);
    
    // Get detailed statistics
    let stats = evaluator.get_statistics();
    let report = stats.generate_report();
    println!("\n{}", report);
}
```

### 9. Inspect PST Telemetry

```rust
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use shogi_engine::types::{BitboardBoard, CapturedPieces, Player};

fn main() {
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    let mut board = BitboardBoard::new();
    // ... populate board with an interesting position ...

    let captured = CapturedPieces::new();
    let _score = evaluator.evaluate(&board, Player::Black, &captured);

    if let Some(telemetry) = evaluator.telemetry_snapshot() {
        if let Some(pst) = telemetry.pst {
            println!(
                "PST totals → mg: {}  eg: {}  |avg magnitude| ≈ {}",
                pst.total_mg,
                pst.total_eg,
                pst.per_piece
                    .iter()
                    .map(|entry| (entry.mg.abs() + entry.eg.abs()) as f64)
                    .sum::<f64>()
                    / pst.per_piece.len().max(1) as f64
            );

            println!("Top contributors:");
            for entry in pst.per_piece.iter().take(5) {
                println!("  {:?}: mg {}  eg {}", entry.piece, entry.mg, entry.eg);
            }
        }
    }
}
```

> **Tip:** Enable `SearchEngine::debug_logging` to mirror these totals in search traces (`[EvalTelemetry] pst_total …`). With the optimized evaluator enabled, the profiler report now includes average PST contribution (mg / eg / |total|) alongside timing percentages, making it easy to spot regressions during self-play or nightly runs.

### 5. Cache Management

```rust
use shogi_engine::evaluation::integration::IntegratedEvaluator;

fn main() {
    let mut evaluator = IntegratedEvaluator::new();
    
    // Evaluate many positions
    for position in positions {
        let score = evaluator.evaluate(&position.board, position.player, &position.captured);
    }
    
    // Check cache stats
    let cache_stats = evaluator.cache_stats();
    println!("Phase cache: {} entries", cache_stats.phase_cache_size);
    println!("Eval cache: {} entries", cache_stats.eval_cache_size);
    
    // Clear if needed (e.g., between games)
    evaluator.clear_caches();
}
```

### 6. Component Selection

```rust
use shogi_engine::evaluation::integration::*;

fn evaluate_with_components(enable_opening: bool, enable_endgame: bool) -> i32 {
    let mut config = IntegratedEvaluationConfig::default();
    
    config.components = ComponentFlags {
        material: true,
        piece_square_tables: true,
        position_features: true,
        opening_principles: enable_opening,
        endgame_patterns: enable_endgame,
    };
    
    let mut evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    evaluator.evaluate(&board, Player::Black, &captured_pieces)
}
```

## Search Integration Examples

### 7. Basic Search with Tapered Evaluation

```rust
use shogi_engine::search::SearchEngine;

fn main() {
    // Create search engine (tapered evaluation active by default)
    let mut search_engine = SearchEngine::new(None, 64);
    
    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Search (automatically benefits from tapered evaluation)
    let (best_move, score) = search_engine.search_iterative(
        &mut board,
        &captured_pieces,
        Player::Black,
        5,      // depth
        5000    // time limit ms
    );
    
    println!("Best move: {}", best_move.to_usi_string());
    println!("Score: {}", score);
}
```

### 8. Phase-Aware Search

```rust
use shogi_engine::search::SearchEngine;

fn search_with_phase_tracking(board: &mut BitboardBoard, player: Player) {
    let mut search_engine = SearchEngine::new(None, 64);
    
    // Access tapered search enhancer
    let enhancer = search_engine.get_tapered_search_enhancer_mut();
    
    // Track phase
    let phase = enhancer.track_phase(board);
    println!("Current phase: {}", phase);
    
    // This information can be used for phase-aware decisions
    // (In actual search, this would be called within negamax/alpha-beta)
}
```

## Tuning Examples

### 9. Automated Weight Tuning

```rust
use shogi_engine::evaluation::tuning::*;

fn tune_evaluation_weights() -> Result<(), TuningError> {
    let mut tuner = TaperedEvaluationTuner::new();
    
    // Load training data from game database
    let positions = load_training_positions("games.db");
    tuner.add_training_data(positions);
    
    // Split data
    tuner.split_data(0.2);
    
    // Optimize
    let results = tuner.optimize()?;
    
    println!("Optimization complete!");
    println!("  Iterations: {}", results.iterations);
    println!("  Training error: {:.4}", results.training_error);
    println!("  Validation error: {:.4}", results.validation_error);
    println!("  Duration: {:?}", results.duration);
    
    // Get optimized weights
    let weights = results.optimized_weights;
    println!("\nOptimized weights:");
    println!("  Material: {}", weights.material_weight);
    println!("  Position: {}", weights.position_weight);
    
    Ok(())
}

fn load_training_positions(path: &str) -> Vec<TuningPosition> {
    // Load from your game database
    vec![]
}
```

### 10. Custom Tuning Configuration

```rust
use shogi_engine::evaluation::tuning::*;

fn tune_with_genetic_algorithm() -> Result<(), TuningError> {
    let config = TuningConfig {
        method: OptimizationMethod::GeneticAlgorithm,
        learning_rate: 0.001,
        max_iterations: 500,
        convergence_threshold: 0.0001,
    };
    
    let mut tuner = TaperedEvaluationTuner::with_config(config);
    
    // Add data and optimize
    tuner.add_training_data(load_positions());
    let results = tuner.optimize()?;
    
    Ok(())
}
```

## Advanced Interpolation Examples

### 11. Spline Interpolation

```rust
use shogi_engine::evaluation::advanced_interpolation::*;

fn evaluate_with_spline() {
    let mut config = AdvancedInterpolationConfig::default();
    config.use_spline = true;
    
    let interpolator = AdvancedInterpolator::with_config(config);
    
    let score = TaperedScore::new_tapered(100, 200);
    let phase = 128;
    
    let result = interpolator.interpolate_spline(score, phase);
    println!("Spline interpolated score: {}", result);
}
```

### 12. Multi-Phase Evaluation

```rust
use shogi_engine::evaluation::advanced_interpolation::*;

fn evaluate_by_position_type(position_type: PositionType) {
    let interpolator = AdvancedInterpolator::new();
    let score = TaperedScore::new_tapered(100, 200);
    let phase = 128;
    
    let result = interpolator.interpolate_multi_phase(score, phase, position_type);
    
    match position_type {
        PositionType::Tactical => println!("Tactical: {}", result),
        PositionType::Positional => println!("Positional: {}", result),
        PositionType::Endgame => println!("Endgame: {}", result),
        PositionType::Standard => println!("Standard: {}", result),
    }
}
```

### 13. Adaptive Interpolation

```rust
use shogi_engine::evaluation::advanced_interpolation::*;

fn evaluate_with_adaptive(board: &BitboardBoard) {
    let interpolator = AdvancedInterpolator::new();
    let score = TaperedScore::new_tapered(100, 200);
    let phase = 128;
    
    // Analyze position characteristics
    let characteristics = PositionCharacteristics {
        material_reduction: 0.4,
        complexity: 0.7,
        king_safety: 0.6,
    };
    
    let result = interpolator.interpolate_adaptive(score, phase, &characteristics);
    println!("Adaptive score: {}", result);
}
```

## Testing Examples

### 14. Unit Testing with Tapered Evaluation

```rust
#[test]
fn test_my_feature() {
    let mut evaluator = IntegratedEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    
    // Assertions
    assert!(score.abs() < 100);
}
```

### 15. Benchmarking

```rust
use criterion::{black_box, Criterion};

fn benchmark_my_evaluation(c: &mut Criterion) {
    c.bench_function("my_eval", |b| {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });
}
```

## Real-World Integration

### 16. Complete Engine Integration

```rust
use shogi_engine::search::SearchEngine;
use shogi_engine::evaluation::PositionEvaluator;

struct MyEngine {
    search_engine: SearchEngine,
}

impl MyEngine {
    fn new() -> Self {
        Self {
            search_engine: SearchEngine::new(None, 128),
        }
    }
    
    fn find_best_move(&mut self, board: &mut BitboardBoard, player: Player) -> Move {
        let captured_pieces = CapturedPieces::new();
        
        // Search uses tapered evaluation automatically
        let (best_move, score) = self.search_engine.search_iterative(
            board,
            &captured_pieces,
            player,
            6,      // depth
            10000   // time limit
        );
        
        println!("Best move: {} (score: {})", best_move.to_usi_string(), score);
        
        best_move
    }
    
    fn get_evaluation_stats(&self) {
        // Access evaluator through search engine
        // (would need getter method added to SearchEngine)
        println!("Statistics tracking available through PositionEvaluator");
    }
}
```

---

*Examples Version: 1.0*
*Generated: October 8, 2025*
*Total Examples: 16*

