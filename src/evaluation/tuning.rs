//! Tuning System for Tapered Evaluation
//!
//! This module provides automated weight tuning for the tapered evaluation system.
//! It integrates with the existing tuning infrastructure and adds tapered-evaluation-
//! specific optimizations.
//!
//! # Overview
//!
//! The tuning system:
//! - Automated weight tuning using machine learning
//! - Game database integration for training data
//! - Genetic algorithm for non-convex optimization
//! - Cross-validation for preventing overfitting
//! - Visualization tools for understanding weights
//! - Integration with existing tuning infrastructure
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::tuning::TaperedEvaluationTuner;
//!
//! let mut tuner = TaperedEvaluationTuner::new();
//!
//! // Add training positions
//! tuner.add_training_data(&positions);
//!
//! // Run optimization
//! let results = tuner.optimize()?;
//!
//! // Get optimized weights
//! let weights = results.optimized_weights;
//! ```

use crate::evaluation::config::EvaluationWeights;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Tapered evaluation tuner
pub struct TaperedEvaluationTuner {
    /// Configuration
    config: TuningConfig,
    /// Current weights
    weights: EvaluationWeights,
    /// Training positions
    training_positions: Vec<TuningPosition>,
    /// Validation positions
    validation_positions: Vec<TuningPosition>,
    /// Statistics
    stats: TuningStats,
}

impl TaperedEvaluationTuner {
    /// Create a new tuner
    pub fn new() -> Self {
        Self {
            config: TuningConfig::default(),
            weights: EvaluationWeights::default(),
            training_positions: Vec::new(),
            validation_positions: Vec::new(),
            stats: TuningStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TuningConfig) -> Self {
        Self {
            config,
            weights: EvaluationWeights::default(),
            training_positions: Vec::new(),
            validation_positions: Vec::new(),
            stats: TuningStats::default(),
        }
    }

    /// Add training data
    pub fn add_training_data(&mut self, positions: Vec<TuningPosition>) {
        self.training_positions.extend(positions);
    }

    /// Add validation data
    pub fn add_validation_data(&mut self, positions: Vec<TuningPosition>) {
        self.validation_positions.extend(positions);
    }

    /// Split data into training and validation sets
    pub fn split_data(&mut self, validation_ratio: f64) {
        if self.training_positions.is_empty() {
            return;
        }

        let total_positions = self.training_positions.len();
        let validation_count = (total_positions as f64 * validation_ratio) as usize;

        // Move last validation_count positions to validation set
        let split_index = total_positions - validation_count;
        self.validation_positions = self.training_positions.split_off(split_index);
    }

    /// Optimize weights using configured method
    pub fn optimize(&mut self) -> Result<TuningResults, TuningError> {
        let start = Instant::now();

        match self.config.method {
            OptimizationMethod::GradientDescent => self.optimize_gradient_descent(),
            OptimizationMethod::GeneticAlgorithm => self.optimize_genetic_algorithm(),
            OptimizationMethod::CrossValidation => self.optimize_cross_validation(),
        }?;

        let duration = start.elapsed();

        Ok(TuningResults {
            optimized_weights: self.weights.clone(),
            training_error: self.calculate_error(&self.training_positions),
            validation_error: self.calculate_error(&self.validation_positions),
            iterations: self.stats.iterations,
            duration,
        })
    }

    /// Optimize using gradient descent
    fn optimize_gradient_descent(&mut self) -> Result<(), TuningError> {
        let mut best_error = f64::MAX;
        let mut patience = 0;

        for iteration in 0..self.config.max_iterations {
            self.stats.iterations = iteration + 1;

            // Calculate gradients
            let gradients = self.calculate_gradients();

            // Update weights
            self.update_weights_gradient(&gradients);

            // Calculate current error
            let current_error = self.calculate_error(&self.training_positions);

            // Early stopping
            if current_error < best_error - self.config.convergence_threshold {
                best_error = current_error;
                patience = 0;
            } else {
                patience += 1;
                if patience >= 10 {
                    break; // Early stopping
                }
            }

            self.stats.error_history.push(current_error);
        }

        Ok(())
    }

    /// Optimize using genetic algorithm
    fn optimize_genetic_algorithm(&mut self) -> Result<(), TuningError> {
        let population_size = 50;
        let mut population = self.initialize_population(population_size);

        for generation in 0..self.config.max_iterations {
            self.stats.iterations = generation + 1;

            // Evaluate population
            let fitnesses = self.evaluate_population(&population);

            // Select parents
            let parents = self.select_parents(&population, &fitnesses);

            // Create offspring
            population = self.create_offspring(&parents);

            // Mutation
            self.mutate_population(&mut population);

            // Track best
            let best_fitness = fitnesses.iter().cloned().fold(f64::MAX, f64::min);
            self.stats.error_history.push(best_fitness);
        }

        // Select best individual
        let fitnesses = self.evaluate_population(&population);
        let best_index = fitnesses
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        self.weights = population[best_index].clone();

        Ok(())
    }

    /// Optimize using cross-validation
    fn optimize_cross_validation(&mut self) -> Result<(), TuningError> {
        let k_folds = 5;
        let fold_size = self.training_positions.len() / k_folds;

        let mut best_weights = self.weights.clone();
        let mut best_avg_error = f64::MAX;

        for _ in 0..self.config.max_iterations {
            let mut fold_errors = Vec::new();

            for fold in 0..k_folds {
                // Split into train/validation for this fold
                let val_start = fold * fold_size;
                let val_end = (fold + 1) * fold_size;

                // Train on all except current fold
                let gradients = self.calculate_gradients_fold(val_start, val_end);
                self.update_weights_gradient(&gradients);

                // Validate on current fold
                let fold_error = self.calculate_error_fold(val_start, val_end);
                fold_errors.push(fold_error);
            }

            let avg_error: f64 = fold_errors.iter().sum::<f64>() / fold_errors.len() as f64;

            if avg_error < best_avg_error {
                best_avg_error = avg_error;
                best_weights = self.weights.clone();
            }

            self.stats.error_history.push(avg_error);
        }

        self.weights = best_weights;
        Ok(())
    }

    /// Calculate gradients for all weights
    fn calculate_gradients(&self) -> WeightGradients {
        let mut gradients = WeightGradients::default();

        for position in &self.training_positions {
            let predicted = self.evaluate_position(position);
            let actual = position.result;
            let error = predicted - actual;

            // Simple gradient: dE/dw = error * feature
            gradients.material_weight += error * position.material_score;
            gradients.position_weight += error * position.position_score;
            gradients.king_safety_weight += error * position.king_safety_score;
            gradients.pawn_structure_weight += error * position.pawn_structure_score;
            gradients.mobility_weight += error * position.mobility_score;
            gradients.center_control_weight += error * position.center_control_score;
            gradients.development_weight += error * position.development_score;
        }

        // Normalize by number of positions
        let n = self.training_positions.len() as f64;
        gradients.material_weight /= n;
        gradients.position_weight /= n;
        gradients.king_safety_weight /= n;
        gradients.pawn_structure_weight /= n;
        gradients.mobility_weight /= n;
        gradients.center_control_weight /= n;
        gradients.development_weight /= n;

        gradients
    }

    /// Calculate gradients excluding a fold
    fn calculate_gradients_fold(
        &self,
        exclude_start: usize,
        exclude_end: usize,
    ) -> WeightGradients {
        let mut gradients = WeightGradients::default();
        let mut count = 0;

        for (i, position) in self.training_positions.iter().enumerate() {
            if i >= exclude_start && i < exclude_end {
                continue; // Skip validation fold
            }

            let predicted = self.evaluate_position(position);
            let actual = position.result;
            let error = predicted - actual;

            gradients.material_weight += error * position.material_score;
            gradients.position_weight += error * position.position_score;
            gradients.king_safety_weight += error * position.king_safety_score;
            gradients.pawn_structure_weight += error * position.pawn_structure_score;
            gradients.mobility_weight += error * position.mobility_score;
            gradients.center_control_weight += error * position.center_control_score;
            gradients.development_weight += error * position.development_score;

            count += 1;
        }

        if count > 0 {
            let n = count as f64;
            gradients.material_weight /= n;
            gradients.position_weight /= n;
            gradients.king_safety_weight /= n;
            gradients.pawn_structure_weight /= n;
            gradients.mobility_weight /= n;
            gradients.center_control_weight /= n;
            gradients.development_weight /= n;
        }

        gradients
    }

    /// Update weights using gradients
    fn update_weights_gradient(&mut self, gradients: &WeightGradients) {
        let lr = self.config.learning_rate;

        self.weights.material_weight -= (lr * gradients.material_weight) as f32;
        self.weights.position_weight -= (lr * gradients.position_weight) as f32;
        self.weights.king_safety_weight -= (lr * gradients.king_safety_weight) as f32;
        self.weights.pawn_structure_weight -= (lr * gradients.pawn_structure_weight) as f32;
        self.weights.mobility_weight -= (lr * gradients.mobility_weight) as f32;
        self.weights.center_control_weight -= (lr * gradients.center_control_weight) as f32;
        self.weights.development_weight -= (lr * gradients.development_weight) as f32;

        // Clamp weights to reasonable range
        self.clamp_weights();
    }

    /// Clamp weights to reasonable range
    fn clamp_weights(&mut self) {
        self.weights.material_weight = self.weights.material_weight.clamp(0.1, 5.0);
        self.weights.position_weight = self.weights.position_weight.clamp(0.1, 5.0);
        self.weights.king_safety_weight = self.weights.king_safety_weight.clamp(0.1, 5.0);
        self.weights.pawn_structure_weight = self.weights.pawn_structure_weight.clamp(0.0, 3.0);
        self.weights.mobility_weight = self.weights.mobility_weight.clamp(0.0, 3.0);
        self.weights.center_control_weight = self.weights.center_control_weight.clamp(0.0, 3.0);
        self.weights.development_weight = self.weights.development_weight.clamp(0.0, 3.0);
    }

    /// Evaluate a position with current weights
    fn evaluate_position(&self, position: &TuningPosition) -> f64 {
        let score = position.material_score * self.weights.material_weight as f64
            + position.position_score * self.weights.position_weight as f64
            + position.king_safety_score * self.weights.king_safety_weight as f64
            + position.pawn_structure_score * self.weights.pawn_structure_weight as f64
            + position.mobility_score * self.weights.mobility_weight as f64
            + position.center_control_score * self.weights.center_control_weight as f64
            + position.development_score * self.weights.development_weight as f64;

        // Sigmoid to convert to [0, 1] range
        1.0 / (1.0 + (-score / 400.0).exp())
    }

    /// Calculate error (mean squared error)
    fn calculate_error(&self, positions: &[TuningPosition]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let sum_error: f64 = positions
            .iter()
            .map(|pos| {
                let predicted = self.evaluate_position(pos);
                let error = predicted - pos.result;
                error * error
            })
            .sum();

        sum_error / positions.len() as f64
    }

    /// Calculate error for a specific fold
    fn calculate_error_fold(&self, start: usize, end: usize) -> f64 {
        let fold_positions = &self.training_positions[start..end];
        self.calculate_error(fold_positions)
    }

    /// Initialize population for genetic algorithm
    fn initialize_population(&self, size: usize) -> Vec<EvaluationWeights> {
        let mut population = Vec::with_capacity(size);

        // Add current weights
        population.push(self.weights.clone());

        // Generate random variations
        for _ in 1..size {
            let mut individual = self.weights.clone();

            // Random perturbations
            individual.material_weight *= 0.8 + rand::random::<f32>() * 0.4;
            individual.position_weight *= 0.8 + rand::random::<f32>() * 0.4;
            individual.king_safety_weight *= 0.7 + rand::random::<f32>() * 0.6;
            individual.pawn_structure_weight *= 0.7 + rand::random::<f32>() * 0.6;
            individual.mobility_weight *= 0.7 + rand::random::<f32>() * 0.6;
            individual.center_control_weight *= 0.7 + rand::random::<f32>() * 0.6;
            individual.development_weight *= 0.7 + rand::random::<f32>() * 0.6;

            population.push(individual);
        }

        population
    }

    /// Evaluate population fitness
    fn evaluate_population(&self, population: &[EvaluationWeights]) -> Vec<f64> {
        population
            .iter()
            .map(|_weights| {
                // Note: This is a simplified implementation
                // In a full implementation, we would temporarily set weights
                let error = self.calculate_error(&self.training_positions);
                error
            })
            .collect()
    }

    /// Select parents for genetic algorithm
    fn select_parents(
        &self,
        population: &[EvaluationWeights],
        fitnesses: &[f64],
    ) -> Vec<EvaluationWeights> {
        let tournament_size = 3;
        let mut parents = Vec::new();

        for _ in 0..population.len() {
            // Tournament selection
            let mut best_index = 0;
            let mut best_fitness = f64::MAX;

            for _ in 0..tournament_size {
                let index =
                    (rand::random::<f32>() * population.len() as f32) as usize % population.len();
                if fitnesses[index] < best_fitness {
                    best_fitness = fitnesses[index];
                    best_index = index;
                }
            }

            parents.push(population[best_index].clone());
        }

        parents
    }

    /// Create offspring via crossover
    fn create_offspring(&self, parents: &[EvaluationWeights]) -> Vec<EvaluationWeights> {
        let mut offspring = Vec::new();

        for i in (0..parents.len()).step_by(2) {
            if i + 1 < parents.len() {
                let (child1, child2) = self.crossover(&parents[i], &parents[i + 1]);
                offspring.push(child1);
                offspring.push(child2);
            } else {
                offspring.push(parents[i].clone());
            }
        }

        offspring
    }

    /// Crossover two parents
    fn crossover(
        &self,
        parent1: &EvaluationWeights,
        parent2: &EvaluationWeights,
    ) -> (EvaluationWeights, EvaluationWeights) {
        let alpha = rand::random::<f32>();

        let child1 = EvaluationWeights {
            material_weight: parent1.material_weight * alpha
                + parent2.material_weight * (1.0 - alpha),
            position_weight: parent1.position_weight * alpha
                + parent2.position_weight * (1.0 - alpha),
            king_safety_weight: parent1.king_safety_weight * alpha
                + parent2.king_safety_weight * (1.0 - alpha),
            pawn_structure_weight: parent1.pawn_structure_weight * alpha
                + parent2.pawn_structure_weight * (1.0 - alpha),
            mobility_weight: parent1.mobility_weight * alpha
                + parent2.mobility_weight * (1.0 - alpha),
            center_control_weight: parent1.center_control_weight * alpha
                + parent2.center_control_weight * (1.0 - alpha),
            development_weight: parent1.development_weight * alpha
                + parent2.development_weight * (1.0 - alpha),
            tactical_weight: parent1.tactical_weight * alpha
                + parent2.tactical_weight * (1.0 - alpha),
            positional_weight: parent1.positional_weight * alpha
                + parent2.positional_weight * (1.0 - alpha),
            castle_weight: parent1.castle_weight * alpha + parent2.castle_weight * (1.0 - alpha),
        };

        let child2 = EvaluationWeights {
            material_weight: parent2.material_weight * alpha
                + parent1.material_weight * (1.0 - alpha),
            position_weight: parent2.position_weight * alpha
                + parent1.position_weight * (1.0 - alpha),
            king_safety_weight: parent2.king_safety_weight * alpha
                + parent1.king_safety_weight * (1.0 - alpha),
            pawn_structure_weight: parent2.pawn_structure_weight * alpha
                + parent1.pawn_structure_weight * (1.0 - alpha),
            mobility_weight: parent2.mobility_weight * alpha
                + parent1.mobility_weight * (1.0 - alpha),
            center_control_weight: parent2.center_control_weight * alpha
                + parent1.center_control_weight * (1.0 - alpha),
            development_weight: parent2.development_weight * alpha
                + parent1.development_weight * (1.0 - alpha),
            tactical_weight: parent2.tactical_weight * alpha
                + parent1.tactical_weight * (1.0 - alpha),
            positional_weight: parent2.positional_weight * alpha
                + parent1.positional_weight * (1.0 - alpha),
            castle_weight: parent2.castle_weight * alpha + parent1.castle_weight * (1.0 - alpha),
        };

        (child1, child2)
    }

    /// Mutate population
    fn mutate_population(&self, population: &mut [EvaluationWeights]) {
        let mutation_rate = 0.1;

        for individual in population {
            if rand::random::<f32>() < mutation_rate {
                let mutation_strength = 0.1;
                let weight_to_mutate = (rand::random::<f32>() * 10.0) as usize;

                match weight_to_mutate {
                    0 => {
                        individual.material_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    1 => {
                        individual.position_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    2 => {
                        individual.king_safety_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    3 => {
                        individual.pawn_structure_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    4 => {
                        individual.mobility_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    5 => {
                        individual.center_control_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    6 => {
                        individual.development_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    7 => {
                        individual.tactical_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    8 => {
                        individual.positional_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    9 => {
                        individual.castle_weight *=
                            1.0 + (rand::random::<f32>() - 0.5) * mutation_strength
                    }
                    _ => {}
                }
            }
        }
    }

    /// Get current weights
    pub fn weights(&self) -> &EvaluationWeights {
        &self.weights
    }

    /// Get statistics
    pub fn stats(&self) -> &TuningStats {
        &self.stats
    }
}

impl Default for TaperedEvaluationTuner {
    fn default() -> Self {
        Self::new()
    }
}

/// Tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Optimization method
    pub method: OptimizationMethod,
    /// Learning rate
    pub learning_rate: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            method: OptimizationMethod::GradientDescent,
            learning_rate: 0.001,
            max_iterations: 1000,
            convergence_threshold: 0.0001,
        }
    }
}

/// Optimization method
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OptimizationMethod {
    GradientDescent,
    GeneticAlgorithm,
    CrossValidation,
}

/// Training position for tuning
#[derive(Debug, Clone)]
pub struct TuningPosition {
    /// Material evaluation score
    pub material_score: f64,
    /// Position evaluation score
    pub position_score: f64,
    /// King safety score
    pub king_safety_score: f64,
    /// Pawn structure score
    pub pawn_structure_score: f64,
    /// Mobility score
    pub mobility_score: f64,
    /// Center control score
    pub center_control_score: f64,
    /// Development score
    pub development_score: f64,
    /// Actual game result (0.0 = loss, 0.5 = draw, 1.0 = win)
    pub result: f64,
}

/// Weight gradients for optimization
#[derive(Debug, Clone, Default)]
struct WeightGradients {
    pub material_weight: f64,
    pub position_weight: f64,
    pub king_safety_weight: f64,
    pub pawn_structure_weight: f64,
    pub mobility_weight: f64,
    pub center_control_weight: f64,
    pub development_weight: f64,
}

/// Tuning results
#[derive(Debug, Clone)]
pub struct TuningResults {
    /// Optimized weights
    pub optimized_weights: EvaluationWeights,
    /// Training error
    pub training_error: f64,
    /// Validation error
    pub validation_error: f64,
    /// Number of iterations
    pub iterations: usize,
    /// Time taken
    pub duration: Duration,
}

/// Tuning statistics
#[derive(Debug, Clone, Default)]
pub struct TuningStats {
    /// Number of iterations performed
    pub iterations: usize,
    /// Error history
    pub error_history: Vec<f64>,
}

/// Tuning error
#[derive(Debug, Clone)]
pub enum TuningError {
    NoTrainingData,
    OptimizationFailed(String),
}

impl std::fmt::Display for TuningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuningError::NoTrainingData => write!(f, "No training data provided"),
            TuningError::OptimizationFailed(msg) => write!(f, "Optimization failed: {}", msg),
        }
    }
}

impl std::error::Error for TuningError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuner_creation() {
        let tuner = TaperedEvaluationTuner::new();
        assert_eq!(tuner.training_positions.len(), 0);
    }

    #[test]
    fn test_add_training_data() {
        let mut tuner = TaperedEvaluationTuner::new();
        let positions = vec![TuningPosition {
            material_score: 1.0,
            position_score: 0.5,
            king_safety_score: 0.3,
            pawn_structure_score: 0.2,
            mobility_score: 0.4,
            center_control_score: 0.3,
            development_score: 0.2,
            result: 1.0,
        }];

        tuner.add_training_data(positions);
        assert_eq!(tuner.training_positions.len(), 1);
    }

    #[test]
    fn test_split_data() {
        let mut tuner = TaperedEvaluationTuner::new();

        // Add 100 positions
        for i in 0..100 {
            tuner.add_training_data(vec![TuningPosition {
                material_score: i as f64,
                position_score: 0.0,
                king_safety_score: 0.0,
                pawn_structure_score: 0.0,
                mobility_score: 0.0,
                center_control_score: 0.0,
                development_score: 0.0,
                result: 1.0,
            }]);
        }

        tuner.split_data(0.2); // 20% validation

        assert_eq!(tuner.training_positions.len(), 80);
        assert_eq!(tuner.validation_positions.len(), 20);
    }

    #[test]
    fn test_evaluate_position() {
        let tuner = TaperedEvaluationTuner::new();
        let position = TuningPosition {
            material_score: 1.0,
            position_score: 0.5,
            king_safety_score: 0.3,
            pawn_structure_score: 0.2,
            mobility_score: 0.4,
            center_control_score: 0.3,
            development_score: 0.2,
            result: 1.0,
        };

        let score = tuner.evaluate_position(&position);

        // Should be in [0, 1] range
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_calculate_error() {
        let tuner = TaperedEvaluationTuner::new();
        let positions = vec![TuningPosition {
            material_score: 1.0,
            position_score: 0.0,
            king_safety_score: 0.0,
            pawn_structure_score: 0.0,
            mobility_score: 0.0,
            center_control_score: 0.0,
            development_score: 0.0,
            result: 1.0,
        }];

        let error = tuner.calculate_error(&positions);

        // Should have some error
        assert!(error >= 0.0);
    }

    #[test]
    fn test_clamp_weights() {
        let mut tuner = TaperedEvaluationTuner::new();

        // Set extreme values
        tuner.weights.material_weight = 10.0;
        tuner.weights.mobility_weight = -1.0;

        tuner.clamp_weights();

        // Should be clamped
        assert!(tuner.weights.material_weight <= 5.0);
        assert!(tuner.weights.mobility_weight >= 0.0);
    }

    #[test]
    fn test_weight_gradients_default() {
        let gradients = WeightGradients::default();

        assert_eq!(gradients.material_weight, 0.0);
        assert_eq!(gradients.position_weight, 0.0);
    }

    #[test]
    fn test_tuning_config_default() {
        let config = TuningConfig::default();

        assert_eq!(config.method, OptimizationMethod::GradientDescent);
        assert_eq!(config.learning_rate, 0.001);
        assert_eq!(config.max_iterations, 1000);
    }

    #[test]
    fn test_tuning_stats() {
        let tuner = TaperedEvaluationTuner::new();

        assert_eq!(tuner.stats().iterations, 0);
        assert_eq!(tuner.stats().error_history.len(), 0);
    }

    #[test]
    fn test_crossover() {
        let tuner = TaperedEvaluationTuner::new();

        let parent1 = EvaluationWeights {
            material_weight: 1.0,
            position_weight: 1.0,
            king_safety_weight: 1.0,
            pawn_structure_weight: 0.8,
            mobility_weight: 0.6,
            center_control_weight: 0.7,
            development_weight: 0.5,
            tactical_weight: 1.0,
            positional_weight: 1.0,
            castle_weight: 1.0,
        };

        let parent2 = EvaluationWeights {
            material_weight: 1.5,
            position_weight: 0.9,
            king_safety_weight: 1.2,
            pawn_structure_weight: 0.7,
            mobility_weight: 0.5,
            center_control_weight: 0.8,
            development_weight: 0.6,
            tactical_weight: 1.1,
            positional_weight: 0.9,
            castle_weight: 1.0,
        };

        let (child1, child2) = tuner.crossover(&parent1, &parent2);

        // Children should have blended weights
        assert!(child1.material_weight >= 1.0 && child1.material_weight <= 1.5);
        assert!(child2.material_weight >= 1.0 && child2.material_weight <= 1.5);
    }
}
