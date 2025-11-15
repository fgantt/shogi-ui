//! Evaluation-Related Types
//!
//! This module contains all types related to position evaluation: tapered evaluation,
//! material evaluation, piece-square tables, king safety, and evaluation configuration.
//!
//! Extracted from `types.rs` (now `all.rs`) as part of Task 1.0: File Modularization and Structure Improvements.

use serde::{Deserialize, Serialize};

use super::core::PieceType;

// ============================================================================
// Tapered Evaluation Types
// ============================================================================

/// Maximum game phase value (opening)
pub const GAME_PHASE_MAX: i32 = 256;

/// Phase values for different piece types
pub const PIECE_PHASE_VALUES: [(PieceType, i32); 12] = [
    (PieceType::Lance, 1),
    (PieceType::Knight, 1),
    (PieceType::Silver, 1),
    (PieceType::Gold, 2),
    (PieceType::Bishop, 2),
    (PieceType::Rook, 3),
    (PieceType::PromotedPawn, 2),
    (PieceType::PromotedLance, 2),
    (PieceType::PromotedKnight, 2),
    (PieceType::PromotedSilver, 2),
    (PieceType::PromotedBishop, 3),
    (PieceType::PromotedRook, 3),
];

/// Tapered score combining middlegame and endgame values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaperedScore {
    /// Middlegame score (0-256 phase)
    pub mg: i32,
    /// Endgame score (0-256 phase)
    pub eg: i32,
}

impl TaperedScore {
    /// Create a new TaperedScore with both values equal
    pub fn new(value: i32) -> Self {
        Self {
            mg: value,
            eg: value,
        }
    }

    /// Create a TaperedScore with different mg and eg values
    pub fn new_tapered(mg: i32, eg: i32) -> Self {
        Self { mg, eg }
    }

    /// Interpolate between mg and eg based on game phase
    /// phase: 0 = endgame, GAME_PHASE_MAX = opening
    pub fn interpolate(&self, phase: i32) -> i32 {
        (self.mg * phase + self.eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX
    }
}

impl Default for TaperedScore {
    fn default() -> Self {
        Self { mg: 0, eg: 0 }
    }
}

impl std::ops::AddAssign for TaperedScore {
    fn add_assign(&mut self, other: Self) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

impl std::ops::SubAssign for TaperedScore {
    fn sub_assign(&mut self, other: Self) {
        self.mg -= other.mg;
        self.eg -= other.eg;
    }
}

impl std::ops::Add for TaperedScore {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            mg: self.mg + other.mg,
            eg: self.eg + other.eg,
        }
    }
}

impl std::ops::Sub for TaperedScore {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            mg: self.mg - other.mg,
            eg: self.eg - other.eg,
        }
    }
}

impl std::ops::Neg for TaperedScore {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            mg: -self.mg,
            eg: -self.eg,
        }
    }
}

impl std::ops::Mul<f32> for TaperedScore {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            mg: (self.mg as f32 * rhs) as i32,
            eg: (self.eg as f32 * rhs) as i32,
        }
    }
}

// ============================================================================
// Evaluation Feature Constants
// ============================================================================

/// Total number of evaluation features for tuning
pub const NUM_EVAL_FEATURES: usize = 2000;

/// Number of middlegame features (first half of feature vector)
pub const NUM_MG_FEATURES: usize = NUM_EVAL_FEATURES / 2;

/// Number of endgame features (second half of feature vector)
pub const NUM_EG_FEATURES: usize = NUM_EVAL_FEATURES / 2;

// Material feature indices (14 piece types × 2 players = 28 features)
pub const MATERIAL_PAWN_INDEX: usize = 0;
pub const MATERIAL_LANCE_INDEX: usize = 1;
pub const MATERIAL_KNIGHT_INDEX: usize = 2;
pub const MATERIAL_SILVER_INDEX: usize = 3;
pub const MATERIAL_GOLD_INDEX: usize = 4;
pub const MATERIAL_BISHOP_INDEX: usize = 5;
pub const MATERIAL_ROOK_INDEX: usize = 6;
pub const MATERIAL_KING_INDEX: usize = 7;
pub const MATERIAL_PROMOTED_PAWN_INDEX: usize = 8;
pub const MATERIAL_PROMOTED_LANCE_INDEX: usize = 9;
pub const MATERIAL_PROMOTED_KNIGHT_INDEX: usize = 10;
pub const MATERIAL_PROMOTED_SILVER_INDEX: usize = 11;
pub const MATERIAL_PROMOTED_BISHOP_INDEX: usize = 12;
pub const MATERIAL_PROMOTED_ROOK_INDEX: usize = 13;
pub const MATERIAL_WHITE_PAWN_INDEX: usize = 14;
pub const MATERIAL_WHITE_LANCE_INDEX: usize = 15;
pub const MATERIAL_WHITE_KNIGHT_INDEX: usize = 16;
pub const MATERIAL_WHITE_SILVER_INDEX: usize = 17;
pub const MATERIAL_WHITE_GOLD_INDEX: usize = 18;
pub const MATERIAL_WHITE_BISHOP_INDEX: usize = 19;
pub const MATERIAL_WHITE_ROOK_INDEX: usize = 20;
pub const MATERIAL_WHITE_KING_INDEX: usize = 21;
pub const MATERIAL_WHITE_PROMOTED_PAWN_INDEX: usize = 22;
pub const MATERIAL_WHITE_PROMOTED_LANCE_INDEX: usize = 23;
pub const MATERIAL_WHITE_PROMOTED_KNIGHT_INDEX: usize = 24;
pub const MATERIAL_WHITE_PROMOTED_SILVER_INDEX: usize = 25;
pub const MATERIAL_WHITE_PROMOTED_BISHOP_INDEX: usize = 26;
pub const MATERIAL_WHITE_PROMOTED_ROOK_INDEX: usize = 27;

// Positional features (piece-square tables)
pub const PST_PAWN_MG_START: usize = 28;
pub const PST_PAWN_EG_START: usize = PST_PAWN_MG_START + 81;
pub const PST_LANCE_MG_START: usize = PST_PAWN_EG_START + 81;
pub const PST_LANCE_EG_START: usize = PST_LANCE_MG_START + 81;
pub const PST_KNIGHT_MG_START: usize = PST_LANCE_EG_START + 81;
pub const PST_KNIGHT_EG_START: usize = PST_KNIGHT_MG_START + 81;
pub const PST_SILVER_MG_START: usize = PST_KNIGHT_EG_START + 81;
pub const PST_SILVER_EG_START: usize = PST_SILVER_MG_START + 81;
pub const PST_GOLD_MG_START: usize = PST_SILVER_EG_START + 81;
pub const PST_GOLD_EG_START: usize = PST_GOLD_MG_START + 81;
pub const PST_BISHOP_MG_START: usize = PST_GOLD_EG_START + 81;
pub const PST_BISHOP_EG_START: usize = PST_BISHOP_MG_START + 81;
pub const PST_ROOK_MG_START: usize = PST_BISHOP_EG_START + 81;
pub const PST_ROOK_EG_START: usize = PST_ROOK_MG_START + 81;

// King safety features
pub const KING_SAFETY_CASTLE_INDEX: usize = 500;
pub const KING_SAFETY_ATTACK_INDEX: usize = 501;
pub const KING_SAFETY_THREAT_INDEX: usize = 502;
pub const KING_SAFETY_SHIELD_INDEX: usize = 503;
pub const KING_SAFETY_EXPOSURE_INDEX: usize = 504;

// Pawn structure features
pub const PAWN_STRUCTURE_CHAINS_INDEX: usize = 600;
pub const PAWN_STRUCTURE_ADVANCEMENT_INDEX: usize = 601;
pub const PAWN_STRUCTURE_ISOLATION_INDEX: usize = 602;
pub const PAWN_STRUCTURE_PASSED_INDEX: usize = 603;
pub const PAWN_STRUCTURE_BACKWARD_INDEX: usize = 604;

// Mobility features
pub const MOBILITY_TOTAL_MOVES_INDEX: usize = 700;
pub const MOBILITY_PIECE_MOVES_INDEX: usize = 701;
pub const MOBILITY_ATTACK_MOVES_INDEX: usize = 702;
pub const MOBILITY_DEFENSE_MOVES_INDEX: usize = 703;

// Coordination features
pub const COORDINATION_CONNECTED_ROOKS_INDEX: usize = 800;
pub const COORDINATION_BISHOP_PAIR_INDEX: usize = 801;
pub const COORDINATION_ATTACK_PATTERNS_INDEX: usize = 802;
pub const COORDINATION_PIECE_SUPPORT_INDEX: usize = 803;

// Center control features
pub const CENTER_CONTROL_CENTER_SQUARES_INDEX: usize = 900;
pub const CENTER_CONTROL_OUTPOST_INDEX: usize = 901;
pub const CENTER_CONTROL_SPACE_INDEX: usize = 902;

// Development features
pub const DEVELOPMENT_MAJOR_PIECES_INDEX: usize = 1000;
pub const DEVELOPMENT_MINOR_PIECES_INDEX: usize = 1001;
pub const DEVELOPMENT_CASTLING_INDEX: usize = 1002;

// ============================================================================
// King Safety Configuration
// ============================================================================

/// Configuration for advanced king safety evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KingSafetyConfig {
    /// Enable or disable advanced king safety evaluation
    pub enabled: bool,
    /// Weight for castle structure evaluation
    pub castle_weight: f32,
    /// Weight for attack analysis
    pub attack_weight: f32,
    /// Weight for threat evaluation
    pub threat_weight: f32,
    /// Phase adjustment factor for endgame
    pub phase_adjustment: f32,
    /// Enable performance mode for fast evaluation
    pub performance_mode: bool,
    /// Minimum quality required to treat a castle as fully formed
    #[serde(default = "KingSafetyConfig::default_castle_quality_threshold")]
    pub castle_quality_threshold: f32,
    /// Minimum quality below which the king is considered bare
    #[serde(default = "KingSafetyConfig::default_partial_castle_threshold")]
    pub partial_castle_threshold: f32,
    /// Penalty applied when a partial castle is detected
    #[serde(default = "KingSafetyConfig::default_partial_castle_penalty")]
    pub partial_castle_penalty: TaperedScore,
    /// Penalty applied when no meaningful castle is present
    #[serde(default = "KingSafetyConfig::default_bare_king_penalty")]
    pub bare_king_penalty: TaperedScore,
    /// Bonus applied proportional to defender coverage ratio
    #[serde(default = "KingSafetyConfig::default_coverage_bonus")]
    pub coverage_bonus: TaperedScore,
    /// Bonus applied proportional to pawn-shield coverage ratio
    #[serde(default = "KingSafetyConfig::default_pawn_shield_bonus")]
    pub pawn_shield_bonus: TaperedScore,
    /// Bonus applied proportional to core (primary) defender retention
    #[serde(default = "KingSafetyConfig::default_primary_bonus")]
    pub primary_bonus: TaperedScore,
    /// Penalty applied per missing primary defender
    #[serde(default = "KingSafetyConfig::default_primary_defender_penalty")]
    pub primary_defender_penalty: TaperedScore,
    /// Penalty applied per missing pawn shield element
    #[serde(default = "KingSafetyConfig::default_pawn_shield_penalty")]
    pub pawn_shield_penalty: TaperedScore,
    /// Penalty applied when the king is largely exposed (very low quality)
    #[serde(default = "KingSafetyConfig::default_exposed_king_penalty")]
    pub exposed_king_penalty: TaperedScore,
    /// Weighting for combining pattern-derived coverage with zone coverage
    #[serde(default = "KingSafetyConfig::default_pattern_coverage_weight")]
    pub pattern_coverage_weight: f32,
    #[serde(default = "KingSafetyConfig::default_zone_coverage_weight")]
    pub zone_coverage_weight: f32,
    /// Weighting for combining pawn shield sources
    #[serde(default = "KingSafetyConfig::default_pattern_shield_weight")]
    pub pattern_shield_weight: f32,
    #[serde(default = "KingSafetyConfig::default_zone_shield_weight")]
    pub zone_shield_weight: f32,
    /// Exposure blending weights
    #[serde(default = "KingSafetyConfig::default_exposure_zone_weight")]
    pub exposure_zone_weight: f32,
    #[serde(default = "KingSafetyConfig::default_exposure_shield_weight")]
    pub exposure_shield_weight: f32,
    #[serde(default = "KingSafetyConfig::default_exposure_primary_weight")]
    pub exposure_primary_weight: f32,
    /// Additional penalty when opponent pieces occupy the king zone
    #[serde(default = "KingSafetyConfig::default_infiltration_penalty")]
    pub infiltration_penalty: TaperedScore,
}

impl Default for KingSafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            castle_weight: 0.3,
            attack_weight: 0.3,
            threat_weight: 0.2,
            phase_adjustment: 0.8,
            performance_mode: true,
            castle_quality_threshold: Self::default_castle_quality_threshold(),
            partial_castle_threshold: Self::default_partial_castle_threshold(),
            partial_castle_penalty: Self::default_partial_castle_penalty(),
            bare_king_penalty: Self::default_bare_king_penalty(),
            coverage_bonus: Self::default_coverage_bonus(),
            pawn_shield_bonus: Self::default_pawn_shield_bonus(),
            primary_bonus: Self::default_primary_bonus(),
            primary_defender_penalty: Self::default_primary_defender_penalty(),
            pawn_shield_penalty: Self::default_pawn_shield_penalty(),
            exposed_king_penalty: Self::default_exposed_king_penalty(),
            pattern_coverage_weight: Self::default_pattern_coverage_weight(),
            zone_coverage_weight: Self::default_zone_coverage_weight(),
            pattern_shield_weight: Self::default_pattern_shield_weight(),
            zone_shield_weight: Self::default_zone_shield_weight(),
            exposure_zone_weight: Self::default_exposure_zone_weight(),
            exposure_shield_weight: Self::default_exposure_shield_weight(),
            exposure_primary_weight: Self::default_exposure_primary_weight(),
            infiltration_penalty: Self::default_infiltration_penalty(),
        }
    }
}

impl KingSafetyConfig {
    fn default_castle_quality_threshold() -> f32 {
        0.75
    }

    fn default_partial_castle_threshold() -> f32 {
        0.4
    }

    fn default_partial_castle_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-60, -30)
    }

    fn default_bare_king_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-160, -80)
    }

    fn default_coverage_bonus() -> TaperedScore {
        TaperedScore::new_tapered(40, 20)
    }

    fn default_pawn_shield_bonus() -> TaperedScore {
        TaperedScore::new_tapered(60, 30)
    }

    fn default_primary_bonus() -> TaperedScore {
        TaperedScore::new_tapered(50, 20)
    }

    fn default_primary_defender_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-80, -40)
    }

    fn default_pawn_shield_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-30, -15)
    }

    fn default_exposed_king_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-120, -60)
    }

    fn default_pattern_coverage_weight() -> f32 {
        0.6
    }

    fn default_zone_coverage_weight() -> f32 {
        0.4
    }

    fn default_pattern_shield_weight() -> f32 {
        0.5
    }

    fn default_zone_shield_weight() -> f32 {
        0.5
    }

    fn default_exposure_zone_weight() -> f32 {
        0.5
    }

    fn default_exposure_shield_weight() -> f32 {
        0.3
    }

    fn default_exposure_primary_weight() -> f32 {
        0.2
    }

    fn default_infiltration_penalty() -> TaperedScore {
        TaperedScore::new_tapered(-90, -45)
    }
}

// ============================================================================
// Tapered Evaluation Configuration
// ============================================================================

/// Configuration for tapered evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaperedEvaluationConfig {
    /// Enable or disable tapered evaluation
    pub enabled: bool,
    /// Cache game phase calculation per search node
    pub cache_game_phase: bool,
    /// Maximum number of phase entries to retain in the local cache
    #[serde(default = "TaperedEvaluationConfig::default_phase_cache_size")]
    pub phase_cache_size: usize,
    /// Use SIMD optimizations (future feature)
    pub use_simd: bool,
    /// Memory pool size for TaperedScore objects
    pub memory_pool_size: usize,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// King safety evaluation configuration
    pub king_safety: KingSafetyConfig,
}

impl Default for TaperedEvaluationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_game_phase: true,
            phase_cache_size: Self::default_phase_cache_size(),
            use_simd: false,
            memory_pool_size: 1000,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }
}

impl TaperedEvaluationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn default_phase_cache_size() -> usize {
        4
    }

    /// Create a configuration with tapered evaluation disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            cache_game_phase: false,
            phase_cache_size: 0,
            use_simd: false,
            memory_pool_size: 0,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            cache_game_phase: true,
            phase_cache_size: 8,
            use_simd: false,
            memory_pool_size: 2000,
            enable_performance_monitoring: true,
            king_safety: KingSafetyConfig::default(),
        }
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            cache_game_phase: false,
            phase_cache_size: 1,
            use_simd: false,
            memory_pool_size: 100,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }
}

// ============================================================================
// Evaluation Metrics and Statistics
// ============================================================================

/// Evaluation metrics for performance monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct EvaluationMetrics {
    pub total_evaluations: u64,
    pub average_evaluation_time_ns: u64,
    pub component_evaluation_times: std::collections::HashMap<String, u64>,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Phase calculation time in nanoseconds
    pub phase_calc_time_ns: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tapered_score() {
        let score = TaperedScore::new(100);
        assert_eq!(score.mg, 100);
        assert_eq!(score.eg, 100);
    }

    #[test]
    fn test_tapered_score_interpolation() {
        let score = TaperedScore::new_tapered(200, 100);
        // At full opening (phase = GAME_PHASE_MAX), should return mg
        assert_eq!(score.interpolate(GAME_PHASE_MAX), 200);
        // At full endgame (phase = 0), should return eg
        assert_eq!(score.interpolate(0), 100);
        // At midpoint, should return average
        assert_eq!(score.interpolate(GAME_PHASE_MAX / 2), 150);
    }

    #[test]
    fn test_tapered_score_arithmetic() {
        let a = TaperedScore::new_tapered(100, 50);
        let b = TaperedScore::new_tapered(200, 100);
        let sum = a + b;
        assert_eq!(sum.mg, 300);
        assert_eq!(sum.eg, 150);
        
        let neg = -a;
        assert_eq!(neg.mg, -100);
        assert_eq!(neg.eg, -50);
    }

    #[test]
    fn test_king_safety_config() {
        let config = KingSafetyConfig::default();
        assert!(config.enabled);
        assert_eq!(config.castle_weight, 0.3);
    }

    #[test]
    fn test_tapered_evaluation_config() {
        let config = TaperedEvaluationConfig::default();
        assert!(config.enabled);
        assert!(config.cache_game_phase);
    }
}

