# Bit-Scanning Optimization API Reference

This document provides comprehensive documentation for the bit-scanning optimization API, including usage examples, performance characteristics, and migration guidance.

## Table of Contents

1. [Overview](#overview)
2. [API Modules](#api-modules)
3. [Core Functions](#core-functions)
4. [Performance Characteristics](#performance-characteristics)
5. [Platform Support](#platform-support)
6. [Usage Examples](#usage-examples)
7. [Migration Guide](#migration-guide)
8. [Best Practices](#best-practices)

## Overview

The bit-scanning optimization API provides high-performance bit manipulation operations optimized for both native and WebAssembly environments. The API is organized into logical modules for easy discovery and use.

### Key Features

- **Hardware Acceleration**: Automatic detection and use of CPU features (POPCNT, BMI1, BMI2)
- **WASM Compatibility**: Optimized implementations for WebAssembly environments
- **Adaptive Selection**: Automatic algorithm selection based on platform capabilities
- **Zero Allocation**: Efficient operations without heap allocations
- **Comprehensive Testing**: Thoroughly tested across all supported platforms

## API Modules

### 1. Core Bit-Scanning (`bitscan`)

The core module provides fundamental bit-scanning operations:

```rust
use shogi_engine::bitboards::api::bitscan;

// Population count
let count = bitscan::popcount(0b1010); // Returns 2

// Find bit positions
let lsb = bitscan::bit_scan_forward(0b1000); // Returns Some(3)
let msb = bitscan::bit_scan_reverse(0b1010); // Returns Some(3)

// Get all positions
let positions = bitscan::get_all_bit_positions(0b1010); // Returns vec![1, 3]

// Iterate over bits
for pos in bitscan::bits(0b1010) {
    println!("Bit at position: {}", pos);
}
```

### 2. Bit Manipulation Utilities (`utils`)

Utility functions for common bit manipulation operations:

```rust
use shogi_engine::bitboards::api::utils;

// Extract bits
let (lsb, cleared) = utils::extract_lsb(0b1010); // (0b0010, 0b1000)
let (msb, cleared) = utils::extract_msb(0b1010); // (0b1000, 0b0010)

// Set operations
let intersection = utils::intersection(0b1010, 0b0110); // Returns 0b0010
let union = utils::union(0b1010, 0b0110); // Returns 0b1110

// Check overlaps
let has_overlap = utils::overlaps(0b1010, 0b0101); // Returns true
```

### 3. Square Coordinate Conversion (`squares`)

Convert between bit positions, coordinates, and algebraic notation:

```rust
use shogi_engine::bitboards::api::squares;
use shogi_engine::types::{Position, Player};

// Convert between bit positions and coordinates
let pos = squares::bit_to_square(40); // Position { row: 4, col: 4 }
let bit = squares::square_to_bit(pos); // Returns 40

// Algebraic notation
let name = squares::bit_to_square_name(40); // Returns "5e"
let bit = squares::square_name_to_bit("5e"); // Returns 40

// Shogi-specific utilities
let is_promo = squares::is_promotion_zone(63, Player::Black); // Returns true
let center_squares = squares::get_center_squares(); // Returns vec![40, 39, 41, 31, 49]
```

### 4. Platform Detection (`platform`)

Access platform capabilities and create optimized instances:

```rust
use shogi_engine::bitboards::api::platform;

// Check platform capabilities
let caps = platform::get_platform_capabilities();
println!("Has POPCNT: {}", caps.has_popcnt);
println!("Is WASM: {}", caps.is_wasm);

// Create optimized optimizer
let optimizer = platform::create_optimizer();
let count = optimizer.popcount(0b1010);
```

### 5. Performance Analysis (`analysis`)

Analyze bitboard patterns and geometric structures:

```rust
use shogi_engine::bitboards::api::analysis;

// Analyze bitboard patterns
let analysis = analysis::analyze_bitboard(0b1010);
println!("Population count: {}", analysis.popcount);
println!("First bit: {:?}", analysis.first_bit);
println!("Last bit: {:?}", analysis.last_bit);

// Analyze geometric patterns
let geo_analysis = analysis::analyze_geometry(0b1010);
println!("Rank distribution: {:?}", geo_analysis.rank_counts);
println!("File distribution: {:?}", geo_analysis.file_counts);
```

### 6. Lookup Tables (`lookup`)

Access precomputed masks and lookup tables:

```rust
use shogi_engine::bitboards::api::lookup;

// Precomputed masks
let rank_mask = lookup::get_rank_mask(0); // Top rank mask
let file_mask = lookup::get_file_mask(4); // Center file mask

// Validate tables
assert!(lookup::validate_all_tables());
```

### 7. Backward Compatibility (`compat`)

Legacy API for existing code:

```rust
use shogi_engine::bitboards::api::compat;

// Legacy function names (deprecated but functional)
let count = compat::count_bits(0b1010); // Use bitscan::popcount instead
let first = compat::find_first_bit(0b1000); // Use bitscan::bit_scan_forward instead
let last = compat::find_last_bit(0b1000); // Use bitscan::bit_scan_reverse instead
```

## Core Functions

### Population Count

```rust
// Count set bits in a bitboard
let count = bitscan::popcount(bitboard);
```

**Performance**: O(1) with hardware acceleration, O(log n) with software fallback

### Bit Position Finding

```rust
// Find least significant bit
let lsb = bitscan::bit_scan_forward(bitboard);

// Find most significant bit  
let msb = bitscan::bit_scan_reverse(bitboard);
```

**Performance**: O(1) with hardware acceleration, O(log n) with software fallback

### Bit Iteration

```rust
// Iterate over all set bits
for pos in bitscan::bits(bitboard) {
    // Process each bit position
}

// Reverse iteration
for pos in bitboard.bits_rev() {
    // Process from MSB to LSB
}
```

**Performance**: O(k) where k is the number of set bits

### Bit Manipulation

```rust
// Extract and clear bits
let (lsb, remaining) = utils::extract_lsb(bitboard);
let (msb, remaining) = utils::extract_msb(bitboard);

// Set operations
let intersection = utils::intersection(bb1, bb2);
let union = utils::union(bb1, bb2);
let difference = utils::difference(bb1, bb2);
```

**Performance**: O(1) for all operations

## Performance Characteristics

### Hardware Acceleration

The API automatically detects and uses available CPU features:

- **POPCNT**: x86_64 population count instruction
- **BMI1**: Bit manipulation instruction set 1
- **BMI2**: Bit manipulation instruction set 2
- **ARM CLZ/CTZ**: Count leading/trailing zeros instructions

### WASM Optimization

For WebAssembly environments:

- **SWAR Algorithms**: SIMD Within A Register for parallel operations
- **4-bit Lookup Tables**: Optimized for small bitboards
- **De Bruijn Sequences**: Fast bit position finding
- **No SIMD Dependencies**: Universal compatibility

### Algorithm Selection

The system automatically selects the best algorithm based on:

1. **Platform Capabilities**: Hardware features available
2. **Bitboard Characteristics**: Size, density, patterns
3. **Environment**: Native vs WASM
4. **Performance Requirements**: Speed vs memory trade-offs

## Platform Support

### Native Platforms

- **x86_64**: Full hardware acceleration support
- **ARM64**: Software optimizations with hardware fallbacks
- **Other Architectures**: Generic software implementations

### WebAssembly

- **All WASM Targets**: Universal compatibility
- **No SIMD Dependencies**: Works on all browsers
- **Optimized Algorithms**: SWAR and lookup table optimizations

## Usage Examples

### Basic Bit Manipulation

```rust
use shogi_engine::bitboards::api::{bitscan, utils};

fn analyze_bitboard(bb: u128) {
    // Count bits
    let count = bitscan::popcount(bb);
    println!("Bit count: {}", count);
    
    // Find positions
    if let Some(lsb) = bitscan::bit_scan_forward(bb) {
        println!("First bit at: {}", lsb);
    }
    
    // Extract bits
    let (lsb, remaining) = utils::extract_lsb(bb);
    println!("LSB: 0x{:X}, Remaining: 0x{:X}", lsb, remaining);
}
```

### Shogi Board Analysis

```rust
use shogi_engine::bitboards::api::{squares, analysis};
use shogi_engine::types::Player;

fn analyze_shogi_position(bitboard: u128) {
    // Analyze patterns
    let analysis = analysis::analyze_bitboard(bitboard);
    println!("Pattern analysis: {:?}", analysis);
    
    // Check promotion zones
    for bit in bitscan::bits(bitboard) {
        if squares::is_promotion_zone(bit, Player::Black) {
            println!("Black promotion square: {}", squares::bit_to_square_name(bit));
        }
    }
    
    // Geometric analysis
    let geo = analysis::analyze_geometry(bitboard);
    println!("Rank distribution: {:?}", geo.rank_counts);
}
```

### Performance Optimization

```rust
use shogi_engine::bitboards::api::platform;

fn optimized_operations() {
    // Create platform-optimized instance
    let optimizer = platform::create_optimizer();
    
    // Use for multiple operations
    let results: Vec<u32> = bitboards.iter()
        .map(|&bb| optimizer.popcount(bb))
        .collect();
    
    // Platform information
    let caps = platform::get_platform_capabilities();
    if caps.has_popcnt {
        println!("Using hardware-accelerated population count");
    }
}
```

## Migration Guide

### From Legacy API

If you're using the old bit-scanning functions:

```rust
// Old API (deprecated)
use shogi_engine::bitboards::api::compat;

let count = compat::count_bits(bitboard);        // Deprecated
let first = compat::find_first_bit(bitboard);    // Deprecated
let last = compat::find_last_bit(bitboard);      // Deprecated

// New API (recommended)
use shogi_engine::bitboards::api::bitscan;

let count = bitscan::popcount(bitboard);         // Recommended
let first = bitscan::bit_scan_forward(bitboard); // Recommended
let last = bitscan::bit_scan_reverse(bitboard);  // Recommended
```

### Module Organization

The new API is organized into logical modules:

```rust
// Import specific modules as needed
use shogi_engine::bitboards::api::{
    bitscan,    // Core bit-scanning operations
    utils,      // Bit manipulation utilities
    squares,    // Coordinate conversion
    platform,   // Platform detection
    analysis,   // Performance analysis
    lookup,     // Precomputed tables
    compat      // Backward compatibility
};
```

### Performance Improvements

The new API provides significant performance improvements:

- **Hardware Acceleration**: Automatic CPU feature detection
- **Adaptive Selection**: Best algorithm for each operation
- **WASM Optimization**: Specialized implementations for web
- **Memory Efficiency**: Zero-allocation operations

## Best Practices

### 1. Use Appropriate Modules

```rust
// For basic bit operations
use shogi_engine::bitboards::api::bitscan;

// For Shogi-specific operations
use shogi_engine::bitboards::api::squares;

// For performance-critical code
use shogi_engine::bitboards::api::platform;
let optimizer = platform::create_optimizer();
```

### 2. Leverage Platform Optimization

```rust
// Check capabilities
let caps = platform::get_platform_capabilities();
if caps.has_popcnt {
    // Use hardware-accelerated operations
}

// Create optimized instances for repeated operations
let optimizer = platform::create_optimizer();
for bitboard in large_collection {
    let count = optimizer.popcount(bitboard);
    // Process...
}
```

### 3. Use Iterators for Bit Traversal

```rust
// Efficient bit iteration
for pos in bitscan::bits(bitboard) {
    // Process each bit position
    process_square(pos);
}

// Reverse iteration when needed
for pos in bitboard.bits_rev() {
    // Process from MSB to LSB
    process_square(pos);
}
```

### 4. Validate Inputs

```rust
// Check if square is valid
if squares::is_valid_shogi_square(bit) {
    let name = squares::bit_to_square_name(bit);
    println!("Valid square: {}", name);
}
```

### 5. Use Analysis for Debugging

```rust
// Analyze bitboard patterns
let analysis = analysis::analyze_bitboard(bitboard);
println!("Debug info: {:?}", analysis);

// Geometric analysis for Shogi
let geo = analysis::analyze_geometry(bitboard);
if geo.has_complete_lines() {
    println!("Complete lines detected");
}
```

## Conclusion

The bit-scanning optimization API provides a comprehensive, high-performance solution for bit manipulation operations in Shogi engines. With automatic platform optimization, WASM compatibility, and extensive functionality, it serves as a solid foundation for efficient bitboard-based game engines.

For more information, see the individual module documentation and the migration guide for transitioning from legacy APIs.
