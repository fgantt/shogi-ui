//! Cache management system for transposition table
//! 
//! This module provides comprehensive cache management functionality including
//! age counter systems, statistics tracking, cache warming strategies, and
//! performance monitoring for the transposition table.

use crate::types::*;
use crate::search::transposition_config::TranspositionConfig;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Age counter system for transposition table entries
/// 
/// This struct manages the global age counter that is used to track
/// the age of entries in the transposition table for replacement policies.
#[derive(Debug, Clone)]
pub struct AgeCounter {
    /// Current global age counter
    current_age: u32,
    /// Maximum age before wrapping
    max_age: u32,
    /// Age increment frequency (in nodes or time)
    increment_frequency: AgeIncrementFrequency,
    /// Last increment time
    last_increment: Instant,
    /// Statistics for age management
    stats: AgeCounterStats,
}

/// Frequency of age increments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeIncrementFrequency {
    /// Increment every N nodes
    PerNodes(u32),
    /// Increment every N milliseconds
    PerTime(Duration),
    /// Increment every probe
    PerProbe,
    /// Manual increment only
    Manual,
}

/// Statistics for age counter system
#[derive(Debug, Clone, Default)]
pub struct AgeCounterStats {
    /// Total number of age increments
    pub total_increments: u64,
    /// Number of age wraps (when age reaches max_age)
    pub age_wraps: u64,
    /// Average time between increments
    pub avg_increment_interval_ms: f64,
    /// Last increment time
    pub last_increment_time: Option<Instant>,
}

impl AgeCounter {
    /// Get current time, with WASM compatibility
    #[cfg(target_arch = "wasm32")]
    fn get_current_time() -> Instant {
        // In WASM, we can't use Instant::now(), so we'll use a dummy implementation
        // This is a workaround since WASM doesn't support time operations
        unsafe { std::mem::zeroed() }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_current_time() -> Instant {
        Instant::now()
    }

    /// Create a new age counter
    pub fn new(config: &TranspositionConfig) -> Self {
        Self {
            current_age: 0,
            max_age: config.max_age,
            increment_frequency: AgeIncrementFrequency::PerNodes(1000), // Default: every 1000 nodes
            last_increment: Self::get_current_time(),
            stats: AgeCounterStats::default(),
        }
    }
    
    /// Create an age counter with custom frequency
    pub fn with_frequency(config: &TranspositionConfig, frequency: AgeIncrementFrequency) -> Self {
        Self {
            current_age: 0,
            max_age: config.max_age,
            increment_frequency: frequency,
            last_increment: Self::get_current_time(),
            stats: AgeCounterStats::default(),
        }
    }
    
    /// Get the current age
    pub fn current_age(&self) -> u32 {
        self.current_age
    }
    
    /// Increment the age counter
    /// 
    /// This method handles automatic age incrementing based on the configured
    /// frequency and manages age wrapping when the maximum age is reached.
    pub fn increment_age(&mut self, node_count: u64) -> bool {
        let should_increment = match self.increment_frequency {
            AgeIncrementFrequency::PerNodes(interval) => {
                node_count % interval as u64 == 0
            }
            AgeIncrementFrequency::PerTime(duration) => {
                self.last_increment.elapsed() >= duration
            }
            AgeIncrementFrequency::PerProbe => {
                true // Always increment on probe
            }
            AgeIncrementFrequency::Manual => {
                false // Only manual increments
            }
        };
        
        if should_increment {
            self.current_age = self.current_age.wrapping_add(1);
            
            // Handle age wrapping
            if self.current_age == 0 {
                self.stats.age_wraps += 1;
                self.current_age = 1; // Start from 1 after wrap
            }
            
            self.stats.total_increments += 1;
            self.stats.last_increment_time = Some(Self::get_current_time());
            
            // Update average increment interval
            let elapsed = self.last_increment.elapsed();
            if self.stats.total_increments > 1 {
                let current_avg = self.stats.avg_increment_interval_ms;
                let new_interval = elapsed.as_millis() as f64;
                self.stats.avg_increment_interval_ms = 
                    (current_avg * (self.stats.total_increments - 1) as f64 + new_interval) 
                    / self.stats.total_increments as f64;
            } else {
                self.stats.avg_increment_interval_ms = elapsed.as_millis() as f64;
            }
            
            self.last_increment = Self::get_current_time();
            true
        } else {
            false
        }
    }
    
    /// Manually increment the age (regardless of frequency)
    pub fn force_increment(&mut self) {
        self.current_age = self.current_age.wrapping_add(1);
        
        if self.current_age == 0 {
            self.stats.age_wraps += 1;
            self.current_age = 1;
        }
        
        self.stats.total_increments += 1;
        self.stats.last_increment_time = Some(Self::get_current_time());
        self.last_increment = Self::get_current_time();
    }
    
    /// Check if an entry is expired based on age
    pub fn is_entry_expired(&self, entry_age: u32) -> bool {
        let age_difference = self.current_age.wrapping_sub(entry_age);
        age_difference > self.max_age
    }
    
    /// Get the age difference between current age and entry age
    pub fn age_difference(&self, entry_age: u32) -> u32 {
        self.current_age.wrapping_sub(entry_age)
    }
    
    /// Get age counter statistics
    pub fn get_stats(&self) -> &AgeCounterStats {
        &self.stats
    }
    
    /// Reset age counter and statistics
    pub fn reset(&mut self) {
        self.current_age = 0;
        self.stats = AgeCounterStats::default();
        self.last_increment = Self::get_current_time();
    }
}

/// Cache management system for transposition table
/// 
/// This struct provides comprehensive cache management including statistics
/// tracking, hit rate calculation, cache warming, and performance monitoring.
#[derive(Debug)]
pub struct CacheManager {
    /// Age counter for entry aging
    age_counter: AgeCounter,
    /// Cache statistics
    cache_stats: CacheStats,
    /// Configuration
    config: TranspositionConfig,
    /// Cache warming data
    warming_data: CacheWarmingData,
    /// Performance monitoring
    performance_monitor: PerformanceMonitor,
}

/// Comprehensive cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of probes
    pub total_probes: u64,
    /// Number of hits
    pub hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Number of stores
    pub stores: u64,
    /// Number of replacements
    pub replacements: u64,
    /// Number of expired entries removed
    pub expired_removals: u64,
    /// Total memory usage in bytes
    pub memory_usage: usize,
    /// Cache warming hits
    pub warming_hits: u64,
    /// Cache warming misses
    pub warming_misses: u64,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            (self.hits as f64 / self.total_probes as f64) * 100.0
        }
    }
    
    /// Calculate miss rate as a percentage
    pub fn miss_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            (self.misses as f64 / self.total_probes as f64) * 100.0
        }
    }
    
    /// Calculate replacement rate
    pub fn replacement_rate(&self) -> f64 {
        if self.stores == 0 {
            0.0
        } else {
            (self.replacements as f64 / self.stores as f64) * 100.0
        }
    }
    
    /// Calculate cache warming hit rate
    pub fn warming_hit_rate(&self) -> f64 {
        let warming_total = self.warming_hits + self.warming_misses;
        if warming_total == 0 {
            0.0
        } else {
            (self.warming_hits as f64 / warming_total as f64) * 100.0
        }
    }
    
    /// Get memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage as f64 / (1024.0 * 1024.0)
    }
}

/// Cache warming data and strategies
#[derive(Debug, Default)]
pub struct CacheWarmingData {
    /// Known positions for warming
    warming_positions: HashMap<u64, TranspositionEntry>,
    /// Warming strategy
    strategy: CacheWarmingStrategy,
    /// Statistics for warming
    warming_stats: WarmingStats,
}

/// Cache warming strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWarmingStrategy {
    /// No warming
    None,
    /// Warm with opening book positions
    OpeningBook,
    /// Warm with tactical positions
    Tactical,
    /// Warm with endgame positions
    Endgame,
    /// Warm with all types
    All,
}

impl Default for CacheWarmingStrategy {
    fn default() -> Self {
        CacheWarmingStrategy::None
    }
}

/// Statistics for cache warming
#[derive(Debug, Clone, Default)]
pub struct WarmingStats {
    /// Number of positions warmed
    pub positions_warmed: u64,
    /// Time spent warming
    pub warming_time_ms: u64,
    /// Average warming time per position
    pub avg_warming_time_ms: f64,
}

/// Performance monitoring system
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    /// Probe timing data
    probe_times: Vec<Duration>,
    /// Store timing data
    store_times: Vec<Duration>,
    /// Maximum timing samples to keep
    max_samples: usize,
    /// Performance thresholds
    thresholds: PerformanceThresholds,
}

/// Performance thresholds for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum acceptable probe time in microseconds
    pub max_probe_time_us: u64,
    /// Maximum acceptable store time in microseconds
    pub max_store_time_us: u64,
    /// Minimum acceptable hit rate percentage
    pub min_hit_rate_percent: f64,
    /// Maximum acceptable memory usage in MB
    pub max_memory_mb: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_probe_time_us: 100, // 100 microseconds
            max_store_time_us: 50,  // 50 microseconds
            min_hit_rate_percent: 85.0, // 85% hit rate
            max_memory_mb: 512.0,   // 512 MB
        }
    }
}

impl CacheManager {
    /// Get current time, with WASM compatibility
    #[cfg(target_arch = "wasm32")]
    fn get_current_time() -> Instant {
        // In WASM, we can't use Instant::now(), so we'll use a dummy implementation
        unsafe { std::mem::zeroed() }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_current_time() -> Instant {
        Instant::now()
    }

    /// Create a new cache manager
    pub fn new(config: TranspositionConfig) -> Self {
        Self {
            age_counter: AgeCounter::new(&config),
            cache_stats: CacheStats::default(),
            config,
            warming_data: CacheWarmingData::default(),
            performance_monitor: PerformanceMonitor::default(),
        }
    }
    
    /// Get the current age from the age counter
    pub fn current_age(&self) -> u32 {
        self.age_counter.current_age()
    }
    
    /// Increment age counter
    pub fn increment_age(&mut self, node_count: u64) -> bool {
        self.age_counter.increment_age(node_count)
    }
    
    /// Force age increment
    pub fn force_age_increment(&mut self) {
        self.age_counter.force_increment();
    }
    
    /// Check if an entry is expired
    pub fn is_entry_expired(&self, entry_age: u32) -> bool {
        self.age_counter.is_entry_expired(entry_age)
    }
    
    /// Record a cache probe
    pub fn record_probe(&mut self, start_time: Instant, hit: bool) {
        let probe_time = start_time.elapsed();
        self.performance_monitor.record_probe_time(probe_time);
        
        self.cache_stats.total_probes += 1;
        if hit {
            self.cache_stats.hits += 1;
        } else {
            self.cache_stats.misses += 1;
        }
    }
    
    /// Record a cache store
    pub fn record_store(&mut self, start_time: Instant, was_replacement: bool) {
        let store_time = start_time.elapsed();
        self.performance_monitor.record_store_time(store_time);
        
        self.cache_stats.stores += 1;
        if was_replacement {
            self.cache_stats.replacements += 1;
        }
    }
    
    /// Record expired entry removal
    pub fn record_expired_removal(&mut self) {
        self.cache_stats.expired_removals += 1;
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&mut self, usage: usize) {
        self.cache_stats.memory_usage = usage;
    }
    
    /// Get cache hit rate
    pub fn get_hit_rate(&self) -> f64 {
        self.cache_stats.hit_rate()
    }
    
    /// Get cache miss rate
    pub fn get_miss_rate(&self) -> f64 {
        self.cache_stats.miss_rate()
    }
    
    /// Get comprehensive cache statistics
    pub fn get_cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }
    
    /// Get age counter statistics
    pub fn get_age_stats(&self) -> &AgeCounterStats {
        self.age_counter.get_stats()
    }
    
    /// Warm the cache with known positions
    pub fn warm_cache(&mut self, positions: &[(u64, TranspositionEntry)]) {
        let start_time = Self::get_current_time();
        
        for (hash, entry) in positions {
            self.warming_data.warming_positions.insert(*hash, entry.clone());
        }
        
        let warming_time = start_time.elapsed();
        self.warming_data.warming_stats.positions_warmed += positions.len() as u64;
        self.warming_data.warming_stats.warming_time_ms += warming_time.as_millis() as u64;
        
        // Update average warming time
        let total_positions = self.warming_data.warming_stats.positions_warmed;
        if total_positions > 0 {
            self.warming_data.warming_stats.avg_warming_time_ms = 
                self.warming_data.warming_stats.warming_time_ms as f64 / total_positions as f64;
        }
    }
    
    /// Check if a position is in warming data
    pub fn is_warming_position(&self, hash: u64) -> bool {
        self.warming_data.warming_positions.contains_key(&hash)
    }
    
    /// Get warming position entry
    pub fn get_warming_position(&self, hash: u64) -> Option<&TranspositionEntry> {
        self.warming_data.warming_positions.get(&hash)
    }
    
    /// Record cache warming hit/miss
    pub fn record_warming_result(&mut self, hit: bool) {
        if hit {
            self.cache_stats.warming_hits += 1;
        } else {
            self.cache_stats.warming_misses += 1;
        }
    }
    
    /// Set cache warming strategy
    pub fn set_warming_strategy(&mut self, strategy: CacheWarmingStrategy) {
        self.warming_data.strategy = strategy;
    }
    
    /// Get cache warming statistics
    pub fn get_warming_stats(&self) -> &WarmingStats {
        &self.warming_data.warming_stats
    }
    
    /// Check performance against thresholds
    pub fn check_performance(&self) -> PerformanceReport {
        self.performance_monitor.check_performance(&self.cache_stats)
    }
    
    /// Get performance monitoring data
    pub fn get_performance_data(&self) -> PerformanceData {
        self.performance_monitor.get_performance_data()
    }
    
    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.cache_stats = CacheStats::default();
        self.age_counter.reset();
        self.warming_data.warming_stats = WarmingStats::default();
        self.performance_monitor.reset();
    }
    
    /// Clean expired entries from warming data
    pub fn clean_expired_warming_entries(&mut self) {
        let current_age = self.current_age();
        self.warming_data.warming_positions.retain(|_, entry| {
            let age_difference = current_age.wrapping_sub(entry.age);
            age_difference <= self.config.max_age
        });
    }
}

impl PerformanceMonitor {
    /// Record probe time
    fn record_probe_time(&mut self, duration: Duration) {
        self.probe_times.push(duration);
        if self.probe_times.len() > self.max_samples {
            self.probe_times.remove(0);
        }
    }
    
    /// Record store time
    fn record_store_time(&mut self, duration: Duration) {
        self.store_times.push(duration);
        if self.store_times.len() > self.max_samples {
            self.store_times.remove(0);
        }
    }
    
    /// Check performance against thresholds
    fn check_performance(&self, cache_stats: &CacheStats) -> PerformanceReport {
        let mut issues = Vec::new();
        
        // Check probe time
        if let Some(avg_probe_time) = self.average_probe_time() {
            if avg_probe_time.as_micros() > self.thresholds.max_probe_time_us as u128 {
                issues.push(PerformanceIssue::ProbeTimeTooHigh {
                    current_us: avg_probe_time.as_micros(),
                    threshold_us: self.thresholds.max_probe_time_us,
                });
            }
        }
        
        // Check store time
        if let Some(avg_store_time) = self.average_store_time() {
            if avg_store_time.as_micros() > self.thresholds.max_store_time_us as u128 {
                issues.push(PerformanceIssue::StoreTimeTooHigh {
                    current_us: avg_store_time.as_micros(),
                    threshold_us: self.thresholds.max_store_time_us,
                });
            }
        }
        
        // Check hit rate
        let hit_rate = cache_stats.hit_rate();
        if hit_rate < self.thresholds.min_hit_rate_percent {
            issues.push(PerformanceIssue::HitRateTooLow {
                current_percent: hit_rate,
                threshold_percent: self.thresholds.min_hit_rate_percent,
            });
        }
        
        // Check memory usage
        let memory_mb = cache_stats.memory_usage_mb();
        if memory_mb > self.thresholds.max_memory_mb {
            issues.push(PerformanceIssue::MemoryUsageTooHigh {
                current_mb: memory_mb,
                threshold_mb: self.thresholds.max_memory_mb,
            });
        }
        
        let overall_status = if issues.is_empty() { 
            PerformanceStatus::Good 
        } else { 
            PerformanceStatus::IssuesDetected 
        };
        
        PerformanceReport {
            issues,
            overall_status,
        }
    }
    
    /// Get performance data
    fn get_performance_data(&self) -> PerformanceData {
        PerformanceData {
            average_probe_time: self.average_probe_time(),
            average_store_time: self.average_store_time(),
            probe_time_samples: self.probe_times.len(),
            store_time_samples: self.store_times.len(),
        }
    }
    
    /// Calculate average probe time
    fn average_probe_time(&self) -> Option<Duration> {
        if self.probe_times.is_empty() {
            None
        } else {
            let total: Duration = self.probe_times.iter().sum();
            Some(Duration::from_nanos(total.as_nanos() as u64 / self.probe_times.len() as u64))
        }
    }
    
    /// Calculate average store time
    fn average_store_time(&self) -> Option<Duration> {
        if self.store_times.is_empty() {
            None
        } else {
            let total: Duration = self.store_times.iter().sum();
            Some(Duration::from_nanos(total.as_nanos() as u64 / self.store_times.len() as u64))
        }
    }
    
    /// Reset performance monitor
    fn reset(&mut self) {
        self.probe_times.clear();
        self.store_times.clear();
    }
}

/// Performance report
#[derive(Debug)]
pub struct PerformanceReport {
    pub issues: Vec<PerformanceIssue>,
    pub overall_status: PerformanceStatus,
}

/// Performance issues
#[derive(Debug)]
pub enum PerformanceIssue {
    ProbeTimeTooHigh { current_us: u128, threshold_us: u64 },
    StoreTimeTooHigh { current_us: u128, threshold_us: u64 },
    HitRateTooLow { current_percent: f64, threshold_percent: f64 },
    MemoryUsageTooHigh { current_mb: f64, threshold_mb: f64 },
}

/// Overall performance status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceStatus {
    Good,
    IssuesDetected,
}

/// Performance monitoring data
#[derive(Debug)]
pub struct PerformanceData {
    pub average_probe_time: Option<Duration>,
    pub average_store_time: Option<Duration>,
    pub probe_time_samples: usize,
    pub store_time_samples: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_age_counter_basic() {
        let config = TranspositionConfig::debug_config();
        let mut age_counter = AgeCounter::new(&config);
        
        assert_eq!(age_counter.current_age(), 0);
        
        // Test manual increment
        age_counter.force_increment();
        assert_eq!(age_counter.current_age(), 1);
        
        // Test node-based increment
        assert!(age_counter.increment_age(1000));
        assert_eq!(age_counter.current_age(), 2);
        
        // Test that increment doesn't happen for non-multiple nodes
        assert!(!age_counter.increment_age(1001));
        assert_eq!(age_counter.current_age(), 2);
    }
    
    #[test]
    fn test_age_counter_wrapping() {
        let mut config = TranspositionConfig::debug_config();
        config.max_age = 3; // Small max age for testing
        
        let mut age_counter = AgeCounter::with_frequency(&config, AgeIncrementFrequency::Manual);
        
        // Increment to max age
        age_counter.force_increment(); // age = 1
        age_counter.force_increment(); // age = 2
        age_counter.force_increment(); // age = 3
        
        // Next increment should wrap
        age_counter.force_increment(); // age = 1 (wrapped)
        assert_eq!(age_counter.current_age(), 1);
        
        let stats = age_counter.get_stats();
        assert_eq!(stats.age_wraps, 1);
    }
    
    #[test]
    fn test_entry_expiration() {
        let config = TranspositionConfig::debug_config();
        let mut age_counter = AgeCounter::new(&config);
        
        let entry_age = 10;
        
        // Entry should not be expired initially
        assert!(!age_counter.is_entry_expired(entry_age));
        
        // Increment age significantly
        for _ in 0..150 { // More than max_age (100)
            age_counter.force_increment();
        }
        
        // Entry should now be expired
        assert!(age_counter.is_entry_expired(entry_age));
    }
    
    #[test]
    fn test_cache_manager_basic() {
        let config = TranspositionConfig::debug_config();
        let mut manager = CacheManager::new(config);
        
        assert_eq!(manager.current_age(), 0);
        
        // Test age increment
        manager.force_age_increment();
        assert_eq!(manager.current_age(), 1);
        
        // Test probe recording
        let start = Instant::now();
        manager.record_probe(start, true);
        
        let stats = manager.get_cache_stats();
        assert_eq!(stats.total_probes, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate(), 100.0);
    }
    
    #[test]
    fn test_cache_warming() {
        let config = TranspositionConfig::debug_config();
        let mut manager = CacheManager::new(config);
        
        // Create test warming positions
        let positions = vec![
            (12345, create_test_entry(100, 5, TranspositionFlag::Exact, 1)),
            (67890, create_test_entry(-50, 3, TranspositionFlag::LowerBound, 1)),
        ];
        
        manager.warm_cache(&positions);
        
        // Check that positions are in warming data
        assert!(manager.is_warming_position(12345));
        assert!(manager.is_warming_position(67890));
        assert!(!manager.is_warming_position(99999));
        
        // Check warming stats
        let warming_stats = manager.get_warming_stats();
        assert_eq!(warming_stats.positions_warmed, 2);
        assert!(warming_stats.warming_time_ms > 0);
    }
    
    #[test]
    fn test_performance_monitoring() {
        let config = TranspositionConfig::debug_config();
        let mut manager = CacheManager::new(config);
        
        // Record some probe times
        for _ in 0..5 {
            let start = Instant::now();
            thread::sleep(Duration::from_micros(10)); // Simulate work
            manager.record_probe(start, true);
        }
        
        let perf_data = manager.get_performance_data();
        assert_eq!(perf_data.probe_time_samples, 5);
        assert!(perf_data.average_probe_time.is_some());
        
        // Check performance report
        let report = manager.check_performance();
        assert_eq!(report.overall_status, PerformanceStatus::Good);
    }
    
    #[test]
    fn test_cache_statistics() {
        let config = TranspositionConfig::debug_config();
        let mut manager = CacheManager::new(config);
        
        // Record various operations
        manager.record_probe(Instant::now(), true);
        manager.record_probe(Instant::now(), false);
        manager.record_probe(Instant::now(), true);
        
        manager.record_store(Instant::now(), false);
        manager.record_store(Instant::now(), true);
        
        manager.record_expired_removal();
        manager.update_memory_usage(1024 * 1024); // 1 MB
        
        let stats = manager.get_cache_stats();
        assert_eq!(stats.total_probes, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.stores, 2);
        assert_eq!(stats.replacements, 1);
        assert_eq!(stats.expired_removals, 1);
        assert_eq!(stats.memory_usage, 1024 * 1024);
        
        // Check calculated rates
        assert!((stats.hit_rate() - 66.67).abs() < 0.1);
        assert!((stats.miss_rate() - 33.33).abs() < 0.1);
        assert!((stats.replacement_rate() - 50.0).abs() < 0.1);
        assert_eq!(stats.memory_usage_mb(), 1.0);
    }
    
    #[test]
    fn test_statistics_reset() {
        let config = TranspositionConfig::debug_config();
        let mut manager = CacheManager::new(config);
        
        // Add some data
        manager.record_probe(Instant::now(), true);
        manager.force_age_increment();
        
        // Reset and verify
        manager.reset_stats();
        
        let stats = manager.get_cache_stats();
        assert_eq!(stats.total_probes, 0);
        assert_eq!(manager.current_age(), 0);
    }
    
    fn create_test_entry(score: i32, depth: u8, flag: TranspositionFlag, age: u32) -> TranspositionEntry {
        let mut entry = TranspositionEntry::new_with_age(score, depth, flag, None, 0);
        entry.age = age;
        entry
    }
}
