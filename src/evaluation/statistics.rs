//! Statistics and Monitoring Module
//!
//! This module provides comprehensive statistics tracking and monitoring for the
//! tapered evaluation system. Tracks:
//! - Evaluation statistics (count, averages, distributions)
//! - Phase distribution across evaluations
//! - Accuracy metrics (prediction quality)
//! - Performance metrics (timing, throughput)
//! - Export capabilities (JSON, CSV)
//!
//! # Overview
//!
//! The statistics system:
//! - Real-time tracking of evaluation metrics
//! - Phase distribution analysis
//! - Accuracy measurement
//! - Performance monitoring
//! - Export to various formats
//! - Minimal overhead when disabled
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::statistics::EvaluationStatistics;
//!
//! let mut stats = EvaluationStatistics::new();
//! stats.enable();
//!
//! // Record evaluations
//! stats.record_evaluation(150, 200);
//! stats.record_phase(128);
//!
//! // Get report
//! let report = stats.generate_report();
//! println!("{}", report);
//! ```

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Comprehensive evaluation statistics tracker
#[derive(Debug, Clone)]
pub struct EvaluationStatistics {
    /// Enable statistics tracking
    enabled: bool,
    /// Evaluation count
    evaluation_count: u64,
    /// Score statistics
    score_stats: ScoreStatistics,
    /// Phase statistics
    phase_stats: PhaseStatistics,
    /// Accuracy metrics
    accuracy_metrics: AccuracyMetrics,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Start time for session tracking
    session_start: Option<Instant>,
}

impl EvaluationStatistics {
    /// Create a new statistics tracker (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            evaluation_count: 0,
            score_stats: ScoreStatistics::default(),
            phase_stats: PhaseStatistics::default(),
            accuracy_metrics: AccuracyMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            session_start: None,
        }
    }

    /// Enable statistics tracking
    pub fn enable(&mut self) {
        self.enabled = true;
        self.session_start = Some(Instant::now());
    }

    /// Disable statistics tracking
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Record an evaluation
    #[inline]
    pub fn record_evaluation(&mut self, score: i32, phase: i32) {
        if !self.enabled {
            return;
        }

        self.evaluation_count += 1;
        self.score_stats.record(score);
        self.phase_stats.record(phase);
    }

    /// Record phase only
    #[inline]
    pub fn record_phase(&mut self, phase: i32) {
        if !self.enabled {
            return;
        }
        self.phase_stats.record(phase);
    }

    /// Record accuracy (predicted vs actual)
    #[inline]
    pub fn record_accuracy(&mut self, predicted: i32, actual: i32) {
        if !self.enabled {
            return;
        }
        self.accuracy_metrics.record(predicted, actual);
    }

    /// Record performance timing
    #[inline]
    pub fn record_timing(&mut self, duration_ns: u64) {
        if !self.enabled {
            return;
        }
        self.performance_metrics.record_timing(duration_ns);
    }

    /// Generate comprehensive report
    pub fn generate_report(&self) -> StatisticsReport {
        let session_duration = self.session_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::from_secs(0));

        StatisticsReport {
            enabled: self.enabled,
            evaluation_count: self.evaluation_count,
            score_stats: self.score_stats.clone(),
            phase_stats: self.phase_stats.clone(),
            accuracy_metrics: self.accuracy_metrics.clone(),
            performance_metrics: self.performance_metrics.clone(),
            session_duration_secs: session_duration.as_secs_f64(),
            evaluations_per_second: if session_duration.as_secs_f64() > 0.0 {
                self.evaluation_count as f64 / session_duration.as_secs_f64()
            } else {
                0.0
            },
        }
    }

    /// Export statistics to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.evaluation_count = 0;
        self.score_stats = ScoreStatistics::default();
        self.phase_stats = PhaseStatistics::default();
        self.accuracy_metrics = AccuracyMetrics::default();
        self.performance_metrics = PerformanceMetrics::default();
        self.session_start = Some(Instant::now());
    }

    /// Get evaluation count
    pub fn count(&self) -> u64 {
        self.evaluation_count
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Score statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreStatistics {
    /// Total score sum
    sum: i64,
    /// Minimum score seen
    min: i32,
    /// Maximum score seen
    max: i32,
    /// Count of evaluations
    count: u64,
    /// Score distribution (bucketed)
    distribution: [u64; 10], // -10K to +10K in 2K buckets
}

impl ScoreStatistics {
    fn record(&mut self, score: i32) {
        self.sum += score as i64;
        self.min = if self.count == 0 { score } else { self.min.min(score) };
        self.max = if self.count == 0 { score } else { self.max.max(score) };
        self.count += 1;

        // Update distribution
        let bucket = ((score + 10000) / 2000).clamp(0, 9) as usize;
        self.distribution[bucket] += 1;
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

/// Phase statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhaseStatistics {
    /// Total phase sum
    sum: i64,
    /// Phase distribution
    opening_count: u64,    // phase >= 192
    middlegame_count: u64, // 64 <= phase < 192
    endgame_count: u64,    // phase < 64
    /// Detailed distribution (26 buckets, 10 phase units each)
    distribution: [u64; 26],
}

impl PhaseStatistics {
    fn record(&mut self, phase: i32) {
        self.sum += phase as i64;

        // Update phase category counts
        if phase >= 192 {
            self.opening_count += 1;
        } else if phase >= 64 {
            self.middlegame_count += 1;
        } else {
            self.endgame_count += 1;
        }

        // Update distribution
        let bucket = (phase / 10).clamp(0, 25) as usize;
        self.distribution[bucket] += 1;
    }

    pub fn average(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            self.sum as f64 / total as f64
        }
    }

    pub fn opening_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.opening_count as f64 / total as f64) * 100.0
        }
    }

    pub fn middlegame_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.middlegame_count as f64 / total as f64) * 100.0
        }
    }

    pub fn endgame_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.endgame_count as f64 / total as f64) * 100.0
        }
    }
}

/// Accuracy metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// Sum of squared errors
    sum_squared_error: f64,
    /// Sum of absolute errors
    sum_absolute_error: f64,
    /// Count of predictions
    count: u64,
}

impl AccuracyMetrics {
    fn record(&mut self, predicted: i32, actual: i32) {
        let error = (predicted - actual) as f64;
        self.sum_squared_error += error * error;
        self.sum_absolute_error += error.abs();
        self.count += 1;
    }

    pub fn mean_squared_error(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_squared_error / self.count as f64
        }
    }

    pub fn root_mean_squared_error(&self) -> f64 {
        self.mean_squared_error().sqrt()
    }

    pub fn mean_absolute_error(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_absolute_error / self.count as f64
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Sum of timing measurements (nanoseconds)
    total_time_ns: u64,
    /// Count of timing measurements
    timing_count: u64,
    /// Minimum time
    min_time_ns: u64,
    /// Maximum time
    max_time_ns: u64,
}

impl PerformanceMetrics {
    fn record_timing(&mut self, duration_ns: u64) {
        self.total_time_ns += duration_ns;
        self.timing_count += 1;
        
        if self.timing_count == 1 {
            self.min_time_ns = duration_ns;
            self.max_time_ns = duration_ns;
        } else {
            self.min_time_ns = self.min_time_ns.min(duration_ns);
            self.max_time_ns = self.max_time_ns.max(duration_ns);
        }
    }

    pub fn average_time_ns(&self) -> f64 {
        if self.timing_count == 0 {
            0.0
        } else {
            self.total_time_ns as f64 / self.timing_count as f64
        }
    }

    pub fn average_time_us(&self) -> f64 {
        self.average_time_ns() / 1000.0
    }

    pub fn throughput_per_second(&self) -> f64 {
        if self.average_time_ns() > 0.0 {
            1_000_000_000.0 / self.average_time_ns()
        } else {
            0.0
        }
    }
}

/// Complete statistics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsReport {
    /// Whether statistics were enabled
    pub enabled: bool,
    /// Total evaluation count
    pub evaluation_count: u64,
    /// Score statistics
    pub score_stats: ScoreStatistics,
    /// Phase statistics
    pub phase_stats: PhaseStatistics,
    /// Accuracy metrics
    pub accuracy_metrics: AccuracyMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Session duration in seconds
    pub session_duration_secs: f64,
    /// Evaluations per second
    pub evaluations_per_second: f64,
}

impl std::fmt::Display for StatisticsReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Evaluation Statistics Report")?;
        writeln!(f, "============================")?;
        writeln!(f)?;
        writeln!(f, "Session Overview:")?;
        writeln!(f, "  Total Evaluations: {}", self.evaluation_count)?;
        writeln!(f, "  Session Duration: {:.2} seconds", self.session_duration_secs)?;
        writeln!(f, "  Throughput: {:.0} evals/sec", self.evaluations_per_second)?;
        writeln!(f)?;
        writeln!(f, "Score Statistics:")?;
        writeln!(f, "  Average Score: {:.2}", self.score_stats.average())?;
        writeln!(f, "  Min Score: {}", self.score_stats.min)?;
        writeln!(f, "  Max Score: {}", self.score_stats.max)?;
        writeln!(f)?;
        writeln!(f, "Phase Distribution:")?;
        writeln!(f, "  Average Phase: {:.2}", self.phase_stats.average())?;
        writeln!(f, "  Opening (≥192): {:.1}%", self.phase_stats.opening_percentage())?;
        writeln!(f, "  Middlegame (64-191): {:.1}%", self.phase_stats.middlegame_percentage())?;
        writeln!(f, "  Endgame (<64): {:.1}%", self.phase_stats.endgame_percentage())?;
        writeln!(f)?;
        writeln!(f, "Accuracy Metrics:")?;
        writeln!(f, "  Mean Absolute Error: {:.2}", self.accuracy_metrics.mean_absolute_error())?;
        writeln!(f, "  Root Mean Squared Error: {:.2}", self.accuracy_metrics.root_mean_squared_error())?;
        writeln!(f)?;
        writeln!(f, "Performance Metrics:")?;
        writeln!(f, "  Average Time: {:.2} μs", self.performance_metrics.average_time_us())?;
        writeln!(f, "  Min Time: {} ns", self.performance_metrics.min_time_ns)?;
        writeln!(f, "  Max Time: {} ns", self.performance_metrics.max_time_ns)?;
        writeln!(f, "  Throughput: {:.0} evals/sec", self.performance_metrics.throughput_per_second())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_creation() {
        let stats = EvaluationStatistics::new();
        assert!(!stats.is_enabled());
        assert_eq!(stats.count(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut stats = EvaluationStatistics::new();
        
        assert!(!stats.is_enabled());
        
        stats.enable();
        assert!(stats.is_enabled());
        
        stats.disable();
        assert!(!stats.is_enabled());
    }

    #[test]
    fn test_record_evaluation() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_evaluation(200, 180);
        stats.record_evaluation(100, 128);

        assert_eq!(stats.count(), 3);
    }

    #[test]
    fn test_score_statistics() {
        let mut score_stats = ScoreStatistics::default();
        
        score_stats.record(100);
        score_stats.record(200);
        score_stats.record(150);

        assert_eq!(score_stats.average(), 150.0);
        assert_eq!(score_stats.min, 100);
        assert_eq!(score_stats.max, 200);
    }

    #[test]
    fn test_phase_statistics() {
        let mut phase_stats = PhaseStatistics::default();
        
        // Opening
        phase_stats.record(256);
        phase_stats.record(200);
        
        // Middlegame
        phase_stats.record(128);
        phase_stats.record(100);
        
        // Endgame
        phase_stats.record(32);
        phase_stats.record(10);

        assert_eq!(phase_stats.opening_count, 2);
        assert_eq!(phase_stats.middlegame_count, 2);
        assert_eq!(phase_stats.endgame_count, 2);
        assert_eq!(phase_stats.opening_percentage(), 100.0 / 3.0);
    }

    #[test]
    fn test_accuracy_metrics() {
        let mut accuracy = AccuracyMetrics::default();
        
        accuracy.record(100, 110); // Error: -10
        accuracy.record(200, 190); // Error: +10
        accuracy.record(150, 150); // Error: 0

        assert_eq!(accuracy.mean_absolute_error(), 20.0 / 3.0);
        assert!((accuracy.mean_squared_error() - 200.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn test_performance_metrics() {
        let mut perf = PerformanceMetrics::default();
        
        perf.record_timing(1000);
        perf.record_timing(1500);
        perf.record_timing(1200);

        assert_eq!(perf.average_time_ns(), 1233.3333333333333);
        assert_eq!(perf.min_time_ns, 1000);
        assert_eq!(perf.max_time_ns, 1500);
    }

    #[test]
    fn test_generate_report() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_accuracy(150, 145);
        stats.record_timing(1000);

        let report = stats.generate_report();
        
        assert_eq!(report.evaluation_count, 1);
        assert!(report.enabled);
    }

    #[test]
    fn test_export_json() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);

        let json = stats.export_json();
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("evaluation_count"));
    }

    #[test]
    fn test_reset() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        assert_eq!(stats.count(), 1);

        stats.reset();
        assert_eq!(stats.count(), 0);
    }

    #[test]
    fn test_disabled_no_recording() {
        let mut stats = EvaluationStatistics::new();
        // Not enabled

        stats.record_evaluation(150, 200);
        assert_eq!(stats.count(), 0); // Should not record
    }

    #[test]
    fn test_phase_percentages() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        // Record 2 opening, 6 middlegame, 2 endgame
        stats.record_phase(256);
        stats.record_phase(200);
        stats.record_phase(128);
        stats.record_phase(100);
        stats.record_phase(90);
        stats.record_phase(80);
        stats.record_phase(70);
        stats.record_phase(65);
        stats.record_phase(32);
        stats.record_phase(10);

        let report = stats.generate_report();
        assert_eq!(report.phase_stats.opening_percentage(), 20.0);
        assert_eq!(report.phase_stats.middlegame_percentage(), 60.0);
        assert_eq!(report.phase_stats.endgame_percentage(), 20.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let perf = PerformanceMetrics {
            total_time_ns: 1000,
            timing_count: 1,
            min_time_ns: 1000,
            max_time_ns: 1000,
        };

        let throughput = perf.throughput_per_second();
        assert_eq!(throughput, 1_000_000.0); // 1M evals/sec at 1μs each
    }

    #[test]
    fn test_report_display() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_accuracy(150, 145);

        let report = stats.generate_report();
        let display = format!("{}", report);

        assert!(display.contains("Evaluation Statistics Report"));
        assert!(display.contains("Total Evaluations"));
        assert!(display.contains("Phase Distribution"));
    }
}

