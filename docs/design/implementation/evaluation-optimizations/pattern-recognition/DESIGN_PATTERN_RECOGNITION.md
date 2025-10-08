# Pattern Recognition Design

## Overview

This document outlines the design for implementing pattern recognition in the Shogi engine. Pattern recognition identifies tactical and positional patterns to provide more accurate position evaluation and better game understanding.

## Current State

The engine currently uses basic material counting and simple positional bonuses, missing many important strategic and tactical patterns that strong play requires.

## Design Goals

1. **Comprehensive Coverage**: Recognize all important tactical and positional patterns
2. **Accuracy**: Correctly identify patterns with minimal false positives
3. **Performance**: Fast pattern detection suitable for real-time search
4. **Tunability**: Easy to adjust pattern weights and parameters
5. **Extensibility**: Easy to add new patterns

## Technical Architecture

### 1. Piece-Square Tables

**Purpose**: Provide basic positional evaluation based on piece placement.

**Components**:
- Tables for each piece type
- Symmetric access for both players
- Configuration and loading system
- Efficient lookup

**Implementation**:
```rust
use std::collections::HashMap;
use crate::types::{PieceType, Player, Square};

/// Piece-square tables for positional evaluation
pub struct PieceSquareTables {
    /// Tables indexed by piece type: [square] -> value
    tables: HashMap<PieceType, Vec<i32>>,
}

impl PieceSquareTables {
    /// Create with default tables
    pub fn new() -> Self {
        let mut tables = HashMap::new();
        
        for piece_type in PieceType::all() {
            tables.insert(piece_type, Self::create_default_table(piece_type));
        }
        
        Self { tables }
    }
    
    /// Get positional value for a piece on a square
    pub fn get_value(&self, piece_type: PieceType, square: Square, player: Player) -> i32 {
        let sq_index = if player == Player::Gote {
            80 - square.index() // Flip for opponent
        } else {
            square.index()
        };
        
        if let Some(table) = self.tables.get(&piece_type) {
            table[sq_index]
        } else {
            0
        }
    }
    
    /// Evaluate all pieces on board
    pub fn evaluate(&self, board: &impl BoardTrait) -> i32 {
        let mut score = 0;
        
        for player in [Player::Sente, Player::Gote] {
            let mut player_score = 0;
            
            for piece_type in PieceType::all() {
                for square in board.get_pieces(player, piece_type) {
                    player_score += self.get_value(piece_type, square, player);
                }
            }
            
            if player == board.side_to_move() {
                score += player_score;
            } else {
                score -= player_score;
            }
        }
        
        score
    }
    
    /// Create default table for a piece type
    fn create_default_table(piece_type: PieceType) -> Vec<i32> {
        let mut table = vec![0; 81];
        
        match piece_type {
            PieceType::Pawn => {
                // Encourage pawn advancement
                for square in 0..81 {
                    let rank = square / 9;
                    table[square] = (8 - rank) * 5;
                }
            }
            PieceType::King => {
                // Encourage king safety (stay back)
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    // Prefer corners
                    let corner_bonus = if rank == 0 && (file <= 2 || file >= 6) { 20 } else { 0 };
                    
                    // Penalty for center
                    let center_penalty = if (3..=5).contains(&file) && (3..=5).contains(&rank) {
                        -30
                    } else {
                        0
                    };
                    
                    table[square] = corner_bonus + center_penalty;
                }
            }
            _ => {
                // Default: slight bonus for center
                for square in 0..81 {
                    let file = square % 9;
                    let rank = square / 9;
                    
                    let center_bonus = if (3..=5).contains(&file) && (3..=5).contains(&rank) {
                        10
                    } else {
                        0
                    };
                    
                    table[square] = center_bonus;
                }
            }
        }
        
        table
    }
    
    /// Load custom tables from configuration
    pub fn load_tables(&mut self, config: &PatternConfig) {
        // Load tables from configuration
    }
}
```

### 2. Pawn Structure Analyzer

**Purpose**: Evaluate pawn structure quality including weaknesses and strengths.

**Technical Details**:
- Doubled pawns detection
- Isolated pawns identification
- Passed pawns evaluation
- Pawn chains analysis
- Pawn advancement bonuses

**Implementation**:
```rust
/// Pawn structure analyzer
pub struct PawnStructureAnalyzer {
    doubled_pawn_penalty: i32,
    isolated_pawn_penalty: i32,
    passed_pawn_bonus: [i32; 9], // By rank
    pawn_chain_bonus: i32,
}

impl PawnStructureAnalyzer {
    pub fn new() -> Self {
        Self {
            doubled_pawn_penalty: -20,
            isolated_pawn_penalty: -15,
            passed_pawn_bonus: [0, 5, 10, 20, 40, 80, 160, 320, 640],
            pawn_chain_bonus: 10,
        }
    }
    
    /// Evaluate pawn structure for a player
    pub fn evaluate(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let mut score = 0;
        
        // Get all pawn positions
        let pawns = board.get_pieces(player, PieceType::Pawn);
        
        // Check for doubled pawns
        score += self.evaluate_doubled_pawns(&pawns);
        
        // Check for isolated pawns
        score += self.evaluate_isolated_pawns(&pawns);
        
        // Check for passed pawns
        score += self.evaluate_passed_pawns(board, player, &pawns);
        
        // Check for pawn chains
        score += self.evaluate_pawn_chains(&pawns);
        
        score
    }
    
    /// Detect and penalize doubled pawns
    fn evaluate_doubled_pawns(&self, pawns: &[Square]) -> i32 {
        let mut file_counts = [0u8; 9];
        
        for &pawn in pawns {
            let file = pawn.file();
            file_counts[file as usize] += 1;
        }
        
        let mut penalty = 0;
        for count in file_counts {
            if count > 1 {
                penalty += self.doubled_pawn_penalty * (count - 1) as i32;
            }
        }
        
        penalty
    }
    
    /// Detect and penalize isolated pawns
    fn evaluate_isolated_pawns(&self, pawns: &[Square]) -> i32 {
        let mut files_with_pawns = [false; 9];
        
        for &pawn in pawns {
            files_with_pawns[pawn.file() as usize] = true;
        }
        
        let mut penalty = 0;
        
        for &pawn in pawns {
            let file = pawn.file() as usize;
            let has_neighbor = 
                (file > 0 && files_with_pawns[file - 1]) ||
                (file < 8 && files_with_pawns[file + 1]);
            
            if !has_neighbor {
                penalty += self.isolated_pawn_penalty;
            }
        }
        
        penalty
    }
    
    /// Detect and reward passed pawns
    fn evaluate_passed_pawns(&self, board: &impl BoardTrait, player: Player, pawns: &[Square]) -> i32 {
        let opponent = player.opponent();
        let opponent_pawns = board.get_pieces(opponent, PieceType::Pawn);
        
        let mut bonus = 0;
        
        for &pawn in pawns {
            if self.is_passed_pawn(pawn, player, &opponent_pawns) {
                let rank = if player == Player::Sente {
                    pawn.rank()
                } else {
                    8 - pawn.rank()
                };
                bonus += self.passed_pawn_bonus[rank as usize];
            }
        }
        
        bonus
    }
    
    /// Check if a pawn is passed
    fn is_passed_pawn(&self, pawn: Square, player: Player, opponent_pawns: &[Square]) -> bool {
        let file = pawn.file();
        let rank = pawn.rank();
        
        // Check if any opponent pawn can block
        for &opp_pawn in opponent_pawns {
            let opp_file = opp_pawn.file();
            let opp_rank = opp_pawn.rank();
            
            // Check if opponent pawn is in front and on same or adjacent file
            let in_front = if player == Player::Sente {
                opp_rank > rank
            } else {
                opp_rank < rank
            };
            
            let blocking_file = (opp_file as i8 - file as i8).abs() <= 1;
            
            if in_front && blocking_file {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate pawn chains (pawns protecting each other)
    fn evaluate_pawn_chains(&self, pawns: &[Square]) -> i32 {
        let mut bonus = 0;
        
        for &pawn in pawns {
            // Check if pawn is protected by another pawn
            for &other in pawns {
                if other != pawn && self.is_pawn_protecting(other, pawn) {
                    bonus += self.pawn_chain_bonus;
                    break;
                }
            }
        }
        
        bonus
    }
    
    /// Check if one pawn protects another
    fn is_pawn_protecting(&self, protector: Square, protected: Square) -> bool {
        let file_diff = (protector.file() as i8 - protected.file() as i8).abs();
        let rank_diff = protector.rank() as i8 - protected.rank() as i8;
        
        // Protector must be one rank behind and one file away
        file_diff == 1 && rank_diff == -1
    }
}
```

### 3. King Safety Analyzer

**Purpose**: Evaluate king safety and exposure to attacks.

**Implementation**:
```rust
/// King safety analyzer
pub struct KingSafetyAnalyzer {
    pawn_shield_bonus: i32,
    exposed_king_penalty: i32,
    attack_weight: [i32; 7], // By piece type
    escape_square_bonus: i32,
}

impl KingSafetyAnalyzer {
    pub fn new() -> Self {
        Self {
            pawn_shield_bonus: 15,
            exposed_king_penalty: -30,
            attack_weight: [0, 1, 2, 2, 3, 4, 5], // Indexed by piece type
            escape_square_bonus: 10,
        }
    }
    
    /// Evaluate king safety for a player
    pub fn evaluate(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let mut score = 0;
        
        let king_square = board.king_square(player);
        
        // Evaluate pawn shield
        score += self.evaluate_pawn_shield(board, player, king_square);
        
        // Count attacking pieces
        score -= self.evaluate_attack_pressure(board, player, king_square);
        
        // Evaluate escape squares
        score += self.evaluate_escape_squares(board, player, king_square);
        
        score
    }
    
    /// Evaluate pawn shield around king
    fn evaluate_pawn_shield(&self, board: &impl BoardTrait, player: Player, king_square: Square) -> i32 {
        let pawns = board.get_pieces(player, PieceType::Pawn);
        let mut shield_count = 0;
        
        // Check squares in front of king
        let king_file = king_square.file();
        let king_rank = king_square.rank();
        
        for file_offset in -1..=1 {
            let check_file = king_file as i8 + file_offset;
            if check_file < 0 || check_file > 8 {
                continue;
            }
            
            for rank_offset in 1..=2 {
                let check_rank = if player == Player::Sente {
                    king_rank as i8 + rank_offset
                } else {
                    king_rank as i8 - rank_offset
                };
                
                if check_rank < 0 || check_rank > 8 {
                    continue;
                }
                
                let check_square = Square::new(check_file as u8, check_rank as u8);
                
                if pawns.contains(&check_square) {
                    shield_count += 1;
                }
            }
        }
        
        shield_count * self.pawn_shield_bonus
    }
    
    /// Evaluate attack pressure on king
    fn evaluate_attack_pressure(&self, board: &impl BoardTrait, player: Player, king_square: Square) -> i32 {
        let opponent = player.opponent();
        let mut pressure = 0;
        
        // Count attacking pieces
        for piece_type in PieceType::all() {
            if piece_type == PieceType::King {
                continue;
            }
            
            let pieces = board.get_pieces(opponent, piece_type);
            
            for &piece_square in pieces {
                if board.attacks(piece_square, king_square) {
                    pressure += self.attack_weight[piece_type as usize];
                }
            }
        }
        
        pressure
    }
    
    /// Evaluate escape squares for king
    fn evaluate_escape_squares(&self, board: &impl BoardTrait, player: Player, king_square: Square) -> i32 {
        let mut escape_squares = 0;
        
        // Check all adjacent squares
        for file_offset in -1..=1 {
            for rank_offset in -1..=1 {
                if file_offset == 0 && rank_offset == 0 {
                    continue;
                }
                
                let new_file = king_square.file() as i8 + file_offset;
                let new_rank = king_square.rank() as i8 + rank_offset;
                
                if new_file < 0 || new_file > 8 || new_rank < 0 || new_rank > 8 {
                    continue;
                }
                
                let new_square = Square::new(new_file as u8, new_rank as u8);
                
                // Check if square is safe
                if !board.is_occupied(new_square) && !board.is_attacked_by(new_square, player.opponent()) {
                    escape_squares += 1;
                }
            }
        }
        
        escape_squares * self.escape_square_bonus
    }
}
```

### 4. Piece Coordination Analyzer

**Purpose**: Evaluate how well pieces work together.

**Implementation**:
```rust
/// Piece coordination analyzer
pub struct PieceCoordinationAnalyzer {
    battery_bonus: i32,
    connected_rooks_bonus: i32,
    piece_support_bonus: i32,
}

impl PieceCoordinationAnalyzer {
    pub fn new() -> Self {
        Self {
            battery_bonus: 25,
            connected_rooks_bonus: 20,
            piece_support_bonus: 10,
        }
    }
    
    /// Evaluate piece coordination for a player
    pub fn evaluate(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let mut score = 0;
        
        // Check for batteries (rook + bishop on same line)
        score += self.evaluate_batteries(board, player);
        
        // Check for connected rooks
        score += self.evaluate_connected_rooks(board, player);
        
        // Check for piece support
        score += self.evaluate_piece_support(board, player);
        
        score
    }
    
    /// Detect rook-bishop batteries
    fn evaluate_batteries(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let rooks = board.get_pieces(player, PieceType::Rook);
        let bishops = board.get_pieces(player, PieceType::Bishop);
        
        let mut bonus = 0;
        
        for &rook in rooks {
            for &bishop in bishops {
                if self.is_battery(rook, bishop) {
                    bonus += self.battery_bonus;
                }
            }
        }
        
        bonus
    }
    
    /// Check if two pieces form a battery
    fn is_battery(&self, square1: Square, square2: Square) -> bool {
        // Check if pieces are on same file, rank, or diagonal
        let same_file = square1.file() == square2.file();
        let same_rank = square1.rank() == square2.rank();
        let same_diagonal = (square1.file() as i8 - square2.file() as i8).abs() ==
                           (square1.rank() as i8 - square2.rank() as i8).abs();
        
        same_file || same_rank || same_diagonal
    }
    
    /// Evaluate connected rooks
    fn evaluate_connected_rooks(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let rooks = board.get_pieces(player, PieceType::Rook);
        
        if rooks.len() < 2 {
            return 0;
        }
        
        let mut bonus = 0;
        
        for i in 0..rooks.len() {
            for j in (i+1)..rooks.len() {
                if self.are_rooks_connected(board, rooks[i], rooks[j]) {
                    bonus += self.connected_rooks_bonus;
                }
            }
        }
        
        bonus
    }
    
    /// Check if two rooks are connected
    fn are_rooks_connected(&self, board: &impl BoardTrait, rook1: Square, rook2: Square) -> bool {
        // Check if rooks are on same rank or file with no pieces between
        let same_file = rook1.file() == rook2.file();
        let same_rank = rook1.rank() == rook2.rank();
        
        if !same_file && !same_rank {
            return false;
        }
        
        // Check if path is clear
        board.is_path_clear(rook1, rook2)
    }
    
    /// Evaluate piece support
    fn evaluate_piece_support(&self, board: &impl BoardTrait, player: Player) -> i32 {
        let mut bonus = 0;
        
        for piece_type in PieceType::all() {
            if piece_type == PieceType::King {
                continue;
            }
            
            let pieces = board.get_pieces(player, piece_type);
            
            for &piece in pieces {
                let defenders = board.count_defenders(piece, player);
                bonus += defenders as i32 * self.piece_support_bonus;
            }
        }
        
        bonus
    }
}
```

### 5. Main Pattern Recognizer

**Purpose**: Coordinate all pattern analyzers and provide unified interface.

**Implementation**:
```rust
/// Main pattern recognition system
pub struct PatternRecognizer {
    piece_square_tables: PieceSquareTables,
    pawn_structure: PawnStructureAnalyzer,
    king_safety: KingSafetyAnalyzer,
    piece_coordination: PieceCoordinationAnalyzer,
    weights: PatternWeights,
}

impl PatternRecognizer {
    pub fn new() -> Self {
        Self {
            piece_square_tables: PieceSquareTables::new(),
            pawn_structure: PawnStructureAnalyzer::new(),
            king_safety: KingSafetyAnalyzer::new(),
            piece_coordination: PieceCoordinationAnalyzer::new(),
            weights: PatternWeights::default(),
        }
    }
    
    /// Evaluate all patterns for a position
    pub fn evaluate(&self, board: &impl BoardTrait) -> i32 {
        let mut score = 0;
        
        // Piece-square tables
        score += self.piece_square_tables.evaluate(board) * self.weights.piece_square_weight / 100;
        
        // Pawn structure (for both players)
        let pawn_score = self.pawn_structure.evaluate(board, board.side_to_move()) -
                        self.pawn_structure.evaluate(board, board.side_to_move().opponent());
        score += pawn_score * self.weights.pawn_structure_weight / 100;
        
        // King safety (for both players)
        let king_score = self.king_safety.evaluate(board, board.side_to_move()) -
                        self.king_safety.evaluate(board, board.side_to_move().opponent());
        score += king_score * self.weights.king_safety_weight / 100;
        
        // Piece coordination (for both players)
        let coord_score = self.piece_coordination.evaluate(board, board.side_to_move()) -
                         self.piece_coordination.evaluate(board, board.side_to_move().opponent());
        score += coord_score * self.weights.piece_coordination_weight / 100;
        
        score
    }
}

/// Pattern evaluation weights
#[derive(Debug, Clone)]
pub struct PatternWeights {
    pub piece_square_weight: i32,
    pub pawn_structure_weight: i32,
    pub king_safety_weight: i32,
    pub piece_coordination_weight: i32,
}

impl Default for PatternWeights {
    fn default() -> Self {
        Self {
            piece_square_weight: 100,
            pawn_structure_weight: 100,
            king_safety_weight: 150,
            piece_coordination_weight: 80,
        }
    }
}
```

## Integration Points

### Evaluation Engine Integration

```rust
impl Evaluator {
    fn evaluate_with_patterns(&self, board: &impl BoardTrait) -> i32 {
        let mut score = 0;
        
        // Material evaluation
        score += self.evaluate_material(board);
        
        // Pattern evaluation
        score += self.pattern_recognizer.evaluate(board);
        
        score
    }
}
```

## Performance Considerations

### Computational Complexity
- **Piece-Square Tables**: O(n) where n is number of pieces
- **Pawn Structure**: O(p²) where p is number of pawns
- **King Safety**: O(n) where n is number of attacking pieces
- **Piece Coordination**: O(n²) worst case for piece pairs

### Memory Usage
- **Piece-Square Tables**: ~11KB (14 piece types × 81 squares × 10 bytes)
- **Pattern Weights**: Negligible
- **Temporary Data**: Minimal

### Cache Efficiency
- Sequential access to piece-square tables
- Good spatial locality for adjacent square checks

## Testing Strategy

### Unit Tests
1. **Piece-Square Tables**: Test value lookups
2. **Pawn Structure**: Test pattern detection
3. **King Safety**: Test attack counting
4. **Piece Coordination**: Test coordination bonuses

### Integration Tests
1. **Complete Evaluation**: Test all patterns together
2. **Known Positions**: Validate against test positions
3. **Performance**: Benchmark pattern detection speed

## Configuration Options

```rust
pub struct PatternConfig {
    pub weights: PatternWeights,
    pub enable_piece_squares: bool,
    pub enable_pawn_structure: bool,
    pub enable_king_safety: bool,
    pub enable_piece_coordination: bool,
}
```

## Expected Performance Impact

### Evaluation Accuracy
- **20-30% Improvement**: More accurate position assessment
- **Better Tactical Awareness**: Pattern detection improves tactics
- **Stronger Positional Play**: Better understanding of position quality

### Performance
- **<10% Overhead**: Pattern detection is fast
- **Minimal Memory**: Small additional memory usage

## Future Enhancements

1. **Machine Learning**: Learn optimal pattern weights
2. **Dynamic Patterns**: Position-type specific pattern sets
3. **Advanced Tactics**: More sophisticated tactical pattern recognition
4. **Pattern Caching**: Cache pattern detection results

## Conclusion

The pattern recognition design provides comprehensive position understanding through multiple complementary analyzers. The implementation focuses on:

- Accurate pattern detection
- Fast performance
- Easy tuning and configuration
- Comprehensive coverage of important patterns

This design enables significantly more accurate evaluation and stronger play through better position understanding.

