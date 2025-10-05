# Bit-Scanning Optimization Implementation Plan

## Overview

This document outlines the implementation plan for bit-scanning optimizations in the Shogi engine. Bit-scanning operations are fundamental to bitboard manipulation and occur frequently during move generation and evaluation. The optimization focuses on hardware-accelerated instructions, lookup tables, and specialized algorithms to achieve 10-20% faster bitboard operations.

## Current State Analysis

### Existing Implementation
- **Current Method**: Uses `trailing_zeros()` for bit scanning
- **Performance Issues**: Generic implementation not optimized for bitboard operations
- **Frequency**: Called thousands of times per second during search
- **Code Location**: `src/bitboards.rs` - bitboard manipulation functions

### Performance Bottlenecks
- Generic bit-scanning functions not optimized for 64-bit operations
- No hardware acceleration for population count
- Inefficient bit position determination
- Lack of specialized lookup tables for common operations

## Technical Specification

### Bit-Scanning Operations
1. **Population Count (Popcount)**: Count number of set bits
2. **Bit-Scan Forward (BSF)**: Find first set bit
3. **Bit-Scan Reverse (BSR)**: Find last set bit
4. **Bit Position**: Convert bit index to square coordinates

### Hardware Acceleration
- **x86_64**: `popcnt`, `bsf`, `bsr` instructions
- **ARM**: `clz`, `ctz` instructions
- **Fallback**: Software implementations for unsupported platforms

### Lookup Table Optimization
- **4-bit Tables**: For small bitboard operations
- **De Bruijn Sequences**: For bit position determination
- **Precomputed Masks**: For common bit patterns

## Implementation Phases

### Phase 1: Hardware-Accelerated Functions (Week 1)

#### 1.1 Platform Detection and Fallbacks
**File**: `src/bitboards/platform_detection.rs`

**Responsibilities**:
- Detect CPU capabilities at runtime
- Provide fallback implementations
- Handle different architectures

**Key Functions**:
```rust
#[cfg(target_arch = "x86_64")]
fn has_popcnt() -> bool

#[cfg(target_arch = "x86_64")]
fn has_bmi1() -> bool

fn get_best_popcount_impl() -> PopcountImpl
fn get_best_bitscan_impl() -> BitscanImpl
```

**Testing Strategy**:
- Cross-platform compatibility testing
- Performance benchmarks on different architectures
- Fallback validation

#### 1.2 Hardware-Accelerated Popcount
**File**: `src/bitboards/popcount.rs`

**Responsibilities**:
- Implement hardware-accelerated population count
- Provide software fallbacks
- Optimize for different bitboard sizes

**Key Functions**:
```rust
#[cfg(target_arch = "x86_64")]
fn popcount_hw(bb: Bitboard) -> u32

fn popcount_sw(bb: Bitboard) -> u32
fn popcount_optimized(bb: Bitboard) -> u32
fn popcount_parallel(bb: Bitboard) -> u32
```

**Implementation Details**:
```rust
// Hardware-accelerated population count
#[cfg(target_arch = "x86_64")]
fn popcount_hw(bb: Bitboard) -> u32 {
    unsafe { std::arch::x86_64::_popcnt64(bb as i64) as u32 }
}

// Software fallback using bit manipulation
fn popcount_sw(bb: Bitboard) -> u32 {
    let mut count = 0;
    let mut bits = bb;
    while bits != 0 {
        count += 1;
        bits &= bits - 1; // Clear least significant bit
    }
    count
}
```

#### 1.3 Hardware-Accelerated Bit Scanning
**File**: `src/bitboards/bitscan.rs`

**Responsibilities**:
- Implement bit-scan forward and reverse
- Handle edge cases (zero bitboards)
- Optimize for common patterns

**Key Functions**:
```rust
fn bit_scan_forward(bb: Bitboard) -> Option<u8>
fn bit_scan_reverse(bb: Bitboard) -> Option<u8>
fn bit_scan_forward_hw(bb: Bitboard) -> Option<u8>
fn bit_scan_reverse_hw(bb: Bitboard) -> Option<u8>
```

**Implementation Details**:
```rust
#[cfg(target_arch = "x86_64")]
fn bit_scan_forward_hw(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        None
    } else {
        Some(unsafe { std::arch::x86_64::_tzcnt_u64(bb) as u8 })
    }
}

#[cfg(target_arch = "x86_64")]
fn bit_scan_reverse_hw(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        None
    } else {
        Some(63 - unsafe { std::arch::x86_64::_lzcnt_u64(bb) as u8 })
    }
}
```

### Phase 2: Lookup Table Optimization (Week 1-2)

## Technical Deep Dive: Core Optimization Techniques

### 4-bit Lookup Tables

#### Mathematical Foundation
4-bit lookup tables are based on the principle of **divide-and-conquer** optimization. Instead of processing all 64 or 128 bits of a bitboard at once, we decompose the operation into smaller, manageable chunks of 4 bits each. This approach leverages several key advantages:

**Memory Efficiency**: A 4-bit lookup table requires only 16 entries (2^4 = 16 possible bit patterns), making it extremely cache-friendly and memory-efficient.

**Computational Complexity**: Instead of O(n) operations for n bits, we achieve O(n/4) operations with constant-time lookups for each 4-bit chunk.

**Cache Locality**: The small table size (16 bytes) ensures it fits entirely in L1 cache, providing near-instant access times.

#### How 4-bit Tables Work

**Step 1: Pattern Enumeration**
For a 4-bit value, there are exactly 16 possible bit patterns (0000 to 1111). Each pattern corresponds to a specific count of set bits:

```
Pattern  Binary  Count  Pattern  Binary  Count
0        0000    0      8        1000    1
1        0001    1      9        1001    2
2        0010    1      A        1010    2
3        0011    2      B        1011    3
4        0100    1      C        1100    2
5        0101    2      D        1101    3
6        0110    2      E        1110    3
7        0111    3      F        1111    4
```

**Step 2: Table Construction**
```rust
const POPCOUNT_4BIT: [u8; 16] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4];
```

**Step 3: Processing Algorithm**
```rust
fn popcount_4bit(bb: Bitboard) -> u32 {
    let mut count = 0;
    let mut bits = bb;
    
    // Process 4 bits at a time
    while bits != 0 {
        // Extract lowest 4 bits
        let chunk = bits & 0xF;
        // Look up the count for this 4-bit pattern
        count += POPCOUNT_4BIT[chunk as usize] as u32;
        // Shift to next 4-bit chunk
        bits >>= 4;
    }
    
    count
}
```

#### Performance Characteristics
- **Time Complexity**: O(log n) where n is the number of bits
- **Space Complexity**: O(1) - constant 16-byte table
- **Cache Efficiency**: 100% L1 cache hit rate
- **Best Case**: When bitboard has many zero bits (early termination possible)
- **Worst Case**: When bitboard is densely populated (all 16 chunks must be processed)

#### Applications in Bitboard Operations
1. **Population Count**: Counting set bits in bitboards
2. **Bit Pattern Recognition**: Identifying specific piece arrangements
3. **Move Generation**: Quick evaluation of piece mobility
4. **Evaluation Functions**: Rapid assessment of board control

### De Bruijn Sequences

#### Mathematical Foundation
De Bruijn sequences are cyclic sequences of symbols where every possible substring of length k appears exactly once. For bit scanning, we use a special type called a **De Bruijn cycle** with specific mathematical properties that make it ideal for bit position determination.

**Key Properties**:
- **Uniqueness**: Each k-bit substring appears exactly once
- **Cyclicity**: The sequence wraps around seamlessly
- **Optimality**: Provides the most efficient bit-to-position mapping possible

#### The De Bruijn Magic Number
For 64-bit bitboards, we use the De Bruijn sequence:
```
0x03f79d71b4cb0a89
```

**Why This Number?**
This specific value has the property that when multiplied by any power of 2 (isolated bit), the result's high-order bits contain a unique identifier for the bit position.

#### How De Bruijn Bit Scanning Works

**Step 1: Bit Isolation**
```rust
let isolated_bit = bb & (!bb + 1);  // Isolate least significant bit
```
This creates a bitboard with only the least significant set bit remaining.

**Step 2: Multiplication Magic**
```rust
let magic_result = isolated_bit.wrapping_mul(DEBRUIJN64);
```
The multiplication spreads the bit position information into the high-order bits of the result.

**Step 3: Position Extraction**
```rust
let position = DEBRUIJN_TABLE[(magic_result >> 58) as usize];
```
The high-order 6 bits (64 - 58 = 6 bits) of the multiplication result provide a unique index into our lookup table.

**Step 4: The Lookup Table**
```rust
const DEBRUIJN_TABLE: [u8; 64] = [
    0, 1, 48, 2, 57, 49, 28, 3, 61, 58, 50, 42, 38, 29, 17, 4,
    62, 55, 59, 36, 53, 51, 43, 22, 45, 39, 33, 30, 24, 18, 12, 5,
    63, 47, 56, 27, 60, 41, 37, 16, 54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6
];
```

#### Mathematical Derivation
The De Bruijn sequence is constructed using graph theory:
1. **Graph Construction**: Create a graph with 2^(k-1) vertices, each representing a (k-1)-bit string
2. **Edge Creation**: Add edges between vertices that can be connected by adding one bit
3. **Eulerian Path**: Find an Eulerian path through this graph
4. **Sequence Generation**: The path gives us the De Bruijn sequence

#### Performance Analysis
- **Time Complexity**: O(1) - constant time operation
- **Space Complexity**: O(1) - 64-byte lookup table
- **Operations**: 4 basic operations (AND, NOT, ADD, MUL, SHR, LOOKUP)
- **Branch-Free**: No conditional jumps, excellent for pipelining

#### Advantages Over Traditional Methods
1. **No Loops**: Eliminates the need for bit-by-bit iteration
2. **Predictable Performance**: Constant execution time regardless of bit position
3. **Cache Friendly**: Small lookup table fits in L1 cache
4. **Hardware Optimized**: Works well with modern CPU pipelines

### Precomputed Masks

#### Conceptual Foundation
Precomputed masks are bitboard patterns that represent common geometric relationships on the chess/shogi board. They encode spatial relationships and are used to quickly extract or manipulate groups of squares that share geometric properties.

**Core Principle**: Trade memory for computation time by pre-calculating commonly needed bitboard patterns.

#### Types of Precomputed Masks

##### 1. Rank Masks (Horizontal Lines)
```rust
const RANK_MASKS: [Bitboard; 9] = [
    0x00000000000000FF,  // Rank 1: squares 0-7
    0x000000000000FF00,  // Rank 2: squares 8-15
    0x0000000000FF0000,  // Rank 3: squares 16-23
    0x00000000FF000000,  // Rank 4: squares 24-31
    0x000000FF00000000,  // Rank 5: squares 32-39
    0x0000FF0000000000,  // Rank 6: squares 40-47
    0x00FF000000000000,  // Rank 7: squares 48-55
    0xFF00000000000000,  // Rank 8: squares 56-63
    0x0000000000000000,  // Rank 9: (invalid)
];
```

**Mathematical Pattern**: Each rank mask is a bitboard where all bits in a specific rank are set. The pattern follows: `0xFF << (rank * 8)`

##### 2. File Masks (Vertical Lines)
```rust
const FILE_MASKS: [Bitboard; 9] = [
    0x0101010101010101,  // File A: squares 0,8,16,24,32,40,48,56
    0x0202020202020202,  // File B: squares 1,9,17,25,33,41,49,57
    0x0404040404040404,  // File C: squares 2,10,18,26,34,42,50,58
    0x0808080808080808,  // File D: squares 3,11,19,27,35,43,51,59
    0x1010101010101010,  // File E: squares 4,12,20,28,36,44,52,60
    0x2020202020202020,  // File F: squares 5,13,21,29,37,45,53,61
    0x4040404040404040,  // File G: squares 6,14,22,30,38,46,54,62
    0x8080808080808080,  // File H: squares 7,15,23,31,39,47,55,63
    0x0000000000000000,  // File I: (invalid)
];
```

**Mathematical Pattern**: Each file mask has bits set at intervals of 8 squares. The pattern follows: `1 << file` repeated every 8 bits.

##### 3. Diagonal Masks
```rust
const DIAGONAL_MASKS: [Bitboard; 15] = [
    0x0000000000000001,  // Diagonal 0: square 0
    0x0000000000000102,  // Diagonal 1: squares 1,8
    0x0000000000010204,  // Diagonal 2: squares 2,9,16
    0x0000000001020408,  // Diagonal 3: squares 3,10,17,24
    0x0000000102040810,  // Diagonal 4: squares 4,11,18,25,32
    0x0000010204081020,  // Diagonal 5: squares 5,12,19,26,33,40
    0x0001020408102040,  // Diagonal 6: squares 6,13,20,27,34,41,48
    0x0102040810204080,  // Diagonal 7: squares 7,14,21,28,35,42,49,56
    0x0204081020408000,  // Diagonal 8: squares 15,22,29,36,43,50,57
    0x0408102040800000,  // Diagonal 9: squares 23,30,37,44,51,58
    0x0810204080000000,  // Diagonal 10: squares 31,38,45,52,59
    0x1020408000000000,  // Diagonal 11: squares 39,46,53,60
    0x2040800000000000,  // Diagonal 12: squares 47,54,61
    0x4080000000000000,  // Diagonal 13: squares 55,62
    0x8000000000000000,  // Diagonal 14: square 63
];
```

**Mathematical Pattern**: Diagonal masks represent squares that lie on the same diagonal. The pattern is more complex and requires careful calculation based on the diagonal's starting position.

#### Advanced Mask Operations

##### Mask Intersection and Union
```rust
// Get all squares on rank 3 and file C
let rank3_fileC = RANK_MASKS[3] & FILE_MASKS[2];

// Get all squares on rank 3 or file C
let rank3_or_fileC = RANK_MASKS[3] | FILE_MASKS[2];
```

##### Mask-Based Move Generation
```rust
// Get all empty squares on rank 3
let empty_rank3 = RANK_MASKS[3] & !occupied_squares;

// Get all enemy pieces on file C
let enemy_fileC = FILE_MASKS[2] & enemy_pieces;
```

##### Geometric Transformations
```rust
// Mirror horizontally (flip files)
fn mirror_horizontal(bb: Bitboard) -> Bitboard {
    let mut result = 0;
    for file in 0..8 {
        result |= ((bb & FILE_MASKS[file]) >> file) << (7 - file);
    }
    result
}

// Mirror vertically (flip ranks)
fn mirror_vertical(bb: Bitboard) -> Bitboard {
    let mut result = 0;
    for rank in 0..8 {
        result |= ((bb & RANK_MASKS[rank]) >> (rank * 8)) << ((7 - rank) * 8);
    }
    result
}
```

#### Memory vs. Computation Trade-off

**Memory Usage**:
- Rank masks: 9 × 8 bytes = 72 bytes
- File masks: 9 × 8 bytes = 72 bytes  
- Diagonal masks: 15 × 8 bytes = 120 bytes
- **Total**: 264 bytes (fits comfortably in L1 cache)

**Computation Savings**:
- Without masks: O(n) operations to determine rank/file/diagonal
- With masks: O(1) lookup and bitwise operation
- **Speed improvement**: 10-50x faster for geometric queries

#### Performance Characteristics
- **Lookup Time**: O(1) - single array access
- **Memory Access**: Highly cache-friendly (small, frequently accessed)
- **Branch-Free**: No conditional logic, excellent for pipelining
- **Parallelizable**: Multiple mask operations can be performed simultaneously

#### Applications in Shogi Engine
1. **Move Generation**: Quickly identify valid moves along ranks, files, and diagonals
2. **Attack Detection**: Determine if squares are under attack along specific directions
3. **Evaluation**: Assess control of ranks, files, and diagonals
4. **Position Analysis**: Identify geometric patterns and structures
5. **Move Validation**: Check if moves follow geometric constraints

#### 2.1 De Bruijn Sequence Implementation
**File**: `src/bitboards/debruijn.rs`

**Responsibilities**:
- Implement De Bruijn sequence bit scanning
- Provide lookup tables
- Optimize for 64-bit operations

**Key Functions**:
```rust
const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;
const DEBRUIJN_TABLE: [u8; 64] = [
    0, 1, 48, 2, 57, 49, 28, 3, 61, 58, 50, 42, 38, 29, 17, 4,
    62, 55, 59, 36, 53, 51, 43, 22, 45, 39, 33, 30, 24, 18, 12, 5,
    63, 47, 56, 27, 60, 41, 37, 16, 54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6
];

fn bit_scan_forward_debruijn(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        None
    } else {
        Some(DEBRUIJN_TABLE[((bb & (!bb + 1)).wrapping_mul(DEBRUIJN64) >> 58) as usize])
    }
}
```

#### 2.2 4-bit Lookup Tables
**File**: `src/bitboards/lookup_tables.rs`

**Responsibilities**:
- Implement 4-bit lookup tables for small operations
- Optimize common bit patterns
- Provide fast bit counting for small bitboards

**Key Functions**:
```rust
const POPCOUNT_4BIT: [u8; 16] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4];

fn popcount_4bit(bb: Bitboard) -> u32 {
    let mut count = 0;
    let mut bits = bb;
    while bits != 0 {
        count += POPCOUNT_4BIT[(bits & 0xF) as usize] as u32;
        bits >>= 4;
    }
    count
}
```

#### 2.3 Precomputed Masks
**File**: `src/bitboards/masks.rs`

**Responsibilities**:
- Precompute common bit patterns
- Optimize bit manipulation operations
- Provide fast bit extraction

**Key Functions**:
```rust
const RANK_MASKS: [Bitboard; 9] = [/* precomputed rank masks */];
const FILE_MASKS: [Bitboard; 9] = [/* precomputed file masks */];
const DIAGONAL_MASKS: [Bitboard; 15] = [/* precomputed diagonal masks */];

fn get_rank_mask(rank: u8) -> Bitboard
fn get_file_mask(file: u8) -> Bitboard
fn get_diagonal_mask(diagonal: u8) -> Bitboard
```

### Phase 3: Specialized Bit Operations (Week 2)

#### 3.1 Bit Iterator Optimization
**File**: `src/bitboards/bit_iterator.rs`

**Responsibilities**:
- Implement efficient bit iteration
- Optimize for common patterns
- Provide iterator interface

**Key Functions**:
```rust
struct BitIterator {
    bits: Bitboard,
    current: Option<u8>,
}

impl Iterator for BitIterator {
    type Item = u8;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.current {
            self.bits &= self.bits - 1; // Clear current bit
            self.current = bit_scan_forward(self.bits);
            Some(pos)
        } else {
            None
        }
    }
}
```

#### 3.2 Bit Manipulation Utilities
**File**: `src/bitboards/bit_utils.rs`

**Responsibilities**:
- Provide utility functions for bit manipulation
- Optimize common operations
- Handle edge cases

**Key Functions**:
```rust
fn isolate_lsb(bb: Bitboard) -> Bitboard
fn isolate_msb(bb: Bitboard) -> Bitboard
fn clear_lsb(bb: Bitboard) -> Bitboard
fn clear_msb(bb: Bitboard) -> Bitboard
fn bit_count(bb: Bitboard) -> u32
fn bit_positions(bb: Bitboard) -> Vec<u8>
```

#### 3.3 Square Coordinate Conversion
**File**: `src/bitboards/square_utils.rs`

**Responsibilities**:
- Convert between bit positions and square coordinates
- Optimize coordinate operations
- Handle Shogi-specific square numbering

**Key Functions**:
```rust
fn bit_to_square(bit: u8) -> Square
fn square_to_bit(square: Square) -> u8
fn bit_to_coords(bit: u8) -> (u8, u8)
fn coords_to_bit(file: u8, rank: u8) -> u8
```

## File Structure

```
src/bitboards/
├── mod.rs
├── platform_detection.rs
├── popcount.rs
├── bitscan.rs
├── debruijn.rs
├── lookup_tables.rs
├── masks.rs
├── bit_iterator.rs
├── bit_utils.rs
├── square_utils.rs
└── tests/
    ├── popcount_tests.rs
    ├── bitscan_tests.rs
    ├── lookup_tests.rs
    └── performance_tests.rs
```

## Testing Strategy

### Unit Tests
- Individual function correctness
- Edge case handling (zero bitboards, single bits)
- Cross-platform compatibility
- Performance regression testing

### Integration Tests
- Integration with existing bitboard operations
- Move generation performance
- Memory usage validation
- Correctness across all board positions

### Performance Tests
- Benchmark against reference implementations
- CPU cycle counting for critical paths
- Memory access pattern analysis
- Comparison with generic implementations

## Performance Targets

### Speed Improvements
- **Population Count**: 5-10x faster than generic implementation
- **Bit Scanning**: 3-5x faster than generic implementation
- **Overall Bitboard Operations**: 10-20% improvement

### Memory Usage
- **Lookup Tables**: < 1KB for all tables
- **Runtime Memory**: No additional allocation
- **Cache Efficiency**: Optimized for L1 cache access

## Risk Mitigation

### Technical Risks
1. **Platform Compatibility**: Ensure fallbacks work on all target platforms
2. **Performance Regression**: Maintain backward compatibility
3. **Hardware Dependencies**: Graceful degradation on older CPUs

### Mitigation Strategies
- Comprehensive cross-platform testing
- Performance regression testing
- Feature detection and fallback mechanisms
- Gradual rollout with performance monitoring

## Success Criteria

### Functional Requirements
- [ ] All bit-scanning operations work correctly
- [ ] No performance regressions in existing code
- [ ] Cross-platform compatibility maintained
- [ ] Backward compatibility preserved

### Performance Requirements
- [ ] 10-20% improvement in bitboard operations
- [ ] < 5 CPU cycles for popcount operations
- [ ] < 10 CPU cycles for bit-scan operations
- [ ] No additional memory allocation

### Quality Requirements
- [ ] 100% test coverage for bit-scanning code
- [ ] No performance regressions
- [ ] Clean, maintainable code structure
- [ ] Comprehensive documentation

## Implementation Timeline

### Week 1
- **Days 1-2**: Platform detection and hardware acceleration
- **Days 3-4**: Lookup table implementation
- **Days 5-7**: Testing and validation

### Week 2
- **Days 1-2**: Specialized bit operations
- **Days 3-4**: Integration and optimization
- **Days 5-7**: Performance testing and documentation

## Dependencies

### Internal Dependencies
- `src/bitboards.rs` - Existing bitboard implementation
- `src/types.rs` - Square and position types
- `src/moves.rs` - Move generation system

### External Dependencies
- No additional external dependencies required
- Uses standard Rust library features
- Platform-specific intrinsics for hardware acceleration

## Future Enhancements

### Potential Improvements
1. **SIMD Optimization**: Vectorized bit operations
2. **Custom Allocators**: Optimized memory allocation
3. **JIT Compilation**: Runtime optimization of bit operations
4. **Hardware-Specific Tuning**: Architecture-specific optimizations

### Integration Opportunities
- Integration with magic bitboards
- Optimization of move generation loops
- Caching of frequently accessed bit patterns
- Integration with evaluation functions

## Conclusion

The Bit-Scanning Optimization implementation provides essential performance improvements for bitboard operations, which are fundamental to the Shogi engine's move generation and evaluation. The combination of hardware acceleration, lookup tables, and specialized algorithms will deliver significant performance gains while maintaining code clarity and cross-platform compatibility.

The expected 10-20% improvement in bitboard operations will contribute to overall engine performance and provide a solid foundation for future optimizations.
