//! Statistics tracking for move ordering
//!
//! This module contains structures and methods for tracking performance
//! metrics and statistics for the move ordering system.

// Statistics structures - no external dependencies needed

// Task 1.22: Import types needed for PerformanceStats and StatisticsExport
// Note: CacheSizes, HotPathStats, and OrderingStats are already in this module
use super::{MemoryUsage, MoveOrderingConfig};

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
    /// Number of counter-move hits
    pub counter_move_hits: u64,
    /// Number of counter-move misses
    pub counter_move_misses: u64,
    /// Counter-move hit rate percentage
    pub counter_move_hit_rate: f64,
    /// Number of counter-moves stored
    pub counter_moves_stored: u64,
    /// Number of cache evictions (Task 3.0)
    pub cache_evictions: u64,
    /// Number of cache evictions due to size limit (Task 3.0)
    pub cache_evictions_size_limit: u64,
    /// Number of cache evictions due to policy (Task 3.0)
    pub cache_evictions_policy: u64,
    /// Cache hit rate by entry age (Task 3.0)
    pub cache_hit_rate_by_age: f64,
    /// Cache hit rate by entry depth (Task 3.0)
    pub cache_hit_rate_by_depth: f64,
    /// Number of weight adjustments made (Task 5.0)
    pub weight_adjustments: u64,
    /// Learning effectiveness improvement (Task 5.0)
    pub learning_effectiveness: f64,
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
    /// Number of SEE cache evictions (Task 7.0)
    pub see_cache_evictions: u64,
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
/// Task 10.0: Enhanced with per-move-type tracking
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
    /// Task 10.0: Per-move-type hit rates
    pub capture_hit_rate: f64,
    /// Task 10.0: Promotion move hit rate
    pub promotion_hit_rate: f64,
    /// Task 10.0: Quiet move hit rate
    pub quiet_hit_rate: f64,
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

// ==================== Transposition Table Integration Statistics ====================

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

// ==================== Task 10.0: Enhanced Statistics ====================

/// Move type distribution statistics
/// Task 10.0: Tracks distribution of different move types and their ordering effectiveness
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MoveTypeDistribution {
    /// Total number of capture moves ordered
    pub captures: u64,
    /// Total number of promotion moves ordered
    pub promotions: u64,
    /// Total number of quiet moves ordered
    pub quiet_moves: u64,
    /// Total number of check moves ordered
    pub check_moves: u64,
    /// Total number of drop moves ordered
    pub drop_moves: u64,
    /// Capture move ordering effectiveness (best move percentage)
    pub capture_effectiveness: f64,
    /// Promotion move ordering effectiveness
    pub promotion_effectiveness: f64,
    /// Quiet move ordering effectiveness
    pub quiet_effectiveness: f64,
    /// Check move ordering effectiveness
    pub check_effectiveness: f64,
}

/// Depth-specific statistics
/// Task 10.0: Tracks statistics at different search depths
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DepthSpecificStats {
    /// Statistics by depth (0-20)
    /// Each entry contains: (moves_ordered, cache_hits, cache_misses, best_move_index_avg)
    pub stats_by_depth: Vec<DepthStats>,
}

/// Statistics for a specific depth
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DepthStats {
    /// Search depth
    pub depth: u8,
    /// Number of moves ordered at this depth
    pub moves_ordered: u64,
    /// Cache hits at this depth
    pub cache_hits: u64,
    /// Cache misses at this depth
    pub cache_misses: u64,
    /// Average index of best move (lower is better)
    pub best_move_index_avg: f64,
    /// PV move hit rate at this depth
    pub pv_hit_rate: f64,
    /// Killer move hit rate at this depth
    pub killer_hit_rate: f64,
    /// History hit rate at this depth
    pub history_hit_rate: f64,
}

/// Game phase-specific statistics
/// Task 10.0: Tracks statistics by game phase (opening, middlegame, endgame)
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct GamePhaseStats {
    /// Opening phase statistics
    pub opening: PhaseStats,
    /// Middlegame phase statistics
    pub middlegame: PhaseStats,
    /// Endgame phase statistics
    pub endgame: PhaseStats,
}

/// Statistics for a specific game phase
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PhaseStats {
    /// Number of moves ordered in this phase
    pub moves_ordered: u64,
    /// Average ordering time in this phase (microseconds)
    pub avg_ordering_time_us: f64,
    /// Cache hit rate in this phase
    pub cache_hit_rate: f64,
    /// PV move hit rate in this phase
    pub pv_hit_rate: f64,
    /// Killer move hit rate in this phase
    pub killer_hit_rate: f64,
    /// History hit rate in this phase
    pub history_hit_rate: f64,
    /// Average best move index in this phase
    pub best_move_index_avg: f64,
    /// Heuristic effectiveness scores by phase
    pub heuristic_effectiveness: Vec<(String, f64)>,
}

/// Enhanced statistics tracker
/// Task 10.0: Comprehensive statistics tracking with aggregation
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EnhancedStatistics {
    /// Move type distribution
    pub move_type_distribution: MoveTypeDistribution,
    /// Depth-specific statistics
    pub depth_specific_stats: DepthSpecificStats,
    /// Game phase-specific statistics
    pub game_phase_stats: GamePhaseStats,
    /// Overall ordering effectiveness score (0-100)
    pub overall_effectiveness: f64,
}

impl EnhancedStatistics {
    /// Create new enhanced statistics tracker
    pub fn new() -> Self {
        Self {
            move_type_distribution: MoveTypeDistribution::default(),
            depth_specific_stats: DepthSpecificStats {
                stats_by_depth: (0..21)
                    .map(|d| DepthStats {
                        depth: d,
                        ..Default::default()
                    })
                    .collect(),
            },
            game_phase_stats: GamePhaseStats::default(),
            overall_effectiveness: 0.0,
        }
    }

    /// Record a move ordering operation
    /// Task 10.0: Updates appropriate statistics based on move type, depth, and phase
    pub fn record_ordering(
        &mut self,
        moves: &[crate::types::Move],
        depth: u8,
        phase: crate::types::GamePhase,
        best_move_index: Option<usize>,
        ordering_time_us: u64,
        cache_hit: bool,
    ) {
        // Update move type distribution
        for move_ in moves {
            if move_.is_capture {
                self.move_type_distribution.captures += 1;
            }
            if move_.is_promotion {
                self.move_type_distribution.promotions += 1;
            }
            if !move_.is_capture && !move_.is_promotion && !move_.gives_check {
                self.move_type_distribution.quiet_moves += 1;
            }
            if move_.gives_check {
                self.move_type_distribution.check_moves += 1;
            }
            if move_.is_drop() {
                self.move_type_distribution.drop_moves += 1;
            }
        }

        // Update depth-specific statistics
        if depth < 21 {
            let depth_stats = &mut self.depth_specific_stats.stats_by_depth[depth as usize];
            depth_stats.moves_ordered += moves.len() as u64;
            if cache_hit {
                depth_stats.cache_hits += 1;
            } else {
                depth_stats.cache_misses += 1;
            }
            if let Some(index) = best_move_index {
                // Update running average of best move index
                let total = depth_stats.moves_ordered;
                depth_stats.best_move_index_avg =
                    (depth_stats.best_move_index_avg * (total - 1) as f64 + index as f64)
                        / total as f64;
            }
        }

        // Update game phase-specific statistics
        let phase_stats = match phase {
            crate::types::GamePhase::Opening => &mut self.game_phase_stats.opening,
            crate::types::GamePhase::Middlegame => &mut self.game_phase_stats.middlegame,
            crate::types::GamePhase::Endgame => &mut self.game_phase_stats.endgame,
        };

        phase_stats.moves_ordered += moves.len() as u64;
        let total = phase_stats.moves_ordered;
        phase_stats.avg_ordering_time_us = (phase_stats.avg_ordering_time_us * (total - 1) as f64
            + ordering_time_us as f64)
            / total as f64;

        if let Some(index) = best_move_index {
            phase_stats.best_move_index_avg =
                (phase_stats.best_move_index_avg * (total - 1) as f64 + index as f64)
                    / total as f64;
        }
    }

    /// Calculate overall effectiveness score
    /// Task 10.0: Combines multiple effectiveness metrics into single score
    pub fn calculate_overall_effectiveness(&mut self) {
        // Calculate based on:
        // - Average best move index across all depths (lower is better)
        // - Cache hit rates
        // - Heuristic effectiveness

        let avg_best_move_index: f64 = self
            .depth_specific_stats
            .stats_by_depth
            .iter()
            .filter(|s| s.moves_ordered > 0)
            .map(|s| s.best_move_index_avg)
            .sum::<f64>()
            / self
                .depth_specific_stats
                .stats_by_depth
                .iter()
                .filter(|s| s.moves_ordered > 0)
                .count()
                .max(1) as f64;

        // Normalize: index 0 = 100%, index 10+ = 0%
        let index_score = ((10.0 - avg_best_move_index.min(10.0)) / 10.0 * 100.0).max(0.0);

        self.overall_effectiveness = index_score;
    }

    /// Get statistics summary
    pub fn get_summary(&self) -> StatisticsSummary {
        StatisticsSummary {
            total_moves: self.move_type_distribution.captures
                + self.move_type_distribution.promotions
                + self.move_type_distribution.quiet_moves,
            capture_percentage: if self.move_type_distribution.captures > 0 {
                (self.move_type_distribution.captures as f64
                    / (self.move_type_distribution.captures
                        + self.move_type_distribution.quiet_moves)
                        .max(1) as f64)
                    * 100.0
            } else {
                0.0
            },
            overall_effectiveness: self.overall_effectiveness,
        }
    }
}

/// Statistics summary for quick overview
#[derive(Debug, Clone, serde::Serialize)]
pub struct StatisticsSummary {
    /// Total moves ordered
    pub total_moves: u64,
    /// Percentage of capture moves
    pub capture_percentage: f64,
    /// Overall effectiveness score (0-100)
    pub overall_effectiveness: f64,
}

// Task 1.22: Extracted from mod.rs - Performance statistics and export structures

/// Comprehensive performance statistics
///
/// Combines ordering statistics with memory usage and cache information
/// for comprehensive performance analysis.
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

/// Statistics export data structure
///
/// Contains all information needed to export statistics for analysis,
/// including timestamp, statistics, configuration, and memory usage.
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
