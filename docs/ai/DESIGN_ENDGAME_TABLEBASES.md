# Design Document: Endgame Tablebases

## 1. Overview

This document provides a detailed design for implementing endgame tablebases in the Shogi engine. The design focuses on creating a modular, extensible system that can handle multiple endgame scenarios while maintaining performance and memory efficiency for web-based deployment.

## 2. Architecture

### 2.1 Core Components

The endgame tablebase system consists of several key components:

```
src/tablebase/
├── mod.rs                 # Main module exports and configuration
├── micro_tablebase.rs    # Core tablebase implementation
├── endgame_solvers/      # Individual endgame solvers
│   ├── mod.rs
│   ├── king_gold_vs_king.rs
│   ├── king_silver_vs_king.rs
│   └── king_rook_vs_king.rs
├── position_cache.rs     # Position caching system
├── solver_traits.rs      # Common traits for endgame solvers
└── tablebase_config.rs   # Configuration management
```

### 2.2 Integration Points

The tablebase system integrates with the existing engine at these key points:

1. **Search Engine Integration**: `src/search.rs` - Probe tablebase before starting search
2. **Engine Integration**: `src/lib.rs` - Add tablebase to ShogiEngine struct
3. **Move Generation**: `src/moves.rs` - Use tablebase moves in move ordering
4. **Evaluation**: `src/evaluation.rs` - Use tablebase scores in evaluation

## 3. Detailed Design

### 3.1 Core Data Structures

#### 3.1.1 TablebaseResult

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablebaseResult {
    /// The optimal move for this position
    pub best_move: Option<Move>,
    /// Distance to mate (positive = winning, negative = losing, 0 = draw)
    pub distance_to_mate: Option<i32>,
    /// Position outcome
    pub outcome: TablebaseOutcome,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Number of moves to mate (if known)
    pub moves_to_mate: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TablebaseOutcome {
    Win,
    Loss,
    Draw,
    Unknown,
}
```

#### 3.1.2 EndgameSolver Trait

```rust
pub trait EndgameSolver: Send + Sync {
    /// Check if this solver can handle the given position
    fn can_solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> bool;
    
    /// Solve the position and return the optimal move
    fn solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<TablebaseResult>;
    
    /// Get the solver's priority (higher = more important)
    fn priority(&self) -> u8;
    
    /// Get the solver's name for debugging
    fn name(&self) -> &'static str;
}
```

#### 3.1.3 MicroTablebase

```rust
pub struct MicroTablebase {
    solvers: Vec<Box<dyn EndgameSolver>>,
    position_cache: PositionCache,
    config: TablebaseConfig,
    stats: TablebaseStats,
}

impl MicroTablebase {
    pub fn new() -> Self {
        let mut solvers: Vec<Box<dyn EndgameSolver>> = vec![
            Box::new(KingGoldVsKingSolver::new()),
            Box::new(KingSilverVsKingSolver::new()),
            Box::new(KingRookVsKingSolver::new()),
        ];
        
        // Sort by priority (highest first)
        solvers.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        
        Self {
            solvers,
            position_cache: PositionCache::new(),
            config: TablebaseConfig::default(),
            stats: TablebaseStats::new(),
        }
    }
    
    pub fn probe(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<TablebaseResult> {
        // Check cache first
        if let Some(cached_result) = self.position_cache.get(board, player, captured_pieces) {
            self.stats.cache_hits += 1;
            return Some(cached_result);
        }
        
        // Try each solver in priority order
        for solver in &self.solvers {
            if solver.can_solve(board, player, captured_pieces) {
                if let Some(result) = solver.solve(board, player, captured_pieces) {
                    self.stats.solver_hits += 1;
                    self.stats.solver_breakdown.insert(solver.name().to_string(), 
                        self.stats.solver_breakdown.get(solver.name()).unwrap_or(&0) + 1);
                    
                    // Cache the result
                    self.position_cache.put(board, player, captured_pieces, result);
                    return Some(result);
                }
            }
        }
        
        self.stats.misses += 1;
        None
    }
}
```

### 3.2 Endgame Solvers

#### 3.2.1 King + Gold vs King Solver

```rust
pub struct KingGoldVsKingSolver {
    config: KingGoldConfig,
}

impl EndgameSolver for KingGoldVsKingSolver {
    fn can_solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> bool {
        // Check if position has exactly 3 pieces and no captured pieces
        if board.count_all_pieces() != 3 || !captured_pieces.is_empty() {
            return false;
        }
        
        // Check if we have King + Gold vs King
        let pieces = self.extract_pieces(board);
        self.is_king_gold_vs_king(&pieces, player)
    }
    
    fn solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<TablebaseResult> {
        let pieces = self.extract_pieces(board);
        let (attacking_king, attacking_gold, defending_king) = self.identify_pieces(&pieces, player)?;
        
        // Calculate optimal move using known mating patterns
        let best_move = self.calculate_mating_move(board, attacking_king, attacking_gold, defending_king, player)?;
        
        // Calculate distance to mate
        let distance_to_mate = self.calculate_distance_to_mate(board, attacking_king, attacking_gold, defending_king, player);
        
        Some(TablebaseResult {
            best_move: Some(best_move),
            distance_to_mate: Some(distance_to_mate),
            outcome: if distance_to_mate > 0 { TablebaseOutcome::Win } else { TablebaseOutcome::Loss },
            confidence: 1.0,
            moves_to_mate: Some(distance_to_mate.abs() as u8),
        })
    }
    
    fn priority(&self) -> u8 { 100 }
    fn name(&self) -> &'static str { "King+Gold vs King" }
}

impl KingGoldVsKingSolver {
    fn calculate_mating_move(&self, board: &BitboardBoard, attacking_king: Position, 
                           attacking_gold: Position, defending_king: Position, player: Player) -> Option<Move> {
        // Implement the standard King + Gold vs King mating algorithm
        // This involves:
        // 1. Moving the king to restrict the opponent's king
        // 2. Using the gold to deliver checkmate
        // 3. Following known mating patterns
        
        // Phase 1: Approach with the king
        if self.king_distance(attacking_king, defending_king) > 2 {
            return self.approach_with_king(board, attacking_king, defending_king, player);
        }
        
        // Phase 2: Coordinate king and gold for mate
        if self.king_distance(attacking_king, defending_king) <= 2 {
            return self.coordinate_king_gold_mate(board, attacking_king, attacking_gold, defending_king, player);
        }
        
        None
    }
    
    fn approach_with_king(&self, board: &BitboardBoard, king: Position, target: Position, player: Player) -> Option<Move> {
        // Calculate the best approach move for the king
        let direction = self.direction_to_target(king, target);
        let new_king_pos = Position::new(
            (king.row as i8 + direction.0).clamp(0, 8) as u8,
            (king.col as i8 + direction.1).clamp(0, 8) as u8,
        );
        
        // Check if the move is legal
        if self.is_legal_king_move(board, king, new_king_pos, player) {
            Some(Move::new_move(king, new_king_pos, PieceType::King, player, false, false, None, false, false))
        } else {
            None
        }
    }
    
    fn coordinate_king_gold_mate(&self, board: &BitboardBoard, king: Position, gold: Position, 
                                target: Position, player: Player) -> Option<Move> {
        // Implement the coordination algorithm for delivering mate
        // This is the most complex part and requires careful implementation
        // of the standard King + Gold vs King mating technique
        
        // Try to find a move that either:
        // 1. Delivers immediate checkmate
        // 2. Further restricts the opponent's king
        // 3. Improves the coordination between king and gold
        
        self.find_best_coordination_move(board, king, gold, target, player)
    }
}
```

#### 3.2.2 Position Cache

```rust
pub struct PositionCache {
    cache: HashMap<u64, TablebaseResult>,
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl PositionCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 10000, // Configurable
            hits: 0,
            misses: 0,
        }
    }
    
    pub fn get(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<TablebaseResult> {
        let key = self.generate_key(board, player, captured_pieces);
        self.cache.get(&key).copied()
    }
    
    pub fn put(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces, result: TablebaseResult) {
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }
        
        let key = self.generate_key(board, player, captured_pieces);
        self.cache.insert(key, result);
    }
    
    fn generate_key(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> u64 {
        // Generate a hash key for the position
        // This should be fast and collision-resistant
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        board.hash(&mut hasher);
        player.hash(&mut hasher);
        captured_pieces.hash(&mut hasher);
        hasher.finish()
    }
}
```

### 3.3 Integration with Search Engine

#### 3.3.1 Modified Search Flow

```rust
impl SearchEngine {
    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                          player: Player, depth: u8, time_limit_ms: u32, 
                          alpha: i32, beta: i32) -> Option<(Move, i32)> {
        // 1. Check tablebase first (highest priority)
        if let Some(tablebase_result) = self.tablebase.probe(board, player, captured_pieces) {
            if let Some(best_move) = tablebase_result.best_move {
                // Use tablebase move with high confidence
                let score = self.convert_tablebase_score(tablebase_result);
                return Some((best_move, score));
            }
        }
        
        // 2. Check opening book
        if let Some(book_move) = self.opening_book.get_best_move(&board.to_fen(player, captured_pieces)) {
            return Some((book_move, 0)); // Neutral score for book moves
        }
        
        // 3. Proceed with normal search
        self.negamax_with_context(board, captured_pieces, player, depth, alpha, beta, 
                                 &TimeSource::now(), time_limit_ms, &mut Vec::new(), 
                                 true, false, false, false)
    }
    
    fn convert_tablebase_score(&self, result: TablebaseResult) -> i32 {
        match result.outcome {
            TablebaseOutcome::Win => {
                if let Some(distance) = result.distance_to_mate {
                    // Closer to mate = higher score
                    10000 - distance
                } else {
                    10000
                }
            },
            TablebaseOutcome::Loss => {
                if let Some(distance) = result.distance_to_mate {
                    // Closer to mate = lower score
                    -10000 + distance
                } else {
                    -10000
                }
            },
            TablebaseOutcome::Draw => 0,
            TablebaseOutcome::Unknown => 0,
        }
    }
}
```

#### 3.3.2 Move Ordering Integration

```rust
impl SearchEngine {
    fn sort_moves(&self, moves: &[Move], board: &BitboardBoard, pv_move: Option<Move>) -> Vec<Move> {
        let mut scored_moves: Vec<(Move, i32)> = moves.iter().map(|m| (*m, 0)).collect();
        
        for (move_, score) in &mut scored_moves {
            // 1. Principal variation move (highest priority)
            if Some(*move_) == pv_move {
                *score += 10000;
            }
            
            // 2. Tablebase moves (very high priority)
            if let Some(tablebase_result) = self.tablebase.probe(board, move_.player, &CapturedPieces::new()) {
                if tablebase_result.best_move == Some(*move_) {
                    *score += 9000;
                }
            }
            
            // 3. Capture moves
            if move_.is_capture {
                *score += 1000 + move_.captured_piece.map(|p| p.value()).unwrap_or(0);
            }
            
            // 4. Check moves
            if move_.gives_check {
                *score += 500;
            }
            
            // 5. History heuristic
            *score += self.history_table[move_.from.map(|p| p.row as usize).unwrap_or(0)][move_.to.row as usize];
        }
        
        scored_moves.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
        scored_moves.into_iter().map(|(m, _)| m).collect()
    }
}
```

### 3.4 Configuration System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablebaseConfig {
    pub enabled: bool,
    pub cache_size: usize,
    pub max_depth: u8,
    pub confidence_threshold: f32,
    pub solvers: SolverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub king_gold_vs_king: KingGoldConfig,
    pub king_silver_vs_king: KingSilverConfig,
    pub king_rook_vs_king: KingRookConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingGoldConfig {
    pub enabled: bool,
    pub max_moves_to_mate: u8,
    pub use_pattern_matching: bool,
    pub pattern_cache_size: usize,
}

impl Default for TablebaseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size: 10000,
            max_depth: 20,
            confidence_threshold: 0.8,
            solvers: SolverConfig::default(),
        }
    }
}
```

### 3.5 Statistics and Monitoring

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablebaseStats {
    pub total_probes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub solver_hits: u64,
    pub misses: u64,
    pub solver_breakdown: HashMap<String, u64>,
    pub average_probe_time_ms: f64,
    pub total_probe_time_ms: u64,
}

impl TablebaseStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_probes as f64
        }
    }
    
    pub fn solver_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            self.solver_hits as f64 / self.total_probes as f64
        }
    }
}
```

## 4. Implementation Phases

### Phase 1: Core Infrastructure
1. Create the basic tablebase module structure
2. Implement the `EndgameSolver` trait
3. Create the `MicroTablebase` struct
4. Implement basic position caching

### Phase 2: King + Gold vs King Solver
1. Implement the `KingGoldVsKingSolver`
2. Create the mating algorithm for this endgame
3. Add comprehensive tests
4. Integrate with the search engine

### Phase 3: Additional Solvers
1. Implement `KingSilverVsKingSolver`
2. Implement `KingRookVsKingSolver`
3. Add more complex endgame patterns
4. Optimize performance

### Phase 4: Advanced Features
1. Add configuration system
2. Implement statistics and monitoring
3. Add performance optimizations
4. Create comprehensive documentation

## 5. Testing Strategy

### 5.1 Unit Tests
- Test each solver individually with known positions
- Verify correct move generation for each endgame type
- Test position caching functionality
- Validate configuration system

### 5.2 Integration Tests
- Test tablebase integration with search engine
- Verify move ordering improvements
- Test performance under various conditions
- Validate statistics collection

### 5.3 End-to-End Tests
- Play complete games to reach tablebase positions
- Verify perfect play in supported endgames
- Test with different difficulty levels
- Validate user experience improvements

## 6. Performance Considerations

### 6.1 Memory Usage
- Position cache limited to 10,000 entries by default
- Solvers use minimal memory for pattern matching
- Configurable cache sizes for different components

### 6.2 CPU Performance
- Tablebase probing should be very fast (< 1ms)
- Caching reduces repeated calculations
- Solvers optimized for common patterns

### 6.3 WASM Compatibility
- All data structures serializable for WASM
- Minimal external dependencies
- Efficient memory usage for web deployment

## 7. Future Extensions

### 7.1 Additional Endgames
- King + Bishop vs King
- King + Rook vs King + Pawn
- More complex multi-piece endgames

### 7.2 Advanced Features
- Endgame database generation
- Position learning from games
- Adaptive difficulty based on tablebase coverage

### 7.3 API Integration
- Remote tablebase server support
- Cloud-based endgame databases
- Real-time position analysis

## 8. Conclusion

This design provides a comprehensive framework for implementing endgame tablebases in the Shogi engine. The modular architecture allows for incremental implementation and easy extension, while the performance optimizations ensure the system remains suitable for web-based deployment.

The focus on the most common and important endgames (King + Gold vs King) provides immediate value, while the extensible design allows for future growth and enhancement.
