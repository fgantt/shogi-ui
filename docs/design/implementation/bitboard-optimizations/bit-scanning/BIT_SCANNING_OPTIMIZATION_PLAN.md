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
