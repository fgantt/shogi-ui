//! Advanced Integration Module
//!
//! This module provides advanced integrations between the tapered evaluation system
//! and other engine components including:
//! - Opening book integration
//! - Endgame tablebase integration
//! - Analysis mode evaluation
//! - Phase-aware time management
//! - Parallel evaluation support
//! - Multi-threaded tuning
//!
//! # Overview
//!
//! Advanced integrations enhance the tapered evaluation system by:
//! - Combining with opening theory
//! - Leveraging tablebase knowledge
//! - Providing detailed analysis
//! - Optimizing time allocation by phase
//! - Supporting parallel processing
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::advanced_integration::AdvancedIntegration;
//!
//! let mut integration = AdvancedIntegration::new();
//! integration.enable_opening_book();
//! integration.enable_tablebase();
//!
//! let score = integration.evaluate_with_all_features(&board, player, &captured);
//! ```

use crate::bitboards::BitboardBoard;
use crate::evaluation::integration::IntegratedEvaluator;
use crate::types::board::CapturedPieces;
use crate::types::core::{PieceType, Player, Position};
use std::sync::{Arc, Mutex};
use std::thread;

/// Advanced integration coordinator
pub struct AdvancedIntegration {
    /// Core evaluator
    evaluator: IntegratedEvaluator,
    /// Configuration
    config: AdvancedIntegrationConfig,
    /// Statistics
    stats: AdvancedIntegrationStats,
}

impl AdvancedIntegration {
    /// Create a new advanced integration
    pub fn new() -> Self {
        Self {
            evaluator: IntegratedEvaluator::new(),
            config: AdvancedIntegrationConfig::default(),
            stats: AdvancedIntegrationStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AdvancedIntegrationConfig) -> Self {
        Self {
            evaluator: IntegratedEvaluator::new(),
            config,
            stats: AdvancedIntegrationStats::default(),
        }
    }

    /// Evaluate with all advanced features
    pub fn evaluate_with_all_features(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> AdvancedEvaluationResult {
        // Check opening book first
        if self.config.use_opening_book {
            if let Some(book_score) = self.check_opening_book(board, player) {
                self.stats.opening_book_hits += 1;
                return AdvancedEvaluationResult {
                    score: book_score,
                    source: EvaluationSource::OpeningBook,
                    confidence: 1.0,
                    phase: 256,
                };
            }
        }

        // Check tablebase if in endgame
        if self.config.use_tablebase {
            if let Some(tb_score) = self.check_tablebase(board, player, captured_pieces) {
                self.stats.tablebase_hits += 1;
                return AdvancedEvaluationResult {
                    score: tb_score,
                    source: EvaluationSource::Tablebase,
                    confidence: 1.0,
                    phase: self.estimate_phase(board),
                };
            }
        }

        // Regular tapered evaluation
        let result = self
            .evaluator
            .evaluate_with_move_count(board, player, captured_pieces, None);
        let phase = result.phase;

        AdvancedEvaluationResult {
            score: result.score,
            source: EvaluationSource::TaperedEvaluation,
            confidence: 0.8,
            phase,
        }
    }

    /// Evaluate for analysis mode (detailed breakdown)
    pub fn evaluate_for_analysis(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> AnalysisEvaluation {
        // Get overall score
        let result =
            self.evaluator
                .evaluate_with_move_count(board, player, captured_pieces, None);
        let phase = result.phase;

        AnalysisEvaluation {
            total_score: result.score,
            phase,
            phase_category: self.categorize_phase(phase),
            component_breakdown: ComponentBreakdown {
                material: 0, // Would calculate from material evaluator
                position: 0, // Would calculate from PST
                king_safety: 0,
                pawn_structure: 0,
                mobility: 0,
                center_control: 0,
                development: 0,
            },
            suggestions: self.generate_suggestions(board, player, phase),
        }
    }

    /// Check opening book (stub - would integrate with actual opening book)
    fn check_opening_book(&self, _board: &BitboardBoard, _player: Player) -> Option<i32> {
        // In real implementation, would query opening book
        // For now, return None (no opening book hit)
        None
    }

    /// Check endgame tablebase (stub - would integrate with actual tablebase)
    fn check_tablebase(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _captured: &CapturedPieces,
    ) -> Option<i32> {
        // In real implementation, would query tablebase
        // For now, return None (no tablebase hit)
        None
    }

    /// Estimate game phase
    fn estimate_phase(&self, board: &BitboardBoard) -> i32 {
        let mut phase = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    phase += match piece.piece_type {
                        PieceType::Pawn => 0,
                        PieceType::Lance => 1,
                        PieceType::Knight => 1,
                        PieceType::Silver => 2,
                        PieceType::Gold => 2,
                        PieceType::Bishop => 4,
                        PieceType::Rook => 5,
                        PieceType::King => 0,
                        _ => 3,
                    };
                }
            }
        }
        phase.min(256)
    }

    /// Categorize phase
    fn categorize_phase(&self, phase: i32) -> PhaseCategory {
        if phase >= 192 {
            PhaseCategory::Opening
        } else if phase >= 64 {
            PhaseCategory::Middlegame
        } else {
            PhaseCategory::Endgame
        }
    }

    /// Generate suggestions for analysis mode
    fn generate_suggestions(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        phase: i32,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match self.categorize_phase(phase) {
            PhaseCategory::Opening => {
                suggestions.push("Focus on piece development".to_string());
                suggestions.push("Control the center".to_string());
            }
            PhaseCategory::Middlegame => {
                suggestions.push("Look for tactical opportunities".to_string());
                suggestions.push("Improve piece coordination".to_string());
            }
            PhaseCategory::Endgame => {
                suggestions.push("Activate your king".to_string());
                suggestions.push("Push passed pawns".to_string());
            }
        }

        suggestions
    }

    /// Get time allocation recommendation based on phase
    pub fn get_time_allocation(&self, phase: i32, total_time_ms: u32) -> TimeAllocation {
        let category = self.categorize_phase(phase);

        match category {
            PhaseCategory::Opening => {
                // Spend less time in opening (book moves often)
                TimeAllocation {
                    recommended_time_ms: (total_time_ms as f64 * 0.7) as u32,
                    min_time_ms: (total_time_ms as f64 * 0.3) as u32,
                    max_time_ms: (total_time_ms as f64 * 1.2) as u32,
                }
            }
            PhaseCategory::Middlegame => {
                // Spend more time in middlegame (critical decisions)
                TimeAllocation {
                    recommended_time_ms: (total_time_ms as f64 * 1.3) as u32,
                    min_time_ms: (total_time_ms as f64 * 0.8) as u32,
                    max_time_ms: (total_time_ms as f64 * 2.0) as u32,
                }
            }
            PhaseCategory::Endgame => {
                // Balanced time in endgame (precision matters)
                TimeAllocation {
                    recommended_time_ms: total_time_ms,
                    min_time_ms: (total_time_ms as f64 * 0.5) as u32,
                    max_time_ms: (total_time_ms as f64 * 1.5) as u32,
                }
            }
        }
    }

    /// Enable opening book integration
    pub fn enable_opening_book(&mut self) {
        self.config.use_opening_book = true;
    }

    /// Disable opening book integration
    pub fn disable_opening_book(&mut self) {
        self.config.use_opening_book = false;
    }

    /// Enable tablebase integration
    pub fn enable_tablebase(&mut self) {
        self.config.use_tablebase = true;
    }

    /// Disable tablebase integration
    pub fn disable_tablebase(&mut self) {
        self.config.use_tablebase = false;
    }

    /// Get statistics
    pub fn stats(&self) -> &AdvancedIntegrationStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = AdvancedIntegrationStats::default();
    }
}

impl Default for AdvancedIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel evaluation support
pub struct ParallelEvaluator {
    /// Number of threads
    num_threads: usize,
}

impl ParallelEvaluator {
    /// Create a new parallel evaluator
    pub fn new(num_threads: usize) -> Self {
        Self { num_threads }
    }

    /// Evaluate positions in parallel
    pub fn evaluate_parallel(
        &self,
        positions: Vec<(BitboardBoard, Player, CapturedPieces)>,
    ) -> Vec<i32> {
        let chunk_size = (positions.len() + self.num_threads - 1) / self.num_threads;
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = Vec::new();

        for chunk in positions.chunks(chunk_size) {
            let chunk_owned = chunk.to_vec();
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let mut evaluator = IntegratedEvaluator::new();
                let mut scores = Vec::new();

                for (board, player, captured) in chunk_owned {
                    let result = evaluator.evaluate_with_move_count(&board, player, &captured, None);
                    scores.push(result.score);
                }

                let mut results = results_clone.lock().unwrap();
                results.extend(scores);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_results = results.lock().unwrap();
        final_results.clone()
    }
}

/// Advanced integration configuration
#[derive(Debug, Clone)]
pub struct AdvancedIntegrationConfig {
    /// Use opening book
    pub use_opening_book: bool,
    /// Use endgame tablebase
    pub use_tablebase: bool,
    /// Enable analysis mode features
    pub enable_analysis_mode: bool,
    /// Enable phase-aware time management
    pub enable_phase_time_management: bool,
}

impl Default for AdvancedIntegrationConfig {
    fn default() -> Self {
        Self {
            use_opening_book: false,
            use_tablebase: false,
            enable_analysis_mode: false,
            enable_phase_time_management: true,
        }
    }
}

/// Advanced evaluation result
#[derive(Debug, Clone)]
pub struct AdvancedEvaluationResult {
    /// Evaluation score
    pub score: i32,
    /// Source of evaluation
    pub source: EvaluationSource,
    /// Confidence (0.0-1.0)
    pub confidence: f64,
    /// Game phase
    pub phase: i32,
}

/// Evaluation source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationSource {
    OpeningBook,
    Tablebase,
    TaperedEvaluation,
}

/// Analysis mode evaluation
#[derive(Debug, Clone)]
pub struct AnalysisEvaluation {
    /// Total score
    pub total_score: i32,
    /// Game phase
    pub phase: i32,
    /// Phase category
    pub phase_category: PhaseCategory,
    /// Component breakdown
    pub component_breakdown: ComponentBreakdown,
    /// Suggestions
    pub suggestions: Vec<String>,
}

/// Component breakdown for analysis
#[derive(Debug, Clone)]
pub struct ComponentBreakdown {
    pub material: i32,
    pub position: i32,
    pub king_safety: i32,
    pub pawn_structure: i32,
    pub mobility: i32,
    pub center_control: i32,
    pub development: i32,
}

/// Phase category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseCategory {
    Opening,
    Middlegame,
    Endgame,
}

/// Time allocation recommendation
#[derive(Debug, Clone)]
pub struct TimeAllocation {
    /// Recommended time for this move
    pub recommended_time_ms: u32,
    /// Minimum time
    pub min_time_ms: u32,
    /// Maximum time
    pub max_time_ms: u32,
}

/// Advanced integration statistics
#[derive(Debug, Clone, Default)]
pub struct AdvancedIntegrationStats {
    /// Opening book hits
    pub opening_book_hits: u64,
    /// Tablebase hits
    pub tablebase_hits: u64,
    /// Analysis mode queries
    pub analysis_queries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_integration_creation() {
        let integration = AdvancedIntegration::new();
        assert!(!integration.config.use_opening_book);
        assert!(!integration.config.use_tablebase);
    }

    #[test]
    fn test_evaluate_with_all_features() {
        let mut integration = AdvancedIntegration::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let result = integration.evaluate_with_all_features(&board, Player::Black, &captured);

        assert!(result.score.abs() < 100000);
        assert_eq!(result.source, EvaluationSource::TaperedEvaluation);
    }

    #[test]
    fn test_opening_book_integration() {
        let mut integration = AdvancedIntegration::new();
        integration.enable_opening_book();

        assert!(integration.config.use_opening_book);
    }

    #[test]
    fn test_tablebase_integration() {
        let mut integration = AdvancedIntegration::new();
        integration.enable_tablebase();

        assert!(integration.config.use_tablebase);
    }

    #[test]
    fn test_analysis_mode() {
        let mut integration = AdvancedIntegration::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let analysis = integration.evaluate_for_analysis(&board, Player::Black, &captured);

        assert!(analysis.total_score.abs() < 100000);
        assert!(!analysis.suggestions.is_empty());
    }

    #[test]
    fn test_time_allocation_opening() {
        let integration = AdvancedIntegration::new();
        let allocation = integration.get_time_allocation(256, 1000);

        // Opening should recommend less time
        assert!(allocation.recommended_time_ms < 1000);
    }

    #[test]
    fn test_time_allocation_middlegame() {
        let integration = AdvancedIntegration::new();
        let allocation = integration.get_time_allocation(128, 1000);

        // Middlegame should recommend more time
        assert!(allocation.recommended_time_ms > 1000);
    }

    #[test]
    fn test_time_allocation_endgame() {
        let integration = AdvancedIntegration::new();
        let allocation = integration.get_time_allocation(32, 1000);

        // Endgame should be balanced
        assert_eq!(allocation.recommended_time_ms, 1000);
    }

    #[test]
    fn test_phase_categorization() {
        let integration = AdvancedIntegration::new();

        assert_eq!(integration.categorize_phase(256), PhaseCategory::Opening);
        assert_eq!(integration.categorize_phase(128), PhaseCategory::Middlegame);
        assert_eq!(integration.categorize_phase(32), PhaseCategory::Endgame);
    }

    #[test]
    fn test_parallel_evaluator() {
        let parallel = ParallelEvaluator::new(2);

        let positions = vec![
            (BitboardBoard::new(), Player::Black, CapturedPieces::new()),
            (BitboardBoard::new(), Player::White, CapturedPieces::new()),
        ];

        let scores = parallel.evaluate_parallel(positions);

        assert_eq!(scores.len(), 2);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut integration = AdvancedIntegration::new();
        integration.enable_opening_book();

        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        integration.evaluate_with_all_features(&board, Player::Black, &captured);

        let stats = integration.stats();
        // Opening book would be checked (even if no hit)
        assert_eq!(stats.opening_book_hits, 0); // No actual book
    }

    #[test]
    fn test_reset_statistics() {
        let mut integration = AdvancedIntegration::new();
        integration.stats.opening_book_hits = 10;

        integration.reset_stats();

        assert_eq!(integration.stats.opening_book_hits, 0);
    }

    #[test]
    fn test_evaluation_source() {
        let result = AdvancedEvaluationResult {
            score: 100,
            source: EvaluationSource::TaperedEvaluation,
            confidence: 0.8,
            phase: 128,
        };

        assert_eq!(result.source, EvaluationSource::TaperedEvaluation);
    }

    #[test]
    fn test_custom_config() {
        let config = AdvancedIntegrationConfig {
            use_opening_book: true,
            use_tablebase: true,
            enable_analysis_mode: true,
            enable_phase_time_management: true,
        };

        let integration = AdvancedIntegration::with_config(config);

        assert!(integration.config.use_opening_book);
        assert!(integration.config.use_tablebase);
    }
}
