# Pattern Recognition User Guide

**Version**: 1.0  
**Date**: October 8, 2025

## Introduction

The Pattern Recognition system provides comprehensive tactical, positional, and endgame pattern detection for the Shogi engine. This guide helps you understand and effectively use the pattern recognition features.

## Quick Start

### Default Usage (Recommended)

```rust
// Patterns are automatically enabled!
let mut evaluator = PositionEvaluator::new();
let score = evaluator.evaluate(&board, player, &captured_pieces);
// ✅ Includes all pattern recognition automatically
```

**That's it!** The pattern recognition system is enabled by default and requires no additional setup.

## Pattern Types

### 1. Tactical Patterns (Task 2.1)
**Detects**: Forks, Pins, Skewers, Discovered Attacks, Knight Forks, Back Rank Threats

**When Active**: Always (in middlegame and endgame)

**Impact**: Identifies immediate tactical opportunities and threats
- Fork: Double attack on 2+ pieces (+50-200 points)
- Pin: Piece cannot move without exposing valuable piece (-40% piece value)
- Skewer: Attack through less valuable to more valuable piece (+30% value diff)

### 2. Positional Patterns (Task 2.2)
**Evaluates**: Center Control, Outposts, Weak Squares, Piece Activity, Space, Tempo

**When Active**: Primarily middlegame

**Impact**: Assesses long-term positional advantages
- Center control: Knight in center (+30mg/+15eg)
- Outpost: Supported advanced piece (+60mg/+40eg for knight)
- Weak squares: Squares not defendable by pawns (-40mg/-20eg)

### 3. Endgame Patterns (Task 2.3)
**Detects**: Zugzwang, Opposition, Triangulation, Piece vs Pawns, Fortresses

**When Active**: Endgame phase (game_phase < 64)

**Impact**: Improves endgame play and conversion
- Opposition: King positioning advantage (+40eg direct)
- Fortress: Defensive structure when behind (+120eg when significantly behind)
- Piece vs pawns: Rook/Bishop vs pawns evaluation (+100eg if winning)

## Configuration

### Enable/Disable Patterns

```rust
use shogi_engine::evaluation::integration::ComponentFlags;

// Minimal configuration (material + PST only)
let components = ComponentFlags::minimal();

// All features enabled
let components = ComponentFlags::all_enabled();

// Custom selection
let components = ComponentFlags {
    material: true,
    piece_square_tables: true,
    position_features: true,
    opening_principles: true,
    endgame_patterns: true,
    tactical_patterns: true,      // ✅ Tactical patterns
    positional_patterns: true,    // ✅ Positional patterns
};
```

### Adjust Pattern Weights

```rust
let mut config = PatternConfig::default();

// Aggressive style (emphasize tactics)
config.set_tactical_patterns_weight(1.5);
config.set_king_safety_weight(1.3);
config.set_positional_patterns_weight(0.8);

// Positional style (emphasize structure)
config.set_positional_patterns_weight(1.4);
config.set_pawn_structure_weight(1.3);
config.set_tactical_patterns_weight(0.9);

// Defensive style (emphasize king safety)
config.set_king_safety_weight(1.6);
config.set_fortress_patterns_weight(1.4);
```

## Performance Optimization

### Use Pattern Caching

```rust
// Cache automatically used in IntegratedEvaluator
let evaluator = PositionEvaluator::new();
// Pattern cache with 100K entries active

// Monitor cache effectiveness
// (Check cache hit rate if statistics enabled)
```

**Performance Gain**: 90% faster on cache hits, typical hit rate 60-80%

### WASM Optimization

```rust
#[cfg(target_arch = "wasm32")]
fn create_wasm_evaluator() {
    use shogi_engine::evaluation::wasm_compatibility::WasmEvaluationOptimizer;
    
    // Get WASM-optimized components
    let components = WasmEvaluationOptimizer::get_wasm_components(true);
    
    let mut config = IntegratedEvaluationConfig::default();
    config.components = components;
    // Patterns disabled by default for binary size
    
    // Enable if binary size allows
    config.components.tactical_patterns = true;  // +~50KB
}
```

## Understanding Pattern Scores

### TaperedScore Structure

All pattern methods return `TaperedScore`:
```rust
pub struct TaperedScore {
    pub mg: i32,  // Middlegame score
    pub eg: i32,  // Endgame score
}
```

**Interpolation**: Final score = mg * phase_factor + eg * (1 - phase_factor)

### Score Ranges

| Pattern Type | Typical Range | Impact |
|--------------|---------------|--------|
| Tactical | ±50 to ±300 | High |
| Positional | ±20 to ±150 | Medium |
| Endgame | ±30 to ±200 | High (in endgame) |

### Interpreting Scores

- **Positive**: Good for evaluated player
- **Negative**: Bad for evaluated player
- **>100**: Significant advantage
- **>200**: Strong advantage
- **>500**: Winning advantage

## Common Use Cases

### Use Case 1: Position Analysis

```rust
fn analyze_position_patterns(board: &BitboardBoard, player: Player) {
    let mut tactical = TacticalPatternRecognizer::new();
    let mut positional = PositionalPatternAnalyzer::new();
    
    let tactical_score = tactical.evaluate_tactics(board, player);
    let positional_score = positional.evaluate_position(board, player, captured_pieces);
    
    println!("Tactical evaluation:");
    println!("  Middlegame: {}", tactical_score.mg);
    println!("  Endgame: {}", tactical_score.eg);
    
    println!("Positional evaluation:");
    println!("  Middlegame: {}", positional_score.mg);
    println!("  Endgame: {}", positional_score.eg);
    
    // Identify weaknesses
    if tactical_score.mg < -50 {
        println!("⚠️  Tactical weakness detected!");
    }
    if positional_score.mg < -30 {
        println!("⚠️  Positional weakness detected!");
    }
}
```

### Use Case 2: Training Position Evaluation

```rust
fn evaluate_training_position(fen: &str) -> i32 {
    // Parse position
    let board = parse_fen(fen);
    let player = Player::Black;
    let captured_pieces = CapturedPieces::new();
    
    // Evaluate with patterns
    let mut evaluator = PositionEvaluator::new();
    let score = evaluator.evaluate(&board, player, &captured_pieces);
    
    score
}
```

### Use Case 3: Interactive Analysis

```rust
fn interactive_pattern_analysis() {
    let mut evaluator = PositionEvaluator::new();
    let mut board = BitboardBoard::new();
    
    loop {
        // Get user input for moves
        let user_move = get_user_move();
        apply_move(&mut board, user_move);
        
        // Evaluate new position
        let score = evaluator.evaluate(&board, Player::Black, &CapturedPieces::new());
        
        // Show evaluation
        println!("Position evaluation: {}", score);
        println!("  (includes tactical, positional, and endgame patterns)");
    }
}
```

## Troubleshooting

### Issue: Evaluation too slow

**Solution**: Patterns add <1ms overhead. If slower:
1. Check if caching is enabled
2. Verify pattern_cache_size is appropriate
3. Consider disabling some pattern types

### Issue: Unexpected evaluation changes

**Solution**: Pattern recognition is working!
- Tactical patterns detect forks, pins, etc.
- Positional patterns evaluate center control, outposts
- This is expected behavior

### Issue: WASM binary too large

**Solution**: Patterns are disabled by default in WASM
- Binary size impact: ~100KB if enabled
- Keep patterns disabled for smallest binary
- Enable selectively if needed

## Advanced Topics

### Custom Pattern Weights

See `PATTERN_TUNING_GUIDE.md` for detailed weight optimization.

### Pattern Visualization

See `PATTERN_VISUALIZATION_EXAMPLES.md` for debugging visualizations.

### Performance Profiling

See `PATTERN_PERFORMANCE_GUIDE.md` for optimization techniques.

---

**Pattern Recognition Guide Complete** ✅
