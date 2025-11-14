use crate::types::{Move, PieceType, Player, Position};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced book move with comprehensive metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookMove {
    /// Source position (None for drops)
    pub from: Option<Position>,
    /// Destination position
    pub to: Position,
    /// Type of piece being moved
    pub piece_type: PieceType,
    /// Whether this is a drop move
    pub is_drop: bool,
    /// Whether this move promotes the piece
    pub is_promotion: bool,
    /// Move weight/frequency (0-1000, higher = more common)
    pub weight: u32,
    /// Position evaluation in centipawns after this move
    pub evaluation: i32,
    /// Opening name this move belongs to (optional)
    pub opening_name: Option<String>,
    /// Move notation in USI format (optional, for debugging)
    pub move_notation: Option<String>,
}

/// Position entry containing FEN and associated moves
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionEntry {
    /// FEN string representing the position
    pub fen: String,
    /// List of available moves from this position
    pub moves: Vec<BookMove>,
}

/// Lazy position entry for rarely accessed positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyPositionEntry {
    /// FEN string representing the position
    pub fen: String,
    /// Binary data containing the moves (loaded on demand)
    pub moves_data: Box<[u8]>,
    /// Number of moves in this position
    pub move_count: u32,
    /// Whether this entry has been loaded into memory
    pub loaded: bool,
}

/// Error types for opening book operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpeningBookError {
    /// Invalid FEN string
    InvalidFen(String),
    /// Invalid move data
    InvalidMove(String),
    /// Binary format parsing error
    BinaryFormatError(String),
    /// JSON parsing error
    JsonParseError(String),
    /// File I/O error
    IoError(String),
    /// Hash collision in lookup table
    HashCollision(String),
}

/// Static error messages to reduce allocations
mod error_messages {
    #[allow(dead_code)]
    pub const OPENING_BOOK_NOT_LOADED: &str = "Opening book not loaded";
    #[allow(dead_code)]
    pub const EMPTY_FEN_STRING: &str = "Empty FEN string";
    #[allow(dead_code)]
    pub const INSUFFICIENT_HEADER_DATA: &str = "Insufficient data for header";
    #[allow(dead_code)]
    pub const INVALID_MAGIC_NUMBER: &str = "Invalid magic number";
    #[allow(dead_code)]
    pub const UNEXPECTED_END_OF_DATA: &str = "Unexpected end of data";
    #[allow(dead_code)]
    pub const MISSING_DESTINATION_POSITION: &str = "Missing destination position";
    #[allow(dead_code)]
    pub const MISSING_PIECE_TYPE: &str = "Missing piece type";
}

/// Header for streaming chunks
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// Number of positions in this chunk
    pub position_count: usize,
    /// Offset of this chunk in the original data
    pub chunk_offset: u64,
    /// Size of this chunk in bytes
    pub chunk_size: usize,
}

/// Hash collision statistics for monitoring hash function quality
#[derive(Debug, Clone, Default)]
pub struct HashCollisionStats {
    /// Total number of hash collisions detected
    pub total_collisions: u64,
    /// Collision rate (collisions / total positions)
    pub collision_rate: f64,
    /// Maximum chain length observed in HashMap
    pub max_chain_length: usize,
    /// Total number of positions added
    pub total_positions: u64,
}

impl HashCollisionStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate and update collision rate
    pub fn update_collision_rate(&mut self) {
        if self.total_positions > 0 {
            self.collision_rate = self.total_collisions as f64 / self.total_positions as f64;
        } else {
            self.collision_rate = 0.0;
        }
    }

    /// Record a collision
    pub fn record_collision(&mut self, chain_length: usize) {
        self.total_collisions += 1;
        if chain_length > self.max_chain_length {
            self.max_chain_length = chain_length;
        }
        self.update_collision_rate();
    }

    /// Record a position addition
    pub fn record_position(&mut self) {
        self.total_positions += 1;
        self.update_collision_rate();
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsageStats {
    /// Number of loaded positions
    pub loaded_positions: usize,
    /// Memory used by loaded positions (bytes)
    pub loaded_positions_size: usize,
    /// Number of lazy positions
    pub lazy_positions: usize,
    /// Memory used by lazy positions (bytes)
    pub lazy_positions_size: usize,
    /// Number of cached positions
    pub cached_positions: usize,
    /// Memory used by cache (bytes)
    pub cache_size: usize,
    /// Memory used by temp buffer (bytes)
    pub temp_buffer_size: usize,
    /// Total memory usage (bytes)
    pub total_size: usize,
    /// Memory efficiency percentage (loaded vs total)
    pub memory_efficiency: f64,
}

/// Memory optimization result
#[derive(Debug, Clone)]
pub struct MemoryOptimizationResult {
    /// Number of optimizations applied
    pub optimizations_applied: usize,
    /// List of optimizations applied
    pub optimizations: Vec<String>,
    /// Estimated memory saved (bytes)
    pub memory_saved: usize,
}

/// High-performance opening book with HashMap-based lookup
#[derive(Debug, Clone, Serialize)]
pub struct OpeningBook {
    /// HashMap for O(1) position lookup (FEN hash -> PositionEntry)
    positions: HashMap<u64, PositionEntry>,
    /// Lazy-loaded positions (only loaded when accessed)
    lazy_positions: HashMap<u64, LazyPositionEntry>,
    /// Cache for frequently accessed positions (LRU cache)
    #[serde(skip)]
    position_cache: LruCache<u64, PositionEntry>,
    /// Reusable buffer for temporary operations (reduces allocations)
    #[serde(skip)]
    temp_buffer: Vec<u8>,
    /// Total number of moves in the book
    total_moves: usize,
    /// Whether the book has been loaded
    loaded: bool,
    /// Opening book metadata
    metadata: OpeningBookMetadata,
    /// Hash collision statistics for monitoring hash function quality
    #[serde(skip)]
    hash_collision_stats: HashCollisionStats,
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> Deserialize<'de> for OpeningBook {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OpeningBookData {
            positions: HashMap<u64, PositionEntry>,
            lazy_positions: HashMap<u64, LazyPositionEntry>,
            total_moves: usize,
            loaded: bool,
            metadata: OpeningBookMetadata,
        }

        let data = OpeningBookData::deserialize(deserializer)?;

        Ok(OpeningBook {
            positions: data.positions,
            lazy_positions: data.lazy_positions,
            position_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            temp_buffer: Vec::with_capacity(1024), // Pre-allocate 1KB buffer
            total_moves: data.total_moves,
            loaded: data.loaded,
            metadata: data.metadata,
            hash_collision_stats: HashCollisionStats::new(),
        })
    }
}

/// Metadata about the opening book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningBookMetadata {
    /// Version of the opening book format
    pub version: u32,
    /// Number of positions in the book
    pub position_count: usize,
    /// Number of total moves in the book
    pub move_count: usize,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last updated timestamp
    pub updated_at: Option<String>,
    /// Whether streaming mode is enabled
    pub streaming_enabled: bool,
    /// Chunk size for streaming (in bytes)
    pub chunk_size: usize,
}

/// Prefill entry used for transposition table initialization
#[derive(Debug, Clone)]
pub struct OpeningBookPrefillEntry {
    /// FEN string representing the position
    pub fen: String,
    /// Book move selected for this position
    pub book_move: BookMove,
    /// Player to move in this position
    pub player: Player,
}

impl BookMove {
    /// Create a new book move
    pub fn new(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        is_drop: bool,
        is_promotion: bool,
        weight: u32,
        evaluation: i32,
    ) -> Self {
        Self {
            from,
            to,
            piece_type,
            is_drop,
            is_promotion,
            weight,
            evaluation,
            opening_name: None,
            move_notation: None,
        }
    }

    /// Create a book move with opening name and notation
    pub fn new_with_metadata(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        is_drop: bool,
        is_promotion: bool,
        weight: u32,
        evaluation: i32,
        opening_name: Option<String>,
        move_notation: Option<String>,
    ) -> Self {
        Self {
            from,
            to,
            piece_type,
            is_drop,
            is_promotion,
            weight,
            evaluation,
            opening_name,
            move_notation,
        }
    }

    /// Convert to engine Move format
    pub fn to_engine_move(&self, player: Player) -> Move {
        Move {
            from: self.from,
            to: self.to,
            piece_type: self.piece_type,
            player,
            is_promotion: self.is_promotion,
            is_capture: false, // Will be determined by engine
            captured_piece: None,
            gives_check: false, // Will be determined by engine
            is_recapture: false,
        }
    }
}

impl PositionEntry {
    /// Create a new position entry
    pub fn new(fen: String, moves: Vec<BookMove>) -> Self {
        Self { fen, moves }
    }

    /// Add a move to this position
    pub fn add_move(&mut self, book_move: BookMove) {
        self.moves.push(book_move);
    }

    /// Get the best move by weight and evaluation
    pub fn get_best_move(&self) -> Option<&BookMove> {
        self.moves.iter().max_by(|a, b| {
            // Primary sort by weight, secondary by evaluation
            match a.weight.cmp(&b.weight) {
                std::cmp::Ordering::Equal => a.evaluation.cmp(&b.evaluation),
                other => other,
            }
        })
    }

    /// Get the best move by evaluation only
    pub fn get_best_move_by_evaluation(&self) -> Option<&BookMove> {
        self.moves.iter().max_by_key(|m| m.evaluation)
    }

    /// Get moves sorted by weight (best first)
    pub fn get_moves_by_weight(&self) -> Vec<&BookMove> {
        let mut moves: Vec<&BookMove> = self.moves.iter().collect();
        moves.sort_by(|a, b| b.weight.cmp(&a.weight));
        moves
    }

    /// Get moves sorted by evaluation (best first)
    pub fn get_moves_by_evaluation(&self) -> Vec<&BookMove> {
        let mut moves: Vec<&BookMove> = self.moves.iter().collect();
        moves.sort_by(|a, b| b.evaluation.cmp(&a.evaluation));
        moves
    }

    /// Get moves sorted by opening principles quality score (best first)
    /// 
    /// This method sorts moves by a provided quality score function.
    /// The quality_scores vector should have the same length as self.moves,
    /// where quality_scores[i] is the quality score for self.moves[i].
    pub fn get_moves_by_quality(&self, quality_scores: &[i32]) -> Vec<&BookMove> {
        if quality_scores.len() != self.moves.len() {
            return Vec::new();
        }
        
        let mut moves_with_scores: Vec<(usize, i32)> = quality_scores
            .iter()
            .enumerate()
            .map(|(i, &score)| (i, score))
            .collect();
        
        // Sort by quality score (descending - highest first)
        moves_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Return moves in sorted order
        moves_with_scores
            .iter()
            .map(|(i, _)| &self.moves[*i])
            .collect()
    }

    /// Get the best move by opening principles quality score
    /// 
    /// This method returns the move with the highest quality score.
    /// The quality_scores vector should have the same length as self.moves.
    pub fn get_best_move_by_quality(&self, quality_scores: &[i32]) -> Option<&BookMove> {
        if quality_scores.is_empty() || self.moves.is_empty() {
            return None;
        }
        
        let best_idx = quality_scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)?;
        
        self.moves.get(best_idx)
    }

    /// Get a random move weighted by move weights
    pub fn get_random_move(&self) -> Option<&BookMove> {
        use rand::Rng;

        let total_weight: u32 = self.moves.iter().map(|m| m.weight).sum();
        if total_weight == 0 || self.moves.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut random_value = rng.gen_range(0..total_weight);

        for book_move in &self.moves {
            if random_value < book_move.weight {
                return Some(book_move);
            }
            random_value -= book_move.weight;
        }

        self.moves.first()
    }
}

impl OpeningBook {
    /// Create a new empty opening book
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            lazy_positions: HashMap::new(),
            position_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            temp_buffer: Vec::with_capacity(1024), // Pre-allocate 1KB buffer
            total_moves: 0,
            loaded: false,
            metadata: OpeningBookMetadata {
                version: 1,
                position_count: 0,
                move_count: 0,
                created_at: None,
                updated_at: None,
                streaming_enabled: false,
                chunk_size: 0,
            },
            hash_collision_stats: HashCollisionStats::new(),
        }
    }

    /// Create opening book from binary data
    pub fn from_binary(data: &[u8]) -> Result<Self, OpeningBookError> {
        let mut reader = binary_format::BinaryReader::new(data.to_vec());
        reader.read_opening_book()
    }

    /// Create opening book from binary data using lightweight operations
    pub fn from_binary_boxed(data: Box<[u8]>) -> Result<Self, OpeningBookError> {
        let mut reader = binary_format::BinaryReader::new(data.into_vec());
        reader.read_opening_book()
    }

    /// Load opening book from binary data
    pub fn load_from_binary(&mut self, data: &[u8]) -> Result<(), OpeningBookError> {
        let book = Self::from_binary(data)?;
        self.positions = book.positions;
        self.total_moves = book.total_moves;
        self.loaded = book.loaded;
        self.metadata = book.metadata;
        Ok(())
    }

    /// Create opening book from JSON data (for migration)
    pub fn from_json(json_data: &str) -> Result<Self, OpeningBookError> {
        use crate::opening_book_converter::OpeningBookConverter;
        let converter = OpeningBookConverter::new();
        let (book, _stats) = converter.convert_from_json(json_data)?;
        Ok(book)
    }

    /// Load opening book from JSON data (for backward compatibility)
    pub fn load_from_json(&mut self, json_data: &str) -> Result<(), OpeningBookError> {
        let book = Self::from_json(json_data)?;
        self.positions = book.positions;
        self.total_moves = book.total_moves;
        self.loaded = book.loaded;
        self.metadata = book.metadata;
        Ok(())
    }

    /// Legacy method for backward compatibility
    pub fn get_move(&mut self, fen: &str) -> Option<Move> {
        self.get_best_move(fen)
    }

    /// Get all moves for a position
    pub fn get_moves(&mut self, fen: &str) -> Option<Vec<BookMove>> {
        let hash = self.hash_fen(fen);

        // First check cache
        if let Some(entry) = self.position_cache.get(&hash) {
            return Some(entry.moves.clone());
        }

        // Check regular positions
        if let Some(entry) = self.positions.get(&hash) {
            // Add to cache for future access
            self.position_cache.put(hash, entry.clone());
            return Some(entry.moves.clone());
        }

        // Check lazy positions and load if found
        if self.lazy_positions.contains_key(&hash) {
            if let Ok(()) = self.load_lazy_position(hash) {
                if let Some(entry) = self.positions.get(&hash) {
                    // Add to cache for future access
                    self.position_cache.put(hash, entry.clone());
                    return Some(entry.moves.clone());
                }
            }
        }

        None
    }

    /// Get the best move for a position with weight-based selection
    pub fn get_best_move(&mut self, fen: &str) -> Option<Move> {
        let hash = self.hash_fen(fen);
        let player = Self::determine_player_from_fen(fen);

        // First check cache
        if let Some(entry) = self.position_cache.get(&hash) {
            if let Some(book_move) = entry.get_best_move() {
                return Some(book_move.to_engine_move(player));
            }
        }

        // Check regular positions
        if let Some(entry) = self.positions.get(&hash) {
            // Add to cache for future access
            self.position_cache.put(hash, entry.clone());
            if let Some(book_move) = entry.get_best_move() {
                return Some(book_move.to_engine_move(player));
            }
        }

        // Check lazy positions and load if found
        if self.lazy_positions.contains_key(&hash) {
            if let Ok(()) = self.load_lazy_position(hash) {
                if let Some(entry) = self.positions.get(&hash) {
                    // Add to cache for future access
                    self.position_cache.put(hash, entry.clone());
                    if let Some(book_move) = entry.get_best_move() {
                        return Some(book_move.to_engine_move(player));
                    }
                }
            }
        }

        None
    }

    /// Get the best move for a position prioritized by opening principles (Task 19.0 - Task 3.0)
    ///
    /// This method evaluates all book moves using opening principles and returns the best one.
    /// If opening principles evaluation is not available, falls back to weight-based selection.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string of the position
    /// * `board` - Current board state (for opening principles evaluation)
    /// * `captured_pieces` - Current captured pieces state
    /// * `move_count` - Number of moves played so far
    /// * `opening_evaluator` - Optional opening principles evaluator (if None, uses weight-based selection)
    ///
    /// # Returns
    ///
    /// Best move according to opening principles, or None if no moves available
    pub fn get_best_move_with_principles(
        &mut self,
        fen: &str,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &crate::types::CapturedPieces,
        move_count: u32,
        opening_evaluator: Option<&mut crate::evaluation::opening_principles::OpeningPrincipleEvaluator>,
    ) -> Option<Move> {
        let hash = self.hash_fen(fen);
        let player = Self::determine_player_from_fen(fen);

        // Get position entry
        let entry = if let Some(entry) = self.position_cache.get(&hash) {
            entry.clone()
        } else if let Some(entry) = self.positions.get(&hash) {
            let entry_clone = entry.clone();
            self.position_cache.put(hash, entry_clone.clone());
            entry_clone
        } else if self.lazy_positions.contains_key(&hash) {
            if let Ok(()) = self.load_lazy_position(hash) {
                if let Some(entry) = self.positions.get(&hash) {
                    let entry_clone = entry.clone();
                    self.position_cache.put(hash, entry_clone.clone());
                    entry_clone
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        };

        // If no evaluator provided, fall back to weight-based selection
        let evaluator = match opening_evaluator {
            Some(eval) => eval,
            None => {
                if let Some(book_move) = entry.get_best_move() {
                    return Some(book_move.to_engine_move(player));
                }
                return None;
            }
        };

        // Evaluate all moves using opening principles
        let mut moves_with_scores: Vec<(BookMove, i32)> = Vec::new();
        
        for book_move in &entry.moves {
            let engine_move = book_move.to_engine_move(player);
            
            // Validate move (log warnings if violations found)
            let is_valid = evaluator.validate_book_move(board, player, &engine_move, move_count);
            
            if is_valid {
                // Evaluate move quality
                let quality_score = evaluator.evaluate_book_move_quality(
                    board,
                    player,
                    &engine_move,
                    captured_pieces,
                    move_count,
                );
                
                moves_with_scores.push((book_move.clone(), quality_score));
                
                // Debug logging
                #[cfg(debug_assertions)]
                crate::debug_utils::debug_log(&format!(
                    "[OPENING_BOOK] Book move {} has opening principles quality score: {}",
                    book_move.move_notation.as_ref().unwrap_or(&engine_move.to_usi_string()),
                    quality_score
                ));
            }
        }

        // Sort by quality score (highest first)
        moves_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Track prioritization
        if !moves_with_scores.is_empty() {
            evaluator.stats_mut().book_moves_prioritized += 1;
        }

        // Return best move
        moves_with_scores
            .first()
            .map(|(book_move, _)| book_move.to_engine_move(player))
    }

    /// Get a random move for a position with weighted random selection
    pub fn get_random_move(&mut self, fen: &str) -> Option<Move> {
        let hash = self.hash_fen(fen);
        let player = Self::determine_player_from_fen(fen);

        // First check cache
        if let Some(entry) = self.position_cache.get(&hash) {
            if let Some(book_move) = entry.get_random_move() {
                return Some(book_move.to_engine_move(player));
            }
        }

        // Check regular positions
        if let Some(entry) = self.positions.get(&hash) {
            // Add to cache for future access
            self.position_cache.put(hash, entry.clone());
            if let Some(book_move) = entry.get_random_move() {
                return Some(book_move.to_engine_move(player));
            }
        }

        // Check lazy positions and load if found
        if self.lazy_positions.contains_key(&hash) {
            if let Ok(()) = self.load_lazy_position(hash) {
                if let Some(entry) = self.positions.get(&hash) {
                    // Add to cache for future access
                    self.position_cache.put(hash, entry.clone());
                    if let Some(book_move) = entry.get_random_move() {
                        return Some(book_move.to_engine_move(player));
                    }
                }
            }
        }

        None
    }

    /// Get all moves for a position with enhanced metadata
    pub fn get_moves_with_metadata(&self, fen: &str) -> Option<Vec<(BookMove, Move)>> {
        if let Some(entry) = self.positions.get(&self.hash_fen(fen)) {
            let player = Self::determine_player_from_fen(fen);
            let moves: Vec<(BookMove, Move)> = entry
                .moves
                .iter()
                .map(|book_move| (book_move.clone(), book_move.to_engine_move(player)))
                .collect();
            return Some(moves);
        }
        None
    }

    /// Load a lazy position into memory
    fn load_lazy_position(&mut self, hash: u64) -> Result<(), OpeningBookError> {
        if let Some(lazy_entry) = self.lazy_positions.remove(&hash) {
            // Parse the binary data to get the moves
            let moves = self.parse_moves_from_binary(&lazy_entry.moves_data)?;

            // Create a regular position entry
            let position_entry = PositionEntry {
                fen: lazy_entry.fen,
                moves,
            };

            // Move to regular positions
            self.positions.insert(hash, position_entry);
        }
        Ok(())
    }

    /// Parse moves from binary data
    fn parse_moves_from_binary(&self, data: &[u8]) -> Result<Vec<BookMove>, OpeningBookError> {
        let mut reader = binary_format::BinaryReader::new(data.to_vec());
        let mut moves = Vec::new();

        // Read move count
        let move_count = reader.read_u32()? as usize;

        // Read each move
        for _ in 0..move_count {
            moves.push(reader.read_book_move()?);
        }

        Ok(moves)
    }

    /// Add a position entry to the book
    pub fn add_position(&mut self, fen: String, moves: Vec<BookMove>) {
        let hash = self.hash_fen(&fen);
        let entry = PositionEntry::new(fen.clone(), moves);
        self.total_moves += entry.moves.len();
        
        // Detect hash collisions: if insert returns Some, check if it's a true collision
        if let Some(old_entry) = self.positions.insert(hash, entry) {
            // If the FENs are different, this is a hash collision (same hash, different FEN)
            if old_entry.fen != fen {
                // True hash collision detected
                let chain_length = self.count_positions_with_hash(hash);
                self.hash_collision_stats.record_collision(chain_length);
                
                // Debug logging
                #[cfg(feature = "verbose-debug")]
                {
                    log::debug!(
                        "Hash collision detected: hash={}, old_fen={}, new_fen={}, chain_length={}",
                        hash,
                        old_entry.fen,
                        fen,
                        chain_length
                    );
                }
            }
            // If FENs are the same, we're just overwriting the same position (not a collision)
        }
        
        self.hash_collision_stats.record_position();
        self.metadata.position_count = self.positions.len();
        self.metadata.move_count = self.total_moves;
    }
    
    /// Count positions that would hash to the same value
    /// This is an approximation since we can't access HashMap internals
    fn count_positions_with_hash(&self, hash: u64) -> usize {
        // Since we can't access HashMap internals, we estimate based on
        // how many positions we've seen with this hash
        // In practice, HashMap uses open addressing, so chain length is typically 1-2
        // We'll use a conservative estimate: if we see a collision, assume chain length of 2
        // This will be updated as we see more collisions
        if self.hash_collision_stats.max_chain_length > 0 {
            self.hash_collision_stats.max_chain_length + 1
        } else {
            2 // First collision means at least 2 entries share the hash
        }
    }

    /// Add a position entry to lazy storage (for rarely accessed positions)
    pub fn add_lazy_position(
        &mut self,
        fen: String,
        moves: Vec<BookMove>,
    ) -> Result<(), OpeningBookError> {
        let hash = self.hash_fen(&fen);

        // Serialize moves to binary data
        let moves_data = self.serialize_moves_to_binary(&moves)?;

        let lazy_entry = LazyPositionEntry {
            fen,
            moves_data,
            move_count: moves.len() as u32,
            loaded: false,
        };

        self.total_moves += moves.len();
        self.lazy_positions.insert(hash, lazy_entry);
        self.metadata.position_count = self.positions.len() + self.lazy_positions.len();
        self.metadata.move_count = self.total_moves;
        Ok(())
    }

    /// Serialize moves to binary data
    fn serialize_moves_to_binary(&self, moves: &[BookMove]) -> Result<Box<[u8]>, OpeningBookError> {
        let writer = binary_format::BinaryWriter::new();
        let mut bytes = Vec::new();

        // Write move count
        bytes.extend_from_slice(&(moves.len() as u32).to_le_bytes());

        // Write each move
        for book_move in moves {
            bytes.extend_from_slice(&writer.write_book_move(book_move)?);
        }

        Ok(bytes.into_boxed_slice())
    }

    /// Check if the book is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Mark the book as loaded
    pub fn mark_loaded(mut self) -> Self {
        self.loaded = true;
        self
    }

    /// Get book statistics
    pub fn get_stats(&self) -> &OpeningBookMetadata {
        &self.metadata
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.position_cache.len(), self.position_cache.cap().get())
    }

    /// Clear the position cache
    pub fn clear_cache(&mut self) {
        self.position_cache.clear();
    }

    /// Get a reusable temporary buffer (clears and returns for use)
    pub fn get_temp_buffer(&mut self) -> &mut Vec<u8> {
        self.temp_buffer.clear();
        &mut self.temp_buffer
    }

    /// Enable streaming mode for large opening books
    pub fn enable_streaming_mode(&mut self, chunk_size: usize) {
        // Clear existing positions to free memory
        self.positions.clear();
        self.position_cache.clear();

        // Set up streaming parameters
        self.metadata.streaming_enabled = true;
        self.metadata.chunk_size = chunk_size;
    }

    /// Load a chunk of positions from binary data (for streaming)
    pub fn load_chunk(
        &mut self,
        chunk_data: &[u8],
        _chunk_offset: u64,
    ) -> Result<usize, OpeningBookError> {
        let mut reader = binary_format::BinaryReader::new(chunk_data.to_vec());
        let mut loaded_count = 0;

        // Read chunk header
        let chunk_header = reader.read_chunk_header()?;

        // Load positions from this chunk
        for _ in 0..chunk_header.position_count {
            if let Ok((fen, moves)) = reader.read_position_entry() {
                let hash = self.hash_fen(&fen);

                // Store in lazy positions to save memory
                if let Ok(moves_data) = self.serialize_moves_to_binary(&moves) {
                    let lazy_entry = LazyPositionEntry {
                        fen: fen.clone(),
                        moves_data,
                        move_count: moves.len() as u32,
                        loaded: false,
                    };
                    self.lazy_positions.insert(hash, lazy_entry);
                    loaded_count += 1;
                }
            }
        }

        Ok(loaded_count)
    }

    /// Get streaming statistics
    pub fn get_streaming_stats(&self) -> (usize, usize, usize) {
        (
            self.positions.len(),      // Loaded positions
            self.lazy_positions.len(), // Lazy positions
            self.position_cache.len(), // Cached positions
        )
    }

    /// Get detailed memory usage statistics
    pub fn get_memory_usage(&self) -> MemoryUsageStats {
        let loaded_positions_size = self.positions.len() * std::mem::size_of::<PositionEntry>();
        let lazy_positions_size = self
            .lazy_positions
            .values()
            .map(|entry| entry.moves_data.len() + std::mem::size_of::<LazyPositionEntry>())
            .sum::<usize>();
        let cache_size = self.position_cache.len() * std::mem::size_of::<PositionEntry>();
        let temp_buffer_size = self.temp_buffer.capacity();

        let total_size =
            loaded_positions_size + lazy_positions_size + cache_size + temp_buffer_size;

        MemoryUsageStats {
            loaded_positions: self.positions.len(),
            loaded_positions_size,
            lazy_positions: self.lazy_positions.len(),
            lazy_positions_size,
            cached_positions: self.position_cache.len(),
            cache_size,
            temp_buffer_size,
            total_size,
            memory_efficiency: if total_size > 0 {
                (loaded_positions_size as f64 / total_size as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Get hash quality metrics
    ///
    /// Returns statistics about hash collisions, which can help assess
    /// the quality of the hash function being used.
    pub fn get_hash_quality_metrics(&self) -> HashCollisionStats {
        self.hash_collision_stats.clone()
    }

    /// Get unified statistics for the opening book
    ///
    /// This method aggregates statistics from various sources:
    /// - Memory usage statistics
    /// - Hash collision statistics
    /// - Opening principles integration (if available)
    /// - Move ordering integration (if available)
    ///
    /// Note: Opening principles and move ordering statistics must be updated
    /// separately by calling `update_statistics_from_opening_principles()` and
    /// `update_statistics_from_move_ordering()` respectively.
    pub fn get_statistics(&self) -> statistics::BookStatistics {
        let mut stats = statistics::BookStatistics::new();

        // Add memory usage statistics
        stats.set_memory_stats(self.get_memory_usage());
        
        // Add hash collision statistics
        stats.set_hash_collision_stats(self.get_hash_quality_metrics());

        stats
    }

    /// Update statistics from opening principles evaluator
    ///
    /// This should be called periodically to sync statistics from the opening
    /// principles evaluator.
    pub fn update_statistics_from_opening_principles(
        &self,
        stats: &mut statistics::BookStatistics,
        opening_principles_stats: &crate::evaluation::opening_principles::OpeningPrincipleStats,
    ) {
        stats.update_from_opening_principles(opening_principles_stats);
    }

    /// Update statistics from move ordering integration
    ///
    /// This should be called periodically to sync statistics from the move
    /// ordering module.
    pub fn update_statistics_from_move_ordering(
        &self,
        stats: &mut statistics::BookStatistics,
        move_ordering_stats: &crate::search::move_ordering::AdvancedIntegrationStats,
    ) {
        stats.update_from_move_ordering(move_ordering_stats);
    }

    /// Optimize memory usage based on current patterns
    pub fn optimize_memory_usage(&mut self) -> MemoryOptimizationResult {
        let mut optimizations = Vec::new();

        // Check if we should enable streaming mode
        let memory_usage = self.get_memory_usage();
        if memory_usage.total_size > 50 * 1024 * 1024 {
            // 50MB threshold
            self.enable_streaming_mode(1024 * 1024); // 1MB chunks
            optimizations.push("Enabled streaming mode for large opening book".to_string());
        }

        // Clear cache if it's too large
        if self.position_cache.len() > 1000 {
            self.position_cache.clear();
            optimizations.push("Cleared LRU cache to free memory".to_string());
        }

        // Suggest lazy loading for rarely accessed positions
        if self.positions.len() > 10000 && self.lazy_positions.len() < self.positions.len() / 2 {
            optimizations.push("Consider moving more positions to lazy loading".to_string());
        }

        MemoryOptimizationResult {
            optimizations_applied: optimizations.len(),
            optimizations,
            memory_saved: memory_usage.total_size,
        }
    }

    /// Benchmark hash functions and return performance statistics
    pub fn benchmark_hash_functions(&self, test_fens: &[&str]) -> Vec<(String, u64, u64)> {
        let mut results = Vec::new();

        for fen in test_fens {
            let start = std::time::Instant::now();
            let _hash1 = self.hash_fen_fnv1a(fen);
            let fnv1a_time = start.elapsed().as_nanos() as u64;

            let start = std::time::Instant::now();
            let _hash2 = self.hash_fen_simple(fen);
            let simple_time = start.elapsed().as_nanos() as u64;

            let start = std::time::Instant::now();
            let _hash3 = self.hash_fen_bitwise(fen);
            let bitwise_time = start.elapsed().as_nanos() as u64;

            results.push(("FNV-1a".to_string(), fnv1a_time, 0));
            results.push(("Simple".to_string(), simple_time, 0));
            results.push(("Bitwise".to_string(), bitwise_time, 0));
        }

        results
    }

    /// Convert opening book to binary format
    pub fn to_binary(&self) -> Result<Box<[u8]>, OpeningBookError> {
        let mut writer = binary_format::BinaryWriter::new();
        writer
            .write_opening_book(self)
            .map(|vec| vec.into_boxed_slice())
    }

    /// Validate the opening book integrity
    pub fn validate(&self) -> Result<(), OpeningBookError> {
        // Check if book is loaded
        if !self.loaded {
            return Err(OpeningBookError::BinaryFormatError(
                "Opening book not loaded".to_string(),
            ));
        }

        // Validate metadata consistency
        if self.metadata.position_count != self.positions.len() {
            return Err(OpeningBookError::BinaryFormatError(format!(
                "Position count mismatch: metadata={}, actual={}",
                self.metadata.position_count,
                self.positions.len()
            )));
        }

        if self.metadata.move_count != self.total_moves {
            return Err(OpeningBookError::BinaryFormatError(format!(
                "Move count mismatch: metadata={}, actual={}",
                self.metadata.move_count, self.total_moves
            )));
        }

        // Validate each position entry
        for (_hash, entry) in &self.positions {
            // Validate FEN is not empty
            if entry.fen.is_empty() {
                return Err(OpeningBookError::InvalidFen("Empty FEN string".to_string()));
            }

            // Validate moves
            for (i, book_move) in entry.moves.iter().enumerate() {
                // Validate positions are within bounds
                if let Some(from) = book_move.from {
                    if !from.is_valid() {
                        return Err(OpeningBookError::InvalidMove(format!(
                            "Invalid from position in move {}: {:?}",
                            i, from
                        )));
                    }
                }

                if !book_move.to.is_valid() {
                    return Err(OpeningBookError::InvalidMove(format!(
                        "Invalid to position in move {}: {:?}",
                        i, book_move.to
                    )));
                }

                // Validate weight is reasonable
                if book_move.weight > 10000 {
                    return Err(OpeningBookError::InvalidMove(format!(
                        "Weight too high in move {}: {}",
                        i, book_move.weight
                    )));
                }

                // Validate evaluation is reasonable
                if book_move.evaluation.abs() > 10000 {
                    return Err(OpeningBookError::InvalidMove(format!(
                        "Evaluation too extreme in move {}: {}",
                        i, book_move.evaluation
                    )));
                }
            }
        }

        Ok(())
    }

    /// Hash a FEN string for lookup using a lightweight hash
    fn hash_fen(&self, fen: &str) -> u64 {
        // Use FNV-1a hash for better performance in constrained environments
        // FNV-1a is faster than DefaultHasher and has good distribution
        self.hash_fen_fnv1a(fen)
    }

    /// FNV-1a hash function for lightweight hashing
    fn hash_fen_fnv1a(&self, fen: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        let prime: u64 = 0x100000001b3; // FNV prime

        for &byte in fen.as_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(prime);
        }

        hash
    }

    /// Alternative hash function using a simple but fast algorithm
    fn hash_fen_simple(&self, fen: &str) -> u64 {
        let mut hash: u64 = 5381;

        for &byte in fen.as_bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }

        hash
    }

    /// Hash function using bit manipulation for maximum performance
    fn hash_fen_bitwise(&self, fen: &str) -> u64 {
        let mut hash: u64 = 0;
        let mut i = 0;

        for &byte in fen.as_bytes() {
            hash ^= (byte as u64) << (i % 56);
            i += 1;
        }

        hash
    }

    /// Determine player from FEN string
    /// Determine player to move from FEN string
    pub fn determine_player_from_fen(fen: &str) -> Player {
        // FEN format: "board position active_player captured_pieces move_number"
        // The active player is the 4th field (index 3)
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() >= 4 {
            match parts[3] {
                "b" | "B" => Player::Black,
                "w" | "W" => Player::White,
                _ => Player::Black, // Default to Black if unclear
            }
        } else {
            Player::Black // Default to Black if FEN is malformed
        }
    }

    /// Collect all entries suitable for transposition table prefill
    pub fn collect_prefill_entries(&mut self) -> Vec<OpeningBookPrefillEntry> {
        // Materialize all lazy positions to ensure comprehensive coverage
        let lazy_hashes: Vec<u64> = self.lazy_positions.keys().cloned().collect();
        for hash in lazy_hashes {
            let _ = self.load_lazy_position(hash);
        }

        let mut results = Vec::new();
        for entry in self.positions.values() {
            if let Some(best_move) = entry.get_best_move() {
                results.push(OpeningBookPrefillEntry {
                    fen: entry.fen.clone(),
                    book_move: best_move.clone(),
                    player: Self::determine_player_from_fen(&entry.fen),
                });
            }
        }

        results
    }

    /// Convert book move to engine move with proper move properties
    pub fn convert_book_move_to_engine_move(
        &self,
        book_move: &BookMove,
        player: Player,
        board: &crate::bitboards::BitboardBoard,
    ) -> Move {
        let mut engine_move = book_move.to_engine_move(player);

        // Determine if this is a capture move
        if let Some(_from) = book_move.from {
            if let Some(piece) = board.get_piece(book_move.to) {
                engine_move.is_capture = true;
                engine_move.captured_piece = Some(piece.clone());
            }
        }

        // Determine if this move gives check (simplified heuristic)
        engine_move.gives_check = self.does_move_give_check(&engine_move, board, player);

        engine_move
    }

    /// Check if a move gives check (simplified heuristic)
    fn does_move_give_check(
        &self,
        _move: &Move,
        _board: &crate::bitboards::BitboardBoard,
        _player: Player,
    ) -> bool {
        // This is a simplified implementation
        // In a full implementation, this would check if the move attacks the opponent's king
        false
    }
}

/// Builder pattern for constructing opening book entries
pub struct OpeningBookBuilder {
    book: OpeningBook,
}

impl OpeningBookBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            book: OpeningBook::new(),
        }
    }

    /// Add a position with moves
    pub fn add_position(mut self, fen: String, moves: Vec<BookMove>) -> Self {
        self.book.add_position(fen, moves);
        self
    }

    /// Add a single move to a position
    pub fn add_move_to_position(mut self, fen: String, book_move: BookMove) -> Self {
        let hash = self.book.hash_fen(&fen);

        if let Some(entry) = self.book.positions.get_mut(&hash) {
            entry.add_move(book_move);
            self.book.total_moves += 1;
            self.book.metadata.move_count = self.book.total_moves;
        } else {
            // Create new position entry
            let entry = PositionEntry::new(fen.clone(), vec![book_move]);
            self.book.total_moves += 1;
            self.book.positions.insert(hash, entry);
            self.book.metadata.position_count = self.book.positions.len();
            self.book.metadata.move_count = self.book.total_moves;
        }
        self
    }

    /// Set metadata
    pub fn with_metadata(
        mut self,
        version: u32,
        created_at: Option<String>,
        updated_at: Option<String>,
    ) -> Self {
        self.book.metadata.version = version;
        self.book.metadata.created_at = created_at;
        self.book.metadata.updated_at = updated_at;
        self
    }

    /// Mark the book as loaded
    pub fn mark_loaded(mut self) -> Self {
        self.book.loaded = true;
        self
    }

    /// Build the opening book
    pub fn build(self) -> OpeningBook {
        self.book
    }
}

impl Default for OpeningBookBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for constructing book moves
pub struct BookMoveBuilder {
    from: Option<Position>,
    to: Option<Position>,
    piece_type: Option<PieceType>,
    is_drop: bool,
    is_promotion: bool,
    weight: u32,
    evaluation: i32,
    opening_name: Option<String>,
    move_notation: Option<String>,
}

impl BookMoveBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            piece_type: None,
            is_drop: false,
            is_promotion: false,
            weight: 100,   // Default weight
            evaluation: 0, // Default evaluation
            opening_name: None,
            move_notation: None,
        }
    }

    /// Set source position
    pub fn from(mut self, from: Position) -> Self {
        self.from = Some(from);
        self.is_drop = false;
        self
    }

    /// Set as drop move
    pub fn as_drop(mut self) -> Self {
        self.from = None;
        self.is_drop = true;
        self
    }

    /// Set destination position
    pub fn to(mut self, to: Position) -> Self {
        self.to = Some(to);
        self
    }

    /// Set piece type
    pub fn piece_type(mut self, piece_type: PieceType) -> Self {
        self.piece_type = Some(piece_type);
        self
    }

    /// Set as promotion move
    pub fn promote(mut self) -> Self {
        self.is_promotion = true;
        self
    }

    /// Set move weight
    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Set position evaluation
    pub fn evaluation(mut self, evaluation: i32) -> Self {
        self.evaluation = evaluation;
        self
    }

    /// Set opening name
    pub fn opening_name(mut self, opening_name: String) -> Self {
        self.opening_name = Some(opening_name);
        self
    }

    /// Set move notation
    pub fn move_notation(mut self, move_notation: String) -> Self {
        self.move_notation = Some(move_notation);
        self
    }

    /// Build the book move
    pub fn build(self) -> Result<BookMove, OpeningBookError> {
        let to = self.to.ok_or_else(|| {
            OpeningBookError::InvalidMove("Missing destination position".to_string())
        })?;
        let piece_type = self
            .piece_type
            .ok_or_else(|| OpeningBookError::InvalidMove("Missing piece type".to_string()))?;

        Ok(BookMove {
            from: self.from,
            to,
            piece_type,
            is_drop: self.is_drop,
            is_promotion: self.is_promotion,
            weight: self.weight,
            evaluation: self.evaluation,
            opening_name: self.opening_name,
            move_notation: self.move_notation,
        })
    }
}

impl Default for BookMoveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary format implementation for opening books
#[path = "opening_book/binary_format.rs"]
pub mod binary_format;

/// Unified statistics API for opening book
#[path = "opening_book/statistics.rs"]
pub mod statistics;

pub use statistics::BookStatistics;

/// Helper functions for coordinate conversion
pub mod coordinate_utils {
    use super::*;

    /// Convert USI coordinate string to Position
    /// Format: "1a", "5e", "9i" etc. (file + rank)
    pub fn string_to_position(coord: &str) -> Result<Position, OpeningBookError> {
        Position::from_usi_string(coord).map_err(|e| {
            OpeningBookError::InvalidMove(format!("Invalid USI coordinate '{}': {}", coord, e))
        })
    }

    /// Convert Position to USI coordinate string
    pub fn position_to_string(pos: Position) -> String {
        let file = 9 - pos.col;
        let rank = (b'a' + pos.row) as char;
        format!("{}{}", file, rank)
    }

    /// Parse piece type from string
    pub fn parse_piece_type(piece_str: &str) -> Result<PieceType, OpeningBookError> {
        PieceType::from_str(piece_str).ok_or_else(|| {
            OpeningBookError::InvalidMove(format!("Invalid piece type: {}", piece_str))
        })
    }
}
