//! Performance profiling and monitoring for the tablebase system
//!
//! This module provides comprehensive performance profiling capabilities
//! for monitoring tablebase operations, identifying bottlenecks, and
//! optimizing performance.

use crate::utils::time::TimeSource;
use std::collections::HashMap;
use std::time::Duration;

/// Performance metrics for a specific operation
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Number of times this operation was performed
    pub call_count: u64,
    /// Total time spent in this operation
    pub total_time: Duration,
    /// Average time per call
    pub average_time: Duration,
    /// Minimum time for a single call
    pub min_time: Duration,
    /// Maximum time for a single call
    pub max_time: Duration,
    /// Standard deviation of call times
    pub std_deviation: Duration,
    /// Time spent in different phases of the operation
    pub phase_times: HashMap<String, Duration>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            call_count: 0,
            total_time: Duration::ZERO,
            average_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            std_deviation: Duration::ZERO,
            phase_times: HashMap::new(),
        }
    }

    /// Record a new operation timing
    pub fn record_call(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_time += duration;

        if duration < self.min_time {
            self.min_time = duration;
        }
        if duration > self.max_time {
            self.max_time = duration;
        }

        self.average_time = self.total_time / self.call_count as u32;
        self.update_std_deviation();
    }

    /// Record time for a specific phase
    pub fn record_phase(&mut self, phase_name: String, duration: Duration) {
        *self.phase_times.entry(phase_name).or_insert(Duration::ZERO) += duration;
    }

    /// Update standard deviation calculation
    fn update_std_deviation(&mut self) {
        // Simplified standard deviation calculation
        // In a real implementation, we'd store individual timings
        if self.call_count > 1 {
            let variance = (self.max_time - self.min_time).as_nanos() as f64 / 4.0;
            self.std_deviation = Duration::from_nanos(variance.sqrt() as u64);
        }
    }

    /// Get performance summary as a string
    pub fn summary(&self) -> String {
        format!(
            "Calls: {}, Total: {:.2}ms, Avg: {:.2}ms, Min: {:.2}ms, Max: {:.2}ms, StdDev: {:.2}ms",
            self.call_count,
            self.total_time.as_secs_f64() * 1000.0,
            self.average_time.as_secs_f64() * 1000.0,
            self.min_time.as_secs_f64() * 1000.0,
            self.max_time.as_secs_f64() * 1000.0,
            self.std_deviation.as_secs_f64() * 1000.0
        )
    }

    /// Get phase breakdown as a string
    pub fn phase_breakdown(&self) -> String {
        let mut phases = Vec::new();
        for (phase, duration) in &self.phase_times {
            phases.push(format!(
                "{}: {:.2}ms",
                phase,
                duration.as_secs_f64() * 1000.0
            ));
        }
        phases.join(", ")
    }
}

/// Performance profiler for tablebase operations
pub struct TablebaseProfiler {
    /// Metrics for different operations
    metrics: HashMap<String, PerformanceMetrics>,
    /// Current operation being timed
    current_operation: Option<String>,
    /// Start time of current operation
    start_time: Option<TimeSource>,
    /// Current phase being timed
    current_phase: Option<String>,
    /// Phase start time
    phase_start_time: Option<TimeSource>,
    /// Whether profiling is enabled
    enabled: bool,
    /// Maximum number of metrics to keep
    max_metrics: usize,
}

impl TablebaseProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            current_operation: None,
            start_time: None,
            current_phase: None,
            phase_start_time: None,
            enabled: true,
            max_metrics: 100,
        }
    }

    /// Create a profiler with specified maximum metrics
    pub fn with_max_metrics(max_metrics: usize) -> Self {
        Self {
            metrics: HashMap::new(),
            current_operation: None,
            start_time: None,
            current_phase: None,
            phase_start_time: None,
            enabled: true,
            max_metrics,
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start timing an operation
    pub fn start_operation(&mut self, operation_name: String) {
        if !self.enabled {
            return;
        }

        // End any current operation
        self.end_operation();

        if !self.metrics.contains_key(&operation_name) {
            self.metrics
                .insert(operation_name.clone(), PerformanceMetrics::new());
        }

        self.current_operation = Some(operation_name);
        self.start_time = Some(TimeSource::now());
    }

    /// End the current operation
    pub fn end_operation(&mut self) {
        if !self.enabled || self.current_operation.is_none() || self.start_time.is_none() {
            return;
        }

        let operation_name = self.current_operation.take().unwrap();
        let duration = Duration::from_millis(self.start_time.take().unwrap().elapsed_ms() as u64);

        // Record the operation
        self.record_operation(operation_name, duration);

        // Clear current phase
        self.current_phase = None;
        self.phase_start_time = None;
    }

    /// Start timing a phase within the current operation
    pub fn start_phase(&mut self, phase_name: String) {
        if !self.enabled {
            return;
        }

        // End any current phase
        self.end_phase();

        self.current_phase = Some(phase_name);
        self.phase_start_time = Some(TimeSource::now());
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        if !self.enabled || self.current_phase.is_none() || self.phase_start_time.is_none() {
            return;
        }

        let phase_name = self.current_phase.take().unwrap();
        let duration =
            Duration::from_millis(self.phase_start_time.take().unwrap().elapsed_ms() as u64);

        // Record the phase
        if let Some(operation_name) = &self.current_operation {
            if let Some(metrics) = self.metrics.get_mut(operation_name) {
                metrics.record_phase(phase_name, duration);
            }
        }
    }

    /// Record an operation with its duration
    pub fn record_operation(&mut self, operation_name: String, duration: Duration) {
        if !self.enabled {
            return;
        }

        // Clean up old metrics if we have too many
        if self.metrics.len() >= self.max_metrics {
            self.cleanup_old_metrics();
        }

        self.metrics
            .entry(operation_name)
            .or_insert_with(PerformanceMetrics::new)
            .record_call(duration);
    }

    /// Get metrics for a specific operation
    pub fn get_metrics(&self, operation_name: &str) -> Option<&PerformanceMetrics> {
        self.metrics.get(operation_name)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, PerformanceMetrics> {
        &self.metrics
    }

    /// Get performance summary for all operations
    pub fn get_summary(&self) -> String {
        let mut summary = Vec::new();
        summary.push("Tablebase Performance Summary:".to_string());
        summary.push("=".repeat(40).to_string());

        for (operation, metrics) in &self.metrics {
            summary.push(format!("{}: {}", operation, metrics.summary()));
            if !metrics.phase_times.is_empty() {
                summary.push(format!("  Phases: {}", metrics.phase_breakdown()));
            }
        }

        summary.join("\n")
    }

    /// Get the most expensive operations
    pub fn get_most_expensive_operations(
        &self,
        count: usize,
    ) -> Vec<(&String, &PerformanceMetrics)> {
        let mut operations: Vec<_> = self.metrics.iter().collect();
        operations.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));
        operations.truncate(count);
        operations
    }

    /// Get the slowest operations by average time
    pub fn get_slowest_operations(&self, count: usize) -> Vec<(&String, &PerformanceMetrics)> {
        let mut operations: Vec<_> = self.metrics.iter().collect();
        operations.sort_by(|a, b| b.1.average_time.cmp(&a.1.average_time));
        operations.truncate(count);
        operations
    }

    /// Get the most frequently called operations
    pub fn get_most_frequent_operations(
        &self,
        count: usize,
    ) -> Vec<(&String, &PerformanceMetrics)> {
        let mut operations: Vec<_> = self.metrics.iter().collect();
        operations.sort_by(|a, b| b.1.call_count.cmp(&a.1.call_count));
        operations.truncate(count);
        operations
    }

    /// Clean up old metrics to stay within limits
    fn cleanup_old_metrics(&mut self) {
        if self.metrics.len() < self.max_metrics {
            return;
        }

        // Remove metrics with the lowest call counts
        let mut operations: Vec<_> = self
            .metrics
            .iter()
            .map(|(k, v)| (k.clone(), v.call_count))
            .collect();
        operations.sort_by(|a, b| a.1.cmp(&b.1));

        let to_remove = operations.len() - self.max_metrics + 1;
        for (operation_name, _) in operations.iter().take(to_remove) {
            self.metrics.remove(operation_name);
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.metrics.clear();
        self.current_operation = None;
        self.start_time = None;
        self.current_phase = None;
        self.phase_start_time = None;
    }

    /// Get memory usage of the profiler
    pub fn get_memory_usage(&self) -> usize {
        // Estimate memory usage
        self.metrics.len() * std::mem::size_of::<PerformanceMetrics>()
            + self
                .metrics
                .values()
                .map(|m| m.phase_times.len())
                .sum::<usize>()
                * 64
    }
}

impl Default for TablebaseProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance profiler for specific tablebase operations
pub struct OperationProfiler<'a> {
    profiler: &'a mut TablebaseProfiler,
    #[allow(dead_code)]
    operation_name: String,
}

impl<'a> OperationProfiler<'a> {
    /// Create a new operation profiler
    pub fn new(profiler: &'a mut TablebaseProfiler, operation_name: String) -> Self {
        profiler.start_operation(operation_name.clone());
        Self {
            profiler,
            operation_name,
        }
    }

    /// Start timing a phase
    pub fn start_phase(&mut self, phase_name: String) {
        self.profiler.start_phase(phase_name);
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        self.profiler.end_phase();
    }

    /// Record a phase with a closure
    pub fn time_phase<F, R>(&mut self, phase_name: String, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start_phase(phase_name);
        let result = f();
        self.end_phase();
        result
    }
}

impl<'a> Drop for OperationProfiler<'a> {
    fn drop(&mut self) {
        self.profiler.end_operation();
    }
}

/// Macro for easy operation profiling
#[macro_export]
macro_rules! profile_operation {
    ($profiler:expr, $operation:expr, $code:block) => {{
        let mut op_profiler = OperationProfiler::new($profiler, $operation.to_string());
        $code
    }};
}

/// Macro for easy phase profiling
#[macro_export]
macro_rules! profile_phase {
    ($profiler:expr, $phase:expr, $code:block) => {{
        $profiler.start_phase($phase.to_string());
        let result = $code;
        $profiler.end_phase();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        // Record some calls
        metrics.record_call(Duration::from_millis(10));
        metrics.record_call(Duration::from_millis(20));
        metrics.record_call(Duration::from_millis(30));

        assert_eq!(metrics.call_count, 3);
        assert_eq!(metrics.total_time, Duration::from_millis(60));
        assert_eq!(metrics.average_time, Duration::from_millis(20));
        assert_eq!(metrics.min_time, Duration::from_millis(10));
        assert_eq!(metrics.max_time, Duration::from_millis(30));
    }

    #[test]
    fn test_profiler_operations() {
        let mut profiler = TablebaseProfiler::new();

        // Test operation profiling
        profiler.start_operation("test_operation".to_string());
        thread::sleep(Duration::from_millis(10));
        profiler.end_operation();

        let metrics = profiler.get_metrics("test_operation").unwrap();
        assert_eq!(metrics.call_count, 1);
        assert!(metrics.total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_phase_profiling() {
        let mut profiler = TablebaseProfiler::new();

        profiler.start_operation("test_operation".to_string());

        profiler.start_phase("phase1".to_string());
        thread::sleep(Duration::from_millis(5));
        profiler.end_phase();

        profiler.start_phase("phase2".to_string());
        thread::sleep(Duration::from_millis(5));
        profiler.end_phase();

        profiler.end_operation();

        let metrics = profiler.get_metrics("test_operation").unwrap();
        assert_eq!(metrics.phase_times.len(), 2);
        assert!(metrics.phase_times.contains_key("phase1"));
        assert!(metrics.phase_times.contains_key("phase2"));
    }

    #[test]
    fn test_operation_profiler() {
        let mut profiler = TablebaseProfiler::new();

        {
            let mut op_profiler = OperationProfiler::new(&mut profiler, "test_op".to_string());
            op_profiler.start_phase("phase1".to_string());
            thread::sleep(Duration::from_millis(5));
            op_profiler.end_phase();
        }

        let metrics = profiler.get_metrics("test_op").unwrap();
        assert_eq!(metrics.call_count, 1);
        assert!(metrics.phase_times.contains_key("phase1"));
    }
}
