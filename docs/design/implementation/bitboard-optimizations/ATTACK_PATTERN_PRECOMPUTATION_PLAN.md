# Attack Pattern Precomputation Implementation Plan

## Overview

This document outlines the implementation plan for Attack Pattern Precomputation, a critical optimization that eliminates runtime calculations by precomputing all possible attack patterns for non-sliding pieces. This optimization provides high impact by eliminating runtime calculations and improving move generation performance.

## Current State Analysis

### Existing Implementation
- **Current Method**: Attack patterns calculated on-demand during move generation
- **Performance Issues**: Repeated calculations for the same piece positions
- **Memory Usage**: Minimal (no precomputed tables)
- **Code Location**: `src/moves.rs` - piece-specific move generation

### Performance Bottlenecks
- King, knight, gold, and silver attack patterns recalculated every time
- No caching of attack patterns
- Inefficient for positions with multiple pieces of the same type
- Redundant calculations during search

## Technical Specification

### Attack Pattern Types
1. **King Attacks**: 8 directions from each square
2. **Knight Attacks**: 2 directions from each square (forward only)
3. **Gold Attacks**: 6 directions from each square
4. **Silver Attacks**: 6 directions from each square

### Memory Requirements
- **King Attacks**: 81 positions × 8 directions = 648 patterns
- **Knight Attacks**: 81 positions × 2 directions = 162 patterns
- **Gold Attacks**: 81 positions × 6 directions = 486 patterns
- **Silver Attacks**: 81 positions × 6 directions = 486 patterns
- **Total Memory**: ~50KB for all attack patterns

### Data Structures

```rust
#[derive(Clone, Copy, Debug)]
struct AttackTables {
    king_attacks: [Bitboard; 81],
    knight_attacks: [Bitboard; 81],
    gold_attacks: [Bitboard; 81],
    silver_attacks: [Bitboard; 81],
}

#[derive(Clone, Copy, Debug)]
struct PieceAttackTable {
    attacks: [Bitboard; 81],
    piece_type: PieceType,
}
```

## Implementation Phases

### Phase 1: Attack Pattern Generation (Week 1)

#### 1.1 King Attack Pattern Generator
**File**: `src/bitboards/king_attacks.rs`

**Responsibilities**:
- Generate king attack patterns for all 81 squares
- Handle board boundaries correctly
- Optimize for memory access patterns

**Key Functions**:
```rust
fn generate_king_attacks(square: u8) -> Bitboard
fn generate_all_king_attacks() -> [Bitboard; 81]
fn is_valid_square(square: u8) -> bool
fn get_king_directions() -> [(i8, i8); 8]
```

**Implementation Details**:
```rust
const KING_DIRECTIONS: [(i8, i8); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),           (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

fn generate_king_attacks(square: u8) -> Bitboard {
    let (file, rank) = square_to_coords(square);
    let mut attacks = 0;
    
    for &(df, dr) in &KING_DIRECTIONS {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if is_valid_coords(new_file, new_rank) {
            let new_square = coords_to_square(new_file as u8, new_rank as u8);
            attacks |= 1 << new_square;
        }
    }
    
    attacks
}
```

#### 1.2 Knight Attack Pattern Generator
**File**: `src/bitboards/knight_attacks.rs`

**Responsibilities**:
- Generate knight attack patterns for all 81 squares
- Handle Shogi-specific knight movement (forward only)
- Account for board boundaries

**Key Functions**:
```rust
fn generate_knight_attacks(square: u8, player: Player) -> Bitboard
fn generate_all_knight_attacks() -> ([Bitboard; 81], [Bitboard; 81])
fn get_knight_directions(player: Player) -> [(i8, i8); 2]
```

**Implementation Details**:
```rust
const KNIGHT_DIRECTIONS: [(i8, i8); 2] = [(-2, -1), (-2, 1)];

fn generate_knight_attacks(square: u8, player: Player) -> Bitboard {
    let (file, rank) = square_to_coords(square);
    let mut attacks = 0;
    
    for &(df, dr) in &KNIGHT_DIRECTIONS {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if is_valid_coords(new_file, new_rank) {
            let new_square = coords_to_square(new_file as u8, new_rank as u8);
            attacks |= 1 << new_square;
        }
    }
    
    attacks
}
```

#### 1.3 Gold Attack Pattern Generator
**File**: `src/bitboards/gold_attacks.rs`

**Responsibilities**:
- Generate gold attack patterns for all 81 squares
- Handle Shogi-specific gold movement
- Account for board boundaries

**Key Functions**:
```rust
fn generate_gold_attacks(square: u8, player: Player) -> Bitboard
fn generate_all_gold_attacks() -> ([Bitboard; 81], [Bitboard; 81])
fn get_gold_directions(player: Player) -> [(i8, i8); 6]
```

**Implementation Details**:
```rust
const GOLD_DIRECTIONS: [(i8, i8); 6] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),  (0, 1),
    (1, 0),
];

fn generate_gold_attacks(square: u8, player: Player) -> Bitboard {
    let (file, rank) = square_to_coords(square);
    let mut attacks = 0;
    
    for &(df, dr) in &GOLD_DIRECTIONS {
        let new_file = file as i8 + df;
        let new_rank = rank as i8 + dr;
        
        if is_valid_coords(new_file, new_rank) {
            let new_square = coords_to_square(new_file as u8, new_rank as u8);
            attacks |= 1 << new_square;
        }
    }
    
    attacks
}
```

#### 1.4 Silver Attack Pattern Generator
**File**: `src/bitboards/silver_attacks.rs`

**Responsibilities**:
- Generate silver attack patterns for all 81 squares
- Handle Shogi-specific silver movement
- Account for board boundaries

**Key Functions**:
```rust
fn generate_silver_attacks(square: u8, player: Player) -> Bitboard
fn generate_all_silver_attacks() -> ([Bitboard; 81], [Bitboard; 81])
fn get_silver_directions(player: Player) -> [(i8, i8); 6]
```

### Phase 2: Attack Table Construction (Week 1-2)

#### 2.1 Attack Table Builder
**File**: `src/bitboards/attack_table_builder.rs`

**Responsibilities**:
- Build complete attack tables for all piece types
- Handle initialization and validation
- Provide lookup functionality

**Key Functions**:
```rust
impl AttackTables {
    fn new() -> Self
    fn build_all_tables() -> Self
    fn get_king_attacks(&self, square: u8) -> Bitboard
    fn get_knight_attacks(&self, square: u8, player: Player) -> Bitboard
    fn get_gold_attacks(&self, square: u8, player: Player) -> Bitboard
    fn get_silver_attacks(&self, square: u8, player: Player) -> Bitboard
}
```

**Implementation Details**:
```rust
impl AttackTables {
    fn new() -> Self {
        let mut tables = Self {
            king_attacks: [0; 81],
            knight_attacks: [0; 81],
            gold_attacks: [0; 81],
            silver_attacks: [0; 81],
        };
        
        tables.precompute_all_attacks();
        tables
    }
    
    fn precompute_all_attacks(&mut self) {
        for square in 0..81 {
            self.king_attacks[square] = generate_king_attacks(square);
            self.knight_attacks[square] = generate_knight_attacks(square, Player::Black);
            self.gold_attacks[square] = generate_gold_attacks(square, Player::Black);
            self.silver_attacks[square] = generate_silver_attacks(square, Player::Black);
        }
    }
}
```

#### 2.2 Table Validation
**File**: `src/bitboards/table_validator.rs`

**Responsibilities**:
- Validate attack table correctness
- Compare with reference implementation
- Performance testing

**Key Functions**:
```rust
fn validate_attack_tables(tables: &AttackTables) -> bool
fn benchmark_attack_tables() -> PerformanceComparison
fn test_all_positions() -> ValidationResult
```

### Phase 3: Integration (Week 2)

#### 3.1 Move Generation Integration
**File**: `src/moves.rs` (modifications)

**Changes**:
- Replace runtime calculations with table lookups
- Update piece-specific move generation
- Maintain backward compatibility during transition

**Key Functions**:
```rust
fn generate_king_moves_precomputed(board: &BitboardBoard, square: u8) -> Vec<Move>
fn generate_knight_moves_precomputed(board: &BitboardBoard, square: u8, player: Player) -> Vec<Move>
fn generate_gold_moves_precomputed(board: &BitboardBoard, square: u8, player: Player) -> Vec<Move>
fn generate_silver_moves_precomputed(board: &BitboardBoard, square: u8, player: Player) -> Vec<Move>
```

#### 3.2 Performance Optimization
**File**: `src/bitboards/optimized_attacks.rs`

**Responsibilities**:
- Optimize lookup performance
- Cache frequently accessed patterns
- Minimize memory access

**Key Functions**:
```rust
fn fast_king_lookup(square: u8) -> Bitboard
fn fast_knight_lookup(square: u8, player: Player) -> Bitboard
fn fast_gold_lookup(square: u8, player: Player) -> Bitboard
fn fast_silver_lookup(square: u8, player: Player) -> Bitboard
```

## File Structure

```
src/bitboards/
├── mod.rs
├── king_attacks.rs
├── knight_attacks.rs
├── gold_attacks.rs
├── silver_attacks.rs
├── attack_table_builder.rs
├── table_validator.rs
├── optimized_attacks.rs
└── tests/
    ├── king_attack_tests.rs
    ├── knight_attack_tests.rs
    ├── gold_attack_tests.rs
    ├── silver_attack_tests.rs
    ├── table_builder_tests.rs
    └── integration_tests.rs
```

## Testing Strategy

### Unit Tests
- Individual attack pattern generation
- Table lookup correctness
- Edge case handling (corners, edges)
- Cross-platform compatibility

### Integration Tests
- Full move generation with precomputed attacks
- Performance comparison with runtime calculation
- Memory usage validation
- Correctness across all board positions

### Performance Tests
- Attack lookup speed benchmarks
- Memory access pattern analysis
- Comparison with reference implementations
- Cache hit rate optimization

## Performance Targets

### Speed Improvements
- **King Move Generation**: 5-10x faster than runtime calculation
- **Knight Move Generation**: 5-10x faster than runtime calculation
- **Gold Move Generation**: 5-10x faster than runtime calculation
- **Silver Move Generation**: 5-10x faster than runtime calculation

### Memory Usage
- **Table Size**: ~50KB for all attack patterns
- **Initialization Time**: < 1ms for complete table generation
- **Lookup Time**: < 1 CPU cycle per attack lookup

## Risk Mitigation

### Technical Risks
1. **Memory Usage**: Monitor table sizes and optimize if needed
2. **Initialization Time**: Ensure fast table generation
3. **Correctness**: Validate all attack patterns thoroughly

### Mitigation Strategies
- Comprehensive testing with all possible board positions
- Performance regression testing
- Gradual rollout with feature flags
- Fallback to runtime calculation if tables fail

## Success Criteria

### Functional Requirements
- [ ] All attack patterns generated correctly
- [ ] No performance regressions in move generation
- [ ] Memory usage within specified limits
- [ ] Backward compatibility maintained

### Performance Requirements
- [ ] 5-10x speed improvement for non-sliding piece generation
- [ ] < 1ms initialization time
- [ ] < 1 CPU cycle per attack lookup
- [ ] Memory usage < 100KB total

### Quality Requirements
- [ ] 100% test coverage for attack pattern code
- [ ] No memory leaks or performance issues
- [ ] Clean, maintainable code structure
- [ ] Comprehensive documentation

## Implementation Timeline

### Week 1
- **Days 1-2**: King and knight attack pattern generation
- **Days 3-4**: Gold and silver attack pattern generation
- **Days 5-7**: Attack table construction and testing

### Week 2
- **Days 1-2**: Integration with move generation
- **Days 3-4**: Performance optimization
- **Days 5-7**: Testing, validation, and documentation

## Dependencies

### Internal Dependencies
- `src/moves.rs` - Move generation system
- `src/bitboards.rs` - Existing bitboard implementation
- `src/types.rs` - Piece and position types

### External Dependencies
- No additional external dependencies required
- Uses standard Rust library features

## Future Enhancements

### Potential Improvements
1. **Compressed Tables**: Use bit packing to reduce memory usage
2. **SIMD Optimization**: Vectorized attack pattern generation
3. **Lazy Loading**: Load attack tables on demand
4. **Custom Allocators**: Optimize memory allocation patterns

### Integration Opportunities
- Integration with magic bitboards
- Caching of frequently accessed patterns
- Parallel initialization of attack tables
- Integration with move ordering heuristics

## Conclusion

The Attack Pattern Precomputation implementation provides essential performance improvements for non-sliding piece move generation, which are fundamental to the Shogi engine's move generation system. The precomputed tables eliminate runtime calculations and provide significant performance gains while maintaining code clarity and correctness.

The expected 5-10x performance improvement for non-sliding piece generation will contribute significantly to overall engine performance and provide a solid foundation for future optimizations.
