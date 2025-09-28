# Design Document: Quiescence Search Implementation

## 1. Overview

This document provides a detailed design for implementing quiescence search in the Shogi engine. Quiescence search is a critical component that prevents the "horizon effect" by extending the search beyond the nominal depth to evaluate only tactical moves (captures, checks, and promotions) until the position becomes "quiet."

## 2. Current State Analysis

### 2.1 Existing Implementation
The current engine already has a basic quiescence search implementation in `src/search.rs`:

```rust
fn quiescence_search(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32
```

**Current Features:**
- ✅ Stand-pat evaluation
- ✅ Beta cutoff optimization
- ✅ Alpha-beta pruning
- ✅ Time management integration
- ✅ Depth limiting (max depth = 5)
- ✅ Noisy move generation (captures only)

**Current Limitations:**
- ❌ Limited to captures only (missing checks and promotions)
- ❌ No move ordering optimization for quiescence
- ❌ No delta pruning
- ❌ No futility pruning
- ❌ No transposition table integration
- ❌ No selective extensions

## 3. Enhanced Design Specification

### 3.1 Core Architecture

The quiescence search will be enhanced as a specialized search function that operates at the leaf nodes of the main negamax search. It will use a different move generation strategy focused on tactical moves.

#### 3.1.1 Integration Points

**Primary Integration:**
- Called from `negamax` when `depth == 0`
- Operates as a specialized tactical search
- Uses separate move generation for tactical moves

**Secondary Integration:**
- Time management through existing `should_stop` mechanism
- Transposition table for position caching
- Move ordering using specialized quiescence heuristics

### 3.2 Enhanced Move Generation

#### 3.2.1 Noisy Move Categories

The enhanced quiescence search will consider multiple types of "noisy" moves:

1. **Captures** (existing)
   - All piece captures
   - Drop captures
   - Priority: MVV-LVA (Most Valuable Victim, Least Valuable Aggressor)

2. **Checks** (new)
   - Direct checks to opponent king
   - Discovered checks
   - Priority: Check evasion moves

3. **Promotions** (new)
   - Pawn promotions
   - Lance promotions
   - Knight promotions
   - Silver promotions
   - Priority: Promotion value vs. material cost

4. **Tactical Threats** (new)
   - Moves that create immediate threats
   - Moves that defend against threats
   - Priority: Threat value assessment

#### 3.2.2 Move Generation Strategy

```rust
pub fn generate_quiescence_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
    let mut moves = Vec::new();
    
    // 1. Generate captures (highest priority)
    moves.extend(self.generate_legal_captures(board, player, captured_pieces));
    
    // 2. Generate checks
    moves.extend(self.generate_checks(board, player, captured_pieces));
    
    // 3. Generate promotions
    moves.extend(self.generate_promotions(board, player, captured_pieces));
    
    // 4. Generate tactical threats
    moves.extend(self.generate_tactical_threats(board, player, captured_pieces));
    
    // Remove duplicates and sort by priority
    moves.sort_by(|a, b| self.compare_quiescence_moves(a, b));
    moves
}
```

### 3.3 Advanced Optimizations

#### 3.3.1 Delta Pruning

Prevent searching moves that cannot improve the position significantly:

```rust
fn should_prune_delta(&self, move_: &Move, stand_pat: i32, alpha: i32) -> bool {
    let material_gain = move_.captured_piece_value();
    let promotion_bonus = move_.promotion_value();
    let total_gain = material_gain + promotion_bonus;
    
    // If the best possible outcome is still worse than alpha, prune
    stand_pat + total_gain <= alpha
}
```

#### 3.3.2 Futility Pruning

Skip moves that are unlikely to improve the position:

```rust
fn should_prune_futility(&self, move_: &Move, stand_pat: i32, alpha: i32, depth: u8) -> bool {
    let futility_margin = match depth {
        1 => 100,  // Small margin for depth 1
        2 => 200,  // Larger margin for depth 2
        _ => 300,  // Even larger margin for deeper searches
    };
    
    let material_gain = move_.captured_piece_value();
    stand_pat + material_gain + futility_margin <= alpha
}
```

#### 3.3.3 Selective Extensions

Extend search for critical positions:

```rust
fn should_extend(&self, board: &BitboardBoard, move_: &Move, depth: u8) -> bool {
    // Extend for checks
    if move_.gives_check {
        return true;
    }
    
    // Extend for recaptures
    if move_.is_recapture {
        return true;
    }
    
    // Extend for promotions
    if move_.is_promotion {
        return true;
    }
    
    // Extend for captures of high-value pieces
    if move_.is_capture && move_.captured_piece_value() > 500 {
        return true;
    }
    
    false
}
```

### 3.4 Move Ordering Enhancements

#### 3.4.1 Quiescence-Specific Ordering

```rust
fn compare_quiescence_moves(&self, a: &Move, b: &Move) -> Ordering {
    // 1. Checks first
    match (a.gives_check, b.gives_check) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }
    
    // 2. MVV-LVA for captures
    if a.is_capture && b.is_capture {
        let a_value = a.captured_piece_value() - a.piece_value();
        let b_value = b.captured_piece_value() - b.piece_value();
        return b_value.cmp(&a_value);
    }
    
    // 3. Promotions
    match (a.is_promotion, b.is_promotion) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }
    
    // 4. Default ordering
    Ordering::Equal
}
```

### 3.5 Transposition Table Integration

#### 3.5.1 Q-Search TT Entries

```rust
#[derive(Clone)]
struct QuiescenceEntry {
    score: i32,
    depth: u8,
    flag: TranspositionFlag,
    best_move: Option<Move>,
}

impl SearchEngine {
    fn quiescence_search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
        let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
        
        // Check transposition table
        if let Some(entry) = self.quiescence_tt.get(&fen_key) {
            if entry.depth >= depth {
                match entry.flag {
                    TranspositionFlag::Exact => return entry.score,
                    TranspositionFlag::LowerBound => if entry.score >= beta { return entry.score; },
                    TranspositionFlag::UpperBound => if entry.score <= alpha { return entry.score; },
                }
            }
        }
        
        // ... rest of quiescence search implementation
        
        // Store result in transposition table
        let flag = if best_score <= alpha { TranspositionFlag::UpperBound } 
                  else if best_score >= beta { TranspositionFlag::LowerBound } 
                  else { TranspositionFlag::Exact };
        
        self.quiescence_tt.insert(fen_key, QuiescenceEntry {
            score: best_score,
            depth,
            flag,
            best_move: best_move_for_tt,
        });
        
        best_score
    }
}
```

### 3.6 Performance Considerations

#### 3.6.1 Depth Limiting Strategy

```rust
const MAX_QUIESCENCE_DEPTH: u8 = 8;
const QUIESCENCE_DEPTH_REDUCTION: u8 = 1;

fn quiescence_search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
    // Early termination conditions
    if depth == 0 {
        return self.evaluator.evaluate(board, player, captured_pieces);
    }
    
    if depth > MAX_QUIESCENCE_DEPTH {
        return self.evaluator.evaluate(board, player, captured_pieces);
    }
    
    // ... rest of implementation
}
```

#### 3.6.2 Time Management

```rust
fn quiescence_search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
    // Time check at the beginning
    if self.should_stop(&start_time, time_limit_ms) {
        return 0;
    }
    
    // Time check before each move
    for move_ in sorted_noisy_moves {
        if self.should_stop(&start_time, time_limit_ms) { 
            break; 
        }
        
        // ... move processing
    }
}
```

### 3.7 Testing and Validation

#### 3.7.1 Test Positions

Create specific test positions that demonstrate quiescence search benefits:

1. **Tactical Puzzles**: Positions where immediate captures are available
2. **Horizon Effect Tests**: Positions where the engine without q-search would blunder
3. **Endgame Positions**: Complex endgame scenarios requiring deep tactical analysis
4. **Sacrifice Positions**: Positions where material sacrifice leads to tactical advantage

#### 3.7.2 Performance Benchmarks

1. **Node Count Analysis**: Compare nodes searched with/without quiescence
2. **Time Analysis**: Measure search time impact
3. **Accuracy Tests**: Solve tactical puzzles with different q-search configurations
4. **Self-Play**: Engine vs. engine matches with different q-search settings

### 3.8 Configuration Parameters

#### 3.8.1 Tunable Parameters

```rust
pub struct QuiescenceConfig {
    pub max_depth: u8,                    // Maximum quiescence depth
    pub enable_delta_pruning: bool,       // Enable delta pruning
    pub enable_futility_pruning: bool,    // Enable futility pruning
    pub enable_selective_extensions: bool, // Enable selective extensions
    pub enable_tt: bool,                 // Enable transposition table
    pub futility_margin: i32,            // Futility pruning margin
    pub delta_margin: i32,               // Delta pruning margin
}
```

#### 3.8.2 Default Configuration

```rust
impl Default for QuiescenceConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            enable_delta_pruning: true,
            enable_futility_pruning: true,
            enable_selective_extensions: true,
            enable_tt: true,
            futility_margin: 200,
            delta_margin: 100,
        }
    }
}
```

## 4. Implementation Plan

### 4.1 Phase 1: Core Enhancements
1. Implement enhanced move generation for quiescence
2. Add delta pruning and futility pruning
3. Implement selective extensions
4. Add quiescence-specific move ordering

### 4.2 Phase 2: Advanced Features
1. Integrate transposition table for quiescence
2. Add configuration system
3. Implement performance monitoring
4. Add comprehensive testing suite

### 4.3 Phase 3: Optimization
1. Fine-tune parameters through testing
2. Optimize move generation performance
3. Implement advanced pruning techniques
4. Performance profiling and optimization

## 5. Expected Benefits

### 5.1 Tactical Accuracy
- Elimination of horizon effect
- Better evaluation of tactical sequences
- Improved handling of complex tactical positions

### 5.2 Search Efficiency
- Reduced search time through pruning
- Better move ordering
- Transposition table benefits

### 5.3 Engine Strength
- Significant improvement in tactical positions
- Better endgame play
- More accurate evaluation of sacrifices

## 6. Risk Assessment

### 6.1 Performance Risks
- Increased search time in some positions
- Memory usage from transposition table
- Complexity of implementation

### 6.2 Mitigation Strategies
- Careful parameter tuning
- Performance monitoring
- Gradual implementation with testing
- Fallback to simpler implementation if needed

## 7. Conclusion

The enhanced quiescence search implementation will significantly improve the engine's tactical capabilities while maintaining good performance through careful optimization and pruning techniques. The modular design allows for gradual implementation and testing of individual components.
