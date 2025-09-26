# Design Document: Tapered Evaluation System

## 1. Overview

This document provides a detailed design for implementing a tapered evaluation system in the Shogi engine. The tapered evaluation allows the engine to use different evaluation weights based on the current game phase (opening, middlegame, endgame), leading to more nuanced and positionally aware strategic play.

## 2. Current State Analysis

### 2.1 Existing Evaluation Structure

The current evaluation system in `src/evaluation.rs` consists of:

- **PositionEvaluator**: Main evaluation struct with piece-square tables
- **Evaluation Components**:
  - Material and positional evaluation
  - Pawn structure analysis
  - King safety assessment
  - Mobility evaluation
  - Piece coordination
  - Center control
  - Development evaluation

- **Piece-Square Tables**: Separate tables for each piece type (pawn, lance, knight, silver, gold, bishop, rook)

### 2.2 Current Limitations

1. **Static Evaluation**: All evaluation terms use fixed weights regardless of game phase
2. **No Phase Awareness**: The engine doesn't distinguish between opening, middlegame, and endgame
3. **Suboptimal Strategic Play**: King safety is equally important in all phases, while pawn promotion potential is not weighted appropriately for endgames

## 3. Design Goals

### 3.1 Primary Objectives

1. **Phase-Aware Evaluation**: Implement dual scoring (middlegame/endgame) for all evaluation terms
2. **Smooth Transitions**: Use interpolation to smoothly transition between phases
3. **Maintainable Architecture**: Ensure the refactoring doesn't break existing functionality
4. **Performance Efficiency**: Minimize computational overhead of the new system

### 3.2 Success Criteria

- Engine demonstrates better strategic understanding in different game phases
- King safety evaluation is more prominent in middlegame than endgame
- Pawn promotion potential is more valued in endgame than opening
- No regression in tactical play or search performance

## 4. Core Architecture

### 4.1 TaperedScore Structure

```rust
/// Represents a dual-phase evaluation score
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaperedScore {
    /// Middlegame score (0-256 phase)
    pub mg: i32,
    /// Endgame score (0-256 phase)
    pub eg: i32,
}

impl TaperedScore {
    /// Create a new TaperedScore with both values equal
    pub fn new(value: i32) -> Self {
        Self { mg: value, eg: value }
    }
    
    /// Create a TaperedScore with different mg and eg values
    pub fn new_tapered(mg: i32, eg: i32) -> Self {
        Self { mg, eg }
    }
    
    /// Interpolate between mg and eg based on game phase
    pub fn interpolate(&self, phase: i32) -> i32 {
        (self.mg * phase + self.eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX
    }
}

impl std::ops::AddAssign for TaperedScore {
    fn add_assign(&mut self, other: Self) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

impl std::ops::Add for TaperedScore {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            mg: self.mg + other.mg,
            eg: self.eg + other.eg,
        }
    }
}

impl std::ops::Sub for TaperedScore {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            mg: self.mg - other.mg,
            eg: self.eg - other.eg,
        }
    }
}

impl std::ops::Neg for TaperedScore {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            mg: -self.mg,
            eg: -self.eg,
        }
    }
}
```

### 4.2 Game Phase Calculation

```rust
/// Maximum game phase value (opening)
const GAME_PHASE_MAX: i32 = 256;

/// Phase values for different piece types
const PIECE_PHASE_VALUES: [(PieceType, i32); 6] = [
    (PieceType::Knight, 1),
    (PieceType::Silver, 1),
    (PieceType::Gold, 2),
    (PieceType::Bishop, 2),
    (PieceType::Rook, 3),
    (PieceType::Lance, 1),
];

impl PositionEvaluator {
    /// Calculate the current game phase (0 = endgame, 256 = opening)
    fn calculate_game_phase(&self, board: &BitboardBoard) -> i32 {
        let mut phase = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if let Some(phase_value) = self.get_piece_phase_value(piece.piece_type) {
                        phase += phase_value;
                    }
                }
            }
        }
        
        // Scale to 0-256 range
        phase.min(GAME_PHASE_MAX)
    }
    
    /// Get phase value for a piece type
    fn get_piece_phase_value(&self, piece_type: PieceType) -> Option<i32> {
        PIECE_PHASE_VALUES
            .iter()
            .find(|(pt, _)| *pt == piece_type)
            .map(|(_, value)| *value)
    }
}
```

### 4.3 Updated Piece-Square Tables

```rust
/// Dual-phase piece-square tables
#[derive(Clone)]
struct PieceSquareTables {
    // Middlegame tables
    pawn_table_mg: [[i32; 9]; 9],
    lance_table_mg: [[i32; 9]; 9],
    knight_table_mg: [[i32; 9]; 9],
    silver_table_mg: [[i32; 9]; 9],
    gold_table_mg: [[i32; 9]; 9],
    bishop_table_mg: [[i32; 9]; 9],
    rook_table_mg: [[i32; 9]; 9],
    
    // Endgame tables
    pawn_table_eg: [[i32; 9]; 9],
    lance_table_eg: [[i32; 9]; 9],
    knight_table_eg: [[i32; 9]; 9],
    silver_table_eg: [[i32; 9]; 9],
    gold_table_eg: [[i32; 9]; 9],
    bishop_table_eg: [[i32; 9]; 9],
    rook_table_eg: [[i32; 9]; 9],
}

impl PieceSquareTables {
    /// Get positional value for a piece (returns TaperedScore)
    fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> TaperedScore {
        let (mg_table, eg_table) = self.get_tables(piece_type);
        let (row, col) = self.get_table_coords(pos, player);
        
        let mg_value = mg_table[row as usize][col as usize];
        let eg_value = eg_table[row as usize][col as usize];
        
        TaperedScore::new_tapered(mg_value, eg_value)
    }
    
    /// Get both mg and eg tables for a piece type
    fn get_tables(&self, piece_type: PieceType) -> (&[[i32; 9]; 9], &[[i32; 9]; 9]) {
        match piece_type {
            PieceType::Pawn => (&self.pawn_table_mg, &self.pawn_table_eg),
            PieceType::Lance => (&self.lance_table_mg, &self.lance_table_eg),
            PieceType::Knight => (&self.knight_table_mg, &self.knight_table_eg),
            PieceType::Silver => (&self.silver_table_mg, &self.silver_table_eg),
            PieceType::Gold => (&self.gold_table_mg, &self.gold_table_eg),
            PieceType::Bishop => (&self.bishop_table_mg, &self.bishop_table_eg),
            PieceType::Rook => (&self.rook_table_mg, &self.rook_table_eg),
            _ => return (&[[0; 9]; 9], &[[0; 9]; 9]), // No positional value for other pieces
        }
    }
}
```

## 5. Evaluation Component Refactoring

### 5.1 Main Evaluation Function

```rust
impl PositionEvaluator {
    /// Evaluate the current position from the perspective of the given player
    pub fn evaluate(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        // 1. Calculate game phase
        let game_phase = self.calculate_game_phase(board);
        
        // 2. Accumulate all evaluation terms
        let mut total_score = TaperedScore::default();
        
        // Add tempo bonus (same in all phases)
        total_score += TaperedScore::new(10);
        
        // Material and positional evaluation
        total_score += self.evaluate_material_and_position(board, player);
        
        // Pawn structure
        total_score += self.evaluate_pawn_structure(board, player);
        
        // King safety (more important in middlegame)
        total_score += self.evaluate_king_safety(board, player);
        
        // Mobility
        total_score += self.evaluate_mobility(board, player, captured_pieces);
        
        // Piece coordination
        total_score += self.evaluate_piece_coordination(board, player);
        
        // Center control
        total_score += self.evaluate_center_control(board, player);
        
        // Development
        total_score += self.evaluate_development(board, player);
        
        // 3. Interpolate final score based on game phase
        let final_score = total_score.interpolate(game_phase);
        
        // 4. Return score from perspective of current player
        if player == board.side_to_move() {
            final_score
        } else {
            -final_score
        }
    }
}
```

### 5.2 Material and Positional Evaluation

```rust
fn evaluate_material_and_position(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let mut score = TaperedScore::default();
    
    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = piece.piece_type.base_value();
                let positional_value = self.piece_square_tables.get_value(piece.piece_type, pos, piece.player);
                
                // Material values are the same in all phases
                let material_score = TaperedScore::new(piece_value);
                let total_piece_score = material_score + positional_value;
                
                if piece.player == player {
                    score += total_piece_score;
                } else {
                    score -= total_piece_score;
                }
            }
        }
    }
    
    score
}
```

### 5.3 King Safety Evaluation

```rust
fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let mut score = TaperedScore::default();
    
    let king_pos = self.find_king_position(board, player);
    if king_pos.is_none() {
        return score;
    }
    
    let king_pos = king_pos.unwrap();
    
    // King shield evaluation - more important in middlegame
    let shield_bonus = self.evaluate_king_shield(board, king_pos, player);
    score += TaperedScore::new_tapered(shield_bonus, shield_bonus / 4);
    
    // Enemy attackers penalty - much more important in middlegame
    let attacker_penalty = self.evaluate_enemy_attackers(board, king_pos, player);
    score -= TaperedScore::new_tapered(attacker_penalty, attacker_penalty / 3);
    
    score
}
```

### 5.4 Pawn Structure Evaluation

```rust
fn evaluate_pawn_structure(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let mut score = TaperedScore::default();
    
    // Pawn chains - more valuable in endgame
    let chain_bonus = self.evaluate_pawn_chains(board, player);
    score += TaperedScore::new_tapered(chain_bonus, chain_bonus * 2);
    
    // Advanced pawns - much more valuable in endgame
    let advancement_bonus = self.evaluate_pawn_advancement(board, player);
    score += TaperedScore::new_tapered(advancement_bonus, advancement_bonus * 3);
    
    // Isolated pawns - penalty is similar in both phases
    let isolation_penalty = self.evaluate_isolated_pawns(board, player);
    score -= TaperedScore::new(isolation_penalty);
    
    score
}
```

## 6. Piece-Square Table Design

### 6.1 Middlegame Tables

Middlegame tables emphasize:
- **King Safety**: Kings should be well-protected
- **Development**: Pieces should be developed from starting positions
- **Center Control**: Control of central squares is crucial
- **Mobility**: Pieces should have good mobility

### 6.2 Endgame Tables

Endgame tables emphasize:
- **King Activity**: Kings should be active and centralized
- **Pawn Advancement**: Pawns should be advanced toward promotion
- **Piece Coordination**: Pieces should work together
- **Promotion Potential**: Pieces should be positioned for promotion

### 6.3 Example Table Differences

```rust
// Example: Rook table differences
fn init_rook_table_mg() -> [[i32; 9]; 9] {
    // Middlegame: Rooks on back rank are fine, center control important
    [
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
    ]
}

fn init_rook_table_eg() -> [[i32; 9]; 9] {
    // Endgame: Rooks should be active, back rank is less important
    [
        [-10, -5, 0, 5, 5, 5, 0, -5, -10],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [5, 10, 15, 20, 20, 20, 15, 10, 5],
        [10, 15, 20, 25, 25, 25, 20, 15, 10],
        [10, 15, 20, 25, 25, 25, 20, 15, 10],
        [10, 15, 20, 25, 25, 25, 20, 15, 10],
        [5, 10, 15, 20, 20, 20, 15, 10, 5],
        [0, 5, 10, 15, 15, 15, 10, 5, 0],
        [-10, -5, 0, 5, 5, 5, 0, -5, -10],
    ]
}
```

## 7. Implementation Strategy

### 7.1 Phase 1: Core Infrastructure

1. **Add TaperedScore to types.rs**
   - Define the TaperedScore struct with all necessary operations
   - Add game phase constants and helper functions

2. **Update PositionEvaluator**
   - Add game phase calculation method
   - Modify main evaluate function to use interpolation

### 7.2 Phase 2: Piece-Square Tables Refactoring

1. **Create dual-phase tables**
   - Implement separate mg/eg tables for each piece type
   - Update get_value method to return TaperedScore

2. **Design table values**
   - Create middlegame-optimized tables
   - Create endgame-optimized tables
   - Ensure smooth transitions between phases

### 7.3 Phase 3: Evaluation Component Updates

1. **Refactor each evaluation function**
   - Update return types to TaperedScore
   - Adjust weights for different phases
   - Maintain backward compatibility during transition

2. **Update material evaluation**
   - Keep material values constant across phases
   - Only positional values should be tapered

### 7.4 Phase 4: Testing and Tuning

1. **Unit tests**
   - Test game phase calculation
   - Test TaperedScore operations
   - Test interpolation accuracy

2. **Integration tests**
   - Test evaluation consistency
   - Test performance impact
   - Test strategic improvements

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_phase_calculation() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new(); // Starting position
        
        // Starting position should be maximum phase
        let phase = evaluator.calculate_game_phase(&board);
        assert_eq!(phase, GAME_PHASE_MAX);
    }
    
    #[test]
    fn test_tapered_score_interpolation() {
        let score = TaperedScore::new_tapered(100, 200);
        
        // At phase 0 (endgame), should return eg value
        assert_eq!(score.interpolate(0), 200);
        
        // At phase 256 (opening), should return mg value
        assert_eq!(score.interpolate(GAME_PHASE_MAX), 100);
        
        // At phase 128 (middlegame), should return average
        assert_eq!(score.interpolate(128), 150);
    }
    
    #[test]
    fn test_king_safety_phase_dependency() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        let mg_score = evaluator.evaluate_king_safety(&board, Player::Black);
        
        // King safety should be higher in middlegame than endgame
        assert!(mg_score.mg > mg_score.eg);
    }
}
```

### 8.2 Integration Tests

```rust
#[test]
fn test_evaluation_consistency() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    // Evaluation should be consistent across multiple calls
    let score1 = evaluator.evaluate(&board, Player::Black, &captured);
    let score2 = evaluator.evaluate(&board, Player::Black, &captured);
    assert_eq!(score1, score2);
    
    // Evaluation should be symmetric (Black vs White)
    let score_black = evaluator.evaluate(&board, Player::Black, &captured);
    let score_white = evaluator.evaluate(&board, Player::White, &captured);
    assert_eq!(score_black, -score_white);
}
```

### 8.3 Performance Tests

```rust
#[test]
fn test_evaluation_performance() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = evaluator.evaluate(&board, Player::Black, &captured);
    }
    let duration = start.elapsed();
    
    // Should complete 1000 evaluations in reasonable time
    assert!(duration.as_millis() < 1000);
}
```

## 9. Performance Considerations

### 9.1 Computational Overhead

- **Game Phase Calculation**: O(1) - only needs to count pieces once per evaluation
- **TaperedScore Operations**: Minimal overhead for arithmetic operations
- **Table Lookups**: Same as current system, just two lookups instead of one

### 9.2 Memory Usage

- **Piece-Square Tables**: Double the memory usage (mg + eg tables)
- **TaperedScore**: 8 bytes per score (2 i32s) vs 4 bytes currently
- **Overall Impact**: ~2x memory usage for evaluation data

### 9.3 Optimization Strategies

1. **Cache Game Phase**: Calculate once per search node, reuse for all evaluations
2. **SIMD Operations**: Use SIMD for TaperedScore arithmetic where possible
3. **Table Compression**: Use smaller data types if precision allows

## 10. Migration Plan

### 10.1 Backward Compatibility

- Keep existing evaluation functions during transition
- Add new tapered functions alongside old ones
- Use feature flags to switch between implementations

### 10.2 Gradual Rollout

1. **Phase 1**: Implement core infrastructure, test thoroughly
2. **Phase 2**: Add tapered evaluation as optional feature
3. **Phase 3**: A/B test against current system
4. **Phase 4**: Full migration once proven superior

### 10.3 Rollback Strategy

- Maintain old evaluation system as fallback
- Use configuration to switch between systems
- Monitor performance metrics during transition

## 11. Future Enhancements

### 11.1 Advanced Phase Detection

- **Material-based phases**: More sophisticated phase calculation
- **Position-based phases**: Consider piece activity and king safety
- **Dynamic phases**: Adjust phase calculation based on position characteristics

### 11.2 Machine Learning Integration

- **Automated tuning**: Use ML to optimize table values
- **Position classification**: ML-based phase detection
- **Adaptive evaluation**: Learn from game outcomes

### 11.3 Extended Evaluation

- **Tactical patterns**: Phase-aware tactical evaluation
- **Strategic plans**: Long-term planning based on game phase
- **Endgame knowledge**: Specialized endgame evaluation

## 12. Conclusion

The tapered evaluation system represents a significant improvement to the Shogi engine's strategic understanding. By implementing phase-aware evaluation, the engine will make more nuanced decisions that reflect the changing nature of the game as it progresses from opening to endgame.

The design prioritizes maintainability, performance, and gradual migration, ensuring that the implementation can be integrated smoothly into the existing codebase while providing clear benefits in strategic play.

The success of this implementation will be measured not only by improved strategic play but also by maintaining the engine's tactical strength and search performance. The comprehensive testing strategy and migration plan ensure that these goals can be achieved reliably.
