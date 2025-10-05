# Bit-Scanning API Reference

Complete API reference for the bit-scanning optimization system.

## Table of Contents

1. [Core Functions](#core-functions)
2. [Performance-Optimized Functions](#performance-optimized-functions)
3. [Common Case Optimization](#common-case-optimization)
4. [Cache Optimization](#cache-optimization)
5. [Branch Prediction Optimization](#branch-prediction-optimization)
6. [Bit Manipulation Utilities](#bit-manipulation-utilities)
7. [Bit Iterator](#bit-iterator)
8. [Square Coordinate Conversion](#square-coordinate-conversion)
9. [Platform Detection](#platform-detection)
10. [API Modules](#api-modules)
11. [Backward Compatibility](#backward-compatibility)

## Core Functions

### Population Count

#### `popcount(bb: Bitboard) -> u32`
Counts the number of set bits in a bitboard.

**Parameters**:
- `bb`: The bitboard to count bits in

**Returns**: Number of set bits (0-128)

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1010_1010;
let count = popcount(bb);  // Returns 4
```

**Performance**: Automatically selects the best available implementation based on platform capabilities.

#### `popcount_optimized(bb: Bitboard) -> u32`
Alias for `popcount()` with explicit optimization selection.

### Bit Scanning

#### `bit_scan_forward(bb: Bitboard) -> Option<u8>`
Finds the position of the least significant set bit.

**Parameters**:
- `bb`: The bitboard to scan

**Returns**: `Some(position)` if bits are set, `None` if bitboard is empty

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1000_1000;
let pos = bit_scan_forward(bb);  // Returns Some(3)
```

#### `bit_scan_reverse(bb: Bitboard) -> Option<u8>`
Finds the position of the most significant set bit.

**Parameters**:
- `bb`: The bitboard to scan

**Returns**: `Some(position)` if bits are set, `None` if bitboard is empty

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1000_1000;
let pos = bit_scan_reverse(bb);  // Returns Some(7)
```

#### `get_all_bit_positions(bb: Bitboard) -> Vec<u8>`
Gets all positions of set bits in a bitboard.

**Parameters**:
- `bb`: The bitboard to analyze

**Returns**: Vector of bit positions, ordered from LSB to MSB

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1010_1010;
let positions = get_all_bit_positions(bb);  // Returns vec![1, 3, 5, 7]
```

## Performance-Optimized Functions

### Critical Path Functions

#### `popcount_critical(bb: Bitboard) -> u32`
Maximum performance population count for tight loops.

**Use Case**: Performance-critical code where every cycle matters.

**Example**:
```rust
use shogi_engine::bitboards::critical_paths::*;

// In a tight loop
for bb in bitboards {
    let count = popcount_critical(bb);  // Maximum performance
}
```

#### `bit_scan_forward_critical(bb: Bitboard) -> Option<u8>`
Maximum performance bit scan forward for tight loops.

**Use Case**: Performance-critical code requiring LSB position.

### Branch-Optimized Functions

#### `popcount_branch_optimized(bb: Bitboard) -> u32`
Population count with branch prediction optimization.

**Use Case**: Code with predictable bitboard patterns.

#### `bit_scan_forward_optimized(bb: Bitboard) -> Option<u8>`
Bit scan forward with branch prediction optimization.

**Use Case**: Code with predictable bitboard patterns.

#### `bit_scan_reverse_optimized(bb: Bitboard) -> Option<u8>`
Bit scan reverse with branch prediction optimization.

**Use Case**: Code with predictable bitboard patterns.

## Common Case Optimization

### Empty Board Detection

#### `is_empty_optimized(bb: Bitboard) -> bool`
Optimized check for empty bitboards.

**Use Case**: Most common case in Shogi engines.

**Example**:
```rust
use shogi_engine::bitboards::common_cases::*;

if is_empty_optimized(bb) {
    return;  // Early return for most common case
}
```

#### `is_not_empty_optimized(bb: Bitboard) -> bool`
Optimized check for non-empty bitboards.

### Single Piece Detection

#### `is_single_piece_optimized(bb: Bitboard) -> bool`
Optimized check for single piece on board.

**Use Case**: Common case in Shogi engines.

**Example**:
```rust
use shogi_engine::bitboards::common_cases::*;

if is_single_piece_optimized(bb) {
    let pos = single_piece_position_optimized(bb);
    // Handle single piece efficiently
}
```

#### `single_piece_position_optimized(bb: Bitboard) -> u8`
Gets position of single piece (assumes only one piece is set).

**Use Case**: When you know there's only one piece.

### Multiple Pieces Detection

#### `is_multiple_pieces_optimized(bb: Bitboard) -> bool`
Optimized check for multiple pieces on board.

**Use Case**: When handling multiple pieces.

## Cache Optimization

### Cache-Aligned Data Structures

#### `CacheAlignedPopcountTable`
Cache-aligned lookup table for population count.

**Example**:
```rust
use shogi_engine::bitboards::cache_opt::*;

let table = CacheAlignedPopcountTable::new();
let count = table.get_popcount(0b1010);  // Returns 2
```

#### `CacheAlignedBitPositionTable`
Cache-aligned lookup table for bit positions.

#### `CacheAlignedRankMasks`
Cache-aligned rank masks for 9x9 board.

#### `CacheAlignedFileMasks`
Cache-aligned file masks for 9x9 board.

### Cache-Optimized Functions

#### `popcount_cache_optimized(bb: Bitboard) -> u32`
Population count using cache-optimized lookup tables.

**Use Case**: Memory-intensive operations with large datasets.

#### `get_bit_positions_cache_optimized(bb: Bitboard) -> Vec<u8>`
Bit position enumeration using cache-optimized lookup tables.

### Prefetching

#### `prefetch_bitboard(bb: Bitboard)`
Prefetches a bitboard into cache.

**Example**:
```rust
use shogi_engine::bitboards::cache_opt::*;

unsafe {
    prefetch_bitboard(bb);
    let count = popcount_cache_optimized(bb);
}
```

#### `prefetch_bitboard_sequence(bitboards: &[Bitboard])`
Prefetches a sequence of bitboards.

#### `process_bitboard_sequence(bitboards: &[Bitboard]) -> Vec<u32>`
Processes a sequence of bitboards with prefetching.

## Bit Manipulation Utilities

### Bit Isolation

#### `isolate_lsb(bb: Bitboard) -> Bitboard`
Isolates the least significant bit.

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1010;
let lsb = isolate_lsb(bb);  // Returns 0b0010
```

#### `isolate_msb(bb: Bitboard) -> Bitboard`
Isolates the most significant bit.

#### `extract_lsb(bb: Bitboard) -> (Bitboard, Bitboard)`
Extracts LSB and returns (isolated_bit, remaining_bitboard).

#### `extract_msb(bb: Bitboard) -> (Bitboard, Bitboard)`
Extracts MSB and returns (isolated_bit, remaining_bitboard).

### Bit Clearing

#### `clear_lsb(bb: Bitboard) -> Bitboard`
Clears the least significant bit.

#### `clear_msb(bb: Bitboard) -> Bitboard`
Clears the most significant bit.

### Bit Rotation and Reversal

#### `rotate_left(bb: Bitboard, amount: u32) -> Bitboard`
Rotates bitboard left by specified amount.

#### `rotate_right(bb: Bitboard, amount: u32) -> Bitboard`
Rotates bitboard right by specified amount.

#### `reverse_bits(bb: Bitboard) -> Bitboard`
Reverses the order of bits in the bitboard.

### Set Operations

#### `intersection(a: Bitboard, b: Bitboard) -> Bitboard`
Returns intersection of two bitboards.

#### `union(a: Bitboard, b: Bitboard) -> Bitboard`
Returns union of two bitboards.

#### `symmetric_difference(a: Bitboard, b: Bitboard) -> Bitboard`
Returns symmetric difference of two bitboards.

#### `difference(a: Bitboard, b: Bitboard) -> Bitboard`
Returns difference of two bitboards.

#### `complement(bb: Bitboard) -> Bitboard`
Returns complement of bitboard.

### Boolean Operations

#### `overlaps(a: Bitboard, b: Bitboard) -> bool`
Checks if two bitboards overlap.

#### `is_subset(a: Bitboard, b: Bitboard) -> bool`
Checks if first bitboard is subset of second.

## Bit Iterator

### Basic Iterator

#### `bits(bb: Bitboard) -> BitIterator`
Creates iterator over set bits.

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1010;
for pos in bits(bb) {
    println!("Bit at position: {}", pos);
}
```

#### `BitIterator`
Iterator that yields bit positions from LSB to MSB.

**Methods**:
- `new(bb: Bitboard) -> BitIterator`
- `from_position(bb: Bitboard, start_pos: u8) -> BitIterator`
- `peek() -> Option<u8>`
- `skip(n: usize) -> BitIterator`
- `count() -> usize`
- `last() -> Option<u8>`
- `nth(n: usize) -> Option<u8>`

### Reverse Iterator

#### `bits_rev(bb: Bitboard) -> ReverseBitIterator`
Creates reverse iterator over set bits.

**Example**:
```rust
use shogi_engine::bitboards::*;

let bb: Bitboard = 0b1010;
for pos in bb.bits_rev() {
    println!("Bit at position: {}", pos);
}
```

## Square Coordinate Conversion

### Basic Conversion

#### `bit_to_square(bit_pos: u8) -> Position`
Converts bit position to Position struct.

**Example**:
```rust
use shogi_engine::bitboards::*;

let pos = bit_to_square(40);  // Returns Position { row: 4, col: 4 }
```

#### `square_to_bit(square: Position) -> u8`
Converts Position struct to bit position.

#### `bit_to_coords(bit_pos: u8) -> (u8, u8)`
Converts bit position to (file, rank) coordinates.

#### `coords_to_bit(file: u8, rank: u8) -> u8`
Converts (file, rank) coordinates to bit position.

### Algebraic Notation

#### `bit_to_square_name(bit_pos: u8) -> String`
Converts bit position to algebraic notation.

**Example**:
```rust
use shogi_engine::bitboards::*;

let name = bit_to_square_name(40);  // Returns "5e"
```

#### `square_name_to_bit(name: &str) -> u8`
Converts algebraic notation to bit position.

### Shogi-Specific Utilities

#### `is_valid_shogi_square(bit_pos: u8) -> bool`
Checks if bit position is valid for 9x9 Shogi board.

#### `is_promotion_zone(bit_pos: u8, player: Player) -> bool`
Checks if square is in promotion zone for given player.

#### `square_distance(square1: u8, square2: u8) -> u8`
Calculates distance between two squares.

#### `promotion_zone_mask(player: Player) -> Bitboard`
Returns bitboard mask for player's promotion zone.

#### `get_center_squares() -> Vec<u8>`
Returns vector of center square positions.

#### `is_center_square(bit_pos: u8) -> bool`
Checks if square is in center of board.

## Platform Detection

### Platform Capabilities

#### `get_platform_capabilities() -> &PlatformCapabilities`
Gets platform capabilities for optimization selection.

**Example**:
```rust
use shogi_engine::bitboards::*;

let caps = get_platform_capabilities();
if caps.has_popcnt {
    println!("POPCNT instruction available");
}
```

#### `PlatformCapabilities`
Structure containing platform capabilities:
- `has_popcnt: bool` - POPCNT instruction available
- `has_bmi1: bool` - BMI1 instructions available
- `has_bmi2: bool` - BMI2 instructions available
- `is_wasm: bool` - Running in WASM environment
- `architecture: Architecture` - CPU architecture

### Best Implementation Selection

#### `get_best_popcount_impl() -> PopcountImpl`
Gets best population count implementation for current platform.

#### `get_best_bitscan_impl() -> BitscanImpl`
Gets best bit scanning implementation for current platform.

## API Modules

### Core API

#### `api::bitscan`
Core bit-scanning functions with automatic optimization selection.

**Functions**:
- `popcount(bb: Bitboard) -> u32`
- `bit_scan_forward(bb: Bitboard) -> Option<u8>`
- `bit_scan_reverse(bb: Bitboard) -> Option<u8>`
- `get_all_bit_positions(bb: Bitboard) -> Vec<u8>`

#### `api::utils`
Bit manipulation utility functions.

**Functions**:
- `extract_lsb(bb: Bitboard) -> (Bitboard, Bitboard)`
- `extract_msb(bb: Bitboard) -> (Bitboard, Bitboard)`
- `overlaps(a: Bitboard, b: Bitboard) -> bool`
- `is_subset(a: Bitboard, b: Bitboard) -> bool`

#### `api::squares`
Square coordinate conversion functions.

**Functions**:
- `bit_to_square(bit_pos: u8) -> Position`
- `square_to_bit(square: Position) -> u8`
- `bit_to_square_name(bit_pos: u8) -> String`
- `square_name_to_bit(name: &str) -> u8`

#### `api::platform`
Platform detection and optimization functions.

**Functions**:
- `create_optimizer() -> BitScanningOptimizer`
- `get_platform_capabilities() -> &PlatformCapabilities`

#### `api::analysis`
Geometric analysis functions.

**Functions**:
- `analyze_geometry(bb: Bitboard) -> GeometricAnalysis`

### Specialized Modules

#### `api::lookup`
Lookup table functions.

#### `api::masks`
Precomputed mask functions.

#### `api::debruijn`
De Bruijn sequence functions.

## Backward Compatibility

### Compatibility Functions

#### `api::compat::count_bits(bb: Bitboard) -> u32`
Legacy function for population count.

**Deprecated**: Use `popcount()` instead.

#### `api::compat::find_first_bit(bb: Bitboard) -> Option<u8>`
Legacy function for bit scan forward.

**Deprecated**: Use `bit_scan_forward()` instead.

#### `api::compat::find_last_bit(bb: Bitboard) -> Option<u8>`
Legacy function for bit scan reverse.

**Deprecated**: Use `bit_scan_reverse()` instead.

### Migration

To migrate from legacy functions:

```rust
// Old code
let count = count_bits(bb);
let first = find_first_bit(bb);
let last = find_last_bit(bb);

// New code
let count = popcount(bb);
let first = bit_scan_forward(bb);
let last = bit_scan_reverse(bb);
```

## Performance Characteristics

### Platform-Specific Performance

| Function | x86_64 (POPCNT) | ARM64 | WASM |
|----------|-----------------|-------|------|
| `popcount` | ~1 cycle | ~3-5 cycles | ~3-8 cycles |
| `bit_scan_forward` | ~1 cycle | ~2-4 cycles | ~3-6 cycles |
| `bit_scan_reverse` | ~1 cycle | ~2-4 cycles | ~3-6 cycles |

### Optimization Levels

1. **Hardware-Accelerated**: Uses CPU features for maximum performance
2. **Cache-Optimized**: Uses cache-aligned lookup tables
3. **Branch-Optimized**: Uses branch prediction hints
4. **Critical Path**: Optimized for tight loops
5. **Adaptive**: Automatically selects best implementation

## Error Handling

### Common Errors

#### Empty Bitboard
```rust
let bb: Bitboard = 0;
let pos = bit_scan_forward(bb);  // Returns None
```

#### Invalid Square Position
```rust
let pos = bit_to_square(200);  // Panics for invalid position
```

#### Invalid Algebraic Notation
```rust
let pos = square_name_to_bit("z99");  // Panics for invalid notation
```

### Best Practices

1. **Always check for empty bitboards**:
   ```rust
   if let Some(pos) = bit_scan_forward(bb) {
       // Process bit position
   }
   ```

2. **Validate square positions**:
   ```rust
   if is_valid_shogi_square(bit_pos) {
       let square = bit_to_square(bit_pos);
   }
   ```

3. **Use appropriate optimization level**:
   ```rust
   // For tight loops
   let count = popcount_critical(bb);
   
   // For common cases
   let count = popcount_branch_optimized(bb);
   
   // For general use
   let count = popcount(bb);
   ```

This API reference provides complete documentation for all bit-scanning optimization functions. For usage examples, see `examples/bit-scanning-examples.rs` and for performance guidance, see `docs/bit-scanning-performance-guide.md`.