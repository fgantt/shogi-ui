//! Performance monitoring and automatic optimization for magic bitboards
//!
//! This module provides runtime performance monitoring and adaptive optimization
//! for magic bitboard operations.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Performance monitor for magic bitboard operations
#[derive(Clone)]
pub struct PerformanceMonitor {
    /// Total number of lookups performed
    total_lookups: Arc<AtomicU64>,
    /// Total time spent in lookups (nanoseconds)
    total_lookup_time_ns: Arc<AtomicU64>,
    /// Number of cache hits
    cache_hits: Arc<AtomicU64>,
    /// Number of cache misses
    cache_misses: Arc<AtomicU64>,
    /// Monitoring enabled flag
    enabled: Arc<AtomicBool>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            total_lookups: Arc::new(AtomicU64::new(0)),
            total_lookup_time_ns: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Create with monitoring disabled
    pub fn disabled() -> Self {
        let monitor = Self::new();
        monitor.enabled.store(false, Ordering::Relaxed);
        monitor
    }

    /// Record a lookup operation
    pub fn record_lookup(&self, duration: Duration) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.total_lookups.fetch_add(1, Ordering::Relaxed);
        self.total_lookup_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current statistics
    pub fn stats(&self) -> MonitorStats {
        let total_lookups = self.total_lookups.load(Ordering::Relaxed);
        let total_time_ns = self.total_lookup_time_ns.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);

        MonitorStats {
            total_lookups,
            average_lookup_time: if total_lookups > 0 {
                Duration::from_nanos(total_time_ns / total_lookups)
            } else {
                Duration::ZERO
            },
            cache_hit_rate: if cache_hits + cache_misses > 0 {
                cache_hits as f64 / (cache_hits + cache_misses) as f64
            } else {
                0.0
            },
            cache_hits,
            cache_misses,
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.total_lookups.store(0, Ordering::Relaxed);
        self.total_lookup_time_ns.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }

    /// Enable monitoring
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable monitoring
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct MonitorStats {
    pub total_lookups: u64,
    pub average_lookup_time: Duration,
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl MonitorStats {
    /// Get lookups per second estimate
    pub fn lookups_per_second(&self) -> f64 {
        if self.average_lookup_time.as_nanos() > 0 {
            1_000_000_000.0 / self.average_lookup_time.as_nanos() as f64
        } else {
            0.0
        }
    }

    /// Check if performance is optimal
    pub fn is_optimal(&self) -> bool {
        // Performance is optimal if:
        // 1. Average lookup time < 100ns
        // 2. Cache hit rate > 80% (if caching is used)
        self.average_lookup_time.as_nanos() < 100
            && (self.cache_hit_rate > 0.8 || self.cache_hits + self.cache_misses == 0)
    }

    /// Get performance grade
    pub fn grade(&self) -> PerformanceGrade {
        let lookup_time_ns = self.average_lookup_time.as_nanos();

        if lookup_time_ns < 50 {
            PerformanceGrade::Excellent
        } else if lookup_time_ns < 100 {
            PerformanceGrade::Good
        } else if lookup_time_ns < 500 {
            PerformanceGrade::Fair
        } else {
            PerformanceGrade::Poor
        }
    }
}

/// Performance grade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceGrade {
    Excellent, // < 50ns average
    Good,      // < 100ns average
    Fair,      // < 500ns average
    Poor,      // >= 500ns average
}

/// Adaptive optimizer that adjusts settings based on performance
pub struct AdaptiveOptimizer {
    monitor: PerformanceMonitor,
    optimization_threshold: Duration,
    check_interval: u64,
}

impl AdaptiveOptimizer {
    /// Create a new adaptive optimizer
    pub fn new(monitor: PerformanceMonitor) -> Self {
        Self {
            monitor,
            optimization_threshold: Duration::from_nanos(100),
            check_interval: 10_000,
        }
    }

    /// Check if optimization should be triggered
    pub fn should_optimize(&self) -> bool {
        let stats = self.monitor.stats();

        // Optimize if:
        // 1. We have enough data
        // 2. Performance is below threshold
        stats.total_lookups > self.check_interval
            && stats.average_lookup_time > self.optimization_threshold
    }

    /// Get optimization recommendations
    pub fn recommendations(&self) -> Vec<OptimizationRecommendation> {
        let stats = self.monitor.stats();
        let mut recommendations = Vec::new();

        // Check cache performance
        if stats.cache_misses > stats.cache_hits && stats.cache_hits + stats.cache_misses > 1000 {
            recommendations.push(OptimizationRecommendation::IncreaseCacheSize);
        }

        // Check lookup time
        if stats.average_lookup_time.as_nanos() > 500 {
            recommendations.push(OptimizationRecommendation::OptimizeLookupAlgorithm);
        }

        // Check if caching would help
        if stats.cache_hits + stats.cache_misses == 0 && stats.total_lookups > 10_000 {
            recommendations.push(OptimizationRecommendation::EnableCaching);
        }

        recommendations
    }
}

/// Optimization recommendations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationRecommendation {
    IncreaseCacheSize,
    OptimizeLookupAlgorithm,
    EnableCaching,
    EnablePrefetching,
    EnableSIMD,
    ReduceMemoryUsage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(monitor.is_enabled());

        let stats = monitor.stats();
        assert_eq!(stats.total_lookups, 0);
    }

    #[test]
    fn test_record_lookup() {
        let monitor = PerformanceMonitor::new();

        monitor.record_lookup(Duration::from_nanos(50));
        monitor.record_lookup(Duration::from_nanos(60));

        let stats = monitor.stats();
        assert_eq!(stats.total_lookups, 2);
        assert!(stats.average_lookup_time.as_nanos() > 0);
    }

    #[test]
    fn test_cache_tracking() {
        let monitor = PerformanceMonitor::new();

        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_miss();

        let stats = monitor.stats();
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert!((stats.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_monitor_enable_disable() {
        let monitor = PerformanceMonitor::new();

        monitor.record_lookup(Duration::from_nanos(100));
        assert_eq!(monitor.stats().total_lookups, 1);

        monitor.disable();
        monitor.record_lookup(Duration::from_nanos(100));
        assert_eq!(monitor.stats().total_lookups, 1); // Should not increase

        monitor.enable();
        monitor.record_lookup(Duration::from_nanos(100));
        assert_eq!(monitor.stats().total_lookups, 2); // Should increase
    }

    #[test]
    fn test_performance_grade() {
        let monitor = PerformanceMonitor::new();

        monitor.record_lookup(Duration::from_nanos(30));
        let stats = monitor.stats();

        assert_eq!(stats.grade(), PerformanceGrade::Excellent);
    }

    #[test]
    fn test_adaptive_optimizer() {
        let monitor = PerformanceMonitor::new();
        let optimizer = AdaptiveOptimizer::new(monitor.clone());

        // Record poor performance
        for _ in 0..20_000 {
            monitor.record_lookup(Duration::from_nanos(600));
        }

        assert!(optimizer.should_optimize(), "Should trigger optimization");

        let recommendations = optimizer.recommendations();
        assert!(!recommendations.is_empty(), "Should have recommendations");
    }
}
