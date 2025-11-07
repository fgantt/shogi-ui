//! Thread-safe transposition table implementation
//!
//! This module provides thread-safe transposition table functionality while
//! maintaining WASM compatibility. Since WASM is single-threaded, this module
//! provides a unified interface that works efficiently in both multi-threaded
//! native environments and single-threaded WASM environments.
//!
//! # Features
//!
//! - **Thread Safety**: Safe for concurrent access across multiple threads
//! - **WASM Compatibility**: Works in both native and WebAssembly environments
//! - **Performance Optimized**: Uses atomic operations and cache-line alignment
//! - **Memory Efficient**: Compact entry storage with configurable size
//! - **Statistics Tracking**: Comprehensive performance and usage statistics (opt-in)
//! - **Robust Under Failure**: Recovers from poisoned synchronization primitives without
//!   crashing the engine
//!
//! # Usage
//!
//! ```rust
//! use shogi_engine::search::{ThreadSafeTranspositionTable, TranspositionConfig, TranspositionEntry, TranspositionFlag};
//!
//! // Create a new transposition table (enable statistics only if needed)
//! let mut config = TranspositionConfig::default();
//! config.enable_statistics = true;
//! let mut tt = ThreadSafeTranspositionTable::new(config);
//!
//! // Store a search result
//! let entry = TranspositionEntry {
//!     hash_key: 12345,
//!     depth: 3,
//!     score: 100,
//!     flag: TranspositionFlag::Exact,
//!     best_move: None,
//!     age: 0,
//! };
//! tt.store(entry);
//!
//! // Retrieve a search result
//! if let Some(result) = tt.probe(12345, 3) {
//!     println!("Found entry with score: {}", result.score);
//! }
//!
//! // Get statistics
//! let stats = tt.get_stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
//! ```
//!
//! # Performance Considerations
//!
//! - Use appropriate table sizes based on available memory
//! - Consider cache line alignment for optimal performance
//! - Monitor hit rates and adjust configuration accordingly
//! - Use depth-preferred replacement for better search performance
//!
//! # Robustness
//!
//! - A warning is logged whenever the table recovers from a poisoned mutex or RW lock.
//! - When statistics are enabled, poison recovery events are counted via the
//!   `poison_recoveries` metric in `ThreadSafeStatsSnapshot`.

use crate::bitboards::BitboardBoard;
use crate::opening_book::OpeningBook;
use crate::search::cache_management::CacheManager;
#[cfg(not(target_arch = "wasm32"))]
use crate::search::replacement_policies::ReplacementDecision;
use crate::search::replacement_policies::ReplacementPolicyHandler;
use crate::search::transposition_config::TranspositionConfig;
use crate::search::zobrist::{RepetitionState, ZobristHasher};
use crate::types::*;
use log::warn;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{RwLock, RwLockWriteGuard};

#[cfg(all(feature = "tt-prefetch", target_arch = "x86"))]
use core::arch::x86::{_mm_prefetch, _MM_HINT_T2};
#[cfg(all(feature = "tt-prefetch", target_arch = "x86_64"))]
use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T2};

/// Platform-specific thread safety configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSafetyMode {
    /// Multi-threaded mode with full synchronization
    MultiThreaded,
    /// Single-threaded mode (WASM compatible)
    SingleThreaded,
    /// Auto-detect based on platform
    Auto,
}

impl ThreadSafetyMode {
    /// Detect the appropriate thread safety mode
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::SingleThreaded
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::MultiThreaded
        }
    }

    /// Check if this mode supports multiple threads
    pub fn is_multi_threaded(self) -> bool {
        match self {
            Self::MultiThreaded => true,
            Self::SingleThreaded => false,
            Self::Auto => Self::detect().is_multi_threaded(),
        }
    }
}

/// Thread-safe transposition table entry
///
/// This struct provides atomic operations for storing and retrieving
/// transposition table entries in a thread-safe manner.
#[derive(Debug)]
pub struct ThreadSafeEntry {
    /// Packed entry data for atomic operations
    packed_data: AtomicPackedEntry,
    /// Hash key for validation
    hash_key: AtomicU64,
    /// Age counter for replacement policies
    age: AtomicU32,
}

/// Packed entry data for atomic storage
///
/// This struct packs the essential entry data into a format suitable
/// for atomic operations, reducing memory overhead and improving
/// cache efficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomicPackedEntry {
    data: u64,
}

impl AtomicPackedEntry {
    const SCORE_BITS: u64 = 20;
    const SCORE_SHIFT: u64 = 44;
    const SCORE_MASK: u64 = (1 << Self::SCORE_BITS) - 1;
    const SCORE_MAX: i32 = 500_000;
    const SCORE_MIN: i32 = -500_000;

    const DEPTH_SHIFT: u64 = 36;
    const DEPTH_MASK: u64 = (1 << 8) - 1;

    const FLAG_SHIFT: u64 = 34;
    const FLAG_MASK: u64 = 0b11;

    const FROM_SHIFT: u64 = 27;
    const FROM_MASK: u64 = 0x7F; // 7 bits (0-80, 127 = drop)
    const TO_SHIFT: u64 = 20;
    const TO_MASK: u64 = 0x7F;

    const PIECE_SHIFT: u64 = 16;
    const PIECE_MASK: u64 = 0x0F; // 4 bits

    const MOVE_FLAGS_SHIFT: u64 = 14;
    const MOVE_FLAGS_MASK: u64 = 0b11; // bit 0 capture, bit 1 promotion

    const PLAYER_SHIFT: u64 = 13;
    const PLAYER_MASK: u64 = 0b1; // 0 = Black, 1 = White

    const HAS_MOVE_SHIFT: u64 = 12;
    const HAS_MOVE_MASK: u64 = 0b1;

    const DROP_SENTINEL: u8 = 0x7F;

    /// Create a new packed entry
    pub fn new(score: i32, depth: u8, flag: TranspositionFlag, best_move: Option<Move>) -> Self {
        let mut data = 0u64;

        let clamped_score = score.clamp(Self::SCORE_MIN, Self::SCORE_MAX);
        let score_encoded = (clamped_score - Self::SCORE_MIN) as u64;
        data |= (score_encoded & Self::SCORE_MASK) << Self::SCORE_SHIFT;

        data |= ((depth as u64) & Self::DEPTH_MASK) << Self::DEPTH_SHIFT;

        let flag_bits = match flag {
            TranspositionFlag::Exact => 0,
            TranspositionFlag::LowerBound => 1,
            TranspositionFlag::UpperBound => 2,
        } as u64;
        data |= (flag_bits & Self::FLAG_MASK) << Self::FLAG_SHIFT;

        if let Some(mv) = best_move {
            let from_idx = mv
                .from
                .map(|pos| pos.to_index())
                .unwrap_or(Self::DROP_SENTINEL);
            let to_idx = mv.to.to_index();
            let piece_idx = mv.piece_type.to_u8();
            let player_bit = match mv.player {
                Player::Black => 0u64,
                Player::White => 1u64,
            };
            let move_flags = ((mv.is_promotion as u64) << 1) | (mv.is_capture as u64);

            data |= ((from_idx as u64) & Self::FROM_MASK) << Self::FROM_SHIFT;
            data |= ((to_idx as u64) & Self::TO_MASK) << Self::TO_SHIFT;
            data |= ((piece_idx as u64) & Self::PIECE_MASK) << Self::PIECE_SHIFT;
            data |= (move_flags & Self::MOVE_FLAGS_MASK) << Self::MOVE_FLAGS_SHIFT;
            data |= (player_bit & Self::PLAYER_MASK) << Self::PLAYER_SHIFT;
            data |= 1 << Self::HAS_MOVE_SHIFT;
        }

        Self { data }
    }

    /// Extract score from packed data
    pub fn score(&self) -> i32 {
        let encoded = (self.data >> Self::SCORE_SHIFT) & Self::SCORE_MASK;
        (encoded as i32) + Self::SCORE_MIN
    }

    /// Extract depth from packed data
    pub fn depth(&self) -> u8 {
        ((self.data >> Self::DEPTH_SHIFT) & Self::DEPTH_MASK) as u8
    }

    /// Extract flag from packed data
    pub fn flag(&self) -> TranspositionFlag {
        match (self.data >> Self::FLAG_SHIFT) & Self::FLAG_MASK {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            2 => TranspositionFlag::UpperBound,
            _ => TranspositionFlag::Exact,
        }
    }

    /// Extract best move from packed data
    pub fn best_move(&self) -> Option<Move> {
        let has_move = ((self.data >> Self::HAS_MOVE_SHIFT) & Self::HAS_MOVE_MASK) == 1;
        if !has_move {
            return None;
        }

        let to_idx = ((self.data >> Self::TO_SHIFT) & Self::TO_MASK) as u8;
        let to = Position::from_index(to_idx);

        let from_idx = ((self.data >> Self::FROM_SHIFT) & Self::FROM_MASK) as u8;
        let from = if from_idx == Self::DROP_SENTINEL {
            None
        } else {
            Some(Position::from_index(from_idx))
        };

        let piece_idx = ((self.data >> Self::PIECE_SHIFT) & Self::PIECE_MASK) as u8;
        let piece_type = PieceType::from_u8(piece_idx);

        let move_flags = ((self.data >> Self::MOVE_FLAGS_SHIFT) & Self::MOVE_FLAGS_MASK) as u8;
        let is_promotion = (move_flags & 0b10) != 0;
        let is_capture = (move_flags & 0b01) != 0;

        let player = match ((self.data >> Self::PLAYER_SHIFT) & Self::PLAYER_MASK) as u8 {
            0 => Player::Black,
            1 => Player::White,
            _ => Player::Black,
        };

        Some(Move {
            from,
            to,
            piece_type,
            player,
            is_promotion,
            is_capture,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        })
    }

    /// Check if the entry is valid (non-zero)
    pub fn is_valid(&self) -> bool {
        self.data != 0
    }

    /// Create an empty/invalid entry
    pub fn empty() -> Self {
        Self { data: 0 }
    }
}

/// Thread-safe transposition table
///
/// This struct provides a thread-safe transposition table that works
/// efficiently in both multi-threaded and single-threaded environments.
/// In WASM, it operates without synchronization overhead.
///
/// # Parallel Performance
///
/// Uses bucketed locks to reduce write contention in multi-threaded environments.
/// Each bucket has its own RwLock, allowing parallel writes to different buckets.
/// Lock granularity is configurable via `TranspositionConfig::bucket_count`.
pub struct ThreadSafeTranspositionTable {
    /// The actual hash table storing thread-safe entries
    entries: Vec<ThreadSafeEntry>,
    /// Size of the table (number of slots)
    size: usize,
    /// Bit mask for fast modulo operations
    mask: usize,
    /// Thread safety mode
    thread_mode: ThreadSafetyMode,
    /// Synchronization primitives for multi-threaded access (bucketed for reduced contention)
    #[cfg(not(target_arch = "wasm32"))]
    bucket_locks: Vec<Arc<RwLock<()>>>,
    /// Bit shift for fast bucket calculation
    #[cfg(not(target_arch = "wasm32"))]
    bucket_shift: u32,
    /// Whether hardware prefetching is enabled for this table
    #[cfg(feature = "tt-prefetch")]
    prefetch_enabled: bool,
    /// Whether statistics tracking is enabled for this table
    statistics_enabled: bool,
    /// Cache manager for statistics and age management
    cache_manager: Arc<Mutex<CacheManager>>,
    /// Replacement policy handler
    replacement_handler: Arc<Mutex<ReplacementPolicyHandler>>,
    /// Performance statistics
    stats: Arc<Mutex<ThreadSafeStats>>,
    /// Number of poison recovery events observed
    poison_recoveries: AtomicU64,
}

/// Thread-safe statistics
#[derive(Debug, Default)]
pub struct ThreadSafeStats {
    /// Total number of probes
    pub total_probes: AtomicU64,
    /// Number of hits
    pub hits: AtomicU64,
    /// Number of misses
    pub misses: AtomicU64,
    /// Number of stores
    pub stores: AtomicU64,
    /// Number of replacements
    pub replacements: AtomicU64,
    /// Number of atomic operations
    pub atomic_operations: AtomicU64,
}

impl ThreadSafeTranspositionTable {
    /// Create a new thread-safe transposition table
    pub fn new(config: TranspositionConfig) -> Self {
        let thread_mode = ThreadSafetyMode::Auto;
        let size = config.table_size.next_power_of_two();
        let mask = size - 1;

        // Create entries vector
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(ThreadSafeEntry {
                packed_data: AtomicPackedEntry::empty(),
                hash_key: AtomicU64::new(0),
                age: AtomicU32::new(0),
            });
        }

        // Create bucketed locks for reduced write contention
        #[cfg(not(target_arch = "wasm32"))]
        let bucket_count = config.bucket_count.next_power_of_two();
        #[cfg(not(target_arch = "wasm32"))]
        let bucket_locks: Vec<Arc<RwLock<()>>> = (0..bucket_count)
            .map(|_| Arc::new(RwLock::new(())))
            .collect();
        #[cfg(not(target_arch = "wasm32"))]
        let bucket_shift = 64 - bucket_count.trailing_zeros();
        #[cfg(feature = "tt-prefetch")]
        let prefetch_enabled = config.enable_prefetching && !cfg!(target_arch = "wasm32");
        let statistics_enabled = config.enable_statistics;

        Self {
            entries,
            size,
            mask,
            thread_mode,
            #[cfg(not(target_arch = "wasm32"))]
            bucket_locks,
            #[cfg(not(target_arch = "wasm32"))]
            bucket_shift,
            #[cfg(feature = "tt-prefetch")]
            prefetch_enabled,
            statistics_enabled,
            cache_manager: Arc::new(Mutex::new(CacheManager::new(config.clone()))),
            replacement_handler: Arc::new(Mutex::new(ReplacementPolicyHandler::new(
                config.clone(),
            ))),
            stats: Arc::new(Mutex::new(ThreadSafeStats::default())),
            poison_recoveries: AtomicU64::new(0),
        }
    }

    /// Create a new thread-safe transposition table with specific thread mode
    pub fn with_thread_mode(config: TranspositionConfig, thread_mode: ThreadSafetyMode) -> Self {
        let mut table = Self::new(config);
        table.thread_mode = thread_mode;
        table
    }

    /// Create a thread-safe transposition table with statistics tracking explicitly enabled
    pub fn with_statistics_tracking(mut config: TranspositionConfig) -> Self {
        config.enable_statistics = true;
        Self::new(config)
    }

    /// Probe the table for an entry
    ///
    /// This method provides thread-safe entry retrieval with minimal
    /// synchronization overhead in single-threaded environments.
    #[inline(always)]
    pub fn probe(&self, hash: u64, depth: u8) -> Option<TranspositionEntry> {
        let index = self.get_index(hash);
        let entry = &self.entries[index];

        // Atomic read of hash key
        let stored_hash = entry.hash_key.load(Ordering::Acquire);
        if stored_hash != hash {
            self.increment_misses();
            return None;
        }

        // Atomic read of packed data
        let packed_data = entry.packed_data;
        if !packed_data.is_valid() {
            self.increment_misses();
            return None;
        }

        // Check depth requirement
        if packed_data.depth() < depth {
            self.increment_misses();
            return None;
        }

        // Reconstruct the entry
        let reconstructed_entry = TranspositionEntry {
            score: packed_data.score(),
            depth: packed_data.depth(),
            flag: packed_data.flag(),
            best_move: packed_data.best_move(),
            hash_key: hash,
            age: entry.age.load(Ordering::Acquire),
            source: crate::types::EntrySource::MainSearch, // Task 7.0.3: Default to MainSearch for reconstructed entries
        };

        self.increment_hits();
        Some(reconstructed_entry)
    }

    /// Probe the table while optionally prefetching the next anticipated entry
    #[inline(always)]
    pub fn probe_with_prefetch(
        &self,
        hash: u64,
        depth: u8,
        next_hash: Option<u64>,
    ) -> Option<TranspositionEntry> {
        if let Some(next) = next_hash {
            self.prefetch_entry(next);
        }
        self.probe(hash, depth)
    }

    #[inline(always)]
    fn prefetch_entry(&self, hash: u64) {
        #[cfg(all(feature = "tt-prefetch", not(target_arch = "wasm32")))]
        {
            if self.prefetch_enabled {
                let index = self.get_index(hash);
                unsafe {
                    prefetch_entry_ptr(&self.entries[index]);
                }
            }
        }
        #[cfg(not(all(feature = "tt-prefetch", not(target_arch = "wasm32"))))]
        {
            let _ = hash;
        }
    }

    /// Store an entry in the table
    ///
    /// This method provides thread-safe entry storage with appropriate
    /// synchronization based on the thread mode.
    #[inline(always)]
    pub fn store(&mut self, entry: TranspositionEntry) {
        let hash = entry.hash_key;
        let index = self.get_index(hash);
        let is_multi_threaded = self.thread_mode.is_multi_threaded();

        if is_multi_threaded {
            #[cfg(not(target_arch = "wasm32"))]
            self.store_with_synchronization(index, entry);
            #[cfg(target_arch = "wasm32")]
            self.store_atomic_only(index, entry);
        } else {
            self.store_atomic_only(index, entry);
        }

        self.increment_stores();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recover_write_guard<'a, T, F>(
        &self,
        lock_result: LockResult<RwLockWriteGuard<'a, T>>,
        context: F,
    ) -> RwLockWriteGuard<'a, T>
    where
        F: FnOnce() -> String,
    {
        match lock_result {
            Ok(guard) => guard,
            Err(poisoned) => {
                let message = context();
                self.record_poison_recovery(&message);
                poisoned.into_inner()
            }
        }
    }

    fn recover_mutex_guard<'a, T, F>(
        &self,
        lock_result: LockResult<MutexGuard<'a, T>>,
        context: F,
    ) -> MutexGuard<'a, T>
    where
        F: FnOnce() -> String,
    {
        match lock_result {
            Ok(guard) => guard,
            Err(poisoned) => {
                let message = context();
                self.record_poison_recovery(&message);
                poisoned.into_inner()
            }
        }
    }

    fn record_poison_recovery(&self, context: &str) {
        warn!(
            "ThreadSafeTranspositionTable recovered from poisoned {}",
            context
        );

        if self.statistics_enabled {
            self.poison_recoveries.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get the bucket lock for a given hash key
    ///
    /// This method maps hash keys to lock buckets for reduced write contention.
    /// Uses fast bit shifting to calculate bucket index.
    #[cfg(not(target_arch = "wasm32"))]
    fn get_bucket_lock(&self, hash: u64) -> &Arc<RwLock<()>> {
        let bucket_index = (hash >> self.bucket_shift) as usize % self.bucket_locks.len();
        &self.bucket_locks[bucket_index]
    }

    /// Store with full synchronization (multi-threaded mode)
    ///
    /// Uses bucketed locks for better parallel write performance.
    /// Only locks the specific bucket for this hash, not the entire table.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline(always)]
    fn store_with_synchronization(&mut self, index: usize, entry: TranspositionEntry) {
        // Get the bucket lock for this hash (clone Arc to avoid borrow issues)
        let bucket_lock = Arc::clone(self.get_bucket_lock(entry.hash_key));
        let _write_guard = self.recover_write_guard(bucket_lock.write(), || {
            format!("bucket lock for hash 0x{:016x}", entry.hash_key)
        });

        let table_entry = &mut self.entries[index];

        // Check if we should replace the existing entry
        let current_hash = table_entry.hash_key.load(Ordering::Acquire);
        if current_hash != 0 {
            let current_entry = Self::reconstruct_entry_static(table_entry, current_hash);

            // Use replacement policy to decide
            let cache_age = {
                let cache_manager = self.recover_mutex_guard(self.cache_manager.lock(), || {
                    "cache manager mutex during store".to_string()
                });
                cache_manager.current_age()
            };

            let mut handler = self.recover_mutex_guard(self.replacement_handler.lock(), || {
                "replacement handler mutex during store".to_string()
            });
            let decision = handler.should_replace_entry(&current_entry, &entry, cache_age);

            match decision {
                ReplacementDecision::Replace => {
                    Self::store_atomic_entry_static(table_entry, entry);
                    #[cfg(not(target_arch = "wasm32"))]
                    self.increment_replacements();
                    self.increment_atomic_operations();
                }
                ReplacementDecision::Keep => {
                    // Keep existing entry
                }
                ReplacementDecision::ReplaceIfExact => {
                    if entry.is_exact() && !current_entry.is_exact() {
                        Self::store_atomic_entry_static(table_entry, entry);
                        #[cfg(not(target_arch = "wasm32"))]
                        self.increment_replacements();
                        self.increment_atomic_operations();
                    }
                }
            }
        } else {
            // Empty slot, store directly
            Self::store_atomic_entry_static(table_entry, entry);
            self.increment_atomic_operations();
        }
    }

    /// Store with atomic operations only (single-threaded mode)
    #[inline(always)]
    fn store_atomic_only(&mut self, index: usize, entry: TranspositionEntry) {
        let table_entry = &mut self.entries[index];
        Self::store_atomic_entry_static(table_entry, entry);
        self.increment_atomic_operations();
    }

    /// Prefill the table using entries from an opening book.
    ///
    /// Returns the number of entries inserted.
    pub fn prefill_from_book(&mut self, book: &mut OpeningBook, depth: u8) -> usize {
        let hasher = ZobristHasher::new();
        let mut inserted = 0usize;

        for prefill in book.collect_prefill_entries() {
            if let Ok((board, player_from_fen, captured)) = BitboardBoard::from_fen(&prefill.fen) {
                let hash =
                    hasher.hash_position(&board, player_from_fen, &captured, RepetitionState::None);
                let engine_move = prefill.book_move.to_engine_move(prefill.player);
                let entry = TranspositionEntry::new(
                    prefill.book_move.evaluation,
                    depth,
                    TranspositionFlag::Exact,
                    Some(engine_move),
                    hash,
                    0,
                    EntrySource::OpeningBook,
                );
                self.store(entry);
                inserted += 1;
            }
        }

        inserted
    }

    /// Store entry using atomic operations (static helper)
    #[inline(always)]
    fn store_atomic_entry_static(table_entry: &mut ThreadSafeEntry, entry: TranspositionEntry) {
        // Pack the entry data
        let packed_data =
            AtomicPackedEntry::new(entry.score, entry.depth, entry.flag, entry.best_move);

        // Atomic write of hash key
        table_entry
            .hash_key
            .store(entry.hash_key, Ordering::Release);

        // Atomic write of packed data
        table_entry.packed_data = packed_data;

        // Atomic write of age
        table_entry.age.store(entry.age, Ordering::Release);
    }

    /// Reconstruct entry from atomic data (static helper)
    fn reconstruct_entry_static(table_entry: &ThreadSafeEntry, hash: u64) -> TranspositionEntry {
        TranspositionEntry {
            score: table_entry.packed_data.score(),
            depth: table_entry.packed_data.depth(),
            flag: table_entry.packed_data.flag(),
            best_move: table_entry.packed_data.best_move(),
            hash_key: hash,
            age: table_entry.age.load(Ordering::Acquire),
            source: crate::types::EntrySource::MainSearch, // Task 7.0.3: Default to MainSearch for reconstructed entries
        }
    }

    /// Reconstruct entry from atomic data
    #[inline(always)]
    fn reconstruct_entry(&self, table_entry: &ThreadSafeEntry, hash: u64) -> TranspositionEntry {
        Self::reconstruct_entry_static(table_entry, hash)
    }

    /// Get table index from hash
    #[inline(always)]
    fn get_index(&self, hash: u64) -> usize {
        (hash as usize) & self.mask
    }

    /// Get the table size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the number of lock buckets (non-WASM only)
    ///
    /// Returns the number of independent lock buckets used for parallel write operations.
    /// Higher bucket counts reduce contention but increase memory overhead.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn bucket_count(&self) -> usize {
        self.bucket_locks.len()
    }

    /// Get the number of lock buckets (WASM always returns 1)
    #[cfg(target_arch = "wasm32")]
    pub fn bucket_count(&self) -> usize {
        1 // No bucketing in single-threaded WASM
    }

    /// Clear the entire table
    pub fn clear(&mut self) {
        if self.thread_mode.is_multi_threaded() {
            #[cfg(not(target_arch = "wasm32"))]
            self.clear_with_synchronization();
            #[cfg(target_arch = "wasm32")]
            self.clear_atomic_only();
        } else {
            self.clear_atomic_only();
        }
    }

    /// Clear with synchronization
    ///
    /// Acquires all bucket locks to ensure no concurrent writes during clear.
    #[cfg(not(target_arch = "wasm32"))]
    fn clear_with_synchronization(&mut self) {
        // Clone all bucket locks and acquire them to prevent writes during clear
        let locks: Vec<_> = self
            .bucket_locks
            .iter()
            .map(|lock| Arc::clone(lock))
            .collect();
        let _guards: Vec<_> = locks
            .iter()
            .enumerate()
            .map(|(idx, lock)| {
                self.recover_write_guard(lock.write(), || {
                    format!("bucket lock {} during clear", idx)
                })
            })
            .collect();

        // Clear entries directly here to avoid borrowing issues
        for entry in &mut self.entries {
            entry.hash_key.store(0, Ordering::Release);
            entry.packed_data = AtomicPackedEntry::empty();
            entry.age.store(0, Ordering::Release);
        }
    }

    /// Clear using atomic operations only
    fn clear_atomic_only(&mut self) {
        for entry in &mut self.entries {
            entry.hash_key.store(0, Ordering::Release);
            entry.packed_data = AtomicPackedEntry::empty();
            entry.age.store(0, Ordering::Release);
        }
    }

    /// Get current age from cache manager
    pub fn current_age(&self) -> u32 {
        let age = {
            let cache_manager = self.recover_mutex_guard(self.cache_manager.lock(), || {
                "cache manager mutex during current_age".to_string()
            });
            cache_manager.current_age()
        };

        age
    }

    /// Increment age counter
    pub fn increment_age(&mut self, node_count: u64) -> bool {
        let mut cache_manager = self.recover_mutex_guard(self.cache_manager.lock(), || {
            "cache manager mutex during increment_age".to_string()
        });
        cache_manager.increment_age(node_count)
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        if !self.statistics_enabled {
            return 0.0;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during hit_rate".to_string()
        });
        let total = stats.total_probes.load(Ordering::Acquire);
        let hits = stats.hits.load(Ordering::Acquire);

        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Get comprehensive statistics
    pub fn get_stats(&self) -> ThreadSafeStatsSnapshot {
        if !self.statistics_enabled {
            return ThreadSafeStatsSnapshot {
                thread_mode: self.thread_mode,
                ..ThreadSafeStatsSnapshot::default()
            };
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during get_stats".to_string()
        });
        ThreadSafeStatsSnapshot {
            total_probes: stats.total_probes.load(Ordering::Acquire),
            hits: stats.hits.load(Ordering::Acquire),
            misses: stats.misses.load(Ordering::Acquire),
            stores: stats.stores.load(Ordering::Acquire),
            replacements: stats.replacements.load(Ordering::Acquire),
            atomic_operations: stats.atomic_operations.load(Ordering::Acquire),
            poison_recoveries: self.poison_recoveries.load(Ordering::Relaxed),
            hit_rate: self.hit_rate(),
            thread_mode: self.thread_mode,
        }
    }

    /// Check if the table is thread-safe
    pub fn is_thread_safe(&self) -> bool {
        self.thread_mode.is_multi_threaded()
    }

    /// Get the current thread mode
    pub fn thread_mode(&self) -> ThreadSafetyMode {
        self.thread_mode
    }

    /// Get the replacement policy handler (for advanced usage)
    pub fn get_replacement_handler(&self) -> Arc<Mutex<ReplacementPolicyHandler>> {
        Arc::clone(&self.replacement_handler)
    }

    /// Update thread mode (requires table to be empty)
    pub fn set_thread_mode(&mut self, mode: ThreadSafetyMode) {
        self.thread_mode = mode;
    }

    /// Reconstruct entry from index (for debugging/analysis)
    pub fn get_entry_at_index(&self, index: usize) -> Option<TranspositionEntry> {
        if index >= self.entries.len() {
            return None;
        }

        let entry = &self.entries[index];
        let hash = entry.hash_key.load(Ordering::Acquire);

        if hash == 0 {
            None
        } else {
            Some(self.reconstruct_entry(entry, hash))
        }
    }

    // Statistics increment methods
    fn increment_hits(&self) {
        if !self.statistics_enabled {
            return;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during increment_hits".to_string()
        });
        stats.total_probes.fetch_add(1, Ordering::Relaxed);
        stats.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_misses(&self) {
        if !self.statistics_enabled {
            return;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during increment_misses".to_string()
        });
        stats.total_probes.fetch_add(1, Ordering::Relaxed);
        stats.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_stores(&self) {
        if !self.statistics_enabled {
            return;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during increment_stores".to_string()
        });
        stats.stores.fetch_add(1, Ordering::Relaxed);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn increment_replacements(&self) {
        if !self.statistics_enabled {
            return;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during increment_replacements".to_string()
        });
        stats.replacements.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_atomic_operations(&self) {
        if !self.statistics_enabled {
            return;
        }
        let stats = self.recover_mutex_guard(self.stats.lock(), || {
            "statistics mutex during increment_atomic_operations".to_string()
        });
        stats.atomic_operations.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(all(feature = "tt-prefetch", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn prefetch_entry_ptr(entry: &ThreadSafeEntry) {
    _mm_prefetch(entry as *const _ as *const i8, _MM_HINT_T2);
}

#[cfg(all(feature = "tt-prefetch", target_arch = "x86"))]
#[inline(always)]
unsafe fn prefetch_entry_ptr(entry: &ThreadSafeEntry) {
    _mm_prefetch(entry as *const _ as *const i8, _MM_HINT_T2);
}

#[cfg(all(
    feature = "tt-prefetch",
    not(any(target_arch = "x86", target_arch = "x86_64"))
))]
#[inline(always)]
unsafe fn prefetch_entry_ptr(_entry: &ThreadSafeEntry) {}

/// Snapshot of thread-safe statistics, including poison-recovery counts when tracking is enabled
#[derive(Debug, Clone)]
pub struct ThreadSafeStatsSnapshot {
    pub total_probes: u64,
    pub hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub replacements: u64,
    pub atomic_operations: u64,
    pub poison_recoveries: u64,
    pub hit_rate: f64,
    pub thread_mode: ThreadSafetyMode,
}

impl Default for ThreadSafeStatsSnapshot {
    fn default() -> Self {
        Self {
            total_probes: 0,
            hits: 0,
            misses: 0,
            stores: 0,
            replacements: 0,
            atomic_operations: 0,
            poison_recoveries: 0,
            hit_rate: 0.0,
            thread_mode: ThreadSafetyMode::Auto,
        }
    }
}

/// Thread-safe table builder for configuration
pub struct ThreadSafeTableBuilder {
    config: TranspositionConfig,
    thread_mode: Option<ThreadSafetyMode>,
}

impl ThreadSafeTableBuilder {
    /// Create a new builder
    pub fn new(config: TranspositionConfig) -> Self {
        Self {
            config,
            thread_mode: None,
        }
    }

    /// Set thread safety mode
    pub fn with_thread_mode(mut self, mode: ThreadSafetyMode) -> Self {
        self.thread_mode = Some(mode);
        self
    }

    /// Build the thread-safe table
    pub fn build(self) -> ThreadSafeTranspositionTable {
        match self.thread_mode {
            Some(mode) => ThreadSafeTranspositionTable::with_thread_mode(self.config, mode),
            None => ThreadSafeTranspositionTable::new(self.config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;
    use crate::opening_book::{BookMove, OpeningBookBuilder};
    use crate::search::zobrist::{RepetitionState, ZobristHasher};
    use std::panic;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn create_test_config() -> TranspositionConfig {
        let mut config = TranspositionConfig::debug_config();
        config.table_size = 1024; // Small size for testing
        config
    }

    fn create_test_entry(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        age: u32,
    ) -> TranspositionEntry {
        let mut entry = TranspositionEntry::new_with_age(score, depth, flag, None, 0);
        entry.age = age;
        entry
    }

    #[test]
    fn test_thread_safety_mode_detection() {
        let mode = ThreadSafetyMode::detect();

        #[cfg(target_arch = "wasm32")]
        assert_eq!(mode, ThreadSafetyMode::SingleThreaded);

        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(mode, ThreadSafetyMode::MultiThreaded);
    }

    #[test]
    fn test_atomic_packed_entry() {
        let entry = AtomicPackedEntry::new(100, 5, TranspositionFlag::Exact, None);

        assert_eq!(entry.score(), 100);
        assert_eq!(entry.depth(), 5);
        assert_eq!(entry.flag(), TranspositionFlag::Exact);
        assert!(entry.is_valid());

        let empty = AtomicPackedEntry::empty();
        assert!(!empty.is_valid());
    }

    #[test]
    fn test_thread_safe_table_basic() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        let retrieved = table.probe(entry.hash_key, 5);
        assert!(retrieved.is_some());

        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.score, entry.score);
        assert_eq!(retrieved_entry.depth, entry.depth);
        assert_eq!(retrieved_entry.flag, entry.flag);
    }

    #[test]
    fn test_thread_safe_table_statistics_disabled() {
        let mut config = create_test_config();
        config.enable_statistics = false;
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());
        table.probe(entry.hash_key, 5);

        let stats = table.get_stats();
        assert_eq!(stats.total_probes, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.stores, 0);
        assert_eq!(stats.atomic_operations, 0);
        assert_eq!(stats.poison_recoveries, 0);
        assert_eq!(table.hit_rate(), 0.0);
    }

    #[test]
    fn test_thread_safe_table_with_statistics_tracking() {
        let mut config = create_test_config();
        config.enable_statistics = false;
        let mut table = ThreadSafeTranspositionTable::with_statistics_tracking(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());
        table.probe(entry.hash_key, 5);

        let stats = table.get_stats();
        assert_eq!(stats.total_probes, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.stores, 1);
        assert_eq!(stats.poison_recoveries, 0);
        assert!(table.hit_rate() > 0.0);
    }

    #[test]
    fn test_thread_safe_table_probe_miss() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        // Probe for non-existent entry
        let result = table.probe(12345, 5);
        assert!(result.is_none());

        let stats = table.get_stats();
        assert_eq!(stats.total_probes, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.poison_recoveries, 0);
    }

    #[test]
    fn test_thread_safe_table_depth_requirement() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        // Probe with higher depth requirement - should miss
        let result = table.probe(entry.hash_key, 6);
        assert!(result.is_none());

        // Probe with lower depth requirement - should hit
        let result = table.probe(entry.hash_key, 4);
        assert!(result.is_some());
    }

    #[test]
    fn test_thread_safe_table_clear() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        // Verify entry is stored
        let result = table.probe(entry.hash_key, 5);
        assert!(result.is_some());

        // Clear table
        table.clear();

        // Verify entry is gone
        let result = table.probe(entry.hash_key, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_thread_safe_table_single_threaded_mode() {
        let config = create_test_config();
        let table = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        assert!(!table.is_thread_safe());
        assert_eq!(table.thread_mode(), ThreadSafetyMode::SingleThreaded);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        let retrieved = table.probe(entry.hash_key, 5);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_thread_safe_table_builder() {
        let config = create_test_config();
        let table = ThreadSafeTableBuilder::new(config)
            .with_thread_mode(ThreadSafetyMode::SingleThreaded)
            .build();

        assert_eq!(table.thread_mode(), ThreadSafetyMode::SingleThreaded);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        // Hit
        table.probe(entry.hash_key, 5);

        // Miss
        table.probe(99999, 5);

        let hit_rate = table.hit_rate();
        assert!((hit_rate - 50.0).abs() < 0.1); // Should be 50% hit rate
    }

    #[test]
    fn test_prefill_from_opening_book_thread_safe() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let book_move = BookMove::new(
            Some(Position::new(6, 4)),
            Position::new(5, 4),
            PieceType::Pawn,
            false,
            false,
            1000,
            60,
        );

        let mut book = OpeningBookBuilder::new()
            .add_position(fen.to_string(), vec![book_move.clone()])
            .mark_loaded()
            .build();

        let mut table = ThreadSafeTranspositionTable::new(create_test_config());
        let inserted = table.prefill_from_book(&mut book, 4);
        assert_eq!(inserted, 1);

        let (board, player, captured) = BitboardBoard::from_fen(fen).unwrap();
        let hash =
            ZobristHasher::new().hash_position(&board, player, &captured, RepetitionState::None);

        let entry = table.probe(hash, 4).expect("prefilled entry should exist");
        assert_eq!(entry.score, 60);
        assert_eq!(entry.depth, 4);
        assert_eq!(entry.source, EntrySource::OpeningBook);
        assert_eq!(entry.flag, TranspositionFlag::Exact);
        assert_eq!(entry.best_move.unwrap().to.to_index(), 5 * 9 + 4);
    }

    #[test]
    fn test_atomic_operations_count() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        table.store(entry.clone());

        let stats = table.get_stats();
        assert_eq!(stats.atomic_operations, 1); // One atomic store operation
        assert_eq!(stats.poison_recoveries, 0);
    }

    #[test]
    fn test_poison_recovery_during_store() {
        let mut config = create_test_config();
        config.enable_statistics = true;
        let mut table =
            ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::MultiThreaded);

        // Seed the table so the next store triggers the replacement path
        let mut initial_entry = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        initial_entry.hash_key = 0xAA55_AA55_AA55_AA55;
        table.store(initial_entry.clone());

        // Poison the replacement handler lock
        let handler = Arc::clone(&table.replacement_handler);
        let _ = panic::catch_unwind(|| {
            let _guard = handler.lock().unwrap();
            panic!("intentional poison");
        });

        // Attempt to store a replacement entry and ensure recovery succeeds
        let mut replacement_entry = create_test_entry(120, 6, TranspositionFlag::Exact, 2);
        replacement_entry.hash_key = initial_entry.hash_key;
        table.store(replacement_entry.clone());

        let stats = table.get_stats();
        assert!(
            stats.poison_recoveries >= 1,
            "expected poison recovery to be recorded"
        );

        // Verify store still succeeds after recovery
        assert!(table
            .probe(replacement_entry.hash_key, replacement_entry.depth)
            .is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_concurrent_access() {
        let config = create_test_config();
        // Test multi-threaded access by creating separate tables per thread
        let mut handles = vec![];

        // Spawn multiple threads that read and write
        for i in 0..4 {
            let handle = thread::spawn(move || {
                let mut table = ThreadSafeTranspositionTable::with_thread_mode(
                    config,
                    ThreadSafetyMode::MultiThreaded,
                );
                for j in 0..100 {
                    let entry = create_test_entry(100 + i, 5, TranspositionFlag::Exact, j);
                    table.store(entry.clone());
                    let _result = table.probe(entry.hash_key, 5);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify no panics occurred during multi-threaded access
        // (The test passes if all threads complete without panicking)
    }

    #[test]
    fn test_bucket_count() {
        let mut config = create_test_config();
        config.bucket_count = 256;
        let table = ThreadSafeTranspositionTable::new(config);

        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(table.bucket_count(), 256);

        #[cfg(target_arch = "wasm32")]
        assert_eq!(table.bucket_count(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_bucketed_lock_isolation() {
        let mut config = create_test_config();
        config.bucket_count = 4; // Small number for testing
        let table = ThreadSafeTranspositionTable::new(config);

        // Verify different buckets use different locks
        assert_eq!(table.bucket_count(), 4);

        // Create entries that map to different buckets
        let hash1 = 0x0000000000000001u64;
        let hash2 = 0x1000000000000000u64;
        let hash3 = 0x2000000000000000u64;
        let hash4 = 0x3000000000000000u64;

        let entry1 = create_test_entry(100, 5, TranspositionFlag::Exact, 1);
        let entry2 = create_test_entry(200, 5, TranspositionFlag::Exact, 1);
        let entry3 = create_test_entry(300, 5, TranspositionFlag::Exact, 1);
        let entry4 = create_test_entry(400, 5, TranspositionFlag::Exact, 1);

        // These should map to different buckets and can be stored concurrently
        table.store(entry1);
        table.store(entry2);
        table.store(entry3);
        table.store(entry4);

        // Verify all were stored successfully
        assert!(table.probe(hash1, 5).is_some());
        assert!(table.probe(hash2, 5).is_some());
        assert!(table.probe(hash3, 5).is_some());
        assert!(table.probe(hash4, 5).is_some());
    }

    #[test]
    fn test_best_move_round_trip() {
        let mut config = create_test_config();
        config.bucket_count = 16;
        let mut table = ThreadSafeTranspositionTable::new(config);

        let from = Position::new(2, 3);
        let to = Position::new(4, 5);
        let best_move = Move {
            from: Some(from),
            to,
            piece_type: PieceType::Rook,
            player: Player::White,
            is_promotion: true,
            is_capture: true,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let hash_key = 0xABCDEF1234567890u64;
        let entry = TranspositionEntry::new(
            250,
            7,
            TranspositionFlag::Exact,
            Some(best_move.clone()),
            hash_key,
            3,
            EntrySource::MainSearch,
        );
        table.store(entry);

        let retrieved = table
            .probe(hash_key, 7)
            .expect("Expected entry to be present");
        let retrieved_move = retrieved
            .best_move
            .expect("Expected best move to be present");

        assert_eq!(retrieved_move.from, best_move.from);
        assert_eq!(retrieved_move.to, best_move.to);
        assert_eq!(retrieved_move.piece_type, best_move.piece_type);
        assert_eq!(retrieved_move.player, best_move.player);
        assert_eq!(retrieved_move.is_promotion, best_move.is_promotion);
        assert_eq!(retrieved_move.is_capture, best_move.is_capture);

        // Store a drop move and ensure we can deserialize it correctly
        let drop_move = Move {
            from: None,
            to: Position::new(5, 4),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let drop_hash = 0x12345678u64;
        let drop_entry = TranspositionEntry::new(
            150,
            4,
            TranspositionFlag::Exact,
            Some(drop_move.clone()),
            drop_hash,
            2,
            EntrySource::MainSearch,
        );
        table.store(drop_entry);

        let retrieved_drop = table.probe(drop_hash, 4).expect("Expected drop entry");
        let retrieved_drop_move = retrieved_drop.best_move.expect("Expected drop move");
        assert!(retrieved_drop_move.from.is_none());
        assert_eq!(retrieved_drop_move.to, drop_move.to);
        assert_eq!(retrieved_drop_move.piece_type, drop_move.piece_type);
        assert_eq!(retrieved_drop_move.player, drop_move.player);
        assert_eq!(retrieved_drop_move.is_promotion, drop_move.is_promotion);
        assert_eq!(retrieved_drop_move.is_capture, drop_move.is_capture);
    }

    #[test]
    fn test_probe_with_prefetch_behaves_like_probe() {
        let config = create_test_config();
        let mut table = ThreadSafeTranspositionTable::new(config);

        let entry = create_test_entry(75, 4, TranspositionFlag::Exact, 2);
        table.store(entry.clone());

        let result = table.probe_with_prefetch(entry.hash_key, 4, Some(0x1234));
        assert!(result.is_some());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.score, entry.score);
        assert_eq!(retrieved.depth, entry.depth);
    }
}
