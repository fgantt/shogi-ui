# Pattern Visualization Examples

**Version**: 1.0  
**Date**: October 8, 2025

## Overview

Pattern visualization helps debug and understand pattern detection. This guide shows how to visualize patterns for analysis.

## Basic Board Visualization

```rust
use shogi_engine::evaluation::pattern_advanced::PatternVisualizer;

let board = BitboardBoard::new();
let patterns = vec![];  // Detected patterns

let visualization = PatternVisualizer::visualize_board(&board, &patterns);
println!("{}", visualization);
```

**Output**:
```
  a b c d e f g h i
9 . . . . . . . . . 
8 . . . . . . . . . 
7 . . . . . . . . . 
6 . . . . . . . . . 
5 . . . . * . . . .   ← Pattern square (marked with *)
4 . . . . . . . . . 
3 . . . . . . . . . 
2 . . . . . . . . . 
1 . . . . . . . . . 
```

## Pattern Explanation Visualization

```rust
let system = AdvancedPatternSystem::new();
let explanations = system.explain_patterns(&board, player);

for explanation in explanations {
    println!("\n{}", "=".repeat(50));
    println!("Pattern: {}", explanation.pattern_name);
    println!("Description: {}", explanation.description);
    println!("Value: {} centipawns", explanation.value);
    println!("Squares involved:");
    for square in &explanation.squares {
        println!("  - ({}, {})", square.row, square.col);
    }
}
```

**Output**:
```
==================================================
Pattern: Fork
Description: Knight forks king and rook
Value: 250 centipawns
Squares involved:
  - (4, 4)  ← Knight position
  - (3, 2)  ← King
  - (3, 6)  ← Rook
```

## Complete Example: Pattern Analysis Display

```rust
fn display_pattern_analysis(board: &BitboardBoard, player: Player) {
    println!("\n{}", "=".repeat(60));
    println!("PATTERN RECOGNITION ANALYSIS");
    println!("{}", "=".repeat(60));
    
    // Tactical patterns
    let mut tactical = TacticalPatternRecognizer::new();
    let tactical_score = tactical.evaluate_tactics(board, player);
    
    println!("\n📍 TACTICAL PATTERNS:");
    println!("  Score: {}mg / {}eg", tactical_score.mg, tactical_score.eg);
    
    let stats = tactical.stats();
    if stats.forks_found.load(Ordering::Relaxed) > 0 {
        println!("  ⚡ Forks detected: {}", stats.forks_found.load(Ordering::Relaxed));
    }
    if stats.pins_found.load(Ordering::Relaxed) > 0 {
        println!("  📌 Pins detected: {}", stats.pins_found.load(Ordering::Relaxed));
    }
    
    // Positional patterns
    let mut positional = PositionalPatternAnalyzer::new();
    let positional_score = positional.evaluate_position(board, player, captured_pieces);
    
    println!("\n🎯 POSITIONAL PATTERNS:");
    println!("  Score: {}mg / {}eg", positional_score.mg, positional_score.eg);
    
    let stats = positional.stats();
    if stats.outposts_found > 0 {
        println!("  🏰 Outposts: {}", stats.outposts_found);
    }
    if stats.weak_squares_found > 0 {
        println!("  ⚠️  Weak squares: {}", stats.weak_squares_found);
    }
    
    // Endgame patterns
    let mut endgame = EndgamePatternEvaluator::new();
    let endgame_score = endgame.evaluate_endgame(board, player, &captured);
    
    println!("\n♔ ENDGAME PATTERNS:");
    println!("  Score: {}mg / {}eg", endgame_score.mg, endgame_score.eg);
    
    // Summary
    let total_pattern_score = tactical_score.mg + positional_score.mg + endgame_score.mg;
    println!("\n📊 TOTAL PATTERN CONTRIBUTION: {} centipawns", total_pattern_score);
    println!("{}", "=".repeat(60));
}
```

**Example Output**:
```
============================================================
PATTERN RECOGNITION ANALYSIS
============================================================

📍 TACTICAL PATTERNS:
  Score: 120mg / 80eg
  ⚡ Forks detected: 1
  📌 Pins detected: 2

🎯 POSITIONAL PATTERNS:
  Score: 85mg / 60eg
  🏰 Outposts: 1
  ⚠️  Weak squares: 3

♔ ENDGAME PATTERNS:
  Score: 0mg / 0eg

📊 TOTAL PATTERN CONTRIBUTION: 205 centipawns
============================================================
```

---

**Visualization Examples Complete** ✅
