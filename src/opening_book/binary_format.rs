/// Binary format implementation for opening books
///
/// This module provides reading and writing of the Shogi Binary Opening Book (SBOB) format.
/// The format is optimized for fast lookups and minimal memory usage.
// Note: This module is a child module of opening_book
// Types are imported from the parent module using super::
use super::{
    BookMove, ChunkHeader, OpeningBook, OpeningBookError, OpeningBookMetadata, Position,
    PositionEntry,
};
use crate::types::core::PieceType;
use lru::LruCache;
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// Magic number for Shogi Binary Opening Book format
const MAGIC_NUMBER: [u8; 4] = *b"SBOB";

/// Current format version
const FORMAT_VERSION: u32 = 1;

/// Binary format header
#[derive(Debug, Clone)]
pub struct BinaryHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub entry_count: u64,
    pub hash_table_size: u64,
    pub total_moves: u64,
    pub created_at: u64, // Unix timestamp
    pub updated_at: u64, // Unix timestamp
}

/// Hash table entry for position lookup
#[derive(Debug, Clone)]
pub struct HashTableEntry {
    pub position_hash: u64,
    pub entry_offset: u64,
}

/// Binary format writer
pub struct BinaryWriter {
    buffer: Vec<u8>,
}

/// Binary format reader
pub struct BinaryReader {
    data: Box<[u8]>,
    position: usize,
}

impl BinaryHeader {
    /// Create a new header
    pub fn new(entry_count: u64, hash_table_size: u64, total_moves: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            magic: MAGIC_NUMBER,
            version: FORMAT_VERSION,
            entry_count,
            hash_table_size,
            total_moves,
            created_at: now,
            updated_at: now,
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48); // 4 + 4 + 8 + 8 + 8 + 8 + 8
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.entry_count.to_le_bytes());
        bytes.extend_from_slice(&self.hash_table_size.to_le_bytes());
        bytes.extend_from_slice(&self.total_moves.to_le_bytes());
        bytes.extend_from_slice(&self.created_at.to_le_bytes());
        bytes.extend_from_slice(&self.updated_at.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, OpeningBookError> {
        if data.len() < 48 {
            return Err(OpeningBookError::BinaryFormatError(
                "Insufficient data for header".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read magic: {}", e))
        })?;

        if magic != MAGIC_NUMBER {
            return Err(OpeningBookError::BinaryFormatError(
                "Invalid magic number".to_string(),
            ));
        }

        let mut version_bytes = [0u8; 4];
        cursor.read_exact(&mut version_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read version: {}", e))
        })?;
        let version = u32::from_le_bytes(version_bytes);

        if version != FORMAT_VERSION {
            return Err(OpeningBookError::BinaryFormatError(format!(
                "Unsupported version: {}",
                version
            )));
        }

        let mut entry_count_bytes = [0u8; 8];
        cursor.read_exact(&mut entry_count_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read entry count: {}", e))
        })?;
        let entry_count = u64::from_le_bytes(entry_count_bytes);

        let mut hash_table_size_bytes = [0u8; 8];
        cursor.read_exact(&mut hash_table_size_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read hash table size: {}", e))
        })?;
        let hash_table_size = u64::from_le_bytes(hash_table_size_bytes);

        let mut total_moves_bytes = [0u8; 8];
        cursor.read_exact(&mut total_moves_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read total moves: {}", e))
        })?;
        let total_moves = u64::from_le_bytes(total_moves_bytes);

        let mut created_at_bytes = [0u8; 8];
        cursor.read_exact(&mut created_at_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read created at: {}", e))
        })?;
        let created_at = u64::from_le_bytes(created_at_bytes);

        let mut updated_at_bytes = [0u8; 8];
        cursor.read_exact(&mut updated_at_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Failed to read updated at: {}", e))
        })?;
        let updated_at = u64::from_le_bytes(updated_at_bytes);

        Ok(Self {
            magic,
            version,
            entry_count,
            hash_table_size,
            total_moves,
            created_at,
            updated_at,
        })
    }
}

impl BinaryWriter {
    /// Create a new writer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Write opening book to binary format
    pub fn write_opening_book(&mut self, book: &OpeningBook) -> Result<Vec<u8>, OpeningBookError> {
        self.buffer.clear();

        // Calculate hash table size (next power of 2 >= entry_count)
        let entry_count = book.positions.len() as u64;
        let hash_table_size = if entry_count == 0 {
            0
        } else {
            entry_count.next_power_of_two()
        };

        // Create header
        let header = BinaryHeader::new(entry_count, hash_table_size, book.total_moves as u64);
        self.buffer.extend_from_slice(&header.to_bytes());

        // Create hash table
        let mut hash_table = Vec::with_capacity(hash_table_size as usize);
        let mut position_entries: Vec<Box<[u8]>> = Vec::new();
        let mut current_offset = 48 + (hash_table_size * 16) as usize; // Header + hash table

        // Handle empty book case
        if entry_count == 0 {
            return Ok(self.buffer.clone());
        }

        // Sort positions by hash for consistent ordering
        let mut sorted_positions: Vec<_> = book.positions.iter().collect();
        sorted_positions.sort_by_key(|(hash, _)| **hash);

        for (hash, entry) in sorted_positions {
            // Write position entry
            let entry_bytes = self.write_position_entry(entry)?;
            let entry_len = entry_bytes.len();
            position_entries.push(entry_bytes);

            // Add to hash table
            hash_table.push(HashTableEntry {
                position_hash: *hash,
                entry_offset: current_offset as u64,
            });

            current_offset += entry_len;
        }

        // Write hash table
        for entry in &hash_table {
            self.buffer
                .extend_from_slice(&entry.position_hash.to_le_bytes());
            self.buffer
                .extend_from_slice(&entry.entry_offset.to_le_bytes());
        }

        // Pad hash table to size (only if we have entries to pad)
        if !hash_table.is_empty() && hash_table.len() < hash_table_size as usize {
            while hash_table.len() < hash_table_size as usize {
                self.buffer.extend_from_slice(&[0u8; 16]);
            }
        }

        // Write position entries
        for entry_bytes in position_entries {
            self.buffer.extend_from_slice(&entry_bytes);
        }

        Ok(self.buffer.clone())
    }

    /// Write a position entry to bytes
    fn write_position_entry(&self, entry: &PositionEntry) -> Result<Box<[u8]>, OpeningBookError> {
        let mut bytes = Vec::new();

        // Write FEN string
        let fen_bytes = entry.fen.as_bytes();
        bytes.extend_from_slice(&(fen_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(fen_bytes);

        // Write move count
        bytes.extend_from_slice(&(entry.moves.len() as u32).to_le_bytes());

        // Write moves
        for book_move in &entry.moves {
            bytes.extend_from_slice(&self.write_book_move(book_move)?);
        }

        Ok(bytes.into_boxed_slice())
    }

    /// Write a book move to bytes
    pub fn write_book_move(&self, book_move: &BookMove) -> Result<Box<[u8]>, OpeningBookError> {
        let mut bytes = Vec::with_capacity(24); // Fixed size for book move

        // From position (2 bytes: row << 8 | col, or 0xFFFF for None)
        let from_bytes = if let Some(from) = book_move.from {
            ((from.row as u16) << 8 | from.col as u16).to_le_bytes()
        } else {
            [0xFF, 0xFF]
        };
        bytes.extend_from_slice(&from_bytes);

        // To position (2 bytes: row << 8 | col)
        let to_bytes = ((book_move.to.row as u16) << 8 | book_move.to.col as u16).to_le_bytes();
        bytes.extend_from_slice(&to_bytes);

        // Piece type (1 byte)
        bytes.push(book_move.piece_type.to_u8());

        // Flags (1 byte: bit 0 = is_drop, bit 1 = is_promotion)
        let mut flags = 0u8;
        if book_move.is_drop {
            flags |= 0x01;
        }
        if book_move.is_promotion {
            flags |= 0x02;
        }
        bytes.push(flags);

        // Weight (4 bytes)
        bytes.extend_from_slice(&book_move.weight.to_le_bytes());

        // Evaluation (4 bytes)
        bytes.extend_from_slice(&book_move.evaluation.to_le_bytes());

        // Opening name length and data (4 bytes + variable)
        if let Some(ref name) = book_move.opening_name {
            let name_bytes = name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(name_bytes);
        } else {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        // Move notation length and data (4 bytes + variable)
        if let Some(ref notation) = book_move.move_notation {
            let notation_bytes = notation.as_bytes();
            bytes.extend_from_slice(&(notation_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(notation_bytes);
        } else {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        Ok(bytes.into_boxed_slice())
    }
}

impl BinaryReader {
    /// Create a new reader
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: data.into_boxed_slice(),
            position: 0,
        }
    }

    /// Read opening book from binary format
    pub fn read_opening_book(&mut self) -> Result<OpeningBook, OpeningBookError> {
        // Read header
        let header = BinaryHeader::from_bytes(&self.data[0..48])?;
        self.position = 48;

        // Read hash table
        let hash_table_size = header.hash_table_size as usize;
        let mut hash_table = Vec::with_capacity(hash_table_size);

        // Handle empty book case
        if hash_table_size == 0 {
            return Ok(OpeningBook {
                positions: HashMap::new(),
                lazy_positions: HashMap::new(),
                position_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
                temp_buffer: Vec::with_capacity(1024),
                total_moves: 0,
                loaded: false,
                metadata: OpeningBookMetadata {
                    version: header.version,
                    position_count: 0,
                    move_count: 0,
                    created_at: Some(header.created_at.to_string()),
                    updated_at: Some(header.updated_at.to_string()),
                    streaming_enabled: false,
                    chunk_size: 0,
                },
                hash_collision_stats: crate::opening_book::HashCollisionStats::new(),
                chunk_manager: None,
            });
        }

        for _ in 0..hash_table_size {
            let position_hash = self.read_u64()?;
            let entry_offset = self.read_u64()?;
            hash_table.push(HashTableEntry {
                position_hash,
                entry_offset,
            });
        }

        // Read position entries
        let mut positions = HashMap::new();
        let mut total_moves = 0;

        for entry in &hash_table {
            if entry.position_hash == 0 && entry.entry_offset == 0 {
                continue; // Skip empty hash table slots
            }

            self.position = entry.entry_offset as usize;
            let (fen, moves) = self.read_position_entry()?;
            total_moves += moves.len();

            positions.insert(entry.position_hash, PositionEntry { fen, moves });
        }

        let position_count = positions.len();
        Ok(OpeningBook {
            positions,
            lazy_positions: HashMap::new(),
            position_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            temp_buffer: Vec::with_capacity(1024),
            total_moves,
            loaded: true,
            metadata: OpeningBookMetadata {
                version: header.version,
                position_count,
                move_count: total_moves,
                created_at: Some(header.created_at.to_string()),
                updated_at: Some(header.updated_at.to_string()),
                streaming_enabled: false,
                chunk_size: 0,
            },
            hash_collision_stats: crate::opening_book::HashCollisionStats::new(),
            chunk_manager: None,
        })
    }

    /// Read chunk header for streaming
    pub fn read_chunk_header(&mut self) -> Result<ChunkHeader, OpeningBookError> {
        let position_count = self.read_u32()? as usize;
        let chunk_offset = self.read_u64()?;
        let chunk_size = self.read_u32()? as usize;

        Ok(ChunkHeader {
            position_count,
            chunk_offset,
            chunk_size,
        })
    }

    /// Read a position entry
    pub fn read_position_entry(&mut self) -> Result<(String, Vec<BookMove>), OpeningBookError> {
        // Read FEN string
        let fen_len = self.read_u32()? as usize;
        let fen_bytes = self.read_bytes(fen_len)?;
        let fen = String::from_utf8(fen_bytes).map_err(|e| {
            OpeningBookError::BinaryFormatError(format!("Invalid UTF-8 in FEN: {}", e))
        })?;

        // Read move count
        let move_count = self.read_u32()? as usize;

        // Read moves
        let mut moves = Vec::with_capacity(move_count);
        for _ in 0..move_count {
            moves.push(self.read_book_move()?);
        }

        Ok((fen, moves))
    }

    /// Read a book move
    pub fn read_book_move(&mut self) -> Result<BookMove, OpeningBookError> {
        // Read from position
        let from_bytes = self.read_u16()?;
        let from = if from_bytes == 0xFFFF {
            None
        } else {
            Some(Position::new(
                ((from_bytes >> 8) & 0xFF) as u8,
                (from_bytes & 0xFF) as u8,
            ))
        };

        // Read to position
        let to_bytes = self.read_u16()?;
        let to = Position::new(((to_bytes >> 8) & 0xFF) as u8, (to_bytes & 0xFF) as u8);

        // Read piece type
        let piece_type_byte = self.read_u8()?;
        let piece_type = PieceType::from_u8(piece_type_byte);

        // Read flags
        let flags = self.read_u8()?;
        let is_drop = (flags & 0x01) != 0;
        let is_promotion = (flags & 0x02) != 0;

        // Read weight
        let weight = self.read_u32()?;

        // Read evaluation
        let evaluation = self.read_i32()?;

        // Read opening name
        let name_len = self.read_u32()? as usize;
        let opening_name = if name_len > 0 {
            let name_bytes = self.read_bytes(name_len)?;
            Some(String::from_utf8(name_bytes).map_err(|e| {
                OpeningBookError::BinaryFormatError(format!("Invalid UTF-8 in opening name: {}", e))
            })?)
        } else {
            None
        };

        // Read move notation
        let notation_len = self.read_u32()? as usize;
        let move_notation = if notation_len > 0 {
            let notation_bytes = self.read_bytes(notation_len)?;
            Some(String::from_utf8(notation_bytes).map_err(|e| {
                OpeningBookError::BinaryFormatError(format!(
                    "Invalid UTF-8 in move notation: {}",
                    e
                ))
            })?)
        } else {
            None
        };

        Ok(BookMove {
            from,
            to,
            piece_type,
            is_drop,
            is_promotion,
            weight,
            evaluation,
            opening_name,
            move_notation,
        })
    }

    /// Helper methods for reading primitive types
    fn read_u8(&mut self) -> Result<u8, OpeningBookError> {
        if self.position >= self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, OpeningBookError> {
        if self.position + 1 >= self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let bytes = [self.data[self.position], self.data[self.position + 1]];
        self.position += 2;
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_u32(&mut self) -> Result<u32, OpeningBookError> {
        if self.position + 3 >= self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ];
        self.position += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, OpeningBookError> {
        if self.position + 7 >= self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ];
        self.position += 8;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_i32(&mut self) -> Result<i32, OpeningBookError> {
        if self.position + 3 >= self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ];
        self.position += 4;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, OpeningBookError> {
        if self.position + len > self.data.len() {
            return Err(OpeningBookError::BinaryFormatError(
                "Unexpected end of data".to_string(),
            ));
        }
        let bytes = self.data[self.position..self.position + len].to_vec();
        self.position += len;
        Ok(bytes)
    }
}

impl Default for BinaryWriter {
    fn default() -> Self {
        Self::new()
    }
}
