# Implementation Plan: Opening Book (Revised)

## 1. Objective

To integrate a high-performance, professional-grade opening book into the Rust WASM engine. This will allow the engine to play the first several moves of a game instantly based on a comprehensive library of standard openings, saving calculation time, improving early-game play quality, and introducing strategic variety.

## 2. Current Implementation Issues

After analyzing the existing `openingBook.json` and `opening_book.rs`, several critical issues have been identified:

### 2.1 Data Structure Problems
- **Inefficient Lookup**: Linear search through all openings instead of direct position lookup
- **Poor Organization**: Moves grouped by opening name rather than by position
- **Inconsistent Coordinates**: String coordinates like "27", "26" don't clearly map to engine's (row, col) system
- **Missing Metadata**: No move weights, frequencies, or evaluation scores

### 2.2 Move Representation Issues
- **Incomplete Information**: Missing piece type for drops, unclear promotion logic
- **No Move Ordering**: No way to prioritize better moves or add variety
- **Hard to Maintain**: Adding new positions requires manual JSON editing

### 2.3 Performance Issues
- **Memory Inefficiency**: Large JSON structure loaded entirely into memory
- **Slow Parsing**: Complex nested structure requires multiple iterations
- **No Caching**: No mechanism to cache frequently accessed positions

## 3. Recommended New Implementation

### 3.1 New Data Format: Binary Opening Book

Instead of JSON, use a custom binary format optimized for:
- **Fast Lookup**: Direct position-to-moves mapping
- **Memory Efficiency**: Compact representation with minimal overhead
- **Extensibility**: Easy to add new positions and metadata

#### Binary Format Structure
```
[Header]
- Magic Number: 4 bytes ("SBOB" - Shogi Binary Opening Book)
- Version: 4 bytes (1)
- Entry Count: 8 bytes (number of positions)
- Hash Table Size: 8 bytes (for collision handling)

[Hash Table]
- Position Hash: 8 bytes (FEN hash)
- Entry Offset: 8 bytes (offset to position data)

[Position Entries]
- FEN String Length: 4 bytes
- FEN String: variable length
- Move Count: 4 bytes
- Moves: variable length array of move entries

[Move Entry]
- From Position: 2 bytes (row << 8 | col)
- To Position: 2 bytes (row << 8 | col)
- Piece Type: 1 byte (enum value)
- Is Drop: 1 byte (boolean)
- Is Promotion: 1 byte (boolean)
- Weight: 4 bytes (move frequency/strength)
- Evaluation: 4 bytes (position evaluation)
```

### 3.2 New Rust Implementation

#### Core Data Structures

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::types::{Move, Position, PieceType, Player};

#[derive(Debug, Clone)]
pub struct BookMove {
    pub from: Option<Position>,
    pub to: Position,
    pub piece_type: PieceType,
    pub is_drop: bool,
    pub is_promotion: bool,
    pub weight: u32,        // Move frequency/strength (0-1000)
    pub evaluation: i32,    // Position evaluation in centipawns
}

#[derive(Debug)]
pub struct PositionEntry {
    pub fen: String,
    pub moves: Vec<BookMove>,
}

pub struct OpeningBook {
    positions: HashMap<u64, PositionEntry>, // FEN hash -> position data
    total_moves: usize,
    loaded: bool,
}

impl OpeningBook {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            total_moves: 0,
            loaded: false,
        }
    }

    pub fn load_from_binary(&mut self, data: &[u8]) -> Result<(), String> {
        // Parse binary format and populate positions HashMap
        // Implementation details...
    }

    pub fn load_from_json(&mut self, json_data: &str) -> Result<(), String> {
        // Convert existing JSON format to new structure
        // This allows migration from current format
    }

    pub fn get_moves(&self, fen: &str) -> Option<&Vec<BookMove>> {
        let hash = self.hash_fen(fen);
        self.positions.get(&hash).map(|entry| &entry.moves)
    }

    pub fn get_best_move(&self, fen: &str) -> Option<Move> {
        if let Some(moves) = self.get_moves(fen) {
            // Select move based on weight and evaluation
            let best_move = moves.iter()
                .max_by_key(|m| m.weight)
                .or_else(|| moves.first())?;
            
            Some(self.convert_to_engine_move(best_move))
        } else {
            None
        }
    }

    pub fn get_random_move(&self, fen: &str) -> Option<Move> {
        if let Some(moves) = self.get_moves(fen) {
            // Weighted random selection based on move weights
            let total_weight: u32 = moves.iter().map(|m| m.weight).sum();
            if total_weight == 0 { return None; }
            
            let mut rng = rand::thread_rng();
            let mut random_value = rng.gen_range(0..total_weight);
            
            for book_move in moves {
                if random_value < book_move.weight {
                    return Some(self.convert_to_engine_move(book_move));
                }
                random_value -= book_move.weight;
            }
        }
        None
    }

    fn hash_fen(&self, fen: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        fen.hash(&mut hasher);
        hasher.finish()
    }

    fn convert_to_engine_move(&self, book_move: &BookMove) -> Move {
        Move {
            from: book_move.from,
            to: book_move.to,
            piece_type: book_move.piece_type,
            player: Player::Black, // Will be corrected by caller
            is_promotion: book_move.is_promotion,
            is_capture: false, // Will be determined by engine
            captured_piece: None,
            gives_check: false, // Will be determined by engine
            is_recapture: false,
        }
    }
}
```

### 3.3 Integration with Engine

```rust
// In src/lib.rs
impl ShogiEngine {
    pub fn get_best_move(&self, depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>, on_info: Option<js_sys::Function>) -> Option<Move> {
        // 1. Check opening book first
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        
        if let Some(mut book_move) = self.opening_book.get_best_move(&fen) {
            // Correct the player and determine other move properties
            book_move.player = self.current_player;
            book_move.is_capture = self.board.get_piece(book_move.to).is_some();
            book_move.gives_check = self.would_give_check(&book_move);
            
            if let Some(on_info) = &on_info {
                let info = format!("info string Opening book move: {} -> {}", 
                    book_move.from.map_or("drop".to_string(), |f| format!("{}{}", f.col + 1, (f.row + b'a') as char)),
                    format!("{}{}", book_move.to.col + 1, (book_move.to.row + b'a') as char)
                );
                let _ = on_info.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(&info));
            }
            
            return Some(book_move);
        }

        // 2. If no book move, proceed with search
        // ... existing search logic ...
    }
}
```

## 4. Migration Strategy

### 4.1 Phase 1: Create New Format
1. Design and implement the binary format parser
2. Create conversion tools from JSON to binary
3. Implement the new `OpeningBook` struct

### 4.2 Phase 2: Data Migration
1. Convert existing `openingBook.json` to new format
2. Add move weights and evaluations based on professional games
3. Expand the opening book with more positions and variations

### 4.3 Phase 3: Integration
1. Replace old opening book implementation
2. Add comprehensive testing
3. Optimize for WASM performance

## 5. Advanced Features

### 5.1 Move Weighting System
- **Frequency**: How often the move is played in professional games
- **Success Rate**: Win percentage when this move is played
- **Evaluation**: Engine evaluation of the resulting position
- **Novelty**: How often the engine has played this move recently

### 5.2 Adaptive Learning
- Track which book moves lead to wins/losses
- Adjust move weights based on engine performance
- Learn from opponent responses to book moves

### 5.3 Opening Classification
- Tag positions with opening names (Yagura, Kakugawari, etc.)
- Provide opening statistics and recommendations
- Support for different playing styles (aggressive, positional, etc.)

## 6. Performance Optimizations

### 6.1 Memory Efficiency
- Use `Box<[u8]>` for binary data instead of `Vec<u8>`
- Implement position compression for similar positions
- Lazy loading of rarely accessed positions

### 6.2 Lookup Optimization
- Use perfect hashing for position lookup
- Implement position caching for frequently accessed FENs
- Pre-compute common position hashes

### 6.3 WASM-Specific Optimizations
- Minimize heap allocations
- Use `wasm_bindgen` efficiently for data transfer
- Implement streaming for large opening books

## 7. Verification and Testing

### 7.1 Unit Tests
- Test binary format parsing and generation
- Verify move conversion accuracy
- Test hash function consistency

### 7.2 Integration Tests
- Test opening book integration with search engine
- Verify move selection algorithms
- Test performance with large opening books

### 7.3 Gameplay Tests
- Play complete games using only opening book
- Verify move quality and variety
- Test against different opponent strategies

### 7.4 Performance Benchmarks
- Measure lookup times for various positions
- Compare memory usage vs. JSON format
- Test WASM binary size impact

## 8. Future Enhancements

### 8.1 Dynamic Opening Book
- Update opening book based on game results
- Learn from engine's search improvements
- Adapt to opponent playing style

### 8.2 Multi-Engine Support
- Support different opening books for different playing styles
- A/B testing of opening variations
- Personalized opening recommendations

### 8.3 Cloud Integration
- Sync opening book updates from server
- Share successful opening lines with community
- Download additional opening databases

