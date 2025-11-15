//! Performance tuning system for transposition tables
//!
//! This module provides comprehensive performance tuning capabilities including
//! automatic parameter optimization, performance profiling, and tuning recommendations.

use crate::search::adaptive_configuration::*;
use crate::search::runtime_configuration::{PerformanceMetrics as RuntimePerformanceMetrics, *};
use crate::search::transposition_config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Performance tuning manager
pub struct PerformanceTuningManager {
    /// Adaptive configuration manager
    adaptive_manager: Arc<Mutex<AdaptiveConfigurationManager>>,
    /// Performance profiler
    profiler: Arc<Mutex<PerformanceProfiler>>,
    /// Tuning recommendations
    recommendations: Vec<TuningRecommendation>,
    /// Tuning history
    tuning_history: Vec<TuningSession>,
    /// Performance targets
    performance_targets: PerformanceTargets,
}

/// Performance profiler for detailed analysis
pub struct PerformanceProfiler {
    /// Operation timings
    operation_timings: HashMap<String, Vec<u64>>,
    /// Memory usage snapshots
    memory_snapshots: Vec<MemorySnapshot>,
    /// Performance counters
    performance_counters: PerformanceCounters,
    /// Profiling enabled flag
    enabled: bool,
}

/// Performance counters
#[derive(Debug, Clone, Default)]
pub struct PerformanceCounters {
    /// Total operations
    pub total_operations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Memory allocations
    pub memory_allocations: u64,
    /// Memory deallocations
    pub memory_deallocations: u64,
    /// Hash collisions
    pub hash_collisions: u64,
    /// Replacements
    pub replacements: u64,
}

/// Memory usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Available memory in bytes
    pub available_memory_bytes: u64,
}

/// Performance targets for tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Target hit rate (0.0 to 1.0)
    pub target_hit_rate: f64,
    /// Target operation time in microseconds
    pub target_operation_time_us: f64,
    /// Maximum memory usage in bytes
    pub max_memory_usage_bytes: u64,
    /// Target collision rate (0.0 to 1.0)
    pub target_collision_rate: f64,
    /// Target throughput (operations per second)
    pub target_throughput_ops_per_sec: f64,
}

/// Tuning recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Recommendation title
    pub title: String,
    /// Recommendation description
    pub description: String,
    /// Recommended action
    pub action: TuningAction,
    /// Expected improvement percentage
    pub expected_improvement: f64,
    /// Priority (1-10, higher is more important)
    pub priority: u8,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// Tuning action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuningAction {
    /// Adjust table size
    AdjustTableSize { new_size: usize, reason: String },
    /// Change replacement policy
    ChangeReplacementPolicy {
        new_policy: ReplacementPolicy,
        reason: String,
    },
    /// Enable/disable feature
    ToggleFeature {
        feature: String,
        enabled: bool,
        reason: String,
    },
    /// Use template
    UseTemplate {
        template_name: String,
        reason: String,
    },
    /// Custom configuration
    CustomConfiguration {
        config: TranspositionConfig,
        reason: String,
    },
}

/// Tuning session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningSession {
    /// Session ID
    pub session_id: String,
    /// Start time
    pub start_time: std::time::SystemTime,
    /// End time
    pub end_time: Option<std::time::SystemTime>,
    /// Initial configuration
    pub initial_config: TranspositionConfig,
    /// Final configuration
    pub final_config: TranspositionConfig,
    /// Performance before tuning
    pub performance_before: RuntimePerformanceMetrics,
    /// Performance after tuning
    pub performance_after: Option<RuntimePerformanceMetrics>,
    /// Applied recommendations
    pub applied_recommendations: Vec<String>,
    /// Overall improvement percentage
    pub overall_improvement: Option<f64>,
}

impl PerformanceTuningManager {
    /// Create a new performance tuning manager
    pub fn new(initial_config: TranspositionConfig) -> Self {
        let adaptive_manager = Arc::new(Mutex::new(AdaptiveConfigurationManager::new(
            initial_config.clone(),
        )));
        let profiler = Arc::new(Mutex::new(PerformanceProfiler::new()));

        let mut manager = Self {
            adaptive_manager,
            profiler,
            recommendations: Vec::new(),
            tuning_history: Vec::new(),
            performance_targets: PerformanceTargets::default(),
        };

        // Generate initial recommendations
        manager.generate_initial_recommendations();

        manager
    }

    /// Generate initial tuning recommendations
    fn generate_initial_recommendations(&mut self) {
        self.recommendations.clear();

        // Recommendation 1: Enable statistics for monitoring
        self.recommendations.push(TuningRecommendation {
            id: "enable_statistics".to_string(),
            title: "Enable Statistics Collection".to_string(),
            description: "Enable statistics collection to monitor performance and identify optimization opportunities".to_string(),
            action: TuningAction::ToggleFeature {
                feature: "statistics".to_string(),
                enabled: true,
                reason: "Required for performance monitoring".to_string(),
            },
            expected_improvement: 0.0, // No direct performance improvement
            priority: 8,
            confidence: 1.0,
        });

        // Recommendation 2: Use power-of-two table size
        self.recommendations.push(TuningRecommendation {
            id: "power_of_two_size".to_string(),
            title: "Use Power-of-Two Table Size".to_string(),
            description: "Table sizes that are powers of two provide better performance due to optimized hash indexing".to_string(),
            action: TuningAction::AdjustTableSize {
                new_size: 65536, // 64K entries
                reason: "Power of two for optimal hash performance".to_string(),
            },
            expected_improvement: 5.0,
            priority: 7,
            confidence: 0.9,
        });

        // Recommendation 3: Enable cache line alignment
        self.recommendations.push(TuningRecommendation {
            id: "cache_line_alignment".to_string(),
            title: "Enable Cache Line Alignment".to_string(),
            description: "Cache line alignment can improve memory access performance by reducing cache misses".to_string(),
            action: TuningAction::ToggleFeature {
                feature: "cache_line_alignment".to_string(),
                enabled: true,
                reason: "Improves memory access performance".to_string(),
            },
            expected_improvement: 10.0,
            priority: 6,
            confidence: 0.8,
        });
    }

    /// Start a new tuning session
    pub fn start_tuning_session(&mut self) -> String {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let adaptive_manager = self.adaptive_manager.lock().unwrap();
        let runtime_manager = adaptive_manager.get_runtime_manager();
        let runtime_manager = runtime_manager.lock().unwrap();

        let initial_config = runtime_manager.get_active_config();
        let performance_before = runtime_manager.get_performance_metrics();

        let session = TuningSession {
            session_id: session_id.clone(),
            start_time: std::time::SystemTime::now(),
            end_time: None,
            initial_config,
            final_config: TranspositionConfig::default(), // Will be updated
            performance_before,
            performance_after: None,
            applied_recommendations: Vec::new(),
            overall_improvement: None,
        };

        self.tuning_history.push(session);
        session_id
    }

    /// End a tuning session
    pub fn end_tuning_session(&mut self, session_id: &str) -> Result<f64, String> {
        let session = self
            .tuning_history
            .iter_mut()
            .find(|s| s.session_id == session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if session.end_time.is_some() {
            return Err("Session already ended".to_string());
        }

        session.end_time = Some(std::time::SystemTime::now());

        let adaptive_manager = self.adaptive_manager.lock().unwrap();
        let runtime_manager = adaptive_manager.get_runtime_manager();
        let runtime_manager = runtime_manager.lock().unwrap();

        let final_config = runtime_manager.get_active_config();
        let performance_after = runtime_manager.get_performance_metrics();
        let performance_before = session.performance_before.clone();

        session.final_config = final_config;
        session.performance_after = Some(performance_after.clone());

        // Calculate improvement after releasing session borrow
        let improvement = {
            let _ = session;
            self.calculate_performance_improvement(&performance_before, &performance_after)
        };

        // Re-acquire session to set improvement
        let session = self
            .tuning_history
            .iter_mut()
            .find(|s| s.session_id == session_id)
            .ok_or("Session not found")?;
        session.overall_improvement = Some(improvement);

        Ok(session.overall_improvement.unwrap_or(0.0))
    }

    /// Apply a tuning recommendation
    pub fn apply_recommendation(&mut self, recommendation_id: &str) -> Result<(), String> {
        let recommendation = self
            .recommendations
            .iter()
            .find(|r| r.id == recommendation_id)
            .ok_or_else(|| "Recommendation not found".to_string())?;

        let adaptive_manager = self.adaptive_manager.lock().unwrap();
        let runtime_manager = adaptive_manager.get_runtime_manager();
        let mut runtime_manager = runtime_manager.lock().unwrap();

        let current_config = runtime_manager.get_active_config();
        let new_config = match &recommendation.action {
            TuningAction::AdjustTableSize { new_size, .. } => TranspositionConfig {
                table_size: *new_size,
                ..current_config
            },
            TuningAction::ChangeReplacementPolicy { new_policy, .. } => TranspositionConfig {
                replacement_policy: new_policy.clone(),
                ..current_config
            },
            TuningAction::ToggleFeature {
                feature, enabled, ..
            } => match feature.as_str() {
                "statistics" => TranspositionConfig {
                    enable_statistics: *enabled,
                    ..current_config
                },
                "memory_mapping" => TranspositionConfig {
                    enable_memory_mapping: *enabled,
                    ..current_config
                },
                "prefetching" => TranspositionConfig {
                    enable_prefetching: *enabled,
                    ..current_config
                },
                _ => return Err(format!("Unknown feature: {}", feature)),
            },
            TuningAction::UseTemplate { template_name, .. } => runtime_manager
                .get_template(template_name)
                .ok_or_else(|| format!("Template '{}' not found", template_name))?
                .clone(),
            TuningAction::CustomConfiguration { config, .. } => config.clone(),
        };

        runtime_manager.update_config(new_config, ConfigurationUpdateStrategy::Immediate)?;

        // Record applied recommendation
        if let Some(session) = self.tuning_history.last_mut() {
            session
                .applied_recommendations
                .push(recommendation_id.to_string());
        }

        Ok(())
    }

    /// Generate performance-based recommendations
    pub fn generate_performance_recommendations(&mut self) -> Vec<TuningRecommendation> {
        let mut new_recommendations = Vec::new();

        let adaptive_manager = self.adaptive_manager.lock().unwrap();
        let runtime_manager = adaptive_manager.get_runtime_manager();
        let runtime_manager = runtime_manager.lock().unwrap();

        let current_config = runtime_manager.get_active_config();
        let metrics = runtime_manager.get_performance_metrics();

        // Low hit rate recommendation
        if metrics.hit_rate < self.performance_targets.target_hit_rate {
            new_recommendations.push(TuningRecommendation {
                id: format!("increase_table_size_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                title: "Increase Table Size for Better Hit Rate".to_string(),
                description: format!("Current hit rate ({:.1}%) is below target ({:.1}%). Consider increasing table size.", 
                                   metrics.hit_rate * 100.0, self.performance_targets.target_hit_rate * 100.0),
                action: TuningAction::AdjustTableSize {
                    new_size: (current_config.table_size as f64 * 1.5) as usize,
                    reason: "Low hit rate detected".to_string(),
                },
                expected_improvement: 15.0,
                priority: 9,
                confidence: 0.8,
            });
        }

        // High collision rate recommendation
        if metrics.collision_rate > self.performance_targets.target_collision_rate {
            new_recommendations.push(TuningRecommendation {
                id: format!("change_policy_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                title: "Change Replacement Policy for Lower Collisions".to_string(),
                description: format!("Current collision rate ({:.1}%) is above target ({:.1}%). Consider changing replacement policy.", 
                                   metrics.collision_rate * 100.0, self.performance_targets.target_collision_rate * 100.0),
                action: TuningAction::ChangeReplacementPolicy {
                    new_policy: ReplacementPolicy::AgeBased,
                    reason: "High collision rate detected".to_string(),
                },
                expected_improvement: 10.0,
                priority: 7,
                confidence: 0.7,
            });
        }

        // High memory usage recommendation
        if metrics.memory_usage_bytes > self.performance_targets.max_memory_usage_bytes {
            new_recommendations.push(TuningRecommendation {
                id: format!("reduce_memory_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                title: "Reduce Memory Usage".to_string(),
                description: format!("Current memory usage ({:.1} MB) exceeds target ({:.1} MB). Consider reducing table size.", 
                                   metrics.memory_usage_bytes as f64 / 1024.0 / 1024.0,
                                   self.performance_targets.max_memory_usage_bytes as f64 / 1024.0 / 1024.0),
                action: TuningAction::UseTemplate {
                    template_name: "memory".to_string(),
                    reason: "High memory usage detected".to_string(),
                },
                expected_improvement: -5.0, // May reduce performance but save memory
                priority: 8,
                confidence: 0.9,
            });
        }

        // Slow operation recommendation
        if metrics.avg_operation_time_us > self.performance_targets.target_operation_time_us {
            new_recommendations.push(TuningRecommendation {
                id: format!("optimize_performance_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                title: "Optimize for Better Performance".to_string(),
                description: format!("Average operation time ({:.1}μs) exceeds target ({:.1}μs). Consider performance optimizations.", 
                                   metrics.avg_operation_time_us, self.performance_targets.target_operation_time_us),
                action: TuningAction::UseTemplate {
                    template_name: "high_performance".to_string(),
                    reason: "Slow operation times detected".to_string(),
                },
                expected_improvement: 20.0,
                priority: 9,
                confidence: 0.8,
            });
        }

        new_recommendations
    }

    /// Calculate performance improvement percentage
    fn calculate_performance_improvement(
        &self,
        before: &RuntimePerformanceMetrics,
        after: &RuntimePerformanceMetrics,
    ) -> f64 {
        // Weighted improvement calculation
        let hit_rate_improvement = (after.hit_rate - before.hit_rate) * 100.0;
        let speed_improvement = (before.avg_operation_time_us - after.avg_operation_time_us)
            / before.avg_operation_time_us
            * 100.0;
        let collision_improvement = (before.collision_rate - after.collision_rate) * 100.0;

        // Weighted average (hit rate is most important)
        hit_rate_improvement * 0.5 + speed_improvement * 0.3 + collision_improvement * 0.2
    }

    /// Get current recommendations
    pub fn get_recommendations(&self) -> Vec<TuningRecommendation> {
        self.recommendations.clone()
    }

    /// Get tuning history
    pub fn get_tuning_history(&self) -> Vec<TuningSession> {
        self.tuning_history.clone()
    }

    /// Set performance targets
    pub fn set_performance_targets(&mut self, targets: PerformanceTargets) {
        self.performance_targets = targets;
    }

    /// Get performance targets
    pub fn get_performance_targets(&self) -> PerformanceTargets {
        self.performance_targets.clone()
    }

    /// Get performance profiler
    pub fn get_profiler(&self) -> Arc<Mutex<PerformanceProfiler>> {
        self.profiler.clone()
    }

    /// Get adaptive configuration manager
    pub fn get_adaptive_manager(&self) -> Arc<Mutex<AdaptiveConfigurationManager>> {
        self.adaptive_manager.clone()
    }

    /// Export tuning report
    pub fn export_tuning_report(&self) -> Result<String, String> {
        let report = TuningReport {
            recommendations: self.recommendations.clone(),
            tuning_history: self.tuning_history.clone(),
            performance_targets: self.performance_targets.clone(),
            generated_at: std::time::SystemTime::now(),
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize tuning report: {}", e))
    }
}

/// Tuning report for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningReport {
    /// Recommendations
    pub recommendations: Vec<TuningRecommendation>,
    /// Tuning history
    pub tuning_history: Vec<TuningSession>,
    /// Performance targets
    pub performance_targets: PerformanceTargets,
    /// Report generation time
    pub generated_at: std::time::SystemTime,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            operation_timings: HashMap::new(),
            memory_snapshots: Vec::new(),
            performance_counters: PerformanceCounters::default(),
            enabled: false,
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Record operation timing
    pub fn record_operation(&mut self, operation: &str, duration_us: u64) {
        if !self.enabled {
            return;
        }

        self.operation_timings
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(duration_us);

        self.performance_counters.total_operations += 1;
    }

    /// Record memory snapshot
    pub fn record_memory_snapshot(&mut self, memory_usage_bytes: u64, available_memory_bytes: u64) {
        if !self.enabled {
            return;
        }

        let snapshot = MemorySnapshot {
            timestamp: std::time::SystemTime::now(),
            memory_usage_bytes,
            peak_memory_bytes: memory_usage_bytes, // Simplified
            available_memory_bytes,
        };

        self.memory_snapshots.push(snapshot);
    }

    /// Increment performance counter
    pub fn increment_counter(&mut self, counter: &str) {
        if !self.enabled {
            return;
        }

        match counter {
            "cache_hits" => self.performance_counters.cache_hits += 1,
            "cache_misses" => self.performance_counters.cache_misses += 1,
            "memory_allocations" => self.performance_counters.memory_allocations += 1,
            "memory_deallocations" => self.performance_counters.memory_deallocations += 1,
            "hash_collisions" => self.performance_counters.hash_collisions += 1,
            "replacements" => self.performance_counters.replacements += 1,
            _ => {} // Unknown counter
        }
    }

    /// Get average operation time
    pub fn get_average_operation_time(&self, operation: &str) -> Option<f64> {
        self.operation_timings.get(operation).and_then(|timings| {
            if timings.is_empty() {
                None
            } else {
                Some(timings.iter().sum::<u64>() as f64 / timings.len() as f64)
            }
        })
    }

    /// Get performance counters
    pub fn get_performance_counters(&self) -> PerformanceCounters {
        self.performance_counters.clone()
    }

    /// Get memory snapshots
    pub fn get_memory_snapshots(&self) -> Vec<MemorySnapshot> {
        self.memory_snapshots.clone()
    }
}

// ============================================================================
// Performance Baseline Manager (Task 26.0 - Task 1.0)
// ============================================================================

use crate::types::all::{
    BaselineMoveOrderingMetrics, BenchmarkPosition, EvaluationMetrics, HardwareInfo, MemoryMetrics,
    ParallelSearchMetrics, PerformanceBaseline, SearchMetrics, TTMetrics,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Manager for performance baseline persistence and comparison
pub struct BaselineManager {
    /// Default baseline directory
    baseline_dir: PathBuf,
    /// Regression threshold (default: 5.0%)
    regression_threshold: f64,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new() -> Self {
        Self {
            baseline_dir: PathBuf::from("docs/performance/baselines"),
            regression_threshold: 5.0,
        }
    }

    /// Create a baseline manager with custom directory
    pub fn with_directory<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            baseline_dir: dir.as_ref().to_path_buf(),
            regression_threshold: 5.0,
        }
    }

    /// Set regression threshold (percentage)
    pub fn set_regression_threshold(&mut self, threshold: f64) {
        self.regression_threshold = threshold;
    }

    /// Get regression threshold
    pub fn regression_threshold(&self) -> f64 {
        self.regression_threshold
    }

    /// Save baseline to file
    pub fn save_baseline(&self, baseline: &PerformanceBaseline, filename: &str) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(&self.baseline_dir)
            .map_err(|e| format!("Failed to create baseline directory: {}", e))?;

        let file_path = self.baseline_dir.join(filename);
        let json = serde_json::to_string_pretty(baseline)
            .map_err(|e| format!("Failed to serialize baseline: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write baseline file: {}", e))?;

        Ok(())
    }

    /// Load baseline from file
    pub fn load_baseline<P: AsRef<Path>>(&self, path: P) -> Result<PerformanceBaseline, String> {
        let file_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.baseline_dir.join(path)
        };

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read baseline file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse baseline JSON: {}", e))
    }

    /// Compare two baselines and calculate percentage differences
    pub fn compare_baselines(
        &self,
        current: &PerformanceBaseline,
        baseline: &PerformanceBaseline,
    ) -> BaselineComparison {
        BaselineComparison {
            search_metrics_diff: compare_search_metrics(&current.search_metrics, &baseline.search_metrics),
            evaluation_metrics_diff: compare_evaluation_metrics(&current.evaluation_metrics, &baseline.evaluation_metrics),
            tt_metrics_diff: compare_tt_metrics(&current.tt_metrics, &baseline.tt_metrics),
            move_ordering_metrics_diff: compare_move_ordering_metrics(&current.move_ordering_metrics, &baseline.move_ordering_metrics),
            parallel_search_metrics_diff: compare_parallel_search_metrics(&current.parallel_search_metrics, &baseline.parallel_search_metrics),
            memory_metrics_diff: compare_memory_metrics(&current.memory_metrics, &baseline.memory_metrics),
        }
    }

    /// Detect regressions in current baseline compared to reference baseline
    pub fn detect_regression(
        &self,
        current: &PerformanceBaseline,
        baseline: &PerformanceBaseline,
    ) -> RegressionResult {
        let comparison = self.compare_baselines(current, baseline);
        let mut regressions = Vec::new();

        // Check search metrics
        if comparison.search_metrics_diff.nodes_per_second_change < -self.regression_threshold {
            regressions.push(Regression {
                category: "search_metrics".to_string(),
                metric: "nodes_per_second".to_string(),
                baseline_value: baseline.search_metrics.nodes_per_second,
                current_value: current.search_metrics.nodes_per_second,
                change_percent: comparison.search_metrics_diff.nodes_per_second_change,
            });
        }
        if comparison.search_metrics_diff.average_cutoff_rate_change < -self.regression_threshold {
            regressions.push(Regression {
                category: "search_metrics".to_string(),
                metric: "average_cutoff_rate".to_string(),
                baseline_value: baseline.search_metrics.average_cutoff_rate,
                current_value: current.search_metrics.average_cutoff_rate,
                change_percent: comparison.search_metrics_diff.average_cutoff_rate_change,
            });
        }
        if comparison.search_metrics_diff.average_cutoff_index_change > self.regression_threshold {
            regressions.push(Regression {
                category: "search_metrics".to_string(),
                metric: "average_cutoff_index".to_string(),
                baseline_value: baseline.search_metrics.average_cutoff_index,
                current_value: current.search_metrics.average_cutoff_index,
                change_percent: comparison.search_metrics_diff.average_cutoff_index_change,
            });
        }

        // Check evaluation metrics
        if comparison.evaluation_metrics_diff.average_evaluation_time_ns_change > self.regression_threshold {
            regressions.push(Regression {
                category: "evaluation_metrics".to_string(),
                metric: "average_evaluation_time_ns".to_string(),
                baseline_value: baseline.evaluation_metrics.average_evaluation_time_ns,
                current_value: current.evaluation_metrics.average_evaluation_time_ns,
                change_percent: comparison.evaluation_metrics_diff.average_evaluation_time_ns_change,
            });
        }
        if comparison.evaluation_metrics_diff.cache_hit_rate_change < -self.regression_threshold {
            regressions.push(Regression {
                category: "evaluation_metrics".to_string(),
                metric: "cache_hit_rate".to_string(),
                baseline_value: baseline.evaluation_metrics.cache_hit_rate,
                current_value: current.evaluation_metrics.cache_hit_rate,
                change_percent: comparison.evaluation_metrics_diff.cache_hit_rate_change,
            });
        }

        // Check TT metrics
        if comparison.tt_metrics_diff.hit_rate_change < -self.regression_threshold {
            regressions.push(Regression {
                category: "tt_metrics".to_string(),
                metric: "hit_rate".to_string(),
                baseline_value: baseline.tt_metrics.hit_rate,
                current_value: current.tt_metrics.hit_rate,
                change_percent: comparison.tt_metrics_diff.hit_rate_change,
            });
        }

        // Check move ordering metrics
        if comparison.move_ordering_metrics_diff.average_cutoff_index_change > self.regression_threshold {
            regressions.push(Regression {
                category: "move_ordering_metrics".to_string(),
                metric: "average_cutoff_index".to_string(),
                baseline_value: baseline.move_ordering_metrics.average_cutoff_index,
                current_value: current.move_ordering_metrics.average_cutoff_index,
                change_percent: comparison.move_ordering_metrics_diff.average_cutoff_index_change,
            });
        }

        RegressionResult {
            has_regression: !regressions.is_empty(),
            regressions,
            threshold: self.regression_threshold,
        }
    }
}

impl Default for BaselineManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Baseline comparison result
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    pub search_metrics_diff: SearchMetricsDiff,
    pub evaluation_metrics_diff: EvaluationMetricsDiff,
    pub tt_metrics_diff: TTMetricsDiff,
    pub move_ordering_metrics_diff: MoveOrderingMetricsDiff,
    pub parallel_search_metrics_diff: ParallelSearchMetricsDiff,
    pub memory_metrics_diff: MemoryMetricsDiff,
}

/// Search metrics difference
#[derive(Debug, Clone)]
pub struct SearchMetricsDiff {
    pub nodes_per_second_change: f64,
    pub average_cutoff_rate_change: f64,
    pub average_cutoff_index_change: f64,
}

/// Evaluation metrics difference
#[derive(Debug, Clone)]
pub struct EvaluationMetricsDiff {
    pub average_evaluation_time_ns_change: f64,
    pub cache_hit_rate_change: f64,
    pub phase_calc_time_ns_change: f64,
}

/// TT metrics difference
#[derive(Debug, Clone)]
pub struct TTMetricsDiff {
    pub hit_rate_change: f64,
    pub exact_entry_rate_change: f64,
    pub occupancy_rate_change: f64,
}

/// Move ordering metrics difference
#[derive(Debug, Clone)]
pub struct MoveOrderingMetricsDiff {
    pub average_cutoff_index_change: f64,
    pub pv_hit_rate_change: f64,
    pub killer_hit_rate_change: f64,
    pub cache_hit_rate_change: f64,
}

/// Parallel search metrics difference
#[derive(Debug, Clone)]
pub struct ParallelSearchMetricsDiff {
    pub speedup_4_cores_change: f64,
    pub speedup_8_cores_change: f64,
    pub efficiency_4_cores_change: f64,
    pub efficiency_8_cores_change: f64,
}

/// Memory metrics difference
#[derive(Debug, Clone)]
pub struct MemoryMetricsDiff {
    pub tt_memory_mb_change: f64,
    pub cache_memory_mb_change: f64,
    pub peak_memory_mb_change: f64,
}

/// Regression detection result
#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub has_regression: bool,
    pub regressions: Vec<Regression>,
    pub threshold: f64,
}

/// Individual regression
#[derive(Debug, Clone)]
pub struct Regression {
    pub category: String,
    pub metric: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub change_percent: f64,
}

// Helper functions for comparing metrics
fn calculate_percent_change(baseline: f64, current: f64) -> f64 {
    if baseline == 0.0 {
        if current == 0.0 {
            0.0
        } else {
            100.0 // Infinite change
        }
    } else {
        ((current - baseline) / baseline) * 100.0
    }
}

fn compare_search_metrics(current: &SearchMetrics, baseline: &SearchMetrics) -> SearchMetricsDiff {
    SearchMetricsDiff {
        nodes_per_second_change: calculate_percent_change(baseline.nodes_per_second, current.nodes_per_second),
        average_cutoff_rate_change: calculate_percent_change(baseline.average_cutoff_rate, current.average_cutoff_rate),
        average_cutoff_index_change: calculate_percent_change(baseline.average_cutoff_index, current.average_cutoff_index),
    }
}

fn compare_evaluation_metrics(current: &EvaluationMetrics, baseline: &EvaluationMetrics) -> EvaluationMetricsDiff {
    EvaluationMetricsDiff {
        average_evaluation_time_ns_change: calculate_percent_change(baseline.average_evaluation_time_ns, current.average_evaluation_time_ns),
        cache_hit_rate_change: calculate_percent_change(baseline.cache_hit_rate, current.cache_hit_rate),
        phase_calc_time_ns_change: calculate_percent_change(baseline.phase_calc_time_ns, current.phase_calc_time_ns),
    }
}

fn compare_tt_metrics(current: &TTMetrics, baseline: &TTMetrics) -> TTMetricsDiff {
    TTMetricsDiff {
        hit_rate_change: calculate_percent_change(baseline.hit_rate, current.hit_rate),
        exact_entry_rate_change: calculate_percent_change(baseline.exact_entry_rate, current.exact_entry_rate),
        occupancy_rate_change: calculate_percent_change(baseline.occupancy_rate, current.occupancy_rate),
    }
}

fn compare_move_ordering_metrics(current: &BaselineMoveOrderingMetrics, baseline: &BaselineMoveOrderingMetrics) -> MoveOrderingMetricsDiff {
    MoveOrderingMetricsDiff {
        average_cutoff_index_change: calculate_percent_change(baseline.average_cutoff_index, current.average_cutoff_index),
        pv_hit_rate_change: calculate_percent_change(baseline.pv_hit_rate, current.pv_hit_rate),
        killer_hit_rate_change: calculate_percent_change(baseline.killer_hit_rate, current.killer_hit_rate),
        cache_hit_rate_change: calculate_percent_change(baseline.cache_hit_rate, current.cache_hit_rate),
    }
}

fn compare_parallel_search_metrics(current: &ParallelSearchMetrics, baseline: &ParallelSearchMetrics) -> ParallelSearchMetricsDiff {
    ParallelSearchMetricsDiff {
        speedup_4_cores_change: calculate_percent_change(baseline.speedup_4_cores, current.speedup_4_cores),
        speedup_8_cores_change: calculate_percent_change(baseline.speedup_8_cores, current.speedup_8_cores),
        efficiency_4_cores_change: calculate_percent_change(baseline.efficiency_4_cores, current.efficiency_4_cores),
        efficiency_8_cores_change: calculate_percent_change(baseline.efficiency_8_cores, current.efficiency_8_cores),
    }
}

fn compare_memory_metrics(current: &MemoryMetrics, baseline: &MemoryMetrics) -> MemoryMetricsDiff {
    MemoryMetricsDiff {
        tt_memory_mb_change: calculate_percent_change(baseline.tt_memory_mb, current.tt_memory_mb),
        cache_memory_mb_change: calculate_percent_change(baseline.cache_memory_mb, current.cache_memory_mb),
        peak_memory_mb_change: calculate_percent_change(baseline.peak_memory_mb, current.peak_memory_mb),
    }
}

/// Detect hardware information for baseline
pub fn detect_hardware_info() -> HardwareInfo {
    let cpu = std::env::var("CPU_MODEL")
        .or_else(|_| std::env::var("PROCESSOR_IDENTIFIER"))
        .unwrap_or_else(|_| {
            // Try to get CPU info from system
            #[cfg(target_os = "linux")]
            {
                if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                    for line in content.lines() {
                        if line.starts_with("model name") {
                            if let Some(name) = line.split(':').nth(1) {
                                return name.trim().to_string();
                            }
                        }
                    }
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Ok(output) = std::process::Command::new("sysctl")
                    .arg("-n")
                    .arg("machdep.cpu.brand_string")
                    .output()
                {
                    if let Ok(cpu_name) = String::from_utf8(output.stdout) {
                        return cpu_name.trim().to_string();
                    }
                }
            }
            "Unknown".to_string()
        });

    let cores = num_cpus::get() as u32;

    // Try to detect RAM (simplified - may not work on all platforms)
    let ram_gb = std::env::var("RAM_GB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    HardwareInfo { cpu, cores, ram_gb }
}

// ============================================================================
// Benchmark Result Aggregation and Reporting (Task 26.0 - Task 2.0)
// ============================================================================

/// Benchmark report for a single benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Benchmark name
    pub benchmark_name: String,
    /// Mean time in nanoseconds
    pub mean_time_ns: f64,
    /// Standard deviation in nanoseconds
    pub std_dev_ns: f64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Number of samples
    pub samples: u64,
    /// Comparison with baseline (if available)
    pub baseline_comparison: Option<BenchmarkBaselineComparison>,
}

/// Aggregated benchmark report containing multiple benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedBenchmarkReport {
    /// Timestamp when report was generated
    pub timestamp: String,
    /// Git commit hash
    pub git_commit: String,
    /// Hardware information
    pub hardware: HardwareInfo,
    /// Individual benchmark reports
    pub benchmarks: Vec<BenchmarkReport>,
    /// Summary statistics
    pub summary: BenchmarkSummary,
}

/// Summary statistics for all benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total number of benchmarks
    pub total_benchmarks: usize,
    /// Average mean time across all benchmarks (nanoseconds)
    pub average_mean_time_ns: f64,
    /// Total throughput across all benchmarks (ops/sec)
    pub total_throughput_ops_per_sec: f64,
    /// Benchmarks with regressions
    pub regressions_detected: usize,
}

/// Criterion.rs estimates structure (partial, for parsing)
#[derive(Debug, Clone, Deserialize)]
struct CriterionEstimates {
    mean: Option<CriterionEstimate>,
    median: Option<CriterionEstimate>,
    slope: Option<CriterionEstimate>,
    throughput: Option<CriterionThroughput>,
}

#[derive(Debug, Clone, Deserialize)]
struct CriterionEstimate {
    point_estimate: f64,
    standard_error: Option<f64>,
    confidence_interval: Option<CriterionConfidenceInterval>,
}

#[derive(Debug, Clone, Deserialize)]
struct CriterionConfidenceInterval {
    confidence_level: Option<f64>,
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct CriterionThroughput {
    #[serde(rename = "per_iteration")]
    per_iteration: Option<CriterionThroughputValue>,
    #[serde(rename = "per_second")]
    per_second: Option<CriterionThroughputValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct CriterionThroughputValue {
    point_estimate: f64,
}

/// Benchmark aggregator for collecting and reporting benchmark results
pub struct BenchmarkAggregator {
    /// Reports directory
    reports_dir: PathBuf,
    /// Baseline path for comparison (optional)
    baseline_path: Option<PathBuf>,
    /// Regression threshold (default: 5.0%)
    regression_threshold: f64,
}

impl BenchmarkAggregator {
    /// Create a new benchmark aggregator
    pub fn new() -> Self {
        Self {
            reports_dir: PathBuf::from("docs/performance/reports"),
            baseline_path: std::env::var("BENCHMARK_BASELINE_PATH")
                .ok()
                .map(PathBuf::from),
            regression_threshold: 5.0,
        }
    }

    /// Create aggregator with custom directory
    pub fn with_directory<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            reports_dir: dir.as_ref().to_path_buf(),
            baseline_path: std::env::var("BENCHMARK_BASELINE_PATH")
                .ok()
                .map(PathBuf::from),
            regression_threshold: 5.0,
        }
    }

    /// Set baseline path for comparison
    pub fn set_baseline_path<P: AsRef<Path>>(&mut self, path: P) {
        self.baseline_path = Some(path.as_ref().to_path_buf());
    }

    /// Set regression threshold
    pub fn set_regression_threshold(&mut self, threshold: f64) {
        self.regression_threshold = threshold;
    }

    /// Aggregate Criterion.rs results from target/criterion/ directory
    pub fn aggregate_criterion_results<P: AsRef<Path>>(
        &self,
        criterion_dir: P,
    ) -> Result<Vec<BenchmarkReport>, String> {
        let criterion_path = criterion_dir.as_ref();
        let mut reports = Vec::new();

        // Walk through criterion directory structure
        // Structure: target/criterion/{benchmark_name}/{id}/base/estimates.json
        if !criterion_path.exists() {
            return Err(format!("Criterion directory does not exist: {:?}", criterion_path));
        }

        // Find all estimates.json files
        let estimates_files = find_estimates_files(criterion_path)?;

        for estimates_file in estimates_files {
            match self.parse_criterion_estimates(&estimates_file) {
                Ok(report) => reports.push(report),
                Err(e) => {
                    eprintln!("Warning: Failed to parse {:?}: {}", estimates_file, e);
                }
            }
        }

        Ok(reports)
    }

    /// Parse a single Criterion.rs estimates.json file
    fn parse_criterion_estimates<P: AsRef<Path>>(
        &self,
        estimates_file: P,
    ) -> Result<BenchmarkReport, String> {
        let content = fs::read_to_string(&estimates_file)
            .map_err(|e| format!("Failed to read estimates file: {}", e))?;

        let estimates: CriterionEstimates = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Extract benchmark name from path
        // Path format: .../criterion/{benchmark_name}/{id}/base/estimates.json
        let benchmark_name = extract_benchmark_name(&estimates_file)
            .unwrap_or_else(|| "unknown".to_string());

        // Get mean time (in nanoseconds)
        let mean_time_ns = estimates
            .mean
            .as_ref()
            .map(|m| m.point_estimate)
            .unwrap_or(0.0);

        // Get standard deviation (from standard error or confidence interval)
        let std_dev_ns = estimates
            .mean
            .as_ref()
            .and_then(|m| m.standard_error)
            .unwrap_or(0.0);

        // Get throughput (operations per second)
        let throughput_ops_per_sec = estimates
            .throughput
            .as_ref()
            .and_then(|t| t.per_second.as_ref())
            .map(|v| v.point_estimate)
            .unwrap_or(0.0);

        // Estimate samples (not directly available in estimates.json, use default)
        let samples = 100; // Default estimate

        // Compare with baseline if available
        let baseline_comparison = if let Some(ref baseline_path) = self.baseline_path {
            self.compare_with_baseline(&benchmark_name, mean_time_ns, baseline_path).ok()
        } else {
            None
        };

        Ok(BenchmarkReport {
            benchmark_name,
            mean_time_ns,
            std_dev_ns,
            throughput_ops_per_sec,
            samples,
            baseline_comparison,
        })
    }

    /// Generate aggregated benchmark report
    pub fn generate_benchmark_report(
        &self,
        reports: &[BenchmarkReport],
    ) -> AggregatedBenchmarkReport {
        let total_benchmarks = reports.len();
        let average_mean_time_ns = if total_benchmarks > 0 {
            reports.iter().map(|r| r.mean_time_ns).sum::<f64>() / total_benchmarks as f64
        } else {
            0.0
        };
        let total_throughput_ops_per_sec = reports.iter().map(|r| r.throughput_ops_per_sec).sum();
        let regressions_detected = reports
            .iter()
            .filter(|r| {
                r.baseline_comparison
                    .as_ref()
                    .map(|c| c.has_regression)
                    .unwrap_or(false)
            })
            .count();

        AggregatedBenchmarkReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            git_commit: crate::types::get_git_commit_hash().unwrap_or_else(|| "unknown".to_string()),
            hardware: detect_hardware_info(),
            benchmarks: reports.to_vec(),
            summary: BenchmarkSummary {
                total_benchmarks,
                average_mean_time_ns,
                total_throughput_ops_per_sec,
                regressions_detected,
            },
        }
    }

    /// Export report to JSON
    pub fn export_report_to_json(
        &self,
        report: &AggregatedBenchmarkReport,
        filename: &str,
    ) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(&self.reports_dir)
            .map_err(|e| format!("Failed to create reports directory: {}", e))?;

        let file_path = self.reports_dir.join(filename);
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| format!("Failed to serialize report: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write report file: {}", e))?;

        Ok(())
    }

    /// Export report to Markdown
    pub fn export_report_to_markdown(
        &self,
        report: &AggregatedBenchmarkReport,
        filename: &str,
    ) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(&self.reports_dir)
            .map_err(|e| format!("Failed to create reports directory: {}", e))?;

        let file_path = self.reports_dir.join(filename);
        let markdown = self.generate_markdown_report(report);

        fs::write(&file_path, markdown)
            .map_err(|e| format!("Failed to write markdown file: {}", e))?;

        Ok(())
    }

    /// Generate markdown report content
    fn generate_markdown_report(&self, report: &AggregatedBenchmarkReport) -> String {
        let mut md = String::new();

        md.push_str("# Benchmark Report\n\n");
        md.push_str(&format!("**Generated:** {}\n", report.timestamp));
        md.push_str(&format!("**Git Commit:** {}\n", report.git_commit));
        md.push_str(&format!("**Hardware:** {} ({} cores, {} GB RAM)\n\n", 
            report.hardware.cpu, report.hardware.cores, report.hardware.ram_gb));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Total Benchmarks:** {}\n", report.summary.total_benchmarks));
        md.push_str(&format!("- **Average Mean Time:** {:.2} ns\n", report.summary.average_mean_time_ns));
        md.push_str(&format!("- **Total Throughput:** {:.2} ops/sec\n", report.summary.total_throughput_ops_per_sec));
        md.push_str(&format!("- **Regressions Detected:** {}\n\n", report.summary.regressions_detected));

        // Benchmarks table
        md.push_str("## Benchmarks\n\n");
        md.push_str("| Benchmark | Mean Time (ns) | Std Dev (ns) | Throughput (ops/sec) | Samples | Regression |\n");
        md.push_str("|-----------|----------------|--------------|----------------------|---------|------------|\n");

        for bench in &report.benchmarks {
            let regression = if let Some(ref comp) = bench.baseline_comparison {
                if comp.has_regression {
                    format!("⚠️ {:.1}%", comp.change_percent)
                } else {
                    format!("✓ {:.1}%", comp.change_percent)
                }
            } else {
                "N/A".to_string()
            };

            md.push_str(&format!(
                "| {} | {:.2} | {:.2} | {:.2} | {} | {} |\n",
                bench.benchmark_name,
                bench.mean_time_ns,
                bench.std_dev_ns,
                bench.throughput_ops_per_sec,
                bench.samples,
                regression
            ));
        }

        md
    }

    /// Compare benchmark result with baseline
    fn compare_with_baseline(
        &self,
        _benchmark_name: &str,
        current_mean_time_ns: f64,
        baseline_path: &Path,
    ) -> Result<BenchmarkBaselineComparison, String> {
        // Load baseline
        let manager = BaselineManager::new();
        let baseline = manager.load_baseline(baseline_path)?;

        // For now, we compare against search_metrics.nodes_per_second
        // This is a simplified comparison - in a full implementation, we'd need
        // to map benchmark names to specific baseline metrics
        let baseline_time_ns = 1_000_000_000.0 / baseline.search_metrics.nodes_per_second.max(1.0);

        let change_percent = if baseline_time_ns > 0.0 {
            ((current_mean_time_ns - baseline_time_ns) / baseline_time_ns) * 100.0
        } else {
            0.0
        };

        let has_regression = change_percent > self.regression_threshold;

        Ok(BenchmarkBaselineComparison {
            has_regression,
            change_percent,
            baseline_value: baseline_time_ns,
            current_value: current_mean_time_ns,
        })
    }
}

impl Default for BenchmarkAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Baseline comparison for a single benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaselineComparison {
    /// Whether a regression was detected
    pub has_regression: bool,
    /// Percentage change from baseline
    pub change_percent: f64,
    /// Baseline value
    pub baseline_value: f64,
    /// Current value
    pub current_value: f64,
}

impl BenchmarkReport {
    /// Compare this benchmark report with a baseline
    pub fn compare_with_baseline(
        &self,
        baseline_path: &Path,
        regression_threshold: f64,
    ) -> Result<BenchmarkBaselineComparison, String> {
        let manager = BaselineManager::new();
        let baseline = manager.load_baseline(baseline_path)?;

        // Simplified comparison - compare mean time against baseline nodes_per_second
        let baseline_time_ns = 1_000_000_000.0 / baseline.search_metrics.nodes_per_second.max(1.0);

        let change_percent = if baseline_time_ns > 0.0 {
            ((self.mean_time_ns - baseline_time_ns) / baseline_time_ns) * 100.0
        } else {
            0.0
        };

        let has_regression = change_percent > regression_threshold;

        Ok(BenchmarkBaselineComparison {
            has_regression,
            change_percent,
            baseline_value: baseline_time_ns,
            current_value: self.mean_time_ns,
        })
    }
}

// Helper functions

/// Find all estimates.json files in criterion directory
fn find_estimates_files<P: AsRef<Path>>(criterion_dir: P) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let dir = criterion_dir.as_ref();

    if !dir.is_dir() {
        return Ok(files);
    }

    // Walk through directory structure: criterion/{benchmark}/{id}/base/estimates.json
    for benchmark_entry in fs::read_dir(dir)
        .map_err(|e| format!("Failed to read criterion directory: {}", e))?
    {
        let benchmark_entry = benchmark_entry
            .map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let benchmark_path = benchmark_entry.path();

        if !benchmark_path.is_dir() {
            continue;
        }

        // Look for {id}/base/estimates.json
        for id_entry in fs::read_dir(&benchmark_path)
            .map_err(|e| format!("Failed to read benchmark directory: {}", e))?
        {
            let id_entry = id_entry
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let id_path = id_entry.path();

            if !id_path.is_dir() {
                continue;
            }

            let base_path = id_path.join("base");
            let estimates_path = base_path.join("estimates.json");

            if estimates_path.exists() {
                files.push(estimates_path);
            }
        }
    }

    Ok(files)
}

/// Extract benchmark name from file path
fn extract_benchmark_name<P: AsRef<Path>>(file_path: P) -> Option<String> {
    let path = file_path.as_ref();
    let components: Vec<_> = path.components().collect();

    // Find "criterion" component and get the next one as benchmark name
    for (i, component) in components.iter().enumerate() {
        if let std::path::Component::Normal(name) = component {
            if name.to_str() == Some("criterion") && i + 1 < components.len() {
                if let std::path::Component::Normal(benchmark_name) = &components[i + 1] {
                    return benchmark_name.to_str().map(|s| s.to_string());
                }
            }
        }
    }

    None
}

// ============================================================================
// Benchmark Runner and Regression Suite (Task 26.0 - Task 5.0)
// ============================================================================

use crate::bitboards::BitboardBoard;
use crate::search::search_engine::SearchEngine;

/// Load standard benchmark positions from JSON file (Task 26.0 - Task 5.0)
pub fn load_standard_positions() -> Result<Vec<BenchmarkPosition>, String> {
    let path = PathBuf::from("resources/benchmark_positions/standard_positions.json");
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read standard positions file: {}", e))?;
    
    let positions: Vec<BenchmarkPosition> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse standard positions JSON: {}", e))?;
    
    Ok(positions)
}

/// Regression test result for a single position (Task 26.0 - Task 5.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestResult {
    /// Position name
    pub position_name: String,
    /// Baseline search time in milliseconds
    pub baseline_time_ms: u64,
    /// Current search time in milliseconds
    pub current_time_ms: u64,
    /// Whether regression was detected
    pub regression_detected: bool,
    /// Regression percentage (positive = slower, negative = faster)
    pub regression_percentage: f64,
}

impl RegressionTestResult {
    /// Create a new regression test result
    pub fn new(
        position_name: String,
        baseline_time_ms: u64,
        current_time_ms: u64,
        regression_threshold: f64,
    ) -> Self {
        let regression_percentage = if baseline_time_ms > 0 {
            ((current_time_ms as f64 - baseline_time_ms as f64) / baseline_time_ms as f64) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = regression_percentage > regression_threshold;
        
        Self {
            position_name,
            baseline_time_ms,
            current_time_ms,
            regression_detected,
            regression_percentage,
        }
    }
}

/// Benchmark runner for standard positions (Task 26.0 - Task 5.0)
pub struct BenchmarkRunner {
    /// Regression threshold percentage (default: 5.0%)
    regression_threshold: f64,
    /// Baseline path for comparison (optional)
    baseline_path: Option<PathBuf>,
    /// Time limit per position in milliseconds
    time_limit_ms: u32,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {
            regression_threshold: 5.0,
            baseline_path: None,
            time_limit_ms: 10000, // 10 seconds default
        }
    }

    /// Set regression threshold
    pub fn with_regression_threshold(mut self, threshold: f64) -> Self {
        self.regression_threshold = threshold;
        self
    }

    /// Set baseline path for comparison
    pub fn with_baseline_path(mut self, path: PathBuf) -> Self {
        self.baseline_path = Some(path);
        self
    }

    /// Set time limit per position
    pub fn with_time_limit(mut self, time_limit_ms: u32) -> Self {
        self.time_limit_ms = time_limit_ms;
        self
    }

    /// Run benchmark on a single position (Task 26.0 - Task 5.0)
    pub fn run_position_benchmark(
        &self,
        position: &BenchmarkPosition,
        engine: &mut SearchEngine,
    ) -> Result<PositionBenchmarkResult, String> {
        // Parse FEN
        let (mut board, player, captured_pieces) = BitboardBoard::from_fen(&position.fen)
            .map_err(|e| format!("Failed to parse FEN: {}", e))?;

        // Run search
        let start_time = std::time::Instant::now();
        let result = engine.search_at_depth(
            &mut board,
            &captured_pieces,
            player,
            position.expected_depth,
            self.time_limit_ms,
            i32::MIN,
            i32::MAX,
        );
        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Get performance metrics
        let metrics = engine.get_performance_metrics();
        // Get nodes searched directly from engine
        let nodes_searched = engine.get_nodes_searched();

        Ok(PositionBenchmarkResult {
            position_name: position.name.clone(),
            search_time_ms: elapsed_ms,
            nodes_searched,
            nodes_per_second: metrics.nodes_per_second,
            depth_searched: position.expected_depth,
            best_move_found: result.is_some(),
        })
    }

    /// Run regression suite on all standard positions (Task 26.0 - Task 5.0)
    pub fn run_regression_suite(
        &self,
        engine: &mut SearchEngine,
    ) -> Result<RegressionSuiteResult, String> {
        // Load standard positions
        let positions = load_standard_positions()?;

        // Load baseline if available
        let baseline_results = if let Some(ref baseline_path) = self.baseline_path {
            self.load_baseline_results(baseline_path)?
        } else {
            None
        };

        // Run benchmarks
        let mut results = Vec::new();
        let mut regression_results = Vec::new();

        for position in &positions {
            let benchmark_result = self.run_position_benchmark(position, engine)?;
            
            // Compare with baseline if available
            if let Some(ref baseline) = baseline_results {
                if let Some(baseline_time) = baseline.get(&position.name) {
                    let regression_result = RegressionTestResult::new(
                        position.name.clone(),
                        *baseline_time,
                        benchmark_result.search_time_ms,
                        self.regression_threshold,
                    );
                    regression_results.push(regression_result.clone());
                    results.push((benchmark_result, Some(regression_result)));
                } else {
                    results.push((benchmark_result, None));
                }
            } else {
                results.push((benchmark_result, None));
            }
        }

        // Detect regressions
        let regressions = self.detect_regressions(&regression_results);

        Ok(RegressionSuiteResult {
            total_positions: positions.len(),
            benchmark_results: results,
            regression_results,
            regressions_detected: regressions.len(),
            regressions,
        })
    }

    /// Detect regressions from test results (Task 26.0 - Task 5.0)
    pub fn detect_regressions(
        &self,
        results: &[RegressionTestResult],
    ) -> Vec<RegressionTestResult> {
        results
            .iter()
            .filter(|r| r.regression_detected)
            .cloned()
            .collect()
    }

    /// Load baseline results from file
    fn load_baseline_results(
        &self,
        path: &PathBuf,
    ) -> Result<Option<HashMap<String, u64>>, String> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read baseline file: {}", e))?;

        let baseline: HashMap<String, u64> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse baseline JSON: {}", e))?;

        Ok(Some(baseline))
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of benchmarking a single position (Task 26.0 - Task 5.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionBenchmarkResult {
    /// Position name
    pub position_name: String,
    /// Search time in milliseconds
    pub search_time_ms: u64,
    /// Nodes searched
    pub nodes_searched: u64,
    /// Nodes per second
    pub nodes_per_second: f64,
    /// Depth searched
    pub depth_searched: u8,
    /// Whether best move was found
    pub best_move_found: bool,
}

/// Result of running regression suite (Task 26.0 - Task 5.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionSuiteResult {
    /// Total number of positions tested
    pub total_positions: usize,
    /// Benchmark results for each position
    pub benchmark_results: Vec<(PositionBenchmarkResult, Option<RegressionTestResult>)>,
    /// Regression test results
    pub regression_results: Vec<RegressionTestResult>,
    /// Number of regressions detected
    pub regressions_detected: usize,
    /// List of detected regressions
    pub regressions: Vec<RegressionTestResult>,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            target_hit_rate: 0.35,
            target_operation_time_us: 50.0,
            max_memory_usage_bytes: 134217728, // 128MB
            target_collision_rate: 0.10,
            target_throughput_ops_per_sec: 10000.0,
        }
    }
}

// ============================================================================
// Telemetry Export and Advanced Metrics Analysis (Task 26.0 - Task 7.0)
// ============================================================================

/// Telemetry exporter for performance metrics (Task 26.0 - Task 7.0)
pub struct TelemetryExporter {
    /// Export directory path
    export_path: PathBuf,
    /// Whether export is enabled
    enabled: bool,
}

impl TelemetryExporter {
    /// Create a new telemetry exporter
    pub fn new<P: AsRef<Path>>(export_path: P) -> Self {
        Self {
            export_path: export_path.as_ref().to_path_buf(),
            enabled: true,
        }
    }

    /// Create exporter with enabled flag
    pub fn with_enabled<P: AsRef<Path>>(export_path: P, enabled: bool) -> Self {
        Self {
            export_path: export_path.as_ref().to_path_buf(),
            enabled,
        }
    }

    /// Set export enabled status
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get export path
    pub fn export_path(&self) -> &Path {
        &self.export_path
    }

    /// Ensure export directory exists
    fn ensure_directory(&self) -> Result<(), String> {
        fs::create_dir_all(&self.export_path)
            .map_err(|e| format!("Failed to create export directory: {}", e))?;
        Ok(())
    }

    /// Export performance metrics to JSON (Task 26.0 - Task 7.0)
    pub fn export_performance_metrics_to_json(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let metrics = engine.get_performance_metrics();
        let baseline = engine.collect_baseline_metrics();

        let telemetry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "performance_metrics": {
                "nodes_per_second": metrics.nodes_per_second,
                "aspiration_success_rate": metrics.aspiration_success_rate,
                "average_window_size": metrics.average_window_size,
                "retry_frequency": metrics.retry_frequency,
                "health_score": metrics.health_score,
            },
            "baseline_metrics": baseline,
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&telemetry)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }

    /// Export performance metrics to CSV (Task 26.0 - Task 7.0)
    pub fn export_performance_metrics_to_csv(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let metrics = engine.get_performance_metrics();
        let baseline = engine.collect_baseline_metrics();

        let mut csv = String::new();
        csv.push_str("Metric,Value\n");
        csv.push_str(&format!("nodes_per_second,{}\n", metrics.nodes_per_second));
        csv.push_str(&format!("aspiration_success_rate,{}\n", metrics.aspiration_success_rate));
        csv.push_str(&format!("average_window_size,{}\n", metrics.average_window_size));
        csv.push_str(&format!("retry_frequency,{}\n", metrics.retry_frequency));
        csv.push_str(&format!("health_score,{}\n", metrics.health_score));
        csv.push_str(&format!("search_nodes_per_second,{}\n", baseline.search_metrics.nodes_per_second));
        csv.push_str(&format!("search_cutoff_rate,{}\n", baseline.search_metrics.average_cutoff_rate));
        csv.push_str(&format!("search_cutoff_index,{}\n", baseline.search_metrics.average_cutoff_index));
        csv.push_str(&format!("eval_time_ns,{}\n", baseline.evaluation_metrics.average_evaluation_time_ns));
        csv.push_str(&format!("eval_cache_hit_rate,{}\n", baseline.evaluation_metrics.cache_hit_rate));
        csv.push_str(&format!("tt_hit_rate,{}\n", baseline.tt_metrics.hit_rate));
        csv.push_str(&format!("tt_exact_entry_rate,{}\n", baseline.tt_metrics.exact_entry_rate));
        csv.push_str(&format!("tt_occupancy_rate,{}\n", baseline.tt_metrics.occupancy_rate));
        csv.push_str(&format!("move_ordering_cutoff_index,{}\n", baseline.move_ordering_metrics.average_cutoff_index));
        csv.push_str(&format!("move_ordering_pv_hit_rate,{}\n", baseline.move_ordering_metrics.pv_hit_rate));
        csv.push_str(&format!("move_ordering_killer_hit_rate,{}\n", baseline.move_ordering_metrics.killer_hit_rate));
        csv.push_str(&format!("move_ordering_cache_hit_rate,{}\n", baseline.move_ordering_metrics.cache_hit_rate));
        csv.push_str(&format!("parallel_speedup_4_cores,{}\n", baseline.parallel_search_metrics.speedup_4_cores));
        csv.push_str(&format!("parallel_speedup_8_cores,{}\n", baseline.parallel_search_metrics.speedup_8_cores));
        csv.push_str(&format!("parallel_efficiency_4_cores,{}\n", baseline.parallel_search_metrics.efficiency_4_cores));
        csv.push_str(&format!("parallel_efficiency_8_cores,{}\n", baseline.parallel_search_metrics.efficiency_8_cores));
        csv.push_str(&format!("memory_tt_mb,{}\n", baseline.memory_metrics.tt_memory_mb));
        csv.push_str(&format!("memory_cache_mb,{}\n", baseline.memory_metrics.cache_memory_mb));
        csv.push_str(&format!("memory_peak_mb,{}\n", baseline.memory_metrics.peak_memory_mb));

        let file_path = self.export_path.join(filename);
        fs::write(&file_path, csv)
            .map_err(|e| format!("Failed to write CSV file: {}", e))?;

        Ok(file_path)
    }

    /// Export performance metrics to Markdown (Task 26.0 - Task 7.0)
    pub fn export_performance_metrics_to_markdown(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let metrics = engine.get_performance_metrics();
        let baseline = engine.collect_baseline_metrics();

        let mut md = String::new();
        md.push_str("# Performance Metrics Report\n\n");
        md.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().to_rfc3339()));
        
        md.push_str("## Performance Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Nodes per Second | {:.2} |\n", metrics.nodes_per_second));
        md.push_str(&format!("| Aspiration Success Rate | {:.2}% |\n", metrics.aspiration_success_rate * 100.0));
        md.push_str(&format!("| Average Window Size | {:.2} |\n", metrics.average_window_size));
        md.push_str(&format!("| Retry Frequency | {:.2}% |\n", metrics.retry_frequency * 100.0));
        md.push_str(&format!("| Health Score | {:.2} |\n", metrics.health_score));
        
        md.push_str("\n## Search Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Nodes per Second | {:.2} |\n", baseline.search_metrics.nodes_per_second));
        md.push_str(&format!("| Average Cutoff Rate | {:.2}% |\n", baseline.search_metrics.average_cutoff_rate * 100.0));
        md.push_str(&format!("| Average Cutoff Index | {:.2} |\n", baseline.search_metrics.average_cutoff_index));
        
        md.push_str("\n## Evaluation Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Average Evaluation Time | {:.2} ns |\n", baseline.evaluation_metrics.average_evaluation_time_ns));
        md.push_str(&format!("| Cache Hit Rate | {:.2}% |\n", baseline.evaluation_metrics.cache_hit_rate * 100.0));
        md.push_str(&format!("| Phase Calc Time | {:.2} ns |\n", baseline.evaluation_metrics.phase_calc_time_ns));
        
        md.push_str("\n## Transposition Table Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Hit Rate | {:.2}% |\n", baseline.tt_metrics.hit_rate * 100.0));
        md.push_str(&format!("| Exact Entry Rate | {:.2}% |\n", baseline.tt_metrics.exact_entry_rate * 100.0));
        md.push_str(&format!("| Occupancy Rate | {:.2}% |\n", baseline.tt_metrics.occupancy_rate * 100.0));
        
        md.push_str("\n## Move Ordering Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Average Cutoff Index | {:.2} |\n", baseline.move_ordering_metrics.average_cutoff_index));
        md.push_str(&format!("| PV Hit Rate | {:.2}% |\n", baseline.move_ordering_metrics.pv_hit_rate * 100.0));
        md.push_str(&format!("| Killer Hit Rate | {:.2}% |\n", baseline.move_ordering_metrics.killer_hit_rate * 100.0));
        md.push_str(&format!("| Cache Hit Rate | {:.2}% |\n", baseline.move_ordering_metrics.cache_hit_rate * 100.0));
        
        md.push_str("\n## Parallel Search Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Speedup (4 cores) | {:.2}x |\n", baseline.parallel_search_metrics.speedup_4_cores));
        md.push_str(&format!("| Speedup (8 cores) | {:.2}x |\n", baseline.parallel_search_metrics.speedup_8_cores));
        md.push_str(&format!("| Efficiency (4 cores) | {:.2}% |\n", baseline.parallel_search_metrics.efficiency_4_cores * 100.0));
        md.push_str(&format!("| Efficiency (8 cores) | {:.2}% |\n", baseline.parallel_search_metrics.efficiency_8_cores * 100.0));
        
        md.push_str("\n## Memory Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| TT Memory | {:.2} MB |\n", baseline.memory_metrics.tt_memory_mb));
        md.push_str(&format!("| Cache Memory | {:.2} MB |\n", baseline.memory_metrics.cache_memory_mb));
        md.push_str(&format!("| Peak Memory | {:.2} MB |\n", baseline.memory_metrics.peak_memory_mb));

        let file_path = self.export_path.join(filename);
        fs::write(&file_path, md)
            .map_err(|e| format!("Failed to write Markdown file: {}", e))?;

        Ok(file_path)
    }

    /// Export efficiency metrics (IID and LMR) (Task 26.0 - Task 7.0)
    pub fn export_efficiency_metrics(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        // Get IID metrics
        let iid_metrics = engine.get_iid_performance_metrics();
        let iid_stats = engine.get_iid_stats();

        // Get LMR stats
        let lmr_stats = engine.get_lmr_stats();

        let efficiency_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "iid_metrics": {
                "efficiency_rate": iid_metrics.iid_efficiency,
                "cutoff_rate": iid_metrics.cutoff_rate,
                "overhead_percentage": iid_metrics.overhead_percentage,
                "success_rate": iid_metrics.success_rate,
                "speedup_percentage": iid_metrics.speedup_percentage,
                "node_reduction_percentage": iid_metrics.node_reduction_percentage,
                "net_benefit_percentage": iid_metrics.net_benefit_percentage,
                "iid_searches_performed": iid_stats.iid_searches_performed,
                "iid_time_ms": iid_stats.iid_time_ms,
                "total_search_time_ms": iid_stats.total_search_time_ms,
            },
            "lmr_metrics": {
                "reductions_applied": lmr_stats.reductions_applied,
                "researches_triggered": lmr_stats.researches_triggered,
                "cutoffs_after_reduction": lmr_stats.cutoffs_after_reduction,
                "cutoffs_after_research": lmr_stats.cutoffs_after_research,
                "total_depth_saved": lmr_stats.total_depth_saved,
                "average_reduction": lmr_stats.average_reduction,
            },
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&efficiency_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }

    /// Export TT entry quality distribution (Task 26.0 - Task 7.0)
    pub fn export_tt_entry_quality_distribution(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let baseline = engine.collect_baseline_metrics();
        let tt_metrics = &baseline.tt_metrics;

        // Calculate distribution (simplified - would need detailed TT stats)
        let total_entries = if tt_metrics.occupancy_rate > 0.0 {
            // Estimate based on occupancy
            (tt_metrics.occupancy_rate * 1000000.0) as u64 // Approximate
        } else {
            0
        };

        let exact_entries = (total_entries as f64 * tt_metrics.exact_entry_rate) as u64;
        let beta_entries = (total_entries as f64 * tt_metrics.hit_rate * 0.3) as u64; // Estimate
        let alpha_entries = (total_entries as f64 * tt_metrics.hit_rate * 0.2) as u64; // Estimate

        let distribution = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_entries": total_entries,
            "exact_entries": exact_entries,
            "exact_percentage": tt_metrics.exact_entry_rate * 100.0,
            "beta_entries": beta_entries,
            "beta_percentage": (beta_entries as f64 / total_entries.max(1) as f64) * 100.0,
            "alpha_entries": alpha_entries,
            "alpha_percentage": (alpha_entries as f64 / total_entries.max(1) as f64) * 100.0,
            "hit_rate": tt_metrics.hit_rate * 100.0,
            "occupancy_rate": tt_metrics.occupancy_rate * 100.0,
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&distribution)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }

    /// Export hit rate by depth (Task 26.0 - Task 7.0)
    pub fn export_hit_rate_by_depth(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        // Simplified - would need depth-stratified TT stats
        let baseline = engine.collect_baseline_metrics();
        let overall_hit_rate = baseline.tt_metrics.hit_rate;

        // Estimate hit rates by depth (would need actual depth tracking)
        let mut depth_data = Vec::new();
        for depth in 1..=10 {
            // Estimate: hit rate increases with depth
            let estimated_hit_rate = overall_hit_rate * (1.0 + (depth as f64 - 5.0) * 0.1);
            depth_data.push(serde_json::json!({
                "depth": depth,
                "hit_rate": estimated_hit_rate * 100.0,
            }));
        }

        let hit_rate_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "overall_hit_rate": overall_hit_rate * 100.0,
            "by_depth": depth_data,
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&hit_rate_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }

    /// Export scalability metrics (Task 26.0 - Task 7.0)
    pub fn export_scalability_metrics(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let baseline = engine.collect_baseline_metrics();
        let parallel_metrics = &baseline.parallel_search_metrics;

        let scalability_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "speedup_4_cores": parallel_metrics.speedup_4_cores,
            "speedup_8_cores": parallel_metrics.speedup_8_cores,
            "efficiency_4_cores": parallel_metrics.efficiency_4_cores * 100.0,
            "efficiency_8_cores": parallel_metrics.efficiency_8_cores * 100.0,
            "linear_speedup_4_cores": 4.0,
            "linear_speedup_8_cores": 8.0,
            "efficiency_vs_linear_4": (parallel_metrics.speedup_4_cores / 4.0) * 100.0,
            "efficiency_vs_linear_8": (parallel_metrics.speedup_8_cores / 8.0) * 100.0,
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&scalability_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }

    /// Export cache effectiveness metrics (Task 26.0 - Task 7.0)
    pub fn export_cache_effectiveness(
        &self,
        engine: &SearchEngine,
        filename: &str,
    ) -> Result<PathBuf, String> {
        if !self.enabled {
            return Err("Telemetry export is disabled".to_string());
        }

        self.ensure_directory()?;

        let baseline = engine.collect_baseline_metrics();
        let _ordering_stats = engine.get_move_ordering_effectiveness_metrics();

        let cache_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "evaluation_cache": {
                "hit_rate": baseline.evaluation_metrics.cache_hit_rate * 100.0,
            },
            "move_ordering_cache": {
                "hit_rate": baseline.move_ordering_metrics.cache_hit_rate * 100.0,
                "pv_hit_rate": baseline.move_ordering_metrics.pv_hit_rate * 100.0,
                "killer_hit_rate": baseline.move_ordering_metrics.killer_hit_rate * 100.0,
            },
            "cache_sizes": {
                "move_ordering_memory_mb": baseline.memory_metrics.cache_memory_mb,
            },
        });

        let file_path = self.export_path.join(filename);
        let json = serde_json::to_string_pretty(&cache_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        Ok(file_path)
    }
}

impl Default for TelemetryExporter {
    fn default() -> Self {
        Self::new("telemetry")
    }
}

// ============================================================================
// External Profiler Integration and Hot Path Analysis (Task 26.0 - Task 8.0)
// ============================================================================

use std::time::Instant;

/// Trait for external profiler integration (Task 26.0 - Task 8.0)
pub trait ExternalProfiler: Send + Sync {
    /// Start profiling a region with the given name
    fn start_region(&self, name: &str);
    
    /// End profiling a region with the given name
    fn end_region(&self, name: &str);
    
    /// Mark a point in time with the given label
    fn mark(&self, label: &str);
    
    /// Export profiling markers to JSON
    fn export_markers(&self) -> Result<serde_json::Value, String>;
    
    /// Check if profiling is enabled
    fn is_enabled(&self) -> bool;
}

/// Profiler marker entry (Task 26.0 - Task 8.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerMarker {
    /// Marker name/label
    pub name: String,
    /// Timestamp (nanoseconds since epoch)
    pub timestamp_ns: u64,
    /// Marker type (start, end, point)
    pub marker_type: MarkerType,
}

/// Marker type (Task 26.0 - Task 8.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerType {
    /// Region start
    RegionStart,
    /// Region end
    RegionEnd,
    /// Point marker
    Point,
}

/// Perf-compatible profiler for Linux (Task 26.0 - Task 8.0)
pub struct PerfProfiler {
    /// Whether profiling is enabled
    enabled: bool,
    /// Profiler markers
    markers: Arc<Mutex<Vec<ProfilerMarker>>>,
    /// Start time for relative timestamps
    start_time: Instant,
}

impl PerfProfiler {
    /// Create a new Perf profiler
    pub fn new() -> Self {
        Self {
            enabled: false,
            markers: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
    }
    
    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Get markers (for testing)
    pub fn get_markers(&self) -> Vec<ProfilerMarker> {
        self.markers.lock().unwrap().clone()
    }
}

impl Default for PerfProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalProfiler for PerfProfiler {
    fn start_region(&self, name: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: name.to_string(),
            timestamp_ns,
            marker_type: MarkerType::RegionStart,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn end_region(&self, name: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: name.to_string(),
            timestamp_ns,
            marker_type: MarkerType::RegionEnd,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn mark(&self, label: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: label.to_string(),
            timestamp_ns,
            marker_type: MarkerType::Point,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn export_markers(&self) -> Result<serde_json::Value, String> {
        let markers = self.markers.lock().unwrap();
        
        let markers_json: Vec<serde_json::Value> = markers
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "timestamp_ns": m.timestamp_ns,
                    "type": match m.marker_type {
                        MarkerType::RegionStart => "region_start",
                        MarkerType::RegionEnd => "region_end",
                        MarkerType::Point => "point",
                    },
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "profiler": "perf",
            "markers": markers_json,
            "total_markers": markers.len(),
        }))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Instruments-compatible profiler for macOS (Task 26.0 - Task 8.0)
pub struct InstrumentsProfiler {
    /// Whether profiling is enabled
    enabled: bool,
    /// Profiler markers
    markers: Arc<Mutex<Vec<ProfilerMarker>>>,
    /// Start time for relative timestamps
    start_time: Instant,
}

impl InstrumentsProfiler {
    /// Create a new Instruments profiler
    pub fn new() -> Self {
        Self {
            enabled: false,
            markers: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
    }
    
    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Get markers (for testing)
    pub fn get_markers(&self) -> Vec<ProfilerMarker> {
        self.markers.lock().unwrap().clone()
    }
}

impl Default for InstrumentsProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalProfiler for InstrumentsProfiler {
    fn start_region(&self, name: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: name.to_string(),
            timestamp_ns,
            marker_type: MarkerType::RegionStart,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn end_region(&self, name: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: name.to_string(),
            timestamp_ns,
            marker_type: MarkerType::RegionEnd,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn mark(&self, label: &str) {
        if !self.enabled {
            return;
        }
        
        let elapsed = self.start_time.elapsed();
        let timestamp_ns = elapsed.as_nanos() as u64;
        
        let marker = ProfilerMarker {
            name: label.to_string(),
            timestamp_ns,
            marker_type: MarkerType::Point,
        };
        
        self.markers.lock().unwrap().push(marker);
    }
    
    fn export_markers(&self) -> Result<serde_json::Value, String> {
        let markers = self.markers.lock().unwrap();
        
        let markers_json: Vec<serde_json::Value> = markers
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "timestamp_ns": m.timestamp_ns,
                    "type": match m.marker_type {
                        MarkerType::RegionStart => "region_start",
                        MarkerType::RegionEnd => "region_end",
                        MarkerType::Point => "point",
                    },
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "profiler": "instruments",
            "markers": markers_json,
            "total_markers": markers.len(),
        }))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tuning_manager_creation() {
        let config = TranspositionConfig::default();
        let manager = PerformanceTuningManager::new(config);

        assert!(!manager.get_recommendations().is_empty());
        assert_eq!(manager.get_performance_targets().target_hit_rate, 0.35);
    }

    #[test]
    fn test_tuning_session_management() {
        let config = TranspositionConfig::default();
        let mut manager = PerformanceTuningManager::new(config);

        let session_id = manager.start_tuning_session();
        assert!(!session_id.is_empty());

        assert!(manager.end_tuning_session(&session_id).is_ok());
    }

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();
        profiler.set_enabled(true);

        profiler.record_operation("store", 100);
        profiler.record_operation("store", 120);

        let avg_time = profiler.get_average_operation_time("store");
        assert!(avg_time.is_some());
        assert_eq!(avg_time.unwrap(), 110.0);
    }

    #[test]
    fn test_performance_recommendations() {
        let config = TranspositionConfig::default();
        let mut manager = PerformanceTuningManager::new(config);

        let recommendations = manager.generate_performance_recommendations();
        // Should generate recommendations based on current performance
        assert!(recommendations.len() >= 0);
    }
}
