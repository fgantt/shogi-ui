# Advanced Alpha-Beta Pruning Implementation Plan

## Overview

This document outlines the implementation plan for advanced alpha-beta pruning techniques in the Shogi engine, based on the analysis from the Optimization Strategies document. The goal is to implement sophisticated pruning techniques that can reduce search tree size by 30-50%.

## Current State Analysis

**Current Implementation**: Basic alpha-beta with null move pruning
**Target**: Advanced pruning with LMR, futility pruning, delta pruning, and razoring
**Expected Impact**: High - 30-50% reduction in search tree size

## Implementation Strategy

### Phase 1: Foundation and Infrastructure (Week 1)

#### 1.1 Search State Management
- Implement proper search state tracking
- Add depth and move number tracking
- Create pruning decision infrastructure

#### 1.2 Constants and Parameters
- Define pruning thresholds and margins
- Implement configurable pruning parameters
- Add tuning infrastructure for parameters

### Phase 2: Core Pruning Techniques (Week 2-3)

#### 2.1 Late Move Reduction (LMR)
- Implement LMR logic for non-capture moves
- Add move ordering integration
- Create reduction depth calculation

#### 2.2 Futility Pruning
- Implement static evaluation-based pruning
- Add check detection integration
- Create futility margin calculations

#### 2.3 Delta Pruning
- Implement capture value-based pruning
- Add material gain calculations
- Integrate with move generation

#### 2.4 Razoring
- Implement depth-based razoring
- Add quiet position detection
- Create razoring thresholds

### Phase 3: Integration and Optimization (Week 4)

#### 3.1 Move Ordering Integration
- Integrate with existing move ordering
- Add pruning-aware move sorting
- Optimize move generation order

#### 3.2 Performance Optimization
- Optimize pruning decision overhead
- Add conditional pruning logic
- Implement efficient check detection

#### 3.3 Testing and Validation
- Create comprehensive test suite
- Add performance benchmarks
- Validate pruning correctness

## Technical Implementation Details

### 1. Search State Structure

```rust
struct SearchState {
    depth: u8,
    move_number: u8,
    alpha: i32,
    beta: i32,
    is_in_check: bool,
    static_eval: i32,
    best_move: Option<Move>,
}

impl SearchState {
    fn new(depth: u8, alpha: i32, beta: i32) -> Self {
        Self {
            depth,
            move_number: 0,
            alpha,
            beta,
            is_in_check: false,
            static_eval: 0,
            best_move: None,
        }
    }
}
```

### 2. Pruning Parameters

```rust
struct PruningParameters {
    // Futility pruning
    futility_margin: [i32; 8], // Per depth
    futility_depth_limit: u8,
    
    // Late move reduction
    lmr_base_reduction: u8,
    lmr_move_threshold: u8,
    lmr_depth_threshold: u8,
    
    // Delta pruning
    delta_margin: i32,
    
    // Razoring
    razoring_depth_limit: u8,
    razoring_margin: i32,
}

impl Default for PruningParameters {
    fn default() -> Self {
        Self {
            futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
            futility_depth_limit: 3,
            lmr_base_reduction: 1,
            lmr_move_threshold: 3,
            lmr_depth_threshold: 2,
            delta_margin: 200,
            razoring_depth_limit: 3,
            razoring_margin: 300,
        }
    }
}
```

### 3. Advanced Negamax Implementation

```rust
impl SearchEngine {
    fn negamax_with_advanced_pruning(
        &mut self,
        board: &mut BitboardBoard,
        depth: u8,
        alpha: i32,
        beta: i32,
        move_number: u8,
    ) -> i32 {
        let mut state = SearchState::new(depth, alpha, beta);
        
        // Terminal node check
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }
        
        // Check for repetition/draw
        if self.is_repetition(board) {
            return 0;
        }
        
        // Update search state
        state.is_in_check = self.is_in_check(board);
        state.static_eval = self.evaluate_position(board);
        
        // Razoring
        if self.should_razor(&state) {
            return self.razor_search(board, depth, alpha, beta);
        }
        
        // Generate moves
        let mut moves = self.generate_moves(board);
        self.order_moves(&mut moves, board, state.best_move);
        
        if moves.is_empty() {
            return if state.is_in_check { -MATE_SCORE } else { 0 };
        }
        
        let mut best_score = -INFINITY;
        let mut alpha = alpha;
        
        for (i, mv) in moves.iter().enumerate() {
            state.move_number = i as u8;
            
            // Futility pruning
            if self.should_futility_prune(&state, mv) {
                continue;
            }
            
            // Delta pruning
            if self.should_delta_prune(&state, mv) {
                continue;
            }
            
            // Make move
            let undo_info = self.make_move(board, mv);
            
            // Late move reduction
            let reduction = self.calculate_lmr_reduction(&state, mv);
            let search_depth = depth - 1 - reduction;
            
            let score = if reduction > 0 {
                // Reduced depth search
                let reduced_score = -self.negamax_with_advanced_pruning(
                    board, search_depth, -alpha - 1, -alpha, 0
                );
                
                // Re-search with full depth if score is promising
                if reduced_score > alpha && reduced_score < beta {
                    -self.negamax_with_advanced_pruning(
                        board, depth - 1, -beta, -alpha, 0
                    )
                } else {
                    reduced_score
                }
            } else {
                -self.negamax_with_advanced_pruning(
                    board, depth - 1, -beta, -alpha, 0
                )
            };
            
            // Unmake move
            self.unmake_move(board, mv, undo_info);
            
            // Update best score and alpha
            if score > best_score {
                best_score = score;
                state.best_move = Some(*mv);
                
                if score > alpha {
                    alpha = score;
                    
                    // Beta cutoff
                    if score >= beta {
                        self.update_killer_moves(mv, depth);
                        self.update_history(mv, depth);
                        break;
                    }
                }
            }
        }
        
        // Store in transposition table
        self.store_transposition(board, depth, best_score, state.best_move);
        
        best_score
    }
}
```

### 4. Pruning Decision Functions

```rust
impl SearchEngine {
    fn should_razor(&self, state: &SearchState) -> bool {
        state.depth <= self.params.razoring_depth_limit &&
        !state.is_in_check &&
        state.static_eval + self.params.razoring_margin < state.alpha
    }
    
    fn should_futility_prune(&self, state: &SearchState, mv: &Move) -> bool {
        state.depth <= self.params.futility_depth_limit &&
        !state.is_in_check &&
        !self.is_capture(mv) &&
        !self.is_promotion(mv) &&
        state.static_eval + self.params.futility_margin[state.depth as usize] < state.alpha
    }
    
    fn should_delta_prune(&self, state: &SearchState, mv: &Move) -> bool {
        if !self.is_capture(mv) {
            return false;
        }
        
        let capture_value = self.get_piece_value(self.get_captured_piece(mv));
        let moving_piece_value = self.get_piece_value(self.get_moving_piece(mv));
        
        state.static_eval + capture_value - moving_piece_value + self.params.delta_margin < state.alpha
    }
    
    fn calculate_lmr_reduction(&self, state: &SearchState, mv: &Move) -> u8 {
        if state.move_number <= self.params.lmr_move_threshold ||
           state.depth <= self.params.lmr_depth_threshold ||
           self.is_capture(mv) ||
           self.is_promotion(mv) ||
           state.is_in_check {
            return 0;
        }
        
        let reduction = self.params.lmr_base_reduction + 
                       (state.move_number / 8) as u8 +
                       (state.depth / 4) as u8;
        
        reduction.min(state.depth - 1)
    }
}
```

### 5. Razoring Implementation

```rust
impl SearchEngine {
    fn razor_search(&mut self, board: &mut BitboardBoard, depth: u8, alpha: i32, beta: i32) -> i32 {
        let razor_depth = 1;
        let razor_score = -self.negamax_with_advanced_pruning(
            board, razor_depth, -beta, -alpha, 0
        );
        
        if razor_score >= beta {
            return beta;
        }
        
        // Re-search with full depth if razor score is promising
        if razor_score > alpha {
            -self.negamax_with_advanced_pruning(
                board, depth - 1, -beta, -alpha, 0
            )
        } else {
            razor_score
        }
    }
}
```

## Testing Strategy

### 1. Unit Tests
- Test each pruning technique individually
- Verify pruning decisions are correct
- Test edge cases and boundary conditions

### 2. Integration Tests
- Test pruning techniques in combination
- Verify search correctness with pruning
- Test performance improvements

### 3. Performance Benchmarks
- Measure search tree size reduction
- Compare search time improvements
- Validate pruning effectiveness

### 4. Position Testing
- Test on known positions
- Verify tactical sequences are preserved
- Test endgame positions

## Configuration and Tuning

### 1. Parameter Tuning
- Implement automated parameter tuning
- Create parameter validation
- Add runtime parameter adjustment

### 2. Performance Monitoring
- Add pruning statistics
- Monitor pruning effectiveness
- Track search performance metrics

### 3. Adaptive Parameters
- Implement position-dependent parameters
- Add dynamic parameter adjustment
- Create parameter learning system

## Implementation Timeline

### Week 1: Foundation
- [ ] Implement SearchState structure
- [ ] Create PruningParameters
- [ ] Add basic pruning infrastructure
- [ ] Set up testing framework

### Week 2: Core Pruning
- [ ] Implement Late Move Reduction
- [ ] Add Futility Pruning
- [ ] Create Delta Pruning
- [ ] Implement Razoring

### Week 3: Integration
- [ ] Integrate with existing search
- [ ] Add move ordering integration
- [ ] Implement performance optimizations
- [ ] Create comprehensive tests

### Week 4: Testing and Tuning
- [ ] Run performance benchmarks
- [ ] Tune pruning parameters
- [ ] Validate search correctness
- [ ] Document implementation

## Success Metrics

1. **Search Tree Reduction**: 30-50% reduction in nodes searched
2. **Performance Improvement**: 20-40% faster search times
3. **Correctness**: No tactical sequences lost
4. **Stability**: Consistent performance across positions

## Risk Mitigation

1. **Pruning Bugs**: Comprehensive testing and validation
2. **Performance Regression**: Careful benchmarking
3. **Tactical Loss**: Conservative pruning parameters
4. **Integration Issues**: Gradual implementation approach

## Future Enhancements

1. **Adaptive Pruning**: Position-dependent pruning decisions
2. **Machine Learning**: Learned pruning parameters
3. **Advanced Techniques**: Extended futility pruning, multi-cut pruning
4. **Parallel Integration**: Thread-safe pruning implementation

## Conclusion

This implementation plan provides a structured approach to implementing advanced alpha-beta pruning techniques. The phased approach ensures proper testing and validation while maintaining search correctness. The expected 30-50% reduction in search tree size will significantly improve the engine's performance and playing strength.
