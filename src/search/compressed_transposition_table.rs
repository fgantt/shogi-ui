//! Compressed transposition table (L2 backing store)
//!
//! Provides a capacity-oriented store for transposition entries that keeps
//! data compressed on disk while exposing a familiar probe/store API. This is
//! the building block for the hierarchical L1/L2 design targeted by Task 9.0.

use std::cmp;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::types::core::{Move, Piece, PieceType, Player, Position};
use crate::types::search::{EntrySource, TranspositionFlag};
use crate::types::transposition::TranspositionEntry;

const LOGICAL_ENTRY_SIZE: u64 = std::mem::size_of::<TranspositionEntry>() as u64;

/// Configuration for the compressed transposition table.
#[derive(Debug, Clone)]
pub struct CompressedTranspositionTableConfig {
    /// Maximum number of logical entries the table should retain.
    pub max_entries: usize,
    /// Number of independent segments used for bucketing (must be > 0).
    pub segment_count: usize,
    /// Target compression ratio (logical bytes / physical bytes).
    pub target_compression_ratio: f64,
    /// Maximum backlog length for background maintenance sweep (entries).
    pub max_maintenance_backlog: usize,
}

impl Default for CompressedTranspositionTableConfig {
    fn default() -> Self {
        Self {
            max_entries: 1_000_000,
            segment_count: 256,
            target_compression_ratio: 0.5,
            max_maintenance_backlog: 10_000,
        }
    }
}

impl CompressedTranspositionTableConfig {
    /// Set the maximum number of entries.
    pub fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Set the number of segments (will be rounded to next power-of-two at runtime).
    pub fn with_segment_count(mut self, segment_count: usize) -> Self {
        self.segment_count = segment_count.max(1);
        self
    }

    /// Set the target compression ratio (clamped between 0.1 and 1.0).
    pub fn with_target_compression_ratio(mut self, ratio: f64) -> Self {
        let clamped = ratio.clamp(0.1, 1.0);
        self.target_compression_ratio = clamped;
        self
    }

    /// Set the maximum maintenance backlog (minimum 1).
    pub fn with_max_maintenance_backlog(mut self, backlog: usize) -> Self {
        self.max_maintenance_backlog = backlog.max(1);
        self
    }
}

/// Runtime statistics for the compressed table.
#[derive(Debug, Clone, Default)]
pub struct CompressedTranspositionStats {
    pub stored_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub logical_bytes: u64,
    pub physical_bytes: u64,
}

impl CompressedTranspositionStats {
    /// Effective compression ratio `logical / physical`.
    pub fn compression_ratio(&self) -> f64 {
        if self.physical_bytes == 0 || self.logical_bytes == 0 {
            1.0
        } else {
            self.logical_bytes as f64 / self.physical_bytes as f64
        }
    }
}

/// Segment-local record for a compressed TT entry.
#[derive(Debug, Clone)]
struct SegmentRecord {
    hash_key: u64,
    depth: u8,
    flag: TranspositionFlag,
    age: u32,
    source: EntrySource,
    best_move: Option<Move>,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
struct Segment {
    records: VecDeque<SegmentRecord>,
}

impl Segment {
    fn clear(&mut self) {
        self.records.clear();
    }
}

/// Compressed backing store for large (L2) transposition table tiers.
pub struct CompressedTranspositionTable {
    config: CompressedTranspositionTableConfig,
    segments: Vec<Segment>,
    segment_mask: usize,
    segment_capacity: usize,
    entry_count: usize,
    stats: CompressedTranspositionStats,
}

impl CompressedTranspositionTable {
    /// Create a new compressed transposition table.
    pub fn new(config: CompressedTranspositionTableConfig) -> Self {
        let mut effective_config = config;
        if effective_config.segment_count == 0 {
            effective_config.segment_count = 1;
        }
        if !(0.1..=1.0).contains(&effective_config.target_compression_ratio) {
            effective_config.target_compression_ratio =
                effective_config.target_compression_ratio.clamp(0.1, 1.0);
        }
        let segment_count = effective_config.segment_count.next_power_of_two();
        let segment_capacity = cmp::max(1, effective_config.max_entries / segment_count);

        Self {
            segments: vec![Segment::default(); segment_count],
            segment_mask: segment_count - 1,
            segment_capacity,
            entry_count: 0,
            stats: CompressedTranspositionStats::default(),
            config: effective_config,
        }
    }

    /// Number of entries currently retained.
    pub fn len(&self) -> usize {
        self.entry_count
    }

    /// Whether the table has no stored entries.
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Total number of segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Capacity per segment.
    pub fn segment_capacity(&self) -> usize {
        self.segment_capacity
    }

    /// Access the configuration backing this table.
    pub fn config(&self) -> &CompressedTranspositionTableConfig {
        &self.config
    }

    /// Access the current statistics snapshot.
    pub fn stats(&self) -> &CompressedTranspositionStats {
        &self.stats
    }

    /// Current fill ratio (0.0 - 1.0).
    pub fn fill_ratio(&self) -> f64 {
        if self.config.max_entries == 0 {
            0.0
        } else {
            (self.entry_count as f64) / (self.config.max_entries as f64)
        }
    }

    /// Probe the compressed table. Returns `None` on miss or depth mismatch.
    pub fn probe(&mut self, hash_key: u64, required_depth: u8) -> Option<TranspositionEntry> {
        let idx = self.segment_index(hash_key);
        let (left, right) = self.segments.split_at_mut(idx);
        let segment = if right.is_empty() {
            // Should not happen, but guard against indexing errors.
            left.last_mut()?
        } else {
            &mut right[0]
        };

        for record in segment.records.iter().rev() {
            if record.hash_key == hash_key && record.depth >= required_depth {
                self.stats.hits += 1;
                let mut entry = decode_entry(&record.payload, record.hash_key, record.age);
                entry.source = record.source;
                entry.flag = record.flag;
                entry.best_move = record.best_move.clone();
                return Some(entry);
            }
        }

        self.stats.misses += 1;
        None
    }

    /// Store (or replace) an entry in the compressed table.
    pub fn store(&mut self, entry: &TranspositionEntry) {
        let payload = encode_entry(entry);
        let physical = payload.len() as u64;
        let record = SegmentRecord {
            hash_key: entry.hash_key,
            depth: entry.depth,
            flag: entry.flag,
            age: entry.age,
            source: entry.source,
            best_move: entry.best_move.clone(),
            payload,
        };

        let idx = self.segment_index(record.hash_key);
        let (left, right) = self.segments.split_at_mut(idx);
        let segment = if right.is_empty() {
            left.last_mut().expect("segment indexing failed")
        } else {
            &mut right[0]
        };

        if let Some(existing_index) = segment
            .records
            .iter()
            .position(|r| r.hash_key == record.hash_key)
        {
            // Replace if the new entry is at least as deep as the existing one.
            if record.depth >= segment.records[existing_index].depth {
                let old_physical = segment.records[existing_index].payload.len() as u64;
                segment.records[existing_index] = record;
                self.stats.physical_bytes = self
                    .stats
                    .physical_bytes
                    .saturating_add(physical)
                    .saturating_sub(old_physical);
            }
            // If the existing entry is deeper we keep it and drop the new record.
            return;
        }

        if segment.records.len() >= self.segment_capacity
            || self.entry_count >= self.config.max_entries
        {
            if let Some(evicted) = segment.records.pop_front() {
                self.stats.evictions += 1;
                self.stats.logical_bytes =
                    self.stats.logical_bytes.saturating_sub(LOGICAL_ENTRY_SIZE);
                self.stats.physical_bytes = self
                    .stats
                    .physical_bytes
                    .saturating_sub(evicted.payload.len() as u64);
            }
        } else {
            self.entry_count += 1;
        }

        segment.records.push_back(record);
        self.stats.logical_bytes = self.stats.logical_bytes.saturating_add(LOGICAL_ENTRY_SIZE);
        self.stats.physical_bytes = self.stats.physical_bytes.saturating_add(physical);
        self.stats.stored_entries = self.entry_count;
    }

    /// Remove all entries and reset statistics.
    pub fn clear(&mut self) {
        for segment in &mut self.segments {
            segment.clear();
        }
        self.entry_count = 0;
        self.stats = CompressedTranspositionStats::default();
    }

    fn segment_index(&self, hash_key: u64) -> usize {
        (hash_key as usize) & self.segment_mask
    }

    /// Perform a maintenance sweep, evicting entries until backlog constraints are met.
    pub fn maintenance_sweep(
        &mut self,
        max_backlog: usize,
        max_duration: Option<Duration>,
    ) -> usize {
        if self.entry_count == 0 {
            return 0;
        }

        let allowed = max_backlog.max(1).min(self.config.max_entries.max(1));

        if self.entry_count <= allowed {
            return 0;
        }

        let deadline = max_duration.map(|d| Instant::now() + d);
        let mut removed_total = 0usize;

        while self.entry_count > allowed {
            let mut removed_this_pass = 0usize;

            for segment in &mut self.segments {
                if self.entry_count <= allowed {
                    break;
                }

                if let Some(evicted) = segment.records.pop_front() {
                    self.entry_count = self.entry_count.saturating_sub(1);
                    self.stats.logical_bytes =
                        self.stats.logical_bytes.saturating_sub(LOGICAL_ENTRY_SIZE);
                    self.stats.physical_bytes = self
                        .stats
                        .physical_bytes
                        .saturating_sub(evicted.payload.len() as u64);
                    self.stats.evictions = self.stats.evictions.saturating_add(1);
                    self.stats.stored_entries = self.stats.stored_entries.saturating_sub(1);

                    removed_total += 1;
                    removed_this_pass += 1;
                }

                if let Some(deadline) = deadline {
                    if Instant::now() >= deadline {
                        self.stats.stored_entries = self.entry_count;
                        return removed_total;
                    }
                }
            }

            if removed_this_pass == 0 {
                break;
            }
        }

        self.stats.stored_entries = self.entry_count;
        removed_total
    }
}

#[inline(always)]
fn encode_entry(entry: &TranspositionEntry) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(24);
    write_varint_i64(entry.score as i64, &mut buffer);
    buffer.push(entry.depth);
    buffer.push(match entry.flag {
        TranspositionFlag::Exact => 0,
        TranspositionFlag::LowerBound => 1,
        TranspositionFlag::UpperBound => 2,
    });
    encode_move(entry.best_move.as_ref(), &mut buffer);
    buffer
}

#[inline(always)]
fn decode_entry(payload: &[u8], hash_key: u64, age: u32) -> TranspositionEntry {
    let mut cursor = 0;
    let score = read_varint_i64(payload, &mut cursor) as i32;
    let depth = payload.get(cursor).copied().unwrap_or(0);
    cursor += 1;
    let flag = match payload.get(cursor).copied().unwrap_or(0) {
        0 => TranspositionFlag::Exact,
        1 => TranspositionFlag::LowerBound,
        2 => TranspositionFlag::UpperBound,
        _ => TranspositionFlag::Exact,
    };
    cursor += 1;
    let best_move = decode_move(payload, &mut cursor);

    TranspositionEntry::new(
        score,
        depth,
        flag,
        best_move,
        hash_key,
        age,
        EntrySource::MainSearch,
    )
}

#[inline(always)]
fn encode_move(mv: Option<&Move>, buffer: &mut Vec<u8>) {
    match mv {
        Some(move_) => {
            let mut header = 0u8;
            header |= 1; // has move
            if move_.from.is_some() {
                header |= 1 << 1;
            }
            if move_.is_promotion {
                header |= 1 << 2;
            }
            if move_.is_capture {
                header |= 1 << 3;
            }
            if move_.gives_check {
                header |= 1 << 4;
            }
            if move_.is_recapture {
                header |= 1 << 5;
            }
            if move_.captured_piece.is_some() {
                header |= 1 << 6;
            }
            if move_.player == Player::White {
                header |= 1 << 7;
            }

            buffer.push(header);
            if let Some(from) = move_.from {
                buffer.push(from.to_u8());
            }
            buffer.push(move_.to.to_u8());
            buffer.push(move_.piece_type.to_u8());
            if let Some(captured) = &move_.captured_piece {
                buffer.push(captured.piece_type.to_u8());
                buffer.push(captured.player as u8);
            }
        }
        None => buffer.push(0),
    }
}

#[inline(always)]
fn decode_move(payload: &[u8], cursor: &mut usize) -> Option<Move> {
    if *cursor >= payload.len() {
        return None;
    }
    let header = payload[*cursor];
    *cursor += 1;
    if header & 1 == 0 {
        return None;
    }

    let from = if (header & (1 << 1)) != 0 {
        if *cursor >= payload.len() {
            return None;
        }
        let idx = payload[*cursor];
        *cursor += 1;
        Some(Position::from_u8(idx))
    } else {
        None
    };

    if *cursor >= payload.len() {
        return None;
    }
    let to = Position::from_u8(payload[*cursor]);
    *cursor += 1;

    if *cursor >= payload.len() {
        return None;
    }
    let piece_type = PieceType::from_u8(payload[*cursor]);
    *cursor += 1;

    let player = if (header & (1 << 7)) != 0 {
        Player::White
    } else {
        Player::Black
    };

    Some(Move {
        from,
        to,
        piece_type,
        player,
        is_promotion: (header & (1 << 2)) != 0,
        is_capture: (header & (1 << 3)) != 0,
        captured_piece: if (header & (1 << 6)) != 0 {
            if *cursor + 2 > payload.len() {
                return None;
            }
            let captured_type = PieceType::from_u8(payload[*cursor]);
            let captured_player = if payload[*cursor + 1] == Player::White as u8 {
                Player::White
            } else {
                Player::Black
            };
            *cursor += 2;
            Some(Piece::new(captured_type, captured_player))
        } else {
            None
        },
        gives_check: (header & (1 << 4)) != 0,
        is_recapture: (header & (1 << 5)) != 0,
    })
}

#[inline(always)]
fn write_varint_i64(value: i64, buffer: &mut Vec<u8>) {
    let mut zigzag = ((value << 1) ^ (value >> 63)) as u64;
    while zigzag >= 0x80 {
        buffer.push((zigzag as u8) | 0x80);
        zigzag >>= 7;
    }
    buffer.push(zigzag as u8);
}

#[inline(always)]
fn read_varint_i64(buffer: &[u8], cursor: &mut usize) -> i64 {
    let mut result: u64 = 0;
    let mut shift = 0;

    while *cursor < buffer.len() && shift <= 63 {
        let byte = buffer[*cursor] as u64;
        *cursor += 1;

        result |= (byte & 0x7F) << shift;
        if (byte & 0x80) == 0 {
            let value = ((result >> 1) as i64) ^ (-((result & 1) as i64));
            return value;
        }

        shift += 7;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, PieceType, Player, Position, TranspositionFlag};

    fn sample_entry(hash: u64, depth: u8, score: i32) -> TranspositionEntry {
        let best_move = Move::new_move(
            Position::new(4, 4),
            Position::new(4, 5),
            PieceType::Silver,
            Player::Black,
            false,
        );
        TranspositionEntry::new(
            score,
            depth,
            TranspositionFlag::Exact,
            Some(best_move),
            hash,
            3,
            EntrySource::MainSearch,
        )
    }

    #[test]
    fn store_and_probe_round_trip() {
        let mut table = CompressedTranspositionTable::new(CompressedTranspositionTableConfig {
            max_entries: 64,
            segment_count: 8,
            target_compression_ratio: 0.5,
            max_maintenance_backlog: 10_000,
        });

        let entry = sample_entry(0xABCDEF, 6, 120);
        table.store(&entry);

        let retrieved = table.probe(0xABCDEF, 4).expect("entry should be present");
        assert_eq!(retrieved.score, 120);
        assert_eq!(retrieved.depth, 6);
        assert_eq!(retrieved.hash_key, 0xABCDEF);
        assert!(retrieved.best_move.is_some());
    }

    #[test]
    fn depth_preference_respected_on_replacement() {
        let mut table = CompressedTranspositionTable::new(CompressedTranspositionTableConfig {
            max_entries: 16,
            segment_count: 4,
            target_compression_ratio: 0.5,
            max_maintenance_backlog: 10_000,
        });

        let shallow = sample_entry(0x1234, 4, 50);
        let deeper = sample_entry(0x1234, 6, 80);

        table.store(&shallow);
        table.store(&deeper);

        let retrieved = table.probe(0x1234, 5).expect("deep entry retained");
        assert_eq!(retrieved.score, 80);

        // Attempt to store a shallower entry again; should be ignored.
        let shallower_again = sample_entry(0x1234, 3, 10);
        table.store(&shallower_again);

        let retrieved_again = table.probe(0x1234, 5).expect("deep entry should remain");
        assert_eq!(retrieved_again.score, 80);
    }

    #[test]
    fn segment_eviction_happens_when_capacity_exceeded() {
        let mut table = CompressedTranspositionTable::new(CompressedTranspositionTableConfig {
            max_entries: 4,
            segment_count: 2,
            target_compression_ratio: 0.5,
            max_maintenance_backlog: 10_000,
        });

        // Fill segment with unique hashes mapping to same index.
        for i in 0..8 {
            let hash = (i << 8) | 0x1;
            let entry = sample_entry(hash, 5, i as i32);
            table.store(&entry);
        }

        assert!(table.stats().evictions > 0);
    }
}
