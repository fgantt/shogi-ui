//! Search-Related Types
//!
//! This module contains all types related to search algorithms: configurations, statistics,
//! enums, and helper types for quiescence search, null-move pruning, late move reductions,
//! internal iterative deepening, aspiration windows, time management, and search state.
//!
//! Extracted from `types.rs` as part of Task 1.0: File Modularization and Structure Improvements.
//!
//! This is a large module (~3000+ lines) that consolidates all search-related types from
//! the original 10,482-line types.rs file for better organization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import from sibling modules
use super::core::{Move, Position};
use super::board::GamePhase;

// ============================================================================
// Transposition Table Types (used by search)
// ============================================================================

/// Transposition table entry flag
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

impl TranspositionFlag {
    /// Get a string representation of the flag
    pub fn to_string(&self) -> &'static str {
        match self {
            TranspositionFlag::Exact => "Exact",
            TranspositionFlag::LowerBound => "LowerBound",
            TranspositionFlag::UpperBound => "UpperBound",
        }
    }
}

/// Source of transposition table entry for priority management
/// Used to prevent shallow auxiliary search entries from overwriting deeper main search entries
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntrySource {
    /// Entry from main search path (highest priority)
    MainSearch,
    /// Entry from null move pruning search (lower priority)
    NullMoveSearch,
    /// Entry from internal iterative deepening search (lower priority)
    IIDSearch,
    /// Entry from quiescence search (lower priority)
    QuiescenceSearch,
    /// Entry seeded from opening book prefill (lowest intrinsic priority)
    OpeningBook,
}

impl EntrySource {
    /// Convert the entry source into its compact discriminant form.
    pub fn to_discriminant(self) -> u32 {
        self as u32
    }

    /// Reconstruct an entry source from a compact discriminant value.
    /// Falls back to `EntrySource::MainSearch` for unknown values.
    pub fn from_discriminant(value: u32) -> Self {
        match value {
            0 => EntrySource::MainSearch,
            1 => EntrySource::NullMoveSearch,
            2 => EntrySource::IIDSearch,
            3 => EntrySource::QuiescenceSearch,
            4 => EntrySource::OpeningBook,
            _ => EntrySource::MainSearch,
        }
    }
}

/// Transposition table replacement policy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TTReplacementPolicy {
    Simple,         // Simple cleanup: remove half entries arbitrarily (original behavior)
    LRU,            // Least Recently Used: prefer keeping recently accessed entries
    DepthPreferred, // Prefer keeping entries with deeper depth
    Hybrid,         // Hybrid: combine LRU and depth-preferred
}

impl Default for TTReplacementPolicy {
    fn default() -> Self {
        TTReplacementPolicy::DepthPreferred // Default to depth-preferred for better tactical accuracy
    }
}

/// Transposition table entry specifically for quiescence search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiescenceEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
    pub hash_key: u64,
    /// For LRU tracking - number of times this entry was accessed
    pub access_count: u64,
    /// For LRU tracking - age when last accessed
    pub last_access_age: u64,
    /// Cached stand-pat evaluation (optional, not all entries have it)
    pub stand_pat_score: Option<i32>,
}

// ============================================================================
// Position Classification and Complexity Types
// ============================================================================

/// Position classification for adaptive reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionClassification {
    Tactical, // Position with high tactical activity (many cutoffs)
    Quiet,    // Position with low tactical activity (few cutoffs)
    Neutral,  // Position with moderate tactical activity or insufficient data
}

/// Position complexity levels for adaptive LMR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionComplexity {
    Low,
    Medium,
    High,
    Unknown,
}

/// Move type classification for LMR decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Check,
    Capture,
    Promotion,
    Killer,
    TranspositionTable,
    Escape,
    Center,
    Quiet,
}

// ============================================================================
// Quiescence Search Types
// ============================================================================

/// Configuration for quiescence search parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuiescenceConfig {
    pub max_depth: u8,                              // Maximum quiescence depth
    pub enable_delta_pruning: bool,                 // Enable delta pruning
    pub enable_futility_pruning: bool,              // Enable futility pruning
    pub enable_selective_extensions: bool,          // Enable selective extensions
    pub enable_tt: bool,                            // Enable transposition table
    pub enable_adaptive_pruning: bool, // Enable adaptive pruning (adjusts margins based on depth/move count)
    pub futility_margin: i32,          // Futility pruning margin
    pub delta_margin: i32,             // Delta pruning margin
    pub high_value_capture_threshold: i32, // Threshold for high-value captures (excluded from futility pruning)
    pub tt_size_mb: usize,                 // Quiescence TT size in MB
    pub tt_cleanup_threshold: usize,       // Threshold for TT cleanup
    pub tt_replacement_policy: TTReplacementPolicy, // Replacement policy for TT cleanup
}

impl Default for QuiescenceConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            enable_delta_pruning: true,
            enable_futility_pruning: true,
            enable_selective_extensions: true,
            enable_tt: true,
            enable_adaptive_pruning: true, // Adaptive pruning enabled by default
            futility_margin: 200,
            delta_margin: 100,
            high_value_capture_threshold: 200, // High-value captures (200+ centipawns) excluded from futility pruning
            tt_size_mb: 4,                     // 4MB for quiescence TT
            tt_cleanup_threshold: 10000,       // Clean up when TT has 10k entries
            tt_replacement_policy: TTReplacementPolicy::DepthPreferred, // Default to depth-preferred
        }
    }
}

impl QuiescenceConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.max_depth == 0 {
            return Err("max_depth must be greater than 0".to_string());
        }
        if self.max_depth > 20 {
            return Err("max_depth should not exceed 20 for performance reasons".to_string());
        }
        if self.futility_margin < 0 {
            return Err("futility_margin must be non-negative".to_string());
        }
        if self.futility_margin > 1000 {
            return Err("futility_margin should not exceed 1000".to_string());
        }
        if self.delta_margin < 0 {
            return Err("delta_margin must be non-negative".to_string());
        }
        if self.delta_margin > 1000 {
            return Err("delta_margin should not exceed 1000".to_string());
        }
        if self.tt_size_mb == 0 {
            return Err("tt_size_mb must be greater than 0".to_string());
        }
        if self.tt_size_mb > 1024 {
            return Err("tt_size_mb should not exceed 1024MB".to_string());
        }
        if self.tt_cleanup_threshold == 0 {
            return Err("tt_cleanup_threshold must be greater than 0".to_string());
        }
        if self.tt_cleanup_threshold > 1000000 {
            return Err("tt_cleanup_threshold should not exceed 1,000,000".to_string());
        }
        if self.high_value_capture_threshold < 0 {
            return Err("high_value_capture_threshold must be non-negative".to_string());
        }
        if self.high_value_capture_threshold > 1000 {
            return Err("high_value_capture_threshold should not exceed 1000".to_string());
        }
        Ok(())
    }

    /// Create a new validated configuration
    pub fn new_validated(
        max_depth: u8,
        enable_delta_pruning: bool,
        enable_futility_pruning: bool,
        enable_selective_extensions: bool,
        enable_tt: bool,
        tt_size_mb: usize,
        tt_cleanup_threshold: usize,
        futility_margin: i32,
        delta_margin: i32,
        high_value_capture_threshold: i32,
    ) -> Result<Self, String> {
        let config = Self {
            max_depth,
            enable_delta_pruning,
            enable_futility_pruning,
            enable_selective_extensions,
            enable_tt,
            enable_adaptive_pruning: true, // Default value
            futility_margin,
            delta_margin,
            high_value_capture_threshold,
            tt_size_mb,
            tt_cleanup_threshold,
            tt_replacement_policy: TTReplacementPolicy::DepthPreferred, // Default value
        };
        config.validate()?;
        Ok(config)
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "QuiescenceConfig: depth={}, delta_pruning={}, futility_pruning={}, extensions={}, tt={}, tt_size={}MB, cleanup_threshold={}",
            self.max_depth,
            self.enable_delta_pruning,
            self.enable_futility_pruning,
            self.enable_selective_extensions,
            self.enable_tt,
            self.tt_size_mb,
            self.tt_cleanup_threshold
        )
    }
}

/// Performance statistics for quiescence search
#[derive(Debug, Clone, Default)]
pub struct QuiescenceStats {
    pub nodes_searched: u64,
    pub delta_prunes: u64,
    pub futility_prunes: u64,
    pub extensions: u64,
    pub tt_hits: u64,
    pub tt_misses: u64,
    pub moves_ordered: u64,
    pub check_moves_found: u64,
    pub capture_moves_found: u64,
    pub promotion_moves_found: u64,
    pub checks_excluded_from_futility: u64, // Checks excluded from futility pruning
    pub high_value_captures_excluded_from_futility: u64, // High-value captures excluded from futility pruning
    pub move_ordering_cutoffs: u64, // Number of beta cutoffs from move ordering
    pub move_ordering_total_moves: u64, // Total moves ordered
    pub move_ordering_first_move_cutoffs: u64, // Cutoffs from first move in ordering
    pub move_ordering_second_move_cutoffs: u64, // Cutoffs from second move in ordering
    pub stand_pat_tt_hits: u64,     // Number of times stand-pat was retrieved from TT
    pub stand_pat_tt_misses: u64,   // Number of times stand-pat was not found in TT
}

impl QuiescenceStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = QuiescenceStats::default();
    }

    /// Get the total number of pruning operations
    pub fn total_prunes(&self) -> u64 {
        self.delta_prunes + self.futility_prunes
    }

    /// Get the pruning efficiency as a percentage
    pub fn pruning_efficiency(&self) -> f64 {
        if self.nodes_searched == 0 {
            return 0.0;
        }
        (self.total_prunes() as f64 / self.nodes_searched as f64) * 100.0
    }

    /// Get a summary string of the statistics
    pub fn summary(&self) -> String {
        format!(
            "QuiescenceStats: nodes={}, prunes={}, extensions={}, tt_hits={}, tt_misses={}",
            self.nodes_searched,
            self.total_prunes(),
            self.extensions,
            self.tt_hits,
            self.tt_misses
        )
    }

    /// Get the transposition table hit rate as a percentage
    pub fn tt_hit_rate(&self) -> f64 {
        let total_tt_attempts = self.tt_hits + self.tt_misses;
        if total_tt_attempts == 0 {
            return 0.0;
        }
        (self.tt_hits as f64 / total_tt_attempts as f64) * 100.0
    }

    /// Get the extension rate as a percentage
    pub fn extension_rate(&self) -> f64 {
        if self.nodes_searched == 0 {
            return 0.0;
        }
        (self.extensions as f64 / self.nodes_searched as f64) * 100.0
    }

    /// Generate a performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Quiescence Performance: {} nodes, {:.1}% pruning, {:.1}% extensions, {:.1}% TT hit rate",
            self.nodes_searched,
            self.pruning_efficiency(),
            self.extension_rate(),
            self.tt_hit_rate()
        )
    }
}

// ============================================================================
// Null Move Pruning Types
// ============================================================================

/// Dynamic reduction formula options for null move pruning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DynamicReductionFormula {
    /// Static reduction: always use reduction_factor
    Static,
    /// Linear reduction: R = 2 + depth / 6 (integer division, creates steps)
    Linear,
    /// Smooth reduction: R = 2 + (depth / 6.0).round() (floating-point with rounding for smoother scaling)
    Smooth,
}

impl Default for DynamicReductionFormula {
    fn default() -> Self {
        DynamicReductionFormula::Linear
    }
}

impl DynamicReductionFormula {
    /// Calculate reduction value for given depth and base reduction
    pub fn calculate_reduction(&self, depth: u8, base_reduction: u8) -> u8 {
        match self {
            DynamicReductionFormula::Static => base_reduction,
            DynamicReductionFormula::Linear => {
                // Linear: R = base_reduction + depth / 6
                base_reduction + (depth / 6)
            }
            DynamicReductionFormula::Smooth => {
                // Smooth: R = base_reduction + (depth / 6.0).round()
                let reduction_add = (depth as f32 / 6.0).round() as u8;
                base_reduction + reduction_add
            }
        }
    }
}

/// Advanced reduction strategies for Null Move Pruning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NullMoveReductionStrategy {
    /// Static reduction: Always use base `reduction_factor` (R = reduction_factor)
    Static,
    /// Dynamic reduction: Use `dynamic_reduction_formula` (Linear/Smooth scaling)
    Dynamic,
    /// Depth-based reduction: Reduction varies by depth (smaller at shallow depths, larger at deep depths)
    DepthBased,
    /// Material-based reduction: Reduction adjusted by material count (fewer pieces = smaller reduction)
    MaterialBased,
    /// Position-type-based reduction: Different reduction for opening/middlegame/endgame
    PositionTypeBased,
}

impl Default for NullMoveReductionStrategy {
    fn default() -> Self {
        NullMoveReductionStrategy::Dynamic
    }
}

impl NullMoveReductionStrategy {
    /// Get a string representation of the strategy
    pub fn to_string(&self) -> &'static str {
        match self {
            NullMoveReductionStrategy::Static => "Static",
            NullMoveReductionStrategy::Dynamic => "Dynamic",
            NullMoveReductionStrategy::DepthBased => "DepthBased",
            NullMoveReductionStrategy::MaterialBased => "MaterialBased",
            NullMoveReductionStrategy::PositionTypeBased => "PositionTypeBased",
        }
    }
}

/// Preset configurations for Null Move Pruning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NullMovePreset {
    /// Conservative preset: Higher safety margins, lower reduction, stricter endgame detection
    Conservative,
    /// Aggressive preset: Lower safety margins, higher reduction, relaxed endgame detection
    Aggressive,
    /// Balanced preset: Default values optimized for general play
    Balanced,
}

impl NullMovePreset {
    /// Get a string representation of the preset
    pub fn to_string(&self) -> &'static str {
        match self {
            NullMovePreset::Conservative => "Conservative",
            NullMovePreset::Aggressive => "Aggressive",
            NullMovePreset::Balanced => "Balanced",
        }
    }

    /// Parse a preset from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "conservative" => Some(NullMovePreset::Conservative),
            "aggressive" => Some(NullMovePreset::Aggressive),
            "balanced" => Some(NullMovePreset::Balanced),
            _ => None,
        }
    }
}

// Note: NullMoveConfig and NullMoveStats are very large structs (200+ lines each)
// They will be included in a follow-up commit due to size constraints.
// For now, we'll add placeholders and complete the extraction incrementally.

/// Configuration for null move pruning parameters
/// 
/// This is a large struct with many fields. The full implementation will be
/// added in a follow-up commit to manage file size.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NullMoveConfig {
    pub enabled: bool,
    pub min_depth: u8,
    pub reduction_factor: u8,
    pub max_pieces_threshold: u8,
    pub enable_dynamic_reduction: bool,
    pub enable_endgame_detection: bool,
    pub verification_margin: i32,
    pub dynamic_reduction_formula: DynamicReductionFormula,
    pub enable_mate_threat_detection: bool,
    pub mate_threat_margin: i32,
    pub enable_endgame_type_detection: bool,
    pub material_endgame_threshold: u8,
    pub king_activity_threshold: u8,
    pub zugzwang_threshold: u8,
    pub preset: Option<NullMovePreset>,
    pub reduction_strategy: NullMoveReductionStrategy,
    // Advanced reduction strategy parameters
    pub depth_scaling_factor: u8,
    pub min_depth_for_scaling: u8,
    pub material_adjustment_factor: u8,
    pub piece_count_threshold: u8,
    pub threshold_step: u8,
    pub opening_reduction_factor: u8,
    pub middlegame_reduction_factor: u8,
    pub endgame_reduction_factor: u8,
    pub enable_per_depth_reduction: bool,
    pub reduction_factor_by_depth: HashMap<u8, u8>,
    pub enable_per_position_type_threshold: bool,
    pub opening_pieces_threshold: u8,
    pub middlegame_pieces_threshold: u8,
    pub endgame_pieces_threshold: u8,
}

impl Default for NullMoveConfig {
    fn default() -> Self {
        // Use balanced preset as default
        Self::from_preset(NullMovePreset::Balanced)
    }
}

impl NullMoveConfig {
    /// Create a configuration from a preset
    pub fn from_preset(preset: NullMovePreset) -> Self {
        match preset {
            NullMovePreset::Conservative => Self {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 14,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 400,
                dynamic_reduction_formula: DynamicReductionFormula::Linear,
                enable_mate_threat_detection: true,
                mate_threat_margin: 600,
                enable_endgame_type_detection: true,
                material_endgame_threshold: 14,
                king_activity_threshold: 10,
                zugzwang_threshold: 8,
                preset: Some(NullMovePreset::Conservative),
                reduction_strategy: NullMoveReductionStrategy::Dynamic,
                depth_scaling_factor: 1,
                min_depth_for_scaling: 4,
                material_adjustment_factor: 1,
                piece_count_threshold: 20,
                threshold_step: 4,
                opening_reduction_factor: 2,
                middlegame_reduction_factor: 2,
                endgame_reduction_factor: 1,
                enable_per_depth_reduction: false,
                reduction_factor_by_depth: HashMap::new(),
                enable_per_position_type_threshold: false,
                opening_pieces_threshold: 14,
                middlegame_pieces_threshold: 14,
                endgame_pieces_threshold: 14,
            },
            NullMovePreset::Aggressive => Self {
                enabled: true,
                min_depth: 2,
                reduction_factor: 3,
                max_pieces_threshold: 10,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 100,
                dynamic_reduction_formula: DynamicReductionFormula::Smooth,
                enable_mate_threat_detection: false,
                mate_threat_margin: 400,
                enable_endgame_type_detection: false,
                material_endgame_threshold: 10,
                king_activity_threshold: 6,
                zugzwang_threshold: 4,
                preset: Some(NullMovePreset::Aggressive),
                reduction_strategy: NullMoveReductionStrategy::Dynamic,
                depth_scaling_factor: 1,
                min_depth_for_scaling: 3,
                material_adjustment_factor: 1,
                piece_count_threshold: 20,
                threshold_step: 4,
                opening_reduction_factor: 4,
                middlegame_reduction_factor: 3,
                endgame_reduction_factor: 2,
                enable_per_depth_reduction: false,
                reduction_factor_by_depth: HashMap::new(),
                enable_per_position_type_threshold: false,
                opening_pieces_threshold: 10,
                middlegame_pieces_threshold: 10,
                endgame_pieces_threshold: 10,
            },
            NullMovePreset::Balanced => Self {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 200,
                dynamic_reduction_formula: DynamicReductionFormula::Linear,
                enable_mate_threat_detection: false,
                mate_threat_margin: 500,
                enable_endgame_type_detection: false,
                material_endgame_threshold: 12,
                king_activity_threshold: 8,
                zugzwang_threshold: 6,
                preset: Some(NullMovePreset::Balanced),
                reduction_strategy: NullMoveReductionStrategy::Dynamic,
                depth_scaling_factor: 1,
                min_depth_for_scaling: 4,
                material_adjustment_factor: 1,
                piece_count_threshold: 20,
                threshold_step: 4,
                opening_reduction_factor: 3,
                middlegame_reduction_factor: 2,
                endgame_reduction_factor: 1,
                enable_per_depth_reduction: false,
                reduction_factor_by_depth: HashMap::new(),
                enable_per_position_type_threshold: false,
                opening_pieces_threshold: 12,
                middlegame_pieces_threshold: 12,
                endgame_pieces_threshold: 12,
            },
        }
    }

    /// Apply a preset to this configuration
    pub fn apply_preset(&mut self, preset: NullMovePreset) {
        let preset_config = Self::from_preset(preset);
        *self = preset_config;
    }

    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.min_depth > 10 {
            return Err("min_depth should not exceed 10 for performance reasons".to_string());
        }
        if self.reduction_factor == 0 {
            return Err("reduction_factor must be greater than 0".to_string());
        }
        if self.reduction_factor > 5 {
            return Err("reduction_factor should not exceed 5".to_string());
        }
        if self.max_pieces_threshold == 0 {
            return Err("max_pieces_threshold must be greater than 0".to_string());
        }
        if self.max_pieces_threshold > 40 {
            return Err("max_pieces_threshold should not exceed 40".to_string());
        }
        if self.verification_margin < 0 {
            return Err("verification_margin must be non-negative".to_string());
        }
        if self.verification_margin > 1000 {
            return Err("verification_margin should not exceed 1000 centipawns".to_string());
        }
        Ok(())
    }
}

/// Performance statistics for null move pruning
#[derive(Debug, Clone, Default)]
pub struct NullMoveStats {
    /// Number of null move attempts
    pub attempts: u64,
    pub null_moves_tried: u64,
    pub null_moves_successful: u64,
    pub verifications_triggered: u64,
    pub verifications_successful: u64,
    /// Number of verification searches attempted
    pub verification_attempts: u64,
    pub cutoffs_after_null_move: u64,
    pub cutoffs_after_verification: u64,
    /// Total depth reductions applied
    pub depth_reductions: u64,
    pub total_depth_saved: u64,
    pub average_reduction: f64,
    pub endgame_positions_detected: u64,
    pub endgame_positions_skipped: u64,
    /// Number of mate threat detection attempts
    pub mate_threat_attempts: u64,
    pub mate_threats_detected: u64,
    /// Alias for mate_threats_detected (for compatibility)
    pub mate_threat_detected: u64,
    pub mate_threats_skipped: u64,
    /// Number of times null move was disabled in check
    pub disabled_in_check: u64,
    /// Number of times null move was disabled in endgame
    pub disabled_endgame: u64,
    /// Number of times null move was skipped due to time pressure
    pub skipped_time_pressure: u64,
    /// Total number of cutoffs (alias for cutoffs_after_null_move + cutoffs_after_verification)
    pub cutoffs: u64,
    /// Number of verification searches that resulted in cutoffs
    pub verification_cutoffs: u64,
}

impl NullMoveStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = NullMoveStats::default();
    }

    /// Get the null move success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.null_moves_tried == 0 {
            return 0.0;
        }
        (self.null_moves_successful as f64 / self.null_moves_tried as f64) * 100.0
    }

    /// Get the verification success rate as a percentage
    pub fn verification_rate(&self) -> f64 {
        if self.verifications_triggered == 0 {
            return 0.0;
        }
        (self.verifications_successful as f64 / self.verifications_triggered as f64) * 100.0
    }
}

// ============================================================================
// Late Move Reductions (LMR) Types
// ============================================================================

/// Configuration for position classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionClassificationConfig {
    /// Tactical threshold: cutoff ratio above which position is classified as tactical (default: 0.3)
    pub tactical_threshold: f64,
    /// Quiet threshold: cutoff ratio below which position is classified as quiet (default: 0.1)
    pub quiet_threshold: f64,
    /// Material imbalance threshold: material difference above which position is more tactical (default: 300 centipawns)
    pub material_imbalance_threshold: i32,
    /// Minimum moves threshold: minimum moves considered before classification (default: 5)
    pub min_moves_threshold: u64,
}

impl Default for PositionClassificationConfig {
    fn default() -> Self {
        Self {
            tactical_threshold: 0.3,
            quiet_threshold: 0.1,
            material_imbalance_threshold: 300,
            min_moves_threshold: 5,
        }
    }
}

/// Configuration for escape move detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscapeMoveConfig {
    /// Enable escape move exemption from LMR (default: true)
    pub enable_escape_move_exemption: bool,
    /// Use threat-based detection instead of heuristic (default: true)
    pub use_threat_based_detection: bool,
    /// Fallback to heuristic if threat detection unavailable (default: false)
    pub fallback_to_heuristic: bool,
}

impl Default for EscapeMoveConfig {
    fn default() -> Self {
        Self {
            enable_escape_move_exemption: true,
            use_threat_based_detection: true,
            fallback_to_heuristic: false,
        }
    }
}

/// Tuning aggressiveness level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TuningAggressiveness {
    Conservative, // Small, gradual adjustments
    Moderate,     // Balanced adjustments
    Aggressive,   // Larger, more frequent adjustments
}

impl Default for TuningAggressiveness {
    fn default() -> Self {
        Self::Moderate
    }
}

/// Advanced reduction strategies for LMR
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdvancedReductionStrategy {
    /// Basic reduction (current implementation)
    Basic,
    /// Depth-based reduction scaling (non-linear formulas)
    DepthBased,
    /// Material-based reduction adjustment (reduce more in material-imbalanced positions)
    MaterialBased,
    /// History-based reduction (reduce more for moves with poor history scores)
    HistoryBased,
    /// Combined: Use multiple strategies together
    Combined,
}

impl Default for AdvancedReductionStrategy {
    fn default() -> Self {
        Self::Basic
    }
}

/// Configuration for conditional capture/promotion exemptions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionalExemptionConfig {
    /// Enable conditional capture exemption (default: false - all captures exempted)
    pub enable_conditional_capture_exemption: bool,
    /// Minimum captured piece value to exempt from LMR (default: 100 centipawns)
    pub min_capture_value_threshold: i32,
    /// Minimum depth for conditional capture exemption (default: 5)
    pub min_depth_for_conditional_capture: u8,
    /// Enable conditional promotion exemption (default: false - all promotions exempted)
    pub enable_conditional_promotion_exemption: bool,
    /// Only exempt tactical promotions (default: true)
    pub exempt_tactical_promotions_only: bool,
    /// Minimum depth for conditional promotion exemption (default: 5)
    pub min_depth_for_conditional_promotion: u8,
}

impl Default for ConditionalExemptionConfig {
    fn default() -> Self {
        Self {
            enable_conditional_capture_exemption: false,
            min_capture_value_threshold: 100,
            min_depth_for_conditional_capture: 5,
            enable_conditional_promotion_exemption: false,
            exempt_tactical_promotions_only: true,
            min_depth_for_conditional_promotion: 5,
        }
    }
}

/// Configuration for advanced reduction strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedReductionConfig {
    /// Enable advanced reduction strategies (default: false)
    pub enabled: bool,
    /// Selected reduction strategy (default: Basic)
    pub strategy: AdvancedReductionStrategy,
    // Additional fields will be added as needed
}

impl Default for AdvancedReductionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: AdvancedReductionStrategy::Basic,
        }
    }
}

/// Configuration for adaptive tuning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveTuningConfig {
    /// Enable adaptive tuning (default: false)
    pub enabled: bool,
    /// Tuning aggressiveness level (default: Moderate)
    pub aggressiveness: TuningAggressiveness,
    /// Minimum data threshold for tuning decisions
    pub min_data_threshold: u64,
    // Additional fields will be added as needed
}

impl Default for AdaptiveTuningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            aggressiveness: TuningAggressiveness::Moderate,
            min_data_threshold: 100,
        }
    }
}

// Note: LMRConfig and LMRStats are very large structs (500+ lines combined)
// They will be included in a follow-up commit. For now, we add a placeholder.

/// Configuration for Late Move Reductions (LMR) parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LMRConfig {
    pub enabled: bool,
    pub min_depth: u8,
    pub min_move_index: u8,
    pub base_reduction: u8,
    pub max_reduction: u8,
    pub enable_dynamic_reduction: bool,
    pub enable_adaptive_reduction: bool,
    pub enable_extended_exemptions: bool,
    pub re_search_margin: i32,
    pub enable_position_type_margin: bool,
    pub tactical_re_search_margin: i32,
    pub quiet_re_search_margin: i32,
    pub classification_config: PositionClassificationConfig,
    pub escape_move_config: EscapeMoveConfig,
    pub adaptive_tuning_config: AdaptiveTuningConfig,
    pub advanced_reduction_config: AdvancedReductionConfig,
    pub conditional_exemption_config: ConditionalExemptionConfig,
}

impl Default for LMRConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 2,
            min_move_index: 3,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
            re_search_margin: 50,
            enable_position_type_margin: false,
            tactical_re_search_margin: 75,
            quiet_re_search_margin: 25,
            classification_config: PositionClassificationConfig::default(),
            escape_move_config: EscapeMoveConfig::default(),
            adaptive_tuning_config: AdaptiveTuningConfig::default(),
            advanced_reduction_config: AdvancedReductionConfig::default(),
            conditional_exemption_config: ConditionalExemptionConfig::default(),
        }
    }
}

impl LMRConfig {
    /// Validate the LMR configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.base_reduction > self.max_reduction {
            return Err("base_reduction cannot be greater than max_reduction".to_string());
        }
        if self.re_search_margin < 0 {
            return Err("re_search_margin must be non-negative".to_string());
        }
        Ok(())
    }
}

/// Performance statistics for Late Move Reductions
/// 
/// This is a large struct with many fields. The full implementation will be
/// added in a follow-up commit to manage file size.
#[derive(Debug, Clone, Default)]
pub struct LMRStats {
    pub moves_considered: u64,
    pub reductions_applied: u64,
    pub researches_triggered: u64,
    pub cutoffs_after_reduction: u64,
    pub cutoffs_after_research: u64,
    pub total_depth_saved: u64,
    pub average_reduction: f64,
    pub re_search_margin_prevented: u64,
    pub re_search_margin_allowed: u64,
    pub tt_move_exempted: u64,
    pub tt_move_missed: u64,
    pub iid_move_explicitly_exempted: u64,
    pub iid_move_reduced_count: u64,
    pub tactical_researches: u64,
    pub quiet_researches: u64,
    pub neutral_researches: u64,
    pub phase_stats: HashMap<GamePhase, LMRPhaseStats>,
    /// Position classification statistics
    pub classification_stats: PositionClassificationStats,
    /// Escape move detection statistics
    pub escape_move_stats: EscapeMoveStats,
    /// Adaptive tuning statistics
    pub adaptive_tuning_stats: AdaptiveTuningStats,
    /// Move ordering effectiveness statistics
    pub move_ordering_stats: MoveOrderingEffectivenessStats,
}

impl LMRStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = LMRStats::default();
    }

    /// Get the efficiency as a percentage
    pub fn efficiency(&self) -> f64 {
        if self.moves_considered == 0 {
            return 0.0;
        }
        (self.reductions_applied as f64 / self.moves_considered as f64) * 100.0
    }

    /// Get the research rate as a percentage
    pub fn research_rate(&self) -> f64 {
        if self.reductions_applied == 0 {
            return 0.0;
        }
        (self.researches_triggered as f64 / self.reductions_applied as f64) * 100.0
    }

    /// Get the cutoff rate as a percentage
    pub fn cutoff_rate(&self) -> f64 {
        let total_cutoffs = self.cutoffs_after_reduction + self.cutoffs_after_research;
        if self.moves_considered == 0 {
            return 0.0;
        }
        (total_cutoffs as f64 / self.moves_considered as f64) * 100.0
    }

    /// Get total cutoffs
    pub fn total_cutoffs(&self) -> u64 {
        self.cutoffs_after_reduction + self.cutoffs_after_research
    }

    /// Record LMR statistics for a specific game phase
    pub fn record_phase_stats(
        &mut self,
        phase: GamePhase,
        moves_considered: u64,
        reductions_applied: u64,
        researches_triggered: u64,
        cutoffs_after_reduction: u64,
        cutoffs_after_research: u64,
        depth_saved: u64,
    ) {
        let stats = self
            .phase_stats
            .entry(phase)
            .or_insert_with(LMRPhaseStats::default);
        stats.moves_considered += moves_considered;
        stats.reductions_applied += reductions_applied;
        stats.researches_triggered += researches_triggered;
        stats.cutoffs_after_reduction += cutoffs_after_reduction;
        stats.cutoffs_after_research += cutoffs_after_research;
        stats.total_depth_saved += depth_saved;
    }

    /// Check if move ordering has degraded
    pub fn check_ordering_degradation(&self) -> bool {
        // Simple heuristic: if research rate is too high, ordering may be degraded
        self.research_rate() > 50.0
    }

    /// Check if performance meets minimum thresholds
    pub fn check_performance_thresholds(&self) -> (bool, Vec<String>) {
        let mut alerts = Vec::new();
        let mut is_healthy = true;

        if self.research_rate() > 50.0 {
            alerts.push("Research rate too high - move ordering may be degraded".to_string());
            is_healthy = false;
        }

        if self.efficiency() < 10.0 {
            alerts.push("LMR efficiency too low".to_string());
            is_healthy = false;
        }

        (is_healthy, alerts)
    }

    /// Get performance alerts
    pub fn get_performance_alerts(&self) -> Vec<String> {
        let (_, alerts) = self.check_performance_thresholds();
        alerts
    }

    /// Export metrics for analysis
    pub fn export_metrics(&self) -> String {
        format!(
            "LMR Metrics: {} moves considered, {} reductions, {:.1}% efficiency, {:.1}% research rate",
            self.moves_considered,
            self.reductions_applied,
            self.efficiency(),
            self.research_rate()
        )
    }

    /// Generate a performance report
    pub fn performance_report(&self) -> String {
        format!(
            "LMR Performance: {:.1}% efficiency, {:.1}% research rate, {:.1}% cutoff rate",
            self.efficiency(),
            self.research_rate(),
            self.cutoff_rate()
        )
    }

    /// Get move ordering metrics
    pub fn get_move_ordering_metrics(&self) -> String {
        format!(
            "Move Ordering: {} total cutoffs, {:.1}% effectiveness",
            self.move_ordering_stats.total_cutoffs,
            self.move_ordering_stats.ordering_effectiveness()
        )
    }

    /// Get ordering vs LMR report
    pub fn get_ordering_vs_lmr_report(&self) -> String {
        format!(
            "Ordering vs LMR: {:.1}% ordering effectiveness, {:.1}% LMR efficiency",
            self.move_ordering_stats.ordering_effectiveness(),
            self.efficiency()
        )
    }
}

/// LMR statistics by game phase
#[derive(Debug, Clone, Default)]
pub struct LMRPhaseStats {
    pub moves_considered: u64,
    pub reductions_applied: u64,
    pub researches_triggered: u64,
    pub cutoffs_after_reduction: u64,
    pub cutoffs_after_research: u64,
    pub total_depth_saved: u64,
}

/// Position classification statistics
#[derive(Debug, Clone, Default)]
pub struct PositionClassificationStats {
    pub tactical_positions: u64,
    pub quiet_positions: u64,
    pub neutral_positions: u64,
}

/// Escape move detection statistics
#[derive(Debug, Clone, Default)]
pub struct EscapeMoveStats {
    pub escape_moves_detected: u64,
    pub escape_moves_exempted: u64,
}

/// Adaptive tuning statistics
#[derive(Debug, Clone, Default)]
pub struct AdaptiveTuningStats {
    pub tuning_adjustments: u64,
    pub successful_adjustments: u64,
}

impl AdaptiveTuningStats {
    /// Record a parameter change
    pub fn record_parameter_change(&mut self) {
        self.tuning_adjustments += 1;
    }

    /// Record an adjustment reason
    pub fn record_adjustment_reason(&mut self, _reason: &str) {
        // For now, just increment successful adjustments
        self.successful_adjustments += 1;
    }

    /// Record a tuning attempt
    pub fn record_tuning_attempt(&mut self) {
        self.tuning_adjustments += 1;
    }
}

/// Move ordering effectiveness statistics
#[derive(Debug, Clone, Default)]
pub struct MoveOrderingEffectivenessStats {
    pub total_cutoffs: u64,
    pub cutoffs_after_threshold: u64,
    pub late_ordered_cutoffs: u64,
    pub early_ordered_no_cutoffs: u64,
    pub cutoff_index_sum: u64,
    pub cutoff_index_count: u64,
}

impl MoveOrderingEffectivenessStats {
    /// Get the percentage of cutoffs after threshold
    pub fn cutoffs_after_threshold_percentage(&self) -> f64 {
        if self.total_cutoffs == 0 {
            return 0.0;
        }
        (self.cutoffs_after_threshold as f64 / self.total_cutoffs as f64) * 100.0
    }

    /// Get the average cutoff index
    pub fn average_cutoff_index(&self) -> f64 {
        if self.cutoff_index_count == 0 {
            return 0.0;
        }
        self.cutoff_index_sum as f64 / self.cutoff_index_count as f64
    }

    /// Get the ordering effectiveness as a percentage
    pub fn ordering_effectiveness(&self) -> f64 {
        if self.total_cutoffs == 0 {
            return 0.0;
        }
        let early_cutoffs = self.total_cutoffs - self.cutoffs_after_threshold;
        (early_cutoffs as f64 / self.total_cutoffs as f64) * 100.0
    }

    /// Record a cutoff
    pub fn record_cutoff(&mut self, index: usize) {
        self.total_cutoffs += 1;
        if index >= 3 {
            self.cutoffs_after_threshold += 1;
        }
        self.cutoff_index_sum += index as u64;
        self.cutoff_index_count += 1;
    }

    /// Record a position with no cutoff
    pub fn record_no_cutoff(&mut self) {
        self.early_ordered_no_cutoffs += 1;
    }
}

/// LMR playing style presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LMRPlayingStyle {
    Conservative,
    Aggressive,
    Balanced,
}

// ============================================================================
// Internal Iterative Deepening (IID) Types
// ============================================================================

/// IID depth selection strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IIDDepthStrategy {
    /// Fixed depth: Always use `iid_depth_ply`
    Fixed,
    /// Relative depth: Use `main_depth - 2` (capped)
    Relative,
    /// Adaptive depth: Adjust based on position complexity
    Adaptive,
    /// Dynamic depth: Use configurable base depth with complexity adjustments
    Dynamic,
}

impl Default for IIDDepthStrategy {
    fn default() -> Self {
        IIDDepthStrategy::Fixed
    }
}

/// IID configuration presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IIDPreset {
    /// Conservative preset: Lower time overhead threshold, higher min_depth, shallower IID depth
    Conservative,
    /// Aggressive preset: Higher time overhead threshold, lower min_depth, deeper IID depth
    Aggressive,
    /// Balanced preset: Default values optimized for general play
    Balanced,
}

impl IIDPreset {
    /// Get a string representation of the preset
    pub fn to_string(&self) -> &'static str {
        match self {
            IIDPreset::Conservative => "Conservative",
            IIDPreset::Aggressive => "Aggressive",
            IIDPreset::Balanced => "Balanced",
        }
    }

    /// Parse a preset from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "conservative" => Some(IIDPreset::Conservative),
            "aggressive" => Some(IIDPreset::Aggressive),
            "balanced" => Some(IIDPreset::Balanced),
            _ => None,
        }
    }
}

// Note: IIDConfig and IIDStats are very large structs (500+ lines combined)
// They will be included in a follow-up commit. For now, we add placeholders.

/// Configuration for Internal Iterative Deepening
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IIDConfig {
    pub enabled: bool,
    pub min_depth: u8,
    pub iid_depth_ply: u8,
    pub max_legal_moves: usize,
    pub time_overhead_threshold: f64,
    pub depth_strategy: IIDDepthStrategy,
    pub enable_time_pressure_detection: bool,
    pub enable_adaptive_tuning: bool,
    pub dynamic_base_depth: u8,
    pub dynamic_max_depth: u8,
    pub adaptive_min_depth: bool,
    pub max_estimated_iid_time_ms: u32,
    pub max_estimated_iid_time_percentage: bool,
    pub enable_complexity_based_adjustments: bool,
    pub complexity_threshold_low: usize,
    pub complexity_threshold_medium: usize,
    pub complexity_depth_adjustment_low: i8,
    pub complexity_depth_adjustment_medium: i8,
    pub complexity_depth_adjustment_high: i8,
    pub enable_adaptive_move_count_threshold: bool,
    pub tactical_move_count_multiplier: f64,
    pub quiet_move_count_multiplier: f64,
    pub time_pressure_base_threshold: f64,
    pub time_pressure_complexity_multiplier: f64,
    pub time_pressure_depth_multiplier: f64,
    pub tt_move_min_depth_for_skip: u8,
    pub tt_move_max_age_for_skip: u32,
    pub preset: Option<IIDPreset>,
    pub enable_game_phase_based_adjustment: bool,
    pub enable_material_based_adjustment: bool,
    pub enable_time_based_adjustment: bool,
    pub game_phase_opening_multiplier: f64,
    pub game_phase_middlegame_multiplier: f64,
    pub game_phase_endgame_multiplier: f64,
    pub material_depth_multiplier: f64,
    pub material_threshold_for_adjustment: u8,
    pub time_depth_multiplier: f64,
    pub time_threshold_for_adjustment: f64,
}

impl Default for IIDConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 4,
            iid_depth_ply: 2,
            max_legal_moves: 35,
            time_overhead_threshold: 0.15,
            depth_strategy: IIDDepthStrategy::Fixed,
            enable_time_pressure_detection: true,
            enable_adaptive_tuning: false,
            dynamic_base_depth: 2,
            dynamic_max_depth: 4,
            adaptive_min_depth: false,
            max_estimated_iid_time_ms: 50,
            max_estimated_iid_time_percentage: false,
            enable_complexity_based_adjustments: true,
            complexity_threshold_low: 10,
            complexity_threshold_medium: 25,
            complexity_depth_adjustment_low: -1,
            complexity_depth_adjustment_medium: 0,
            complexity_depth_adjustment_high: 1,
            enable_adaptive_move_count_threshold: true,
            tactical_move_count_multiplier: 1.5,
            quiet_move_count_multiplier: 0.8,
            time_pressure_base_threshold: 0.10,
            time_pressure_complexity_multiplier: 1.0,
            time_pressure_depth_multiplier: 1.0,
            tt_move_min_depth_for_skip: 3,
            tt_move_max_age_for_skip: 100,
            preset: None,
            enable_game_phase_based_adjustment: false,
            enable_material_based_adjustment: false,
            enable_time_based_adjustment: false,
            game_phase_opening_multiplier: 1.0,
            game_phase_middlegame_multiplier: 1.0,
            game_phase_endgame_multiplier: 1.0,
            material_depth_multiplier: 1.0,
            material_threshold_for_adjustment: 20,
            time_depth_multiplier: 1.0,
            time_threshold_for_adjustment: 0.15,
        }
    }
}

impl IIDConfig {
    /// Create an IIDConfig from a preset
    pub fn from_preset(preset: IIDPreset) -> Self {
        let mut config = match preset {
            IIDPreset::Conservative => Self {
                enabled: true,
                min_depth: 5,
                iid_depth_ply: 1,
                max_legal_moves: 30,
                time_overhead_threshold: 0.10,
                depth_strategy: IIDDepthStrategy::Fixed,
                enable_time_pressure_detection: true,
                enable_adaptive_tuning: false,
                dynamic_base_depth: 1,
                dynamic_max_depth: 3,
                adaptive_min_depth: false,
                max_estimated_iid_time_ms: 30,
                max_estimated_iid_time_percentage: false,
                enable_complexity_based_adjustments: true,
                complexity_threshold_low: 10,
                complexity_threshold_medium: 25,
                complexity_depth_adjustment_low: -1,
                complexity_depth_adjustment_medium: 0,
                complexity_depth_adjustment_high: 1,
                enable_adaptive_move_count_threshold: true,
                tactical_move_count_multiplier: 1.2,
                quiet_move_count_multiplier: 0.7,
                time_pressure_base_threshold: 0.08,
                time_pressure_complexity_multiplier: 0.9,
                time_pressure_depth_multiplier: 0.9,
                tt_move_min_depth_for_skip: 4,
                tt_move_max_age_for_skip: 80,
                preset: Some(IIDPreset::Conservative),
                enable_game_phase_based_adjustment: false,
                enable_material_based_adjustment: false,
                enable_time_based_adjustment: false,
                game_phase_opening_multiplier: 1.0,
                game_phase_middlegame_multiplier: 1.0,
                game_phase_endgame_multiplier: 1.0,
                material_depth_multiplier: 1.0,
                material_threshold_for_adjustment: 20,
                time_depth_multiplier: 1.0,
                time_threshold_for_adjustment: 0.15,
            },
            IIDPreset::Aggressive => Self {
                enabled: true,
                min_depth: 3,
                iid_depth_ply: 3,
                max_legal_moves: 40,
                time_overhead_threshold: 0.20,
                depth_strategy: IIDDepthStrategy::Dynamic,
                enable_time_pressure_detection: true,
                enable_adaptive_tuning: false,
                dynamic_base_depth: 2,
                dynamic_max_depth: 5,
                adaptive_min_depth: true,
                max_estimated_iid_time_ms: 70,
                max_estimated_iid_time_percentage: false,
                enable_complexity_based_adjustments: true,
                complexity_threshold_low: 10,
                complexity_threshold_medium: 25,
                complexity_depth_adjustment_low: -1,
                complexity_depth_adjustment_medium: 0,
                complexity_depth_adjustment_high: 1,
                enable_adaptive_move_count_threshold: true,
                tactical_move_count_multiplier: 1.8,
                quiet_move_count_multiplier: 0.9,
                time_pressure_base_threshold: 0.12,
                time_pressure_complexity_multiplier: 1.2,
                time_pressure_depth_multiplier: 1.1,
                tt_move_min_depth_for_skip: 2,
                tt_move_max_age_for_skip: 120,
                preset: Some(IIDPreset::Aggressive),
                enable_game_phase_based_adjustment: true,
                enable_material_based_adjustment: true,
                enable_time_based_adjustment: true,
                game_phase_opening_multiplier: 1.2,
                game_phase_middlegame_multiplier: 1.0,
                game_phase_endgame_multiplier: 0.8,
                material_depth_multiplier: 1.1,
                material_threshold_for_adjustment: 20,
                time_depth_multiplier: 0.9,
                time_threshold_for_adjustment: 0.15,
            },
            IIDPreset::Balanced => Self {
                enabled: true,
                min_depth: 4,
                iid_depth_ply: 2,
                max_legal_moves: 35,
                time_overhead_threshold: 0.15,
                depth_strategy: IIDDepthStrategy::Fixed,
                enable_time_pressure_detection: true,
                enable_adaptive_tuning: false,
                dynamic_base_depth: 2,
                dynamic_max_depth: 4,
                adaptive_min_depth: false,
                max_estimated_iid_time_ms: 50,
                max_estimated_iid_time_percentage: false,
                enable_complexity_based_adjustments: true,
                complexity_threshold_low: 10,
                complexity_threshold_medium: 25,
                complexity_depth_adjustment_low: -1,
                complexity_depth_adjustment_medium: 0,
                complexity_depth_adjustment_high: 1,
                enable_adaptive_move_count_threshold: true,
                tactical_move_count_multiplier: 1.5,
                quiet_move_count_multiplier: 0.8,
                time_pressure_base_threshold: 0.10,
                time_pressure_complexity_multiplier: 1.0,
                time_pressure_depth_multiplier: 1.0,
                tt_move_min_depth_for_skip: 3,
                tt_move_max_age_for_skip: 100,
                preset: Some(IIDPreset::Balanced),
                enable_game_phase_based_adjustment: false,
                enable_material_based_adjustment: false,
                enable_time_based_adjustment: false,
                game_phase_opening_multiplier: 1.0,
                game_phase_middlegame_multiplier: 1.0,
                game_phase_endgame_multiplier: 1.0,
                material_depth_multiplier: 1.0,
                material_threshold_for_adjustment: 20,
                time_depth_multiplier: 1.0,
                time_threshold_for_adjustment: 0.15,
            },
        };

        // Validate the configuration
        if let Err(_) = config.validate() {
            // If validation fails, fall back to default
            config = Self::default();
            config.preset = Some(preset);
        }

        config
    }

    /// Apply a preset to this configuration
    pub fn apply_preset(&mut self, preset: IIDPreset) {
        *self = Self::from_preset(preset);
    }

    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth < 2 {
            return Err("min_depth must be at least 2".to_string());
        }
        if self.min_depth > 15 {
            return Err("min_depth should not exceed 15 for performance reasons".to_string());
        }
        if self.iid_depth_ply == 0 {
            return Err("iid_depth_ply must be greater than 0".to_string());
        }
        if self.iid_depth_ply > 5 {
            return Err("iid_depth_ply should not exceed 5 for performance reasons".to_string());
        }
        if self.max_legal_moves == 0 {
            return Err("max_legal_moves must be greater than 0".to_string());
        }
        if self.max_legal_moves > 100 {
            return Err("max_legal_moves should not exceed 100".to_string());
        }
        if self.time_overhead_threshold < 0.0 || self.time_overhead_threshold > 1.0 {
            return Err("time_overhead_threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        let preset_str = if let Some(preset) = self.preset {
            format!(", preset={}", preset.to_string())
        } else {
            String::new()
        };

        format!(
            "IIDConfig: enabled={}, min_depth={}, iid_depth_ply={}, max_moves={}, overhead_threshold={:.2}, strategy={:?}{}",
            self.enabled,
            self.min_depth,
            self.iid_depth_ply,
            self.max_legal_moves,
            self.time_overhead_threshold,
            self.depth_strategy,
            preset_str
        )
    }
}

/// Performance statistics for Internal Iterative Deepening
/// 
/// This is a large struct with many fields. The full implementation will be
/// added in a follow-up commit to manage file size.
#[derive(Debug, Clone, Default)]
pub struct IIDStats {
    pub iid_searches_performed: u64,
    pub iid_move_first_improved_alpha: u64,
    pub iid_move_caused_cutoff: u64,
    pub total_iid_nodes: u64,
    pub iid_time_ms: u64,
    pub total_search_time_ms: u64,
    pub positions_skipped_tt_move: u64,
    pub positions_skipped_depth: u64,
    pub positions_skipped_move_count: u64,
    pub positions_skipped_time_pressure: u64,
    pub iid_searches_failed: u64,
    pub iid_moves_ineffective: u64,
    pub iid_move_extracted_from_tt: u64,
    pub iid_move_extracted_from_tracked: u64,
    pub dynamic_depth_selections: HashMap<u8, u64>,
    pub dynamic_depth_low_complexity: u64,
    pub dynamic_depth_medium_complexity: u64,
    pub dynamic_depth_high_complexity: u64,
    pub total_predicted_iid_time_ms: u64,
    pub total_actual_iid_time_ms: u64,
    pub positions_skipped_time_estimation: u64,
    pub total_nodes_without_iid: u64,
    pub total_time_without_iid_ms: u64,
    pub nodes_saved: u64,
    pub efficiency_speedup_correlation_sum: f64,
    pub correlation_data_points: u64,
    pub performance_measurement_accuracy_sum: f64,
    pub performance_measurement_samples: u64,
    pub time_pressure_detection_correct: u64,
    pub time_pressure_detection_total: u64,
    pub tt_move_condition_skips: u64,
    pub tt_move_condition_tt_move_used: u64,
    pub complexity_distribution_low: u64,
    pub complexity_distribution_medium: u64,
    pub complexity_distribution_high: u64,
    pub complexity_distribution_unknown: u64,
    pub complexity_effectiveness: HashMap<PositionComplexity, (u64, u64, u64, u64)>,
    pub game_phase_adjustment_applied: u64,
    pub game_phase_adjustment_effective: u64,
    pub material_adjustment_applied: u64,
    pub material_adjustment_effective: u64,
    pub time_adjustment_applied: u64,
    pub time_adjustment_effective: u64,
    pub game_phase_opening_adjustments: u64,
    pub game_phase_middlegame_adjustments: u64,
    pub game_phase_endgame_adjustments: u64,
    pub iid_move_ordered_first: u64,
    pub iid_move_not_ordered_first: u64,
    pub cutoffs_from_iid_moves: u64,
    pub cutoffs_from_non_iid_moves: u64,
    pub total_cutoffs: u64,
    pub iid_move_position_sum: u64,
    pub iid_move_position_tracked: u64,
    pub ordering_effectiveness_with_iid_cutoffs: u64,
    pub ordering_effectiveness_with_iid_total: u64,
    pub ordering_effectiveness_without_iid_cutoffs: u64,
    pub ordering_effectiveness_without_iid_total: u64,
    pub iid_efficiency_ordering_correlation_sum: f64,
    pub iid_efficiency_ordering_correlation_points: u64,
}

impl IIDStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = IIDStats::default();
    }

    /// Get the IID efficiency rate as a percentage
    pub fn efficiency_rate(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        (self.iid_move_first_improved_alpha as f64 / self.iid_searches_performed as f64) * 100.0
    }

    /// Get the IID cutoff rate as a percentage
    pub fn cutoff_rate(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        (self.iid_move_caused_cutoff as f64 / self.iid_searches_performed as f64) * 100.0
    }
}

/// Efficient board state representation for IID search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDBoardState {
    /// Compact position key for quick comparison
    pub key: u64,
    /// Material balance (Black - White)
    pub material_balance: i32,
    /// Total number of pieces on board
    pub piece_count: u8,
    /// King positions (Black, White)
    pub king_positions: (Option<Position>, Option<Position>),
    /// Cached move generation results
    pub move_cache: Option<Vec<Move>>,
}

/// Statistics for IID overhead monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDOverheadStats {
    /// Total number of IID searches performed
    pub total_iid_searches: u64,
    /// Total time spent in IID searches (milliseconds)
    pub total_iid_time_ms: u64,
    /// Total time spent in all searches (milliseconds)
    pub total_search_time_ms: u64,
    /// Calculated overhead percentage
    pub overhead_percentage: f64,
    /// Average overhead percentage across all searches
    pub average_overhead: f64,
    /// Current threshold for IID overhead
    pub current_threshold: f64,
    /// Number of threshold adjustments made
    pub threshold_adjustments: u64,
    /// Total number of searches (alias for total_iid_searches)
    pub total_searches: u64,
    /// Number of times IID was skipped due to time pressure
    pub time_pressure_skips: u64,
}

// ============================================================================
// Aspiration Window Types
// ============================================================================

/// Aspiration window playing style presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AspirationWindowPlayingStyle {
    Conservative,
    Aggressive,
    Balanced,
}

// Note: AspirationWindowConfig and AspirationWindowStats are large structs
// They will be included in a follow-up commit. For now, we add placeholders.

/// Configuration for aspiration windows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AspirationWindowConfig {
    pub enabled: bool,
    pub base_window_size: i32,
    pub max_window_size: i32,
    pub min_depth: u8,
    pub dynamic_scaling: bool,
    pub enable_adaptive_sizing: bool,
    pub max_researches: u8,
    pub enable_statistics: bool,
    pub use_static_eval_for_init: bool,
    pub enable_position_type_tracking: bool,
    pub disable_statistics_in_production: bool,
}

impl AspirationWindowConfig {
    /// Validate the aspiration window configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.base_window_size > self.max_window_size {
            return Err("base_window_size cannot be greater than max_window_size".to_string());
        }
        Ok(())
    }
}

impl Default for AspirationWindowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_window_size: 50,
            max_window_size: 200,
            min_depth: 2,
            dynamic_scaling: true,
            enable_adaptive_sizing: true,
            max_researches: 2,
            enable_statistics: true,
            use_static_eval_for_init: true,
            enable_position_type_tracking: true,
            disable_statistics_in_production: false,
        }
    }
}

/// Performance statistics for aspiration windows
#[derive(Debug, Clone, Default)]
pub struct AspirationWindowStats {
    pub total_searches: u64,
    pub fail_lows: u64,
    pub fail_highs: u64,
    pub successful_searches: u64,
    /// Total re-searches performed
    pub total_researches: u64,
    pub average_window_size: f64,
    /// Time saved (estimated)
    pub estimated_time_saved_ms: u64,
    /// Nodes saved (estimated)
    pub estimated_nodes_saved: u64,
    pub window_size_by_position_type: WindowSizeByPositionType,
    pub success_rate_by_position_type: SuccessRateByPositionType,
    /// Success rate by depth (for depth analysis)
    pub success_rate_by_depth: Vec<f64>,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Configuration effectiveness score
    pub configuration_effectiveness: f64,
}

/// Window size statistics by position type
#[derive(Debug, Clone, Default)]
pub struct WindowSizeByPositionType {
    /// Average window size in opening
    pub opening_avg_window_size: f64,
    /// Average window size in middlegame
    pub middlegame_avg_window_size: f64,
    /// Average window size in endgame
    pub endgame_avg_window_size: f64,
    /// Number of searches in opening
    pub opening_searches: u64,
    /// Number of searches in middlegame
    pub middlegame_searches: u64,
    /// Number of searches in endgame
    pub endgame_searches: u64,
}

/// Success rate statistics by position type
#[derive(Debug, Clone, Default)]
pub struct SuccessRateByPositionType {
    /// Success rate in opening
    pub opening_success_rate: f64,
    /// Success rate in middlegame
    pub middlegame_success_rate: f64,
    /// Success rate in endgame
    pub endgame_success_rate: f64,
    /// Successful searches in opening
    pub opening_successful: u64,
    /// Successful searches in middlegame
    pub middlegame_successful: u64,
    /// Successful searches in endgame
    pub endgame_successful: u64,
    /// Total searches in opening
    pub opening_total: u64,
    /// Total searches in middlegame
    pub middlegame_total: u64,
    /// Total searches in endgame
    pub endgame_total: u64,
}

impl AspirationWindowStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = AspirationWindowStats::default();
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.successful_searches as f64 / self.total_searches as f64) * 100.0
    }

    /// Get performance trend (simplified - returns success rate)
    pub fn get_performance_trend(&self) -> f64 {
        self.success_rate()
    }

    /// Update window size statistics
    pub fn update_window_size_stats(&mut self, _window_size: i32) {
        // For now, this is a placeholder
        // In a full implementation, this would track window size distribution
    }

    /// Get the research rate as a percentage
    pub fn research_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.total_researches as f64 / self.total_searches as f64) * 100.0
    }

    /// Update window size by position type
    pub fn update_window_size_by_position_type(&mut self, _position_type: GamePhase, _window_size: i32) {
        // Placeholder implementation
    }

    /// Update success rate by position type
    pub fn update_success_rate_by_position_type(&mut self, _position_type: GamePhase, _success: bool) {
        // Placeholder implementation
    }

    /// Update depth statistics
    pub fn update_depth_stats(&mut self, _depth: u8, _success: bool) {
        // Placeholder implementation
    }

    /// Update time statistics
    pub fn update_time_stats(&mut self, _time_ms: u64) {
        // Placeholder implementation
    }

    /// Update memory statistics
    pub fn update_memory_stats(&mut self, _memory_bytes: u64) {
        // Placeholder implementation
    }

    /// Initialize depth tracking
    pub fn initialize_depth_tracking(&mut self) {
        // Placeholder implementation
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        format!(
            "Aspiration Window: {:.1}% success rate, {} researches",
            self.success_rate(),
            self.total_researches
        )
    }

    /// Get depth analysis
    pub fn get_depth_analysis(&self) -> String {
        format!("Depth analysis: {} searches", self.total_searches)
    }

    /// Calculate performance metrics
    pub fn calculate_performance_metrics(&self) -> f64 {
        self.success_rate()
    }

    /// Get fail low rate
    pub fn fail_low_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.fail_lows as f64 / self.total_searches as f64) * 100.0
    }

    /// Get efficiency
    pub fn efficiency(&self) -> f64 {
        self.success_rate()
    }

    /// Add performance data point for trend analysis
    pub fn add_performance_data_point(&mut self, _performance: f64) {
        // Placeholder implementation
    }
}

// ============================================================================
// Time Management Types
// ============================================================================

/// Time allocation strategy for iterative deepening
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeAllocationStrategy {
    /// Equal time allocation per depth
    Equal,
    /// Exponential time allocation (later depths get more time)
    Exponential,
    /// Adaptive allocation based on previous depth completion times
    Adaptive,
}

impl Default for TimeAllocationStrategy {
    fn default() -> Self {
        TimeAllocationStrategy::Adaptive
    }
}

/// Time budget allocation tracking
#[derive(Debug, Clone, Default)]
pub struct TimeBudgetStats {
    /// Depth completion times in milliseconds
    pub depth_completion_times_ms: Vec<u32>,
    /// Estimated time per depth based on history
    pub estimated_time_per_depth_ms: Vec<u32>,
    /// Actual time used per depth
    pub actual_time_per_depth_ms: Vec<u32>,
    /// Time budget allocated per depth
    pub budget_per_depth_ms: Vec<u32>,
    /// Number of depths completed
    pub depths_completed: u8,
    /// Number of depths that exceeded budget
    pub depths_exceeded_budget: u8,
    /// Average time estimation accuracy (0.0 to 1.0)
    pub estimation_accuracy: f64,
}

/// Configuration for time management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeManagementConfig {
    /// Enable time management
    pub enabled: bool,
    /// Time buffer percentage (0.0 to 1.0)
    pub buffer_percentage: f64,
    /// Minimum time per move in milliseconds
    pub min_time_ms: u32,
    /// Maximum time per move in milliseconds
    pub max_time_ms: u32,
    /// Time increment per move in milliseconds
    pub increment_ms: u32,
    /// Enable time pressure detection
    pub enable_pressure_detection: bool,
    /// Time pressure threshold (0.0 to 1.0)
    pub pressure_threshold: f64,
    /// Time allocation strategy for iterative deepening
    pub allocation_strategy: TimeAllocationStrategy,
    /// Safety margin as percentage of total time (0.0 to 1.0)
    pub safety_margin: f64,
    /// Minimum time per depth in milliseconds
    pub min_time_per_depth_ms: u32,
    /// Maximum time per depth in milliseconds (0 = no limit)
    pub max_time_per_depth_ms: u32,
    /// Enable check position optimization
    pub enable_check_optimization: bool,
    /// Check position max depth threshold
    pub check_max_depth: u8,
    /// Check position time limit in milliseconds
    pub check_time_limit_ms: u32,
    /// Enable time budget allocation
    pub enable_time_budget: bool,
    /// Time check frequency: check every N nodes instead of every node
    pub time_check_frequency: u32,
    /// Absolute safety margin in milliseconds
    pub absolute_safety_margin_ms: u32,
}

impl Default for TimeManagementConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_percentage: 0.1,
            min_time_ms: 100,
            max_time_ms: 30000,
            increment_ms: 0,
            enable_pressure_detection: true,
            pressure_threshold: 0.2,
            allocation_strategy: TimeAllocationStrategy::Adaptive,
            safety_margin: 0.1, // 10% safety margin
            min_time_per_depth_ms: 50,
            max_time_per_depth_ms: 0, // No limit by default
            enable_check_optimization: true,
            check_max_depth: 5,
            check_time_limit_ms: 5000,
            enable_time_budget: true,
            time_check_frequency: 1024, // Check every 1024 nodes (reduce overhead)
            absolute_safety_margin_ms: 100, // 100ms absolute safety margin
        }
    }
}

impl TimeManagementConfig {
    /// Validate time management configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.buffer_percentage < 0.0 || self.buffer_percentage > 1.0 {
            return Err("Buffer percentage must be between 0.0 and 1.0".to_string());
        }

        if self.min_time_ms > self.max_time_ms {
            return Err("Min time cannot be greater than max time".to_string());
        }

        if self.pressure_threshold < 0.0 || self.pressure_threshold > 1.0 {
            return Err("Pressure threshold must be between 0.0 and 1.0".to_string());
        }

        if self.safety_margin < 0.0 || self.safety_margin > 0.5 {
            return Err("safety_margin must be between 0.0 and 0.5".to_string());
        }
        if self.min_time_per_depth_ms == 0 {
            return Err("min_time_per_depth_ms must be greater than 0".to_string());
        }
        if self.max_time_per_depth_ms > 0 && self.max_time_per_depth_ms < self.min_time_per_depth_ms
        {
            return Err("max_time_per_depth_ms must be >= min_time_per_depth_ms".to_string());
        }
        if self.check_max_depth == 0 || self.check_max_depth > 10 {
            return Err("check_max_depth must be between 1 and 10".to_string());
        }

        if self.time_check_frequency == 0 {
            return Err("time_check_frequency must be greater than 0".to_string());
        }
        if self.time_check_frequency > 100000 {
            return Err(
                "time_check_frequency should not exceed 100000 for performance reasons".to_string(),
            );
        }

        if self.absolute_safety_margin_ms > 10000 {
            return Err(
                "absolute_safety_margin_ms should not exceed 10000ms (10 seconds)".to_string(),
            );
        }

        Ok(())
    }

    /// Calculate time allocation for a move
    pub fn calculate_time_allocation(&self, total_time_ms: u32, moves_remaining: u32) -> u32 {
        if !self.enabled || moves_remaining == 0 {
            return self.min_time_ms;
        }

        let base_time = total_time_ms / moves_remaining;
        let buffered_time = (base_time as f64 * (1.0 - self.buffer_percentage)) as u32;

        buffered_time.max(self.min_time_ms).min(self.max_time_ms)
    }

    /// Check if in time pressure
    pub fn is_time_pressure(&self, time_remaining_ms: u32, total_time_ms: u32) -> bool {
        if !self.enable_pressure_detection || total_time_ms == 0 {
            return false;
        }

        let time_ratio = time_remaining_ms as f64 / total_time_ms as f64;
        time_ratio < self.pressure_threshold
    }
}

// ============================================================================
// Search State and Pruning Types
// ============================================================================

/// Search state for advanced alpha-beta pruning
#[derive(Debug, Clone)]
pub struct SearchState {
    pub depth: u8,
    pub move_number: u8,
    pub alpha: i32,
    pub beta: i32,
    pub is_in_check: bool,
    pub static_eval: i32,
    pub best_move: Option<Move>,
    pub position_hash: u64,
    pub game_phase: GamePhase,
    /// Position classification for adaptive reduction (optional, computed by SearchEngine)
    pub position_classification: Option<PositionClassification>,
    /// Transposition table best move (optional, retrieved from TT probe)
    pub tt_move: Option<Move>,
    /// Advanced reduction strategies configuration (optional)
    pub advanced_reduction_config: Option<AdvancedReductionConfig>,
    /// Best score found so far (for diagnostic purposes)
    pub best_score: i32,
    /// Number of nodes searched (for diagnostic purposes)
    #[allow(dead_code)]
    pub nodes_searched: u64,
    /// Whether aspiration windows are enabled (for diagnostic purposes)
    pub aspiration_enabled: bool,
    /// Number of researches performed (for diagnostic purposes)
    pub researches: u8,
    /// Health score of the search (for diagnostic purposes)
    pub health_score: f64,
}

impl SearchState {
    pub fn new(depth: u8, alpha: i32, beta: i32) -> Self {
        Self {
            depth,
            move_number: 0,
            alpha,
            beta,
            is_in_check: false,
            static_eval: 0,
            best_move: None,
            position_hash: 0,
            game_phase: GamePhase::Middlegame,
            position_classification: None,
            tt_move: None,
            advanced_reduction_config: None,
            best_score: 0,
            nodes_searched: 0,
            aspiration_enabled: false,
            researches: 0,
            health_score: 1.0,
        }
    }

    /// Update search state with current position information
    pub fn update_fields(
        &mut self,
        is_in_check: bool,
        static_eval: i32,
        position_hash: u64,
        game_phase: GamePhase,
    ) {
        self.is_in_check = is_in_check;
        self.static_eval = static_eval;
        self.position_hash = position_hash;
        self.game_phase = game_phase;
    }

    /// Set position classification for adaptive reduction
    pub fn set_position_classification(&mut self, classification: PositionClassification) {
        self.position_classification = Some(classification);
    }

    /// Set the transposition table best move
    pub fn set_tt_move(&mut self, tt_move: Option<Move>) {
        self.tt_move = tt_move;
    }

    /// Set advanced reduction strategies configuration
    pub fn set_advanced_reduction_config(&mut self, config: AdvancedReductionConfig) {
        self.advanced_reduction_config = Some(config);
    }
}

/// Pruning decision result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PruningDecision {
    Search,        // Search normally
    ReducedSearch, // Search with reduced depth
    Skip,          // Skip this move
    Razor,         // Use razoring
}

impl PruningDecision {
    pub fn is_pruned(&self) -> bool {
        matches!(self, PruningDecision::Skip)
    }

    pub fn needs_reduction(&self) -> bool {
        matches!(self, PruningDecision::ReducedSearch)
    }
}

/// Parameters for advanced alpha-beta pruning techniques
#[derive(Debug, Clone, PartialEq)]
pub struct PruningParameters {
    // Futility pruning parameters
    pub futility_margin: [i32; 8],
    pub futility_depth_limit: u8,
    pub extended_futility_depth: u8,

    // Late move reduction parameters
    pub lmr_base_reduction: u8,
    pub lmr_move_threshold: u8,
    pub lmr_depth_threshold: u8,
    pub lmr_max_reduction: u8,
    pub lmr_enable_extended_exemptions: bool,
    pub lmr_enable_adaptive_reduction: bool,

    // Delta pruning parameters
    pub delta_margin: i32,
    pub delta_depth_limit: u8,

    // Razoring parameters
    pub razoring_depth_limit: u8,
    pub razoring_margin: i32,
    pub razoring_margin_endgame: i32,

    // Multi-cut pruning parameters
    pub multi_cut_threshold: u8,
    pub multi_cut_depth_limit: u8,

    // Adaptive parameters
    pub adaptive_enabled: bool,
    pub position_dependent_margins: bool,
    
    // Razoring enable flag
    pub razoring_enabled: bool,
    // Late move pruning parameters
    pub late_move_pruning_enabled: bool,
    pub late_move_pruning_move_threshold: u8,
}

impl Default for PruningParameters {
    fn default() -> Self {
        Self {
            futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
            futility_depth_limit: 3,
            extended_futility_depth: 5,
            lmr_base_reduction: 1,
            lmr_move_threshold: 3,
            lmr_depth_threshold: 2,
            lmr_max_reduction: 3,
            lmr_enable_extended_exemptions: true,
            lmr_enable_adaptive_reduction: true,
            delta_margin: 200,
            delta_depth_limit: 4,
            razoring_depth_limit: 3,
            razoring_margin: 300,
            razoring_margin_endgame: 200,
            multi_cut_threshold: 3,
            multi_cut_depth_limit: 4,
            adaptive_enabled: false,
            position_dependent_margins: false,
            razoring_enabled: true,
            late_move_pruning_enabled: true,
            late_move_pruning_move_threshold: 4,
        }
    }
}

/// Statistics for pruning effectiveness monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningStatistics {
    pub total_moves: u64,
    pub pruned_moves: u64,
    pub futility_pruned: u64,
    pub delta_pruned: u64,
    pub razored: u64,
    pub lmr_applied: u64,
    pub re_searches: u64,
    pub multi_cuts: u64,
}

impl PruningStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_decision(&mut self, decision: PruningDecision) {
        self.total_moves += 1;

        match decision {
            PruningDecision::Skip => self.pruned_moves += 1,
            PruningDecision::Razor => self.razored += 1,
            _ => {}
        }
    }

    pub fn get_pruning_rate(&self) -> f64 {
        if self.total_moves == 0 {
            0.0
        } else {
            self.pruned_moves as f64 / self.total_moves as f64
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Pruning effectiveness metrics for analysis
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningEffectiveness {
    pub futility_rate: f64,
    pub delta_rate: f64,
    pub razoring_rate: f64,
    pub multi_cut_rate: f64,
    pub lmr_rate: f64,
    pub overall_effectiveness: f64,
}

/// Pruning frequency statistics for detailed analysis
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningFrequencyStats {
    pub total_moves: u64,
    pub pruned_moves: u64,
    pub futility_pruned: u64,
    pub delta_pruned: u64,
    pub razored: u64,
    pub lmr_applied: u64,
    pub multi_cuts: u64,
    pub re_searches: u64,
    pub pruning_rate: f64,
    pub cache_hit_rate: f64,
}

/// Search performance metrics for monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchPerformanceMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: usize,
    pub position_cache_size: usize,
    pub check_cache_size: usize,
    pub total_cache_operations: u64,
    pub cache_hit_rate: f64,
}

/// Comprehensive search metrics for performance monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CoreSearchMetrics {
    /// Total number of nodes searched
    pub total_nodes: u64,
    /// Total number of alpha-beta cutoffs
    pub total_cutoffs: u64,
    /// Total number of transposition table probes
    pub total_tt_probes: u64,
    /// Total number of transposition table hits
    pub total_tt_hits: u64,
    /// Total number of aspiration window searches
    pub total_aspiration_searches: u64,
    /// Number of successful aspiration window searches (no re-search needed)
    pub successful_aspiration_searches: u64,
    /// Number of beta cutoffs (move ordering effectiveness indicator)
    pub beta_cutoffs: u64,
    /// Number of exact score entries in TT
    pub tt_exact_hits: u64,
    /// Number of lower bound entries in TT
    pub tt_lower_bound_hits: u64,
    /// Number of upper bound entries in TT
    pub tt_upper_bound_hits: u64,
    /// Number of times auxiliary entry was prevented from overwriting deeper main entry
    pub tt_auxiliary_overwrites_prevented: u64,
    /// Number of times main entry preserved another main entry
    pub tt_main_entries_preserved: u64,
    /// Number of evaluation cache hits
    pub evaluation_cache_hits: u64,
    /// Number of evaluation calls saved through caching
    pub evaluation_calls_saved: u64,
}

impl CoreSearchMetrics {
    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Generate a comprehensive metrics report
    pub fn generate_report(&self) -> String {
        format!(
            "Core Search Metrics: {} nodes, {} cutoffs, {:.1}% TT hit rate",
            self.total_nodes,
            self.total_cutoffs,
            if self.total_tt_probes > 0 {
                (self.total_tt_hits as f64 / self.total_tt_probes as f64) * 100.0
            } else {
                0.0
            }
        )
    }
}

// Re-export search-related types from all.rs (temporary until moved to search.rs)
// These types are still in all.rs but should be accessible via types::search::
pub use super::all::{EngineConfig, EnginePreset, ParallelOptions, TimePressure, TimePressureThresholds};

// Note: Additional search-related types (SearchMetrics, ParallelSearchMetrics, etc.)
// will be added in follow-up commits as needed. This module provides the core search types.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposition_flag() {
        assert_eq!(TranspositionFlag::Exact.to_string(), "Exact");
        assert_eq!(TranspositionFlag::LowerBound.to_string(), "LowerBound");
        assert_eq!(TranspositionFlag::UpperBound.to_string(), "UpperBound");
    }

    #[test]
    fn test_entry_source() {
        assert_eq!(EntrySource::from_discriminant(0), EntrySource::MainSearch);
        assert_eq!(EntrySource::from_discriminant(4), EntrySource::OpeningBook);
        assert_eq!(EntrySource::MainSearch.to_discriminant(), 0);
    }

    #[test]
    fn test_quiescence_config() {
        let config = QuiescenceConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_depth, 8);
    }

    #[test]
    fn test_null_move_preset() {
        let config = NullMoveConfig::from_preset(NullMovePreset::Balanced);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.verification_margin, 200);
    }

    #[test]
    fn test_iid_config() {
        let config = IIDConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.min_depth, 4);
    }

    #[test]
    fn test_search_state() {
        let mut state = SearchState::new(5, -1000, 1000);
        assert_eq!(state.depth, 5);
        assert_eq!(state.alpha, -1000);
        assert_eq!(state.beta, 1000);
        
        state.set_position_classification(PositionClassification::Tactical);
        assert_eq!(state.position_classification, Some(PositionClassification::Tactical));
    }

    #[test]
    fn test_pruning_decision() {
        assert!(PruningDecision::Skip.is_pruned());
        assert!(!PruningDecision::Search.is_pruned());
        assert!(PruningDecision::ReducedSearch.needs_reduction());
    }
}

