# Move Ordering Improvements Design

## Overview

This document outlines the design for implementing sophisticated move ordering improvements in the Shogi engine. Move ordering is critical for alpha-beta search efficiency, as better move ordering leads to more alpha-beta cutoffs and faster search performance.

## Current State

The engine currently uses basic move sorting by piece value, which is inefficient and doesn't leverage the power of alpha-beta pruning.

## Design Goals

1. **High Efficiency**: Maximize alpha-beta cutoffs through optimal move ordering
2. **Comprehensive Heuristics**: Use multiple ordering criteria for best results
3. **Performance**: Fast move ordering with minimal overhead
4. **Adaptability**: Learn from search results to improve ordering
5. **Integration**: Seamless integration with existing search algorithms

## Technical Architecture

### 1. Move Ordering System

**Purpose**: Coordinate multiple move ordering heuristics for optimal search performance.

**Components**:
- Principal Variation (PV) move ordering
- Killer move heuristic
- History heuristic
- Static Exchange Evaluation (SEE)
- Transposition table integration
- Move generation integration

**Implementation**:
```rust
pub struct MoveOrdering {
    pv_move: Option<Move>,
    killer_moves: [[Option<Move>; 2]; 64], // [depth][killer_index]
    history_table: [[i32; 81]; 81], // [from][to]
    see_cache: HashMap<Move, i32>,
    move_scores: Vec<i32>,
    ordering_stats: OrderingStats,
}

impl MoveOrdering {
    pub fn new() -> Self {
        Self {
            pv_move: None,
            killer_moves: [[None; 2]; 64],
            history_table: [[0; 81]; 81],
            see_cache: HashMap::new(),
            move_scores: Vec::new(),
            ordering_stats: OrderingStats::new(),
        }
    }
    
    pub fn order_moves(&mut self, moves: &mut Vec<Move>, board: &BitboardBoard, 
                      tt: &TranspositionTable, depth: u8) {
        self.move_scores.clear();
        self.move_scores.resize(moves.len(), 0);
        
        // Score each move
        for (i, &mv) in moves.iter().enumerate() {
            self.move_scores[i] = self.score_move(mv, board, tt, depth);
        }
        
        // Sort moves by score (descending)
        let mut move_indices: Vec<usize> = (0..moves.len()).collect();
        move_indices.sort_by(|&a, &b| self.move_scores[b].cmp(&self.move_scores[a]));
        
        // Reorder moves array
        let mut reordered_moves = Vec::with_capacity(moves.len());
        for &i in &move_indices {
            reordered_moves.push(moves[i]);
        }
        *moves = reordered_moves;
    }
}
```

### 2. Principal Variation (PV) Move Ordering

**Purpose**: Prioritize the best move from previous search iteration.

**Technical Details**:
- Store best move from transposition table
- Place PV move at front of move list
- Use PV move for move ordering in next iteration
- Update PV move based on search results

**Implementation**:
```rust
impl MoveOrdering {
    fn score_pv_move(&self, mv: Move, board: &BitboardBoard, 
                    tt: &TranspositionTable) -> i32 {
        let hash_key = tt.hash_position(board);
        
        if let Some(entry) = tt.probe(hash_key) {
            if let Some(best_move) = entry.best_move {
                if mv == best_move {
                    return 10000; // Highest priority
                }
            }
        }
        
        0
    }
    
    fn update_pv_move(&mut self, mv: Move) {
        self.pv_move = Some(mv);
    }
    
    fn clear_pv_move(&mut self) {
        self.pv_move = None;
    }
}
```

### 3. Killer Move Heuristic

**Purpose**: Prioritize moves that caused beta cutoffs at the same depth.

**Technical Details**:
- Track moves that caused beta cutoffs
- Store up to 2 killer moves per depth
- Prioritize killer moves in move ordering
- Age killer moves over time

**Implementation**:
```rust
impl MoveOrdering {
    fn score_killer_move(&self, mv: Move, depth: u8) -> i32 {
        let depth_idx = depth.min(63) as usize;
        
        for killer_idx in 0..2 {
            if let Some(killer_move) = self.killer_moves[depth_idx][killer_idx] {
                if mv == killer_move {
                    return 9000 - killer_idx as i32 * 100; // High priority
                }
            }
        }
        
        0
    }
    
    fn add_killer_move(&mut self, mv: Move, depth: u8) {
        let depth_idx = depth.min(63) as usize;
        
        // Don't add if already a killer move
        if self.is_killer_move(mv, depth) {
            return;
        }
        
        // Shift existing killer moves and add new one
        self.killer_moves[depth_idx][1] = self.killer_moves[depth_idx][0];
        self.killer_moves[depth_idx][0] = Some(mv);
    }
    
    fn is_killer_move(&self, mv: Move, depth: u8) -> bool {
        let depth_idx = depth.min(63) as usize;
        
        for killer_idx in 0..2 {
            if let Some(killer_move) = self.killer_moves[depth_idx][killer_idx] {
                if mv == killer_move {
                    return true;
                }
            }
        }
        
        false
    }
}
```

### 4. History Heuristic

**Purpose**: Track move success rates across all positions and depths.

**Technical Details**:
- Maintain history table for all move combinations
- Update history values based on search results
- Use history values for move ordering
- Implement aging to prevent stale data

**Implementation**:
```rust
impl MoveOrdering {
    fn score_history_move(&self, mv: Move) -> i32 {
        let from_idx = mv.from as usize;
        let to_idx = mv.to as usize;
        
        // Return history value scaled to appropriate range
        (self.history_table[from_idx][to_idx] / 100).clamp(0, 1000)
    }
    
    fn update_history(&mut self, mv: Move, depth: u8, bonus: i32) {
        let from_idx = mv.from as usize;
        let to_idx = mv.to as usize;
        
        // Scale bonus by depth (deeper searches get higher bonuses)
        let scaled_bonus = bonus * (depth + 1) as i32;
        
        // Update history value
        self.history_table[from_idx][to_idx] += scaled_bonus;
        
        // Prevent overflow
        if self.history_table[from_idx][to_idx] > 1000000 {
            self.age_history_table();
        }
    }
    
    fn age_history_table(&mut self) {
        // Age all history values to prevent overflow
        for from_idx in 0..81 {
            for to_idx in 0..81 {
                self.history_table[from_idx][to_idx] /= 2;
            }
        }
    }
}
```

### 5. Static Exchange Evaluation (SEE)

**Purpose**: Evaluate captures based on material gain/loss.

**Technical Details**:
- Calculate material exchange value for captures
- Prioritize captures with positive material gain
- Cache SEE results for performance
- Handle complex capture sequences

**Implementation**:
```rust
impl MoveOrdering {
    fn score_see_move(&mut self, mv: Move, board: &BitboardBoard) -> i32 {
        if !mv.is_capture() {
            return 0;
        }
        
        // Check cache first
        if let Some(&see_value) = self.see_cache.get(&mv) {
            return see_value;
        }
        
        // Calculate SEE value
        let see_value = self.calculate_see(mv, board);
        
        // Cache result
        self.see_cache.insert(mv, see_value);
        
        // Scale SEE value for move ordering
        (see_value * 100).clamp(-1000, 1000)
    }
    
    fn calculate_see(&self, mv: Move, board: &BitboardBoard) -> i32 {
        let mut board_copy = board.clone();
        let mut attackers = Vec::new();
        let mut defenders = Vec::new();
        
        // Find all attackers and defenders of the target square
        self.find_attackers_defenders(mv.to, &board_copy, &mut attackers, &mut defenders);
        
        // Simulate the exchange
        let mut material_gain = 0;
        let mut is_attacker_turn = true;
        
        while !attackers.is_empty() || !defenders.is_empty() {
            let piece_to_move = if is_attacker_turn {
                attackers.pop()
            } else {
                defenders.pop()
            };
            
            if let Some(piece) = piece_to_move {
                material_gain += piece.value();
                is_attacker_turn = !is_attacker_turn;
            } else {
                break;
            }
        }
        
        material_gain
    }
    
    fn find_attackers_defenders(&self, square: u8, board: &BitboardBoard,
                               attackers: &mut Vec<PieceType>,
                               defenders: &mut Vec<PieceType>) {
        // Implementation depends on piece attack generation
        // This is a simplified version
        for piece_type in 0..14 {
            if board.has_piece(piece_type, square as usize) {
                if board.piece_owner(piece_type, square as usize) == board.side_to_move() {
                    attackers.push(piece_type);
                } else {
                    defenders.push(piece_type);
                }
            }
        }
    }
}
```

### 6. Move Scoring Integration

**Purpose**: Combine all heuristics into a single move score.

**Technical Details**:
- Weight different heuristics appropriately
- Handle move type-specific scoring
- Ensure consistent scoring across positions
- Optimize scoring performance

**Implementation**:
```rust
impl MoveOrdering {
    fn score_move(&mut self, mv: Move, board: &BitboardBoard, 
                 tt: &TranspositionTable, depth: u8) -> i32 {
        let mut score = 0;
        
        // PV move (highest priority)
        score += self.score_pv_move(mv, board, tt);
        
        // Killer moves (high priority)
        score += self.score_killer_move(mv, depth);
        
        // SEE for captures (medium priority)
        if mv.is_capture() {
            score += self.score_see_move(mv, board);
        }
        
        // History heuristic (low priority)
        score += self.score_history_move(mv);
        
        // Piece value (fallback)
        score += self.score_piece_value(mv);
        
        // Move type bonuses
        score += self.score_move_type(mv, board);
        
        score
    }
    
    fn score_piece_value(&self, mv: Move) -> i32 {
        mv.piece_type.value() as i32
    }
    
    fn score_move_type(&self, mv: Move, board: &BitboardBoard) -> i32 {
        let mut bonus = 0;
        
        // Promotion bonus
        if mv.is_promotion() {
            bonus += 500;
        }
        
        // Center control bonus
        if self.is_center_square(mv.to) {
            bonus += 100;
        }
        
        // Development bonus
        if self.is_development_move(mv, board) {
            bonus += 200;
        }
        
        bonus
    }
    
    fn is_center_square(&self, square: u8) -> bool {
        let file = square % 9;
        let rank = square / 9;
        
        // Center squares in Shogi
        (file >= 3 && file <= 5) && (rank >= 3 && rank <= 5)
    }
    
    fn is_development_move(&self, mv: Move, board: &BitboardBoard) -> bool {
        // Check if move develops a piece from back rank
        let from_rank = mv.from / 9;
        let to_rank = mv.to / 9;
        
        // Moving from back rank to more central position
        from_rank == 0 && to_rank > 0
    }
}
```

### 7. Performance Optimization

**Purpose**: Ensure move ordering doesn't become a bottleneck.

**Technical Details**:
- Cache frequently used calculations
- Optimize hot paths in move ordering
- Use efficient data structures
- Minimize memory allocations

**Implementation**:
```rust
impl MoveOrdering {
    fn optimize_ordering(&mut self) {
        // Pre-allocate move scores vector
        self.move_scores.reserve(200); // Typical max moves
        
        // Clear SEE cache periodically
        if self.see_cache.len() > 10000 {
            self.see_cache.clear();
        }
        
        // Update statistics
        self.ordering_stats.update();
    }
    
    fn get_ordering_stats(&self) -> &OrderingStats {
        &self.ordering_stats
    }
}

pub struct OrderingStats {
    pub total_moves_ordered: u64,
    pub pv_move_hits: u64,
    pub killer_move_hits: u64,
    pub see_calculations: u64,
    pub history_updates: u64,
    pub ordering_time: std::time::Duration,
}

impl OrderingStats {
    pub fn new() -> Self {
        Self {
            total_moves_ordered: 0,
            pv_move_hits: 0,
            killer_move_hits: 0,
            see_calculations: 0,
            history_updates: 0,
            ordering_time: std::time::Duration::ZERO,
        }
    }
    
    pub fn update(&mut self) {
        // Update statistics based on recent operations
    }
    
    pub fn get_hit_rate(&self) -> f64 {
        if self.total_moves_ordered == 0 {
            return 0.0;
        }
        
        let hits = self.pv_move_hits + self.killer_move_hits;
        hits as f64 / self.total_moves_ordered as f64
    }
}
```

## Integration Points

### Search Algorithm Integration

```rust
impl SearchEngine {
    fn negamax_with_ordering(&mut self, board: &mut BitboardBoard, depth: u8,
                            alpha: i32, beta: i32) -> i32 {
        // Generate moves
        let mut moves = self.generate_moves(board);
        
        // Order moves using move ordering system
        self.move_ordering.order_moves(&mut moves, board, &self.transposition_table, depth);
        
        // Search moves in order
        let mut best_score = i32::MIN + 1;
        let mut best_move = None;
        
        for (i, mv) in moves.iter().enumerate() {
            board.make_move(*mv);
            let score = -self.negamax_with_ordering(board, depth - 1, -beta, -alpha);
            board.unmake_move(*mv);
            
            if score > best_score {
                best_score = score;
                best_move = Some(*mv);
                
                if score > alpha {
                    alpha = score;
                    if alpha >= beta {
                        // Beta cutoff - update killer move
                        self.move_ordering.add_killer_move(*mv, depth);
                        break;
                    }
                }
            }
            
            // Update history heuristic
            self.move_ordering.update_history(*mv, depth, 
                if score > alpha { 1 } else { -1 });
        }
        
        // Update PV move
        if let Some(best_move) = best_move {
            self.move_ordering.update_pv_move(best_move);
        }
        
        best_score
    }
}
```

### Transposition Table Integration

```rust
impl MoveOrdering {
    fn integrate_with_transposition_table(&mut self, tt: &TranspositionTable, 
                                        board: &BitboardBoard) {
        let hash_key = tt.hash_position(board);
        
        if let Some(entry) = tt.probe(hash_key) {
            if let Some(best_move) = entry.best_move {
                self.update_pv_move(best_move);
            }
        }
    }
}
```

## Performance Considerations

### Memory Usage

- **History Table**: 81 × 81 × 4 bytes = ~26KB
- **Killer Moves**: 64 × 2 × 8 bytes = ~1KB
- **SEE Cache**: Variable, typically 1-10KB
- **Move Scores**: Temporary allocation, ~1KB per search

### Computational Complexity

- **Move Ordering**: O(n log n) where n is number of moves
- **SEE Calculation**: O(k) where k is number of pieces in exchange
- **History Updates**: O(1) per move
- **Killer Move Updates**: O(1) per cutoff

### Cache Efficiency

- **History Table**: Good spatial locality
- **Killer Moves**: Excellent cache efficiency
- **SEE Cache**: Moderate cache efficiency
- **Move Scores**: Temporary, good cache efficiency

## Testing Strategy

### Unit Tests

1. **Move Scoring**: Verify individual heuristic scores
2. **Move Ordering**: Test complete ordering system
3. **History Updates**: Verify history table updates
4. **Killer Moves**: Test killer move management
5. **SEE Calculation**: Verify static exchange evaluation

### Performance Tests

1. **Ordering Speed**: Measure move ordering overhead
2. **Search Improvement**: Measure search performance gains
3. **Memory Usage**: Monitor memory consumption
4. **Cache Efficiency**: Measure cache hit rates

### Integration Tests

1. **Search Integration**: Test with search algorithm
2. **Transposition Table**: Test integration with TT
3. **Move Generation**: Test with move generation
4. **Endgame Performance**: Test in endgame positions

## Configuration Options

### Heuristic Weights

```rust
pub struct OrderingWeights {
    pub pv_move_weight: i32,
    pub killer_move_weight: i32,
    pub see_weight: i32,
    pub history_weight: i32,
    pub piece_value_weight: i32,
}

impl Default for OrderingWeights {
    fn default() -> Self {
        Self {
            pv_move_weight: 10000,
            killer_move_weight: 9000,
            see_weight: 100,
            history_weight: 1,
            piece_value_weight: 10,
        }
    }
}
```

### Performance Tuning

- **History Aging**: Configurable aging rate
- **SEE Cache Size**: Configurable cache size
- **Killer Move Count**: Configurable number of killer moves per depth
- **Ordering Threshold**: Skip ordering for small move lists

## Expected Performance Impact

### Search Performance

- **15-25% Improvement**: In overall search speed
- **30-50% More Cutoffs**: From better move ordering
- **Deeper Search**: Same time budget reaches deeper
- **Better Evaluation**: More accurate position assessment

### Memory Usage

- **~30KB**: Additional memory for move ordering
- **Temporary Allocations**: Minimal impact on memory
- **Cache Efficiency**: Good utilization of CPU cache

### Hit Rate Targets

- **PV Move Hits**: 20-30% of positions
- **Killer Move Hits**: 10-20% of positions
- **History Hits**: 5-15% of positions
- **Overall Improvement**: 15-25% better ordering

## Future Enhancements

### Advanced Features

1. **Machine Learning**: Learn optimal move ordering from games
2. **Position-Specific Ordering**: Different strategies for different positions
3. **Dynamic Weights**: Adjust weights based on game phase
4. **Multi-Threading**: Parallel move ordering for large move lists

### Optimization Opportunities

1. **WASM Compatibility**: Ensure all optimizations work in WebAssembly
2. **Cross-Platform Performance**: Optimize for both native and WASM targets
3. **Predictive Ordering**: Predict move quality before full evaluation
4. **Adaptive Strategies**: Adjust strategies based on performance

### WASM-Specific Considerations

1. **Memory Management**: Use WASM-compatible memory allocation patterns
2. **Performance Portability**: Ensure optimizations work across platforms
3. **Size Optimization**: Minimize WASM binary size impact
4. **Runtime Compatibility**: Test thoroughly in browser environments

## Conclusion

The move ordering improvements design provides a comprehensive solution for optimizing move ordering in the Shogi engine. The implementation focuses on multiple complementary heuristics that work together to maximize alpha-beta cutoffs and improve search performance.

Key benefits include:
- Significant improvement in search efficiency
- Better utilization of alpha-beta pruning
- Adaptive learning from search results
- Comprehensive move evaluation
- Excellent performance characteristics

The design provides a solid foundation for future enhancements while delivering immediate performance improvements to the search algorithm.
