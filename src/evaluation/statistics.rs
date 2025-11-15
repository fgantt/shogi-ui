//! Statistics and Monitoring Module
//!
//! This module provides comprehensive statistics tracking and monitoring for the
//! tapered evaluation system. Tracks:
//! - Evaluation statistics (count, averages, distributions)
//! - Phase distribution across evaluations
//! - Accuracy metrics (prediction quality)
//! - Performance metrics (timing, throughput)
//! - Export capabilities (JSON, CSV)
//!
//! # Overview
//!
//! The statistics system:
//! - Real-time tracking of evaluation metrics
//! - Phase distribution analysis
//! - Accuracy measurement
//! - Performance monitoring
//! - Export to various formats
//! - Minimal overhead when disabled
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::statistics::EvaluationStatistics;
//!
//! let mut stats = EvaluationStatistics::new();
//! stats.enable();
//!
//! // Record evaluations
//! stats.record_evaluation(150, 200);
//! stats.record_phase(128);
//!
//! // Get report
//! let report = stats.generate_report();
//! println!("{}", report);
//! ```

use crate::evaluation::castles::CastleCacheStats;
use crate::evaluation::king_safety::KingSafetyStatsSnapshot;
use crate::evaluation::material::MaterialTelemetry;
use crate::evaluation::performance::PerformanceReport;
use crate::evaluation::phase_transition::PhaseTransitionSnapshot;
use crate::evaluation::position_features::PositionFeatureStats;
use crate::evaluation::positional_patterns::PositionalStatsSnapshot;
use crate::evaluation::tactical_patterns::TacticalStatsSnapshot;
use crate::evaluation::tapered_eval::TaperedEvaluationSnapshot;
use crate::types::core::PieceType;
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive evaluation statistics tracker
#[derive(Debug, Clone)]
pub struct EvaluationStatistics {
    /// Enable statistics tracking
    enabled: bool,
    /// Evaluation count
    evaluation_count: u64,
    /// Score statistics
    score_stats: ScoreStatistics,
    /// Phase statistics
    phase_stats: PhaseStatistics,
    /// Accuracy metrics
    accuracy_metrics: AccuracyMetrics,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Start time for session tracking
    session_start: Option<Instant>,
    /// Latest telemetry snapshot from the evaluator
    telemetry: Option<EvaluationTelemetry>,
    /// Aggregated piece-square table statistics
    pst_stats: PieceSquareStatisticsAggregate,
    /// Whether to collect position feature statistics
    collect_position_feature_stats: bool,
    /// Latest position feature statistics snapshot
    position_feature_stats: Option<PositionFeatureStats>,
    /// Latest tactical statistics snapshot
    tactical_stats: Option<TacticalStatsSnapshot>,
    /// Latest positional statistics snapshot
    positional_stats: Option<PositionalStatsSnapshot>,
    /// Latest king safety statistics snapshot
    king_safety_stats: Option<KingSafetyStatsSnapshot>,
}

/// Aggregated telemetry emitted by the integrated evaluator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationTelemetry {
    pub tapered: Option<TaperedEvaluationSnapshot>,
    pub phase_transition: Option<PhaseTransitionSnapshot>,
    pub performance: Option<PerformanceReport>,
    pub material: Option<MaterialTelemetry>,
    pub pst: Option<PieceSquareTelemetry>,
    pub position_features: Option<PositionFeatureStats>,
    pub positional: Option<PositionalStatsSnapshot>,
    pub tactical: Option<TacticalStatsSnapshot>,
    pub king_safety: Option<KingSafetyStatsSnapshot>,
    pub castle_patterns: Option<CastleCacheStats>,
    /// Weight contributions: component name -> contribution percentage (0.0-1.0)
    pub weight_contributions: HashMap<String, f32>,
}

impl EvaluationTelemetry {
    pub fn from_snapshots(
        tapered: TaperedEvaluationSnapshot,
        phase_transition: PhaseTransitionSnapshot,
        performance: Option<PerformanceReport>,
        material: Option<MaterialTelemetry>,
        pst: Option<PieceSquareTelemetry>,
        position_features: Option<PositionFeatureStats>,
        positional: Option<PositionalStatsSnapshot>,
        tactical: Option<TacticalStatsSnapshot>,
        king_safety: Option<KingSafetyStatsSnapshot>,
        castle_patterns: Option<CastleCacheStats>,
    ) -> Self {
        Self {
            tapered: Some(tapered),
            weight_contributions: HashMap::new(),
            phase_transition: Some(phase_transition),
            performance,
            material,
            pst,
            position_features,
            positional,
            tactical,
            king_safety,
            castle_patterns,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceSquarePieceTelemetry {
    pub piece: PieceType,
    pub mg: i32,
    pub eg: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PieceSquareTelemetry {
    pub total_mg: i32,
    pub total_eg: i32,
    pub per_piece: Vec<PieceSquarePieceTelemetry>,
}

impl PieceSquareTelemetry {
    pub fn from_contributions(
        total: TaperedScore,
        per_piece: &[TaperedScore; PieceType::COUNT],
    ) -> Self {
        const PIECE_TYPES: [PieceType; PieceType::COUNT] = [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::King,
            PieceType::PromotedPawn,
            PieceType::PromotedLance,
            PieceType::PromotedKnight,
            PieceType::PromotedSilver,
            PieceType::PromotedBishop,
            PieceType::PromotedRook,
        ];

        let mut entries = Vec::new();
        for (piece, contribution) in PIECE_TYPES.iter().zip(per_piece.iter()) {
            if contribution.mg != 0 || contribution.eg != 0 {
                entries.push(PieceSquarePieceTelemetry {
                    piece: *piece,
                    mg: contribution.mg,
                    eg: contribution.eg,
                });
            }
        }

        Self {
            total_mg: total.mg,
            total_eg: total.eg,
            per_piece: entries,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PieceSquareStatisticsAggregate {
    total_mg_sum: i64,
    total_eg_sum: i64,
    sample_count: u64,
    per_piece: Vec<PieceSquarePieceAggregate>,
    last_total_mg: Option<i32>,
    last_total_eg: Option<i32>,
    previous_total_mg: Option<i32>,
    previous_total_eg: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PieceSquarePieceAggregate {
    sum_mg: i64,
    sum_eg: i64,
    samples: u64,
}

impl PieceSquarePieceAggregate {
    fn record(&mut self, mg: i32, eg: i32) {
        self.sum_mg += mg as i64;
        self.sum_eg += eg as i64;
        self.samples += 1;
    }

    fn average(&self) -> (f64, f64) {
        if self.samples == 0 {
            (0.0, 0.0)
        } else {
            (
                self.sum_mg as f64 / self.samples as f64,
                self.sum_eg as f64 / self.samples as f64,
            )
        }
    }
}

impl PieceSquareStatisticsAggregate {
    fn ensure_capacity(&mut self) {
        if self.per_piece.len() != PieceType::COUNT {
            self.per_piece = vec![PieceSquarePieceAggregate::default(); PieceType::COUNT];
        }
    }

    fn record(&mut self, telemetry: &PieceSquareTelemetry) {
        self.ensure_capacity();
        self.total_mg_sum += telemetry.total_mg as i64;
        self.total_eg_sum += telemetry.total_eg as i64;
        self.sample_count += 1;

        self.previous_total_mg = self.last_total_mg;
        self.previous_total_eg = self.last_total_eg;
        self.last_total_mg = Some(telemetry.total_mg);
        self.last_total_eg = Some(telemetry.total_eg);

        for entry in &telemetry.per_piece {
            let idx = entry.piece.as_index();
            if let Some(aggregate) = self.per_piece.get_mut(idx) {
                aggregate.record(entry.mg, entry.eg);
            }
        }
    }

    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    pub fn average_total_mg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.total_mg_sum as f64 / self.sample_count as f64
        }
    }

    pub fn average_total_eg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.total_eg_sum as f64 / self.sample_count as f64
        }
    }

    pub fn average_for_piece(&self, piece: PieceType) -> (f64, f64) {
        let idx = piece.as_index();
        self.per_piece
            .get(idx)
            .map(|entry| entry.average())
            .unwrap_or((0.0, 0.0))
    }

    pub fn last_totals(&self) -> Option<(i32, i32)> {
        match (self.last_total_mg, self.last_total_eg) {
            (Some(mg), Some(eg)) => Some((mg, eg)),
            _ => None,
        }
    }

    pub fn previous_totals(&self) -> Option<(i32, i32)> {
        match (self.previous_total_mg, self.previous_total_eg) {
            (Some(mg), Some(eg)) => Some((mg, eg)),
            _ => None,
        }
    }

    pub fn top_contributors(&self, limit: usize) -> Vec<(PieceType, f64, f64)> {
        if self.per_piece.len() != PieceType::COUNT {
            return Vec::new();
        }
        let mut contributors: Vec<(PieceType, f64, f64)> = PieceType::iter()
            .map(|piece| {
                let (mg, eg) = self.average_for_piece(piece);
                (piece, mg, eg)
            })
            .collect();

        contributors.sort_by(|a, b| {
            let am = a.1.abs() + a.2.abs();
            let bm = b.1.abs() + b.2.abs();
            bm.partial_cmp(&am).unwrap_or(std::cmp::Ordering::Equal)
        });

        contributors
            .into_iter()
            .filter(|(_, mg, eg)| (*mg != 0.0) || (*eg != 0.0))
            .take(limit)
            .collect()
    }
}

impl PieceType {
    fn iter() -> impl Iterator<Item = PieceType> {
        [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::King,
            PieceType::PromotedPawn,
            PieceType::PromotedLance,
            PieceType::PromotedKnight,
            PieceType::PromotedSilver,
            PieceType::PromotedBishop,
            PieceType::PromotedRook,
        ]
        .into_iter()
    }
}

impl EvaluationStatistics {
    /// Create a new statistics tracker (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            evaluation_count: 0,
            score_stats: ScoreStatistics::default(),
            phase_stats: PhaseStatistics::default(),
            accuracy_metrics: AccuracyMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            session_start: None,
            telemetry: None,
            pst_stats: PieceSquareStatisticsAggregate::default(),
            collect_position_feature_stats: false,
            position_feature_stats: None,
            tactical_stats: None,
            positional_stats: None,
            king_safety_stats: None,
        }
    }

    /// Enable statistics tracking
    pub fn enable(&mut self) {
        self.enabled = true;
        self.session_start = Some(Instant::now());
    }

    /// Control whether position feature statistics should be collected.
    pub fn set_collect_position_feature_stats(&mut self, collect: bool) {
        self.collect_position_feature_stats = collect;
        if !collect {
            self.position_feature_stats = None;
        }
    }

    /// Record the latest position feature statistics snapshot.
    pub fn record_position_feature_stats(&mut self, stats: PositionFeatureStats) {
        if self.collect_position_feature_stats {
            self.position_feature_stats = Some(stats);
        }
    }

    /// Record the latest positional pattern statistics snapshot.
    pub fn record_positional_stats(&mut self, stats: PositionalStatsSnapshot) {
        self.positional_stats = Some(stats);
    }

    /// Record the latest king safety statistics snapshot.
    pub fn record_king_safety_stats(&mut self, stats: KingSafetyStatsSnapshot) {
        self.king_safety_stats = Some(stats);
    }

    /// Disable statistics tracking
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Record an evaluation
    #[inline]
    pub fn record_evaluation(&mut self, score: i32, phase: i32) {
        if !self.enabled {
            return;
        }

        self.evaluation_count += 1;
        self.score_stats.record(score);
        self.phase_stats.record(phase);
    }

    /// Record phase only
    #[inline]
    pub fn record_phase(&mut self, phase: i32) {
        if !self.enabled {
            return;
        }
        self.phase_stats.record(phase);
    }

    /// Record accuracy (predicted vs actual)
    #[inline]
    pub fn record_accuracy(&mut self, predicted: i32, actual: i32) {
        if !self.enabled {
            return;
        }
        self.accuracy_metrics.record(predicted, actual);
    }

    /// Record performance timing
    #[inline]
    pub fn record_timing(&mut self, duration_ns: u64) {
        if !self.enabled {
            return;
        }
        self.performance_metrics.record_timing(duration_ns);
    }

    /// Update the latest telemetry snapshot captured from the evaluator.
    pub fn update_telemetry(&mut self, telemetry: EvaluationTelemetry) {
        if self.enabled {
            if let Some(ref pst) = telemetry.pst {
                self.pst_stats.record(pst);
            }
            if let Some(ref tactical) = telemetry.tactical {
                self.tactical_stats = Some(tactical.clone());
            }
            if self.collect_position_feature_stats {
                if let Some(ref stats) = telemetry.position_features {
                    self.position_feature_stats = Some(stats.clone());
                }
            }
            if let Some(ref positional) = telemetry.positional {
                self.positional_stats = Some(positional.clone());
            }
            if let Some(ref king_safety) = telemetry.king_safety {
                self.king_safety_stats = Some(king_safety.clone());
            }
        }
        self.telemetry = Some(telemetry);
    }

    /// Access the most recent telemetry snapshot, if any.
    pub fn telemetry(&self) -> Option<&EvaluationTelemetry> {
        self.telemetry.as_ref()
    }

    /// Access the most recent tactical statistics snapshot, if any.
    pub fn tactical_stats(&self) -> Option<&TacticalStatsSnapshot> {
        self.tactical_stats.as_ref()
    }

    /// Access the most recent king safety statistics snapshot, if any.
    pub fn king_safety_stats(&self) -> Option<&KingSafetyStatsSnapshot> {
        self.king_safety_stats.as_ref()
    }

    /// Generate comprehensive report
    pub fn generate_report(&self) -> StatisticsReport {
        let session_duration = self
            .session_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::from_secs(0));

        StatisticsReport {
            enabled: self.enabled,
            evaluation_count: self.evaluation_count,
            score_stats: self.score_stats.clone(),
            phase_stats: self.phase_stats.clone(),
            accuracy_metrics: self.accuracy_metrics.clone(),
            performance_metrics: self.performance_metrics.clone(),
            session_duration_secs: session_duration.as_secs_f64(),
            evaluations_per_second: if session_duration.as_secs_f64() > 0.0 {
                self.evaluation_count as f64 / session_duration.as_secs_f64()
            } else {
                0.0
            },
            telemetry: self.telemetry.clone(),
            pst_stats: self.pst_stats.clone(),
            position_feature_stats: self.position_feature_stats.clone(),
            tactical_stats: self.tactical_stats.clone(),
            positional_stats: self.positional_stats.clone(),
            king_safety_stats: self.king_safety_stats.clone(),
        }
    }

    /// Export statistics to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.evaluation_count = 0;
        self.score_stats = ScoreStatistics::default();
        self.phase_stats = PhaseStatistics::default();
        self.accuracy_metrics = AccuracyMetrics::default();
        self.performance_metrics = PerformanceMetrics::default();
        self.session_start = Some(Instant::now());
        self.telemetry = None;
        self.pst_stats = PieceSquareStatisticsAggregate::default();
        if self.collect_position_feature_stats {
            self.position_feature_stats = None;
        }
        self.tactical_stats = None;
        self.positional_stats = None;
        self.king_safety_stats = None;
    }

    /// Get evaluation count
    pub fn count(&self) -> u64 {
        self.evaluation_count
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Access aggregated PST statistics.
    pub fn pst_statistics(&self) -> &PieceSquareStatisticsAggregate {
        &self.pst_stats
    }
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Score statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreStatistics {
    /// Total score sum
    sum: i64,
    /// Minimum score seen
    min: i32,
    /// Maximum score seen
    max: i32,
    /// Count of evaluations
    count: u64,
    /// Score distribution (bucketed)
    distribution: [u64; 10], // -10K to +10K in 2K buckets
}

impl ScoreStatistics {
    fn record(&mut self, score: i32) {
        self.sum += score as i64;
        self.min = if self.count == 0 {
            score
        } else {
            self.min.min(score)
        };
        self.max = if self.count == 0 {
            score
        } else {
            self.max.max(score)
        };
        self.count += 1;

        // Update distribution
        let bucket = ((score + 10000) / 2000).clamp(0, 9) as usize;
        self.distribution[bucket] += 1;
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

/// Phase statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhaseStatistics {
    /// Total phase sum
    sum: i64,
    /// Phase distribution
    opening_count: u64, // phase >= 192
    middlegame_count: u64, // 64 <= phase < 192
    endgame_count: u64,    // phase < 64
    /// Detailed distribution (26 buckets, 10 phase units each)
    distribution: [u64; 26],
}

impl PhaseStatistics {
    fn record(&mut self, phase: i32) {
        self.sum += phase as i64;

        // Update phase category counts
        if phase >= 192 {
            self.opening_count += 1;
        } else if phase >= 64 {
            self.middlegame_count += 1;
        } else {
            self.endgame_count += 1;
        }

        // Update distribution
        let bucket = (phase / 10).clamp(0, 25) as usize;
        self.distribution[bucket] += 1;
    }

    pub fn average(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            self.sum as f64 / total as f64
        }
    }

    pub fn opening_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.opening_count as f64 / total as f64) * 100.0
        }
    }

    pub fn middlegame_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.middlegame_count as f64 / total as f64) * 100.0
        }
    }

    pub fn endgame_percentage(&self) -> f64 {
        let total = self.opening_count + self.middlegame_count + self.endgame_count;
        if total == 0 {
            0.0
        } else {
            (self.endgame_count as f64 / total as f64) * 100.0
        }
    }
}

/// Accuracy metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// Sum of squared errors
    sum_squared_error: f64,
    /// Sum of absolute errors
    sum_absolute_error: f64,
    /// Count of predictions
    count: u64,
}

impl AccuracyMetrics {
    fn record(&mut self, predicted: i32, actual: i32) {
        let error = (predicted - actual) as f64;
        self.sum_squared_error += error * error;
        self.sum_absolute_error += error.abs();
        self.count += 1;
    }

    pub fn mean_squared_error(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_squared_error / self.count as f64
        }
    }

    pub fn root_mean_squared_error(&self) -> f64 {
        self.mean_squared_error().sqrt()
    }

    pub fn mean_absolute_error(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_absolute_error / self.count as f64
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Sum of timing measurements (nanoseconds)
    total_time_ns: u64,
    /// Count of timing measurements
    timing_count: u64,
    /// Minimum time
    min_time_ns: u64,
    /// Maximum time
    max_time_ns: u64,
}

impl PerformanceMetrics {
    fn record_timing(&mut self, duration_ns: u64) {
        self.total_time_ns += duration_ns;
        self.timing_count += 1;

        if self.timing_count == 1 {
            self.min_time_ns = duration_ns;
            self.max_time_ns = duration_ns;
        } else {
            self.min_time_ns = self.min_time_ns.min(duration_ns);
            self.max_time_ns = self.max_time_ns.max(duration_ns);
        }
    }

    pub fn average_time_ns(&self) -> f64 {
        if self.timing_count == 0 {
            0.0
        } else {
            self.total_time_ns as f64 / self.timing_count as f64
        }
    }

    pub fn average_time_us(&self) -> f64 {
        self.average_time_ns() / 1000.0
    }

    pub fn throughput_per_second(&self) -> f64 {
        if self.average_time_ns() > 0.0 {
            1_000_000_000.0 / self.average_time_ns()
        } else {
            0.0
        }
    }
}

/// Complete statistics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsReport {
    /// Whether statistics were enabled
    pub enabled: bool,
    /// Total evaluation count
    pub evaluation_count: u64,
    /// Score statistics
    pub score_stats: ScoreStatistics,
    /// Phase statistics
    pub phase_stats: PhaseStatistics,
    /// Accuracy metrics
    pub accuracy_metrics: AccuracyMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Session duration in seconds
    pub session_duration_secs: f64,
    /// Evaluations per second
    pub evaluations_per_second: f64,
    /// Latest telemetry snapshot, if available
    pub telemetry: Option<EvaluationTelemetry>,
    /// Aggregated PST statistics
    pub pst_stats: PieceSquareStatisticsAggregate,
    /// Latest position feature statistics snapshot
    pub position_feature_stats: Option<PositionFeatureStats>,
    /// Latest tactical statistics snapshot
    pub tactical_stats: Option<TacticalStatsSnapshot>,
    /// Latest positional statistics snapshot
    pub positional_stats: Option<PositionalStatsSnapshot>,
    /// Latest king safety statistics snapshot
    pub king_safety_stats: Option<KingSafetyStatsSnapshot>,
}

impl std::fmt::Display for StatisticsReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Evaluation Statistics Report")?;
        writeln!(f, "============================")?;
        writeln!(f)?;
        writeln!(f, "Session Overview:")?;
        writeln!(f, "  Total Evaluations: {}", self.evaluation_count)?;
        writeln!(
            f,
            "  Session Duration: {:.2} seconds",
            self.session_duration_secs
        )?;
        writeln!(
            f,
            "  Throughput: {:.0} evals/sec",
            self.evaluations_per_second
        )?;
        writeln!(f)?;
        writeln!(f, "Score Statistics:")?;
        writeln!(f, "  Average Score: {:.2}", self.score_stats.average())?;
        writeln!(f, "  Min Score: {}", self.score_stats.min)?;
        writeln!(f, "  Max Score: {}", self.score_stats.max)?;
        writeln!(f)?;
        writeln!(f, "Phase Distribution:")?;
        writeln!(f, "  Average Phase: {:.2}", self.phase_stats.average())?;
        writeln!(
            f,
            "  Opening (≥192): {:.1}%",
            self.phase_stats.opening_percentage()
        )?;
        writeln!(
            f,
            "  Middlegame (64-191): {:.1}%",
            self.phase_stats.middlegame_percentage()
        )?;
        writeln!(
            f,
            "  Endgame (<64): {:.1}%",
            self.phase_stats.endgame_percentage()
        )?;
        writeln!(f)?;
        writeln!(f, "Accuracy Metrics:")?;
        writeln!(
            f,
            "  Mean Absolute Error: {:.2}",
            self.accuracy_metrics.mean_absolute_error()
        )?;
        writeln!(
            f,
            "  Root Mean Squared Error: {:.2}",
            self.accuracy_metrics.root_mean_squared_error()
        )?;
        writeln!(f)?;
        writeln!(f, "Performance Metrics:")?;
        writeln!(
            f,
            "  Average Time: {:.2} μs",
            self.performance_metrics.average_time_us()
        )?;
        writeln!(f, "  Min Time: {} ns", self.performance_metrics.min_time_ns)?;
        writeln!(f, "  Max Time: {} ns", self.performance_metrics.max_time_ns)?;
        writeln!(
            f,
            "  Throughput: {:.0} evals/sec",
            self.performance_metrics.throughput_per_second()
        )?;
        if self.pst_stats.sample_count() > 0 {
            writeln!(f)?;
            writeln!(f, "PST Aggregates:")?;
            writeln!(f, "  Samples: {}", self.pst_stats.sample_count())?;
            writeln!(
                f,
                "  Avg Total: mg {:.2} eg {:.2}",
                self.pst_stats.average_total_mg(),
                self.pst_stats.average_total_eg()
            )?;
            let top = self.pst_stats.top_contributors(5);
            if !top.is_empty() {
                writeln!(f, "  Top Average Contributors:")?;
                for (piece, mg, eg) in top {
                    writeln!(f, "    {:?}: mg {:.2} eg {:.2}", piece, mg, eg)?;
                }
            }
        }

        if let Some(stats) = &self.position_feature_stats {
            writeln!(f)?;
            writeln!(f, "Position Feature Statistics:")?;
            writeln!(f, "  King Safety Evals: {}", stats.king_safety_evals)?;
            writeln!(f, "  Pawn Structure Evals: {}", stats.pawn_structure_evals)?;
            writeln!(f, "  Mobility Evals: {}", stats.mobility_evals)?;
            writeln!(f, "  Center Control Evals: {}", stats.center_control_evals)?;
            writeln!(f, "  Development Evals: {}", stats.development_evals)?;
        }
        if let Some(stats) = &self.tactical_stats {
            writeln!(f)?;
            writeln!(f, "Tactical Pattern Statistics:")?;
            writeln!(f, "  Evaluations: {}", stats.evaluations)?;
            writeln!(
                f,
                "  Checks (fork/pin/skewer/discovered/knight/back-rank): {}/{}/{}/{}/{}/{}",
                stats.fork_checks,
                stats.pin_checks,
                stats.skewer_checks,
                stats.discovered_checks,
                stats.knight_fork_checks,
                stats.back_rank_checks
            )?;
            writeln!(
                f,
                "  Findings (fork/pin/skewer/discovered/knight/back-rank): {}/{}/{}/{}/{}/{}",
                stats.forks_found,
                stats.pins_found,
                stats.skewers_found,
                stats.discovered_attacks_found,
                stats.knight_forks_found,
                stats.back_rank_threats_found
            )?;
        }
        if let Some(telemetry) = &self.telemetry {
            writeln!(f)?;
            writeln!(f, "Evaluation Telemetry:")?;
            if let Some(tapered) = telemetry.tapered {
                writeln!(
                    f,
                    "  Tapered Phase Calculations: {}",
                    tapered.phase_calculations
                )?;
                writeln!(f, "  Tapered Cache Hits: {}", tapered.cache_hits)?;
                writeln!(
                    f,
                    "  Tapered Cache Hit Rate: {:.2}%",
                    tapered.cache_hit_rate * 100.0
                )?;
                writeln!(
                    f,
                    "  Tapered Interpolations: {}",
                    tapered.total_interpolations
                )?;
            }
            if let Some(phase) = telemetry.phase_transition {
                writeln!(
                    f,
                    "  Phase Transition Interpolations: {}",
                    phase.interpolations
                )?;
            }
            if let Some(performance) = telemetry.performance.as_ref() {
                writeln!(f)?;
                writeln!(f, "  Profiler Snapshot:")?;
                writeln!(
                    f,
                    "    Avg Evaluation: {:.2} ns",
                    performance.avg_evaluation_ns
                )?;
                writeln!(
                    f,
                    "    Avg Phase Calc: {:.2} ns",
                    performance.avg_phase_calc_ns
                )?;
                writeln!(
                    f,
                    "    Avg Interpolation: {:.2} ns",
                    performance.avg_interpolation_ns
                )?;
            }
            if let Some(pst) = telemetry.pst.as_ref() {
                writeln!(f)?;
                writeln!(
                    f,
                    "  PST Contribution: total mg {} | total eg {}",
                    pst.total_mg, pst.total_eg
                )?;
                if !pst.per_piece.is_empty() {
                    let mut contributors = pst.per_piece.clone();
                    contributors
                        .sort_by(|a, b| (b.mg.abs() + b.eg.abs()).cmp(&(a.mg.abs() + a.eg.abs())));
                    writeln!(f, "    Top Contributors:")?;
                    for entry in contributors.iter().take(5) {
                        writeln!(
                            f,
                            "      {:?}: mg {} eg {}",
                            entry.piece, entry.mg, entry.eg
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_creation() {
        let stats = EvaluationStatistics::new();
        assert!(!stats.is_enabled());
        assert_eq!(stats.count(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut stats = EvaluationStatistics::new();

        assert!(!stats.is_enabled());

        stats.enable();
        assert!(stats.is_enabled());

        stats.disable();
        assert!(!stats.is_enabled());
    }

    #[test]
    fn test_record_evaluation() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_evaluation(200, 180);
        stats.record_evaluation(100, 128);

        assert_eq!(stats.count(), 3);
    }

    #[test]
    fn test_score_statistics() {
        let mut score_stats = ScoreStatistics::default();

        score_stats.record(100);
        score_stats.record(200);
        score_stats.record(150);

        assert_eq!(score_stats.average(), 150.0);
        assert_eq!(score_stats.min, 100);
        assert_eq!(score_stats.max, 200);
    }

    #[test]
    fn test_phase_statistics() {
        let mut phase_stats = PhaseStatistics::default();

        // Opening
        phase_stats.record(256);
        phase_stats.record(200);

        // Middlegame
        phase_stats.record(128);
        phase_stats.record(100);

        // Endgame
        phase_stats.record(32);
        phase_stats.record(10);

        assert_eq!(phase_stats.opening_count, 2);
        assert_eq!(phase_stats.middlegame_count, 2);
        assert_eq!(phase_stats.endgame_count, 2);
        assert_eq!(phase_stats.opening_percentage(), 100.0 / 3.0);
    }

    #[test]
    fn test_accuracy_metrics() {
        let mut accuracy = AccuracyMetrics::default();

        accuracy.record(100, 110); // Error: -10
        accuracy.record(200, 190); // Error: +10
        accuracy.record(150, 150); // Error: 0

        assert_eq!(accuracy.mean_absolute_error(), 20.0 / 3.0);
        assert!((accuracy.mean_squared_error() - 200.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn test_performance_metrics() {
        let mut perf = PerformanceMetrics::default();

        perf.record_timing(1000);
        perf.record_timing(1500);
        perf.record_timing(1200);

        assert_eq!(perf.average_time_ns(), 1233.3333333333333);
        assert_eq!(perf.min_time_ns, 1000);
        assert_eq!(perf.max_time_ns, 1500);
    }

    #[test]
    fn test_generate_report() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_accuracy(150, 145);
        stats.record_timing(1000);

        let report = stats.generate_report();

        assert_eq!(report.evaluation_count, 1);
        assert!(report.enabled);
    }

    #[test]
    fn test_export_json() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);

        let json = stats.export_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("evaluation_count"));
    }

    #[test]
    fn test_reset() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        assert_eq!(stats.count(), 1);

        stats.reset();
        assert_eq!(stats.count(), 0);
    }

    #[test]
    fn test_disabled_no_recording() {
        let mut stats = EvaluationStatistics::new();
        // Not enabled

        stats.record_evaluation(150, 200);
        assert_eq!(stats.count(), 0); // Should not record
    }

    #[test]
    fn test_phase_percentages() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        // Record 2 opening, 6 middlegame, 2 endgame
        stats.record_phase(256);
        stats.record_phase(200);
        stats.record_phase(128);
        stats.record_phase(100);
        stats.record_phase(90);
        stats.record_phase(80);
        stats.record_phase(70);
        stats.record_phase(65);
        stats.record_phase(32);
        stats.record_phase(10);

        let report = stats.generate_report();
        assert_eq!(report.phase_stats.opening_percentage(), 20.0);
        assert_eq!(report.phase_stats.middlegame_percentage(), 60.0);
        assert_eq!(report.phase_stats.endgame_percentage(), 20.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let perf = PerformanceMetrics {
            total_time_ns: 1000,
            timing_count: 1,
            min_time_ns: 1000,
            max_time_ns: 1000,
        };

        let throughput = perf.throughput_per_second();
        assert_eq!(throughput, 1_000_000.0); // 1M evals/sec at 1μs each
    }

    #[test]
    fn test_report_display() {
        let mut stats = EvaluationStatistics::new();
        stats.enable();

        stats.record_evaluation(150, 200);
        stats.record_accuracy(150, 145);

        let report = stats.generate_report();
        let display = format!("{}", report);

        assert!(display.contains("Evaluation Statistics Report"));
        assert!(display.contains("Total Evaluations"));
        assert!(display.contains("Phase Distribution"));
    }
}
