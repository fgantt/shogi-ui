# Opening Book Examples

This document provides practical examples for developers working with the new opening book system.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Creating Custom Opening Books](#creating-custom-opening-books)
- [Performance Optimization](#performance-optimization)
- [Streaming Large Books](#streaming-large-books)
- [Memory Management](#memory-management)
- [Integration Examples](#integration-examples)
- [Troubleshooting](#troubleshooting)

## Basic Usage

### Simple Opening Book Lookup

```rust
use shogi_engine::{ShogiEngine, Position, PieceType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine (automatically loads opening book)
    let mut engine = ShogiEngine::new();
    
    // Check if opening book is loaded
    if !engine.is_opening_book_loaded() {
        println!("Opening book not loaded");
        return Ok(());
    }
    
    // Get opening book move for starting position
    let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    
    if let Some(move_) = engine.get_best_move(fen) {
        println!("Best opening move: {}", move_.to_usi_string());
    } else {
        println!("No opening book move available");
    }
    
    Ok(())
}
```

### Getting All Available Moves

```rust
use shogi_engine::ShogiEngine;

fn analyze_opening_position(engine: &mut ShogiEngine, fen: &str) {
    if let Some(moves) = engine.get_all_opening_book_moves() {
        println!("Available opening moves for position:");
        for (i, move_info) in moves.iter().enumerate() {
            println!("{}. {} (Weight: {}, Eval: {})", 
                     i + 1, 
                     move_info.move_notation, 
                     move_info.weight, 
                     move_info.evaluation);
        }
    } else {
        println!("No opening book moves available");
    }
}
```

### Random Move Selection

```rust
use shogi_engine::ShogiEngine;

fn get_random_opening_move(engine: &mut ShogiEngine, fen: &str) -> Option<String> {
    engine.get_random_opening_book_move()
        .map(|move_| move_.to_usi_string())
}
```

## Creating Custom Opening Books

### Building from Scratch

```rust
use shogi_engine::opening_book::{OpeningBook, BookMove, Position, PieceType};

fn create_custom_opening_book() -> Result<OpeningBook, Box<dyn std::error::Error>> {
    let mut book = OpeningBook::new();
    
    // Add Yagura opening
    let yagura_moves = vec![
        BookMove::new(
            Some(Position::new(6, 6)),  // 7g
            Position::new(5, 6),         // 6g
            PieceType::Pawn,
            false, false,
            900, 30,
            Some("Yagura".to_string()),
            Some("7g7f".to_string())
        ),
        BookMove::new(
            Some(Position::new(2, 2)),  // 3c
            Position::new(3, 2),         // 4c
            PieceType::Pawn,
            false, false,
            850, 25,
            Some("Yagura".to_string()),
            Some("3c4c".to_string())
        ),
    ];
    
    book.add_position("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(), yagura_moves);
    
    // Add more positions...
    
    Ok(book)
}
```

### Converting from JSON

```rust
use shogi_engine::opening_book_converter::OpeningBookConverter;

fn convert_json_to_binary(json_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = std::fs::read_to_string(json_file)?;
    let converter = OpeningBookConverter::new();
    let book = converter.convert_from_json(&json_data)?;
    let binary_data = book.to_binary()?;
    std::fs::write(output_file, &binary_data)?;
    println!("Converted {} to {}", json_file, output_file);
    Ok(())
}
```

### Loading Custom Opening Book

```rust
use shogi_engine::ShogiEngine;

fn load_custom_opening_book(engine: &mut ShogiEngine, book_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binary_data = std::fs::read(book_file)?;
    engine.load_opening_book_from_binary(&binary_data)?;
    println!("Loaded custom opening book from {}", book_file);
    Ok(())
}
```

## Performance Optimization

### Memory Monitoring

```rust
use shogi_engine::ShogiEngine;

fn monitor_memory_usage(engine: &mut ShogiEngine) {
    let memory_stats = engine.get_opening_book_stats();
    println!("Opening Book Memory Usage:");
    println!("  Positions: {}", memory_stats.position_count);
    println!("  Moves: {}", memory_stats.move_count);
    
    // Get detailed memory statistics
    if let Some(detailed_stats) = engine.get_memory_usage() {
        println!("  Total Memory: {} bytes", detailed_stats.total_size);
        println!("  Memory Efficiency: {:.1}%", detailed_stats.memory_efficiency);
    }
}
```

### Automatic Optimization

```rust
use shogi_engine::ShogiEngine;

fn optimize_opening_book(engine: &mut ShogiEngine) {
    let result = engine.optimize_memory_usage();
    if result.optimizations_applied > 0 {
        println!("Applied {} optimizations:", result.optimizations_applied);
        for optimization in result.optimizations {
            println!("  - {}", optimization);
        }
    } else {
        println!("No optimizations needed");
    }
}
```

### Cache Management

```rust
use shogi_engine::ShogiEngine;

fn manage_cache(engine: &mut ShogiEngine) {
    let (cache_len, cache_cap) = engine.get_cache_stats();
    println!("Cache: {}/{} items", cache_len, cache_cap);
    
    // Clear cache if it's too large
    if cache_len > 1000 {
        engine.clear_cache();
        println!("Cache cleared");
    }
}
```

## Streaming Large Books

### Enabling Streaming Mode

```rust
use shogi_engine::ShogiEngine;

fn enable_streaming(engine: &mut ShogiEngine) {
    // Enable streaming for large opening books (>50MB)
    engine.enable_streaming_mode(1024 * 1024); // 1MB chunks
    println!("Streaming mode enabled");
}
```

### Loading Chunks

```rust
use shogi_engine::ShogiEngine;

fn load_opening_book_chunks(engine: &mut ShogiEngine, chunk_files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for (i, chunk_file) in chunk_files.iter().enumerate() {
        let chunk_data = std::fs::read(chunk_file)?;
        let loaded_count = engine.load_chunk(&chunk_data, i as u64 * 1024 * 1024)?;
        println!("Loaded {} positions from chunk {}", loaded_count, i + 1);
    }
    Ok(())
}
```

### Streaming Statistics

```rust
use shogi_engine::ShogiEngine;

fn print_streaming_stats(engine: &mut ShogiEngine) {
    let (loaded, lazy, cached) = engine.get_streaming_stats();
    println!("Streaming Statistics:");
    println!("  Loaded positions: {}", loaded);
    println!("  Lazy positions: {}", lazy);
    println!("  Cached positions: {}", cached);
}
```

## Memory Management

### Memory Usage Tracking

```rust
use shogi_engine::ShogiEngine;

fn track_memory_usage(engine: &mut ShogiEngine) {
    let memory_stats = engine.get_memory_usage();
    
    println!("Detailed Memory Usage:");
    println!("  Loaded positions: {} ({} bytes)", 
             memory_stats.loaded_positions, memory_stats.loaded_positions_size);
    println!("  Lazy positions: {} ({} bytes)", 
             memory_stats.lazy_positions, memory_stats.lazy_positions_size);
    println!("  Cached positions: {} ({} bytes)", 
             memory_stats.cached_positions, memory_stats.cache_size);
    println!("  Temp buffer: {} bytes", memory_stats.temp_buffer_size);
    println!("  Total: {} bytes", memory_stats.total_size);
    println!("  Efficiency: {:.1}%", memory_stats.memory_efficiency);
}
```

### Memory Optimization Strategies

```rust
use shogi_engine::ShogiEngine;

fn optimize_memory_strategies(engine: &mut ShogiEngine) {
    let memory_stats = engine.get_memory_usage();
    
    // Strategy 1: Enable streaming for large books
    if memory_stats.total_size > 50 * 1024 * 1024 {
        engine.enable_streaming_mode(1024 * 1024);
        println!("Enabled streaming mode for large opening book");
    }
    
    // Strategy 2: Clear cache if it's too large
    let (cache_len, cache_cap) = engine.get_cache_stats();
    if cache_len > cache_cap * 3 / 4 {
        engine.clear_cache();
        println!("Cleared cache to free memory");
    }
    
    // Strategy 3: Apply automatic optimizations
    let result = engine.optimize_memory_usage();
    if result.optimizations_applied > 0 {
        println!("Applied {} automatic optimizations", result.optimizations_applied);
    }
}
```

## Integration Examples

### Web Application Integration

```rust
use shogi_engine::ShogiEngine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ShogiWebEngine {
    engine: ShogiEngine,
}

#[wasm_bindgen]
impl ShogiWebEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            engine: ShogiEngine::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn get_opening_move(&mut self, fen: &str) -> Option<String> {
        self.engine.get_best_move(fen)
            .map(|move_| move_.to_usi_string())
    }
    
    #[wasm_bindgen]
    pub fn get_opening_book_stats(&self) -> String {
        let stats = self.engine.get_opening_book_stats();
        format!("{{\"positions\":{},\"moves\":{}}}", 
                stats.position_count, stats.move_count)
    }
}
```

### Game Loop Integration

```rust
use shogi_engine::ShogiEngine;

fn game_loop_with_opening_book(engine: &mut ShogiEngine) {
    let mut game_fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    let mut move_count = 0;
    
    loop {
        // Try opening book first (for first 20 moves)
        let move_ = if move_count < 20 {
            engine.get_best_move(game_fen)
                .or_else(|| engine.get_random_opening_book_move())
        } else {
            None
        };
        
        let move_ = match move_ {
            Some(move_) => {
                println!("Opening book move: {}", move_.to_usi_string());
                move_
            }
            None => {
                // Fall back to AI search
                println!("Using AI search...");
                engine.get_best_move_with_depth(5).unwrap()
            }
        };
        
        // Apply move and update FEN
        // ... (move application logic)
        
        move_count += 1;
        if move_count > 100 { break; } // End game
    }
}
```

### AI Integration

```rust
use shogi_engine::ShogiEngine;

fn ai_with_opening_book(engine: &mut ShogiEngine, fen: &str, depth: u8) -> Option<String> {
    // Check opening book first
    if let Some(opening_move) = engine.get_best_move(fen) {
        return Some(opening_move.to_usi_string());
    }
    
    // Fall back to AI search
    engine.get_best_move_with_depth(depth)
        .map(|move_| move_.to_usi_string())
}
```

## Troubleshooting

### Common Issues and Solutions

```rust
use shogi_engine::{ShogiEngine, OpeningBookError};

fn troubleshoot_opening_book(engine: &mut ShogiEngine) {
    // Issue 1: Opening book not loaded
    if !engine.is_opening_book_loaded() {
        println!("Opening book not loaded. Checking for binary file...");
        
        if let Ok(binary_data) = std::fs::read("dist/opening_book.bin") {
            match engine.load_opening_book_from_binary(&binary_data) {
                Ok(()) => println!("Opening book loaded successfully"),
                Err(e) => println!("Failed to load opening book: {:?}", e),
            }
        } else {
            println!("Binary opening book not found. Falling back to JSON...");
            if let Ok(json_data) = std::fs::read_to_string("src/ai/openingBook.json") {
                match engine.load_opening_book_from_json(&json_data) {
                    Ok(()) => println!("JSON opening book loaded"),
                    Err(e) => println!("Failed to load JSON opening book: {:?}", e),
                }
            }
        }
    }
    
    // Issue 2: Memory usage too high
    let memory_stats = engine.get_memory_usage();
    if memory_stats.total_size > 100 * 1024 * 1024 {
        println!("High memory usage detected: {} bytes", memory_stats.total_size);
        let result = engine.optimize_memory_usage();
        println!("Applied {} optimizations", result.optimizations_applied);
    }
    
    // Issue 3: No moves found
    let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    if engine.get_best_move(fen).is_none() {
        println!("No opening book moves found for position: {}", fen);
        println!("This might be normal for endgame positions");
    }
}
```

### Debug Information

```rust
use shogi_engine::ShogiEngine;

fn debug_opening_book(engine: &mut ShogiEngine) {
    println!("=== Opening Book Debug Information ===");
    
    // Basic info
    println!("Loaded: {}", engine.is_opening_book_loaded());
    
    if let Some(stats) = engine.get_opening_book_stats() {
        println!("Positions: {}", stats.position_count);
        println!("Moves: {}", stats.move_count);
    }
    
    // Memory info
    if let Some(memory_stats) = engine.get_memory_usage() {
        println!("Memory Usage:");
        println!("  Total: {} bytes", memory_stats.total_size);
        println!("  Efficiency: {:.1}%", memory_stats.memory_efficiency);
    }
    
    // Cache info
    let (cache_len, cache_cap) = engine.get_cache_stats();
    println!("Cache: {}/{}", cache_len, cache_cap);
    
    // Streaming info
    let (loaded, lazy, cached) = engine.get_streaming_stats();
    println!("Streaming: Loaded={}, Lazy={}, Cached={}", loaded, lazy, cached);
}
```

### Performance Testing

```rust
use shogi_engine::ShogiEngine;
use std::time::Instant;

fn performance_test(engine: &mut ShogiEngine) {
    let test_fens = vec![
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1",
        // Add more test positions
    ];
    
    println!("=== Performance Test ===");
    
    for (i, fen) in test_fens.iter().enumerate() {
        let start = Instant::now();
        let move_ = engine.get_best_move(fen);
        let duration = start.elapsed();
        
        println!("Test {}: {} ms - {}", 
                 i + 1, 
                 duration.as_millis(), 
                 if move_.is_some() { "Found" } else { "Not found" });
    }
}
```

These examples provide practical guidance for developers working with the opening book system. The code is designed to be both educational and production-ready, covering common use cases and edge cases that developers might encounter.
