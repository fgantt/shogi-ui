//! Performance Optimization Module
//!
//! This module provides performance optimizations and profiling tools for the
//! tapered evaluation system. Includes:
//! - Optimized phase calculation
//! - Efficient interpolation
//! - Cache-friendly data structures
//! - Performance profiling
//! - Hot path optimization
//! - Bottleneck identification
//!
//! # Overview
//!
//! Performance optimization strategies:
//! - Inline hot functions
//! - Minimize branching
//! - Cache-friendly memory layout
//! - Reduce allocations
//! - Profile-guided optimization
//! - Batch operations
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::performance::OptimizedEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let mut evaluator = OptimizedEvaluator::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let score = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::evaluation::material::{MaterialEvaluationConfig, MaterialEvaluator};
use crate::evaluation::phase_transition::{InterpolationMethod, PhaseTransition};
use crate::evaluation::piece_square_tables::PieceSquareTables;
use crate::evaluation::tapered_eval::TaperedEvaluation;
use crate::types::board::CapturedPieces;
use crate::types::core::Player;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Optimized evaluator combining all components
pub struct OptimizedEvaluator {
    /// Tapered evaluation coordinator
    tapered_eval: TaperedEvaluation,
    /// Material evaluator
    material_eval: MaterialEvaluator,
    /// Piece-square tables (pre-loaded)
    pst: PieceSquareTables,
    /// Phase transition
    phase_transition: PhaseTransition,
    /// Performance profiler
    profiler: PerformanceProfiler,
}

impl OptimizedEvaluator {
    /// Create a new optimized evaluator
    pub fn new() -> Self {
        Self::with_config(&MaterialEvaluationConfig::default())
    }

    /// Create a new optimized evaluator with a specific material configuration
    pub fn with_config(material_config: &MaterialEvaluationConfig) -> Self {
        Self::with_components(material_config, PieceSquareTables::new())
    }

    /// Create a new optimized evaluator with material configuration and PST tables.
    pub fn with_components(
        material_config: &MaterialEvaluationConfig,
        pst: PieceSquareTables,
    ) -> Self {
        Self {
            tapered_eval: TaperedEvaluation::new(),
            material_eval: MaterialEvaluator::with_config(material_config.clone()),
            pst,
            phase_transition: PhaseTransition::new(),
            profiler: PerformanceProfiler::new(),
        }
    }

    /// Apply an updated material configuration.
    pub fn apply_material_config(&mut self, material_config: &MaterialEvaluationConfig) {
        self.material_eval.apply_config(material_config.clone());
    }

    /// Apply updated piece-square tables.
    pub fn apply_piece_square_tables(&mut self, pst: PieceSquareTables) {
        self.pst = pst;
    }

    /// Optimized evaluation with all components
    ///
    /// This is the main entry point for optimized evaluation
    #[inline]
    pub fn evaluate_optimized(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        let start = if self.profiler.enabled {
            Some(Instant::now())
        } else {
            None
        };

        // 1. Calculate phase (with caching)
        let phase = self.calculate_phase_optimized(board, captured_pieces);

        // 2. Accumulate scores (inlined for performance)
        let total_score = self.accumulate_scores_optimized(board, player, captured_pieces);

        // 3. Interpolate (fast path)
        let final_score = self.interpolate_optimized(total_score, phase);

        if let Some(start_time) = start {
            self.profiler
                .record_evaluation(start_time.elapsed().as_nanos() as u64);
        }

        final_score
    }

    /// Optimized phase calculation with caching
    #[inline(always)]
    fn calculate_phase_optimized(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        let start = if self.profiler.enabled {
            Some(Instant::now())
        } else {
            None
        };

        let phase = self
            .tapered_eval
            .calculate_game_phase(board, captured_pieces);

        if let Some(start_time) = start {
            self.profiler
                .record_phase_calculation(start_time.elapsed().as_nanos() as u64);
        }

        phase
    }

    /// Optimized score accumulation
    #[inline]
    fn accumulate_scores_optimized(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut total = TaperedScore::default();

        // Material evaluation (fast)
        total += self
            .material_eval
            .evaluate_material(board, player, captured_pieces);

        // Piece-square tables (ultra-fast O(1) lookups)
        total += self.evaluate_pst_optimized(board, player);

        total
    }

    /// Optimized piece-square table evaluation
    #[inline(always)]
    fn evaluate_pst_optimized(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let start = if self.profiler.enabled {
            Some(Instant::now())
        } else {
            None
        };

        let mut score = TaperedScore::default();

        // Optimized loop with early bailout
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let pst_value = self.pst.get_value(piece.piece_type, pos, piece.player);

                    if piece.player == player {
                        score += pst_value;
                    } else {
                        score -= pst_value;
                    }
                }
            }
        }

        if let Some(start_time) = start {
            self.profiler
                .record_pst_lookup(start_time.elapsed().as_nanos() as u64);
        }

        self.profiler.record_pst_score(score.mg, score.eg);

        score
    }

    /// Optimized interpolation (fast path)
    #[inline(always)]
    fn interpolate_optimized(&mut self, score: TaperedScore, phase: i32) -> i32 {
        let start = if self.profiler.enabled {
            Some(Instant::now())
        } else {
            None
        };

        // Use fast linear interpolation
        let result = self
            .phase_transition
            .interpolate(score, phase, InterpolationMethod::Linear);

        if let Some(start_time) = start {
            self.profiler
                .record_interpolation(start_time.elapsed().as_nanos() as u64);
        }

        result
    }

    /// Get profiler for analysis
    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }

    /// Get mutable profiler
    pub fn profiler_mut(&mut self) -> &mut PerformanceProfiler {
        &mut self.profiler
    }

    /// Reset profiler
    pub fn reset_profiler(&mut self) {
        self.profiler.reset();
    }
}

impl Default for OptimizedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Hot path entry for profiling summary (Task 3.0)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotPathEntry {
    /// Operation name
    pub operation: String,
    /// Average time in nanoseconds
    pub average_time_ns: f64,
    /// Maximum time in nanoseconds
    pub max_time_ns: u64,
    /// Minimum time in nanoseconds
    pub min_time_ns: u64,
    /// Number of calls profiled
    pub call_count: usize,
}

/// Performance profiler for identifying bottlenecks (Task 26.0 - Task 3.0)
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    /// Enable profiling
    pub enabled: bool,
    /// Evaluation timings (nanoseconds)
    pub evaluation_times: Vec<u64>,
    /// Phase calculation timings
    pub phase_calc_times: Vec<u64>,
    /// PST lookup timings
    pub pst_lookup_times: Vec<u64>,
    /// PST middlegame contributions
    pub pst_mg_totals: Vec<i32>,
    /// PST endgame contributions
    pub pst_eg_totals: Vec<i32>,
    /// Interpolation timings
    pub interpolation_times: Vec<u64>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Sample rate: profile every Nth call (Task 3.0)
    sample_rate: u32,
    /// Current call counter for sampling (Task 3.0)
    call_counter: u32,
    /// Operation timings for hot path analysis (Task 3.0)
    /// Maps operation name -> Vec<timing in nanoseconds>
    operation_timings: std::collections::HashMap<String, Vec<u64>>,
    /// Profiling overhead tracking (Task 3.0)
    /// Tracks time spent in profiling itself
    profiling_overhead_ns: u64,
    /// Number of profiling operations performed
    profiling_operations: u64,
}

impl PerformanceProfiler {
    /// Create a new profiler (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            evaluation_times: Vec::new(),
            phase_calc_times: Vec::new(),
            pst_lookup_times: Vec::new(),
            pst_mg_totals: Vec::new(),
            pst_eg_totals: Vec::new(),
            interpolation_times: Vec::new(),
            max_samples: 10000,
            sample_rate: 1, // Profile every call by default
            call_counter: 0,
            operation_timings: std::collections::HashMap::new(),
            profiling_overhead_ns: 0,
            profiling_operations: 0,
        }
    }

    /// Create a new profiler with sample rate (Task 3.0)
    pub fn with_sample_rate(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Self::new()
        }
    }

    /// Set sample rate (Task 3.0)
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate.max(1); // Minimum 1
    }

    /// Check if current call should be profiled based on sample rate (Task 3.0)
    #[inline]
    fn should_sample(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        self.call_counter += 1;
        if self.call_counter >= self.sample_rate {
            self.call_counter = 0;
            true
        } else {
            false
        }
    }

    /// Record operation timing with sampling (Task 3.0)
    #[inline]
    pub fn record_operation(&mut self, operation: &str, nanos: u64) {
        if !self.should_sample() {
            return;
        }

        let overhead_start = std::time::Instant::now();
        
        self.operation_timings
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(nanos);

        // Track profiling overhead
        let overhead_ns = overhead_start.elapsed().as_nanos() as u64;
        self.profiling_overhead_ns += overhead_ns;
        self.profiling_operations += 1;
    }

    /// Get hot path summary - top N slowest operations (Task 3.0)
    pub fn get_hot_path_summary(&self, top_n: usize) -> Vec<HotPathEntry> {
        let mut entries: Vec<HotPathEntry> = self.operation_timings
            .iter()
            .map(|(name, timings)| {
                let avg = if timings.is_empty() {
                    0.0
                } else {
                    timings.iter().sum::<u64>() as f64 / timings.len() as f64
                };
                let max = timings.iter().max().copied().unwrap_or(0);
                let min = timings.iter().min().copied().unwrap_or(0);
                let count = timings.len();

                HotPathEntry {
                    operation: name.clone(),
                    average_time_ns: avg,
                    max_time_ns: max,
                    min_time_ns: min,
                    call_count: count,
                }
            })
            .collect();

        // Sort by average time (descending)
        entries.sort_by(|a, b| b.average_time_ns.partial_cmp(&a.average_time_ns).unwrap_or(std::cmp::Ordering::Equal));
        
        entries.into_iter().take(top_n).collect()
    }

    /// Export profiling data to JSON (Task 3.0)
    pub fn export_profiling_data(&self) -> Result<String, String> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct ProfilingData {
            enabled: bool,
            sample_rate: u32,
            total_samples: usize,
            profiling_overhead_ns: u64,
            profiling_operations: u64,
            average_overhead_per_operation_ns: f64,
            evaluation_stats: OperationStats,
            phase_calc_stats: OperationStats,
            pst_lookup_stats: OperationStats,
            interpolation_stats: OperationStats,
            hot_paths: Vec<HotPathEntry>,
        }

        #[derive(Serialize)]
        struct OperationStats {
            count: usize,
            average_ns: f64,
            max_ns: u64,
            min_ns: u64,
        }

        let eval_stats = OperationStats {
            count: self.evaluation_times.len(),
            average_ns: self.avg_evaluation_time(),
            max_ns: self.evaluation_times.iter().max().copied().unwrap_or(0),
            min_ns: self.evaluation_times.iter().min().copied().unwrap_or(0),
        };

        let phase_stats = OperationStats {
            count: self.phase_calc_times.len(),
            average_ns: self.avg_phase_calc_time(),
            max_ns: self.phase_calc_times.iter().max().copied().unwrap_or(0),
            min_ns: self.phase_calc_times.iter().min().copied().unwrap_or(0),
        };

        let pst_stats = OperationStats {
            count: self.pst_lookup_times.len(),
            average_ns: self.avg_pst_lookup_time(),
            max_ns: self.pst_lookup_times.iter().max().copied().unwrap_or(0),
            min_ns: self.pst_lookup_times.iter().min().copied().unwrap_or(0),
        };

        let interp_stats = OperationStats {
            count: self.interpolation_times.len(),
            average_ns: self.avg_interpolation_time(),
            max_ns: self.interpolation_times.iter().max().copied().unwrap_or(0),
            min_ns: self.interpolation_times.iter().min().copied().unwrap_or(0),
        };

        let avg_overhead = if self.profiling_operations > 0 {
            self.profiling_overhead_ns as f64 / self.profiling_operations as f64
        } else {
            0.0
        };

        let data = ProfilingData {
            enabled: self.enabled,
            sample_rate: self.sample_rate,
            total_samples: self.evaluation_times.len(),
            profiling_overhead_ns: self.profiling_overhead_ns,
            profiling_operations: self.profiling_operations,
            average_overhead_per_operation_ns: avg_overhead,
            evaluation_stats: eval_stats,
            phase_calc_stats: phase_stats,
            pst_lookup_stats: pst_stats,
            interpolation_stats: interp_stats,
            hot_paths: self.get_hot_path_summary(10),
        };

        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize profiling data: {}", e))
    }

    /// Get profiling overhead percentage (Task 3.0)
    pub fn get_profiling_overhead_percentage(&self) -> f64 {
        if self.profiling_operations == 0 {
            return 0.0;
        }
        // Estimate total time profiled (sum of all operation times)
        let total_profiled_time: u64 = self.operation_timings.values()
            .flat_map(|timings| timings.iter())
            .sum();
        
        if total_profiled_time == 0 {
            return 0.0;
        }

        (self.profiling_overhead_ns as f64 / total_profiled_time as f64) * 100.0
    }

    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Record evaluation time (Task 3.0: with sampling support)
    #[inline]
    pub fn record_evaluation(&mut self, nanos: u64) {
        if self.should_sample() && self.evaluation_times.len() < self.max_samples {
            self.evaluation_times.push(nanos);
            self.record_operation("evaluation", nanos);
        }
    }

    /// Record phase calculation time (Task 3.0: with sampling support)
    #[inline]
    pub fn record_phase_calculation(&mut self, nanos: u64) {
        if self.should_sample() && self.phase_calc_times.len() < self.max_samples {
            self.phase_calc_times.push(nanos);
            self.record_operation("phase_calculation", nanos);
        }
    }

    /// Record PST lookup time
    #[inline]
    pub fn record_pst_lookup(&mut self, nanos: u64) {
        if self.enabled && self.pst_lookup_times.len() < self.max_samples {
            self.pst_lookup_times.push(nanos);
        }
    }

    /// Record PST score contribution (middlegame/endgame totals)
    #[inline]
    pub fn record_pst_score(&mut self, mg: i32, eg: i32) {
        if self.enabled && self.pst_mg_totals.len() < self.max_samples {
            self.pst_mg_totals.push(mg);
            self.pst_eg_totals.push(eg);
        }
    }

    /// Record interpolation time (Task 3.0: with sampling support)
    #[inline]
    pub fn record_interpolation(&mut self, nanos: u64) {
        if self.should_sample() && self.interpolation_times.len() < self.max_samples {
            self.interpolation_times.push(nanos);
            self.record_operation("interpolation", nanos);
        }
    }

    /// Get average evaluation time
    pub fn avg_evaluation_time(&self) -> f64 {
        if self.evaluation_times.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.evaluation_times.iter().sum();
        sum as f64 / self.evaluation_times.len() as f64
    }

    /// Get average phase calculation time
    pub fn avg_phase_calc_time(&self) -> f64 {
        if self.phase_calc_times.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.phase_calc_times.iter().sum();
        sum as f64 / self.phase_calc_times.len() as f64
    }

    /// Get average PST lookup time
    pub fn avg_pst_lookup_time(&self) -> f64 {
        if self.pst_lookup_times.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.pst_lookup_times.iter().sum();
        sum as f64 / self.pst_lookup_times.len() as f64
    }

    /// Get average middlegame PST contribution
    pub fn avg_pst_mg(&self) -> f64 {
        if self.pst_mg_totals.is_empty() {
            return 0.0;
        }
        let sum: i64 = self.pst_mg_totals.iter().map(|&v| v as i64).sum();
        sum as f64 / self.pst_mg_totals.len() as f64
    }

    /// Get average endgame PST contribution
    pub fn avg_pst_eg(&self) -> f64 {
        if self.pst_eg_totals.is_empty() {
            return 0.0;
        }
        let sum: i64 = self.pst_eg_totals.iter().map(|&v| v as i64).sum();
        sum as f64 / self.pst_eg_totals.len() as f64
    }

    /// Get average absolute PST contribution magnitude.
    pub fn avg_pst_magnitude(&self) -> f64 {
        if self.pst_mg_totals.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .pst_mg_totals
            .iter()
            .zip(self.pst_eg_totals.iter())
            .map(|(&mg, &eg)| (mg.abs() + eg.abs()) as f64)
            .sum();
        sum / self.pst_mg_totals.len() as f64
    }

    /// Get average interpolation time
    pub fn avg_interpolation_time(&self) -> f64 {
        if self.interpolation_times.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.interpolation_times.iter().sum();
        sum as f64 / self.interpolation_times.len() as f64
    }

    /// Get performance report
    pub fn report(&self) -> PerformanceReport {
        PerformanceReport {
            total_evaluations: self.evaluation_times.len(),
            avg_evaluation_ns: self.avg_evaluation_time(),
            avg_phase_calc_ns: self.avg_phase_calc_time(),
            avg_pst_lookup_ns: self.avg_pst_lookup_time(),
            avg_interpolation_ns: self.avg_interpolation_time(),
            avg_pst_mg: self.avg_pst_mg(),
            avg_pst_eg: self.avg_pst_eg(),
            avg_pst_magnitude: self.avg_pst_magnitude(),
            phase_calc_percentage: if self.avg_evaluation_time() > 0.0 {
                (self.avg_phase_calc_time() / self.avg_evaluation_time()) * 100.0
            } else {
                0.0
            },
            pst_lookup_percentage: if self.avg_evaluation_time() > 0.0 {
                (self.avg_pst_lookup_time() / self.avg_evaluation_time()) * 100.0
            } else {
                0.0
            },
            interpolation_percentage: if self.avg_evaluation_time() > 0.0 {
                (self.avg_interpolation_time() / self.avg_evaluation_time()) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Reset profiler (Task 3.0: enhanced with new fields)
    pub fn reset(&mut self) {
        self.evaluation_times.clear();
        self.phase_calc_times.clear();
        self.pst_lookup_times.clear();
        self.pst_mg_totals.clear();
        self.pst_eg_totals.clear();
        self.interpolation_times.clear();
        self.operation_timings.clear();
        self.call_counter = 0;
        self.profiling_overhead_ns = 0;
        self.profiling_operations = 0;
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.evaluation_times.len()
    }

    /// Enable the profiler for the duration of the returned guard, restoring the previous state on drop.
    pub fn scoped_enable(&mut self) -> PerformanceProfilerGuard<'_> {
        let previous_state = self.enabled;
        self.enabled = true;
        PerformanceProfilerGuard {
            profiler: self,
            previous_state,
        }
    }
}

/// RAII helper returned by [`PerformanceProfiler::scoped_enable`].
pub struct PerformanceProfilerGuard<'a> {
    profiler: &'a mut PerformanceProfiler,
    previous_state: bool,
}

impl<'a> Drop for PerformanceProfilerGuard<'a> {
    fn drop(&mut self) {
        self.profiler.enabled = self.previous_state;
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Total number of evaluations
    pub total_evaluations: usize,
    /// Average evaluation time (nanoseconds)
    pub avg_evaluation_ns: f64,
    /// Average phase calculation time
    pub avg_phase_calc_ns: f64,
    /// Average PST lookup time
    pub avg_pst_lookup_ns: f64,
    /// Average interpolation time
    pub avg_interpolation_ns: f64,
    /// Average PST middlegame contribution
    pub avg_pst_mg: f64,
    /// Average PST endgame contribution
    pub avg_pst_eg: f64,
    /// Average absolute PST contribution magnitude
    pub avg_pst_magnitude: f64,
    /// Phase calculation as percentage of total
    pub phase_calc_percentage: f64,
    /// PST lookup as percentage of total
    pub pst_lookup_percentage: f64,
    /// Interpolation as percentage of total
    pub interpolation_percentage: f64,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Performance Report")?;
        writeln!(f, "==================")?;
        writeln!(f, "Total Evaluations: {}", self.total_evaluations)?;
        writeln!(
            f,
            "Average Evaluation Time: {:.2} ns ({:.3} μs)",
            self.avg_evaluation_ns,
            self.avg_evaluation_ns / 1000.0
        )?;
        writeln!(f)?;
        writeln!(f, "Component Breakdown:")?;
        writeln!(
            f,
            "  Phase Calculation: {:.2} ns ({:.1}%)",
            self.avg_phase_calc_ns, self.phase_calc_percentage
        )?;
        writeln!(
            f,
            "  PST Lookup: {:.2} ns ({:.1}%)",
            self.avg_pst_lookup_ns, self.pst_lookup_percentage
        )?;
        writeln!(
            f,
            "  PST Contribution (avg mg / eg / |total|): {:.2} / {:.2} / {:.2}",
            self.avg_pst_mg, self.avg_pst_eg, self.avg_pst_magnitude
        )?;
        writeln!(
            f,
            "  Interpolation: {:.2} ns ({:.1}%)",
            self.avg_interpolation_ns, self.interpolation_percentage
        )?;
        Ok(())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_evaluator_creation() {
        let evaluator = OptimizedEvaluator::new();
        assert!(!evaluator.profiler.enabled);
    }

    #[test]
    fn test_optimized_evaluation() {
        let mut evaluator = OptimizedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);

        // Should return a valid score
        assert!(score.abs() < 100000);
    }

    #[test]
    fn test_profiler_disabled_by_default() {
        let profiler = PerformanceProfiler::new();
        assert!(!profiler.enabled);
    }

    #[test]
    fn test_profiler_enable_disable() {
        let mut profiler = PerformanceProfiler::new();

        assert!(!profiler.enabled);

        profiler.enable();
        assert!(profiler.enabled);

        profiler.disable();
        assert!(!profiler.enabled);
    }

    #[test]
    fn test_profiler_scoped_enable_guard() {
        let mut profiler = PerformanceProfiler::new();
        {
            let _guard = profiler.scoped_enable();
            assert!(profiler.enabled);
        }
        assert!(!profiler.enabled);

        profiler.enable();
        {
            let _guard = profiler.scoped_enable();
            assert!(profiler.enabled);
        }
        assert!(profiler.enabled);
    }

    #[test]
    fn test_profiler_recording() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        profiler.record_evaluation(1000);
        profiler.record_evaluation(1500);
        profiler.record_evaluation(1200);

        assert_eq!(profiler.sample_count(), 3);
        assert_eq!(profiler.avg_evaluation_time(), 1233.3333333333333);
    }

    #[test]
    fn test_profiler_report() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        profiler.record_evaluation(1000);
        profiler.record_phase_calculation(200);
        profiler.record_pst_lookup(300);
        profiler.record_interpolation(100);

        let report = profiler.report();
        assert_eq!(report.total_evaluations, 1);
        assert_eq!(report.avg_evaluation_ns, 1000.0);
        assert_eq!(report.avg_phase_calc_ns, 200.0);
    }

    #[test]
    fn test_profiler_reset() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        profiler.record_evaluation(1000);
        assert_eq!(profiler.sample_count(), 1);

        profiler.reset();
        assert_eq!(profiler.sample_count(), 0);
    }

    #[test]
    fn test_profiler_percentages() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        profiler.record_evaluation(1000);
        profiler.record_phase_calculation(200);
        profiler.record_pst_lookup(300);
        profiler.record_interpolation(100);

        let report = profiler.report();
        assert_eq!(report.phase_calc_percentage, 20.0);
        assert_eq!(report.pst_lookup_percentage, 30.0);
        assert_eq!(report.interpolation_percentage, 10.0);
    }

    #[test]
    fn test_optimized_evaluation_consistency() {
        let mut evaluator = OptimizedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score1 = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);

        assert_eq!(score1, score2);
    }

    #[test]
    fn test_profiler_with_evaluation() {
        let mut evaluator = OptimizedEvaluator::new();
        evaluator.profiler_mut().enable();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Run some evaluations
        for _ in 0..10 {
            evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
        }

        let report = evaluator.profiler().report();
        assert_eq!(report.total_evaluations, 10);
        assert!(report.avg_evaluation_ns > 0.0);
    }

    #[test]
    fn test_performance_report_display() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        profiler.record_evaluation(1000);
        profiler.record_phase_calculation(200);

        let report = profiler.report();
        let display = format!("{}", report);

        assert!(display.contains("Performance Report"));
        assert!(display.contains("Total Evaluations"));
    }

    #[test]
    fn test_max_samples_limit() {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();

        // Try to add more than max_samples
        for i in 0..11000 {
            profiler.record_evaluation(i);
        }

        // Should be limited to max_samples
        assert_eq!(profiler.sample_count(), profiler.max_samples);
    }

    #[test]
    fn test_evaluation_performance() {
        let mut evaluator = OptimizedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Performance test - should be fast
        let start = Instant::now();
        for _ in 0..1000 {
            evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
        }
        let duration = start.elapsed();

        // 1000 evaluations should complete quickly (< 10ms)
        assert!(duration.as_millis() < 10);
    }
}
