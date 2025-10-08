//! Move Ordering System
//! 
//! This module provides the core move ordering functionality for the Shogi engine.
//! It implements various heuristics to prioritize moves for better alpha-beta pruning.
//! 
//! # Features
//! 
//! - **Basic Move Ordering Structure**: Core framework for move ordering
//! - **Statistics Tracking**: Performance metrics and hit rates
//! - **Memory Usage Tracking**: Monitor memory consumption
//! - **Configuration System**: Flexible weights and settings
//! - **Move Scoring Infrastructure**: Foundation for various heuristics
//! 
//! # Usage
//! 
//! ```rust
//! use shogi_engine::search::move_ordering::{MoveOrdering, OrderingStats, OrderingWeights};
//! use shogi_engine::types::{Move, Player};
//! use shogi_engine::bitboards::BitboardBoard;
//! 
//! // Create move orderer with default configuration
//! let mut orderer = MoveOrdering::new();
//! 
//! // Order moves for a position
//! let moves = vec![/* your moves */];
//! let ordered_moves = orderer.order_moves(&moves);
//! 
//! // Get performance statistics
//! let stats = orderer.get_stats();
//! println!("Total moves ordered: {}", stats.total_moves_ordered);
//! ```

use crate::types::*;
use crate::time_utils::TimeSource;
use std::collections::HashMap;
use std::ptr;
use std::fmt;

// ==================== Error Handling Types ====================

/// Result type for move ordering operations
pub type MoveOrderingResult<T> = Result<T, MoveOrderingError>;

/// Errors that can occur during move ordering operations
#[derive(Debug, Clone, PartialEq)]
pub enum MoveOrderingError {
    /// Invalid move provided
    InvalidMove(String),
    /// Configuration error
    ConfigurationError(String),
    /// Memory allocation error
    MemoryError(String),
    /// Statistics error
    StatisticsError(String),
    /// Cache operation failed
    CacheError(String),
    /// SEE calculation failed
    SEEError(String),
    /// Hash calculation error
    HashError(String),
    /// General operation error
    OperationError(String),
}

impl fmt::Display for MoveOrderingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveOrderingError::InvalidMove(msg) => write!(f, "Invalid move: {}", msg),
            MoveOrderingError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            MoveOrderingError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            MoveOrderingError::StatisticsError(msg) => write!(f, "Statistics error: {}", msg),
            MoveOrderingError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            MoveOrderingError::SEEError(msg) => write!(f, "SEE calculation error: {}", msg),
            MoveOrderingError::HashError(msg) => write!(f, "Hash error: {}", msg),
            MoveOrderingError::OperationError(msg) => write!(f, "Operation error: {}", msg),
        }
    }
}

/// Result of a move in transposition table context
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveResult {
    /// Move caused a beta cutoff
    Cutoff,
    /// Move has an exact score
    Exact,
    /// Move has a bound (upper or lower)
    Bound,
}

/// Statistics for transposition table integration
#[derive(Debug, Clone, PartialEq)]
pub struct TTIntegrationStats {
    /// Number of TT integration hits
    pub tt_integration_hits: u64,
    /// Number of TT integration updates
    pub tt_integration_updates: u64,
    /// Number of cutoff updates from TT
    pub tt_cutoff_updates: u64,
    /// Number of exact updates from TT
    pub tt_exact_updates: u64,
    /// Number of bound updates from TT
    pub tt_bound_updates: u64,
    /// Number of killer moves from TT
    pub killer_moves_from_tt: u64,
    /// Number of PV moves from TT
    pub pv_moves_from_tt: u64,
    /// Number of history updates from TT
    pub history_updates_from_tt: u64,
    /// Number of cutoff history updates
    pub cutoff_history_updates: u64,
}

// ==================== Performance Tuning Types ====================

/// Result of runtime performance tuning
#[derive(Debug, Clone)]
pub struct PerformanceTuningResult {
    /// Number of adjustments made
    pub adjustments_made: usize,
    /// List of adjustments applied
    pub adjustments: Vec<String>,
    /// Cache hit rate before tuning
    pub cache_hit_rate_before: f64,
    /// Average ordering time before tuning
    pub avg_ordering_time_before: f64,
}

/// Performance monitoring report
#[derive(Debug, Clone)]
pub struct PerformanceMonitoringReport {
    /// Overall health score (0-100)
    pub overall_health_score: f64,
    /// Current cache hit rate
    pub cache_hit_rate: f64,
    /// Average ordering time in microseconds
    pub avg_ordering_time_us: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// PV move hit rate
    pub pv_hit_rate: f64,
    /// Killer move hit rate
    pub killer_hit_rate: f64,
    /// History heuristic hit rate
    pub history_hit_rate: f64,
    /// Performance warnings
    pub warnings: Vec<String>,
    /// Tuning recommendations
    pub recommendations: Vec<String>,
}

/// Automatic optimization result
#[derive(Debug, Clone)]
pub struct AutoOptimizationResult {
    /// Number of optimizations applied
    pub optimizations_applied: usize,
    /// List of optimizations
    pub optimizations: Vec<String>,
    /// Performance snapshot before optimization
    pub performance_before: PerformanceSnapshot,
    /// Performance snapshot after optimization
    pub performance_after: PerformanceSnapshot,
}

/// Performance snapshot for comparison
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Cache hit rate at snapshot time
    pub cache_hit_rate: f64,
    /// Average ordering time at snapshot time
    pub avg_ordering_time_us: f64,
    /// Memory usage at snapshot time
    pub memory_usage_bytes: usize,
}

/// Performance comparison between two snapshots
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Change in cache hit rate
    pub cache_hit_rate_change: f64,
    /// Change in ordering time (negative is better)
    pub ordering_time_change: f64,
    /// Change in memory usage (negative is better)
    pub memory_usage_change: i64,
    /// Whether performance improved overall
    pub is_improved: bool,
}

/// Tuning recommendation
#[derive(Debug, Clone)]
pub struct TuningRecommendation {
    /// Category of the recommendation
    pub category: TuningCategory,
    /// Priority level
    pub priority: TuningPriority,
    /// Description of the recommendation
    pub description: String,
    /// Expected impact of applying the recommendation
    pub expected_impact: String,
}

/// Tuning category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuningCategory {
    /// Cache size tuning
    CacheSize,
    /// Weight adjustment
    Weights,
    /// Performance optimization
    Performance,
    /// Memory optimization
    Memory,
    /// Heuristic configuration
    Heuristics,
}

/// Tuning priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TuningPriority {
    /// Low priority - optional optimization
    Low,
    /// Medium priority - recommended optimization
    Medium,
    /// High priority - important optimization
    High,
    /// Critical - should be applied immediately
    Critical,
}

/// Platform-specific memory limits
#[derive(Debug, Clone)]
pub struct PlatformMemoryLimits {
    /// Maximum total memory usage in bytes
    pub max_total_memory_bytes: usize,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Maximum SEE cache size
    pub max_see_cache_size: usize,
    /// Recommended cache size for this platform
    pub recommended_cache_size: usize,
    /// Recommended SEE cache size for this platform
    pub recommended_see_cache_size: usize,
}

/// Configuration for parallel search
#[derive(Debug, Clone)]
pub struct ParallelSearchConfig {
    /// Base configuration to use for each thread
    pub config: MoveOrderingConfig,
    /// Whether to use thread-safe caches
    pub thread_safe_caches: bool,
    /// Whether to share history table across threads
    pub shared_history: bool,
    /// Whether to share PV moves across threads
    pub shared_pv: bool,
    /// Whether to share killer moves across threads
    pub shared_killers: bool,
}

/// Statistics for advanced integrations
#[derive(Debug, Clone, Default)]
pub struct AdvancedIntegrationStats {
    /// Number of opening book integrations
    pub opening_book_integrations: u64,
    /// Number of tablebase integrations
    pub tablebase_integrations: u64,
    /// Number of analysis mode orderings
    pub analysis_orderings: u64,
    /// Number of phase-specific orderings
    pub phase_specific_orderings: u64,
}


/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue
    Low,
    /// Medium severity - operation should be retried
    Medium,
    /// High severity - operation should be aborted
    High,
    /// Critical severity - system should be reset
    Critical,
}

/// Error logging entry
#[derive(Debug, Clone)]
pub struct ErrorLogEntry {
    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
    /// Error details
    pub error: MoveOrderingError,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Context information
    pub context: String,
    /// Stack trace or additional details
    pub details: Option<String>,
}

/// Error handler for move ordering operations
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    /// Error log entries
    pub error_log: Vec<ErrorLogEntry>,
    /// Maximum number of errors to keep in log
    pub max_log_size: usize,
    /// Error reporting enabled
    pub reporting_enabled: bool,
    /// Graceful degradation enabled
    pub graceful_degradation_enabled: bool,
    /// Error recovery enabled
    pub recovery_enabled: bool,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self {
            error_log: Vec::new(),
            max_log_size: 1000,
            reporting_enabled: true,
            graceful_degradation_enabled: true,
            recovery_enabled: true,
        }
    }
}

impl ErrorHandler {
    /// Log an error entry
    pub fn log_error(&mut self, error: MoveOrderingError, severity: ErrorSeverity, context: String) {
        let entry = ErrorLogEntry {
            timestamp: std::time::SystemTime::now(),
            error,
            severity,
            context,
            details: None,
        };

        self.error_log.push(entry);

        // Trim log if it exceeds maximum size
        if self.error_log.len() > self.max_log_size {
            self.error_log.remove(0);
        }
    }

    /// Get recent errors
    pub fn get_recent_errors(&self, count: usize) -> Vec<&ErrorLogEntry> {
        let start = if self.error_log.len() > count {
            self.error_log.len() - count
        } else {
            0
        };
        self.error_log.iter().skip(start).collect()
    }

    /// Clear error log
    pub fn clear_errors(&mut self) {
        self.error_log.clear();
    }

    /// Check if errors indicate system instability
    pub fn is_system_unstable(&self) -> bool {
        let recent_errors = self.get_recent_errors(10);
        let critical_count = recent_errors.iter()
            .filter(|e| e.severity == ErrorSeverity::Critical)
            .count();
        let high_count = recent_errors.iter()
            .filter(|e| e.severity == ErrorSeverity::High)
            .count();

        critical_count > 0 || high_count >= 3
    }
}

// ==================== Memory Management Types ====================

/// Memory pool for efficient allocation of frequently used objects
#[derive(Debug, Clone)]
pub struct MemoryPool {
    /// Pool of Vec<Move> for move lists
    move_vec_pool: Vec<Vec<Move>>,
    /// Pool of Vec<(i32, usize)> for move scores
    move_score_vec_pool: Vec<Vec<(i32, usize)>>,
    /// Pool of Vec<u64> for hash vectors
    hash_vec_pool: Vec<Vec<u64>>,
    /// Pool of Vec<i32> for integer vectors
    int_vec_pool: Vec<Vec<i32>>,
    /// Maximum pool size per type
    max_pool_size: usize,
    /// Current pool sizes
    pool_sizes: MemoryPoolSizes,
}

/// Memory pool size tracking
#[derive(Debug, Clone, Default)]
pub struct MemoryPoolSizes {
    /// Number of move vector pools
    pub move_vec_count: usize,
    /// Number of move score vector pools
    pub move_score_vec_count: usize,
    /// Number of hash vector pools
    pub hash_vec_count: usize,
    /// Number of integer vector pools
    pub int_vec_count: usize,
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self {
            move_vec_pool: Vec::with_capacity(8),
            move_score_vec_pool: Vec::with_capacity(8),
            hash_vec_pool: Vec::with_capacity(8),
            int_vec_pool: Vec::with_capacity(8),
            max_pool_size: 16,
            pool_sizes: MemoryPoolSizes::default(),
        }
    }
}

impl MemoryPool {
    /// Get a move vector from the pool or create a new one
    pub fn get_move_vec(&mut self) -> Vec<Move> {
        if let Some(mut vec) = self.move_vec_pool.pop() {
            vec.clear();
            self.pool_sizes.move_vec_count -= 1;
            vec
        } else {
            Vec::with_capacity(64) // Pre-allocate reasonable capacity
        }
    }

    /// Return a move vector to the pool
    pub fn return_move_vec(&mut self, mut vec: Vec<Move>) {
        if self.pool_sizes.move_vec_count < self.max_pool_size {
            vec.clear();
            self.move_vec_pool.push(vec);
            self.pool_sizes.move_vec_count += 1;
        }
        // If pool is full, drop the vector (it will be deallocated)
    }

    /// Get a move score vector from the pool or create a new one
    pub fn get_move_score_vec(&mut self) -> Vec<(i32, usize)> {
        if let Some(mut vec) = self.move_score_vec_pool.pop() {
            vec.clear();
            self.pool_sizes.move_score_vec_count -= 1;
            vec
        } else {
            Vec::with_capacity(64)
        }
    }

    /// Return a move score vector to the pool
    pub fn return_move_score_vec(&mut self, mut vec: Vec<(i32, usize)>) {
        if self.pool_sizes.move_score_vec_count < self.max_pool_size {
            vec.clear();
            self.move_score_vec_pool.push(vec);
            self.pool_sizes.move_score_vec_count += 1;
        }
    }

    /// Get a hash vector from the pool or create a new one
    pub fn get_hash_vec(&mut self) -> Vec<u64> {
        if let Some(mut vec) = self.hash_vec_pool.pop() {
            vec.clear();
            self.pool_sizes.hash_vec_count -= 1;
            vec
        } else {
            Vec::with_capacity(64)
        }
    }

    /// Return a hash vector to the pool
    pub fn return_hash_vec(&mut self, mut vec: Vec<u64>) {
        if self.pool_sizes.hash_vec_count < self.max_pool_size {
            vec.clear();
            self.hash_vec_pool.push(vec);
            self.pool_sizes.hash_vec_count += 1;
        }
    }

    /// Get an integer vector from the pool or create a new one
    pub fn get_int_vec(&mut self) -> Vec<i32> {
        if let Some(mut vec) = self.int_vec_pool.pop() {
            vec.clear();
            self.pool_sizes.int_vec_count -= 1;
            vec
        } else {
            Vec::with_capacity(64)
        }
    }

    /// Return an integer vector to the pool
    pub fn return_int_vec(&mut self, mut vec: Vec<i32>) {
        if self.pool_sizes.int_vec_count < self.max_pool_size {
            vec.clear();
            self.int_vec_pool.push(vec);
            self.pool_sizes.int_vec_count += 1;
        }
    }

    /// Clear all pools and free memory
    pub fn clear_all_pools(&mut self) {
        self.move_vec_pool.clear();
        self.move_score_vec_pool.clear();
        self.hash_vec_pool.clear();
        self.int_vec_pool.clear();
        self.pool_sizes = MemoryPoolSizes::default();
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self) -> MemoryPoolSizes {
        self.pool_sizes.clone()
    }
}

/// Memory usage tracker for monitoring and leak detection
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    /// Current memory usage by component
    current_usage: MemoryUsageBreakdown,
    /// Peak memory usage by component
    peak_usage: MemoryUsageBreakdown,
    /// Memory allocation history
    allocation_history: Vec<AllocationEvent>,
    /// Maximum history size
    max_history_size: usize,
    /// Memory leak detection enabled
    leak_detection_enabled: bool,
    /// Memory usage thresholds
    thresholds: MemoryThresholds,
}

/// Memory usage breakdown by component
#[derive(Debug, Clone, Default)]
pub struct MemoryUsageBreakdown {
    /// Move ordering struct memory
    pub struct_memory: usize,
    /// Cache memory usage
    pub cache_memory: usize,
    /// Pool memory usage
    pub pool_memory: usize,
    /// Statistics memory usage
    pub statistics_memory: usize,
    /// Error handler memory usage
    pub error_handler_memory: usize,
    /// Total memory usage
    pub total_memory: usize,
}

/// Memory allocation event for tracking
#[derive(Debug, Clone)]
pub struct AllocationEvent {
    /// Timestamp of allocation
    pub timestamp: std::time::SystemTime,
    /// Type of allocation
    pub allocation_type: AllocationType,
    /// Size of allocation
    pub size: usize,
    /// Component that performed allocation
    pub component: String,
    /// Whether this was a deallocation
    pub is_deallocation: bool,
}

/// Types of memory allocations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AllocationType {
    /// Move vector allocation
    MoveVector,
    /// Move score vector allocation
    MoveScoreVector,
    /// Hash vector allocation
    HashVector,
    /// Integer vector allocation
    IntegerVector,
    /// Cache allocation
    Cache,
    /// Statistics allocation
    Statistics,
    /// Error handler allocation
    ErrorHandler,
}

/// Memory usage thresholds for monitoring
#[derive(Debug, Clone)]
pub struct MemoryThresholds {
    /// Warning threshold (bytes)
    pub warning_threshold: usize,
    /// Critical threshold (bytes)
    pub critical_threshold: usize,
    /// Maximum allowed memory (bytes)
    pub max_memory: usize,
}

impl Default for MemoryThresholds {
    fn default() -> Self {
        Self {
            warning_threshold: 10 * 1024 * 1024,  // 10 MB
            critical_threshold: 50 * 1024 * 1024,  // 50 MB
            max_memory: 100 * 1024 * 1024,         // 100 MB
        }
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self {
            current_usage: MemoryUsageBreakdown::default(),
            peak_usage: MemoryUsageBreakdown::default(),
            allocation_history: Vec::with_capacity(1000),
            max_history_size: 1000,
            leak_detection_enabled: true,
            thresholds: MemoryThresholds::default(),
        }
    }
}

impl MemoryTracker {
    /// Record a memory allocation event
    pub fn record_allocation(&mut self, allocation_type: AllocationType, size: usize, component: String) {
        let event = AllocationEvent {
            timestamp: std::time::SystemTime::now(),
            allocation_type,
            size,
            component,
            is_deallocation: false,
        };

        self.allocation_history.push(event);

        // Trim history if it exceeds maximum size
        if self.allocation_history.len() > self.max_history_size {
            self.allocation_history.remove(0);
        }

        // Update current usage
        self.update_current_usage();
    }

    /// Record a memory deallocation event
    pub fn record_deallocation(&mut self, allocation_type: AllocationType, size: usize, component: String) {
        let event = AllocationEvent {
            timestamp: std::time::SystemTime::now(),
            allocation_type,
            size,
            component,
            is_deallocation: true,
        };

        self.allocation_history.push(event);

        // Trim history if it exceeds maximum size
        if self.allocation_history.len() > self.max_history_size {
            self.allocation_history.remove(0);
        }

        // Update current usage
        self.update_current_usage();
    }

    /// Update current memory usage based on allocation history
    fn update_current_usage(&mut self) {
        let mut usage = MemoryUsageBreakdown::default();

        for event in &self.allocation_history {
            if event.is_deallocation {
                continue;
            }

            match event.allocation_type {
                AllocationType::MoveVector => usage.struct_memory += event.size,
                AllocationType::MoveScoreVector => usage.struct_memory += event.size,
                AllocationType::HashVector => usage.struct_memory += event.size,
                AllocationType::IntegerVector => usage.struct_memory += event.size,
                AllocationType::Cache => usage.cache_memory += event.size,
                AllocationType::Statistics => usage.statistics_memory += event.size,
                AllocationType::ErrorHandler => usage.error_handler_memory += event.size,
            }
        }

        usage.total_memory = usage.struct_memory + usage.cache_memory + 
                           usage.pool_memory + usage.statistics_memory + 
                           usage.error_handler_memory;

        // Update peak usage if current usage is higher
        if usage.total_memory > self.peak_usage.total_memory {
            self.peak_usage = usage.clone();
        }

        self.current_usage = usage;
    }

    /// Check for potential memory leaks
    pub fn check_for_leaks(&self) -> Vec<MemoryLeakWarning> {
        if !self.leak_detection_enabled {
            return Vec::new();
        }

        let mut warnings = Vec::new();
        let now = std::time::SystemTime::now();

        // Check for allocations without corresponding deallocations
        let mut allocations = std::collections::HashMap::new();

        for event in &self.allocation_history {
            let key = (event.allocation_type.clone(), event.component.clone());
            
            if event.is_deallocation {
                allocations.remove(&key);
            } else {
                allocations.insert(key, event);
            }
        }

        // Report potential leaks
        for ((allocation_type, component), event) in allocations {
            if let Ok(duration) = now.duration_since(event.timestamp) {
                if duration.as_secs() > 60 { // Consider it a leak if allocated more than 1 minute ago
                    warnings.push(MemoryLeakWarning {
                        allocation_type,
                        component,
                        size: event.size,
                        age_seconds: duration.as_secs(),
                    });
                }
            }
        }

        warnings
    }

    /// Get current memory usage
    pub fn get_current_usage(&self) -> &MemoryUsageBreakdown {
        &self.current_usage
    }

    /// Get peak memory usage
    pub fn get_peak_usage(&self) -> &MemoryUsageBreakdown {
        &self.peak_usage
    }

    /// Check if memory usage exceeds thresholds
    pub fn check_thresholds(&self) -> MemoryThresholdStatus {
        let total = self.current_usage.total_memory;

        if total > self.thresholds.max_memory {
            MemoryThresholdStatus::Exceeded
        } else if total > self.thresholds.critical_threshold {
            MemoryThresholdStatus::Critical
        } else if total > self.thresholds.warning_threshold {
            MemoryThresholdStatus::Warning
        } else {
            MemoryThresholdStatus::Normal
        }
    }

    /// Clear allocation history
    pub fn clear_history(&mut self) {
        self.allocation_history.clear();
    }
}

/// Memory leak warning
#[derive(Debug, Clone)]
pub struct MemoryLeakWarning {
    /// Type of allocation that may be leaked
    pub allocation_type: AllocationType,
    /// Component that performed the allocation
    pub component: String,
    /// Size of the potential leak
    pub size: usize,
    /// Age of the allocation in seconds
    pub age_seconds: u64,
}

/// Memory threshold status
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryThresholdStatus {
    /// Normal memory usage
    Normal,
    /// Warning level memory usage
    Warning,
    /// Critical level memory usage
    Critical,
    /// Maximum memory exceeded
    Exceeded,
}

/// Comprehensive memory leak report
#[derive(Debug, Clone)]
pub struct MemoryLeakReport {
    /// Memory leak warnings
    pub warnings: Vec<MemoryLeakWarning>,
    /// Current memory usage
    pub current_usage: MemoryUsageBreakdown,
    /// Peak memory usage
    pub peak_usage: MemoryUsageBreakdown,
    /// Memory pool statistics
    pub pool_stats: MemoryPoolSizes,
    /// Whether leaks were detected
    pub leak_detected: bool,
    /// Timestamp of the report
    pub timestamp: std::time::SystemTime,
}

/// Memory cleanup report
#[derive(Debug, Clone)]
pub struct MemoryCleanupReport {
    /// Memory usage before cleanup
    pub before_usage: MemoryUsageBreakdown,
    /// Memory usage after cleanup
    pub after_usage: MemoryUsageBreakdown,
    /// Amount of memory freed
    pub memory_freed: usize,
    /// Whether cleanup was successful
    pub cleanup_successful: bool,
    /// Timestamp of the cleanup
    pub timestamp: std::time::SystemTime,
}

/// Memory pressure levels for selective cleanup
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressureLevel {
    /// Low memory pressure
    Low,
    /// Medium memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

// ==================== Advanced Features Types ====================

/// Advanced features manager for move ordering
#[derive(Debug, Clone)]
pub struct AdvancedFeatures {
    /// Position-specific ordering strategies
    position_strategies: PositionSpecificStrategies,
    /// Machine learning model for move ordering
    ml_model: MachineLearningModel,
    /// Dynamic weight adjustment system
    dynamic_weights: DynamicWeightAdjuster,
    /// Multi-threading support
    #[allow(dead_code)] // Kept for future implementation
    threading_support: ThreadingSupport,
    /// Predictive move ordering
    predictive_ordering: PredictiveOrdering,
    /// Advanced cache warming
    cache_warming: AdvancedCacheWarming,
}

/// Position-specific ordering strategies
#[derive(Debug, Clone)]
pub struct PositionSpecificStrategies {
    /// Opening phase strategy
    opening_strategy: OrderingStrategy,
    /// Middlegame phase strategy
    middlegame_strategy: OrderingStrategy,
    /// Endgame phase strategy
    endgame_strategy: OrderingStrategy,
    /// Tactical position strategy
    tactical_strategy: OrderingStrategy,
    /// Positional position strategy
    positional_strategy: OrderingStrategy,
    /// Current game phase
    current_phase: GamePhase,
}

/// Ordering strategy configuration
#[derive(Debug, Clone)]
pub struct OrderingStrategy {
    /// Strategy name
    pub name: String,
    /// Weights for this strategy
    pub weights: OrderingWeights,
    /// Priority adjustments
    pub priority_adjustments: PriorityAdjustments,
    /// Heuristic preferences
    pub heuristic_preferences: HeuristicPreferences,
}

/// Priority adjustments for different move types
#[derive(Debug, Clone, Default)]
pub struct PriorityAdjustments {
    /// Capture move priority multiplier
    pub capture_priority: f64,
    /// Promotion move priority multiplier
    pub promotion_priority: f64,
    /// Development move priority multiplier
    pub development_priority: f64,
    /// Center control priority multiplier
    pub center_priority: f64,
    /// King safety priority multiplier
    pub king_safety_priority: f64,
}

/// Heuristic preferences for different strategies
#[derive(Debug, Clone, Default)]
pub struct HeuristicPreferences {
    /// Prefer tactical heuristics
    pub prefer_tactical: bool,
    /// Prefer positional heuristics
    pub prefer_positional: bool,
    /// Prefer development heuristics
    pub prefer_development: bool,
    /// Prefer endgame heuristics
    pub prefer_endgame: bool,
}

/// Game phase enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    /// Opening phase (first 20 moves)
    Opening,
    /// Middlegame phase (moves 21-60)
    Middlegame,
    /// Endgame phase (after move 60)
    Endgame,
    /// Tactical position (many captures/threats)
    Tactical,
    /// Positional position (quiet maneuvering)
    Positional,
}

/// Machine learning model for move ordering
#[derive(Debug, Clone)]
pub struct MachineLearningModel {
    /// Model type
    pub model_type: MLModelType,
    /// Model parameters
    pub parameters: MLParameters,
    /// Training data
    pub training_data: Vec<TrainingExample>,
    /// Model accuracy
    pub accuracy: f64,
    /// Model enabled
    pub enabled: bool,
}

/// Machine learning model types
#[derive(Debug, Clone, PartialEq)]
pub enum MLModelType {
    /// Linear regression model
    LinearRegression,
    /// Decision tree model
    DecisionTree,
    /// Neural network model
    NeuralNetwork,
    /// Random forest model
    RandomForest,
}

/// Machine learning parameters
#[derive(Debug, Clone)]
pub struct MLParameters {
    /// Learning rate
    pub learning_rate: f64,
    /// Regularization parameter
    pub regularization: f64,
    /// Number of features
    pub num_features: usize,
    /// Model complexity
    pub complexity: f64,
}

/// Training example for machine learning
#[derive(Debug, Clone)]
pub struct TrainingExample {
    /// Input features
    pub features: Vec<f64>,
    /// Expected output (move score)
    pub target: f64,
    /// Position context
    pub context: PositionContext,
}

/// Position context for training
#[derive(Debug, Clone)]
pub struct PositionContext {
    /// Game phase
    pub phase: GamePhase,
    /// Material balance
    pub material_balance: i32,
    /// King safety score
    pub king_safety: i32,
    /// Center control score
    pub center_control: i32,
}

/// Dynamic weight adjustment system
#[derive(Debug, Clone)]
pub struct DynamicWeightAdjuster {
    /// Current weights
    pub current_weights: OrderingWeights,
    /// Weight adjustment history
    pub adjustment_history: Vec<WeightAdjustment>,
    /// Performance tracking
    pub performance_tracker: PerformanceTracker,
    /// Adjustment enabled
    pub enabled: bool,
}

/// Weight adjustment record
#[derive(Debug, Clone)]
pub struct WeightAdjustment {
    /// Timestamp of adjustment
    pub timestamp: std::time::SystemTime,
    /// Old weights
    pub old_weights: OrderingWeights,
    /// New weights
    pub new_weights: OrderingWeights,
    /// Reason for adjustment
    pub reason: String,
    /// Performance impact
    pub performance_impact: f64,
}

/// Performance tracker for weight adjustments
#[derive(Debug, Clone, Default)]
pub struct PerformanceTracker {
    /// Recent performance scores
    pub recent_scores: Vec<f64>,
    /// Performance trend
    pub trend: PerformanceTrend,
    /// Best known weights
    pub best_weights: Option<OrderingWeights>,
    /// Best performance score
    pub best_score: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving,
    /// Performance is declining
    Declining,
    /// Performance is stable
    Stable,
    /// Performance is volatile
    Volatile,
}

impl Default for PerformanceTrend {
    fn default() -> Self {
        PerformanceTrend::Stable
    }
}

/// Multi-threading support for move ordering
#[derive(Debug, Clone)]
pub struct ThreadingSupport {
    /// Number of threads
    pub num_threads: usize,
    /// Thread pool (placeholder for future implementation)
    pub thread_pool: Option<()>,
    /// Parallel move scoring
    pub parallel_scoring: bool,
    /// Parallel cache operations
    pub parallel_cache: bool,
    /// Thread safety enabled
    pub thread_safe: bool,
}

/// Predictive move ordering system
#[derive(Debug, Clone)]
pub struct PredictiveOrdering {
    /// Prediction model
    pub prediction_model: PredictionModel,
    /// Historical patterns
    pub historical_patterns: Vec<MovePattern>,
    /// Prediction accuracy
    pub accuracy: f64,
    /// Prediction enabled
    pub enabled: bool,
}

/// Prediction model for move ordering
#[derive(Debug, Clone)]
pub struct PredictionModel {
    /// Model type
    pub model_type: PredictionModelType,
    /// Model parameters
    pub parameters: PredictionParameters,
    /// Training data
    pub training_data: Vec<PredictionExample>,
}

/// Prediction model types
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionModelType {
    /// Pattern-based prediction
    PatternBased,
    /// Statistical prediction
    Statistical,
    /// Hybrid prediction
    Hybrid,
}

/// Prediction parameters
#[derive(Debug, Clone)]
pub struct PredictionParameters {
    /// Lookahead depth
    pub lookahead_depth: usize,
    /// Pattern matching threshold
    pub pattern_threshold: f64,
    /// Statistical confidence
    pub confidence: f64,
}

/// Prediction example for training
#[derive(Debug, Clone)]
pub struct PredictionExample {
    /// Position features
    pub position_features: Vec<f64>,
    /// Predicted move
    pub predicted_move: Move,
    /// Actual best move
    pub actual_best_move: Move,
    /// Prediction accuracy
    pub accuracy: f64,
}

/// Move pattern for prediction
#[derive(Debug, Clone)]
pub struct MovePattern {
    /// Pattern features
    pub features: Vec<f64>,
    /// Associated moves
    pub moves: Vec<Move>,
    /// Pattern frequency
    pub frequency: usize,
    /// Pattern success rate
    pub success_rate: f64,
}

/// Advanced cache warming system
#[derive(Debug, Clone)]
pub struct AdvancedCacheWarming {
    /// Warming strategies
    pub strategies: Vec<CacheWarmingStrategy>,
    /// Warming enabled
    pub enabled: bool,
    /// Warming performance
    pub performance: CacheWarmingPerformance,
}

/// Cache warming strategy
#[derive(Debug, Clone)]
pub struct CacheWarmingStrategy {
    /// Strategy name
    pub name: String,
    /// Strategy type
    pub strategy_type: CacheWarmingType,
    /// Strategy parameters
    pub parameters: CacheWarmingParameters,
    /// Strategy effectiveness
    pub effectiveness: f64,
}

/// Cache warming strategy types
#[derive(Debug, Clone, PartialEq)]
pub enum CacheWarmingType {
    /// Precompute common positions
    PrecomputeCommon,
    /// Pattern-based warming
    PatternBased,
    /// Statistical warming
    Statistical,
    /// Hybrid warming
    Hybrid,
}

/// Cache warming parameters
#[derive(Debug, Clone)]
pub struct CacheWarmingParameters {
    /// Warming depth
    pub depth: usize,
    /// Warming time limit
    pub time_limit_ms: u64,
    /// Warming memory limit
    pub memory_limit_mb: usize,
}

/// Cache warming performance metrics
#[derive(Debug, Clone, Default)]
pub struct CacheWarmingPerformance {
    /// Cache hit rate improvement
    pub hit_rate_improvement: f64,
    /// Warming time
    pub warming_time_ms: u64,
    /// Memory usage
    pub memory_usage_mb: f64,
    /// Effectiveness score
    pub effectiveness_score: f64,
}

impl Default for AdvancedFeatures {
    fn default() -> Self {
        Self {
            position_strategies: PositionSpecificStrategies::default(),
            ml_model: MachineLearningModel::default(),
            dynamic_weights: DynamicWeightAdjuster::default(),
            threading_support: ThreadingSupport::default(),
            predictive_ordering: PredictiveOrdering::default(),
            cache_warming: AdvancedCacheWarming::default(),
        }
    }
}

impl Default for PositionSpecificStrategies {
    fn default() -> Self {
        Self {
            opening_strategy: OrderingStrategy::opening(),
            middlegame_strategy: OrderingStrategy::middlegame(),
            endgame_strategy: OrderingStrategy::endgame(),
            tactical_strategy: OrderingStrategy::tactical(),
            positional_strategy: OrderingStrategy::positional(),
            current_phase: GamePhase::Middlegame,
        }
    }
}

impl OrderingStrategy {
    /// Create opening strategy
    pub fn opening() -> Self {
        Self {
            name: "Opening".to_string(),
            weights: OrderingWeights {
                capture_weight: 1000,
                promotion_weight: 800,
                center_control_weight: 600,
                development_weight: 500,
                tactical_weight: 400,
                piece_value_weight: 300,
                position_value_weight: 200,
                quiet_weight: 100,
                see_weight: 700,
                pv_move_weight: 900,
                killer_move_weight: 600,
                history_weight: 400,
            },
            priority_adjustments: PriorityAdjustments {
                development_priority: 1.5,
                center_priority: 1.3,
                capture_priority: 1.0,
                promotion_priority: 1.0,
                king_safety_priority: 0.8,
            },
            heuristic_preferences: HeuristicPreferences {
                prefer_development: true,
                prefer_positional: true,
                prefer_tactical: false,
                prefer_endgame: false,
            },
        }
    }

    /// Create middlegame strategy
    pub fn middlegame() -> Self {
        Self {
            name: "Middlegame".to_string(),
            weights: OrderingWeights {
                capture_weight: 1000,
                promotion_weight: 800,
                center_control_weight: 500,
                development_weight: 300,
                tactical_weight: 600,
                piece_value_weight: 400,
                position_value_weight: 500,
                quiet_weight: 200,
                see_weight: 800,
                pv_move_weight: 900,
                killer_move_weight: 700,
                history_weight: 600,
            },
            priority_adjustments: PriorityAdjustments {
                capture_priority: 1.2,
                king_safety_priority: 1.3,
                center_priority: 1.1,
                development_priority: 0.8,
                promotion_priority: 1.0,
            },
            heuristic_preferences: HeuristicPreferences {
                prefer_tactical: true,
                prefer_positional: true,
                prefer_development: false,
                prefer_endgame: false,
            },
        }
    }

    /// Create endgame strategy
    pub fn endgame() -> Self {
        Self {
            name: "Endgame".to_string(),
            weights: OrderingWeights {
                capture_weight: 1000,
                promotion_weight: 1000,
                center_control_weight: 400,
                development_weight: 200,
                tactical_weight: 500,
                piece_value_weight: 600,
                position_value_weight: 600,
                quiet_weight: 300,
                see_weight: 900,
                pv_move_weight: 900,
                killer_move_weight: 600,
                history_weight: 500,
            },
            priority_adjustments: PriorityAdjustments {
                promotion_priority: 1.5,
                // piece_value_weight: 1.3, // Not available in PriorityAdjustments
                capture_priority: 1.2,
                king_safety_priority: 1.0,
                center_priority: 0.7,
                development_priority: 0.5,
            },
            heuristic_preferences: HeuristicPreferences {
                prefer_endgame: true,
                prefer_tactical: true,
                prefer_positional: false,
                prefer_development: false,
            },
        }
    }

    /// Create tactical strategy
    pub fn tactical() -> Self {
        Self {
            name: "Tactical".to_string(),
            weights: OrderingWeights {
                capture_weight: 1000,
                promotion_weight: 900,
                center_control_weight: 300,
                development_weight: 200,
                tactical_weight: 700,
                piece_value_weight: 500,
                position_value_weight: 300,
                quiet_weight: 100,
                see_weight: 1000,
                pv_move_weight: 900,
                killer_move_weight: 800,
                history_weight: 400,
            },
            priority_adjustments: PriorityAdjustments {
                capture_priority: 1.5,
                // see_weight: 1.4, // Not available in PriorityAdjustments
                king_safety_priority: 1.3,
                promotion_priority: 1.2,
                center_priority: 0.8,
                development_priority: 0.6,
            },
            heuristic_preferences: HeuristicPreferences {
                prefer_tactical: true,
                prefer_endgame: false,
                prefer_positional: false,
                prefer_development: false,
            },
        }
    }

    /// Create positional strategy
    pub fn positional() -> Self {
        Self {
            name: "Positional".to_string(),
            weights: OrderingWeights {
                capture_weight: 800,
                promotion_weight: 700,
                center_control_weight: 700,
                development_weight: 600,
                tactical_weight: 500,
                piece_value_weight: 400,
                position_value_weight: 800,
                quiet_weight: 500,
                see_weight: 600,
                pv_move_weight: 900,
                killer_move_weight: 500,
                history_weight: 700,
            },
            priority_adjustments: PriorityAdjustments {
                // position_weight: 1.4, // Not available in PriorityAdjustments
                center_priority: 1.3,
                development_priority: 1.2,
                // quiet_weight: 1.1, // Not available in PriorityAdjustments
                capture_priority: 0.9,
                king_safety_priority: 1.0,
                promotion_priority: 1.0,
            },
            heuristic_preferences: HeuristicPreferences {
                prefer_positional: true,
                prefer_development: true,
                prefer_tactical: false,
                prefer_endgame: false,
            },
        }
    }
}

impl Default for MachineLearningModel {
    fn default() -> Self {
        Self {
            model_type: MLModelType::LinearRegression,
            parameters: MLParameters {
                learning_rate: 0.01,
                regularization: 0.001,
                num_features: 20,
                complexity: 1.0,
            },
            training_data: Vec::new(),
            accuracy: 0.0,
            enabled: false,
        }
    }
}

impl Default for DynamicWeightAdjuster {
    fn default() -> Self {
        Self {
            current_weights: OrderingWeights::default(),
            adjustment_history: Vec::new(),
            performance_tracker: PerformanceTracker::default(),
            enabled: false,
        }
    }
}

impl Default for ThreadingSupport {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            thread_pool: None,
            parallel_scoring: false,
            parallel_cache: false,
            thread_safe: false,
        }
    }
}

impl Default for PredictiveOrdering {
    fn default() -> Self {
        Self {
            prediction_model: PredictionModel::default(),
            historical_patterns: Vec::new(),
            accuracy: 0.0,
            enabled: false,
        }
    }
}

impl Default for PredictionModel {
    fn default() -> Self {
        Self {
            model_type: PredictionModelType::PatternBased,
            parameters: PredictionParameters {
                lookahead_depth: 3,
                pattern_threshold: 0.7,
                confidence: 0.8,
            },
            training_data: Vec::new(),
        }
    }
}

impl Default for AdvancedCacheWarming {
    fn default() -> Self {
        Self {
            strategies: vec![
                CacheWarmingStrategy {
                    name: "Common Positions".to_string(),
                    strategy_type: CacheWarmingType::PrecomputeCommon,
                    parameters: CacheWarmingParameters {
                        depth: 2,
                        time_limit_ms: 1000,
                        memory_limit_mb: 50,
                    },
                    effectiveness: 0.0,
                },
            ],
            enabled: false,
            performance: CacheWarmingPerformance::default(),
        }
    }
}

/// Advanced feature flags for enabling/disabling features
#[derive(Debug, Clone, Default)]
pub struct AdvancedFeatureFlags {
    /// Position-specific strategies
    pub position_specific_strategies: bool,
    /// Machine learning
    pub machine_learning: bool,
    /// Dynamic weight adjustment
    pub dynamic_weights: bool,
    /// Predictive ordering
    pub predictive_ordering: bool,
    /// Cache warming
    pub cache_warming: bool,
}

/// Advanced feature status
#[derive(Debug, Clone)]
pub struct AdvancedFeatureStatus {
    /// Position-specific strategies enabled
    pub position_specific_strategies: bool,
    /// Machine learning enabled
    pub machine_learning: bool,
    /// Dynamic weights enabled
    pub dynamic_weights: bool,
    /// Predictive ordering enabled
    pub predictive_ordering: bool,
    /// Cache warming enabled
    pub cache_warming: bool,
    /// Current game phase
    pub current_phase: GamePhase,
    /// Machine learning accuracy
    pub ml_accuracy: f64,
    /// Prediction accuracy
    pub prediction_accuracy: f64,
}

/// Core move ordering system
/// 
/// This struct provides the fundamental move ordering functionality,
/// including basic sorting, statistics tracking, memory management,
/// and Principal Variation (PV) move ordering.
pub struct MoveOrdering {
    /// Performance statistics for move ordering
    pub stats: OrderingStats,
    /// Comprehensive configuration system
    pub config: MoveOrderingConfig,
    /// Memory usage tracking
    pub memory_usage: MemoryUsage,
    /// Move scoring cache for performance optimization (cache-friendly)
    move_score_cache: HashMap<u64, i32>,
    /// Fast cache for frequently accessed scores (L1 cache simulation)
    fast_score_cache: Vec<(u64, i32)>,
    /// Transposition table reference for PV move retrieval
    transposition_table: *const crate::search::ThreadSafeTranspositionTable,
    /// Hash calculator for position hashing
    hash_calculator: crate::search::ShogiHashHandler,
    /// PV move cache for performance optimization
    pv_move_cache: HashMap<u64, Option<Move>>,
    /// Killer moves organized by depth
    /// Each depth can have multiple killer moves
    killer_moves: HashMap<u8, Vec<Move>>,
    /// Current search depth for killer move management
    current_depth: u8,
    /// History table for move scoring
    /// Maps (piece_type, from_square, to_square) -> history score
    history_table: HashMap<(PieceType, Position, Position), u32>,
    /// Simple history table for position-based history (9x9 board)
    simple_history_table: [[i32; 9]; 9],
    /// History update counter for aging
    history_update_counter: u64,
    /// Pattern-based search integrator (Phase 3 - Task 3.2, available for search enhancements)
    #[allow(dead_code)]
    pattern_integrator: crate::evaluation::pattern_search_integration::PatternSearchIntegrator,
    /// SEE cache for performance optimization
    /// Maps (from_square, to_square) -> SEE value
    see_cache: HashMap<(Position, Position), i32>,
    /// Maximum SEE cache size
    max_see_cache_size: usize,
    /// Object pool for move scoring vectors (memory optimization)
    move_score_pool: Vec<(i32, usize)>,
    /// Object pool for move vectors (memory optimization)
    move_pool: Vec<Move>,
    /// Error handler for robust error management
    error_handler: ErrorHandler,
    /// Memory pool manager for efficient allocations
    memory_pool: MemoryPool,
    /// Memory usage tracker
    memory_tracker: MemoryTracker,
    /// Advanced features manager
    advanced_features: AdvancedFeatures,
    /// PV moves organized by depth
    pv_moves: HashMap<u8, Move>,
}

/// Performance statistics for move ordering
/// 
/// Tracks various metrics to monitor the effectiveness and performance
/// of the move ordering system.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct OrderingStats {
    /// Total number of moves ordered
    pub total_moves_ordered: u64,
    /// Total time spent on move ordering (microseconds)
    pub total_ordering_time_us: u64,
    /// Average time per move ordering operation (microseconds)
    pub avg_ordering_time_us: f64,
    /// Number of cache hits in move scoring
    pub cache_hits: u64,
    /// Number of cache misses in move scoring
    pub cache_misses: u64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Number of moves sorted
    pub moves_sorted: u64,
    /// Number of scoring operations performed
    pub scoring_operations: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_memory_usage_bytes: usize,
    /// Number of memory allocations
    pub memory_allocations: u64,
    /// Number of memory deallocations
    pub memory_deallocations: u64,
    /// Number of PV move hits
    pub pv_move_hits: u64,
    /// Number of PV move misses
    pub pv_move_misses: u64,
    /// PV move hit rate percentage
    pub pv_move_hit_rate: f64,
    /// Number of transposition table lookups
    pub tt_lookups: u64,
    /// Number of successful transposition table hits
    pub tt_hits: u64,
    /// Number of killer move hits
    pub killer_move_hits: u64,
    /// Number of killer move misses
    pub killer_move_misses: u64,
    /// Killer move hit rate percentage
    pub killer_move_hit_rate: f64,
    /// Number of killer moves stored
    pub killer_moves_stored: u64,
    /// Number of history heuristic hits
    pub history_hits: u64,
    /// Number of history heuristic misses
    pub history_misses: u64,
    /// History heuristic hit rate percentage
    pub history_hit_rate: f64,
    /// Number of history table updates
    pub history_updates: u64,
    /// Number of history table aging operations
    pub history_aging_operations: u64,
    /// Number of SEE calculations performed
    pub see_calculations: u64,
    /// Number of SEE cache hits
    pub see_cache_hits: u64,
    /// Number of SEE cache misses
    pub see_cache_misses: u64,
    /// SEE cache hit rate percentage
    pub see_cache_hit_rate: f64,
    /// Total time spent on SEE calculations (microseconds)
    pub see_calculation_time_us: u64,
    /// Average time per SEE calculation (microseconds)
    pub avg_see_calculation_time_us: f64,
    /// Hot path profiling data
    pub hot_path_stats: HotPathStats,
    /// Detailed heuristic statistics
    pub heuristic_stats: HeuristicStats,
    /// Advanced timing statistics
    pub timing_stats: TimingStats,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Cache performance statistics
    pub cache_stats: CacheStats,
    /// Transposition table integration statistics
    pub tt_integration_hits: u64,
    /// Number of TT integration updates
    pub tt_integration_updates: u64,
    /// Number of cutoff updates from TT
    pub tt_cutoff_updates: u64,
    /// Number of exact updates from TT
    pub tt_exact_updates: u64,
    /// Number of bound updates from TT
    pub tt_bound_updates: u64,
    /// Number of killer moves from TT
    pub killer_moves_from_tt: u64,
    /// Number of PV moves from TT
    pub pv_moves_from_tt: u64,
    /// Number of history updates from TT
    pub history_updates_from_tt: u64,
    /// Number of cutoff history updates
    pub cutoff_history_updates: u64,
    /// Number of opening book integrations
    pub opening_book_integrations: u64,
    /// Number of tablebase integrations
    pub tablebase_integrations: u64,
    /// Number of analysis mode orderings
    pub analysis_orderings: u64,
    /// Number of phase-specific orderings
    pub phase_specific_orderings: u64,
}

/// Hot path performance statistics for profiling bottlenecks
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HotPathStats {
    /// Number of score_move calls
    pub score_move_calls: u64,
    /// Number of cache lookups
    pub cache_lookups: u64,
    /// Number of hash calculations
    pub hash_calculations: u64,
    /// Time spent in score_move (microseconds)
    pub score_move_time_us: u64,
    /// Time spent in cache operations (microseconds)
    pub cache_time_us: u64,
    /// Time spent in hash calculations (microseconds)
    pub hash_time_us: u64,
}

/// Detailed heuristic statistics for tracking individual heuristic performance
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HeuristicStats {
    /// Capture move statistics
    pub capture_stats: HeuristicPerformance,
    /// Promotion move statistics
    pub promotion_stats: HeuristicPerformance,
    /// Tactical move statistics
    pub tactical_stats: HeuristicPerformance,
    /// Piece value statistics
    pub piece_value_stats: HeuristicPerformance,
    /// Position value statistics
    pub position_stats: HeuristicPerformance,
    /// Development move statistics
    pub development_stats: HeuristicPerformance,
    /// Quiet move statistics
    pub quiet_stats: HeuristicPerformance,
    /// PV move statistics
    pub pv_stats: HeuristicPerformance,
    /// Killer move statistics
    pub killer_stats: HeuristicPerformance,
    /// History move statistics
    pub history_stats: HeuristicPerformance,
    /// SEE move statistics
    pub see_stats: HeuristicPerformance,
}

/// Individual heuristic performance metrics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HeuristicPerformance {
    /// Number of times this heuristic was applied
    pub applications: u64,
    /// Number of times this heuristic contributed to the best move
    pub best_move_contributions: u64,
    /// Average score contribution from this heuristic
    pub avg_score_contribution: f64,
    /// Total score contribution from this heuristic
    pub total_score_contribution: i64,
    /// Time spent in this heuristic (microseconds)
    pub execution_time_us: u64,
    /// Average execution time per application (microseconds)
    pub avg_execution_time_us: f64,
}

/// Advanced timing statistics for detailed performance analysis
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TimingStats {
    /// Move scoring timing breakdown
    pub move_scoring_times: OperationTiming,
    /// Move ordering timing breakdown
    pub move_ordering_times: OperationTiming,
    /// Cache operation timing breakdown
    pub cache_times: OperationTiming,
    /// Hash calculation timing breakdown
    pub hash_times: OperationTiming,
    /// SEE calculation timing breakdown
    pub see_times: OperationTiming,
    /// PV move retrieval timing breakdown
    pub pv_times: OperationTiming,
    /// Killer move operations timing breakdown
    pub killer_times: OperationTiming,
    /// History table operations timing breakdown
    pub history_times: OperationTiming,
}

/// Timing statistics for a specific operation
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct OperationTiming {
    /// Total time spent in this operation (microseconds)
    pub total_time_us: u64,
    /// Number of operations performed
    pub operation_count: u64,
    /// Average time per operation (microseconds)
    pub avg_time_us: f64,
    /// Minimum time recorded (microseconds)
    pub min_time_us: u64,
    /// Maximum time recorded (microseconds)
    pub max_time_us: u64,
    /// Standard deviation of operation times
    pub std_dev_time_us: f64,
}

/// Detailed memory usage statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MemoryStats {
    /// Current memory usage breakdown
    pub current_usage: MemoryBreakdown,
    /// Peak memory usage breakdown
    pub peak_usage: MemoryBreakdown,
    /// Memory allocation statistics
    pub allocation_stats: AllocationStats,
    /// Memory fragmentation metrics
    pub fragmentation_stats: FragmentationStats,
}

/// Memory usage breakdown by component
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MemoryBreakdown {
    /// Move score cache memory usage
    pub move_score_cache_bytes: usize,
    /// Fast cache memory usage
    pub fast_cache_bytes: usize,
    /// PV move cache memory usage
    pub pv_cache_bytes: usize,
    /// Killer moves memory usage
    pub killer_moves_bytes: usize,
    /// History table memory usage
    pub history_table_bytes: usize,
    /// SEE cache memory usage
    pub see_cache_bytes: usize,
    /// Object pools memory usage
    pub object_pools_bytes: usize,
    /// Total memory usage
    pub total_bytes: usize,
}

/// Memory allocation statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AllocationStats {
    /// Total number of allocations
    pub total_allocations: u64,
    /// Number of deallocations
    pub total_deallocations: u64,
    /// Current number of active allocations
    pub active_allocations: u64,
    /// Peak number of active allocations
    pub peak_allocations: u64,
    /// Average allocation size
    pub avg_allocation_size: f64,
    /// Total memory allocated
    pub total_allocated_bytes: u64,
}

/// Memory fragmentation statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FragmentationStats {
    /// Fragmentation percentage
    pub fragmentation_percentage: f64,
    /// Number of free memory blocks
    pub free_blocks: u64,
    /// Average free block size
    pub avg_free_block_size: f64,
    /// Largest free block size
    pub largest_free_block: u64,
}

/// Cache performance statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CacheStats {
    /// Move score cache statistics
    pub move_score_cache: CachePerformance,
    /// Fast cache statistics
    pub fast_cache: CachePerformance,
    /// PV move cache statistics
    pub pv_cache: CachePerformance,
    /// SEE cache statistics
    pub see_cache: CachePerformance,
}

/// Cache performance metrics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CachePerformance {
    /// Cache hit rate percentage
    pub hit_rate: f64,
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Cache insertions
    pub insertions: u64,
    /// Current cache size
    pub current_size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache utilization percentage
    pub utilization: f64,
}

/// Comprehensive performance statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceStats {
    /// Total moves ordered
    pub total_moves_ordered: u64,
    /// Average ordering time per operation
    pub avg_ordering_time_us: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// SEE cache hit rate percentage
    pub see_cache_hit_rate: f64,
    /// Hot path performance data
    pub hot_path_stats: HotPathStats,
    /// Memory usage information
    pub memory_usage: MemoryUsage,
    /// Cache size information
    pub cache_sizes: CacheSizes,
}

/// Cache size information for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheSizes {
    /// Move score cache size
    pub move_score_cache: usize,
    /// Fast cache size
    pub fast_cache: usize,
    /// PV cache size
    pub pv_cache: usize,
    /// SEE cache size
    pub see_cache: usize,
    /// History table size
    pub history_table: usize,
}

/// Bottleneck analysis results
#[derive(Debug, Clone, serde::Serialize)]
pub struct BottleneckAnalysis {
    /// List of identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
    /// Overall performance score (0-100)
    pub overall_score: u8,
}

/// Individual bottleneck information
#[derive(Debug, Clone, serde::Serialize)]
pub struct Bottleneck {
    /// Category of the bottleneck
    pub category: BottleneckCategory,
    /// Severity of the bottleneck
    pub severity: BottleneckSeverity,
    /// Description of the bottleneck
    pub description: String,
    /// Recommendation for fixing the bottleneck
    pub recommendation: String,
}

/// Categories of performance bottlenecks
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum BottleneckCategory {
    /// Cache-related performance issues
    Cache,
    /// Hot path performance issues
    HotPath,
    /// Memory usage issues
    Memory,
    /// SEE cache performance issues
    SEECache,
    /// Hash calculation issues
    HashCalculation,
}

/// Severity levels for bottlenecks
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum BottleneckSeverity {
    /// Critical issue requiring immediate attention
    Critical,
    /// High priority issue
    High,
    /// Medium priority issue
    Medium,
    /// Low priority issue
    Low,
}

/// Statistics export data structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct StatisticsExport {
    /// Timestamp of export
    pub timestamp: u64,
    /// Complete ordering statistics
    pub ordering_stats: OrderingStats,
    /// Configuration used
    pub config: MoveOrderingConfig,
    /// Memory usage information
    pub memory_usage: MemoryUsage,
    /// Current cache sizes
    pub cache_sizes: CacheSizes,
}

/// Performance summary for quick analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceSummary {
    /// Total moves ordered
    pub total_moves_ordered: u64,
    /// Average ordering time per operation
    pub avg_ordering_time_us: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// SEE cache hit rate percentage
    pub see_cache_hit_rate: f64,
    /// Current memory usage in MB
    pub memory_usage_mb: f64,
    /// Peak memory usage in MB
    pub peak_memory_mb: f64,
    /// Most effective heuristic
    pub most_effective_heuristic: String,
    /// Overall performance score (0-100)
    pub performance_score: u8,
    /// Number of identified bottlenecks
    pub bottleneck_count: usize,
}

/// Performance chart data for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceChartData {
    /// Cache hit rates for different caches
    pub cache_hit_rates: CacheHitRates,
    /// Heuristic effectiveness percentages
    pub heuristic_effectiveness: HeuristicEffectiveness,
    /// Memory usage trend data
    pub memory_usage_trend: MemoryUsageTrend,
    /// Timing breakdown data
    pub timing_breakdown: TimingBreakdown,
}

/// Cache hit rates for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheHitRates {
    /// Move score cache hit rate
    pub move_score_cache: f64,
    /// Fast cache hit rate
    pub fast_cache: f64,
    /// PV cache hit rate
    pub pv_cache: f64,
    /// SEE cache hit rate
    pub see_cache: f64,
}

/// Heuristic effectiveness for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeuristicEffectiveness {
    /// Capture heuristic effectiveness
    pub capture: f64,
    /// Promotion heuristic effectiveness
    pub promotion: f64,
    /// Tactical heuristic effectiveness
    pub tactical: f64,
    /// PV heuristic effectiveness
    pub pv: f64,
    /// Killer heuristic effectiveness
    pub killer: f64,
    /// History heuristic effectiveness
    pub history: f64,
}

/// Memory usage trend for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryUsageTrend {
    /// Current memory usage in MB
    pub current_mb: f64,
    /// Peak memory usage in MB
    pub peak_mb: f64,
    /// Total allocation count
    pub allocation_count: u64,
}

/// Timing breakdown for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimingBreakdown {
    /// Average move scoring time in microseconds
    pub move_scoring_avg_us: f64,
    /// Average move ordering time in microseconds
    pub move_ordering_avg_us: f64,
    /// Average cache operation time in microseconds
    pub cache_avg_us: f64,
    /// Average hash calculation time in microseconds
    pub hash_avg_us: f64,
}

/// Performance trend analysis results
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceTrendAnalysis {
    /// Cache efficiency trend analysis
    pub cache_efficiency_trend: TrendAnalysis,
    /// Memory usage trend analysis
    pub memory_usage_trend: TrendAnalysis,
    /// Heuristic effectiveness trend analysis
    pub heuristic_effectiveness_trend: TrendAnalysis,
    /// Timing trend analysis
    pub timing_trend: TrendAnalysis,
    /// Overall performance trend analysis
    pub overall_performance_trend: TrendAnalysis,
}

/// Individual trend analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct TrendAnalysis {
    /// Direction of the trend
    pub direction: TrendDirection,
    /// Confidence level in the trend (0.0 to 1.0)
    pub confidence: f64,
    /// Recommendation based on the trend
    pub recommendation: String,
}

/// Trend direction indicators
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TrendDirection {
    /// Performance is improving
    Improving,
    /// Performance is declining
    Declining,
    /// Performance is stable
    Stable,
}

/// Comprehensive configuration system for move ordering
/// 
/// This struct contains all configuration options for the move ordering system,
/// including weights, cache settings, and behavioral parameters.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveOrderingConfig {
    /// Heuristic weights for move scoring
    pub weights: OrderingWeights,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Killer move configuration
    pub killer_config: KillerConfig,
    /// History heuristic configuration
    pub history_config: HistoryConfig,
    /// Performance configuration
    pub performance_config: PerformanceConfig,
    /// Debug and logging configuration
    pub debug_config: DebugConfig,
}

/// Configuration weights for move ordering heuristics
/// 
/// Allows fine-tuning of different move ordering strategies
/// to optimize performance for specific positions or game phases.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrderingWeights {
    /// Weight for capture moves
    pub capture_weight: i32,
    /// Weight for promotion moves
    pub promotion_weight: i32,
    /// Weight for center control moves
    pub center_control_weight: i32,
    /// Weight for development moves
    pub development_weight: i32,
    /// Weight for piece value
    pub piece_value_weight: i32,
    /// Weight for position value
    pub position_value_weight: i32,
    /// Weight for tactical moves
    pub tactical_weight: i32,
    /// Weight for quiet moves
    pub quiet_weight: i32,
    /// Weight for PV moves (highest priority)
    pub pv_move_weight: i32,
    /// Weight for killer moves
    pub killer_move_weight: i32,
    /// Weight for history heuristic moves
    pub history_weight: i32,
    /// Weight for SEE (Static Exchange Evaluation) moves
    pub see_weight: i32,
}

/// Cache configuration for move ordering
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheConfig {
    /// Maximum cache size for move scores
    pub max_cache_size: usize,
    /// Enable cache warming
    pub enable_cache_warming: bool,
    /// Cache warming size (percentage of max_cache_size)
    pub cache_warming_ratio: f32,
    /// Enable automatic cache optimization
    pub enable_auto_optimization: bool,
    /// Cache hit rate threshold for optimization (percentage)
    pub optimization_hit_rate_threshold: f64,
    /// Maximum SEE cache size
    pub max_see_cache_size: usize,
    /// Enable SEE cache
    pub enable_see_cache: bool,
}

/// Killer move configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct KillerConfig {
    /// Maximum number of killer moves per depth
    pub max_killer_moves_per_depth: usize,
    /// Enable killer move aging
    pub enable_killer_aging: bool,
    /// Killer move aging factor (0.0 to 1.0)
    pub killer_aging_factor: f32,
    /// Enable depth-based killer move management
    pub enable_depth_based_management: bool,
}

/// History heuristic configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryConfig {
    /// Maximum history score to prevent overflow
    pub max_history_score: u32,
    /// History aging factor (0.0 to 1.0)
    pub history_aging_factor: f32,
    /// Enable automatic history aging
    pub enable_automatic_aging: bool,
    /// History aging frequency (number of updates between aging)
    pub aging_frequency: u64,
    /// Enable history score clamping
    pub enable_score_clamping: bool,
}

/// Performance configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Performance monitoring interval (milliseconds)
    pub monitoring_interval_ms: u64,
    /// Enable memory usage tracking
    pub enable_memory_tracking: bool,
    /// Memory usage warning threshold (bytes)
    pub memory_warning_threshold: usize,
    /// Enable automatic performance optimization
    pub enable_auto_optimization: bool,
}

/// Debug and logging configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct DebugConfig {
    /// Enable debug logging
    pub enable_debug_logging: bool,
    /// Enable move ordering statistics
    pub enable_statistics: bool,
    /// Enable detailed performance metrics
    pub enable_detailed_metrics: bool,
    /// Log level (0 = none, 1 = basic, 2 = detailed, 3 = verbose)
    pub log_level: u8,
}

impl Default for MoveOrderingConfig {
    fn default() -> Self {
        Self {
            weights: OrderingWeights::default(),
            cache_config: CacheConfig::default(),
            killer_config: KillerConfig::default(),
            history_config: HistoryConfig::default(),
            performance_config: PerformanceConfig::default(),
            debug_config: DebugConfig::default(),
        }
    }
}

impl Default for OrderingWeights {
    fn default() -> Self {
        Self {
            capture_weight: 1000,
            promotion_weight: 800,
            center_control_weight: 100,
            development_weight: 150,
            piece_value_weight: 50,
            position_value_weight: 75,
            tactical_weight: 300,
            quiet_weight: 25,
            pv_move_weight: 10000, // Highest priority for PV moves
            killer_move_weight: 5000, // High priority for killer moves
            history_weight: 2500, // Medium-high priority for history moves
            see_weight: 2000, // High priority for SEE moves
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            enable_cache_warming: true,
            cache_warming_ratio: 0.5, // 50% of max cache size
            enable_auto_optimization: true,
            optimization_hit_rate_threshold: 30.0, // 30% hit rate threshold
            max_see_cache_size: 500,
            enable_see_cache: true,
        }
    }
}

impl Default for KillerConfig {
    fn default() -> Self {
        Self {
            max_killer_moves_per_depth: 2,
            enable_killer_aging: false, // Disabled by default
            killer_aging_factor: 0.9,
            enable_depth_based_management: true,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_history_score: 10000,
            history_aging_factor: 0.9,
            enable_automatic_aging: true,
            aging_frequency: 1000, // Age every 1000 updates
            enable_score_clamping: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            monitoring_interval_ms: 1000, // 1 second
            enable_memory_tracking: true,
            memory_warning_threshold: 10 * 1024 * 1024, // 10MB
            enable_auto_optimization: true,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_debug_logging: false,
            enable_statistics: true,
            enable_detailed_metrics: false,
            log_level: 1, // Basic logging
        }
    }
}

impl MoveOrderingConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration and return any errors
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate weights
        if self.weights.capture_weight < 0 {
            errors.push("Capture weight must be non-negative".to_string());
        }
        if self.weights.promotion_weight < 0 {
            errors.push("Promotion weight must be non-negative".to_string());
        }
        if self.weights.tactical_weight < 0 {
            errors.push("Tactical weight must be non-negative".to_string());
        }
        if self.weights.quiet_weight < 0 {
            errors.push("Quiet weight must be non-negative".to_string());
        }
        if self.weights.pv_move_weight < 0 {
            errors.push("PV move weight must be non-negative".to_string());
        }
        if self.weights.killer_move_weight < 0 {
            errors.push("Killer move weight must be non-negative".to_string());
        }
        if self.weights.history_weight < 0 {
            errors.push("History weight must be non-negative".to_string());
        }

        // Validate cache configuration
        if self.cache_config.max_cache_size == 0 {
            errors.push("Max cache size must be greater than 0".to_string());
        }
        if self.cache_config.cache_warming_ratio < 0.0 || self.cache_config.cache_warming_ratio > 1.0 {
            errors.push("Cache warming ratio must be between 0.0 and 1.0".to_string());
        }
        if self.cache_config.optimization_hit_rate_threshold < 0.0 || self.cache_config.optimization_hit_rate_threshold > 100.0 {
            errors.push("Optimization hit rate threshold must be between 0.0 and 100.0".to_string());
        }

        // Validate killer configuration
        if self.killer_config.max_killer_moves_per_depth == 0 {
            errors.push("Max killer moves per depth must be greater than 0".to_string());
        }
        if self.killer_config.killer_aging_factor < 0.0 || self.killer_config.killer_aging_factor > 1.0 {
            errors.push("Killer aging factor must be between 0.0 and 1.0".to_string());
        }

        // Validate history configuration
        if self.history_config.max_history_score == 0 {
            errors.push("Max history score must be greater than 0".to_string());
        }
        if self.history_config.history_aging_factor < 0.0 || self.history_config.history_aging_factor > 1.0 {
            errors.push("History aging factor must be between 0.0 and 1.0".to_string());
        }
        if self.history_config.aging_frequency == 0 {
            errors.push("Aging frequency must be greater than 0".to_string());
        }

        // Validate performance configuration
        if self.performance_config.monitoring_interval_ms == 0 {
            errors.push("Monitoring interval must be greater than 0".to_string());
        }
        if self.performance_config.memory_warning_threshold == 0 {
            errors.push("Memory warning threshold must be greater than 0".to_string());
        }

        // Validate debug configuration
        if self.debug_config.log_level > 3 {
            errors.push("Log level must be between 0 and 3".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        let mut config = Self::default();
        
        // Optimize cache settings
        config.cache_config.max_cache_size = 5000;
        config.cache_config.enable_cache_warming = true;
        config.cache_config.cache_warming_ratio = 0.7;
        config.cache_config.enable_auto_optimization = true;
        config.cache_config.optimization_hit_rate_threshold = 20.0;

        // Optimize killer move settings
        config.killer_config.max_killer_moves_per_depth = 3;
        config.killer_config.enable_depth_based_management = true;

        // Optimize history settings
        config.history_config.max_history_score = 15000;
        config.history_config.enable_automatic_aging = true;
        config.history_config.aging_frequency = 500;

        // Enable performance monitoring
        config.performance_config.enable_performance_monitoring = true;
        config.performance_config.enable_auto_optimization = true;

        // Disable debug logging for performance
        config.debug_config.enable_debug_logging = false;
        config.debug_config.log_level = 0;

        config
    }

    /// Create a configuration optimized for debugging
    pub fn debug_optimized() -> Self {
        let mut config = Self::default();
        
        // Smaller cache for debugging
        config.cache_config.max_cache_size = 500;
        config.cache_config.enable_cache_warming = false;
        config.cache_config.enable_auto_optimization = false;

        // Reduced killer moves for debugging
        config.killer_config.max_killer_moves_per_depth = 1;

        // Smaller history table for debugging
        config.history_config.max_history_score = 5000;
        config.history_config.enable_automatic_aging = false;

        // Enable all debugging features
        config.debug_config.enable_debug_logging = true;
        config.debug_config.enable_statistics = true;
        config.debug_config.enable_detailed_metrics = true;
        config.debug_config.log_level = 3;

        config
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        let mut config = Self::default();
        
        // Minimal cache settings
        config.cache_config.max_cache_size = 100;
        config.cache_config.enable_cache_warming = false;
        config.cache_config.enable_auto_optimization = false;

        // Minimal killer move settings
        config.killer_config.max_killer_moves_per_depth = 1;
        config.killer_config.enable_depth_based_management = false;

        // Minimal history settings
        config.history_config.max_history_score = 1000;
        config.history_config.enable_automatic_aging = true;
        config.history_config.aging_frequency = 100;

        // Enable memory tracking
        config.performance_config.enable_memory_tracking = true;
        config.performance_config.memory_warning_threshold = 1024 * 1024; // 1MB

        // Minimal debug settings
        config.debug_config.enable_debug_logging = false;
        config.debug_config.enable_detailed_metrics = false;
        config.debug_config.log_level = 0;

        config
    }

    /// Merge this configuration with another, with the other taking precedence
    pub fn merge(&self, other: &MoveOrderingConfig) -> MoveOrderingConfig {
        MoveOrderingConfig {
            weights: OrderingWeights {
                capture_weight: other.weights.capture_weight,
                promotion_weight: other.weights.promotion_weight,
                center_control_weight: other.weights.center_control_weight,
                development_weight: other.weights.development_weight,
                piece_value_weight: other.weights.piece_value_weight,
                position_value_weight: other.weights.position_value_weight,
                tactical_weight: other.weights.tactical_weight,
                quiet_weight: other.weights.quiet_weight,
                pv_move_weight: other.weights.pv_move_weight,
                killer_move_weight: other.weights.killer_move_weight,
                history_weight: other.weights.history_weight,
                see_weight: other.weights.see_weight,
            },
            cache_config: CacheConfig {
                max_cache_size: other.cache_config.max_cache_size,
                enable_cache_warming: other.cache_config.enable_cache_warming,
                cache_warming_ratio: other.cache_config.cache_warming_ratio,
                enable_auto_optimization: other.cache_config.enable_auto_optimization,
                optimization_hit_rate_threshold: other.cache_config.optimization_hit_rate_threshold,
                max_see_cache_size: other.cache_config.max_see_cache_size,
                enable_see_cache: other.cache_config.enable_see_cache,
            },
            killer_config: KillerConfig {
                max_killer_moves_per_depth: other.killer_config.max_killer_moves_per_depth,
                enable_killer_aging: other.killer_config.enable_killer_aging,
                killer_aging_factor: other.killer_config.killer_aging_factor,
                enable_depth_based_management: other.killer_config.enable_depth_based_management,
            },
            history_config: HistoryConfig {
                max_history_score: other.history_config.max_history_score,
                history_aging_factor: other.history_config.history_aging_factor,
                enable_automatic_aging: other.history_config.enable_automatic_aging,
                aging_frequency: other.history_config.aging_frequency,
                enable_score_clamping: other.history_config.enable_score_clamping,
            },
            performance_config: PerformanceConfig {
                enable_performance_monitoring: other.performance_config.enable_performance_monitoring,
                monitoring_interval_ms: other.performance_config.monitoring_interval_ms,
                enable_memory_tracking: other.performance_config.enable_memory_tracking,
                memory_warning_threshold: other.performance_config.memory_warning_threshold,
                enable_auto_optimization: other.performance_config.enable_auto_optimization,
            },
            debug_config: DebugConfig {
                enable_debug_logging: other.debug_config.enable_debug_logging,
                enable_statistics: other.debug_config.enable_statistics,
                enable_detailed_metrics: other.debug_config.enable_detailed_metrics,
                log_level: other.debug_config.log_level,
            },
        }
    }

    /// Create a configuration from a JSON string
    pub fn from_json(_json: &str) -> Result<Self, String> {
        // This would typically use serde_json, but we'll implement a simple version
        // For now, return an error indicating JSON parsing is not implemented
        Err("JSON configuration parsing not implemented yet".to_string())
    }

    /// Serialize the configuration to JSON
    pub fn to_json(&self) -> Result<String, String> {
        // This would typically use serde_json, but we'll implement a simple version
        // For now, return an error indicating JSON serialization is not implemented
        Err("JSON configuration serialization not implemented yet".to_string())
    }
}

/// Memory usage tracking for move ordering
/// 
/// Monitors memory consumption to ensure efficient resource usage
/// and detect potential memory leaks.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MemoryUsage {
    /// Current memory usage in bytes
    pub current_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_bytes: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Total bytes allocated
    pub total_allocated_bytes: usize,
    /// Total bytes deallocated
    pub total_deallocated_bytes: usize,
}

impl MoveOrdering {
    /// Create a new move orderer with default configuration
    pub fn new() -> Self {
        let config = MoveOrderingConfig::default();
        Self::with_config(config)
    }

    /// Create a new move orderer with custom configuration
    pub fn with_config(config: MoveOrderingConfig) -> Self {
        Self {
            stats: OrderingStats {
                hot_path_stats: HotPathStats::default(),
                heuristic_stats: HeuristicStats::default(),
                timing_stats: TimingStats::default(),
                memory_stats: MemoryStats::default(),
                cache_stats: CacheStats::default(),
                ..OrderingStats::default()
            },
            config: config.clone(),
            memory_usage: MemoryUsage::default(),
            move_score_cache: HashMap::new(),
            fast_score_cache: Vec::with_capacity(64), // Small L1 cache for hot scores
            transposition_table: ptr::null(),
            hash_calculator: crate::search::ShogiHashHandler::new(config.cache_config.max_cache_size),
            pv_move_cache: HashMap::new(),
            killer_moves: HashMap::new(),
            current_depth: 0,
            history_table: HashMap::new(),
            history_update_counter: 0,
            pattern_integrator: crate::evaluation::pattern_search_integration::PatternSearchIntegrator::new(),
            see_cache: HashMap::new(),
            max_see_cache_size: config.cache_config.max_see_cache_size,
            move_score_pool: Vec::with_capacity(256), // Pre-allocate for common move lists
            move_pool: Vec::with_capacity(256), // Pre-allocate for common move lists
            error_handler: ErrorHandler::default(),
            memory_pool: MemoryPool::default(),
            memory_tracker: MemoryTracker::default(),
            advanced_features: AdvancedFeatures::default(),
            simple_history_table: [[0; 9]; 9],
            pv_moves: HashMap::new(),
        }
    }

    /// Create a new move orderer with performance-optimized configuration
    pub fn performance_optimized() -> Self {
        let config = MoveOrderingConfig::performance_optimized();
        Self::with_config(config)
    }

    /// Create a new move orderer with debug-optimized configuration
    pub fn debug_optimized() -> Self {
        let config = MoveOrderingConfig::debug_optimized();
        Self::with_config(config)
    }

    /// Create a new move orderer with memory-optimized configuration
    pub fn memory_optimized() -> Self {
        let config = MoveOrderingConfig::memory_optimized();
        Self::with_config(config)
    }

    /// Order moves using basic sorting heuristics
    /// 
    /// This is the core method that takes a list of moves and returns them
    /// ordered by priority using various heuristics.
    pub fn order_moves(&mut self, moves: &[Move]) -> MoveOrderingResult<Vec<Move>> {
        if moves.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // OPTIMIZATION: Use memory pool to reduce memory allocations
        let mut ordered_moves = self.memory_pool.get_move_vec();
        ordered_moves.reserve(moves.len());

        // OPTIMIZATION: Use memory pool for move scores to reduce allocations
        let mut move_scores = self.memory_pool.get_move_score_vec();
        move_scores.reserve(moves.len());

        // OPTIMIZATION: Pre-compute scores to avoid redundant calculations during sorting
        for (i, move_) in moves.iter().enumerate() {
            let score = self.score_move(move_)?;
            move_scores.push((score, i));
        }

        // OPTIMIZATION: Sort by score using stable sort for deterministic ordering
        move_scores.sort_by(|a, b| b.0.cmp(&a.0));

        // OPTIMIZATION: Rebuild ordered moves using pre-computed scores
        for (_, index) in &move_scores {
            ordered_moves.push(moves[*index].clone());
        }

        // OPTIMIZATION: Return objects to memory pool for reuse
        self.memory_pool.return_move_score_vec(move_scores);

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        // OPTIMIZATION: Return result and return ordered moves to pool
        let result = Ok(ordered_moves);
        // Note: We can't return ordered_moves to pool here since it's returned to caller
        // The caller should return it to the pool when done
        result
    }

    /// Score a move using comprehensive heuristics (optimized hot path)
    /// 
    /// Combines all available heuristics to assign a score to each move,
    /// which determines its priority in the ordering. This is the main
    /// scoring method that integrates all move evaluation strategies.
    /// 
    /// PERFORMANCE OPTIMIZATION: Inlined critical scoring functions
    /// and reduced function call overhead for hot path operations.
    pub fn score_move(&mut self, move_: &Move) -> MoveOrderingResult<i32> {
        // Validate the move first
        self.validate_move(move_)?;

        let start_time = TimeSource::now();
        self.stats.hot_path_stats.score_move_calls += 1;

        let hash_start = TimeSource::now();
        let move_hash = self.get_move_hash_fast(move_);
        self.stats.hot_path_stats.hash_time_us += hash_start.elapsed_ms() as u64 * 1000;
        self.stats.hot_path_stats.hash_calculations += 1;
        
        let cache_start = TimeSource::now();
        
        // OPTIMIZATION: Check fast cache first (L1 cache simulation)
        for &(hash, score) in &self.fast_score_cache {
            if hash == move_hash {
                self.stats.cache_hits += 1;
                self.stats.hot_path_stats.cache_lookups += 1;
                self.stats.hot_path_stats.cache_time_us += cache_start.elapsed_ms() as u64 * 1000;
                self.stats.hot_path_stats.score_move_time_us += start_time.elapsed_ms() as u64 * 1000;
                return Ok(score);
            }
        }

        // Check main cache (L2 cache simulation)
        if let Some(&cached_score) = self.move_score_cache.get(&move_hash) {
            self.stats.cache_hits += 1;
            self.stats.hot_path_stats.cache_lookups += 1;
            
            // OPTIMIZATION: Promote to fast cache if frequently accessed
            if self.fast_score_cache.len() < 64 {
                self.fast_score_cache.push((move_hash, cached_score));
            }
            
            self.stats.hot_path_stats.cache_time_us += cache_start.elapsed_ms() as u64 * 1000;
            self.stats.hot_path_stats.score_move_time_us += start_time.elapsed_ms() as u64 * 1000;
            return Ok(cached_score);
        }

        self.stats.cache_misses += 1;
        self.stats.scoring_operations += 1;

        let mut score = 0;

        // OPTIMIZATION: Inline critical scoring functions to reduce call overhead
        // 1. Capture scoring (highest priority for tactical moves)
        let score_capture = if move_.is_capture {
            let capture_score = self.score_capture_move_inline(move_);
            score += capture_score;
            capture_score
        } else { 0 };

        // 2. Promotion scoring (high priority for strategic moves)
        let score_promotion = if move_.is_promotion {
            let promotion_score = self.score_promotion_move_inline(move_);
            score += promotion_score;
            promotion_score
        } else { 0 };

        // 3. Tactical scoring (checks, threats, etc.)
        let score_tactical = if move_.gives_check {
            score += self.config.weights.tactical_weight;
            self.config.weights.tactical_weight
        } else { 0 };

        // 4. Piece value scoring (base piece values) - inlined for performance
        let score_piece = move_.piece_type.base_value() / 20; // Scaled down for move ordering
        score += score_piece;

        // 5. Position scoring (center control, king safety, etc.) - optimized
        let score_position = self.score_position_value_fast(move_);
        score += score_position;

        // 6. Development scoring (piece development, mobility) - optimized
        let score_development = self.score_development_move_fast(move_);
        score += score_development;

        // 7. Quiet move scoring (positional considerations) - only for non-tactical moves
        let score_quiet = if !move_.is_capture && !move_.is_promotion && !move_.gives_check {
            score += self.config.weights.quiet_weight;
            self.config.weights.quiet_weight
        } else { 0 };

        // Cache the score (with size limit)
        if self.move_score_cache.len() < self.config.cache_config.max_cache_size {
            self.move_score_cache.insert(move_hash, score);
        }

        // OPTIMIZATION: Update profiling statistics
        let total_time = start_time.elapsed_ms() as u64 * 1000;
        let cache_time = cache_start.elapsed_ms() as u64 * 1000;
        
        self.stats.hot_path_stats.cache_time_us += cache_time;
        self.stats.hot_path_stats.score_move_time_us += total_time;
        
        // Update detailed timing statistics
        self.record_timing("move_scoring", total_time);
        self.record_timing("cache", cache_time);

        // Update heuristic statistics
        self.update_heuristic_stats("capture", move_.is_capture, score_capture);
        self.update_heuristic_stats("promotion", move_.is_promotion, score_promotion);
        self.update_heuristic_stats("tactical", move_.gives_check, score_tactical);
        self.update_heuristic_stats("piece_value", true, score_piece);
        self.update_heuristic_stats("position", true, score_position);
        self.update_heuristic_stats("development", true, score_development);
        self.update_heuristic_stats("quiet", !move_.is_capture && !move_.is_promotion && !move_.gives_check, score_quiet);

        Ok(score)
    }

    /// Update heuristic performance statistics
    fn update_heuristic_stats(&mut self, heuristic_name: &str, applied: bool, score_contribution: i32) {
        if !applied {
            return;
        }

        let heuristic_stats = match heuristic_name {
            "capture" => &mut self.stats.heuristic_stats.capture_stats,
            "promotion" => &mut self.stats.heuristic_stats.promotion_stats,
            "tactical" => &mut self.stats.heuristic_stats.tactical_stats,
            "piece_value" => &mut self.stats.heuristic_stats.piece_value_stats,
            "position" => &mut self.stats.heuristic_stats.position_stats,
            "development" => &mut self.stats.heuristic_stats.development_stats,
            "quiet" => &mut self.stats.heuristic_stats.quiet_stats,
            "pv" => &mut self.stats.heuristic_stats.pv_stats,
            "killer" => &mut self.stats.heuristic_stats.killer_stats,
            "history" => &mut self.stats.heuristic_stats.history_stats,
            "see" => &mut self.stats.heuristic_stats.see_stats,
            _ => return,
        };

        heuristic_stats.applications += 1;
        heuristic_stats.total_score_contribution += score_contribution as i64;
        heuristic_stats.avg_score_contribution = 
            heuristic_stats.total_score_contribution as f64 / heuristic_stats.applications as f64;
    }

    /// Record that a heuristic contributed to the best move
    #[allow(dead_code)] // Kept for future use and debugging
    fn record_best_move_contribution(&mut self, heuristic_name: &str) {
        let heuristic_stats = match heuristic_name {
            "capture" => &mut self.stats.heuristic_stats.capture_stats,
            "promotion" => &mut self.stats.heuristic_stats.promotion_stats,
            "tactical" => &mut self.stats.heuristic_stats.tactical_stats,
            "piece_value" => &mut self.stats.heuristic_stats.piece_value_stats,
            "position" => &mut self.stats.heuristic_stats.position_stats,
            "development" => &mut self.stats.heuristic_stats.development_stats,
            "quiet" => &mut self.stats.heuristic_stats.quiet_stats,
            "pv" => &mut self.stats.heuristic_stats.pv_stats,
            "killer" => &mut self.stats.heuristic_stats.killer_stats,
            "history" => &mut self.stats.heuristic_stats.history_stats,
            "see" => &mut self.stats.heuristic_stats.see_stats,
            _ => return,
        };

        heuristic_stats.best_move_contributions += 1;
    }

    /// Record timing for an operation
    fn record_timing(&mut self, operation_name: &str, duration_us: u64) {
        let timing_stats = match operation_name {
            "move_scoring" => &mut self.stats.timing_stats.move_scoring_times,
            "move_ordering" => &mut self.stats.timing_stats.move_ordering_times,
            "cache" => &mut self.stats.timing_stats.cache_times,
            "hash" => &mut self.stats.timing_stats.hash_times,
            "see" => &mut self.stats.timing_stats.see_times,
            "pv" => &mut self.stats.timing_stats.pv_times,
            "killer" => &mut self.stats.timing_stats.killer_times,
            "history" => &mut self.stats.timing_stats.history_times,
            _ => return,
        };

        timing_stats.total_time_us += duration_us;
        timing_stats.operation_count += 1;
        timing_stats.avg_time_us = timing_stats.total_time_us as f64 / timing_stats.operation_count as f64;
        
        if timing_stats.min_time_us == 0 || duration_us < timing_stats.min_time_us {
            timing_stats.min_time_us = duration_us;
        }
        if duration_us > timing_stats.max_time_us {
            timing_stats.max_time_us = duration_us;
        }
    }

    /// Update cache performance statistics
    #[allow(dead_code)] // Kept for future use and debugging
    fn update_cache_stats(&mut self, cache_name: &str, hit: bool, size: usize, max_size: usize) {
        let cache_stats = match cache_name {
            "move_score_cache" => &mut self.stats.cache_stats.move_score_cache,
            "fast_cache" => &mut self.stats.cache_stats.fast_cache,
            "pv_cache" => &mut self.stats.cache_stats.pv_cache,
            "see_cache" => &mut self.stats.cache_stats.see_cache,
            _ => return,
        };

        if hit {
            cache_stats.hits += 1;
        } else {
            cache_stats.misses += 1;
        }

        cache_stats.current_size = size;
        cache_stats.max_size = max_size;
        cache_stats.utilization = if max_size > 0 { (size as f64 / max_size as f64) * 100.0 } else { 0.0 };
        
        let total_attempts = cache_stats.hits + cache_stats.misses;
        cache_stats.hit_rate = if total_attempts > 0 { (cache_stats.hits as f64 / total_attempts as f64) * 100.0 } else { 0.0 };
    }

    /// Update memory usage statistics
    #[allow(dead_code)] // Kept for future use and debugging
    fn update_memory_stats(&mut self) {
        let current_usage = MemoryBreakdown {
            move_score_cache_bytes: self.move_score_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<i32>()),
            fast_cache_bytes: self.fast_score_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<i32>()),
            pv_cache_bytes: self.pv_move_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<Move>()),
            killer_moves_bytes: self.killer_moves.len() * (std::mem::size_of::<u8>() + std::mem::size_of::<Vec<Move>>()),
            history_table_bytes: self.history_table.len() * (std::mem::size_of::<(PieceType, Position, Position)>() + std::mem::size_of::<u32>()),
            see_cache_bytes: self.see_cache.len() * (std::mem::size_of::<(Position, Position)>() + std::mem::size_of::<i32>()),
            object_pools_bytes: self.move_score_pool.capacity() * (std::mem::size_of::<(i32, usize)>()) + self.move_pool.capacity() * (std::mem::size_of::<Move>()),
            total_bytes: 0,
        };

        let total_bytes = current_usage.move_score_cache_bytes + 
                         current_usage.fast_cache_bytes + 
                         current_usage.pv_cache_bytes + 
                         current_usage.killer_moves_bytes + 
                         current_usage.history_table_bytes + 
                         current_usage.see_cache_bytes + 
                         current_usage.object_pools_bytes;

        let mut current_usage = current_usage;
        current_usage.total_bytes = total_bytes;

        // Update peak usage if current usage is higher
        if total_bytes > self.stats.memory_stats.peak_usage.total_bytes {
            self.stats.memory_stats.peak_usage = current_usage.clone();
        }

        self.stats.memory_stats.current_usage = current_usage;
    }

    /// Score a move using Static Exchange Evaluation (SEE)
    /// 
    /// SEE evaluates the material gain/loss from a sequence of captures
    /// starting with the given move. This provides a more accurate assessment
    /// of capture moves than simple piece values.
    pub fn score_see_move(&mut self, move_: &Move, board: &crate::bitboards::BitboardBoard) -> MoveOrderingResult<i32> {
        // Validate the move first
        self.validate_move(move_)?;

        if !move_.is_capture {
            return Ok(0);
        }

        let see_value = self.calculate_see(move_, board)?;
        let see_score = (see_value * self.config.weights.see_weight) / 1000;
        
        // Update statistics
        self.stats.see_calculations += 1;
        
        Ok(see_score)
    }

    /// Calculate Static Exchange Evaluation (SEE) for a move (optimized)
    /// 
    /// This method simulates the sequence of captures that would follow
    /// the given move and returns the net material gain/loss.
    /// 
    /// PERFORMANCE OPTIMIZATION: Fast cache key generation and optimized lookup.
    pub fn calculate_see(&mut self, move_: &Move, board: &crate::bitboards::BitboardBoard) -> MoveOrderingResult<i32> {
        let start_time = TimeSource::now();
        
        // OPTIMIZATION: Fast cache key generation using bit manipulation
        if self.config.cache_config.enable_see_cache {
            let from_pos = move_.from.unwrap_or(Position::new(0, 0));
            let cache_key = (from_pos, move_.to);
            
            // OPTIMIZATION: Use direct hash lookup instead of HashMap iteration
            if let Some(&cached_value) = self.see_cache.get(&cache_key) {
                self.stats.see_cache_hits += 1;
                self.stats.see_calculation_time_us += start_time.elapsed_ms() as u64 * 1000;
                return Ok(cached_value);
            }
            self.stats.see_cache_misses += 1;
        }

        let see_value = self.calculate_see_internal(move_, board);
        
        // Cache the result if enabled and cache not full
        if self.config.cache_config.enable_see_cache && 
           self.see_cache.len() < self.max_see_cache_size {
            let cache_key = (move_.from.unwrap_or(Position::new(0, 0)), move_.to);
            self.see_cache.insert(cache_key, see_value);
        }

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.see_calculation_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_see_calculation_time_us = 
            self.stats.see_calculation_time_us as f64 / self.stats.see_calculations as f64;

        Ok(see_value)
    }

    /// Internal SEE calculation implementation
    /// 
    /// This method performs the actual SEE calculation by simulating
    /// the exchange sequence.
    fn calculate_see_internal(&self, move_: &Move, board: &crate::bitboards::BitboardBoard) -> i32 {
        let _from = move_.from.unwrap_or(Position::new(0, 0));
        let to = move_.to;
        
        // Get the piece being captured
        let captured_piece = match &move_.captured_piece {
            Some(piece) => piece,
            None => return 0, // No capture, no SEE value
        };
        
        // Start with the value of the captured piece
        let mut gain = captured_piece.piece_type.base_value();
        
        // Find all attackers and defenders of the target square
        let (attackers, defenders) = self.find_attackers_defenders(to, board);
        
        // If no defenders, it's a winning capture
        if defenders.is_empty() {
            return gain;
        }
        
        // Simulate the exchange sequence
        let mut attackers_list = attackers;
        let mut defenders_list = defenders;
        let mut to_move = Player::Black; // Start with attacker
        
        // Continue until no more pieces can capture
        while !attackers_list.is_empty() {
            // Find the least valuable attacker
            let attacker_index = self.find_least_valuable_attacker(&attackers_list, to_move);
            if attacker_index.is_none() {
                break;
            }
            
            let attacker = attackers_list.remove(attacker_index.unwrap());
            
            // If this is the first move, we already accounted for the captured piece
            if attackers_list.len() == defenders_list.len() {
                // This is the original move, already accounted for
                to_move = to_move.opposite();
                continue;
            }
            
            // Add the value of the captured piece (the previous attacker)
            gain += attacker.piece_type.base_value();
            
            // Remove the captured piece from defenders
            if let Some(defender_index) = self.find_piece_in_list(&defenders_list, &attacker) {
                defenders_list.remove(defender_index);
            }
            
            // Switch sides
            to_move = to_move.opposite();
            std::mem::swap(&mut attackers_list, &mut defenders_list);
        }
        
        gain
    }

    /// Find all attackers and defenders of a given square
    /// 
    /// This method identifies all pieces that can attack or defend
    /// the target square, organized by player.
    fn find_attackers_defenders(&self, _square: Position, _board: &crate::bitboards::BitboardBoard) -> (Vec<crate::types::Piece>, Vec<crate::types::Piece>) {
        let attackers = Vec::new();
        let defenders = Vec::new();
        
        // This is a simplified implementation that would need to be
        // integrated with the actual board representation and attack generation
        // For now, we'll return empty vectors as placeholders
        
        // TODO: Implement actual attacker/defender finding using:
        // 1. Board piece lookup
        // 2. Attack pattern generation
        // 3. Ray casting for sliding pieces
        
        (attackers, defenders)
    }

    /// Find the least valuable attacker in a list
    /// 
    /// This method finds the piece with the lowest value that can
    /// participate in the exchange sequence.
    fn find_least_valuable_attacker(&self, attackers: &[crate::types::Piece], player: Player) -> Option<usize> {
        let mut min_value = i32::MAX;
        let mut min_index = None;
        
        for (index, attacker) in attackers.iter().enumerate() {
            if attacker.player == player {
                let value = attacker.piece_type.base_value();
                if value < min_value {
                    min_value = value;
                    min_index = Some(index);
                }
            }
        }
        
        min_index
    }

    /// Find a piece in a list by piece type and player
    fn find_piece_in_list(&self, pieces: &[crate::types::Piece], target: &crate::types::Piece) -> Option<usize> {
        pieces.iter().position(|piece| 
            piece.piece_type == target.piece_type &&
            piece.player == target.player
        )
    }

    /// Score a capture move
    /// 
    /// Captures are generally high-priority moves that should be tried early.
    /// The score is based on the value of the captured piece.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_capture_move(&self, move_: &Move) -> i32 {
        if !move_.is_capture {
            return 0;
        }

        let mut score = self.config.weights.capture_weight;
        
        // Add value of captured piece
        if let Some(captured) = &move_.captured_piece {
            score += captured.piece_type.base_value();
            
            // Bonus for capturing higher-value pieces
            match captured.piece_type {
                PieceType::King => score += 1000,
                PieceType::Rook => score += 500,
                PieceType::Bishop => score += 300,
                PieceType::Gold => score += 200,
                PieceType::Silver => score += 150,
                PieceType::Knight => score += 100,
                PieceType::Lance => score += 80,
                PieceType::Pawn => score += 50,
                // Promoted pieces
                PieceType::PromotedPawn => score += 250,
                PieceType::PromotedLance => score += 230,
                PieceType::PromotedKnight => score += 210,
                PieceType::PromotedSilver => score += 200,
                PieceType::PromotedBishop => score += 350,
                PieceType::PromotedRook => score += 550,
            }
        }

        // Bonus for capturing with lower-value pieces (good exchange)
        match move_.piece_type {
            PieceType::Pawn => score += 100,
            PieceType::Lance => score += 80,
            PieceType::Knight => score += 60,
            PieceType::Silver => score += 40,
            PieceType::Gold => score += 30,
            PieceType::Bishop => score += 20,
            PieceType::Rook => score += 10,
            PieceType::King => score += 5,
            // Promoted pieces
            PieceType::PromotedPawn => score += 110,
            PieceType::PromotedLance => score += 90,
            PieceType::PromotedKnight => score += 70,
            PieceType::PromotedSilver => score += 50,
            PieceType::PromotedBishop => score += 30,
            PieceType::PromotedRook => score += 20,
        }

        score
    }

    /// Score a promotion move
    /// 
    /// Promotions are strategic moves that can significantly change
    /// the value and capabilities of a piece.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_promotion_move(&self, move_: &Move) -> i32 {
        if !move_.is_promotion {
            return 0;
        }

        let mut score = self.config.weights.promotion_weight;
        
        // Add promotion value
        score += move_.promotion_value();
        
        // Bonus for promoting to more valuable pieces
        match move_.piece_type {
            PieceType::Pawn => score += 200, // Pawn to Gold is very valuable
            PieceType::Lance => score += 180,
            PieceType::Knight => score += 160,
            PieceType::Silver => score += 140,
            PieceType::Gold => score += 120,
            PieceType::Bishop => score += 120,
            PieceType::Rook => score += 100,
            PieceType::King => score += 50,
            // Promoted pieces
            PieceType::PromotedPawn => score += 220,
            PieceType::PromotedLance => score += 200,
            PieceType::PromotedKnight => score += 180,
            PieceType::PromotedSilver => score += 160,
            PieceType::PromotedBishop => score += 140,
            PieceType::PromotedRook => score += 120,
        }

        // Bonus for promoting in center or near enemy king
        let center_bonus = self.score_position_value(&move_.to);
        score += center_bonus / 2;

        score
    }

    /// Score a tactical move
    /// 
    /// Tactical moves include checks, threats, and other forcing moves
    /// that can lead to immediate tactical advantages.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_tactical_move(&self, move_: &Move) -> i32 {
        let mut score = 0;

        // Check bonus
        if move_.gives_check {
            score += self.config.weights.tactical_weight;
            
            // Bonus for different types of checks
            match move_.piece_type {
                PieceType::Pawn => score += 50, // Pawn checks are often surprising
                PieceType::Knight => score += 40,
                PieceType::Lance => score += 30,
                PieceType::Silver => score += 25,
                PieceType::Gold => score += 20,
                PieceType::Bishop => score += 35,
                PieceType::Rook => score += 30,
                PieceType::King => score += 10,
                // Promoted pieces
                PieceType::PromotedPawn => score += 55,
                PieceType::PromotedLance => score += 45,
                PieceType::PromotedKnight => score += 35,
                PieceType::PromotedSilver => score += 30,
                PieceType::PromotedBishop => score += 40,
                PieceType::PromotedRook => score += 35,
            }
        }

        // Additional tactical bonuses could be added here
        // when move analysis is implemented

        score
    }

    /// Score piece value
    /// 
    /// Base scoring based on the intrinsic value of the piece being moved.
    /// Generally, more valuable pieces should be moved with more consideration.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_piece_value(&self, move_: &Move) -> i32 {
        let base_value = move_.piece_type.base_value();
        (base_value * self.config.weights.piece_value_weight) / 100
    }

    /// Score position value comprehensively
    /// 
    /// Evaluates the positional value of the move, including center control,
    /// king safety, piece activity, and other positional factors.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_position_value_comprehensive(&self, move_: &Move) -> i32 {
        let mut score = 0;

        // Center control scoring
        score += self.score_position_value(&move_.to) * self.config.weights.position_value_weight / 100;

        // King safety scoring
        score += self.score_king_safety(move_);

        // Piece activity scoring
        score += self.score_piece_activity(move_);

        // Pawn structure scoring
        if move_.piece_type == PieceType::Pawn {
            score += self.score_pawn_structure(move_);
        }

        score
    }

    /// Score development move
    /// 
    /// Evaluates how well the move develops the piece toward better positions,
    /// increases mobility, or improves piece coordination.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_development_move(&self, move_: &Move) -> i32 {
        if let Some(from) = move_.from {
            let mut score = self.score_development_value(from, move_.to) * self.config.weights.development_weight / 100;
            
            // Bonus for moving from back rank (development)
            if from.row <= 2 {
                score += 20;
            }
            
            // Bonus for moving toward center
            let center_distance_from = self.distance_to_center(from);
            let center_distance_to = self.distance_to_center(move_.to);
            if center_distance_to < center_distance_from {
                score += 15;
            }
            
            score
        } else {
            0
        }
    }

    /// Score quiet move
    /// 
    /// Evaluates quiet (non-capturing, non-promoting) moves based on
    /// positional considerations and strategic value.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_quiet_move(&self, move_: &Move) -> i32 {
        if move_.is_capture || move_.is_promotion || move_.gives_check {
            return 0; // Not a quiet move
        }

        let mut score = self.config.weights.quiet_weight;

        // Bonus for improving piece mobility
        score += self.score_mobility_improvement(move_);

        // Bonus for improving piece coordination
        score += self.score_coordination_improvement(move_);

        // Bonus for moves that support other pieces
        score += self.score_support_value(move_);

        score
    }

    /// Score king safety
    /// 
    /// Evaluates how the move affects king safety for both sides.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_king_safety(&self, _move_: &Move) -> i32 {
        // King safety evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Score piece activity
    /// 
    /// Evaluates how the move affects piece activity and mobility.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_piece_activity(&self, _move_: &Move) -> i32 {
        // Piece activity evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Score pawn structure
    /// 
    /// Evaluates how the move affects pawn structure and pawn chains.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_pawn_structure(&self, _move_: &Move) -> i32 {
        // Pawn structure evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Score mobility improvement
    /// 
    /// Evaluates how the move improves piece mobility.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_mobility_improvement(&self, _move_: &Move) -> i32 {
        // Mobility evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Score coordination improvement
    /// 
    /// Evaluates how the move improves piece coordination.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_coordination_improvement(&self, _move_: &Move) -> i32 {
        // Coordination evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Score support value
    /// 
    /// Evaluates how the move supports other pieces or positions.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_support_value(&self, _move_: &Move) -> i32 {
        // Support evaluation could be implemented here
        // when position analysis is available
        0
    }

    /// Calculate distance to center
    /// 
    /// Returns the Manhattan distance from a position to the center of the board.
    #[allow(dead_code)] // Kept for debugging and future use
    fn distance_to_center(&self, position: Position) -> i32 {
        let center_row = 4;
        let center_col = 4;
        (position.row as i32 - center_row).abs() + (position.col as i32 - center_col).abs()
    }

    /// Calculate position value for move scoring
    /// 
    /// Higher values for positions closer to the center of the board,
    /// which are generally more valuable in Shogi.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_position_value(&self, position: &Position) -> i32 {
        let center_row = 4.0; // Middle row of 9x9 board
        let center_col = 4.0; // Middle column of 9x9 board
        
        let row_diff = (position.row as f64 - center_row).abs();
        let col_diff = (position.col as f64 - center_col).abs();
        
        let distance_from_center = row_diff + col_diff;
        
        // Closer to center = higher score
        (8.0 - distance_from_center) as i32
    }

    /// Calculate development value for move scoring
    /// 
    /// Rewards moves that develop pieces toward the center
    /// or improve piece activity.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_development_value(&self, from: Position, to: Position) -> i32 {
        let from_center_dist = self.distance_from_center(from);
        let to_center_dist = self.distance_from_center(to);
        
        // Reward moving closer to center
        if to_center_dist < from_center_dist {
            1
        } else if to_center_dist > from_center_dist {
            -1
        } else {
            0
        }
    }

    /// Calculate distance from center for a position
    #[allow(dead_code)] // Kept for debugging and future use
    fn distance_from_center(&self, position: Position) -> f64 {
        let center_row = 4.0;
        let center_col = 4.0;
        
        let row_diff = position.row as f64 - center_row;
        let col_diff = position.col as f64 - center_col;
        
        (row_diff * row_diff + col_diff * col_diff).sqrt()
    }

    /// Generate a hash for move caching
    #[allow(dead_code)] // Kept for debugging and future use
    fn get_move_hash(&self, move_: &Move) -> u64 {
        let mut hash = 0u64;
        
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.row as u64);
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.col as u64);
        
        if let Some(from) = move_.from {
            hash = hash.wrapping_mul(31).wrapping_add(from.row as u64);
            hash = hash.wrapping_mul(31).wrapping_add(from.col as u64);
        }
        
        hash = hash.wrapping_mul(31).wrapping_add(move_.piece_type as u64);
        hash = hash.wrapping_mul(31).wrapping_add(move_.player as u64);
        
        hash
    }

    /// Fast hash calculation for move caching (optimized hot path)
    /// 
    /// Uses bit manipulation for maximum performance in the hot scoring path.
    fn get_move_hash_fast(&self, move_: &Move) -> u64 {
        // OPTIMIZATION: Use bit manipulation instead of arithmetic operations
        let from = move_.from.map(|pos| pos.to_u8() as u64).unwrap_or(0);
        let to = move_.to.to_u8() as u64;
        let piece_type = move_.piece_type.to_u8() as u64;
        let flags = (move_.is_promotion as u64) | ((move_.is_capture as u64) << 1) | ((move_.gives_check as u64) << 2);
        
        // Combine using bit shifts for maximum performance
        from << 32 | to << 24 | piece_type << 16 | flags << 8
    }

    /// Inline capture move scoring for hot path optimization
    fn score_capture_move_inline(&self, move_: &Move) -> i32 {
        if let Some(captured_piece) = &move_.captured_piece {
            // MVV-LVA: Most Valuable Victim - Least Valuable Attacker
            let victim_value = captured_piece.piece_type.base_value();
            let attacker_value = move_.piece_type.base_value();
            
            // Scale the score based on the exchange value
            let exchange_value = victim_value - attacker_value;
            self.config.weights.capture_weight + exchange_value / 10
        } else {
            0
        }
    }

    /// Inline promotion move scoring for hot path optimization
    fn score_promotion_move_inline(&self, move_: &Move) -> i32 {
        if move_.is_promotion {
            // Base promotion bonus
            let mut score = self.config.weights.promotion_weight;
            
            // Bonus for promoting to center squares
            let center_distance = self.get_center_distance_fast(move_.to);
            if center_distance <= 1 {
                score += 50;
            }
            
            score
        } else {
            0
        }
    }

    /// Fast position value calculation (optimized for hot path)
    fn score_position_value_fast(&self, move_: &Move) -> i32 {
        let mut score = 0i32;
        
        // Center control bonus
        let center_distance = self.get_center_distance_fast(move_.to);
        if center_distance <= 2 {
            score += (3 - center_distance as i32) * 25;
        }
        
        // Edge penalty
        if move_.to.row == 0 || move_.to.row == 8 || move_.to.col == 0 || move_.to.col == 8 {
            score -= 20;
        }
        
        score
    }

    /// Fast development move scoring (optimized for hot path)
    fn score_development_move_fast(&self, move_: &Move) -> i32 {
        // Simple development bonus for moving pieces from starting positions
        if let Some(from) = move_.from {
            // Bonus for moving from starting rank (ranks 0, 1, 7, 8)
            if from.row == 0 || from.row == 1 || from.row == 7 || from.row == 8 {
                return self.config.weights.development_weight / 2;
            }
        }
        
        0
    }

    /// Fast center distance calculation (optimized for hot path)
    fn get_center_distance_fast(&self, pos: Position) -> u8 {
        // Distance from center (4,4) using Manhattan distance
        let dr = if pos.row > 4 { pos.row - 4 } else { 4 - pos.row };
        let dc = if pos.col > 4 { pos.col - 4 } else { 4 - pos.col };
        dr + dc
    }

    /// Update memory usage statistics
    fn update_memory_usage(&mut self) {
        // Calculate current memory usage
        let move_score_cache_memory = self.move_score_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<i32>());
        let pv_cache_memory = self.pv_move_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<Option<Move>>());
        let killer_moves_memory = self.killer_moves.values().map(|moves| moves.len() * std::mem::size_of::<Move>()).sum::<usize>();
        let history_table_memory = self.history_table.len() * (std::mem::size_of::<(PieceType, Position, Position)>() + std::mem::size_of::<u32>());
        let see_cache_memory = self.see_cache.len() * (std::mem::size_of::<(Position, Position)>() + std::mem::size_of::<i32>());
        let struct_memory = std::mem::size_of::<Self>();
        
        self.memory_usage.current_bytes = move_score_cache_memory + pv_cache_memory + killer_moves_memory + history_table_memory + see_cache_memory + struct_memory;
        self.memory_usage.peak_bytes = self.memory_usage.peak_bytes.max(self.memory_usage.current_bytes);
        
        // Update statistics
        self.stats.memory_usage_bytes = self.memory_usage.current_bytes;
        self.stats.peak_memory_usage_bytes = self.memory_usage.peak_bytes;
        
        // Update cache hit rate
        let total_cache_attempts = self.stats.cache_hits + self.stats.cache_misses;
        if total_cache_attempts > 0 {
            self.stats.cache_hit_rate = (self.stats.cache_hits as f64 / total_cache_attempts as f64) * 100.0;
        }
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> &OrderingStats {
        &self.stats
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> &MemoryUsage {
        &self.memory_usage
    }


    /// Set individual heuristic weights for fine-tuning
    pub fn set_capture_weight(&mut self, weight: i32) {
        self.config.weights.capture_weight = weight;
    }

    pub fn set_promotion_weight(&mut self, weight: i32) {
        self.config.weights.promotion_weight = weight;
    }

    pub fn set_center_control_weight(&mut self, weight: i32) {
        self.config.weights.center_control_weight = weight;
    }

    pub fn set_development_weight(&mut self, weight: i32) {
        self.config.weights.development_weight = weight;
    }

    pub fn set_piece_value_weight(&mut self, weight: i32) {
        self.config.weights.piece_value_weight = weight;
    }

    pub fn set_position_value_weight(&mut self, weight: i32) {
        self.config.weights.position_value_weight = weight;
    }

    pub fn set_tactical_weight(&mut self, weight: i32) {
        self.config.weights.tactical_weight = weight;
    }

    pub fn set_quiet_weight(&mut self, weight: i32) {
        self.config.weights.quiet_weight = weight;
    }

    pub fn set_pv_move_weight(&mut self, weight: i32) {
        self.config.weights.pv_move_weight = weight;
    }

    pub fn set_killer_move_weight(&mut self, weight: i32) {
        self.config.weights.killer_move_weight = weight;
    }

    pub fn set_history_weight(&mut self, weight: i32) {
        self.config.weights.history_weight = weight;
    }

    pub fn set_see_weight(&mut self, weight: i32) {
        self.config.weights.see_weight = weight;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &MoveOrderingConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: MoveOrderingConfig) -> Result<(), Vec<String>> {
        // Validate the new configuration
        config.validate()?;
        
        // Update configuration
        self.config = config;
        
        // Apply configuration changes
        self.apply_configuration_changes();
        
        Ok(())
    }

    /// Get current heuristic weights
    pub fn get_weights(&self) -> &OrderingWeights {
        &self.config.weights
    }

    /// Update configuration weights
    pub fn set_weights(&mut self, weights: OrderingWeights) {
        self.config.weights = weights;
    }

    /// Reset configuration to default values
    pub fn reset_config_to_default(&mut self) {
        self.config = MoveOrderingConfig::default();
        self.apply_configuration_changes();
    }

    /// Apply configuration changes to internal state
    fn apply_configuration_changes(&mut self) {
        // Update cache size if needed
        if self.move_score_cache.len() > self.config.cache_config.max_cache_size {
            // Trim cache to new size
            let mut keys_to_remove: Vec<u64> = Vec::new();
            for (i, key) in self.move_score_cache.keys().enumerate() {
                if i >= self.config.cache_config.max_cache_size {
                    keys_to_remove.push(*key);
                }
            }
            for key in keys_to_remove {
                self.move_score_cache.remove(&key);
            }
        }

        // Update killer move limits if needed
        for (_depth, killer_list) in self.killer_moves.iter_mut() {
            if killer_list.len() > self.config.killer_config.max_killer_moves_per_depth {
                killer_list.truncate(self.config.killer_config.max_killer_moves_per_depth);
            }
        }

        // Update history table if needed
        if self.config.history_config.enable_score_clamping {
            for score in self.history_table.values_mut() {
                if *score > self.config.history_config.max_history_score {
                    *score = self.config.history_config.max_history_score;
                }
            }
        }

        self.update_memory_usage();
    }

    /// Optimize move scoring performance
    /// 
    /// This method can be called to optimize the move scoring system
    /// based on current performance statistics.
    pub fn optimize_performance(&mut self) {
        // Adjust cache size based on hit rate
        let hit_rate = self.get_cache_hit_rate();
        if hit_rate > 80.0 && self.config.cache_config.max_cache_size < 5000 {
            self.config.cache_config.max_cache_size = (self.config.cache_config.max_cache_size * 3) / 2; // 1.5x
        } else if hit_rate < 20.0 && self.config.cache_config.max_cache_size > 100 {
            self.config.cache_config.max_cache_size = (self.config.cache_config.max_cache_size * 4) / 5; // 0.8x
        }

        // Clear cache if it's too large and hit rate is low
        if self.move_score_cache.len() > self.config.cache_config.max_cache_size && hit_rate < 30.0 {
            self.move_score_cache.clear();
        }

        self.update_memory_usage();
    }

    /// Get cache hit rate
    /// 
    /// Returns the current cache hit rate percentage.
    pub fn get_cache_hit_rate(&self) -> f64 {
        if self.stats.cache_hits + self.stats.cache_misses > 0 {
            (self.stats.cache_hits as f64 / (self.stats.cache_hits + self.stats.cache_misses) as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Set cache size for performance tuning
    /// 
    /// Adjusts the maximum cache size based on memory constraints
    /// and performance requirements.
    pub fn set_cache_size(&mut self, size: usize) {
        self.config.cache_config.max_cache_size = size;
        
        // Trim cache if it's larger than new size
        if self.move_score_cache.len() > size {
            let mut keys_to_remove: Vec<u64> = Vec::new();
            for (i, key) in self.move_score_cache.keys().enumerate() {
                if i >= size {
                    keys_to_remove.push(*key);
                }
            }
            for key in keys_to_remove {
                self.move_score_cache.remove(&key);
            }
        }
        
        self.update_memory_usage();
    }

    /// Get maximum cache size
    pub fn get_max_cache_size(&self) -> usize {
        self.config.cache_config.max_cache_size
    }

    /// Warm up the cache with common moves
    /// 
    /// This method can be used to pre-populate the cache with
    /// frequently occurring moves to improve performance.
    pub fn warm_up_cache(&mut self, moves: &[Move]) {
        for move_ in moves.iter().take(self.config.cache_config.max_cache_size / 2) {
            let _ = self.score_move(move_);
        }
    }

    /// Get comprehensive move scoring statistics
    /// 
    /// Returns detailed statistics about move scoring performance.
    pub fn get_scoring_stats(&self) -> (u64, u64, f64, u64, usize, usize) {
        (
            self.stats.scoring_operations,
            self.stats.cache_hits,
            self.get_cache_hit_rate(),
            self.stats.cache_misses,
            self.get_cache_size(),
            self.get_max_cache_size()
        )
    }

    /// Clear the move scoring cache
    pub fn clear_cache(&mut self) {
        self.move_score_cache.clear();
        self.pv_move_cache.clear();
        self.killer_moves.clear();
        self.history_table.clear();
        self.stats.cache_hits = 0;
        self.stats.cache_misses = 0;
        self.stats.cache_hit_rate = 0.0;
        self.update_memory_usage();
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.stats = OrderingStats::default();
        self.memory_usage = MemoryUsage::default();
        self.clear_cache();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        (self.stats.cache_hits, self.stats.cache_misses, self.stats.cache_hit_rate)
    }

    /// Set maximum cache size
    pub fn set_max_cache_size(&mut self, max_size: usize) {
        self.config.cache_config.max_cache_size = max_size;
        
        // Trim cache if necessary
        if self.move_score_cache.len() > max_size {
            let excess = self.move_score_cache.len() - max_size;
            let keys_to_remove: Vec<u64> = self.move_score_cache.keys().take(excess).copied().collect();
            for key in keys_to_remove {
                self.move_score_cache.remove(&key);
            }
        }
    }

    /// Get current cache size
    pub fn get_cache_size(&self) -> usize {
        self.move_score_cache.len()
    }

    /// Check if cache is at maximum size
    pub fn is_cache_full(&self) -> bool {
        self.move_score_cache.len() >= self.config.cache_config.max_cache_size
    }

    // ==================== PV Move Ordering Methods ====================

    /// Set the transposition table reference for PV move retrieval
    pub fn set_transposition_table(&mut self, tt: &crate::search::ThreadSafeTranspositionTable) {
        self.transposition_table = tt as *const crate::search::ThreadSafeTranspositionTable;
    }

    /// Score a move that matches the PV move from transposition table
    /// 
    /// PV moves get the highest priority score to ensure they are tried first.
    pub fn score_pv_move(&mut self, _move_: &Move) -> i32 {
        self.config.weights.pv_move_weight
    }

    /// Get the PV move for a given position from the transposition table
    /// 
    /// This method queries the transposition table to find the best move
    /// for the current position and caches the result for performance.
    pub fn get_pv_move(&mut self, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Option<Move> {
        if self.transposition_table.is_null() {
            return None;
        }

        // Calculate position hash
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        
        // Check cache first
        if let Some(cached_move) = self.pv_move_cache.get(&position_hash) {
            if cached_move.is_some() {
                self.stats.pv_move_hits += 1;
            } else {
                self.stats.pv_move_misses += 1;
            }
            return cached_move.clone();
        }

        // Query transposition table
        self.stats.tt_lookups += 1;
        
        // Safe access to transposition table
        let tt_entry = unsafe {
            (*self.transposition_table).probe(position_hash, depth)
        };

        let pv_move = if let Some(entry) = tt_entry {
            self.stats.tt_hits += 1;
            entry.best_move
        } else {
            self.stats.pv_move_misses += 1;
            None
        };

        // Cache the result
        if self.pv_move_cache.len() < self.config.cache_config.max_cache_size {
            self.pv_move_cache.insert(position_hash, pv_move.clone());
        }

        // Update PV move statistics
        if pv_move.is_some() {
            self.stats.pv_move_hits += 1;
        } else {
            self.stats.pv_move_misses += 1;
        }

        // Update PV move hit rate
        let total_pv_attempts = self.stats.pv_move_hits + self.stats.pv_move_misses;
        if total_pv_attempts > 0 {
            self.stats.pv_move_hit_rate = (self.stats.pv_move_hits as f64 / total_pv_attempts as f64) * 100.0;
        }

        pv_move
    }

    /// Update the PV move for a position in the transposition table
    /// 
    /// This method stores the best move found during search back into
    /// the transposition table for future reference.
    pub fn update_pv_move(&mut self, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, best_move: Move, score: i32) {
        if self.transposition_table.is_null() {
            return;
        }

        // Calculate position hash
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        
        // Create transposition table entry
        let entry = TranspositionEntry {
            score,
            depth,
            flag: TranspositionFlag::Exact,
            best_move: Some(best_move.clone()),
            hash_key: position_hash,
            age: 0, // Will be set by the transposition table
        };

        // Store in transposition table
        unsafe {
            if let Some(tt_ref) = self.transposition_table.as_ref() {
                let tt_mut = tt_ref as *const crate::search::ThreadSafeTranspositionTable as *mut crate::search::ThreadSafeTranspositionTable;
                (*tt_mut).store(entry);
            }
        }

        // Update cache
        if self.pv_move_cache.len() < self.config.cache_config.max_cache_size {
            self.pv_move_cache.insert(position_hash, Some(best_move));
        }
    }

    /// Clear the PV move cache
    /// 
    /// This method clears all cached PV moves, typically called
    /// when starting a new search or when memory needs to be freed.
    pub fn clear_pv_move_cache(&mut self) {
        self.pv_move_cache.clear();
        self.stats.pv_move_hits = 0;
        self.stats.pv_move_misses = 0;
        self.stats.pv_move_hit_rate = 0.0;
        self.stats.tt_lookups = 0;
        self.stats.tt_hits = 0;
    }

    /// Check if a move matches the PV move for a position
    /// 
    /// This method determines if a given move is the PV move
    /// stored in the transposition table for the current position.
    pub fn is_pv_move(&mut self, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, move_: &Move) -> bool {
        if let Some(pv_move) = self.get_pv_move(board, captured_pieces, player, depth) {
            self.moves_equal(&pv_move, move_)
        } else {
            false
        }
    }

    /// Compare two moves for equality
    /// 
    /// This is a helper method to check if two moves are the same,
    /// used for PV move matching.
    fn moves_equal(&self, a: &Move, b: &Move) -> bool {
        a.from == b.from && 
        a.to == b.to && 
        a.piece_type == b.piece_type && 
        a.player == b.player &&
        a.is_promotion == b.is_promotion
    }

    /// Order moves with PV move prioritization
    /// 
    /// This enhanced version of order_moves prioritizes PV moves from
    /// the transposition table, giving them the highest priority.
    pub fn order_moves_with_pv(&mut self, moves: &[Move], board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Get PV move for this position
        let pv_move = self.get_pv_move(board, captured_pieces, player, depth);

        // Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Sort moves by score with PV move prioritization
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_pv(a, &pv_move);
            let score_b = self.score_move_with_pv(b, &pv_move);
            score_b.cmp(&score_a)
        });

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        ordered_moves
    }

    /// Score a move with PV move consideration
    /// 
    /// This method scores a move, giving highest priority to PV moves
    /// and falling back to regular move scoring for other moves.
    fn score_move_with_pv(&mut self, move_: &Move, pv_move: &Option<Move>) -> i32 {
        // Check if this is the PV move
        if let Some(ref pv) = pv_move {
            if self.moves_equal(move_, pv) {
                return self.score_pv_move(move_);
            }
        }

        // Use regular move scoring
        self.score_move(move_).unwrap_or(0)
    }

    /// Get PV move statistics
    /// 
    /// Returns statistics about PV move usage and effectiveness.
    pub fn get_pv_stats(&self) -> (u64, u64, f64, u64, u64) {
        (
            self.stats.pv_move_hits,
            self.stats.pv_move_misses,
            self.stats.pv_move_hit_rate,
            self.stats.tt_lookups,
            self.stats.tt_hits
        )
    }

    /// Get transposition table hit rate
    /// 
    /// Returns the hit rate for transposition table lookups.
    pub fn get_tt_hit_rate(&self) -> f64 {
        if self.stats.tt_lookups > 0 {
            (self.stats.tt_hits as f64 / self.stats.tt_lookups as f64) * 100.0
        } else {
            0.0
        }
    }

    // ==================== Killer Move Heuristic Methods ====================

    /// Set the current search depth for killer move management
    /// 
    /// This method should be called at the beginning of each search depth
    /// to ensure killer moves are properly organized by depth.
    pub fn set_current_depth(&mut self, depth: u8) {
        self.current_depth = depth;
    }

    /// Get the current search depth
    pub fn get_current_depth(&self) -> u8 {
        self.current_depth
    }

    /// Score a move that matches a killer move
    /// 
    /// Killer moves get high priority to encourage trying moves that
    /// caused beta cutoffs in previous searches at the same depth.
    pub fn score_killer_move(&mut self, _move_: &Move) -> i32 {
        self.config.weights.killer_move_weight
    }

    /// Add a killer move for the current depth
    /// 
    /// This method stores a move that caused a beta cutoff, making it
    /// a candidate for early consideration in future searches at the same depth.
    pub fn add_killer_move(&mut self, move_: Move) {
        let depth = self.current_depth;
        
        // Check if this move is already a killer move at this depth
        let is_duplicate = if let Some(killer_list) = self.killer_moves.get(&depth) {
            killer_list.iter().any(|killer| self.moves_equal(killer, &move_))
        } else {
            false
        };
        
        if !is_duplicate {
            // Get or create the killer moves list for this depth
            let killer_list = self.killer_moves.entry(depth).or_insert_with(Vec::new);
            
            // Add the new killer move
            killer_list.push(move_);
            self.stats.killer_moves_stored += 1;
            
                    // Limit the number of killer moves per depth
                    if killer_list.len() > self.config.killer_config.max_killer_moves_per_depth {
                        killer_list.remove(0); // Remove oldest killer move
                    }
        }
        
        self.update_memory_usage();
    }

    /// Check if a move is a killer move at the current depth
    /// 
    /// This method determines if a given move is stored as a killer move
    /// for the current search depth.
    pub fn is_killer_move(&mut self, move_: &Move) -> bool {
        let depth = self.current_depth;
        
        if let Some(killer_list) = self.killer_moves.get(&depth) {
            killer_list.iter().any(|killer| self.moves_equal(killer, move_))
        } else {
            false
        }
    }

    /// Get all killer moves for a specific depth
    /// 
    /// Returns a reference to the killer moves list for the given depth,
    /// or None if no killer moves exist for that depth.
    pub fn get_killer_moves(&self, depth: u8) -> Option<&Vec<Move>> {
        self.killer_moves.get(&depth)
    }

    /// Get all killer moves for the current depth
    /// 
    /// Returns a reference to the killer moves list for the current depth,
    /// or None if no killer moves exist for the current depth.
    pub fn get_current_killer_moves(&self) -> Option<&Vec<Move>> {
        self.get_killer_moves(self.current_depth)
    }

    /// Clear killer moves for a specific depth
    /// 
    /// This method removes all killer moves stored for the given depth.
    pub fn clear_killer_moves_for_depth(&mut self, depth: u8) {
        if let Some(killer_list) = self.killer_moves.get_mut(&depth) {
            killer_list.clear();
        }
        self.update_memory_usage();
    }

    /// Clear all killer moves
    /// 
    /// This method removes all killer moves from all depths.
    pub fn clear_all_killer_moves(&mut self) {
        self.killer_moves.clear();
        self.stats.killer_move_hits = 0;
        self.stats.killer_move_misses = 0;
        self.stats.killer_move_hit_rate = 0.0;
        self.stats.killer_moves_stored = 0;
        self.update_memory_usage();
    }

    /// Set the maximum number of killer moves per depth
    /// 
    /// This method allows configuration of how many killer moves
    /// are stored for each search depth.
    pub fn set_max_killer_moves_per_depth(&mut self, max_moves: usize) {
        self.config.killer_config.max_killer_moves_per_depth = max_moves;
        
        // Trim existing killer move lists if necessary
        for killer_list in self.killer_moves.values_mut() {
            if killer_list.len() > max_moves {
                killer_list.truncate(max_moves);
            }
        }
        
        self.update_memory_usage();
    }

    /// Get the maximum number of killer moves per depth
    pub fn get_max_killer_moves_per_depth(&self) -> usize {
        self.config.killer_config.max_killer_moves_per_depth
    }

    /// Get killer move statistics
    /// 
    /// Returns statistics about killer move usage and effectiveness.
    pub fn get_killer_move_stats(&self) -> (u64, u64, f64, u64) {
        (
            self.stats.killer_move_hits,
            self.stats.killer_move_misses,
            self.stats.killer_move_hit_rate,
            self.stats.killer_moves_stored
        )
    }

    /// Get killer move hit rate
    /// 
    /// Returns the hit rate for killer move lookups.
    pub fn get_killer_move_hit_rate(&self) -> f64 {
        if self.stats.killer_move_hits + self.stats.killer_move_misses > 0 {
            (self.stats.killer_move_hits as f64 / (self.stats.killer_move_hits + self.stats.killer_move_misses) as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Order moves with killer move prioritization
    /// 
    /// This enhanced version of order_moves prioritizes killer moves
    /// from the current search depth, giving them high priority.
    pub fn order_moves_with_killer(&mut self, moves: &[Move]) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Get killer moves for current depth
        let killer_moves = self.get_current_killer_moves().cloned().unwrap_or_default();

        // Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Sort moves by score with killer move prioritization
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_killer(a, &killer_moves);
            let score_b = self.score_move_with_killer(b, &killer_moves);
            score_b.cmp(&score_a)
        });

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        ordered_moves
    }

    /// Score a move with killer move consideration
    /// 
    /// This method scores a move, giving high priority to killer moves
    /// and falling back to regular move scoring for other moves.
    fn score_move_with_killer(&mut self, move_: &Move, killer_moves: &[Move]) -> i32 {
        // Check if this is a killer move
        if killer_moves.iter().any(|killer| self.moves_equal(move_, killer)) {
            self.stats.killer_move_hits += 1;
            return self.score_killer_move(move_);
        }

        // Use regular move scoring
        self.stats.killer_move_misses += 1;
        self.score_move(move_).unwrap_or(0)
    }

    /// Order moves with both PV and killer move prioritization
    /// 
    /// This method combines PV move and killer move prioritization
    /// for optimal move ordering.
    pub fn order_moves_with_pv_and_killer(&mut self, moves: &[Move], board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Set current depth for killer move management
        self.set_current_depth(depth);

        // Get PV move for this position
        let pv_move = self.get_pv_move(board, captured_pieces, player, depth);

        // Get killer moves for current depth
        let killer_moves = self.get_current_killer_moves().cloned().unwrap_or_default();

        // Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Sort moves by score with PV and killer move prioritization
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_pv_and_killer(a, &pv_move, &killer_moves);
            let score_b = self.score_move_with_pv_and_killer(b, &pv_move, &killer_moves);
            score_b.cmp(&score_a)
        });

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        ordered_moves
    }

    /// Score a move with both PV and killer move consideration
    /// 
    /// This method scores a move with the following priority:
    /// 1. PV moves (highest priority)
    /// 2. Killer moves (high priority)
    /// 3. Regular moves (normal priority)
    fn score_move_with_pv_and_killer(&mut self, move_: &Move, pv_move: &Option<Move>, killer_moves: &[Move]) -> i32 {
        // Check if this is the PV move (highest priority)
        if let Some(ref pv) = pv_move {
            if self.moves_equal(move_, pv) {
                return self.score_pv_move(move_);
            }
        }

        // Check if this is a killer move (high priority)
        if killer_moves.iter().any(|killer| self.moves_equal(move_, killer)) {
            self.stats.killer_move_hits += 1;
            return self.score_killer_move(move_);
        }

        // Use regular move scoring
        self.stats.killer_move_misses += 1;
        self.score_move(move_).unwrap_or(0)
    }

    // ==================== History Heuristic Methods ====================

    /// Score a move using history heuristic
    /// 
    /// Returns a score based on how often this move has been successful
    /// in previous searches.
    pub fn score_history_move(&mut self, move_: &Move) -> i32 {
        if let Some(from) = move_.from {
            let key = (move_.piece_type, from, move_.to);
            if let Some(&history_score) = self.history_table.get(&key) {
                self.stats.history_hits += 1;
                // Scale history score to match other weights
                (history_score as i32 * self.config.weights.history_weight) / 1000
            } else {
                self.stats.history_misses += 1;
                0
            }
        } else {
            self.stats.history_misses += 1;
            0
        }
    }

    /// Update history score for a move
    /// 
    /// This method should be called when a move causes a cutoff or
    /// improves the alpha bound during search.
    pub fn update_history_score(&mut self, move_: &Move, depth: u8) {
        if let Some(from) = move_.from {
            let key = (move_.piece_type, from, move_.to);
            let bonus = (depth * depth) as u32; // Bonus proportional to depth
            
            let current_score = self.history_table.get(&key).copied().unwrap_or(0);
            let new_score = current_score + bonus;
            
            // Prevent overflow
            let final_score = new_score.min(self.config.history_config.max_history_score);
            self.history_table.insert(key, final_score);
            
            self.stats.history_updates += 1;
            self.history_update_counter += 1;
            
            // Check if automatic aging should be performed
            if self.config.history_config.enable_automatic_aging {
                if self.history_update_counter % self.config.history_config.aging_frequency == 0 {
                    self.age_history_table();
                }
            }
            
            self.update_memory_usage();
        }
    }

    /// Get history score for a move
    /// 
    /// Returns the current history score for the given move, or 0 if not found.
    pub fn get_history_score(&mut self, move_: &Move) -> u32 {
        if let Some(from) = move_.from {
            let key = (move_.piece_type, from, move_.to);
            self.history_table.get(&key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    /// Age the history table to prevent overflow
    /// 
    /// This method reduces all history scores by the aging factor,
    /// helping to prevent overflow and giving more weight to recent moves.
    pub fn age_history_table(&mut self) {
        if self.history_table.is_empty() {
            return;
        }

        let aging_factor = self.config.history_config.history_aging_factor;
        let mut entries_to_remove = Vec::new();
        
        for (key, score) in self.history_table.iter_mut() {
            *score = (*score as f32 * aging_factor) as u32;
            if *score == 0 {
                entries_to_remove.push(*key);
            }
        }
        
        // Remove entries with zero scores
        for key in entries_to_remove {
            self.history_table.remove(&key);
        }
        
        self.stats.history_aging_operations += 1;
        self.update_memory_usage();
    }

    /// Clear the history table
    /// 
    /// This method removes all history entries and resets statistics.
    pub fn clear_history_table(&mut self) {
        self.history_table.clear();
        self.stats.history_hits = 0;
        self.stats.history_misses = 0;
        self.stats.history_hit_rate = 0.0;
        self.stats.history_updates = 0;
        self.stats.history_aging_operations = 0;
        self.update_memory_usage();
    }

    /// Set the maximum history score
    /// 
    /// This method configures the maximum value for history scores
    /// to prevent overflow.
    pub fn set_max_history_score(&mut self, max_score: u32) {
        self.config.history_config.max_history_score = max_score;
        
        // Trim any existing scores that exceed the new limit
        for score in self.history_table.values_mut() {
            if *score > max_score {
                *score = max_score;
            }
        }
    }

    /// Get the maximum history score
    pub fn get_max_history_score(&self) -> u32 {
        self.config.history_config.max_history_score
    }

    /// Set the history aging factor
    /// 
    /// This method configures how much history scores are reduced
    /// during aging operations (0.0 to 1.0).
    pub fn set_history_aging_factor(&mut self, factor: f32) {
        self.config.history_config.history_aging_factor = factor.clamp(0.0, 1.0);
    }

    /// Get the history aging factor
    pub fn get_history_aging_factor(&self) -> f32 {
        self.config.history_config.history_aging_factor
    }

    /// Get history heuristic statistics
    /// 
    /// Returns comprehensive statistics about history heuristic usage.
    pub fn get_history_stats(&self) -> (u64, u64, f64, u64, u64) {
        (
            self.stats.history_hits,
            self.stats.history_misses,
            self.stats.history_hit_rate,
            self.stats.history_updates,
            self.stats.history_aging_operations
        )
    }

    /// Get history heuristic hit rate
    /// 
    /// Returns the hit rate for history heuristic lookups.
    pub fn get_history_hit_rate(&self) -> f64 {
        if self.stats.history_hits + self.stats.history_misses > 0 {
            (self.stats.history_hits as f64 / (self.stats.history_hits + self.stats.history_misses) as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get history update counter
    /// 
    /// Returns the current value of the history update counter,
    /// which is used for automatic aging.
    pub fn get_history_update_counter(&self) -> u64 {
        self.history_update_counter
    }

    /// Reset history update counter
    /// 
    /// Resets the history update counter to zero.
    /// This is useful for testing or when you want to reset
    /// the automatic aging cycle.
    pub fn reset_history_update_counter(&mut self) {
        self.history_update_counter = 0;
    }

    // ==================== SEE Cache Management ====================

    /// Clear the SEE cache
    /// 
    /// Removes all entries from the SEE cache, freeing memory
    /// and resetting cache statistics.
    pub fn clear_see_cache(&mut self) {
        self.see_cache.clear();
        self.stats.see_cache_hits = 0;
        self.stats.see_cache_misses = 0;
        self.update_memory_usage();
    }

    /// Get SEE cache size
    /// 
    /// Returns the current number of entries in the SEE cache.
    pub fn get_see_cache_size(&self) -> usize {
        self.see_cache.len()
    }

    /// Get SEE cache hit rate
    /// 
    /// Returns the current SEE cache hit rate percentage.
    pub fn get_see_cache_hit_rate(&self) -> f64 {
        let total_attempts = self.stats.see_cache_hits + self.stats.see_cache_misses;
        if total_attempts > 0 {
            (self.stats.see_cache_hits as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Set maximum SEE cache size
    /// 
    /// Adjusts the maximum number of entries in the SEE cache.
    /// If the current cache is larger than the new size, it will be trimmed.
    pub fn set_max_see_cache_size(&mut self, max_size: usize) {
        self.max_see_cache_size = max_size;
        self.config.cache_config.max_see_cache_size = max_size;
        
        // Trim cache if necessary
        if self.see_cache.len() > max_size {
            let excess = self.see_cache.len() - max_size;
            let keys_to_remove: Vec<(Position, Position)> = 
                self.see_cache.keys().take(excess).copied().collect();
            
            for key in keys_to_remove {
                self.see_cache.remove(&key);
            }
        }
        
        self.update_memory_usage();
    }

    /// Get SEE statistics
    /// 
    /// Returns comprehensive statistics about SEE calculation performance.
    pub fn get_see_stats(&self) -> (u64, u64, u64, f64, u64, f64) {
        (
            self.stats.see_calculations,
            self.stats.see_cache_hits,
            self.stats.see_cache_misses,
            self.get_see_cache_hit_rate(),
            self.stats.see_calculation_time_us,
            self.stats.avg_see_calculation_time_us,
        )
    }

    /// Enable or disable SEE cache
    /// 
    /// Controls whether SEE results are cached for performance optimization.
    pub fn set_see_cache_enabled(&mut self, enabled: bool) {
        self.config.cache_config.enable_see_cache = enabled;
        if !enabled {
            self.clear_see_cache();
        }
    }

    // ==================== Performance Benchmarking ====================

    /// Benchmark move scoring performance
    /// 
    /// Returns timing statistics for move scoring operations.
    pub fn benchmark_move_scoring(&mut self, moves: &[Move], iterations: usize) -> (u64, f64) {
        let start_time = TimeSource::now();
        
        for _ in 0..iterations {
            for move_ in moves {
                self.score_move(move_).unwrap_or(0);
            }
        }
        
        let total_time = start_time.elapsed_ms() as u64 * 1000;
        let avg_time_per_move = total_time as f64 / (moves.len() * iterations) as f64;
        
        (total_time, avg_time_per_move)
    }

    /// Benchmark move ordering performance
    /// 
    /// Returns timing statistics for complete move ordering operations.
    pub fn benchmark_move_ordering(&mut self, moves: &[Move], iterations: usize) -> (u64, f64) {
        let start_time = TimeSource::now();
        
        for _ in 0..iterations {
            let _ = self.order_moves(moves);
        }
        
        let total_time = start_time.elapsed_ms() as u64 * 1000;
        let avg_time_per_ordering = total_time as f64 / iterations as f64;
        
        (total_time, avg_time_per_ordering)
    }

    /// Benchmark cache performance
    /// 
    /// Returns cache hit rates and timing for cache operations.
    pub fn benchmark_cache_performance(&mut self, moves: &[Move], iterations: usize) -> (f64, u64) {
        let initial_hits = self.stats.cache_hits;
        let initial_misses = self.stats.cache_misses;
        
        let start_time = TimeSource::now();
        
        for _ in 0..iterations {
            for move_ in moves {
                self.score_move(move_).unwrap_or(0);
            }
        }
        
        let total_time = start_time.elapsed_ms() as u64 * 1000;
        
        let new_hits = self.stats.cache_hits - initial_hits;
        let new_misses = self.stats.cache_misses - initial_misses;
        let total_attempts = new_hits + new_misses;
        
        let hit_rate = if total_attempts > 0 {
            (new_hits as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };
        
        (hit_rate, total_time)
    }

    /// Get comprehensive performance statistics
    /// 
    /// Returns all performance metrics for analysis and optimization.
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            total_moves_ordered: self.stats.total_moves_ordered,
            avg_ordering_time_us: self.stats.avg_ordering_time_us,
            cache_hit_rate: self.stats.cache_hit_rate,
            see_cache_hit_rate: self.get_see_cache_hit_rate(),
            hot_path_stats: self.stats.hot_path_stats.clone(),
            memory_usage: self.memory_usage.clone(),
            cache_sizes: CacheSizes {
                move_score_cache: self.move_score_cache.len(),
                fast_cache: self.fast_score_cache.len(),
                pv_cache: self.pv_move_cache.len(),
                see_cache: self.see_cache.len(),
                history_table: self.history_table.len(),
            },
        }
    }

    // ==================== Statistics Export ====================

    /// Export comprehensive statistics to JSON format
    /// 
    /// Returns a JSON string containing all performance statistics for analysis.
    pub fn export_statistics_json(&self) -> String {
        let export_data = StatisticsExport {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            ordering_stats: self.stats.clone(),
            config: self.config.clone(),
            memory_usage: self.memory_usage.clone(),
            cache_sizes: CacheSizes {
                move_score_cache: self.move_score_cache.len(),
                fast_cache: self.fast_score_cache.len(),
                pv_cache: self.pv_move_cache.len(),
                see_cache: self.see_cache.len(),
                history_table: self.history_table.len(),
            },
        };

        serde_json::to_string_pretty(&export_data).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export statistics to CSV format for spreadsheet analysis
    /// 
    /// Returns CSV data with key performance metrics.
    pub fn export_statistics_csv(&self) -> String {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Metric,Value,Unit\n");
        
        // Basic statistics
        csv.push_str(&format!("Total Moves Ordered,{},\n", self.stats.total_moves_ordered));
        csv.push_str(&format!("Average Ordering Time,{:.2},microseconds\n", self.stats.avg_ordering_time_us));
        csv.push_str(&format!("Cache Hit Rate,{:.2},percent\n", self.stats.cache_hit_rate));
        csv.push_str(&format!("SEE Cache Hit Rate,{:.2},percent\n", self.get_see_cache_hit_rate()));
        
        // Heuristic statistics
        csv.push_str(&format!("Capture Applications,{},\n", self.stats.heuristic_stats.capture_stats.applications));
        csv.push_str(&format!("Promotion Applications,{},\n", self.stats.heuristic_stats.promotion_stats.applications));
        csv.push_str(&format!("Tactical Applications,{},\n", self.stats.heuristic_stats.tactical_stats.applications));
        
        // Memory statistics
        csv.push_str(&format!("Current Memory Usage,{:.2},MB\n", 
            self.memory_usage.current_bytes as f64 / 1_000_000.0));
        csv.push_str(&format!("Peak Memory Usage,{:.2},MB\n", 
            self.memory_usage.peak_bytes as f64 / 1_000_000.0));
        
        csv
    }

    /// Export performance summary for quick analysis
    /// 
    /// Returns a concise summary of key performance metrics.
    pub fn export_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_moves_ordered: self.stats.total_moves_ordered,
            avg_ordering_time_us: self.stats.avg_ordering_time_us,
            cache_hit_rate: self.stats.cache_hit_rate,
            see_cache_hit_rate: self.get_see_cache_hit_rate(),
            memory_usage_mb: self.memory_usage.current_bytes as f64 / 1_000_000.0,
            peak_memory_mb: self.memory_usage.peak_bytes as f64 / 1_000_000.0,
            most_effective_heuristic: self.get_most_effective_heuristic(),
            performance_score: self.calculate_performance_score(),
            bottleneck_count: self.profile_bottlenecks().bottlenecks.len(),
        }
    }

    /// Get the most effective heuristic based on best move contributions
    fn get_most_effective_heuristic(&self) -> String {
        let heuristics = [
            ("capture", self.stats.heuristic_stats.capture_stats.best_move_contributions),
            ("promotion", self.stats.heuristic_stats.promotion_stats.best_move_contributions),
            ("tactical", self.stats.heuristic_stats.tactical_stats.best_move_contributions),
            ("piece_value", self.stats.heuristic_stats.piece_value_stats.best_move_contributions),
            ("position", self.stats.heuristic_stats.position_stats.best_move_contributions),
            ("development", self.stats.heuristic_stats.development_stats.best_move_contributions),
            ("quiet", self.stats.heuristic_stats.quiet_stats.best_move_contributions),
            ("pv", self.stats.heuristic_stats.pv_stats.best_move_contributions),
            ("killer", self.stats.heuristic_stats.killer_stats.best_move_contributions),
            ("history", self.stats.heuristic_stats.history_stats.best_move_contributions),
            ("see", self.stats.heuristic_stats.see_stats.best_move_contributions),
        ];

        heuristics.iter()
            .max_by_key(|(_, contributions)| contributions)
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| "none".to_string())
    }

    // ==================== Statistics Visualization ====================

    /// Generate a text-based performance report
    /// 
    /// Returns a formatted string with performance statistics for console display.
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        
        // Header
        report.push_str("=== MOVE ORDERING PERFORMANCE REPORT ===\n\n");
        
        // Overall Statistics
        report.push_str("OVERALL PERFORMANCE:\n");
        report.push_str(&format!("  Total Moves Ordered: {}\n", self.stats.total_moves_ordered));
        report.push_str(&format!("  Average Ordering Time: {:.2} μs\n", self.stats.avg_ordering_time_us));
        report.push_str(&format!("  Performance Score: {}/100\n", self.calculate_performance_score()));
        report.push_str("\n");
        
        // Cache Performance
        report.push_str("CACHE PERFORMANCE:\n");
        report.push_str(&format!("  Move Score Cache Hit Rate: {:.1}%\n", self.stats.cache_stats.move_score_cache.hit_rate));
        report.push_str(&format!("  Fast Cache Hit Rate: {:.1}%\n", self.stats.cache_stats.fast_cache.hit_rate));
        report.push_str(&format!("  PV Cache Hit Rate: {:.1}%\n", self.stats.cache_stats.pv_cache.hit_rate));
        report.push_str(&format!("  SEE Cache Hit Rate: {:.1}%\n", self.get_see_cache_hit_rate()));
        report.push_str("\n");
        
        // Memory Usage
        report.push_str("MEMORY USAGE:\n");
        report.push_str(&format!("  Current: {:.2} MB\n", self.memory_usage.current_bytes as f64 / 1_000_000.0));
        report.push_str(&format!("  Peak: {:.2} MB\n", self.memory_usage.peak_bytes as f64 / 1_000_000.0));
        report.push_str("\n");
        
        // Heuristic Effectiveness
        report.push_str("HEURISTIC EFFECTIVENESS:\n");
        let heuristics = [
            ("Capture", self.stats.heuristic_stats.capture_stats.applications, self.stats.heuristic_stats.capture_stats.best_move_contributions),
            ("Promotion", self.stats.heuristic_stats.promotion_stats.applications, self.stats.heuristic_stats.promotion_stats.best_move_contributions),
            ("Tactical", self.stats.heuristic_stats.tactical_stats.applications, self.stats.heuristic_stats.tactical_stats.best_move_contributions),
            ("PV", self.stats.heuristic_stats.pv_stats.applications, self.stats.heuristic_stats.pv_stats.best_move_contributions),
            ("Killer", self.stats.heuristic_stats.killer_stats.applications, self.stats.heuristic_stats.killer_stats.best_move_contributions),
            ("History", self.stats.heuristic_stats.history_stats.applications, self.stats.heuristic_stats.history_stats.best_move_contributions),
        ];
        
        for (name, applications, contributions) in heuristics {
            let effectiveness = if applications > 0 { (contributions as f64 / applications as f64) * 100.0 } else { 0.0 };
            report.push_str(&format!("  {}: {:.1}% effective ({} contributions / {} applications)\n", 
                name, effectiveness, contributions, applications));
        }
        report.push_str("\n");
        
        // Bottleneck Analysis
        let analysis = self.profile_bottlenecks();
        if !analysis.bottlenecks.is_empty() {
            report.push_str("IDENTIFIED BOTTLENECKS:\n");
            for bottleneck in &analysis.bottlenecks {
                report.push_str(&format!("  [{}] {}: {}\n", 
                    match bottleneck.severity {
                        BottleneckSeverity::Critical => "CRITICAL",
                        BottleneckSeverity::High => "HIGH",
                        BottleneckSeverity::Medium => "MEDIUM",
                        BottleneckSeverity::Low => "LOW",
                    },
                    bottleneck.description,
                    bottleneck.recommendation));
            }
        } else {
            report.push_str("No significant bottlenecks identified.\n");
        }
        
        report
    }

    /// Generate a performance chart data for visualization
    /// 
    /// Returns data suitable for creating charts and graphs.
    pub fn generate_performance_chart_data(&self) -> PerformanceChartData {
        PerformanceChartData {
            cache_hit_rates: CacheHitRates {
                move_score_cache: self.stats.cache_stats.move_score_cache.hit_rate,
                fast_cache: self.stats.cache_stats.fast_cache.hit_rate,
                pv_cache: self.stats.cache_stats.pv_cache.hit_rate,
                see_cache: self.get_see_cache_hit_rate(),
            },
            heuristic_effectiveness: HeuristicEffectiveness {
                capture: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.capture_stats),
                promotion: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.promotion_stats),
                tactical: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.tactical_stats),
                pv: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.pv_stats),
                killer: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.killer_stats),
                history: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.history_stats),
            },
            memory_usage_trend: MemoryUsageTrend {
                current_mb: self.memory_usage.current_bytes as f64 / 1_000_000.0,
                peak_mb: self.memory_usage.peak_bytes as f64 / 1_000_000.0,
                allocation_count: self.stats.memory_stats.allocation_stats.total_allocations,
            },
            timing_breakdown: TimingBreakdown {
                move_scoring_avg_us: self.stats.timing_stats.move_scoring_times.avg_time_us,
                move_ordering_avg_us: self.stats.timing_stats.move_ordering_times.avg_time_us,
                cache_avg_us: self.stats.timing_stats.cache_times.avg_time_us,
                hash_avg_us: self.stats.timing_stats.hash_times.avg_time_us,
            },
        }
    }

    /// Calculate heuristic effectiveness percentage
    fn calculate_heuristic_effectiveness(&self, stats: &HeuristicPerformance) -> f64 {
        if stats.applications > 0 {
            (stats.best_move_contributions as f64 / stats.applications as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Profile and identify performance bottlenecks
    /// 
    /// Analyzes the current performance statistics to identify optimization opportunities.
    pub fn profile_bottlenecks(&self) -> BottleneckAnalysis {
        let mut bottlenecks = Vec::new();
        
        // Analyze cache performance
        if self.stats.cache_hit_rate < 50.0 {
            bottlenecks.push(Bottleneck {
                category: BottleneckCategory::Cache,
                severity: BottleneckSeverity::High,
                description: format!("Low cache hit rate: {:.1}%", self.stats.cache_hit_rate),
                recommendation: "Consider increasing cache size or improving cache key generation".to_string(),
            });
        }
        
        // Analyze hot path performance
        if self.stats.hot_path_stats.score_move_calls > 0 {
            let avg_score_time = self.stats.hot_path_stats.score_move_time_us as f64 / 
                               self.stats.hot_path_stats.score_move_calls as f64;
            
            if avg_score_time > 100.0 { // More than 100 microseconds per score
                bottlenecks.push(Bottleneck {
                    category: BottleneckCategory::HotPath,
                    severity: BottleneckSeverity::Medium,
                    description: format!("Slow move scoring: {:.1}μs per move", avg_score_time),
                    recommendation: "Consider inlining more scoring functions or optimizing hash calculation".to_string(),
                });
            }
        }
        
        // Analyze memory usage
        if self.memory_usage.current_bytes > 10_000_000 { // More than 10MB
            bottlenecks.push(Bottleneck {
                category: BottleneckCategory::Memory,
                severity: BottleneckSeverity::Medium,
                description: format!("High memory usage: {:.1}MB", 
                                   self.memory_usage.current_bytes as f64 / 1_000_000.0),
                recommendation: "Consider reducing cache sizes or implementing cache aging".to_string(),
            });
        }
        
        // Analyze SEE cache performance
        if self.stats.see_calculations > 0 {
            let see_hit_rate = self.get_see_cache_hit_rate();
            if see_hit_rate < 30.0 {
                bottlenecks.push(Bottleneck {
                    category: BottleneckCategory::SEECache,
                    severity: BottleneckSeverity::Low,
                    description: format!("Low SEE cache hit rate: {:.1}%", see_hit_rate),
                    recommendation: "Consider enabling SEE cache or increasing SEE cache size".to_string(),
                });
            }
        }
        
        BottleneckAnalysis {
            bottlenecks,
            overall_score: self.calculate_performance_score(),
        }
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(&self) -> u8 {
        let mut score = 100u8;
        
        // Deduct points for poor cache performance
        if self.stats.cache_hit_rate < 50.0 {
            score = score.saturating_sub(20);
        } else if self.stats.cache_hit_rate < 70.0 {
            score = score.saturating_sub(10);
        }
        
        // Deduct points for slow hot path
        if self.stats.hot_path_stats.score_move_calls > 0 {
            let avg_score_time = self.stats.hot_path_stats.score_move_time_us as f64 / 
                               self.stats.hot_path_stats.score_move_calls as f64;
            if avg_score_time > 100.0 {
                score = score.saturating_sub(15);
            } else if avg_score_time > 50.0 {
                score = score.saturating_sub(8);
            }
        }
        
        // Deduct points for high memory usage
        if self.memory_usage.current_bytes > 10_000_000 {
            score = score.saturating_sub(10);
        } else if self.memory_usage.current_bytes > 5_000_000 {
            score = score.saturating_sub(5);
        }
        
        score
    }

    // ==================== Performance Trend Analysis ====================

    /// Analyze performance trends over time
    /// 
    /// Returns trend analysis data for identifying performance patterns.
    pub fn analyze_performance_trends(&self) -> PerformanceTrendAnalysis {
        PerformanceTrendAnalysis {
            cache_efficiency_trend: self.analyze_cache_efficiency_trend(),
            memory_usage_trend: self.analyze_memory_usage_trend(),
            heuristic_effectiveness_trend: self.analyze_heuristic_effectiveness_trend(),
            timing_trend: self.analyze_timing_trend(),
            overall_performance_trend: self.analyze_overall_performance_trend(),
        }
    }

    /// Analyze cache efficiency trends
    fn analyze_cache_efficiency_trend(&self) -> TrendAnalysis {
        let current_hit_rate = self.stats.cache_hit_rate;
        let see_hit_rate = self.get_see_cache_hit_rate();
        
        // Simple trend analysis based on current performance
        let trend_direction = if current_hit_rate > 70.0 && see_hit_rate > 50.0 {
            TrendDirection::Improving
        } else if current_hit_rate < 50.0 || see_hit_rate < 30.0 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        TrendAnalysis {
            direction: trend_direction,
            confidence: self.calculate_trend_confidence(current_hit_rate, see_hit_rate),
            recommendation: self.generate_cache_trend_recommendation(current_hit_rate, see_hit_rate),
        }
    }

    /// Analyze memory usage trends
    fn analyze_memory_usage_trend(&self) -> TrendAnalysis {
        let current_usage = self.memory_usage.current_bytes as f64 / 1_000_000.0;
        let peak_usage = self.memory_usage.peak_bytes as f64 / 1_000_000.0;
        let utilization = if peak_usage > 0.0 { (current_usage / peak_usage) * 100.0 } else { 0.0 };

        let trend_direction = if utilization > 90.0 {
            TrendDirection::Declining
        } else if utilization < 50.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        TrendAnalysis {
            direction: trend_direction,
            confidence: self.calculate_trend_confidence(utilization, current_usage),
            recommendation: self.generate_memory_trend_recommendation(current_usage, peak_usage),
        }
    }

    /// Analyze heuristic effectiveness trends
    fn analyze_heuristic_effectiveness_trend(&self) -> TrendAnalysis {
        let effectiveness_scores = [
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.capture_stats),
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.promotion_stats),
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.tactical_stats),
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.pv_stats),
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.killer_stats),
            self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.history_stats),
        ];

        let avg_effectiveness = effectiveness_scores.iter().sum::<f64>() / effectiveness_scores.len() as f64;
        
        let trend_direction = if avg_effectiveness > 60.0 {
            TrendDirection::Improving
        } else if avg_effectiveness < 30.0 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        TrendAnalysis {
            direction: trend_direction,
            confidence: avg_effectiveness / 100.0,
            recommendation: self.generate_heuristic_trend_recommendation(avg_effectiveness),
        }
    }

    /// Analyze timing trends
    fn analyze_timing_trend(&self) -> TrendAnalysis {
        let avg_scoring_time = self.stats.timing_stats.move_scoring_times.avg_time_us;
        let avg_ordering_time = self.stats.timing_stats.move_ordering_times.avg_time_us;
        
        // Consider good performance if times are reasonable
        let trend_direction = if avg_scoring_time < 50.0 && avg_ordering_time < 100.0 {
            TrendDirection::Improving
        } else if avg_scoring_time > 200.0 || avg_ordering_time > 500.0 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        TrendAnalysis {
            direction: trend_direction,
            confidence: self.calculate_trend_confidence(avg_scoring_time, avg_ordering_time),
            recommendation: self.generate_timing_trend_recommendation(avg_scoring_time, avg_ordering_time),
        }
    }

    /// Analyze overall performance trends
    fn analyze_overall_performance_trend(&self) -> TrendAnalysis {
        let performance_score = self.calculate_performance_score();
        
        let trend_direction = if performance_score > 80 {
            TrendDirection::Improving
        } else if performance_score < 50 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        TrendAnalysis {
            direction: trend_direction,
            confidence: performance_score as f64 / 100.0,
            recommendation: self.generate_overall_trend_recommendation(performance_score),
        }
    }

    /// Calculate trend confidence based on metrics
    fn calculate_trend_confidence(&self, metric1: f64, metric2: f64) -> f64 {
        // Simple confidence calculation based on metric consistency
        let diff = (metric1 - metric2).abs();
        let max_metric = metric1.max(metric2);
        
        if max_metric == 0.0 {
            0.5
        } else {
            1.0 - (diff / max_metric).min(1.0)
        }
    }

    /// Generate cache trend recommendations
    fn generate_cache_trend_recommendation(&self, hit_rate: f64, see_hit_rate: f64) -> String {
        if hit_rate < 50.0 {
            "Consider increasing cache size or improving cache key generation".to_string()
        } else if see_hit_rate < 30.0 {
            "Enable SEE cache or increase SEE cache size".to_string()
        } else {
            "Cache performance is good, monitor for changes".to_string()
        }
    }

    /// Generate memory trend recommendations
    fn generate_memory_trend_recommendation(&self, current_mb: f64, peak_mb: f64) -> String {
        if current_mb > peak_mb * 0.9 {
            "High memory usage detected, consider reducing cache sizes".to_string()
        } else if current_mb < peak_mb * 0.5 {
            "Memory usage is efficient, consider increasing cache sizes for better performance".to_string()
        } else {
            "Memory usage is within normal range".to_string()
        }
    }

    /// Generate heuristic trend recommendations
    fn generate_heuristic_trend_recommendation(&self, avg_effectiveness: f64) -> String {
        if avg_effectiveness < 30.0 {
            "Low heuristic effectiveness, consider tuning heuristic weights".to_string()
        } else if avg_effectiveness > 60.0 {
            "High heuristic effectiveness, system is well-tuned".to_string()
        } else {
            "Moderate heuristic effectiveness, monitor for improvement opportunities".to_string()
        }
    }

    /// Generate timing trend recommendations
    fn generate_timing_trend_recommendation(&self, scoring_time: f64, ordering_time: f64) -> String {
        if scoring_time > 200.0 {
            "Move scoring is slow, consider optimizing scoring functions".to_string()
        } else if ordering_time > 500.0 {
            "Move ordering is slow, consider optimizing sorting algorithm".to_string()
        } else {
            "Timing performance is acceptable".to_string()
        }
    }

    /// Generate overall trend recommendations
    fn generate_overall_trend_recommendation(&self, score: u8) -> String {
        if score < 50 {
            "Overall performance needs significant improvement".to_string()
        } else if score > 80 {
            "Overall performance is excellent".to_string()
        } else {
            "Overall performance is good with room for optimization".to_string()
        }
    }

    // ==================== Error Handling Methods ====================

    /// Handle errors with appropriate logging and recovery
    #[allow(dead_code)] // Kept for future use and debugging
    fn handle_error(&mut self, error: MoveOrderingError, severity: ErrorSeverity, context: String) -> MoveOrderingResult<()> {
        // Log the error
        self.error_handler.log_error(error.clone(), severity.clone(), context);

        // Check if graceful degradation should be applied
        if self.error_handler.graceful_degradation_enabled {
            match severity {
                ErrorSeverity::Low | ErrorSeverity::Medium => {
                    // Continue with degraded functionality
                    return Ok(());
                }
                ErrorSeverity::High => {
                    // Attempt recovery if enabled
                    if self.error_handler.recovery_enabled {
                        self.attempt_error_recovery(&error)?;
                        return Ok(());
                    }
                    return Err(error);
                }
                ErrorSeverity::Critical => {
                    // Critical errors always fail
                    return Err(error);
                }
            }
        }

        Err(error)
    }

    /// Attempt to recover from an error
    #[allow(dead_code)] // Kept for future use and debugging
    fn attempt_error_recovery(&mut self, error: &MoveOrderingError) -> MoveOrderingResult<()> {
        match error {
            MoveOrderingError::CacheError(_) => {
                // Clear caches and continue
                self.clear_all_caches();
                Ok(())
            }
            MoveOrderingError::MemoryError(_) => {
                // Reduce memory usage and continue
                self.reduce_memory_usage();
                Ok(())
            }
            MoveOrderingError::StatisticsError(_) => {
                // Reset statistics and continue
                self.reset_statistics();
                Ok(())
            }
            _ => {
                // For other errors, return the error
                Err(error.clone())
            }
        }
    }

    /// Clear all caches to recover from cache errors
    #[allow(dead_code)] // Kept for future use and debugging
    fn clear_all_caches(&mut self) {
        self.move_score_cache.clear();
        self.fast_score_cache.clear();
        self.pv_move_cache.clear();
        self.see_cache.clear();
    }

    /// Reduce memory usage to recover from memory errors
    #[allow(dead_code)] // Kept for future use and debugging
    fn reduce_memory_usage(&mut self) {
        // Clear caches
        self.clear_all_caches();
        
        // Shrink object pools
        self.move_score_pool.shrink_to_fit();
        self.move_pool.shrink_to_fit();
        
        // Update memory statistics
        self.update_memory_usage();
    }

    /// Reset statistics to recover from statistics errors
    #[allow(dead_code)] // Kept for future use and debugging
    fn reset_statistics(&mut self) {
        self.stats = OrderingStats {
            hot_path_stats: HotPathStats::default(),
            heuristic_stats: HeuristicStats::default(),
            timing_stats: TimingStats::default(),
            memory_stats: MemoryStats::default(),
            cache_stats: CacheStats::default(),
            ..OrderingStats::default()
        };
    }

    /// Validate a move before processing
    fn validate_move(&self, move_: &Move) -> MoveOrderingResult<()> {
        // Check if move has required fields
        if move_.to.row >= 9 || move_.to.col >= 9 {
            return Err(MoveOrderingError::InvalidMove(
                format!("Invalid move destination: {:?}", move_.to)
            ));
        }

        if let Some(from) = move_.from {
            if from.row >= 9 || from.col >= 9 {
                return Err(MoveOrderingError::InvalidMove(
                    format!("Invalid move source: {:?}", from)
                ));
            }
        }

        // Check piece type validity
        match move_.piece_type {
            PieceType::Pawn | PieceType::Lance | PieceType::Knight | 
            PieceType::Silver | PieceType::Gold | PieceType::Bishop | 
            PieceType::Rook | PieceType::King => Ok(()),
            _ => Err(MoveOrderingError::InvalidMove(
                format!("Invalid piece type: {:?}", move_.piece_type)
            )),
        }
    }

    /// Get error handler reference
    pub fn get_error_handler(&self) -> &ErrorHandler {
        &self.error_handler
    }

    /// Get mutable error handler reference
    pub fn get_error_handler_mut(&mut self) -> &mut ErrorHandler {
        &mut self.error_handler
    }

    /// Check if system is in error state
    pub fn is_in_error_state(&self) -> bool {
        self.error_handler.is_system_unstable()
    }

    /// Get recent errors
    pub fn get_recent_errors(&self, count: usize) -> Vec<&ErrorLogEntry> {
        self.error_handler.get_recent_errors(count)
    }

    /// Clear error log
    pub fn clear_error_log(&mut self) {
        self.error_handler.clear_errors();
    }

    // ==================== Memory Management Methods ====================

    /// Get memory pool reference
    pub fn get_memory_pool(&self) -> &MemoryPool {
        &self.memory_pool
    }

    /// Get mutable memory pool reference
    pub fn get_memory_pool_mut(&mut self) -> &mut MemoryPool {
        &mut self.memory_pool
    }

    /// Get memory tracker reference
    pub fn get_memory_tracker(&self) -> &MemoryTracker {
        &self.memory_tracker
    }

    /// Get mutable memory tracker reference
    pub fn get_memory_tracker_mut(&mut self) -> &mut MemoryTracker {
        &mut self.memory_tracker
    }

    /// Record memory allocation
    #[allow(dead_code)] // Kept for future use and debugging
    fn record_allocation(&mut self, allocation_type: AllocationType, size: usize, component: String) {
        self.memory_tracker.record_allocation(allocation_type, size, component);
    }

    /// Record memory deallocation
    #[allow(dead_code)] // Kept for future use and debugging
    fn record_deallocation(&mut self, allocation_type: AllocationType, size: usize, component: String) {
        self.memory_tracker.record_deallocation(allocation_type, size, component);
    }

    /// Check for memory leaks
    pub fn check_memory_leaks(&self) -> Vec<MemoryLeakWarning> {
        self.memory_tracker.check_for_leaks()
    }

    /// Get current memory usage
    pub fn get_current_memory_usage(&self) -> &MemoryUsageBreakdown {
        self.memory_tracker.get_current_usage()
    }

    /// Get peak memory usage
    pub fn get_peak_memory_usage(&self) -> &MemoryUsageBreakdown {
        self.memory_tracker.get_peak_usage()
    }

    /// Check memory thresholds
    pub fn check_memory_thresholds(&self) -> MemoryThresholdStatus {
        self.memory_tracker.check_thresholds()
    }

    /// Get memory pool statistics
    pub fn get_memory_pool_stats(&self) -> MemoryPoolSizes {
        self.memory_pool.get_pool_stats()
    }

    /// Perform comprehensive memory leak detection
    pub fn detect_memory_leaks(&self) -> MemoryLeakReport {
        let warnings = self.check_memory_leaks();
        let current_usage = self.get_current_memory_usage();
        let peak_usage = self.get_peak_memory_usage();
        let pool_stats = self.get_memory_pool_stats();
        
        let leak_detected = !warnings.is_empty();
        MemoryLeakReport {
            warnings,
            current_usage: current_usage.clone(),
            peak_usage: peak_usage.clone(),
            pool_stats,
            leak_detected,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Enable or disable memory leak detection
    pub fn set_leak_detection(&mut self, enabled: bool) {
        self.memory_tracker.leak_detection_enabled = enabled;
    }

    /// Clear memory allocation history
    pub fn clear_memory_history(&mut self) {
        self.memory_tracker.clear_history();
    }

    /// Get memory allocation history
    pub fn get_allocation_history(&self) -> &Vec<AllocationEvent> {
        &self.memory_tracker.allocation_history
    }

    // ==================== Advanced Features Methods ====================

    /// Get advanced features reference
    pub fn get_advanced_features(&self) -> &AdvancedFeatures {
        &self.advanced_features
    }

    /// Get mutable advanced features reference
    pub fn get_advanced_features_mut(&mut self) -> &mut AdvancedFeatures {
        &mut self.advanced_features
    }

    /// Determine game phase based on position characteristics
    pub fn determine_game_phase(&self, move_count: usize, material_balance: i32, tactical_complexity: f64) -> GamePhase {
        // Simple phase determination logic
        if move_count < 20 {
            GamePhase::Opening
        } else if move_count > 60 {
            GamePhase::Endgame
        } else if tactical_complexity > 0.7 {
            GamePhase::Tactical
        } else if material_balance.abs() < 200 && tactical_complexity < 0.3 {
            GamePhase::Positional
        } else {
            GamePhase::Middlegame
        }
    }

    /// Update game phase and adjust strategy accordingly
    pub fn update_game_phase(&mut self, move_count: usize, material_balance: i32, tactical_complexity: f64) {
        let new_phase = self.determine_game_phase(move_count, material_balance, tactical_complexity);
        
        if new_phase != self.advanced_features.position_strategies.current_phase {
            self.advanced_features.position_strategies.current_phase = new_phase;
            
            // Update weights based on new phase
            self.apply_phase_strategy();
        }
    }

    /// Apply the current phase strategy
    fn apply_phase_strategy(&mut self) {
        let strategy = match self.advanced_features.position_strategies.current_phase {
            GamePhase::Opening => &self.advanced_features.position_strategies.opening_strategy,
            GamePhase::Middlegame => &self.advanced_features.position_strategies.middlegame_strategy,
            GamePhase::Endgame => &self.advanced_features.position_strategies.endgame_strategy,
            GamePhase::Tactical => &self.advanced_features.position_strategies.tactical_strategy,
            GamePhase::Positional => &self.advanced_features.position_strategies.positional_strategy,
        };

        // Update current weights with phase-specific weights
        self.config.weights = strategy.weights.clone();
    }

    /// Score move using position-specific strategy
    pub fn score_move_with_strategy(&mut self, move_: &Move) -> MoveOrderingResult<i32> {
        // Get current strategy (clone to avoid borrowing issues)
        let strategy = match self.advanced_features.position_strategies.current_phase {
            GamePhase::Opening => self.advanced_features.position_strategies.opening_strategy.clone(),
            GamePhase::Middlegame => self.advanced_features.position_strategies.middlegame_strategy.clone(),
            GamePhase::Endgame => self.advanced_features.position_strategies.endgame_strategy.clone(),
            GamePhase::Tactical => self.advanced_features.position_strategies.tactical_strategy.clone(),
            GamePhase::Positional => self.advanced_features.position_strategies.positional_strategy.clone(),
        };

        // Apply strategy-specific scoring
        let base_score = self.score_move(move_)?;
        let adjusted_score = self.apply_strategy_adjustments(move_, base_score, &strategy);

        Ok(adjusted_score)
    }

    /// Apply strategy-specific adjustments to move score
    fn apply_strategy_adjustments(&self, move_: &Move, base_score: i32, strategy: &OrderingStrategy) -> i32 {
        let mut adjusted_score = base_score;

        // Apply priority adjustments
        if move_.is_capture {
            adjusted_score = (adjusted_score as f64 * strategy.priority_adjustments.capture_priority) as i32;
        }
        if move_.is_promotion {
            adjusted_score = (adjusted_score as f64 * strategy.priority_adjustments.promotion_priority) as i32;
        }
        if self.is_development_move(move_) {
            adjusted_score = (adjusted_score as f64 * strategy.priority_adjustments.development_priority) as i32;
        }
        if self.is_center_move(move_) {
            adjusted_score = (adjusted_score as f64 * strategy.priority_adjustments.center_priority) as i32;
        }
        if self.is_king_safety_move(move_) {
            adjusted_score = (adjusted_score as f64 * strategy.priority_adjustments.king_safety_priority) as i32;
        }

        adjusted_score
    }

    /// Check if move is a development move
    fn is_development_move(&self, move_: &Move) -> bool {
        // Simple heuristic: moves that bring pieces to more active squares
        if let Some(from) = move_.from {
            let from_rank = from.row;
            let to_rank = move_.to.row;
            
            // Moving pieces forward (toward center/opponent)
            match move_.piece_type {
                PieceType::Pawn => to_rank > from_rank,
                PieceType::Lance => to_rank > from_rank,
                PieceType::Knight => to_rank > from_rank + 1,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Check if move is a center move
    fn is_center_move(&self, move_: &Move) -> bool {
        let center_files = [3, 4, 5]; // Files d, e, f
        let center_ranks = [3, 4, 5]; // Ranks 4, 5, 6
        
        center_files.contains(&move_.to.col) && center_ranks.contains(&move_.to.row)
    }

    /// Check if move is a king safety move
    fn is_king_safety_move(&self, move_: &Move) -> bool {
        // Simple heuristic: moves that protect the king or move it to safety
        match move_.piece_type {
            PieceType::King => true,
            _ => {
                // Check if move defends king area
                if let Some(_from) = move_.from {
                    let king_area_files = [3, 4, 5];
                    let king_area_ranks = [0, 1, 2, 6, 7, 8];
                    
                    king_area_files.contains(&move_.to.col) && king_area_ranks.contains(&move_.to.row)
                } else {
                    false
                }
            }
        }
    }

    /// Train machine learning model with new data
    pub fn train_ml_model(&mut self, training_examples: Vec<TrainingExample>) -> MoveOrderingResult<f64> {
        if !self.advanced_features.ml_model.enabled {
            return Err(MoveOrderingError::OperationError(
                "Machine learning model is not enabled".to_string()
            ));
        }

        // Add training examples
        self.advanced_features.ml_model.training_data.extend(training_examples);

        // Simple training simulation (in real implementation, this would train the actual model)
        let accuracy = self.simulate_ml_training();
        self.advanced_features.ml_model.accuracy = accuracy;

        Ok(accuracy)
    }

    /// Simulate machine learning training (placeholder)
    fn simulate_ml_training(&self) -> f64 {
        // Simple simulation: accuracy improves with more training data
        let data_size = self.advanced_features.ml_model.training_data.len();
        let base_accuracy = 0.5;
        let improvement = (data_size as f64 / 1000.0).min(0.4);
        
        base_accuracy + improvement
    }

    /// Predict move score using machine learning model
    pub fn predict_move_score(&mut self, _move_: &Move, position_features: Vec<f64>) -> MoveOrderingResult<i32> {
        if !self.advanced_features.ml_model.enabled {
            return Err(MoveOrderingError::OperationError(
                "Machine learning model is not enabled".to_string()
            ));
        }

        // Simple prediction simulation
        let prediction = self.simulate_ml_prediction(position_features);
        Ok(prediction)
    }

    /// Simulate machine learning prediction (placeholder)
    fn simulate_ml_prediction(&self, features: Vec<f64>) -> i32 {
        // Simple simulation: weighted sum of features
        let mut score = 0.0;
        for (i, feature) in features.iter().enumerate() {
            score += feature * (i as f64 + 1.0) * 100.0;
        }
        
        score as i32
    }

    /// Adjust weights dynamically based on performance
    pub fn adjust_weights_dynamically(&mut self, performance_score: f64) -> MoveOrderingResult<()> {
        if !self.advanced_features.dynamic_weights.enabled {
            return Err(MoveOrderingError::OperationError(
                "Dynamic weight adjustment is not enabled".to_string()
            ));
        }

        let old_weights = self.config.weights.clone();
        let new_weights = self.calculate_optimal_weights(performance_score);
        
        // Record adjustment
        let adjustment = WeightAdjustment {
            timestamp: std::time::SystemTime::now(),
            old_weights: old_weights.clone(),
            new_weights: new_weights.clone(),
            reason: format!("Performance-based adjustment: {:.2}", performance_score),
            performance_impact: performance_score,
        };

        self.advanced_features.dynamic_weights.adjustment_history.push(adjustment);
        self.advanced_features.dynamic_weights.current_weights = new_weights.clone();
        self.config.weights = new_weights;

        Ok(())
    }

    /// Calculate optimal weights based on performance
    fn calculate_optimal_weights(&self, performance_score: f64) -> OrderingWeights {
        let mut weights = self.config.weights.clone();
        
        // Simple adjustment logic based on performance
        if performance_score > 0.8 {
            // Good performance: increase weights slightly
            weights.capture_weight = (weights.capture_weight as f64 * 1.05) as i32;
            weights.pv_move_weight = (weights.pv_move_weight as f64 * 1.05) as i32;
        } else if performance_score < 0.5 {
            // Poor performance: adjust weights more significantly
            weights.capture_weight = (weights.capture_weight as f64 * 0.95) as i32;
            weights.history_weight = (weights.history_weight as f64 * 1.1) as i32;
        }

        weights
    }

    /// Enable or disable advanced features
    pub fn set_advanced_features_enabled(&mut self, features: AdvancedFeatureFlags) {
        if features.position_specific_strategies {
            // Position-specific strategies are always enabled
        }
        if features.machine_learning {
            self.advanced_features.ml_model.enabled = features.machine_learning;
        }
        if features.dynamic_weights {
            self.advanced_features.dynamic_weights.enabled = features.dynamic_weights;
        }
        if features.predictive_ordering {
            self.advanced_features.predictive_ordering.enabled = features.predictive_ordering;
        }
        if features.cache_warming {
            self.advanced_features.cache_warming.enabled = features.cache_warming;
        }
    }

    /// Get advanced features status
    pub fn get_advanced_features_status(&self) -> AdvancedFeatureStatus {
        AdvancedFeatureStatus {
            position_specific_strategies: true, // Always enabled
            machine_learning: self.advanced_features.ml_model.enabled,
            dynamic_weights: self.advanced_features.dynamic_weights.enabled,
            predictive_ordering: self.advanced_features.predictive_ordering.enabled,
            cache_warming: self.advanced_features.cache_warming.enabled,
            current_phase: self.advanced_features.position_strategies.current_phase.clone(),
            ml_accuracy: self.advanced_features.ml_model.accuracy,
            prediction_accuracy: self.advanced_features.predictive_ordering.accuracy,
        }
    }

    /// Perform comprehensive memory cleanup
    pub fn cleanup_memory(&mut self) -> MemoryCleanupReport {
        let before_usage = self.get_current_memory_usage().clone();
        
        // Clear all caches
        self.clear_all_caches();
        
        // Clear memory pools
        self.memory_pool.clear_all_pools();
        
        // Clear allocation history
        self.memory_tracker.clear_history();
        
        // Clear error log
        self.error_handler.clear_errors();
        
        // Force garbage collection by shrinking vectors
        self.move_score_cache.shrink_to_fit();
        self.fast_score_cache.shrink_to_fit();
        self.pv_move_cache.shrink_to_fit();
        self.see_cache.shrink_to_fit();
        self.history_table.shrink_to_fit();
        
        let after_usage = self.get_current_memory_usage().clone();
        let memory_freed = before_usage.total_memory.saturating_sub(after_usage.total_memory);
        
        MemoryCleanupReport {
            before_usage,
            after_usage,
            memory_freed,
            cleanup_successful: true,
            timestamp: std::time::SystemTime::now(),
        }
    }


    /// Perform selective memory cleanup based on memory pressure
    pub fn selective_cleanup(&mut self, pressure_level: MemoryPressureLevel) -> MemoryCleanupReport {
        let before_usage = self.get_current_memory_usage().clone();
        
        match pressure_level {
            MemoryPressureLevel::Low => {
                // Only clear old cache entries
                self.clear_old_cache_entries();
            }
            MemoryPressureLevel::Medium => {
                // Clear caches and shrink vectors
                self.clear_all_caches();
                self.shrink_vectors();
            }
            MemoryPressureLevel::High => {
                // Aggressive cleanup
                self.clear_all_caches();
                self.memory_pool.clear_all_pools();
                self.shrink_vectors();
            }
            MemoryPressureLevel::Critical => {
                // Complete cleanup
                self.cleanup_memory();
            }
        }
        
        let after_usage = self.get_current_memory_usage().clone();
        let memory_freed = before_usage.total_memory.saturating_sub(after_usage.total_memory);
        
        MemoryCleanupReport {
            before_usage,
            after_usage,
            memory_freed,
            cleanup_successful: true,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Clear old cache entries to free memory
    fn clear_old_cache_entries(&mut self) {
        // For now, we'll clear all cache entries
        // In a more sophisticated implementation, we could track entry ages
        if self.move_score_cache.len() > 1000 {
            self.move_score_cache.clear();
        }
        if self.see_cache.len() > 500 {
            self.see_cache.clear();
        }
    }

    /// Shrink vectors to free memory
    fn shrink_vectors(&mut self) {
        self.move_score_cache.shrink_to_fit();
        self.fast_score_cache.shrink_to_fit();
        self.pv_move_cache.shrink_to_fit();
        self.see_cache.shrink_to_fit();
        self.history_table.shrink_to_fit();
        self.move_score_pool.shrink_to_fit();
        self.move_pool.shrink_to_fit();
    }

    /// Order moves with history heuristic prioritization
    /// 
    /// This enhanced version of order_moves prioritizes moves based on
    /// their history heuristic scores.
    pub fn order_moves_with_history(&mut self, moves: &[Move]) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Sort moves by score with history heuristic prioritization
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_history(a);
            let score_b = self.score_move_with_history(b);
            score_b.cmp(&score_a)
        });

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        ordered_moves
    }

    /// Score a move with history heuristic consideration
    /// 
    /// This method scores a move, giving priority to moves with
    /// high history scores and falling back to regular move scoring.
    fn score_move_with_history(&mut self, move_: &Move) -> i32 {
        // Check if this move has history score
        let history_score = self.score_history_move(move_);
        if history_score > 0 {
            return history_score;
        }

        // Use regular move scoring
        self.score_move(move_).unwrap_or(0)
    }

    /// Order moves with PV, killer, and history prioritization
    /// 
    /// This method combines all three prioritization strategies
    /// for optimal move ordering.
    pub fn order_moves_with_all_heuristics(&mut self, moves: &[Move], board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Set current depth for killer move management
        self.set_current_depth(depth);

        // Get PV move for this position
        let pv_move = self.get_pv_move(board, captured_pieces, player, depth);

        // Get killer moves for current depth
        let killer_moves = self.get_current_killer_moves().cloned().unwrap_or_default();

        // Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Sort moves by score with all heuristics prioritization
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_all_heuristics(a, &pv_move, &killer_moves);
            let score_b = self.score_move_with_all_heuristics(b, &pv_move, &killer_moves);
            score_b.cmp(&score_a)
        });

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.total_ordering_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_ordering_time_us = 
            self.stats.total_ordering_time_us as f64 / self.stats.total_moves_ordered as f64;

        // Update memory usage
        self.update_memory_usage();

        ordered_moves
    }

    /// Score a move with all heuristics consideration
    /// 
    /// This method scores a move with the following priority:
    /// 1. PV moves (highest priority)
    /// 2. Killer moves (high priority)
    /// 3. History moves (medium-high priority)
    /// 4. Regular moves (normal priority)
    fn score_move_with_all_heuristics(&mut self, move_: &Move, pv_move: &Option<Move>, killer_moves: &[Move]) -> i32 {
        // Check if this is the PV move (highest priority)
        if let Some(ref pv) = pv_move {
            if self.moves_equal(move_, pv) {
                return self.score_pv_move(move_);
            }
        }

        // Check if this is a killer move (high priority)
        if killer_moves.iter().any(|killer| self.moves_equal(move_, killer)) {
            self.stats.killer_move_hits += 1;
            return self.score_killer_move(move_);
        }

        // Check if this move has history score (medium-high priority)
        let history_score = self.score_history_move(move_);
        if history_score > 0 {
            return history_score;
        }

        // Use regular move scoring
        self.stats.killer_move_misses += 1;
        self.score_move(move_).unwrap_or(0)
    }

    // ==================== Transposition Table Integration ====================

    /// Integrate move ordering with transposition table
    /// 
    /// This method enhances move ordering by using transposition table data
    /// to prioritize moves that have been successful in previous searches.
    pub fn integrate_with_transposition_table(&mut self, tt_entry: Option<&TranspositionEntry>, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> MoveOrderingResult<()> {
        if let Some(entry) = tt_entry {
            // Update PV move if we have a best move from the transposition table
            if let Some(ref best_move) = entry.best_move {
                self.update_pv_move(board, captured_pieces, player, depth, best_move.clone(), entry.score);
                self.stats.tt_integration_hits += 1;
                
                // Update killer moves if this was a cutoff move
                if entry.is_lower_bound() || entry.is_exact() {
                    self.add_killer_move(best_move.clone());
                }
                
                // Update history heuristic based on transposition table results
                self.update_history_from_tt(best_move, entry.score, depth);
            }
            
            // Update statistics
            self.stats.tt_integration_updates += 1;
        }
        
        Ok(())
    }

    /// Use transposition table for PV move identification
    /// 
    /// Retrieves the best move from the transposition table and prioritizes
    /// it in the move ordering.
    pub fn get_pv_move_from_tt(&self, tt_entry: Option<&TranspositionEntry>) -> Option<Move> {
        tt_entry.and_then(|entry| entry.best_move.clone())
    }

    /// Update move ordering based on transposition table results
    /// 
    /// This method adjusts move ordering weights and priorities based on
    /// the results found in the transposition table.
    pub fn update_ordering_from_tt_result(&mut self, tt_entry: &TranspositionEntry, move_result: MoveResult) -> MoveOrderingResult<()> {
        match move_result {
            MoveResult::Cutoff => {
                // This move caused a cutoff, increase its priority
                if let Some(ref best_move) = tt_entry.best_move {
                    self.update_killer_move_priority(best_move, tt_entry.depth);
                    self.update_history_from_cutoff(best_move, tt_entry.depth);
                }
                self.stats.tt_cutoff_updates += 1;
            },
            MoveResult::Exact => {
                // This move has an exact score, use it for PV
                if let Some(ref best_move) = tt_entry.best_move {
                    self.update_pv_move_from_tt(best_move, tt_entry.score, tt_entry.depth);
                }
                self.stats.tt_exact_updates += 1;
            },
            MoveResult::Bound => {
                // This move has a bound, use it for ordering
                if let Some(ref best_move) = tt_entry.best_move {
                    self.update_bound_move_priority(best_move, tt_entry.flag, tt_entry.depth);
                }
                self.stats.tt_bound_updates += 1;
            }
        }
        
        Ok(())
    }

    /// Update history heuristic from transposition table data
    fn update_history_from_tt(&mut self, move_: &Move, score: i32, depth: u8) {
        if let (Some(from), Some(to)) = (move_.from, Some(move_.to)) {
            let history_value = if score > 0 { depth as i32 + 1 } else { -(depth as i32 + 1) };
            let from_idx = from.row as usize;
            let to_idx = to.row as usize;
            self.simple_history_table[from_idx][to_idx] += history_value;
            
            // Clamp history values to prevent overflow
            let max_history = 10000;
            if self.simple_history_table[from_idx][to_idx] > max_history {
                self.simple_history_table[from_idx][to_idx] = max_history;
            } else if self.simple_history_table[from_idx][to_idx] < -max_history {
                self.simple_history_table[from_idx][to_idx] = -max_history;
            }
            
            self.stats.history_updates_from_tt += 1;
        }
    }

    /// Update killer move priority based on transposition table
    fn update_killer_move_priority(&mut self, move_: &Move, _depth: u8) {
        // Add to killer moves with higher priority
        self.add_killer_move(move_.clone());
        
        // Update killer move statistics
        self.stats.killer_moves_from_tt += 1;
    }

    /// Update PV move from transposition table
    fn update_pv_move_from_tt(&mut self, move_: &Move, _score: i32, depth: u8) {
        // Store as PV move for future reference
        self.pv_moves.insert(depth, move_.clone());
        
        // Update PV move statistics
        self.stats.pv_moves_from_tt += 1;
    }

    /// Update bound move priority
    fn update_bound_move_priority(&mut self, move_: &Move, flag: TranspositionFlag, depth: u8) {
        match flag {
            TranspositionFlag::LowerBound => {
                // Lower bound means this move is at least this good
                self.add_killer_move(move_.clone());
            },
            TranspositionFlag::UpperBound => {
                // Upper bound means this move is at most this good
                // Don't prioritize it as highly
            },
            TranspositionFlag::Exact => {
                // Exact score, treat as PV move
                self.update_pv_move_from_tt(move_, 0, depth);
            }
        }
    }

    /// Update history from cutoff move
    fn update_history_from_cutoff(&mut self, move_: &Move, depth: u8) {
        if let (Some(from), Some(to)) = (move_.from, Some(move_.to)) {
            let bonus = depth as i32 + 2; // Higher bonus for cutoff moves
            let from_idx = from.row as usize;
            let to_idx = to.row as usize;
            self.simple_history_table[from_idx][to_idx] += bonus;
            self.stats.cutoff_history_updates += 1;
        }
    }

    /// Get transposition table integration statistics
    pub fn get_tt_integration_stats(&self) -> TTIntegrationStats {
        TTIntegrationStats {
            tt_integration_hits: self.stats.tt_integration_hits,
            tt_integration_updates: self.stats.tt_integration_updates,
            tt_cutoff_updates: self.stats.tt_cutoff_updates,
            tt_exact_updates: self.stats.tt_exact_updates,
            tt_bound_updates: self.stats.tt_bound_updates,
            killer_moves_from_tt: self.stats.killer_moves_from_tt,
            pv_moves_from_tt: self.stats.pv_moves_from_tt,
            history_updates_from_tt: self.stats.history_updates_from_tt,
            cutoff_history_updates: self.stats.cutoff_history_updates,
        }
    }

    // ==================== Runtime Performance Tuning ====================

    /// Tune performance at runtime based on current statistics
    /// 
    /// This method analyzes current performance metrics and automatically
    /// adjusts configuration to optimize performance.
    pub fn tune_performance_runtime(&mut self) -> PerformanceTuningResult {
        let stats = &self.stats;
        let mut adjustments = Vec::new();
        let mut config = self.config.clone();
        
        // Tune cache size based on hit rate
        if stats.cache_hit_rate < 50.0 && config.cache_config.max_cache_size < 500000 {
            let old_size = config.cache_config.max_cache_size;
            config.cache_config.max_cache_size = (config.cache_config.max_cache_size * 3) / 2;
            adjustments.push(format!("Increased cache size: {} -> {}", old_size, config.cache_config.max_cache_size));
        } else if stats.cache_hit_rate > 95.0 && config.cache_config.max_cache_size > 10000 {
            let old_size = config.cache_config.max_cache_size;
            config.cache_config.max_cache_size = (config.cache_config.max_cache_size * 3) / 4;
            adjustments.push(format!("Decreased cache size: {} -> {}", old_size, config.cache_config.max_cache_size));
        }
        
        // Tune SEE cache size based on hit rate
        if stats.see_cache_hit_rate < 40.0 && config.cache_config.max_see_cache_size < 250000 {
            let old_size = config.cache_config.max_see_cache_size;
            config.cache_config.max_see_cache_size = (config.cache_config.max_see_cache_size * 3) / 2;
            adjustments.push(format!("Increased SEE cache size: {} -> {}", old_size, config.cache_config.max_see_cache_size));
        }
        
        // Tune history aging frequency based on hit rate
        if stats.history_hit_rate < 20.0 && config.history_config.aging_frequency < 300 {
            let old_freq = config.history_config.aging_frequency;
            config.history_config.aging_frequency += 50;
            adjustments.push(format!("Increased history aging frequency: {} -> {}", old_freq, config.history_config.aging_frequency));
        }
        
        // Apply adjustments
        if !adjustments.is_empty() {
            self.config = config;
        }
        
        PerformanceTuningResult {
            adjustments_made: adjustments.len(),
            adjustments,
            cache_hit_rate_before: stats.cache_hit_rate,
            avg_ordering_time_before: stats.avg_ordering_time_us,
        }
    }

    /// Monitor performance and return recommendations
    /// 
    /// This method analyzes current performance and returns recommendations
    /// for tuning without automatically applying them.
    pub fn monitor_performance(&self) -> PerformanceMonitoringReport {
        let stats = &self.stats;
        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();
        
        // Check cache hit rate
        if stats.cache_hit_rate < 50.0 {
            warnings.push("Low cache hit rate".to_string());
            recommendations.push("Consider increasing cache size".to_string());
        } else if stats.cache_hit_rate > 90.0 {
            recommendations.push("Excellent cache hit rate - could reduce cache size to save memory".to_string());
        }
        
        // Check ordering time
        if stats.avg_ordering_time_us > 100.0 {
            warnings.push("High average ordering time".to_string());
            recommendations.push("Consider disabling SEE or increasing cache size".to_string());
        }
        
        // Check memory usage
        if stats.memory_usage_bytes > 20 * 1024 * 1024 {
            warnings.push("High memory usage (> 20 MB)".to_string());
            recommendations.push("Consider reducing cache sizes or clearing caches periodically".to_string());
        }
        
        // Check heuristic effectiveness
        if stats.pv_move_hit_rate < 20.0 {
            warnings.push("Low PV move hit rate".to_string());
            recommendations.push("Ensure PV moves are being updated during search".to_string());
        }
        
        if stats.killer_move_hit_rate < 15.0 {
            warnings.push("Low killer move hit rate".to_string());
            recommendations.push("Ensure killer moves are added at beta cutoffs".to_string());
        }
        
        if stats.history_hit_rate < 15.0 {
            warnings.push("Low history hit rate".to_string());
            recommendations.push("Ensure history is updated for all moves tried".to_string());
        }
        
        // Calculate overall health score (0-100)
        let mut health_score = 100.0;
        health_score -= (50.0 - stats.cache_hit_rate).max(0.0);
        health_score -= (stats.avg_ordering_time_us - 50.0).max(0.0) / 10.0;
        health_score -= (30.0 - stats.pv_move_hit_rate).max(0.0);
        health_score = health_score.max(0.0);
        
        PerformanceMonitoringReport {
            overall_health_score: health_score,
            cache_hit_rate: stats.cache_hit_rate,
            avg_ordering_time_us: stats.avg_ordering_time_us,
            memory_usage_mb: stats.memory_usage_bytes as f64 / 1_048_576.0,
            pv_hit_rate: stats.pv_move_hit_rate,
            killer_hit_rate: stats.killer_move_hit_rate,
            history_hit_rate: stats.history_hit_rate,
            warnings,
            recommendations,
        }
    }

    /// Automatically optimize configuration based on performance
    /// 
    /// This method applies automatic optimizations to improve performance
    /// based on current statistics and usage patterns.
    pub fn auto_optimize(&mut self) -> AutoOptimizationResult {
        let start_stats = self.stats.clone();
        let mut optimizations = Vec::new();
        
        // Adjust weights based on heuristic effectiveness
        let weight_adjustments = self.adjust_weights_based_on_effectiveness();
        optimizations.extend(weight_adjustments);
        
        // Optimize cache configuration
        let cache_optimizations = self.optimize_cache_configuration();
        optimizations.extend(cache_optimizations);
        
        // Optimize heuristic enablement
        let heuristic_optimizations = self.optimize_heuristic_enablement();
        optimizations.extend(heuristic_optimizations);
        
        AutoOptimizationResult {
            optimizations_applied: optimizations.len(),
            optimizations,
            performance_before: PerformanceSnapshot {
                cache_hit_rate: start_stats.cache_hit_rate,
                avg_ordering_time_us: start_stats.avg_ordering_time_us,
                memory_usage_bytes: start_stats.memory_usage_bytes,
            },
            performance_after: PerformanceSnapshot {
                cache_hit_rate: self.stats.cache_hit_rate,
                avg_ordering_time_us: self.stats.avg_ordering_time_us,
                memory_usage_bytes: self.stats.memory_usage_bytes,
            },
        }
    }

    /// Adjust weights based on heuristic effectiveness
    fn adjust_weights_based_on_effectiveness(&mut self) -> Vec<String> {
        let mut adjustments = Vec::new();
        
        // Adjust PV weight based on hit rate
        if self.stats.pv_move_hit_rate > 40.0 && self.config.weights.pv_move_weight < 12000 {
            self.config.weights.pv_move_weight += 1000;
            adjustments.push(format!("Increased PV weight to {}", self.config.weights.pv_move_weight));
        } else if self.stats.pv_move_hit_rate < 20.0 && self.config.weights.pv_move_weight > 8000 {
            self.config.weights.pv_move_weight -= 500;
            adjustments.push(format!("Decreased PV weight to {}", self.config.weights.pv_move_weight));
        }
        
        // Adjust killer weight based on hit rate
        if self.stats.killer_move_hit_rate > 30.0 && self.config.weights.killer_move_weight < 6000 {
            self.config.weights.killer_move_weight += 500;
            adjustments.push(format!("Increased killer weight to {}", self.config.weights.killer_move_weight));
        } else if self.stats.killer_move_hit_rate < 15.0 && self.config.weights.killer_move_weight > 3000 {
            self.config.weights.killer_move_weight -= 500;
            adjustments.push(format!("Decreased killer weight to {}", self.config.weights.killer_move_weight));
        }
        
        // Adjust history weight based on hit rate
        if self.stats.history_hit_rate > 40.0 && self.config.weights.history_weight < 400 {
            self.config.weights.history_weight += 50;
            adjustments.push(format!("Increased history weight to {}", self.config.weights.history_weight));
        } else if self.stats.history_hit_rate < 15.0 && self.config.weights.history_weight > 50 {
            self.config.weights.history_weight -= 25;
            adjustments.push(format!("Decreased history weight to {}", self.config.weights.history_weight));
        }
        
        adjustments
    }

    /// Optimize cache configuration
    fn optimize_cache_configuration(&mut self) -> Vec<String> {
        let mut optimizations = Vec::new();
        
        // Adjust main cache size
        if self.stats.cache_hit_rate < 60.0 && self.config.cache_config.max_cache_size < 500000 {
            let old_size = self.config.cache_config.max_cache_size;
            self.config.cache_config.max_cache_size = std::cmp::min(
                self.config.cache_config.max_cache_size * 2,
                500000
            );
            optimizations.push(format!("Increased cache size: {} -> {}", old_size, self.config.cache_config.max_cache_size));
        }
        
        // Adjust SEE cache size
        if self.stats.see_cache_hit_rate < 50.0 && self.config.cache_config.max_see_cache_size < 250000 {
            let old_size = self.config.cache_config.max_see_cache_size;
            self.config.cache_config.max_see_cache_size = std::cmp::min(
                self.config.cache_config.max_see_cache_size * 2,
                250000
            );
            optimizations.push(format!("Increased SEE cache size: {} -> {}", old_size, self.config.cache_config.max_see_cache_size));
        }
        
        // Enable auto optimization if caches are effective
        if self.stats.cache_hit_rate > 70.0 && !self.config.cache_config.enable_auto_optimization {
            self.config.cache_config.enable_auto_optimization = true;
            optimizations.push("Enabled automatic cache optimization".to_string());
        }
        
        optimizations
    }

    /// Optimize heuristic enablement
    fn optimize_heuristic_enablement(&mut self) -> Vec<String> {
        let mut optimizations = Vec::new();
        
        // Disable SEE cache if it's too slow
        if self.stats.avg_see_calculation_time_us > 50.0 && self.config.cache_config.enable_see_cache {
            self.config.cache_config.enable_see_cache = false;
            optimizations.push("Disabled SEE cache due to high calculation time".to_string());
        }
        
        // Increase max killer moves if hit rate is good
        if self.stats.killer_move_hit_rate > 30.0 && self.config.killer_config.max_killer_moves_per_depth < 3 {
            self.config.killer_config.max_killer_moves_per_depth += 1;
            optimizations.push(format!("Increased max killer moves to {}", self.config.killer_config.max_killer_moves_per_depth));
        }
        
        // Adjust history aging frequency
        if self.stats.history_hit_rate < 20.0 && self.config.history_config.aging_frequency < 300 {
            let old_freq = self.config.history_config.aging_frequency;
            self.config.history_config.aging_frequency += 50;
            optimizations.push(format!("Increased history aging frequency: {} -> {}", old_freq, self.config.history_config.aging_frequency));
        }
        
        optimizations
    }

    /// Get performance tuning recommendations
    /// 
    /// Returns recommendations for improving performance without applying them.
    pub fn get_tuning_recommendations(&self) -> Vec<TuningRecommendation> {
        let mut recommendations = Vec::new();
        let stats = &self.stats;
        
        // Cache size recommendations
        if stats.cache_hit_rate < 60.0 {
            recommendations.push(TuningRecommendation {
                category: TuningCategory::CacheSize,
                priority: TuningPriority::High,
                description: format!(
                    "Cache hit rate is {:.2}%, consider increasing cache size from {} to {}",
                    stats.cache_hit_rate,
                    self.config.cache_config.max_cache_size,
                    self.config.cache_config.max_cache_size * 2
                ),
                expected_impact: "Improved cache hit rate, faster ordering".to_string(),
            });
        }
        
        // Weight adjustment recommendations
        if stats.pv_move_hit_rate > 40.0 && self.config.weights.pv_move_weight < 12000 {
            recommendations.push(TuningRecommendation {
                category: TuningCategory::Weights,
                priority: TuningPriority::Medium,
                description: format!(
                    "PV hit rate is {:.2}%, consider increasing PV weight from {} to {}",
                    stats.pv_move_hit_rate,
                    self.config.weights.pv_move_weight,
                    self.config.weights.pv_move_weight + 1000
                ),
                expected_impact: "Better move ordering from PV moves".to_string(),
            });
        }
        
        // Performance recommendations
        if stats.avg_ordering_time_us > 100.0 {
            recommendations.push(TuningRecommendation {
                category: TuningCategory::Performance,
                priority: TuningPriority::High,
                description: format!(
                    "Average ordering time is {:.2}μs, consider disabling SEE or increasing caches",
                    stats.avg_ordering_time_us
                ),
                expected_impact: "Faster move ordering".to_string(),
            });
        }
        
        // Memory recommendations
        if stats.memory_usage_bytes > 20 * 1024 * 1024 {
            recommendations.push(TuningRecommendation {
                category: TuningCategory::Memory,
                priority: TuningPriority::Medium,
                description: format!(
                    "Memory usage is {:.2} MB, consider reducing cache sizes",
                    stats.memory_usage_bytes as f64 / 1_048_576.0
                ),
                expected_impact: "Reduced memory footprint".to_string(),
            });
        }
        
        recommendations
    }

    /// Create performance snapshot for comparison
    pub fn create_performance_snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            cache_hit_rate: self.stats.cache_hit_rate,
            avg_ordering_time_us: self.stats.avg_ordering_time_us,
            memory_usage_bytes: self.stats.memory_usage_bytes,
        }
    }

    /// Compare performance between two snapshots
    pub fn compare_performance(before: &PerformanceSnapshot, after: &PerformanceSnapshot) -> PerformanceComparison {
        PerformanceComparison {
            cache_hit_rate_change: after.cache_hit_rate - before.cache_hit_rate,
            ordering_time_change: after.avg_ordering_time_us - before.avg_ordering_time_us,
            memory_usage_change: after.memory_usage_bytes as i64 - before.memory_usage_bytes as i64,
            is_improved: after.cache_hit_rate > before.cache_hit_rate 
                && after.avg_ordering_time_us < before.avg_ordering_time_us,
        }
    }

    // ==================== Advanced Integration ====================

    /// Integrate with opening book for enhanced move ordering
    /// 
    /// Uses opening book data to prioritize theoretically strong moves.
    pub fn integrate_with_opening_book(&mut self, book_moves: &[crate::opening_book::BookMove], board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> MoveOrderingResult<()> {
        if book_moves.is_empty() {
            return Ok(());
        }
        
        // Get the best book move by weight and evaluation
        if let Some(best_book_move) = book_moves.iter().max_by(|a, b| {
            match a.weight.cmp(&b.weight) {
                std::cmp::Ordering::Equal => a.evaluation.cmp(&b.evaluation),
                other => other,
            }
        }) {
            // Convert book move to regular move and set as PV
            let pv_move = best_book_move.to_engine_move(player);
            self.update_pv_move(board, captured_pieces, player, depth, pv_move, best_book_move.evaluation);
            
            // Update history for all book moves based on their weights
            for book_move in book_moves {
                let move_ = book_move.to_engine_move(player);
                let bonus = (book_move.weight / 100) as i32; // Convert weight to history bonus
                if let (Some(from), Some(to)) = (move_.from, Some(move_.to)) {
                    let from_idx = from.row as usize;
                    let to_idx = to.row as usize;
                    self.simple_history_table[from_idx][to_idx] += bonus;
                }
            }
            
            self.stats.opening_book_integrations += 1;
        }
        
        Ok(())
    }

    /// Integrate with endgame tablebase for enhanced move ordering
    /// 
    /// Uses tablebase data to prioritize moves leading to winning positions.
    pub fn integrate_with_tablebase(&mut self, tablebase_result: &crate::tablebase::TablebaseResult, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> MoveOrderingResult<()> {
        // If tablebase provides a best move, use it as PV
        if let Some(ref best_move) = tablebase_result.best_move {
            // Calculate score based on distance to mate
            let score = if let Some(dtm) = tablebase_result.distance_to_mate {
                if dtm > 0 {
                    10000 - dtm // Winning position
                } else {
                    -10000 - dtm // Losing position
                }
            } else {
                0 // Draw
            };
            
            self.update_pv_move(board, captured_pieces, player, depth, best_move.clone(), score);
            self.add_killer_move(best_move.clone());
            
            // Update history with high bonus for tablebase moves
            if let (Some(from), Some(to)) = (best_move.from, Some(best_move.to)) {
                let from_idx = from.row as usize;
                let to_idx = to.row as usize;
                let bonus = 1000; // High bonus for tablebase-recommended moves
                self.simple_history_table[from_idx][to_idx] += bonus;
            }
            
            self.stats.tablebase_integrations += 1;
        }
        
        Ok(())
    }

    /// Order moves for analysis mode
    /// 
    /// In analysis mode, we want to explore all reasonable moves thoroughly.
    pub fn order_moves_for_analysis(&mut self, moves: &[Move], board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }
        
        // Use all heuristics but with more balanced weights for exploration
        let mut analysis_ordered = self.order_moves_with_all_heuristics(moves, board, captured_pieces, player, depth);
        
        // In analysis mode, also consider quiet moves more
        analysis_ordered.sort_by(|a, b| {
            let score_a = self.score_move_for_analysis(a);
            let score_b = self.score_move_for_analysis(b);
            score_b.cmp(&score_a)
        });
        
        self.stats.analysis_orderings += 1;
        analysis_ordered
    }

    /// Score a move for analysis mode
    fn score_move_for_analysis(&mut self, move_: &Move) -> i32 {
        // Use regular scoring but with more balanced weights
        let mut score = self.score_move(move_).unwrap_or(0);
        
        // Boost quiet positional moves in analysis
        if !move_.is_capture && !move_.is_promotion {
            score += 50; // Small boost for quiet moves to ensure they're explored
        }
        
        score
    }

    /// Adjust move ordering based on time management
    /// 
    /// When time is limited, prioritize faster move evaluation.
    pub fn order_moves_with_time_management(&mut self, moves: &[Move], time_remaining_ms: u32, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }
        
        if time_remaining_ms < 1000 {
            // Low on time - use fast basic ordering
            self.order_moves(moves).unwrap_or_else(|_| moves.to_vec())
        } else if time_remaining_ms < 5000 {
            // Medium time - use PV and killer only
            self.order_moves_with_pv_and_killer(moves, board, captured_pieces, player, depth)
        } else {
            // Plenty of time - use all heuristics
            self.order_moves_with_all_heuristics(moves, board, captured_pieces, player, depth)
        }
    }

    /// Order moves for specific game phase
    /// 
    /// Adjusts move ordering based on game phase (opening, middlegame, endgame).
    pub fn order_moves_for_game_phase(&mut self, moves: &[Move], game_phase: GamePhase, board: &crate::bitboards::BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }
        
        // Temporarily adjust weights for the game phase
        let original_weights = self.config.weights.clone();
        
        match game_phase {
            GamePhase::Opening => {
                // Prioritize development and center control
                self.config.weights.development_weight = 500;
                self.config.weights.center_control_weight = 400;
                self.config.weights.position_value_weight = 300;
            },
            GamePhase::Middlegame => {
                // Balanced tactical and positional play
                self.config.weights.tactical_weight = 800;
                self.config.weights.capture_weight = 1200;
                self.config.weights.position_value_weight = 400;
            },
            GamePhase::Endgame => {
                // Focus on promotion and piece value
                self.config.weights.promotion_weight = 1000;
                self.config.weights.piece_value_weight = 800;
                self.config.weights.tactical_weight = 600;
            },
            GamePhase::Tactical => {
                // Prioritize forcing moves
                self.config.weights.capture_weight = 2000;
                self.config.weights.tactical_weight = 1500;
                self.config.weights.see_weight = 1200;
            },
            GamePhase::Positional => {
                // Prioritize positional moves
                self.config.weights.position_value_weight = 800;
                self.config.weights.center_control_weight = 600;
                self.config.weights.development_weight = 500;
            },
        }
        
        // Order with adjusted weights
        let ordered = self.order_moves_with_all_heuristics(moves, board, captured_pieces, player, depth);
        
        // Restore original weights
        self.config.weights = original_weights;
        
        self.stats.phase_specific_orderings += 1;
        ordered
    }

    /// Prepare move ordering for parallel search
    /// 
    /// Returns a configuration optimized for parallel/multi-threaded search.
    pub fn prepare_for_parallel_search(&mut self) -> ParallelSearchConfig {
        // For parallel search, we want thread-safe operations
        // and independent move orderers per thread
        
        ParallelSearchConfig {
            config: self.config.clone(),
            thread_safe_caches: false, // Each thread gets its own caches
            shared_history: true, // History can be shared (read-only during ordering)
            shared_pv: true, // PV can be shared
            shared_killers: false, // Killer moves are depth-specific, don't share
        }
    }

    /// Get statistics for advanced integrations
    pub fn get_advanced_integration_stats(&self) -> AdvancedIntegrationStats {
        AdvancedIntegrationStats {
            opening_book_integrations: self.stats.opening_book_integrations,
            tablebase_integrations: self.stats.tablebase_integrations,
            analysis_orderings: self.stats.analysis_orderings,
            phase_specific_orderings: self.stats.phase_specific_orderings,
        }
    }

    // ==================== WASM-Specific Optimizations ====================

    /// Get WASM-optimized configuration
    /// 
    /// Returns a configuration optimized for WebAssembly environments
    /// with reduced memory usage and simplified operations.
    #[cfg(target_arch = "wasm32")]
    pub fn wasm_optimized_config() -> MoveOrderingConfig {
        let mut config = MoveOrderingConfig::default();
        
        // Reduce cache sizes for WASM memory constraints
        config.cache_config.max_cache_size = 50000;
        config.cache_config.max_see_cache_size = 25000;
        config.cache_config.enable_auto_optimization = false;
        
        // Limit killer moves for memory efficiency
        config.killer_config.max_killer_moves_per_depth = 2;
        
        // Optimize for WASM performance
        config.performance_config.enable_performance_monitoring = false; // Reduce overhead
        
        config
    }

    /// Get native-optimized configuration
    /// 
    /// Returns a configuration optimized for native environments
    /// with larger caches and more features enabled.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_optimized_config() -> MoveOrderingConfig {
        let mut config = MoveOrderingConfig::default();
        
        // Larger caches for native environments
        config.cache_config.max_cache_size = 500000;
        config.cache_config.max_see_cache_size = 250000;
        config.cache_config.enable_auto_optimization = true;
        
        // More killer moves for better accuracy
        config.killer_config.max_killer_moves_per_depth = 3;
        
        // Enable all performance features
        config.performance_config.enable_performance_monitoring = true;
        config.performance_config.enable_memory_tracking = true;
        config.performance_config.enable_auto_optimization = true;
        
        config
    }

    /// Create WASM-optimized move orderer
    /// 
    /// Creates a move orderer specifically optimized for WASM environments.
    #[cfg(target_arch = "wasm32")]
    pub fn new_wasm_optimized() -> Self {
        Self::with_config(Self::wasm_optimized_config())
    }

    /// Create native-optimized move orderer
    /// 
    /// Creates a move orderer specifically optimized for native environments.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native_optimized() -> Self {
        Self::with_config(Self::native_optimized_config())
    }

    /// Get platform-specific configuration
    /// 
    /// Returns an optimized configuration for the current platform.
    pub fn platform_optimized_config() -> MoveOrderingConfig {
        #[cfg(target_arch = "wasm32")]
        {
            Self::wasm_optimized_config()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::native_optimized_config()
        }
    }

    /// Check if running in WASM environment
    pub fn is_wasm_environment() -> bool {
        cfg!(target_arch = "wasm32")
    }

    /// Get memory limits for current platform
    /// 
    /// Returns recommended memory limits based on platform.
    pub fn get_platform_memory_limits() -> PlatformMemoryLimits {
        #[cfg(target_arch = "wasm32")]
        {
            PlatformMemoryLimits {
                max_total_memory_bytes: 10 * 1024 * 1024, // 10 MB for WASM
                max_cache_size: 50000,
                max_see_cache_size: 25000,
                recommended_cache_size: 25000,
                recommended_see_cache_size: 10000,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            PlatformMemoryLimits {
                max_total_memory_bytes: 100 * 1024 * 1024, // 100 MB for native
                max_cache_size: 1000000,
                max_see_cache_size: 500000,
                recommended_cache_size: 200000,
                recommended_see_cache_size: 100000,
            }
        }
    }
}

impl Default for MoveOrdering {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Player, PieceType, Position};

    fn create_test_move(from: Option<Position>, to: Position, piece_type: PieceType, player: Player) -> Move {
        Move {
            from,
            to,
            piece_type,
            player,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
        }
    }

    #[test]
    fn test_move_ordering_creation() {
        let orderer = MoveOrdering::new();
        assert_eq!(orderer.stats.total_moves_ordered, 0);
        assert_eq!(orderer.move_score_cache.len(), 0);
    }

    #[test]
    fn test_move_ordering_with_config() {
        let weights = OrderingWeights {
            capture_weight: 2000,
            promotion_weight: 1500,
            ..Default::default()
        };
        let config = MoveOrderingConfig {
            weights,
            ..Default::default()
        };
        let orderer = MoveOrdering::with_config(config);
        assert_eq!(orderer.config.weights.capture_weight, 2000);
        assert_eq!(orderer.config.weights.promotion_weight, 1500);
    }

    #[test]
    fn test_order_moves_empty() {
        let mut orderer = MoveOrdering::new();
        let ordered = orderer.order_moves(&[]);
        assert!(ordered.is_empty());
        assert_eq!(orderer.stats.total_moves_ordered, 0);
    }

    #[test]
    fn test_order_moves_single() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        )];
        
        let ordered = orderer.order_moves(&moves);
        assert_eq!(ordered.len(), 1);
        assert_eq!(orderer.stats.total_moves_ordered, 1);
    }

    #[test]
    fn test_move_scoring() {
        let mut orderer = MoveOrdering::new();
        
        // Test capture move scoring
        let mut capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        capture_move.is_capture = true;
        capture_move.captured_piece = Some(Piece {
            piece_type: PieceType::Gold,
            player: Player::White,
        });
        
        let score = orderer.score_move(&capture_move);
        assert!(score > 0);
        
        // Test promotion move scoring
        let mut promotion_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(0, 1),
            PieceType::Pawn,
            Player::Black
        );
        promotion_move.is_promotion = true;
        
        let promotion_score = orderer.score_move(&promotion_move);
        assert!(promotion_score > 0);
    }

    #[test]
    fn test_position_value_scoring() {
        let orderer = MoveOrdering::new();
        
        let center_position = Position::new(4, 4);
        let edge_position = Position::new(0, 0);
        
        let center_score = orderer.score_position_value(&center_position);
        let edge_score = orderer.score_position_value(&edge_position);
        
        assert!(center_score > edge_score);
    }

    #[test]
    fn test_cache_functionality() {
        let mut orderer = MoveOrdering::new();
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // First scoring should be a cache miss
        let _ = orderer.score_move(&move_);
        assert_eq!(orderer.stats.cache_misses, 1);
        assert_eq!(orderer.stats.cache_hits, 0);
        
        // Second scoring should be a cache hit
        let _ = orderer.score_move(&move_);
        assert_eq!(orderer.stats.cache_misses, 1);
        assert_eq!(orderer.stats.cache_hits, 1);
        
        // Cache should contain the move
        assert_eq!(orderer.get_cache_size(), 1);
    }

    #[test]
    fn test_memory_tracking() {
        let mut orderer = MoveOrdering::new();
        
        // Initially no memory usage
        assert_eq!(orderer.memory_usage.current_bytes, 0);
        
        // Add some moves to trigger memory usage
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(2, 2)), Position::new(3, 2), PieceType::Silver, Player::Black),
        ];
        
        let _ = orderer.order_moves(&moves);
        
        // Memory usage should be updated
        assert!(orderer.memory_usage.current_bytes > 0);
        assert!(orderer.stats.memory_usage_bytes > 0);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_cache_size(2);
        
        // Add more moves than cache limit
        for i in 0..5 {
            let move_ = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::Black
            );
            let _ = orderer.score_move(&move_);
        }
        
        // Cache should not exceed limit
        assert!(orderer.get_cache_size() <= 2);
    }

    #[test]
    fn test_statistics_reset() {
        let mut orderer = MoveOrdering::new();
        
        // Add some data
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        )];
        let _ = orderer.order_moves(&moves);
        
        // Verify data exists
        assert_eq!(orderer.stats.total_moves_ordered, 1);
        assert!(orderer.get_cache_size() > 0);
        
        // Reset statistics
        orderer.reset_stats();
        
        // Verify reset
        assert_eq!(orderer.stats.total_moves_ordered, 0);
        assert_eq!(orderer.get_cache_size(), 0);
        assert_eq!(orderer.memory_usage.current_bytes, 0);
    }

    // ==================== PV Move Ordering Tests ====================

    #[test]
    fn test_pv_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_pv_move(&move_);
        assert_eq!(score, orderer.config.weights.pv_move_weight);
        assert!(score > 1000); // Should be higher than other move types
    }

    #[test]
    fn test_pv_move_cache_functionality() {
        let mut orderer = MoveOrdering::new();
        
        // Initially no PV moves cached
        assert_eq!(orderer.pv_move_cache.len(), 0);
        
        // Clear PV move cache
        orderer.clear_pv_move_cache();
        assert_eq!(orderer.pv_move_cache.len(), 0);
        assert_eq!(orderer.stats.pv_move_hits, 0);
        assert_eq!(orderer.stats.pv_move_misses, 0);
    }

    #[test]
    fn test_moves_equal_functionality() {
        let orderer = MoveOrdering::new();
        
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let move2 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let move3 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(3, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Same moves should be equal
        assert!(orderer.moves_equal(&move1, &move2));
        
        // Different moves should not be equal
        assert!(!orderer.moves_equal(&move1, &move3));
    }

    #[test]
    fn test_pv_move_statistics() {
        let mut orderer = MoveOrdering::new();
        
        // Initially no statistics
        let (hits, misses, hit_rate, tt_lookups, tt_hits) = orderer.get_pv_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(tt_lookups, 0);
        assert_eq!(tt_hits, 0);
        
        // Test transposition table hit rate
        let tt_hit_rate = orderer.get_tt_hit_rate();
        assert_eq!(tt_hit_rate, 0.0);
    }

    #[test]
    fn test_pv_move_ordering_without_transposition_table() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(2, 2)), Position::new(3, 2), PieceType::Silver, Player::Black),
        ];
        
        // Should work even without transposition table (falls back to regular ordering)
        let ordered = orderer.order_moves_with_pv(&moves, &board, &captured_pieces, Player::Black, 3);
        assert_eq!(ordered.len(), 2);
    }

    #[test]
    fn test_pv_move_weight_configuration() {
        let custom_weights = OrderingWeights {
            pv_move_weight: 50000,
            ..Default::default()
        };
        let config = MoveOrderingConfig {
            weights: custom_weights,
            ..Default::default()
        };
        let orderer = MoveOrdering::with_config(config);
        
        assert_eq!(orderer.config.weights.pv_move_weight, 50000);
    }

    #[test]
    fn test_pv_move_cache_size_limit() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_cache_size(2);
        
        // Add more entries than cache limit
        for i in 0..5 {
            let hash = i as u64;
            let move_ = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::Black
            );
            orderer.pv_move_cache.insert(hash, Some(move_));
        }
        
        // Cache should not exceed limit
        assert!(orderer.pv_move_cache.len() <= 2);
    }

    #[test]
    fn test_memory_usage_with_pv_cache() {
        let mut orderer = MoveOrdering::new();
        
        // Initially minimal memory usage
        let initial_memory = orderer.memory_usage.current_bytes;
        
        // Add some PV moves to cache
        for i in 0..10 {
            let hash = i as u64;
            let move_ = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::Black
            );
            orderer.pv_move_cache.insert(hash, Some(move_));
        }
        
        // Update memory usage
        orderer.update_memory_usage();
        
        // Memory usage should have increased
        assert!(orderer.memory_usage.current_bytes > initial_memory);
    }

    // ==================== Killer Move Heuristic Tests ====================

    #[test]
    fn test_killer_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_killer_move(&move_);
        assert_eq!(score, orderer.config.weights.killer_move_weight);
        assert!(score > 1000); // Should be higher than regular moves
    }

    #[test]
    fn test_killer_move_storage() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Initially no killer moves
        assert!(orderer.get_current_killer_moves().is_none());
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Should now have killer moves
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), 1);
        assert!(orderer.moves_equal(&killer_moves.unwrap()[0], &killer_move));
        
        // Statistics should be updated
        assert_eq!(orderer.stats.killer_moves_stored, 1);
    }

    #[test]
    fn test_killer_move_detection() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Test killer move detection
        assert!(orderer.is_killer_move(&killer_move));
        assert!(!orderer.is_killer_move(&regular_move));
    }

    #[test]
    fn test_depth_based_killer_move_management() {
        let mut orderer = MoveOrdering::new();
        
        let killer_move_1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let killer_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        // Add killer moves at different depths
        orderer.set_current_depth(1);
        orderer.add_killer_move(killer_move_1.clone());
        
        orderer.set_current_depth(2);
        orderer.add_killer_move(killer_move_2.clone());
        
        // Check killer moves at depth 1
        orderer.set_current_depth(1);
        assert!(orderer.is_killer_move(&killer_move_1));
        assert!(!orderer.is_killer_move(&killer_move_2));
        
        // Check killer moves at depth 2
        orderer.set_current_depth(2);
        assert!(!orderer.is_killer_move(&killer_move_1));
        assert!(orderer.is_killer_move(&killer_move_2));
    }

    #[test]
    fn test_killer_move_limit_per_depth() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_killer_moves_per_depth(2);
        orderer.set_current_depth(3);
        
        // Add more killer moves than the limit
        for i in 0..5 {
            let killer_move = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::Black
            );
            orderer.add_killer_move(killer_move);
        }
        
        // Should only have 2 killer moves (the limit)
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), 2);
    }

    #[test]
    fn test_killer_move_ordering() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Order moves
        let moves = vec![regular_move.clone(), killer_move.clone()];
        let ordered = orderer.order_moves_with_killer(&moves);
        
        // Killer move should be first
        assert_eq!(ordered.len(), 2);
        assert!(orderer.moves_equal(&ordered[0], &killer_move));
        assert!(orderer.moves_equal(&ordered[1], &regular_move));
    }

    #[test]
    fn test_killer_move_statistics() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        // Initially no statistics
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(stored, 0);
        
        // Add killer move
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        orderer.add_killer_move(killer_move.clone());
        
        // Test killer move detection (should increment hits)
        assert!(orderer.is_killer_move(&killer_move));
        
        // Statistics should be updated
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert!(hits > 0);
        assert!(stored > 0);
    }

    #[test]
    fn test_killer_move_clear_functionality() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        // Add killer moves
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        orderer.add_killer_move(killer_move.clone());
        
        // Verify killer move is stored
        assert!(orderer.is_killer_move(&killer_move));
        assert!(orderer.get_current_killer_moves().is_some());
        
        // Clear killer moves for current depth
        orderer.clear_killer_moves_for_depth(3);
        
        // Verify killer move is cleared
        assert!(!orderer.is_killer_move(&killer_move));
        assert!(orderer.get_current_killer_moves().is_none());
    }

    #[test]
    fn test_killer_move_clear_all() {
        let mut orderer = MoveOrdering::new();
        
        // Add killer moves at different depths
        orderer.set_current_depth(1);
        let killer_move_1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        orderer.add_killer_move(killer_move_1.clone());
        
        orderer.set_current_depth(2);
        let killer_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        orderer.add_killer_move(killer_move_2.clone());
        
        // Verify killer moves are stored
        orderer.set_current_depth(1);
        assert!(orderer.is_killer_move(&killer_move_1));
        orderer.set_current_depth(2);
        assert!(orderer.is_killer_move(&killer_move_2));
        
        // Clear all killer moves
        orderer.clear_all_killer_moves();
        
        // Verify all killer moves are cleared
        orderer.set_current_depth(1);
        assert!(!orderer.is_killer_move(&killer_move_1));
        orderer.set_current_depth(2);
        assert!(!orderer.is_killer_move(&killer_move_2));
        
        // Statistics should be reset
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(stored, 0);
    }

    #[test]
    fn test_pv_and_killer_move_combined_ordering() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black
        );
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Create test position and board
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;
        
        // Store PV move in transposition table
        orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move.clone(), 100);
        
        // Order moves with both PV and killer prioritization
        let moves = vec![regular_move.clone(), killer_move.clone(), pv_move.clone()];
        let ordered = orderer.order_moves_with_pv_and_killer(&moves, &board, &captured_pieces, player, depth);
        
        // PV move should be first, killer move second, regular move last
        assert_eq!(ordered.len(), 3);
        assert!(orderer.moves_equal(&ordered[0], &pv_move));
        assert!(orderer.moves_equal(&ordered[1], &killer_move));
        assert!(orderer.moves_equal(&ordered[2], &regular_move));
    }

    #[test]
    fn test_killer_move_configuration() {
        let custom_weights = OrderingWeights {
            killer_move_weight: 7500,
            ..Default::default()
        };
        let config = MoveOrderingConfig {
            weights: custom_weights,
            ..Default::default()
        };
        let orderer = MoveOrdering::with_config(config);
        
        assert_eq!(orderer.config.weights.killer_move_weight, 7500);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 2);
    }

    #[test]
    fn test_killer_move_max_per_depth_configuration() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_killer_moves_per_depth(5);
        
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 5);
        
        // Add more moves than the new limit
        orderer.set_current_depth(3);
        for i in 0..10 {
            let killer_move = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::Black
            );
            orderer.add_killer_move(killer_move);
        }
        
        // Should only have 5 killer moves (the new limit)
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), 5);
    }

    // ==================== History Heuristic Tests ====================

    #[test]
    fn test_history_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Initially no history score
        let score = orderer.score_history_move(&move_);
        assert_eq!(score, 0);
        
        // Add history score
        orderer.update_history_score(&move_, 3);
        
        // Should now have history score
        let score = orderer.score_history_move(&move_);
        assert!(score > 0);
        assert!(score < orderer.config.weights.history_weight);
    }

    #[test]
    fn test_history_score_update() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Initially no history score
        assert_eq!(orderer.get_history_score(&move_), 0);
        
        // Update history score
        orderer.update_history_score(&move_, 3);
        
        // Should now have history score
        let score = orderer.get_history_score(&move_);
        assert!(score > 0);
        assert_eq!(score, 9); // 3 * 3 = 9
        
        // Statistics should be updated
        assert_eq!(orderer.stats.history_updates, 1);
    }

    #[test]
    fn test_history_score_accumulation() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update history score multiple times
        orderer.update_history_score(&move_, 2);
        orderer.update_history_score(&move_, 3);
        orderer.update_history_score(&move_, 4);
        
        // Score should accumulate
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 4 + 9 + 16); // 2*2 + 3*3 + 4*4 = 29
        
        // Statistics should be updated
        assert_eq!(orderer.stats.history_updates, 3);
    }

    #[test]
    fn test_history_score_max_limit() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_history_score(100);
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update with large depth to exceed limit
        orderer.update_history_score(&move_, 20); // 20*20 = 400
        
        // Score should be capped at max
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_history_table_aging() {
        let mut orderer = MoveOrdering::new();
        orderer.set_history_aging_factor(0.5);
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Add history score
        orderer.update_history_score(&move_, 4); // 4*4 = 16
        assert_eq!(orderer.get_history_score(&move_), 16);
        
        // Age the table
        orderer.age_history_table();
        
        // Score should be reduced
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 8); // 16 * 0.5 = 8
        
        // Statistics should be updated
        assert_eq!(orderer.stats.history_aging_operations, 1);
    }

    #[test]
    fn test_history_table_aging_removes_zero_scores() {
        let mut orderer = MoveOrdering::new();
        orderer.set_history_aging_factor(0.1); // Very aggressive aging
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Add small history score
        orderer.update_history_score(&move_, 2); // 2*2 = 4
        assert_eq!(orderer.get_history_score(&move_), 4);
        
        // Age the table (should reduce to 0)
        orderer.age_history_table();
        
        // Score should be 0 and entry should be removed
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_history_move_ordering() {
        let mut orderer = MoveOrdering::new();
        
        let history_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        // Add history score to one move
        orderer.update_history_score(&history_move, 3);
        
        // Order moves
        let moves = vec![regular_move.clone(), history_move.clone()];
        let ordered = orderer.order_moves_with_history(&moves);
        
        // History move should be first
        assert_eq!(ordered.len(), 2);
        assert!(orderer.moves_equal(&ordered[0], &history_move));
        assert!(orderer.moves_equal(&ordered[1], &regular_move));
    }

    #[test]
    fn test_history_statistics() {
        let mut orderer = MoveOrdering::new();
        
        // Initially no statistics
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(updates, 0);
        assert_eq!(aging_ops, 0);
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update history score
        orderer.update_history_score(&move_, 3);
        
        // Test history move detection (should increment hits)
        orderer.score_history_move(&move_);
        
        // Test non-history move (should increment misses)
        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        orderer.score_history_move(&regular_move);
        
        // Statistics should be updated
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(updates, 1);
        assert_eq!(aging_ops, 0);
    }

    #[test]
    fn test_history_clear_functionality() {
        let mut orderer = MoveOrdering::new();
        
        // Add history scores
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        orderer.update_history_score(&move_, 3);
        
        // Verify history score is stored
        assert!(orderer.get_history_score(&move_) > 0);
        
        // Clear history table
        orderer.clear_history_table();
        
        // Verify history score is cleared
        assert_eq!(orderer.get_history_score(&move_), 0);
        
        // Statistics should be reset
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(updates, 0);
        assert_eq!(aging_ops, 0);
    }

    #[test]
    fn test_history_configuration() {
        let custom_weights = OrderingWeights {
            history_weight: 3000,
            ..Default::default()
        };
        let config = MoveOrderingConfig {
            weights: custom_weights,
            ..Default::default()
        };
        let mut orderer = MoveOrdering::with_config(config);
        
        assert_eq!(orderer.config.weights.history_weight, 3000);
        assert_eq!(orderer.get_max_history_score(), 10000);
        assert_eq!(orderer.get_history_aging_factor(), 0.9);
        
        // Test configuration changes
        orderer.set_max_history_score(5000);
        orderer.set_history_aging_factor(0.8);
        
        assert_eq!(orderer.get_max_history_score(), 5000);
        assert_eq!(orderer.get_history_aging_factor(), 0.8);
    }

    #[test]
    fn test_all_heuristics_combined_ordering() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        let history_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black
        );
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Add history score
        orderer.update_history_score(&history_move, 3);
        
        // Create test position and board
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;
        
        // Store PV move in transposition table
        orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move.clone(), 100);
        
        // Order moves with all heuristics
        let moves = vec![regular_move.clone(), history_move.clone(), killer_move.clone(), pv_move.clone()];
        let ordered = orderer.order_moves_with_all_heuristics(&moves, &board, &captured_pieces, player, depth);
        
        // PV move should be first, killer move second, history move third, regular move last
        assert_eq!(ordered.len(), 4);
        assert!(orderer.moves_equal(&ordered[0], &pv_move));
        assert!(orderer.moves_equal(&ordered[1], &killer_move));
        assert!(orderer.moves_equal(&ordered[2], &history_move));
        assert!(orderer.moves_equal(&ordered[3], &regular_move));
    }

    #[test]
    fn test_history_with_different_piece_types() {
        let mut orderer = MoveOrdering::new();
        
        let piece_types = vec![
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ];
        
        // Add history scores for different piece types
        for (i, piece_type) in piece_types.iter().enumerate() {
            let move_ = create_test_move(
                Some(Position::new(i as u8, 0)),
                Position::new((i + 1) as u8, 0),
                *piece_type,
                Player::Black
            );
            orderer.update_history_score(&move_, 2);
            assert!(orderer.get_history_score(&move_) > 0);
        }
        
        // Verify all history scores are stored
        assert_eq!(orderer.stats.history_updates, piece_types.len() as u64);
    }

    #[test]
    fn test_history_with_different_players() {
        let mut orderer = MoveOrdering::new();
        
        // Add history scores for both players
        let black_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let white_move = create_test_move(
            Some(Position::new(7, 7)),
            Position::new(6, 7),
            PieceType::Pawn,
            Player::White
        );
        
        orderer.update_history_score(&black_move, 3);
        orderer.update_history_score(&white_move, 4);
        
        // Both should have history scores
        assert!(orderer.get_history_score(&black_move) > 0);
        assert!(orderer.get_history_score(&white_move) > 0);
        
        // Verify statistics
        assert_eq!(orderer.stats.history_updates, 2);
    }

    // ==================== Move Scoring Integration Tests ====================

    #[test]
    fn test_comprehensive_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_move(&move_);
        assert!(score > 0);
        
        // Verify statistics are updated
        assert_eq!(orderer.stats.scoring_operations, 1);
        assert_eq!(orderer.stats.cache_misses, 1);
    }

    #[test]
    fn test_capture_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let mut capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        capture_move.is_capture = true;
        
        let score = orderer.score_move(&capture_move);
        assert!(score >= orderer.config.weights.capture_weight);
    }

    #[test]
    fn test_promotion_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let mut promotion_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        promotion_move.is_promotion = true;
        
        let score = orderer.score_move(&promotion_move);
        assert!(score >= orderer.config.weights.promotion_weight);
    }

    #[test]
    fn test_tactical_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let mut tactical_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        tactical_move.gives_check = true;
        
        let score = orderer.score_move(&tactical_move);
        assert!(score >= orderer.config.weights.tactical_weight);
    }

    #[test]
    fn test_piece_value_scoring() {
        let mut orderer = MoveOrdering::new();
        
        let pawn_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let rook_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Rook,
            Player::Black
        );
        
        let pawn_score = orderer.score_move(&pawn_move);
        let rook_score = orderer.score_move(&rook_move);
        
        // Rook should score higher than pawn due to piece value
        assert!(rook_score > pawn_score);
    }

    #[test]
    fn test_position_value_scoring_comprehensive() {
        let mut orderer = MoveOrdering::new();
        
        let center_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(4, 4), // Center position
            PieceType::Pawn,
            Player::Black
        );
        
        let edge_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(0, 0), // Edge position
            PieceType::Pawn,
            Player::Black
        );
        
        let center_score = orderer.score_move(&center_move);
        let edge_score = orderer.score_move(&edge_move);
        
        // Center move should score higher due to position value
        assert!(center_score > edge_score);
    }

    #[test]
    fn test_development_move_scoring() {
        let mut orderer = MoveOrdering::new();
        
        let development_move = create_test_move(
            Some(Position::new(1, 1)), // Back rank
            Position::new(3, 1), // Forward
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_move(&development_move);
        assert!(score > 0);
        
        // Should include development bonus
        assert!(score >= orderer.config.weights.development_weight / 100);
    }

    #[test]
    fn test_quiet_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let quiet_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_move(&quiet_move);
        assert!(score >= orderer.config.weights.quiet_weight);
    }

    #[test]
    fn test_move_scoring_cache() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // First call should miss cache
        let score1 = orderer.score_move(&move_);
        assert_eq!(orderer.stats.cache_misses, 1);
        assert_eq!(orderer.stats.cache_hits, 0);
        
        // Second call should hit cache
        let score2 = orderer.score_move(&move_);
        assert_eq!(score1, score2);
        assert_eq!(orderer.stats.cache_hits, 1);
        assert_eq!(orderer.stats.cache_misses, 1);
    }

    #[test]
    fn test_heuristic_weight_configuration() {
        let mut orderer = MoveOrdering::new();
        
        // Test individual weight setters
        orderer.set_capture_weight(2000);
        assert_eq!(orderer.config.weights.capture_weight, 2000);
        
        orderer.set_promotion_weight(1600);
        assert_eq!(orderer.config.weights.promotion_weight, 1600);
        
        orderer.set_tactical_weight(600);
        assert_eq!(orderer.config.weights.tactical_weight, 600);
        
        orderer.set_quiet_weight(50);
        assert_eq!(orderer.config.weights.quiet_weight, 50);
        
        // Test reset to defaults
        orderer.reset_config_to_default();
        assert_eq!(orderer.config.weights.capture_weight, 1000);
        assert_eq!(orderer.config.weights.promotion_weight, 800);
        assert_eq!(orderer.config.weights.tactical_weight, 300);
        assert_eq!(orderer.config.weights.quiet_weight, 25);
    }

    #[test]
    fn test_performance_optimization() {
        let mut orderer = MoveOrdering::new();
        
        // Test cache size configuration
        orderer.set_cache_size(500);
        assert_eq!(orderer.get_max_cache_size(), 500);
        
        // Test cache warming
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(2, 2)), Position::new(3, 2), PieceType::Silver, Player::Black),
        ];
        orderer.warm_up_cache(&moves);
        
        // Cache should be populated
        assert!(orderer.get_cache_size() > 0);
        
        // Test performance optimization
        orderer.optimize_performance();
        
        // Should still be functional
        assert!(orderer.get_max_cache_size() > 0);
    }

    #[test]
    fn test_scoring_statistics() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Initially no statistics
        let (ops, hits, hit_rate, misses, cache_size, max_cache_size) = orderer.get_scoring_stats();
        assert_eq!(ops, 0);
        assert_eq!(hits, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(misses, 0);
        assert_eq!(cache_size, 0);
        assert!(max_cache_size > 0);
        
        // Score move
        orderer.score_move(&move_);
        
        // Statistics should be updated
        let (ops, hits, hit_rate, misses, cache_size, max_cache_size) = orderer.get_scoring_stats();
        assert_eq!(ops, 1);
        assert_eq!(hits, 0);
        assert_eq!(misses, 1);
        assert_eq!(cache_size, 1);
        assert!(max_cache_size > 0);
        
        // Score same move again (should hit cache)
        orderer.score_move(&move_);
        
        let (ops, hits, hit_rate, misses, cache_size, max_cache_size) = orderer.get_scoring_stats();
        assert_eq!(ops, 2);
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert!(hit_rate > 0.0);
        assert_eq!(cache_size, 1);
    }

    #[test]
    fn test_comprehensive_move_types() {
        let mut orderer = MoveOrdering::new();
        
        // Test different move types
        let mut capture_promotion_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        capture_promotion_move.is_capture = true;
        capture_promotion_move.is_promotion = true;
        
        let mut tactical_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        tactical_move.gives_check = true;
        
        let quiet_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black
        );
        
        let capture_score = orderer.score_move(&capture_promotion_move);
        let tactical_score = orderer.score_move(&tactical_move);
        let quiet_score = orderer.score_move(&quiet_move);
        
        // Capture+promotion should score highest
        assert!(capture_score > tactical_score);
        assert!(capture_score > quiet_score);
        
        // Tactical should score higher than quiet
        assert!(tactical_score > quiet_score);
        
        // All scores should be positive
        assert!(capture_score > 0);
        assert!(tactical_score > 0);
        assert!(quiet_score > 0);
    }

    #[test]
    fn test_move_scoring_with_all_heuristics() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);
        
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black
        );
        
        let history_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black
        );
        
        let regular_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black
        );
        
        // Add killer move
        orderer.add_killer_move(killer_move.clone());
        
        // Add history score
        orderer.update_history_score(&history_move, 3);
        
        // Create test position and board
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;
        
        // Store PV move
        orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move.clone(), 100);
        
        // Test scoring with all heuristics
        let pv_score = orderer.score_move_with_all_heuristics(&pv_move, &Some(pv_move.clone()), &[killer_move.clone()]);
        let killer_score = orderer.score_move_with_all_heuristics(&killer_move, &Some(pv_move.clone()), &[killer_move.clone()]);
        let history_score = orderer.score_move_with_all_heuristics(&history_move, &Some(pv_move.clone()), &[killer_move.clone()]);
        let regular_score = orderer.score_move_with_all_heuristics(&regular_move, &Some(pv_move.clone()), &[killer_move.clone()]);
        
        // PV should score highest
        assert!(pv_score > killer_score);
        assert!(pv_score > history_score);
        assert!(pv_score > regular_score);
        
        // Killer should score higher than history
        assert!(killer_score > history_score);
        assert!(killer_score > regular_score);
        
        // History should score higher than regular
        assert!(history_score > regular_score);
    }

    // ==================== History Update Counter Tests ====================

    #[test]
    fn test_history_update_counter() {
        let mut orderer = MoveOrdering::new();
        
        // Initially counter should be 0
        assert_eq!(orderer.get_history_update_counter(), 0);
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update history score
        orderer.update_history_score(&move_, 3);
        
        // Counter should be incremented
        assert_eq!(orderer.get_history_update_counter(), 1);
        
        // Update again
        orderer.update_history_score(&move_, 2);
        
        // Counter should be incremented again
        assert_eq!(orderer.get_history_update_counter(), 2);
    }

    #[test]
    fn test_history_update_counter_reset() {
        let mut orderer = MoveOrdering::new();
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update history score multiple times
        orderer.update_history_score(&move_, 3);
        orderer.update_history_score(&move_, 2);
        orderer.update_history_score(&move_, 4);
        
        // Counter should be 3
        assert_eq!(orderer.get_history_update_counter(), 3);
        
        // Reset counter
        orderer.reset_history_update_counter();
        
        // Counter should be 0
        assert_eq!(orderer.get_history_update_counter(), 0);
        
        // Update again
        orderer.update_history_score(&move_, 1);
        
        // Counter should be 1
        assert_eq!(orderer.get_history_update_counter(), 1);
    }

    #[test]
    fn test_automatic_history_aging_with_counter() {
        let mut config = MoveOrderingConfig::new();
        config.history_config.enable_automatic_aging = true;
        config.history_config.aging_frequency = 5; // Age every 5 updates
        config.history_config.history_aging_factor = 0.8;
        
        let mut orderer = MoveOrdering::with_config(config);
        
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // Update history score 4 times (should not trigger aging)
        for i in 1..=4 {
            orderer.update_history_score(&move_, 3);
            assert_eq!(orderer.get_history_update_counter(), i);
        }
        
        // Score should accumulate without aging
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 4 * 9); // 4 updates * 3*3 = 36
        
        // 5th update should trigger automatic aging
        orderer.update_history_score(&move_, 3);
        assert_eq!(orderer.get_history_update_counter(), 5);
        
        // Score should be aged
        let aged_score = orderer.get_history_score(&move_);
        assert!(aged_score < score); // Should be reduced by aging
        assert_eq!(aged_score, (5 * 9) * 8 / 10); // (5*9) * 0.8 = 36
    }

    // ==================== SEE (Static Exchange Evaluation) Tests ====================

    #[test]
    fn test_see_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Test SEE scoring for capture move
        let see_score = orderer.score_see_move(&capture_move, &board);
        
        // SEE score should be non-negative for capture moves
        assert!(see_score >= 0);
        
        // Statistics should be updated
        assert_eq!(orderer.stats.see_calculations, 1);
    }

    #[test]
    fn test_see_calculation() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Test SEE calculation
        let see_value = orderer.calculate_see(&capture_move, &board);
        
        // SEE value should be calculated
        assert!(see_value >= 0);
        
        // Statistics should be updated
        assert_eq!(orderer.stats.see_calculations, 1);
    }

    #[test]
    fn test_see_cache_functionality() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Enable SEE cache
        orderer.set_see_cache_enabled(true);
        
        // First calculation should miss cache
        let see_value1 = orderer.calculate_see(&capture_move, &board);
        assert_eq!(orderer.stats.see_cache_misses, 1);
        assert_eq!(orderer.stats.see_cache_hits, 0);
        
        // Second calculation should hit cache
        let see_value2 = orderer.calculate_see(&capture_move, &board);
        assert_eq!(orderer.stats.see_cache_hits, 1);
        assert_eq!(see_value1, see_value2);
        
        // Cache should have one entry
        assert_eq!(orderer.get_see_cache_size(), 1);
    }

    #[test]
    fn test_see_cache_management() {
        let mut orderer = MoveOrdering::new();
        
        // Test cache clearing
        orderer.clear_see_cache();
        assert_eq!(orderer.get_see_cache_size(), 0);
        assert_eq!(orderer.stats.see_cache_hits, 0);
        assert_eq!(orderer.stats.see_cache_misses, 0);
        
        // Test cache size setting
        orderer.set_max_see_cache_size(100);
        assert_eq!(orderer.max_see_cache_size, 100);
        
        // Test cache disabling
        orderer.set_see_cache_enabled(false);
        assert!(!orderer.config.cache_config.enable_see_cache);
        assert_eq!(orderer.get_see_cache_size(), 0);
    }

    #[test]
    fn test_see_statistics() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Perform some SEE calculations
        orderer.calculate_see(&capture_move, &board);
        orderer.calculate_see(&capture_move, &board); // Should hit cache
        
        let (calculations, hits, misses, hit_rate, time_us, avg_time) = orderer.get_see_stats();
        
        assert_eq!(calculations, 2);
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert!(hit_rate >= 0.0);
        assert!(time_us > 0);
        assert!(avg_time > 0.0);
    }

    #[test]
    fn test_see_weight_configuration() {
        let mut orderer = MoveOrdering::new();
        
        // Test default SEE weight
        assert_eq!(orderer.config.weights.see_weight, 2000);
        
        // Test setting SEE weight
        orderer.set_see_weight(3000);
        assert_eq!(orderer.config.weights.see_weight, 3000);
    }

    #[test]
    fn test_see_cache_hit_rate() {
        let mut orderer = MoveOrdering::new();
        
        // Initially no hits or misses
        assert_eq!(orderer.get_see_cache_hit_rate(), 0.0);
        
        // After some cache operations, hit rate should be calculated
        orderer.set_see_cache_enabled(true);
        let board = crate::bitboards::BitboardBoard::new();
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Miss
        orderer.calculate_see(&capture_move, &board);
        // Hit
        orderer.calculate_see(&capture_move, &board);
        
        let hit_rate = orderer.get_see_cache_hit_rate();
        assert!(hit_rate > 0.0);
        assert!(hit_rate <= 100.0);
    }

    #[test]
    fn test_see_with_non_capture_move() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        
        let quiet_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        
        // SEE should return 0 for non-capture moves
        let see_score = orderer.score_see_move(&quiet_move, &board);
        assert_eq!(see_score, 0);
        
        let see_value = orderer.calculate_see(&quiet_move, &board);
        assert_eq!(see_value, 0);
    }

    #[test]
    fn test_see_cache_size_limits() {
        let mut orderer = MoveOrdering::new();
        
        // Set small cache size
        orderer.set_max_see_cache_size(2);
        
        // Enable cache
        orderer.set_see_cache_enabled(true);
        
        // Cache should not exceed maximum size
        assert!(orderer.get_see_cache_size() <= orderer.max_see_cache_size);
    }

    // ==================== Performance Optimization Tests ====================

    #[test]
    fn test_fast_hash_calculation() {
        let orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Test that fast hash calculation works
        let hash = orderer.get_move_hash_fast(&move_);
        assert!(hash > 0);
        
        // Test that hash is deterministic
        let hash2 = orderer.get_move_hash_fast(&move_);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_inline_scoring_functions() {
        let orderer = MoveOrdering::new();
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Test inline capture scoring
        let capture_score = orderer.score_capture_move_inline(&capture_move);
        assert!(capture_score >= 0);
        
        // Test inline promotion scoring
        let promotion_score = orderer.score_promotion_move_inline(&capture_move);
        assert!(promotion_score >= 0);
        
        // Test fast position scoring
        let position_score = orderer.score_position_value_fast(&capture_move);
        assert!(position_score >= 0);
        
        // Test fast development scoring
        let development_score = orderer.score_development_move_fast(&capture_move);
        assert!(development_score >= 0);
    }

    #[test]
    fn test_object_pooling() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(2, 1)), Position::new(3, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Test that object pools are used
        let ordered = orderer.order_moves(&moves);
        assert_eq!(ordered.len(), 2);
        
        // Test that pools are returned for reuse
        let ordered2 = orderer.order_moves(&moves);
        assert_eq!(ordered2.len(), 2);
    }

    #[test]
    fn test_fast_cache_performance() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // First call should populate fast cache
        let score1 = orderer.score_move(&move_);
        
        // Second call should hit fast cache
        let score2 = orderer.score_move(&move_);
        assert_eq!(score1, score2);
        
        // Fast cache should have entries
        assert!(!orderer.fast_score_cache.is_empty());
    }

    #[test]
    fn test_performance_benchmarking() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(2, 1)), Position::new(3, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Test move scoring benchmark
        let (total_time, avg_time) = orderer.benchmark_move_scoring(&moves, 10);
        assert!(total_time > 0);
        assert!(avg_time > 0.0);
        
        // Test move ordering benchmark
        let (total_time_ordering, avg_time_ordering) = orderer.benchmark_move_ordering(&moves, 5);
        assert!(total_time_ordering > 0);
        assert!(avg_time_ordering > 0.0);
        
        // Test cache performance benchmark
        let (hit_rate, cache_time) = orderer.benchmark_cache_performance(&moves, 5);
        assert!(hit_rate >= 0.0);
        assert!(hit_rate <= 100.0);
        assert!(cache_time > 0);
    }

    #[test]
    fn test_performance_statistics() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Perform some operations to generate statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test performance statistics
        let stats = orderer.get_performance_stats();
        assert!(stats.total_moves_ordered > 0);
        assert!(stats.cache_hit_rate >= 0.0);
        assert!(stats.cache_sizes.move_score_cache >= 0);
        assert!(stats.cache_sizes.fast_cache >= 0);
    }

    #[test]
    fn test_bottleneck_analysis() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Perform operations to generate performance data
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test bottleneck analysis
        let analysis = orderer.profile_bottlenecks();
        assert!(analysis.overall_score >= 0);
        assert!(analysis.overall_score <= 100);
        
        // Bottlenecks should be identified if performance is poor
        if analysis.overall_score < 80 {
            assert!(!analysis.bottlenecks.is_empty());
        }
    }

    #[test]
    fn test_hot_path_profiling() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Perform score_move operations to generate hot path data
        orderer.score_move(&move_);
        orderer.score_move(&move_);
        
        // Test hot path statistics
        assert!(orderer.stats.hot_path_stats.score_move_calls > 0);
        assert!(orderer.stats.hot_path_stats.cache_lookups > 0);
        assert!(orderer.stats.hot_path_stats.hash_calculations > 0);
    }

    #[test]
    fn test_center_distance_calculation() {
        let orderer = MoveOrdering::new();
        
        // Test center distance calculation
        let center = Position::new(4, 4);
        assert_eq!(orderer.get_center_distance_fast(center), 0);
        
        let corner = Position::new(0, 0);
        assert_eq!(orderer.get_center_distance_fast(corner), 8);
        
        let edge = Position::new(4, 0);
        assert_eq!(orderer.get_center_distance_fast(edge), 4);
    }

    // ==================== Advanced Statistics Tests ====================

    #[test]
    fn test_detailed_statistics_initialization() {
        let orderer = MoveOrdering::new();
        
        // Test that all statistics structures are initialized
        assert_eq!(orderer.stats.heuristic_stats.capture_stats.applications, 0);
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.operation_count, 0);
        assert_eq!(orderer.stats.memory_stats.current_usage.total_bytes, 0);
        assert_eq!(orderer.stats.cache_stats.move_score_cache.hits, 0);
    }

    #[test]
    fn test_heuristic_statistics_tracking() {
        let mut orderer = MoveOrdering::new();
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        // Test heuristic statistics tracking
        orderer.update_heuristic_stats("capture", true, 100);
        orderer.update_heuristic_stats("promotion", true, 200);
        orderer.update_heuristic_stats("tactical", true, 50);
        
        assert_eq!(orderer.stats.heuristic_stats.capture_stats.applications, 1);
        assert_eq!(orderer.stats.heuristic_stats.capture_stats.total_score_contribution, 100);
        assert_eq!(orderer.stats.heuristic_stats.promotion_stats.applications, 1);
        assert_eq!(orderer.stats.heuristic_stats.tactical_stats.applications, 1);
    }

    #[test]
    fn test_timing_statistics_tracking() {
        let mut orderer = MoveOrdering::new();
        
        // Test timing statistics recording
        orderer.record_timing("move_scoring", 100);
        orderer.record_timing("move_scoring", 200);
        orderer.record_timing("cache", 50);
        
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.operation_count, 2);
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.total_time_us, 300);
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.avg_time_us, 150.0);
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.min_time_us, 100);
        assert_eq!(orderer.stats.timing_stats.move_scoring_times.max_time_us, 200);
        
        assert_eq!(orderer.stats.timing_stats.cache_times.operation_count, 1);
        assert_eq!(orderer.stats.timing_stats.cache_times.total_time_us, 50);
    }

    #[test]
    fn test_cache_statistics_tracking() {
        let mut orderer = MoveOrdering::new();
        
        // Test cache statistics tracking
        orderer.update_cache_stats("move_score_cache", true, 100, 500);
        orderer.update_cache_stats("move_score_cache", false, 101, 500);
        orderer.update_cache_stats("fast_cache", true, 50, 100);
        
        assert_eq!(orderer.stats.cache_stats.move_score_cache.hits, 1);
        assert_eq!(orderer.stats.cache_stats.move_score_cache.misses, 1);
        assert_eq!(orderer.stats.cache_stats.move_score_cache.hit_rate, 50.0);
        assert_eq!(orderer.stats.cache_stats.move_score_cache.utilization, 20.2);
        
        assert_eq!(orderer.stats.cache_stats.fast_cache.hits, 1);
        assert_eq!(orderer.stats.cache_stats.fast_cache.hit_rate, 100.0);
    }

    #[test]
    fn test_statistics_export_json() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test JSON export
        let json_export = orderer.export_statistics_json();
        assert!(!json_export.is_empty());
        assert!(json_export.contains("ordering_stats"));
        assert!(json_export.contains("timestamp"));
    }

    #[test]
    fn test_statistics_export_csv() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test CSV export
        let csv_export = orderer.export_statistics_csv();
        assert!(!csv_export.is_empty());
        assert!(csv_export.contains("Metric,Value,Unit"));
        assert!(csv_export.contains("Total Moves Ordered"));
        assert!(csv_export.contains("Cache Hit Rate"));
    }

    #[test]
    fn test_performance_summary() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test performance summary
        let summary = orderer.export_performance_summary();
        assert!(summary.total_moves_ordered > 0);
        assert!(summary.performance_score >= 0);
        assert!(summary.performance_score <= 100);
        assert!(!summary.most_effective_heuristic.is_empty());
    }

    #[test]
    fn test_performance_report_generation() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test performance report generation
        let report = orderer.generate_performance_report();
        assert!(!report.is_empty());
        assert!(report.contains("MOVE ORDERING PERFORMANCE REPORT"));
        assert!(report.contains("OVERALL PERFORMANCE"));
        assert!(report.contains("CACHE PERFORMANCE"));
        assert!(report.contains("MEMORY USAGE"));
        assert!(report.contains("HEURISTIC EFFECTIVENESS"));
    }

    #[test]
    fn test_performance_chart_data() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test chart data generation
        let chart_data = orderer.generate_performance_chart_data();
        assert!(chart_data.cache_hit_rates.move_score_cache >= 0.0);
        assert!(chart_data.cache_hit_rates.move_score_cache <= 100.0);
        assert!(chart_data.heuristic_effectiveness.capture >= 0.0);
        assert!(chart_data.memory_usage_trend.current_mb >= 0.0);
        assert!(chart_data.timing_breakdown.move_scoring_avg_us >= 0.0);
    }

    #[test]
    fn test_performance_trend_analysis() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 1), PieceType::Pawn, Player::Black),
        ];
        
        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);
        
        // Test trend analysis
        let trend_analysis = orderer.analyze_performance_trends();
        assert!(trend_analysis.cache_efficiency_trend.confidence >= 0.0);
        assert!(trend_analysis.cache_efficiency_trend.confidence <= 1.0);
        assert!(!trend_analysis.cache_efficiency_trend.recommendation.is_empty());
        assert!(!trend_analysis.memory_usage_trend.recommendation.is_empty());
        assert!(!trend_analysis.heuristic_effectiveness_trend.recommendation.is_empty());
        assert!(!trend_analysis.timing_trend.recommendation.is_empty());
        assert!(!trend_analysis.overall_performance_trend.recommendation.is_empty());
    }

    #[test]
    fn test_most_effective_heuristic() {
        let mut orderer = MoveOrdering::new();
        
        // Set up some heuristic statistics
        orderer.stats.heuristic_stats.capture_stats.best_move_contributions = 10;
        orderer.stats.heuristic_stats.promotion_stats.best_move_contributions = 5;
        orderer.stats.heuristic_stats.tactical_stats.best_move_contributions = 15;
        
        // Test most effective heuristic identification
        let most_effective = orderer.get_most_effective_heuristic();
        assert_eq!(most_effective, "tactical");
    }

    #[test]
    fn test_heuristic_effectiveness_calculation() {
        let orderer = MoveOrdering::new();
        let mut stats = HeuristicPerformance::default();
        
        stats.applications = 100;
        stats.best_move_contributions = 30;
        
        // Test effectiveness calculation
        let effectiveness = orderer.calculate_heuristic_effectiveness(&stats);
        assert_eq!(effectiveness, 30.0);
    }

    #[test]
    fn test_memory_statistics_update() {
        let mut orderer = MoveOrdering::new();
        
        // Test memory statistics update
        orderer.update_memory_stats();
        
        assert!(orderer.stats.memory_stats.current_usage.total_bytes >= 0);
        assert!(orderer.stats.memory_stats.peak_usage.total_bytes >= 0);
    }

    #[test]
    fn test_best_move_contribution_recording() {
        let mut orderer = MoveOrdering::new();
        
        // Test best move contribution recording
        orderer.record_best_move_contribution("capture");
        orderer.record_best_move_contribution("capture");
        orderer.record_best_move_contribution("promotion");
        
        assert_eq!(orderer.stats.heuristic_stats.capture_stats.best_move_contributions, 2);
        assert_eq!(orderer.stats.heuristic_stats.promotion_stats.best_move_contributions, 1);
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_error_types() {
        // Test error creation and display
        let invalid_move_error = MoveOrderingError::InvalidMove("Test error".to_string());
        assert_eq!(format!("{}", invalid_move_error), "Invalid move: Test error");
        
        let cache_error = MoveOrderingError::CacheError("Cache full".to_string());
        assert_eq!(format!("{}", cache_error), "Cache error: Cache full");
        
        let see_error = MoveOrderingError::SEEError("SEE calculation failed".to_string());
        assert_eq!(format!("{}", see_error), "SEE calculation error: SEE calculation failed");
    }

    #[test]
    fn test_error_handler_functionality() {
        let mut error_handler = ErrorHandler::default();
        
        // Test error logging
        let error = MoveOrderingError::InvalidMove("Test error".to_string());
        error_handler.log_error(error.clone(), ErrorSeverity::Medium, "Test context".to_string());
        
        // Test recent errors retrieval
        let recent_errors = error_handler.get_recent_errors(5);
        assert_eq!(recent_errors.len(), 1);
        assert_eq!(recent_errors[0].error, error);
        assert_eq!(recent_errors[0].severity, ErrorSeverity::Medium);
        
        // Test error log clearing
        error_handler.clear_errors();
        assert_eq!(error_handler.get_recent_errors(5).len(), 0);
    }

    #[test]
    fn test_system_stability_check() {
        let mut error_handler = ErrorHandler::default();
        
        // Test with no errors - should be stable
        assert!(!error_handler.is_system_unstable());
        
        // Test with low severity errors - should still be stable
        for _ in 0..5 {
            error_handler.log_error(
                MoveOrderingError::InvalidMove("Low severity".to_string()),
                ErrorSeverity::Low,
                "Test".to_string()
            );
        }
        assert!(!error_handler.is_system_unstable());
        
        // Test with critical error - should be unstable
        error_handler.log_error(
            MoveOrderingError::MemoryError("Critical memory error".to_string()),
            ErrorSeverity::Critical,
            "Test".to_string()
        );
        assert!(error_handler.is_system_unstable());
    }

    #[test]
    fn test_move_validation() {
        let orderer = MoveOrdering::new();
        
        // Test valid move
        let valid_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        assert!(orderer.validate_move(&valid_move).is_ok());
        
        // Test invalid destination
        let mut invalid_move = valid_move.clone();
        invalid_move.to = Position::new(10, 10); // Invalid position
        assert!(orderer.validate_move(&invalid_move).is_err());
        
        // Test invalid source
        let mut invalid_move = valid_move.clone();
        invalid_move.from = Some(Position::new(10, 10)); // Invalid position
        assert!(orderer.validate_move(&invalid_move).is_err());
    }

    #[test]
    fn test_error_handling_in_score_move() {
        let mut orderer = MoveOrdering::new();
        
        // Test with invalid move
        let mut invalid_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        invalid_move.to = Position::new(10, 10); // Invalid position
        
        let result = orderer.score_move(&invalid_move);
        assert!(result.is_err());
        
        // Check that error was logged
        let errors = orderer.get_recent_errors(1);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error, MoveOrderingError::InvalidMove(_)));
    }

    #[test]
    fn test_error_handling_in_order_moves() {
        let mut orderer = MoveOrdering::new();
        
        // Test with mix of valid and invalid moves
        let moves = vec![
            create_test_move(Some(Position::new(1, 1)), Position::new(2, 2), PieceType::Pawn, Player::Black),
            create_test_move(Some(Position::new(10, 10)), Position::new(2, 2), PieceType::Pawn, Player::Black), // Invalid
            create_test_move(Some(Position::new(3, 3)), Position::new(4, 4), PieceType::Rook, Player::Black),
        ];
        
        let result = orderer.order_moves(&moves);
        assert!(result.is_err());
        
        // Check that error was logged
        let errors = orderer.get_recent_errors(1);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error, MoveOrderingError::InvalidMove(_)));
    }

    #[test]
    fn test_graceful_degradation() {
        let mut orderer = MoveOrdering::new();
        
        // Enable graceful degradation
        orderer.error_handler.graceful_degradation_enabled = true;
        
        // Test with low severity error
        let result = orderer.handle_error(
            MoveOrderingError::InvalidMove("Low severity error".to_string()),
            ErrorSeverity::Low,
            "Test context".to_string()
        );
        assert!(result.is_ok());
        
        // Test with high severity error
        let result = orderer.handle_error(
            MoveOrderingError::CacheError("Cache error".to_string()),
            ErrorSeverity::High,
            "Test context".to_string()
        );
        // Should attempt recovery and succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_recovery_mechanisms() {
        let mut orderer = MoveOrdering::new();
        
        // Test cache error recovery
        let result = orderer.attempt_error_recovery(&MoveOrderingError::CacheError("Test".to_string()));
        assert!(result.is_ok());
        // Caches should be cleared
        assert!(orderer.move_score_cache.is_empty());
        
        // Test memory error recovery
        let result = orderer.attempt_error_recovery(&MoveOrderingError::MemoryError("Test".to_string()));
        assert!(result.is_ok());
        // Memory usage should be reduced
        
        // Test statistics error recovery
        let result = orderer.attempt_error_recovery(&MoveOrderingError::StatisticsError("Test".to_string()));
        assert!(result.is_ok());
        // Statistics should be reset
        
        // Test unsupported error recovery
        let result = orderer.attempt_error_recovery(&MoveOrderingError::InvalidMove("Test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_logging_and_reporting() {
        let mut orderer = MoveOrdering::new();
        
        // Log some errors
        orderer.error_handler.log_error(
            MoveOrderingError::InvalidMove("Error 1".to_string()),
            ErrorSeverity::Medium,
            "Context 1".to_string()
        );
        
        orderer.error_handler.log_error(
            MoveOrderingError::CacheError("Error 2".to_string()),
            ErrorSeverity::High,
            "Context 2".to_string()
        );
        
        // Test error retrieval
        let errors = orderer.get_recent_errors(10);
        assert_eq!(errors.len(), 2);
        
        // Test error log clearing
        orderer.clear_error_log();
        assert_eq!(orderer.get_recent_errors(10).len(), 0);
    }

    #[test]
    fn test_error_state_detection() {
        let mut orderer = MoveOrdering::new();
        
        // Initially should not be in error state
        assert!(!orderer.is_in_error_state());
        
        // Add critical error
        orderer.error_handler.log_error(
            MoveOrderingError::MemoryError("Critical error".to_string()),
            ErrorSeverity::Critical,
            "Test".to_string()
        );
        
        // Should now be in error state
        assert!(orderer.is_in_error_state());
    }

    // ==================== Memory Management Tests ====================

    #[test]
    fn test_memory_pool_functionality() {
        let mut pool = MemoryPool::default();
        
        // Test move vector pool
        let mut vec1 = pool.get_move_vec();
        vec1.push(create_test_move(Some(Position::new(1, 1)), Position::new(2, 2), PieceType::Pawn, Player::Black));
        pool.return_move_vec(vec1);
        
        let stats = pool.get_pool_stats();
        assert_eq!(stats.move_vec_count, 1);
        
        // Test move score vector pool
        let mut vec2 = pool.get_move_score_vec();
        vec2.push((100, 0));
        pool.return_move_score_vec(vec2);
        
        let stats = pool.get_pool_stats();
        assert_eq!(stats.move_score_vec_count, 1);
        
        // Test pool clearing
        pool.clear_all_pools();
        let stats = pool.get_pool_stats();
        assert_eq!(stats.move_vec_count, 0);
        assert_eq!(stats.move_score_vec_count, 0);
    }

    #[test]
    fn test_memory_tracker_functionality() {
        let mut tracker = MemoryTracker::default();
        
        // Test allocation recording
        tracker.record_allocation(AllocationType::MoveVector, 1024, "test".to_string());
        tracker.record_allocation(AllocationType::Cache, 2048, "test".to_string());
        
        let current_usage = tracker.get_current_usage();
        assert!(current_usage.total_memory > 0);
        
        // Test deallocation recording
        tracker.record_deallocation(AllocationType::MoveVector, 1024, "test".to_string());
        
        // Test threshold checking
        let status = tracker.check_thresholds();
        assert_eq!(status, MemoryThresholdStatus::Normal);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut tracker = MemoryTracker::default();
        
        // Record an allocation without deallocation
        tracker.record_allocation(AllocationType::MoveVector, 1024, "test".to_string());
        
        // Check for leaks (should detect the allocation as potential leak)
        let leaks = tracker.check_for_leaks();
        assert!(!leaks.is_empty());
        
        // Record deallocation
        tracker.record_deallocation(AllocationType::MoveVector, 1024, "test".to_string());
        
        // Check again (should not detect leaks now)
        let leaks = tracker.check_for_leaks();
        assert!(leaks.is_empty());
    }

    #[test]
    fn test_memory_usage_monitoring() {
        let mut orderer = MoveOrdering::new();
        
        // Test initial memory usage
        let initial_usage = orderer.get_current_memory_usage();
        assert!(initial_usage.total_memory >= 0);
        
        // Test memory pool statistics
        let pool_stats = orderer.get_memory_pool_stats();
        assert_eq!(pool_stats.move_vec_count, 0);
        assert_eq!(pool_stats.move_score_vec_count, 0);
        
        // Test memory threshold checking
        let status = orderer.check_memory_thresholds();
        assert_eq!(status, MemoryThresholdStatus::Normal);
    }

    #[test]
    fn test_memory_cleanup() {
        let mut orderer = MoveOrdering::new();
        
        // Get initial memory usage
        let initial_usage = orderer.get_current_memory_usage().clone();
        
        // Perform cleanup
        let cleanup_report = orderer.cleanup_memory();
        
        // Verify cleanup was successful
        assert!(cleanup_report.cleanup_successful);
        assert!(cleanup_report.memory_freed >= 0);
        
        // Verify memory usage decreased or stayed the same
        let final_usage = orderer.get_current_memory_usage();
        assert!(final_usage.total_memory <= initial_usage.total_memory);
    }

    #[test]
    fn test_selective_memory_cleanup() {
        let mut orderer = MoveOrdering::new();
        
        // Test different pressure levels
        let low_pressure_report = orderer.selective_cleanup(MemoryPressureLevel::Low);
        assert!(low_pressure_report.cleanup_successful);
        
        let medium_pressure_report = orderer.selective_cleanup(MemoryPressureLevel::Medium);
        assert!(medium_pressure_report.cleanup_successful);
        
        let high_pressure_report = orderer.selective_cleanup(MemoryPressureLevel::High);
        assert!(high_pressure_report.cleanup_successful);
        
        let critical_pressure_report = orderer.selective_cleanup(MemoryPressureLevel::Critical);
        assert!(critical_pressure_report.cleanup_successful);
    }

    #[test]
    fn test_memory_leak_reporting() {
        let mut orderer = MoveOrdering::new();
        
        // Perform leak detection
        let leak_report = orderer.detect_memory_leaks();
        
        // Verify report structure
        assert!(!leak_report.leak_detected || !leak_report.warnings.is_empty());
        assert!(leak_report.current_usage.total_memory >= 0);
        assert!(leak_report.peak_usage.total_memory >= 0);
    }

    #[test]
    fn test_memory_pool_integration() {
        let mut orderer = MoveOrdering::new();
        
        // Test memory pool access
        let pool = orderer.get_memory_pool();
        let stats = pool.get_pool_stats();
        assert_eq!(stats.move_vec_count, 0);
        
        // Test mutable access
        let mut pool = orderer.get_memory_pool_mut();
        let vec = pool.get_move_vec();
        pool.return_move_vec(vec);
        
        let stats = pool.get_pool_stats();
        assert_eq!(stats.move_vec_count, 1);
    }

    #[test]
    fn test_memory_tracker_integration() {
        let mut orderer = MoveOrdering::new();
        
        // Test memory tracker access
        let tracker = orderer.get_memory_tracker();
        let usage = tracker.get_current_usage();
        assert!(usage.total_memory >= 0);
        
        // Test mutable access
        let mut tracker = orderer.get_memory_tracker_mut();
        tracker.record_allocation(AllocationType::MoveVector, 1024, "test".to_string());
        
        let usage = tracker.get_current_usage();
        assert!(usage.total_memory > 0);
    }

    #[test]
    fn test_memory_allocation_history() {
        let mut orderer = MoveOrdering::new();
        
        // Test allocation history
        let history = orderer.get_allocation_history();
        assert!(history.is_empty());
        
        // Record some allocations
        orderer.memory_tracker.record_allocation(AllocationType::MoveVector, 1024, "test".to_string());
        orderer.memory_tracker.record_allocation(AllocationType::Cache, 2048, "test".to_string());
        
        let history = orderer.get_allocation_history();
        assert_eq!(history.len(), 2);
        
        // Clear history
        orderer.clear_memory_history();
        let history = orderer.get_allocation_history();
        assert!(history.is_empty());
    }

    #[test]
    fn test_memory_leak_detection_control() {
        let mut orderer = MoveOrdering::new();
        
        // Test leak detection enable/disable
        orderer.set_leak_detection(false);
        // Should not detect leaks when disabled
        
        orderer.set_leak_detection(true);
        // Should detect leaks when enabled
    }

    // ==================== Advanced Features Tests ====================

    #[test]
    fn test_position_specific_strategies() {
        let mut orderer = MoveOrdering::new();
        
        // Test game phase determination
        let opening_phase = orderer.determine_game_phase(10, 0, 0.3);
        assert_eq!(opening_phase, GamePhase::Opening);
        
        let endgame_phase = orderer.determine_game_phase(70, 0, 0.3);
        assert_eq!(endgame_phase, GamePhase::Endgame);
        
        let tactical_phase = orderer.determine_game_phase(30, 0, 0.8);
        assert_eq!(tactical_phase, GamePhase::Tactical);
        
        let positional_phase = orderer.determine_game_phase(30, 50, 0.2);
        assert_eq!(positional_phase, GamePhase::Positional);
        
        let middlegame_phase = orderer.determine_game_phase(30, 0, 0.5);
        assert_eq!(middlegame_phase, GamePhase::Middlegame);
    }

    #[test]
    fn test_game_phase_update() {
        let mut orderer = MoveOrdering::new();
        
        // Test phase update
        orderer.update_game_phase(10, 0, 0.3);
        assert_eq!(orderer.advanced_features.position_strategies.current_phase, GamePhase::Opening);
        
        orderer.update_game_phase(70, 0, 0.3);
        assert_eq!(orderer.advanced_features.position_strategies.current_phase, GamePhase::Endgame);
    }

    #[test]
    fn test_strategy_scoring() {
        let mut orderer = MoveOrdering::new();
        
        // Test move scoring with strategy
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        
        let score = orderer.score_move_with_strategy(&move_);
        assert!(score.is_ok());
    }

    #[test]
    fn test_move_classification() {
        let orderer = MoveOrdering::new();
        
        // Test development move
        let development_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        assert!(orderer.is_development_move(&development_move));
        
        // Test center move
        let center_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(4, 4),
            PieceType::Pawn,
            Player::Black
        );
        assert!(orderer.is_center_move(&center_move));
        
        // Test king safety move
        let king_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::King,
            Player::Black
        );
        assert!(orderer.is_king_safety_move(&king_move));
    }

    #[test]
    fn test_machine_learning_model() {
        let mut orderer = MoveOrdering::new();
        
        // Enable ML model
        orderer.advanced_features.ml_model.enabled = true;
        
        // Test training
        let training_examples = vec![
            TrainingExample {
                features: vec![1.0, 2.0, 3.0],
                target: 100.0,
                context: PositionContext {
                    phase: GamePhase::Middlegame,
                    material_balance: 0,
                    king_safety: 0,
                    center_control: 0,
                },
            },
        ];
        
        let accuracy = orderer.train_ml_model(training_examples);
        assert!(accuracy.is_ok());
        assert!(accuracy.unwrap() > 0.0);
        
        // Test prediction
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black
        );
        let features = vec![1.0, 2.0, 3.0];
        let prediction = orderer.predict_move_score(&move_, features);
        assert!(prediction.is_ok());
    }

    #[test]
    fn test_dynamic_weight_adjustment() {
        let mut orderer = MoveOrdering::new();
        
        // Enable dynamic weights
        orderer.advanced_features.dynamic_weights.enabled = true;
        
        // Test weight adjustment
        let result = orderer.adjust_weights_dynamically(0.8);
        assert!(result.is_ok());
        
        // Check that adjustment was recorded
        assert!(!orderer.advanced_features.dynamic_weights.adjustment_history.is_empty());
    }

    #[test]
    fn test_advanced_features_control() {
        let mut orderer = MoveOrdering::new();
        
        // Test enabling features
        let features = AdvancedFeatureFlags {
            machine_learning: true,
            dynamic_weights: true,
            predictive_ordering: true,
            cache_warming: true,
            position_specific_strategies: true,
        };
        
        orderer.set_advanced_features_enabled(features);
        
        // Test status
        let status = orderer.get_advanced_features_status();
        assert!(status.machine_learning);
        assert!(status.dynamic_weights);
        assert!(status.predictive_ordering);
        assert!(status.cache_warming);
        assert!(status.position_specific_strategies);
    }

    #[test]
    fn test_ordering_strategies() {
        // Test opening strategy
        let opening_strategy = OrderingStrategy::opening();
        assert_eq!(opening_strategy.name, "Opening");
        assert!(opening_strategy.heuristic_preferences.prefer_development);
        assert!(opening_strategy.heuristic_preferences.prefer_positional);
        
        // Test middlegame strategy
        let middlegame_strategy = OrderingStrategy::middlegame();
        assert_eq!(middlegame_strategy.name, "Middlegame");
        assert!(middlegame_strategy.heuristic_preferences.prefer_tactical);
        
        // Test endgame strategy
        let endgame_strategy = OrderingStrategy::endgame();
        assert_eq!(endgame_strategy.name, "Endgame");
        assert!(endgame_strategy.heuristic_preferences.prefer_endgame);
        
        // Test tactical strategy
        let tactical_strategy = OrderingStrategy::tactical();
        assert_eq!(tactical_strategy.name, "Tactical");
        assert!(tactical_strategy.heuristic_preferences.prefer_tactical);
        
        // Test positional strategy
        let positional_strategy = OrderingStrategy::positional();
        assert_eq!(positional_strategy.name, "Positional");
        assert!(positional_strategy.heuristic_preferences.prefer_positional);
    }

    #[test]
    fn test_advanced_features_integration() {
        let mut orderer = MoveOrdering::new();
        
        // Test that advanced features are initialized
        let features = orderer.get_advanced_features();
        assert_eq!(features.position_strategies.current_phase, GamePhase::Middlegame);
        
        // Test mutable access
        let mut features = orderer.get_advanced_features_mut();
        features.ml_model.enabled = true;
        
        // Verify change
        assert!(orderer.advanced_features.ml_model.enabled);
    }

    #[test]
    fn test_priority_adjustments() {
        let mut orderer = MoveOrdering::new();
        
        // Test priority adjustments
        let strategy = &orderer.advanced_features.position_strategies.opening_strategy;
        
        // Opening strategy should favor development
        assert!(strategy.priority_adjustments.development_priority > 1.0);
        
        // Test tactical strategy
        let tactical_strategy = &orderer.advanced_features.position_strategies.tactical_strategy;
        assert!(tactical_strategy.priority_adjustments.capture_priority > 1.0);
    }

    #[test]
    fn test_heuristic_preferences() {
        let orderer = MoveOrdering::new();
        
        // Test opening preferences
        let opening_strategy = &orderer.advanced_features.position_strategies.opening_strategy;
        assert!(opening_strategy.heuristic_preferences.prefer_development);
        assert!(opening_strategy.heuristic_preferences.prefer_positional);
        assert!(!opening_strategy.heuristic_preferences.prefer_tactical);
        
        // Test tactical preferences
        let tactical_strategy = &orderer.advanced_features.position_strategies.tactical_strategy;
        assert!(tactical_strategy.heuristic_preferences.prefer_tactical);
        assert!(!tactical_strategy.heuristic_preferences.prefer_development);
    }

    // ==================== Configuration System Tests ====================

    #[test]
    fn test_configuration_creation() {
        let config = MoveOrderingConfig::new();
        
        // Test default values
        assert_eq!(config.weights.capture_weight, 1000);
        assert_eq!(config.cache_config.max_cache_size, 1000);
        assert_eq!(config.killer_config.max_killer_moves_per_depth, 2);
        assert_eq!(config.history_config.max_history_score, 10000);
    }

    #[test]
    fn test_configuration_validation() {
        let mut config = MoveOrderingConfig::new();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid capture weight should fail
        config.weights.capture_weight = -1;
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("Capture weight")));
        }
        
        // Invalid cache size should fail
        config.weights.capture_weight = 1000; // Fix previous error
        config.cache_config.max_cache_size = 0;
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("Max cache size")));
        }
    }

    #[test]
    fn test_performance_optimized_configuration() {
        let config = MoveOrderingConfig::performance_optimized();
        
        // Should have optimized settings
        assert_eq!(config.cache_config.max_cache_size, 5000);
        assert!(config.cache_config.enable_cache_warming);
        assert_eq!(config.killer_config.max_killer_moves_per_depth, 3);
        assert_eq!(config.history_config.max_history_score, 15000);
        assert!(!config.debug_config.enable_debug_logging);
    }

    #[test]
    fn test_debug_optimized_configuration() {
        let config = MoveOrderingConfig::debug_optimized();
        
        // Should have debug settings
        assert_eq!(config.cache_config.max_cache_size, 500);
        assert!(!config.cache_config.enable_cache_warming);
        assert_eq!(config.killer_config.max_killer_moves_per_depth, 1);
        assert!(config.debug_config.enable_debug_logging);
        assert_eq!(config.debug_config.log_level, 3);
    }

    #[test]
    fn test_memory_optimized_configuration() {
        let config = MoveOrderingConfig::memory_optimized();
        
        // Should have minimal memory settings
        assert_eq!(config.cache_config.max_cache_size, 100);
        assert!(!config.cache_config.enable_cache_warming);
        assert_eq!(config.killer_config.max_killer_moves_per_depth, 1);
        assert_eq!(config.history_config.max_history_score, 1000);
        assert!(!config.debug_config.enable_debug_logging);
    }

    #[test]
    fn test_configuration_merge() {
        let base_config = MoveOrderingConfig::new();
        let mut override_config = MoveOrderingConfig::new();
        override_config.weights.capture_weight = 2000;
        override_config.cache_config.max_cache_size = 2000;
        
        let merged_config = base_config.merge(&override_config);
        
        // Should use override values
        assert_eq!(merged_config.weights.capture_weight, 2000);
        assert_eq!(merged_config.cache_config.max_cache_size, 2000);
        
        // Should keep other default values
        assert_eq!(merged_config.weights.promotion_weight, 800);
    }

    #[test]
    fn test_move_ordering_with_configuration() {
        let mut config = MoveOrderingConfig::new();
        config.weights.capture_weight = 2000;
        config.cache_config.max_cache_size = 500;
        
        let mut orderer = MoveOrdering::with_config(config);
        
        // Should use custom configuration
        assert_eq!(orderer.get_weights().capture_weight, 2000);
        assert_eq!(orderer.get_max_cache_size(), 500);
        
        // Test move scoring with custom weights
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black
        );
        let score = orderer.score_move(&capture_move);
        assert!(score >= 2000);
    }

    #[test]
    fn test_configuration_updates() {
        let mut orderer = MoveOrdering::new();
        
        // Test weight updates
        orderer.set_capture_weight(3000);
        assert_eq!(orderer.get_weights().capture_weight, 3000);
        
        orderer.set_promotion_weight(2500);
        assert_eq!(orderer.get_weights().promotion_weight, 2500);
        
        // Test cache configuration updates
        orderer.set_cache_size(1500);
        assert_eq!(orderer.get_max_cache_size(), 1500);
        
        // Test killer move configuration updates
        orderer.set_max_killer_moves_per_depth(4);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 4);
    }

    #[test]
    fn test_configuration_validation_in_move_ordering() {
        let mut invalid_config = MoveOrderingConfig::new();
        invalid_config.weights.capture_weight = -1; // Invalid
        
        let result = MoveOrdering::with_config(invalid_config.clone()).set_config(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_configuration_application() {
        let mut config = MoveOrderingConfig::new();
        config.cache_config.max_cache_size = 100;
        
        let mut orderer = MoveOrdering::with_config(config);
        
        // Fill cache beyond new limit
        for i in 0..150 {
            let move_ = create_test_move(
                Some(Position::new(i % 9, i / 9)),
                Position::new((i + 1) % 9, (i + 1) / 9),
                PieceType::Pawn,
                Player::Black
            );
            orderer.score_move(&move_);
        }
        
        // Apply configuration changes (should trim cache)
        let new_config = MoveOrderingConfig::new();
        orderer.set_config(new_config).unwrap();
        
        // Cache should be trimmed to new size
        assert!(orderer.get_cache_size() <= 1000); // Default cache size
    }

    #[test]
    fn test_specialized_constructors() {
        // Test performance optimized constructor
        let perf_orderer = MoveOrdering::performance_optimized();
        assert_eq!(perf_orderer.get_max_cache_size(), 5000);
        
        // Test debug optimized constructor
        let debug_orderer = MoveOrdering::debug_optimized();
        assert_eq!(debug_orderer.get_max_cache_size(), 500);
        
        // Test memory optimized constructor
        let memory_orderer = MoveOrdering::memory_optimized();
        assert_eq!(memory_orderer.get_max_cache_size(), 100);
    }

    #[test]
    fn test_configuration_reset() {
        let mut orderer = MoveOrdering::new();
        
        // Modify configuration
        orderer.set_capture_weight(5000);
        orderer.set_cache_size(2000);
        
        // Reset to defaults
        orderer.reset_config_to_default();
        
        // Should be back to defaults
        assert_eq!(orderer.get_weights().capture_weight, 1000);
        assert_eq!(orderer.get_max_cache_size(), 1000);
    }

    #[test]
    fn test_configuration_getters() {
        let config = MoveOrderingConfig::new();
        let orderer = MoveOrdering::with_config(config);
        
        // Test configuration getter
        let retrieved_config = orderer.get_config();
        assert_eq!(retrieved_config.weights.capture_weight, 1000);
        
        // Test weights getter
        let weights = orderer.get_weights();
        assert_eq!(weights.capture_weight, 1000);
    }

    // ==================== Transposition Table Integration Tests ====================

    #[test]
    fn test_transposition_table_integration() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Test with no transposition table entry
        let result = orderer.integrate_with_transposition_table(None, &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        // Test with transposition table entry
        let tt_entry = TranspositionEntry {
            score: 100,
            depth: 3,
            flag: TranspositionFlag::Exact,
            best_move: Some(Move::new(Some(Position::new(4, 4)), Position::new(4, 3), PieceType::Pawn, false, false, false, false)),
            hash_key: 12345,
            age: 1,
        };

        let result = orderer.integrate_with_transposition_table(Some(&tt_entry), &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        // Check that statistics were updated
        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 1);
        assert_eq!(stats.tt_integration_updates, 1);
    }

    #[test]
    fn test_pv_move_from_tt() {
        let orderer = MoveOrdering::new();
        
        // Test with no TT entry
        let pv_move = orderer.get_pv_move_from_tt(None);
        assert!(pv_move.is_none());

        // Test with TT entry containing best move
        let tt_entry = TranspositionEntry {
            score: 50,
            depth: 2,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(Move::new(Some(Position::new(3, 3)), Position::new(3, 2), PieceType::Silver, false, false, false, false)),
            hash_key: 67890,
            age: 2,
        };

        let pv_move = orderer.get_pv_move_from_tt(Some(&tt_entry));
        assert!(pv_move.is_some());
        assert_eq!(pv_move.unwrap().from.unwrap(), Position::new(3, 3));
    }

    #[test]
    fn test_update_ordering_from_tt_result() {
        let mut orderer = MoveOrdering::new();
        
        let tt_entry = TranspositionEntry {
            score: 75,
            depth: 4,
            flag: TranspositionFlag::Exact,
            best_move: Some(Move::new(Some(Position::new(5, 5)), Position::new(5, 4), PieceType::Gold, false, false, false, false)),
            hash_key: 11111,
            age: 3,
        };

        // Test cutoff result
        let result = orderer.update_ordering_from_tt_result(&tt_entry, MoveResult::Cutoff);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_cutoff_updates, 1);

        // Test exact result
        let result = orderer.update_ordering_from_tt_result(&tt_entry, MoveResult::Exact);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_exact_updates, 1);
    }

    #[test]
    fn test_transposition_table_integration_performance() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 4;

        // Create multiple transposition table entries for performance testing
        let tt_entries = vec![
            TranspositionEntry {
                score: 100,
                depth: 4,
                flag: TranspositionFlag::Exact,
                best_move: Some(Move::new(Some(Position::new(4, 4)), Position::new(4, 3), PieceType::Pawn, false, false, false, false)),
                hash_key: 12345,
                age: 1,
            },
            TranspositionEntry {
                score: -50,
                depth: 3,
                flag: TranspositionFlag::LowerBound,
                best_move: Some(Move::new(Some(Position::new(3, 3)), Position::new(3, 2), PieceType::Silver, false, false, false, false)),
                hash_key: 67890,
                age: 2,
            },
            TranspositionEntry {
                score: 25,
                depth: 2,
                flag: TranspositionFlag::UpperBound,
                best_move: Some(Move::new(Some(Position::new(5, 5)), Position::new(5, 4), PieceType::Gold, false, false, false, false)),
                hash_key: 11111,
                age: 3,
            },
        ];

        // Performance test: integrate multiple TT entries
        let start_time = TimeSource::now();
        for entry in &tt_entries {
            let result = orderer.integrate_with_transposition_table(Some(entry), &board, &captured_pieces, player, depth);
            assert!(result.is_ok());
        }
        let elapsed_ms = start_time.elapsed_ms();

        // Verify performance is reasonable (should be fast)
        assert!(elapsed_ms < 100, "TT integration took {}ms, should be < 100ms", elapsed_ms);

        // Verify statistics were updated correctly
        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 3);
        assert_eq!(stats.tt_integration_updates, 3);
    }

    #[test]
    fn test_transposition_table_pv_move_performance() {
        let orderer = MoveOrdering::new();
        
        // Create a large number of TT entries for performance testing
        let mut tt_entries = Vec::new();
        for i in 0..1000 {
            tt_entries.push(TranspositionEntry {
                score: i as i32,
                depth: (i % 10) as u8 + 1,
                flag: if i % 3 == 0 { TranspositionFlag::Exact } else if i % 3 == 1 { TranspositionFlag::LowerBound } else { TranspositionFlag::UpperBound },
                best_move: Some(Move::new(
                    Some(Position::new((i % 9) as u8 + 1, (i % 9) as u8 + 1)), 
                    Position::new(((i + 1) % 9) as u8 + 1, ((i + 1) % 9) as u8 + 1), 
                    PieceType::Pawn, 
                    false, false, false, false
                )),
                hash_key: i as u64,
                age: i as u32,
            });
        }

        // Performance test: get PV moves from many TT entries
        let start_time = TimeSource::now();
        let mut pv_moves = Vec::new();
        for entry in &tt_entries {
            if let Some(pv_move) = orderer.get_pv_move_from_tt(Some(entry)) {
                pv_moves.push(pv_move);
            }
        }
        let elapsed_ms = start_time.elapsed_ms();

        // Verify performance is reasonable
        assert!(elapsed_ms < 50, "PV move extraction took {}ms, should be < 50ms", elapsed_ms);
        assert_eq!(pv_moves.len(), 1000);
    }

    #[test]
    fn test_transposition_table_integration_validation() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 5;

        // Test 1: Integration with exact transposition table entry
        let exact_entry = TranspositionEntry {
            score: 150,
            depth: 5,
            flag: TranspositionFlag::Exact,
            best_move: Some(Move::new(Some(Position::new(4, 4)), Position::new(4, 3), PieceType::Pawn, false, false, false, false)),
            hash_key: 12345,
            age: 1,
        };

        let result = orderer.integrate_with_transposition_table(Some(&exact_entry), &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 1);
        assert_eq!(stats.tt_integration_updates, 1);
        assert_eq!(stats.pv_moves_from_tt, 1);

        // Test 2: Integration with lower bound entry (should update killer moves)
        let lower_bound_entry = TranspositionEntry {
            score: 75,
            depth: 4,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(Move::new(Some(Position::new(3, 3)), Position::new(3, 2), PieceType::Silver, false, false, false, false)),
            hash_key: 67890,
            age: 2,
        };

        let result = orderer.integrate_with_transposition_table(Some(&lower_bound_entry), &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 2);
        assert_eq!(stats.killer_moves_from_tt, 1);

        // Test 3: Integration with upper bound entry (should not update killer moves)
        let upper_bound_entry = TranspositionEntry {
            score: -25,
            depth: 3,
            flag: TranspositionFlag::UpperBound,
            best_move: Some(Move::new(Some(Position::new(5, 5)), Position::new(5, 4), PieceType::Gold, false, false, false, false)),
            hash_key: 11111,
            age: 3,
        };

        let result = orderer.integrate_with_transposition_table(Some(&upper_bound_entry), &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 3);
        // Killer moves should not be updated for upper bound entries
        assert_eq!(stats.killer_moves_from_tt, 1);

        // Test 4: Update ordering from TT result
        let result = orderer.update_ordering_from_tt_result(&exact_entry, MoveResult::Cutoff);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_cutoff_updates, 1);
        assert_eq!(stats.cutoff_history_updates, 1);

        // Test 5: Get PV move from TT
        let pv_move = orderer.get_pv_move_from_tt(Some(&exact_entry));
        assert!(pv_move.is_some());
        assert_eq!(pv_move.unwrap().from.unwrap(), Position::new(4, 4));

        // Test 6: No TT entry
        let pv_move = orderer.get_pv_move_from_tt(None);
        assert!(pv_move.is_none());
    }

    #[test]
    fn test_transposition_table_integration_edge_cases() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::White;
        let depth = 1;

        // Test 1: TT entry with no best move
        let entry_no_move = TranspositionEntry {
            score: 0,
            depth: 1,
            flag: TranspositionFlag::Exact,
            best_move: None,
            hash_key: 99999,
            age: 1,
        };

        let result = orderer.integrate_with_transposition_table(Some(&entry_no_move), &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_updates, 1);
        assert_eq!(stats.tt_integration_hits, 0); // No best move, so no hits

        // Test 2: Multiple updates with same move
        let entry1 = TranspositionEntry {
            score: 100,
            depth: 2,
            flag: TranspositionFlag::Exact,
            best_move: Some(Move::new(Some(Position::new(1, 1)), Position::new(1, 2), PieceType::Pawn, false, false, false, false)),
            hash_key: 11111,
            age: 1,
        };

        let entry2 = TranspositionEntry {
            score: 200,
            depth: 3,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(Move::new(Some(Position::new(1, 1)), Position::new(1, 2), PieceType::Pawn, false, false, false, false)),
            hash_key: 22222,
            age: 2,
        };

        let result1 = orderer.integrate_with_transposition_table(Some(&entry1), &board, &captured_pieces, player, depth);
        let result2 = orderer.integrate_with_transposition_table(Some(&entry2), &board, &captured_pieces, player, depth);
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 2);
        assert_eq!(stats.killer_moves_from_tt, 1); // Same move, so only counted once
    }

    // ==================== Comprehensive Testing Suite ====================

    #[test]
    fn test_comprehensive_move_ordering_stress() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;

        // Stress test with large number of moves
        let mut moves = Vec::new();
        for i in 0..1000 {
            let from_row = (i % 9) as u8;
            let from_col = (i / 9 % 9) as u8;
            let to_row = ((i + 1) % 9) as u8;
            let to_col = ((i + 1) / 9 % 9) as u8;
            moves.push(Move::new_move(
                Position::new(from_row, from_col),
                Position::new(to_row, to_col),
                PieceType::Pawn,
                player,
                false
            ));
        }

        // Test ordering large number of moves multiple times
        for _ in 0..10 {
            let result = orderer.order_moves(&moves);
            assert!(result.is_ok());
            let ordered = result.unwrap();
            assert_eq!(ordered.len(), moves.len());
        }

        // Verify statistics are being tracked
        let stats = orderer.get_stats();
        assert!(stats.total_moves_ordered > 0);
        assert!(stats.moves_sorted > 0);
    }

    #[test]
    fn test_comprehensive_integration_all_heuristics() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 5;

        // Create test moves
        let moves = vec![
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
            Move::new_move(Position::new(8, 1), Position::new(7, 1), PieceType::Lance, player, false),
            Move::new_move(Position::new(7, 1), Position::new(6, 3), PieceType::Knight, player, false),
        ];

        // Test 1: Order with PV
        let ordered_pv = orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_pv.len(), moves.len());

        // Test 2: Add killer move and order with PV and killer
        orderer.add_killer_move(moves[1].clone());
        let ordered_both = orderer.order_moves_with_pv_and_killer(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_both.len(), moves.len());

        // Test 3: Update history and order with all heuristics
        orderer.update_history(&moves[0], true, depth);
        let ordered_all = orderer.order_moves_with_all_heuristics(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_all.len(), moves.len());

        // Verify all statistics are updated
        let stats = orderer.get_stats();
        assert!(stats.total_moves_ordered > 0);
        assert!(stats.history_updates > 0);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut orderer = MoveOrdering::new();
        
        // Get initial memory usage
        let initial_memory = orderer.get_current_memory_usage();
        assert!(initial_memory > 0);

        // Create moves and order them
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
            Move::new_move(Position::new(1, 1), Position::new(2, 1), PieceType::Silver, Player::Black, false),
            Move::new_move(Position::new(2, 2), Position::new(3, 2), PieceType::Gold, Player::Black, false),
        ];

        for _ in 0..100 {
            let _ = orderer.order_moves(&moves);
        }

        // Check memory usage is tracked
        let final_memory = orderer.get_current_memory_usage();
        assert!(final_memory > 0);

        // Get peak memory usage
        let peak_memory = orderer.get_peak_memory_usage();
        assert!(peak_memory >= initial_memory);

        // Verify no excessive memory growth (memory leaks)
        assert!(final_memory < initial_memory * 10, "Potential memory leak detected");
    }

    #[test]
    fn test_regression_move_ordering_determinism() {
        // Test that move ordering is deterministic for the same input
        let mut orderer1 = MoveOrdering::new();
        let mut orderer2 = MoveOrdering::new();

        let moves = vec![
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, Player::Black, false),
            Move::new_move(Position::new(8, 1), Position::new(7, 1), PieceType::Lance, Player::Black, false),
            Move::new_move(Position::new(7, 1), Position::new(6, 3), PieceType::Knight, Player::Black, false),
        ];

        let result1 = orderer1.order_moves(&moves);
        let result2 = orderer2.order_moves(&moves);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let ordered1 = result1.unwrap();
        let ordered2 = result2.unwrap();

        // Results should be identical for same configuration
        assert_eq!(ordered1.len(), ordered2.len());
        for (m1, m2) in ordered1.iter().zip(ordered2.iter()) {
            assert_eq!(m1.from, m2.from);
            assert_eq!(m1.to, m2.to);
            assert_eq!(m1.piece_type, m2.piece_type);
        }
    }

    #[test]
    fn test_regression_killer_move_depth_management() {
        // Regression test for killer move depth management
        let mut orderer = MoveOrdering::new();

        // Add killer moves at different depths
        for depth in 1..=10 {
            let move_ = Move::new_move(
                Position::new(depth % 9, depth % 9),
                Position::new((depth + 1) % 9, (depth + 1) % 9),
                PieceType::Pawn,
                Player::Black,
                false
            );
            orderer.set_current_depth(depth);
            orderer.add_killer_move(move_);
        }

        // Verify killer moves are stored correctly
        for depth in 1..=10 {
            orderer.set_current_depth(depth);
            let killers = orderer.get_current_killer_moves();
            assert!(killers.is_some());
        }

        // Clear killer moves and verify
        orderer.clear_killer_moves();
        orderer.set_current_depth(5);
        let killers = orderer.get_current_killer_moves();
        assert!(killers.is_none() || killers.unwrap().is_empty());
    }

    #[test]
    fn test_known_position_opening() {
        // Test move ordering on a known opening position
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new(); // Starting position
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;

        // In the opening position, common moves should be ordered
        let moves = vec![
            // P-76 (common opening)
            Move::new_move(Position::new(6, 6), Position::new(5, 6), PieceType::Pawn, player, false),
            // P-26
            Move::new_move(Position::new(6, 1), Position::new(5, 1), PieceType::Pawn, player, false),
            // P-56
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
        ];

        let result = orderer.order_moves(&moves);
        assert!(result.is_ok());
        let ordered = result.unwrap();
        assert_eq!(ordered.len(), 3);

        // All moves should be present
        for original_move in &moves {
            assert!(ordered.iter().any(|m| m.from == original_move.from && m.to == original_move.to));
        }
    }

    #[test]
    fn test_end_to_end_search_integration() {
        // End-to-end test: move ordering integrated with search concepts
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 4;

        // Simulate search tree behavior
        for search_depth in 1..=depth {
            orderer.set_current_depth(search_depth);
            
            // Generate some moves
            let moves = vec![
                Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
                Move::new_move(Position::new(7, 1), Position::new(6, 3), PieceType::Knight, player, false),
            ];

            // Order moves
            let ordered = orderer.order_moves_with_all_heuristics(&moves, &board, &captured_pieces, player, search_depth);
            assert!(!ordered.is_empty());

            // Simulate finding a good move (update history and killer)
            if let Some(best_move) = ordered.first() {
                orderer.add_killer_move(best_move.clone());
                orderer.update_history(best_move, true, search_depth);
            }

            // Simulate transposition table hit
            let tt_entry = TranspositionEntry {
                score: 100,
                depth: search_depth,
                flag: TranspositionFlag::Exact,
                best_move: ordered.first().cloned(),
                hash_key: search_depth as u64 * 12345,
                age: 1,
            };
            let _ = orderer.integrate_with_transposition_table(Some(&tt_entry), &board, &captured_pieces, player, search_depth);
        }

        // Verify statistics reflect the search
        let stats = orderer.get_stats();
        assert!(stats.total_moves_ordered > 0);
        assert!(stats.history_updates > 0);
        assert!(stats.killer_moves_stored > 0);

        let tt_stats = orderer.get_tt_integration_stats();
        assert!(tt_stats.tt_integration_updates > 0);
    }

    #[test]
    fn test_performance_benchmarks_basic() {
        // Basic performance benchmark test
        let mut orderer = MoveOrdering::new();

        // Create a moderate number of moves
        let mut moves = Vec::new();
        for i in 0..50 {
            moves.push(Move::new_move(
                Position::new((i % 9) as u8, (i / 9 % 9) as u8),
                Position::new(((i + 1) % 9) as u8, ((i + 1) / 9 % 9) as u8),
                PieceType::Pawn,
                Player::Black,
                false
            ));
        }

        // Measure ordering time
        let start = TimeSource::now();
        for _ in 0..100 {
            let _ = orderer.order_moves(&moves);
        }
        let elapsed = start.elapsed_ms();

        // Should complete 100 orderings in reasonable time (< 1 second)
        assert!(elapsed < 1000, "Move ordering took {}ms for 100 iterations, should be < 1000ms", elapsed);

        // Check statistics
        let stats = orderer.get_stats();
        assert!(stats.total_moves_ordered >= 5000); // 100 iterations * 50 moves
    }

    #[test]
    fn test_stress_concurrent_operations() {
        // Stress test with many operations
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;

        // Perform many different operations
        for iteration in 0..100 {
            let depth = (iteration % 10) as u8 + 1;
            orderer.set_current_depth(depth);

            // Create moves
            let moves = vec![
                Move::new_move(Position::new((iteration % 9) as u8, 0), Position::new((iteration % 9) as u8, 1), PieceType::Pawn, player, false),
                Move::new_move(Position::new(0, (iteration % 9) as u8), Position::new(1, (iteration % 9) as u8), PieceType::Lance, player, false),
            ];

            // Order with different methods
            let _ = orderer.order_moves(&moves);
            let _ = orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);
            let _ = orderer.order_moves_with_all_heuristics(&moves, &board, &captured_pieces, player, depth);

            // Update heuristics
            orderer.add_killer_move(moves[0].clone());
            orderer.update_history(&moves[0], iteration % 2 == 0, depth);

            // Age history periodically
            if iteration % 10 == 0 {
                orderer.age_history_table();
            }
        }

        // Verify orderer is still functional
        let test_moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, player, false),
        ];
        let result = orderer.order_moves(&test_moves);
        assert!(result.is_ok());

        // Verify statistics
        let stats = orderer.get_stats();
        assert!(stats.total_moves_ordered > 100);
        assert!(stats.history_updates > 0);
        assert!(stats.killer_moves_stored > 0);
    }

    #[test]
    fn test_edge_case_empty_moves() {
        let mut orderer = MoveOrdering::new();
        let empty_moves: Vec<Move> = vec![];

        let result = orderer.order_moves(&empty_moves);
        assert!(result.is_ok());
        let ordered = result.unwrap();
        assert_eq!(ordered.len(), 0);
    }

    #[test]
    fn test_edge_case_single_move() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
        ];

        let result = orderer.order_moves(&moves);
        assert!(result.is_ok());
        let ordered = result.unwrap();
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].from, moves[0].from);
        assert_eq!(ordered[0].to, moves[0].to);
    }

    #[test]
    fn test_regression_history_aging() {
        // Regression test for history aging
        let mut orderer = MoveOrdering::new();
        
        // Add history values
        let move1 = Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false);
        orderer.update_history(&move1, true, 5);

        // Get initial history value
        let initial_value = orderer.get_history_value(&move1);
        assert!(initial_value > 0);

        // Age history multiple times
        for _ in 0..5 {
            orderer.age_history_table();
        }

        // History value should be reduced after aging
        let aged_value = orderer.get_history_score(&move1);
        assert!(aged_value < initial_value, "History aging should reduce values");
    }

    // ==================== Performance Tuning Tests ====================

    #[test]
    fn test_runtime_performance_tuning() {
        let mut orderer = MoveOrdering::new();
        
        // Create conditions for tuning
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
        ];
        
        // Run some operations to generate statistics
        for _ in 0..50 {
            let _ = orderer.order_moves(&moves);
        }
        
        // Apply runtime tuning
        let result = orderer.tune_performance_runtime();
        
        // Verify result structure
        assert!(result.adjustments_made >= 0);
        assert!(result.cache_hit_rate_before >= 0.0);
        assert!(result.avg_ordering_time_before >= 0.0);
    }

    #[test]
    fn test_performance_monitoring() {
        let mut orderer = MoveOrdering::new();
        
        // Generate some statistics
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
        ];
        
        for _ in 0..20 {
            let _ = orderer.order_moves(&moves);
        }
        
        // Get monitoring report
        let report = orderer.monitor_performance();
        
        // Verify report structure
        assert!(report.overall_health_score >= 0.0 && report.overall_health_score <= 100.0);
        assert!(report.cache_hit_rate >= 0.0);
        assert!(report.avg_ordering_time_us >= 0.0);
        assert!(report.memory_usage_mb >= 0.0);
    }

    #[test]
    fn test_auto_optimization() {
        let mut orderer = MoveOrdering::new();
        
        // Generate statistics
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
            Move::new_move(Position::new(1, 1), Position::new(2, 1), PieceType::Silver, Player::Black, false),
        ];
        
        for _ in 0..30 {
            let _ = orderer.order_moves(&moves);
        }
        
        // Apply auto optimization
        let result = orderer.auto_optimize();
        
        // Verify result
        assert!(result.optimizations_applied >= 0);
        assert!(result.performance_before.cache_hit_rate >= 0.0);
        assert!(result.performance_after.cache_hit_rate >= 0.0);
    }

    #[test]
    fn test_tuning_recommendations() {
        let orderer = MoveOrdering::new();
        
        // Get recommendations
        let recommendations = orderer.get_tuning_recommendations();
        
        // Verify recommendations structure
        for rec in &recommendations {
            assert!(!rec.description.is_empty());
            assert!(!rec.expected_impact.is_empty());
        }
    }

    #[test]
    fn test_performance_snapshot_comparison() {
        let snapshot1 = PerformanceSnapshot {
            cache_hit_rate: 60.0,
            avg_ordering_time_us: 80.0,
            memory_usage_bytes: 1000000,
        };
        
        let snapshot2 = PerformanceSnapshot {
            cache_hit_rate: 75.0,
            avg_ordering_time_us: 60.0,
            memory_usage_bytes: 1200000,
        };
        
        let comparison = MoveOrdering::compare_performance(&snapshot1, &snapshot2);
        
        // Verify comparison
        assert_eq!(comparison.cache_hit_rate_change, 15.0);
        assert_eq!(comparison.ordering_time_change, -20.0);
        assert_eq!(comparison.memory_usage_change, 200000);
        assert!(comparison.is_improved); // Better cache hit rate and faster ordering
    }

    #[test]
    fn test_adaptive_weight_adjustment() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        
        // Set up high PV hit rate scenario
        for i in 0..20 {
            let move_ = Move::new_move(
                Position::new((i % 9) as u8, 0),
                Position::new((i % 9) as u8, 1),
                PieceType::Pawn,
                player,
                false
            );
            orderer.update_pv_move(&board, &captured_pieces, player, i % 5, move_, 100);
        }
        
        let initial_weight = orderer.get_weights().pv_move_weight;
        
        // Apply weight adjustment
        let adjustments = orderer.adjust_weights_based_on_effectiveness();
        
        // If PV hit rate is high, weight should increase (or stay same if already max)
        let final_weight = orderer.get_weights().pv_move_weight;
        assert!(final_weight >= initial_weight || initial_weight >= 12000);
    }

    // ==================== WASM Compatibility Tests ====================

    #[test]
    fn test_wasm_optimized_config() {
        #[cfg(target_arch = "wasm32")]
        {
            let config = MoveOrdering::wasm_optimized_config();
            
            // Verify WASM-specific settings
            assert!(config.cache_config.max_cache_size <= 100000, "WASM cache should be limited");
            assert!(config.cache_config.max_see_cache_size <= 50000, "WASM SEE cache should be limited");
            assert!(config.killer_config.max_killer_moves_per_depth <= 2, "WASM killer moves should be limited");
        }
    }

    #[test]
    fn test_native_optimized_config() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let config = MoveOrdering::native_optimized_config();
            
            // Verify native-specific settings
            assert!(config.cache_config.max_cache_size >= 100000, "Native cache should be larger");
            assert!(config.performance_config.enable_performance_monitoring, "Native should enable monitoring");
        }
    }

    #[test]
    fn test_platform_detection() {
        let is_wasm = MoveOrdering::is_wasm_environment();
        
        #[cfg(target_arch = "wasm32")]
        assert!(is_wasm, "Should detect WASM environment");
        
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!is_wasm, "Should detect native environment");
    }

    #[test]
    fn test_platform_memory_limits() {
        let limits = MoveOrdering::get_platform_memory_limits();
        
        // Verify limits are reasonable
        assert!(limits.max_total_memory_bytes > 0);
        assert!(limits.max_cache_size > 0);
        assert!(limits.max_see_cache_size > 0);
        assert!(limits.recommended_cache_size <= limits.max_cache_size);
        assert!(limits.recommended_see_cache_size <= limits.max_see_cache_size);
        
        #[cfg(target_arch = "wasm32")]
        {
            assert!(limits.max_total_memory_bytes <= 20 * 1024 * 1024, "WASM memory limit should be conservative");
            assert!(limits.max_cache_size <= 100000, "WASM cache limit should be conservative");
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(limits.max_total_memory_bytes >= 50 * 1024 * 1024, "Native memory limit should be generous");
            assert!(limits.max_cache_size >= 100000, "Native cache limit should be generous");
        }
    }

    #[test]
    fn test_wasm_compatibility_time_source() {
        // Verify TimeSource is being used (WASM-compatible)
        let mut orderer = MoveOrdering::new();
        
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
        ];
        
        // This should not panic on WASM
        for _ in 0..10 {
            let _ = orderer.order_moves(&moves);
        }
        
        // Verify timing statistics are tracked
        let stats = orderer.get_stats();
        assert!(stats.total_ordering_time_us >= 0);
    }

    #[test]
    fn test_wasm_array_indexing() {
        // Verify array indexing is safe for WASM
        let mut orderer = MoveOrdering::new();
        
        // Test with various positions to ensure no index out of bounds
        for row in 0..9 {
            for col in 0..9 {
                let move_ = Move::new_move(
                    Position::new(row, col),
                    Position::new((row + 1) % 9, (col + 1) % 9),
                    PieceType::Pawn,
                    Player::Black,
                    false
                );
                
                // This should not panic with index out of bounds
                orderer.update_history_score(&move_, 3);
            }
        }
        
        // Verify history table is populated
        let stats = orderer.get_stats();
        assert!(stats.history_updates > 0);
    }

    #[test]
    fn test_platform_optimized_config() {
        let config = MoveOrdering::platform_optimized_config();
        
        // Verify configuration is valid
        assert!(config.cache_config.max_cache_size > 0);
        assert!(config.cache_config.max_see_cache_size > 0);
        assert!(config.killer_config.max_killer_moves_per_depth > 0);
        
        // Create orderer with platform config
        let orderer = MoveOrdering::with_config(config);
        
        // Verify it works
        let moves = vec![
            Move::new_move(Position::new(0, 0), Position::new(1, 0), PieceType::Pawn, Player::Black, false),
        ];
        
        let result = orderer.order_moves(&moves);
        assert!(result.is_ok());
    }

    // ==================== Advanced Integration Tests ====================

    #[test]
    fn test_opening_book_integration() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 1;

        // Create mock book moves
        let book_moves = vec![
            crate::opening_book::BookMove {
                from: Some(Position::new(6, 4)),
                to: Position::new(5, 4),
                piece_type: PieceType::Pawn,
                is_drop: false,
                is_promotion: false,
                weight: 800,
                evaluation: 50,
                opening_name: Some("Standard Opening".to_string()),
                move_notation: Some("P-76".to_string()),
            },
        ];

        // Integrate with opening book
        let result = orderer.integrate_with_opening_book(&book_moves, &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        // Verify statistics
        let adv_stats = orderer.get_advanced_integration_stats();
        assert_eq!(adv_stats.opening_book_integrations, 1);
    }

    #[test]
    fn test_tablebase_integration_advanced() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 1;

        // Create mock tablebase result
        let tb_result = crate::tablebase::TablebaseResult {
            best_move: Some(Move::new_move(
                Position::new(4, 4),
                Position::new(3, 4),
                PieceType::King,
                player,
                false
            )),
            distance_to_mate: Some(5),
            moves_to_mate: Some(5),
            outcome: crate::tablebase::TablebaseOutcome::Win,
            confidence: 1.0,
        };

        // Integrate with tablebase
        let result = orderer.integrate_with_tablebase(&tb_result, &board, &captured_pieces, player, depth);
        assert!(result.is_ok());

        // Verify statistics
        let adv_stats = orderer.get_advanced_integration_stats();
        assert_eq!(adv_stats.tablebase_integrations, 1);
    }

    #[test]
    fn test_analysis_mode_ordering() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        let moves = vec![
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
            Move::new_move(Position::new(6, 5), Position::new(5, 5), PieceType::Pawn, player, false),
        ];

        let ordered = orderer.order_moves_for_analysis(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered.len(), moves.len());

        let adv_stats = orderer.get_advanced_integration_stats();
        assert_eq!(adv_stats.analysis_orderings, 1);
    }

    #[test]
    fn test_time_management_ordering() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        let moves = vec![
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
            Move::new_move(Position::new(6, 5), Position::new(5, 5), PieceType::Pawn, player, false),
        ];

        // Test with different time constraints
        let ordered_low = orderer.order_moves_with_time_management(&moves, 500, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_low.len(), moves.len());

        let ordered_high = orderer.order_moves_with_time_management(&moves, 10000, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_high.len(), moves.len());
    }

    #[test]
    fn test_game_phase_ordering() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        let moves = vec![
            Move::new_move(Position::new(6, 4), Position::new(5, 4), PieceType::Pawn, player, false),
        ];

        // Test different game phases
        let _ = orderer.order_moves_for_game_phase(&moves, GamePhase::Opening, &board, &captured_pieces, player, depth);
        let _ = orderer.order_moves_for_game_phase(&moves, GamePhase::Middlegame, &board, &captured_pieces, player, depth);
        let _ = orderer.order_moves_for_game_phase(&moves, GamePhase::Endgame, &board, &captured_pieces, player, depth);

        let adv_stats = orderer.get_advanced_integration_stats();
        assert_eq!(adv_stats.phase_specific_orderings, 3);
    }

    #[test]
    fn test_parallel_search_preparation() {
        let mut orderer = MoveOrdering::new();

        let parallel_config = orderer.prepare_for_parallel_search();

        // Verify configuration
        assert!(!parallel_config.thread_safe_caches);
        assert!(parallel_config.shared_history);
        assert!(parallel_config.shared_pv);
        assert!(!parallel_config.shared_killers);
    }
}
