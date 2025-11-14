//! Configuration management for the tablebase system
//!
//! This module provides configuration structures and management for
//! the tablebase system, including solver-specific settings and
//! performance tuning options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration for the tablebase system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablebaseConfig {
    /// Whether the tablebase system is enabled
    pub enabled: bool,
    /// Maximum cache size for position caching
    pub cache_size: usize,
    /// Maximum depth for tablebase searches
    pub max_depth: u8,
    /// Confidence threshold for tablebase results
    pub confidence_threshold: f32,
    /// Solver-specific configurations
    pub solvers: SolverConfig,
    /// Performance tuning options
    pub performance: PerformanceConfig,
    /// Memory usage monitoring and limits
    pub memory: MemoryConfig,
}

impl Default for TablebaseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size: 10000,
            max_depth: 20,
            confidence_threshold: 0.8,
            solvers: SolverConfig::default(),
            performance: PerformanceConfig::default(),
            memory: MemoryConfig::default(),
        }
    }
}

impl TablebaseConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            cache_size: 50000,
            max_depth: 15,
            confidence_threshold: 0.9,
            solvers: SolverConfig::performance_optimized(),
            performance: PerformanceConfig::performance_optimized(),
            memory: MemoryConfig::performance_optimized(),
        }
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            cache_size: 1000,
            max_depth: 10,
            confidence_threshold: 0.7,
            solvers: SolverConfig::memory_optimized(),
            performance: PerformanceConfig::memory_optimized(),
            memory: MemoryConfig::memory_optimized(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.cache_size == 0 {
            return Err("Cache size must be greater than 0".to_string());
        }

        if self.max_depth == 0 {
            return Err("Max depth must be greater than 0".to_string());
        }

        if self.confidence_threshold < 0.0 || self.confidence_threshold > 1.0 {
            return Err("Confidence threshold must be between 0.0 and 1.0".to_string());
        }

        self.solvers.validate()?;
        self.performance.validate()?;
        self.memory.validate()?;

        Ok(())
    }

    /// Load configuration from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: TablebaseConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.validate()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from a JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        let config: TablebaseConfig =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    /// Convert configuration to JSON string
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;

        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

    /// Merge with another configuration, taking values from other where they exist
    pub fn merge_with(&mut self, other: &TablebaseConfig) {
        self.enabled = other.enabled;
        self.cache_size = other.cache_size;
        self.max_depth = other.max_depth;
        self.confidence_threshold = other.confidence_threshold;
        self.solvers.merge_with(&other.solvers);
        self.performance.merge_with(&other.performance);
        self.memory.merge_with(&other.memory);
    }
}

/// Configuration for individual solvers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// King + Gold vs King solver configuration
    pub king_gold_vs_king: KingGoldConfig,
    /// King + Silver vs King solver configuration
    pub king_silver_vs_king: KingSilverConfig,
    /// King + Rook vs King solver configuration
    pub king_rook_vs_king: KingRookConfig,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            king_gold_vs_king: KingGoldConfig::default(),
            king_silver_vs_king: KingSilverConfig::default(),
            king_rook_vs_king: KingRookConfig::default(),
        }
    }
}

impl SolverConfig {
    /// Create a performance-optimized solver configuration
    pub fn performance_optimized() -> Self {
        Self {
            king_gold_vs_king: KingGoldConfig::performance_optimized(),
            king_silver_vs_king: KingSilverConfig::performance_optimized(),
            king_rook_vs_king: KingRookConfig::performance_optimized(),
        }
    }

    /// Create a memory-optimized solver configuration
    pub fn memory_optimized() -> Self {
        Self {
            king_gold_vs_king: KingGoldConfig::memory_optimized(),
            king_silver_vs_king: KingSilverConfig::memory_optimized(),
            king_rook_vs_king: KingRookConfig::memory_optimized(),
        }
    }

    /// Validate the solver configuration
    pub fn validate(&self) -> Result<(), String> {
        self.king_gold_vs_king.validate()?;
        self.king_silver_vs_king.validate()?;
        self.king_rook_vs_king.validate()?;
        Ok(())
    }

    /// Merge with another solver configuration
    pub fn merge_with(&mut self, other: &SolverConfig) {
        self.king_gold_vs_king.merge_with(&other.king_gold_vs_king);
        self.king_silver_vs_king
            .merge_with(&other.king_silver_vs_king);
        self.king_rook_vs_king.merge_with(&other.king_rook_vs_king);
    }
}

/// Configuration for King + Gold vs King solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingGoldConfig {
    /// Whether this solver is enabled
    pub enabled: bool,
    /// Maximum moves to mate this solver can handle
    pub max_moves_to_mate: u8,
    /// Whether to use pattern matching optimization
    pub use_pattern_matching: bool,
    /// Size of pattern cache
    pub pattern_cache_size: usize,
    /// Priority of this solver
    pub priority: u8,
}

impl Default for KingGoldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 20,
            use_pattern_matching: true,
            pattern_cache_size: 1000,
            priority: 100,
        }
    }
}

impl KingGoldConfig {
    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 30,
            use_pattern_matching: true,
            pattern_cache_size: 5000,
            priority: 100,
        }
    }

    /// Create a memory-optimized configuration
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 10,
            use_pattern_matching: false,
            pattern_cache_size: 100,
            priority: 100,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_moves_to_mate == 0 {
            return Err("Max moves to mate must be greater than 0".to_string());
        }
        if self.pattern_cache_size == 0 {
            return Err("Pattern cache size must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Merge with another configuration
    pub fn merge_with(&mut self, other: &KingGoldConfig) {
        self.enabled = other.enabled;
        self.max_moves_to_mate = other.max_moves_to_mate;
        self.use_pattern_matching = other.use_pattern_matching;
        self.pattern_cache_size = other.pattern_cache_size;
        self.priority = other.priority;
    }
}

/// Configuration for King + Silver vs King solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingSilverConfig {
    /// Whether this solver is enabled
    pub enabled: bool,
    /// Maximum moves to mate this solver can handle
    pub max_moves_to_mate: u8,
    /// Whether to use pattern matching optimization
    pub use_pattern_matching: bool,
    /// Size of pattern cache
    pub pattern_cache_size: usize,
    /// Priority of this solver
    pub priority: u8,
}

impl Default for KingSilverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 25,
            use_pattern_matching: true,
            pattern_cache_size: 800,
            priority: 90,
        }
    }
}

impl KingSilverConfig {
    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 35,
            use_pattern_matching: true,
            pattern_cache_size: 4000,
            priority: 90,
        }
    }

    /// Create a memory-optimized configuration
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 12,
            use_pattern_matching: false,
            pattern_cache_size: 80,
            priority: 90,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_moves_to_mate == 0 {
            return Err("Max moves to mate must be greater than 0".to_string());
        }
        if self.pattern_cache_size == 0 {
            return Err("Pattern cache size must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Merge with another configuration
    pub fn merge_with(&mut self, other: &KingSilverConfig) {
        self.enabled = other.enabled;
        self.max_moves_to_mate = other.max_moves_to_mate;
        self.use_pattern_matching = other.use_pattern_matching;
        self.pattern_cache_size = other.pattern_cache_size;
        self.priority = other.priority;
    }
}

/// Configuration for King + Rook vs King solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingRookConfig {
    /// Whether this solver is enabled
    pub enabled: bool,
    /// Maximum moves to mate this solver can handle
    pub max_moves_to_mate: u8,
    /// Whether to use pattern matching optimization
    pub use_pattern_matching: bool,
    /// Size of pattern cache
    pub pattern_cache_size: usize,
    /// Priority of this solver
    pub priority: u8,
}

impl Default for KingRookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 30,
            use_pattern_matching: true,
            pattern_cache_size: 1200,
            priority: 80,
        }
    }
}

impl KingRookConfig {
    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 40,
            use_pattern_matching: true,
            pattern_cache_size: 6000,
            priority: 80,
        }
    }

    /// Create a memory-optimized configuration
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            max_moves_to_mate: 15,
            use_pattern_matching: false,
            pattern_cache_size: 120,
            priority: 80,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_moves_to_mate == 0 {
            return Err("Max moves to mate must be greater than 0".to_string());
        }
        if self.pattern_cache_size == 0 {
            return Err("Pattern cache size must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Merge with another configuration
    pub fn merge_with(&mut self, other: &KingRookConfig) {
        self.enabled = other.enabled;
        self.max_moves_to_mate = other.max_moves_to_mate;
        self.use_pattern_matching = other.use_pattern_matching;
        self.pattern_cache_size = other.pattern_cache_size;
        self.priority = other.priority;
    }
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    /// Whether to enable adaptive caching
    pub enable_adaptive_caching: bool,
    /// Cache eviction strategy
    pub eviction_strategy: EvictionStrategy,
    /// Maximum probe time in milliseconds
    pub max_probe_time_ms: u64,
    /// Whether to enable parallel solving
    pub enable_parallel_solving: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            enable_adaptive_caching: true,
            eviction_strategy: EvictionStrategy::Random,
            max_probe_time_ms: 100,
            enable_parallel_solving: false,
        }
    }
}

impl PerformanceConfig {
    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enable_monitoring: true,
            enable_adaptive_caching: true,
            eviction_strategy: EvictionStrategy::LRU,
            max_probe_time_ms: 50,
            enable_parallel_solving: true,
        }
    }

    /// Create a memory-optimized configuration
    pub fn memory_optimized() -> Self {
        Self {
            enable_monitoring: false,
            enable_adaptive_caching: false,
            eviction_strategy: EvictionStrategy::Random,
            max_probe_time_ms: 200,
            enable_parallel_solving: false,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_probe_time_ms == 0 {
            return Err("Max probe time must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Merge with another configuration
    pub fn merge_with(&mut self, other: &PerformanceConfig) {
        self.enable_monitoring = other.enable_monitoring;
        self.enable_adaptive_caching = other.enable_adaptive_caching;
        self.eviction_strategy = other.eviction_strategy;
        self.max_probe_time_ms = other.max_probe_time_ms;
        self.enable_parallel_solving = other.enable_parallel_solving;
    }
}

/// Memory usage monitoring and limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Whether memory monitoring is enabled
    pub enable_monitoring: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Memory usage warning threshold (0.0 to 1.0)
    pub warning_threshold: f32,
    /// Memory usage critical threshold (0.0 to 1.0)
    pub critical_threshold: f32,
    /// Whether to enable automatic cache eviction when memory limit is reached
    pub enable_auto_eviction: bool,
    /// Memory usage check interval in milliseconds
    pub check_interval_ms: u64,
    /// Whether to log memory usage statistics
    pub enable_logging: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB default
            warning_threshold: 0.7,             // 70%
            critical_threshold: 0.9,            // 90%
            enable_auto_eviction: true,
            check_interval_ms: 1000, // 1 second
            enable_logging: false,
        }
    }
}

impl MemoryConfig {
    /// Create a memory-optimized configuration
    pub fn memory_optimized() -> Self {
        Self {
            enable_monitoring: true,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            warning_threshold: 0.5,             // 50%
            critical_threshold: 0.8,            // 80%
            enable_auto_eviction: true,
            check_interval_ms: 500, // 0.5 seconds
            enable_logging: true,
        }
    }

    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enable_monitoring: false,
            max_memory_bytes: 200 * 1024 * 1024, // 200MB
            warning_threshold: 0.8,              // 80%
            critical_threshold: 0.95,            // 95%
            enable_auto_eviction: true,
            check_interval_ms: 5000, // 5 seconds
            enable_logging: false,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.warning_threshold >= self.critical_threshold {
            return Err("Warning threshold must be less than critical threshold".to_string());
        }
        if self.warning_threshold < 0.0 || self.warning_threshold > 1.0 {
            return Err("Warning threshold must be between 0.0 and 1.0".to_string());
        }
        if self.critical_threshold < 0.0 || self.critical_threshold > 1.0 {
            return Err("Critical threshold must be between 0.0 and 1.0".to_string());
        }
        if self.max_memory_bytes == 0 {
            return Err("Max memory must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Merge with another configuration
    pub fn merge_with(&mut self, other: &MemoryConfig) {
        self.enable_monitoring = other.enable_monitoring;
        self.max_memory_bytes = other.max_memory_bytes;
        self.warning_threshold = other.warning_threshold;
        self.critical_threshold = other.critical_threshold;
        self.enable_auto_eviction = other.enable_auto_eviction;
        self.check_interval_ms = other.check_interval_ms;
        self.enable_logging = other.enable_logging;
    }
}

/// Cache eviction strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionStrategy {
    /// Random eviction (fastest)
    Random,
    /// Least Recently Used eviction
    LRU,
    /// Least Frequently Used eviction
    LFU,
}

/// Statistics for monitoring tablebase performance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TablebaseStats {
    /// Total number of tablebase probes
    pub total_probes: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of solver hits
    pub solver_hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Breakdown of hits by solver name
    pub solver_breakdown: HashMap<String, u64>,
    /// Average probe time in milliseconds
    pub average_probe_time_ms: f64,
    /// Total probe time in milliseconds
    pub total_probe_time_ms: u64,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Number of memory warnings triggered
    pub memory_warnings: u64,
    /// Number of memory critical alerts triggered
    pub memory_critical_alerts: u64,
    /// Number of automatic evictions due to memory pressure
    pub auto_evictions: u64,
    /// Total time spent in position analysis (ms)
    pub position_analysis_time_ms: u64,
    /// Number of position analysis invocations
    pub position_analysis_calls: u64,
    /// Total time spent selecting solvers (ms)
    pub solver_selection_time_ms: u64,
    /// Number of solver selection invocations
    pub solver_selection_calls: u64,
}

impl TablebaseStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self {
            total_probes: 0,
            cache_hits: 0,
            cache_misses: 0,
            solver_hits: 0,
            misses: 0,
            solver_breakdown: HashMap::new(),
            average_probe_time_ms: 0.0,
            total_probe_time_ms: 0,
            current_memory_bytes: 0,
            peak_memory_bytes: 0,
            memory_warnings: 0,
            memory_critical_alerts: 0,
            auto_evictions: 0,
            position_analysis_time_ms: 0,
            position_analysis_calls: 0,
            solver_selection_time_ms: 0,
            solver_selection_calls: 0,
        }
    }

    /// Record a probe with timing information
    pub fn record_probe(
        &mut self,
        cache_hit: bool,
        solver_hit: bool,
        solver_name: Option<&str>,
        probe_time_ms: u64,
    ) {
        self.total_probes += 1;
        self.total_probe_time_ms += probe_time_ms;
        self.average_probe_time_ms = self.total_probe_time_ms as f64 / self.total_probes as f64;

        if cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        if solver_hit {
            self.solver_hits += 1;
            if let Some(name) = solver_name {
                *self.solver_breakdown.entry(name.to_string()).or_insert(0) += 1;
            }
        } else {
            self.misses += 1;
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.total_probes = 0;
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.solver_hits = 0;
        self.misses = 0;
        self.solver_breakdown.clear();
        self.average_probe_time_ms = 0.0;
        self.total_probe_time_ms = 0;
        self.current_memory_bytes = 0;
        self.peak_memory_bytes = 0;
        self.memory_warnings = 0;
        self.memory_critical_alerts = 0;
        self.auto_evictions = 0;
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_probes as f64
        }
    }

    /// Get solver hit rate
    pub fn solver_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            self.solver_hits as f64 / self.total_probes as f64
        }
    }

    /// Get overall hit rate
    pub fn overall_hit_rate(&self) -> f64 {
        if self.total_probes == 0 {
            0.0
        } else {
            (self.cache_hits + self.solver_hits) as f64 / self.total_probes as f64
        }
    }

    /// Get average probe time in milliseconds
    pub fn average_probe_time(&self) -> f64 {
        self.average_probe_time_ms
    }

    /// Get total probe time in milliseconds
    pub fn total_probe_time(&self) -> u64 {
        self.total_probe_time_ms
    }

    /// Get solver breakdown as a percentage
    pub fn solver_breakdown_percentages(&self) -> HashMap<String, f64> {
        let mut percentages = HashMap::new();
        if self.solver_hits > 0 {
            for (solver, hits) in &self.solver_breakdown {
                percentages.insert(
                    solver.clone(),
                    (*hits as f64 / self.solver_hits as f64) * 100.0,
                );
            }
        }
        percentages
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Tablebase Performance Summary:\n\
            Total Probes: {}\n\
            Cache Hit Rate: {:.2}%\n\
            Solver Hit Rate: {:.2}%\n\
            Overall Hit Rate: {:.2}%\n\
            Average Probe Time: {:.2}ms\n\
            Total Probe Time: {}ms\n\
            Avg Position Analysis Time: {:.2}ms\n\
            Avg Solver Selection Time: {:.2}ms",
            self.total_probes,
            self.cache_hit_rate() * 100.0,
            self.solver_hit_rate() * 100.0,
            self.overall_hit_rate() * 100.0,
            self.average_probe_time_ms,
            self.total_probe_time_ms,
            self.average_position_analysis_time(),
            self.average_solver_selection_time()
        )
    }

    /// Update memory usage statistics
    pub fn update_memory_usage(&mut self, current_bytes: usize) {
        self.current_memory_bytes = current_bytes;
        if current_bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = current_bytes;
        }
    }

    /// Record a memory warning
    pub fn record_memory_warning(&mut self) {
        self.memory_warnings += 1;
    }

    /// Record a memory critical alert
    pub fn record_memory_critical_alert(&mut self) {
        self.memory_critical_alerts += 1;
    }

    /// Record position analysis timing
    pub fn record_position_analysis_time(&mut self, time_ms: u64) {
        self.position_analysis_calls += 1;
        self.position_analysis_time_ms += time_ms;
    }

    /// Record solver selection timing
    pub fn record_solver_selection_time(&mut self, time_ms: u64) {
        self.solver_selection_calls += 1;
        self.solver_selection_time_ms += time_ms;
    }

    /// Average time spent analyzing positions
    pub fn average_position_analysis_time(&self) -> f64 {
        if self.position_analysis_calls == 0 {
            0.0
        } else {
            self.position_analysis_time_ms as f64 / self.position_analysis_calls as f64
        }
    }

    /// Average time spent selecting solvers
    pub fn average_solver_selection_time(&self) -> f64 {
        if self.solver_selection_calls == 0 {
            0.0
        } else {
            self.solver_selection_time_ms as f64 / self.solver_selection_calls as f64
        }
    }

    /// Record an automatic eviction due to memory pressure
    pub fn record_auto_eviction(&mut self) {
        self.auto_evictions += 1;
    }

    /// Get memory usage as a percentage of peak
    pub fn memory_usage_percentage(&self, max_memory: usize) -> f64 {
        if max_memory == 0 {
            0.0
        } else {
            (self.current_memory_bytes as f64 / max_memory as f64) * 100.0
        }
    }

    /// Get memory usage summary
    pub fn memory_summary(&self, max_memory: usize) -> String {
        format!(
            "Memory Usage Summary:\n\
            Current Memory: {} bytes ({:.2}% of max)\n\
            Peak Memory: {} bytes\n\
            Memory Warnings: {}\n\
            Critical Alerts: {}\n\
            Auto Evictions: {}",
            self.current_memory_bytes,
            self.memory_usage_percentage(max_memory),
            self.peak_memory_bytes,
            self.memory_warnings,
            self.memory_critical_alerts,
            self.auto_evictions
        )
    }

    /// Check if memory usage is above warning threshold
    pub fn is_memory_warning(&self, max_memory: usize, warning_threshold: f32) -> bool {
        self.memory_usage_percentage(max_memory) >= (warning_threshold * 100.0) as f64
    }

    /// Check if memory usage is above critical threshold
    pub fn is_memory_critical(&self, max_memory: usize, critical_threshold: f32) -> bool {
        self.memory_usage_percentage(max_memory) >= (critical_threshold * 100.0) as f64
    }
}

impl Default for TablebaseStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tablebase_config_default() {
        let config = TablebaseConfig::default();
        assert!(config.enabled);
        assert_eq!(config.cache_size, 10000);
        assert_eq!(config.max_depth, 20);
        assert_eq!(config.confidence_threshold, 0.8);
    }

    #[test]
    fn test_tablebase_config_validation() {
        let config = TablebaseConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = TablebaseConfig::default();
        invalid_config.cache_size = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = TablebaseConfig::default();
        invalid_config.confidence_threshold = 1.5;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_solver_config_default() {
        let config = SolverConfig::default();
        assert!(config.king_gold_vs_king.enabled);
        assert_eq!(config.king_gold_vs_king.priority, 100);
        assert!(config.king_silver_vs_king.enabled);
        assert_eq!(config.king_silver_vs_king.priority, 90);
        assert!(config.king_rook_vs_king.enabled);
        assert_eq!(config.king_rook_vs_king.priority, 80);
    }

    #[test]
    fn test_performance_config() {
        let config = PerformanceConfig::default();
        assert!(config.enable_monitoring);
        assert!(config.enable_adaptive_caching);
        assert_eq!(config.eviction_strategy, EvictionStrategy::Random);

        let perf_config = PerformanceConfig::performance_optimized();
        assert!(perf_config.enable_parallel_solving);
        assert_eq!(perf_config.eviction_strategy, EvictionStrategy::LRU);

        let mem_config = PerformanceConfig::memory_optimized();
        assert!(!mem_config.enable_monitoring);
        assert!(!mem_config.enable_parallel_solving);
    }

    #[test]
    fn test_tablebase_stats() {
        let mut stats = TablebaseStats::new();
        assert_eq!(stats.total_probes, 0);
        assert_eq!(stats.cache_hit_rate(), 0.0);

        stats.total_probes = 100;
        stats.cache_hits = 30;
        stats.solver_hits = 20;
        stats.misses = 50;

        assert_eq!(stats.cache_hit_rate(), 0.3);
        assert_eq!(stats.solver_hit_rate(), 0.2);
        assert_eq!(stats.overall_hit_rate(), 0.5);
    }

    #[test]
    fn test_tablebase_stats_record_probe() {
        let mut stats = TablebaseStats::new();

        // Record a cache hit
        stats.record_probe(true, false, None, 5);
        assert_eq!(stats.total_probes, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.solver_hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.average_probe_time_ms, 5.0);

        // Record a solver hit
        stats.record_probe(false, true, Some("KingGoldVsKing"), 10);
        assert_eq!(stats.total_probes, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.solver_hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.average_probe_time_ms, 7.5);
        assert_eq!(stats.solver_breakdown.get("KingGoldVsKing"), Some(&1));
    }

    #[test]
    fn test_tablebase_stats_reset() {
        let mut stats = TablebaseStats::new();
        stats.record_probe(true, true, Some("KingGoldVsKing"), 5);
        stats.record_probe(false, false, None, 10);

        assert_eq!(stats.total_probes, 2);
        assert!(!stats.solver_breakdown.is_empty());

        stats.reset();
        assert_eq!(stats.total_probes, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.solver_hits, 0);
        assert_eq!(stats.misses, 0);
        assert!(stats.solver_breakdown.is_empty());
        assert_eq!(stats.average_probe_time_ms, 0.0);
    }

    #[test]
    fn test_tablebase_stats_performance_summary() {
        let mut stats = TablebaseStats::new();
        stats.record_probe(true, false, None, 5);
        stats.record_probe(false, true, Some("KingGoldVsKing"), 10);

        let summary = stats.performance_summary();
        assert!(summary.contains("Total Probes: 2"));
        assert!(summary.contains("Cache Hit Rate: 50.00%"));
        assert!(summary.contains("Solver Hit Rate: 50.00%"));
        assert!(summary.contains("Overall Hit Rate: 100.00%"));
    }

    #[test]
    fn test_tablebase_config_json_serialization() {
        let config = TablebaseConfig::performance_optimized();

        // Test JSON serialization
        let json = config.to_json().unwrap();
        assert!(json.contains("enabled"));
        assert!(json.contains("cache_size"));
        assert!(json.contains("performance"));

        // Test JSON deserialization
        let parsed_config = TablebaseConfig::from_json(&json).unwrap();
        assert_eq!(config.enabled, parsed_config.enabled);
        assert_eq!(config.cache_size, parsed_config.cache_size);
        assert_eq!(config.max_depth, parsed_config.max_depth);
    }

    #[test]
    fn test_tablebase_config_merge() {
        let mut base_config = TablebaseConfig::default();
        let mut override_config = TablebaseConfig::default();
        override_config.cache_size = 20000;
        override_config.max_depth = 15;
        override_config.solvers.king_gold_vs_king.enabled = false;

        base_config.merge_with(&override_config);

        assert_eq!(base_config.cache_size, 20000);
        assert_eq!(base_config.max_depth, 15);
        assert!(!base_config.solvers.king_gold_vs_king.enabled);
    }

    #[test]
    fn test_tablebase_config_validation_edge_cases() {
        // Test invalid cache size
        let mut config = TablebaseConfig::default();
        config.cache_size = 0;
        assert!(config.validate().is_err());

        // Test invalid max depth
        let mut config = TablebaseConfig::default();
        config.max_depth = 0;
        assert!(config.validate().is_err());

        // Test invalid confidence threshold
        let mut config = TablebaseConfig::default();
        config.confidence_threshold = 1.5;
        assert!(config.validate().is_err());

        config.confidence_threshold = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_solver_config_merge() {
        let mut base_config = KingGoldConfig::default();
        let mut override_config = KingGoldConfig::default();
        override_config.enabled = false;
        override_config.max_moves_to_mate = 50;
        override_config.priority = 200;

        base_config.merge_with(&override_config);

        assert!(!base_config.enabled);
        assert_eq!(base_config.max_moves_to_mate, 50);
        assert_eq!(base_config.priority, 200);
    }

    #[test]
    fn test_performance_config_merge() {
        let mut base_config = PerformanceConfig::default();
        let mut override_config = PerformanceConfig::default();
        override_config.enable_monitoring = false;
        override_config.eviction_strategy = EvictionStrategy::LRU;
        override_config.max_probe_time_ms = 500;

        base_config.merge_with(&override_config);

        assert!(!base_config.enable_monitoring);
        assert_eq!(base_config.eviction_strategy, EvictionStrategy::LRU);
        assert_eq!(base_config.max_probe_time_ms, 500);
    }
}
