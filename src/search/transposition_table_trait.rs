//! Transposition Table Trait
//!
//! This module provides a unified trait for all transposition table implementations,
//! enabling polymorphic usage and easier testing. This is part of Task 3.0 - Integration
//! Synchronization and Coordination Fixes.

use crate::types::transposition::TranspositionEntry;
use std::cell::RefCell;

/// Unified trait for transposition table implementations
///
/// This trait provides a common interface for all transposition table types,
/// allowing polymorphic usage throughout the search engine. Different implementations
/// provide different characteristics (thread-safety, memory efficiency, etc.).
///
/// Note: Some implementations use interior mutability (e.g., ThreadSafeTranspositionTable)
/// and can take `&self` for all methods, while others require `&mut self` for mutations.
/// The trait signatures allow for both patterns via implementation.
///
/// # Task 3.0 (Task 3.15, Task 3.21)
pub trait TranspositionTableTrait {
    /// Probe the table for an entry with the given hash key
    ///
    /// # Arguments
    /// * `hash_key` - The hash key of the position to look up
    /// * `depth` - Minimum depth required for the entry to be valid
    ///
    /// # Returns
    /// `Some(TranspositionEntry)` if a valid entry is found, `None` otherwise
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry>;

    /// Probe the table while optionally prefetching the next anticipated entry
    ///
    /// This is a performance optimization for implementations that support prefetching.
    /// The default implementation simply calls `probe()` ignoring the prefetch hint.
    ///
    /// # Arguments
    /// * `hash_key` - The hash key of the position to look up
    /// * `depth` - Minimum depth required for the entry to be valid
    /// * `next_hash` - Optional hash key for the next anticipated probe (for prefetching)
    ///
    /// # Returns
    /// `Some(TranspositionEntry)` if a valid entry is found, `None` otherwise
    ///
    /// # Task 3.0 (Task 3.21)
    fn probe_with_prefetch(&self, hash_key: u64, depth: u8, _next_hash: Option<u64>) -> Option<TranspositionEntry> {
        // Default implementation just calls probe - implementations can override for prefetching
        self.probe(hash_key, depth)
    }

    /// Store an entry in the table
    ///
    /// # Arguments
    /// * `entry` - The transposition entry to store
    fn store(&mut self, entry: TranspositionEntry);

    /// Clear all entries from the table
    fn clear(&mut self);

    /// Get the size (capacity) of the table
    ///
    /// Returns the maximum number of entries the table can hold.
    fn size(&self) -> usize;

    /// Get hit rate as a percentage (0.0 to 100.0)
    ///
    /// Returns the hit rate if statistics are available, otherwise returns 0.0.
    /// This is an optional method - not all implementations track statistics.
    ///
    /// # Task 3.0 (Task 3.21)
    fn hit_rate(&self) -> f64 {
        0.0 // Default: no statistics available
    }

    /// Prefill the table with entries from an opening book.
    ///
    /// This is an optional method - not all implementations support opening book prefilling.
    /// The default implementation is a no-op that returns 0.
    ///
    /// # Arguments
    /// * `book` - The opening book to extract positions from
    /// * `depth` - The depth to assign to prefilled entries
    ///
    /// # Returns
    /// The number of entries inserted.
    ///
    /// # Task 3.0 (Task 3.21)
    fn prefill_from_book(&mut self, _book: &mut crate::opening_book::OpeningBook, _depth: u8) -> usize {
        0 // Default: no-op
    }
}

// Task 3.16: Implement TranspositionTableTrait for TranspositionTable (basic, single-threaded)
// Note: TranspositionTable requires &mut self for probe/store/clear, so we wrap it in RefCell
// to provide the &self interface required by the trait.
impl TranspositionTableTrait for RefCell<crate::search::transposition_table::TranspositionTable> {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        self.borrow_mut().probe(hash_key, depth)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        self.get_mut().store(entry);
    }

    fn clear(&mut self) {
        self.get_mut().clear();
    }

    fn size(&self) -> usize {
        self.borrow().get_size()
    }

    fn prefill_from_book(&mut self, book: &mut crate::opening_book::OpeningBook, depth: u8) -> usize {
        self.get_mut().prefill_from_book(book, depth)
    }
}

// Task 3.17: Implement TranspositionTableTrait for ThreadSafeTranspositionTable
impl TranspositionTableTrait for crate::search::thread_safe_table::ThreadSafeTranspositionTable {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        // ThreadSafeTranspositionTable uses interior mutability, so probe takes &self
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::probe(self, hash_key, depth)
    }

    fn probe_with_prefetch(&self, hash_key: u64, depth: u8, next_hash: Option<u64>) -> Option<TranspositionEntry> {
        // ThreadSafeTranspositionTable supports prefetching
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::probe_with_prefetch(self, hash_key, depth, next_hash)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        // ThreadSafeTranspositionTable::store takes &self, so we can call it on &mut self
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::store(self, entry);
    }

    fn clear(&mut self) {
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::clear(self);
    }

    fn size(&self) -> usize {
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::size(self)
    }

    fn hit_rate(&self) -> f64 {
        // ThreadSafeTranspositionTable tracks statistics
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::hit_rate(self)
    }

    fn prefill_from_book(&mut self, book: &mut crate::opening_book::OpeningBook, depth: u8) -> usize {
        crate::search::thread_safe_table::ThreadSafeTranspositionTable::prefill_from_book(self, book, depth)
    }
}

// Also implement the trait for RefCell<ThreadSafeTranspositionTable> for use in factory function
impl TranspositionTableTrait for RefCell<crate::search::thread_safe_table::ThreadSafeTranspositionTable> {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        // ThreadSafeTranspositionTable uses interior mutability, so we can borrow immutably
        self.borrow().probe(hash_key, depth)
    }

    fn probe_with_prefetch(&self, hash_key: u64, depth: u8, next_hash: Option<u64>) -> Option<TranspositionEntry> {
        // ThreadSafeTranspositionTable supports prefetching
        self.borrow().probe_with_prefetch(hash_key, depth, next_hash)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        // ThreadSafeTranspositionTable::store takes &self, so we can borrow immutably
        self.borrow().store(entry);
    }

    fn clear(&mut self) {
        // ThreadSafeTranspositionTable::clear takes &self, so we can borrow immutably
        // Note: We take &mut self for the trait signature, but the underlying clear() only needs &self
        self.get_mut().clear();
    }

    fn size(&self) -> usize {
        self.borrow().size()
    }

    fn hit_rate(&self) -> f64 {
        // ThreadSafeTranspositionTable tracks statistics
        self.borrow().hit_rate()
    }

    fn prefill_from_book(&mut self, book: &mut crate::opening_book::OpeningBook, depth: u8) -> usize {
        // Note: prefill_from_book takes &mut self, but ThreadSafeTranspositionTable uses interior mutability
        // so we can borrow immutably from RefCell, then call the method which takes &mut self on the inner value
        // Actually, ThreadSafeTranspositionTable::prefill_from_book takes &mut self, so we need get_mut()
        self.get_mut().prefill_from_book(book, depth)
    }
}

// Task 3.19: Implement TranspositionTableTrait for MultiLevelTranspositionTable
// Note: MultiLevelTranspositionTable requires &mut self for probe/store/clear, so we wrap it in RefCell
// to provide the &self interface required by the trait.
impl TranspositionTableTrait for RefCell<crate::search::multi_level_transposition_table::MultiLevelTranspositionTable> {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        self.borrow_mut().probe(hash_key, depth)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        self.get_mut().store(entry);
    }

    fn clear(&mut self) {
        self.get_mut().clear();
    }

    fn size(&self) -> usize {
        // Return total capacity across all levels
        let table = self.borrow();
        // Use get_stats() which is public to get level memory usage, then estimate entries
        // This is approximate - we could add a public method to get total size if needed
        let stats = table.get_stats();
        // Estimate based on total memory usage (approximately 100 bytes per entry)
        (stats.total_memory_usage / 100) as usize
    }

    fn prefill_from_book(&mut self, _book: &mut crate::opening_book::OpeningBook, _depth: u8) -> usize {
        // MultiLevelTranspositionTable doesn't support prefill_from_book
        // Could be implemented in the future if needed
        0
    }
}

// Task 3.18: Implement TranspositionTableTrait for HierarchicalTranspositionTable
// Note: HierarchicalTranspositionTable::probe returns Option<(TranspositionEntry, HitLevel)>
// which is different from the trait signature, so we adapt it.
impl TranspositionTableTrait for RefCell<crate::search::hierarchical_transposition_table::HierarchicalTranspositionTable> {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        self.borrow_mut().probe(hash_key, depth).map(|(entry, _level)| entry)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        self.get_mut().store(entry);
    }

    fn clear(&mut self) {
        self.get_mut().clear();
    }

    fn size(&self) -> usize {
        // Return combined capacity of L1 and L2
        // Note: Since fields are private, we use snapshot() to get L2 stats
        // and estimate L1 size from default config. This is approximate.
        // TODO: Add public size() method to HierarchicalTranspositionTable
        let table = self.borrow();
        let snapshot = table.snapshot();
        // Estimate: L1 is typically ~1M entries, L2 from stats
        let l2_capacity = snapshot.l2_stats.stored_entries.max(8_000_000); // Default L2 size
        let l1_estimate = 1_000_000; // Default L1 size from config
        l1_estimate + l2_capacity
    }

    fn prefill_from_book(&mut self, _book: &mut crate::opening_book::OpeningBook, _depth: u8) -> usize {
        // HierarchicalTranspositionTable doesn't support prefill_from_book
        // Could be implemented in the future if needed
        0
    }
}

// Task 3.20: Implement TranspositionTableTrait for CompressedTranspositionTable
// Note: CompressedTranspositionTable::store takes &TranspositionEntry, so we adapt it.
impl TranspositionTableTrait for RefCell<crate::search::compressed_transposition_table::CompressedTranspositionTable> {
    fn probe(&self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        self.borrow_mut().probe(hash_key, depth)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        self.get_mut().store(&entry);
    }

    fn clear(&mut self) {
        self.get_mut().clear();
    }

    fn size(&self) -> usize {
        // Return max capacity from config
        let table = self.borrow();
        // Use public config() method to get max_entries
        table.config().max_entries
    }

    fn prefill_from_book(&mut self, _book: &mut crate::opening_book::OpeningBook, _depth: u8) -> usize {
        // CompressedTranspositionTable doesn't support prefill_from_book
        // Could be implemented in the future if needed
        0
    }
}

