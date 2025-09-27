# Design Document: Internal Iterative Deepening (IID)

## 1. Executive Summary

Internal Iterative Deepening (IID) is a search enhancement technique that improves move ordering by performing a shallow search before the main search at critical nodes. This document provides a comprehensive design for implementing IID in the shogi engine, including architecture, algorithms, performance considerations, and integration with existing search features.

## 2. Problem Statement

### Current State
The engine's alpha-beta search efficiency is limited by move ordering quality. While existing heuristics (history table, killer moves, MVV-LVA) provide reasonable ordering, they are not always optimal, especially in complex positions where tactical patterns are not immediately apparent.

### Goals
- Improve search efficiency by finding better first moves
- Reduce total nodes searched to reach the same depth
- Increase playing strength through deeper searches in the same time
- Maintain compatibility with existing search features (LMR, null move, aspiration windows)

## 3. Technical Architecture

### 3.1 Core Concept

IID works by performing a shallow, fast search at a node before the main search begins. The best move found in this preliminary search is promoted to be searched first in the main search, improving alpha-beta pruning effectiveness.

```
Node at depth N
├── IID Search (depth 2-3)
│   ├── Find best move quickly
│   └── Promote to first position
└── Main Search (depth N)
    ├── Search IID move first (high probability of cutoff)
    └── Search remaining moves with standard ordering
```

### 3.2 Integration Points

IID integrates with the existing search infrastructure at these key points:

1. **negamax_with_context**: Main entry point for IID logic
2. **sort_moves**: Modified to prioritize IID results
3. **Transposition Table**: IID skips when TT provides good moves
4. **Statistics System**: Tracks IID effectiveness

## 4. Detailed Design

### 4.1 Data Structures

#### IID Configuration
```rust
#[derive(Debug, Clone)]
pub struct IIDConfig {
    pub enabled: bool,
    pub min_depth: u8,           // Minimum depth to apply IID
    pub iid_depth_ply: u8,       // Fixed depth for IID search
    pub max_legal_moves: usize,  // Only apply IID if move count below threshold
    pub time_overhead_threshold: f64, // Maximum time overhead allowed
}
```

#### IID Statistics
```rust
#[derive(Debug, Default, Clone)]
pub struct IIDStats {
    pub iid_searches_performed: u64,
    pub iid_move_first_improved_alpha: u64,
    pub iid_move_caused_cutoff: u64,
    pub total_iid_nodes: u64,
    pub iid_time_ms: u64,
    pub positions_skipped_tt_move: u64,
    pub positions_skipped_depth: u64,
    pub positions_skipped_move_count: u64,
}
```

#### IID Performance Metrics
```rust
#[derive(Debug, Clone)]
pub struct IIDPerformanceMetrics {
    pub iid_efficiency: f64,           // Alpha improvements per IID search
    pub cutoff_rate: f64,              // Percentage of IID moves causing cutoffs
    pub overhead_percentage: f64,      // Time overhead vs total search time
    pub nodes_saved_per_iid: f64,      // Average nodes saved per IID search
}
```

### 4.2 Algorithm Design

#### 4.2.1 IID Trigger Conditions

IID is applied when ALL of the following conditions are met:

```rust
fn should_apply_iid(&self, depth: u8, tt_move: Option<&Move>, legal_moves: &[Move]) -> bool {
    // 1. IID must be enabled
    if !self.iid_config.enabled { return false; }
    
    // 2. Sufficient depth for IID to be meaningful
    if depth < self.iid_config.min_depth { return false; }
    
    // 3. No transposition table move available
    if tt_move.is_some() { return false; }
    
    // 4. Reasonable number of legal moves (avoid IID in tactical positions)
    if legal_moves.len() > self.iid_config.max_legal_moves { return false; }
    
    // 5. Not in quiescence search
    if depth == 0 { return false; }
    
    // 6. Not in time pressure (optional condition)
    if self.is_time_pressure() { return false; }
    
    true
}
```

#### 4.2.2 IID Depth Selection

The IID search depth is calculated using multiple strategies:

```rust
fn calculate_iid_depth(&self, main_depth: u8) -> u8 {
    match self.iid_config.strategy {
        IIDDepthStrategy::Fixed => self.iid_config.iid_depth_ply,
        IIDDepthStrategy::Relative => {
            // Use depth - 2, but ensure minimum of 2
            std::cmp::max(2, main_depth.saturating_sub(2))
        },
        IIDDepthStrategy::Adaptive => {
            // Adjust based on position complexity and time remaining
            let base_depth = if main_depth > 6 { 3 } else { 2 };
            if self.has_complex_tactics() { base_depth + 1 } else { base_depth }
        }
    }
}
```

#### 4.2.3 IID Search Implementation

```rust
fn perform_iid_search(&mut self, 
                     board: &mut BitboardBoard, 
                     captured_pieces: &CapturedPieces, 
                     player: Player, 
                     iid_depth: u8, 
                     alpha: i32, 
                     beta: i32, 
                     start_time: &TimeSource, 
                     time_limit_ms: u32, 
                     history: &mut Vec<String>) -> Option<Move> {
    
    let iid_start_time = std::time::Instant::now();
    
    // Perform shallow search with null window for efficiency
    let iid_score = self.negamax_with_context(
        board, 
        captured_pieces, 
        player, 
        iid_depth, 
        alpha - 1,  // Null window
        alpha, 
        start_time, 
        time_limit_ms, 
        history, 
        true,  // can_null_move
        false, // is_root
        false, // has_capture
        false  // has_check
    );
    
    // Record IID statistics
    let iid_time = iid_start_time.elapsed().as_millis() as u64;
    self.iid_stats.iid_time_ms += iid_time;
    self.iid_stats.total_iid_nodes += self.get_nodes_since_marker();
    
    // Only return move if IID found something promising
    if iid_score > alpha {
        // Extract the best move from transposition table
        let fen_key = board.to_fen(player, captured_pieces);
        if let Some(entry) = self.transposition_table.get(&fen_key) {
            return entry.best_move.clone();
        }
    }
    
    None
}
```

### 4.3 Move Ordering Integration

#### 4.3.1 Enhanced Sort Moves Function

```rust
fn sort_moves(&self, moves: &[Move], board: &BitboardBoard, iid_move: Option<&Move>) -> Vec<Move> {
    let mut scored_moves: Vec<(Move, i32)> = moves.iter().map(|m| {
        let score = self.score_move(m, board, iid_move);
        (m.clone(), score)
    }).collect();
    
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
    scored_moves.into_iter().map(|(m, _)| m).collect()
}

fn score_move(&self, move_: &Move, board: &BitboardBoard, iid_move: Option<&Move>) -> i32 {
    // Priority 1: IID move gets maximum score
    if let Some(iid_mv) = iid_move {
        if self.moves_equal(move_, iid_mv) {
            return i32::MAX;
        }
    }
    
    // Priority 2: Transposition table move
    if let Some(tt_move) = self.get_tt_move(board) {
        if self.moves_equal(move_, tt_move) {
            return 1_000_000;
        }
    }
    
    // Priority 3: Standard move scoring (existing logic)
    let mut score = 0;
    
    // Captures with MVV-LVA
    if move_.is_capture {
        if let Some(captured_piece) = &move_.captured_piece {
            score += captured_piece.piece_type.base_value() * 10;
        }
        score += 1000;
    }
    
    // Promotions
    if move_.is_promotion { score += 800; }
    
    // Killer moves
    for killer in &self.killer_moves {
        if let Some(k) = killer {
            if self.moves_equal(move_, k) {
                score += 900;
                break;
            }
        }
    }
    
    // History heuristic
    if let Some(from) = move_.from {
        score += self.history_table[from.row as usize][from.col as usize];
    }
    
    score
}
```

### 4.4 Integration with Main Search

#### 4.4.1 Modified negamax_with_context

```rust
fn negamax_with_context(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool, is_root: bool, has_capture: bool, has_check: bool) -> i32 {
    // ... existing code (TT lookup, null move pruning, etc.) ...
    
    if depth == 0 {
        return self.quiescence_search(&mut board.clone(), captured_pieces, player, alpha, beta, &start_time, time_limit_ms, 5);
    }
    
    let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
    if legal_moves.is_empty() {
        return if board.is_king_in_check(player, captured_pieces) { -100000 } else { 0 };
    }
    
    // === IID LOGIC ===
    let mut iid_move = None;
    if self.should_apply_iid(depth, tt_move, &legal_moves) {
        let iid_depth = self.calculate_iid_depth(depth);
        iid_move = self.perform_iid_search(
            &mut board.clone(), 
            captured_pieces, 
            player, 
            iid_depth, 
            alpha, 
            beta, 
            start_time, 
            time_limit_ms, 
            history
        );
        
        self.iid_stats.iid_searches_performed += 1;
    }
    // === END IID LOGIC ===
    
    let sorted_moves = self.sort_moves(&legal_moves, board, iid_move.as_ref());
    let mut best_score = -200000;
    let mut best_move_for_tt = None;
    
    // Track if IID move improved alpha
    let mut iid_move_improved_alpha = false;
    
    history.push(fen_key.clone());
    
    let mut move_index = 0;
    for move_ in sorted_moves {
        if self.should_stop(&start_time, time_limit_ms) { break; }
        move_index += 1;
        
        // ... existing move search logic ...
        
        if score > best_score {
            best_score = score;
            best_move_for_tt = Some(move_.clone());
            if score > alpha {
                alpha = score;
                
                // Track if this was the IID move
                if let Some(iid_mv) = &iid_move {
                    if self.moves_equal(&move_, iid_mv) && !iid_move_improved_alpha {
                        iid_move_improved_alpha = true;
                        self.iid_stats.iid_move_first_improved_alpha += 1;
                    }
                }
                
                // ... existing alpha improvement logic ...
            }
            if alpha >= beta { 
                // Track if IID move caused cutoff
                if let Some(iid_mv) = &iid_move {
                    if self.moves_equal(&move_, iid_mv) {
                        self.iid_stats.iid_move_caused_cutoff += 1;
                    }
                }
                break; 
            }
        }
    }
    
    // ... rest of existing function ...
}
```

## 5. Performance Considerations

### 5.1 Overhead Analysis

IID introduces computational overhead through:
- Additional shallow searches
- Memory allocation for cloned boards
- Statistics tracking

**Mitigation Strategies:**
1. **Selective Application**: Only apply IID when conditions suggest high benefit
2. **Efficient Depth**: Use minimal IID depth (2-3 plies)
3. **Null Window**: Use null window in IID search for efficiency
4. **Early Termination**: Stop IID if time pressure detected

### 5.2 Memory Management

```rust
// Reuse board clones where possible
fn perform_iid_search(&mut self, ...) -> Option<Move> {
    // Create single board clone for IID
    let mut iid_board = board.clone();
    let mut iid_captured = captured_pieces.clone();
    
    // Perform IID search
    let score = self.negamax_with_context(
        &mut iid_board, 
        &iid_captured, 
        player, 
        iid_depth, 
        alpha - 1, 
        alpha, 
        start_time, 
        time_limit_ms, 
        history, 
        true, false, false, false
    );
    
    // Extract best move from TT
    self.extract_best_move_from_tt(&iid_board, player, &iid_captured)
}
```

### 5.3 Time Management

```rust
fn is_time_pressure(&self, start_time: &TimeSource, time_limit_ms: u32) -> bool {
    let elapsed = start_time.elapsed().as_millis() as u32;
    let remaining = time_limit_ms.saturating_sub(elapsed);
    remaining < time_limit_ms / 10 // Less than 10% time remaining
}
```

## 6. Configuration and Tuning

### 6.1 Default Configuration

```rust
impl Default for IIDConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 4,           // Apply IID at depth 4+
            iid_depth_ply: 2,       // 2-ply IID search
            max_legal_moves: 35,    // Skip IID in tactical positions
            time_overhead_threshold: 0.15, // Max 15% time overhead
        }
    }
}
```

### 6.2 Tuning Parameters

Key parameters for optimization:
- `min_depth`: Balance between overhead and benefit
- `iid_depth_ply`: Deeper = more accurate but slower
- `max_legal_moves`: Threshold for tactical vs positional positions
- `time_overhead_threshold`: Maximum acceptable overhead

### 6.3 Adaptive Tuning

```rust
fn adapt_iid_config(&mut self, performance_metrics: &IIDPerformanceMetrics) {
    if performance_metrics.overhead_percentage > self.iid_config.time_overhead_threshold {
        // Reduce IID frequency if overhead too high
        self.iid_config.max_legal_moves = std::cmp::max(20, self.iid_config.max_legal_moves - 5);
    }
    
    if performance_metrics.iid_efficiency < 0.3 {
        // Increase IID depth if efficiency low
        self.iid_config.iid_depth_ply = std::cmp::min(4, self.iid_config.iid_depth_ply + 1);
    }
}
```

## 7. Testing and Validation

### 7.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_iid_trigger_conditions() {
        let mut engine = SearchEngine::new(None, 64);
        
        // Test various conditions for IID application
        assert!(engine.should_apply_iid(4, None, &generate_test_moves(10)));
        assert!(!engine.should_apply_iid(3, None, &generate_test_moves(10))); // Too shallow
        assert!(!engine.should_apply_iid(4, Some(&test_move()), &generate_test_moves(10))); // TT move exists
        assert!(!engine.should_apply_iid(4, None, &generate_test_moves(40))); // Too many moves
    }
    
    #[test]
    fn test_iid_move_prioritization() {
        let mut engine = SearchEngine::new(None, 64);
        let moves = generate_test_moves(10);
        let iid_move = moves[5].clone();
        
        let sorted = engine.sort_moves(&moves, &test_board(), Some(&iid_move));
        assert_eq!(sorted[0], iid_move); // IID move should be first
    }
}
```

### 7.2 Performance Benchmarks

```rust
fn benchmark_iid_effectiveness() {
    let test_positions = load_standard_test_suite();
    
    // Test with IID disabled
    let mut engine_no_iid = SearchEngine::new(None, 64);
    engine_no_iid.iid_config.enabled = false;
    
    // Test with IID enabled
    let mut engine_with_iid = SearchEngine::new(None, 64);
    engine_with_iid.iid_config.enabled = true;
    
    let mut nodes_without_iid = 0;
    let mut nodes_with_iid = 0;
    
    for position in &test_positions {
        // Search with IID disabled
        engine_no_iid.reset_stats();
        engine_no_iid.search(position, 6, 5000);
        nodes_without_iid += engine_no_iid.get_nodes_searched();
        
        // Search with IID enabled
        engine_with_iid.reset_stats();
        engine_with_iid.search(position, 6, 5000);
        nodes_with_iid += engine_with_iid.get_nodes_searched();
    }
    
    let efficiency_gain = (nodes_without_iid as f64 - nodes_with_iid as f64) / nodes_without_iid as f64;
    assert!(efficiency_gain > 0.05, "IID should provide at least 5% efficiency gain");
}
```

### 7.3 Strength Testing

```rust
fn test_playing_strength_improvement() {
    let mut engine_v1 = create_engine_v1(); // Without IID
    let mut engine_v2 = create_engine_v2(); // With IID
    
    let match_result = play_match(&mut engine_v1, &mut engine_v2, 1000, 3.0);
    
    // Engine with IID should show measurable improvement
    assert!(match_result.engine2_score > match_result.engine1_score + 50,
           "IID should provide measurable strength improvement");
}
```

## 8. Monitoring and Debugging

### 8.1 Statistics Tracking

```rust
impl SearchEngine {
    pub fn get_iid_stats(&self) -> &IIDStats {
        &self.iid_stats
    }
    
    pub fn get_iid_performance_metrics(&self) -> IIDPerformanceMetrics {
        let stats = &self.iid_stats;
        
        IIDPerformanceMetrics {
            iid_efficiency: if stats.iid_searches_performed > 0 {
                stats.iid_move_first_improved_alpha as f64 / stats.iid_searches_performed as f64
            } else { 0.0 },
            
            cutoff_rate: if stats.iid_searches_performed > 0 {
                stats.iid_move_caused_cutoff as f64 / stats.iid_searches_performed as f64
            } else { 0.0 },
            
            overhead_percentage: if self.total_search_time_ms > 0 {
                stats.iid_time_ms as f64 / self.total_search_time_ms as f64 * 100.0
            } else { 0.0 },
            
            nodes_saved_per_iid: if stats.iid_searches_performed > 0 {
                self.calculate_nodes_saved() as f64 / stats.iid_searches_performed as f64
            } else { 0.0 },
        }
    }
}
```

### 8.2 Debug Logging

```rust
fn log_iid_decision(&self, depth: u8, tt_move: Option<&Move>, move_count: usize, applied: bool) {
    if self.debug_mode {
        println!("IID Decision: depth={}, tt_move={}, moves={}, applied={}", 
                depth, 
                tt_move.is_some(), 
                move_count, 
                applied);
    }
}
```

## 9. Integration with Existing Features

### 9.1 Transposition Table Integration

IID works synergistically with the transposition table:
- IID is skipped when TT provides a good move
- IID results are stored in TT for future use
- TT helps extract the best move from IID search

### 9.2 Late Move Reduction Compatibility

IID and LMR work together:
- IID finds the best move to search first
- LMR reduces depth for later moves
- Combined effect: better first move + reduced search of others

### 9.3 Aspiration Window Integration

IID can be used within aspiration windows:
- Apply IID at root level to find initial best move
- Use IID result to set initial aspiration bounds
- Re-apply IID if aspiration window fails

## 10. Future Enhancements

### 10.1 Multi-PV IID

Extend IID to find multiple principal variations:
```rust
fn perform_multi_pv_iid(&mut self, ...) -> Vec<Move> {
    // Find top 3 moves from IID search
    // Use all for move ordering, not just the best
}
```

### 10.2 Dynamic IID Depth

Adjust IID depth based on position characteristics:
```rust
fn calculate_dynamic_iid_depth(&self, position_complexity: f64) -> u8 {
    let base_depth = 2;
    if position_complexity > 0.8 { base_depth + 1 } else { base_depth }
}
```

### 10.3 IID with Probing

Use IID results to probe deeper into promising lines:
```rust
fn probe_iid_result(&mut self, iid_move: &Move, ...) -> i32 {
    // Perform deeper search on IID move to verify
}
```

## 11. Conclusion

Internal Iterative Deepening represents a significant enhancement to the search engine's efficiency. The design presented here provides:

- **Clear Integration**: Seamless integration with existing search features
- **Performance Focus**: Minimal overhead with maximum benefit
- **Comprehensive Monitoring**: Detailed statistics and performance metrics
- **Flexible Configuration**: Tunable parameters for optimization
- **Robust Testing**: Comprehensive validation framework

The implementation should provide measurable improvements in both search efficiency and playing strength, making it a valuable addition to the engine's search capabilities.
