//! Performance Monitoring and Analysis Tools for Automated Tuning
//!
//! This module provides comprehensive tools for monitoring training progress,
//! analyzing performance metrics, generating reports, and managing long-running
//! optimization processes.
//!
//! ## Checkpoint Configuration
//!
//! Checkpoints can be saved to a configurable path via `PerformanceConfig::checkpoint_path`.
//! If `checkpoint_path` is `None`, the default path "checkpoints/" is used.
//! The checkpoint directory is automatically created if it doesn't exist.
//!
//! Example:
//! ```rust,no_run
//! use shogi_engine::tuning::types::PerformanceConfig;
//! use shogi_engine::tuning::performance::TuningProfiler;
//!
//! let mut config = PerformanceConfig::default();
//! config.checkpoint_path = Some("my_checkpoints/".to_string());
//! let mut profiler = TuningProfiler::new(config);
//! profiler.create_checkpoint(100, 0.5, None, None)?;
//! ```

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::types::{OptimizationMethod, PerformanceConfig, TuningResults, ValidationResults};
use super::optimizer::IncrementalState;

/// Verbosity levels for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

/// Performance metrics for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Feature extraction timing
    pub feature_extraction_time: Duration,
    /// Optimization timing
    pub optimization_time: Duration,
    /// Validation timing
    pub validation_time: Duration,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Number of iterations completed
    pub iterations_completed: usize,
    /// Convergence rate (error reduction per iteration)
    pub convergence_rate: f64,
    /// Final error value
    pub final_error: f64,
    /// Average error reduction per iteration
    pub avg_error_reduction: f64,
}

/// Checkpoint data for resume functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData {
    /// Timestamp when checkpoint was created
    pub timestamp: u64,
    /// Current iteration number
    pub iteration: usize,
    /// Current weights
    pub weights: Vec<f64>,
    /// Current error value
    pub current_error: f64,
    /// Optimization method being used
    pub optimization_method: OptimizationMethod,
    /// Performance metrics up to this point
    pub metrics: PerformanceMetrics,
    /// Validation results if available
    pub validation_results: Option<ValidationResults>,
    /// Incremental learning state (if incremental learning is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_state: Option<IncrementalStateCheckpoint>,
}

/// Incremental learning state for checkpointing
///
/// This is a serializable version of IncrementalState for checkpointing.
/// Note: AdamState, LBFGSState, and GeneticAlgorithmState are not serialized
/// as they can be reconstructed from the optimization method configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalStateCheckpoint {
    /// Current weights
    pub weights: Vec<f64>,
    /// Number of positions processed so far
    pub positions_processed: usize,
    /// Total number of updates performed
    pub update_count: usize,
    /// Error history for tracking progress
    pub error_history: Vec<f64>,
}

/// Progress tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Current iteration
    pub current_iteration: usize,
    /// Total iterations (if known)
    pub total_iterations: Option<usize>,
    /// Current error
    pub current_error: f64,
    /// Initial error
    pub initial_error: f64,
    /// Progress percentage (0.0 to 1.0)
    pub progress_percentage: f64,
    /// Estimated time remaining
    pub eta_seconds: Option<f64>,
    /// Average time per iteration
    pub avg_time_per_iteration: Duration,
}

/// Statistical analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    /// Mean error across iterations
    pub mean_error: f64,
    /// Standard deviation of errors
    pub error_std_dev: f64,
    /// Minimum error achieved
    pub min_error: f64,
    /// Maximum error
    pub max_error: f64,
    /// Error improvement percentage
    pub improvement_percentage: f64,
    /// Convergence speed (iterations to reach 90% of final improvement)
    pub convergence_speed: Option<usize>,
    /// Stability metric (error variance in last 10% of iterations)
    pub stability_metric: f64,
}

/// Performance profiler for tuning processes
pub struct TuningProfiler {
    config: PerformanceConfig,
    start_time: Instant,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    error_history: Arc<Mutex<Vec<f64>>>,
    iteration_times: Arc<Mutex<Vec<Duration>>>,
    memory_snapshots: Arc<Mutex<Vec<(Instant, usize)>>>,
    checkpoint_frequency: usize,
    last_checkpoint: Option<Instant>,
    #[allow(dead_code)]
    log_level: LogLevel,
}

impl TuningProfiler {
    /// Create a new tuning profiler
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config: config.clone(),
            start_time: Instant::now(),
            metrics: Arc::new(Mutex::new(PerformanceMetrics {
                feature_extraction_time: Duration::ZERO,
                optimization_time: Duration::ZERO,
                validation_time: Duration::ZERO,
                memory_usage_bytes: 0,
                iterations_completed: 0,
                convergence_rate: 0.0,
                final_error: 0.0,
                avg_error_reduction: 0.0,
            })),
            error_history: Arc::new(Mutex::new(Vec::new())),
            iteration_times: Arc::new(Mutex::new(Vec::new())),
            memory_snapshots: Arc::new(Mutex::new(Vec::new())),
            checkpoint_frequency: config.checkpoint_frequency,
            last_checkpoint: None,
            log_level: LogLevel::Info,
        }
    }

    /// Create profiler with custom log level
    pub fn with_log_level(config: PerformanceConfig, log_level: LogLevel) -> Self {
        Self {
            log_level,
            ..Self::new(config)
        }
    }

    /// Start timing an operation
    pub fn start_timing(&self, operation: &str) -> OperationTimer {
        debug!("Starting timing for operation: {}", operation);
        OperationTimer::new(operation.to_string(), Instant::now())
    }

    /// Record feature extraction timing
    pub fn record_feature_extraction_time(&self, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.feature_extraction_time += duration;
        debug!("Feature extraction time: {:?}", duration);
    }

    /// Record optimization timing
    pub fn record_optimization_time(&self, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.optimization_time += duration;
        debug!("Optimization time: {:?}", duration);
    }

    /// Record validation timing
    pub fn record_validation_time(&self, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.validation_time += duration;
        debug!("Validation time: {:?}", duration);
    }

    /// Record memory usage
    pub fn record_memory_usage(&self, bytes: usize) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.memory_usage_bytes = bytes;

        let mut snapshots = self.memory_snapshots.lock().unwrap();
        snapshots.push((Instant::now(), bytes));

        // Keep only last 1000 snapshots to prevent memory bloat
        if snapshots.len() > 1000 {
            let keep_count = 1000;
            let remove_count = snapshots.len() - keep_count;
            snapshots.drain(0..remove_count);
        }

        debug!(
            "Memory usage: {} bytes ({:.2} MB)",
            bytes,
            bytes as f64 / 1_048_576.0
        );
    }

    /// Record iteration completion
    pub fn record_iteration(&mut self, iteration: usize, error: f64, iteration_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.iterations_completed = iteration;
        metrics.final_error = error;

        let mut error_history = self.error_history.lock().unwrap();
        error_history.push(error);

        let mut iteration_times = self.iteration_times.lock().unwrap();
        iteration_times.push(iteration_time);

        // Calculate convergence rate
        if error_history.len() > 1 {
            let recent_errors = &error_history[error_history.len().saturating_sub(10)..];
            if recent_errors.len() > 1 {
                let total_reduction = recent_errors[0] - recent_errors[recent_errors.len() - 1];
                metrics.convergence_rate = total_reduction / recent_errors.len() as f64;
            }
        }

        // Calculate average error reduction
        if error_history.len() > 1 {
            let total_reduction = error_history[0] - error;
            metrics.avg_error_reduction = total_reduction / iteration as f64;
        }

        // Check if we need to create a checkpoint before dropping locks
        let should_checkpoint = iteration % self.checkpoint_frequency == 0;

        // Drop all locks before calling create_checkpoint
        drop(metrics);
        drop(error_history);
        drop(iteration_times);

        self.log_iteration(iteration, error, iteration_time);

        if should_checkpoint {
            let _ = self.create_checkpoint(iteration, error, None, None);
        }
    }

    /// Create a checkpoint
    ///
    /// Uses the checkpoint path from `PerformanceConfig`. If `checkpoint_path` is `None`,
    /// defaults to "checkpoints/". The directory will be created if it doesn't exist.
    pub fn create_checkpoint(
        &mut self,
        iteration: usize,
        error: f64,
        weights: Option<Vec<f64>>,
        optimization_method: Option<OptimizationMethod>,
    ) -> Result<(), std::io::Error> {
        self.create_checkpoint_with_incremental_state(
            iteration,
            error,
            weights,
            optimization_method,
            None,
        )
    }

    /// Create a checkpoint with incremental learning state
    ///
    /// This method allows saving incremental learning state for resume functionality.
    pub fn create_checkpoint_with_incremental_state(
        &mut self,
        iteration: usize,
        error: f64,
        weights: Option<Vec<f64>>,
        optimization_method: Option<OptimizationMethod>,
        incremental_state: Option<&IncrementalState>,
    ) -> Result<(), std::io::Error> {
        let checkpoint_data = CheckpointData {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            iteration,
            weights: weights.unwrap_or_default(),
            current_error: error,
            optimization_method: optimization_method.unwrap_or(
                OptimizationMethod::GradientDescent {
                    learning_rate: 0.01,
                },
            ),
            metrics: self.metrics.lock().unwrap().clone(),
            validation_results: None,
            incremental_state: incremental_state.map(|s| s.to_checkpoint()),
        };

        // Use configured checkpoint path or default to "checkpoints/"
        let checkpoint_dir_str = self
            .config
            .checkpoint_path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("checkpoints/");
        let checkpoint_dir = Path::new(checkpoint_dir_str);

        // Create directory if it doesn't exist
        if !checkpoint_dir.exists() {
            std::fs::create_dir_all(checkpoint_dir)?;
        }

        let checkpoint_file = checkpoint_dir.join(format!("checkpoint_iter_{}.json", iteration));
        let file = File::create(checkpoint_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &checkpoint_data)?;

        self.last_checkpoint = Some(Instant::now());
        info!("Checkpoint created at iteration {} in {}", iteration, checkpoint_dir_str);

        Ok(())
    }

    /// Load checkpoint data
    pub fn load_checkpoint<P: AsRef<Path>>(path: P) -> Result<CheckpointData, std::io::Error> {
        let file = File::open(path)?;
        let checkpoint: CheckpointData = serde_json::from_reader(file)?;
        info!("Loaded checkpoint from iteration {}", checkpoint.iteration);
        Ok(checkpoint)
    }

    /// Get current progress information
    pub fn get_progress(&self) -> ProgressInfo {
        let metrics = self.metrics.lock().unwrap();
        let error_history = self.error_history.lock().unwrap();
        let iteration_times = self.iteration_times.lock().unwrap();

        let current_iteration = metrics.iterations_completed;
        let current_error = metrics.final_error;
        let initial_error = error_history.first().copied().unwrap_or(current_error);

        let progress_percentage = if let Some(total) = self.config.max_iterations {
            current_iteration as f64 / total as f64
        } else {
            0.0
        };

        let avg_time_per_iteration = if !iteration_times.is_empty() {
            let total_time: Duration = iteration_times.iter().sum();
            total_time / iteration_times.len() as u32
        } else {
            Duration::ZERO
        };

        let eta_seconds = if !iteration_times.is_empty() {
            if let Some(total) = self.config.max_iterations {
                let remaining = total.saturating_sub(current_iteration);
                Some(avg_time_per_iteration.as_secs_f64() * remaining as f64)
            } else {
                None
            }
        } else {
            None
        };

        ProgressInfo {
            current_iteration,
            total_iterations: self.config.max_iterations,
            current_error,
            initial_error,
            progress_percentage,
            eta_seconds,
            avg_time_per_iteration,
        }
    }

    /// Generate statistical analysis
    pub fn generate_statistical_analysis(&self) -> StatisticalAnalysis {
        let error_history = self.error_history.lock().unwrap();
        let metrics = self.metrics.lock().unwrap();

        if error_history.is_empty() {
            return StatisticalAnalysis {
                mean_error: 0.0,
                error_std_dev: 0.0,
                min_error: 0.0,
                max_error: 0.0,
                improvement_percentage: 0.0,
                convergence_speed: None,
                stability_metric: 0.0,
            };
        }

        let mean_error = error_history.iter().sum::<f64>() / error_history.len() as f64;
        let variance = error_history
            .iter()
            .map(|&x| (x - mean_error).powi(2))
            .sum::<f64>()
            / error_history.len() as f64;
        let error_std_dev = variance.sqrt();

        let min_error = error_history.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_error = error_history
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        let initial_error = error_history[0];
        let final_error = metrics.final_error;
        let improvement_percentage = ((initial_error - final_error) / initial_error) * 100.0;

        // Calculate convergence speed (iterations to reach 90% of improvement)
        let target_error = initial_error - (initial_error - final_error) * 0.9;
        let convergence_speed = error_history.iter().position(|&x| x <= target_error);

        // Calculate stability metric (error variance in last 10% of iterations)
        let last_10_percent = error_history.len() / 10;
        let last_errors = &error_history[error_history.len().saturating_sub(last_10_percent)..];
        let last_mean = last_errors.iter().sum::<f64>() / last_errors.len() as f64;
        let stability_metric = last_errors
            .iter()
            .map(|&x| (x - last_mean).powi(2))
            .sum::<f64>()
            / last_errors.len() as f64;

        StatisticalAnalysis {
            mean_error,
            error_std_dev,
            min_error,
            max_error,
            improvement_percentage,
            convergence_speed,
            stability_metric,
        }
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self, results: &TuningResults) -> String {
        let metrics = self.metrics.lock().unwrap();
        let analysis = self.generate_statistical_analysis();
        let progress = self.get_progress();

        format!(
            "=== PERFORMANCE REPORT ===\n\
             Training Duration: {:.2} seconds\n\
             Iterations Completed: {}\n\
             Final Error: {:.6}\n\
             Initial Error: {:.6}\n\
             Improvement: {:.2}%\n\
             \n\
             === TIMING BREAKDOWN ===\n\
             Feature Extraction: {:.2}s ({:.1}%)\n\
             Optimization: {:.2}s ({:.1}%)\n\
             Validation: {:.2}s ({:.1}%)\n\
             Average Time per Iteration: {:?}\n\
             \n\
             === PERFORMANCE METRICS ===\n\
             Memory Usage: {:.2} MB\n\
             Convergence Rate: {:.6}\n\
             Average Error Reduction: {:.6}\n\
             Convergence Speed: {}\n\
             Stability Metric: {:.6}\n\
             \n\
             === STATISTICAL ANALYSIS ===\n\
             Mean Error: {:.6}\n\
             Error Std Dev: {:.6}\n\
             Min Error: {:.6}\n\
             Max Error: {:.6}\n\
             \n\
             === CONVERGENCE STATUS ===\n\
             Converged: {}\n\
             ETA: {}\n\
             Progress: {:.1}%",
            results.training_time_seconds,
            metrics.iterations_completed,
            metrics.final_error,
            progress.initial_error,
            analysis.improvement_percentage,
            metrics.feature_extraction_time.as_secs_f64(),
            (metrics.feature_extraction_time.as_secs_f64() / results.training_time_seconds) * 100.0,
            metrics.optimization_time.as_secs_f64(),
            (metrics.optimization_time.as_secs_f64() / results.training_time_seconds) * 100.0,
            metrics.validation_time.as_secs_f64(),
            (metrics.validation_time.as_secs_f64() / results.training_time_seconds) * 100.0,
            progress.avg_time_per_iteration,
            metrics.memory_usage_bytes as f64 / 1_048_576.0,
            metrics.convergence_rate,
            metrics.avg_error_reduction,
            analysis
                .convergence_speed
                .map_or("N/A".to_string(), |s| format!("{} iterations", s)),
            analysis.stability_metric,
            analysis.mean_error,
            analysis.error_std_dev,
            analysis.min_error,
            analysis.max_error,
            results.converged,
            progress
                .eta_seconds
                .map_or("N/A".to_string(), |eta| format!("{:.1}s", eta)),
            progress.progress_percentage * 100.0
        )
    }

    /// Save performance report to file
    pub fn save_report<P: AsRef<Path>>(
        &self,
        path: P,
        results: &TuningResults,
    ) -> Result<(), std::io::Error> {
        let report = self.generate_report(results);
        let mut file = File::create(path)?;
        file.write_all(report.as_bytes())?;
        Ok(())
    }

    /// Log iteration progress
    fn log_iteration(&self, iteration: usize, error: f64, iteration_time: Duration) {
        if self.config.enable_logging {
            let progress = self.get_progress();
            let eta_str = progress
                .eta_seconds
                .map_or("N/A".to_string(), |eta| format!("{:.1}s", eta));

            info!(
                "Iteration {}: Error = {:.6}, Time = {:?}, ETA = {}",
                iteration, error, iteration_time, eta_str
            );
        }
    }

    /// Get memory usage (simplified implementation)
    pub fn get_memory_usage(&self) -> usize {
        // This is a simplified implementation
        // In a real implementation, you might use system-specific APIs
        let snapshots = self.memory_snapshots.lock().unwrap();
        snapshots.last().map_or(0, |(_, bytes)| *bytes)
    }

    /// Detect performance regression
    pub fn detect_performance_regression(&self, baseline_metrics: &PerformanceMetrics) -> bool {
        let current_metrics = self.metrics.lock().unwrap();

        // Check if convergence rate is significantly worse
        let convergence_regression =
            current_metrics.convergence_rate < baseline_metrics.convergence_rate * 0.5;

        // Check if memory usage is significantly higher
        let memory_regression =
            current_metrics.memory_usage_bytes > baseline_metrics.memory_usage_bytes * 2;

        convergence_regression || memory_regression
    }

    /// Get elapsed time since start
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Reset profiler state
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        *self.metrics.lock().unwrap() = PerformanceMetrics {
            feature_extraction_time: Duration::ZERO,
            optimization_time: Duration::ZERO,
            validation_time: Duration::ZERO,
            memory_usage_bytes: 0,
            iterations_completed: 0,
            convergence_rate: 0.0,
            final_error: 0.0,
            avg_error_reduction: 0.0,
        };
        self.error_history.lock().unwrap().clear();
        self.iteration_times.lock().unwrap().clear();
        self.memory_snapshots.lock().unwrap().clear();
        self.last_checkpoint = None;
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
}

impl OperationTimer {
    fn new(operation: String, start_time: Instant) -> Self {
        Self {
            operation,
            start_time,
        }
    }

    /// Stop the timer and return the duration
    pub fn stop(self) -> Duration {
        let duration = self.start_time.elapsed();
        debug!("Operation '{}' took {:?}", self.operation, duration);
        duration
    }
}

/// Initialize logging system
pub fn init_logging(log_level: LogLevel) {
    let level_str = match log_level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    };

    std::env::set_var("RUST_LOG", level_str);
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::super::types::{OptimizationMethod, PerformanceConfig};
    use super::*;

    #[test]
    fn test_tuning_profiler_creation() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);
        assert_eq!(profiler.elapsed_time().as_secs(), 0);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new("test_operation".to_string(), Instant::now());
        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.stop();
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_recording() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        profiler.record_feature_extraction_time(Duration::from_secs(1));
        profiler.record_optimization_time(Duration::from_secs(2));
        profiler.record_validation_time(Duration::from_secs(1));
        profiler.record_memory_usage(1024 * 1024); // 1 MB

        let metrics = profiler.get_metrics();
        assert_eq!(metrics.feature_extraction_time, Duration::from_secs(1));
        assert_eq!(metrics.optimization_time, Duration::from_secs(2));
        assert_eq!(metrics.validation_time, Duration::from_secs(1));
        assert_eq!(metrics.memory_usage_bytes, 1024 * 1024);
    }

    #[test]
    fn test_iteration_recording() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        profiler.record_iteration(1, 1.0, Duration::from_millis(100));
        profiler.record_iteration(2, 0.8, Duration::from_millis(90));
        profiler.record_iteration(3, 0.6, Duration::from_millis(80));

        let metrics = profiler.get_metrics();
        assert_eq!(metrics.iterations_completed, 3);
        assert_eq!(metrics.final_error, 0.6);
        assert!(metrics.convergence_rate > 0.0);
    }

    #[test]
    fn test_progress_calculation() {
        let mut config = PerformanceConfig::default();
        config.max_iterations = Some(100);

        let mut profiler = TuningProfiler::new(config);

        profiler.record_iteration(25, 0.5, Duration::from_millis(100));
        profiler.record_iteration(50, 0.3, Duration::from_millis(90));

        let progress = profiler.get_progress();
        assert_eq!(progress.current_iteration, 50);
        assert_eq!(progress.total_iterations, Some(100));
        assert_eq!(progress.progress_percentage, 0.5);
        assert!(progress.eta_seconds.is_some());
    }

    #[test]
    fn test_statistical_analysis() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        // Record decreasing error values
        profiler.record_iteration(1, 1.0, Duration::from_millis(100));
        profiler.record_iteration(2, 0.8, Duration::from_millis(90));
        profiler.record_iteration(3, 0.6, Duration::from_millis(80));
        profiler.record_iteration(4, 0.4, Duration::from_millis(70));
        profiler.record_iteration(5, 0.2, Duration::from_millis(60));

        let analysis = profiler.generate_statistical_analysis();
        assert_eq!(analysis.min_error, 0.2);
        assert_eq!(analysis.max_error, 1.0);
        assert_eq!(analysis.improvement_percentage, 80.0);
        assert!(analysis.convergence_speed.is_some());
    }

    #[test]
    fn test_checkpoint_creation_and_loading() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        profiler.record_iteration(10, 0.5, Duration::from_millis(100));

        // Create checkpoint
        let weights = vec![1.0, 2.0, 3.0];
        let method = OptimizationMethod::Adam {
            learning_rate: 0.01,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        };
        profiler
            .create_checkpoint(10, 0.5, Some(weights.clone()), Some(method))
            .unwrap();

        // Load checkpoint from the default location
        let checkpoint_path = Path::new("checkpoints/checkpoint_iter_10.json");
        let loaded_checkpoint = TuningProfiler::load_checkpoint(checkpoint_path).unwrap();
        assert_eq!(loaded_checkpoint.iteration, 10);
        assert_eq!(loaded_checkpoint.current_error, 0.5);
        assert_eq!(loaded_checkpoint.weights, weights);

        // Clean up
        let _ = std::fs::remove_dir_all("checkpoints");
    }

    #[test]
    fn test_checkpoint_path_configuration() {
        // Test with custom checkpoint path
        let custom_path = "test_checkpoints/";
        let mut config = PerformanceConfig::default();
        config.checkpoint_path = Some(custom_path.to_string());
        let mut profiler = TuningProfiler::new(config);

        let weights = vec![1.0, 2.0, 3.0];
        let method = OptimizationMethod::GradientDescent {
            learning_rate: 0.01,
        };

        // Create checkpoint with custom path
        profiler
            .create_checkpoint(5, 0.3, Some(weights.clone()), Some(method))
            .unwrap();

        // Verify checkpoint was created in custom path
        let checkpoint_path = Path::new("test_checkpoints/checkpoint_iter_5.json");
        assert!(checkpoint_path.exists(), "Checkpoint should be created in custom path");

        // Load and verify checkpoint
        let loaded_checkpoint = TuningProfiler::load_checkpoint(checkpoint_path).unwrap();
        assert_eq!(loaded_checkpoint.iteration, 5);
        assert_eq!(loaded_checkpoint.current_error, 0.3);
        assert_eq!(loaded_checkpoint.weights, weights);

        // Cleanup
        let _ = std::fs::remove_dir_all("test_checkpoints");
    }

    #[test]
    fn test_checkpoint_path_creation() {
        // Test that directory is created if it doesn't exist
        let custom_path = "new_checkpoint_dir/";
        let mut config = PerformanceConfig::default();
        config.checkpoint_path = Some(custom_path.to_string());

        // Verify directory doesn't exist initially
        let checkpoint_dir = Path::new(custom_path);
        if checkpoint_dir.exists() {
            let _ = std::fs::remove_dir_all(checkpoint_dir);
        }
        assert!(!checkpoint_dir.exists(), "Directory should not exist initially");

        // Create profiler and checkpoint
        let mut profiler = TuningProfiler::new(config);
        profiler
            .create_checkpoint(1, 0.1, None, None)
            .unwrap();

        // Verify directory was created
        assert!(checkpoint_dir.exists(), "Directory should be created automatically");
        assert!(checkpoint_dir.is_dir(), "Path should be a directory");

        // Verify checkpoint file exists
        let checkpoint_file = checkpoint_dir.join("checkpoint_iter_1.json");
        assert!(checkpoint_file.exists(), "Checkpoint file should exist");

        // Cleanup
        let _ = std::fs::remove_dir_all(custom_path);
    }

    #[test]
    fn test_performance_regression_detection() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        let baseline_metrics = PerformanceMetrics {
            feature_extraction_time: Duration::ZERO,
            optimization_time: Duration::ZERO,
            validation_time: Duration::ZERO,
            memory_usage_bytes: 1024 * 1024, // 1 MB
            iterations_completed: 0,
            convergence_rate: 0.1,
            final_error: 0.0,
            avg_error_reduction: 0.0,
        };

        // Record poor performance
        profiler.record_iteration(1, 1.0, Duration::from_millis(100));
        profiler.record_iteration(2, 0.9, Duration::from_millis(100));
        profiler.record_memory_usage(3 * 1024 * 1024); // 3 MB (3x baseline)

        let regression_detected = profiler.detect_performance_regression(&baseline_metrics);
        assert!(regression_detected);
    }

    #[test]
    fn test_profiler_reset() {
        let config = PerformanceConfig::default();
        let mut profiler = TuningProfiler::new(config);

        profiler.record_iteration(5, 0.5, Duration::from_millis(100));
        profiler.record_memory_usage(1024 * 1024);

        profiler.reset();

        let metrics = profiler.get_metrics();
        assert_eq!(metrics.iterations_completed, 0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.final_error, 0.0);
    }
}
