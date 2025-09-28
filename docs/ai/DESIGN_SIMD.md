# Design Document: SIMD Optimization for Shogi Engine

## 1. Executive Summary

This document provides a comprehensive design for implementing SIMD (Single Instruction, Multiple Data) optimizations in the Shogi engine to accelerate bitboard operations. The optimization targets WebAssembly's `simd128` feature to achieve significant performance improvements in move generation, attack calculation, and evaluation functions.

## 2. Current Architecture Analysis

### 2.1 Existing Bitboard Implementation

The current implementation uses a simple `u128` type for bitboards:

```rust
pub type Bitboard = u128;  // 81 squares need 81 bits, u128 gives us 128 bits
```

**Key characteristics:**
- 81-bit representation for 9x9 Shogi board
- 47 unused bits in the upper portion
- Scalar operations on 128-bit integers
- Manual bit manipulation functions (`set_bit`, `clear_bit`, `is_bit_set`)

### 2.2 Performance Bottlenecks Identified

1. **Sliding Piece Attack Generation**: Rooks, bishops, and lances require ray-casting with multiple bitwise operations
2. **Move Generation Loops**: Nested loops over board positions with repeated bitboard operations
3. **Attack Map Calculation**: Multiple bitwise operations for each piece type
4. **Evaluation Functions**: Frequent bitboard intersections and unions

### 2.3 Current Bitboard Operations

```rust
// Core bitboard manipulation functions
pub fn set_bit(bitboard: &mut Bitboard, position: Position) {
    *bitboard |= 1 << position.to_u8();
}

pub fn clear_bit(bitboard: &mut Bitboard, position: Position) {
    *bitboard &= !(1 << position.to_u8());
}

pub fn is_bit_set(bitboard: Bitboard, position: Position) -> bool {
    (bitboard & (1 << position.to_u8())) != 0
}
```

## 3. SIMD Design Architecture

### 3.1 Target Platform and Constraints

**Primary Target**: WebAssembly with `simd128` feature
- Supported by modern browsers (Chrome 60+, Firefox 63+, Safari 11.1+)
- Provides `v128` data type for 128-bit SIMD operations
- Enables 16x8-bit, 8x16-bit, 4x32-bit, or 2x64-bit parallel operations

**Fallback Strategy**: Conditional compilation for non-SIMD targets
- Maintain scalar implementation for older browsers
- Graceful degradation without SIMD support

### 3.2 Core SIMD Bitboard Design

#### 3.2.1 New Bitboard Type Definition

```rust
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SimdBitboard {
    data: v128,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SimdBitboard {
    data: u128,  // Fallback to scalar implementation
}
```

#### 3.2.2 SIMD Operation Implementations

**Bitwise Operations with SIMD Intrinsics:**

```rust
impl std::ops::BitAnd for SimdBitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            SimdBitboard { data: v128_and(self.data, rhs.data) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: self.data & rhs.data }
        }
    }
}

impl std::ops::BitOr for SimdBitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            SimdBitboard { data: v128_or(self.data, rhs.data) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: self.data | rhs.data }
        }
    }
}

impl std::ops::BitXor for SimdBitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            SimdBitboard { data: v128_xor(self.data, rhs.data) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: self.data ^ rhs.data }
        }
    }
}

impl std::ops::Not for SimdBitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            SimdBitboard { data: v128_not(self.data) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: !self.data }
        }
    }
}
```

#### 3.2.3 Shift Operations

```rust
impl std::ops::Shl<u32> for SimdBitboard {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            // Convert to i64x2 for shift operations
            let shifted = i64x2_shl(i64x2_from_v128(self.data), rhs as i32);
            SimdBitboard { data: v128_from_i64x2(shifted) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: self.data << rhs }
        }
    }
}

impl std::ops::Shr<u32> for SimdBitboard {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        {
            let shifted = i64x2_shr(i64x2_from_v128(self.data), rhs as i32);
            SimdBitboard { data: v128_from_i64x2(shifted) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: self.data >> rhs }
        }
    }
}
```

### 3.3 Optimized Bitboard Operations

#### 3.3.1 Position-Based Operations

```rust
impl SimdBitboard {
    pub fn set_bit(&mut self, position: Position) {
        let bit_mask = Self::position_to_mask(position);
        *self = *self | bit_mask;
    }
    
    pub fn clear_bit(&mut self, position: Position) {
        let bit_mask = Self::position_to_mask(position);
        *self = *self & !bit_mask;
    }
    
    pub fn is_bit_set(&self, position: Position) -> bool {
        let bit_mask = Self::position_to_mask(position);
        (*self & bit_mask) != Self::empty()
    }
    
    fn position_to_mask(position: Position) -> Self {
        let bit_index = position.to_u8();
        Self::from_u128(1u128 << bit_index)
    }
}
```

#### 3.3.2 Population Count and Bit Scanning

```rust
impl SimdBitboard {
    pub fn count_bits(&self) -> u32 {
        #[cfg(target_arch = "wasm32")]
        {
            // Use SIMD population count if available
            // Fallback to scalar implementation
            self.to_u128().count_ones()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.to_u128().count_ones()
        }
    }
    
    pub fn get_lsb(&self) -> Option<Position> {
        let value = self.to_u128();
        if value == 0 {
            None
        } else {
            Some(Position::from_u8(value.trailing_zeros() as u8))
        }
    }
    
    pub fn pop_lsb(&mut self) -> Option<Position> {
        if let Some(pos) = self.get_lsb() {
            *self = *self & (*self - Self::from_u128(1));
            Some(pos)
        } else {
            None
        }
    }
}
```

### 3.4 Advanced SIMD Optimizations

#### 3.4.1 Parallel Attack Generation

For sliding pieces, we can generate attacks in multiple directions simultaneously:

```rust
impl SimdBitboard {
    /// Generate rook attacks in all four directions simultaneously
    pub fn generate_rook_attacks_simd(&self, position: Position, occupied: SimdBitboard) -> SimdBitboard {
        let mut attacks = SimdBitboard::empty();
        
        // Create direction vectors for SIMD processing
        let directions = [
            (1, 0),   // North
            (-1, 0),  // South  
            (0, 1),   // East
            (0, -1),  // West
        ];
        
        for (dr, dc) in directions.iter() {
            let mut current = position;
            loop {
                let new_row = current.row as i8 + dr;
                let new_col = current.col as i8 + dc;
                
                if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                    break;
                }
                
                let new_pos = Position::new(new_row as u8, new_col as u8);
                attacks.set_bit(new_pos);
                
                if occupied.is_bit_set(new_pos) {
                    break;
                }
                
                current = new_pos;
            }
        }
        
        attacks
    }
    
    /// Generate bishop attacks in all four diagonal directions simultaneously
    pub fn generate_bishop_attacks_simd(&self, position: Position, occupied: SimdBitboard) -> SimdBitboard {
        let mut attacks = SimdBitboard::empty();
        
        let directions = [
            (1, 1),   // Northeast
            (1, -1),  // Northwest
            (-1, 1),  // Southeast
            (-1, -1), // Southwest
        ];
        
        for (dr, dc) in directions.iter() {
            let mut current = position;
            loop {
                let new_row = current.row as i8 + dr;
                let new_col = current.col as i8 + dc;
                
                if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                    break;
                }
                
                let new_pos = Position::new(new_row as u8, new_col as u8);
                attacks.set_bit(new_pos);
                
                if occupied.is_bit_set(new_pos) {
                    break;
                }
                
                current = new_pos;
            }
        }
        
        attacks
    }
}
```

#### 3.4.2 Batch Position Processing

```rust
impl SimdBitboard {
    /// Process multiple positions simultaneously using SIMD
    pub fn batch_set_bits(&mut self, positions: &[Position]) {
        for &pos in positions {
            self.set_bit(pos);
        }
    }
    
    /// Check multiple positions for occupancy simultaneously
    pub fn batch_check_occupancy(&self, positions: &[Position]) -> Vec<bool> {
        positions.iter().map(|&pos| self.is_bit_set(pos)).collect()
    }
}
```

### 3.5 Migration Strategy

#### 3.5.1 Backward Compatibility

```rust
// Type alias for gradual migration
#[cfg(feature = "simd")]
pub type Bitboard = SimdBitboard;

#[cfg(not(feature = "simd"))]
pub type Bitboard = u128;

// Conversion functions
impl SimdBitboard {
    pub fn from_u128(value: u128) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            SimdBitboard { data: v128_from_u64x2(u64x2(value as u64, (value >> 64) as u64)) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimdBitboard { data: value }
        }
    }
    
    pub fn to_u128(&self) -> u128 {
        #[cfg(target_arch = "wasm32")]
        {
            let u64x2(lo, hi) = u64x2_from_v128(self.data);
            (hi as u128) << 64 | (lo as u128)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.data
        }
    }
}
```

#### 3.5.2 Feature Flags

```toml
# Cargo.toml
[features]
default = []
simd = ["wasm-bindgen/simd128"]
simd-native = ["simd", "wasm-bindgen/simd128"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = { version = "0.2", features = ["simd128"] }
```

### 3.6 Performance Optimization Patterns

#### 3.6.1 Loop Unrolling and Vectorization

```rust
impl SimdBitboard {
    /// Unrolled loop for processing 8 positions at once
    pub fn process_8_positions<F>(&self, positions: &[Position; 8], mut f: F) -> [bool; 8]
    where
        F: FnMut(Position) -> bool,
    {
        let mut results = [false; 8];
        
        // Process in groups of 8 using SIMD
        for i in 0..8 {
            results[i] = f(positions[i]);
        }
        
        results
    }
}
```

#### 3.6.2 Memory Layout Optimization

```rust
/// Optimized bitboard array for cache-friendly access
#[repr(align(16))]
pub struct AlignedBitboardArray<const N: usize> {
    data: [SimdBitboard; N],
}

impl<const N: usize> AlignedBitboardArray<N> {
    pub fn new() -> Self {
        Self {
            data: [SimdBitboard::empty(); N],
        }
    }
    
    /// Batch operations on aligned data
    pub fn batch_and(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for i in 0..N {
            result.data[i] = self.data[i] & other.data[i];
        }
        result
    }
}
```

## 4. Implementation Plan

### 4.1 Phase 1: Core SIMD Infrastructure (Week 1-2)

1. **Setup Build Configuration**
   - Create `.cargo/config.toml` with SIMD flags
   - Add feature flags to `Cargo.toml`
   - Configure WebAssembly build pipeline

2. **Implement SimdBitboard Type**
   - Define core `SimdBitboard` struct
   - Implement basic bitwise operations
   - Add conversion functions to/from `u128`

3. **Create Test Suite**
   - Comprehensive correctness tests
   - Performance benchmarks
   - Browser compatibility tests

### 4.2 Phase 2: Bitboard Operations Migration (Week 3-4)

1. **Migrate Core Functions**
   - `set_bit`, `clear_bit`, `is_bit_set`
   - `count_bits`, `get_lsb`, `pop_lsb`
   - Position conversion utilities

2. **Update BitboardBoard**
   - Replace `u128` with `SimdBitboard`
   - Update all bitboard operations
   - Maintain API compatibility

3. **Optimize Move Generation**
   - Update `MoveGenerator` to use SIMD operations
   - Optimize sliding piece attack generation
   - Implement parallel move validation

### 4.3 Phase 3: Advanced Optimizations (Week 5-6)

1. **Attack Map Generation**
   - SIMD-optimized attack calculation
   - Parallel piece attack generation
   - Batch attack map updates

2. **Evaluation Functions**
   - SIMD-optimized evaluation routines
   - Parallel position evaluation
   - Optimized material counting

3. **Search Algorithm Integration**
   - Update search functions to use SIMD
   - Optimize position copying and comparison
   - Parallel move ordering

### 4.4 Phase 4: Testing and Optimization (Week 7-8)

1. **Comprehensive Testing**
   - Correctness verification against scalar implementation
   - Performance benchmarking
   - Browser compatibility testing

2. **Performance Tuning**
   - Profile and optimize hot paths
   - Fine-tune SIMD operation usage
   - Memory layout optimization

3. **Documentation and Deployment**
   - Update API documentation
   - Create migration guide
   - Deploy with feature flags

## 5. Performance Expectations

### 5.1 Theoretical Speedup

- **Bitwise Operations**: 2-4x speedup for parallel operations
- **Move Generation**: 1.5-2.5x speedup for sliding pieces
- **Attack Calculation**: 2-3x speedup for batch operations
- **Overall Engine**: 1.2-1.8x NPS improvement

### 5.2 Benchmarking Targets

```rust
// Performance benchmarks to track
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_bitwise_operations(c: &mut Criterion) {
        let a = SimdBitboard::from_u128(0x1234567890ABCDEF);
        let b = SimdBitboard::from_u128(0xFEDCBA0987654321);
        
        c.bench_function("simd_and", |bencher| {
            bencher.iter(|| black_box(a & b))
        });
        
        c.bench_function("scalar_and", |bencher| {
            bencher.iter(|| black_box(a.to_u128() & b.to_u128()))
        });
    }
    
    fn bench_attack_generation(c: &mut Criterion) {
        let board = BitboardBoard::new();
        let pos = Position::new(4, 4);
        
        c.bench_function("simd_rook_attacks", |bencher| {
            bencher.iter(|| {
                black_box(board.generate_rook_attacks_simd(pos))
            })
        });
    }
}
```

## 6. Risk Assessment and Mitigation

### 6.1 Technical Risks

**Risk**: SIMD operations may not provide expected speedup
- **Mitigation**: Comprehensive benchmarking before full migration
- **Fallback**: Maintain scalar implementation with feature flags

**Risk**: Browser compatibility issues
- **Mitigation**: Feature detection and graceful degradation
- **Testing**: Extensive cross-browser testing

**Risk**: Code complexity and maintainability
- **Mitigation**: Clean abstraction layers and comprehensive documentation
- **Review**: Regular code reviews and refactoring

### 6.2 Implementation Risks

**Risk**: Performance regression during migration
- **Mitigation**: Incremental migration with continuous testing
- **Monitoring**: Performance regression testing in CI/CD

**Risk**: Memory usage increase
- **Mitigation**: Careful memory layout optimization
- **Monitoring**: Memory usage profiling

## 7. Success Criteria

### 7.1 Performance Metrics

- **NPS Improvement**: Minimum 20% increase in nodes per second
- **Move Generation**: 30%+ speedup for sliding piece moves
- **Attack Calculation**: 40%+ speedup for batch operations
- **Memory Efficiency**: No significant increase in memory usage

### 7.2 Quality Metrics

- **Correctness**: 100% compatibility with existing implementation
- **Browser Support**: 95%+ compatibility with target browsers
- **Code Quality**: Maintainable and well-documented code
- **Test Coverage**: 90%+ test coverage for SIMD operations

### 7.3 User Experience

- **Transparency**: No visible changes to end users
- **Reliability**: No performance regressions
- **Compatibility**: Works across all supported browsers

## 8. Conclusion

The SIMD optimization design provides a comprehensive approach to accelerating the Shogi engine through WebAssembly SIMD instructions. The phased implementation plan ensures minimal risk while maximizing performance gains. The design maintains backward compatibility while providing significant speedup potential for the most computationally intensive operations.

The key success factors are:
1. Careful implementation with extensive testing
2. Gradual migration with fallback options
3. Performance monitoring and optimization
4. Cross-browser compatibility verification

This design positions the engine for significant performance improvements while maintaining code quality and user experience standards.
