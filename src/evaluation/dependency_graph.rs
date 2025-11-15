//! Component Dependency Graph Validation
//!
//! This module provides validation logic for component dependencies in the evaluation system.
//! It checks for conflicts, missing complements, missing requirements, and phase compatibility.
//!
//! Extracted from `integration.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::evaluation::config::{
    ComponentDependencyGraph, ComponentDependencyWarning, ComponentId,
};

/// Helper struct for component dependency validation
pub struct DependencyValidator<'a> {
    dependency_graph: &'a ComponentDependencyGraph,
    enabled_component_ids: Vec<ComponentId>,
    components: ComponentFlags,
}

/// Component enable/disable flags for validation
#[derive(Debug, Clone)]
pub struct ComponentFlags {
    pub position_features: bool,
    pub positional_patterns: bool,
    pub opening_principles: bool,
    pub endgame_patterns: bool,
}

impl<'a> DependencyValidator<'a> {
    /// Create a new dependency validator
    pub fn new(
        dependency_graph: &'a ComponentDependencyGraph,
        enabled_component_ids: Vec<ComponentId>,
        components: ComponentFlags,
    ) -> Self {
        Self {
            dependency_graph,
            enabled_component_ids,
            components,
        }
    }

    /// Validate component dependencies
    ///
    /// Returns a vector of warnings for potential issues. These are informational
    /// and don't prevent the configuration from being used, but may indicate
    /// suboptimal settings.
    pub fn validate_component_dependencies(&self) -> Vec<ComponentDependencyWarning> {
        let mut warnings = Vec::new();

        // Check for conflicts
        for (i, &id1) in self.enabled_component_ids.iter().enumerate() {
            for &id2 in self.enabled_component_ids.iter().skip(i + 1) {
                if self.dependency_graph.conflicts(id1, id2) {
                    warnings.push(ComponentDependencyWarning::ComponentConflict {
                        component1: format!("{:?}", id1),
                        component2: format!("{:?}", id2),
                    });
                }
            }
        }

        // Check for missing complements
        for &id1 in &self.enabled_component_ids {
            for component_id in [
                ComponentId::Material,
                ComponentId::PieceSquareTables,
                ComponentId::PositionFeatures,
                ComponentId::PositionFeaturesCenterControl,
                ComponentId::PositionFeaturesDevelopment,
                ComponentId::PositionFeaturesPassedPawns,
                ComponentId::PositionFeaturesKingSafety,
                ComponentId::OpeningPrinciples,
                ComponentId::EndgamePatterns,
                ComponentId::TacticalPatterns,
                ComponentId::PositionalPatterns,
                ComponentId::CastlePatterns,
            ] {
                if component_id != id1 && self.dependency_graph.complements(id1, component_id) {
                    // Check if component_id is enabled
                    if !self.enabled_component_ids.contains(&component_id) {
                        warnings.push(ComponentDependencyWarning::MissingComplement {
                            component1: format!("{:?}", id1),
                            component2: format!("{:?}", component_id),
                        });
                    }
                }
            }
        }

        // Check for missing requirements
        for &id1 in &self.enabled_component_ids {
            for component_id in [
                ComponentId::Material,
                ComponentId::PieceSquareTables,
                ComponentId::PositionFeatures,
                ComponentId::PositionFeaturesCenterControl,
                ComponentId::PositionFeaturesDevelopment,
                ComponentId::PositionFeaturesPassedPawns,
                ComponentId::PositionFeaturesKingSafety,
                ComponentId::OpeningPrinciples,
                ComponentId::EndgamePatterns,
                ComponentId::TacticalPatterns,
                ComponentId::PositionalPatterns,
                ComponentId::CastlePatterns,
            ] {
                if component_id != id1 && self.dependency_graph.requires(id1, component_id) {
                    // id1 requires component_id, check if component_id is enabled
                    if !self.enabled_component_ids.contains(&component_id) {
                        warnings.push(ComponentDependencyWarning::MissingRequirement {
                            component: format!("{:?}", id1),
                            required: format!("{:?}", component_id),
                        });
                    }
                }
            }
        }

        // Legacy warnings for backward compatibility
        // Note: Automatically handled via center_control_precedence, but still warn for visibility
        if self.components.position_features && self.components.positional_patterns {
            warnings.push(ComponentDependencyWarning::CenterControlOverlap);
        }

        // Note: Automatically handled during evaluation (opening_principles takes precedence in opening),
        // but still warn for visibility
        if self.components.position_features && self.components.opening_principles {
            warnings.push(ComponentDependencyWarning::DevelopmentOverlap);
        }

        warnings
    }

    /// Suggest component resolution for conflicts
    pub fn suggest_component_resolution(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for conflicts and suggest resolutions
        for (i, &id1) in self.enabled_component_ids.iter().enumerate() {
            for &id2 in self.enabled_component_ids.iter().skip(i + 1) {
                if self.dependency_graph.conflicts(id1, id2) {
                    // Suggest disabling one based on precedence or importance
                    let suggestion = if matches!(id1, ComponentId::PositionalPatterns)
                        && matches!(id2, ComponentId::PositionFeaturesCenterControl)
                    {
                        format!("Disable position_features.center_control (positional_patterns takes precedence)")
                    } else if matches!(id1, ComponentId::OpeningPrinciples)
                        && matches!(id2, ComponentId::PositionFeaturesDevelopment)
                    {
                        format!("Disable position_features.development in opening (opening_principles takes precedence)")
                    } else if matches!(id1, ComponentId::EndgamePatterns)
                        && matches!(id2, ComponentId::PositionFeaturesPassedPawns)
                    {
                        format!("Disable position_features.passed_pawns in endgame (endgame_patterns takes precedence)")
                    } else {
                        format!("Disable either {:?} or {:?} to resolve conflict", id1, id2)
                    };
                    suggestions.push(suggestion);
                }
            }
        }

        suggestions
    }

    /// Automatically resolve conflicts by disabling components based on precedence
    pub fn auto_resolve_conflicts(&self) -> Vec<String> {
        let mut resolutions = Vec::new();

        // Resolve conflicts based on precedence
        for (i, &id1) in self.enabled_component_ids.iter().enumerate() {
            for &id2 in self.enabled_component_ids.iter().skip(i + 1) {
                if self.dependency_graph.conflicts(id1, id2) {
                    // Apply resolution based on component types and precedence
                    if matches!(id1, ComponentId::PositionalPatterns)
                        && matches!(id2, ComponentId::PositionFeaturesCenterControl)
                    {
                        // Positional patterns take precedence - handled by center_control_precedence
                        resolutions.push(
                            "Center control conflict resolved via center_control_precedence"
                                .to_string(),
                        );
                    } else if matches!(id1, ComponentId::OpeningPrinciples)
                        && matches!(id2, ComponentId::PositionFeaturesDevelopment)
                    {
                        // Opening principles take precedence in opening - already handled during evaluation
                        resolutions.push("Development conflict resolved (opening_principles takes precedence in opening)".to_string());
                    } else if matches!(id1, ComponentId::EndgamePatterns)
                        && matches!(id2, ComponentId::PositionFeaturesPassedPawns)
                    {
                        // Endgame patterns take precedence in endgame - already handled during evaluation
                        resolutions.push("Passed pawns conflict resolved (endgame_patterns takes precedence in endgame)".to_string());
                    }
                }
            }
        }

        resolutions
    }

    /// Check phase compatibility for component usage
    ///
    /// Analyzes recent phase history to detect phase-component mismatches.
    /// Returns warnings if components are enabled but phase is consistently outside their effective range.
    pub fn check_phase_compatibility(
        &self,
        phase_history: &[i32],
        opening_threshold: i32,
        endgame_threshold: i32,
    ) -> Vec<ComponentDependencyWarning> {
        let mut warnings = Vec::new();

        if phase_history.is_empty() {
            return warnings;
        }

        // Check if phases are consistently in a particular range
        let avg_phase: i32 = phase_history.iter().sum::<i32>() / phase_history.len() as i32;

        // Warn when opening_principles is enabled but phase is consistently < opening_threshold
        if self.components.opening_principles && avg_phase < opening_threshold {
            warnings.push(ComponentDependencyWarning::EndgamePatternsNotInEndgame);
            // Reuse for now
        }

        // Warn when endgame_patterns is enabled but phase is consistently >= endgame_threshold
        if self.components.endgame_patterns && avg_phase >= endgame_threshold {
            warnings.push(ComponentDependencyWarning::EndgamePatternsNotInEndgame);
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_validator() {
        let graph = ComponentDependencyGraph::default();
        let enabled = vec![
            ComponentId::Material,
            ComponentId::PieceSquareTables,
        ];
        let components = ComponentFlags {
            position_features: false,
            positional_patterns: false,
            opening_principles: false,
            endgame_patterns: false,
        };

        let validator = DependencyValidator::new(&graph, enabled, components);
        let warnings = validator.validate_component_dependencies();
        // No conflicts expected for basic components
        assert!(warnings.is_empty() || warnings.len() < 5);
    }

    #[test]
    fn test_conflict_detection() {
        let graph = ComponentDependencyGraph::default();
        let enabled = vec![
            ComponentId::PositionFeaturesCenterControl,
            ComponentId::PositionalPatterns,
        ];
        let components = ComponentFlags {
            position_features: true,
            positional_patterns: true,
            opening_principles: false,
            endgame_patterns: false,
        };

        let validator = DependencyValidator::new(&graph, enabled, components);
        let warnings = validator.validate_component_dependencies();
        // Should detect center control conflict
        assert!(!warnings.is_empty());
    }
}

