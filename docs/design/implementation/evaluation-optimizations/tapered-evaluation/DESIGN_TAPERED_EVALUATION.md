# Tapered Evaluation Design

## Overview

This document outlines the design for implementing tapered evaluation in the Shogi engine. Tapered evaluation allows different evaluation weights for different game phases (opening, middlegame, endgame), providing more accurate position assessment as the game progresses.

## Current State

The engine currently uses a single set of evaluation weights regardless of game phase, which leads to suboptimal evaluation in opening and endgame positions.

## Design Goals

1. **Phase-Aware Evaluation**: Different weights for opening, middlegame, and endgame
2. **Smooth Transitions**: Continuous evaluation changes between phases
3. **Accuracy**: More precise position assessment in all game phases
4. **Performance**: Minimal overhead for phase calculation and interpolation
5. **Tunability**: Easy to adjust weights for each phase

## Technical Architecture

### 1. Tapered Score System

**Purpose**: Store and interpolate evaluation scores based on game phase.

**Components**:
- Opening score storage
- Endgame score storage
- Phase calculation
- Score interpolation
- Configuration management

**Implementation**:
```rust
/// Tapered score with separate opening and endgame values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaperedScore {
    pub opening: i32,
    pub endgame: i32,
}

impl TaperedScore {
    /// Create a new tapered score
    pub fn new(opening: i32, endgame: i32) -> Self {
        Self { opening, endgame }
    }
    
    /// Interpolate between opening and endgame based on game phase
    pub fn interpolate(&self, phase: f32) -> i32 {
        // phase: 0.0 = opening, 1.0 = endgame
        let opening_weight = 1.0 - phase;
        let endgame_weight = phase;
        
        ((self.opening as f32 * opening_weight) + 
         (self.endgame as f32 * endgame_weight)) as i32
    }
    
    /// Add two tapered scores
    pub fn add(&self, other: TaperedScore) -> TaperedScore {
        TaperedScore {
            opening: self.opening + other.opening,
            endgame: self.endgame + other.endgame,
        }
    }
    
    /// Subtract two tapered scores
    pub fn sub(&self, other: TaperedScore) -> TaperedScore {
        TaperedScore {
            opening: self.opening - other.opening,
            endgame: self.endgame - other.endgame,
        }
    }
}

impl std::ops::Add for TaperedScore {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl std::ops::Sub for TaperedScore {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}
```

### 2. Game Phase Calculation

**Purpose**: Determine the current game phase based on material remaining on the board.

**Technical Details**:
- Calculate based on total material remaining
- Consider piece types and their values
- Handle promoted pieces appropriately
- Normalize to range [0.0, 1.0]

**Implementation**:
```rust
/// Game phase calculator
pub struct PhaseCalculator {
    // Material values for phase calculation
    pawn_phase: u8,
    lance_phase: u8,
    knight_phase: u8,
    silver_phase: u8,
    gold_phase: u8,
    bishop_phase: u8,
    rook_phase: u8,
    
    // Maximum phase value (starting position)
    max_phase: u16,
}

impl PhaseCalculator {
    /// Create a new phase calculator with standard values
    pub fn new() -> Self {
        Self {
            pawn_phase: 0,
            lance_phase: 1,
            knight_phase: 1,
            silver_phase: 2,
            gold_phase: 2,
            bishop_phase: 4,
            rook_phase: 5,
            max_phase: 0, // Will be calculated
        }
    }
    
    /// Calculate maximum phase value for starting position
    pub fn calculate_max_phase(&mut self) {
        // Standard shogi starting position:
        // 9 pawns, 1 lance, 1 knight, 1 silver, 1 gold, 1 bishop, 1 rook per side
        let phase_per_side = 
            9 * self.pawn_phase as u16 +
            2 * self.lance_phase as u16 +
            2 * self.knight_phase as u16 +
            2 * self.silver_phase as u16 +
            2 * self.gold_phase as u16 +
            1 * self.bishop_phase as u16 +
            1 * self.rook_phase as u16;
        
        self.max_phase = phase_per_side * 2; // Both sides
    }
    
    /// Calculate game phase from board position
    /// Returns value between 0.0 (opening) and 1.0 (endgame)
    pub fn calculate_phase(&self, board: &dyn BoardTrait) -> f32 {
        let mut current_phase = 0u16;
        
        // Count material for phase calculation
        for player in [Player::Sente, Player::Gote] {
            current_phase += self.count_player_phase(board, player);
        }
        
        // Calculate phase ratio (inverted: more material = opening, less = endgame)
        if self.max_phase == 0 {
            return 0.5; // Default to middlegame
        }
        
        let phase_ratio = current_phase as f32 / self.max_phase as f32;
        
        // Invert and clamp: 0.0 = endgame (no material), 1.0 = opening (full material)
        // We want 0.0 = opening, 1.0 = endgame, so we subtract from 1.0
        (1.0 - phase_ratio).clamp(0.0, 1.0)
    }
    
    /// Count phase value for a player's pieces
    fn count_player_phase(&self, board: &dyn BoardTrait, player: Player) -> u16 {
        let mut phase = 0u16;
        
        // Count pieces on board
        for piece_type in PieceType::all() {
            let count = board.count_pieces(player, piece_type);
            phase += self.get_piece_phase_value(piece_type) as u16 * count as u16;
        }
        
        phase
    }
    
    /// Get phase value for a piece type
    fn get_piece_phase_value(&self, piece_type: PieceType) -> u8 {
        match piece_type {
            PieceType::Pawn | PieceType::PromotedPawn => self.pawn_phase,
            PieceType::Lance | PieceType::PromotedLance => self.lance_phase,
            PieceType::Knight | PieceType::PromotedKnight => self.knight_phase,
            PieceType::Silver | PieceType::PromotedSilver => self.silver_phase,
            PieceType::Gold => self.gold_phase,
            PieceType::Bishop | PieceType::PromotedBishop => self.bishop_phase,
            PieceType::Rook | PieceType::PromotedRook => self.rook_phase,
            PieceType::King => 0, // King doesn't affect phase
        }
    }
}
```

### 3. Tapered Evaluation Engine

**Purpose**: Coordinate phase-aware evaluation across all evaluation components.

**Technical Details**:
- Manage phase calculation
- Coordinate piece-square tables
- Interpolate all evaluation terms
- Provide unified evaluation interface

**Implementation**:
```rust
/// Main tapered evaluation engine
pub struct TaperedEvaluation {
    phase_calculator: PhaseCalculator,
    material_weights: MaterialWeights,
    piece_square_tables: PieceSquareTables,
    position_evaluator: PositionEvaluator,
}

impl TaperedEvaluation {
    /// Create a new tapered evaluation engine
    pub fn new() -> Self {
        let mut phase_calculator = PhaseCalculator::new();
        phase_calculator.calculate_max_phase();
        
        Self {
            phase_calculator,
            material_weights: MaterialWeights::new(),
            piece_square_tables: PieceSquareTables::new(),
            position_evaluator: PositionEvaluator::new(),
        }
    }
    
    /// Evaluate a position with tapered evaluation
    pub fn evaluate(&self, board: &dyn BoardTrait) -> i32 {
        // Calculate current game phase
        let phase = self.phase_calculator.calculate_phase(board);
        
        // Evaluate all components
        let material_score = self.evaluate_material(board);
        let piece_square_score = self.evaluate_piece_squares(board);
        let position_score = self.evaluate_position(board);
        
        // Combine and interpolate scores
        let total_score = material_score + piece_square_score + position_score;
        total_score.interpolate(phase)
    }
    
    /// Evaluate material with phase awareness
    fn evaluate_material(&self, board: &dyn BoardTrait) -> TaperedScore {
        self.material_weights.evaluate(board)
    }
    
    /// Evaluate piece-square tables with phase awareness
    fn evaluate_piece_squares(&self, board: &dyn BoardTrait) -> TaperedScore {
        self.piece_square_tables.evaluate(board)
    }
    
    /// Evaluate position with phase awareness
    fn evaluate_position(&self, board: &dyn BoardTrait) -> TaperedScore {
        self.position_evaluator.evaluate(board)
    }
}
```

### 4. Material Evaluation

**Purpose**: Evaluate material with phase-specific weights.

**Implementation**:
```rust
/// Material evaluation with phase awareness
pub struct MaterialWeights {
    // Piece values for opening and endgame
    pawn: TaperedScore,
    lance: TaperedScore,
    knight: TaperedScore,
    silver: TaperedScore,
    gold: TaperedScore,
    bishop: TaperedScore,
    rook: TaperedScore,
    
    // Promoted piece values
    promoted_pawn: TaperedScore,
    promoted_lance: TaperedScore,
    promoted_knight: TaperedScore,
    promoted_silver: TaperedScore,
    promoted_bishop: TaperedScore,
    promoted_rook: TaperedScore,
}

impl MaterialWeights {
    /// Create material weights with standard values
    pub fn new() -> Self {
        Self {
            // Standard shogi values, adjusted for phases
            pawn: TaperedScore::new(100, 120),
            lance: TaperedScore::new(350, 320),
            knight: TaperedScore::new(400, 380),
            silver: TaperedScore::new(500, 520),
            gold: TaperedScore::new(600, 620),
            bishop: TaperedScore::new(850, 900),
            rook: TaperedScore::new(1000, 1100),
            
            // Promoted pieces generally more valuable in endgame
            promoted_pawn: TaperedScore::new(600, 650),
            promoted_lance: TaperedScore::new(600, 650),
            promoted_knight: TaperedScore::new(600, 650),
            promoted_silver: TaperedScore::new(600, 650),
            promoted_bishop: TaperedScore::new(1000, 1150),
            promoted_rook: TaperedScore::new(1200, 1400),
        }
    }
    
    /// Evaluate material for a position
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        let mut score = TaperedScore::new(0, 0);
        
        // Evaluate for both players
        let sente_score = self.evaluate_player(board, Player::Sente);
        let gote_score = self.evaluate_player(board, Player::Gote);
        
        score = score + sente_score - gote_score;
        score
    }
    
    /// Evaluate material for one player
    fn evaluate_player(&self, board: &dyn BoardTrait, player: Player) -> TaperedScore {
        let mut score = TaperedScore::new(0, 0);
        
        // Count pieces on board
        for piece_type in PieceType::all() {
            let count = board.count_pieces(player, piece_type);
            let piece_value = self.get_piece_value(piece_type);
            score = score + piece_value * count as i32;
        }
        
        // Count pieces in hand
        for piece_type in PieceType::hand_pieces() {
            let count = board.count_hand_pieces(player, piece_type);
            let piece_value = self.get_hand_piece_value(piece_type);
            score = score + piece_value * count as i32;
        }
        
        score
    }
    
    /// Get tapered value for a piece type
    fn get_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            PieceType::Pawn => self.pawn,
            PieceType::Lance => self.lance,
            PieceType::Knight => self.knight,
            PieceType::Silver => self.silver,
            PieceType::Gold => self.gold,
            PieceType::Bishop => self.bishop,
            PieceType::Rook => self.rook,
            PieceType::PromotedPawn => self.promoted_pawn,
            PieceType::PromotedLance => self.promoted_lance,
            PieceType::PromotedKnight => self.promoted_knight,
            PieceType::PromotedSilver => self.promoted_silver,
            PieceType::PromotedBishop => self.promoted_bishop,
            PieceType::PromotedRook => self.promoted_rook,
            PieceType::King => TaperedScore::new(0, 0),
        }
    }
    
    /// Get tapered value for hand pieces (unpromoted)
    fn get_hand_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        // Hand pieces have slightly different values
        match piece_type {
            PieceType::Pawn => TaperedScore::new(120, 140),
            PieceType::Lance => TaperedScore::new(380, 350),
            PieceType::Knight => TaperedScore::new(430, 410),
            PieceType::Silver => TaperedScore::new(540, 560),
            PieceType::Gold => TaperedScore::new(650, 670),
            PieceType::Bishop => TaperedScore::new(900, 950),
            PieceType::Rook => TaperedScore::new(1050, 1150),
            _ => TaperedScore::new(0, 0),
        }
    }
}

impl std::ops::Mul<i32> for TaperedScore {
    type Output = Self;
    
    fn mul(self, rhs: i32) -> Self {
        TaperedScore {
            opening: self.opening * rhs,
            endgame: self.endgame * rhs,
        }
    }
}
```

### 5. Piece-Square Tables

**Purpose**: Provide position-dependent evaluation with phase awareness.

**Implementation**:
```rust
/// Piece-square tables with phase awareness
pub struct PieceSquareTables {
    // Tables for each piece type
    // [piece_type][phase (0=opening, 1=endgame)][square]
    tables: HashMap<PieceType, [Vec<i32>; 2]>,
}

impl PieceSquareTables {
    /// Create piece-square tables with default values
    pub fn new() -> Self {
        let mut tables = HashMap::new();
        
        // Initialize tables for each piece type
        for piece_type in PieceType::all() {
            let opening_table = Self::create_opening_table(piece_type);
            let endgame_table = Self::create_endgame_table(piece_type);
            tables.insert(piece_type, [opening_table, endgame_table]);
        }
        
        Self { tables }
    }
    
    /// Evaluate piece-square tables for a position
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        let mut score = TaperedScore::new(0, 0);
        
        // Evaluate for both players
        let sente_score = self.evaluate_player(board, Player::Sente);
        let gote_score = self.evaluate_player(board, Player::Gote);
        
        score = score + sente_score - gote_score;
        score
    }
    
    /// Evaluate piece-square tables for one player
    fn evaluate_player(&self, board: &dyn BoardTrait, player: Player) -> TaperedScore {
        let mut score = TaperedScore::new(0, 0);
        
        for piece_type in PieceType::all() {
            let pieces = board.get_pieces(player, piece_type);
            
            for &square in pieces {
                let opening_value = self.get_square_value(piece_type, square, 0);
                let endgame_value = self.get_square_value(piece_type, square, 1);
                score = score + TaperedScore::new(opening_value, endgame_value);
            }
        }
        
        score
    }
    
    /// Get value for a piece on a square in a specific phase
    fn get_square_value(&self, piece_type: PieceType, square: usize, phase: usize) -> i32 {
        if let Some(tables) = self.tables.get(&piece_type) {
            tables[phase][square]
        } else {
            0
        }
    }
    
    /// Create opening piece-square table for a piece type
    fn create_opening_table(piece_type: PieceType) -> Vec<i32> {
        let mut table = vec![0; 81];
        
        match piece_type {
            PieceType::Pawn => {
                // In opening, pawns near center are valuable
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    // Encourage center control
                    let center_bonus = if (3..=5).contains(&file) { 10 } else { 0 };
                    
                    // Encourage advancement
                    let advance_bonus = (8 - rank) * 5;
                    
                    table[square] = center_bonus + advance_bonus;
                }
            }
            PieceType::King => {
                // In opening, king should stay protected
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    // Prefer castled position
                    let castle_bonus = if rank == 0 && (file <= 2 || file >= 6) { 20 } else { 0 };
                    
                    // Penalty for king in center
                    let center_penalty = if (3..=5).contains(&file) { -30 } else { 0 };
                    
                    table[square] = castle_bonus + center_penalty;
                }
            }
            _ => {
                // Default: slight bonus for center squares
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    let center_bonus = if (3..=5).contains(&file) && (3..=5).contains(&rank) {
                        5
                    } else {
                        0
                    };
                    
                    table[square] = center_bonus;
                }
            }
        }
        
        table
    }
    
    /// Create endgame piece-square table for a piece type
    fn create_endgame_table(piece_type: PieceType) -> Vec<i32> {
        let mut table = vec![0; 81];
        
        match piece_type {
            PieceType::King => {
                // In endgame, king should be active
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    // Prefer center positions
                    let center_bonus = if (3..=5).contains(&file) && (3..=5).contains(&rank) {
                        30
                    } else {
                        0
                    };
                    
                    // Bonus for activity
                    let activity_bonus = rank * 5;
                    
                    table[square] = center_bonus + activity_bonus;
                }
            }
            PieceType::Pawn => {
                // In endgame, passed pawns are very valuable
                for square in 0..81 {
                    let rank = square / 9;
                    
                    // Higher bonus for advanced pawns
                    let advance_bonus = (8 - rank) * 10;
                    
                    table[square] = advance_bonus;
                }
            }
            _ => {
                // Default: general activity bonus
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    let activity_bonus = if (2..=6).contains(&file) && (2..=6).contains(&rank) {
                        10
                    } else {
                        0
                    };
                    
                    table[square] = activity_bonus;
                }
            }
        }
        
        table
    }
}
```

### 6. Position Evaluation

**Purpose**: Evaluate positional factors with phase awareness.

**Implementation**:
```rust
/// Position evaluator with phase awareness
pub struct PositionEvaluator {
    king_safety: KingSafetyEvaluator,
    pawn_structure: PawnStructureEvaluator,
    piece_mobility: PieceMobilityEvaluator,
}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {
            king_safety: KingSafetyEvaluator::new(),
            pawn_structure: PawnStructureEvaluator::new(),
            piece_mobility: PieceMobilityEvaluator::new(),
        }
    }
    
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        let mut score = TaperedScore::new(0, 0);
        
        // King safety (more important in opening/middlegame)
        score = score + self.king_safety.evaluate(board);
        
        // Pawn structure
        score = score + self.pawn_structure.evaluate(board);
        
        // Piece mobility
        score = score + self.piece_mobility.evaluate(board);
        
        score
    }
}

/// King safety evaluation with phase awareness
pub struct KingSafetyEvaluator {}

impl KingSafetyEvaluator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        // King safety more important in opening, less in endgame
        let safety_value = self.calculate_king_safety(board);
        TaperedScore::new(safety_value * 3, safety_value / 2)
    }
    
    fn calculate_king_safety(&self, board: &dyn BoardTrait) -> i32 {
        // Simplified king safety calculation
        0
    }
}

/// Pawn structure evaluation with phase awareness
pub struct PawnStructureEvaluator {}

impl PawnStructureEvaluator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        // Pawn structure important in all phases
        let structure_value = self.calculate_pawn_structure(board);
        TaperedScore::new(structure_value, structure_value)
    }
    
    fn calculate_pawn_structure(&self, board: &dyn BoardTrait) -> i32 {
        // Simplified pawn structure calculation
        0
    }
}

/// Piece mobility evaluation with phase awareness
pub struct PieceMobilityEvaluator {}

impl PieceMobilityEvaluator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn evaluate(&self, board: &dyn BoardTrait) -> TaperedScore {
        // Mobility more important in middlegame
        let mobility_value = self.calculate_mobility(board);
        TaperedScore::new(mobility_value, mobility_value / 2)
    }
    
    fn calculate_mobility(&self, board: &dyn BoardTrait) -> i32 {
        // Simplified mobility calculation
        0
    }
}
```

## Integration Points

### Evaluation Engine Integration

```rust
impl Evaluator {
    fn evaluate_with_tapered(&self, board: &dyn BoardTrait) -> i32 {
        // Use tapered evaluation
        self.tapered_evaluation.evaluate(board)
    }
}
```

### Search Algorithm Integration

```rust
impl SearchEngine {
    fn search_with_tapered(&mut self, board: &mut dyn BoardTrait, depth: u8) -> i32 {
        // Phase tracking during search
        let phase = self.tapered_evaluation.phase_calculator.calculate_phase(board);
        
        // Use phase information for pruning decisions
        // ...
    }
}
```

## Performance Considerations

### Memory Usage
- **Tapered Scores**: 8 bytes per score (2 × i32)
- **Piece-Square Tables**: ~1.3KB per piece type × 14 types × 2 phases = ~36KB
- **Phase Calculator**: Negligible
- **Total**: ~40KB additional memory

### Computational Complexity
- **Phase Calculation**: O(n) where n is number of pieces
- **Score Interpolation**: O(1) per score
- **Piece-Square Lookup**: O(1) per piece
- **Total Evaluation**: O(n) where n is number of pieces

### Cache Efficiency
- Tapered scores are small and cache-friendly
- Piece-square tables have good spatial locality
- Phase calculation can be cached between evaluations

## Testing Strategy

### Unit Tests
1. **Tapered Score**: Test interpolation and arithmetic
2. **Phase Calculation**: Test with various positions
3. **Material Evaluation**: Test phase-aware values
4. **Piece-Square Tables**: Test table lookups
5. **Position Evaluation**: Test all components

### Integration Tests
1. **Evaluation Accuracy**: Compare with known positions
2. **Phase Transitions**: Verify smooth transitions
3. **Search Integration**: Test with search algorithm
4. **Performance**: Benchmark evaluation speed

## Configuration Options

```rust
pub struct TaperedEvalConfig {
    pub enable_tapered: bool,
    pub material_weights: MaterialWeights,
    pub piece_square_tables: Option<PieceSquareTables>,
    pub position_weights: PositionWeights,
}
```

## Expected Performance Impact

### Evaluation Accuracy
- **20-30% Improvement**: In position assessment
- **Better Endgame Play**: Stronger play in simplified positions
- **Better Opening Play**: Improved development and piece coordination

### Performance
- **<5% Overhead**: For phase calculation and interpolation
- **Minimal Memory**: Only ~40KB additional memory
- **Cache-Friendly**: Good CPU cache utilization

## Future Enhancements

1. **Multi-Phase Evaluation**: Separate middlegame phase
2. **Adaptive Phases**: Position-type specific phase boundaries
3. **Neural Network Integration**: Learn optimal phase weights
4. **Dynamic Tuning**: Adjust weights during play

## Conclusion

The tapered evaluation design provides a comprehensive solution for phase-aware position evaluation in the Shogi engine. The implementation focuses on:

- Accurate phase calculation
- Smooth score interpolation
- Comprehensive evaluation coverage
- Minimal performance overhead
- Easy tuning and configuration

This design provides the foundation for significantly more accurate evaluation throughout all game phases.

