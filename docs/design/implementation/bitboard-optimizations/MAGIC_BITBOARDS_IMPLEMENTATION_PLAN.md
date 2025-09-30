# Magic Bitboards Implementation Plan

## Overview

This document outlines the implementation plan for Magic Bitboards, a critical optimization for sliding piece move generation in the Shogi engine. Magic bitboards use precomputed lookup tables with carefully chosen magic numbers to hash occupied squares, providing 3-5x faster sliding piece move generation.

## Current State Analysis

### Existing Implementation
- **Current Method**: Basic ray-casting for rook and bishop moves
- **Performance Issues**: Runtime calculation of attack patterns
- **Memory Usage**: Minimal (no precomputed tables)
- **Code Location**: `src/moves.rs` - sliding piece move generation

### Performance Bottlenecks
- Sliding piece move generation is called frequently during search
- Ray-casting requires multiple bit operations per move
- No caching of attack patterns
- Inefficient for positions with many sliding pieces

## Technical Specification

### Magic Bitboard Algorithm
1. **Magic Number**: Carefully chosen 64-bit constant for each square
2. **Mask**: Bitboard representing relevant occupied squares
3. **Shift**: Number of bits to shift the hash result
4. **Attack Table**: Precomputed attack patterns for all blocker configurations

### Data Structures

```rust
#[derive(Clone, Copy, Debug)]
struct MagicBitboard {
    magic_number: u64,
    mask: Bitboard,
    shift: u8,
    attacks: Vec<Bitboard>,
}

#[derive(Clone, Copy, Debug)]
struct MagicTable {
    rook_magics: [MagicBitboard; 81],
    bishop_magics: [MagicBitboard; 81],
}
```

### Memory Requirements
- **Rook Attacks**: ~2MB (81 squares × ~4096 entries × 8 bytes)
- **Bishop Attacks**: ~1MB (81 squares × ~512 entries × 8 bytes)
- **Total**: ~3MB for complete magic bitboard tables

## Implementation Phases

### Phase 1: Magic Number Generation (Week 1)

#### 1.1 Magic Number Finder
**File**: `src/bitboards/magic_finder.rs`

**Responsibilities**:
- Generate magic numbers for each square
- Validate magic number uniqueness
- Handle edge cases (corners, edges)

**Key Functions**:
```rust
fn find_magic_number(square: u8, piece_type: PieceType) -> Option<u64>
fn validate_magic_number(magic: u64, square: u8, piece_type: PieceType) -> bool
fn generate_all_magic_numbers() -> (Vec<u64>, Vec<u64>)
```

**Testing Strategy**:
- Unit tests for magic number validation
- Property-based testing for uniqueness
- Performance benchmarks for generation time

#### 1.2 Attack Pattern Generator
**File**: `src/bitboards/attack_generator.rs`

**Responsibilities**:
- Generate all possible attack patterns for each square
- Handle different piece types (rook, bishop)
- Account for board boundaries

**Key Functions**:
```rust
fn generate_rook_attacks(square: u8, blockers: Bitboard) -> Bitboard
fn generate_bishop_attacks(square: u8, blockers: Bitboard) -> Bitboard
fn generate_all_attack_patterns(square: u8, piece_type: PieceType) -> Vec<Bitboard>
```

### Phase 2: Magic Table Construction (Week 1-2)

#### 2.1 Magic Table Builder
**File**: `src/bitboards/magic_table.rs`

**Responsibilities**:
- Build complete magic bitboard tables
- Handle table initialization and validation
- Provide lookup functionality

**Key Functions**:
```rust
impl MagicTable {
    fn new() -> Self
    fn build_rook_table() -> [MagicBitboard; 81]
    fn build_bishop_table() -> [MagicBitboard; 81]
    fn get_rook_attacks(&self, square: u8, occupied: Bitboard) -> Bitboard
    fn get_bishop_attacks(&self, square: u8, occupied: Bitboard) -> Bitboard
}
```

#### 2.2 Table Validation
**File**: `src/bitboards/table_validator.rs`

**Responsibilities**:
- Validate magic table correctness
- Compare with reference implementation
- Performance testing

**Key Functions**:
```rust
fn validate_magic_table(table: &MagicTable) -> bool
fn benchmark_magic_vs_raycast() -> PerformanceComparison
fn test_all_positions() -> ValidationResult
```

### Phase 3: Integration (Week 2)

#### 3.1 Move Generation Integration
**File**: `src/moves.rs` (modifications)

**Changes**:
- Replace ray-casting with magic bitboard lookups
- Update sliding piece move generation
- Maintain backward compatibility during transition

**Key Functions**:
```rust
fn generate_rook_moves_magic(board: &BitboardBoard, square: u8) -> Vec<Move>
fn generate_bishop_moves_magic(board: &BitboardBoard, square: u8) -> Vec<Move>
fn generate_sliding_moves_magic(board: &BitboardBoard, piece: Piece) -> Vec<Move>
```

#### 3.2 Performance Optimization
**File**: `src/bitboards/optimized_lookup.rs`

**Responsibilities**:
- Optimize lookup performance
- Cache frequently accessed patterns
- Minimize memory access

**Key Functions**:
```rust
fn fast_rook_lookup(square: u8, occupied: Bitboard) -> Bitboard
fn fast_bishop_lookup(square: u8, occupied: Bitboard) -> Bitboard
fn prefetch_attacks(square: u8, piece_type: PieceType)
```

## File Structure

```
src/bitboards/
├── mod.rs
├── magic_finder.rs
├── attack_generator.rs
├── magic_table.rs
├── table_validator.rs
├── optimized_lookup.rs
└── tests/
    ├── magic_finder_tests.rs
    ├── attack_generator_tests.rs
    ├── magic_table_tests.rs
    └── integration_tests.rs
```

## Testing Strategy

### Unit Tests
- Magic number generation and validation
- Attack pattern generation correctness
- Table lookup accuracy
- Edge case handling

### Integration Tests
- Full move generation with magic bitboards
- Performance comparison with ray-casting
- Memory usage validation
- Correctness across all board positions

### Performance Tests
- Move generation speed benchmarks
- Memory access pattern analysis
- Cache hit rate optimization
- Comparison with reference implementations

## Performance Targets

### Speed Improvements
- **Rook Move Generation**: 3-5x faster than ray-casting
- **Bishop Move Generation**: 3-5x faster than ray-casting
- **Overall Move Generation**: 2-3x improvement for positions with sliding pieces

### Memory Usage
- **Initialization Time**: < 1 second for complete table generation
- **Memory Footprint**: ~3MB for all magic bitboard tables
- **Lookup Time**: < 10 CPU cycles per attack lookup

## Risk Mitigation

### Technical Risks
1. **Magic Number Collisions**: Implement robust validation and fallback
2. **Memory Usage**: Monitor and optimize table sizes
3. **Initialization Time**: Consider lazy loading or precomputed tables

### Mitigation Strategies
- Comprehensive testing with all possible board positions
- Performance regression testing
- Gradual rollout with feature flags
- Fallback to ray-casting if magic bitboards fail

## Success Criteria

### Functional Requirements
- [ ] All sliding piece moves generated correctly
- [ ] No performance regressions in move generation
- [ ] Memory usage within specified limits
- [ ] Backward compatibility maintained

### Performance Requirements
- [ ] 3-5x speed improvement for sliding piece generation
- [ ] < 1 second initialization time
- [ ] < 10 CPU cycles per attack lookup
- [ ] Memory usage < 4MB total

### Quality Requirements
- [ ] 100% test coverage for magic bitboard code
- [ ] No memory leaks or performance issues
- [ ] Clean, maintainable code structure
- [ ] Comprehensive documentation

## Implementation Timeline

### Week 1
- **Days 1-2**: Magic number generation and validation
- **Days 3-4**: Attack pattern generation
- **Days 5-7**: Magic table construction and testing

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
2. **SIMD Optimization**: Vectorized lookup operations
3. **Lazy Loading**: Load magic tables on demand
4. **Custom Allocators**: Optimize memory allocation patterns

### Integration Opportunities
- Integration with transposition tables
- Caching frequently accessed patterns
- Parallel initialization of magic tables
- Integration with move ordering heuristics

## Conclusion

The Magic Bitboards implementation represents a critical optimization for the Shogi engine, providing significant performance improvements for sliding piece move generation. The phased approach ensures thorough testing and validation while maintaining system stability. The expected 3-5x performance improvement will significantly enhance the engine's search capabilities and overall playing strength.
