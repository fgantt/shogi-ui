//! Component Orchestration and Coordination
//!
//! This module provides component orchestration logic for the evaluation system.
//! It handles component coordination decisions, conflict resolution, and phase-aware
//! gating to avoid double-counting and ensure optimal evaluation order.
//!
//! Extracted from `integration.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::evaluation::config::PhaseBoundaryConfig;
use crate::evaluation::integration::{CenterControlPrecedence, ComponentFlags};
use std::collections::HashMap;

// ============================================================================
// Component Coordination Decisions
// ============================================================================

/// Component coordination decisions for a single evaluation
#[derive(Debug, Clone)]
pub struct ComponentCoordination {
    /// Skip passed pawn evaluation in position_features (endgame patterns handles it)
    pub skip_passed_pawn_evaluation: bool,
    /// Skip development evaluation in position_features (opening principles handles it)
    pub skip_development_in_features: bool,
    /// Skip center control in position_features (positional patterns handles it)
    pub skip_center_control_in_features: bool,
    /// Skip center control in positional_patterns (position features handles it)
    pub skip_center_control_in_positional: bool,
    /// Whether to evaluate opening principles (phase-aware)
    pub evaluate_opening_principles: bool,
    /// Whether to evaluate endgame patterns (phase-aware)
    pub evaluate_endgame_patterns: bool,
    /// Opening fade factor for gradual transitions (0.0-1.0)
    pub opening_fade_factor: f32,
    /// Endgame fade factor for gradual transitions (0.0-1.0)
    pub endgame_fade_factor: f32,
}

impl ComponentCoordination {
    /// Create coordination decisions based on configuration and phase
    pub fn new(
        components: &ComponentFlags,
        phase: i32,
        phase_boundaries: &PhaseBoundaryConfig,
        center_control_precedence: CenterControlPrecedence,
        enable_gradual_phase_transitions: bool,
    ) -> Self {
        let endgame_threshold = phase_boundaries.endgame_threshold;
        let opening_threshold = phase_boundaries.opening_threshold;

        // Passed pawn coordination: Skip in position_features when endgame_patterns handles it
        let skip_passed_pawn_evaluation =
            components.endgame_patterns && phase < endgame_threshold;

        // Development coordination: Skip in position_features when opening_principles handles it
        let skip_development_in_features =
            components.opening_principles && phase >= opening_threshold;

        // Center control coordination: Use precedence to determine which component evaluates it
        let (skip_center_control_in_features, skip_center_control_in_positional) =
            if components.position_features && components.positional_patterns {
                match center_control_precedence {
                    CenterControlPrecedence::PositionalPatterns => (true, false),
                    CenterControlPrecedence::PositionFeatures => (false, true),
                    CenterControlPrecedence::Both => (false, false),
                }
            } else {
                (false, false)
            };

        // Phase-aware gating for opening principles
        let evaluate_opening_principles = components.opening_principles && phase >= opening_threshold;

        // Phase-aware gating for endgame patterns
        let evaluate_endgame_patterns = components.endgame_patterns && phase < endgame_threshold;

        // Calculate fade factors for gradual transitions
        let opening_fade_factor = if enable_gradual_phase_transitions {
            phase_boundaries.calculate_opening_fade_factor(phase)
        } else {
            if evaluate_opening_principles {
                1.0
            } else {
                0.0
            }
        };

        let endgame_fade_factor = if enable_gradual_phase_transitions {
            phase_boundaries.calculate_endgame_fade_factor(phase)
        } else {
            if evaluate_endgame_patterns {
                1.0
            } else {
                0.0
            }
        };

        Self {
            skip_passed_pawn_evaluation,
            skip_development_in_features,
            skip_center_control_in_features,
            skip_center_control_in_positional,
            evaluate_opening_principles,
            evaluate_endgame_patterns,
            opening_fade_factor,
            endgame_fade_factor,
        }
    }

    /// Check if a component should be evaluated based on coordination decisions
    pub fn should_evaluate_component(&self, component: ComponentType) -> bool {
        match component {
            ComponentType::PositionFeaturesPassedPawns => !self.skip_passed_pawn_evaluation,
            ComponentType::PositionFeaturesDevelopment => !self.skip_development_in_features,
            ComponentType::PositionFeaturesCenterControl => !self.skip_center_control_in_features,
            ComponentType::PositionalPatternsCenterControl => !self.skip_center_control_in_positional,
            ComponentType::OpeningPrinciples => self.evaluate_opening_principles,
            ComponentType::EndgamePatterns => self.evaluate_endgame_patterns,
            _ => true, // Other components are not affected by coordination
        }
    }
}

/// Component types for coordination decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Material,
    PieceSquareTables,
    PositionFeaturesKingSafety,
    PositionFeaturesPawnStructure,
    PositionFeaturesPassedPawns,
    PositionFeaturesMobility,
    PositionFeaturesCenterControl,
    PositionFeaturesDevelopment,
    OpeningPrinciples,
    EndgamePatterns,
    TacticalPatterns,
    PositionalPatterns,
    PositionalPatternsCenterControl,
    CastlePatterns,
}

// ============================================================================
// Component Evaluation Order
// ============================================================================

/// Determines the optimal order for evaluating components
pub struct ComponentOrder {
    /// Ordered list of components to evaluate
    pub order: Vec<ComponentType>,
}

impl ComponentOrder {
    /// Get the default evaluation order
    ///
    /// Components are ordered to:
    /// 1. Evaluate fundamental components first (material, PST)
    /// 2. Evaluate position features
    /// 3. Evaluate phase-specific components (opening, endgame)
    /// 4. Evaluate pattern recognizers (tactical, positional, castle)
    pub fn default() -> Self {
        Self {
            order: vec![
                ComponentType::Material,
                ComponentType::PieceSquareTables,
                ComponentType::PositionFeaturesKingSafety,
                ComponentType::PositionFeaturesPawnStructure,
                ComponentType::PositionFeaturesPassedPawns,
                ComponentType::PositionFeaturesMobility,
                ComponentType::PositionFeaturesCenterControl,
                ComponentType::PositionFeaturesDevelopment,
                ComponentType::OpeningPrinciples,
                ComponentType::EndgamePatterns,
                ComponentType::TacticalPatterns,
                ComponentType::PositionalPatterns,
                ComponentType::CastlePatterns,
            ],
        }
    }

    /// Get evaluation order filtered by enabled components and coordination decisions
    pub fn filtered(
        components: &ComponentFlags,
        coordination: &ComponentCoordination,
    ) -> Vec<ComponentType> {
        let default_order = Self::default();
        default_order
            .order
            .into_iter()
            .filter(|component_type| {
                // Check if component is enabled
                let enabled = match component_type {
                    ComponentType::Material => components.material,
                    ComponentType::PieceSquareTables => components.piece_square_tables,
                    ComponentType::PositionFeaturesKingSafety
                    | ComponentType::PositionFeaturesPawnStructure
                    | ComponentType::PositionFeaturesPassedPawns
                    | ComponentType::PositionFeaturesMobility
                    | ComponentType::PositionFeaturesCenterControl
                    | ComponentType::PositionFeaturesDevelopment => components.position_features,
                    ComponentType::OpeningPrinciples => components.opening_principles,
                    ComponentType::EndgamePatterns => components.endgame_patterns,
                    ComponentType::TacticalPatterns => components.tactical_patterns,
                    ComponentType::PositionalPatterns
                    | ComponentType::PositionalPatternsCenterControl => {
                        components.positional_patterns
                    }
                    ComponentType::CastlePatterns => components.castle_patterns,
                };

                if !enabled {
                    return false;
                }

                // Check coordination decisions
                coordination.should_evaluate_component(*component_type)
            })
            .collect()
    }
}

// ============================================================================
// Component Contribution Tracking
// ============================================================================

/// Helper for tracking component contributions during evaluation
pub struct ComponentContributionTracker {
    contributions: HashMap<String, i32>,
    total_absolute: i32,
}

impl ComponentContributionTracker {
    /// Create a new contribution tracker
    pub fn new() -> Self {
        Self {
            contributions: HashMap::new(),
            total_absolute: 0,
        }
    }

    /// Record a component contribution
    pub fn record(&mut self, component: &str, contribution: i32) {
        self.contributions.insert(component.to_string(), contribution);
        self.total_absolute += contribution.abs();
    }

    /// Convert absolute contributions to percentages
    pub fn to_percentages(&self) -> HashMap<String, f32> {
        let mut percentages = HashMap::new();

        if self.total_absolute == 0 {
            return percentages;
        }

        for (component, contribution) in &self.contributions {
            let percentage = (contribution.abs() as f32) / (self.total_absolute as f32);
            percentages.insert(component.clone(), percentage);
        }

        percentages
    }

    /// Get all contributions
    pub fn contributions(&self) -> &HashMap<String, i32> {
        &self.contributions
    }

    /// Clear all contributions
    pub fn clear(&mut self) {
        self.contributions.clear();
        self.total_absolute = 0;
    }
}

impl Default for ComponentContributionTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Conflict Resolution
// ============================================================================

/// Resolve conflicts between components based on precedence rules
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve center control conflict
    ///
    /// Returns which component should evaluate center control based on precedence.
    pub fn resolve_center_control_conflict(
        components: &ComponentFlags,
        precedence: CenterControlPrecedence,
    ) -> (bool, bool) {
        // (evaluate_in_features, evaluate_in_positional)
        if !components.position_features || !components.positional_patterns {
            // No conflict if only one is enabled
            return (components.position_features, components.positional_patterns);
        }

        match precedence {
            CenterControlPrecedence::PositionalPatterns => (false, true),
            CenterControlPrecedence::PositionFeatures => (true, false),
            CenterControlPrecedence::Both => (true, true),
        }
    }

    /// Check if there's a development conflict
    pub fn has_development_conflict(
        components: &ComponentFlags,
        phase: i32,
        opening_threshold: i32,
    ) -> bool {
        components.position_features
            && components.opening_principles
            && phase >= opening_threshold
    }

    /// Check if there's a passed pawn conflict
    pub fn has_passed_pawn_conflict(
        components: &ComponentFlags,
        phase: i32,
        endgame_threshold: i32,
    ) -> bool {
        components.position_features
            && components.endgame_patterns
            && phase < endgame_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::config::PhaseBoundaryConfig;

    #[test]
    fn test_component_coordination() {
        let components = ComponentFlags {
            material: true,
            piece_square_tables: true,
            position_features: true,
            opening_principles: true,
            endgame_patterns: true,
            tactical_patterns: true,
            positional_patterns: true,
            castle_patterns: true,
        };
        let phase = 128; // Middlegame
        let phase_boundaries = PhaseBoundaryConfig::default();
        let coordination = ComponentCoordination::new(
            &components,
            phase,
            &phase_boundaries,
            CenterControlPrecedence::PositionalPatterns,
            false,
        );

        // In middlegame, opening principles should not be evaluated
        assert!(!coordination.evaluate_opening_principles);
        // Endgame patterns should not be evaluated
        assert!(!coordination.evaluate_endgame_patterns);
    }

    #[test]
    fn test_conflict_resolution() {
        let components = ComponentFlags {
            material: true,
            piece_square_tables: true,
            position_features: true,
            opening_principles: true,
            endgame_patterns: true,
            tactical_patterns: true,
            positional_patterns: true,
            castle_patterns: true,
        };
        let (eval_features, eval_positional) = ConflictResolver::resolve_center_control_conflict(
            &components,
            CenterControlPrecedence::PositionalPatterns,
        );
        assert!(!eval_features);
        assert!(eval_positional);
    }

    #[test]
    fn test_contribution_tracker() {
        let mut tracker = ComponentContributionTracker::new();
        tracker.record("material", 100);
        tracker.record("pst", 50);

        let percentages = tracker.to_percentages();
        // material: 100 / 150 = 0.666..., pst: 50 / 150 = 0.333...
        assert!((percentages.get("material").unwrap() - 0.666).abs() < 0.01);
        assert!((percentages.get("pst").unwrap() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_component_order() {
        let components = ComponentFlags {
            material: true,
            piece_square_tables: true,
            position_features: true,
            opening_principles: false,
            endgame_patterns: false,
            tactical_patterns: true,
            positional_patterns: true,
            castle_patterns: true,
        };
        let phase = 128;
        let phase_boundaries = PhaseBoundaryConfig::default();
        let coordination = ComponentCoordination::new(
            &components,
            phase,
            &phase_boundaries,
            CenterControlPrecedence::PositionalPatterns,
            false,
        );

        let order = ComponentOrder::filtered(&components, &coordination);
        assert!(!order.is_empty());
        // Material and PST should be first
        assert_eq!(order[0], ComponentType::Material);
        assert_eq!(order[1], ComponentType::PieceSquareTables);
    }
}

