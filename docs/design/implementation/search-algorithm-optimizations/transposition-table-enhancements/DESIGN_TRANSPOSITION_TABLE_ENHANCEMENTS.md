# Transposition Table Enhancements Design

## Overview

This document outlines the design for implementing efficient transposition table enhancements in the Shogi engine. The transposition table serves as a cache for previously searched positions, eliminating duplicate work and significantly improving search performance.

## Current State

The engine currently uses a basic hash table with string keys for position caching, which is inefficient and limited in scope.

## Design Goals

1. **High Performance**: Fast position lookup and storage
2. **Memory Efficiency**: Optimal memory usage with configurable size
3. **Collision Handling**: Robust handling of hash collisions
4. **Cache Management**: Effective replacement policies
5. **Thread Safety**: Support for parallel search operations

## Technical Architecture

### 1. Zobrist Hashing System

**Purpose**: Generate unique hash keys for board positions using bitwise operations.

**Components**:
- Piece position keys: `[piece_type][position]` lookup table
- Side to move key: Single key for player turn
- Hand pieces keys: Keys for pieces in hand (dropped pieces)
- Repetition keys: Keys for position repetition tracking

**Implementation**:
```rust
struct ZobristTable {
    piece_keys: [[u64; 81]; 14], // [piece_type][position]
    side_key: u64,
    hand_keys: [[u64; 8]; 14], // [piece_type][count] - pieces in hand
    repetition_keys: [u64; 4], // For repetition tracking
}

impl ZobristTable {
    fn new() -> Self {
        let mut table = Self {
            piece_keys: [[0; 81]; 14],
            side_key: 0,
            hand_keys: [[0; 8]; 14],
            repetition_keys: [0; 4],
        };
        table.initialize_keys();
        table
    }
    
    fn initialize_keys(&mut self) {
        let mut rng = StdRng::seed_from_u64(0x123456789ABCDEF);
        
        // Initialize piece keys
        for piece_type in 0..14 {
            for position in 0..81 {
                self.piece_keys[piece_type][position] = rng.gen();
            }
        }
        
        // Initialize other keys
        self.side_key = rng.gen();
        
        // Initialize hand piece keys
        for piece_type in 0..14 {
            for count in 0..8 {
                self.hand_keys[piece_type][count] = rng.gen();
            }
        }
        
        // Initialize repetition keys
        for i in 0..4 {
            self.repetition_keys[i] = rng.gen();
        }
    }
}
```

### 2. Transposition Table Structure

**Purpose**: Store position evaluation results with metadata.

**Entry Structure**:
```rust
#[derive(Clone, Copy)]
struct TranspositionEntry {
    hash_key: u64,        // Full Zobrist hash key
    depth: u8,            // Search depth when stored
    score: i32,           // Evaluation score
    flag: TranspositionFlag, // Entry type (exact, alpha, beta)
    best_move: Option<Move>, // Best move found (if any)
    age: u8,              // Entry age for replacement
}

#[derive(Clone, Copy, PartialEq)]
enum TranspositionFlag {
    Exact,    // Score is exact evaluation
    Alpha,    // Score is lower bound (alpha cutoff)
    Beta,     // Score is upper bound (beta cutoff)
}
```

**Table Structure**:
```rust
struct TranspositionTable {
    table: Vec<Option<TranspositionEntry>>,
    size: usize,
    mask: usize,           // Size - 1 for fast modulo
    age: u8,              // Current age counter
    hits: u64,            // Cache hit counter
    misses: u64,          // Cache miss counter
}

impl TranspositionTable {
    fn new(size_mb: usize) -> Self {
        let size = (size_mb * 1024 * 1024) / std::mem::size_of::<Option<TranspositionEntry>>();
        let actual_size = size.next_power_of_two();
        
        Self {
            table: vec![None; actual_size],
            size: actual_size,
            mask: actual_size - 1,
            age: 0,
            hits: 0,
            misses: 0,
        }
    }
}
```

### 3. Hash Key Generation

**Purpose**: Generate incremental hash keys for position updates.

**Implementation**:
```rust
impl TranspositionTable {
    fn hash_position(&self, board: &BitboardBoard) -> u64 {
        let mut hash = 0;
        
        // XOR piece positions
        for piece_type in 0..14 {
            for position in 0..81 {
                if board.has_piece(piece_type, position) {
                    hash ^= self.zobrist_table.piece_keys[piece_type][position];
                }
            }
        }
        
        // XOR side to move
        if board.side_to_move() == Player::Black {
            hash ^= self.zobrist_table.side_key;
        }
        
        // XOR hand pieces (pieces in hand for drops)
        for piece_type in 0..14 {
            let count = board.pieces_in_hand(piece_type);
            if count > 0 {
                hash ^= self.zobrist_table.hand_keys[piece_type][count.min(7) as usize];
            }
        }
        
        // XOR repetition state
        let repetition_state = board.get_repetition_state();
        hash ^= self.zobrist_table.repetition_keys[repetition_state as usize];
        
        hash
    }
    
    fn update_hash_for_move(&self, hash: u64, mv: &Move, board: &BitboardBoard) -> u64 {
        let mut new_hash = hash;
        
        if mv.is_drop_move() {
            // Handle drop moves (pieces from hand)
            let piece_type = mv.piece_type;
            let current_count = board.pieces_in_hand(piece_type);
            
            // Remove piece from hand
            new_hash ^= self.zobrist_table.hand_keys[piece_type as usize][current_count as usize];
            
            // Add piece to board
            new_hash ^= self.zobrist_table.piece_keys[piece_type as usize][mv.to as usize];
        } else {
            // Handle regular moves
            // Remove piece from source square
            new_hash ^= self.zobrist_table.piece_keys[mv.piece_type as usize][mv.from as usize];
            
            // Add piece to destination square
            new_hash ^= self.zobrist_table.piece_keys[mv.piece_type as usize][mv.to as usize];
            
            // Handle captures
            if let Some(captured_piece) = board.piece_at(mv.to) {
                new_hash ^= self.zobrist_table.piece_keys[captured_piece as usize][mv.to as usize];
                
                // Add captured piece to hand
                let new_hand_count = board.pieces_in_hand(captured_piece);
                new_hash ^= self.zobrist_table.hand_keys[captured_piece as usize][new_hand_count as usize];
            }
            
            // Handle promotions
            if mv.is_promotion() {
                // Remove original piece from destination
                new_hash ^= self.zobrist_table.piece_keys[mv.piece_type as usize][mv.to as usize];
                
                // Add promoted piece to destination
                if let Some(promoted_piece) = mv.promotion {
                    new_hash ^= self.zobrist_table.piece_keys[promoted_piece as usize][mv.to as usize];
                }
            }
        }
        
        // Toggle side to move
        new_hash ^= self.zobrist_table.side_key;
        
        new_hash
    }
}
```

### 4. Replacement Policies

**Purpose**: Determine which entries to replace when the table is full.

**Strategies**:

1. **Always Replace**: Replace any existing entry
2. **Depth Preferred**: Keep entries with higher search depth
3. **Aging**: Replace older entries first

**Implementation**:
```rust
impl TranspositionTable {
    fn store(&mut self, hash_key: u64, entry: TranspositionEntry) {
        let index = (hash_key as usize) & self.mask;
        
        let should_replace = match &self.table[index] {
            None => true, // Empty slot
            Some(existing) => {
                // Always replace if different position
                if existing.hash_key != hash_key {
                    true
                } else {
                    // Same position - replace if new entry has better data
                    self.should_replace_entry(*existing, entry)
                }
            }
        };
        
        if should_replace {
            self.table[index] = Some(entry);
        }
    }
    
    fn should_replace_entry(&self, existing: TranspositionEntry, new: TranspositionEntry) -> bool {
        // Prefer exact scores over bounds
        if new.flag == TranspositionFlag::Exact && existing.flag != TranspositionFlag::Exact {
            return true;
        }
        
        // Prefer higher depth
        if new.depth > existing.depth {
            return true;
        }
        
        // Prefer newer entries (age-based replacement)
        if new.age > existing.age {
            return true;
        }
        
        false
    }
}
```

### 5. Cache Management

**Purpose**: Maintain cache efficiency and statistics.

**Features**:
- Hit/miss ratio tracking
- Age-based entry expiration
- Memory usage monitoring
- Cache warming strategies

**Implementation**:
```rust
impl TranspositionTable {
    fn probe(&mut self, hash_key: u64) -> Option<TranspositionEntry> {
        let index = (hash_key as usize) & self.mask;
        
        if let Some(entry) = &self.table[index] {
            if entry.hash_key == hash_key {
                self.hits += 1;
                return Some(*entry);
            }
        }
        
        self.misses += 1;
        None
    }
    
    fn clear(&mut self) {
        self.table.fill(None);
        self.age = 0;
        self.hits = 0;
        self.misses = 0;
    }
    
    fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
    
    fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
    
    fn get_memory_usage(&self) -> usize {
        self.size * std::mem::size_of::<Option<TranspositionEntry>>()
    }
}
```

### 6. Thread Safety

**Purpose**: Support parallel search operations.

**Requirements**:
- Lock-free operations where possible
- Minimal contention between threads
- Consistent cache state across threads

**Implementation**:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct ThreadSafeTranspositionTable {
    table: Vec<AtomicU64>, // Packed entry data
    size: usize,
    mask: usize,
}

impl ThreadSafeTranspositionTable {
    fn store_atomic(&self, hash_key: u64, entry: TranspositionEntry) {
        let index = (hash_key as usize) & self.mask;
        let packed_entry = self.pack_entry(hash_key, entry);
        
        // Atomic store with compare-and-swap if needed
        loop {
            let current = self.table[index].load(Ordering::Acquire);
            if let Ok(_) = self.table[index].compare_exchange_weak(
                current, 
                packed_entry, 
                Ordering::Release, 
                Ordering::Acquire
            ) {
                break;
            }
        }
    }
}
```

## Performance Considerations

### Memory Layout

- **Cache Line Alignment**: Align entries to cache line boundaries
- **Data Packing**: Pack related data together for better cache utilization
- **Prefetching**: Prefetch likely cache entries

### Hash Quality

- **Distribution**: Ensure uniform hash distribution
- **Collision Rate**: Monitor and minimize hash collisions
- **Key Size**: Balance between collision rate and memory usage

### Replacement Strategy

- **Hit Rate Optimization**: Maximize cache hit rate
- **Depth Preference**: Prioritize deeper search results
- **Age Management**: Prevent stale entries from accumulating

## Integration Points

### Search Algorithm Integration

```rust
impl SearchEngine {
    fn negamax_with_tt(&mut self, board: &mut BitboardBoard, depth: u8, 
                       alpha: i32, beta: i32) -> i32 {
        let hash_key = self.transposition_table.hash_position(board);
        
        // Probe transposition table
        if let Some(entry) = self.transposition_table.probe(hash_key) {
            if entry.depth >= depth {
                match entry.flag {
                    TranspositionFlag::Exact => return entry.score,
                    TranspositionFlag::Alpha => {
                        if entry.score <= alpha { return alpha; }
                    },
                    TranspositionFlag::Beta => {
                        if entry.score >= beta { return beta; }
                    },
                }
            }
        }
        
        // Perform search...
        let score = self.negamax_search(board, depth, alpha, beta);
        
        // Store result in transposition table
        let flag = if score <= alpha {
            TranspositionFlag::Alpha
        } else if score >= beta {
            TranspositionFlag::Beta
        } else {
            TranspositionFlag::Exact
        };
        
        let entry = TranspositionEntry {
            hash_key,
            depth,
            score,
            flag,
            best_move: self.best_move,
            age: self.transposition_table.age,
        };
        
        self.transposition_table.store(hash_key, entry);
        score
    }
}
```

### Move Generation Integration

```rust
impl MoveOrdering {
    fn order_moves_with_tt(&mut self, moves: &mut Vec<Move>, 
                          board: &BitboardBoard, 
                          tt: &TranspositionTable) {
        // Get best move from transposition table
        let hash_key = tt.hash_position(board);
        if let Some(entry) = tt.probe(hash_key) {
            if let Some(best_move) = entry.best_move {
                // Move best move to front
                if let Some(pos) = moves.iter().position(|&m| m == best_move) {
                    moves.swap(0, pos);
                }
            }
        }
        
        // Continue with normal move ordering...
        self.order_remaining_moves(moves, board);
    }
}
```

## Testing Strategy

### Unit Tests

1. **Hash Key Generation**: Verify unique keys for different positions
2. **Entry Storage**: Test storage and retrieval operations
3. **Replacement Logic**: Verify replacement policy behavior
4. **Collision Handling**: Test collision scenarios

### Performance Tests

1. **Hit Rate Measurement**: Monitor cache effectiveness
2. **Memory Usage**: Verify memory consumption stays within limits
3. **Speed Benchmarks**: Measure lookup and storage performance
4. **Parallel Access**: Test thread safety under load

### Integration Tests

1. **Search Integration**: Verify correct integration with search algorithm
2. **Move Ordering**: Test move ordering improvements
3. **Endgame Performance**: Test performance in endgame positions
4. **Opening Performance**: Test performance in opening positions

## Configuration Options

### Memory Allocation

- **Table Size**: Configurable memory allocation (16MB - 1GB)
- **Entry Size**: Optimized entry structure size
- **Cache Lines**: Alignment to processor cache lines

### Replacement Policy

- **Always Replace**: Simple replacement strategy
- **Depth Preferred**: Depth-based replacement
- **Aging**: Age-based replacement with configurable parameters

### Performance Tuning

- **Hash Quality**: Configurable hash key generation
- **Collision Handling**: Adjustable collision resolution
- **Thread Safety**: Configurable locking mechanisms

## Expected Performance Impact

### Search Performance

- **2-3x Reduction**: In duplicate position searches
- **15-25% Improvement**: In overall search speed
- **Better Move Ordering**: From transposition table hints

### Memory Usage

- **16-64MB**: Typical memory allocation
- **Linear Scaling**: Memory usage scales with table size
- **Cache Efficiency**: Optimized for CPU cache utilization

### Hit Rate Targets

- **60-80% Hit Rate**: In typical search positions
- **Higher Hit Rates**: In endgame and tactical positions
- **Lower Hit Rates**: In opening and quiet positions

## Future Enhancements

### Advanced Features

1. **Multi-Level Tables**: Separate tables for different search depths
2. **Compressed Entries**: Reduce memory usage with compression
3. **Predictive Prefetching**: Prefetch likely cache entries
4. **Dynamic Sizing**: Adjust table size based on available memory

### Optimization Opportunities

1. **WASM Compatibility**: Ensure all optimizations work in WebAssembly
2. **Cross-Platform Performance**: Optimize for both native and WASM targets
3. **Machine Learning**: Learn optimal replacement strategies
4. **Adaptive Policies**: Adjust policies based on game phase

### WASM-Specific Considerations

1. **Memory Management**: Use WASM-compatible memory allocation patterns
2. **Performance Portability**: Ensure optimizations work across platforms
3. **Size Optimization**: Minimize WASM binary size impact
4. **Runtime Compatibility**: Test thoroughly in browser environments

## Conclusion

The transposition table enhancement design provides a comprehensive solution for caching search results in the Shogi engine. The implementation focuses on performance, memory efficiency, and thread safety while maintaining simplicity and maintainability.

Key benefits include:
- Significant reduction in duplicate search work
- Improved move ordering through best move storage
- Better overall search performance
- Scalable memory usage
- Thread-safe parallel search support

The design provides a solid foundation for future enhancements while delivering immediate performance improvements to the search algorithm.
