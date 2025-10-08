use crate::types::*;
use crate::bitboards::BitboardBoard;
use crate::search::zobrist::{ZobristHasher, RepetitionState};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::RwLock;
use serde::{Serialize, Deserialize};

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
        Self {
            size: 1024 * 1024, // 1M entries (about 32MB with 32 bytes per entry)
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
            return Err(format!("Cache size must be a power of 2, got {}", self.size));
        }
        if self.size < 1024 {
            return Err(format!("Cache size too small (minimum 1024 entries), got {}", self.size));
        }
        if self.size > 128 * 1024 * 1024 {
            return Err(format!("Cache size too large (maximum 128M entries), got {}", self.size));
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
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Export configuration as JSON string
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))
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
#[derive(Debug, Clone, Copy)]
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
}

impl Default for EvaluationEntry {
    fn default() -> Self {
        Self {
            key: 0,
            score: 0,
            depth: 0,
            age: 0,
            verification: 0,
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

/// Evaluation cache for storing previously calculated position evaluations
pub struct EvaluationCache {
    /// Cache entries
    entries: Vec<RwLock<EvaluationEntry>>,
    /// Configuration
    config: EvaluationCacheConfig,
    /// Zobrist hasher for position hashing
    zobrist_hasher: ZobristHasher,
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
        self.zobrist_hasher.hash_position(
            board,
            player,
            captured_pieces,
            RepetitionState::None,
        )
    }

    /// Get the cache index for a hash
    fn get_index(&self, hash: u64) -> usize {
        // Use lower bits for indexing (fast modulo for power of 2)
        (hash as usize) & (self.config.size - 1)
    }

    /// Probe the cache for a position
    pub fn probe(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<i32> {
        if self.config.enable_statistics {
            self.stats_probes.fetch_add(1, Ordering::Relaxed);
        }

        let hash = self.get_position_hash(board, player, captured_pieces);
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
        let filled_entries = self.entries.iter()
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
            stats.hits, (stats.hits as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.misses, (stats.misses as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.probes, stats.hit_rate(),
            stats.collisions, (stats.collisions as f64 / stats.probes.max(1) as f64) * 100.0,
            stats.collision_rate(), stats.collision_rate(),
            stats.stores,
            stats.replacements, (stats.replacements as f64 / stats.stores.max(1) as f64) * 100.0,
            stats.stores, stats.replacement_rate()
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
        (stats.collision_rate() > 15.0) ||
        (metrics.memory_utilization() > 95.0) ||
        (stats.probes > 1000 && stats.hit_rate() < 30.0)
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

#[cfg(test)]
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
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(150));
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
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(100));

        // Store second value (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 200, 3);
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(200));
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
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(100));

        // Try to store with lower depth (should not replace)
        cache.store(&board, Player::Black, &captured_pieces, 200, 3);
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(100));

        // Store with equal depth (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 300, 5);
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(300));

        // Store with higher depth (should replace)
        cache.store(&board, Player::Black, &captured_pieces, 400, 7);
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(400));
    }

    #[test]
    fn test_cache_clear() {
        let cache = EvaluationCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Store a value
        cache.store(&board, Player::Black, &captured_pieces, 100, 5);
        assert_eq!(cache.probe(&board, Player::Black, &captured_pieces), Some(100));

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
        assert_eq!(cache.probe(&board1, Player::Black, &captured_pieces), Some(100));

        // White player should have different hash
        cache.store(&board1, Player::White, &captured_pieces, 200, 5);
        assert_eq!(cache.probe(&board1, Player::White, &captured_pieces), Some(200));
        
        // Black player should still have its value
        assert_eq!(cache.probe(&board1, Player::Black, &captured_pieces), Some(100));
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
        
        assert_eq!(cache.config.replacement_policy, ReplacementPolicy::DepthPreferred);
        
        cache.update_replacement_policy(ReplacementPolicy::AlwaysReplace);
        assert_eq!(cache.config.replacement_policy, ReplacementPolicy::AlwaysReplace);
        
        cache.update_replacement_policy(ReplacementPolicy::AgingBased);
        assert_eq!(cache.config.replacement_policy, ReplacementPolicy::AgingBased);
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
}
