# Attack Pattern Precomputation Design

## Overview

This document outlines the design for implementing Attack Pattern Precomputation as part of the bitboard optimization strategy. The goal is to precompute all attack patterns for non-sliding pieces (King, Knight, Gold, Silver, and promoted pieces) at initialization time to eliminate runtime calculations and improve move generation performance.

## Current State Analysis

### Existing Implementation
- **Attack Pattern Generation**: Currently handled by `AttackGenerator` in `src/bitboards/magic/attack_generator.rs`
- **Piece Types**: 14 different piece types defined in `src/types.rs`
- **Board Representation**: 9x9 board (81 squares) using bitboards (u128)
- **Magic Bitboards**: Already implemented for sliding pieces (Rook, Bishop)

### Current Performance Issues
1. **Runtime Calculations**: Attack patterns calculated on-demand for each move
2. **Repeated Work**: Same attack patterns calculated multiple times
3. **Cache Misses**: No precomputed lookup tables for non-sliding pieces

## Design Goals

### Primary Objectives
1. **Performance**: Eliminate runtime attack pattern calculations
2. **Memory Efficiency**: Use compact lookup tables (~50KB total)
3. **Integration**: Seamlessly integrate with existing bitboard system
4. **Maintainability**: Clean, well-documented code structure

### Performance Targets
- **Memory Usage**: ~50KB for all piece attack patterns
- **Lookup Time**: O(1) constant time for attack pattern retrieval
- **Initialization**: <100ms for all precomputation
- **Performance Gain**: 2-3x faster move generation for non-sliding pieces

## Technical Design

### Data Structures

#### AttackTables Structure
```rust
#[repr(C, align(64))] // Cache line alignment
pub struct AttackTables {
    // King attacks: 81 positions × 8 directions = 648 patterns
    king_attacks: [Bitboard; 81],
    
    // Knight attacks: 81 positions × 2 directions = 162 patterns  
    knight_attacks: [Bitboard; 81],
    
    // Gold attacks: 81 positions × 6 directions = 486 patterns
    gold_attacks: [Bitboard; 81],
    
    // Silver attacks: 81 positions × 5 directions = 405 patterns
    silver_attacks: [Bitboard; 81],
    
    // Promoted piece attacks (same as Gold)
    promoted_pawn_attacks: [Bitboard; 81],
    promoted_lance_attacks: [Bitboard; 81],
    promoted_knight_attacks: [Bitboard; 81],
    promoted_silver_attacks: [Bitboard; 81],
    
    // Promoted sliding pieces (King-like moves + original sliding)
    promoted_bishop_attacks: [Bitboard; 81],
    promoted_rook_attacks: [Bitboard; 81],
    
    // Metadata for validation and debugging
    _metadata: AttackTablesMetadata,
}

#[derive(Debug, Clone)]
pub struct AttackTablesMetadata {
    pub initialization_time: std::time::Duration,
    pub memory_usage_bytes: usize,
    pub pattern_counts: [usize; 14], // Count per piece type
    pub validation_passed: bool,
}
```

#### Attack Pattern Generator
```rust
pub struct AttackPatternGenerator {
    /// Cache for generated patterns during initialization
    pattern_cache: HashMap<(u8, PieceType), Bitboard>,
    
    /// Direction vectors for each piece type
    direction_cache: HashMap<PieceType, Vec<Direction>>,
    
    /// Validation statistics
    validation_stats: ValidationStats,
}

#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_patterns_generated: usize,
    pub validation_errors: usize,
    pub average_pattern_size: f32,
    pub edge_case_count: usize,
}
```

### Attack Pattern Calculation

#### Direction Vectors
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction {
    pub row_delta: i8,
    pub col_delta: i8,
}

impl Direction {
    pub fn new(row_delta: i8, col_delta: i8) -> Self {
        Self { row_delta, col_delta }
    }
    
    pub fn apply(&self, square: u8) -> Option<u8> {
        let row = (square / 9) as i8;
        let col = (square % 9) as i8;
        
        let new_row = row + self.row_delta;
        let new_col = col + self.col_delta;
        
        if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
            Some((new_row * 9 + new_col) as u8)
        } else {
            None
        }
    }
}
```

#### Piece-Specific Attack Patterns

**King Attacks (8 directions)**:
```rust
const KING_DIRECTIONS: [Direction; 8] = [
    Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
    Direction::new(0, -1),                        Direction::new(0, 1),
    Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
];
```

**Knight Attacks (2 directions, player-dependent)**:
```rust
fn get_knight_directions(player: Player) -> [Direction; 2] {
    match player {
        Player::Black => [
            Direction::new(2, -1),  // Forward-left
            Direction::new(2, 1),   // Forward-right
        ],
        Player::White => [
            Direction::new(-2, -1), // Forward-left (from white perspective)
            Direction::new(-2, 1),  // Forward-right (from white perspective)
        ],
    }
}
```

**Gold Attacks (6 directions, player-dependent)**:
```rust
fn get_gold_directions(player: Player) -> [Direction; 6] {
    match player {
        Player::Black => [
            Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
            Direction::new(0, -1),  Direction::new(0, 1),  Direction::new(1, 0),
        ],
        Player::White => [
            Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
            Direction::new(0, -1),  Direction::new(0, 1),  Direction::new(-1, 0),
        ],
    }
}
```

**Silver Attacks (5 directions, player-dependent)**:
```rust
fn get_silver_directions(player: Player) -> [Direction; 5] {
    match player {
        Player::Black => [
            Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
            Direction::new(1, -1),  Direction::new(1, 1),
        ],
        Player::White => [
            Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
            Direction::new(-1, -1), Direction::new(-1, 1),
        ],
    }
}
```

### Implementation Strategy

#### Phase 1: Core Infrastructure
1. **AttackTables Structure**: Define the main data structure
2. **AttackPatternGenerator**: Create pattern generation logic
3. **Direction System**: Implement direction vectors and application
4. **Basic Integration**: Connect with existing bitboard system

#### Phase 2: Pattern Generation
1. **King Patterns**: Generate all 81 king attack patterns
2. **Knight Patterns**: Generate player-dependent knight patterns
3. **Gold/Silver Patterns**: Generate player-dependent gold/silver patterns
4. **Promoted Patterns**: Generate promoted piece patterns

#### Phase 3: Optimization & Validation
1. **Memory Optimization**: Ensure cache-friendly memory layout
2. **Validation System**: Verify pattern correctness
3. **Performance Testing**: Benchmark against current implementation
4. **Integration Testing**: Ensure compatibility with existing code

#### Phase 4: Advanced Features
1. **Edge Case Handling**: Handle board edge cases properly
2. **Performance Monitoring**: Add performance metrics
3. **Documentation**: Complete API documentation
4. **Error Handling**: Robust error handling and recovery

### Integration Points

#### BitboardBoard Integration
```rust
impl BitboardBoard {
    /// Get precomputed attack pattern for a piece
    pub fn get_attack_pattern(&self, square: Position, piece_type: PieceType, player: Player) -> Bitboard {
        self.attack_tables.get_attack_pattern(square.to_u8(), piece_type, player)
    }
    
    /// Check if a square is attacked by a piece
    pub fn is_square_attacked(&self, from: Position, to: Position, piece_type: PieceType, player: Player) -> bool {
        let attacks = self.get_attack_pattern(from, piece_type, player);
        is_bit_set(attacks, to)
    }
}
```

#### Move Generation Integration
```rust
impl MoveGenerator {
    /// Generate moves using precomputed attack patterns
    pub fn generate_moves_for_piece(&self, board: &BitboardBoard, piece: Piece, position: Position) -> Vec<Move> {
        let attacks = board.get_attack_pattern(position, piece.piece_type, piece.player);
        self.convert_attacks_to_moves(attacks, piece, position, board)
    }
}
```

### Memory Layout Optimization

#### Cache-Friendly Design
- **Array of Structs (AoS)**: Group related attack patterns together
- **Cache Line Alignment**: Align structures to 64-byte cache lines
- **Sequential Access**: Arrange data for sequential memory access patterns
- **Prefetching**: Enable hardware prefetching for common access patterns

#### Memory Usage Breakdown
```
King attacks:        81 × 16 bytes = 1,296 bytes
Knight attacks:      81 × 16 bytes = 1,296 bytes  
Gold attacks:        81 × 16 bytes = 1,296 bytes
Silver attacks:      81 × 16 bytes = 1,296 bytes
Promoted attacks:    6 × 81 × 16 bytes = 7,776 bytes
Total:                              ≈ 12.5 KB
```

### Performance Considerations

#### Initialization Performance
- **Parallel Generation**: Use parallel processing for pattern generation
- **Incremental Loading**: Load patterns as needed
- **Caching Strategy**: Cache frequently accessed patterns
- **Memory Pre-allocation**: Pre-allocate all required memory

#### Runtime Performance
- **O(1) Lookup**: Constant time attack pattern retrieval
- **Cache Efficiency**: Optimize for CPU cache usage
- **Branch Prediction**: Minimize conditional branches
- **SIMD Opportunities**: Identify SIMD optimization opportunities

### Validation & Testing

#### Correctness Validation
1. **Unit Tests**: Test individual pattern generation
2. **Integration Tests**: Test with existing move generation
3. **Performance Tests**: Benchmark against current implementation
4. **Edge Case Tests**: Test boundary conditions and edge cases

#### Validation Metrics
- **Pattern Accuracy**: Verify all generated patterns are correct
- **Memory Usage**: Ensure memory usage is within targets
- **Performance**: Measure lookup time and initialization time
- **Compatibility**: Ensure compatibility with existing code

### Error Handling

#### Error Types
```rust
#[derive(Debug, Clone)]
pub enum AttackTableError {
    InitializationFailed(String),
    PatternGenerationFailed(u8, PieceType),
    ValidationFailed(String),
    MemoryAllocationFailed(usize),
    InvalidSquare(u8),
    InvalidPieceType(PieceType),
}
```

#### Error Recovery
- **Graceful Degradation**: Fall back to runtime calculation on errors
- **Partial Initialization**: Allow partial table initialization
- **Error Reporting**: Provide detailed error information
- **Recovery Strategies**: Implement automatic recovery mechanisms

### Configuration & Tuning

#### Configuration Options
```rust
pub struct AttackTableConfig {
    pub enable_validation: bool,
    pub parallel_generation: bool,
    pub cache_size: usize,
    pub performance_monitoring: bool,
    pub memory_alignment: usize,
}

impl Default for AttackTableConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            parallel_generation: true,
            cache_size: 1000,
            performance_monitoring: false,
            memory_alignment: 64,
        }
    }
}
```

### Future Extensions

#### Potential Enhancements
1. **Dynamic Patterns**: Support for position-dependent patterns
2. **Compression**: Compress attack patterns to reduce memory usage
3. **Lazy Loading**: Load patterns on-demand for memory-constrained environments
4. **SIMD Optimization**: Use SIMD instructions for pattern operations
5. **Multi-threading**: Parallel pattern generation and lookup

#### Compatibility Considerations
- **API Stability**: Maintain backward compatibility
- **Version Management**: Handle different table versions
- **Migration Path**: Provide migration tools for existing code
- **Documentation**: Keep documentation up-to-date

## Implementation Timeline

### Week 1: Core Infrastructure
- [ ] Define AttackTables structure
- [ ] Implement AttackPatternGenerator
- [ ] Create direction system
- [ ] Basic integration with BitboardBoard

### Week 2: Pattern Generation
- [ ] Implement king attack patterns
- [ ] Implement knight attack patterns
- [ ] Implement gold/silver attack patterns
- [ ] Implement promoted piece patterns

### Week 3: Optimization & Testing
- [ ] Memory layout optimization
- [ ] Performance benchmarking
- [ ] Validation system implementation
- [ ] Integration testing

### Week 4: Polish & Documentation
- [ ] Error handling implementation
- [ ] Performance monitoring
- [ ] Complete documentation
- [ ] Final integration testing

## Success Metrics

### Performance Metrics
- **Lookup Time**: <1ns per attack pattern lookup
- **Initialization Time**: <100ms for complete initialization
- **Memory Usage**: <50KB total memory usage
- **Performance Gain**: 2-3x faster move generation

### Quality Metrics
- **Test Coverage**: >95% code coverage
- **Validation Pass Rate**: 100% validation success
- **Error Rate**: <0.1% error rate in production
- **Documentation Coverage**: 100% public API documented

### Integration Metrics
- **API Compatibility**: 100% backward compatibility
- **Performance Regression**: 0% performance regression
- **Memory Regression**: <10% memory usage increase
- **Build Time**: <5% build time increase

## Conclusion

The Attack Pattern Precomputation design provides a comprehensive approach to optimizing non-sliding piece move generation in the Shogi engine. By precomputing all attack patterns at initialization time, we can achieve significant performance improvements while maintaining code clarity and maintainability.

The design emphasizes:
- **Performance**: O(1) lookup time with optimized memory layout
- **Memory Efficiency**: Compact representation with cache-friendly design
- **Integration**: Seamless integration with existing bitboard system
- **Maintainability**: Clean, well-documented code structure
- **Extensibility**: Framework for future enhancements

This optimization is expected to provide a 2-3x performance improvement for non-sliding piece move generation, contributing significantly to the overall engine performance goals.
