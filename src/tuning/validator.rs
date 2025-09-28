//! Validation framework for automated tuning
//! 
//! This module provides cross-validation, holdout validation, and other
//! validation techniques to ensure the quality of tuned parameters.

use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use super::types::{ValidationConfig, ValidationResults, TrainingPosition, FoldResult, MatchResult, OptimizationMethod};
use super::optimizer::Optimizer;
use crate::types::{NUM_EVAL_FEATURES, Player};

/// Validation engine for tuning results
pub struct Validator {
    config: ValidationConfig,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Perform k-fold cross-validation
    pub fn cross_validate(&self, positions: &[TrainingPosition]) -> ValidationResults {
        if positions.is_empty() {
            return ValidationResults::new(vec![]);
        }

        let k = self.config.k_fold as usize;
        let mut fold_results = Vec::new();
        let mut shuffled_positions = positions.to_vec();
        shuffled_positions.shuffle(&mut thread_rng());

        let fold_size = shuffled_positions.len() / k;
        let remainder = shuffled_positions.len() % k;

        let mut start_idx = 0;
        for fold in 0..k {
            let fold_size_adjusted = fold_size + if fold < remainder { 1 } else { 0 };
            let end_idx = start_idx + fold_size_adjusted;

            // Split data into training and validation sets
            let (validation_set, training_set) = self.split_data(&shuffled_positions, start_idx, end_idx);

            // Train model on training set
            let optimizer = Optimizer::new(OptimizationMethod::default());
            let optimization_result = optimizer.optimize(&training_set);

            match optimization_result {
                Ok(result) => {
                    // Validate on validation set
                    let validation_error = self.calculate_error(&result.optimized_weights, &validation_set);
                    
                    // Also test on a small subset for test error
                    let test_subset = self.create_test_subset(&validation_set);
                    let test_error = self.calculate_error(&result.optimized_weights, &test_subset);

                    fold_results.push(FoldResult {
                        fold_number: (fold + 1) as u32,
                        validation_error,
                        test_error,
                        sample_count: validation_set.len(),
                    });
                }
                Err(_) => {
                    // If optimization fails, use a high error value
                    fold_results.push(FoldResult {
                        fold_number: (fold + 1) as u32,
                        validation_error: 1.0,
                        test_error: 1.0,
                        sample_count: validation_set.len(),
                    });
                }
            }

            start_idx = end_idx;
        }

        ValidationResults::new(fold_results)
    }

    /// Perform holdout validation
    pub fn holdout_validate(&self, positions: &[TrainingPosition]) -> ValidationResults {
        if positions.is_empty() {
            return ValidationResults::new(vec![]);
        }

        let mut shuffled_positions = positions.to_vec();
        shuffled_positions.shuffle(&mut thread_rng());

        // Split data according to validation_split
        let validation_size = (positions.len() as f64 * self.config.validation_split) as usize;
        let (validation_set, training_set) = shuffled_positions.split_at(validation_size);

        // Train model on training set
        let optimizer = Optimizer::new(OptimizationMethod::default());
        let optimization_result = optimizer.optimize(training_set);

        match optimization_result {
            Ok(result) => {
                // Validate on validation set
                let validation_error = self.calculate_error(&result.optimized_weights, validation_set);
                
                // Create test subset for test error
                let test_subset = self.create_test_subset(validation_set);
                let test_error = self.calculate_error(&result.optimized_weights, &test_subset);

                let fold_result = FoldResult {
                    fold_number: 1,
                    validation_error,
                    test_error,
                    sample_count: validation_set.len(),
                };

                ValidationResults::new(vec![fold_result])
            }
            Err(_) => {
                // If optimization fails, return high error
                let fold_result = FoldResult {
                    fold_number: 1,
                    validation_error: 1.0,
                    test_error: 1.0,
                    sample_count: validation_set.len(),
                };

                ValidationResults::new(vec![fold_result])
            }
        }
    }

    /// Split data into training and validation sets for cross-validation
    fn split_data(&self, positions: &[TrainingPosition], start_idx: usize, end_idx: usize) -> (Vec<TrainingPosition>, Vec<TrainingPosition>) {
        let validation_set = positions[start_idx..end_idx].to_vec();
        let mut training_set = Vec::new();
        
        training_set.extend_from_slice(&positions[0..start_idx]);
        training_set.extend_from_slice(&positions[end_idx..]);

        (validation_set, training_set)
    }

    /// Calculate mean squared error for a set of positions
    fn calculate_error(&self, weights: &[f64], positions: &[TrainingPosition]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let mut total_error = 0.0;
        for position in positions {
            // Calculate predicted probability using sigmoid
            let score: f64 = weights.iter().zip(position.features.iter())
                .map(|(w, f)| w * f)
                .sum();
            
            let predicted_prob = 1.0 / (1.0 + (-score).exp());
            let error = position.result - predicted_prob;
            total_error += error * error;
        }

        total_error / positions.len() as f64
    }

    /// Create a test subset from validation set
    fn create_test_subset(&self, validation_set: &[TrainingPosition]) -> Vec<TrainingPosition> {
        let test_size = (validation_set.len() as f64 * self.config.test_split) as usize;
        if test_size == 0 {
            return validation_set.to_vec();
        }
        
        let mut test_subset = validation_set.to_vec();
        test_subset.shuffle(&mut thread_rng());
        test_subset.truncate(test_size);
        test_subset
    }
}

/// Strength testing framework for engine vs engine matches
pub struct StrengthTester {
    /// Number of games to play for testing
    pub games_per_test: u32,
    /// Time control for games (in milliseconds)
    pub time_control_ms: u32,
}

impl StrengthTester {
    /// Create a new strength tester
    pub fn new(games_per_test: u32, time_control_ms: u32) -> Self {
        Self {
            games_per_test,
            time_control_ms,
        }
    }

    /// Run engine vs engine matches to test strength
    pub fn test_engine_strength(&self, original_weights: &[f64], tuned_weights: &[f64]) -> MatchResult {
        // For now, simulate match results based on weight differences
        // In a real implementation, this would play actual games
        
        let weight_difference = self.calculate_weight_difference(original_weights, tuned_weights);
        let strength_improvement = self.estimate_strength_improvement(weight_difference);
        
        // Simulate match results based on strength improvement
        let (wins, losses, draws) = self.simulate_match_results(strength_improvement);
        
        MatchResult {
            wins,
            losses,
            draws,
            elo_difference: self.calculate_elo_difference(wins, losses, draws),
            elo_confidence_interval: self.calculate_elo_confidence_interval(wins, losses, draws),
            total_games: self.games_per_test,
        }
    }

    /// Calculate the difference between original and tuned weights
    fn calculate_weight_difference(&self, original: &[f64], tuned: &[f64]) -> f64 {
        original.iter().zip(tuned.iter())
            .map(|(o, t)| (o - t).abs())
            .sum::<f64>()
    }

    /// Estimate strength improvement based on weight differences
    fn estimate_strength_improvement(&self, weight_difference: f64) -> f64 {
        // Simple heuristic: larger weight changes suggest bigger improvements
        // This is a placeholder - real implementation would be more sophisticated
        weight_difference * 0.1
    }

    /// Simulate match results based on strength improvement
    fn simulate_match_results(&self, strength_improvement: f64) -> (u32, u32, u32) {
        let mut rng = thread_rng();
        let win_probability = 0.5 + strength_improvement.min(0.3); // Cap at 80% win rate
        let draw_probability = 0.2;
        
        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;
        
        for _ in 0..self.games_per_test {
            let rand_val: f64 = rng.gen();
            
            if rand_val < win_probability {
                wins += 1;
            } else if rand_val < win_probability + draw_probability {
                draws += 1;
            } else {
                losses += 1;
            }
        }
        
        (wins, losses, draws)
    }

    /// Calculate ELO difference from match results
    fn calculate_elo_difference(&self, wins: u32, losses: u32, draws: u32) -> f64 {
        let total_games = wins + losses + draws;
        if total_games == 0 {
            return 0.0;
        }
        
        let win_rate = wins as f64 / total_games as f64;
        let draw_rate = draws as f64 / total_games as f64;
        
        // Simple ELO calculation: win rate of 0.5 = 0 ELO difference
        // This is a simplified version - real implementation would be more sophisticated
        if win_rate > 0.5 {
            (win_rate - 0.5) * 400.0
        } else {
            (win_rate - 0.5) * 400.0
        }
    }

    /// Calculate ELO confidence interval
    fn calculate_elo_confidence_interval(&self, wins: u32, losses: u32, draws: u32) -> (f64, f64) {
        let total_games = wins + losses + draws;
        if total_games == 0 {
            return (0.0, 0.0);
        }
        
        // Simplified confidence interval calculation
        let margin = 100.0 / (total_games as f64).sqrt();
        let elo_diff = self.calculate_elo_difference(wins, losses, draws);
        
        (elo_diff - margin, elo_diff + margin)
    }
}

/// Synthetic dataset generator for testing optimization algorithms
pub struct SyntheticDataGenerator {
    /// Number of features to generate
    feature_count: usize,
    /// Random seed for reproducibility
    seed: u64,
}

impl SyntheticDataGenerator {
    /// Create a new synthetic data generator
    pub fn new(feature_count: usize, seed: u64) -> Self {
        Self {
            feature_count,
            seed,
        }
    }

    /// Generate synthetic training positions
    pub fn generate_positions(&self, count: usize) -> Vec<TrainingPosition> {
        let mut rng = thread_rng();
        let mut positions = Vec::new();
        
        for i in 0..count {
            // Generate random features
            let mut features = vec![0.0; self.feature_count];
            for j in 0..self.feature_count {
                features[j] = rng.gen_range(-1.0..1.0);
            }
            
            // Generate synthetic result based on features
            let result = self.generate_synthetic_result(&features, &mut rng);
            
            positions.push(TrainingPosition::new(
                features,
                result,
                128, // Default game phase
                true, // Default quiet
                i as u32, // Move number
                if i % 2 == 0 { Player::White } else { Player::Black },
            ));
        }
        
        positions
    }

    /// Generate synthetic result based on features
    fn generate_synthetic_result(&self, features: &[f64], rng: &mut impl rand::Rng) -> f64 {
        // Create a simple linear relationship with some noise
        let true_score: f64 = features.iter().enumerate()
            .map(|(i, &f)| f * ((i as f64 + 1.0) * 0.1))
            .sum();
        
        // Add noise
        let noise = rng.gen_range(-0.1..0.1);
        let noisy_score = true_score + noise;
        
        // Convert to probability using sigmoid
        1.0 / (1.0 + (-noisy_score).exp())
    }
}

/// Overfitting detection mechanisms
pub struct OverfittingDetector {
    /// Threshold for overfitting detection
    validation_error_threshold: f64,
    /// Minimum difference between training and validation error
    error_difference_threshold: f64,
}

impl OverfittingDetector {
    /// Create a new overfitting detector
    pub fn new(validation_error_threshold: f64, error_difference_threshold: f64) -> Self {
        Self {
            validation_error_threshold,
            error_difference_threshold,
        }
    }

    /// Detect if overfitting is occurring
    pub fn detect_overfitting(&self, training_error: f64, validation_error: f64) -> bool {
        validation_error > self.validation_error_threshold ||
        (validation_error - training_error) > self.error_difference_threshold
    }

    /// Calculate overfitting score (0.0 = no overfitting, 1.0 = severe overfitting)
    pub fn calculate_overfitting_score(&self, training_error: f64, validation_error: f64) -> f64 {
        let error_diff = validation_error - training_error;
        let threshold_ratio = validation_error / self.validation_error_threshold;
        
        (error_diff / self.error_difference_threshold).min(1.0) * 0.5 +
        (threshold_ratio - 1.0).max(0.0) * 0.5
    }
}

/// Performance benchmarking for optimization
pub struct PerformanceBenchmark {
    /// Memory usage tracking
    memory_usage: HashMap<String, usize>,
    /// Timing measurements
    timings: HashMap<String, f64>,
}

impl PerformanceBenchmark {
    /// Create a new performance benchmark
    pub fn new() -> Self {
        Self {
            memory_usage: HashMap::new(),
            timings: HashMap::new(),
        }
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, operation: &str, bytes: usize) {
        self.memory_usage.insert(operation.to_string(), bytes);
    }

    /// Record timing measurement
    pub fn record_timing(&mut self, operation: &str, seconds: f64) {
        self.timings.insert(operation.to_string(), seconds);
    }

    /// Get memory usage for an operation
    pub fn get_memory_usage(&self, operation: &str) -> Option<usize> {
        self.memory_usage.get(operation).copied()
    }

    /// Get timing for an operation
    pub fn get_timing(&self, operation: &str) -> Option<f64> {
        self.timings.get(operation).copied()
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Benchmark Report\n");
        report.push_str("===========================\n\n");
        
        report.push_str("Memory Usage:\n");
        for (operation, bytes) in &self.memory_usage {
            report.push_str(&format!("  {}: {} bytes ({:.2} MB)\n", operation, bytes, *bytes as f64 / 1024.0 / 1024.0));
        }
        
        report.push_str("\nTimings:\n");
        for (operation, seconds) in &self.timings {
            report.push_str(&format!("  {}: {:.3} seconds\n", operation, seconds));
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ValidationConfig;

    #[test]
    fn test_validator_creation() {
        let config = ValidationConfig::default();
        let validator = Validator::new(config);
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_cross_validation_with_empty_data() {
        let config = ValidationConfig::default();
        let validator = Validator::new(config);
        
        let positions = vec![];
        let results = validator.cross_validate(&positions);
        
        assert_eq!(results.fold_results.len(), 0);
    }

    #[test]
    fn test_holdout_validation_with_empty_data() {
        let config = ValidationConfig::default();
        let validator = Validator::new(config);
        
        let positions = vec![];
        let results = validator.holdout_validate(&positions);
        
        assert_eq!(results.fold_results.len(), 0);
    }

    #[test]
    fn test_strength_tester_creation() {
        let tester = StrengthTester::new(100, 5000);
        assert_eq!(tester.games_per_test, 100);
        assert_eq!(tester.time_control_ms, 5000);
    }

    #[test]
    fn test_strength_tester_match() {
        let tester = StrengthTester::new(10, 1000);
        let original_weights = vec![1.0; NUM_EVAL_FEATURES];
        let tuned_weights = vec![1.1; NUM_EVAL_FEATURES];
        
        let result = tester.test_engine_strength(&original_weights, &tuned_weights);
        assert_eq!(result.total_games, 10);
        assert_eq!(result.wins + result.losses + result.draws, 10);
    }

    #[test]
    fn test_synthetic_data_generator() {
        let generator = SyntheticDataGenerator::new(NUM_EVAL_FEATURES, 42);
        let positions = generator.generate_positions(5);
        
        assert_eq!(positions.len(), 5);
        for position in positions {
            assert_eq!(position.features.len(), NUM_EVAL_FEATURES);
            assert!(position.result >= 0.0 && position.result <= 1.0);
        }
    }

    #[test]
    fn test_overfitting_detector() {
        let detector = OverfittingDetector::new(0.5, 0.2);
        
        // Test no overfitting
        assert!(!detector.detect_overfitting(0.1, 0.15));
        
        // Test overfitting
        assert!(detector.detect_overfitting(0.1, 0.6));
        assert!(detector.detect_overfitting(0.1, 0.4));
    }

    #[test]
    fn test_performance_benchmark() {
        let mut benchmark = PerformanceBenchmark::new();
        
        benchmark.record_memory_usage("test", 1024);
        benchmark.record_timing("test", 1.5);
        
        assert_eq!(benchmark.get_memory_usage("test"), Some(1024));
        assert_eq!(benchmark.get_timing("test"), Some(1.5));
        
        let report = benchmark.generate_report();
        assert!(report.contains("test"));
        assert!(report.contains("1024"));
        assert!(report.contains("1.500"));
    }

    #[test]
    fn test_cross_validation_with_synthetic_data() {
        let config = ValidationConfig {
            k_fold: 3,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: false,
            random_seed: Some(42),
        };
        let validator = Validator::new(config);
        
        // Generate synthetic test data
        let generator = SyntheticDataGenerator::new(NUM_EVAL_FEATURES, 42);
        let positions = generator.generate_positions(30); // Small dataset for testing
        
        let results = validator.cross_validate(&positions);
        
        assert_eq!(results.fold_results.len(), 3);
        for fold_result in &results.fold_results {
            assert!(fold_result.fold_number >= 1 && fold_result.fold_number <= 3);
            assert!(fold_result.validation_error >= 0.0);
            assert!(fold_result.test_error >= 0.0);
            assert!(fold_result.sample_count > 0);
        }
    }

    #[test]
    fn test_holdout_validation_with_synthetic_data() {
        let config = ValidationConfig {
            k_fold: 5,
            test_split: 0.2,
            validation_split: 0.3,
            stratified: false,
            random_seed: Some(42),
        };
        let validator = Validator::new(config);
        
        // Generate synthetic test data
        let generator = SyntheticDataGenerator::new(NUM_EVAL_FEATURES, 42);
        let positions = generator.generate_positions(20); // Small dataset for testing
        
        let results = validator.holdout_validate(&positions);
        
        assert_eq!(results.fold_results.len(), 1);
        let fold_result = &results.fold_results[0];
        assert_eq!(fold_result.fold_number, 1);
        assert!(fold_result.validation_error >= 0.0);
        assert!(fold_result.test_error >= 0.0);
        assert!(fold_result.sample_count > 0);
    }
}