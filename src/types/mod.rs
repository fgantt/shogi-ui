//! Types Module
//!
//! This module is a re-export hub for all type definitions, organized into focused sub-modules.
//! As part of Task 1.0: File Modularization and Structure Improvements, types have been
//! split into logical groups for better organization and maintainability.
//!
//! # Module Structure
//!
//! - **`core`**: Core domain types (Player, PieceType, Position, Piece, Move)
//! - **`board`**: Board representation types (CapturedPieces, GamePhase)
//! - **`search`**: Search-related types (configs, stats, quiescence, null-move, LMR, IID, etc.)
//! - **`evaluation`**: Evaluation-related types (TaperedScore, feature indices, constants)
//! - **`patterns`**: Pattern recognition types (TacticalIndicators, AttackConfig, etc.)
//! - **`transposition`**: Transposition table types (TranspositionEntry)
//! - **`all`**: Legacy file containing all original type definitions for backward compatibility
//!
//! # Usage
//!
//! New code should import from the focused sub-modules:
//! ```rust
//! use crate::types::core::{Move, Piece, Player};
//! use crate::types::search::{NullMoveConfig, QuiescenceConfig};
//! ```
//!
//! For backward compatibility, all types are also available at the module root:
//! ```rust
//! use crate::types::{Move, Player, NullMoveConfig}; // Still works
//! ```
//!
//! # Migration Status
//!
//! The following types have been extracted to sub-modules and are available via explicit
//! re-exports (which take precedence over `all::*`):
//! - Core types: Player, PieceType, Position, Piece, Move
//! - Board types: CapturedPieces, GamePhase
//! - Search types: All search-related configs, stats, and enums
//! - Evaluation types: TaperedScore, feature indices, constants
//! - Pattern types: TacticalIndicators, AttackConfig, PatternRecognitionStats
//! - Transposition types: TranspositionEntry
//!
//! Once all imports are updated to use sub-modules (Task 1.9), duplicate definitions
//! in `all.rs` can be removed.

// Core domain types
pub mod core;
pub use core::{Move, Piece, PieceType, Player, Position};

// Board representation types
pub mod board;
pub use board::{CapturedPieces, GamePhase};

// Search-related types
pub mod search;
pub use search::{
    AdvancedReductionConfig, AdaptiveTuningConfig, AdaptiveTuningStats, AdvancedReductionStrategy,
    AspirationWindowConfig, AspirationWindowPlayingStyle, AspirationWindowStats,
    ConditionalExemptionConfig, CoreSearchMetrics, DynamicReductionFormula, EntrySource,
    EscapeMoveConfig, EscapeMoveStats, IIDBoardState, IIDConfig, IIDDepthStrategy, IIDOverheadStats,
    IIDPreset, IIDStats, LMRConfig, LMRPhaseStats, LMRPlayingStyle, LMRStats, MoveOrderingEffectivenessStats,
    MoveType, NullMoveConfig, NullMovePreset, NullMoveReductionStrategy, NullMoveStats,
    PositionClassification, PositionClassificationConfig, PositionClassificationStats,
    PositionComplexity, PruningDecision, PruningEffectiveness, PruningFrequencyStats,
    PruningParameters, PruningStatistics, QuiescenceConfig, QuiescenceEntry, QuiescenceStats,
    SearchPerformanceMetrics, SearchState, TimeAllocationStrategy, TimeBudgetStats,
    TimeManagementConfig, TuningAggressiveness, TTReplacementPolicy, TranspositionFlag,
    WindowSizeByPositionType,
};


// Evaluation-related types
pub mod evaluation;
pub use evaluation::{
    EvaluationMetrics, GAME_PHASE_MAX, KingSafetyConfig, PIECE_PHASE_VALUES, TaperedEvaluationConfig,
    TaperedScore,
    // Feature indices
    CENTER_CONTROL_CENTER_SQUARES_INDEX, CENTER_CONTROL_OUTPOST_INDEX, CENTER_CONTROL_SPACE_INDEX,
    COORDINATION_ATTACK_PATTERNS_INDEX, COORDINATION_BISHOP_PAIR_INDEX, COORDINATION_CONNECTED_ROOKS_INDEX,
    COORDINATION_PIECE_SUPPORT_INDEX, DEVELOPMENT_CASTLING_INDEX, DEVELOPMENT_MAJOR_PIECES_INDEX,
    DEVELOPMENT_MINOR_PIECES_INDEX, KING_SAFETY_ATTACK_INDEX, KING_SAFETY_CASTLE_INDEX,
    KING_SAFETY_EXPOSURE_INDEX, KING_SAFETY_SHIELD_INDEX, KING_SAFETY_THREAT_INDEX,
    MATERIAL_BISHOP_INDEX, MATERIAL_GOLD_INDEX, MATERIAL_KING_INDEX, MATERIAL_KNIGHT_INDEX,
    MATERIAL_LANCE_INDEX, MATERIAL_PAWN_INDEX, MATERIAL_PROMOTED_BISHOP_INDEX,
    MATERIAL_PROMOTED_KNIGHT_INDEX, MATERIAL_PROMOTED_LANCE_INDEX, MATERIAL_PROMOTED_PAWN_INDEX,
    MATERIAL_PROMOTED_ROOK_INDEX, MATERIAL_PROMOTED_SILVER_INDEX, MATERIAL_ROOK_INDEX,
    MATERIAL_SILVER_INDEX, MATERIAL_WHITE_BISHOP_INDEX, MATERIAL_WHITE_GOLD_INDEX,
    MATERIAL_WHITE_KING_INDEX, MATERIAL_WHITE_KNIGHT_INDEX, MATERIAL_WHITE_LANCE_INDEX,
    MATERIAL_WHITE_PAWN_INDEX, MATERIAL_WHITE_PROMOTED_BISHOP_INDEX, MATERIAL_WHITE_PROMOTED_KNIGHT_INDEX,
    MATERIAL_WHITE_PROMOTED_LANCE_INDEX, MATERIAL_WHITE_PROMOTED_PAWN_INDEX, MATERIAL_WHITE_PROMOTED_ROOK_INDEX,
    MATERIAL_WHITE_PROMOTED_SILVER_INDEX, MATERIAL_WHITE_ROOK_INDEX, MATERIAL_WHITE_SILVER_INDEX,
    MOBILITY_ATTACK_MOVES_INDEX, MOBILITY_DEFENSE_MOVES_INDEX, MOBILITY_PIECE_MOVES_INDEX,
    MOBILITY_TOTAL_MOVES_INDEX, NUM_EG_FEATURES, NUM_EVAL_FEATURES, NUM_MG_FEATURES,
    PAWN_STRUCTURE_ADVANCEMENT_INDEX, PAWN_STRUCTURE_BACKWARD_INDEX, PAWN_STRUCTURE_CHAINS_INDEX,
    PAWN_STRUCTURE_ISOLATION_INDEX, PAWN_STRUCTURE_PASSED_INDEX, PST_BISHOP_EG_START,
    PST_BISHOP_MG_START, PST_GOLD_EG_START, PST_GOLD_MG_START, PST_KNIGHT_EG_START,
    PST_KNIGHT_MG_START, PST_LANCE_EG_START, PST_LANCE_MG_START, PST_PAWN_EG_START,
    PST_PAWN_MG_START, PST_ROOK_EG_START, PST_ROOK_MG_START, PST_SILVER_EG_START,
    PST_SILVER_MG_START,
};

// Pattern recognition types
pub mod patterns;
pub use patterns::{AttackConfig, PatternRecognitionStats, TacticalIndicators};

// Transposition table types
pub mod transposition;
pub use transposition::TranspositionEntry;
// Note: QuiescenceEntry is in search.rs, not transposition.rs

// Re-export everything from the original types.rs (now all.rs) for backward compatibility
// This ensures all existing code continues to work during the migration.
// Explicit re-exports above take precedence over `all::*` for extracted types.
pub mod all;
pub use all::*;

