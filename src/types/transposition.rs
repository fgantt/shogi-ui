//! Transposition Table Types
//!
//! This module contains types related to transposition tables: entry structures,
//! flags, and quiescence search entries.
//!
//! Extracted from `types.rs` (now `all.rs`) as part of Task 1.0: File Modularization and Structure Improvements.
//!
//! Note: `TranspositionFlag` and `EntrySource` are defined in the `search` module
//! as they are used by search algorithms. They are re-exported here for convenience.

use serde::{Deserialize, Serialize};

use super::core::Move;
use super::search::{EntrySource, TranspositionFlag};

// ============================================================================
// Transposition Table Entry
// ============================================================================

/// Transposition table entry storing position evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspositionEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
    /// Hash key for this entry (used for collision detection)
    pub hash_key: u64,
    /// Age counter for replacement policies
    pub age: u32,
    /// Source of this entry for priority management
    pub source: EntrySource,
}

impl TranspositionEntry {
    /// Create a new transposition table entry
    pub fn new(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        best_move: Option<Move>,
        hash_key: u64,
        age: u32,
        source: EntrySource,
    ) -> Self {
        Self {
            score,
            depth,
            flag,
            best_move,
            hash_key,
            age,
            source,
        }
    }

    /// Create a new entry with default age (0) and MainSearch source
    pub fn new_with_age(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        best_move: Option<Move>,
        hash_key: u64,
    ) -> Self {
        Self::new(
            score,
            depth,
            flag,
            best_move,
            hash_key,
            0,
            EntrySource::MainSearch,
        )
    }

    /// Check if this entry is valid for the given search depth
    pub fn is_valid_for_depth(&self, required_depth: u8) -> bool {
        self.depth >= required_depth
    }

    /// Check if this entry matches the given hash key
    pub fn matches_hash(&self, hash_key: u64) -> bool {
        self.hash_key == hash_key
    }

    /// Check if this entry is exact (not a bound)
    pub fn is_exact(&self) -> bool {
        matches!(self.flag, TranspositionFlag::Exact)
    }

    /// Check if this entry is a lower bound
    pub fn is_lower_bound(&self) -> bool {
        matches!(self.flag, TranspositionFlag::LowerBound)
    }

    /// Check if this entry is an upper bound
    pub fn is_upper_bound(&self) -> bool {
        matches!(self.flag, TranspositionFlag::UpperBound)
    }

    /// Update the age of this entry
    pub fn update_age(&mut self, new_age: u32) {
        self.age = new_age;
    }

    /// Get the memory size of this entry in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// Create a debug string representation
    pub fn debug_string(&self) -> String {
        let move_str = match &self.best_move {
            Some(m) => format!("{}", m.to_usi_string()),
            None => "None".to_string(),
        };

        format!(
            "TranspositionEntry {{ score: {}, depth: {}, flag: {:?}, best_move: {}, hash_key: 0x{:016x}, age: {}, source: {:?} }}",
            self.score, self.depth, self.flag, move_str, self.hash_key, self.age, self.source
        )
    }

    /// Check if this entry should be replaced by another entry
    pub fn should_replace_with(&self, other: &TranspositionEntry) -> bool {
        // Replace if hash keys don't match (collision)
        if !self.matches_hash(other.hash_key) {
            return true;
        }

        // Replace if the new entry has greater depth
        if other.depth > self.depth {
            return true;
        }

        // Replace if depths are equal but new entry is exact and current is not
        if other.depth == self.depth && other.is_exact() && !self.is_exact() {
            return true;
        }

        // Replace if the new entry is newer (higher age)
        if other.age > self.age {
            return true;
        }

        false
    }
}

// ============================================================================
// Quiescence Search Entry
// ============================================================================

/// Transposition table entry specifically for quiescence search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiescenceEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
    /// For LRU tracking - number of times this entry was accessed
    pub access_count: u64,
    /// For LRU tracking - age when last accessed
    pub last_access_age: u64,
    /// Cached stand-pat evaluation (optional, not all entries have it)
    pub stand_pat_score: Option<i32>,
}

impl QuiescenceEntry {
    /// Create a new quiescence entry
    pub fn new(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        best_move: Option<Move>,
    ) -> Self {
        Self {
            score,
            depth,
            flag,
            best_move,
            access_count: 0,
            last_access_age: 0,
            stand_pat_score: None,
        }
    }

    /// Create a new quiescence entry with stand-pat score
    pub fn new_with_stand_pat(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        best_move: Option<Move>,
        stand_pat_score: i32,
    ) -> Self {
        Self {
            score,
            depth,
            flag,
            best_move,
            access_count: 0,
            last_access_age: 0,
            stand_pat_score: Some(stand_pat_score),
        }
    }

    /// Record an access to this entry (for LRU tracking)
    pub fn record_access(&mut self, current_age: u64) {
        self.access_count += 1;
        self.last_access_age = current_age;
    }

    /// Check if this entry has a cached stand-pat score
    pub fn has_stand_pat_score(&self) -> bool {
        self.stand_pat_score.is_some()
    }

    /// Get the cached stand-pat score, if available
    pub fn get_stand_pat_score(&self) -> Option<i32> {
        self.stand_pat_score
    }

    /// Get the memory size of this entry in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposition_entry() {
        let entry = TranspositionEntry::new_with_age(
            100,
            5,
            TranspositionFlag::Exact,
            None,
            0x1234567890abcdef,
        );
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
        assert!(entry.is_exact());
        assert!(!entry.is_lower_bound());
        assert!(!entry.is_upper_bound());
        assert_eq!(entry.age, 0);
        assert_eq!(entry.source, EntrySource::MainSearch);
    }

    #[test]
    fn test_transposition_entry_validity() {
        let entry = TranspositionEntry::new_with_age(
            100,
            5,
            TranspositionFlag::Exact,
            None,
            0x1234567890abcdef,
        );
        assert!(entry.is_valid_for_depth(5));
        assert!(entry.is_valid_for_depth(4));
        assert!(!entry.is_valid_for_depth(6));
    }

    #[test]
    fn test_transposition_entry_replacement() {
        let old_entry = TranspositionEntry::new_with_age(
            100,
            5,
            TranspositionFlag::LowerBound,
            None,
            0x1234567890abcdef,
        );
        let new_entry = TranspositionEntry::new(
            200,
            6,
            TranspositionFlag::Exact,
            None,
            0x1234567890abcdef,
            1,
            EntrySource::MainSearch,
        );
        assert!(old_entry.should_replace_with(&new_entry));
    }

    #[test]
    fn test_quiescence_entry() {
        let entry = QuiescenceEntry::new(
            50,
            3,
            TranspositionFlag::Exact,
            None,
        );
        assert_eq!(entry.score, 50);
        assert_eq!(entry.depth, 3);
        assert!(!entry.has_stand_pat_score());
    }

    #[test]
    fn test_quiescence_entry_with_stand_pat() {
        let entry = QuiescenceEntry::new_with_stand_pat(
            50,
            3,
            TranspositionFlag::Exact,
            None,
            45,
        );
        assert!(entry.has_stand_pat_score());
        assert_eq!(entry.get_stand_pat_score(), Some(45));
    }

    #[test]
    fn test_quiescence_entry_access_tracking() {
        let mut entry = QuiescenceEntry::new(
            50,
            3,
            TranspositionFlag::Exact,
            None,
        );
        assert_eq!(entry.access_count, 0);
        entry.record_access(10);
        assert_eq!(entry.access_count, 1);
        assert_eq!(entry.last_access_age, 10);
    }
}

