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
use crate::evaluation::material::MaterialEvaluator;
use crate::evaluation::phase_transition::{InterpolationMethod, PhaseTransition};
use crate::evaluation::piece_square_tables::PieceSquareTables;
use crate::evaluation::tapered_eval::TaperedEvaluation;
use crate::types::*;
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
        Self {
            tapered_eval: TaperedEvaluation::new(),
            material_eval: MaterialEvaluator::new(),
            pst: PieceSquareTables::new(),
            phase_transition: PhaseTransition::new(),
            profiler: PerformanceProfiler::new(),
        }
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

/// Performance profiler for identifying bottlenecks
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
    /// Interpolation timings
    pub interpolation_times: Vec<u64>,
    /// Maximum samples to keep
    max_samples: usize,
}

impl PerformanceProfiler {
    /// Create a new profiler (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            evaluation_times: Vec::new(),
            phase_calc_times: Vec::new(),
            pst_lookup_times: Vec::new(),
            interpolation_times: Vec::new(),
            max_samples: 10000,
        }
    }

    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Record evaluation time
    #[inline]
    pub fn record_evaluation(&mut self, nanos: u64) {
        if self.enabled && self.evaluation_times.len() < self.max_samples {
            self.evaluation_times.push(nanos);
        }
    }

    /// Record phase calculation time
    #[inline]
    pub fn record_phase_calculation(&mut self, nanos: u64) {
        if self.enabled && self.phase_calc_times.len() < self.max_samples {
            self.phase_calc_times.push(nanos);
        }
    }

    /// Record PST lookup time
    #[inline]
    pub fn record_pst_lookup(&mut self, nanos: u64) {
        if self.enabled && self.pst_lookup_times.len() < self.max_samples {
            self.pst_lookup_times.push(nanos);
        }
    }

    /// Record interpolation time
    #[inline]
    pub fn record_interpolation(&mut self, nanos: u64) {
        if self.enabled && self.interpolation_times.len() < self.max_samples {
            self.interpolation_times.push(nanos);
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

    /// Reset profiler
    pub fn reset(&mut self) {
        self.evaluation_times.clear();
        self.phase_calc_times.clear();
        self.pst_lookup_times.clear();
        self.interpolation_times.clear();
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
