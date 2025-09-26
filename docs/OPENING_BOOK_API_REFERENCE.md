# Opening Book API Reference

This document provides comprehensive API documentation for the new opening book system.

## Table of Contents

- [Core Structures](#core-structures)
- [OpeningBook Methods](#openingbook-methods)
- [ShogiEngine Integration](#shogiengine-integration)
- [Error Handling](#error-handling)
- [Performance Monitoring](#performance-monitoring)
- [Examples](#examples)

## Core Structures

### BookMove

Represents a single move in the opening book with enhanced metadata.

```rust
pub struct BookMove {
    pub from: Option<Position>,        // Source position (None for drops)
    pub to: Position,                  // Destination position
    pub piece_type: PieceType,         // Piece type
    pub is_drop: bool,                 // Whether this is a drop move
    pub is_promotion: bool,            // Whether this is a promotion
    pub weight: u32,                   // Move frequency/strength (1-1000)
    pub evaluation: i32,               // Position evaluation in centipawns
    pub opening_name: Option<String>,  // Opening name (e.g., "Yagura")
    pub move_notation: Option<String>, // USI notation (e.g., "7g7f")
}
```

**Example:**
```rust
let book_move = BookMove::new(
    Some(Position::new(6, 6)),  // from: 7g
    Position::new(5, 6),         // to: 6g
    PieceType::Pawn,
    false,                       // not a drop
    false,                       // not a promotion
    850,                         // weight
    25,                          // evaluation
    Some("Yagura".to_string()),  // opening name
    Some("7g7f".to_string())     // USI notation
);
```

### PositionEntry

Represents a position and its associated moves.

```rust
pub struct PositionEntry {
    pub fen: String,              // FEN string of the position
    pub moves: Vec<BookMove>,     // Available moves from this position
}
```

### OpeningBookMetadata

Contains metadata about the opening book.

```rust
pub struct OpeningBookMetadata {
    pub version: u32,                    // Format version
    pub position_count: usize,           // Number of positions
    pub move_count: usize,               // Total number of moves
    pub created_at: Option<String>,      // Creation timestamp
    pub updated_at: Option<String>,      // Last update timestamp
    pub streaming_enabled: bool,         // Whether streaming is enabled
    pub chunk_size: usize,               // Chunk size for streaming
}
```

### MemoryUsageStats

Detailed memory usage statistics.

```rust
pub struct MemoryUsageStats {
    pub loaded_positions: usize,         // Number of loaded positions
    pub loaded_positions_size: usize,    // Memory used by loaded positions
    pub lazy_positions: usize,           // Number of lazy positions
    pub lazy_positions_size: usize,      // Memory used by lazy positions
    pub cached_positions: usize,         // Number of cached positions
    pub cache_size: usize,               // Memory used by cache
    pub temp_buffer_size: usize,         // Memory used by temp buffer
    pub total_size: usize,               // Total memory usage
    pub memory_efficiency: f64,          // Memory efficiency percentage
}
```

## OpeningBook Methods

### Construction

#### `new() -> OpeningBook`

Creates a new empty opening book.

```rust
let book = OpeningBook::new();
```

#### `from_binary(data: &[u8]) -> Result<OpeningBook, OpeningBookError>`

Creates an opening book from binary data.

```rust
let binary_data = std::fs::read("opening_book.bin")?;
let book = OpeningBook::from_binary(&binary_data)?;
```

#### `from_binary_boxed(data: Box<[u8]>) -> Result<OpeningBook, OpeningBookError>`

Creates an opening book from boxed binary data (WASM optimized).

```rust
let boxed_data = binary_data.into_boxed_slice();
let book = OpeningBook::from_binary_boxed(boxed_data)?;
```

### Move Lookup

#### `get_moves(&mut self, fen: &str) -> Option<Vec<BookMove>>`

Gets all available moves for a position.

```rust
let moves = book.get_moves("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
if let Some(moves) = moves {
    println!("Found {} moves", moves.len());
}
```

#### `get_best_move(&mut self, fen: &str) -> Option<Move>`

Gets the best move for a position based on weight and evaluation.

```rust
let best_move = book.get_best_move("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
if let Some(move_) = best_move {
    println!("Best move: {}", move_.to_usi_string());
}
```

#### `get_random_move(&mut self, fen: &str) -> Option<Move>`

Gets a random move weighted by frequency.

```rust
let random_move = book.get_random_move("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
if let Some(move_) = random_move {
    println!("Random move: {}", move_.to_usi_string());
}
```

#### `get_moves_with_metadata(&self, fen: &str) -> Option<Vec<(BookMove, Move)>>`

Gets moves with both book metadata and engine move format.

```rust
let moves_with_metadata = book.get_moves_with_metadata("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1");
if let Some(moves) = moves_with_metadata {
    for (book_move, engine_move) in moves {
        println!("Move: {}, Weight: {}, Evaluation: {}", 
                 engine_move.to_usi_string(), 
                 book_move.weight, 
                 book_move.evaluation);
    }
}
```

### Data Management

#### `add_position(&mut self, fen: String, moves: Vec<BookMove>)`

Adds a position and its moves to the book.

```rust
let moves = vec![
    BookMove::new(/* ... */),
    BookMove::new(/* ... */),
];
book.add_position("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(), moves);
```

#### `add_lazy_position(&mut self, fen: String, moves: Vec<BookMove>) -> Result<(), OpeningBookError>`

Adds a position to lazy storage (memory efficient).

```rust
let moves = vec![/* ... */];
book.add_lazy_position("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(), moves)?;
```

### Serialization

#### `to_binary(&self) -> Result<Box<[u8]>, OpeningBookError>`

Converts the opening book to binary format.

```rust
let binary_data = book.to_binary()?;
std::fs::write("opening_book.bin", &binary_data)?;
```

### Performance Monitoring

#### `get_memory_usage(&self) -> MemoryUsageStats`

Gets detailed memory usage statistics.

```rust
let memory_stats = book.get_memory_usage();
println!("Total memory: {} bytes", memory_stats.total_size);
println!("Memory efficiency: {:.1}%", memory_stats.memory_efficiency);
```

#### `get_cache_stats(&self) -> (usize, usize)`

Gets cache statistics (length, capacity).

```rust
let (cache_len, cache_cap) = book.get_cache_stats();
println!("Cache: {}/{}", cache_len, cache_cap);
```

#### `clear_cache(&mut self)`

Clears the LRU cache.

```rust
book.clear_cache();
```

### Streaming Support

#### `enable_streaming_mode(&mut self, chunk_size: usize)`

Enables streaming mode for large opening books.

```rust
book.enable_streaming_mode(1024 * 1024); // 1MB chunks
```

#### `load_chunk(&mut self, chunk_data: &[u8], chunk_offset: u64) -> Result<usize, OpeningBookError>`

Loads a chunk of positions from binary data.

```rust
let chunk_data = std::fs::read("chunk_1.bin")?;
let loaded_count = book.load_chunk(&chunk_data, 0)?;
println!("Loaded {} positions", loaded_count);
```

#### `get_streaming_stats(&self) -> (usize, usize, usize)`

Gets streaming statistics (loaded, lazy, cached positions).

```rust
let (loaded, lazy, cached) = book.get_streaming_stats();
println!("Loaded: {}, Lazy: {}, Cached: {}", loaded, lazy, cached);
```

### Optimization

#### `optimize_memory_usage(&mut self) -> MemoryOptimizationResult`

Applies automatic memory optimizations.

```rust
let result = book.optimize_memory_usage();
println!("Applied {} optimizations", result.optimizations_applied);
for optimization in result.optimizations {
    println!("- {}", optimization);
}
```

### Utility Methods

#### `get_temp_buffer(&mut self) -> &mut Vec<u8>`

Gets a reusable temporary buffer.

```rust
let buffer = book.get_temp_buffer();
// Use buffer for temporary operations
```

#### `benchmark_hash_functions(&self, test_fens: &[&str]) -> Vec<(String, u64, u64)>`

Benchmarks different hash functions.

```rust
let test_fens = vec!["lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"];
let results = book.benchmark_hash_functions(&test_fens);
for (name, time, _) in results {
    println!("{}: {} ns", name, time);
}
```

## ShogiEngine Integration

### Engine Methods

#### `is_opening_book_loaded(&self) -> bool`

Checks if the opening book is loaded.

```rust
if engine.is_opening_book_loaded() {
    println!("Opening book is loaded");
}
```

#### `get_opening_book_stats(&self) -> OpeningBookStats`

Gets opening book statistics.

```rust
let stats = engine.get_opening_book_stats();
println!("Positions: {}, Moves: {}", stats.position_count, stats.move_count);
```

#### `get_opening_book_info(&mut self) -> OpeningBookInfo`

Gets detailed opening book information.

```rust
let info = engine.get_opening_book_info();
println!("FEN: {}", info.fen);
println!("Available moves: {}", info.available_moves);
```

#### `get_opening_book_move_info(&mut self) -> Option<OpeningBookMoveInfo>`

Gets information about the current opening book move.

```rust
if let Some(move_info) = engine.get_opening_book_move_info() {
    println!("Move: {}", move_info.move_notation);
    println!("Weight: {}", move_info.weight);
    println!("Evaluation: {}", move_info.evaluation);
}
```

#### `get_random_opening_book_move(&mut self) -> Option<Move>`

Gets a random move from the opening book.

```rust
if let Some(move_) = engine.get_random_opening_book_move() {
    println!("Random move: {}", move_.to_usi_string());
}
```

#### `get_all_opening_book_moves(&mut self) -> Option<Vec<OpeningBookMoveInfo>>`

Gets all available opening book moves.

```rust
if let Some(moves) = engine.get_all_opening_book_moves() {
    for move_info in moves {
        println!("Move: {}, Weight: {}", move_info.move_notation, move_info.weight);
    }
}
```

### Loading Methods

#### `load_opening_book_from_binary(&mut self, data: &[u8]) -> Result<(), OpeningBookError>`

Loads opening book from binary data.

```rust
let binary_data = std::fs::read("opening_book.bin")?;
engine.load_opening_book_from_binary(&binary_data)?;
```

#### `load_opening_book_from_json(&mut self, json_data: &str) -> Result<(), OpeningBookError>`

Loads opening book from JSON data (backward compatibility).

```rust
let json_data = std::fs::read_to_string("opening_book.json")?;
engine.load_opening_book_from_json(&json_data)?;
```

## Error Handling

### OpeningBookError

```rust
pub enum OpeningBookError {
    InvalidFen(String),           // Invalid FEN string
    InvalidMove(String),          // Invalid move data
    BinaryFormatError(String),    // Binary format parsing error
    JsonParseError(String),       // JSON parsing error
    IoError(String),              // File I/O error
    HashCollision(String),        // Hash collision in lookup table
}
```

### Error Handling Example

```rust
match book.get_best_move("invalid_fen") {
    Ok(Some(move_)) => println!("Best move: {}", move_.to_usi_string()),
    Ok(None) => println!("No opening book move available"),
    Err(OpeningBookError::InvalidFen(msg)) => println!("Invalid FEN: {}", msg),
    Err(e) => println!("Error: {:?}", e),
}
```

## Performance Monitoring

### Memory Usage Example

```rust
// Get detailed memory statistics
let memory_stats = book.get_memory_usage();
println!("Memory Usage:");
println!("  Loaded positions: {} ({} bytes)", 
         memory_stats.loaded_positions, memory_stats.loaded_positions_size);
println!("  Lazy positions: {} ({} bytes)", 
         memory_stats.lazy_positions, memory_stats.lazy_positions_size);
println!("  Cached positions: {} ({} bytes)", 
         memory_stats.cached_positions, memory_stats.cache_size);
println!("  Total: {} bytes", memory_stats.total_size);
println!("  Efficiency: {:.1}%", memory_stats.memory_efficiency);

// Apply optimizations
let result = book.optimize_memory_usage();
if result.optimizations_applied > 0 {
    println!("Applied {} optimizations:", result.optimizations_applied);
    for optimization in result.optimizations {
        println!("  - {}", optimization);
    }
}
```

### Cache Management Example

```rust
// Get cache statistics
let (cache_len, cache_cap) = book.get_cache_stats();
println!("Cache: {}/{} items", cache_len, cache_cap);

// Clear cache if it's too large
if cache_len > 1000 {
    book.clear_cache();
    println!("Cache cleared");
}
```

## Examples

### Basic Usage

```rust
use shogi_engine::{ShogiEngine, OpeningBook};

// Create engine
let mut engine = ShogiEngine::new();

// Check if opening book is loaded
if engine.is_opening_book_loaded() {
    // Get opening book move
    if let Some(move_) = engine.get_best_move("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1") {
        println!("Opening book move: {}", move_.to_usi_string());
    }
}
```

### Advanced Usage

```rust
use shogi_engine::{ShogiEngine, OpeningBook, BookMove, Position, PieceType};

// Create custom opening book
let mut book = OpeningBook::new();

// Add a position
let moves = vec![
    BookMove::new(
        Some(Position::new(6, 6)),
        Position::new(5, 6),
        PieceType::Pawn,
        false,
        false,
        850,
        25,
        Some("Yagura".to_string()),
        Some("7g7f".to_string())
    ),
];

book.add_position("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(), moves);

// Convert to binary
let binary_data = book.to_binary()?;

// Load into engine
engine.load_opening_book_from_binary(&binary_data)?;
```

### Streaming Example

```rust
// Enable streaming for large opening book
book.enable_streaming_mode(1024 * 1024); // 1MB chunks

// Load chunks as needed
let chunk_data = std::fs::read("chunk_1.bin")?;
let loaded_count = book.load_chunk(&chunk_data, 0)?;
println!("Loaded {} positions from chunk", loaded_count);

// Get streaming statistics
let (loaded, lazy, cached) = book.get_streaming_stats();
println!("Loaded: {}, Lazy: {}, Cached: {}", loaded, lazy, cached);
```

This API reference provides comprehensive documentation for all opening book functionality. The system is designed to be both powerful and easy to use, with extensive performance monitoring and optimization capabilities.
