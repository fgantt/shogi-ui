# Detailed Design: Late Move Reductions (LMR)

## 1. Overview

Late Move Reductions (LMR) is a search optimization technique that reduces the search depth for moves that appear later in the move ordering. This design document provides a comprehensive implementation plan for integrating LMR into the existing Shogi engine search algorithm.

## 2. Core Concept

LMR is based on the heuristic that well-ordered move lists place the most promising moves first. Moves appearing later in the sorted list are statistically less likely to be the best move, so we can search them with reduced depth to save computation time. If a reduced-depth search shows a move is surprisingly good (beats alpha), we re-search it at full depth to ensure accuracy.

## 3. Integration Points

### 3.1 Primary Integration Location
- **File**: `src/search.rs`
- **Function**: `negamax` (lines 111-191)
- **Specific Location**: Inside the move iteration loop (lines 158-183)

### 3.2 Current Move Loop Structure
```rust
for move_ in sorted_moves {
    if self.should_stop(&start_time, time_limit_ms) { break; }
    let mut new_board = board.clone();
    let mut new_captured = captured_pieces.clone();

    if let Some(captured) = new_board.make_move(&move_) {
        new_captured.add_piece(captured.piece_type, player);
    }

    let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, &start_time, time_limit_ms, history, true);
    // ... rest of the loop
}
```

## 4. Detailed Implementation Design

### 4.1 LMR Configuration Structure

```rust
#[derive(Debug, Clone)]
pub struct LMRConfig {
    pub enabled: bool,
    pub min_depth: u8,           // Minimum depth to apply LMR
    pub min_move_index: u8,      // Minimum move index to consider for reduction
    pub base_reduction: u8,      // Base reduction amount
    pub max_reduction: u8,       // Maximum reduction allowed
    pub enable_dynamic_reduction: bool,  // Use dynamic vs static reduction
    pub enable_adaptive_reduction: bool, // Use position-based adaptation
    pub enable_extended_exemptions: bool, // Extended move exemption rules
}

impl Default for LMRConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        }
    }
}
```

### 4.2 LMR Statistics Structure

```rust
#[derive(Debug, Clone, Default)]
pub struct LMRStats {
    pub moves_considered: u64,      // Total moves considered for LMR
    pub reductions_applied: u64,    // Number of reductions applied
    pub researches_triggered: u64,  // Number of full-depth re-searches
    pub cutoffs_after_reduction: u64, // Cutoffs after reduced search
    pub cutoffs_after_research: u64,  // Cutoffs after full re-search
    pub total_depth_saved: u64,     // Total depth reduction applied
    pub average_reduction: f64,     // Average reduction applied
}

impl LMRStats {
    pub fn research_rate(&self) -> f64 {
        if self.reductions_applied == 0 { 0.0 }
        else { self.researches_triggered as f64 / self.reductions_applied as f64 }
    }
    
    pub fn efficiency(&self) -> f64 {
        if self.moves_considered == 0 { 0.0 }
        else { self.reductions_applied as f64 / self.moves_considered as f64 }
    }
}
```

### 4.3 Enhanced SearchEngine Structure

Add to the existing `SearchEngine` struct:

```rust
pub struct SearchEngine {
    // ... existing fields ...
    lmr_config: LMRConfig,
    lmr_stats: LMRStats,
}
```

### 4.4 Core LMR Logic Implementation

#### 4.4.1 Move Loop Modification

Replace the existing move loop in `negamax` with:

```rust
let mut move_index = 0;
for move_ in sorted_moves {
    if self.should_stop(&start_time, time_limit_ms) { break; }
    move_index += 1;
    
    let mut new_board = board.clone();
    let mut new_captured = captured_pieces.clone();

    if let Some(captured) = new_board.make_move(&move_) {
        new_captured.add_piece(captured.piece_type, player);
    }

    let score = self.search_move_with_lmr(
        &mut new_board, 
        &new_captured, 
        player, 
        depth, 
        alpha, 
        beta, 
        &start_time, 
        time_limit_ms, 
        history, 
        &move_, 
        move_index
    );

    // ... rest of the existing loop logic ...
}
```

#### 4.4.2 LMR Search Method

```rust
fn search_move_with_lmr(&mut self, 
                       board: &mut BitboardBoard, 
                       captured_pieces: &CapturedPieces, 
                       player: Player, 
                       depth: u8, 
                       alpha: i32, 
                       beta: i32, 
                       start_time: &TimeSource, 
                       time_limit_ms: u32, 
                       history: &mut Vec<String>, 
                       move_: &Move, 
                       move_index: usize) -> i32 {
    
    self.lmr_stats.moves_considered += 1;
    
    // Check if LMR should be applied
    if self.should_apply_lmr(move_, depth, move_index) {
        self.lmr_stats.reductions_applied += 1;
        
        // Calculate reduction amount
        let reduction = self.calculate_reduction(move_, depth, move_index);
        self.lmr_stats.total_depth_saved += reduction as u64;
        
        // Perform reduced-depth search with null window
        let reduced_depth = depth - 1 - reduction;
        let score = -self.negamax(
            board, 
            captured_pieces, 
            player.opposite(), 
            reduced_depth, 
            -alpha - 1, 
            -alpha, 
            start_time, 
            time_limit_ms, 
            history, 
            true
        );
        
        // Check if re-search is needed
        if score > alpha {
            self.lmr_stats.researches_triggered += 1;
            
            // Re-search at full depth
            let full_score = -self.negamax(
                board, 
                captured_pieces, 
                player.opposite(), 
                depth - 1, 
                -beta, 
                -alpha, 
                start_time, 
                time_limit_ms, 
                history, 
                true
            );
            
            if full_score >= beta {
                self.lmr_stats.cutoffs_after_research += 1;
            }
            
            return full_score;
        } else {
            if score >= beta {
                self.lmr_stats.cutoffs_after_reduction += 1;
            }
            return score;
        }
    } else {
        // No reduction - perform full-depth search
        -self.negamax(
            board, 
            captured_pieces, 
            player.opposite(), 
            depth - 1, 
            -beta, 
            -alpha, 
            start_time, 
            time_limit_ms, 
            history, 
            true
        )
    }
}
```

#### 4.4.3 LMR Decision Logic

```rust
fn should_apply_lmr(&self, move_: &Move, depth: u8, move_index: usize) -> bool {
    if !self.lmr_config.enabled {
        return false;
    }
    
    // Must meet minimum depth requirement
    if depth < self.lmr_config.min_depth {
        return false;
    }
    
    // Must be beyond minimum move index
    if move_index < self.lmr_config.min_move_index as usize {
        return false;
    }
    
    // Apply exemption rules
    if self.is_move_exempt_from_lmr(move_) {
        return false;
    }
    
    true
}

fn is_move_exempt_from_lmr(&self, move_: &Move) -> bool {
    // Basic exemptions
    if move_.is_capture || move_.is_promotion || move_.gives_check {
        return true;
    }
    
    if self.lmr_config.enable_extended_exemptions {
        // Extended exemptions
        if self.is_killer_move(move_) {
            return true;
        }
        
        if self.is_transposition_table_move(move_) {
            return true;
        }
        
        if self.is_escape_move(move_) {
            return true;
        }
    }
    
    false
}
```

#### 4.4.4 Reduction Calculation

```rust
fn calculate_reduction(&self, move_: &Move, depth: u8, move_index: usize) -> u8 {
    if !self.lmr_config.enable_dynamic_reduction {
        return self.lmr_config.base_reduction;
    }
    
    let mut reduction = self.lmr_config.base_reduction;
    
    // Dynamic reduction based on depth
    if depth >= 6 {
        reduction += 1;
    }
    if depth >= 10 {
        reduction += 1;
    }
    
    // Dynamic reduction based on move index
    if move_index >= 8 {
        reduction += 1;
    }
    if move_index >= 16 {
        reduction += 1;
    }
    
    // Adaptive reduction based on position characteristics
    if self.lmr_config.enable_adaptive_reduction {
        reduction = self.apply_adaptive_reduction(reduction, move_, depth);
    }
    
    // Ensure reduction doesn't exceed maximum
    reduction.min(self.lmr_config.max_reduction)
        .min(depth.saturating_sub(2)) // Don't reduce to zero or negative
}

fn apply_adaptive_reduction(&self, base_reduction: u8, move_: &Move, depth: u8) -> u8 {
    let mut reduction = base_reduction;
    
    // More conservative reduction in tactical positions
    if self.is_tactical_position() {
        reduction = reduction.saturating_sub(1);
    }
    
    // More aggressive reduction in quiet positions
    if self.is_quiet_position() {
        reduction += 1;
    }
    
    // Adjust based on move characteristics
    if self.is_center_move(move_) {
        reduction = reduction.saturating_sub(1);
    }
    
    reduction
}
```

### 4.5 Helper Methods

```rust
fn is_killer_move(&self, move_: &Move) -> bool {
    self.killer_moves.iter().any(|killer| {
        killer.as_ref().map_or(false, |k| self.moves_equal(move_, k))
    })
}

fn is_transposition_table_move(&self, move_: &Move) -> bool {
    // This would require storing the best move from TT lookup
    // Implementation depends on how TT best moves are tracked
    false // Placeholder
}

fn is_escape_move(&self, move_: &Move) -> bool {
    // Check if this move escapes from a threat
    // This would require threat detection logic
    false // Placeholder
}

fn is_tactical_position(&self) -> bool {
    // Determine if position has tactical characteristics
    // Could be based on piece count, material balance, etc.
    false // Placeholder
}

fn is_quiet_position(&self) -> bool {
    // Determine if position is quiet (few captures, checks)
    // Could be based on recent move history
    true // Placeholder
}

fn is_center_move(&self, move_: &Move) -> bool {
    let center_rows = 3..=5;
    let center_cols = 3..=5;
    center_rows.contains(&move_.to.row) && center_cols.contains(&move_.to.col)
}
```

### 4.6 Configuration Management

```rust
impl SearchEngine {
    pub fn update_lmr_config(&mut self, config: LMRConfig) -> Result<(), String> {
        config.validate()?;
        self.lmr_config = config;
        Ok(())
    }
    
    pub fn get_lmr_config(&self) -> &LMRConfig {
        &self.lmr_config
    }
    
    pub fn get_lmr_stats(&self) -> &LMRStats {
        &self.lmr_stats
    }
    
    pub fn reset_lmr_stats(&mut self) {
        self.lmr_stats = LMRStats::default();
    }
}

impl LMRConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("Minimum depth must be at least 1".to_string());
        }
        
        if self.base_reduction == 0 {
            return Err("Base reduction must be at least 1".to_string());
        }
        
        if self.max_reduction < self.base_reduction {
            return Err("Maximum reduction must be >= base reduction".to_string());
        }
        
        Ok(())
    }
}
```

## 5. Integration with Existing Features

### 5.1 Move Ordering Dependencies
LMR effectiveness is highly dependent on move ordering quality. The existing `sort_moves` method must be robust:
- Captures and promotions should be prioritized
- Killer moves should be highly ranked
- History heuristic should be well-tuned

### 5.2 Transposition Table Integration
- LMR should work seamlessly with existing TT
- Re-search results should be stored in TT
- TT hits should be exempt from LMR

### 5.3 Null Move Pruning Compatibility
- LMR and NMP can work together
- LMR should be applied after NMP checks
- Both techniques should be independently configurable

### 5.4 Quiescence Search Integration
- LMR is not applied in quiescence search
- Quiescence search handles tactical moves that LMR exempts
- Both techniques complement each other

## 6. Performance Considerations

### 6.1 Memory Overhead
- LMR config and stats: ~100 bytes per SearchEngine instance
- Minimal impact on existing memory usage

### 6.2 Computational Overhead
- Move exemption checks: O(1) per move
- Reduction calculation: O(1) per move
- Re-search overhead: Only when moves beat alpha

### 6.3 Expected Performance Gains
- 20-40% increase in nodes per second
- 1-2 ply deeper search in same time
- Improved tactical strength

## 7. Testing and Validation

### 7.1 Unit Tests
```rust
#[cfg(test)]
mod lmr_tests {
    use super::*;
    
    #[test]
    fn test_lmr_config_validation() {
        let mut config = LMRConfig::default();
        assert!(config.validate().is_ok());
        
        config.min_depth = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_move_exemption_rules() {
        let engine = SearchEngine::new(None, 16);
        
        let capture_move = Move { is_capture: true, ..Default::default() };
        assert!(engine.is_move_exempt_from_lmr(&capture_move));
        
        let quiet_move = Move { is_capture: false, is_promotion: false, gives_check: false, ..Default::default() };
        assert!(!engine.is_move_exempt_from_lmr(&quiet_move));
    }
    
    #[test]
    fn test_reduction_calculation() {
        let engine = SearchEngine::new(None, 16);
        
        let move_ = Move { ..Default::default() };
        let reduction = engine.calculate_reduction(&move_, 5, 6);
        assert!(reduction >= 1);
        assert!(reduction <= 3);
    }
}
```

### 7.2 Integration Tests
- Test LMR with various position types
- Verify re-search behavior
- Measure performance improvements
- Validate move ordering dependency

### 7.3 Performance Benchmarks
- Compare NPS with/without LMR
- Measure search depth improvements
- Test on tactical and positional positions
- Validate against regression test suite

## 8. Configuration Tuning

### 8.1 Default Parameters
- `min_depth: 3` - Start LMR at reasonable depth
- `min_move_index: 4` - Skip first few moves
- `base_reduction: 1` - Conservative starting point
- `max_reduction: 3` - Prevent over-reduction

### 8.2 Tuning Guidelines
- Monitor research rate (should be 10-30%)
- Adjust reduction based on position type
- Fine-tune exemption rules
- Balance speed vs accuracy

### 8.3 Position-Specific Tuning
- Tactical positions: More conservative
- Quiet positions: More aggressive
- Endgame: Consider piece count
- Opening: Consider development

## 9. Monitoring and Debugging

### 9.1 Statistics Tracking
- Track reduction effectiveness
- Monitor re-search frequency
- Measure depth savings
- Validate move ordering quality

### 9.2 Debug Logging
```rust
fn log_lmr_decision(&self, move_: &Move, depth: u8, move_index: usize, reduction: u8, researched: bool) {
    if self.debug_enabled {
        println!("LMR: move={:?}, depth={}, index={}, reduction={}, researched={}", 
                 move_, depth, move_index, reduction, researched);
    }
}
```

### 9.3 Performance Metrics
- Nodes per second improvement
- Average search depth increase
- Re-search rate
- Cutoff efficiency

## 10. Future Enhancements

### 10.1 Advanced Features
- History-based reduction adjustment
- Position-specific LMR parameters
- Machine learning-based tuning
- Dynamic exemption rules

### 10.2 Integration Opportunities
- Combine with other pruning techniques
- Adaptive time management
- Position evaluation integration
- Opening book coordination

## 11. Implementation Timeline

### Phase 1: Core Implementation (Week 1)
- Add LMR configuration structures
- Implement basic LMR logic
- Add statistics tracking
- Create unit tests

### Phase 2: Integration (Week 2)
- Integrate with existing search loop
- Add configuration management
- Implement helper methods
- Create integration tests

### Phase 3: Optimization (Week 3)
- Tune parameters
- Add adaptive features
- Performance optimization
- Comprehensive testing

### Phase 4: Validation (Week 4)
- Performance benchmarking
- Strength testing
- Regression testing
- Documentation updates

## 12. Risk Mitigation

### 12.1 Potential Issues
- **Move ordering dependency**: Ensure robust move ordering
- **Re-search overhead**: Monitor and tune parameters
- **Tactical weakening**: Careful exemption rules
- **Parameter sensitivity**: Extensive testing

### 12.2 Mitigation Strategies
- Conservative default parameters
- Extensive testing on tactical positions
- Gradual parameter tuning
- Fallback to non-LMR search if issues arise

## 13. Conclusion

This design provides a comprehensive implementation plan for Late Move Reductions in the Shogi engine. The modular approach allows for incremental implementation and testing, while the extensive configuration options enable fine-tuning for optimal performance. The design maintains compatibility with existing features while providing significant performance improvements through intelligent search reduction.
