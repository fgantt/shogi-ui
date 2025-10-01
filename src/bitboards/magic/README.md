# Magic Bitboards Module

A complete implementation of magic bitboards for efficient sliding piece move generation in Shogi.

## Overview

Magic bitboards use precomputed lookup tables with carefully chosen "magic" numbers to provide O(1) attack pattern lookups for sliding pieces (Rook and Bishop). This is 3-5x faster than traditional ray-casting methods.

## Architecture

### Immutable Design
All components use immutable APIs (`&self`) for:
- Thread safety by default
- WASM compatibility
- Clean functional design
- Safe cloning and sharing

### Components

```
magic/
├── magic_finder.rs          - Magic number generation
├── attack_generator.rs      - Attack pattern generation  
├── magic_table.rs           - Table management & storage
├── validator.rs             - Correctness validation
├── memory_pool.rs           - Memory management
└── mod.rs                   - Module exports
```

## Usage

### Quick Start

```rust
use shogi_engine::{BitboardBoard, types::{Position, PieceType, Player}};

// Create board with magic bitboards
let mut board = BitboardBoard::new_with_magic_support()?;
board.init_sliding_generator()?;

// Generate moves using magic bitboards
let moves = board.generate_magic_sliding_moves(
    Position::new(4, 4),  // Center square
    PieceType::Rook,
    Player::Black
)?;
```

### Direct Table Access

```rust
use shogi_engine::types::MagicTable;

// Create or load table
let table = MagicTable::new()?;

// Get attack pattern
let attacks = table.get_attacks(square, PieceType::Rook, occupied);

// Serialize for caching
let bytes = table.serialize()?;
std::fs::write("magic_table.bin", &bytes)?;

// Load later
let bytes = std::fs::read("magic_table.bin")?;
let table = MagicTable::deserialize(&bytes)?;
```

## Performance

### Characteristics
- **O(1) lookup time** for attack patterns
- **~3-5x faster** than ray-casting (theoretical)
- **Immutable & thread-safe**
- **WASM compatible**

### Trade-offs
- **Table creation is slow** (~60 seconds)
  - Solution: Use serialization to cache tables
- **Memory usage** (~few MB for all tables)
  - Acceptable for modern systems

## Testing

```bash
# Fast correctness tests (recommended)
cargo test --test magic_correctness_tests

# Unit tests
cargo test --test magic_tests test_attack_generator

# Integration tests (requires table creation)
cargo test --test magic_integration_tests

# Performance benchmarks
cargo test --test magic_performance_tests
```

### Test Coverage

✅ **9 correctness tests passing**
- Attack pattern validation
- Blocker handling
- Edge cases
- Position round-trips
- All squares coverage

## Implementation Details

### Magic Number Generation

Three algorithms available:
1. **Random Search** - Fast, good for most squares
2. **Brute Force** - Guaranteed to find solution
3. **Heuristic** - Optimized for quality magic numbers

### Attack Pattern Generation

Uses ray-casting to generate reference patterns:
- Supports Rook (orthogonal) and Bishop (diagonal)
- Handles blockers correctly
- Caches generated patterns

### Table Management

- Stores magic bitboards for all 81 squares × 2 piece types
- Efficient memory allocation with MemoryPool
- Serialization/deserialization for persistence
- Validation against ray-casting

## API Reference

### MagicTable

```rust
// Create new table (slow ~60s)
let table = MagicTable::new()?;

// Get attacks (O(1) lookup)
let attacks: Bitboard = table.get_attacks(square, piece_type, occupied);

// Serialize/deserialize
let bytes = table.serialize()?;
let table = MagicTable::deserialize(&bytes)?;

// Validation
table.validate()?;

// Performance stats
let stats = table.performance_stats();
```

### SlidingMoveGenerator

```rust
// Pure functional API
let generator = SlidingMoveGenerator::new(magic_table);

// Generate moves (immutable)
let moves = generator.generate_sliding_moves(&board, from, piece_type, player);

// Batch generation
let moves = generator.generate_sliding_moves_batch(&board, &pieces, player);

// Check settings
if generator.is_magic_enabled() {
    // ...
}
```

## Future Enhancements

See Task 8 in `MAGIC_BITBOARDS_TASKS.md`:
- Parallel table initialization
- Lookup caching with `RefCell`
- SIMD optimizations
- Compressed table format
- Adaptive caching

## References

- Design: `docs/design/implementation/bitboard-optimizations/MAGIC_BITBOARDS_DESIGN.md`
- Tasks: `docs/design/implementation/bitboard-optimizations/MAGIC_BITBOARDS_TASKS.md`
- Implementation Plan: `docs/design/implementation/bitboard-optimizations/MAGIC_BITBOARDS_IMPLEMENTATION_PLAN.md`
- Architecture Decision: `docs/design/implementation/bitboard-optimizations/MAGIC_BITBOARDS_IMMUTABLE_SOLUTION.md`
- Completion Summary: `docs/design/implementation/bitboard-optimizations/MAGIC_BITBOARDS_COMPLETION_SUMMARY.md`

## License

Part of the Shogi Engine project.
