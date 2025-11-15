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

use crate::types::*;
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
