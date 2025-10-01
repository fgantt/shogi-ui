# Endgame Tablebase Integration Implementation Plan

## Overview

This document outlines the implementation plan for optimizing the endgame tablebase system in the Shogi engine. Endgame tablebases provide perfect play information for positions with few pieces, enabling the engine to play endgames with absolute precision.

## Current State

- **Implementation**: Basic tablebase implementation exists
- **Coverage**: Limited piece combinations
- **Performance**: Unoptimized lookups
- **Storage**: Uncompressed or basic compression
- **Integration**: Basic integration with search

### Current Limitations

1. **Slow Lookups**: No caching or indexing optimization
2. **Large Storage**: Uncompressed tablebase files
3. **Limited Coverage**: Only basic endgame positions
4. **Memory Usage**: Full tablebase loaded in memory
5. **Poor Integration**: Not efficiently integrated with search

## Objectives

1. Implement high-performance tablebase probe (<1ms per lookup)
2. Achieve significant compression (>10:1 ratio)
3. Support incremental/on-demand loading
4. Cover all 3-piece, 4-piece, and critical 5-piece endgames
5. Seamless integration with search algorithm
6. Provide DTM (Distance To Mate) and DTZ (Distance To Zero) metrics

## Technical Approach

### Tablebase Architecture

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use lru::LruCache;
use bincode::{serialize, deserialize};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Tablebase result for a position
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TablebaseResult {
    /// Win in N moves (half-moves)
    Win(u16),
    
    /// Loss in N moves (half-moves)
    Loss(u16),
    
    /// Draw
    Draw,
    
    /// Position not in tablebase
    Unknown,
}

/// Compressed tablebase entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TablebaseEntry {
    /// Result of the position
    pub result: TablebaseResult,
    
    /// Best move from this position
    pub best_move: Option<Move>,
    
    /// DTZ (Distance to zeroing move - capture/pawn move)
    pub dtz: u16,
}

/// Optimized tablebase with compression and caching
pub struct OptimizedTablebase {
    /// Piece configuration (e.g., "KRvK" for King+Rook vs King)
    config: String,
    
    /// Compressed tablebase data
    compressed_data: Vec<u8>,
    
    /// Index mapping position hash to compressed data offset
    position_index: HashMap<u64, u32>,
    
    /// LRU cache for recently accessed positions
    cache: Arc<RwLock<LruCache<u64, TablebaseEntry>>>,
    
    /// Decompression buffer (reused)
    decompress_buffer: Vec<u8>,
    
    /// Statistics
    stats: TablebaseStats,
}

/// Tablebase statistics
#[derive(Default, Debug)]
pub struct TablebaseStats {
    pub total_positions: usize,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub probe_count: u64,
}

impl OptimizedTablebase {
    /// Probe tablebase for position result
    pub fn probe(&mut self, board: &BitboardBoard) -> TablebaseResult {
        let position_hash = self.compute_tablebase_hash(board);
        
        self.stats.probe_count += 1;
        
        // Check cache first
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(entry) = cache.get(&position_hash) {
                self.stats.cache_hits += 1;
                return entry.result;
            }
        }
        
        self.stats.cache_misses += 1;
        
        // Look up in index
        let offset = match self.position_index.get(&position_hash) {
            Some(&off) => off,
            None => return TablebaseResult::Unknown,
        };
        
        // Decompress and decode entry
        let entry = match self.decompress_entry(offset) {
            Ok(entry) => entry,
            Err(_) => return TablebaseResult::Unknown,
        };
        
        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.put(position_hash, entry.clone());
        }
        
        entry.result
    }
    
    /// Probe and return full entry with best move
    pub fn probe_full(&mut self, board: &BitboardBoard) -> Option<TablebaseEntry> {
        let position_hash = self.compute_tablebase_hash(board);
        
        // Check cache
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(entry) = cache.get(&position_hash) {
                return Some(entry.clone());
            }
        }
        
        // Look up and decompress
        let offset = self.position_index.get(&position_hash)?;
        let entry = self.decompress_entry(*offset).ok()?;
        
        // Cache and return
        {
            let mut cache = self.cache.write().unwrap();
            cache.put(position_hash, entry.clone());
        }
        
        Some(entry)
    }
    
    /// Compute position hash for tablebase lookup
    fn compute_tablebase_hash(&self, board: &BitboardBoard) -> u64 {
        // Normalize position (account for symmetries)
        let normalized = self.normalize_position(board);
        normalized.zobrist_hash()
    }
    
    /// Normalize position for tablebase (handle symmetries)
    fn normalize_position(&self, board: &BitboardBoard) -> BitboardBoard {
        let mut best_board = board.clone();
        let mut best_hash = board.zobrist_hash();
        
        // Try all symmetries and pick canonical form
        for symmetry in 0..8 {
            let transformed = self.apply_symmetry(board, symmetry);
            let hash = transformed.zobrist_hash();
            
            if hash < best_hash {
                best_hash = hash;
                best_board = transformed;
            }
        }
        
        best_board
    }
    
    /// Apply symmetry transformation
    fn apply_symmetry(&self, board: &BitboardBoard, symmetry: u8) -> BitboardBoard {
        match symmetry {
            0 => board.clone(),
            1 => board.flip_horizontal(),
            2 => board.flip_vertical(),
            3 => board.flip_diagonal(),
            4 => board.rotate_90(),
            5 => board.rotate_180(),
            6 => board.rotate_270(),
            7 => board.flip_color(),
            _ => board.clone(),
        }
    }
    
    /// Decompress tablebase entry
    fn decompress_entry(&mut self, offset: u32) -> Result<TablebaseEntry, String> {
        let offset = offset as usize;
        
        // Read compressed size (4 bytes)
        if offset + 4 > self.compressed_data.len() {
            return Err("Invalid offset".to_string());
        }
        
        let size = u32::from_le_bytes([
            self.compressed_data[offset],
            self.compressed_data[offset + 1],
            self.compressed_data[offset + 2],
            self.compressed_data[offset + 3],
        ]) as usize;
        
        // Decompress data
        let compressed_slice = &self.compressed_data[offset + 4..offset + 4 + size];
        
        let mut decoder = GzDecoder::new(compressed_slice);
        self.decompress_buffer.clear();
        
        use std::io::Read;
        decoder.read_to_end(&mut self.decompress_buffer)
            .map_err(|e| format!("Decompression error: {}", e))?;
        
        // Deserialize entry
        deserialize(&self.decompress_buffer)
            .map_err(|e| format!("Deserialization error: {}", e))
    }
    
    /// Load tablebase from file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        use std::fs::File;
        use std::io::Read;
        
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Deserialize header and data
        // Format: [header][index][compressed_data]
        
        // Parse header (64 bytes)
        if data.len() < 64 {
            return Err("Invalid tablebase file".to_string());
        }
        
        let magic = &data[0..4];
        if magic != b"SHTB" {
            return Err("Invalid magic number".to_string());
        }
        
        let config_len = data[4] as usize;
        let config = String::from_utf8(data[5..5 + config_len].to_vec())
            .map_err(|e| format!("Invalid config: {}", e))?;
        
        let index_offset = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;
        let index_size = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
        let data_offset = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
        
        // Parse index
        let index_data = &data[index_offset..index_offset + index_size];
        let position_index: HashMap<u64, u32> = deserialize(index_data)
            .map_err(|e| format!("Failed to deserialize index: {}", e))?;
        
        // Extract compressed data
        let compressed_data = data[data_offset..].to_vec();
        
        let stats = TablebaseStats {
            total_positions: position_index.len(),
            compressed_size: compressed_data.len(),
            uncompressed_size: 0, // Will be updated on access
            cache_hits: 0,
            cache_misses: 0,
            probe_count: 0,
        };
        
        Ok(Self {
            config,
            compressed_data,
            position_index,
            cache: Arc::new(RwLock::new(LruCache::new(10000))), // Cache 10k positions
            decompress_buffer: Vec::with_capacity(1024),
            stats,
        })
    }
}

/// Tablebase manager for multiple tablebase files
pub struct TablebaseManager {
    /// Map of piece configurations to tablebases
    tablebases: HashMap<String, OptimizedTablebase>,
    
    /// Available tablebase files
    available_configs: Vec<String>,
    
    /// Lazy loading enabled
    lazy_load: bool,
}

impl TablebaseManager {
    pub fn new(tablebase_dir: &str, lazy_load: bool) -> Self {
        let available_configs = Self::scan_tablebase_directory(tablebase_dir);
        
        let mut manager = Self {
            tablebases: HashMap::new(),
            available_configs,
            lazy_load,
        };
        
        if !lazy_load {
            // Load all tablebases immediately
            manager.load_all_tablebases(tablebase_dir);
        }
        
        manager
    }
    
    /// Probe tablebases for position
    pub fn probe(&mut self, board: &BitboardBoard, tablebase_dir: &str) -> TablebaseResult {
        let config = Self::position_to_config(board);
        
        // Load tablebase if not loaded (lazy loading)
        if self.lazy_load && !self.tablebases.contains_key(&config) {
            if self.available_configs.contains(&config) {
                let path = format!("{}/{}.shtb", tablebase_dir, config);
                if let Ok(tb) = OptimizedTablebase::load_from_file(&path) {
                    self.tablebases.insert(config.clone(), tb);
                }
            }
        }
        
        // Probe tablebase
        match self.tablebases.get_mut(&config) {
            Some(tb) => tb.probe(board),
            None => TablebaseResult::Unknown,
        }
    }
    
    /// Get piece configuration for position
    fn position_to_config(board: &BitboardBoard) -> String {
        let mut pieces: Vec<(Player, PieceType)> = Vec::new();
        
        for sq in 0..81 {
            if let Some(piece) = board.get_piece_at(sq) {
                pieces.push((piece.player, piece.piece_type));
            }
        }
        
        // Sort and format (e.g., "KRvK" for King+Rook vs King)
        Self::format_config(&pieces)
    }
    
    fn format_config(pieces: &[(Player, PieceType)]) -> String {
        let mut sente = Vec::new();
        let mut gote = Vec::new();
        
        for (player, piece_type) in pieces {
            if *player == Player::Sente {
                sente.push(piece_type.to_char());
            } else {
                gote.push(piece_type.to_char());
            }
        }
        
        sente.sort();
        gote.sort();
        
        format!("{}v{}", sente.iter().collect::<String>(), gote.iter().collect::<String>())
    }
    
    fn scan_tablebase_directory(dir: &str) -> Vec<String> {
        use std::fs;
        
        let mut configs = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".shtb") {
                        if let Some(config) = name.strip_suffix(".shtb") {
                            configs.push(config.to_string());
                        }
                    }
                }
            }
        }
        
        configs
    }
    
    fn load_all_tablebases(&mut self, dir: &str) {
        for config in &self.available_configs {
            let path = format!("{}/{}.shtb", dir, config);
            if let Ok(tb) = OptimizedTablebase::load_from_file(&path) {
                self.tablebases.insert(config.clone(), tb);
            }
        }
    }
}
```

## Implementation Phases

### Phase 1: Tablebase Generation (Week 1-2)

**Tasks:**
- [ ] Implement retrograde analysis algorithm
- [ ] Generate 3-piece tablebases (KvK, KRvK, etc.)
- [ ] Generate 4-piece tablebases
- [ ] Validate generated tablebases

**Deliverables:**
- Tablebase generator tool
- Complete 3-piece and 4-piece tablebases
- Validation suite

### Phase 2: Compression & Storage (Week 2-3)

**Tasks:**
- [ ] Implement position normalization (symmetries)
- [ ] Add compression (Gzip/LZ4)
- [ ] Create binary file format
- [ ] Implement indexing structure

**Deliverables:**
- Compressed tablebase format
- File format specification
- Compression utilities

### Phase 3: Lookup Optimization (Week 3-4)

**Tasks:**
- [ ] Implement LRU caching
- [ ] Add position hash indexing
- [ ] Optimize decompression
- [ ] Add lazy loading support

**Deliverables:**
- Optimized lookup functions
- Caching system
- Performance benchmarks

### Phase 4: Search Integration (Week 4-5)

**Tasks:**
- [ ] Integrate tablebase probes into search
- [ ] Implement DTM/DTZ distance metrics
- [ ] Add tablebase cutoffs in search
- [ ] Optimize probe timing

**Deliverables:**
- Integrated tablebase search
- Distance-to-mate tracking
- Search optimization

### Phase 5: Advanced Features (Week 5-6)

**Tasks:**
- [ ] Generate 5-piece tablebases (selective)
- [ ] Implement WDL (Win/Draw/Loss) tablebases
- [ ] Add tablebase verification tools
- [ ] Create tablebase statistics

**Deliverables:**
- Extended tablebase coverage
- WDL support
- Analysis tools

### Phase 6: Testing & Optimization (Week 6)

**Tasks:**
- [ ] Performance benchmarking
- [ ] Correctness validation
- [ ] Memory profiling
- [ ] Integration testing

**Deliverables:**
- Test suite
- Performance report
- Optimization guide

## Tablebase File Format

### Binary Structure

```
+------------------+
| Header (64 bytes)|
+------------------+
| Index Section    |
|  - Hash Table    |
|  - Offsets       |
+------------------+
| Data Section     |
|  - Compressed    |
|  - Entries       |
+------------------+

Header:
- Magic: "SHTB" (4 bytes)
- Config length (1 byte)
- Config string (up to 20 bytes)
- Version (2 bytes)
- Index offset (4 bytes)
- Index size (4 bytes)
- Data offset (4 bytes)
- Data size (4 bytes)
- Compression type (1 byte)
- Reserved (24 bytes)
```

## Retrograde Analysis Algorithm

### Generation Process

```rust
pub struct TablebaseGenerator {
    board_generator: BoardGenerator,
    results: HashMap<u64, TablebaseEntry>,
}

impl TablebaseGenerator {
    /// Generate tablebase using retrograde analysis
    pub fn generate(&mut self, config: &str) -> Result<(), String> {
        // Step 1: Generate all legal positions
        let positions = self.board_generator.generate_all_positions(config);
        
        // Step 2: Identify terminal positions (checkmate, stalemate)
        self.mark_terminal_positions(&positions);
        
        // Step 3: Retrograde analysis (work backwards)
        let mut distance = 0;
        let mut changed = true;
        
        while changed {
            changed = false;
            distance += 1;
            
            for position in &positions {
                if self.results.contains_key(&position.hash()) {
                    continue; // Already solved
                }
                
                // Check if all moves lead to known outcomes
                if let Some(result) = self.analyze_position(position, distance) {
                    self.results.insert(position.hash(), result);
                    changed = true;
                }
            }
        }
        
        Ok(())
    }
    
    fn mark_terminal_positions(&mut self, positions: &[BitboardBoard]) {
        for position in positions {
            if position.is_checkmate() {
                let entry = TablebaseEntry {
                    result: TablebaseResult::Loss(0),
                    best_move: None,
                    dtz: 0,
                };
                self.results.insert(position.hash(), entry);
            } else if position.is_stalemate() {
                let entry = TablebaseEntry {
                    result: TablebaseResult::Draw,
                    best_move: None,
                    dtz: 0,
                };
                self.results.insert(position.hash(), entry);
            }
        }
    }
    
    fn analyze_position(&self, position: &BitboardBoard, distance: u16) -> Option<TablebaseEntry> {
        let moves = position.generate_legal_moves();
        
        let mut best_result = TablebaseResult::Loss(u16::MAX);
        let mut best_move = None;
        
        for mv in moves {
            let mut next_pos = position.clone();
            next_pos.make_move(mv);
            
            if let Some(entry) = self.results.get(&next_pos.hash()) {
                // Invert result (opponent's perspective)
                let inverted_result = match entry.result {
                    TablebaseResult::Win(d) => TablebaseResult::Loss(d + 1),
                    TablebaseResult::Loss(d) => TablebaseResult::Win(d + 1),
                    TablebaseResult::Draw => TablebaseResult::Draw,
                    TablebaseResult::Unknown => continue,
                };
                
                // Update best result
                if inverted_result.is_better_than(best_result) {
                    best_result = inverted_result;
                    best_move = Some(mv);
                }
            } else {
                // Position not yet solved
                return None;
            }
        }
        
        Some(TablebaseEntry {
            result: best_result,
            best_move,
            dtz: 0, // Calculate separately
        })
    }
}
```

## Performance Targets

### Lookup Performance

| Metric | Target |
|--------|--------|
| Probe time (cached) | <10μs |
| Probe time (uncached) | <1ms |
| Cache hit rate | >95% |
| Compression ratio | >10:1 |
| Load time (lazy) | <50ms |

### Storage Requirements

| Tablebase | Positions | Uncompressed | Compressed (10:1) |
|-----------|-----------|--------------|-------------------|
| 3-piece   | ~10K      | ~100KB      | ~10KB            |
| 4-piece   | ~1M       | ~10MB       | ~1MB             |
| 5-piece   | ~100M     | ~1GB        | ~100MB           |

## Integration with Search

### Search Cutoffs

```rust
impl SearchEngine {
    fn negamax_with_tablebase(
        &mut self,
        board: &mut BitboardBoard,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        // Probe tablebase if position is in endgame
        if board.piece_count() <= 5 {
            if let Some(entry) = self.tablebase_manager.probe_full(board, &self.tablebase_dir) {
                match entry.result {
                    TablebaseResult::Win(dtm) => {
                        // Return mate score adjusted for current depth
                        return MATE_SCORE - (dtm as i32);
                    }
                    TablebaseResult::Loss(dtm) => {
                        return -MATE_SCORE + (dtm as i32);
                    }
                    TablebaseResult::Draw => {
                        return 0;
                    }
                    TablebaseResult::Unknown => {
                        // Continue normal search
                    }
                }
            }
        }
        
        // Normal alpha-beta search
        // ...
    }
}
```

## Testing Strategy

### Correctness Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_endgame() {
        // KRvK - King and Rook vs King (should be winning)
        let board = BitboardBoard::from_sfen("4k4/9/9/9/9/9/9/4R4/4K4 b - 1");
        
        let mut tb_manager = TablebaseManager::new("./tablebases", false);
        let result = tb_manager.probe(&board, "./tablebases");
        
        match result {
            TablebaseResult::Win(dtm) => {
                assert!(dtm <= 50, "Should mate in <50 moves");
            }
            _ => panic!("Expected winning position"),
        }
    }
    
    #[test]
    fn test_tablebase_symmetry() {
        let board1 = BitboardBoard::from_sfen("4k4/9/9/9/9/9/9/4R4/4K4 b - 1");
        let board2 = board1.flip_horizontal();
        
        let mut tb_manager = TablebaseManager::new("./tablebases", false);
        
        let result1 = tb_manager.probe(&board1, "./tablebases");
        let result2 = tb_manager.probe(&board2, "./tablebases");
        
        assert_eq!(result1, result2, "Symmetric positions should have same result");
    }
}
```

## Dependencies

```toml
[dependencies]
lru = "0.12"              # LRU cache
flate2 = "1.0"           # Gzip compression
bincode = "1.3"          # Binary serialization
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
criterion = "0.5"        # Benchmarking
```

## Success Criteria

- [ ] Generate complete 3-piece and 4-piece tablebases
- [ ] Achieve <1ms uncached lookup time
- [ ] Achieve >10:1 compression ratio
- [ ] Cache hit rate >95% in typical games
- [ ] 100% correctness in endgame tests
- [ ] Seamless integration with search
- [ ] Support for lazy loading

## Future Enhancements

1. **Syzygy-Style DTZ Tablebases**
   - More efficient than DTM
   - Better 50-move rule handling

2. **Online Tablebase Access**
   - Query remote tablebase servers
   - Fallback for missing local tablebases

3. **7-piece Tablebases**
   - Cloud-based storage
   - On-demand streaming

4. **GPU Acceleration**
   - Parallel tablebase generation
   - GPU-based probing

## References

- [Syzygy Tablebases](https://www.chessprogramming.org/Syzygy_Bases)
- [Retrograde Analysis](https://www.chessprogramming.org/Retrograde_Analysis)
- [Endgame Tablebases](https://www.chessprogramming.org/Endgame_Tablebases)
- [Nalimov Tablebases](https://www.chessprogramming.org/Nalimov_Tablebases)

