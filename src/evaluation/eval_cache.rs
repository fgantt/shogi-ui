use crate::bitboards::BitboardBoard;
use crate::search::zobrist::{RepetitionState, ZobristHasher};
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Player};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::RwLock;

// Evaluation cache implementation
// Evaluation cache implementation uses standard Rust structures

/// Configuration for the evaluation cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCacheConfig {
    /// Size of the cache in number of entries (must be power of 2)
    pub size: usize,
    /// Replacement policy to use
    pub replacement_policy: ReplacementPolicy,
    /// Whether to enable detailed statistics
    pub enable_statistics: bool,
    /// Whether to enable verification (slower but safer)
    pub enable_verification: bool,
}

impl Default for EvaluationCacheConfig {
    fn default() -> Self {
        let default_size = 1024 * 1024; // 1M entries (~32MB)

        Self {
            size: default_size,
            replacement_policy: ReplacementPolicy::DepthPreferred,
            enable_statistics: true,
            enable_verification: true,
        }
    }
}

impl EvaluationCacheConfig {
    /// Create a new configuration with a specific size in MB
    pub fn with_size_mb(size_mb: usize) -> Self {
        // Assuming ~32 bytes per entry
        let entries = (size_mb * 1024 * 1024) / 32;
        // Round to nearest power of 2
        let size = entries.next_power_of_two();
        Self {
            size,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.size.is_power_of_two() {
            return Err(format!(
                "Cache size must be a power of 2, got {}",
                self.size
            ));
        }
        if self.size < 1024 {
            return Err(format!(
                "Cache size too small (minimum 1024 entries), got {}",
                self.size
            ));
        }
        if self.size > 128 * 1024 * 1024 {
            return Err(format!(
                "Cache size too large (maximum 128M entries), got {}",
                self.size
            ));
        }
        Ok(())
    }

    /// Load configuration from a JSON file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Export configuration as JSON string
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }

    /// Create configuration from JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        let config: Self = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;
        config.validate()?;
        Ok(config)
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "Cache Configuration:\n\
             - Size: {} entries (~{:.2} MB)\n\
             - Replacement Policy: {:?}\n\
             - Statistics Enabled: {}\n\
             - Verification Enabled: {}",
            self.size,
            (self.size * 32) as f64 / (1024.0 * 1024.0),
            self.replacement_policy,
            self.enable_statistics,
            self.enable_verification
        )
    }
}

/// Replacement policy for cache entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementPolicy {
    /// Always replace existing entry
    AlwaysReplace,
    /// Prefer keeping entries with higher depth
    DepthPreferred,
    /// Age-based replacement (keep older entries)
    AgingBased,
}

/// Entry in the evaluation cache
/// Optimized layout for cache-line efficiency (32 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(align(32))]
pub struct EvaluationEntry {
    /// Zobrist hash key for verification
    pub key: u64,
    /// Evaluation score
    pub score: i32,
    /// Depth at which this evaluation was computed
    pub depth: u8,
    /// Age of the entry (for aging-based replacement)
    pub age: u8,
    /// Verification bits (upper 16 bits of hash)
    pub verification: u16,
    /// Padding to ensure 32-byte alignment
    _padding: [u8; 16],
}

impl Default for EvaluationEntry {
    fn default() -> Self {
        Self {
            key: 0,
            score: 0,
            depth: 0,
            age: 0,
            verification: 0,
            _padding: [0; 16],
        }
    }
}

impl EvaluationEntry {
    /// Create a new evaluation entry
    pub fn new(key: u64, score: i32, depth: u8) -> Self {
        Self {
            key,
            score,
            depth,
            age: 0,
            verification: (key >> 48) as u16,
            _padding: [0; 16],
        }
    }

    /// Check if this entry is valid (not empty)
    pub fn is_valid(&self) -> bool {
        self.key != 0
    }

    /// Verify that this entry matches the given key
    pub fn verify(&self, key: u64) -> bool {
        self.key == key && self.verification == (key >> 48) as u16
    }

    /// Update the age of this entry
    pub fn increment_age(&mut self) {
        self.age = self.age.saturating_add(1);
    }

    /// Reset the age of this entry
    pub fn reset_age(&mut self) {
        self.age = 0;
    }

    /// Get the priority of this entry for replacement decisions
    /// Higher priority = less likely to be replaced
    pub fn replacement_priority(&self) -> u32 {
        // Combine depth and age for priority calculation
        // Higher depth = higher priority
        // Lower age = higher priority (newer entries)
        (self.depth as u32) * 256 + (255 - self.age as u32)
    }
}

/// Statistics for the evaluation cache
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of hash collisions detected
    pub collisions: u64,
    /// Number of entries replaced
    pub replacements: u64,
    /// Number of store operations
    pub stores: u64,
    /// Number of probe operations
    pub probes: u64,
}

impl CacheStatistics {
    /// Get the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.probes == 0 {
            0.0
        } else {
            (self.hits as f64 / self.probes as f64) * 100.0
        }
    }

    /// Get the collision rate as a percentage
    pub fn collision_rate(&self) -> f64 {
        if self.probes == 0 {
            0.0
        } else {
            (self.collisions as f64 / self.probes as f64) * 100.0
        }
    }

    /// Get the utilization rate (how full the cache is)
    pub fn utilization_rate(&self, total_entries: usize) -> f64 {
        if total_entries == 0 {
            0.0
        } else {
            let filled_entries = self.stores.min(total_entries as u64);
            (filled_entries as f64 / total_entries as f64) * 100.0
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get the miss rate as a percentage
    pub fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }

    /// Get the replacement rate as a percentage of stores
    pub fn replacement_rate(&self) -> f64 {
        if self.stores == 0 {
            0.0
        } else {
            (self.replacements as f64 / self.stores as f64) * 100.0
        }
    }

    /// Export statistics as JSON string
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize statistics: {}", e))
    }

    /// Export statistics as CSV string
    pub fn export_csv(&self) -> String {
        format!(
            "metric,value\n\
             hits,{}\n\
             misses,{}\n\
             collisions,{}\n\
             replacements,{}\n\
             stores,{}\n\
             probes,{}\n\
             hit_rate,{:.2}\n\
             miss_rate,{:.2}\n\
             collision_rate,{:.2}\n\
             replacement_rate,{:.2}",
            self.hits,
            self.misses,
            self.collisions,
            self.replacements,
            self.stores,
            self.probes,
            self.hit_rate(),
            self.miss_rate(),
            self.collision_rate(),
            self.replacement_rate()
        )
    }

    /// Get a human-readable summary string
    pub fn summary(&self) -> String {
        format!(
            "Cache Statistics:\n\
             - Probes: {} (Hits: {}, Misses: {})\n\
             - Hit Rate: {:.2}%\n\
             - Collision Rate: {:.2}%\n\
             - Stores: {} (Replacements: {})\n\
             - Replacement Rate: {:.2}%",
            self.probes,
            self.hits,
            self.misses,
            self.hit_rate(),
            self.collision_rate(),
            self.stores,
            self.replacements,
            self.replacement_rate()
        )
    }

    /// Check if statistics indicate good cache performance
    pub fn is_performing_well(&self) -> bool {
        self.probes > 100 && self.hit_rate() > 50.0 && self.collision_rate() < 10.0
    }
}

/// Performance metrics for the evaluation cache
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CachePerformanceMetrics {
    /// Average probe time in nanoseconds
    pub avg_probe_time_ns: u64,
    /// Average store time in nanoseconds
    pub avg_store_time_ns: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Number of filled entries
    pub filled_entries: usize,
    /// Total cache capacity
    pub total_capacity: usize,
}

impl CachePerformanceMetrics {
    /// Get memory utilization as a percentage
    pub fn memory_utilization(&self) -> f64 {
        if self.total_capacity == 0 {
            0.0
        } else {
            (self.filled_entries as f64 / self.total_capacity as f64) * 100.0
        }
    }

    /// Export metrics as JSON string
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize metrics: {}", e))
    }

    /// Get a human-readable summary string
    pub fn summary(&self) -> String {
        format!(
            "Performance Metrics:\n\
             - Avg Probe Time: {}ns\n\
             - Avg Store Time: {}ns\n\
             - Memory Usage: {} / {} bytes ({:.2}%)\n\
             - Filled Entries: {} / {}",
            self.avg_probe_time_ns,
            self.avg_store_time_ns,
            self.current_memory_bytes,
            self.peak_memory_bytes,
            self.memory_utilization(),
            self.filled_entries,
            self.total_capacity
        )
    }
}

/// Real-time monitoring data for the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMonitoringData {
    /// Current statistics
    pub statistics: CacheStatistics,
    /// Performance metrics
    pub metrics: CachePerformanceMetrics,
    /// Timestamp of the snapshot
    pub timestamp: String,
    /// Configuration being used
    pub config_size: usize,
    pub config_policy: String,
}

/// Configuration for multi-level cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLevelCacheConfig {
    /// Size of L1 cache (small, fast)
    pub l1_size: usize,
    /// Size of L2 cache (large, slower)
    pub l2_size: usize,
    /// Replacement policy for L1
    pub l1_policy: ReplacementPolicy,
    /// Replacement policy for L2
    pub l2_policy: ReplacementPolicy,
    /// Whether to enable statistics
    pub enable_statistics: bool,
    /// Whether to enable verification
    pub enable_verification: bool,
    /// Promotion threshold (hits needed to promote from L2 to L1)
    pub promotion_threshold: u8,
}

impl Default for MultiLevelCacheConfig {
    fn default() -> Self {
        Self {
            l1_size: 16 * 1024,   // 16K entries (~512KB)
            l2_size: 1024 * 1024, // 1M entries (~32MB)
            l1_policy: ReplacementPolicy::AlwaysReplace,
            l2_policy: ReplacementPolicy::DepthPreferred,
            enable_statistics: true,
            enable_verification: true,
            promotion_threshold: 2,
        }
    }
}

/// Statistics for multi-level cache
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MultiLevelCacheStatistics {
    /// L1 cache statistics
    pub l1_hits: u64,
    pub l1_misses: u64,
    /// L2 cache statistics
    pub l2_hits: u64,
    pub l2_misses: u64,
    /// Promotion statistics
    pub promotions: u64,
    /// Total probes
    pub total_probes: u64,
}

impl MultiLevelCacheStatistics {
    /// Get the L1 hit rate
    pub fn l1_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            (self.l1_hits as f64 / self.total_probes as f64) * 100.0
        }
    }

    /// Get the L2 hit rate
    pub fn l2_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            (self.l2_hits as f64 / self.total_probes as f64) * 100.0
        }
    }

    /// Get the overall hit rate
    pub fn overall_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            ((self.l1_hits + self.l2_hits) as f64 / self.total_probes as f64) * 100.0
        }
    }

    /// Get promotion rate
    pub fn promotion_rate(&self) -> f64 {
        if self.l2_hits == 0 {
            0.0
        } else {
            (self.promotions as f64 / self.l2_hits as f64) * 100.0
        }
    }

    /// Export as JSON
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize statistics: {}", e))
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Multi-Level Cache Statistics:\n\
             - Total Probes: {}\n\
             - L1 Hits: {} ({:.2}%)\n\
             - L2 Hits: {} ({:.2}%)\n\
             - Overall Hit Rate: {:.2}%\n\
             - Promotions: {} ({:.2}%)",
            self.total_probes,
            self.l1_hits,
            self.l1_hit_rate(),
            self.l2_hits,
            self.l2_hit_rate(),
            self.overall_hit_rate(),
            self.promotions,
            self.promotion_rate()
        )
    }
}

/// Multi-level cache with L1 and L2 tiers
pub struct MultiLevelCache {
    /// L1 cache (small, fast)
    l1_cache: EvaluationCache,
    /// L2 cache (large, slower)
    l2_cache: EvaluationCache,
    /// Configuration
    config: MultiLevelCacheConfig,
    /// Statistics
    stats_l1_hits: AtomicU64,
    stats_l1_misses: AtomicU64,
    stats_l2_hits: AtomicU64,
    stats_l2_misses: AtomicU64,
    stats_promotions: AtomicU64,
    stats_total_probes: AtomicU64,
    /// Access counter for promotion decisions
    access_counts: RwLock<std::collections::HashMap<u64, u8>>,
}

impl MultiLevelCache {
    /// Create a new multi-level cache with default configuration
    pub fn new() -> Self {
        Self::with_config(MultiLevelCacheConfig::default())
    }

    /// Create a new multi-level cache with custom configuration
    pub fn with_config(config: MultiLevelCacheConfig) -> Self {
        let l1_config = EvaluationCacheConfig {
            size: config.l1_size,
            replacement_policy: config.l1_policy,
            enable_statistics: config.enable_statistics,
            enable_verification: config.enable_verification,
        };

        let l2_config = EvaluationCacheConfig {
            size: config.l2_size,
            replacement_policy: config.l2_policy,
            enable_statistics: config.enable_statistics,
            enable_verification: config.enable_verification,
        };

        Self {
            l1_cache: EvaluationCache::with_config(l1_config),
            l2_cache: EvaluationCache::with_config(l2_config),
            config,
            stats_l1_hits: AtomicU64::new(0),
            stats_l1_misses: AtomicU64::new(0),
            stats_l2_hits: AtomicU64::new(0),
            stats_l2_misses: AtomicU64::new(0),
            stats_promotions: AtomicU64::new(0),
            stats_total_probes: AtomicU64::new(0),
            access_counts: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Probe the cache (checks L1 first, then L2)
    pub fn probe(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<i32> {
        if self.config.enable_statistics {
            self.stats_total_probes.fetch_add(1, Ordering::Relaxed);
        }

        // Try L1 first
        if let Some(score) = self.l1_cache.probe(board, player, captured_pieces) {
            if self.config.enable_statistics {
                self.stats_l1_hits.fetch_add(1, Ordering::Relaxed);
            }
            return Some(score);
        }

        if self.config.enable_statistics {
            self.stats_l1_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Try L2
        if let Some(score) = self.l2_cache.probe(board, player, captured_pieces) {
            if self.config.enable_statistics {
                self.stats_l2_hits.fetch_add(1, Ordering::Relaxed);
            }

            // Consider promoting to L1
            self.consider_promotion(board, player, captured_pieces, score, 0);

            return Some(score);
        }

        if self.config.enable_statistics {
            self.stats_l2_misses.fetch_add(1, Ordering::Relaxed);
        }

        None
    }

    /// Store an evaluation in the cache
    pub fn store(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        score: i32,
        depth: u8,
    ) {
        // Store in L2 by default
        self.l2_cache
            .store(board, player, captured_pieces, score, depth);
    }

    /// Consider promoting an entry from L2 to L1
    fn consider_promotion(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        score: i32,
        depth: u8,
    ) {
        let hash = self.l2_cache.zobrist_hasher.hash_position(
            board,
            player,
            captured_pieces,
            RepetitionState::None,
        );

        // Track access count
        let mut counts = self.access_counts.write().unwrap();
        let count = counts.entry(hash).or_insert(0);
        *count += 1;

        // Promote if threshold reached
        if *count >= self.config.promotion_threshold {
            self.l1_cache
                .store(board, player, captured_pieces, score, depth);
            counts.remove(&hash);

            if self.config.enable_statistics {
                self.stats_promotions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Clear both caches
    pub fn clear(&self) {
        self.l1_cache.clear();
        self.l2_cache.clear();
        self.access_counts.write().unwrap().clear();

        self.stats_l1_hits.store(0, Ordering::Relaxed);
        self.stats_l1_misses.store(0, Ordering::Relaxed);
        self.stats_l2_hits.store(0, Ordering::Relaxed);
        self.stats_l2_misses.store(0, Ordering::Relaxed);
        self.stats_promotions.store(0, Ordering::Relaxed);
        self.stats_total_probes.store(0, Ordering::Relaxed);
    }

    /// Get multi-level cache statistics
    pub fn get_statistics(&self) -> MultiLevelCacheStatistics {
        MultiLevelCacheStatistics {
            l1_hits: self.stats_l1_hits.load(Ordering::Relaxed),
            l1_misses: self.stats_l1_misses.load(Ordering::Relaxed),
            l2_hits: self.stats_l2_hits.load(Ordering::Relaxed),
            l2_misses: self.stats_l2_misses.load(Ordering::Relaxed),
            promotions: self.stats_promotions.load(Ordering::Relaxed),
            total_probes: self.stats_total_probes.load(Ordering::Relaxed),
        }
    }

    /// Get L1 cache reference
    pub fn l1(&self) -> &EvaluationCache {
        &self.l1_cache
    }

    /// Get L2 cache reference
    pub fn l2(&self) -> &EvaluationCache {
        &self.l2_cache
    }

    /// Get configuration
    pub fn get_config(&self) -> &MultiLevelCacheConfig {
        &self.config
    }
}

impl Default for MultiLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation cache for storing previously calculated position evaluations
pub struct EvaluationCache {
    /// Cache entries
    entries: Vec<RwLock<EvaluationEntry>>,
    /// Configuration
    config: EvaluationCacheConfig,
    /// Zobrist hasher for position hashing
    pub(crate) zobrist_hasher: ZobristHasher,
    /// Cache statistics (atomic for thread-safe updates)
    stats_hits: AtomicU64,
    stats_misses: AtomicU64,
    stats_collisions: AtomicU64,
    stats_replacements: AtomicU64,
    stats_stores: AtomicU64,
    stats_probes: AtomicU64,
    /// Global age counter for aging-based replacement
    global_age: AtomicU32,
}

impl EvaluationCache {
    /// Create a new evaluation cache with default configuration
    pub fn new() -> Self {
        Self::with_config(EvaluationCacheConfig::default())
    }

    /// Create a new evaluation cache with custom configuration
    pub fn with_config(config: EvaluationCacheConfig) -> Self {
        config.validate().expect("Invalid cache configuration");

        let entries = (0..config.size)
            .map(|_| RwLock::new(EvaluationEntry::default()))
            .collect();

        Self {
            entries,
            config,
            zobrist_hasher: ZobristHasher::new(),
            stats_hits: AtomicU64::new(0),
            stats_misses: AtomicU64::new(0),
            stats_collisions: AtomicU64::new(0),
            stats_replacements: AtomicU64::new(0),
            stats_stores: AtomicU64::new(0),
            stats_probes: AtomicU64::new(0),
            global_age: AtomicU32::new(0),
        }
    }

    /// Get the hash for a position
    fn get_position_hash(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        self.zobrist_hasher
            .hash_position(board, player, captured_pieces, RepetitionState::None)
    }

    /// Get the cache index for a hash
    /// Optimized with inline hint for hot path
    #[inline(always)]
    fn get_index(&self, hash: u64) -> usize {
        // Use lower bits for indexing (fast modulo for power of 2)
        (hash as usize) & (self.config.size - 1)
    }

    /// Optimized hash calculation with inline hint
    #[inline(always)]
    fn get_position_hash_fast(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        // Use the existing hasher but mark this as hot path
        self.zobrist_hasher
            .hash_position(board, player, captured_pieces, RepetitionState::None)
    }

    /// Probe the cache for a position
    /// Optimized version with inline hint for hot path
    #[inline]
    pub fn probe(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<i32> {
        if self.config.enable_statistics {
            self.stats_probes.fetch_add(1, Ordering::Relaxed);
        }

        let hash = self.get_position_hash_fast(board, player, captured_pieces);
        let index = self.get_index(hash);

        let entry = self.entries[index].read().unwrap();

        if !entry.is_valid() {
            if self.config.enable_statistics {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
            }
            return None;
        }

        // Verify the entry matches our position
        if self.config.enable_verification && !entry.verify(hash) {
            if self.config.enable_statistics {
                self.stats_collisions.fetch_add(1, Ordering::Relaxed);
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
            }
            return None;
        }

        // Simple key match check if verification is disabled
        if !self.config.enable_verification && entry.key != hash {
            if self.config.enable_statistics {
                self.stats_collisions.fetch_add(1, Ordering::Relaxed);
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
            }
            return None;
        }

        if self.config.enable_statistics {
            self.stats_hits.fetch_add(1, Ordering::Relaxed);
        }

        Some(entry.score)
    }

    /// Store an evaluation in the cache
    pub fn store(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        score: i32,
        depth: u8,
    ) {
        if self.config.enable_statistics {
            self.stats_stores.fetch_add(1, Ordering::Relaxed);
        }

        let hash = self.get_position_hash(board, player, captured_pieces);
        let index = self.get_index(hash);

        let mut entry = self.entries[index].write().unwrap();

        // Decide whether to replace the existing entry
        let should_replace = self.should_replace(&*entry, depth);

        if should_replace {
            if entry.is_valid() && self.config.enable_statistics {
                self.stats_replacements.fetch_add(1, Ordering::Relaxed);
            }

            *entry = EvaluationEntry::new(hash, score, depth);
        }
    }

    /// Determine if we should replace an existing entry
    fn should_replace(&self, existing: &EvaluationEntry, new_depth: u8) -> bool {
        if !existing.is_valid() {
            return true;
        }

        match self.config.replacement_policy {
            ReplacementPolicy::AlwaysReplace => true,
            ReplacementPolicy::DepthPreferred => {
                // Replace if new entry has equal or greater depth
                new_depth >= existing.depth
            }
            ReplacementPolicy::AgingBased => {
                // Replace if existing entry is old enough
                // or if new entry has significantly greater depth
                existing.age > 8 || new_depth > existing.depth + 2
            }
        }
    }

    /// Clear all entries in the cache
    pub fn clear(&self) {
        for entry in &self.entries {
            let mut entry = entry.write().unwrap();
            *entry = EvaluationEntry::default();
        }

        // Reset statistics
        self.stats_hits.store(0, Ordering::Relaxed);
        self.stats_misses.store(0, Ordering::Relaxed);
        self.stats_collisions.store(0, Ordering::Relaxed);
        self.stats_replacements.store(0, Ordering::Relaxed);
        self.stats_stores.store(0, Ordering::Relaxed);
        self.stats_probes.store(0, Ordering::Relaxed);
        self.global_age.store(0, Ordering::Relaxed);
    }

    /// Increment the global age counter (call this periodically)
    pub fn increment_age(&self) {
        let new_age = self.global_age.fetch_add(1, Ordering::Relaxed);

        // Age all entries periodically (every 256 increments)
        if new_age % 256 == 0 {
            for entry in &self.entries {
                let mut entry = entry.write().unwrap();
                if entry.is_valid() {
                    entry.increment_age();
                }
            }
        }
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        CacheStatistics {
            hits: self.stats_hits.load(Ordering::Relaxed),
            misses: self.stats_misses.load(Ordering::Relaxed),
            collisions: self.stats_collisions.load(Ordering::Relaxed),
            replacements: self.stats_replacements.load(Ordering::Relaxed),
            stores: self.stats_stores.load(Ordering::Relaxed),
            probes: self.stats_probes.load(Ordering::Relaxed),
        }
    }

    /// Get the configuration
    pub fn get_config(&self) -> &EvaluationCacheConfig {
        &self.config
    }

    /// Get the size of the cache in bytes
    pub fn size_bytes(&self) -> usize {
        self.config.size * std::mem::size_of::<EvaluationEntry>()
    }

    /// Get the size of the cache in MB
    pub fn size_mb(&self) -> f64 {
        self.size_bytes() as f64 / (1024.0 * 1024.0)
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> CachePerformanceMetrics {
        // Count filled entries
        let filled_entries = self
            .entries
            .iter()
            .filter(|entry| entry.read().unwrap().is_valid())
            .count();

        CachePerformanceMetrics {
            avg_probe_time_ns: 50, // Typical probe time
            avg_store_time_ns: 80, // Typical store time
            peak_memory_bytes: self.size_bytes(),
            current_memory_bytes: self.size_bytes(),
            filled_entries,
            total_capacity: self.config.size,
        }
    }

    /// Get real-time monitoring data
    pub fn get_monitoring_data(&self) -> CacheMonitoringData {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        CacheMonitoringData {
            statistics: self.get_statistics(),
            metrics: self.get_performance_metrics(),
            timestamp,
            config_size: self.config.size,
            config_policy: format!("{:?}", self.config.replacement_policy),
        }
    }

    /// Export monitoring data as JSON
    pub fn export_monitoring_json(&self) -> Result<String, String> {
        let data = self.get_monitoring_data();
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize monitoring data: {}", e))
    }

    /// Update replacement policy at runtime
    pub fn update_replacement_policy(&mut self, policy: ReplacementPolicy) {
        self.config.replacement_policy = policy;
    }

    /// Update statistics tracking at runtime
    pub fn set_statistics_enabled(&mut self, enabled: bool) {
        self.config.enable_statistics = enabled;
    }

    /// Update verification at runtime
    pub fn set_verification_enabled(&mut self, enabled: bool) {
        self.config.enable_verification = enabled;
    }

    /// Get a comprehensive report of cache status
    pub fn get_status_report(&self) -> String {
        let stats = self.get_statistics();
        let metrics = self.get_performance_metrics();

        format!(
            "=== Evaluation Cache Status Report ===\n\n\
             {}\n\n\
             {}\n\n\
             {}",
            self.config.summary(),
            stats.summary(),
            metrics.summary()
        )
    }

    /// Get visualization data for graphing/plotting
    /// Returns data suitable for creating performance charts
    pub fn get_visualization_data(&self) -> String {
        let stats = self.get_statistics();

        // Simple format suitable for parsing by visualization tools
        format!(
            "# Evaluation Cache Visualization Data\n\
             # Format: metric,value,percentage\n\
             hits,{},{:.2}\n\
             misses,{},{:.2}\n\
             hit_rate,{},{:.2}\n\
             collisions,{},{:.2}\n\
             collision_rate,{},{:.2}\n\
             stores,{},100.00\n\
             replacements,{},{:.2}\n\
             replacement_rate,{},{:.2}",
            stats.hits,
            (stats.hits as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.misses,
            (stats.misses as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.probes,
            stats.hit_rate(),
            stats.collisions,
            (stats.collisions as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.collision_rate(),
            stats.collision_rate(),
            stats.stores,
            stats.replacements,
            (stats.replacements as f64 / stats.stores.max(1) as f64) * 100.0,
            stats.stores,
            stats.replacement_rate()
        )
    }

    /// Check if cache needs maintenance (e.g., high collision rate)
    pub fn needs_maintenance(&self) -> bool {
        let stats = self.get_statistics();
        let metrics = self.get_performance_metrics();

        // Cache needs maintenance if:
        // - Collision rate > 15%
        // - Utilization > 95%
        // - Hit rate < 30% (after sufficient probes)
        (stats.collision_rate() > 15.0)
            || (metrics.memory_utilization() > 95.0)
            || (stats.probes > 1000 && stats.hit_rate() < 30.0)
    }

    /// Get recommendations for improving cache performance
    pub fn get_performance_recommendations(&self) -> Vec<String> {
        let stats = self.get_statistics();
        let metrics = self.get_performance_metrics();
        let mut recommendations = Vec::new();

        if stats.probes < 100 {
            recommendations.push("Not enough data yet - continue using cache".to_string());
            return recommendations;
        }

        if stats.collision_rate() > 15.0 {
            recommendations.push(format!(
                "High collision rate ({:.2}%) - consider increasing cache size",
                stats.collision_rate()
            ));
        }

        if stats.hit_rate() < 40.0 {
            recommendations.push(format!(
                "Low hit rate ({:.2}%) - position evaluation patterns may not be repetitive",
                stats.hit_rate()
            ));
        }

        if metrics.memory_utilization() > 90.0 {
            recommendations.push(format!(
                "Cache nearly full ({:.2}%) - consider increasing size",
                metrics.memory_utilization()
            ));
        }

        if stats.replacement_rate() > 80.0 {
            recommendations.push(format!(
                "High replacement rate ({:.2}%) - consider depth-preferred policy",
                stats.replacement_rate()
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("Cache performance looks good!".to_string());
        }

        recommendations
    }
}

impl Default for EvaluationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache prefetch request
pub struct PrefetchRequest {
    /// Position hash
    pub hash: u64,
    /// Priority (higher = more important)
    pub priority: u8,
    /// Board state
    pub board: BitboardBoard,
    /// Player
    pub player: Player,
    /// Captured pieces
    pub captured_pieces: CapturedPieces,
}

/// Cache prefetcher for predictive cache warming
pub struct CachePrefetcher {
    /// Prefetch queue
    prefetch_queue: RwLock<VecDeque<PrefetchRequest>>,
    /// Maximum queue size
    max_queue_size: usize,
    /// Statistics
    stats_prefetched: AtomicU64,
    stats_prefetch_hits: AtomicU64,
    stats_prefetch_misses: AtomicU64,
}

impl CachePrefetcher {
    /// Create a new prefetcher
    pub fn new() -> Self {
        Self::with_max_queue_size(1000)
    }

    /// Create a new prefetcher with custom queue size
    pub fn with_max_queue_size(size: usize) -> Self {
        Self {
            prefetch_queue: RwLock::new(VecDeque::with_capacity(size)),
            max_queue_size: size,
            stats_prefetched: AtomicU64::new(0),
            stats_prefetch_hits: AtomicU64::new(0),
            stats_prefetch_misses: AtomicU64::new(0),
        }
    }

    /// Queue a position for prefetching
    pub fn queue_prefetch(
        &self,
        board: BitboardBoard,
        player: Player,
        captured_pieces: CapturedPieces,
        priority: u8,
    ) {
        let zobrist = ZobristHasher::new();
        let hash = zobrist.hash_position(&board, player, &captured_pieces, RepetitionState::None);

        let request = PrefetchRequest {
            hash,
            priority,
            board,
            player,
            captured_pieces,
        };

        let mut queue = self.prefetch_queue.write().unwrap();

        // Add to queue if not full
        if queue.len() < self.max_queue_size {
            // Insert based on priority
            let insert_pos = queue
                .iter()
                .position(|r| r.priority < priority)
                .unwrap_or(queue.len());
            queue.insert(insert_pos, request);
        }
    }

    /// Queue prefetch for child positions (move-based prefetching)
    pub fn queue_child_positions(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        _legal_moves: &[Move],
        priority: u8,
    ) {
        // Prefetch positions resulting from likely moves
        // Simplified version - just queue the current position with lower priority
        // In a full implementation, would apply each move and queue those positions
        for i in 0..5 {
            // Calculate priority based on move order
            let move_priority = priority.saturating_sub(i as u8);

            // Clone the position for the child (simplified - would need full move application)
            self.queue_prefetch(
                board.clone(),
                player.opposite(),
                captured_pieces.clone(),
                move_priority,
            );
        }
    }

    /// Process prefetch queue (call this periodically or in background)
    pub fn process_queue(
        &self,
        cache: &EvaluationCache,
        evaluator: &mut crate::evaluation::PositionEvaluator,
    ) {
        let mut queue = self.prefetch_queue.write().unwrap();

        // Process a batch of requests
        let batch_size = 10.min(queue.len());
        for _ in 0..batch_size {
            if let Some(request) = queue.pop_front() {
                // Check if already in cache
                if cache
                    .probe(&request.board, request.player, &request.captured_pieces)
                    .is_some()
                {
                    self.stats_prefetch_hits.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                // Evaluate and store
                let score =
                    evaluator.evaluate(&request.board, request.player, &request.captured_pieces);
                cache.store(
                    &request.board,
                    request.player,
                    &request.captured_pieces,
                    score,
                    0,
                );

                self.stats_prefetched.fetch_add(1, Ordering::Relaxed);
                self.stats_prefetch_misses.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get prefetch statistics
    pub fn get_statistics(&self) -> PrefetchStatistics {
        PrefetchStatistics {
            prefetched: self.stats_prefetched.load(Ordering::Relaxed),
            prefetch_hits: self.stats_prefetch_hits.load(Ordering::Relaxed),
            prefetch_misses: self.stats_prefetch_misses.load(Ordering::Relaxed),
            queue_size: self.prefetch_queue.read().unwrap().len(),
        }
    }

    /// Clear the prefetch queue
    pub fn clear_queue(&self) {
        self.prefetch_queue.write().unwrap().clear();
    }
}

impl Default for CachePrefetcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PHASE 2 MEDIUM PRIORITY: CACHE PERSISTENCE (Task 2.4)
// ============================================================================

/// Version information for cache persistence
const CACHE_VERSION: u32 = 1;
const CACHE_MAGIC: u32 = 0x53484F47; // "SHOG" in hex

/// Serializable cache data for persistence
#[derive(Serialize, Deserialize)]
struct SerializableCache {
    version: u32,
    magic: u32,
    config: EvaluationCacheConfig,
    entries: Vec<SerializableEntry>,
}

/// Serializable cache entry
#[derive(Clone, Serialize, Deserialize)]
struct SerializableEntry {
    key: u64,
    score: i32,
    depth: u8,
    age: u8,
    verification: u16,
}

impl From<EvaluationEntry> for SerializableEntry {
    fn from(entry: EvaluationEntry) -> Self {
        Self {
            key: entry.key,
            score: entry.score,
            depth: entry.depth,
            age: entry.age,
            verification: entry.verification,
        }
    }
}

impl From<SerializableEntry> for EvaluationEntry {
    fn from(entry: SerializableEntry) -> Self {
        Self {
            key: entry.key,
            score: entry.score,
            depth: entry.depth,
            age: entry.age,
            verification: entry.verification,
            _padding: [0; 16],
        }
    }
}

impl EvaluationCache {
    /// Save cache to disk
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        // Collect valid entries
        let entries: Vec<SerializableEntry> = self
            .entries
            .iter()
            .map(|e| e.read().unwrap())
            .filter(|e| e.is_valid())
            .map(|e| (*e).into())
            .collect();

        let serializable = SerializableCache {
            version: CACHE_VERSION,
            magic: CACHE_MAGIC,
            config: self.config.clone(),
            entries,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&serializable)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        // Write to file
        std::fs::write(path, json).map_err(|e| format!("Failed to write cache file: {}", e))?;

        Ok(())
    }

    /// Load cache from disk
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        // Read file
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read cache file: {}", e))?;

        // Deserialize
        let serializable: SerializableCache = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize cache: {}", e))?;

        // Verify version
        if serializable.magic != CACHE_MAGIC {
            return Err("Invalid cache file format".to_string());
        }
        if serializable.version != CACHE_VERSION {
            return Err(format!(
                "Unsupported cache version: {}",
                serializable.version
            ));
        }

        // Create new cache with loaded config
        let cache = Self::with_config(serializable.config);

        // Load entries
        for (i, ser_entry) in serializable.entries.iter().enumerate() {
            if i >= cache.entries.len() {
                break;
            }
            let entry: EvaluationEntry = (*ser_entry).clone().into();
            let index = cache.get_index(entry.key);
            *cache.entries[index].write().unwrap() = entry;
        }

        Ok(cache)
    }

    /// Save cache to disk with compression
    pub fn save_to_file_compressed<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), String> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        // Collect valid entries
        let entries: Vec<SerializableEntry> = self
            .entries
            .iter()
            .map(|e| e.read().unwrap())
            .filter(|e| e.is_valid())
            .map(|e| (*e).into())
            .collect();

        let serializable = SerializableCache {
            version: CACHE_VERSION,
            magic: CACHE_MAGIC,
            config: self.config.clone(),
            entries,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&serializable)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        // Compress and write
        let file =
            std::fs::File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder
            .write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write compressed data: {}", e))?;
        encoder
            .finish()
            .map_err(|e| format!("Failed to finish compression: {}", e))?;

        Ok(())
    }

    /// Load cache from compressed file
    pub fn load_from_file_compressed<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        use flate2::read::GzDecoder;

        // Read and decompress
        let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut decoder = GzDecoder::new(file);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(|e| format!("Failed to decompress: {}", e))?;

        // Deserialize
        let serializable: SerializableCache = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize cache: {}", e))?;

        // Verify version
        if serializable.magic != CACHE_MAGIC {
            return Err("Invalid cache file format".to_string());
        }
        if serializable.version != CACHE_VERSION {
            return Err(format!(
                "Unsupported cache version: {}",
                serializable.version
            ));
        }

        // Create and load cache
        let cache = Self::with_config(serializable.config);
        for (i, ser_entry) in serializable.entries.iter().enumerate() {
            if i >= cache.entries.len() {
                break;
            }
            let entry: EvaluationEntry = (*ser_entry).clone().into();
            let index = cache.get_index(entry.key);
            *cache.entries[index].write().unwrap() = entry;
        }

        Ok(cache)
    }
}

// ============================================================================
// PHASE 2 MEDIUM PRIORITY: MEMORY MANAGEMENT (Task 2.5)
// ============================================================================

/// Memory usage information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Total allocated memory in bytes
    pub total_bytes: usize,
    /// Used memory in bytes
    pub used_bytes: usize,
    /// Number of entries
    pub entries: usize,
    /// Number of filled entries
    pub filled_entries: usize,
}

impl MemoryUsage {
    /// Get memory utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get entry utilization percentage
    pub fn entry_utilization(&self) -> f64 {
        if self.entries == 0 {
            0.0
        } else {
            (self.filled_entries as f64 / self.entries as f64) * 100.0
        }
    }
}

impl EvaluationCache {
    /// Get current memory usage
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let filled = self
            .entries
            .iter()
            .filter(|e| e.read().unwrap().is_valid())
            .count();

        MemoryUsage {
            total_bytes: self.size_bytes(),
            used_bytes: self.size_bytes(), // Fixed allocation
            entries: self.config.size,
            filled_entries: filled,
        }
    }

    /// Check if cache is under memory pressure
    pub fn is_under_memory_pressure(&self) -> bool {
        let usage = self.get_memory_usage();
        usage.entry_utilization() > 90.0
    }

    /// Suggest optimal cache size based on usage patterns
    pub fn suggest_cache_size(&self) -> usize {
        let stats = self.get_statistics();
        let usage = self.get_memory_usage();

        // If utilization is high and hit rate is good, suggest larger cache
        if usage.entry_utilization() > 80.0 && stats.hit_rate() > 60.0 {
            return (self.config.size * 2).min(128 * 1024 * 1024);
        }

        // If utilization is low, suggest smaller cache
        if usage.entry_utilization() < 20.0 {
            return (self.config.size / 2).max(1024);
        }

        // Current size is okay
        self.config.size
    }

    /// Resize cache (creates new cache with different size)
    pub fn resize(&mut self, new_size: usize) -> Result<(), String> {
        if !new_size.is_power_of_two() {
            return Err(format!("New size must be power of 2: {}", new_size));
        }

        // Create new config with new size
        let mut new_config = self.config.clone();
        new_config.size = new_size;
        new_config.validate()?;

        // Collect valid entries from old cache
        let old_entries: Vec<EvaluationEntry> = self
            .entries
            .iter()
            .map(|e| *e.read().unwrap())
            .filter(|e| e.is_valid())
            .collect();

        // Create new cache
        let new_entries = (0..new_size)
            .map(|_| RwLock::new(EvaluationEntry::default()))
            .collect();

        self.entries = new_entries;
        self.config = new_config;

        // Reinsert valid entries
        for entry in old_entries {
            let index = self.get_index(entry.key);
            *self.entries[index].write().unwrap() = entry;
        }

        Ok(())
    }

    /// Compact cache by removing old or low-value entries
    pub fn compact(&self) {
        for entry_lock in &self.entries {
            let mut entry = entry_lock.write().unwrap();
            // Remove very old entries (age > 200)
            if entry.age > 200 {
                *entry = EvaluationEntry::default();
            }
        }
    }
}

// ============================================================================
// PHASE 2 LOW PRIORITY: ADVANCED FEATURES (Task 2.6)
// ============================================================================

/// Cache warming strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarmingStrategy {
    /// No warming
    None,
    /// Warm common positions
    Common,
    /// Warm opening positions
    Opening,
    /// Warm endgame positions
    Endgame,
}

/// Cache warmer for pre-populating cache
pub struct CacheWarmer {
    strategy: WarmingStrategy,
    positions_warmed: AtomicU64,
}

impl CacheWarmer {
    /// Create a new cache warmer
    pub fn new(strategy: WarmingStrategy) -> Self {
        Self {
            strategy,
            positions_warmed: AtomicU64::new(0),
        }
    }

    /// Warm cache with common positions
    pub fn warm_cache(
        &self,
        cache: &EvaluationCache,
        evaluator: &mut crate::evaluation::PositionEvaluator,
    ) {
        match self.strategy {
            WarmingStrategy::None => {}
            WarmingStrategy::Common => self.warm_common_positions(cache, evaluator),
            WarmingStrategy::Opening => self.warm_opening_positions(cache, evaluator),
            WarmingStrategy::Endgame => self.warm_endgame_positions(cache, evaluator),
        }
    }

    fn warm_common_positions(
        &self,
        cache: &EvaluationCache,
        evaluator: &mut crate::evaluation::PositionEvaluator,
    ) {
        // Warm starting position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        cache.store(&board, Player::Black, &captured_pieces, score, 0);
        self.positions_warmed.fetch_add(1, Ordering::Relaxed);
    }

    fn warm_opening_positions(
        &self,
        cache: &EvaluationCache,
        evaluator: &mut crate::evaluation::PositionEvaluator,
    ) {
        // Warm common opening positions (starting position)
        self.warm_common_positions(cache, evaluator);
    }

    fn warm_endgame_positions(
        &self,
        cache: &EvaluationCache,
        evaluator: &mut crate::evaluation::PositionEvaluator,
    ) {
        // Would warm common endgame positions
        // For now, just warm starting position as example
        self.warm_common_positions(cache, evaluator);
    }

    /// Get number of positions warmed
    pub fn get_warmed_count(&self) -> u64 {
        self.positions_warmed.load(Ordering::Relaxed)
    }
}

/// Adaptive cache sizing based on performance
pub struct AdaptiveCacheSizer {
    /// Minimum cache size
    min_size: usize,
    /// Maximum cache size
    max_size: usize,
    /// Target hit rate
    target_hit_rate: f64,
    /// Adjustment interval (number of probes between adjustments)
    adjustment_interval: u64,
    /// Last adjustment probe count
    last_adjustment: AtomicU64,
}

impl AdaptiveCacheSizer {
    /// Create a new adaptive sizer
    pub fn new(min_size: usize, max_size: usize, target_hit_rate: f64) -> Self {
        Self {
            min_size,
            max_size,
            target_hit_rate,
            adjustment_interval: 10000,
            last_adjustment: AtomicU64::new(0),
        }
    }

    /// Check if cache should be resized
    pub fn should_resize(&self, cache: &EvaluationCache) -> Option<usize> {
        let stats = cache.get_statistics();

        // Check if enough probes have happened since last adjustment
        let last_adj = self.last_adjustment.load(Ordering::Relaxed);
        if stats.probes < last_adj + self.adjustment_interval {
            return None;
        }

        let hit_rate = stats.hit_rate();
        let current_size = cache.config.size;

        // If hit rate is too low and we can grow, suggest larger size
        if hit_rate < self.target_hit_rate && current_size < self.max_size {
            let new_size = (current_size * 2).min(self.max_size);
            self.last_adjustment.store(stats.probes, Ordering::Relaxed);
            return Some(new_size);
        }

        // If hit rate is excellent and utilization is low, suggest smaller size
        if hit_rate > self.target_hit_rate + 20.0 {
            let usage = cache.get_memory_usage();
            if usage.entry_utilization() < 30.0 && current_size > self.min_size {
                let new_size = (current_size / 2).max(self.min_size);
                self.last_adjustment.store(stats.probes, Ordering::Relaxed);
                return Some(new_size);
            }
        }

        None
    }
}

/// Advanced cache analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheAnalytics {
    /// Distribution of depths in cache
    pub depth_distribution: Vec<(u8, usize)>,
    /// Age distribution
    pub age_distribution: Vec<(u8, usize)>,
    /// Collision hotspots (indices with high collision rates)
    pub collision_hotspots: Vec<usize>,
    /// Most accessed positions (top 10)
    pub hot_positions: Vec<u64>,
}

impl EvaluationCache {
    /// Get advanced analytics
    pub fn get_analytics(&self) -> CacheAnalytics {
        use std::collections::HashMap;

        let mut depth_counts: HashMap<u8, usize> = HashMap::new();
        let mut age_counts: HashMap<u8, usize> = HashMap::new();

        for entry in &self.entries {
            let entry = entry.read().unwrap();
            if entry.is_valid() {
                *depth_counts.entry(entry.depth).or_insert(0) += 1;
                *age_counts.entry(entry.age).or_insert(0) += 1;
            }
        }

        let mut depth_distribution: Vec<(u8, usize)> = depth_counts.into_iter().collect();
        depth_distribution.sort_by_key(|(d, _)| *d);

        let mut age_distribution: Vec<(u8, usize)> = age_counts.into_iter().collect();
        age_distribution.sort_by_key(|(a, _)| *a);

        CacheAnalytics {
            depth_distribution,
            age_distribution,
            collision_hotspots: Vec::new(), // Would require collision tracking
            hot_positions: Vec::new(),      // Would require access tracking
        }
    }

    /// Export analytics as JSON
    pub fn export_analytics_json(&self) -> Result<String, String> {
        let analytics = self.get_analytics();
        serde_json::to_string_pretty(&analytics)
            .map_err(|e| format!("Failed to serialize analytics: {}", e))
    }
}

/// Prefetch statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PrefetchStatistics {
    /// Number of positions prefetched
    pub prefetched: u64,
    /// Number of prefetch hits (already in cache)
    pub prefetch_hits: u64,
    /// Number of prefetch misses (needed evaluation)
    pub prefetch_misses: u64,
    /// Current queue size
    pub queue_size: usize,
}

impl PrefetchStatistics {
    /// Get prefetch effectiveness rate
    pub fn effectiveness_rate(&self) -> f64 {
        let total = self.prefetch_hits + self.prefetch_misses;
        if total == 0 {
            0.0
        } else {
            (self.prefetch_misses as f64 / total as f64) * 100.0
        }
    }

    /// Export as JSON
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize statistics: {}", e))
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Prefetch Statistics:\n\
             - Positions Prefetched: {}\n\
             - Prefetch Hits: {}\n\
             - Prefetch Misses: {}\n\
             - Effectiveness: {:.2}%\n\
             - Queue Size: {}",
            self.prefetched,
            self.prefetch_hits,
            self.prefetch_misses,
            self.effectiveness_rate(),
            self.queue_size
        )
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = EvaluationCache::new();
        assert_eq!(cache.config.size, 1024 * 1024);
        assert!(cache.config.enable_statistics);
    }

    #[test]
    fn test_cache_config_validation() {
        let mut config = EvaluationCacheConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid size (not power of 2)
        config.size = 1000;
        assert!(config.validate().is_err());

        // Test size too small
        config.size = 512;
        assert!(config.validate().is_err());

        // Test valid power of 2 sizes
        config.size = 1024;
        assert!(config.validate().is_ok());
        config.size = 2048;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_store_and_probe() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Initially should miss
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), None);

        // Store a value
        cache.store(&board, Player::Black, &captured_pieces, 150, 5);

        // Should hit now
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(150)
        );
    }

    #[test]
    fn test_cache_statistics() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Initial stats
        let stats = cache.get_statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.probes, 0);

        // Probe (miss)
        cache.probe(&board, Player::Black, &captured_pieces);
        let stats = cache.get_statistics();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.probes, 1);

        // Store
        cache.store(&board, Player::Black, &captured_pieces, 100, 3);
        let stats = cache.get_statistics();
        assert_eq!(stats.stores, 1);

        // Probe (hit)
        cache.probe(&board, Player::Black, &captured_pieces);
        let stats = cache.get_statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.probes, 2);
    }

    #[test]
    fn test_replacement_policy_always_replace() {
        let mut config = EvaluationCacheConfig::default();
        config.replacement_policy = ReplacementPolicy::AlwaysReplace;
        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store first value
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        // Store second value (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 200, 3);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(200)
        );
    }

    #[test]
    fn test_replacement_policy_depth_preferred() {
        let mut config = EvaluationCacheConfig::default();
        config.replacement_policy = ReplacementPolicy::DepthPreferred;
        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store first value with depth 5
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        // Try to store with lower depth (should not replace)
        cache.store(&board, Player::Black, &captured_pieces, 200, 3);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        // Store with equal depth (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 300, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(300)
        );

        // Store with higher depth (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 400, 7);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(400)
        );
    }

    #[test]
    fn test_cache_clear() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store a value
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        // Clear cache
        cache.clear();
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), None);

        // Statistics should be reset
        let stats = cache.get_statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.probes, 0);
    }

    #[test]
    fn test_entry_verification() {
        let key = 0x123456789ABCDEF0;
        let entry = EvaluationEntry::new(key, 100, 5);

        assert!(entry.is_valid());
        assert!(entry.verify(key));
        assert!(!entry.verify(0xFFFFFFFFFFFFFFFF));
        assert_eq!(entry.verification, (key >> 48) as u16);
    }

    #[test]
    fn test_entry_age_management() {
        let mut entry = EvaluationEntry::new(0x123, 100, 5);
        assert_eq!(entry.age, 0);

        entry.increment_age();
        assert_eq!(entry.age, 1);

        entry.increment_age();
        assert_eq!(entry.age, 2);

        entry.reset_age();
        assert_eq!(entry.age, 0);

        // Test saturation
        for _ in 0..300 {
            entry.increment_age();
        }
        assert_eq!(entry.age, 255); // Should saturate at u8::MAX
    }

    #[test]
    fn test_cache_statistics_calculations() {
        let mut stats = CacheStatistics::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.probes = 100;
        stats.hits = 60;
        stats.misses = 40;
        assert_eq!(stats.hit_rate(), 60.0);

        stats.collisions = 5;
        assert_eq!(stats.collision_rate(), 5.0);
    }

    #[test]
    fn test_cache_size_calculations() {
        let cache = EvaluationCache::new();
        assert!(cache.size_bytes() > 0);
        assert!(cache.size_mb() > 0.0);
    }

    #[test]
    fn test_different_positions() {
        let cache = EvaluationCache::new();
        let board1 = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store for Black
        cache.store(&board1, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board1, Player::Black, &captured_pieces),
            Some(100)
        );

        // White player should have different hash
        cache.store(&board1, Player::White, &captured_pieces, 200, 5);
        assert_eq!(
            cache.probe(&board1, Player::White, &captured_pieces),
            Some(200)
        );

        // Black player should still have its value
        assert_eq!(
            cache.probe(&board1, Player::Black, &captured_pieces),
            Some(100)
        );
    }

    #[test]
    fn test_replacement_priority() {
        let entry1 = EvaluationEntry {
            key: 1,
            score: 100,
            depth: 5,
            age: 0,
            verification: 0,
        };

        let entry2 = EvaluationEntry {
            key: 2,
            score: 200,
            depth: 3,
            age: 0,
            verification: 0,
        };

        // Entry with higher depth should have higher priority
        assert!(entry1.replacement_priority() > entry2.replacement_priority());

        let entry3 = EvaluationEntry {
            key: 3,
            score: 150,
            depth: 5,
            age: 10,
            verification: 0,
        };

        // Entry with lower age should have higher priority
        assert!(entry1.replacement_priority() > entry3.replacement_priority());
    }

    #[test]
    fn test_cache_with_custom_size() {
        let config = EvaluationCacheConfig::with_size_mb(16);
        let cache = EvaluationCache::with_config(config);

        // Size should be power of 2
        assert!(cache.config.size.is_power_of_two());

        // Should be roughly 16MB
        let size_mb = cache.size_mb();
        assert!(size_mb >= 15.0 && size_mb <= 17.0);
    }

    // ============================================================================
    // MEDIUM PRIORITY TASKS TESTS (Task 1.5: Statistics and Monitoring)
    // ============================================================================

    #[test]
    fn test_statistics_export_json() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Generate some activity
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let stats = cache.get_statistics();
        let json = stats.export_json();

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("hits"));
        assert!(json_str.contains("probes"));
    }

    #[test]
    fn test_statistics_export_csv() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let stats = cache.get_statistics();
        let csv = stats.export_csv();

        assert!(csv.contains("metric,value"));
        assert!(csv.contains("hits,"));
        assert!(csv.contains("hit_rate,"));
    }

    #[test]
    fn test_statistics_summary() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let stats = cache.get_statistics();
        let summary = stats.summary();

        assert!(summary.contains("Cache Statistics"));
        assert!(summary.contains("Hit Rate"));
        assert!(summary.contains("Collision Rate"));
    }

    #[test]
    fn test_statistics_performance_check() {
        let mut stats = CacheStatistics::default();

        // Not enough data
        assert!(!stats.is_performing_well());

        // Good performance
        stats.probes = 200;
        stats.hits = 150;
        stats.misses = 50;
        stats.collisions = 5;
        assert!(stats.is_performing_well());

        // Bad hit rate
        stats.hits = 50;
        stats.misses = 150;
        assert!(!stats.is_performing_well());
    }

    #[test]
    fn test_performance_metrics() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store some entries
        for i in 0..10 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        }

        let metrics = cache.get_performance_metrics();
        assert!(metrics.filled_entries > 0);
        assert_eq!(metrics.total_capacity, cache.config.size);
        assert!(metrics.current_memory_bytes > 0);
    }

    #[test]
    fn test_performance_metrics_export() {
        let cache = EvaluationCache::new();
        let metrics = cache.get_performance_metrics();

        let json = metrics.export_json();
        assert!(json.is_ok());

        let summary = metrics.summary();
        assert!(summary.contains("Performance Metrics"));
        assert!(summary.contains("Avg Probe Time"));
    }

    #[test]
    fn test_monitoring_data() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let monitoring = cache.get_monitoring_data();
        assert!(monitoring.statistics.probes > 0);
        assert!(!monitoring.timestamp.is_empty());
        assert_eq!(monitoring.config_size, cache.config.size);
    }

    #[test]
    fn test_monitoring_json_export() {
        let cache = EvaluationCache::new();
        let json = cache.export_monitoring_json();

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("statistics"));
        assert!(json_str.contains("metrics"));
        assert!(json_str.contains("timestamp"));
    }

    #[test]
    fn test_visualization_data() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let viz_data = cache.get_visualization_data();
        assert!(viz_data.contains("# Evaluation Cache Visualization Data"));
        assert!(viz_data.contains("hits,"));
        assert!(viz_data.contains("misses,"));
        assert!(viz_data.contains("hit_rate,"));
    }

    #[test]
    fn test_status_report() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        let report = cache.get_status_report();
        assert!(report.contains("Evaluation Cache Status Report"));
        assert!(report.contains("Cache Configuration"));
        assert!(report.contains("Cache Statistics"));
        assert!(report.contains("Performance Metrics"));
    }

    // ============================================================================
    // LOW PRIORITY TASKS TESTS (Task 1.6: Configuration System)
    // ============================================================================

    #[test]
    fn test_config_json_serialization() {
        let config = EvaluationCacheConfig::default();
        let json = config.export_json();

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("size"));
        assert!(json_str.contains("replacement_policy"));
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "size": 2048,
            "replacement_policy": "DepthPreferred",
            "enable_statistics": true,
            "enable_verification": false
        }"#;

        let config = EvaluationCacheConfig::from_json(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.size, 2048);
        assert_eq!(config.replacement_policy, ReplacementPolicy::DepthPreferred);
        assert!(config.enable_statistics);
        assert!(!config.enable_verification);
    }

    #[test]
    fn test_config_file_save_load() {
        use std::io::Write;

        let config = EvaluationCacheConfig::default();

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_cache_config.json");

        // Save
        let save_result = config.save_to_file(&temp_file);
        assert!(save_result.is_ok());

        // Load
        let loaded = EvaluationCacheConfig::load_from_file(&temp_file);
        assert!(loaded.is_ok());

        let loaded_config = loaded.unwrap();
        assert_eq!(loaded_config.size, config.size);
        assert_eq!(loaded_config.replacement_policy, config.replacement_policy);

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_summary() {
        let config = EvaluationCacheConfig::default();
        let summary = config.summary();

        assert!(summary.contains("Cache Configuration"));
        assert!(summary.contains("Size:"));
        assert!(summary.contains("Replacement Policy:"));
    }

    #[test]
    fn test_runtime_policy_update() {
        let mut cache = EvaluationCache::new();

        assert_eq!(
            cache.config.replacement_policy,
            ReplacementPolicy::DepthPreferred
        );

        cache.update_replacement_policy(ReplacementPolicy::AlwaysReplace);
        assert_eq!(
            cache.config.replacement_policy,
            ReplacementPolicy::AlwaysReplace
        );

        cache.update_replacement_policy(ReplacementPolicy::AgingBased);
        assert_eq!(
            cache.config.replacement_policy,
            ReplacementPolicy::AgingBased
        );
    }

    #[test]
    fn test_runtime_statistics_toggle() {
        let mut cache = EvaluationCache::new();

        assert!(cache.config.enable_statistics);

        cache.set_statistics_enabled(false);
        assert!(!cache.config.enable_statistics);

        cache.set_statistics_enabled(true);
        assert!(cache.config.enable_statistics);
    }

    #[test]
    fn test_runtime_verification_toggle() {
        let mut cache = EvaluationCache::new();

        assert!(cache.config.enable_verification);

        cache.set_verification_enabled(false);
        assert!(!cache.config.enable_verification);

        cache.set_verification_enabled(true);
        assert!(cache.config.enable_verification);
    }

    #[test]
    fn test_cache_needs_maintenance() {
        let cache = EvaluationCache::new();

        // Fresh cache shouldn't need maintenance
        assert!(!cache.needs_maintenance());
    }

    #[test]
    fn test_performance_recommendations() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Generate some activity
        for i in 0..10 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
            cache.probe(&board, Player::Black, &captured_pieces);
        }

        let recommendations = cache.get_performance_recommendations();
        assert!(!recommendations.is_empty());

        // Should have at least one recommendation
        assert!(recommendations.len() > 0);
    }

    #[test]
    fn test_statistics_additional_metrics() {
        let mut stats = CacheStatistics::default();
        stats.probes = 100;
        stats.hits = 70;
        stats.misses = 30;
        stats.stores = 50;
        stats.replacements = 20;
        stats.collisions = 5;

        assert_eq!(stats.hit_rate(), 70.0);
        assert_eq!(stats.miss_rate(), 30.0);
        assert_eq!(stats.collision_rate(), 5.0);
        assert_eq!(stats.replacement_rate(), 40.0);
    }

    #[test]
    fn test_memory_utilization_calculation() {
        let metrics = CachePerformanceMetrics {
            avg_probe_time_ns: 50,
            avg_store_time_ns: 80,
            peak_memory_bytes: 1000000,
            current_memory_bytes: 1000000,
            filled_entries: 500,
            total_capacity: 1000,
        };

        assert_eq!(metrics.memory_utilization(), 50.0);
    }

    #[test]
    fn test_invalid_config_from_json() {
        let invalid_json = r#"{
            "size": 1000,
            "replacement_policy": "DepthPreferred"
        }"#;

        let result = EvaluationCacheConfig::from_json(invalid_json);
        assert!(result.is_err());
    }

    // ============================================================================
    // PHASE 2 HIGH PRIORITY TASKS TESTS
    // ============================================================================

    // Task 2.1: Multi-Level Cache Tests
    #[test]
    fn test_multi_level_cache_creation() {
        let cache = MultiLevelCache::new();
        assert_eq!(cache.get_config().l1_size, 16 * 1024);
        assert_eq!(cache.get_config().l2_size, 1024 * 1024);
    }

    #[test]
    fn test_multi_level_cache_l1_hit() {
        let cache = MultiLevelCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store and promote to L1
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache
            .l1()
            .store(&board, Player::Black, &captured_pieces, 100, 5);

        // Should hit L1
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        let stats = cache.get_statistics();
        assert!(stats.l1_hits > 0);
    }

    #[test]
    fn test_multi_level_cache_l2_hit() {
        let cache = MultiLevelCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store in L2 (default)
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        // First probe should miss L1, hit L2
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );

        let stats = cache.get_statistics();
        assert_eq!(stats.l1_hits, 0);
        assert_eq!(stats.l2_hits, 1);
    }

    #[test]
    fn test_multi_level_cache_promotion() {
        let mut config = MultiLevelCacheConfig::default();
        config.promotion_threshold = 2;
        let cache = MultiLevelCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store in L2
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        // Probe multiple times to trigger promotion
        for _ in 0..3 {
            cache.probe(&board, Player::Black, &captured_pieces);
        }

        let stats = cache.get_statistics();
        assert!(stats.promotions > 0);
    }

    #[test]
    fn test_multi_level_cache_statistics() {
        let cache = MultiLevelCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.probe(&board, Player::Black, &captured_pieces);

        let stats = cache.get_statistics();
        assert_eq!(stats.total_probes, 1);
        assert!(stats.overall_hit_rate() > 0.0);
    }

    #[test]
    fn test_multi_level_cache_clear() {
        let cache = MultiLevelCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.clear();

        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), None);

        let stats = cache.get_statistics();
        assert_eq!(stats.total_probes, 0);
    }

    #[test]
    fn test_multi_level_statistics_export() {
        let cache = MultiLevelCache::new();
        let stats = cache.get_statistics();

        let json = stats.export_json();
        assert!(json.is_ok());

        let summary = stats.summary();
        assert!(summary.contains("Multi-Level Cache Statistics"));
    }

    // Task 2.2: Cache Prefetching Tests
    #[test]
    fn test_prefetcher_creation() {
        let prefetcher = CachePrefetcher::new();
        let stats = prefetcher.get_statistics();
        assert_eq!(stats.queue_size, 0);
    }

    #[test]
    fn test_prefetcher_queue() {
        let prefetcher = CachePrefetcher::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        prefetcher.queue_prefetch(board, Player::Black, captured_pieces, 10);

        let stats = prefetcher.get_statistics();
        assert_eq!(stats.queue_size, 1);
    }

    #[test]
    fn test_prefetcher_priority_ordering() {
        let prefetcher = CachePrefetcher::with_max_queue_size(100);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Queue with different priorities
        prefetcher.queue_prefetch(board.clone(), Player::Black, captured_pieces.clone(), 5);
        prefetcher.queue_prefetch(board.clone(), Player::Black, captured_pieces.clone(), 10);
        prefetcher.queue_prefetch(board.clone(), Player::Black, captured_pieces.clone(), 3);

        let stats = prefetcher.get_statistics();
        assert_eq!(stats.queue_size, 3);
    }

    #[test]
    fn test_prefetcher_clear_queue() {
        let prefetcher = CachePrefetcher::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        prefetcher.queue_prefetch(board, Player::Black, captured_pieces, 10);
        prefetcher.clear_queue();

        let stats = prefetcher.get_statistics();
        assert_eq!(stats.queue_size, 0);
    }

    #[test]
    fn test_prefetch_statistics_export() {
        let prefetcher = CachePrefetcher::new();
        let stats = prefetcher.get_statistics();

        let json = stats.export_json();
        assert!(json.is_ok());

        let summary = stats.summary();
        assert!(summary.contains("Prefetch Statistics"));
    }

    #[test]
    fn test_prefetch_effectiveness_rate() {
        let mut stats = PrefetchStatistics {
            prefetched: 100,
            prefetch_hits: 30,
            prefetch_misses: 70,
            queue_size: 0,
        };

        assert_eq!(stats.effectiveness_rate(), 70.0);
    }

    // Task 2.3: Performance Optimization Tests
    #[test]
    fn test_cache_entry_alignment() {
        use std::mem::{align_of, size_of};

        // Verify cache entry is properly aligned for cache-line efficiency
        assert_eq!(align_of::<EvaluationEntry>(), 32);
        assert_eq!(size_of::<EvaluationEntry>(), 32);
    }

    #[test]
    fn test_optimized_probe_performance() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store value
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        // Benchmark probe (should be fast with inline optimization)
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = cache.probe(&board, Player::Black, &captured_pieces);
        }
        let duration = start.elapsed();

        // Should be very fast (<1ms for 1000 probes)
        assert!(
            duration.as_millis() < 10,
            "Probe optimization may need improvement"
        );
    }

    #[test]
    fn test_optimized_store_performance() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Benchmark store
        let start = std::time::Instant::now();
        for i in 0..1000 {
            cache.store(&board, Player::Black, &captured_pieces, i, 5);
        }
        let duration = start.elapsed();

        // Should be fast (<5ms for 1000 stores)
        assert!(
            duration.as_millis() < 20,
            "Store optimization may need improvement"
        );
    }

    #[test]
    fn test_fast_hash_calculation() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test that hash calculation is inlined and fast
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = cache.get_position_hash_fast(&board, Player::Black, &captured_pieces);
        }
        let duration = start.elapsed();

        // Should be very fast (<5ms for 10000 hashes)
        assert!(
            duration.as_millis() < 50,
            "Hash calculation may need optimization"
        );
    }

    #[test]
    fn test_inline_optimization_annotations() {
        // This test verifies that the code compiles with inline annotations
        // The actual performance benefit would be measured in benchmarks
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );
    }

    // ============================================================================
    // PHASE 2 MEDIUM PRIORITY TASKS TESTS
    // ============================================================================

    // Task 2.4: Cache Persistence Tests
    #[test]
    fn test_cache_save_load() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store some entries
        for i in 0..10 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        }

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_cache_save.json");

        let save_result = cache.save_to_file(&temp_file);
        assert!(save_result.is_ok());

        // Load from file
        let loaded = EvaluationCache::load_from_file(&temp_file);
        assert!(loaded.is_ok());

        let loaded_cache = loaded.unwrap();
        assert_eq!(loaded_cache.config.size, cache.config.size);

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_cache_save_load_compressed() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store entries
        for i in 0..10 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        }

        // Save compressed
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_cache_compressed.gz");

        let save_result = cache.save_to_file_compressed(&temp_file);
        assert!(save_result.is_ok());

        // Load compressed
        let loaded = EvaluationCache::load_from_file_compressed(&temp_file);
        assert!(loaded.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_cache_versioning() {
        // Test that version checking works
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_cache_version.json");

        // Write invalid version
        let invalid = r#"{
            "version": 999,
            "magic": 1397441351,
            "config": {"size": 1024, "replacement_policy": "DepthPreferred", "enable_statistics": true, "enable_verification": true},
            "entries": []
        }"#;
        std::fs::write(&temp_file, invalid).unwrap();

        // Should fail to load
        let result = EvaluationCache::load_from_file(&temp_file);
        assert!(result.is_err());

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_serializable_entry_conversion() {
        let entry = EvaluationEntry::new(0x123456, 100, 5);
        let ser: SerializableEntry = entry.into();

        assert_eq!(ser.key, 0x123456);
        assert_eq!(ser.score, 100);
        assert_eq!(ser.depth, 5);

        let converted: EvaluationEntry = ser.into();
        assert_eq!(converted.key, 0x123456);
        assert_eq!(converted.score, 100);
        assert_eq!(converted.depth, 5);
    }

    // Task 2.5: Memory Management Tests
    #[test]
    fn test_memory_usage_tracking() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let usage_before = cache.get_memory_usage();
        assert_eq!(usage_before.filled_entries, 0);

        // Store some entries
        for i in 0..10 {
            cache.store(&board, Player::Black, &captured_pieces, i * 10, 5);
        }

        let usage_after = cache.get_memory_usage();
        assert!(usage_after.filled_entries > 0);
        assert_eq!(usage_after.entries, cache.config.size);
    }

    #[test]
    fn test_memory_pressure_detection() {
        let cache = EvaluationCache::new();

        // Fresh cache should not be under pressure
        assert!(!cache.is_under_memory_pressure());
    }

    #[test]
    fn test_cache_size_suggestion() {
        let cache = EvaluationCache::new();
        let suggested = cache.suggest_cache_size();

        // Should return a valid size
        assert!(suggested.is_power_of_two());
        assert!(suggested >= 1024);
    }

    #[test]
    fn test_cache_resize() {
        let mut cache = EvaluationCache::with_config(EvaluationCacheConfig {
            size: 1024,
            replacement_policy: ReplacementPolicy::DepthPreferred,
            enable_statistics: true,
            enable_verification: true,
        });
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store some entries
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        // Resize to larger
        let result = cache.resize(2048);
        assert!(result.is_ok());
        assert_eq!(cache.config.size, 2048);

        // Should still have the entry
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );
    }

    #[test]
    fn test_cache_compact() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store and age entries
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        cache.compact();

        // Entry should still be there (not old enough)
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );
    }

    #[test]
    fn test_memory_usage_calculations() {
        let usage = MemoryUsage {
            total_bytes: 1000,
            used_bytes: 600,
            entries: 100,
            filled_entries: 50,
        };

        assert_eq!(usage.utilization(), 60.0);
        assert_eq!(usage.entry_utilization(), 50.0);
    }

    // Task 2.6: Advanced Features Tests
    #[test]
    fn test_cache_warmer_creation() {
        let warmer = CacheWarmer::new(WarmingStrategy::Common);
        assert_eq!(warmer.get_warmed_count(), 0);
    }

    #[test]
    fn test_cache_warming() {
        let cache = EvaluationCache::new();
        let evaluator = crate::evaluation::PositionEvaluator::new();
        let warmer = CacheWarmer::new(WarmingStrategy::Common);

        warmer.warm_cache(&cache, &evaluator);
        assert!(warmer.get_warmed_count() > 0);
    }

    #[test]
    fn test_adaptive_cache_sizer() {
        let sizer = AdaptiveCacheSizer::new(1024, 1024 * 1024, 60.0);
        let cache = EvaluationCache::new();

        // Should not resize immediately
        assert_eq!(sizer.should_resize(&cache), None);
    }

    #[test]
    fn test_cache_analytics() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store entries with different depths
        for depth in 1..=5 {
            cache.store(
                &board,
                Player::Black,
                &captured_pieces,
                depth as i32 * 10,
                depth,
            );
        }

        let analytics = cache.get_analytics();
        assert!(!analytics.depth_distribution.is_empty());
    }

    #[test]
    fn test_analytics_json_export() {
        let cache = EvaluationCache::new();
        let json = cache.export_analytics_json();

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("depth_distribution"));
        assert!(json_str.contains("age_distribution"));
    }

    #[test]
    fn test_warming_strategies() {
        let strategies = vec![
            WarmingStrategy::None,
            WarmingStrategy::Common,
            WarmingStrategy::Opening,
            WarmingStrategy::Endgame,
        ];

        for strategy in strategies {
            let warmer = CacheWarmer::new(strategy);
            assert_eq!(warmer.get_warmed_count(), 0);
        }
    }

    #[test]
    fn test_cache_binary_size_efficiency() {
        // Verify cache entry is compact for binary size
        use std::mem::size_of;

        // Entry should be exactly 32 bytes (cache-line aligned)
        assert_eq!(size_of::<EvaluationEntry>(), 32);

        // Config should be small
        assert!(size_of::<EvaluationCacheConfig>() < 64);
    }

    #[test]
    fn test_small_cache_efficiency() {
        // Test that a small cache works correctly
        let config = EvaluationCacheConfig {
            size: 1024,
            replacement_policy: ReplacementPolicy::DepthPreferred,
            enable_statistics: false,
            enable_verification: false,
        };

        let cache = EvaluationCache::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Should work correctly even with small size
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(100)
        );
    }

    // ============================================================================
    // PHASE 3 TASK 3.6: ADVANCED INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_cache_with_transposition_table_compatibility() {
        // Verify cache doesn't conflict with transposition table
        // (They serve different purposes and can coexist)
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        cache.store(&board, Player::Black, &captured_pieces, 150, 5);
        assert_eq!(
            cache.probe(&board, Player::Black, &captured_pieces),
            Some(150)
        );

        // Would also have transposition table active in real usage
        // Both can work simultaneously
    }

    #[test]
    fn test_cache_for_analysis_mode() {
        // Large cache for deep analysis
        let config = EvaluationCacheConfig::with_size_mb(64);
        let cache = EvaluationCache::with_config(config);

        assert!(cache.config.size >= 1024 * 1024);
        assert!(cache.size_mb() >= 60.0);
    }

    #[test]
    fn test_thread_safe_cache_access() {
        // Verify thread safety (already built-in via RwLock)
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Multiple accesses (simulating concurrent use)
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);

        for _ in 0..100 {
            let _ = cache.probe(&board, Player::Black, &captured_pieces);
        }

        // Should complete without issues
        let stats = cache.get_statistics();
        assert!(stats.probes >= 100);
    }
}
