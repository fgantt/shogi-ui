//! Statistics tracking for move ordering
//! 
//! This module contains structures and methods for tracking performance
//! metrics and statistics for the move ordering system.

use crate::types::*;
use serde::Serialize;

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
    /// SEE cache hit rate percentage
    pub see_cache_hit_rate: f64,
    /// Total time spent on SEE calculations (microseconds)
    pub see_calculation_time_us: u64,
    /// Average time per SEE calculation (microseconds)
    pub avg_see_calculation_time_us: f64,
    // Additional statistics structures would be added here
    // For now, this is a placeholder showing the structure
}

// TODO: Extract additional statistics structures from move_ordering.rs:
// - HotPathStats
// - HeuristicStats
// - TimingStats
// - MemoryStats
// - CacheStats
// - HeuristicPerformance
// - OperationTiming
// - MemoryBreakdown
// - AllocationStats
// - FragmentationStats
// - PerformanceChartData
// - PerformanceTrendAnalysis
// - AdvancedIntegrationStats
// And related methods for updating/accessing these statistics

