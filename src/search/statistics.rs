//! Search Statistics Module
//!
//! This module handles search statistics, telemetry, and profiling for the search engine.
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::types::search::CoreSearchMetrics;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global aggregate of nodes searched across all threads for live reporting.
pub static GLOBAL_NODES_SEARCHED: AtomicU64 = AtomicU64::new(0);

/// Global maximum search depth reached (seldepth) across all threads for live reporting.
pub static GLOBAL_SELDEPTH: AtomicU64 = AtomicU64::new(0);

// Global contention metrics for shared TT
pub static TT_TRY_READS: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_READ_SUCCESSES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_READ_FAILS: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITE_SUCCESSES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITE_FAILS: AtomicU64 = AtomicU64::new(0);

// YBWC metrics
pub static YBWC_SIBLING_BATCHES: AtomicU64 = AtomicU64::new(0);
pub static YBWC_SIBLINGS_EVALUATED: AtomicU64 = AtomicU64::new(0);
// YBWC trigger diagnostics
pub static YBWC_TRIGGER_OPPORTUNITIES: AtomicU64 = AtomicU64::new(0);
pub static YBWC_TRIGGER_ELIGIBLE_DEPTH: AtomicU64 = AtomicU64::new(0);
pub static YBWC_TRIGGER_ELIGIBLE_BRANCH: AtomicU64 = AtomicU64::new(0);
pub static YBWC_TRIGGERED: AtomicU64 = AtomicU64::new(0);

#[inline]
fn take(a: &AtomicU64) -> u64 {
    a.swap(0, Ordering::Relaxed)
}

/// Snapshot and reset global search metrics.
pub struct SearchMetrics {
    pub tt_try_reads: u64,
    pub tt_try_read_successes: u64,
    pub tt_try_read_fails: u64,
    pub tt_try_writes: u64,
    pub tt_try_write_successes: u64,
    pub tt_try_write_fails: u64,
    pub ybwc_sibling_batches: u64,
    pub ybwc_siblings_evaluated: u64,
    pub ybwc_trigger_opportunities: u64,
    pub ybwc_trigger_eligible_depth: u64,
    pub ybwc_trigger_eligible_branch: u64,
    pub ybwc_triggered: u64,
}

/// Snapshot and reset global search metrics.
pub fn snapshot_and_reset_metrics() -> SearchMetrics {
    SearchMetrics {
        tt_try_reads: take(&TT_TRY_READS),
        tt_try_read_successes: take(&TT_TRY_READ_SUCCESSES),
        tt_try_read_fails: take(&TT_TRY_READ_FAILS),
        tt_try_writes: take(&TT_TRY_WRITES),
        tt_try_write_successes: take(&TT_TRY_WRITE_SUCCESSES),
        tt_try_write_fails: take(&TT_TRY_WRITE_FAILS),
        ybwc_sibling_batches: take(&YBWC_SIBLING_BATCHES),
        ybwc_siblings_evaluated: take(&YBWC_SIBLINGS_EVALUATED),
        ybwc_trigger_opportunities: take(&YBWC_TRIGGER_OPPORTUNITIES),
        ybwc_trigger_eligible_depth: take(&YBWC_TRIGGER_ELIGIBLE_DEPTH),
        ybwc_trigger_eligible_branch: take(&YBWC_TRIGGER_ELIGIBLE_BRANCH),
        ybwc_triggered: take(&YBWC_TRIGGERED),
    }
}

/// Print and reset aggregated metrics once (used by benches when SHOGI_AGGREGATE_METRICS=1)
pub fn print_and_reset_search_metrics(tag: &str) {
    let m = snapshot_and_reset_metrics();
    println!(
        "metrics tag={} (aggregate) tt_reads={} tt_read_ok={} tt_read_fail={} tt_writes={} tt_write_ok={} tt_write_fail={} ybwc_batches={} ybwc_siblings={}",
        tag,
        m.tt_try_reads, m.tt_try_read_successes, m.tt_try_read_fails,
        m.tt_try_writes, m.tt_try_write_successes, m.tt_try_write_fails,
        m.ybwc_sibling_batches, m.ybwc_siblings_evaluated
    );
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

/// Statistics manager for search engine
pub struct SearchStatistics {
    /// Total nodes searched in current search
    nodes_searched: u64,
    /// Core search metrics (TT hits, cutoffs, etc.)
    core_metrics: CoreSearchMetrics,
}

impl SearchStatistics {
    /// Create a new SearchStatistics instance
    pub fn new() -> Self {
        Self {
            nodes_searched: 0,
            core_metrics: CoreSearchMetrics::default(),
        }
    }

    /// Increment nodes searched counter
    pub fn increment_nodes(&mut self) {
        self.nodes_searched += 1;
        self.core_metrics.total_nodes += 1;
        GLOBAL_NODES_SEARCHED.fetch_add(1, Ordering::Relaxed);
    }

    /// Update selective depth (maximum depth reached)
    pub fn update_seldepth(&mut self, depth_from_root: u8) {
        let prev_seldepth = GLOBAL_SELDEPTH.load(Ordering::Relaxed);
        if depth_from_root as u64 > prev_seldepth {
            GLOBAL_SELDEPTH.store(depth_from_root as u64, Ordering::Relaxed);
        }
    }

    /// Increment TT probe counter
    pub fn increment_tt_probe(&mut self) {
        self.core_metrics.total_tt_probes += 1;
    }

    /// Record TT hit
    pub fn record_tt_hit(&mut self, is_exact: bool, is_lower_bound: bool, is_upper_bound: bool) {
        self.core_metrics.total_tt_hits += 1;
        if is_exact {
            self.core_metrics.tt_exact_hits += 1;
        } else if is_lower_bound {
            self.core_metrics.tt_lower_bound_hits += 1;
        } else if is_upper_bound {
            self.core_metrics.tt_upper_bound_hits += 1;
        }
    }

    /// Record cutoff
    pub fn record_cutoff(&mut self) {
        self.core_metrics.total_cutoffs += 1;
    }

    /// Record TT auxiliary overwrite prevention
    pub fn record_tt_auxiliary_overwrite_prevented(&mut self) {
        self.core_metrics.tt_auxiliary_overwrites_prevented += 1;
    }

    /// Record TT main entry preservation
    pub fn record_tt_main_entry_preserved(&mut self) {
        self.core_metrics.tt_main_entries_preserved += 1;
    }

    /// Record evaluation cache hit
    pub fn record_evaluation_cache_hit(&mut self) {
        self.core_metrics.evaluation_cache_hits += 1;
    }

    /// Record evaluation call saved
    pub fn record_evaluation_call_saved(&mut self) {
        self.core_metrics.evaluation_calls_saved += 1;
    }

    /// Get nodes searched count
    pub fn get_nodes_searched(&self) -> u64 {
        self.nodes_searched
    }

    /// Get core search metrics
    pub fn get_core_metrics(&self) -> &CoreSearchMetrics {
        &self.core_metrics
    }

    /// Get mutable reference to core metrics (for direct updates if needed)
    pub fn get_core_metrics_mut(&mut self) -> &mut CoreSearchMetrics {
        &mut self.core_metrics
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.nodes_searched = 0;
        self.core_metrics = CoreSearchMetrics::default();
    }

    /// Reset nodes counter (but keep other metrics)
    pub fn reset_nodes(&mut self) {
        self.nodes_searched = 0;
    }
}

impl Default for SearchStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_statistics_creation() {
        let stats = SearchStatistics::new();
        assert_eq!(stats.get_nodes_searched(), 0);
        assert_eq!(stats.get_core_metrics().total_nodes, 0);
    }

    #[test]
    fn test_increment_nodes() {
        let mut stats = SearchStatistics::new();
        stats.increment_nodes();
        assert_eq!(stats.get_nodes_searched(), 1);
        assert_eq!(stats.get_core_metrics().total_nodes, 1);
    }

    #[test]
    fn test_tt_hit_recording() {
        let mut stats = SearchStatistics::new();
        stats.increment_tt_probe();
        stats.record_tt_hit(true, false, false); // Exact hit
        assert_eq!(stats.get_core_metrics().total_tt_probes, 1);
        assert_eq!(stats.get_core_metrics().total_tt_hits, 1);
        assert_eq!(stats.get_core_metrics().tt_exact_hits, 1);
    }

    #[test]
    fn test_reset() {
        let mut stats = SearchStatistics::new();
        stats.increment_nodes();
        stats.record_cutoff();
        stats.reset();
        assert_eq!(stats.get_nodes_searched(), 0);
        assert_eq!(stats.get_core_metrics().total_cutoffs, 0);
    }
}

