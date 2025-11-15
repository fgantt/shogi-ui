//! Telemetry Collection and Reporting
//!
//! This module provides telemetry collection, reporting, and export functionality
//! for the evaluation system. It handles telemetry data aggregation, formatting,
//! and export for tuning and analysis.
//!
//! Extracted from `integration.rs` and `statistics.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::evaluation::statistics::EvaluationTelemetry;
use std::collections::HashMap;

// ============================================================================
// Telemetry Export and Reporting
// ============================================================================

impl EvaluationTelemetry {
    /// Export telemetry data in format suitable for tuning
    ///
    /// Converts telemetry data into a HashMap format that can be used by
    /// tuning algorithms. Includes weight contributions and component scores.
    pub fn export_for_tuning(&self) -> HashMap<String, f64> {
        let mut tuning_data = HashMap::new();

        // Export weight contributions
        for (component, contribution) in &self.weight_contributions {
            tuning_data.insert(
                format!("weight_contribution_{}", component),
                *contribution as f64,
            );
        }

        // Export component scores if available
        // Note: TaperedEvaluationSnapshot doesn't have mg/eg fields, so we skip them

        if let Some(material) = &self.material {
            tuning_data.insert(
                "material_score".to_string(),
                material.phase_weighted_total as f64,
            );
        }

        if let Some(position_features) = &self.position_features {
            tuning_data.insert(
                "king_safety_evals".to_string(),
                position_features.king_safety_evals as f64,
            );
            tuning_data.insert(
                "pawn_structure_evals".to_string(),
                position_features.pawn_structure_evals as f64,
            );
            tuning_data.insert(
                "mobility_evals".to_string(),
                position_features.mobility_evals as f64,
            );
        }

        tuning_data
    }

    /// Export telemetry as JSON string
    ///
    /// Serializes the telemetry data to JSON format for external analysis.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get a summary of telemetry data
    ///
    /// Returns a human-readable summary of the telemetry data.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(material) = &self.material {
            parts.push(format!("Material: {} cp", material.phase_weighted_total));
        }

        if let Some(pst) = &self.pst {
            parts.push(format!("PST: {} mg / {} eg", pst.total_mg, pst.total_eg));
        }

        if let Some(position_features) = &self.position_features {
            parts.push(format!(
                "Position Features: {} evals",
                position_features.king_safety_evals
                    + position_features.pawn_structure_evals
                    + position_features.mobility_evals
            ));
        }

        if !self.weight_contributions.is_empty() {
            parts.push(format!(
                "Weight Contributions: {} components",
                self.weight_contributions.len()
            ));
        }

        if parts.is_empty() {
            "No telemetry data available".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Get weight contributions as a formatted string
    ///
    /// Returns a formatted string showing all weight contributions.
    pub fn weight_contributions_summary(&self) -> String {
        if self.weight_contributions.is_empty() {
            return "No weight contributions".to_string();
        }

        let mut parts: Vec<String> = self
            .weight_contributions
            .iter()
            .map(|(component, contribution)| {
                format!("{}: {:.2}%", component, contribution * 100.0)
            })
            .collect();

        parts.sort();
        format!("Weight Contributions: {}", parts.join(", "))
    }
}

// ============================================================================
// Telemetry Aggregation
// ============================================================================

/// Aggregate multiple telemetry snapshots
///
/// Combines telemetry data from multiple evaluations into aggregate statistics.
pub struct TelemetryAggregator {
    snapshots: Vec<EvaluationTelemetry>,
}

impl TelemetryAggregator {
    /// Create a new telemetry aggregator
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// Add a telemetry snapshot
    pub fn add(&mut self, telemetry: EvaluationTelemetry) {
        self.snapshots.push(telemetry);
    }

    /// Get the number of snapshots
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    /// Calculate average weight contributions across all snapshots
    pub fn average_weight_contributions(&self) -> HashMap<String, f32> {
        let mut aggregated = HashMap::new();
        let count = self.snapshots.len() as f32;

        if count == 0.0 {
            return aggregated;
        }

        for snapshot in &self.snapshots {
            for (component, contribution) in &snapshot.weight_contributions {
                *aggregated.entry(component.clone()).or_insert(0.0) += contribution;
            }
        }

        for contribution in aggregated.values_mut() {
            *contribution /= count;
        }

        aggregated
    }

    /// Generate aggregate report
    pub fn generate_report(&self) -> String {
        if self.snapshots.is_empty() {
            return "No telemetry data collected".to_string();
        }

        let avg_contributions = self.average_weight_contributions();
        let mut parts = vec![format!("Total snapshots: {}", self.snapshots.len())];

        if !avg_contributions.is_empty() {
            let contribution_str: Vec<String> = avg_contributions
                .iter()
                .map(|(component, contribution)| {
                    format!("{}: {:.2}%", component, contribution * 100.0)
                })
                .collect();
            parts.push(format!("Average contributions: {}", contribution_str.join(", ")));
        }

        parts.join("\n")
    }

    /// Export aggregated telemetry as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let aggregated = self.average_weight_contributions();
        serde_json::to_string_pretty(&aggregated)
    }
}

impl Default for TelemetryAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Telemetry Collection Helpers
// ============================================================================

/// Helper for collecting telemetry during evaluation
///
/// This struct helps collect component contributions and build telemetry
/// during the evaluation process.
pub struct TelemetryCollector {
    component_contributions: HashMap<String, f32>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            component_contributions: HashMap::new(),
        }
    }

    /// Record a component contribution
    pub fn record_contribution(&mut self, component: String, contribution: f32) {
        self.component_contributions.insert(component, contribution);
    }

    /// Get all component contributions
    pub fn contributions(&self) -> &HashMap<String, f32> {
        &self.component_contributions
    }

    /// Clear all contributions
    pub fn clear(&mut self) {
        self.component_contributions.clear();
    }

    /// Get the number of components tracked
    pub fn component_count(&self) -> usize {
        self.component_contributions.len()
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_export_for_tuning() {
        let mut telemetry = EvaluationTelemetry::default();
        telemetry.weight_contributions.insert("material".to_string(), 0.3);
        telemetry.weight_contributions.insert("pst".to_string(), 0.2);

        let tuning_data = telemetry.export_for_tuning();
        assert!(tuning_data.contains_key("weight_contribution_material"));
        assert!(tuning_data.contains_key("weight_contribution_pst"));
    }

    #[test]
    fn test_telemetry_summary() {
        let telemetry = EvaluationTelemetry::default();
        let summary = telemetry.summary();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_telemetry_aggregator() {
        let mut aggregator = TelemetryAggregator::new();
        assert_eq!(aggregator.count(), 0);

        let mut telemetry1 = EvaluationTelemetry::default();
        telemetry1.weight_contributions.insert("material".to_string(), 0.3);
        aggregator.add(telemetry1);

        let mut telemetry2 = EvaluationTelemetry::default();
        telemetry2.weight_contributions.insert("material".to_string(), 0.5);
        aggregator.add(telemetry2);

        assert_eq!(aggregator.count(), 2);
        let avg = aggregator.average_weight_contributions();
        assert!((avg.get("material").unwrap() - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_telemetry_collector() {
        let mut collector = TelemetryCollector::new();
        collector.record_contribution("material".to_string(), 0.3);
        collector.record_contribution("pst".to_string(), 0.2);

        assert_eq!(collector.component_count(), 2);
        assert_eq!(collector.contributions().get("material"), Some(&0.3));
    }
}

