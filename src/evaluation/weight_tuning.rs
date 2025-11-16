//! Weight Tuning Integration
//!
//! This module provides weight tuning functionality for the evaluation system.
//! It includes types and methods for optimizing evaluation weights using training
//! positions and telemetry data.
//!
//! Extracted from `integration.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::bitboards::BitboardBoard;
use crate::evaluation::config::EvaluationWeights;
use crate::evaluation::statistics::EvaluationTelemetry;
use crate::evaluation::integration::IntegratedEvaluator;
use crate::tuning::OptimizationMethod;
use crate::types::board::CapturedPieces;
use crate::types::core::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Tuning Types
// ============================================================================

/// Training position for weight tuning
///
/// Note: BitboardBoard and CapturedPieces are not serializable, so this struct
/// cannot be directly serialized. Use position hashes or FEN strings for serialization.
#[derive(Clone)]
pub struct TuningPosition {
    /// Board position
    pub board: BitboardBoard,
    /// Captured pieces
    pub captured_pieces: CapturedPieces,
    /// Player to move
    pub player: Player,
    /// Expected evaluation score from the position's perspective (normalized to -1.0 to 1.0)
    pub expected_score: f64,
    /// Game phase (0 = endgame, 256 = opening)
    pub game_phase: i32,
    /// Move number in game (1-indexed)
    pub move_number: u32,
}

/// Collection of training positions for tuning
#[derive(Clone)]
pub struct TuningPositionSet {
    /// Training positions
    pub positions: Vec<TuningPosition>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl TuningPositionSet {
    /// Create a new tuning position set
    pub fn new(positions: Vec<TuningPosition>) -> Self {
        Self {
            positions,
            metadata: HashMap::new(),
        }
    }

    /// Create an empty tuning position set
    pub fn empty() -> Self {
        Self {
            positions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a position to the set
    pub fn add_position(&mut self, position: TuningPosition) {
        self.positions.push(position);
    }

    /// Get the number of positions
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Optimization method to use
    pub method: OptimizationMethod,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
    /// Learning rate (for gradient-based methods)
    pub learning_rate: f64,
    /// K-factor for sigmoid conversion
    pub k_factor: f64,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            method: OptimizationMethod::Adam {
                learning_rate: 0.001,
                beta1: 0.9,
                beta2: 0.999,
                epsilon: 1e-8,
            },
            max_iterations: 1000,
            convergence_threshold: 1e-6,
            learning_rate: 0.001,
            k_factor: 1.0,
        }
    }
}

/// Tuning result containing optimized weights and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Optimized evaluation weights
    pub optimized_weights: EvaluationWeights,
    /// Final error value
    pub final_error: f64,
    /// Number of iterations completed
    pub iterations: usize,
    /// Convergence reason
    pub convergence_reason: ConvergenceReason,
    /// Total optimization time
    pub optimization_time: Duration,
    /// Error history across iterations
    pub error_history: Vec<f64>,
}

/// Convergence reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergenceReason {
    /// Converged successfully
    Converged,
    /// Reached maximum iterations
    MaxIterations,
    /// Early stopping triggered
    EarlyStopping,
    /// Gradient norm below threshold
    GradientNorm,
}

// ============================================================================
// Weight Tuning Methods
// ============================================================================

/// Tune evaluation weights using training positions
///
/// This function optimizes the evaluation weights to minimize the error between
/// predicted and expected scores on the training positions.
///
/// # Arguments
///
/// * `evaluator` - The evaluator to use for evaluation (will be cloned for temporary evaluations)
/// * `initial_weights` - Starting weights for optimization
/// * `position_set` - Collection of training positions with expected scores
/// * `tuning_config` - Configuration for the tuning process
///
/// # Returns
///
/// * `TuningResult` containing optimized weights and statistics
pub fn tune_weights(
    evaluator: &IntegratedEvaluator,
    initial_weights: &EvaluationWeights,
    position_set: &TuningPositionSet,
    tuning_config: &TuningConfig,
) -> Result<TuningResult, String> {
    if position_set.is_empty() {
        return Err("Position set is empty".to_string());
    }

    let start_time = std::time::Instant::now();
    let mut weights = initial_weights.to_vector();
        let mut error_history = Vec::new();
        let mut prev_error = f64::INFINITY;
        let mut patience_counter = 0;
        const EARLY_STOPPING_PATIENCE: usize = 50;

        // Simple gradient descent optimizer for component weights
        // (Simplified version - full implementation would use the tuning infrastructure's optimizers)
        for iteration in 0..tuning_config.max_iterations {
            let (error, gradients) =
                calculate_error_and_gradients(evaluator, &weights, position_set, tuning_config.k_factor);
            error_history.push(error);

            // Check for convergence
            if error < tuning_config.convergence_threshold {
                let optimized_weights = EvaluationWeights::from_vector(&weights)?;
                return Ok(TuningResult {
                    optimized_weights,
                    final_error: error,
                    iterations: iteration + 1,
                    convergence_reason: ConvergenceReason::Converged,
                    optimization_time: start_time.elapsed(),
                    error_history,
                });
            }

            // Early stopping
            if error < prev_error {
                prev_error = error;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= EARLY_STOPPING_PATIENCE {
                    let optimized_weights = EvaluationWeights::from_vector(&weights)?;
                    return Ok(TuningResult {
                        optimized_weights,
                        final_error: error,
                        iterations: iteration + 1,
                        convergence_reason: ConvergenceReason::EarlyStopping,
                        optimization_time: start_time.elapsed(),
                        error_history,
                    });
                }
            }

            // Update weights using gradient descent
            for (i, gradient) in gradients.iter().enumerate() {
                weights[i] -= tuning_config.learning_rate * gradient;
                // Clamp weights to reasonable range (0.0 to 10.0)
                weights[i] = weights[i].max(0.0).min(10.0);
            }
        }

        let optimized_weights = EvaluationWeights::from_vector(&weights)?;
        Ok(TuningResult {
            optimized_weights,
            final_error: prev_error,
            iterations: tuning_config.max_iterations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        })
    }

/// Calculate error and gradients for current weights
fn calculate_error_and_gradients(
    evaluator: &IntegratedEvaluator,
    weights: &[f64],
    position_set: &TuningPositionSet,
    k_factor: f64,
) -> (f64, Vec<f64>) {
    let mut total_error = 0.0;
    let mut gradients = vec![0.0; 10]; // 10 weights

    // Create a temporary evaluator with the specified weights
    if let Ok(_temp_weights) = EvaluationWeights::from_vector(weights) {
        // Create a new evaluator with modified weights
        // Note: This requires creating a new evaluator with the same config but different weights
        // In the full implementation, this would use a more efficient method or a setter
        let mut temp_evaluator = IntegratedEvaluator::with_config(evaluator.config().clone());
        // TODO: Add set_weights method to IntegratedEvaluator or make weights field public
        // For now, this is a placeholder that needs to be completed during integration
        // temp_evaluator.set_weights(temp_weights);

        for position in &position_set.positions {
            // Evaluate position with current weights
            // Note: This will use default weights until set_weights is implemented
            let result = temp_evaluator.evaluate_with_move_count(
                &position.board,
                position.player,
                &position.captured_pieces,
                None,
            );
            let predicted_score = result.score as f64;

                // Convert to probability using sigmoid
                let predicted_prob = sigmoid(predicted_score * k_factor);
                let expected_prob = position.expected_score;

                // Calculate error (mean squared error)
                let error = expected_prob - predicted_prob;
                total_error += error * error;

                // Calculate gradients using finite differences approximation
                // For each weight, calculate gradient contribution
                let epsilon = 1e-5;
                for i in 0..10 {
                    let mut perturbed_weights = weights.to_vec();
                    perturbed_weights[i] += epsilon;

                    if let Ok(_perturbed_eval_weights) =
                        EvaluationWeights::from_vector(&perturbed_weights)
                    {
                        let mut perturbed_evaluator = IntegratedEvaluator::with_config(evaluator.config().clone());
                        // TODO: Add set_weights method
                        // perturbed_evaluator.set_weights(perturbed_eval_weights);
                        let perturbed_result = perturbed_evaluator.evaluate_with_move_count(
                            &position.board,
                            position.player,
                            &position.captured_pieces,
                            None,
                        );
                        let perturbed_score = perturbed_result.score as f64;
                        let perturbed_prob = sigmoid(perturbed_score * k_factor);

                        let gradient_contribution =
                            (perturbed_prob - predicted_prob) / epsilon * error * (-2.0);
                        gradients[i] += gradient_contribution;
                    }
                }
            }
        }

        // Average
        let n = position_set.len() as f64;
        total_error /= n;
        for gradient in &mut gradients {
            *gradient /= n;
        }

        (total_error, gradients)
    }

/// Tune weights from accumulated telemetry
///
/// Uses accumulated telemetry to suggest weight adjustments.
pub fn tune_from_telemetry(
    telemetry_set: &[EvaluationTelemetry],
    target_contributions: Option<&HashMap<String, f32>>,
    learning_rate: f32,
) -> Result<EvaluationWeights, String> {
        if telemetry_set.is_empty() {
            return Err("Telemetry set is empty".to_string());
        }

        // Use the existing auto_balance_weights functionality
        let config = crate::evaluation::config::TaperedEvalConfig::default();
        let mut temp_config = config.clone();

        // Aggregate telemetry
        let mut aggregated_contributions = HashMap::new();
        for telemetry in telemetry_set {
            for (component, contribution) in &telemetry.weight_contributions {
                *aggregated_contributions
                    .entry(component.clone())
                    .or_insert(0.0) += contribution;
            }
        }

        // Average
        let count = telemetry_set.len() as f32;
        for contribution in aggregated_contributions.values_mut() {
            *contribution /= count;
        }

        // Use auto_balance_weights to suggest adjustments
        // (This is a simplified version - full implementation would use optimizer)
        let components = crate::evaluation::config::ComponentFlagsForValidation {
            material: true,
            piece_square_tables: true,
            position_features: true,
            tactical_patterns: true,
            positional_patterns: true,
            castle_patterns: true,
        };
        temp_config.auto_balance_weights(
            &telemetry_set[0], // Use first telemetry as representative
            &components,
            target_contributions,
            learning_rate,
        );

        Ok(temp_config.weights)
    }

/// Telemetry-to-tuning pipeline
///
/// Collects telemetry from multiple positions and converts them to a tuning position set.
pub fn telemetry_to_tuning_pipeline(
    _evaluator: &IntegratedEvaluator,
    telemetry_positions: &[(
        BitboardBoard,
        CapturedPieces,
        Player,
        EvaluationTelemetry,
        f64,
    )],
) -> TuningPositionSet {
    let mut positions = Vec::new();

    for (board, captured_pieces, player, _telemetry, expected_score) in telemetry_positions {
        // Calculate game phase
        // Note: This requires a public method on IntegratedEvaluator
        // TODO: Add public calculate_phase_cached method or pass phase as parameter
        // For now, using a placeholder - this needs to be implemented during integration
        let game_phase = 128; // Placeholder

        // Create tuning position
        let tuning_position = TuningPosition {
            board: board.clone(),
            captured_pieces: captured_pieces.clone(),
            player: *player,
            expected_score: *expected_score,
            game_phase,
            move_number: 1, // Default - should be provided if available
        };

        positions.push(tuning_position);
    }

    TuningPositionSet::new(positions)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Sigmoid function for probability conversion
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_position_set() {
        let mut set = TuningPositionSet::empty();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_tuning_config_default() {
        let config = TuningConfig::default();
        assert_eq!(config.max_iterations, 1000);
        assert!(config.convergence_threshold > 0.0);
    }

    #[test]
    fn test_sigmoid() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!(sigmoid(10.0) > 0.9);
        assert!(sigmoid(-10.0) < 0.1);
    }
}

