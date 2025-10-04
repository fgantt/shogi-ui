# Advanced Alpha-Beta Pruning Design Document

## Overview

This document outlines the architectural design for implementing advanced alpha-beta pruning techniques in the Shogi engine. The design focuses on modularity, performance, and maintainability while providing significant search tree reduction capabilities.

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Search Engine                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Negamax       │  │  Pruning        │  │  Move       │ │
│  │   Search        │◄─┤  Manager        │◄─┤  Ordering   │ │
│  │                 │  │                 │  │             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│           │                     │                   │       │
│           ▼                     ▼                   ▼       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │  Transposition  │  │  Pruning        │  │  Move       │ │
│  │  Table          │  │  Techniques     │  │  Generation │ │
│  │                 │  │                 │  │             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Component Relationships

1. **Search Engine**: Main orchestrator for the search process
2. **Pruning Manager**: Coordinates different pruning techniques
3. **Pruning Techniques**: Individual pruning implementations
4. **Move Ordering**: Integrates with pruning decisions
5. **Transposition Table**: Works with pruning for efficiency

## Core Components

### 1. Search State Management

#### SearchState Structure
```rust
#[derive(Debug, Clone)]
pub struct SearchState {
    pub depth: u8,
    pub move_number: u8,
    pub alpha: i32,
    pub beta: i32,
    pub is_in_check: bool,
    pub static_eval: i32,
    pub best_move: Option<Move>,
    pub position_hash: u64,
    pub game_phase: GamePhase,
}

impl SearchState {
    pub fn new(depth: u8, alpha: i32, beta: i32) -> Self {
        Self {
            depth,
            move_number: 0,
            alpha,
            beta,
            is_in_check: false,
            static_eval: 0,
            best_move: None,
            position_hash: 0,
            game_phase: GamePhase::Middlegame,
        }
    }
    
    pub fn update(&mut self, board: &BitboardBoard, engine: &SearchEngine) {
        self.is_in_check = engine.is_in_check(board);
        self.static_eval = engine.evaluate_position(board);
        self.position_hash = engine.get_position_hash(board);
        self.game_phase = engine.get_game_phase(board);
    }
}
```

#### Game Phase Enumeration
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
}

impl GamePhase {
    pub fn from_material_count(material: u32) -> Self {
        match material {
            0..=20 => GamePhase::Endgame,
            21..=35 => GamePhase::Middlegame,
            _ => GamePhase::Opening,
        }
    }
}
```

### 2. Pruning Parameters

#### PruningParameters Structure
```rust
#[derive(Debug, Clone)]
pub struct PruningParameters {
    // Futility pruning parameters
    pub futility_margin: [i32; 8],
    pub futility_depth_limit: u8,
    pub extended_futility_depth: u8,
    
    // Late move reduction parameters
    pub lmr_base_reduction: u8,
    pub lmr_move_threshold: u8,
    pub lmr_depth_threshold: u8,
    pub lmr_max_reduction: u8,
    
    // Delta pruning parameters
    pub delta_margin: i32,
    pub delta_depth_limit: u8,
    
    // Razoring parameters
    pub razoring_depth_limit: u8,
    pub razoring_margin: i32,
    pub razoring_margin_endgame: i32,
    
    // Multi-cut pruning parameters
    pub multi_cut_threshold: u8,
    pub multi_cut_depth_limit: u8,
    
    // Adaptive parameters
    pub adaptive_enabled: bool,
    pub position_dependent_margins: bool,
}

impl Default for PruningParameters {
    fn default() -> Self {
        Self {
            futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
            futility_depth_limit: 3,
            extended_futility_depth: 5,
            lmr_base_reduction: 1,
            lmr_move_threshold: 3,
            lmr_depth_threshold: 2,
            lmr_max_reduction: 3,
            delta_margin: 200,
            delta_depth_limit: 4,
            razoring_depth_limit: 3,
            razoring_margin: 300,
            razoring_margin_endgame: 200,
            multi_cut_threshold: 3,
            multi_cut_depth_limit: 4,
            adaptive_enabled: false,
            position_dependent_margins: false,
        }
    }
}
```

### 3. Pruning Manager

#### PruningManager Structure
```rust
pub struct PruningManager {
    parameters: PruningParameters,
    statistics: PruningStatistics,
    adaptive_params: Option<AdaptiveParameters>,
}

impl PruningManager {
    pub fn new(parameters: PruningParameters) -> Self {
        Self {
            parameters,
            statistics: PruningStatistics::new(),
            adaptive_params: None,
        }
    }
    
    pub fn should_prune(&mut self, state: &SearchState, mv: &Move, board: &BitboardBoard) -> PruningDecision {
        let mut decision = PruningDecision::Search;
        
        // Apply pruning techniques in order of safety
        decision = self.check_futility_pruning(state, mv, board, decision);
        decision = self.check_delta_pruning(state, mv, board, decision);
        decision = self.check_razoring(state, board, decision);
        
        self.statistics.record_decision(decision);
        decision
    }
    
    pub fn calculate_lmr_reduction(&self, state: &SearchState, mv: &Move) -> u8 {
        if !self.should_apply_lmr(state, mv) {
            return 0;
        }
        
        let reduction = self.parameters.lmr_base_reduction +
                      (state.move_number / 8) as u8 +
                      (state.depth / 4) as u8;
        
        reduction.min(self.parameters.lmr_max_reduction).min(state.depth - 1)
    }
}
```

#### Pruning Decision Enumeration
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PruningDecision {
    Search,           // Search normally
    ReducedSearch,    // Search with reduced depth
    Skip,             // Skip this move
    Razor,            // Use razoring
}

impl PruningDecision {
    pub fn is_pruned(&self) -> bool {
        matches!(self, PruningDecision::Skip)
    }
    
    pub fn needs_reduction(&self) -> bool {
        matches!(self, PruningDecision::ReducedSearch)
    }
}
```

### 4. Individual Pruning Techniques

#### Futility Pruning
```rust
impl PruningManager {
    fn check_futility_pruning(&self, state: &SearchState, mv: &Move, board: &BitboardBoard, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.futility_depth_limit {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        if self.is_tactical_move(mv) {
            return current;
        }
        
        let margin = self.get_futility_margin(state);
        if state.static_eval + margin < state.alpha {
            return PruningDecision::Skip;
        }
        
        current
    }
    
    fn get_futility_margin(&self, state: &SearchState) -> i32 {
        let base_margin = self.parameters.futility_margin[state.depth as usize];
        
        if self.parameters.position_dependent_margins {
            match state.game_phase {
                GamePhase::Endgame => base_margin / 2,
                GamePhase::Opening => base_margin * 3 / 2,
                GamePhase::Middlegame => base_margin,
            }
        } else {
            base_margin
        }
    }
}
```

#### Delta Pruning
```rust
impl PruningManager {
    fn check_delta_pruning(&self, state: &SearchState, mv: &Move, board: &BitboardBoard, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.delta_depth_limit {
            return current;
        }
        
        if !self.is_capture_move(mv) {
            return current;
        }
        
        let material_gain = self.calculate_material_gain(mv, board);
        let margin = self.parameters.delta_margin;
        
        if state.static_eval + material_gain + margin < state.alpha {
            return PruningDecision::Skip;
        }
        
        current
    }
    
    fn calculate_material_gain(&self, mv: &Move, board: &BitboardBoard) -> i32 {
        let captured_piece = self.get_captured_piece(mv, board);
        let moving_piece = self.get_moving_piece(mv, board);
        
        self.get_piece_value(captured_piece) - self.get_piece_value(moving_piece)
    }
}
```

#### Razoring
```rust
impl PruningManager {
    fn check_razoring(&self, state: &SearchState, board: &BitboardBoard, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.razoring_depth_limit {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        let margin = self.get_razoring_margin(state);
        if state.static_eval + margin < state.alpha {
            return PruningDecision::Razor;
        }
        
        current
    }
    
    fn get_razoring_margin(&self, state: &SearchState) -> i32 {
        match state.game_phase {
            GamePhase::Endgame => self.parameters.razoring_margin_endgame,
            _ => self.parameters.razoring_margin,
        }
    }
}
```

### 5. Statistics and Monitoring

#### PruningStatistics Structure
```rust
#[derive(Debug, Default)]
pub struct PruningStatistics {
    pub total_moves: u64,
    pub pruned_moves: u64,
    pub futility_pruned: u64,
    pub delta_pruned: u64,
    pub razored: u64,
    pub lmr_applied: u64,
    pub re_searches: u64,
    pub multi_cuts: u64,
}

impl PruningStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_decision(&mut self, decision: PruningDecision) {
        self.total_moves += 1;
        
        match decision {
            PruningDecision::Skip => self.pruned_moves += 1,
            PruningDecision::Razor => self.razored += 1,
            _ => {}
        }
    }
    
    pub fn get_pruning_rate(&self) -> f64 {
        if self.total_moves == 0 {
            0.0
        } else {
            self.pruned_moves as f64 / self.total_moves as f64
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
```

### 6. Adaptive Parameters

#### AdaptiveParameters Structure
```rust
pub struct AdaptiveParameters {
    position_analysis: PositionAnalyzer,
    parameter_history: Vec<ParameterSnapshot>,
    learning_rate: f64,
}

impl AdaptiveParameters {
    pub fn new() -> Self {
        Self {
            position_analysis: PositionAnalyzer::new(),
            parameter_history: Vec::new(),
            learning_rate: 0.1,
        }
    }
    
    pub fn adjust_parameters(&mut self, position: &BitboardBoard, performance: &PerformanceMetrics) {
        let position_type = self.position_analysis.analyze(position);
        let adjustment = self.calculate_adjustment(position_type, performance);
        self.apply_adjustment(adjustment);
    }
    
    fn calculate_adjustment(&self, position_type: PositionType, performance: &PerformanceMetrics) -> ParameterAdjustment {
        // Machine learning-based parameter adjustment
        // This would be implemented with actual ML algorithms
        ParameterAdjustment::default()
    }
}
```

## Integration Points

### 1. Search Engine Integration

```rust
impl SearchEngine {
    pub fn negamax_with_pruning(&mut self, board: &mut BitboardBoard, depth: u8, alpha: i32, beta: i32, move_number: u8) -> i32 {
        let mut state = SearchState::new(depth, alpha, beta);
        state.update(board, self);
        
        // Terminal node check
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }
        
        // Generate and order moves
        let mut moves = self.generate_moves(board);
        self.order_moves(&mut moves, board, state.best_move);
        
        if moves.is_empty() {
            return if state.is_in_check { -MATE_SCORE } else { 0 };
        }
        
        let mut best_score = -INFINITY;
        let mut alpha = alpha;
        
        for (i, mv) in moves.iter().enumerate() {
            state.move_number = i as u8;
            
            // Check pruning decision
            let pruning_decision = self.pruning_manager.should_prune(&state, mv, board);
            
            if pruning_decision.is_pruned() {
                continue;
            }
            
            // Make move
            let undo_info = self.make_move(board, mv);
            
            // Calculate search depth
            let search_depth = if pruning_decision.needs_reduction() {
                depth - 1 - self.pruning_manager.calculate_lmr_reduction(&state, mv)
            } else {
                depth - 1
            };
            
            // Search
            let score = if pruning_decision == PruningDecision::Razor {
                self.razor_search(board, depth, alpha, beta)
            } else {
                -self.negamax_with_pruning(board, search_depth, -beta, -alpha, 0)
            };
            
            // Unmake move
            self.unmake_move(board, mv, undo_info);
            
            // Update best score
            if score > best_score {
                best_score = score;
                state.best_move = Some(*mv);
                
                if score > alpha {
                    alpha = score;
                    if score >= beta {
                        self.update_killer_moves(mv, depth);
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

### 2. Move Ordering Integration

```rust
impl SearchEngine {
    pub fn order_moves_with_pruning(&mut self, moves: &mut Vec<Move>, board: &BitboardBoard, pv_move: Option<Move>) {
        moves.sort_by(|a, b| {
            let score_a = self.get_move_score_with_pruning(*a, board, pv_move);
            let score_b = self.get_move_score_with_pruning(*b, board, pv_move);
            score_b.cmp(&score_a)
        });
    }
    
    fn get_move_score_with_pruning(&self, mv: Move, board: &BitboardBoard, pv_move: Option<Move>) -> i32 {
        let mut score = 0;
        
        // PV move gets highest priority
        if Some(mv) == pv_move {
            score += 10000;
        }
        
        // Captures get high priority
        if self.is_capture_move(mv) {
            score += 1000 + self.get_capture_value(mv, board);
        }
        
        // Promotions get high priority
        if self.is_promotion_move(mv) {
            score += 800;
        }
        
        // Killer moves get medium priority
        if self.is_killer_move(mv) {
            score += 500;
        }
        
        // History heuristic
        score += self.get_history_score(mv);
        
        score
    }
}
```

## Performance Considerations

### 1. Memory Layout

- Use cache-friendly data structures
- Align structures to cache line boundaries
- Minimize memory allocations during search

### 2. Branch Prediction

- Optimize for common pruning decisions
- Use likely/unlikely hints for compiler optimization
- Structure code to minimize branch mispredictions

### 3. Instruction-Level Optimization

- Use SIMD instructions where applicable
- Minimize function call overhead
- Optimize hot paths in pruning decisions

## Testing Strategy

### 1. Unit Testing

- Test each pruning technique individually
- Validate pruning decisions
- Test edge cases and boundary conditions

### 2. Integration Testing

- Test pruning techniques in combination
- Validate search correctness
- Test performance improvements

### 3. Performance Testing

- Benchmark search tree reduction
- Measure time improvements
- Validate memory usage

### 4. Position Testing

- Test on tactical positions
- Validate endgame handling
- Test complex positions

## Configuration Management

### 1. Parameter Files

```toml
[pruning]
futility_margin = [0, 100, 200, 300, 400, 500, 600, 700]
futility_depth_limit = 3
lmr_base_reduction = 1
lmr_move_threshold = 3
delta_margin = 200
razoring_depth_limit = 3
razoring_margin = 300
adaptive_enabled = false
```

### 2. Runtime Configuration

- Command-line parameter overrides
- USI protocol parameter setting
- Dynamic parameter adjustment

### 3. Parameter Validation

- Range checking for all parameters
- Consistency validation
- Performance impact assessment

## Error Handling

### 1. Pruning Errors

- Invalid pruning decisions
- Parameter validation failures
- Performance regression detection

### 2. Recovery Strategies

- Fallback to basic pruning
- Parameter reset mechanisms
- Error logging and reporting

### 3. Debugging Support

- Detailed logging of pruning decisions
- Performance profiling integration
- Search tree visualization

## Future Extensibility

### 1. Plugin Architecture

- Modular pruning technique loading
- Dynamic technique selection
- Custom pruning implementations

### 2. Machine Learning Integration

- Learned parameter optimization
- Position-dependent pruning
- Adaptive technique selection

### 3. Hardware Optimization

- GPU-accelerated pruning
- Specialized instruction usage
- Parallel pruning evaluation

## Conclusion

This design provides a comprehensive architecture for implementing advanced alpha-beta pruning techniques. The modular design ensures maintainability while the performance optimizations provide significant speedup. The adaptive capabilities allow for future enhancements and machine learning integration.

The key design principles are:

1. **Modularity**: Separate concerns into distinct components
2. **Performance**: Optimize for speed and memory efficiency
3. **Safety**: Ensure pruning decisions are correct
4. **Extensibility**: Allow for future enhancements
5. **Testability**: Comprehensive testing and validation

This architecture provides the foundation for implementing the advanced pruning techniques outlined in the implementation plan.
