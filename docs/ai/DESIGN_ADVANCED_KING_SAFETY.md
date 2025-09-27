# Design Document: Advanced King Safety Evaluation

## 1. Overview

This document provides a comprehensive design for implementing advanced king safety evaluation in the Shogi engine. The current implementation in `src/evaluation.rs` uses a basic approach that counts nearby friendly pieces and penalizes nearby enemy pieces. This design extends that foundation with sophisticated castle recognition, attack analysis, and threat evaluation systems.

## 2. Current State Analysis

### 2.1 Existing Implementation
The current `evaluate_king_safety` function in `src/evaluation.rs` (lines 238-297) implements:

- **King Shield Evaluation**: Rewards friendly pieces in the 8 squares around the king
  - Gold: 40 points (mg), 20 points (eg)
  - Silver: 30 points (mg), 15 points (eg)
  - Knight: 20 points (mg), 10 points (eg)
  - Lance: 15 points (mg), 7 points (eg)
  - Pawn: 10 points (mg), 5 points (eg)
  - Other pieces: 5 points (mg), 2 points (eg)

- **Enemy Proximity Penalty**: Penalizes enemy pieces within 2 squares of the king
  - 30 points per enemy piece (mg), 15 points (eg)

### 2.2 Limitations
1. **No Castle Recognition**: Cannot distinguish between well-formed castles and random piece placement
2. **No Attack Analysis**: Doesn't evaluate the quality or coordination of enemy attacks
3. **No Threat Assessment**: Doesn't consider potential future threats or tactical patterns
4. **Static Evaluation**: Doesn't adapt to different game phases or strategic contexts

## 3. Design Goals

### 3.1 Primary Objectives
1. **Castle Recognition**: Identify and evaluate common Shogi defensive formations
2. **Attack Analysis**: Assess the quality and danger of enemy attacking patterns
3. **Threat Evaluation**: Consider potential future threats and tactical vulnerabilities
4. **Phase-Aware Evaluation**: Adapt evaluation based on game phase (opening/middlegame/endgame)

### 3.2 Secondary Objectives
1. **Performance**: Maintain evaluation speed while adding complexity
2. **Extensibility**: Design for easy addition of new castle patterns and attack motifs
3. **Tunability**: Provide parameters that can be optimized through automated tuning
4. **Integration**: Seamlessly integrate with existing `TaperedScore` system

## 4. Architecture Design

### 4.1 Module Structure
```
src/evaluation/
├── king_safety.rs          # Main king safety evaluation module
├── castles.rs              # Castle pattern recognition
├── attacks.rs              # Attack analysis and threat evaluation
└── patterns/               # Pattern definitions
    ├── mino.rs            # Mino castle patterns
    ├── anaguma.rs         # Anaguma castle patterns
    ├── yagura.rs          # Yagura castle patterns
    └── common.rs          # Common attack patterns
```

### 4.2 Core Components

#### 4.2.1 King Safety Evaluator
```rust
pub struct KingSafetyEvaluator {
    castle_recognizer: CastleRecognizer,
    attack_analyzer: AttackAnalyzer,
    threat_evaluator: ThreatEvaluator,
    config: KingSafetyConfig,
}

impl KingSafetyEvaluator {
    pub fn evaluate(&self, board: &BitboardBoard, player: Player) -> TaperedScore;
    pub fn evaluate_castle_structure(&self, board: &BitboardBoard, player: Player) -> TaperedScore;
    pub fn evaluate_attacks(&self, board: &BitboardBoard, player: Player) -> TaperedScore;
    pub fn evaluate_threats(&self, board: &BitboardBoard, player: Player) -> TaperedScore;
}
```

#### 4.2.2 Castle Recognizer
```rust
pub struct CastleRecognizer {
    patterns: Vec<CastlePattern>,
    config: CastleConfig,
}

pub struct CastlePattern {
    pub name: &'static str,
    pub pieces: Vec<CastlePiece>,
    pub score: TaperedScore,
    pub flexibility: u8, // How many pieces can be missing and still count
}

pub struct CastlePiece {
    pub piece_type: PieceType,
    pub relative_pos: (i8, i8), // Relative to king position
    pub required: bool,          // Must be present for pattern match
    pub weight: u8,              // Importance in pattern (1-10)
}
```

#### 4.2.3 Attack Analyzer
```rust
pub struct AttackAnalyzer {
    attack_tables: AttackTables,
    config: AttackConfig,
}

pub struct AttackTables {
    // Pre-computed attack bitboards for each piece type and position
    piece_attacks: [[Bitboard; 81]; 14], // 14 piece types, 81 squares
    king_zone_attacks: [Bitboard; 81],   // Attack zones around each square
}

pub struct AttackEvaluation {
    pub num_attackers: u8,
    pub attack_weight: i32,
    pub coordination_bonus: i32,
    pub tactical_threats: i32,
}
```

## 5. Castle Pattern Recognition

### 5.1 Common Shogi Castles

#### 5.1.1 Mino Castle (美濃囲い)
**Description**: One of the most popular and solid defensive formations
**Key Characteristics**:
- King on 7h (Black) or 3b (White)
- Gold on 6h (Black) or 4b (White)
- Silver on 7g (Black) or 3c (White)
- Pawns on 6g, 7f, 8f (Black) or 4c, 3d, 2d (White)

**Pattern Definition**:
```rust
const MINO_CASTLE: CastlePattern = CastlePattern {
    name: "Mino",
    pieces: vec![
        CastlePiece { piece_type: PieceType::Gold, relative_pos: (-1, -1), required: true, weight: 10 },
        CastlePiece { piece_type: PieceType::Silver, relative_pos: (-2, -1), required: true, weight: 9 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-2, -2), required: false, weight: 6 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-1, -2), required: false, weight: 6 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (0, -2), required: false, weight: 6 },
    ],
    score: TaperedScore::new_tapered(180, 60),
    flexibility: 2,
};
```

#### 5.1.2 Anaguma Castle (穴熊囲い)
**Description**: Extremely solid defensive formation, "Hole Bear" castle
**Key Characteristics**:
- King on 8h (Black) or 2b (White)
- Gold on 7h (Black) or 3b (White)
- Silver on 8g (Black) or 2c (White)
- Multiple pawns forming a solid wall

**Pattern Definition**:
```rust
const ANAGUMA_CASTLE: CastlePattern = CastlePattern {
    name: "Anaguma",
    pieces: vec![
        CastlePiece { piece_type: PieceType::Gold, relative_pos: (-1, 0), required: true, weight: 10 },
        CastlePiece { piece_type: PieceType::Silver, relative_pos: (-2, 0), required: true, weight: 9 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-2, -1), required: false, weight: 7 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-2, 1), required: false, weight: 7 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-1, -1), required: false, weight: 6 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-1, 1), required: false, weight: 6 },
    ],
    score: TaperedScore::new_tapered(220, 40),
    flexibility: 3,
};
```

#### 5.1.3 Yagura Castle (矢倉囲い)
**Description**: Traditional castle formation, often used in Yagura openings
**Key Characteristics**:
- King on 6h (Black) or 4b (White)
- Gold on 5h (Black) or 5b (White)
- Silver on 6g (Black) or 4c (White)
- Lance on 9h (Black) or 1b (White)

**Pattern Definition**:
```rust
const YAGURA_CASTLE: CastlePattern = CastlePattern {
    name: "Yagura",
    pieces: vec![
        CastlePiece { piece_type: PieceType::Gold, relative_pos: (-1, -1), required: true, weight: 10 },
        CastlePiece { piece_type: PieceType::Silver, relative_pos: (-2, -1), required: true, weight: 9 },
        CastlePiece { piece_type: PieceType::Lance, relative_pos: (0, 3), required: false, weight: 5 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-2, -2), required: false, weight: 6 },
        CastlePiece { piece_type: PieceType::Pawn, relative_pos: (-1, -2), required: false, weight: 6 },
    ],
    score: TaperedScore::new_tapered(160, 80),
    flexibility: 2,
};
```

### 5.2 Castle Recognition Algorithm

```rust
impl CastleRecognizer {
    pub fn recognize_castle(&self, board: &BitboardBoard, player: Player, king_pos: Position) -> Option<&CastlePattern> {
        for pattern in &self.patterns {
            if self.matches_pattern(board, player, king_pos, pattern) {
                return Some(pattern);
            }
        }
        None
    }

    fn matches_pattern(&self, board: &BitboardBoard, player: Player, king_pos: Position, pattern: &CastlePattern) -> bool {
        let mut matches = 0;
        let mut required_matches = 0;
        let mut total_weight = 0;
        let mut matched_weight = 0;

        for castle_piece in &pattern.pieces {
            let check_pos = Position::new(
                (king_pos.row as i8 + castle_piece.relative_pos.0) as u8,
                (king_pos.col as i8 + castle_piece.relative_pos.1) as u8,
            );

            if let Some(piece) = board.get_piece(check_pos) {
                if piece.piece_type == castle_piece.piece_type && piece.player == player {
                    matches += 1;
                    matched_weight += castle_piece.weight as u32;
                    if castle_piece.required {
                        required_matches += 1;
                    }
                }
            }
            total_weight += castle_piece.weight as u32;
        }

        // Check if all required pieces are present
        let required_pieces = pattern.pieces.iter().filter(|p| p.required).count();
        if required_matches < required_pieces {
            return false;
        }

        // Check if enough pieces match (considering flexibility)
        let min_matches = pattern.pieces.len().saturating_sub(pattern.flexibility as usize);
        matches >= min_matches
    }
}
```

## 6. Attack Analysis

### 6.1 Attack Zone Definition

```rust
pub struct AttackZone {
    pub center: Position,
    pub radius: u8,
    pub squares: Bitboard,
}

impl AttackZone {
    pub fn new(center: Position, radius: u8) -> Self {
        let mut squares = EMPTY_BITBOARD;
        
        for dr in -(radius as i8)..=(radius as i8) {
            for dc in -(radius as i8)..=(radius as i8) {
                let new_row = center.row as i8 + dr;
                let new_col = center.col as i8 + dc;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    set_bit(&mut squares, pos);
                }
            }
        }
        
        Self { center, radius, squares }
    }
}
```

### 6.2 Attack Evaluation

```rust
impl AttackAnalyzer {
    pub fn evaluate_attacks(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = self.find_king_position(board, player)?;
        let attack_zone = AttackZone::new(king_pos, 2);
        let opponent = player.opposite();
        
        let mut evaluation = AttackEvaluation::default();
        
        // Count attackers in the zone
        for piece_type in ALL_PIECE_TYPES {
            let piece_bitboard = board.get_piece_bitboard(piece_type, opponent);
            for attacker_pos in self.iter_bits(piece_bitboard) {
                let attacks = self.get_attacks(piece_type, attacker_pos, board);
                if (attacks & attack_zone.squares).is_not_empty() {
                    evaluation.num_attackers += 1;
                    evaluation.attack_weight += self.get_piece_attack_value(piece_type);
                }
            }
        }
        
        // Evaluate coordination
        evaluation.coordination_bonus = self.evaluate_attack_coordination(board, opponent, &attack_zone);
        
        // Evaluate tactical threats
        evaluation.tactical_threats = self.evaluate_tactical_threats(board, player, &attack_zone);
        
        self.convert_to_tapered_score(evaluation)
    }

    fn get_piece_attack_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Rook => 100,
            PieceType::Bishop => 80,
            PieceType::PromotedRook => 120,
            PieceType::PromotedBishop => 100,
            PieceType::Silver => 60,
            PieceType::Gold => 70,
            PieceType::Knight => 50,
            PieceType::Lance => 40,
            PieceType::Pawn => 30,
            _ => 20,
        }
    }
}
```

### 6.3 Attack Coordination

```rust
impl AttackAnalyzer {
    fn evaluate_attack_coordination(&self, board: &BitboardBoard, player: Player, zone: &AttackZone) -> i32 {
        let mut coordination_score = 0;
        
        // Check for rook-bishop coordination
        if self.has_rook_bishop_coordination(board, player, zone) {
            coordination_score += 50;
        }
        
        // Check for multiple attackers on same square
        coordination_score += self.count_double_attacks(board, player, zone) * 30;
        
        // Check for discovered attacks
        coordination_score += self.count_discovered_attacks(board, player, zone) * 40;
        
        coordination_score
    }

    fn has_rook_bishop_coordination(&self, board: &BitboardBoard, player: Player, zone: &AttackZone) -> bool {
        let mut has_rook = false;
        let mut has_bishop = false;
        
        for piece_type in [PieceType::Rook, PieceType::Bishop, PieceType::PromotedRook, PieceType::PromotedBishop] {
            let piece_bitboard = board.get_piece_bitboard(piece_type, player);
            for pos in self.iter_bits(piece_bitboard) {
                let attacks = self.get_attacks(piece_type, pos, board);
                if (attacks & zone.squares).is_not_empty() {
                    match piece_type {
                        PieceType::Rook | PieceType::PromotedRook => has_rook = true,
                        PieceType::Bishop | PieceType::PromotedBishop => has_bishop = true,
                        _ => {}
                    }
                }
            }
        }
        
        has_rook && has_bishop
    }
}
```

## 7. Threat Evaluation

### 7.1 Tactical Threat Detection

```rust
pub struct ThreatEvaluator {
    tactical_patterns: Vec<TacticalPattern>,
    config: ThreatConfig,
}

pub struct TacticalPattern {
    pub name: &'static str,
    pub pattern_type: TacticalType,
    pub danger_level: u8, // 1-10 scale
    pub detection_fn: fn(&BitboardBoard, Player, Position) -> bool,
}

pub enum TacticalType {
    MatingNet,
    Sacrifice,
    Pin,
    Skewer,
    DiscoveredAttack,
    DoubleAttack,
}

impl ThreatEvaluator {
    pub fn evaluate_threats(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = self.find_king_position(board, player)?;
        let mut threat_score = 0;
        
        for pattern in &self.tactical_patterns {
            if (pattern.detection_fn)(board, player.opposite(), king_pos) {
                threat_score += pattern.danger_level as i32 * 20;
            }
        }
        
        TaperedScore::new_tapered(-threat_score, -threat_score / 3)
    }
}
```

### 7.2 Common Tactical Patterns

```rust
const TACTICAL_PATTERNS: &[TacticalPattern] = &[
    TacticalPattern {
        name: "Rook Pin",
        pattern_type: TacticalType::Pin,
        danger_level: 7,
        detection_fn: detect_rook_pin,
    },
    TacticalPattern {
        name: "Bishop Skewer",
        pattern_type: TacticalType::Skewer,
        danger_level: 6,
        detection_fn: detect_bishop_skewer,
    },
    TacticalPattern {
        name: "Knight Fork",
        pattern_type: TacticalType::DoubleAttack,
        danger_level: 8,
        detection_fn: detect_knight_fork,
    },
    // ... more patterns
];

fn detect_rook_pin(board: &BitboardBoard, player: Player, king_pos: Position) -> bool {
    // Implementation to detect if king is pinned by opponent's rook
    // Check for rook on same rank or file with no blocking pieces
    false // Placeholder
}

fn detect_bishop_skewer(board: &BitboardBoard, player: Player, king_pos: Position) -> bool {
    // Implementation to detect if king is skewered by opponent's bishop
    false // Placeholder
}

fn detect_knight_fork(board: &BitboardBoard, player: Player, king_pos: Position) -> bool {
    // Implementation to detect if opponent's knight can fork king and another piece
    false // Placeholder
}
```

## 8. Configuration and Tuning

### 8.1 Configuration Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingSafetyConfig {
    pub enabled: bool,
    pub castle_weight: f32,
    pub attack_weight: f32,
    pub threat_weight: f32,
    pub phase_adjustment: f32,
    pub performance_mode: bool,
}

impl Default for KingSafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            castle_weight: 1.0,
            attack_weight: 1.0,
            threat_weight: 1.0,
            phase_adjustment: 0.8,
            performance_mode: false,
        }
    }
}
```

### 8.2 Tuning Parameters

```rust
pub struct TuningParameters {
    // Castle pattern scores
    pub mino_score_mg: i32,
    pub mino_score_eg: i32,
    pub anaguma_score_mg: i32,
    pub anaguma_score_eg: i32,
    pub yagura_score_mg: i32,
    pub yagura_score_eg: i32,
    
    // Attack evaluation weights
    pub attacker_values: [i32; 14], // One per piece type
    pub coordination_bonus: i32,
    pub double_attack_bonus: i32,
    
    // Threat evaluation weights
    pub tactical_threat_weights: [i32; 6], // One per tactical type
    pub phase_reduction: f32,
}
```

## 9. Integration with Existing System

### 9.1 Modified Evaluation Function

```rust
impl PositionEvaluator {
    fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        if !self.config.king_safety.enabled {
            return self.evaluate_king_safety_basic(board, player);
        }
        
        let mut total_score = TaperedScore::default();
        
        // Castle structure evaluation
        let castle_score = self.king_safety_evaluator.evaluate_castle_structure(board, player);
        total_score += castle_score * self.config.king_safety.castle_weight;
        
        // Attack analysis
        let attack_score = self.king_safety_evaluator.evaluate_attacks(board, player);
        total_score += attack_score * self.config.king_safety.attack_weight;
        
        // Threat evaluation
        let threat_score = self.king_safety_evaluator.evaluate_threats(board, player);
        total_score += threat_score * self.config.king_safety.threat_weight;
        
        // Apply phase adjustment
        let game_phase = self.calculate_game_phase(board);
        let phase_factor = if game_phase > 128 {
            1.0 // Full weight in opening/middlegame
        } else {
            self.config.king_safety.phase_adjustment // Reduced weight in endgame
        };
        
        total_score * phase_factor
    }
}
```

### 9.2 Performance Considerations

```rust
impl KingSafetyEvaluator {
    pub fn evaluate_fast(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        // Fast evaluation for nodes deep in search tree
        let king_pos = self.find_king_position(board, player)?;
        
        // Quick castle check (only most common patterns)
        let castle_score = self.quick_castle_check(board, player, king_pos);
        
        // Basic attack count
        let attack_score = self.basic_attack_count(board, player, king_pos);
        
        castle_score + attack_score
    }
    
    fn quick_castle_check(&self, board: &BitboardBoard, player: Player, king_pos: Position) -> TaperedScore {
        // Only check Mino and Anaguma patterns for speed
        if let Some(pattern) = self.castle_recognizer.recognize_quick_castle(board, player, king_pos) {
            pattern.score
        } else {
            TaperedScore::default()
        }
    }
}
```

## 10. Testing and Validation

### 10.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mino_castle_recognition() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_mino_castle_position();
        
        let score = evaluator.evaluate_castle_structure(&board, Player::Black);
        assert!(score.mg > 150, "Mino castle should score highly in middlegame");
        assert!(score.eg > 50, "Mino castle should score moderately in endgame");
    }

    #[test]
    fn test_attack_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_attacking_position();
        
        let score = evaluator.evaluate_attacks(&board, Player::Black);
        assert!(score.mg < -100, "Heavy attack should result in negative score");
    }

    #[test]
    fn test_threat_detection() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_tactical_threat_position();
        
        let score = evaluator.evaluate_threats(&board, Player::Black);
        assert!(score.mg < -50, "Tactical threats should be penalized");
    }
}
```

### 10.2 Integration Tests

```rust
#[test]
fn test_king_safety_integration() {
    let evaluator = PositionEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    // Test that king safety evaluation integrates properly
    let total_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
    
    // Verify that king safety contributes to total evaluation
    assert!(total_score.abs() < 10000, "Total evaluation should be reasonable");
}
```

### 10.3 Performance Tests

```rust
#[test]
fn test_king_safety_performance() {
    let evaluator = KingSafetyEvaluator::new();
    let board = BitboardBoard::new();
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = evaluator.evaluate(&board, Player::Black);
    }
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 100, "1000 evaluations should complete in < 100ms");
}
```

## 11. Implementation Plan

### 11.1 Phase 1: Core Infrastructure (Week 1)
1. Create module structure and basic types
2. Implement `KingSafetyEvaluator` struct
3. Create configuration system
4. Add basic integration with existing evaluation

### 11.2 Phase 2: Castle Recognition (Week 2)
1. Implement `CastleRecognizer` and `CastlePattern` types
2. Define Mino, Anaguma, and Yagura patterns
3. Implement pattern matching algorithm
4. Add unit tests for castle recognition

### 11.3 Phase 3: Attack Analysis (Week 3)
1. Implement `AttackAnalyzer` and attack tables
2. Create attack zone evaluation
3. Implement coordination analysis
4. Add attack evaluation tests

### 11.4 Phase 4: Threat Evaluation (Week 4)
1. Implement `ThreatEvaluator` and tactical patterns
2. Create common tactical pattern detectors
3. Implement threat scoring system
4. Add threat evaluation tests

### 11.5 Phase 5: Integration and Optimization (Week 5)
1. Integrate all components into main evaluation
2. Implement performance optimizations
3. Add comprehensive integration tests
4. Performance benchmarking and tuning

### 11.6 Phase 6: Testing and Refinement (Week 6)
1. Extensive testing with real game positions
2. Parameter tuning and optimization
3. Documentation and code review
4. Final integration testing

## 12. Success Metrics

### 12.1 Functional Metrics
- **Castle Recognition Accuracy**: > 95% correct identification of known castle patterns
- **Attack Evaluation Precision**: Correlation > 0.8 with expert evaluations
- **Threat Detection Rate**: > 90% detection of common tactical patterns

### 12.2 Performance Metrics
- **Evaluation Speed**: < 0.1ms per position evaluation
- **Memory Usage**: < 1MB additional memory overhead
- **Search Impact**: < 5% reduction in nodes per second

### 12.3 Quality Metrics
- **Engine Strength**: Measurable improvement in defensive play
- **Tactical Awareness**: Better handling of king safety in tactical positions
- **Strategic Understanding**: Improved castle building and maintenance

## 13. Future Enhancements

### 13.1 Advanced Features
1. **Machine Learning Integration**: Use neural networks for pattern recognition
2. **Dynamic Pattern Learning**: Automatically discover new castle patterns
3. **Opponent Modeling**: Adapt evaluation based on opponent's style
4. **Endgame Specialization**: Specialized king safety for endgame positions

### 13.2 Performance Optimizations
1. **SIMD Instructions**: Vectorized attack table lookups
2. **Cache Optimization**: Better memory access patterns
3. **Parallel Evaluation**: Multi-threaded threat analysis
4. **Incremental Updates**: Only recalculate changed components

### 13.3 Extensibility
1. **Plugin Architecture**: Allow external castle pattern definitions
2. **Configuration UI**: User interface for tuning parameters
3. **Analysis Tools**: Visualization of king safety evaluation
4. **Export/Import**: Save and load evaluation configurations

## 14. Conclusion

This design provides a comprehensive framework for implementing advanced king safety evaluation in the Shogi engine. By combining castle recognition, attack analysis, and threat evaluation, the engine will gain a much deeper understanding of king safety dynamics.

The modular architecture ensures maintainability and extensibility, while the performance considerations guarantee that the enhanced evaluation doesn't significantly impact search speed. The phased implementation plan provides a clear path to completion, with measurable success metrics to validate the implementation.

The integration with the existing `TaperedScore` system ensures compatibility with the current evaluation framework, while the configuration system allows for fine-tuning and optimization. This design establishes a solid foundation for advanced king safety evaluation that can be extended and improved over time.
