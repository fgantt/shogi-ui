# Opening Book Optimization Implementation Plan

## Overview

This document outlines the implementation plan for optimizing the opening book system in the Shogi engine. The goal is to transform the current JSON-based opening book into a high-performance, binary-searchable structure that provides near-instant lookups.

## Current State

- **Implementation**: JSON-based opening book loaded at runtime
- **Lookup Method**: Sequential or hash-based lookup
- **Performance**: ~1-10ms per lookup (depending on book size)
- **Memory Usage**: Uncompressed JSON in memory
- **Size**: Variable, typically 500KB-5MB uncompressed

### Current Issues

1. **Slow Lookups**: Linear search or hash collisions cause delays
2. **Memory Overhead**: JSON parsing and storage inefficient
3. **No Incremental Loading**: Entire book loaded at startup
4. **Poor Cache Locality**: Scattered memory access patterns
5. **Format Limitations**: JSON not optimized for binary data

## Objectives

1. Reduce opening book lookup time to <100μs
2. Minimize memory footprint through compression
3. Enable incremental/lazy loading of book sections
4. Improve cache locality for frequently accessed positions
5. Support large opening books (10MB+ compressed)
6. Maintain backward compatibility with existing book data

## Technical Approach

### Optimized Data Structure

```rust
use std::collections::HashMap;
use std::io::{self, Read, Write};
use bincode::{serialize, deserialize};
use lz4::{Decoder, EncoderBuilder};

/// Optimized opening book with binary search and compression
pub struct OptimizedOpeningBook {
    /// Sorted array of position hashes
    position_hashes: Vec<u64>,
    
    /// Move data for each position (offset into moves array)
    move_offsets: Vec<u32>,
    
    /// Count of moves per position
    move_counts: Vec<u16>,
    
    /// Flattened array of all moves
    moves: Vec<BookMove>,
    
    /// LRU cache for frequently accessed positions
    cache: LruCache<u64, Vec<BookMove>>,
    
    /// Statistics for book usage
    stats: BookStats,
}

/// Individual move in the opening book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookMove {
    /// Move in compact representation
    pub move_data: u32,
    
    /// Weight/frequency (higher = more popular)
    pub weight: u16,
    
    /// Win rate (0-10000, representing 0.00% - 100.00%)
    pub win_rate: u16,
    
    /// Number of games with this move
    pub game_count: u32,
}

/// Opening book statistics
#[derive(Default, Debug)]
pub struct BookStats {
    pub total_positions: usize,
    pub total_moves: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_moves_per_position: f32,
}

impl OptimizedOpeningBook {
    /// Create a new opening book from raw position data
    pub fn new(positions: Vec<(u64, Vec<BookMove>)>) -> Self {
        let mut sorted_positions = positions;
        sorted_positions.sort_by_key(|(hash, _)| *hash);
        
        let mut position_hashes = Vec::new();
        let mut move_offsets = Vec::new();
        let mut move_counts = Vec::new();
        let mut moves = Vec::new();
        
        for (hash, position_moves) in sorted_positions {
            position_hashes.push(hash);
            move_offsets.push(moves.len() as u32);
            move_counts.push(position_moves.len() as u16);
            moves.extend(position_moves);
        }
        
        let stats = BookStats {
            total_positions: position_hashes.len(),
            total_moves: moves.len(),
            cache_hits: 0,
            cache_misses: 0,
            avg_moves_per_position: moves.len() as f32 / position_hashes.len() as f32,
        };
        
        Self {
            position_hashes,
            move_offsets,
            move_counts,
            moves,
            cache: LruCache::new(1024), // Cache 1024 positions
            stats,
        }
    }
    
    /// Lookup moves for a position using binary search
    pub fn lookup(&mut self, position_hash: u64) -> Option<Vec<BookMove>> {
        // Check cache first
        if let Some(moves) = self.cache.get(&position_hash) {
            self.stats.cache_hits += 1;
            return Some(moves.clone());
        }
        
        self.stats.cache_misses += 1;
        
        // Binary search for position
        let index = self.position_hashes
            .binary_search(&position_hash)
            .ok()?;
        
        // Extract moves for this position
        let start = self.move_offsets[index] as usize;
        let count = self.move_counts[index] as usize;
        let moves = self.moves[start..start + count].to_vec();
        
        // Add to cache
        self.cache.put(position_hash, moves.clone());
        
        Some(moves)
    }
    
    /// Get a random move weighted by popularity
    pub fn get_weighted_move(&mut self, position_hash: u64) -> Option<BookMove> {
        let moves = self.lookup(position_hash)?;
        
        // Calculate total weight
        let total_weight: u32 = moves.iter().map(|m| m.weight as u32).sum();
        
        if total_weight == 0 {
            return moves.first().cloned();
        }
        
        // Random selection weighted by move weights
        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_weight);
        
        for mv in moves {
            if random_weight < mv.weight as u32 {
                return Some(mv);
            }
            random_weight -= mv.weight as u32;
        }
        
        None
    }
    
    /// Get the best move by win rate
    pub fn get_best_move(&mut self, position_hash: u64) -> Option<BookMove> {
        let moves = self.lookup(position_hash)?;
        
        moves.into_iter()
            .max_by_key(|m| m.win_rate)
            .clone()
    }
    
    /// Serialize to compressed binary format
    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let serialized = serialize(&self).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Serialization error: {}", e))
        })?;
        
        let mut file = std::fs::File::create(path)?;
        let mut encoder = EncoderBuilder::new()
            .level(4)
            .build(&mut file)?;
        
        encoder.write_all(&serialized)?;
        encoder.finish().1?;
        
        Ok(())
    }
    
    /// Deserialize from compressed binary format
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut decoder = Decoder::new(file)?;
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        deserialize(&decompressed).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Deserialization error: {}", e))
        })
    }
}

/// LRU Cache implementation for opening book
use std::collections::LinkedHashMap;

pub struct LruCache<K, V> {
    map: LinkedHashMap<K, V>,
    capacity: usize,
}

impl<K: std::hash::Hash + Eq, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: LinkedHashMap::new(),
            capacity,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to end (most recently used)
            let value = self.map.remove(key)?;
            self.map.insert(*key, value);
            self.map.get(key)
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity {
            // Remove least recently used
            self.map.pop_front();
        }
        self.map.insert(key, value);
    }
}
```

## Implementation Phases

### Phase 1: Binary Format Design (Week 1)

**Tasks:**
- [ ] Design binary file format specification
- [ ] Implement serialization/deserialization
- [ ] Add compression support (LZ4)
- [ ] Create file format version header

**Deliverables:**
- Binary format specification document
- Serialization utilities
- Format conversion tools

### Phase 2: Data Structure Implementation (Week 1-2)

**Tasks:**
- [ ] Implement `OptimizedOpeningBook` structure
- [ ] Add binary search lookup
- [ ] Implement LRU cache
- [ ] Add statistics tracking

**Deliverables:**
- Core data structure
- Lookup algorithms
- Cache implementation

### Phase 3: Book Builder (Week 2)

**Tasks:**
- [ ] Create tool to convert existing JSON books
- [ ] Implement book merger (combine multiple sources)
- [ ] Add book validation
- [ ] Create book statistics analyzer

**Deliverables:**
- `book_builder` binary tool
- Conversion utilities
- Validation suite

### Phase 4: Integration (Week 3)

**Tasks:**
- [ ] Integrate with existing search engine
- [ ] Add book probing to search
- [ ] Implement move selection strategies
- [ ] Add book statistics to engine output

**Deliverables:**
- Integrated opening book
- Move selection logic
- USI book commands

### Phase 5: Advanced Features (Week 3-4)

**Tasks:**
- [ ] Implement incremental book loading
- [ ] Add position learning (update book from games)
- [ ] Create book trimming tools (remove rare positions)
- [ ] Add book quality metrics

**Deliverables:**
- Incremental loading
- Learning system
- Book maintenance tools

### Phase 6: Testing & Optimization (Week 4)

**Tasks:**
- [ ] Performance benchmarking
- [ ] Memory profiling
- [ ] Cache hit rate optimization
- [ ] Compression ratio analysis

**Deliverables:**
- Performance report
- Optimization recommendations
- Benchmark suite

## Book File Format

### Binary Structure

```
+------------------+
| Header (64 bytes)|
+------------------+
| Position Hashes  |
| (8 bytes each)   |
+------------------+
| Move Offsets     |
| (4 bytes each)   |
+------------------+
| Move Counts      |
| (2 bytes each)   |
+------------------+
| Move Data        |
| (12 bytes each)  |
+------------------+

Header Layout:
- Magic number (4 bytes): "SHOB" (Shogi Opening Book)
- Version (2 bytes): Format version number
- Flags (2 bytes): Feature flags
- Position count (4 bytes): Number of positions
- Move count (4 bytes): Total number of moves
- Compression (1 byte): Compression algorithm (0=none, 1=lz4)
- Reserved (47 bytes): Future use
```

### Compression Strategy

1. **Position Hashes**: Store as-is (already compact)
2. **Moves**: Delta encoding for similar positions
3. **Weights**: Variable-length encoding for common values
4. **Overall**: LZ4 compression on entire file

Expected compression ratio: 3:1 to 5:1

## Book Building Pipeline

### 1. Source Data Collection

```rust
pub struct BookBuilder {
    positions: HashMap<u64, PositionData>,
}

pub struct PositionData {
    moves: HashMap<u32, MoveStats>,
}

pub struct MoveStats {
    wins: u32,
    draws: u32,
    losses: u32,
    total_games: u32,
}

impl BookBuilder {
    /// Add game to book
    pub fn add_game(&mut self, game: &Game, result: GameResult) {
        let mut board = BitboardBoard::starting_position();
        
        for (i, mv) in game.moves.iter().enumerate() {
            if i >= 20 { break; } // Only first 20 moves
            
            let hash = board.hash();
            let position = self.positions.entry(hash).or_insert_with(PositionData::new);
            
            let move_stats = position.moves.entry(mv.to_u32()).or_insert_with(MoveStats::new);
            move_stats.total_games += 1;
            
            match result {
                GameResult::Win if i % 2 == 0 => move_stats.wins += 1,
                GameResult::Loss if i % 2 == 1 => move_stats.wins += 1,
                GameResult::Draw => move_stats.draws += 1,
                _ => move_stats.losses += 1,
            }
            
            board.make_move(*mv);
        }
    }
    
    /// Build optimized book
    pub fn build(&self) -> OptimizedOpeningBook {
        let mut positions = Vec::new();
        
        for (hash, position_data) in &self.positions {
            let mut moves = Vec::new();
            
            for (move_data, stats) in &position_data.moves {
                let win_rate = if stats.total_games > 0 {
                    ((stats.wins as f32 / stats.total_games as f32) * 10000.0) as u16
                } else {
                    5000 // 50% default
                };
                
                moves.push(BookMove {
                    move_data: *move_data,
                    weight: (stats.total_games.min(65535)) as u16,
                    win_rate,
                    game_count: stats.total_games,
                });
            }
            
            // Sort moves by popularity
            moves.sort_by_key(|m| std::cmp::Reverse(m.weight));
            
            positions.push((*hash, moves));
        }
        
        OptimizedOpeningBook::new(positions)
    }
}
```

### 2. Book Conversion Tool

```bash
# Convert existing JSON book
cargo run --bin book_builder -- \
    --input opening_book.json \
    --output opening_book.bin \
    --compress lz4 \
    --min-games 5

# Merge multiple books
cargo run --bin book_builder -- \
    --merge book1.bin book2.bin book3.bin \
    --output merged_book.bin

# Analyze book statistics
cargo run --bin book_builder -- \
    --analyze opening_book.bin \
    --report book_stats.txt
```

## Performance Targets

### Lookup Performance

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Lookup time | 1-10ms | <100μs | 10-100x |
| Cache hit rate | N/A | >90% | New |
| Memory usage | 5MB | 1-2MB | 2.5-5x |
| Load time | 50-100ms | <10ms | 5-10x |

### Benchmark Suite

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_opening_book(c: &mut Criterion) {
    let mut book = OptimizedOpeningBook::load_from_file("opening_book.bin").unwrap();
    let position = BitboardBoard::starting_position();
    let hash = position.hash();
    
    c.bench_function("book_lookup", |b| {
        b.iter(|| {
            book.lookup(black_box(hash))
        })
    });
    
    c.bench_function("book_weighted_move", |b| {
        b.iter(|| {
            book.get_weighted_move(black_box(hash))
        })
    });
    
    c.bench_function("book_best_move", |b| {
        b.iter(|| {
            book.get_best_move(black_box(hash))
        })
    });
}

criterion_group!(benches, benchmark_opening_book);
criterion_main!(benches);
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_search_lookup() {
        let positions = vec![
            (100, vec![BookMove::new(1, 10, 6000, 100)]),
            (200, vec![BookMove::new(2, 20, 5500, 200)]),
            (300, vec![BookMove::new(3, 30, 7000, 300)]),
        ];
        
        let mut book = OptimizedOpeningBook::new(positions);
        
        assert!(book.lookup(200).is_some());
        assert_eq!(book.lookup(200).unwrap().len(), 1);
        assert!(book.lookup(999).is_none());
    }
    
    #[test]
    fn test_cache_functionality() {
        let positions = vec![
            (100, vec![BookMove::new(1, 10, 6000, 100)]),
        ];
        
        let mut book = OptimizedOpeningBook::new(positions);
        
        // First lookup - cache miss
        book.lookup(100);
        assert_eq!(book.stats.cache_misses, 1);
        assert_eq!(book.stats.cache_hits, 0);
        
        // Second lookup - cache hit
        book.lookup(100);
        assert_eq!(book.stats.cache_hits, 1);
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let positions = vec![
            (100, vec![BookMove::new(1, 10, 6000, 100)]),
            (200, vec![BookMove::new(2, 20, 5500, 200)]),
        ];
        
        let book = OptimizedOpeningBook::new(positions);
        book.save_to_file("test_book.bin").unwrap();
        
        let loaded_book = OptimizedOpeningBook::load_from_file("test_book.bin").unwrap();
        
        assert_eq!(book.position_hashes, loaded_book.position_hashes);
        assert_eq!(book.moves, loaded_book.moves);
    }
}
```

### Integration Tests

1. **Book Conversion Tests**
   - Convert existing JSON books
   - Verify data integrity
   - Compare lookup results

2. **Performance Tests**
   - Benchmark lookup times
   - Measure cache effectiveness
   - Profile memory usage

3. **Search Integration Tests**
   - Verify book moves are played
   - Test move selection strategies
   - Validate game outcomes

## Dependencies

```toml
[dependencies]
bincode = "1.3"           # Binary serialization
lz4 = "1.24"             # Compression
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"             # Random move selection

[dev-dependencies]
criterion = "0.5"        # Benchmarking
```

## Migration Strategy

### Phase 1: Parallel Support
- Support both JSON and binary formats
- Auto-detect format on load
- Convert JSON to binary on first load

### Phase 2: Deprecation
- Mark JSON format as deprecated
- Provide conversion tools
- Update documentation

### Phase 3: Binary Only
- Remove JSON support
- Cleanup legacy code
- Binary format as default

## Book Maintenance Tools

### 1. Book Statistics

```bash
cargo run --bin book_tools -- stats opening_book.bin
```

Output:
```
Opening Book Statistics
=======================
Total positions: 15,432
Total moves: 47,891
Average moves per position: 3.1
Most popular position: 0x1234abcd5678ef90 (234 games)
Compression ratio: 4.2:1
File size: 1.2 MB (compressed), 5.1 MB (uncompressed)
```

### 2. Book Trimming

```bash
# Remove positions with <10 games
cargo run --bin book_tools -- trim \
    --input opening_book.bin \
    --output trimmed_book.bin \
    --min-games 10
```

### 3. Book Merging

```bash
# Merge multiple books with conflict resolution
cargo run --bin book_tools -- merge \
    --inputs book1.bin book2.bin book3.bin \
    --output merged.bin \
    --strategy weighted-average
```

## Success Criteria

- [ ] Lookup time <100μs (99th percentile)
- [ ] Memory usage <2MB for typical book
- [ ] Cache hit rate >90% in typical games
- [ ] Load time <10ms
- [ ] Compression ratio >3:1
- [ ] Zero data loss in conversion
- [ ] Support for 100,000+ positions

## Future Enhancements

1. **Polyglot Book Format**
   - Support UCI/USI polyglot format
   - Cross-engine compatibility

2. **Cloud-Based Books**
   - Download book sections on-demand
   - Share book updates across instances

3. **Neural Network Integration**
   - Use NN evaluations as book weights
   - Hybrid opening book + NN

4. **Adaptive Book Learning**
   - Update book from online games
   - Remove underperforming lines

## References

- [Polyglot Book Format](https://www.chessprogramming.org/PolyGlot)
- [Opening Book Techniques](https://www.chessprogramming.org/Opening_Book)
- [LZ4 Compression](https://github.com/lz4/lz4)
- [Bincode Documentation](https://docs.rs/bincode/)

