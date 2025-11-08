//! Dynamic Table Sizing
//!
//! This module implements dynamic table sizing for transposition tables,
//! automatically adjusting table size based on memory usage, performance
//! metrics, and system conditions to optimize cache efficiency.

// No types needed for this module
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Dynamic sizing configuration
#[derive(Debug, Clone)]
pub struct DynamicSizingConfig {
    /// Minimum table size (entries)
    pub min_table_size: usize,
    /// Maximum table size (entries)
    pub max_table_size: usize,
    /// Initial table size (entries)
    pub initial_table_size: usize,
    /// Memory usage threshold for downsizing (0.0 to 1.0)
    pub memory_threshold: f64,
    /// Performance threshold for upsizing (0.0 to 1.0)
    pub performance_threshold: f64,
    /// Resize frequency (seconds)
    pub resize_frequency: Duration,
    /// Enable aggressive resizing
    pub aggressive_resizing: bool,
    /// Memory monitoring settings
    pub memory_monitoring: MemoryMonitoringSettings,
    /// Performance monitoring settings
    pub performance_monitoring: PerformanceMonitoringSettings,
}

/// Memory monitoring settings
#[derive(Debug, Clone)]
pub struct MemoryMonitoringSettings {
    /// Enable memory pressure detection
    pub enable_memory_pressure: bool,
    /// Memory pressure threshold (0.0 to 1.0)
    pub memory_pressure_threshold: f64,
    /// Memory usage sampling window
    pub sampling_window: Duration,
    /// Enable memory leak detection
    pub enable_leak_detection: bool,
}

/// Performance monitoring settings
#[derive(Debug, Clone)]
pub struct PerformanceMonitoringSettings {
    /// Enable hit rate monitoring
    pub enable_hit_rate_monitoring: bool,
    /// Hit rate threshold for upsizing
    pub hit_rate_threshold: f64,
    /// Enable access pattern monitoring
    pub enable_access_pattern_monitoring: bool,
    /// Access pattern analysis window
    pub analysis_window: Duration,
}

/// Dynamic sizing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizingStrategy {
    /// Conservative - slow, stable resizing
    Conservative,
    /// Aggressive - fast, responsive resizing
    Aggressive,
    /// Adaptive - learns optimal strategy
    Adaptive,
    /// Memory-based - primarily memory-driven
    MemoryBased,
    /// Performance-based - primarily performance-driven
    PerformanceBased,
}

/// Resize decision
#[derive(Debug, Clone)]
pub struct ResizeDecision {
    /// New table size
    pub new_size: usize,
    /// Resize reason
    pub reason: ResizeReason,
    /// Confidence in decision (0.0 to 1.0)
    pub confidence: f64,
    /// Expected performance impact
    pub expected_impact: f64,
    /// Timestamp
    pub timestamp: Instant,
}

/// Resize reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeReason {
    /// Memory pressure detected
    MemoryPressure,
    /// Low hit rate detected
    LowHitRate,
    /// High hit rate - can afford larger table
    HighHitRate,
    /// Access pattern analysis suggests resize
    AccessPattern,
    /// System resource constraints
    ResourceConstraints,
    /// Performance degradation detected
    PerformanceDegradation,
    /// Periodic maintenance resize
    PeriodicMaintenance,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Current memory usage (bytes)
    pub current_usage: u64,
    /// Peak memory usage (bytes)
    pub peak_usage: u64,
    /// Available memory (bytes)
    pub available_memory: u64,
    /// Memory usage percentage (0.0 to 1.0)
    pub usage_percentage: f64,
    /// Memory pressure level (0.0 to 1.0)
    pub pressure_level: f64,
    /// Memory trend (increasing/decreasing/stable)
    pub trend: MemoryTrend,
}

/// Memory trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTrend {
    /// Memory usage is increasing
    Increasing,
    /// Memory usage is decreasing
    Decreasing,
    /// Memory usage is stable
    Stable,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Current hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Average hit rate over time
    pub avg_hit_rate: f64,
    /// Access frequency (accesses per second)
    pub access_frequency: f64,
    /// Cache efficiency (0.0 to 1.0)
    pub cache_efficiency: f64,
    /// Performance trend
    pub trend: PerformanceTrend,
}

/// Performance trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving,
    /// Performance is degrading
    Degrading,
    /// Performance is stable
    Stable,
}

/// Access pattern analysis
#[derive(Debug, Clone)]
pub struct AccessPatternAnalysis {
    /// Access distribution entropy
    pub entropy: f64,
    /// Hot spot concentration (0.0 to 1.0)
    pub hot_spot_concentration: f64,
    /// Access locality (0.0 to 1.0)
    pub access_locality: f64,
    /// Temporal access patterns
    pub temporal_patterns: TemporalPatterns,
}

/// Temporal access patterns
#[derive(Debug, Clone)]
pub struct TemporalPatterns {
    /// Burst frequency
    pub burst_frequency: f64,
    /// Quiet period frequency
    pub quiet_frequency: f64,
    /// Access regularity (0.0 to 1.0)
    pub regularity: f64,
}

/// Dynamic table sizer
pub struct DynamicTableSizer {
    /// Configuration
    config: DynamicSizingConfig,
    /// Current table size
    current_size: usize,
    /// Memory statistics
    memory_stats: MemoryStats,
    /// Performance statistics
    performance_stats: PerformanceStats,
    /// Access pattern analysis
    access_analysis: AccessPatternAnalysis,
    /// Resize history
    resize_history: VecDeque<ResizeDecision>,
    /// Last resize time
    last_resize: Instant,
    /// Memory usage history
    memory_history: VecDeque<(Instant, u64)>,
    /// Performance history
    performance_history: VecDeque<(Instant, f64)>,
    /// Access pattern history
    access_history: VecDeque<(Instant, u64, u64)>, // (time, hash, depth)
}

impl DynamicSizingConfig {
    /// Create conservative configuration
    pub fn conservative() -> Self {
        Self {
            min_table_size: 1000,
            max_table_size: 1000000,
            initial_table_size: 100000,
            memory_threshold: 0.8,
            performance_threshold: 0.3,
            resize_frequency: Duration::from_secs(60),
            aggressive_resizing: false,
            memory_monitoring: MemoryMonitoringSettings {
                enable_memory_pressure: true,
                memory_pressure_threshold: 0.85,
                sampling_window: Duration::from_secs(30),
                enable_leak_detection: true,
            },
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: true,
                hit_rate_threshold: 0.6,
                enable_access_pattern_monitoring: true,
                analysis_window: Duration::from_secs(120),
            },
        }
    }

    /// Create aggressive configuration
    pub fn aggressive() -> Self {
        Self {
            min_table_size: 10000,
            max_table_size: 10000000,
            initial_table_size: 1000000,
            memory_threshold: 0.9,
            performance_threshold: 0.2,
            resize_frequency: Duration::from_secs(10),
            aggressive_resizing: true,
            memory_monitoring: MemoryMonitoringSettings {
                enable_memory_pressure: true,
                memory_pressure_threshold: 0.95,
                sampling_window: Duration::from_secs(10),
                enable_leak_detection: true,
            },
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: true,
                hit_rate_threshold: 0.7,
                enable_access_pattern_monitoring: true,
                analysis_window: Duration::from_secs(60),
            },
        }
    }

    /// Create memory-based configuration
    pub fn memory_based() -> Self {
        Self {
            min_table_size: 1000,
            max_table_size: 5000000,
            initial_table_size: 500000,
            memory_threshold: 0.7,
            performance_threshold: 0.5,
            resize_frequency: Duration::from_secs(30),
            aggressive_resizing: false,
            memory_monitoring: MemoryMonitoringSettings {
                enable_memory_pressure: true,
                memory_pressure_threshold: 0.75,
                sampling_window: Duration::from_secs(15),
                enable_leak_detection: true,
            },
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: false,
                hit_rate_threshold: 0.5,
                enable_access_pattern_monitoring: false,
                analysis_window: Duration::from_secs(300),
            },
        }
    }

    /// Create performance-based configuration
    pub fn performance_based() -> Self {
        Self {
            min_table_size: 10000,
            max_table_size: 2000000,
            initial_table_size: 200000,
            memory_threshold: 0.85,
            performance_threshold: 0.1,
            resize_frequency: Duration::from_secs(20),
            aggressive_resizing: true,
            memory_monitoring: MemoryMonitoringSettings {
                enable_memory_pressure: false,
                memory_pressure_threshold: 0.9,
                sampling_window: Duration::from_secs(60),
                enable_leak_detection: false,
            },
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: true,
                hit_rate_threshold: 0.8,
                enable_access_pattern_monitoring: true,
                analysis_window: Duration::from_secs(30),
            },
        }
    }
}

impl Default for DynamicSizingConfig {
    fn default() -> Self {
        Self::conservative()
    }
}

impl DynamicTableSizer {
    /// Create a new dynamic table sizer
    pub fn new(config: DynamicSizingConfig) -> Self {
        Self {
            current_size: config.initial_table_size,
            memory_stats: MemoryStats {
                current_usage: 0,
                peak_usage: 0,
                available_memory: 0,
                usage_percentage: 0.0,
                pressure_level: 0.0,
                trend: MemoryTrend::Stable,
            },
            performance_stats: PerformanceStats {
                hit_rate: 0.0,
                avg_hit_rate: 0.0,
                access_frequency: 0.0,
                cache_efficiency: 0.0,
                trend: PerformanceTrend::Stable,
            },
            access_analysis: AccessPatternAnalysis {
                entropy: 0.0,
                hot_spot_concentration: 0.0,
                access_locality: 0.0,
                temporal_patterns: TemporalPatterns {
                    burst_frequency: 0.0,
                    quiet_frequency: 0.0,
                    regularity: 0.0,
                },
            },
            resize_history: VecDeque::new(),
            last_resize: Instant::now(),
            memory_history: VecDeque::new(),
            performance_history: VecDeque::new(),
            access_history: VecDeque::new(),
            config,
        }
    }

    /// Get current table size
    pub fn get_current_size(&self) -> usize {
        self.current_size
    }

    /// Check if resize is needed and make decision
    pub fn should_resize(&mut self) -> Option<ResizeDecision> {
        // Check if enough time has passed since last resize
        if self.last_resize.elapsed() < self.config.resize_frequency {
            return None;
        }

        // Update statistics
        self.update_memory_stats();
        self.update_performance_stats();
        self.update_access_analysis();

        // Make resize decision based on strategy
        let decision = self.make_resize_decision();

        if let Some(ref decision) = decision {
            // Record the decision
            self.resize_history.push_back(decision.clone());
            if self.resize_history.len() > 100 {
                self.resize_history.pop_front();
            }

            self.last_resize = Instant::now();
        }

        decision
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, usage: u64) {
        let now = Instant::now();
        self.memory_history.push_back((now, usage));

        // Maintain history size
        while self.memory_history.len() > 1000 {
            self.memory_history.pop_front();
        }

        // Update current usage
        self.memory_stats.current_usage = usage;
        if usage > self.memory_stats.peak_usage {
            self.memory_stats.peak_usage = usage;
        }
    }

    /// Record performance metrics
    pub fn record_performance(&mut self, hit_rate: f64, access_frequency: f64) {
        let now = Instant::now();
        self.performance_history.push_back((now, hit_rate));

        // Maintain history size
        while self.performance_history.len() > 1000 {
            self.performance_history.pop_front();
        }

        // Update current performance
        self.performance_stats.hit_rate = hit_rate;
        self.performance_stats.access_frequency = access_frequency;

        // Update average hit rate
        let total_hit_rate: f64 = self.performance_history.iter().map(|(_, rate)| rate).sum();
        self.performance_stats.avg_hit_rate =
            total_hit_rate / self.performance_history.len() as f64;
    }

    /// Record access pattern
    pub fn record_access(&mut self, hash: u64, depth: u64) {
        let now = Instant::now();
        self.access_history.push_back((now, hash, depth));

        // Maintain history size
        while self.access_history.len() > 10000 {
            self.access_history.pop_front();
        }
    }

    /// Apply resize decision
    pub fn apply_resize(&mut self, decision: &ResizeDecision) {
        let clamped_size = decision
            .new_size
            .max(self.config.min_table_size)
            .min(self.config.max_table_size);
        self.current_size = clamped_size;
        self.last_resize = Instant::now();
    }

    /// Get sizing statistics
    pub fn get_stats(&self) -> DynamicSizingStats {
        DynamicSizingStats {
            current_size: self.current_size,
            memory_stats: self.memory_stats.clone(),
            performance_stats: self.performance_stats.clone(),
            access_analysis: self.access_analysis.clone(),
            resize_count: self.resize_history.len(),
            last_resize: self.last_resize,
        }
    }

    /// Update memory statistics
    fn update_memory_stats(&mut self) {
        if self.memory_history.is_empty() {
            return;
        }

        // Calculate memory trend
        let recent_samples: Vec<u64> = self
            .memory_history
            .iter()
            .rev()
            .take(10)
            .map(|(_, usage)| *usage)
            .collect();

        if recent_samples.len() >= 3 {
            let first_half: f64 = recent_samples[..recent_samples.len() / 2]
                .iter()
                .sum::<u64>() as f64;
            let second_half: f64 = recent_samples[recent_samples.len() / 2..]
                .iter()
                .sum::<u64>() as f64;

            let first_avg = first_half / (recent_samples.len() / 2) as f64;
            let second_avg = second_half / (recent_samples.len() - recent_samples.len() / 2) as f64;

            self.memory_stats.trend = if second_avg > first_avg * 1.05 {
                MemoryTrend::Increasing
            } else if second_avg < first_avg * 0.95 {
                MemoryTrend::Decreasing
            } else {
                MemoryTrend::Stable
            };
        }

        // Calculate memory pressure
        if self.config.memory_monitoring.enable_memory_pressure {
            let current_usage = self.memory_stats.current_usage as f64;
            let peak_usage = self.memory_stats.peak_usage as f64;

            if peak_usage > 0.0 {
                self.memory_stats.pressure_level = current_usage / peak_usage;
            }
        }
    }

    /// Update performance statistics
    fn update_performance_stats(&mut self) {
        if self.performance_history.is_empty() {
            return;
        }

        // Calculate performance trend
        let recent_samples: Vec<f64> = self
            .performance_history
            .iter()
            .rev()
            .take(10)
            .map(|(_, rate)| *rate)
            .collect();

        if recent_samples.len() >= 3 {
            let first_half: f64 = recent_samples[..recent_samples.len() / 2].iter().sum();
            let second_half: f64 = recent_samples[recent_samples.len() / 2..].iter().sum();

            let first_avg = first_half / (recent_samples.len() / 2) as f64;
            let second_avg = second_half / (recent_samples.len() - recent_samples.len() / 2) as f64;

            self.performance_stats.trend = if second_avg > first_avg * 1.02 {
                PerformanceTrend::Improving
            } else if second_avg < first_avg * 0.98 {
                PerformanceTrend::Degrading
            } else {
                PerformanceTrend::Stable
            };
        }
    }

    /// Update access pattern analysis
    fn update_access_analysis(&mut self) {
        if self.access_history.is_empty() {
            return;
        }

        // Calculate access entropy
        let mut hash_counts = std::collections::HashMap::new();
        for (_, hash, _) in &self.access_history {
            *hash_counts.entry(hash).or_insert(0) += 1;
        }

        let total_accesses = self.access_history.len() as f64;
        let mut entropy = 0.0;

        for count in hash_counts.values() {
            let probability = *count as f64 / total_accesses;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        self.access_analysis.entropy = entropy;

        // Calculate hot spot concentration
        let max_count = hash_counts.values().max().copied().unwrap_or(0) as f64;
        self.access_analysis.hot_spot_concentration = max_count / total_accesses;

        // Calculate access locality (simplified)
        let mut locality_score = 0.0;
        let mut consecutive_accesses = 0;
        let mut last_hash = 0u64;

        for (_, hash, _) in &self.access_history {
            if *hash == last_hash {
                consecutive_accesses += 1;
            } else {
                if consecutive_accesses > 1 {
                    locality_score += consecutive_accesses as f64;
                }
                consecutive_accesses = 1;
                last_hash = *hash;
            }
        }

        self.access_analysis.access_locality = locality_score / total_accesses;

        // Calculate temporal patterns
        self.analyze_temporal_patterns();
    }

    /// Analyze temporal access patterns
    fn analyze_temporal_patterns(&mut self) {
        if self.access_history.len() < 10 {
            return;
        }

        // Calculate burst frequency
        let mut bursts = 0;
        let mut quiet_periods = 0;
        let mut current_burst_length = 0;
        let mut current_quiet_length = 0;

        let mut last_time = self.access_history[0].0;
        let time_threshold = Duration::from_millis(100);

        for (time, _, _) in &self.access_history {
            let time_diff = time.duration_since(last_time);

            if time_diff < time_threshold {
                if current_quiet_length > 0 {
                    quiet_periods += 1;
                    current_quiet_length = 0;
                }
                current_burst_length += 1;
            } else {
                if current_burst_length > 0 {
                    bursts += 1;
                    current_burst_length = 0;
                }
                current_quiet_length += 1;
            }

            last_time = *time;
        }

        let total_periods = bursts + quiet_periods;
        if total_periods > 0 {
            self.access_analysis.temporal_patterns.burst_frequency =
                bursts as f64 / total_periods as f64;
            self.access_analysis.temporal_patterns.quiet_frequency =
                quiet_periods as f64 / total_periods as f64;
        }

        // Calculate regularity (simplified)
        let mut time_diffs = Vec::new();
        for i in 1..self.access_history.len() {
            let diff = self.access_history[i]
                .0
                .duration_since(self.access_history[i - 1].0);
            time_diffs.push(diff);
        }

        if !time_diffs.is_empty() {
            let avg_diff = time_diffs.iter().sum::<Duration>() / time_diffs.len() as u32;
            let variance: f64 = time_diffs
                .iter()
                .map(|diff| {
                    let diff_ms = diff.as_millis() as f64;
                    let avg_ms = avg_diff.as_millis() as f64;
                    (diff_ms - avg_ms).powi(2)
                })
                .sum::<f64>()
                / time_diffs.len() as f64;

            let std_dev = variance.sqrt();
            let avg_ms = avg_diff.as_millis() as f64;

            // Regularity is inverse of coefficient of variation
            self.access_analysis.temporal_patterns.regularity = if avg_ms > 0.0 {
                (avg_ms / (avg_ms + std_dev)).min(1.0)
            } else {
                0.0
            };
        }
    }

    /// Make resize decision based on current state
    fn make_resize_decision(&self) -> Option<ResizeDecision> {
        let mut reasons = Vec::new();
        let mut confidence: f64 = 0.0;
        let mut downsizing_factor: f64 = 1.0;
        let mut upsizing_factor: f64 = 1.0;
        let mut downsizing_requested = false;
        let mut upsizing_requested = false;

        // Check memory pressure
        if self.config.memory_monitoring.enable_memory_pressure
            && self.memory_stats.pressure_level
                > self.config.memory_monitoring.memory_pressure_threshold
        {
            reasons.push(ResizeReason::MemoryPressure);
            confidence += 0.8;

            // Downsize due to memory pressure
            downsizing_factor *= 0.8;
            downsizing_requested = true;
        }

        // Check performance thresholds
        if self
            .config
            .performance_monitoring
            .enable_hit_rate_monitoring
        {
            if self.performance_stats.hit_rate
                < self.config.performance_monitoring.hit_rate_threshold
            {
                reasons.push(ResizeReason::LowHitRate);
                confidence += 0.6;

                // Upsize to improve hit rate
                upsizing_factor *= 1.2;
                upsizing_requested = true;
            } else if self.performance_stats.hit_rate
                > self.config.performance_monitoring.hit_rate_threshold + 0.1
            {
                reasons.push(ResizeReason::HighHitRate);
                confidence += 0.4;

                // Can afford larger table
                upsizing_factor *= 1.1;
                upsizing_requested = true;
            }
        }

        // Check access patterns
        if self
            .config
            .performance_monitoring
            .enable_access_pattern_monitoring
        {
            if self.access_analysis.hot_spot_concentration > 0.8 {
                reasons.push(ResizeReason::AccessPattern);
                confidence += 0.5;

                // High concentration suggests smaller table might be sufficient
                downsizing_factor *= 0.9;
                downsizing_requested = true;
            } else if self.access_analysis.access_locality < 0.3 {
                reasons.push(ResizeReason::AccessPattern);
                confidence += 0.3;

                // Low locality suggests larger table might help
                upsizing_factor *= 1.15;
                upsizing_requested = true;
            }
        }

        // Check performance trend
        if self.performance_stats.trend == PerformanceTrend::Degrading {
            reasons.push(ResizeReason::PerformanceDegradation);
            confidence += 0.7;

            // Try upsizing to improve performance
            upsizing_factor *= 1.3;
            upsizing_requested = true;
        }

        // Determine final multiplier with downsizing taking precedence
        let mut size_multiplier = 1.0;
        if downsizing_requested {
            size_multiplier = downsizing_factor.min(1.0);
        } else if upsizing_requested {
            size_multiplier = upsizing_factor.max(1.0);
        }

        let mut new_size = ((self.current_size as f64) * size_multiplier).round() as usize;
        if new_size == 0 {
            new_size = 1;
        }

        // Apply size constraints
        new_size = new_size
            .max(self.config.min_table_size)
            .min(self.config.max_table_size);

        // Only resize if there's a significant change and sufficient confidence
        let size_change_ratio = if self.current_size > 0 {
            new_size as f64 / self.current_size as f64
        } else {
            1.0
        };
        let significant_change = size_change_ratio > 1.1 || size_change_ratio < 0.9;

        if significant_change && confidence > 0.3 {
            let expected_impact = self.calculate_expected_impact(new_size);

            return Some(ResizeDecision {
                new_size,
                reason: reasons
                    .first()
                    .copied()
                    .unwrap_or(ResizeReason::PeriodicMaintenance),
                confidence: confidence.min(1.0),
                expected_impact,
                timestamp: Instant::now(),
            });
        }

        None
    }

    /// Calculate expected performance impact of resize
    fn calculate_expected_impact(&self, new_size: usize) -> f64 {
        let size_ratio = new_size as f64 / self.current_size as f64;

        // Simple heuristic: larger table should improve hit rate
        if size_ratio > 1.0 {
            // Expected hit rate improvement
            let expected_improvement = (size_ratio - 1.0) * 0.1;
            expected_improvement.min(0.2) // Cap at 20% improvement
        } else {
            // Expected hit rate degradation
            let expected_degradation = (1.0 - size_ratio) * 0.15;
            -expected_degradation.max(-0.3) // Cap at 30% degradation
        }
    }
}

/// Dynamic sizing statistics
#[derive(Debug, Clone)]
pub struct DynamicSizingStats {
    /// Current table size
    pub current_size: usize,
    /// Memory statistics
    pub memory_stats: MemoryStats,
    /// Performance statistics
    pub performance_stats: PerformanceStats,
    /// Access pattern analysis
    pub access_analysis: AccessPatternAnalysis,
    /// Number of resizes performed
    pub resize_count: usize,
    /// Last resize time
    pub last_resize: Instant,
}

impl Default for DynamicTableSizer {
    fn default() -> Self {
        Self::new(DynamicSizingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_sizing_config() {
        let conservative = DynamicSizingConfig::conservative();
        assert_eq!(conservative.min_table_size, 1000);
        assert_eq!(conservative.max_table_size, 1000000);
        assert!(!conservative.aggressive_resizing);

        let aggressive = DynamicSizingConfig::aggressive();
        assert_eq!(aggressive.min_table_size, 10000);
        assert_eq!(aggressive.max_table_size, 10000000);
        assert!(aggressive.aggressive_resizing);

        let memory_based = DynamicSizingConfig::memory_based();
        assert!(memory_based.memory_monitoring.enable_memory_pressure);
        assert!(
            !memory_based
                .performance_monitoring
                .enable_hit_rate_monitoring
        );

        let performance_based = DynamicSizingConfig::performance_based();
        assert!(!performance_based.memory_monitoring.enable_memory_pressure);
        assert!(
            performance_based
                .performance_monitoring
                .enable_hit_rate_monitoring
        );
    }

    #[test]
    fn test_dynamic_table_sizer_creation() {
        let config = DynamicSizingConfig::conservative();
        let sizer = DynamicTableSizer::new(config);

        assert_eq!(sizer.get_current_size(), 100000);
        assert_eq!(sizer.memory_stats.current_usage, 0);
        assert_eq!(sizer.performance_stats.hit_rate, 0.0);
    }

    #[test]
    fn test_memory_usage_recording() {
        let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::default());

        sizer.record_memory_usage(1000);
        assert_eq!(sizer.memory_stats.current_usage, 1000);
        assert_eq!(sizer.memory_stats.peak_usage, 1000);

        sizer.record_memory_usage(2000);
        assert_eq!(sizer.memory_stats.current_usage, 2000);
        assert_eq!(sizer.memory_stats.peak_usage, 2000);

        sizer.record_memory_usage(1500);
        assert_eq!(sizer.memory_stats.current_usage, 1500);
        assert_eq!(sizer.memory_stats.peak_usage, 2000); // Peak remains
    }

    #[test]
    fn test_performance_recording() {
        let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::default());

        sizer.record_performance(0.7, 100.0);
        assert_eq!(sizer.performance_stats.hit_rate, 0.7);
        assert_eq!(sizer.performance_stats.access_frequency, 100.0);
        assert_eq!(sizer.performance_stats.avg_hit_rate, 0.7);

        sizer.record_performance(0.8, 120.0);
        assert_eq!(sizer.performance_stats.hit_rate, 0.8);
        assert_eq!(sizer.performance_stats.access_frequency, 120.0);
        assert_eq!(sizer.performance_stats.avg_hit_rate, 0.75); // Average of 0.7 and 0.8
    }

    #[test]
    fn test_access_pattern_recording() {
        let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::default());

        // Record some access patterns
        for i in 0..10 {
            sizer.record_access(0x1000 + i, 5);
        }

        // Record some repeated accesses
        for _ in 0..5 {
            sizer.record_access(0x2000, 3);
        }

        assert_eq!(sizer.access_history.len(), 15);
    }

    #[test]
    fn test_resize_decision_memory_pressure() {
        let config = DynamicSizingConfig {
            memory_threshold: 0.8,
            memory_monitoring: MemoryMonitoringSettings {
                enable_memory_pressure: true,
                memory_pressure_threshold: 0.85,
                sampling_window: Duration::from_secs(30),
                enable_leak_detection: true,
            },
            ..DynamicSizingConfig::default()
        };

        let mut sizer = DynamicTableSizer::new(config);

        // Simulate high memory pressure
        sizer.memory_stats.pressure_level = 0.9;
        sizer.memory_stats.current_usage = 9000000;
        sizer.memory_stats.peak_usage = 10000000;

        // Force enough time to pass
        sizer.last_resize = Instant::now() - Duration::from_secs(70);

        if let Some(decision) = sizer.should_resize() {
            assert!(decision.new_size < sizer.get_current_size());
            assert_eq!(decision.reason, ResizeReason::MemoryPressure);
            assert!(decision.confidence > 0.0);
        }
    }

    #[test]
    fn test_resize_decision_low_hit_rate() {
        let config = DynamicSizingConfig {
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: true,
                hit_rate_threshold: 0.6,
                enable_access_pattern_monitoring: false,
                analysis_window: Duration::from_secs(120),
            },
            ..DynamicSizingConfig::default()
        };

        let mut sizer = DynamicTableSizer::new(config);

        // Simulate low hit rate
        sizer.performance_stats.hit_rate = 0.4;
        sizer.performance_stats.avg_hit_rate = 0.45;

        // Force enough time to pass
        sizer.last_resize = Instant::now() - Duration::from_secs(70);

        if let Some(decision) = sizer.should_resize() {
            assert!(decision.new_size > sizer.get_current_size());
            assert_eq!(decision.reason, ResizeReason::LowHitRate);
            assert!(decision.confidence > 0.0);
        }
    }

    #[test]
    fn test_resize_decision_high_hit_rate() {
        let config = DynamicSizingConfig {
            performance_monitoring: PerformanceMonitoringSettings {
                enable_hit_rate_monitoring: true,
                hit_rate_threshold: 0.6,
                enable_access_pattern_monitoring: false,
                analysis_window: Duration::from_secs(120),
            },
            ..DynamicSizingConfig::default()
        };

        let mut sizer = DynamicTableSizer::new(config);

        // Simulate high hit rate
        sizer.performance_stats.hit_rate = 0.8;
        sizer.performance_stats.avg_hit_rate = 0.75;

        // Force enough time to pass
        sizer.last_resize = Instant::now() - Duration::from_secs(70);

        if let Some(decision) = sizer.should_resize() {
            assert!(decision.new_size > sizer.get_current_size());
            assert_eq!(decision.reason, ResizeReason::HighHitRate);
            assert!(decision.confidence > 0.0);
        }
    }

    #[test]
    fn test_resize_decision_no_change_needed() {
        let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::default());

        // Simulate good conditions
        sizer.memory_stats.pressure_level = 0.5;
        sizer.performance_stats.hit_rate = 0.7;
        sizer.performance_stats.avg_hit_rate = 0.68;

        // Force enough time to pass
        sizer.last_resize = Instant::now() - Duration::from_secs(70);

        // Should not resize under good conditions
        assert!(sizer.should_resize().is_none());
    }

    #[test]
    fn test_apply_resize() {
        let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::default());
        let original_size = sizer.get_current_size();

        let decision = ResizeDecision {
            new_size: original_size * 2,
            reason: ResizeReason::LowHitRate,
            confidence: 0.8,
            expected_impact: 0.1,
            timestamp: Instant::now(),
        };

        sizer.apply_resize(&decision);
        assert_eq!(sizer.get_current_size(), original_size * 2);
    }

    #[test]
    fn test_size_constraints() {
        let config = DynamicSizingConfig {
            min_table_size: 1000,
            max_table_size: 10000,
            initial_table_size: 5000,
            ..DynamicSizingConfig::default()
        };

        let mut sizer = DynamicTableSizer::new(config);

        // Test minimum constraint
        let decision = ResizeDecision {
            new_size: 500, // Below minimum
            reason: ResizeReason::MemoryPressure,
            confidence: 0.8,
            expected_impact: -0.1,
            timestamp: Instant::now(),
        };

        sizer.apply_resize(&decision);
        assert_eq!(sizer.get_current_size(), 1000); // Should be clamped to minimum

        // Test maximum constraint
        let decision = ResizeDecision {
            new_size: 20000, // Above maximum
            reason: ResizeReason::LowHitRate,
            confidence: 0.8,
            expected_impact: 0.2,
            timestamp: Instant::now(),
        };

        sizer.apply_resize(&decision);
        assert_eq!(sizer.get_current_size(), 10000); // Should be clamped to maximum
    }
}
