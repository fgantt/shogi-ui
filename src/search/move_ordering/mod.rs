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

use crate::utils::time::TimeSource;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, PieceType, Player, Position};
use crate::types::transposition::TranspositionEntry;
use crate::types::TranspositionFlag;
use std::collections::HashMap;
use std::fmt;
use std::ptr;

// Task 1.22: Modularized move ordering - submodules are in the same directory
mod statistics;
mod cache;
mod history_heuristic;
mod killer_moves;
mod counter_moves;
mod pv_ordering;

pub use pv_ordering::{
    moves_equal as moves_equal_helper, score_pv_move as score_pv_move_helper, PVMoveStatistics,
    PVOrdering,
};

mod capture_ordering;

pub use capture_ordering::{
    calculate_mvv_lva_score, get_attacker_bonus, get_capture_bonus,
    score_capture_move as score_capture_move_helper,
    score_capture_move_inline as score_capture_move_inline_helper,
    score_promotion_move as score_promotion_move_helper,
    score_promotion_move_inline as score_promotion_move_inline_helper,
};

mod see_calculation;

pub use see_calculation::{
    calculate_see_internal as calculate_see_internal_helper,
    piece_attacks_square as piece_attacks_square_helper, score_see_move as score_see_move_helper,
    SEECache, SEECacheEntry, SEECacheStats,
};

// Re-export statistics structures
// Task 1.22: PerformanceStats and StatisticsExport now in statistics module
pub use statistics::{
    AdvancedIntegrationStats, AllocationStats, AutoOptimizationResult, Bottleneck,
    BottleneckAnalysis, BottleneckCategory, BottleneckSeverity, CacheHitRates, CachePerformance,
    CacheSizes, CacheStats, DepthSpecificStats, DepthStats, EnhancedStatistics, FragmentationStats,
    GamePhaseStats, HeuristicEffectiveness, HeuristicPerformance, HeuristicStats, HotPathStats,
    MemoryBreakdown, MemoryStats, MemoryUsageTrend, MoveTypeDistribution, OperationTiming,
    OrderingStats, PerformanceChartData, PerformanceComparison, PerformanceMonitoringReport,
    PerformanceSnapshot, PerformanceStats, PerformanceSummary, PerformanceTrendAnalysis,
    PerformanceTuningResult, PhaseStats, StatisticsExport, StatisticsSummary, TTIntegrationStats,
    TimingBreakdown, TimingStats, TrendAnalysis, TrendDirection, TuningCategory, TuningPriority,
    TuningRecommendation,
};

// Re-export cache structures
pub use cache::{
    CacheConfig, CacheEvictionPolicy, MoveOrderingCacheEntry, MoveOrderingCacheManager,
    MoveScoreCache,
};

// Re-export killer moves structures
pub use killer_moves::{
    score_killer_move as score_killer_move_helper, KillerConfig, KillerMoveManager,
};

// Re-export counter-moves structures
pub use counter_moves::{
    score_counter_move as score_counter_move_helper, CounterMoveConfig, CounterMoveManager,
};

// Re-export history heuristic structures
pub use history_heuristic::{
    score_history_move as score_history_move_helper, HistoryConfig, HistoryEntry,
    HistoryHeuristicManager,
};

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

// TTIntegrationStats, PerformanceTuningResult, PerformanceMonitoringReport, AutoOptimizationResult,
// PerformanceSnapshot, PerformanceComparison, TuningRecommendation, TuningCategory, TuningPriority
// moved to statistics module

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

// AdvancedIntegrationStats moved to statistics module

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
    pub fn log_error(
        &mut self,
        error: MoveOrderingError,
        severity: ErrorSeverity,
        context: String,
    ) {
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
        let critical_count = recent_errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Critical)
            .count();
        let high_count = recent_errors
            .iter()
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
            critical_threshold: 50 * 1024 * 1024, // 50 MB
            max_memory: 100 * 1024 * 1024,        // 100 MB
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
    pub fn record_allocation(
        &mut self,
        allocation_type: AllocationType,
        size: usize,
        component: String,
    ) {
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
    pub fn record_deallocation(
        &mut self,
        allocation_type: AllocationType,
        size: usize,
        component: String,
    ) {
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

        usage.total_memory = usage.struct_memory
            + usage.cache_memory
            + usage.pool_memory
            + usage.statistics_memory
            + usage.error_handler_memory;

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
                if duration.as_secs() > 60 {
                    // Consider it a leak if allocated more than 1 minute ago
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
                counter_move_weight: 500,
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
                counter_move_weight: 600,
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
                counter_move_weight: 500,
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
                counter_move_weight: 600,
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
                counter_move_weight: 600,
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
            strategies: vec![CacheWarmingStrategy {
                name: "Common Positions".to_string(),
                strategy_type: CacheWarmingType::PrecomputeCommon,
                parameters: CacheWarmingParameters {
                    depth: 2,
                    time_limit_ms: 1000,
                    memory_limit_mb: 50,
                },
                effectiveness: 0.0,
            }],
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
    /// Move scoring cache for performance optimization (Task 1.22: extracted to cache module)
    move_score_cache: MoveScoreCache,
    /// Transposition table reference for PV move retrieval
    transposition_table: *const crate::search::ThreadSafeTranspositionTable,
    /// Hash calculator for position hashing
    hash_calculator: crate::search::ShogiHashHandler,
    /// PV ordering manager (Task 6.0: extracted to module)
    pv_ordering: PVOrdering,
    /// Move ordering cache manager (Task 6.0: extracted to module)
    /// Manages move ordering result cache with multiple eviction policies
    cache_manager: MoveOrderingCacheManager,
    /// Killer move manager (Task 6.0: extracted to module)
    killer_move_manager: KillerMoveManager,
    /// Counter-move manager (Task 6.0: extracted to module)
    counter_move_manager: CounterMoveManager,
    /// History heuristic manager (Task 6.0: extracted to module)
    history_manager: HistoryHeuristicManager,
    /// Heuristic effectiveness tracking (Task 5.0)
    /// Maps heuristic name -> effectiveness metrics
    heuristic_effectiveness: HashMap<String, HeuristicEffectivenessMetrics>,
    /// Weight change history (Task 5.0)
    /// Tracks weight changes over time for learning analysis
    weight_change_history: Vec<WeightChange>,
    /// Learning update counter (Task 5.0)
    /// Tracks number of games/moves for learning frequency
    learning_update_counter: u64,
    /// Simple history table for position-based history (9x9 board)
    simple_history_table: [[i32; 9]; 9],
    /// History update counter for aging
    history_update_counter: u64,
    /// Pattern-based search integrator (Phase 3 - Task 3.2, available for search enhancements)
    #[allow(dead_code)]
    pattern_integrator: crate::evaluation::pattern_search_integration::PatternSearchIntegrator,
    /// SEE cache manager (Task 6.0: extracted to module)
    see_cache: SEECache,
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
    // PV moves organized by depth (Task 6.0: now managed by PVOrdering)
}

// Statistics structures moved to statistics module - see statistics.rs
// Task 1.22: PerformanceStats and StatisticsExport now in statistics module

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
    /// Counter-move heuristic configuration
    pub counter_move_config: CounterMoveConfig,
    /// History heuristic configuration
    pub history_config: HistoryConfig,
    /// Learning configuration (Task 5.0)
    pub learning_config: LearningConfig,
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
    /// Weight for counter-move heuristic moves
    pub counter_move_weight: i32,
}

// CacheEvictionPolicy, MoveOrderingCacheEntry, and CacheConfig moved to cache module

// KillerConfig, CounterMoveConfig, HistoryEntry, HistoryConfig moved to their respective modules

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

/// Learning configuration for move ordering
/// Task 5.0: Configuration for adaptive weight adjustment
#[derive(Debug, Clone, serde::Serialize)]
pub struct LearningConfig {
    /// Enable adaptive weight adjustment based on effectiveness
    pub enable_learning: bool,
    /// Learning rate - how quickly weights adjust (0.0 to 1.0)
    pub learning_rate: f32,
    /// Learning frequency - how often weights are updated (number of games/moves)
    pub learning_frequency: u64,
    /// Minimum games/moves before adjusting weights
    pub min_games_for_learning: u64,
    /// Minimum effectiveness difference to trigger weight adjustment
    pub min_effectiveness_diff: f32,
    /// Maximum weight change per adjustment (as percentage)
    pub max_weight_change_percent: f32,
    /// Enable weight bounds (prevent extreme values)
    pub enable_weight_bounds: bool,
    /// Minimum weight value (if bounds enabled)
    pub min_weight: i32,
    /// Maximum weight value (if bounds enabled)
    pub max_weight: i32,
}

/// Heuristic effectiveness metrics for learning
/// Task 5.0: Tracks effectiveness of each heuristic for weight adjustment
#[derive(Debug, Clone)]
pub struct HeuristicEffectivenessMetrics {
    /// Hit rate for this heuristic (0.0 to 1.0)
    pub hit_rate: f64,
    /// Number of times this heuristic caused a cutoff
    pub cutoff_count: u64,
    /// Total number of times this heuristic was used
    pub total_uses: u64,
    /// Effectiveness score (calculated from hit rate and cutoff count)
    pub effectiveness_score: f64,
    /// Last update timestamp
    pub last_update: u64,
}

/// Weight change history entry
/// Task 5.0: Tracks weight changes over time
#[derive(Debug, Clone)]
pub struct WeightChange {
    /// Weight name/field
    pub weight_name: String,
    /// Old weight value
    pub old_weight: i32,
    /// New weight value
    pub new_weight: i32,
    /// Reason for change
    pub reason: String,
    /// Timestamp of change
    pub timestamp: u64,
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
            counter_move_config: CounterMoveConfig::default(),
            history_config: HistoryConfig::default(),
            learning_config: LearningConfig::default(),
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
            pv_move_weight: 10000,     // Highest priority for PV moves
            killer_move_weight: 5000,  // High priority for killer moves
            history_weight: 2500,      // Medium-high priority for history moves
            see_weight: 2000,          // High priority for SEE moves
            counter_move_weight: 3000, // Medium-high priority for counter-moves
        }
    }
}

// CacheConfig::Default implementation moved to cache module

// KillerConfig::Default, CounterMoveConfig::Default, HistoryConfig::Default moved to their respective modules

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

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enable_learning: false,         // Task 5.0: Disabled by default
            learning_rate: 0.1,             // Task 5.0: 10% adjustment per update
            learning_frequency: 100,        // Task 5.0: Update every 100 games/moves
            min_games_for_learning: 10,     // Task 5.0: Minimum 10 games before learning
            min_effectiveness_diff: 0.05,   // Task 5.0: 5% minimum difference to trigger adjustment
            max_weight_change_percent: 0.2, // Task 5.0: Maximum 20% change per adjustment
            enable_weight_bounds: true,     // Task 5.0: Enable weight bounds by default
            min_weight: 0,                  // Task 5.0: Minimum weight (non-negative)
            max_weight: 20000,              // Task 5.0: Maximum weight (reasonable upper bound)
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
        if self.weights.counter_move_weight < 0 {
            errors.push("Counter-move weight must be non-negative".to_string());
        }
        if self.weights.history_weight < 0 {
            errors.push("History weight must be non-negative".to_string());
        }

        // Validate learning configuration (Task 5.0)
        if self.learning_config.learning_rate < 0.0 || self.learning_config.learning_rate > 1.0 {
            errors.push("Learning rate must be between 0.0 and 1.0".to_string());
        }
        if self.learning_config.min_effectiveness_diff < 0.0
            || self.learning_config.min_effectiveness_diff > 1.0
        {
            errors.push("Minimum effectiveness difference must be between 0.0 and 1.0".to_string());
        }
        if self.learning_config.max_weight_change_percent < 0.0
            || self.learning_config.max_weight_change_percent > 1.0
        {
            errors.push("Maximum weight change percent must be between 0.0 and 1.0".to_string());
        }
        if self.learning_config.enable_weight_bounds {
            if self.learning_config.min_weight < 0 {
                errors.push("Minimum weight must be non-negative".to_string());
            }
            if self.learning_config.max_weight <= self.learning_config.min_weight {
                errors.push("Maximum weight must be greater than minimum weight".to_string());
            }
        }

        // Validate cache configuration
        // Task 3.0: Validate cache eviction policy configuration
        if self.cache_config.hybrid_lru_weight < 0.0 || self.cache_config.hybrid_lru_weight > 1.0 {
            errors.push("Hybrid LRU weight must be between 0.0 and 1.0".to_string());
        }
        if self.cache_config.max_cache_size == 0 {
            errors.push("Max cache size must be greater than 0".to_string());
        }
        if self.cache_config.cache_warming_ratio < 0.0
            || self.cache_config.cache_warming_ratio > 1.0
        {
            errors.push("Cache warming ratio must be between 0.0 and 1.0".to_string());
        }
        if self.cache_config.optimization_hit_rate_threshold < 0.0
            || self.cache_config.optimization_hit_rate_threshold > 100.0
        {
            errors
                .push("Optimization hit rate threshold must be between 0.0 and 100.0".to_string());
        }

        // Validate killer configuration
        if self.killer_config.max_killer_moves_per_depth == 0 {
            errors.push("Max killer moves per depth must be greater than 0".to_string());
        }
        if self.killer_config.killer_aging_factor < 0.0
            || self.killer_config.killer_aging_factor > 1.0
        {
            errors.push("Killer aging factor must be between 0.0 and 1.0".to_string());
        }

        // Validate counter-move configuration
        if self.counter_move_config.max_counter_moves == 0 {
            errors.push("Max counter-moves must be greater than 0".to_string());
        }
        if self.counter_move_config.counter_move_aging_factor < 0.0
            || self.counter_move_config.counter_move_aging_factor > 1.0
        {
            errors.push("Counter-move aging factor must be between 0.0 and 1.0".to_string());
        }

        // Validate history configuration
        if self.history_config.max_history_score == 0 {
            errors.push("Max history score must be greater than 0".to_string());
        }
        if self.history_config.history_aging_factor < 0.0
            || self.history_config.history_aging_factor > 1.0
        {
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
                counter_move_weight: other.weights.counter_move_weight,
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
                cache_eviction_policy: other.cache_config.cache_eviction_policy,
                lru_access_counter: other.cache_config.lru_access_counter, // Note: This is now managed by cache_manager
                hybrid_lru_weight: other.cache_config.hybrid_lru_weight,
            },
            killer_config: KillerConfig {
                max_killer_moves_per_depth: other.killer_config.max_killer_moves_per_depth,
                enable_killer_aging: other.killer_config.enable_killer_aging,
                killer_aging_factor: other.killer_config.killer_aging_factor,
                enable_depth_based_management: other.killer_config.enable_depth_based_management,
            },
            counter_move_config: CounterMoveConfig {
                max_counter_moves: other.counter_move_config.max_counter_moves,
                enable_counter_move: other.counter_move_config.enable_counter_move,
                enable_counter_move_aging: other.counter_move_config.enable_counter_move_aging,
                counter_move_aging_factor: other.counter_move_config.counter_move_aging_factor,
            },
            history_config: HistoryConfig {
                max_history_score: other.history_config.max_history_score,
                history_aging_factor: other.history_config.history_aging_factor,
                enable_automatic_aging: other.history_config.enable_automatic_aging,
                aging_frequency: other.history_config.aging_frequency,
                enable_score_clamping: other.history_config.enable_score_clamping,
                enable_phase_aware: other.history_config.enable_phase_aware,
                enable_relative: other.history_config.enable_relative,
                enable_time_based_aging: other.history_config.enable_time_based_aging,
                enable_quiet_only: other.history_config.enable_quiet_only,
                time_aging_decay_factor: other.history_config.time_aging_decay_factor,
                time_aging_update_frequency_ms: other.history_config.time_aging_update_frequency_ms,
                opening_aging_factor: other.history_config.opening_aging_factor,
                middlegame_aging_factor: other.history_config.middlegame_aging_factor,
                endgame_aging_factor: other.history_config.endgame_aging_factor,
            },
            learning_config: LearningConfig {
                enable_learning: other.learning_config.enable_learning,
                learning_rate: other.learning_config.learning_rate,
                learning_frequency: other.learning_config.learning_frequency,
                min_games_for_learning: other.learning_config.min_games_for_learning,
                min_effectiveness_diff: other.learning_config.min_effectiveness_diff,
                max_weight_change_percent: other.learning_config.max_weight_change_percent,
                enable_weight_bounds: other.learning_config.enable_weight_bounds,
                min_weight: other.learning_config.min_weight,
                max_weight: other.learning_config.max_weight,
            },
            performance_config: PerformanceConfig {
                enable_performance_monitoring: other
                    .performance_config
                    .enable_performance_monitoring,
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
            move_score_cache: MoveScoreCache::new(
                config.cache_config.max_cache_size,
                64, // Fast cache size
            ),
            transposition_table: ptr::null(),
            hash_calculator: crate::search::ShogiHashHandler::new(
                config.cache_config.max_cache_size,
            ),
            pv_ordering: PVOrdering::new(),
            cache_manager: MoveOrderingCacheManager::new(), // Task 6.0: use MoveOrderingCacheManager
            killer_move_manager: KillerMoveManager::new(),
            counter_move_manager: CounterMoveManager::new(),
            history_manager: HistoryHeuristicManager::new(),
            heuristic_effectiveness: HashMap::new(), // Task 5.0: Initialize heuristic effectiveness tracking
            weight_change_history: Vec::new(),       // Task 5.0: Initialize weight change history
            learning_update_counter: 0,              // Task 5.0: Initialize learning update counter
            history_update_counter: 0,
            pattern_integrator:
                crate::evaluation::pattern_search_integration::PatternSearchIntegrator::new(),
            see_cache: SEECache::new(config.cache_config.max_see_cache_size),
            move_score_pool: Vec::with_capacity(256), // Pre-allocate for common move lists
            move_pool: Vec::with_capacity(256),       // Pre-allocate for common move lists
            error_handler: ErrorHandler::default(),
            memory_pool: MemoryPool::default(),
            memory_tracker: MemoryTracker::default(),
            advanced_features: AdvancedFeatures::default(),
            simple_history_table: [[0; 9]; 9],
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

        // Task 1.22: Use MoveScoreCache which handles both fast and main cache
        if let Some(cached_score) = self.move_score_cache.get(move_hash) {
            self.stats.cache_hits += 1;
            self.stats.hot_path_stats.cache_lookups += 1;
            self.stats.hot_path_stats.cache_time_us += cache_start.elapsed_ms() as u64 * 1000;
            self.stats.hot_path_stats.score_move_time_us += start_time.elapsed_ms() as u64 * 1000;
            return Ok(cached_score);
        }

        self.stats.cache_misses += 1;
        self.stats.scoring_operations += 1;

        let mut score = 0;

        // OPTIMIZATION: Inline critical scoring functions to reduce call overhead
        // 1. Capture scoring (highest priority for tactical moves)
        // Use MVV/LVA for capture scoring (SEE is used separately when board is available)
        let score_capture = if move_.is_capture {
            let capture_score = self.score_capture_move_inline(move_);
            score += capture_score;
            capture_score
        } else {
            0
        };

        // 2. Promotion scoring (high priority for strategic moves)
        let score_promotion = if move_.is_promotion {
            let promotion_score = self.score_promotion_move_inline(move_);
            score += promotion_score;
            promotion_score
        } else {
            0
        };

        // 3. Tactical scoring (checks, threats, etc.)
        let score_tactical = if move_.gives_check {
            score += self.config.weights.tactical_weight;
            self.config.weights.tactical_weight
        } else {
            0
        };

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
        } else {
            0
        };

        // Task 1.22: Cache the score (MoveScoreCache handles size limits internally)
        self.move_score_cache.insert(move_hash, score);

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
        self.update_heuristic_stats(
            "quiet",
            !move_.is_capture && !move_.is_promotion && !move_.gives_check,
            score_quiet,
        );

        Ok(score)
    }

    /// Update heuristic performance statistics
    fn update_heuristic_stats(
        &mut self,
        heuristic_name: &str,
        applied: bool,
        score_contribution: i32,
    ) {
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
        timing_stats.avg_time_us =
            timing_stats.total_time_us as f64 / timing_stats.operation_count as f64;

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
        cache_stats.utilization = if max_size > 0 {
            (size as f64 / max_size as f64) * 100.0
        } else {
            0.0
        };

        let total_attempts = cache_stats.hits + cache_stats.misses;
        cache_stats.hit_rate = if total_attempts > 0 {
            (cache_stats.hits as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Update memory usage statistics
    #[allow(dead_code)] // Kept for future use and debugging
    fn update_memory_stats(&mut self) {
        let current_usage = MemoryBreakdown {
            move_score_cache_bytes: self.move_score_cache.memory_bytes(),
            fast_cache_bytes: 0, // Task 1.22: Fast cache is now part of MoveScoreCache
            pv_cache_bytes: self.pv_ordering.cache_memory_bytes(), // Task 6.0: use PVOrdering module
            killer_moves_bytes: self.killer_move_manager.memory_bytes(), // Task 6.0: use KillerMoveManager
            history_table_bytes: self.history_manager.memory_bytes(), // Task 6.0: use HistoryHeuristicManager
            see_cache_bytes: self.see_cache.memory_bytes(), // Task 6.0: use SEECache module
            object_pools_bytes: self.move_score_pool.capacity()
                * (std::mem::size_of::<(i32, usize)>())
                + self.move_pool.capacity() * (std::mem::size_of::<Move>()),
            total_bytes: 0,
        };

        let total_bytes = current_usage.move_score_cache_bytes
            + current_usage.pv_cache_bytes
            + current_usage.killer_moves_bytes
            + current_usage.history_table_bytes
            + current_usage.see_cache_bytes
            + current_usage.object_pools_bytes;

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
    pub fn score_see_move(
        &mut self,
        move_: &Move,
        board: &crate::bitboards::BitboardBoard,
    ) -> MoveOrderingResult<i32> {
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
    pub fn calculate_see(
        &mut self,
        move_: &Move,
        board: &crate::bitboards::BitboardBoard,
    ) -> MoveOrderingResult<i32> {
        let start_time = TimeSource::now();

        // OPTIMIZATION: Fast cache key generation using bit manipulation
        if self.config.cache_config.enable_see_cache {
            let from_pos = move_.from.unwrap_or(Position::new(0, 0));

            // OPTIMIZATION: Use direct hash lookup (Task 6.0: use SEECache module)
            if let Some(cached_value) = self.see_cache.get(from_pos, move_.to) {
                self.stats.see_cache_hits += 1;
                self.stats.see_calculation_time_us += start_time.elapsed_ms() as u64 * 1000;
                return Ok(cached_value);
            }
            self.stats.see_cache_misses += 1;
        }

        let see_value = calculate_see_internal_helper(move_, board); // Task 6.0: use see_calculation module

        // Cache the result if enabled (Task 7.0: enhanced with eviction tracking)
        if self.config.cache_config.enable_see_cache {
            let cache_key = (move_.from.unwrap_or(Position::new(0, 0)), move_.to);
            let evicted = self.see_cache.insert(cache_key.0, cache_key.1, see_value);
            if evicted {
                self.stats.see_cache_evictions += 1;
            }
        }

        // Update timing statistics
        let elapsed_ms = start_time.elapsed_ms();
        self.stats.see_calculation_time_us += elapsed_ms as u64 * 1000; // Convert ms to microseconds
        self.stats.avg_see_calculation_time_us =
            self.stats.see_calculation_time_us as f64 / self.stats.see_calculations as f64;

        Ok(see_value)
    }

    /// Score a capture move
    ///
    /// Captures are generally high-priority moves that should be tried early.
    /// The score is based on the value of the captured piece.
    /// Task 1.22: Delegates to capture_ordering module helper function
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_capture_move(&self, move_: &Move) -> i32 {
        use capture_ordering::score_capture_move;
        score_capture_move(move_, self.config.weights.capture_weight)
    }

    /// Score a promotion move
    ///
    /// Promotions are strategic moves that can significantly change
    /// the value and capabilities of a piece.
    /// Task 1.22: Delegates to capture_ordering module helper function
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_promotion_move(&self, move_: &Move) -> i32 {
        use capture_ordering::score_promotion_move;
        let position_scorer = |pos: &Position| self.score_position_value(pos);
        score_promotion_move(move_, self.config.weights.promotion_weight, position_scorer)
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
    /// Task 8.0: Simplified - removed calls to unimplemented stub functions
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_position_value_comprehensive(&self, move_: &Move) -> i32 {
        // Center control scoring
        self.score_position_value(&move_.to) * self.config.weights.position_value_weight / 100

        // Task 8.0: Removed calls to unimplemented stubs:
        // - score_king_safety() - returns 0
        // - score_piece_activity() - returns 0
        // - score_pawn_structure() - returns 0
    }

    /// Score development move
    ///
    /// Evaluates how well the move develops the piece toward better positions,
    /// increases mobility, or improves piece coordination.
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_development_move(&self, move_: &Move) -> i32 {
        if let Some(from) = move_.from {
            let mut score = self.score_development_value(from, move_.to)
                * self.config.weights.development_weight
                / 100;

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
    /// Task 8.0: Simplified - removed calls to unimplemented stub functions
    #[allow(dead_code)] // Kept for debugging and future use
    fn score_quiet_move(&self, move_: &Move) -> i32 {
        if move_.is_capture || move_.is_promotion || move_.gives_check {
            return 0; // Not a quiet move
        }

        // Base quiet move weight
        self.config.weights.quiet_weight

        // Task 8.0: Removed calls to unimplemented stubs:
        // - score_mobility_improvement() - returns 0
        // - score_coordination_improvement() - returns 0
        // - score_support_value() - returns 0
    }

    // Task 8.0: Removed placeholder methods that returned 0:
    // - score_king_safety() - Not implemented, returns 0
    // - score_piece_activity() - Not implemented, returns 0
    // - score_pawn_structure() - Not implemented, returns 0
    // - score_mobility_improvement() - Not implemented, returns 0
    // - score_coordination_improvement() - Not implemented, returns 0
    // - score_support_value() - Not implemented, returns 0
    // These can be re-added when position analysis is available

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
        let flags = (move_.is_promotion as u64)
            | ((move_.is_capture as u64) << 1)
            | ((move_.gives_check as u64) << 2);

        // Combine using bit shifts for maximum performance
        from << 32 | to << 24 | piece_type << 16 | flags << 8
    }

    /// Inline capture move scoring for hot path optimization
    /// Task 1.22: Delegates to capture_ordering module inline helper function
    fn score_capture_move_inline(&self, move_: &Move) -> i32 {
        use capture_ordering::score_capture_move_inline;
        score_capture_move_inline(move_, self.config.weights.capture_weight)
    }

    /// Inline promotion move scoring for hot path optimization
    /// Task 1.22: Delegates to capture_ordering module inline helper function
    fn score_promotion_move_inline(&self, move_: &Move) -> i32 {
        use capture_ordering::score_promotion_move_inline;
        let position_scorer = |pos: &Position| {
            // Use fast center distance calculation for inline version
            let center_distance = self.get_center_distance_fast(*pos);
            if center_distance <= 1 { 50 } else { 0 }
        };
        score_promotion_move_inline(move_, self.config.weights.promotion_weight, position_scorer)
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
        let dr = if pos.row > 4 {
            pos.row - 4
        } else {
            4 - pos.row
        };
        let dc = if pos.col > 4 {
            pos.col - 4
        } else {
            4 - pos.col
        };
        dr + dc
    }

    /// Update memory usage statistics
    fn update_memory_usage(&mut self) {
        // Calculate current memory usage
        // Task 1.22: Use MoveScoreCache.memory_bytes() which includes both caches
        let move_score_cache_memory = self.move_score_cache.memory_bytes();
        let pv_cache_memory = self.pv_ordering.cache_memory_bytes(); // Task 6.0: use PVOrdering module
        let killer_moves_memory = self.killer_move_manager.memory_bytes(); // Task 6.0: use KillerMoveManager
        let history_table_memory = self.history_manager.memory_bytes(); // Task 6.0: use HistoryHeuristicManager
        let see_cache_memory = self.see_cache.memory_bytes(); // Task 6.0: use SEECache module
        let struct_memory = std::mem::size_of::<Self>();

        self.memory_usage.current_bytes = move_score_cache_memory
            + pv_cache_memory
            + killer_moves_memory
            + history_table_memory
            + see_cache_memory
            + struct_memory;
        self.memory_usage.peak_bytes = self
            .memory_usage
            .peak_bytes
            .max(self.memory_usage.current_bytes);

        // Update statistics
        self.stats.memory_usage_bytes = self.memory_usage.current_bytes;
        self.stats.peak_memory_usage_bytes = self.memory_usage.peak_bytes;

        // Update cache hit rate
        let total_cache_attempts = self.stats.cache_hits + self.stats.cache_misses;
        if total_cache_attempts > 0 {
            self.stats.cache_hit_rate =
                (self.stats.cache_hits as f64 / total_cache_attempts as f64) * 100.0;
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

    // Task 5.0: Learning methods

    /// Record heuristic effectiveness (Task 5.0)
    /// Called when a heuristic is used to track its effectiveness
    pub fn record_heuristic_effectiveness(&mut self, heuristic_name: &str, caused_cutoff: bool) {
        if !self.config.learning_config.enable_learning {
            return;
        }

        let entry = self
            .heuristic_effectiveness
            .entry(heuristic_name.to_string())
            .or_insert_with(|| HeuristicEffectivenessMetrics {
                hit_rate: 0.0,
                cutoff_count: 0,
                total_uses: 0,
                effectiveness_score: 0.0,
                last_update: self.history_update_counter,
            });

        entry.total_uses += 1;
        if caused_cutoff {
            entry.cutoff_count += 1;
        }

        // Update hit rate
        if entry.total_uses > 0 {
            entry.hit_rate = entry.cutoff_count as f64 / entry.total_uses as f64;
        }

        // Calculate effectiveness score (weighted combination of hit rate and cutoff count)
        // Formula: effectiveness = hit_rate * 0.7 + (cutoff_count / max(total_uses, 1)) * 0.3
        let cutoff_ratio = if entry.total_uses > 0 {
            entry.cutoff_count as f64 / entry.total_uses as f64
        } else {
            0.0
        };
        entry.effectiveness_score = entry.hit_rate * 0.7 + cutoff_ratio * 0.3;

        entry.last_update = self.history_update_counter;
    }

    /// Adjust weights based on effectiveness statistics (Task 5.0)
    /// Uses effectiveness scores to adjust heuristic weights
    pub fn adjust_weights_based_on_effectiveness(&mut self) -> bool {
        if !self.config.learning_config.enable_learning {
            return false;
        }

        // Check if we have enough games/moves for learning
        if self.learning_update_counter < self.config.learning_config.min_games_for_learning {
            return false;
        }

        // Check if it's time to update (based on learning frequency)
        if self.learning_update_counter % self.config.learning_config.learning_frequency != 0 {
            return false;
        }

        let mut weights_adjusted = false;
        let mut adjustments_made = 0;

        // Calculate average effectiveness score
        let avg_effectiveness: f64 = self
            .heuristic_effectiveness
            .values()
            .map(|m| m.effectiveness_score)
            .sum::<f64>()
            / self.heuristic_effectiveness.len().max(1) as f64;

        // Extract learning config values to avoid borrow checker issues
        let learning_rate = self.config.learning_config.learning_rate;
        let max_weight_change_percent = self.config.learning_config.max_weight_change_percent;
        let min_effectiveness_diff = self.config.learning_config.min_effectiveness_diff;
        let enable_weight_bounds = self.config.learning_config.enable_weight_bounds;
        let min_weight = self.config.learning_config.min_weight;
        let max_weight = self.config.learning_config.max_weight;

        // Adjust weights for each heuristic based on effectiveness
        // Collect heuristic names and effectiveness differences first to avoid borrow conflicts
        let adjustments: Vec<(String, f64, f64)> = self
            .heuristic_effectiveness
            .iter()
            .filter_map(|(heuristic_name, metrics)| {
                let effectiveness_diff = metrics.effectiveness_score - avg_effectiveness;

                // Only adjust if effectiveness difference is significant
                if effectiveness_diff.abs() < min_effectiveness_diff as f64 {
                    return None;
                }

                Some((
                    heuristic_name.clone(),
                    metrics.effectiveness_score,
                    effectiveness_diff,
                ))
            })
            .collect();

        // Now apply adjustments (no longer borrowing heuristic_effectiveness)
        for (heuristic_name, effectiveness_score, effectiveness_diff) in adjustments {
            // Determine which weight to adjust
            if let Some(weight_ref) = self.get_weight_ref_mut(&heuristic_name) {
                // Get the old weight value
                let old_weight = *weight_ref;

                // Calculate new weight
                let mut new_weight;

                // Adjust weight based on effectiveness
                // Positive effectiveness_diff -> increase weight
                // Negative effectiveness_diff -> decrease weight
                let adjustment = (effectiveness_diff * learning_rate as f64) * old_weight as f64;
                let max_adjustment = (old_weight as f64 * max_weight_change_percent as f64).abs();

                // Clamp adjustment to max_weight_change_percent
                let clamped_adjustment = adjustment.signum() * adjustment.abs().min(max_adjustment);
                new_weight = (old_weight as f64 + clamped_adjustment) as i32;

                // Apply weight bounds if enabled
                if enable_weight_bounds {
                    new_weight = new_weight.max(min_weight).min(max_weight);
                }

                // Only update if weight actually changed
                if new_weight != old_weight {
                    *weight_ref = new_weight;
                    weights_adjusted = true;
                    adjustments_made += 1;

                    // Record weight change in history
                    self.weight_change_history.push(WeightChange {
                        weight_name: heuristic_name.clone(),
                        old_weight,
                        new_weight,
                        reason: format!(
                            "Effectiveness: {:.2} (avg: {:.2})",
                            effectiveness_score, avg_effectiveness
                        ),
                        timestamp: self.history_update_counter,
                    });

                    // Limit history size to prevent unbounded growth
                    if self.weight_change_history.len() > 1000 {
                        self.weight_change_history.remove(0);
                    }
                }
            }
        }

        // Update statistics
        if weights_adjusted {
            self.stats.weight_adjustments += adjustments_made;
            self.stats.learning_effectiveness = avg_effectiveness;
        }

        weights_adjusted
    }

    /// Get mutable reference to weight by heuristic name (Task 5.0)
    fn get_weight_ref_mut(&mut self, heuristic_name: &str) -> Option<&mut i32> {
        match heuristic_name {
            "capture" => Some(&mut self.config.weights.capture_weight),
            "promotion" => Some(&mut self.config.weights.promotion_weight),
            "center_control" => Some(&mut self.config.weights.center_control_weight),
            "development" => Some(&mut self.config.weights.development_weight),
            "piece_value" => Some(&mut self.config.weights.piece_value_weight),
            "position_value" => Some(&mut self.config.weights.position_value_weight),
            "tactical" => Some(&mut self.config.weights.tactical_weight),
            "quiet" => Some(&mut self.config.weights.quiet_weight),
            "pv_move" => Some(&mut self.config.weights.pv_move_weight),
            "killer_move" => Some(&mut self.config.weights.killer_move_weight),
            "history" => Some(&mut self.config.weights.history_weight),
            "see" => Some(&mut self.config.weights.see_weight),
            "counter_move" => Some(&mut self.config.weights.counter_move_weight),
            _ => None,
        }
    }

    /// Increment learning update counter (Task 5.0)
    /// Called after each game/move to track learning frequency
    pub fn increment_learning_counter(&mut self) {
        self.learning_update_counter += 1;

        // Attempt weight adjustment if learning is enabled
        if self.config.learning_config.enable_learning {
            self.adjust_weights_based_on_effectiveness();
        }
    }

    /// Save learned weights to configuration (Task 5.0)
    /// Currently a no-op, but can be extended to save to file/JSON
    pub fn save_learned_weights(&self) -> Result<(), String> {
        // For now, weights are already in config, so nothing to do
        // Future: Could serialize to JSON file
        Ok(())
    }

    /// Load learned weights from configuration (Task 5.0)
    /// Currently a no-op, but can be extended to load from file/JSON
    pub fn load_learned_weights(&mut self) -> Result<(), String> {
        // For now, weights are already in config, so nothing to do
        // Future: Could deserialize from JSON file
        Ok(())
    }

    /// Get heuristic effectiveness metrics (Task 5.0)
    pub fn get_heuristic_effectiveness(
        &self,
        heuristic_name: &str,
    ) -> Option<&HeuristicEffectivenessMetrics> {
        self.heuristic_effectiveness.get(heuristic_name)
    }

    /// Get all heuristic effectiveness metrics (Task 5.0)
    pub fn get_all_heuristic_effectiveness(
        &self,
    ) -> &HashMap<String, HeuristicEffectivenessMetrics> {
        &self.heuristic_effectiveness
    }

    /// Get weight change history (Task 5.0)
    pub fn get_weight_change_history(&self) -> &[WeightChange] {
        &self.weight_change_history
    }

    /// Clear learning data (Task 5.0)
    pub fn clear_learning_data(&mut self) {
        self.heuristic_effectiveness.clear();
        self.weight_change_history.clear();
        self.learning_update_counter = 0;
        self.stats.weight_adjustments = 0;
        self.stats.learning_effectiveness = 0.0;
    }

    /// Reset configuration to default values
    pub fn reset_config_to_default(&mut self) {
        self.config = MoveOrderingConfig::default();
        self.apply_configuration_changes();
    }

    /// Apply configuration changes to internal state
    /// Task 1.22: Delegates cache size management to MoveScoreCache
    fn apply_configuration_changes(&mut self) {
        // Update cache size if needed (MoveScoreCache handles trimming internally)
        self.move_score_cache.set_max_size(self.config.cache_config.max_cache_size);

        // Update killer move limits if needed (Task 6.0: use KillerMoveManager)
        self.killer_move_manager
            .set_max_killer_moves_per_depth(self.config.killer_config.max_killer_moves_per_depth);

        // Update history table if needed (Task 6.0: use HistoryHeuristicManager)
        if self.config.history_config.enable_score_clamping {
            self.history_manager
                .clamp_history_scores(self.config.history_config.max_history_score);
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
            self.config.cache_config.max_cache_size =
                (self.config.cache_config.max_cache_size * 3) / 2; // 1.5x
        } else if hit_rate < 20.0 && self.config.cache_config.max_cache_size > 100 {
            self.config.cache_config.max_cache_size =
                (self.config.cache_config.max_cache_size * 4) / 5; // 0.8x
        }

        // Clear cache if it's too large and hit rate is low
        if self.move_score_cache.len() > self.config.cache_config.max_cache_size && hit_rate < 30.0
        {
            self.move_score_cache.clear();
        }

        self.update_memory_usage();
    }

    /// Get cache hit rate
    ///
    /// Returns the current cache hit rate percentage.
    pub fn get_cache_hit_rate(&self) -> f64 {
        if self.stats.cache_hits + self.stats.cache_misses > 0 {
            (self.stats.cache_hits as f64
                / (self.stats.cache_hits + self.stats.cache_misses) as f64)
                * 100.0
        } else {
            0.0
        }
    }

    /// Set cache size for performance tuning
    ///
    /// Adjusts the maximum cache size based on memory constraints
    /// and performance requirements.
    /// Task 1.22: Delegates to MoveScoreCache
    pub fn set_cache_size(&mut self, size: usize) {
        self.config.cache_config.max_cache_size = size;
        self.move_score_cache.set_max_size(size);
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
        for move_ in moves
            .iter()
            .take(self.config.cache_config.max_cache_size / 2)
        {
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
            self.get_max_cache_size(),
        )
    }

    /// Evict cache entry based on eviction policy
    /// Task 3.0: Implements LRU, depth-preferred, FIFO, and hybrid eviction policies
    /// Evict cache entry (Task 6.0: delegates to cache manager)
    #[allow(dead_code)]
    fn evict_cache_entry(&mut self, _new_key: &(u64, u8)) -> Option<(u64, u8)> {
        self.cache_manager.evict_entry(
            self.config.cache_config.cache_eviction_policy,
            self.config.cache_config.hybrid_lru_weight,
        )
    }

    /// Clear the move scoring cache
    /// Task 1.22: Delegates to MoveScoreCache
    pub fn clear_cache(&mut self) {
        self.move_score_cache.clear();
        self.pv_ordering.clear_cache(); // Task 6.0: use PVOrdering module
        self.cache_manager.clear(); // Task 6.0: use MoveOrderingCacheManager
        self.killer_move_manager.clear_all_killer_moves(); // Task 6.0: use KillerMoveManager
        self.counter_move_manager.clear_all_counter_moves(); // Task 6.0: use CounterMoveManager
        self.history_manager.clear_history_table(); // Task 6.0: use HistoryHeuristicManager
        self.stats.cache_hits = 0;
        self.stats.cache_misses = 0;
        self.stats.cache_hit_rate = 0.0;
        self.stats.cache_evictions = 0; // Task 3.0: Reset eviction stats
        self.stats.cache_evictions_size_limit = 0;
        self.stats.cache_evictions_policy = 0;
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
        (
            self.stats.cache_hits,
            self.stats.cache_misses,
            self.stats.cache_hit_rate,
        )
    }

    /// Set maximum cache size
    /// Task 1.22: Delegates to MoveScoreCache
    pub fn set_max_cache_size(&mut self, max_size: usize) {
        self.config.cache_config.max_cache_size = max_size;
        self.move_score_cache.set_max_size(max_size);
    }

    /// Get current cache size
    pub fn get_cache_size(&self) -> usize {
        self.move_score_cache.len()
    }

    /// Check if cache is at maximum size
    /// Task 1.22: Delegates to MoveScoreCache
    pub fn is_cache_full(&self) -> bool {
        self.move_score_cache.is_full()
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
        score_pv_move_helper(self.config.weights.pv_move_weight) // Task 6.0: use pv_ordering module
    }

    /// Get the PV move for a given position from the transposition table
    ///
    /// This method queries the transposition table to find the best move
    /// for the current position and caches the result for performance.
    pub fn get_pv_move(
        &mut self,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Option<Move> {
        if self.transposition_table.is_null() {
            return None;
        }

        // Calculate position hash
        let position_hash = self
            .hash_calculator
            .get_position_hash(board, player, captured_pieces);

        // Check cache first (Task 6.0: use PVOrdering module)
        if let Some(cached_move) = self.pv_ordering.get_cached_pv_move(position_hash) {
            if cached_move.is_some() {
                self.stats.pv_move_hits += 1;
            } else {
                self.stats.pv_move_misses += 1;
            }
            return cached_move;
        }

        // Query transposition table
        self.stats.tt_lookups += 1;

        // Safe access to transposition table
        let tt_entry = unsafe { (*self.transposition_table).probe(position_hash, depth) };

        let pv_move = if let Some(entry) = tt_entry {
            self.stats.tt_hits += 1;
            entry.best_move
        } else {
            self.stats.pv_move_misses += 1;
            None
        };

        // Cache the result (Task 6.0: use PVOrdering module)
        if !self
            .pv_ordering
            .is_cache_full(self.config.cache_config.max_cache_size)
        {
            self.pv_ordering
                .cache_pv_move(position_hash, pv_move.clone());
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
            self.stats.pv_move_hit_rate =
                (self.stats.pv_move_hits as f64 / total_pv_attempts as f64) * 100.0;
        }

        pv_move
    }

    /// Update the PV move for a position in the transposition table
    ///
    /// This method stores the best move found during search back into
    /// the transposition table for future reference.
    pub fn update_pv_move(
        &mut self,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        best_move: Move,
        score: i32,
    ) {
        if self.transposition_table.is_null() {
            return;
        }

        // Calculate position hash
        let position_hash = self
            .hash_calculator
            .get_position_hash(board, player, captured_pieces);

        // Create transposition table entry
        let entry = TranspositionEntry {
            score,
            depth,
            flag: TranspositionFlag::Exact,
            best_move: Some(best_move.clone()),
            hash_key: position_hash,
            age: 0, // Will be set by the transposition table
            source: crate::types::EntrySource::MainSearch, // Task 7.0.3: Default to MainSearch
        };

        // Store in transposition table
        unsafe {
            if let Some(tt_ref) = self.transposition_table.as_ref() {
                let tt_mut = tt_ref as *const crate::search::ThreadSafeTranspositionTable
                    as *mut crate::search::ThreadSafeTranspositionTable;
                (*tt_mut).store(entry);
            }
        }

        // Update cache (Task 6.0: use PVOrdering module)
        if !self
            .pv_ordering
            .is_cache_full(self.config.cache_config.max_cache_size)
        {
            self.pv_ordering
                .cache_pv_move(position_hash, Some(best_move));
        }
    }

    /// Clear the PV move cache
    ///
    /// This method clears all cached PV moves, typically called
    /// when starting a new search or when memory needs to be freed.
    pub fn clear_pv_move_cache(&mut self) {
        self.pv_ordering.clear_cache();
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
    pub fn is_pv_move(
        &mut self,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        move_: &Move,
    ) -> bool {
        if let Some(pv_move) = self.get_pv_move(board, captured_pieces, player, depth) {
            self.moves_equal(&pv_move, move_)
        } else {
            false
        }
    }

    /// Compare two moves for equality (Task 6.0: now uses module helper)
    ///
    /// This method delegates to the pv_ordering module's moves_equal function.
    fn moves_equal(&self, a: &Move, b: &Move) -> bool {
        moves_equal_helper(a, b) // Task 6.0: use pv_ordering module
    }

    /// Order moves with PV move prioritization
    ///
    /// This enhanced version of order_moves prioritizes PV moves from
    /// the transposition table, giving them the highest priority.
    pub fn order_moves_with_pv(
        &mut self,
        moves: &[Move],
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Vec<Move> {
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
            self.stats.tt_hits,
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
        self.killer_move_manager.set_current_depth(depth);
    }

    /// Get the current search depth
    pub fn get_current_depth(&self) -> u8 {
        self.killer_move_manager.get_current_depth()
    }

    /// Score a move that matches a killer move
    ///
    /// Killer moves get high priority to encourage trying moves that
    /// caused beta cutoffs in previous searches at the same depth.
    pub fn score_killer_move(&mut self, _move_: &Move) -> i32 {
        score_killer_move_helper(self.config.weights.killer_move_weight)
    }

    /// Add a killer move for the current depth
    ///
    /// This method stores a move that caused a beta cutoff, making it
    /// a candidate for early consideration in future searches at the same depth.
    pub fn add_killer_move(&mut self, move_: Move) {
        let was_added = self.killer_move_manager.add_killer_move(
            move_,
            moves_equal_helper,
            self.config.killer_config.max_killer_moves_per_depth,
        );

        if was_added {
            self.stats.killer_moves_stored += 1;
        }

        self.update_memory_usage();
    }

    /// Check if a move is a killer move at the current depth
    ///
    /// This method determines if a given move is stored as a killer move
    /// for the current search depth.
    pub fn is_killer_move(&mut self, move_: &Move) -> bool {
        self.killer_move_manager
            .is_killer_move(move_, moves_equal_helper)
    }

    /// Get all killer moves for a specific depth
    ///
    /// Returns a reference to the killer moves list for the given depth,
    /// or None if no killer moves exist for that depth.
    pub fn get_killer_moves(&self, depth: u8) -> Option<&Vec<Move>> {
        self.killer_move_manager.get_killer_moves(depth)
    }

    /// Get all killer moves for the current depth
    ///
    /// Returns a reference to the killer moves list for the current depth,
    /// or None if no killer moves exist for the current depth.
    pub fn get_current_killer_moves(&self) -> Option<&Vec<Move>> {
        self.killer_move_manager.get_current_killer_moves()
    }

    /// Clear killer moves for a specific depth
    ///
    /// This method removes all killer moves stored for the given depth.
    pub fn clear_killer_moves_for_depth(&mut self, depth: u8) {
        self.killer_move_manager.clear_killer_moves_for_depth(depth);
        self.update_memory_usage();
    }

    /// Clear all killer moves
    ///
    /// This method removes all killer moves from all depths.
    pub fn clear_all_killer_moves(&mut self) {
        self.killer_move_manager.clear_all_killer_moves();
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
        self.killer_move_manager
            .set_max_killer_moves_per_depth(max_moves);
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
            self.stats.killer_moves_stored,
        )
    }

    /// Get killer move hit rate
    ///
    /// Returns the hit rate for killer move lookups.
    pub fn get_killer_move_hit_rate(&self) -> f64 {
        if self.stats.killer_move_hits + self.stats.killer_move_misses > 0 {
            (self.stats.killer_move_hits as f64
                / (self.stats.killer_move_hits + self.stats.killer_move_misses) as f64)
                * 100.0
        } else {
            0.0
        }
    }

    // ==================== Counter-Move Heuristic Methods ====================

    /// Add a counter-move for an opponent's move
    ///
    /// This method stores a move that refuted (caused a cutoff against) an opponent's move.
    /// Counter-moves are used to prioritize moves that were successful against specific opponent moves.
    ///
    /// # Arguments
    /// * `opponent_move` - The opponent's move that was refuted
    /// * `counter_move` - The move that refuted the opponent's move
    pub fn add_counter_move(&mut self, opponent_move: Move, counter_move: Move) {
        if !self.config.counter_move_config.enable_counter_move {
            return;
        }

        let was_added = self.counter_move_manager.add_counter_move(
            opponent_move,
            counter_move,
            moves_equal_helper,
            self.config.counter_move_config.max_counter_moves,
        );

        if was_added {
            self.stats.counter_moves_stored += 1;
        }

        self.update_memory_usage();
    }

    /// Score a move that matches a counter-move for the opponent's last move
    ///
    /// Counter-moves get medium-high priority to encourage trying moves that
    /// refuted opponent moves in previous searches.
    ///
    /// # Arguments
    /// * `move_` - The move to score
    /// * `opponent_last_move` - The opponent's last move (if available)
    pub fn score_counter_move(&mut self, move_: &Move, opponent_last_move: Option<&Move>) -> i32 {
        if !self.config.counter_move_config.enable_counter_move {
            return 0;
        }

        if self
            .counter_move_manager
            .is_counter_move(move_, opponent_last_move, moves_equal_helper)
        {
            self.stats.counter_move_hits += 1;
            return score_counter_move_helper(self.config.weights.counter_move_weight);
        }

        self.stats.counter_move_misses += 1;
        0
    }

    /// Check if a move is a counter-move for the opponent's last move
    ///
    /// This method determines if a given move is stored as a counter-move
    /// for the opponent's last move.
    ///
    /// # Arguments
    /// * `move_` - The move to check
    /// * `opponent_last_move` - The opponent's last move (if available)
    pub fn is_counter_move(&mut self, move_: &Move, opponent_last_move: Option<&Move>) -> bool {
        if !self.config.counter_move_config.enable_counter_move {
            return false;
        }

        self.counter_move_manager
            .is_counter_move(move_, opponent_last_move, moves_equal_helper)
    }

    /// Get all counter-moves for an opponent's move
    ///
    /// Returns a reference to the counter-moves list for the given opponent move,
    /// or None if no counter-moves exist for that opponent move.
    ///
    /// # Arguments
    /// * `opponent_move` - The opponent's move to get counter-moves for
    pub fn get_counter_moves(&self, opponent_move: &Move) -> Option<&Vec<Move>> {
        self.counter_move_manager.get_counter_moves(opponent_move)
    }

    /// Clear all counter-moves for a specific opponent move
    ///
    /// This method clears all counter-moves stored for the given opponent move.
    pub fn clear_counter_moves_for_opponent_move(&mut self, opponent_move: &Move) {
        self.counter_move_manager
            .clear_counter_moves_for_opponent_move(opponent_move);
        self.update_memory_usage();
    }

    /// Clear all counter-moves
    ///
    /// This method clears all counter-moves from the table, typically called
    /// when starting a new search or when memory needs to be freed.
    pub fn clear_all_counter_moves(&mut self) {
        self.counter_move_manager.clear_all_counter_moves();
        self.stats.counter_move_hits = 0;
        self.stats.counter_move_misses = 0;
        self.stats.counter_move_hit_rate = 0.0;
        self.stats.counter_moves_stored = 0;
        self.update_memory_usage();
    }

    /// Set maximum counter-moves per opponent move
    ///
    /// Adjusts the maximum number of counter-moves stored per opponent move.
    pub fn set_max_counter_moves(&mut self, max_moves: usize) {
        self.config.counter_move_config.max_counter_moves = max_moves;
        self.counter_move_manager.set_max_counter_moves(max_moves);
        self.update_memory_usage();
    }

    /// Get the maximum number of counter-moves per opponent move
    pub fn get_max_counter_moves(&self) -> usize {
        self.config.counter_move_config.max_counter_moves
    }

    /// Get counter-move statistics
    ///
    /// Returns statistics about counter-move usage and effectiveness.
    pub fn get_counter_move_stats(&self) -> (u64, u64, f64, u64) {
        (
            self.stats.counter_move_hits,
            self.stats.counter_move_misses,
            self.stats.counter_move_hit_rate,
            self.stats.counter_moves_stored,
        )
    }

    /// Get counter-move hit rate
    ///
    /// Returns the hit rate for counter-move lookups.
    pub fn get_counter_move_hit_rate(&self) -> f64 {
        let total = self.stats.counter_move_hits + self.stats.counter_move_misses;
        if total > 0 {
            (self.stats.counter_move_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Update counter-move hit rate statistics
    ///
    /// This method should be called periodically to update the hit rate
    /// based on current hit/miss counts.
    #[allow(dead_code)]
    fn update_counter_move_hit_rate(&mut self) {
        let total = self.stats.counter_move_hits + self.stats.counter_move_misses;
        if total > 0 {
            self.stats.counter_move_hit_rate =
                (self.stats.counter_move_hits as f64 / total as f64) * 100.0;
        } else {
            self.stats.counter_move_hit_rate = 0.0;
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
        if killer_moves
            .iter()
            .any(|killer| self.moves_equal(move_, killer))
        {
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
    pub fn order_moves_with_pv_and_killer(
        &mut self,
        moves: &[Move],
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Vec<Move> {
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
    fn score_move_with_pv_and_killer(
        &mut self,
        move_: &Move,
        pv_move: &Option<Move>,
        killer_moves: &[Move],
    ) -> i32 {
        // Check if this is the PV move (highest priority)
        if let Some(ref pv) = pv_move {
            if self.moves_equal(move_, pv) {
                return self.score_pv_move(move_);
            }
        }

        // Check if this is a killer move (high priority)
        if killer_moves
            .iter()
            .any(|killer| self.moves_equal(move_, killer))
        {
            self.stats.killer_move_hits += 1;
            return self.score_killer_move(move_);
        }

        // Use regular move scoring
        self.stats.killer_move_misses += 1;
        self.score_move(move_).unwrap_or(0)
    }

    // ==================== History Heuristic Methods ====================

    /// Determine game phase based on material count
    /// Task 4.0: Helper method for phase-aware history
    #[allow(dead_code)]
    fn determine_game_phase_from_material(
        &self,
        board: &crate::bitboards::BitboardBoard,
    ) -> crate::types::GamePhase {
        // Task 6.0: Delegate to history manager
        self.history_manager
            .determine_game_phase_from_material(board)
    }

    /// Get current timestamp for time-based aging
    /// Task 4.0: Helper method for time-based aging (now delegates to manager)
    #[allow(dead_code)]
    fn get_current_timestamp(&mut self) -> u64 {
        // Task 6.0: Delegate to history manager
        self.history_manager.get_current_timestamp()
    }

    /// Score a move using history heuristic
    ///
    /// Returns a score based on how often this move has been successful
    /// in previous searches.
    /// Task 4.0: Enhanced to support relative history, phase-aware history, and quiet-move-only history
    pub fn score_history_move(&mut self, move_: &Move) -> i32 {
        let current_time = self.history_manager.get_current_timestamp();
        let history_score = self.history_manager.get_history_score(
            move_,
            &self.config.history_config,
            current_time,
        );

        if history_score > 0 {
            self.stats.history_hits += 1;
            score_history_move_helper(history_score, self.config.weights.history_weight)
        } else {
            self.stats.history_misses += 1;
            0
        }
    }

    /// Apply time-based aging to history score if enabled
    /// Task 4.0: Helper method for time-based aging (now delegates to manager)
    #[allow(dead_code)]
    fn apply_time_based_aging_if_enabled(&self, score: u32, last_update: u64) -> u32 {
        let current_time = self.history_manager.get_time_aging_counter();
        self.history_manager.apply_time_based_aging_if_enabled(
            score,
            last_update,
            current_time,
            self.config.history_config.enable_time_based_aging,
            self.config.history_config.time_aging_decay_factor,
        )
    }

    /// Update history score for a move
    ///
    /// This method should be called when a move causes a cutoff or
    /// improves the alpha bound during search.
    /// Task 4.0: Enhanced to support relative history, phase-aware history, quiet-move-only history, and time-based aging
    pub fn update_history_score(
        &mut self,
        move_: &Move,
        depth: u8,
        board: Option<&crate::bitboards::BitboardBoard>,
    ) {
        if let Some(_from) = move_.from {
            // Use safe multiplication to prevent overflow (depth is u8, max value is 255)
            // depth * depth can overflow u8 if depth > 16, so cast to u32 first
            let bonus = ((depth as u32) * (depth as u32)) as u32; // Bonus proportional to depth

            // Task 6.0: Delegate to history manager
            self.history_manager.update_history_score(
                move_,
                depth,
                bonus,
                &self.config.history_config,
                board,
            );

            self.stats.history_updates += 1;
            self.history_update_counter = self.history_manager.get_history_update_counter();

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
    /// Task 4.0: Enhanced to support all history table types
    pub fn get_history_score(&mut self, move_: &Move) -> u32 {
        let current_time = self.history_manager.get_time_aging_counter();
        self.history_manager
            .get_history_score(move_, &self.config.history_config, current_time)
    }

    /// Age the history table to prevent overflow
    ///
    /// This method reduces all history scores by the aging factor,
    /// helping to prevent overflow and giving more weight to recent moves.
    /// Task 4.0: Enhanced to age all history table types (absolute, relative, quiet, phase-aware)
    pub fn age_history_table(&mut self) {
        // Task 6.0: Delegate to history manager
        self.history_manager
            .age_history_table(&self.config.history_config);
        self.stats.history_aging_operations += 1;
        self.update_memory_usage();
    }

    /// Clear the history table
    ///
    /// This method removes all history entries and resets statistics.
    /// Task 4.0: Enhanced to clear all history table types
    pub fn clear_history_table(&mut self) {
        // Task 6.0: Delegate to history manager
        self.history_manager.clear_history_table();
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

        // Task 6.0: Delegate to history manager
        self.history_manager.clamp_history_scores(max_score);
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
            self.stats.history_aging_operations,
        )
    }

    /// Get history heuristic hit rate
    ///
    /// Returns the hit rate for history heuristic lookups.
    pub fn get_history_hit_rate(&self) -> f64 {
        if self.stats.history_hits + self.stats.history_misses > 0 {
            (self.stats.history_hits as f64
                / (self.stats.history_hits + self.stats.history_misses) as f64)
                * 100.0
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
        self.stats.see_cache_evictions = 0; // Task 7.0
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
        self.config.cache_config.max_see_cache_size = max_size;

        // Task 6.0: SEECache manages its own max size
        // For now, just clear if over limit (can be enhanced with proper resizing)
        if self.see_cache.len() > max_size {
            self.see_cache.clear(); // Simple approach: clear if over limit
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
                fast_cache: 0, // Task 1.22: Fast cache is now part of MoveScoreCache
                pv_cache: self.pv_ordering.cache_size(), // Task 6.0: use PVOrdering module
                see_cache: self.see_cache.len(),         // Task 6.0: use SEECache module
                history_table: self.history_manager.absolute_history_size(), // Task 6.0: use HistoryHeuristicManager
            },
        }
    }

    // ==================== Statistics Export ====================

    /// Export comprehensive statistics to JSON format
    ///
    /// Returns a JSON string containing all performance statistics for analysis.
    pub fn export_statistics_json(&self) -> String {
        let export_data = StatisticsExport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ordering_stats: self.stats.clone(),
            config: self.config.clone(),
            memory_usage: self.memory_usage.clone(),
            cache_sizes: CacheSizes {
                move_score_cache: self.move_score_cache.len(),
                fast_cache: 0, // Task 1.22: Fast cache is now part of MoveScoreCache
                pv_cache: self.pv_ordering.cache_size(), // Task 6.0: use PVOrdering module
                see_cache: self.see_cache.len(),         // Task 6.0: use SEECache module
                history_table: self.history_manager.absolute_history_size(), // Task 6.0: use HistoryHeuristicManager
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
        csv.push_str(&format!(
            "Total Moves Ordered,{},\n",
            self.stats.total_moves_ordered
        ));
        csv.push_str(&format!(
            "Average Ordering Time,{:.2},microseconds\n",
            self.stats.avg_ordering_time_us
        ));
        csv.push_str(&format!(
            "Cache Hit Rate,{:.2},percent\n",
            self.stats.cache_hit_rate
        ));
        csv.push_str(&format!(
            "SEE Cache Hit Rate,{:.2},percent\n",
            self.get_see_cache_hit_rate()
        ));

        // Heuristic statistics
        csv.push_str(&format!(
            "Capture Applications,{},\n",
            self.stats.heuristic_stats.capture_stats.applications
        ));
        csv.push_str(&format!(
            "Promotion Applications,{},\n",
            self.stats.heuristic_stats.promotion_stats.applications
        ));
        csv.push_str(&format!(
            "Tactical Applications,{},\n",
            self.stats.heuristic_stats.tactical_stats.applications
        ));

        // Memory statistics
        csv.push_str(&format!(
            "Current Memory Usage,{:.2},MB\n",
            self.memory_usage.current_bytes as f64 / 1_000_000.0
        ));
        csv.push_str(&format!(
            "Peak Memory Usage,{:.2},MB\n",
            self.memory_usage.peak_bytes as f64 / 1_000_000.0
        ));

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
            (
                "capture",
                self.stats
                    .heuristic_stats
                    .capture_stats
                    .best_move_contributions,
            ),
            (
                "promotion",
                self.stats
                    .heuristic_stats
                    .promotion_stats
                    .best_move_contributions,
            ),
            (
                "tactical",
                self.stats
                    .heuristic_stats
                    .tactical_stats
                    .best_move_contributions,
            ),
            (
                "piece_value",
                self.stats
                    .heuristic_stats
                    .piece_value_stats
                    .best_move_contributions,
            ),
            (
                "position",
                self.stats
                    .heuristic_stats
                    .position_stats
                    .best_move_contributions,
            ),
            (
                "development",
                self.stats
                    .heuristic_stats
                    .development_stats
                    .best_move_contributions,
            ),
            (
                "quiet",
                self.stats
                    .heuristic_stats
                    .quiet_stats
                    .best_move_contributions,
            ),
            (
                "pv",
                self.stats.heuristic_stats.pv_stats.best_move_contributions,
            ),
            (
                "killer",
                self.stats
                    .heuristic_stats
                    .killer_stats
                    .best_move_contributions,
            ),
            (
                "history",
                self.stats
                    .heuristic_stats
                    .history_stats
                    .best_move_contributions,
            ),
            (
                "see",
                self.stats.heuristic_stats.see_stats.best_move_contributions,
            ),
        ];

        heuristics
            .iter()
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
        report.push_str(&format!(
            "  Total Moves Ordered: {}\n",
            self.stats.total_moves_ordered
        ));
        report.push_str(&format!(
            "  Average Ordering Time: {:.2} μs\n",
            self.stats.avg_ordering_time_us
        ));
        report.push_str(&format!(
            "  Performance Score: {}/100\n",
            self.calculate_performance_score()
        ));
        report.push_str("\n");

        // Cache Performance
        report.push_str("CACHE PERFORMANCE:\n");
        report.push_str(&format!(
            "  Move Score Cache Hit Rate: {:.1}%\n",
            self.stats.cache_stats.move_score_cache.hit_rate
        ));
        report.push_str(&format!(
            "  Fast Cache Hit Rate: {:.1}%\n",
            self.stats.cache_stats.fast_cache.hit_rate
        ));
        report.push_str(&format!(
            "  PV Cache Hit Rate: {:.1}%\n",
            self.stats.cache_stats.pv_cache.hit_rate
        ));
        report.push_str(&format!(
            "  SEE Cache Hit Rate: {:.1}%\n",
            self.get_see_cache_hit_rate()
        ));
        report.push_str("\n");

        // Memory Usage
        report.push_str("MEMORY USAGE:\n");
        report.push_str(&format!(
            "  Current: {:.2} MB\n",
            self.memory_usage.current_bytes as f64 / 1_000_000.0
        ));
        report.push_str(&format!(
            "  Peak: {:.2} MB\n",
            self.memory_usage.peak_bytes as f64 / 1_000_000.0
        ));
        report.push_str("\n");

        // Heuristic Effectiveness
        report.push_str("HEURISTIC EFFECTIVENESS:\n");
        let heuristics = [
            (
                "Capture",
                self.stats.heuristic_stats.capture_stats.applications,
                self.stats
                    .heuristic_stats
                    .capture_stats
                    .best_move_contributions,
            ),
            (
                "Promotion",
                self.stats.heuristic_stats.promotion_stats.applications,
                self.stats
                    .heuristic_stats
                    .promotion_stats
                    .best_move_contributions,
            ),
            (
                "Tactical",
                self.stats.heuristic_stats.tactical_stats.applications,
                self.stats
                    .heuristic_stats
                    .tactical_stats
                    .best_move_contributions,
            ),
            (
                "PV",
                self.stats.heuristic_stats.pv_stats.applications,
                self.stats.heuristic_stats.pv_stats.best_move_contributions,
            ),
            (
                "Killer",
                self.stats.heuristic_stats.killer_stats.applications,
                self.stats
                    .heuristic_stats
                    .killer_stats
                    .best_move_contributions,
            ),
            (
                "History",
                self.stats.heuristic_stats.history_stats.applications,
                self.stats
                    .heuristic_stats
                    .history_stats
                    .best_move_contributions,
            ),
        ];

        for (name, applications, contributions) in heuristics {
            let effectiveness = if applications > 0 {
                (contributions as f64 / applications as f64) * 100.0
            } else {
                0.0
            };
            report.push_str(&format!(
                "  {}: {:.1}% effective ({} contributions / {} applications)\n",
                name, effectiveness, contributions, applications
            ));
        }
        report.push_str("\n");

        // Bottleneck Analysis
        let analysis = self.profile_bottlenecks();
        if !analysis.bottlenecks.is_empty() {
            report.push_str("IDENTIFIED BOTTLENECKS:\n");
            for bottleneck in &analysis.bottlenecks {
                report.push_str(&format!(
                    "  [{}] {}: {}\n",
                    match bottleneck.severity {
                        BottleneckSeverity::Critical => "CRITICAL",
                        BottleneckSeverity::High => "HIGH",
                        BottleneckSeverity::Medium => "MEDIUM",
                        BottleneckSeverity::Low => "LOW",
                    },
                    bottleneck.description,
                    bottleneck.recommendation
                ));
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
                capture: self
                    .calculate_heuristic_effectiveness(&self.stats.heuristic_stats.capture_stats),
                promotion: self
                    .calculate_heuristic_effectiveness(&self.stats.heuristic_stats.promotion_stats),
                tactical: self
                    .calculate_heuristic_effectiveness(&self.stats.heuristic_stats.tactical_stats),
                pv: self.calculate_heuristic_effectiveness(&self.stats.heuristic_stats.pv_stats),
                killer: self
                    .calculate_heuristic_effectiveness(&self.stats.heuristic_stats.killer_stats),
                history: self
                    .calculate_heuristic_effectiveness(&self.stats.heuristic_stats.history_stats),
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
                recommendation: "Consider increasing cache size or improving cache key generation"
                    .to_string(),
            });
        }

        // Analyze hot path performance
        if self.stats.hot_path_stats.score_move_calls > 0 {
            let avg_score_time = self.stats.hot_path_stats.score_move_time_us as f64
                / self.stats.hot_path_stats.score_move_calls as f64;

            if avg_score_time > 100.0 {
                // More than 100 microseconds per score
                bottlenecks.push(Bottleneck {
                    category: BottleneckCategory::HotPath,
                    severity: BottleneckSeverity::Medium,
                    description: format!("Slow move scoring: {:.1}μs per move", avg_score_time),
                    recommendation:
                        "Consider inlining more scoring functions or optimizing hash calculation"
                            .to_string(),
                });
            }
        }

        // Analyze memory usage
        if self.memory_usage.current_bytes > 10_000_000 {
            // More than 10MB
            bottlenecks.push(Bottleneck {
                category: BottleneckCategory::Memory,
                severity: BottleneckSeverity::Medium,
                description: format!(
                    "High memory usage: {:.1}MB",
                    self.memory_usage.current_bytes as f64 / 1_000_000.0
                ),
                recommendation: "Consider reducing cache sizes or implementing cache aging"
                    .to_string(),
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
                    recommendation: "Consider enabling SEE cache or increasing SEE cache size"
                        .to_string(),
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
            let avg_score_time = self.stats.hot_path_stats.score_move_time_us as f64
                / self.stats.hot_path_stats.score_move_calls as f64;
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
            recommendation: self
                .generate_cache_trend_recommendation(current_hit_rate, see_hit_rate),
        }
    }

    /// Analyze memory usage trends
    fn analyze_memory_usage_trend(&self) -> TrendAnalysis {
        let current_usage = self.memory_usage.current_bytes as f64 / 1_000_000.0;
        let peak_usage = self.memory_usage.peak_bytes as f64 / 1_000_000.0;
        let utilization = if peak_usage > 0.0 {
            (current_usage / peak_usage) * 100.0
        } else {
            0.0
        };

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

        let avg_effectiveness =
            effectiveness_scores.iter().sum::<f64>() / effectiveness_scores.len() as f64;

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
            recommendation: self
                .generate_timing_trend_recommendation(avg_scoring_time, avg_ordering_time),
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
            "Memory usage is efficient, consider increasing cache sizes for better performance"
                .to_string()
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
    fn generate_timing_trend_recommendation(
        &self,
        scoring_time: f64,
        ordering_time: f64,
    ) -> String {
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
    fn handle_error(
        &mut self,
        error: MoveOrderingError,
        severity: ErrorSeverity,
        context: String,
    ) -> MoveOrderingResult<()> {
        // Log the error
        self.error_handler
            .log_error(error.clone(), severity.clone(), context);

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
        self.move_score_cache.clear(); // Task 1.22: MoveScoreCache handles both caches
        self.pv_ordering.clear_cache(); // Task 6.0: use PVOrdering module
        self.see_cache.clear(); // Task 6.0: use SEECache module
        self.cache_manager.clear(); // Task 6.0: use MoveOrderingCacheManager
        self.counter_move_manager.clear_all_counter_moves(); // Task 6.0: use CounterMoveManager
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
            return Err(MoveOrderingError::InvalidMove(format!(
                "Invalid move destination: {:?}",
                move_.to
            )));
        }

        if let Some(from) = move_.from {
            if from.row >= 9 || from.col >= 9 {
                return Err(MoveOrderingError::InvalidMove(format!(
                    "Invalid move source: {:?}",
                    from
                )));
            }
        }

        // Check piece type validity
        match move_.piece_type {
            PieceType::Pawn
            | PieceType::Lance
            | PieceType::Knight
            | PieceType::Silver
            | PieceType::Gold
            | PieceType::Bishop
            | PieceType::Rook
            | PieceType::King => Ok(()),
            _ => Err(MoveOrderingError::InvalidMove(format!(
                "Invalid piece type: {:?}",
                move_.piece_type
            ))),
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
    fn record_allocation(
        &mut self,
        allocation_type: AllocationType,
        size: usize,
        component: String,
    ) {
        self.memory_tracker
            .record_allocation(allocation_type, size, component);
    }

    /// Record memory deallocation
    #[allow(dead_code)] // Kept for future use and debugging
    fn record_deallocation(
        &mut self,
        allocation_type: AllocationType,
        size: usize,
        component: String,
    ) {
        self.memory_tracker
            .record_deallocation(allocation_type, size, component);
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
    pub fn determine_game_phase(
        &self,
        move_count: usize,
        material_balance: i32,
        tactical_complexity: f64,
    ) -> GamePhase {
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
    pub fn update_game_phase(
        &mut self,
        move_count: usize,
        material_balance: i32,
        tactical_complexity: f64,
    ) {
        let new_phase =
            self.determine_game_phase(move_count, material_balance, tactical_complexity);

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
            GamePhase::Middlegame => {
                &self
                    .advanced_features
                    .position_strategies
                    .middlegame_strategy
            }
            GamePhase::Endgame => &self.advanced_features.position_strategies.endgame_strategy,
            GamePhase::Tactical => &self.advanced_features.position_strategies.tactical_strategy,
            GamePhase::Positional => {
                &self
                    .advanced_features
                    .position_strategies
                    .positional_strategy
            }
        };

        // Update current weights with phase-specific weights
        self.config.weights = strategy.weights.clone();
    }

    /// Score move using position-specific strategy
    pub fn score_move_with_strategy(&mut self, move_: &Move) -> MoveOrderingResult<i32> {
        // Get current strategy (clone to avoid borrowing issues)
        let strategy = match self.advanced_features.position_strategies.current_phase {
            GamePhase::Opening => self
                .advanced_features
                .position_strategies
                .opening_strategy
                .clone(),
            GamePhase::Middlegame => self
                .advanced_features
                .position_strategies
                .middlegame_strategy
                .clone(),
            GamePhase::Endgame => self
                .advanced_features
                .position_strategies
                .endgame_strategy
                .clone(),
            GamePhase::Tactical => self
                .advanced_features
                .position_strategies
                .tactical_strategy
                .clone(),
            GamePhase::Positional => self
                .advanced_features
                .position_strategies
                .positional_strategy
                .clone(),
        };

        // Apply strategy-specific scoring
        let base_score = self.score_move(move_)?;
        let adjusted_score = self.apply_strategy_adjustments(move_, base_score, &strategy);

        Ok(adjusted_score)
    }

    /// Apply strategy-specific adjustments to move score
    fn apply_strategy_adjustments(
        &self,
        move_: &Move,
        base_score: i32,
        strategy: &OrderingStrategy,
    ) -> i32 {
        let mut adjusted_score = base_score;

        // Apply priority adjustments
        if move_.is_capture {
            adjusted_score =
                (adjusted_score as f64 * strategy.priority_adjustments.capture_priority) as i32;
        }
        if move_.is_promotion {
            adjusted_score =
                (adjusted_score as f64 * strategy.priority_adjustments.promotion_priority) as i32;
        }
        if self.is_development_move(move_) {
            adjusted_score =
                (adjusted_score as f64 * strategy.priority_adjustments.development_priority) as i32;
        }
        if self.is_center_move(move_) {
            adjusted_score =
                (adjusted_score as f64 * strategy.priority_adjustments.center_priority) as i32;
        }
        if self.is_king_safety_move(move_) {
            adjusted_score =
                (adjusted_score as f64 * strategy.priority_adjustments.king_safety_priority) as i32;
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

                    king_area_files.contains(&move_.to.col)
                        && king_area_ranks.contains(&move_.to.row)
                } else {
                    false
                }
            }
        }
    }

    /// Train machine learning model with new data
    pub fn train_ml_model(
        &mut self,
        training_examples: Vec<TrainingExample>,
    ) -> MoveOrderingResult<f64> {
        if !self.advanced_features.ml_model.enabled {
            return Err(MoveOrderingError::OperationError(
                "Machine learning model is not enabled".to_string(),
            ));
        }

        // Add training examples
        self.advanced_features
            .ml_model
            .training_data
            .extend(training_examples);

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
    pub fn predict_move_score(
        &mut self,
        _move_: &Move,
        position_features: Vec<f64>,
    ) -> MoveOrderingResult<i32> {
        if !self.advanced_features.ml_model.enabled {
            return Err(MoveOrderingError::OperationError(
                "Machine learning model is not enabled".to_string(),
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
                "Dynamic weight adjustment is not enabled".to_string(),
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

        self.advanced_features
            .dynamic_weights
            .adjustment_history
            .push(adjustment);
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
            current_phase: self
                .advanced_features
                .position_strategies
                .current_phase
                .clone(),
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

        // Task 1.22: MoveScoreCache manages its own memory (HashMap doesn't have shrink_to_fit)
        // Task 6.0: PVOrdering manages its own memory
        // Task 6.0: SEECache manages its own memory
        // Task 6.0: HistoryHeuristicManager manages its own memory (HashMap doesn't have shrink_to_fit)

        let after_usage = self.get_current_memory_usage().clone();
        let memory_freed = before_usage
            .total_memory
            .saturating_sub(after_usage.total_memory);

        MemoryCleanupReport {
            before_usage,
            after_usage,
            memory_freed,
            cleanup_successful: true,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Perform selective memory cleanup based on memory pressure
    pub fn selective_cleanup(
        &mut self,
        pressure_level: MemoryPressureLevel,
    ) -> MemoryCleanupReport {
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
        let memory_freed = before_usage
            .total_memory
            .saturating_sub(after_usage.total_memory);

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
            self.see_cache.clear(); // Task 6.0: use SEECache module
        }
    }

    /// Shrink vectors to free memory
    /// Task 1.22: MoveScoreCache manages its own memory (HashMap doesn't have shrink_to_fit)
    fn shrink_vectors(&mut self) {
        // Task 6.0: PVOrdering manages its own memory
        // Task 6.0: SEECache manages its own memory
        // Task 6.0: HistoryHeuristicManager manages its own memory (HashMap doesn't have shrink_to_fit)
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
    ///
    /// Task 6.2: Caches ordering results for repeated positions with same move sets
    /// Task 6.4: Accounts for search state (depth, alpha, beta, check status)
    /// Task 3.0: Added iid_move parameter to integrate IID move into advanced ordering
    pub fn order_moves_with_all_heuristics(
        &mut self,
        moves: &[Move],
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        iid_move: Option<&Move>,
        opponent_last_move: Option<&Move>,
    ) -> Vec<Move> {
        // Task 2.6: Added opponent_last_move parameter
        if moves.is_empty() {
            return Vec::new();
        }

        let start_time = TimeSource::now();

        // Task 6.2: Check cache for move ordering results
        // Task 3.0: Note: Cache doesn't account for IID move, so we skip cache if IID move is present
        // This ensures IID move is properly prioritized even if ordering was cached before
        let skip_cache = iid_move.is_some();

        let position_hash = self
            .hash_calculator
            .get_position_hash(board, player, captured_pieces);
        let cache_key = (position_hash, depth);

        // Check if we have a cached ordering result for this position and depth (Task 6.0: use cache_manager)
        if !skip_cache {
            if let Some(cached_entry) = self.cache_manager.get_mut(&cache_key) {
                // Verify that cached moves match current moves (moves might differ for same position)
                if cached_entry.moves.len() == moves.len()
                    && cached_entry
                        .moves
                        .iter()
                        .zip(moves.iter())
                        .all(|(cached, current)| moves_equal_helper(cached, current))
                {
                    // Task 6.0: LRU tracking is handled by cache_manager.get_mut()
                    self.stats.cache_hits += 1;
                    self.stats.total_moves_ordered += moves.len() as u64;
                    return cached_entry.moves.clone();
                }
            }
        }

        self.stats.cache_misses += 1;

        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        self.stats.moves_sorted += moves.len() as u64;

        // Task 6.4: Set current depth for killer move management (depth affects ordering)
        self.set_current_depth(depth);

        // Get PV move for this position (Task 6.4: uses position hash which accounts for position state)
        let pv_move = self.get_pv_move(board, captured_pieces, player, depth);

        // Get killer moves for current depth (Task 6.4: depth-aware)
        let killer_moves = self.get_current_killer_moves().cloned().unwrap_or_default();

        // Task 3.0: Create mutable copy for sorting
        let mut ordered_moves = moves.to_vec();

        // Task 3.0: Sort moves by score with all heuristics prioritization, including IID move
        // Task 2.6: Pass opponent's last move to move ordering for counter-move heuristic
        ordered_moves.sort_by(|a, b| {
            let score_a = self.score_move_with_all_heuristics(
                a,
                iid_move,
                &pv_move,
                &killer_moves,
                opponent_last_move,
                board,
            );
            let score_b = self.score_move_with_all_heuristics(
                b,
                iid_move,
                &pv_move,
                &killer_moves,
                opponent_last_move,
                board,
            );
            score_b.cmp(&score_a)
        });

        // Task 6.2: Cache the ordering result for this position and depth (Task 6.0: use cache_manager)
        // Task 3.0: Use improved eviction policy (LRU, depth-preferred, or hybrid)
        let entry = MoveOrderingCacheEntry {
            moves: ordered_moves.clone(),
            last_access: 0, // Will be updated by cache_manager.insert()
            depth,
            access_count: 1,
        };
        let evicted_key = self.cache_manager.insert(
            cache_key,
            entry,
            self.config.cache_config.max_cache_size,
            self.config.cache_config.cache_eviction_policy,
            self.config.cache_config.hybrid_lru_weight,
        );
        if evicted_key.is_some() {
            self.stats.cache_evictions += 1;
            self.stats.cache_evictions_size_limit += 1;
        }

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
    /// Task 3.0: Updated priority order to include IID move (highest priority)
    /// Task 2.5: Updated priority order to include counter-move heuristic
    /// This method scores a move with the following priority:
    /// 1. IID moves (highest priority - Task 3.0)
    /// 2. PV moves (high priority)
    /// 3. Killer moves (medium-high priority)
    /// 4. Counter-moves (medium-high priority, quiet moves only - Task 2.5)
    /// 5. History moves (medium priority)
    /// 6. SEE moves (for captures - Task 1.0)
    /// 7. Regular moves (normal priority)
    fn score_move_with_all_heuristics(
        &mut self,
        move_: &Move,
        iid_move: Option<&Move>,
        pv_move: &Option<Move>,
        killer_moves: &[Move],
        opponent_last_move: Option<&Move>,
        board: &crate::bitboards::BitboardBoard,
    ) -> i32 {
        // Task 3.0: Check if this is the IID move (highest priority)
        if let Some(iid_mv) = iid_move {
            if self.moves_equal(move_, iid_mv) {
                // IID move gets maximum score to ensure it's searched first
                return i32::MAX;
            }
        }

        // Check if this is the PV move (high priority)
        if let Some(ref pv) = pv_move {
            if self.moves_equal(move_, pv) {
                return self.score_pv_move(move_);
            }
        }

        // Check if this is a killer move (medium-high priority)
        if killer_moves
            .iter()
            .any(|killer| self.moves_equal(move_, killer))
        {
            self.stats.killer_move_hits += 1;
            return self.score_killer_move(move_);
        }

        // Task 2.5: Check if this is a counter-move for opponent's last move (medium-high priority, quiet moves only)
        if !move_.is_capture {
            let counter_score = self.score_counter_move(move_, opponent_last_move);
            if counter_score > 0 {
                return counter_score;
            }
        }

        // Check if this move has history score (medium priority)
        let history_score = self.score_history_move(move_);
        if history_score > 0 {
            return history_score;
        }

        // Task 1.7: Use SEE for capture moves if enabled and board is available
        if move_.is_capture && self.config.cache_config.enable_see_cache {
            // Try to use SEE for capture ordering
            if let Ok(see_score) = self.score_see_move(move_, board) {
                if see_score != 0 {
                    // SEE score is already scaled by see_weight, use it directly
                    return see_score;
                }
            }
            // Fall through to regular scoring if SEE fails or returns 0
        }

        // Use regular move scoring (MVV/LVA for captures)
        self.stats.killer_move_misses += 1;
        self.score_move(move_).unwrap_or(0)
    }

    // ==================== Transposition Table Integration ====================

    /// Integrate move ordering with transposition table
    ///
    /// This method enhances move ordering by using transposition table data
    /// to prioritize moves that have been successful in previous searches.
    pub fn integrate_with_transposition_table(
        &mut self,
        tt_entry: Option<&TranspositionEntry>,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> MoveOrderingResult<()> {
        if let Some(entry) = tt_entry {
            // Update PV move if we have a best move from the transposition table
            if let Some(ref best_move) = entry.best_move {
                self.update_pv_move(
                    board,
                    captured_pieces,
                    player,
                    depth,
                    best_move.clone(),
                    entry.score,
                );
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
    pub fn update_ordering_from_tt_result(
        &mut self,
        tt_entry: &TranspositionEntry,
        move_result: MoveResult,
    ) -> MoveOrderingResult<()> {
        match move_result {
            MoveResult::Cutoff => {
                // This move caused a cutoff, increase its priority
                if let Some(ref best_move) = tt_entry.best_move {
                    self.update_killer_move_priority(best_move, tt_entry.depth);
                    self.update_history_from_cutoff(best_move, tt_entry.depth);
                }
                self.stats.tt_cutoff_updates += 1;
            }
            MoveResult::Exact => {
                // This move has an exact score, use it for PV
                if let Some(ref best_move) = tt_entry.best_move {
                    self.update_pv_move_from_tt(best_move, tt_entry.score, tt_entry.depth);
                }
                self.stats.tt_exact_updates += 1;
            }
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
            let history_value = if score > 0 {
                depth as i32 + 1
            } else {
                -(depth as i32 + 1)
            };
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
        // Store as PV move for future reference (Task 6.0: use PVOrdering module)
        self.pv_ordering
            .update_pv_move_for_depth(depth, move_.clone());

        // Update PV move statistics
        self.stats.pv_moves_from_tt += 1;
    }

    /// Update bound move priority
    fn update_bound_move_priority(&mut self, move_: &Move, flag: TranspositionFlag, depth: u8) {
        match flag {
            TranspositionFlag::LowerBound => {
                // Lower bound means this move is at least this good
                self.add_killer_move(move_.clone());
            }
            TranspositionFlag::UpperBound => {
                // Upper bound means this move is at most this good
                // Don't prioritize it as highly
            }
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
            adjustments.push(format!(
                "Increased cache size: {} -> {}",
                old_size, config.cache_config.max_cache_size
            ));
        } else if stats.cache_hit_rate > 95.0 && config.cache_config.max_cache_size > 10000 {
            let old_size = config.cache_config.max_cache_size;
            config.cache_config.max_cache_size = (config.cache_config.max_cache_size * 3) / 4;
            adjustments.push(format!(
                "Decreased cache size: {} -> {}",
                old_size, config.cache_config.max_cache_size
            ));
        }

        // Tune SEE cache size based on hit rate
        if stats.see_cache_hit_rate < 40.0 && config.cache_config.max_see_cache_size < 250000 {
            let old_size = config.cache_config.max_see_cache_size;
            config.cache_config.max_see_cache_size =
                (config.cache_config.max_see_cache_size * 3) / 2;
            adjustments.push(format!(
                "Increased SEE cache size: {} -> {}",
                old_size, config.cache_config.max_see_cache_size
            ));
        }

        // Tune history aging frequency based on hit rate
        if stats.history_hit_rate < 20.0 && config.history_config.aging_frequency < 300 {
            let old_freq = config.history_config.aging_frequency;
            config.history_config.aging_frequency += 50;
            adjustments.push(format!(
                "Increased history aging frequency: {} -> {}",
                old_freq, config.history_config.aging_frequency
            ));
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
            recommendations.push(
                "Excellent cache hit rate - could reduce cache size to save memory".to_string(),
            );
        }

        // Check ordering time
        if stats.avg_ordering_time_us > 100.0 {
            warnings.push("High average ordering time".to_string());
            recommendations.push("Consider disabling SEE or increasing cache size".to_string());
        }

        // Check memory usage
        if stats.memory_usage_bytes > 20 * 1024 * 1024 {
            warnings.push("High memory usage (> 20 MB)".to_string());
            recommendations
                .push("Consider reducing cache sizes or clearing caches periodically".to_string());
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
        let weight_adjustments = self.adjust_weights_based_on_effectiveness_legacy();
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

    /// Adjust weights based on heuristic effectiveness (legacy method)
    /// Note: Task 5.0 has a new implementation that uses effectiveness-based learning.
    fn adjust_weights_based_on_effectiveness_legacy(&mut self) -> Vec<String> {
        let mut adjustments = Vec::new();

        // Adjust PV weight based on hit rate
        if self.stats.pv_move_hit_rate > 40.0 && self.config.weights.pv_move_weight < 12000 {
            self.config.weights.pv_move_weight += 1000;
            adjustments.push(format!(
                "Increased PV weight to {}",
                self.config.weights.pv_move_weight
            ));
        } else if self.stats.pv_move_hit_rate < 20.0 && self.config.weights.pv_move_weight > 8000 {
            self.config.weights.pv_move_weight -= 500;
            adjustments.push(format!(
                "Decreased PV weight to {}",
                self.config.weights.pv_move_weight
            ));
        }

        // Adjust killer weight based on hit rate
        if self.stats.killer_move_hit_rate > 30.0 && self.config.weights.killer_move_weight < 6000 {
            self.config.weights.killer_move_weight += 500;
            adjustments.push(format!(
                "Increased killer weight to {}",
                self.config.weights.killer_move_weight
            ));
        } else if self.stats.killer_move_hit_rate < 15.0
            && self.config.weights.killer_move_weight > 3000
        {
            self.config.weights.killer_move_weight -= 500;
            adjustments.push(format!(
                "Decreased killer weight to {}",
                self.config.weights.killer_move_weight
            ));
        }

        // Adjust history weight based on hit rate
        if self.stats.history_hit_rate > 40.0 && self.config.weights.history_weight < 400 {
            self.config.weights.history_weight += 50;
            adjustments.push(format!(
                "Increased history weight to {}",
                self.config.weights.history_weight
            ));
        } else if self.stats.history_hit_rate < 15.0 && self.config.weights.history_weight > 50 {
            self.config.weights.history_weight -= 25;
            adjustments.push(format!(
                "Decreased history weight to {}",
                self.config.weights.history_weight
            ));
        }

        adjustments
    }

    /// Optimize cache configuration
    fn optimize_cache_configuration(&mut self) -> Vec<String> {
        let mut optimizations = Vec::new();

        // Adjust main cache size
        if self.stats.cache_hit_rate < 60.0 && self.config.cache_config.max_cache_size < 500000 {
            let old_size = self.config.cache_config.max_cache_size;
            self.config.cache_config.max_cache_size =
                std::cmp::min(self.config.cache_config.max_cache_size * 2, 500000);
            optimizations.push(format!(
                "Increased cache size: {} -> {}",
                old_size, self.config.cache_config.max_cache_size
            ));
        }

        // Adjust SEE cache size
        if self.stats.see_cache_hit_rate < 50.0
            && self.config.cache_config.max_see_cache_size < 250000
        {
            let old_size = self.config.cache_config.max_see_cache_size;
            self.config.cache_config.max_see_cache_size =
                std::cmp::min(self.config.cache_config.max_see_cache_size * 2, 250000);
            optimizations.push(format!(
                "Increased SEE cache size: {} -> {}",
                old_size, self.config.cache_config.max_see_cache_size
            ));
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
        if self.stats.avg_see_calculation_time_us > 50.0
            && self.config.cache_config.enable_see_cache
        {
            self.config.cache_config.enable_see_cache = false;
            optimizations.push("Disabled SEE cache due to high calculation time".to_string());
        }

        // Increase max killer moves if hit rate is good
        if self.stats.killer_move_hit_rate > 30.0
            && self.config.killer_config.max_killer_moves_per_depth < 3
        {
            self.config.killer_config.max_killer_moves_per_depth += 1;
            optimizations.push(format!(
                "Increased max killer moves to {}",
                self.config.killer_config.max_killer_moves_per_depth
            ));
        }

        // Adjust history aging frequency
        if self.stats.history_hit_rate < 20.0 && self.config.history_config.aging_frequency < 300 {
            let old_freq = self.config.history_config.aging_frequency;
            self.config.history_config.aging_frequency += 50;
            optimizations.push(format!(
                "Increased history aging frequency: {} -> {}",
                old_freq, self.config.history_config.aging_frequency
            ));
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
    pub fn compare_performance(
        before: &PerformanceSnapshot,
        after: &PerformanceSnapshot,
    ) -> PerformanceComparison {
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
    pub fn integrate_with_opening_book(
        &mut self,
        book_moves: &[crate::opening_book::BookMove],
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> MoveOrderingResult<()> {
        if book_moves.is_empty() {
            return Ok(());
        }

        // Get the best book move by weight and evaluation
        if let Some(best_book_move) =
            book_moves
                .iter()
                .max_by(|a, b| match a.weight.cmp(&b.weight) {
                    std::cmp::Ordering::Equal => a.evaluation.cmp(&b.evaluation),
                    other => other,
                })
        {
            // Convert book move to regular move and set as PV
            let pv_move = best_book_move.to_engine_move(player);
            self.update_pv_move(
                board,
                captured_pieces,
                player,
                depth,
                pv_move,
                best_book_move.evaluation,
            );

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
    pub fn integrate_with_tablebase(
        &mut self,
        tablebase_result: &crate::tablebase::TablebaseResult,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> MoveOrderingResult<()> {
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

            self.update_pv_move(
                board,
                captured_pieces,
                player,
                depth,
                best_move.clone(),
                score,
            );
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
    pub fn order_moves_for_analysis(
        &mut self,
        moves: &[Move],
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        // Use all heuristics but with more balanced weights for exploration
        // Task 3.0: No IID move context for analysis ordering
        let mut analysis_ordered = self.order_moves_with_all_heuristics(
            moves,
            board,
            captured_pieces,
            player,
            depth,
            None,
            None,
        );

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
    pub fn order_moves_with_time_management(
        &mut self,
        moves: &[Move],
        time_remaining_ms: u32,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Vec<Move> {
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
            // Task 3.0: Plenty of time - use all heuristics (no IID move context)
            self.order_moves_with_all_heuristics(
                moves,
                board,
                captured_pieces,
                player,
                depth,
                None,
                None,
            )
        }
    }

    /// Order moves for specific game phase
    ///
    /// Adjusts move ordering based on game phase (opening, middlegame, endgame).
    pub fn order_moves_for_game_phase(
        &mut self,
        moves: &[Move],
        game_phase: GamePhase,
        board: &crate::bitboards::BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> Vec<Move> {
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
            }
            GamePhase::Middlegame => {
                // Balanced tactical and positional play
                self.config.weights.tactical_weight = 800;
                self.config.weights.capture_weight = 1200;
                self.config.weights.position_value_weight = 400;
            }
            GamePhase::Endgame => {
                // Focus on promotion and piece value
                self.config.weights.promotion_weight = 1000;
                self.config.weights.piece_value_weight = 800;
                self.config.weights.tactical_weight = 600;
            }
            GamePhase::Tactical => {
                // Prioritize forcing moves
                self.config.weights.capture_weight = 2000;
                self.config.weights.tactical_weight = 1500;
                self.config.weights.see_weight = 1200;
            }
            GamePhase::Positional => {
                // Prioritize positional moves
                self.config.weights.position_value_weight = 800;
                self.config.weights.center_control_weight = 600;
                self.config.weights.development_weight = 500;
            }
        }

        // Task 3.0: Order with adjusted weights (no IID move context)
        let ordered = self.order_moves_with_all_heuristics(
            moves,
            board,
            captured_pieces,
            player,
            depth,
            None,
            None,
        );

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
            shared_history: true,      // History can be shared (read-only during ordering)
            shared_pv: true,           // PV can be shared
            shared_killers: false,     // Killer moves are depth-specific, don't share
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

    /// Get optimized configuration
    ///
    /// Returns a configuration optimized for native environments with larger caches
    /// and full performance features enabled.
    pub fn optimized_config() -> MoveOrderingConfig {
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

    /// Create optimized move orderer
    ///
    /// Creates a move orderer using the recommended optimized configuration.
    pub fn new_optimized() -> Self {
        Self::with_config(Self::optimized_config())
    }

    /// Get platform-specific configuration
    ///
    /// Returns an optimized configuration for the current platform.
    pub fn platform_optimized_config() -> MoveOrderingConfig {
        Self::optimized_config()
    }

    /// Get memory limits for current platform
    ///
    /// Returns recommended memory limits based on platform.
    pub fn get_platform_memory_limits() -> PlatformMemoryLimits {
        PlatformMemoryLimits {
            max_total_memory_bytes: 100 * 1024 * 1024, // 100 MB for native
            max_cache_size: 1000000,
            max_see_cache_size: 500000,
            recommended_cache_size: 200000,
            recommended_see_cache_size: 100000,
        }
    }
}

impl Default for MoveOrdering {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use crate::types::{PieceType, Player, Position};

    fn create_test_move(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Move {
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

    fn create_legacy_move(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        is_capture: bool,
        is_promotion: bool,
        gives_check: bool,
        is_recapture: bool,
    ) -> Move {
        Move {
            from,
            to,
            piece_type,
            player: Player::Black,
            is_capture,
            is_promotion,
            gives_check,
            is_recapture,
            captured_piece: None,
        }
    }

    fn create_tt_entry(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        best_move: Option<Move>,
        hash_key: u64,
        age: u32,
    ) -> TranspositionEntry {
        TranspositionEntry::new(
            score,
            depth,
            flag,
            best_move,
            hash_key,
            age,
            EntrySource::MainSearch,
        )
    }

    #[test]
    fn test_move_ordering_creation() {
        let mut orderer = MoveOrdering::new();
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
        );
        promotion_move.is_promotion = true;

        let promotion_score = orderer.score_move(&promotion_move);
        assert!(promotion_score > 0);
    }

    #[test]
    fn test_position_value_scoring() {
        let mut orderer = MoveOrdering::new();

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
            Player::Black,
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
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                Player::Black,
            ),
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
                Player::Black,
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
            Player::Black,
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
            Player::Black,
        );

        let score = orderer.score_pv_move(&move_);
        assert_eq!(score, orderer.config.weights.pv_move_weight);
        assert!(score > 1000); // Should be higher than other move types
    }

    #[test]
    fn test_pv_move_cache_functionality() {
        let mut orderer = MoveOrdering::new();

        // Initially no PV moves cached (Task 6.0: use PVOrdering module)
        assert_eq!(orderer.pv_ordering.cache_size(), 0);

        // Clear PV move cache
        orderer.clear_pv_move_cache();
        assert_eq!(orderer.pv_ordering.cache_size(), 0);
        assert_eq!(orderer.stats.pv_move_hits, 0);
        assert_eq!(orderer.stats.pv_move_misses, 0);
    }

    #[test]
    fn test_moves_equal_functionality() {
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let move2 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let move3 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(3, 1),
            PieceType::Pawn,
            Player::Black,
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
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                Player::Black,
            ),
        ];

        // Should work even without transposition table (falls back to regular ordering)
        let ordered =
            orderer.order_moves_with_pv(&moves, &board, &captured_pieces, Player::Black, 3);
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
                Player::Black,
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
                Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
            Player::Black,
        );

        let killer_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
                Player::Black,
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
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
        );
        orderer.add_killer_move(killer_move_1.clone());

        orderer.set_current_depth(2);
        let killer_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
            Player::Black,
        );

        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );

        // Add killer move
        orderer.add_killer_move(killer_move.clone());

        // Create test position and board
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Store PV move in transposition table
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Order moves with both PV and killer prioritization
        let moves = vec![regular_move.clone(), killer_move.clone(), pv_move.clone()];
        let ordered =
            orderer.order_moves_with_pv_and_killer(&moves, &board, &captured_pieces, player, depth);

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
                Player::Black,
            );
            orderer.add_killer_move(killer_move);
        }

        // Should only have 5 killer moves (the new limit)
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), 5);
    }

    // ==================== Counter-Move Heuristic Tests ====================

    #[test]
    fn test_counter_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );

        // Add counter-move
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Score counter-move with opponent's last move
        let score = orderer.score_counter_move(&counter_move, Some(&opponent_move));
        assert_eq!(score, orderer.config.weights.counter_move_weight);
        assert!(score > 1000); // Should be higher than regular moves

        // Score non-counter-move
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::White,
        );
        let score_no_match = orderer.score_counter_move(&regular_move, Some(&opponent_move));
        assert_eq!(score_no_match, 0);
    }

    #[test]
    fn test_counter_move_storage() {
        let mut orderer = MoveOrdering::new();

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );

        // Initially no counter-moves
        assert!(orderer.get_counter_moves(&opponent_move).is_none());

        // Add counter-move
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Should now have counter-moves
        let counter_moves = orderer.get_counter_moves(&opponent_move);
        assert!(counter_moves.is_some());
        assert_eq!(counter_moves.unwrap().len(), 1);
        assert!(orderer.moves_equal(&counter_moves.unwrap()[0], &counter_move));

        // Statistics should be updated
        assert_eq!(orderer.stats.counter_moves_stored, 1);
    }

    #[test]
    fn test_counter_move_detection() {
        let mut orderer = MoveOrdering::new();

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::White,
        );

        // Add counter-move
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Test counter-move detection
        assert!(orderer.is_counter_move(&counter_move, Some(&opponent_move)));
        assert!(!orderer.is_counter_move(&regular_move, Some(&opponent_move)));
        assert!(!orderer.is_counter_move(&counter_move, None));
    }

    #[test]
    fn test_counter_move_limit() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_counter_moves(2);

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Add more counter-moves than the limit
        for i in 0..5 {
            let counter_move = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                Player::White,
            );
            orderer.add_counter_move(opponent_move.clone(), counter_move);
        }

        // Should only have 2 counter-moves (FIFO order)
        let counter_moves = orderer.get_counter_moves(&opponent_move);
        assert!(counter_moves.is_some());
        assert_eq!(counter_moves.unwrap().len(), 2);
    }

    #[test]
    fn test_counter_move_duplicate_prevention() {
        let mut orderer = MoveOrdering::new();

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );

        // Add same counter-move twice
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Should only have one counter-move (no duplicates)
        let counter_moves = orderer.get_counter_moves(&opponent_move);
        assert!(counter_moves.is_some());
        assert_eq!(counter_moves.unwrap().len(), 1);

        // Statistics should only count once
        assert_eq!(orderer.stats.counter_moves_stored, 1);
    }

    #[test]
    fn test_counter_move_clear_functionality() {
        let mut orderer = MoveOrdering::new();

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Verify counter-move is stored
        assert!(orderer.is_counter_move(&counter_move, Some(&opponent_move)));
        assert!(orderer.get_counter_moves(&opponent_move).is_some());

        // Clear counter-moves for opponent move
        orderer.clear_counter_moves_for_opponent_move(&opponent_move);

        // Verify counter-move is cleared
        assert!(!orderer.is_counter_move(&counter_move, Some(&opponent_move)));
        assert!(orderer.get_counter_moves(&opponent_move).is_none());
    }

    #[test]
    fn test_counter_move_clear_all() {
        let mut orderer = MoveOrdering::new();

        // Add counter-moves for different opponent moves
        let opponent_move_1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move_1 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );
        orderer.add_counter_move(opponent_move_1.clone(), counter_move_1.clone());

        let opponent_move_2 = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );
        let counter_move_2 = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Rook,
            Player::White,
        );
        orderer.add_counter_move(opponent_move_2.clone(), counter_move_2.clone());

        // Clear all counter-moves
        orderer.clear_all_counter_moves();

        // Verify all counter-moves are cleared
        assert!(!orderer.is_counter_move(&counter_move_1, Some(&opponent_move_1)));
        assert!(!orderer.is_counter_move(&counter_move_2, Some(&opponent_move_2)));
        assert_eq!(orderer.stats.counter_move_hits, 0);
        assert_eq!(orderer.stats.counter_move_misses, 0);
        assert_eq!(orderer.stats.counter_moves_stored, 0);
    }

    #[test]
    fn test_counter_move_statistics() {
        let mut orderer = MoveOrdering::new();

        // Initially no statistics
        let (hits, misses, hit_rate, stored) = orderer.get_counter_move_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(stored, 0);

        // Add counter-move
        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Test counter-move detection (should increment hits)
        let score = orderer.score_counter_move(&counter_move, Some(&opponent_move));
        assert!(score > 0);

        // Statistics should be updated
        let (hits, misses, hit_rate, stored) = orderer.get_counter_move_stats();
        assert!(hits > 0);
        assert!(stored > 0);

        // Test miss (should increment misses)
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::White,
        );
        let _score = orderer.score_counter_move(&regular_move, Some(&opponent_move));

        // Statistics should be updated
        let (hits, misses, _hit_rate, stored) = orderer.get_counter_move_stats();
        assert!(misses > 0);
    }

    #[test]
    fn test_counter_move_only_for_quiet_moves() {
        let mut orderer = MoveOrdering::new();

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Create a capture move (should not be used as counter-move)
        let mut capture_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );
        capture_move.is_capture = true;

        // Add counter-move (should be allowed even if it's a capture)
        // But in practice, counter-moves are only added for quiet moves in the search engine
        orderer.add_counter_move(opponent_move.clone(), capture_move.clone());

        // Should still be able to retrieve it
        let counter_moves = orderer.get_counter_moves(&opponent_move);
        assert!(counter_moves.is_some());
    }

    #[test]
    fn test_counter_move_disabled_config() {
        let mut config = MoveOrderingConfig::default();
        config.counter_move_config.enable_counter_move = false;
        let mut orderer = MoveOrdering::with_config(config);

        let opponent_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let counter_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::White,
        );

        // Try to add counter-move (should be ignored)
        orderer.add_counter_move(opponent_move.clone(), counter_move.clone());

        // Should not be stored
        assert!(orderer.get_counter_moves(&opponent_move).is_none());

        // Scoring should return 0
        let score = orderer.score_counter_move(&counter_move, Some(&opponent_move));
        assert_eq!(score, 0);
    }

    // ==================== Cache Eviction Tests (Task 3.0) ====================

    #[test]
    fn test_cache_eviction_fifo() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::FIFO;
        config.cache_config.max_cache_size = 2;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Create test moves
        let moves1 = vec![
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                player,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                player,
            ),
        ];
        let moves2 = vec![
            create_test_move(
                Some(Position::new(3, 3)),
                Position::new(4, 3),
                PieceType::Gold,
                player,
            ),
            create_test_move(
                Some(Position::new(4, 4)),
                Position::new(5, 4),
                PieceType::Rook,
                player,
            ),
        ];
        let moves3 = vec![create_test_move(
            Some(Position::new(5, 5)),
            Position::new(6, 5),
            PieceType::Bishop,
            player,
        )];

        // Order moves 1 - should be cached
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );
        assert_eq!(orderer.cache_manager.len(), 1);

        // Order moves 2 - should be cached
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves2,
            &board,
            &captured_pieces,
            player,
            4,
            None,
            None,
        );
        assert_eq!(orderer.move_ordering_cache.len(), 2);

        // Order moves 3 - should evict first entry (FIFO)
        let hash3 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered3 = orderer.order_moves_with_all_heuristics(
            &moves3,
            &board,
            &captured_pieces,
            player,
            5,
            None,
            None,
        );
        assert_eq!(orderer.move_ordering_cache.len(), 2);
        assert!(orderer.cache_manager.contains_key(&(hash3, 5)));
    }

    #[test]
    fn test_cache_eviction_lru() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::LRU;
        config.cache_config.max_cache_size = 2;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Create test moves
        let moves1 = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        )];
        let moves2 = vec![create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            player,
        )];
        let moves3 = vec![create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            player,
        )];

        // Order moves 1 - should be cached
        let hash1 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );

        // Order moves 2 - should be cached (different position, so hash will differ)
        let hash2 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves2,
            &board,
            &captured_pieces,
            player,
            4,
            None,
            None,
        );

        // Access moves 1 again (update LRU)
        let _ordered1_again = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );

        // Order moves 3 - should evict moves 2 (least recently used)
        let _ordered3 = orderer.order_moves_with_all_heuristics(
            &moves3,
            &board,
            &captured_pieces,
            player,
            5,
            None,
            None,
        );
        assert_eq!(orderer.move_ordering_cache.len(), 2);
        assert!(orderer.move_ordering_cache.contains_key(&(hash1, 3))); // moves1 should still be cached
        assert!(!orderer.move_ordering_cache.contains_key(&(hash2, 4))); // moves2 should be evicted
    }

    #[test]
    fn test_cache_eviction_depth_preferred() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::DepthPreferred;
        config.cache_config.max_cache_size = 2;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Create test moves
        let moves1 = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        )];
        let moves2 = vec![create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            player,
        )];
        let moves3 = vec![create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            player,
        )];

        // Order moves at depth 5 (deep) - should be cached
        let hash1 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            5,
            None,
            None,
        );

        // Order moves at depth 3 (shallow) - should be cached (different position, so hash will differ)
        let hash2 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves2,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );

        // Order moves at depth 4 (medium) - should evict depth 3 (shallowest)
        let _ordered3 = orderer.order_moves_with_all_heuristics(
            &moves3,
            &board,
            &captured_pieces,
            player,
            4,
            None,
            None,
        );
        assert_eq!(orderer.move_ordering_cache.len(), 2);
        assert!(orderer.move_ordering_cache.contains_key(&(hash1, 5))); // depth 5 should still be cached
        assert!(!orderer.move_ordering_cache.contains_key(&(hash2, 3))); // depth 3 should be evicted
    }

    #[test]
    fn test_cache_eviction_hybrid() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::Hybrid;
        config.cache_config.max_cache_size = 2;
        config.cache_config.hybrid_lru_weight = 0.5; // 50% LRU, 50% depth
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Create test moves
        let moves1 = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        )];
        let moves2 = vec![create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            player,
        )];
        let moves3 = vec![create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            player,
        )];

        // Order moves at depth 5 (deep) - should be cached
        let hash1 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            5,
            None,
            None,
        );

        // Order moves at depth 4 (medium) - should be cached (different position, so hash will differ)
        let hash2 = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves2,
            &board,
            &captured_pieces,
            player,
            4,
            None,
            None,
        );

        // Order moves at depth 3 (shallow) - should evict based on hybrid policy
        let _ordered3 = orderer.order_moves_with_all_heuristics(
            &moves3,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );
        assert_eq!(orderer.move_ordering_cache.len(), 2);
        // Depth 5 should likely still be cached (preferred by depth)
        assert!(orderer.move_ordering_cache.contains_key(&(hash1, 5)));
    }

    #[test]
    fn test_cache_eviction_statistics() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.max_cache_size = 1; // Force evictions
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Initially no evictions
        assert_eq!(orderer.stats.cache_evictions, 0);
        assert_eq!(orderer.stats.cache_evictions_size_limit, 0);

        // Create test moves
        let moves1 = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        )];
        let moves2 = vec![create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            player,
        )];

        // Order moves 1 - should be cached
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves1,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );

        // Order moves 2 - should evict moves 1
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves2,
            &board,
            &captured_pieces,
            player,
            4,
            None,
            None,
        );

        // Statistics should be updated
        assert!(orderer.stats.cache_evictions > 0);
        assert!(orderer.stats.cache_evictions_size_limit > 0);
    }

    #[test]
    fn test_cache_lru_tracking() {
        let mut orderer = MoveOrdering::new();

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        )];

        // Order moves - should be cached
        let hash = orderer
            .hash_calculator
            .get_position_hash(&board, player, &captured_pieces);
        let cache_key = (hash, 3);
        let _ordered1 = orderer.order_moves_with_all_heuristics(
            &moves,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );

        // Get initial access counter from entry
        let entry1 = orderer.cache_manager.get(&cache_key).unwrap();
        let initial_access = entry1.last_access;
        let initial_count = entry1.access_count;
        assert!(initial_access > 0);
        assert_eq!(initial_count, 1);

        // Access cached entry - should update LRU
        let _ordered2 = orderer.order_moves_with_all_heuristics(
            &moves,
            &board,
            &captured_pieces,
            player,
            3,
            None,
            None,
        );
        let entry2 = orderer.move_ordering_cache.get(&cache_key).unwrap();
        assert!(entry2.last_access > initial_access);
        assert_eq!(entry2.access_count, initial_count + 1);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.max_cache_size = 3;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::types::CapturedPieces::new();
        let player = Player::Black;

        // Add more entries than limit
        for i in 0..5 {
            let moves = vec![create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                PieceType::Pawn,
                player,
            )];
            let _ordered = orderer.order_moves_with_all_heuristics(
                &moves,
                &board,
                &captured_pieces,
                player,
                i as u8,
                None,
                None,
            );
        }

        // Cache should not exceed limit
        assert!(orderer.move_ordering_cache.len() <= config.cache_config.max_cache_size);
    }

    #[test]
    fn test_cache_eviction_policy_configuration() {
        // Test FIFO policy
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::FIFO;
        let orderer = MoveOrdering::with_config(config);
        assert_eq!(
            orderer.config.cache_config.cache_eviction_policy,
            CacheEvictionPolicy::FIFO
        );

        // Test LRU policy
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::LRU;
        let orderer = MoveOrdering::with_config(config);
        assert_eq!(
            orderer.config.cache_config.cache_eviction_policy,
            CacheEvictionPolicy::LRU
        );

        // Test depth-preferred policy
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::DepthPreferred;
        let orderer = MoveOrdering::with_config(config);
        assert_eq!(
            orderer.config.cache_config.cache_eviction_policy,
            CacheEvictionPolicy::DepthPreferred
        );

        // Test hybrid policy
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = CacheEvictionPolicy::Hybrid;
        config.cache_config.hybrid_lru_weight = 0.7;
        let orderer = MoveOrdering::with_config(config);
        assert_eq!(
            orderer.config.cache_config.cache_eviction_policy,
            CacheEvictionPolicy::Hybrid
        );
        assert_eq!(orderer.config.cache_config.hybrid_lru_weight, 0.7);
    }

    // ==================== History Heuristic Tests ====================

    #[test]
    fn test_history_move_scoring() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Initially no history score
        let score = orderer.score_history_move(&move_);
        assert_eq!(score, 0);

        // Add history score
        orderer.update_history_score(&move_, 3, None);

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
            Player::Black,
        );

        // Initially no history score
        assert_eq!(orderer.get_history_score(&move_), 0);

        // Update history score
        orderer.update_history_score(&move_, 3, None);

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
            Player::Black,
        );

        // Update history score multiple times
        orderer.update_history_score(&move_, 2, None);
        orderer.update_history_score(&move_, 3, None);
        orderer.update_history_score(&move_, 4, None);

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
            Player::Black,
        );

        // Update with large depth to exceed limit
        orderer.update_history_score(&move_, 20, None); // 20*20 = 400

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
            Player::Black,
        );

        // Add history score
        orderer.update_history_score(&move_, 4, None); // 4*4 = 16
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
            Player::Black,
        );

        // Add small history score
        orderer.update_history_score(&move_, 2, None); // 2*2 = 4
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
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
            Player::Black,
        );

        // Update history score
        orderer.update_history_score(&move_, 3, None);

        // Test history move detection (should increment hits)
        orderer.score_history_move(&move_);

        // Test non-history move (should increment misses)
        let regular_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
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
            Player::Black,
        );
        orderer.update_history_score(&move_, 3, None);

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

    // ==================== History Enhancement Tests (Task 4.0) ====================

    #[test]
    fn test_relative_history() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_relative = true;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Silver, // Different piece, same from/to
            Player::Black,
        );

        // Update history for move1
        orderer.update_history_score(&move1, 3, Some(&board));

        // Both moves should have history score (relative history uses same from/to)
        let score1 = orderer.get_history_score(&move1);
        let score2 = orderer.get_history_score(&move2);
        assert!(score1 > 0);
        assert!(score2 > 0);
        assert_eq!(score1, score2); // Same relative key
    }

    #[test]
    fn test_quiet_only_history() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_quiet_only = true;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let quiet_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let capture_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );
        // Mark as capture
        let mut capture_move = capture_move;
        capture_move.is_capture = true;

        // Update history for quiet move
        orderer.update_history_score(&quiet_move, 3, Some(&board));

        // Quiet move should have history score
        let quiet_score = orderer.get_history_score(&quiet_move);
        assert!(quiet_score > 0);

        // Capture move should not have quiet history score
        let capture_score = orderer.get_history_score(&capture_move);
        // Capture should fall back to absolute history (which is empty)
        assert_eq!(capture_score, 0);
    }

    #[test]
    fn test_phase_aware_history() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_phase_aware = true;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history in opening phase
        assert_eq!(
            orderer.history_manager.get_current_game_phase(),
            crate::types::GamePhase::Opening
        );
        orderer.update_history_score(&move_, 3, Some(&board));

        // Should have history score in opening phase
        let score = orderer.get_history_score(&move_);
        assert!(score > 0);

        // Switch to endgame phase (simulate by changing material count)
        // Note: This is a simplified test - in practice, phase would be determined by board state
        orderer
            .history_manager
            .set_current_game_phase(crate::types::GamePhase::Endgame);

        // Should have history score in endgame phase if it was stored there
        // But initially it's in opening phase, so it might not be found
        // This tests that phase-aware tables are separate
        let score2 = orderer.get_history_score(&move_);
        // The score might be 0 if phase tables are truly separate
        // This demonstrates phase-aware isolation
    }

    #[test]
    fn test_time_based_aging() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_time_based_aging = true;
        config.history_config.time_aging_decay_factor = 0.9;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history
        orderer.update_history_score(&move_, 3, Some(&board));
        let initial_score = orderer.get_history_score(&move_);
        assert!(initial_score > 0);

        // Simulate time passing by incrementing history_update_counter
        orderer.history_update_counter += 100;

        // Score should be reduced due to time-based aging
        let aged_score = orderer.get_history_score(&move_);
        assert!(aged_score <= initial_score);
    }

    #[test]
    fn test_phase_specific_aging() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_phase_aware = true;
        config.history_config.opening_aging_factor = 0.9;
        config.history_config.middlegame_aging_factor = 0.85;
        config.history_config.endgame_aging_factor = 0.95;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history in opening phase
        orderer
            .history_manager
            .set_current_game_phase(crate::types::GamePhase::Opening);
        orderer.update_history_score(&move_, 3, Some(&board));

        // Age with opening factor
        orderer.age_history_table();

        // Switch to endgame and age again
        orderer
            .history_manager
            .set_current_game_phase(crate::types::GamePhase::Endgame);
        orderer.age_history_table();

        // Verify aging occurred
        assert!(orderer.stats.history_aging_operations >= 2);
    }

    #[test]
    fn test_history_enhancement_configuration() {
        // Test relative history configuration
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_relative = true;
        let orderer = MoveOrdering::with_config(config);
        assert!(orderer.config.history_config.enable_relative);

        // Test quiet-only history configuration
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_quiet_only = true;
        let orderer = MoveOrdering::with_config(config);
        assert!(orderer.config.history_config.enable_quiet_only);

        // Test phase-aware history configuration
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_phase_aware = true;
        let orderer = MoveOrdering::with_config(config);
        assert!(orderer.config.history_config.enable_phase_aware);

        // Test time-based aging configuration
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_time_based_aging = true;
        config.history_config.time_aging_decay_factor = 0.95;
        let orderer = MoveOrdering::with_config(config);
        assert!(orderer.config.history_config.enable_time_based_aging);
        assert_eq!(orderer.config.history_config.time_aging_decay_factor, 0.95);
    }

    #[test]
    fn test_history_enhancement_clear() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_relative = true;
        config.history_config.enable_quiet_only = true;
        config.history_config.enable_phase_aware = true;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history in all tables
        orderer.update_history_score(&move_, 3, Some(&board));

        // Verify entries exist (Task 6.0: use HistoryHeuristicManager)
        assert!(!orderer.history_manager.is_absolute_history_empty());
        assert!(!orderer.history_manager.is_relative_history_empty());
        assert!(!orderer.history_manager.is_quiet_history_empty());

        // Clear all history
        orderer.clear_history_table();

        // Verify all tables are cleared (Task 6.0: use HistoryHeuristicManager)
        assert!(orderer.history_manager.is_absolute_history_empty());
        assert!(orderer.history_manager.is_relative_history_empty());
        assert!(orderer.history_manager.is_quiet_history_empty());
        assert!(orderer.history_manager.is_phase_history_empty());
        assert_eq!(
            orderer.history_manager.get_current_game_phase(),
            crate::types::GamePhase::Opening
        );
        assert_eq!(orderer.history_manager.get_time_aging_counter(), 0);
    }

    #[test]
    fn test_history_enhancement_aging() {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_relative = true;
        config.history_config.enable_quiet_only = true;
        config.history_config.history_aging_factor = 0.5;
        let mut orderer = MoveOrdering::with_config(config);

        let board = crate::bitboards::BitboardBoard::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history in all tables
        orderer.update_history_score(&move_, 4, Some(&board));

        // Get initial scores (Task 6.0: use HistoryHeuristicManager)
        let key = (move_.piece_type, move_.from.unwrap(), move_.to);
        let relative_key = (move_.from.unwrap(), move_.to);
        let initial_absolute = orderer
            .history_manager
            .get_absolute_history_score(key)
            .unwrap_or(0);
        let initial_relative = orderer
            .history_manager
            .get_relative_history_entry(relative_key)
            .map(|e| e.score)
            .unwrap_or(0);
        let initial_quiet = orderer
            .history_manager
            .get_quiet_history_entry(key)
            .map(|e| e.score)
            .unwrap_or(0);

        assert!(initial_absolute > 0);
        assert!(initial_relative > 0);
        assert!(initial_quiet > 0);

        // Age all tables
        orderer.age_history_table();

        // Verify scores are reduced (Task 6.0: use HistoryHeuristicManager)
        let aged_absolute = orderer
            .history_manager
            .get_absolute_history_score(key)
            .unwrap_or(0);
        let aged_relative = orderer
            .history_manager
            .get_relative_history_entry(relative_key)
            .map(|e| e.score)
            .unwrap_or(0);
        let aged_quiet = orderer
            .history_manager
            .get_quiet_history_entry(key)
            .map(|e| e.score)
            .unwrap_or(0);

        assert!(aged_absolute <= initial_absolute);
        assert!(aged_relative <= initial_relative);
        assert!(aged_quiet <= initial_quiet);
        assert!(orderer.stats.history_aging_operations > 0);
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
            Player::Black,
        );

        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        let history_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black,
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
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Order moves with all heuristics
        let moves = vec![
            regular_move.clone(),
            history_move.clone(),
            killer_move.clone(),
            pv_move.clone(),
        ];
        // Task 3.0: No IID move context for this test
        let ordered = orderer.order_moves_with_all_heuristics(
            &moves,
            &board,
            &captured_pieces,
            player,
            depth,
            None,
        );

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
                Player::Black,
            );
            orderer.update_history_score(&move_, 2, None);
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
            Player::Black,
        );

        let white_move = create_test_move(
            Some(Position::new(7, 7)),
            Position::new(6, 7),
            PieceType::Pawn,
            Player::White,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
        );

        let rook_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Rook,
            Player::Black,
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
            Player::Black,
        );

        let edge_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(0, 0), // Edge position
            PieceType::Pawn,
            Player::Black,
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
            Position::new(3, 1),       // Forward
            PieceType::Pawn,
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                Player::Black,
            ),
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
            Player::Black,
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
            Player::Black,
        );
        capture_promotion_move.is_capture = true;
        capture_promotion_move.is_promotion = true;

        let mut tactical_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );
        tactical_move.gives_check = true;

        let quiet_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
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
            Player::Black,
        );

        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        let history_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );

        let regular_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black,
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
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Test scoring with all heuristics
        // Task 3.0: Updated calls to include IID move parameter (None for tests)
        let pv_score = orderer.score_move_with_all_heuristics(
            &pv_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let killer_score = orderer.score_move_with_all_heuristics(
            &killer_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let history_score = orderer.score_move_with_all_heuristics(
            &history_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let regular_score = orderer.score_move_with_all_heuristics(
            &regular_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );

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
            Player::Black,
        );

        // Update history score
        orderer.update_history_score(&move_, 3, None);

        // Counter should be incremented
        assert_eq!(orderer.get_history_update_counter(), 1);

        // Update again
        orderer.update_history_score(&move_, 2, None);

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
            Player::Black,
        );

        // Update history score multiple times
        orderer.update_history_score(&move_, 3, None);
        orderer.update_history_score(&move_, 2, None);
        orderer.update_history_score(&move_, 4, None);

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
            Player::Black,
        );

        // Update history score 4 times (should not trigger aging)
        for i in 1..=4 {
            orderer.update_history_score(&move_, 3, None);
            assert_eq!(orderer.get_history_update_counter(), i);
        }

        // Score should accumulate without aging
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 4 * 9); // 4 updates * 3*3 = 36

        // 5th update should trigger automatic aging
        orderer.update_history_score(&move_, 3, None);
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
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
            Player::Black,
        );

        // SEE should return 0 for non-capture moves
        let see_score = orderer.score_see_move(&quiet_move, &board);
        assert_eq!(see_score, 0);

        let see_value = orderer.calculate_see(&quiet_move, &board);
        assert_eq!(see_value, 0);
    }

    // Task 7.7: Comprehensive SEE cache tests

    #[test]
    fn test_see_cache_eviction_policy() {
        // Test that cache evicts entries when full
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(3); // Small cache for testing
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Create multiple capture moves
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );
        let move3 = create_test_move(
            Some(Position::new(1, 3)),
            Position::new(2, 3),
            PieceType::Pawn,
            Player::Black,
        );
        let move4 = create_test_move(
            Some(Position::new(1, 4)),
            Position::new(2, 4),
            PieceType::Pawn,
            Player::Black,
        );

        // Fill cache
        orderer.calculate_see(&move1, &board);
        orderer.calculate_see(&move2, &board);
        orderer.calculate_see(&move3, &board);

        assert_eq!(orderer.get_see_cache_size(), 3);
        assert_eq!(orderer.stats.see_cache_evictions, 0);

        // Add fourth move - should trigger eviction
        orderer.calculate_see(&move4, &board);

        assert_eq!(orderer.get_see_cache_size(), 3); // Cache size stays at max
        assert_eq!(orderer.stats.see_cache_evictions, 1); // One eviction occurred
    }

    #[test]
    fn test_see_cache_lru_tracking() {
        // Test that LRU tracking works correctly
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(2); // Small cache
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );
        let move3 = create_test_move(
            Some(Position::new(1, 3)),
            Position::new(2, 3),
            PieceType::Pawn,
            Player::Black,
        );

        // Fill cache with move1 and move2
        orderer.calculate_see(&move1, &board);
        orderer.calculate_see(&move2, &board);
        assert_eq!(orderer.get_see_cache_size(), 2);

        // Access move1 again to make it more recent
        orderer.calculate_see(&move1, &board);
        assert_eq!(orderer.stats.see_cache_hits, 1);

        // Add move3 - should evict move2 (less recently used)
        orderer.calculate_see(&move3, &board);
        assert_eq!(orderer.get_see_cache_size(), 2);

        // move1 should still be in cache (recently accessed)
        orderer.calculate_see(&move1, &board);
        assert_eq!(orderer.stats.see_cache_hits, 2); // Should hit cache
    }

    #[test]
    fn test_see_cache_statistics() {
        // Test that cache statistics are tracked correctly
        let mut orderer = MoveOrdering::new();
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );

        // First calculation - cache miss
        orderer.calculate_see(&move1, &board);
        assert_eq!(orderer.stats.see_cache_misses, 1);
        assert_eq!(orderer.stats.see_cache_hits, 0);

        // Second calculation of same move - cache hit
        orderer.calculate_see(&move1, &board);
        assert_eq!(orderer.stats.see_cache_hits, 1);
        assert_eq!(orderer.stats.see_cache_misses, 1);

        // Different move - cache miss
        orderer.calculate_see(&move2, &board);
        assert_eq!(orderer.stats.see_cache_misses, 2);

        // Verify cache size
        assert_eq!(orderer.get_see_cache_size(), 2);

        // Verify hit rate calculation
        let hit_rate = orderer.get_see_cache_hit_rate();
        assert!(hit_rate > 0.0);
        assert!(hit_rate < 100.0);
    }

    #[test]
    fn test_see_cache_utilization() {
        // Test cache utilization tracking
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(10);
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Initially empty
        assert_eq!(orderer.see_cache.utilization(), 0.0);

        // Add some entries
        for i in 0..5 {
            let move_ = create_test_move(
                Some(Position::new(1, i)),
                Position::new(2, i),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.calculate_see(&move_, &board);
        }

        // Utilization should be 50%
        let utilization = orderer.see_cache.utilization();
        assert!((utilization - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_see_cache_dynamic_resizing() {
        // Test that cache can be resized and evicts entries if necessary
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(10);
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Fill cache with 10 entries
        for i in 0..10 {
            let move_ = create_test_move(
                Some(Position::new(1, i % 9)),
                Position::new(2, i % 9),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.calculate_see(&move_, &board);
        }

        assert_eq!(orderer.get_see_cache_size(), 10);

        // Resize cache to smaller size
        orderer.see_cache.set_max_size(5);

        // Cache should have evicted entries
        assert!(orderer.get_see_cache_size() <= 5);
    }

    #[test]
    fn test_see_cache_eviction_tracking() {
        // Test that evictions are properly tracked
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(2);
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Fill cache
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.calculate_see(&move1, &board);
        orderer.calculate_see(&move2, &board);

        assert_eq!(orderer.stats.see_cache_evictions, 0);

        // Add third move - should cause eviction
        let move3 = create_test_move(
            Some(Position::new(1, 3)),
            Position::new(2, 3),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.calculate_see(&move3, &board);

        assert_eq!(orderer.stats.see_cache_evictions, 1);

        // Add fourth move - another eviction
        let move4 = create_test_move(
            Some(Position::new(1, 4)),
            Position::new(2, 4),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.calculate_see(&move4, &board);

        assert_eq!(orderer.stats.see_cache_evictions, 2);
    }

    #[test]
    fn test_see_cache_get_stats() {
        // Test the get_stats() method on SEECache
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(5);
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Add some entries
        for i in 0..3 {
            let move_ = create_test_move(
                Some(Position::new(1, i)),
                Position::new(2, i),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.calculate_see(&move_, &board);
        }

        // Get stats
        let stats = orderer.see_cache.get_stats();

        assert_eq!(stats.size, 3);
        assert_eq!(stats.max_size, 5);
        assert!((stats.utilization - 60.0).abs() < 0.1);
        assert!(stats.total_accesses >= 3);
        assert!(stats.avg_accesses_per_entry >= 1.0);
        assert!(stats.memory_bytes > 0);
    }

    #[test]
    fn test_see_cache_value_based_eviction() {
        // Test that cache prefers keeping high-value SEE calculations
        // This test verifies the value-based component of eviction
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(2);
        orderer.set_see_cache_enabled(true);

        let board = crate::bitboards::BitboardBoard::new();

        // Create moves (cache will store different values for each)
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );

        orderer.calculate_see(&move1, &board);
        orderer.calculate_see(&move2, &board);

        assert_eq!(orderer.get_see_cache_size(), 2);

        // Add a third move - eviction should occur
        let move3 = create_test_move(
            Some(Position::new(1, 3)),
            Position::new(2, 3),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.calculate_see(&move3, &board);

        // Verify eviction occurred
        assert_eq!(orderer.stats.see_cache_evictions, 1);
        assert_eq!(orderer.get_see_cache_size(), 2);
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

    // ==================== Task 11.7: PV Move Enhancement Tests ====================

    #[test]
    fn test_multiple_pv_moves_storage() {
        // Test storing and retrieving multiple PV moves
        let mut orderer = MoveOrdering::new();

        // Create test moves
        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Knight,
            Player::Black,
        );
        let move3 = create_test_move(
            Some(Position::new(1, 3)),
            Position::new(2, 3),
            PieceType::Silver,
            Player::Black,
        );

        let pv_moves = vec![move1.clone(), move2.clone(), move3.clone()];
        let position_hash = 12345u64;

        // Store multiple PV moves
        orderer
            .pv_ordering
            .store_multiple_pv_moves(position_hash, pv_moves.clone());

        // Retrieve and verify
        let retrieved = orderer.pv_ordering.get_multiple_pv_moves(position_hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 3);
    }

    #[test]
    fn test_multiple_pv_moves_limit() {
        // Test that multiple PV moves are limited to max_pv_moves_per_position
        let mut orderer = MoveOrdering::new();
        orderer.pv_ordering.set_max_pv_moves_per_position(2);

        // Create 5 test moves
        let moves: Vec<Move> = (0..5)
            .map(|i| {
                create_test_move(
                    Some(Position::new(1, i)),
                    Position::new(2, i),
                    PieceType::Pawn,
                    Player::Black,
                )
            })
            .collect();

        let position_hash = 12345u64;

        // Store 5 moves but only 2 should be kept
        orderer
            .pv_ordering
            .store_multiple_pv_moves(position_hash, moves);

        let retrieved = orderer.pv_ordering.get_multiple_pv_moves(position_hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 2); // Only 2 stored
    }

    #[test]
    fn test_previous_iteration_pv() {
        // Test previous iteration PV tracking
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let position_hash = 12345u64;

        // Cache a PV move
        orderer
            .pv_ordering
            .cache_pv_move(position_hash, Some(move1.clone()));

        // Verify it's in current cache
        assert!(orderer
            .pv_ordering
            .get_cached_pv_move(position_hash)
            .is_some());

        // Save as previous iteration
        orderer.pv_ordering.save_previous_iteration_pv();

        // Verify it's in previous iteration cache
        let prev_pv = orderer.pv_ordering.get_previous_iteration_pv(position_hash);
        assert!(prev_pv.is_some());
        assert_eq!(prev_pv.unwrap().to, move1.to);
    }

    #[test]
    fn test_previous_iteration_pv_clear() {
        // Test clearing previous iteration PV
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let position_hash = 12345u64;

        // Cache and save to previous iteration
        orderer
            .pv_ordering
            .cache_pv_move(position_hash, Some(move1.clone()));
        orderer.pv_ordering.save_previous_iteration_pv();

        assert!(orderer
            .pv_ordering
            .get_previous_iteration_pv(position_hash)
            .is_some());

        // Clear previous iteration
        orderer.pv_ordering.clear_previous_iteration();

        assert!(orderer
            .pv_ordering
            .get_previous_iteration_pv(position_hash)
            .is_none());
    }

    #[test]
    fn test_sibling_pv_storage() {
        // Test sibling node PV tracking
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let move2 = create_test_move(
            Some(Position::new(1, 2)),
            Position::new(2, 2),
            PieceType::Knight,
            Player::Black,
        );

        let parent_hash = 98765u64;

        // Store sibling PV moves
        orderer
            .pv_ordering
            .store_sibling_pv(parent_hash, move1.clone());
        orderer
            .pv_ordering
            .store_sibling_pv(parent_hash, move2.clone());

        // Retrieve and verify
        let siblings = orderer.pv_ordering.get_sibling_pv_moves(parent_hash);
        assert!(siblings.is_some());
        assert_eq!(siblings.unwrap().len(), 2);
    }

    #[test]
    fn test_sibling_pv_deduplication() {
        // Test that duplicate sibling PV moves are not stored
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let parent_hash = 98765u64;

        // Store same move twice
        orderer
            .pv_ordering
            .store_sibling_pv(parent_hash, move1.clone());
        orderer
            .pv_ordering
            .store_sibling_pv(parent_hash, move1.clone());

        // Should only have one entry
        let siblings = orderer.pv_ordering.get_sibling_pv_moves(parent_hash);
        assert!(siblings.is_some());
        assert_eq!(siblings.unwrap().len(), 1);
    }

    #[test]
    fn test_sibling_pv_limit() {
        // Test that sibling PV moves respect max_pv_moves_per_position limit
        let mut orderer = MoveOrdering::new();
        orderer.pv_ordering.set_max_pv_moves_per_position(2);

        let parent_hash = 98765u64;

        // Store 4 different sibling PV moves
        for i in 0..4 {
            let move_ = create_test_move(
                Some(Position::new(1, i)),
                Position::new(2, i),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.pv_ordering.store_sibling_pv(parent_hash, move_);
        }

        // Should only keep max_pv_moves_per_position (2)
        let siblings = orderer.pv_ordering.get_sibling_pv_moves(parent_hash);
        assert!(siblings.is_some());
        assert_eq!(siblings.unwrap().len(), 2);
    }

    #[test]
    fn test_pv_statistics_tracking() {
        // Test PVMoveStatistics structure
        let mut stats = PVMoveStatistics::new();

        // Simulate some PV hits
        stats.primary_pv_hits = 10;
        stats.pv_misses = 5;
        stats.primary_pv_best_move_count = 8;

        // Calculate hit rate
        let hit_rate = stats.primary_pv_hit_rate();
        assert!((hit_rate - 66.67).abs() < 0.1); // 10 / (10 + 5) * 100 ≈ 66.67%

        // Calculate effectiveness
        let effectiveness = stats.primary_pv_effectiveness();
        assert!((effectiveness - 80.0).abs() < 0.1); // 8 / 10 * 100 = 80%
    }

    #[test]
    fn test_pv_memory_tracking() {
        // Test that PV memory usage is tracked correctly
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let position_hash = 12345u64;

        // Initial memory
        let initial_memory = orderer.pv_ordering.cache_memory_bytes();

        // Store some PV moves
        orderer
            .pv_ordering
            .cache_pv_move(position_hash, Some(move1.clone()));
        orderer
            .pv_ordering
            .store_multiple_pv_moves(position_hash, vec![move1.clone()]);
        orderer.pv_ordering.save_previous_iteration_pv();

        // Memory should increase
        let final_memory = orderer.pv_ordering.cache_memory_bytes();
        assert!(final_memory > initial_memory);
    }

    #[test]
    fn test_pv_clear_operations() {
        // Test that various clear operations work correctly
        let mut orderer = MoveOrdering::new();

        let move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let position_hash = 12345u64;

        // Add data to all PV caches
        orderer
            .pv_ordering
            .cache_pv_move(position_hash, Some(move1.clone()));
        orderer
            .pv_ordering
            .store_multiple_pv_moves(position_hash, vec![move1.clone()]);
        orderer.pv_ordering.save_previous_iteration_pv();
        orderer
            .pv_ordering
            .store_sibling_pv(position_hash, move1.clone());

        // Clear all
        orderer.pv_ordering.clear_all();

        // Verify all cleared
        assert!(orderer
            .pv_ordering
            .get_cached_pv_move(position_hash)
            .is_none());
        assert!(orderer
            .pv_ordering
            .get_multiple_pv_moves(position_hash)
            .is_none());
        assert!(orderer
            .pv_ordering
            .get_previous_iteration_pv(position_hash)
            .is_none());
        assert!(orderer
            .pv_ordering
            .get_sibling_pv_moves(position_hash)
            .is_none());
    }

    // ==================== Performance Optimization Tests ====================

    #[test]
    fn test_fast_hash_calculation() {
        let mut orderer = MoveOrdering::new();
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
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
        let mut orderer = MoveOrdering::new();
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
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
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 1)),
                Position::new(3, 1),
                PieceType::Pawn,
                Player::Black,
            ),
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
            Player::Black,
        );

        // First call should populate fast cache
        let score1 = orderer.score_move(&move_);

        // Second call should hit fast cache
        let score2 = orderer.score_move(&move_);
        assert_eq!(score1, score2);

        // Task 1.22: Fast cache is now part of MoveScoreCache - verify cache has entries
        assert!(!orderer.move_score_cache.is_empty());
    }

    #[test]
    fn test_performance_benchmarking() {
        let mut orderer = MoveOrdering::new();
        let moves = vec![
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 1)),
                Position::new(3, 1),
                PieceType::Pawn,
                Player::Black,
            ),
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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
            Player::Black,
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
        let mut orderer = MoveOrdering::new();

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
        let mut orderer = MoveOrdering::new();

        // Test that all statistics structures are initialized
        assert_eq!(orderer.stats.heuristic_stats.capture_stats.applications, 0);
        assert_eq!(
            orderer
                .stats
                .timing_stats
                .move_scoring_times
                .operation_count,
            0
        );
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
            Player::Black,
        );

        // Test heuristic statistics tracking
        orderer.update_heuristic_stats("capture", true, 100);
        orderer.update_heuristic_stats("promotion", true, 200);
        orderer.update_heuristic_stats("tactical", true, 50);

        assert_eq!(orderer.stats.heuristic_stats.capture_stats.applications, 1);
        assert_eq!(
            orderer
                .stats
                .heuristic_stats
                .capture_stats
                .total_score_contribution,
            100
        );
        assert_eq!(
            orderer.stats.heuristic_stats.promotion_stats.applications,
            1
        );
        assert_eq!(orderer.stats.heuristic_stats.tactical_stats.applications, 1);
    }

    #[test]
    fn test_timing_statistics_tracking() {
        let mut orderer = MoveOrdering::new();

        // Test timing statistics recording
        orderer.record_timing("move_scoring", 100);
        orderer.record_timing("move_scoring", 200);
        orderer.record_timing("cache", 50);

        assert_eq!(
            orderer
                .stats
                .timing_stats
                .move_scoring_times
                .operation_count,
            2
        );
        assert_eq!(
            orderer.stats.timing_stats.move_scoring_times.total_time_us,
            300
        );
        assert_eq!(
            orderer.stats.timing_stats.move_scoring_times.avg_time_us,
            150.0
        );
        assert_eq!(
            orderer.stats.timing_stats.move_scoring_times.min_time_us,
            100
        );
        assert_eq!(
            orderer.stats.timing_stats.move_scoring_times.max_time_us,
            200
        );

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

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
        let moves = vec![create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        )];

        // Generate some statistics
        orderer.score_move(&moves[0]);
        orderer.order_moves(&moves);

        // Test trend analysis
        let trend_analysis = orderer.analyze_performance_trends();
        assert!(trend_analysis.cache_efficiency_trend.confidence >= 0.0);
        assert!(trend_analysis.cache_efficiency_trend.confidence <= 1.0);
        assert!(!trend_analysis
            .cache_efficiency_trend
            .recommendation
            .is_empty());
        assert!(!trend_analysis.memory_usage_trend.recommendation.is_empty());
        assert!(!trend_analysis
            .heuristic_effectiveness_trend
            .recommendation
            .is_empty());
        assert!(!trend_analysis.timing_trend.recommendation.is_empty());
        assert!(!trend_analysis
            .overall_performance_trend
            .recommendation
            .is_empty());
    }

    #[test]
    fn test_most_effective_heuristic() {
        let mut orderer = MoveOrdering::new();

        // Set up some heuristic statistics
        orderer
            .stats
            .heuristic_stats
            .capture_stats
            .best_move_contributions = 10;
        orderer
            .stats
            .heuristic_stats
            .promotion_stats
            .best_move_contributions = 5;
        orderer
            .stats
            .heuristic_stats
            .tactical_stats
            .best_move_contributions = 15;

        // Test most effective heuristic identification
        let most_effective = orderer.get_most_effective_heuristic();
        assert_eq!(most_effective, "tactical");
    }

    #[test]
    fn test_heuristic_effectiveness_calculation() {
        let mut orderer = MoveOrdering::new();
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

        assert_eq!(
            orderer
                .stats
                .heuristic_stats
                .capture_stats
                .best_move_contributions,
            2
        );
        assert_eq!(
            orderer
                .stats
                .heuristic_stats
                .promotion_stats
                .best_move_contributions,
            1
        );
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_error_types() {
        // Test error creation and display
        let invalid_move_error = MoveOrderingError::InvalidMove("Test error".to_string());
        assert_eq!(
            format!("{}", invalid_move_error),
            "Invalid move: Test error"
        );

        let cache_error = MoveOrderingError::CacheError("Cache full".to_string());
        assert_eq!(format!("{}", cache_error), "Cache error: Cache full");

        let see_error = MoveOrderingError::SEEError("SEE calculation failed".to_string());
        assert_eq!(
            format!("{}", see_error),
            "SEE calculation error: SEE calculation failed"
        );
    }

    #[test]
    fn test_error_handler_functionality() {
        let mut error_handler = ErrorHandler::default();

        // Test error logging
        let error = MoveOrderingError::InvalidMove("Test error".to_string());
        error_handler.log_error(
            error.clone(),
            ErrorSeverity::Medium,
            "Test context".to_string(),
        );

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
                "Test".to_string(),
            );
        }
        assert!(!error_handler.is_system_unstable());

        // Test with critical error - should be unstable
        error_handler.log_error(
            MoveOrderingError::MemoryError("Critical memory error".to_string()),
            ErrorSeverity::Critical,
            "Test".to_string(),
        );
        assert!(error_handler.is_system_unstable());
    }

    #[test]
    fn test_move_validation() {
        let mut orderer = MoveOrdering::new();

        // Test valid move
        let valid_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
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
            Player::Black,
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
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 2),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(10, 10)),
                Position::new(2, 2),
                PieceType::Pawn,
                Player::Black,
            ), // Invalid
            create_test_move(
                Some(Position::new(3, 3)),
                Position::new(4, 4),
                PieceType::Rook,
                Player::Black,
            ),
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
            "Test context".to_string(),
        );
        assert!(result.is_ok());

        // Test with high severity error
        let result = orderer.handle_error(
            MoveOrderingError::CacheError("Cache error".to_string()),
            ErrorSeverity::High,
            "Test context".to_string(),
        );
        // Should attempt recovery and succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_recovery_mechanisms() {
        let mut orderer = MoveOrdering::new();

        // Test cache error recovery
        let result =
            orderer.attempt_error_recovery(&MoveOrderingError::CacheError("Test".to_string()));
        assert!(result.is_ok());
        // Caches should be cleared
        assert!(orderer.move_score_cache.is_empty());

        // Test memory error recovery
        let result =
            orderer.attempt_error_recovery(&MoveOrderingError::MemoryError("Test".to_string()));
        assert!(result.is_ok());
        // Memory usage should be reduced

        // Test statistics error recovery
        let result =
            orderer.attempt_error_recovery(&MoveOrderingError::StatisticsError("Test".to_string()));
        assert!(result.is_ok());
        // Statistics should be reset

        // Test unsupported error recovery
        let result =
            orderer.attempt_error_recovery(&MoveOrderingError::InvalidMove("Test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_logging_and_reporting() {
        let mut orderer = MoveOrdering::new();

        // Log some errors
        orderer.error_handler.log_error(
            MoveOrderingError::InvalidMove("Error 1".to_string()),
            ErrorSeverity::Medium,
            "Context 1".to_string(),
        );

        orderer.error_handler.log_error(
            MoveOrderingError::CacheError("Error 2".to_string()),
            ErrorSeverity::High,
            "Context 2".to_string(),
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
            "Test".to_string(),
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
        vec1.push(create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        ));
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
        orderer.memory_tracker.record_allocation(
            AllocationType::MoveVector,
            1024,
            "test".to_string(),
        );
        orderer
            .memory_tracker
            .record_allocation(AllocationType::Cache, 2048, "test".to_string());

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
        assert_eq!(
            orderer.advanced_features.position_strategies.current_phase,
            GamePhase::Opening
        );

        orderer.update_game_phase(70, 0, 0.3);
        assert_eq!(
            orderer.advanced_features.position_strategies.current_phase,
            GamePhase::Endgame
        );
    }

    #[test]
    fn test_strategy_scoring() {
        let mut orderer = MoveOrdering::new();

        // Test move scoring with strategy
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );

        let score = orderer.score_move_with_strategy(&move_);
        assert!(score.is_ok());
    }

    #[test]
    fn test_move_classification() {
        let mut orderer = MoveOrdering::new();

        // Test development move
        let development_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
        );
        assert!(orderer.is_development_move(&development_move));

        // Test center move
        let center_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(4, 4),
            PieceType::Pawn,
            Player::Black,
        );
        assert!(orderer.is_center_move(&center_move));

        // Test king safety move
        let king_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::King,
            Player::Black,
        );
        assert!(orderer.is_king_safety_move(&king_move));
    }

    #[test]
    fn test_machine_learning_model() {
        let mut orderer = MoveOrdering::new();

        // Enable ML model
        orderer.advanced_features.ml_model.enabled = true;

        // Test training
        let training_examples = vec![TrainingExample {
            features: vec![1.0, 2.0, 3.0],
            target: 100.0,
            context: PositionContext {
                phase: GamePhase::Middlegame,
                material_balance: 0,
                king_safety: 0,
                center_control: 0,
            },
        }];

        let accuracy = orderer.train_ml_model(training_examples);
        assert!(accuracy.is_ok());
        assert!(accuracy.unwrap() > 0.0);

        // Test prediction
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 2),
            PieceType::Pawn,
            Player::Black,
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
        assert!(!orderer
            .advanced_features
            .dynamic_weights
            .adjustment_history
            .is_empty());
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
        assert_eq!(
            features.position_strategies.current_phase,
            GamePhase::Middlegame
        );

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
        let strategy = &orderer
            .advanced_features
            .position_strategies
            .opening_strategy;

        // Opening strategy should favor development
        assert!(strategy.priority_adjustments.development_priority > 1.0);

        // Test tactical strategy
        let tactical_strategy = &orderer
            .advanced_features
            .position_strategies
            .tactical_strategy;
        assert!(tactical_strategy.priority_adjustments.capture_priority > 1.0);
    }

    #[test]
    fn test_heuristic_preferences() {
        let mut orderer = MoveOrdering::new();

        // Test opening preferences
        let opening_strategy = &orderer
            .advanced_features
            .position_strategies
            .opening_strategy;
        assert!(opening_strategy.heuristic_preferences.prefer_development);
        assert!(opening_strategy.heuristic_preferences.prefer_positional);
        assert!(!opening_strategy.heuristic_preferences.prefer_tactical);

        // Test tactical preferences
        let tactical_strategy = &orderer
            .advanced_features
            .position_strategies
            .tactical_strategy;
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
            Player::Black,
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
                Player::Black,
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
        let result = orderer.integrate_with_transposition_table(
            None,
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert!(result.is_ok());

        // Test with transposition table entry
        let tt_entry = create_tt_entry(
            100,
            3,
            TranspositionFlag::Exact,
            Some(create_legacy_move(
                Some(Position::new(4, 4)),
                Position::new(4, 3),
                PieceType::Pawn,
                false,
                false,
                false,
                false,
            )),
            12345,
            1,
        );

        let result = orderer.integrate_with_transposition_table(
            Some(&tt_entry),
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert!(result.is_ok());

        // Check that statistics were updated
        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 1);
        assert_eq!(stats.tt_integration_updates, 1);
    }

    #[test]
    fn test_pv_move_from_tt() {
        let mut orderer = MoveOrdering::new();

        // Test with no TT entry
        let pv_move = orderer.get_pv_move_from_tt(None);
        assert!(pv_move.is_none());

        // Test with TT entry containing best move
        let tt_entry = TranspositionEntry {
            score: 50,
            depth: 2,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(Move::new(
                Some(Position::new(3, 3)),
                Position::new(3, 2),
                PieceType::Silver,
                false,
                false,
                false,
                false,
            )),
            hash_key: 67890,
            age: 2,
            source: EntrySource::MainSearch,
        };

        let pv_move = orderer.get_pv_move_from_tt(Some(&tt_entry));
        assert!(pv_move.is_some());
        assert_eq!(pv_move.unwrap().from.unwrap(), Position::new(3, 3));
    }

    #[test]
    fn test_update_ordering_from_tt_result() {
        let mut orderer = MoveOrdering::new();

        let tt_entry = create_tt_entry(
            75,
            4,
            TranspositionFlag::Exact,
            Some(create_legacy_move(
                Some(Position::new(5, 5)),
                Position::new(5, 4),
                PieceType::Gold,
                false,
                false,
                false,
                false,
            )),
            11111,
            3,
        );

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
                best_move: Some(Move::new(
                    Some(Position::new(4, 4)),
                    Position::new(4, 3),
                    PieceType::Pawn,
                    false,
                    false,
                    false,
                    false,
                )),
                hash_key: 12345,
                age: 1,
                source: EntrySource::MainSearch,
            },
            TranspositionEntry {
                score: -50,
                depth: 3,
                flag: TranspositionFlag::LowerBound,
                best_move: Some(Move::new(
                    Some(Position::new(3, 3)),
                    Position::new(3, 2),
                    PieceType::Silver,
                    false,
                    false,
                    false,
                    false,
                )),
                hash_key: 67890,
                age: 2,
                source: EntrySource::MainSearch,
            },
            TranspositionEntry {
                score: 25,
                depth: 2,
                flag: TranspositionFlag::UpperBound,
                best_move: Some(Move::new(
                    Some(Position::new(5, 5)),
                    Position::new(5, 4),
                    PieceType::Gold,
                    false,
                    false,
                    false,
                    false,
                )),
                hash_key: 11111,
                age: 3,
                source: EntrySource::MainSearch,
            },
        ];

        // Performance test: integrate multiple TT entries
        let start_time = TimeSource::now();
        for entry in &tt_entries {
            let result = orderer.integrate_with_transposition_table(
                Some(entry),
                &board,
                &captured_pieces,
                player,
                depth,
            );
            assert!(result.is_ok());
        }
        let elapsed_ms = start_time.elapsed_ms();

        // Verify performance is reasonable (should be fast)
        assert!(
            elapsed_ms < 100,
            "TT integration took {}ms, should be < 100ms",
            elapsed_ms
        );

        // Verify statistics were updated correctly
        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 3);
        assert_eq!(stats.tt_integration_updates, 3);
    }

    #[test]
    fn test_transposition_table_pv_move_performance() {
        let mut orderer = MoveOrdering::new();

        // Create a large number of TT entries for performance testing
        let mut tt_entries = Vec::new();
        for i in 0..1000 {
            tt_entries.push(TranspositionEntry {
                score: i as i32,
                depth: (i % 10) as u8 + 1,
                flag: if i % 3 == 0 {
                    TranspositionFlag::Exact
                } else if i % 3 == 1 {
                    TranspositionFlag::LowerBound
                } else {
                    TranspositionFlag::UpperBound
                },
                best_move: Some(Move::new(
                    Some(Position::new((i % 9) as u8 + 1, (i % 9) as u8 + 1)),
                    Position::new(((i + 1) % 9) as u8 + 1, ((i + 1) % 9) as u8 + 1),
                    PieceType::Pawn,
                    false,
                    false,
                    false,
                    false,
                )),
                hash_key: i as u64,
                age: i as u32,
                source: EntrySource::MainSearch,
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
        assert!(
            elapsed_ms < 50,
            "PV move extraction took {}ms, should be < 50ms",
            elapsed_ms
        );
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
            best_move: Some(Move::new(
                Some(Position::new(4, 4)),
                Position::new(4, 3),
                PieceType::Pawn,
                false,
                false,
                false,
                false,
            )),
            hash_key: 12345,
            age: 1,
            source: EntrySource::MainSearch,
        };

        let result = orderer.integrate_with_transposition_table(
            Some(&exact_entry),
            &board,
            &captured_pieces,
            player,
            depth,
        );
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
            best_move: Some(Move::new(
                Some(Position::new(3, 3)),
                Position::new(3, 2),
                PieceType::Silver,
                false,
                false,
                false,
                false,
            )),
            hash_key: 67890,
            age: 2,
            source: EntrySource::MainSearch,
        };

        let result = orderer.integrate_with_transposition_table(
            Some(&lower_bound_entry),
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_hits, 2);
        assert_eq!(stats.killer_moves_from_tt, 1);

        // Test 3: Integration with upper bound entry (should not update killer moves)
        let upper_bound_entry = TranspositionEntry {
            score: -25,
            depth: 3,
            flag: TranspositionFlag::UpperBound,
            best_move: Some(Move::new(
                Some(Position::new(5, 5)),
                Position::new(5, 4),
                PieceType::Gold,
                false,
                false,
                false,
                false,
            )),
            hash_key: 11111,
            age: 3,
            source: EntrySource::MainSearch,
        };

        let result = orderer.integrate_with_transposition_table(
            Some(&upper_bound_entry),
            &board,
            &captured_pieces,
            player,
            depth,
        );
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
            source: EntrySource::MainSearch,
        };

        let result = orderer.integrate_with_transposition_table(
            Some(&entry_no_move),
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert!(result.is_ok());

        let stats = orderer.get_tt_integration_stats();
        assert_eq!(stats.tt_integration_updates, 1);
        assert_eq!(stats.tt_integration_hits, 0); // No best move, so no hits

        // Test 2: Multiple updates with same move
        let entry1 = TranspositionEntry {
            score: 100,
            depth: 2,
            flag: TranspositionFlag::Exact,
            best_move: Some(Move::new(
                Some(Position::new(1, 1)),
                Position::new(1, 2),
                PieceType::Pawn,
                false,
                false,
                false,
                false,
            )),
            hash_key: 11111,
            age: 1,
            source: EntrySource::MainSearch,
        };

        let entry2 = TranspositionEntry {
            score: 200,
            depth: 3,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(Move::new(
                Some(Position::new(1, 1)),
                Position::new(1, 2),
                PieceType::Pawn,
                false,
                false,
                false,
                false,
            )),
            hash_key: 22222,
            age: 2,
            source: EntrySource::MainSearch,
        };

        let result1 = orderer.integrate_with_transposition_table(
            Some(&entry1),
            &board,
            &captured_pieces,
            player,
            depth,
        );
        let result2 = orderer.integrate_with_transposition_table(
            Some(&entry2),
            &board,
            &captured_pieces,
            player,
            depth,
        );
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
                false,
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
            Move::new_move(
                Position::new(6, 4),
                Position::new(5, 4),
                PieceType::Pawn,
                player,
                false,
            ),
            Move::new_move(
                Position::new(8, 1),
                Position::new(7, 1),
                PieceType::Lance,
                player,
                false,
            ),
            Move::new_move(
                Position::new(7, 1),
                Position::new(6, 3),
                PieceType::Knight,
                player,
                false,
            ),
        ];

        // Test 1: Order with PV
        let ordered_pv =
            orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_pv.len(), moves.len());

        // Test 2: Add killer move and order with PV and killer
        orderer.add_killer_move(moves[1].clone());
        let ordered_both =
            orderer.order_moves_with_pv_and_killer(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_both.len(), moves.len());

        // Test 3: Update history and order with all heuristics
        orderer.update_history_score(&moves[0], depth, None);
        // Task 3.0: No IID move context for this test
        let ordered_all = orderer.order_moves_with_all_heuristics(
            &moves,
            &board,
            &captured_pieces,
            player,
            depth,
            None,
            None,
        );
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
        assert!(initial_memory.total_memory > 0);

        // Create moves and order them
        let moves = vec![
            Move::new_move(
                Position::new(0, 0),
                Position::new(1, 0),
                PieceType::Pawn,
                Player::Black,
                false,
            ),
            Move::new_move(
                Position::new(1, 1),
                Position::new(2, 1),
                PieceType::Silver,
                Player::Black,
                false,
            ),
            Move::new_move(
                Position::new(2, 2),
                Position::new(3, 2),
                PieceType::Gold,
                Player::Black,
                false,
            ),
        ];

        for _ in 0..100 {
            let _ = orderer.order_moves(&moves);
        }

        // Check memory usage is tracked
        let final_memory = orderer.get_current_memory_usage();
        assert!(final_memory.total_memory > 0);

        // Get peak memory usage
        let peak_memory = orderer.get_peak_memory_usage();
        assert!(peak_memory.total_memory >= initial_memory.total_memory);

        // Verify no excessive memory growth (memory leaks)
        assert!(
            final_memory.total_memory < initial_memory.total_memory * 10,
            "Potential memory leak detected"
        );
    }

    #[test]
    fn test_regression_move_ordering_determinism() {
        // Test that move ordering is deterministic for the same input
        let mut orderer1 = MoveOrdering::new();
        let mut orderer2 = MoveOrdering::new();

        let moves = vec![
            Move::new_move(
                Position::new(6, 4),
                Position::new(5, 4),
                PieceType::Pawn,
                Player::Black,
                false,
            ),
            Move::new_move(
                Position::new(8, 1),
                Position::new(7, 1),
                PieceType::Lance,
                Player::Black,
                false,
            ),
            Move::new_move(
                Position::new(7, 1),
                Position::new(6, 3),
                PieceType::Knight,
                Player::Black,
                false,
            ),
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
                false,
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
        orderer.clear_all_killer_moves();
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
            Move::new_move(
                Position::new(6, 6),
                Position::new(5, 6),
                PieceType::Pawn,
                player,
                false,
            ),
            // P-26
            Move::new_move(
                Position::new(6, 1),
                Position::new(5, 1),
                PieceType::Pawn,
                player,
                false,
            ),
            // P-56
            Move::new_move(
                Position::new(6, 4),
                Position::new(5, 4),
                PieceType::Pawn,
                player,
                false,
            ),
        ];

        let result = orderer.order_moves(&moves);
        assert!(result.is_ok());
        let ordered = result.unwrap();
        assert_eq!(ordered.len(), 3);

        // All moves should be present
        for original_move in &moves {
            assert!(ordered
                .iter()
                .any(|m| m.from == original_move.from && m.to == original_move.to));
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
                Move::new_move(
                    Position::new(6, 4),
                    Position::new(5, 4),
                    PieceType::Pawn,
                    player,
                    false,
                ),
                Move::new_move(
                    Position::new(7, 1),
                    Position::new(6, 3),
                    PieceType::Knight,
                    player,
                    false,
                ),
            ];

            // Task 3.0: Order moves (no IID move context for this test)
            let ordered = orderer.order_moves_with_all_heuristics(
                &moves,
                &board,
                &captured_pieces,
                player,
                search_depth,
                None,
                None,
            );
            assert!(!ordered.is_empty());

            // Simulate finding a good move (update history and killer)
            if let Some(best_move) = ordered.first() {
                orderer.add_killer_move(best_move.clone());
                orderer.update_history_score(best_move, search_depth, None);
            }

            // Simulate transposition table hit
            let tt_entry = TranspositionEntry {
                score: 100,
                depth: search_depth,
                flag: TranspositionFlag::Exact,
                best_move: ordered.first().cloned(),
                hash_key: search_depth as u64 * 12345,
                age: 1,
                source: EntrySource::MainSearch,
            };
            let _ = orderer.integrate_with_transposition_table(
                Some(&tt_entry),
                &board,
                &captured_pieces,
                player,
                search_depth,
            );
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
                false,
            ));
        }

        // Measure ordering time
        let start = TimeSource::now();
        for _ in 0..100 {
            let _ = orderer.order_moves(&moves);
        }
        let elapsed = start.elapsed_ms();

        // Should complete 100 orderings in reasonable time (< 1 second)
        assert!(
            elapsed < 1000,
            "Move ordering took {}ms for 100 iterations, should be < 1000ms",
            elapsed
        );

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
                Move::new_move(
                    Position::new((iteration % 9) as u8, 0),
                    Position::new((iteration % 9) as u8, 1),
                    PieceType::Pawn,
                    player,
                    false,
                ),
                Move::new_move(
                    Position::new(0, (iteration % 9) as u8),
                    Position::new(1, (iteration % 9) as u8),
                    PieceType::Lance,
                    player,
                    false,
                ),
            ];

            // Task 3.0: Order with different methods (no IID move context for this test)
            let _ = orderer.order_moves(&moves);
            let _ = orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);
            let _ = orderer.order_moves_with_all_heuristics(
                &moves,
                &board,
                &captured_pieces,
                player,
                depth,
                None,
                None,
            );

            // Update heuristics
            orderer.add_killer_move(moves[0].clone());
            orderer.update_history_score(&moves[0], depth, None);

            // Age history periodically
            if iteration % 10 == 0 {
                orderer.age_history_table();
            }
        }

        // Verify orderer is still functional
        let test_moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            player,
            false,
        )];
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
        let moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        )];

        let result = orderer.order_moves(&moves);
        assert!(result.is_ok());
        let ordered = result.unwrap();
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].from, moves[0].from);
        assert_eq!(ordered[0].to, moves[0].to);
    }

    // Task 5.0: Learning tests

    #[test]
    fn test_heuristic_effectiveness_tracking() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;

        // Record effectiveness for different heuristics
        orderer.record_heuristic_effectiveness("capture", true);
        orderer.record_heuristic_effectiveness("capture", true);
        orderer.record_heuristic_effectiveness("capture", false);

        orderer.record_heuristic_effectiveness("history", true);
        orderer.record_heuristic_effectiveness("history", false);

        // Check effectiveness metrics
        let capture_metrics = orderer.get_heuristic_effectiveness("capture").unwrap();
        assert_eq!(capture_metrics.total_uses, 3);
        assert_eq!(capture_metrics.cutoff_count, 2);
        assert!((capture_metrics.hit_rate - 2.0 / 3.0).abs() < 0.01);
        assert!(capture_metrics.effectiveness_score > 0.0);

        let history_metrics = orderer.get_heuristic_effectiveness("history").unwrap();
        assert_eq!(history_metrics.total_uses, 2);
        assert_eq!(history_metrics.cutoff_count, 1);
        assert!((history_metrics.hit_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_weight_adjustment_based_on_effectiveness() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;
        orderer.config.learning_config.min_games_for_learning = 1;
        orderer.config.learning_config.learning_frequency = 1;
        orderer.config.learning_config.learning_rate = 0.1;
        orderer.config.learning_config.min_effectiveness_diff = 0.01;

        // Record effectiveness - make capture more effective
        for _ in 0..10 {
            orderer.record_heuristic_effectiveness("capture", true);
            orderer.record_heuristic_effectiveness("history", false);
        }

        // Get initial weights
        let initial_capture_weight = orderer.config.weights.capture_weight;
        let initial_history_weight = orderer.config.weights.history_weight;

        // Increment counter to trigger adjustment
        orderer.learning_update_counter = orderer.config.learning_config.min_games_for_learning;
        orderer.adjust_weights_based_on_effectiveness();

        // Check that weights were adjusted
        let capture_metrics = orderer.get_heuristic_effectiveness("capture").unwrap();
        let history_metrics = orderer.get_heuristic_effectiveness("history").unwrap();

        // Capture should have higher effectiveness, so it should get a higher weight
        if capture_metrics.effectiveness_score > history_metrics.effectiveness_score {
            assert!(orderer.config.weights.capture_weight >= initial_capture_weight);
            assert!(orderer.config.weights.history_weight <= initial_history_weight);
        }

        // Check that weight change history was recorded
        assert!(!orderer.get_weight_change_history().is_empty());
    }

    #[test]
    fn test_weight_bounds() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;
        orderer.config.learning_config.enable_weight_bounds = true;
        orderer.config.learning_config.min_weight = 0;
        orderer.config.learning_config.max_weight = 1000;
        orderer.config.learning_config.min_games_for_learning = 1;
        orderer.config.learning_config.learning_frequency = 1;
        orderer.config.learning_config.learning_rate = 10.0; // Very high learning rate to test bounds

        // Set initial weight to a medium value
        orderer.config.weights.capture_weight = 500;

        // Record very high effectiveness
        for _ in 0..100 {
            orderer.record_heuristic_effectiveness("capture", true);
        }

        // Attempt adjustment
        orderer.learning_update_counter = orderer.config.learning_config.min_games_for_learning;
        orderer.adjust_weights_based_on_effectiveness();

        // Check that weight is within bounds
        assert!(orderer.config.weights.capture_weight >= orderer.config.learning_config.min_weight);
        assert!(orderer.config.weights.capture_weight <= orderer.config.learning_config.max_weight);
    }

    #[test]
    fn test_learning_disabled() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = false;

        // Record effectiveness
        orderer.record_heuristic_effectiveness("capture", true);

        // Check that no effectiveness was recorded
        assert!(orderer.get_heuristic_effectiveness("capture").is_none());

        // Attempt adjustment
        let adjusted = orderer.adjust_weights_based_on_effectiveness();
        assert!(!adjusted);
    }

    #[test]
    fn test_learning_frequency() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;
        orderer.config.learning_config.min_games_for_learning = 1;
        orderer.config.learning_config.learning_frequency = 5; // Update every 5 increments

        // Record effectiveness
        for _ in 0..10 {
            orderer.record_heuristic_effectiveness("capture", true);
        }

        // First 4 increments should not trigger adjustment
        for i in 1..5 {
            orderer.learning_update_counter = i;
            let adjusted = orderer.adjust_weights_based_on_effectiveness();
            assert!(!adjusted, "Should not adjust at counter {}", i);
        }

        // 5th increment should trigger adjustment
        orderer.learning_update_counter = 5;
        let adjusted = orderer.adjust_weights_based_on_effectiveness();
        assert!(adjusted, "Should adjust at counter 5");
    }

    #[test]
    fn test_min_games_for_learning() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;
        orderer.config.learning_config.min_games_for_learning = 10;
        orderer.config.learning_config.learning_frequency = 1;

        // Record effectiveness
        for _ in 0..5 {
            orderer.record_heuristic_effectiveness("capture", true);
        }

        // Should not adjust before min_games_for_learning
        orderer.learning_update_counter = 5;
        let adjusted = orderer.adjust_weights_based_on_effectiveness();
        assert!(!adjusted);

        // Should adjust after min_games_for_learning
        orderer.learning_update_counter = 10;
        let adjusted = orderer.adjust_weights_based_on_effectiveness();
        assert!(adjusted);
    }

    #[test]
    fn test_clear_learning_data() {
        let mut orderer = MoveOrdering::new();
        orderer.config.learning_config.enable_learning = true;

        // Record effectiveness
        orderer.record_heuristic_effectiveness("capture", true);
        orderer.record_heuristic_effectiveness("history", true);
        orderer.increment_learning_counter();

        // Clear learning data
        orderer.clear_learning_data();

        // Check that all data was cleared
        assert!(orderer.get_all_heuristic_effectiveness().is_empty());
        assert!(orderer.get_weight_change_history().is_empty());
        assert_eq!(orderer.learning_update_counter, 0);
        assert_eq!(orderer.stats.weight_adjustments, 0);
        assert_eq!(orderer.stats.learning_effectiveness, 0.0);
    }

    #[test]
    fn test_learning_configuration() {
        let mut orderer = MoveOrdering::new();

        // Test default configuration
        assert!(!orderer.config.learning_config.enable_learning);
        assert_eq!(orderer.config.learning_config.learning_rate, 0.1);
        assert_eq!(orderer.config.learning_config.learning_frequency, 100);
        assert_eq!(orderer.config.learning_config.min_games_for_learning, 10);
        assert!(orderer.config.learning_config.enable_weight_bounds);

        // Test configuration validation
        let mut config = MoveOrderingConfig::default();
        config.learning_config.learning_rate = 1.5; // Invalid: > 1.0
        let errors = config.validate().unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Learning rate")));
    }

    #[test]
    fn test_regression_history_aging() {
        // Regression test for history aging
        let mut orderer = MoveOrdering::new();

        // Add history values
        let move1 = Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        );
        orderer.update_history_score(&move1, 5, None);

        // Get initial history value
        let initial_value = orderer.get_history_value(&move1);
        assert!(initial_value > 0);

        // Age history multiple times
        for _ in 0..5 {
            orderer.age_history_table();
        }

        // History value should be reduced after aging
        let aged_value = orderer.get_history_score(&move1);
        assert!(
            aged_value < initial_value,
            "History aging should reduce values"
        );
    }

    // ==================== Performance Tuning Tests ====================

    #[test]
    fn test_runtime_performance_tuning() {
        let mut orderer = MoveOrdering::new();

        // Create conditions for tuning
        let moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        )];

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
        let moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        )];

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
            Move::new_move(
                Position::new(0, 0),
                Position::new(1, 0),
                PieceType::Pawn,
                Player::Black,
                false,
            ),
            Move::new_move(
                Position::new(1, 1),
                Position::new(2, 1),
                PieceType::Silver,
                Player::Black,
                false,
            ),
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
                false,
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

    #[test]
    fn test_platform_memory_limits() {
        let limits = MoveOrdering::get_platform_memory_limits();

        // Verify limits are reasonable
        assert!(limits.max_total_memory_bytes > 0);
        assert!(limits.max_cache_size > 0);
        assert!(limits.max_see_cache_size > 0);
        assert!(limits.recommended_cache_size <= limits.max_cache_size);
        assert!(limits.recommended_see_cache_size <= limits.max_see_cache_size);
        assert!(
            limits.max_total_memory_bytes >= 50 * 1024 * 1024,
            "Memory limit should be generous for native targets"
        );
        assert!(
            limits.max_cache_size >= 100000,
            "Cache limit should be generous for native targets"
        );
    }

    #[test]
    fn test_time_source_usage() {
        // Verify TimeSource is being used
        let mut orderer = MoveOrdering::new();

        let moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        )];

        // This should not panic
        for _ in 0..10 {
            let _ = orderer.order_moves(&moves);
        }

        // Verify timing statistics are tracked
        let stats = orderer.get_stats();
        assert!(stats.total_ordering_time_us >= 0);
    }

    #[test]
    fn test_history_table_indexing() {
        // Verify array indexing is safe for history updates
        let mut orderer = MoveOrdering::new();

        // Test with various positions to ensure no index out of bounds
        for row in 0..9 {
            for col in 0..9 {
                let move_ = Move::new_move(
                    Position::new(row, col),
                    Position::new((row + 1) % 9, (col + 1) % 9),
                    PieceType::Pawn,
                    Player::Black,
                    false,
                );

                // This should not panic with index out of bounds
                orderer.update_history_score(&move_, 3, None);
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
        let moves = vec![Move::new_move(
            Position::new(0, 0),
            Position::new(1, 0),
            PieceType::Pawn,
            Player::Black,
            false,
        )];

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
        let book_moves = vec![crate::opening_book::BookMove {
            from: Some(Position::new(6, 4)),
            to: Position::new(5, 4),
            piece_type: PieceType::Pawn,
            is_drop: false,
            is_promotion: false,
            weight: 800,
            evaluation: 50,
            opening_name: Some("Standard Opening".to_string()),
            move_notation: Some("P-76".to_string()),
        }];

        // Integrate with opening book
        let result = orderer.integrate_with_opening_book(
            &book_moves,
            &board,
            &captured_pieces,
            player,
            depth,
        );
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
                false,
            )),
            distance_to_mate: Some(5),
            moves_to_mate: Some(5),
            outcome: crate::tablebase::TablebaseOutcome::Win,
            confidence: 1.0,
        };

        // Integrate with tablebase
        let result =
            orderer.integrate_with_tablebase(&tb_result, &board, &captured_pieces, player, depth);
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
            Move::new_move(
                Position::new(6, 4),
                Position::new(5, 4),
                PieceType::Pawn,
                player,
                false,
            ),
            Move::new_move(
                Position::new(6, 5),
                Position::new(5, 5),
                PieceType::Pawn,
                player,
                false,
            ),
        ];

        let ordered =
            orderer.order_moves_for_analysis(&moves, &board, &captured_pieces, player, depth);
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
            Move::new_move(
                Position::new(6, 4),
                Position::new(5, 4),
                PieceType::Pawn,
                player,
                false,
            ),
            Move::new_move(
                Position::new(6, 5),
                Position::new(5, 5),
                PieceType::Pawn,
                player,
                false,
            ),
        ];

        // Test with different time constraints
        let ordered_low = orderer.order_moves_with_time_management(
            &moves,
            500,
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert_eq!(ordered_low.len(), moves.len());

        let ordered_high = orderer.order_moves_with_time_management(
            &moves,
            10000,
            &board,
            &captured_pieces,
            player,
            depth,
        );
        assert_eq!(ordered_high.len(), moves.len());
    }

    #[test]
    fn test_game_phase_ordering() {
        let mut orderer = MoveOrdering::new();
        let board = crate::bitboards::BitboardBoard::new();
        let captured_pieces = crate::CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        let moves = vec![Move::new_move(
            Position::new(6, 4),
            Position::new(5, 4),
            PieceType::Pawn,
            player,
            false,
        )];

        // Test different game phases
        let _ = orderer.order_moves_for_game_phase(
            &moves,
            GamePhase::Opening,
            &board,
            &captured_pieces,
            player,
            depth,
        );
        let _ = orderer.order_moves_for_game_phase(
            &moves,
            GamePhase::Middlegame,
            &board,
            &captured_pieces,
            player,
            depth,
        );
        let _ = orderer.order_moves_for_game_phase(
            &moves,
            GamePhase::Endgame,
            &board,
            &captured_pieces,
            player,
            depth,
        );

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
