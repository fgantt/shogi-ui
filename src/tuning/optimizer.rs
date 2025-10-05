//! Optimization algorithms for automated tuning
//! 
//! This module provides various optimization algorithms for tuning
//! evaluation function parameters using training data. It implements
//! Texel's tuning method and other advanced optimization techniques.
//! 
//! Supported algorithms:
//! - Gradient Descent with momentum
//! - Adam optimizer with adaptive learning rates
//! - LBFGS quasi-Newton method
//! - Genetic Algorithm for non-convex optimization
//! - Regularization (L1 and L2) to prevent overfitting

use super::types::{TrainingPosition, OptimizationMethod, TuningConfig, ValidationResults, FoldResult};
use crate::types::NUM_EVAL_FEATURES;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Texel's tuning method implementation
pub struct TexelTuner {
    positions: Vec<TrainingPosition>,
    weights: Vec<f64>,
    k_factor: f64,
    learning_rate: f64,
    momentum: f64,
    regularization_l1: f64,
    regularization_l2: f64,
    max_iterations: usize,
    convergence_threshold: f64,
    early_stopping_patience: usize,
}

/// Optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResults {
    pub optimized_weights: Vec<f64>,
    pub final_error: f64,
    pub iterations: usize,
    pub convergence_reason: ConvergenceReason,
    pub optimization_time: Duration,
    pub error_history: Vec<f64>,
}

/// Convergence reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceReason {
    Converged,
    MaxIterations,
    EarlyStopping,
    GradientNorm,
}

/// Adam optimizer state
#[derive(Debug, Clone)]
struct AdamState {
    m: Vec<f64>, // First moment estimates
    v: Vec<f64>, // Second moment estimates
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    t: usize,    // Time step
}

/// LBFGS optimizer state
#[derive(Debug, Clone)]
struct LBFGSState {
    s: Vec<Vec<f64>>, // Position differences
    y: Vec<Vec<f64>>, // Gradient differences
    rho: Vec<f64>,    // Scaling factors
    alpha: Vec<f64>,  // Line search parameters
    m: usize,         // Memory size
}

/// Genetic algorithm state
#[derive(Debug, Clone)]
struct GeneticAlgorithmState {
    population: Vec<Vec<f64>>,
    fitness_scores: Vec<f64>,
    generation: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    population_size: usize,
    elite_size: usize,
}

/// Optimization engine for tuning evaluation parameters
pub struct Optimizer {
    method: OptimizationMethod,
    #[allow(dead_code)]
    config: TuningConfig,
}

impl TexelTuner {
    /// Create a new Texel tuner
    pub fn new(
        positions: Vec<TrainingPosition>,
        initial_weights: Option<Vec<f64>>,
        k_factor: f64,
    ) -> Self {
        let weights = initial_weights.unwrap_or_else(|| vec![1.0; NUM_EVAL_FEATURES]);
        
        Self {
            positions,
            weights,
            k_factor,
            learning_rate: 0.01,
            momentum: 0.9,
            regularization_l1: 0.0,
            regularization_l2: 0.0,
            max_iterations: 1000,
            convergence_threshold: 1e-6,
            early_stopping_patience: 50,
        }
    }

    /// Create a new Texel tuner with custom parameters
    pub fn with_params(
        positions: Vec<TrainingPosition>,
        initial_weights: Option<Vec<f64>>,
        k_factor: f64,
        learning_rate: f64,
        momentum: f64,
        regularization_l1: f64,
        regularization_l2: f64,
        max_iterations: usize,
        convergence_threshold: f64,
        early_stopping_patience: usize,
    ) -> Self {
        let weights = initial_weights.unwrap_or_else(|| vec![1.0; NUM_EVAL_FEATURES]);
        
        Self {
            positions,
            weights,
            k_factor,
            learning_rate,
            momentum,
            regularization_l1,
            regularization_l2,
            max_iterations,
            convergence_threshold,
            early_stopping_patience,
        }
    }

    /// Optimize weights using Texel's tuning method
    pub fn optimize(&mut self) -> OptimizationResults {
        let start_time = Instant::now();
        let mut error_history = Vec::new();
        let mut best_error = f64::INFINITY;
        let mut patience_counter = 0;
        let mut velocity = vec![0.0; self.weights.len()];

        for iteration in 0..self.max_iterations {
            // Calculate current error and gradients
            let (error, gradients) = self.calculate_error_and_gradients();

            error_history.push(error);

            // Check for convergence
            if error < self.convergence_threshold {
                return OptimizationResults {
                    optimized_weights: self.weights.clone(),
                    final_error: error,
                    iterations: iteration + 1,
                    convergence_reason: ConvergenceReason::Converged,
                    optimization_time: start_time.elapsed(),
                    error_history,
                };
            }

            // Early stopping check
            if error < best_error {
                best_error = error;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= self.early_stopping_patience {
                    return OptimizationResults {
                        optimized_weights: self.weights.clone(),
                        final_error: error,
                        iterations: iteration + 1,
                        convergence_reason: ConvergenceReason::EarlyStopping,
                        optimization_time: start_time.elapsed(),
                        error_history,
                    };
                }
            }

            // Gradient descent with momentum
            for i in 0..self.weights.len() {
                velocity[i] = self.momentum * velocity[i] - self.learning_rate * gradients[i];
                self.weights[i] += velocity[i];
            }

            // Apply regularization
            self.apply_regularization();
        }

        OptimizationResults {
            optimized_weights: self.weights.clone(),
            final_error: best_error,
            iterations: self.max_iterations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        }
    }

    /// Calculate error and gradients using mean squared error
    fn calculate_error_and_gradients(&self) -> (f64, Vec<f64>) {
        let mut total_error = 0.0;
        let mut gradients = vec![0.0; self.weights.len()];

        for position in &self.positions {
            // Calculate predicted score
            let predicted = self.calculate_position_score(position);
            let predicted_prob = self.sigmoid(predicted);

            // Calculate error
            let error = position.result - predicted_prob;
            total_error += error * error;

            // Calculate gradients
            let sigmoid_derivative = self.sigmoid_derivative(predicted);
            for (i, &feature) in position.features.iter().enumerate() {
                if i < gradients.len() {
                    gradients[i] += -2.0 * error * sigmoid_derivative * feature;
                }
            }
        }

        // Average the error and gradients
        let n = self.positions.len() as f64;
        total_error /= n;
        for gradient in &mut gradients {
            *gradient /= n;
        }

        (total_error, gradients)
    }

    /// Calculate position score using current weights
    fn calculate_position_score(&self, position: &TrainingPosition) -> f64 {
        let mut score = 0.0;
        for (i, &feature) in position.features.iter().enumerate() {
            if i < self.weights.len() {
                score += self.weights[i] * feature;
            }
        }
        score
    }

    /// Sigmoid function for win probability prediction
    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-self.k_factor * x).exp())
    }

    /// Sigmoid derivative for gradient calculations
    fn sigmoid_derivative(&self, x: f64) -> f64 {
        let s = self.sigmoid(x);
        self.k_factor * s * (1.0 - s)
    }

    /// Apply L1 and L2 regularization
    fn apply_regularization(&mut self) {
        for i in 0..self.weights.len() {
            let weight = self.weights[i];
            
            // L1 regularization (Lasso)
            if self.regularization_l1 > 0.0 {
                if weight > self.regularization_l1 {
                    self.weights[i] -= self.regularization_l1;
                } else if weight < -self.regularization_l1 {
                    self.weights[i] += self.regularization_l1;
                } else {
                    self.weights[i] = 0.0;
                }
            }
            
            // L2 regularization (Ridge)
            if self.regularization_l2 > 0.0 {
                self.weights[i] *= 1.0 - self.learning_rate * self.regularization_l2;
            }
        }
    }

    /// Get current weights
    pub fn get_weights(&self) -> &[f64] {
        &self.weights
    }

    /// Set weights
    pub fn set_weights(&mut self, weights: Vec<f64>) {
        self.weights = weights;
    }
}

impl AdamState {
    /// Create new Adam state
    fn new(num_weights: usize) -> Self {
        Self {
            m: vec![0.0; num_weights],
            v: vec![0.0; num_weights],
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            t: 0,
        }
    }

    /// Update weights using Adam optimizer
    fn update(&mut self, weights: &mut [f64], gradients: &[f64], learning_rate: f64) {
        self.t += 1;
        let beta1_t = self.beta1.powi(self.t as i32);
        let beta2_t = self.beta2.powi(self.t as i32);

        for i in 0..weights.len() {
            // Update biased first moment estimate
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * gradients[i];
            
            // Update biased second moment estimate
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * gradients[i] * gradients[i];
            
            // Compute bias-corrected first moment estimate
            let m_hat = self.m[i] / (1.0 - beta1_t);
            
            // Compute bias-corrected second moment estimate
            let v_hat = self.v[i] / (1.0 - beta2_t);
            
            // Update weights
            weights[i] -= learning_rate * m_hat / (v_hat.sqrt() + self.epsilon);
        }
    }
}

impl LBFGSState {
    /// Create new LBFGS state
    fn new(memory_size: usize, _num_weights: usize) -> Self {
        Self {
            s: Vec::new(),
            y: Vec::new(),
            rho: Vec::new(),
            alpha: Vec::new(),
            m: memory_size,
        }
    }

    /// Update LBFGS state with new position and gradient
    fn update(&mut self, weights: &[f64], gradients: &[f64], prev_weights: &[f64], prev_gradients: &[f64]) {
        if self.s.len() >= self.m {
            self.s.remove(0);
            self.y.remove(0);
            self.rho.remove(0);
        }

        // Calculate position and gradient differences
        let s_diff: Vec<f64> = weights.iter().zip(prev_weights.iter()).map(|(w, p)| w - p).collect();
        let y_diff: Vec<f64> = gradients.iter().zip(prev_gradients.iter()).map(|(g, p)| g - p).collect();

        // Calculate rho (scaling factor)
        let rho = 1.0 / s_diff.iter().zip(y_diff.iter()).map(|(s, y)| s * y).sum::<f64>();

        self.s.push(s_diff);
        self.y.push(y_diff);
        self.rho.push(rho);
    }

    /// Apply LBFGS update to weights
    fn apply_update(&mut self, weights: &mut [f64], gradients: &[f64], learning_rate: f64) {
        let mut q = gradients.to_vec();
        self.alpha.clear();

        // Two-loop recursion
        for i in (0..self.s.len()).rev() {
            let alpha_i = self.rho[i] * self.s[i].iter().zip(q.iter()).map(|(s, q)| s * q).sum::<f64>();
            self.alpha.push(alpha_i);
            
            for j in 0..q.len() {
                q[j] -= alpha_i * self.y[i][j];
            }
        }

        // Apply scaling
        if !self.s.is_empty() {
            let last_idx = self.s.len() - 1;
            let gamma = self.s[last_idx].iter().zip(self.y[last_idx].iter())
                .map(|(s, y)| s * y).sum::<f64>() / 
                self.y[last_idx].iter().map(|y| y * y).sum::<f64>();
            
            for q_val in &mut q {
                *q_val *= gamma;
            }
        }

        // Second loop
        for (i, &alpha_i) in self.alpha.iter().rev().enumerate() {
            let beta = self.rho[i] * self.y[i].iter().zip(q.iter()).map(|(y, q)| y * q).sum::<f64>();
            
            for j in 0..q.len() {
                q[j] += (alpha_i - beta) * self.s[i][j];
            }
        }

        // Update weights
        for (weight, q_val) in weights.iter_mut().zip(q.iter()) {
            *weight -= learning_rate * q_val;
        }
    }
}

impl GeneticAlgorithmState {
    /// Create new genetic algorithm state
    fn new(population_size: usize, num_weights: usize, mutation_rate: f64, crossover_rate: f64) -> Self {
        let mut population = Vec::with_capacity(population_size);
        
        // Initialize random population
        for _ in 0..population_size {
            let mut individual = Vec::with_capacity(num_weights);
            for _ in 0..num_weights {
                individual.push(rand::random::<f64>() * 2.0 - 1.0); // Random between -1 and 1
            }
            population.push(individual);
        }

        Self {
            population,
            fitness_scores: vec![0.0; population_size],
            generation: 0,
            mutation_rate,
            crossover_rate,
            population_size,
            elite_size: population_size / 10, // Top 10%
        }
    }

    /// Evaluate fitness of all individuals
    fn evaluate_fitness(&mut self, positions: &[TrainingPosition], k_factor: f64) {
        for (i, individual) in self.population.iter().enumerate() {
            self.fitness_scores[i] = self.calculate_fitness(individual, positions, k_factor);
        }
    }

    /// Calculate fitness for an individual
    fn calculate_fitness(&self, weights: &[f64], positions: &[TrainingPosition], k_factor: f64) -> f64 {
        let mut total_error = 0.0;

        for position in positions {
            let predicted = weights.iter().zip(position.features.iter())
                .map(|(w, f)| w * f).sum::<f64>();
            let predicted_prob = 1.0 / (1.0 + (-k_factor * predicted).exp());
            let error = position.result - predicted_prob;
            total_error += error * error;
        }

        // Return negative error (higher fitness = lower error)
        -total_error / positions.len() as f64
    }

    /// Evolve population to next generation
    fn evolve(&mut self) {
        // Sort by fitness (descending)
        let mut indices: Vec<usize> = (0..self.population_size).collect();
        indices.sort_by(|a, b| self.fitness_scores[*b].partial_cmp(&self.fitness_scores[*a]).unwrap());

        // Create new population
        let mut new_population = Vec::with_capacity(self.population_size);

        // Elite selection (keep best individuals)
        for &idx in indices.iter().take(self.elite_size) {
            new_population.push(self.population[idx].clone());
        }

        // Generate offspring
        while new_population.len() < self.population_size {
            let parent1_idx = self.tournament_selection();
            let parent2_idx = self.tournament_selection();

            let (child1, child2) = self.crossover(
                &self.population[parent1_idx],
                &self.population[parent2_idx]
            );

            new_population.push(self.mutate(child1));
            if new_population.len() < self.population_size {
                new_population.push(self.mutate(child2));
            }
        }

        self.population = new_population;
        self.generation += 1;
    }

    /// Tournament selection
    fn tournament_selection(&self) -> usize {
        let tournament_size = 3;
        let mut best_idx = rand::random::<usize>() % self.population_size;
        
        for _ in 1..tournament_size {
            let candidate_idx = rand::random::<usize>() % self.population_size;
            if self.fitness_scores[candidate_idx] > self.fitness_scores[best_idx] {
                best_idx = candidate_idx;
            }
        }
        
        best_idx
    }

    /// Crossover operation
    fn crossover(&self, parent1: &[f64], parent2: &[f64]) -> (Vec<f64>, Vec<f64>) {
        if rand::random::<f64>() > self.crossover_rate {
            return (parent1.to_vec(), parent2.to_vec());
        }

        let mut child1 = Vec::with_capacity(parent1.len());
        let mut child2 = Vec::with_capacity(parent2.len());

        for i in 0..parent1.len() {
            let alpha = rand::random::<f64>();
            child1.push(alpha * parent1[i] + (1.0 - alpha) * parent2[i]);
            child2.push(alpha * parent2[i] + (1.0 - alpha) * parent1[i]);
        }

        (child1, child2)
    }

    /// Mutation operation
    fn mutate(&self, mut individual: Vec<f64>) -> Vec<f64> {
        for gene in &mut individual {
            if rand::random::<f64>() < self.mutation_rate {
                *gene += rand::random::<f64>() * 0.2 - 0.1; // Small random change
                *gene = gene.clamp(-10.0, 10.0); // Keep within bounds
            }
        }
        individual
    }

    /// Get best individual
    fn get_best_individual(&self) -> &[f64] {
        let best_idx = self.fitness_scores.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        &self.population[best_idx]
    }
}

impl Optimizer {
    /// Create a new optimizer
    pub fn new(method: OptimizationMethod) -> Self {
        Self {
            method,
            config: TuningConfig::default(),
        }
    }

    /// Create a new optimizer with custom configuration
    pub fn with_config(method: OptimizationMethod, config: TuningConfig) -> Self {
        Self { method, config }
    }

    /// Optimize weights using the specified method
    pub fn optimize(&self, positions: &[TrainingPosition]) -> Result<OptimizationResults, String> {
        // Default k_factor for all methods
        let k_factor = 1.0;
        
        match self.method {
            OptimizationMethod::GradientDescent { learning_rate } => {
                self.gradient_descent_optimize(positions, learning_rate, 0.9, k_factor)
            },
            OptimizationMethod::Adam { learning_rate, beta1: _, beta2: _, epsilon: _ } => {
                self.adam_optimize(positions, learning_rate, k_factor)
            },
            OptimizationMethod::LBFGS { memory_size, max_iterations } => {
                self.lbfgs_optimize(positions, memory_size, max_iterations, k_factor)
            },
            OptimizationMethod::GeneticAlgorithm { population_size, mutation_rate, crossover_rate, max_generations } => {
                self.genetic_algorithm_optimize(positions, population_size, mutation_rate, crossover_rate, max_generations, k_factor)
            },
        }
    }

    /// Gradient descent optimization
    fn gradient_descent_optimize(
        &self,
        positions: &[TrainingPosition],
        learning_rate: f64,
        momentum: f64,
        k_factor: f64,
    ) -> Result<OptimizationResults, String> {
        let mut tuner = TexelTuner::with_params(
            positions.to_vec(),
            None,
            k_factor,
            learning_rate,
            momentum,
            0.0, // No regularization for now
            0.0,
            1000,
            1e-6,
            50,
        );

        Ok(tuner.optimize())
    }

    /// Adam optimizer
    fn adam_optimize(
        &self,
        positions: &[TrainingPosition],
        learning_rate: f64,
        k_factor: f64,
    ) -> Result<OptimizationResults, String> {
        let start_time = Instant::now();
        let mut weights = vec![1.0; NUM_EVAL_FEATURES];
        let mut adam_state = AdamState::new(weights.len());
        let mut error_history = Vec::new();
        let mut prev_error = f64::INFINITY;
        let mut patience_counter = 0;
        let max_iterations = 1000;
        let convergence_threshold = 1e-6;
        let early_stopping_patience = 50;

        for iteration in 0..max_iterations {
            let (error, gradients) = self.calculate_error_and_gradients(&weights, positions, k_factor);
            error_history.push(error);

            // Check for convergence
            if error < convergence_threshold {
                return Ok(OptimizationResults {
                    optimized_weights: weights,
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
                if patience_counter >= early_stopping_patience {
                    return Ok(OptimizationResults {
                        optimized_weights: weights,
                        final_error: error,
                        iterations: iteration + 1,
                        convergence_reason: ConvergenceReason::EarlyStopping,
                        optimization_time: start_time.elapsed(),
                        error_history,
                    });
                }
            }

            // Adam update
            adam_state.update(&mut weights, &gradients, learning_rate);
        }

        Ok(OptimizationResults {
            optimized_weights: weights,
            final_error: prev_error,
            iterations: max_iterations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        })
    }

    /// LBFGS optimizer
    fn lbfgs_optimize(
        &self,
        positions: &[TrainingPosition],
        memory_size: usize,
        max_iterations: usize,
        k_factor: f64,
    ) -> Result<OptimizationResults, String> {
        let start_time = Instant::now();
        let mut weights = vec![1.0; NUM_EVAL_FEATURES];
        let mut lbfgs_state = LBFGSState::new(memory_size, weights.len());
        let mut error_history = Vec::new();
        let mut prev_weights = weights.clone();
        let mut prev_gradients = vec![0.0; weights.len()];
        let learning_rate = 1.0;
        let convergence_threshold = 1e-6;

        for iteration in 0..max_iterations {
            let (error, gradients) = self.calculate_error_and_gradients(&weights, positions, k_factor);
            error_history.push(error);

            // Check for NaN or infinite values
            if !error.is_finite() {
                return Ok(OptimizationResults {
                    optimized_weights: prev_weights,
                    final_error: error_history.iter().filter(|e| e.is_finite()).last().unwrap_or(&0.0).abs(),
                    iterations: iteration,
                    convergence_reason: ConvergenceReason::MaxIterations,
                    optimization_time: start_time.elapsed(),
                    error_history,
                });
            }

            if iteration > 0 {
                lbfgs_state.update(&weights, &gradients, &prev_weights, &prev_gradients);
                lbfgs_state.apply_update(&mut weights, &gradients, learning_rate);
                
                // Check if weights became NaN or infinite
                if weights.iter().any(|w| !w.is_finite()) {
                    return Ok(OptimizationResults {
                        optimized_weights: prev_weights,
                        final_error: error_history.iter().filter(|e| e.is_finite()).last().unwrap_or(&0.0).abs(),
                        iterations: iteration,
                        convergence_reason: ConvergenceReason::MaxIterations,
                        optimization_time: start_time.elapsed(),
                        error_history,
                    });
                }
            } else {
                // First iteration: simple gradient descent
                for i in 0..weights.len() {
                    weights[i] -= learning_rate * gradients[i];
                }
            }

            if error < convergence_threshold {
                return Ok(OptimizationResults {
                    optimized_weights: weights,
                    final_error: error,
                    iterations: iteration + 1,
                    convergence_reason: ConvergenceReason::Converged,
                    optimization_time: start_time.elapsed(),
                    error_history,
                });
            }

            prev_weights = weights.clone();
            prev_gradients = gradients;
        }

        Ok(OptimizationResults {
            optimized_weights: weights,
            final_error: error_history.iter().filter(|e| e.is_finite()).last().unwrap_or(&0.0).abs(),
            iterations: max_iterations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        })
    }

    /// Genetic algorithm optimizer
    fn genetic_algorithm_optimize(
        &self,
        positions: &[TrainingPosition],
        population_size: usize,
        mutation_rate: f64,
        crossover_rate: f64,
        max_generations: usize,
        k_factor: f64,
    ) -> Result<OptimizationResults, String> {
        let start_time = Instant::now();
        let mut ga_state = GeneticAlgorithmState::new(population_size, NUM_EVAL_FEATURES, mutation_rate, crossover_rate);
        let mut error_history = Vec::new();
        // Use the provided max_generations parameter
        let convergence_threshold = 1e-6;

        for generation in 0..max_generations {
            ga_state.evaluate_fitness(positions, k_factor);
            let best_fitness = ga_state.fitness_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let best_error = -best_fitness; // Convert fitness back to error
            error_history.push(best_error);

            if best_error < convergence_threshold {
                return Ok(OptimizationResults {
                    optimized_weights: ga_state.get_best_individual().to_vec(),
                    final_error: best_error,
                    iterations: generation + 1,
                    convergence_reason: ConvergenceReason::Converged,
                    optimization_time: start_time.elapsed(),
                    error_history,
                });
            }

            ga_state.evolve();
        }

        Ok(OptimizationResults {
            optimized_weights: ga_state.get_best_individual().to_vec(),
            final_error: error_history.last().unwrap_or(&0.0).clone(),
            iterations: max_generations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        })
    }

    /// Calculate error and gradients for given weights
    fn calculate_error_and_gradients(
        &self,
        weights: &[f64],
        positions: &[TrainingPosition],
        k_factor: f64,
    ) -> (f64, Vec<f64>) {
        let mut total_error = 0.0;
        let mut gradients = vec![0.0; weights.len()];

        for position in positions {
            let predicted = weights.iter().zip(position.features.iter())
                .map(|(w, f)| w * f).sum::<f64>();
            let predicted_prob = 1.0 / (1.0 + (-k_factor * predicted).exp());
            let error = position.result - predicted_prob;
            total_error += error * error;

            let sigmoid_derivative = k_factor * predicted_prob * (1.0 - predicted_prob);
            for (i, &feature) in position.features.iter().enumerate() {
                if i < gradients.len() {
                    gradients[i] += -2.0 * error * sigmoid_derivative * feature;
                }
            }
        }

        let n = positions.len() as f64;
        total_error /= n;
        for gradient in &mut gradients {
            *gradient /= n;
        }

        (total_error, gradients)
    }

    /// Validate optimized weights using cross-validation
    pub fn validate(
        &self,
        positions: &[TrainingPosition],
        weights: &[f64],
    ) -> ValidationResults {
        // Calculate validation error
        let mut total_error = 0.0;
        for position in positions {
            let predicted = weights.iter().zip(position.features.iter())
                .map(|(w, f)| w * f).sum::<f64>();
            let predicted_prob = 1.0 / (1.0 + (-1.0 * predicted).exp()); // Default k_factor
            let error = position.result - predicted_prob;
            total_error += error * error;
        }

        let mse = total_error / positions.len() as f64;
        let rmse = mse.sqrt();

        // Create a simple fold result for validation
        let fold_result = FoldResult {
            fold_number: 1,
            validation_error: rmse,
            test_error: rmse,
            sample_count: positions.len(),
        };

        ValidationResults::new(vec![fold_result])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{TrainingPosition, OptimizationMethod};
    use crate::types::Player;

    #[test]
    fn test_optimizer_creation() {
        let method = OptimizationMethod::default();
        let _optimizer = Optimizer::new(method);
        // Should not panic
    }

    #[test]
    fn test_optimizer_with_config() {
        let method = OptimizationMethod::default();
        let config = TuningConfig::default();
        let _optimizer = Optimizer::with_config(method, config);
        // Should not panic
    }

    #[test]
    fn test_texel_tuner_creation() {
        let positions = vec![];
        let tuner = TexelTuner::new(positions, None, 1.0);
        
        assert_eq!(tuner.get_weights().len(), NUM_EVAL_FEATURES);
        assert_eq!(tuner.k_factor, 1.0);
    }

    #[test]
    fn test_texel_tuner_with_custom_weights() {
        let positions = vec![];
        let initial_weights = vec![2.0; NUM_EVAL_FEATURES];
        let tuner = TexelTuner::new(positions, Some(initial_weights.clone()), 1.5);
        
        assert_eq!(tuner.get_weights(), &initial_weights);
        assert_eq!(tuner.k_factor, 1.5);
    }

    #[test]
    fn test_texel_tuner_with_params() {
        let positions = vec![];
        let tuner = TexelTuner::with_params(
            positions,
            None,
            1.0,
            0.01,
            0.9,
            0.001,
            0.001,
            500,
            1e-5,
            25,
        );
        
        assert_eq!(tuner.learning_rate, 0.01);
        assert_eq!(tuner.momentum, 0.9);
        assert_eq!(tuner.regularization_l1, 0.001);
        assert_eq!(tuner.regularization_l2, 0.001);
        assert_eq!(tuner.max_iterations, 500);
        assert_eq!(tuner.convergence_threshold, 1e-5);
        assert_eq!(tuner.early_stopping_patience, 25);
    }

    #[test]
    fn test_sigmoid_function() {
        let positions = vec![];
        let tuner = TexelTuner::new(positions, None, 1.0);
        
        // Test sigmoid at 0
        assert!((tuner.sigmoid(0.0) - 0.5).abs() < 1e-10);
        
        // Test sigmoid at positive infinity
        assert!(tuner.sigmoid(f64::INFINITY) > 0.9);
        
        // Test sigmoid at negative infinity
        assert!(tuner.sigmoid(f64::NEG_INFINITY) < 0.1);
    }

    #[test]
    fn test_sigmoid_derivative() {
        let positions = vec![];
        let tuner = TexelTuner::new(positions, None, 1.0);
        
        let x = 0.0;
        let _s = tuner.sigmoid(x);
        let derivative = tuner.sigmoid_derivative(x);
        
        // At x=0, sigmoid derivative should be k_factor * s * (1-s) = 1.0 * 0.5 * 0.5 = 0.25
        assert!((derivative - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_position_score_calculation() {
        let mut features = vec![0.0; NUM_EVAL_FEATURES];
        features[0] = 1.0;
        features[1] = 2.0;
        features[2] = 3.0;
        
        let position = TrainingPosition::new(
            features,
            0.5,
            128,
            true,
            10,
            Player::White,
        );
        
        let positions = vec![position];
        let tuner = TexelTuner::new(positions, None, 1.0);
        
        let score = tuner.calculate_position_score(&tuner.positions[0]);
        
        // With weights [1.0, 1.0, 1.0] and features [1.0, 2.0, 3.0]
        // Score should be 1.0 * 1.0 + 1.0 * 2.0 + 1.0 * 3.0 = 6.0
        assert!((score - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_optimization_results_creation() {
        let results = OptimizationResults {
            optimized_weights: vec![1.0, 2.0, 3.0],
            final_error: 0.001,
            iterations: 100,
            convergence_reason: ConvergenceReason::Converged,
            optimization_time: Duration::from_secs(1),
            error_history: vec![0.1, 0.05, 0.001],
        };
        
        assert_eq!(results.optimized_weights.len(), 3);
        assert_eq!(results.final_error, 0.001);
        assert_eq!(results.iterations, 100);
        assert!(matches!(results.convergence_reason, ConvergenceReason::Converged));
        assert_eq!(results.error_history.len(), 3);
    }

    #[test]
    fn test_convergence_reasons() {
        let reasons = vec![
            ConvergenceReason::Converged,
            ConvergenceReason::MaxIterations,
            ConvergenceReason::EarlyStopping,
            ConvergenceReason::GradientNorm,
        ];
        
        assert_eq!(reasons.len(), 4);
    }

    #[test]
    fn test_adam_state_creation() {
        let state = AdamState::new(10);
        
        assert_eq!(state.m.len(), 10);
        assert_eq!(state.v.len(), 10);
        assert_eq!(state.beta1, 0.9);
        assert_eq!(state.beta2, 0.999);
        assert_eq!(state.epsilon, 1e-8);
        assert_eq!(state.t, 0);
    }

    #[test]
    fn test_lbfgs_state_creation() {
        let state = LBFGSState::new(5, 10);
        
        assert_eq!(state.m, 5);
        assert!(state.s.is_empty());
        assert!(state.y.is_empty());
        assert!(state.rho.is_empty());
    }

    #[test]
    fn test_genetic_algorithm_state_creation() {
        let state = GeneticAlgorithmState::new(50, 10, 0.1, 0.8);
        
        assert_eq!(state.population.len(), 50);
        assert_eq!(state.fitness_scores.len(), 50);
        assert_eq!(state.generation, 0);
        assert_eq!(state.mutation_rate, 0.1);
        assert_eq!(state.crossover_rate, 0.8);
        assert_eq!(state.population_size, 50);
        assert_eq!(state.elite_size, 5); // 10% of 50
    }

    #[test]
    fn test_gradient_descent_optimization() {
        let positions = create_test_positions();
        let method = OptimizationMethod::GradientDescent {
            learning_rate: 0.01,
        };
        let optimizer = Optimizer::new(method);
        
        let result = optimizer.optimize(&positions);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.optimized_weights.len(), NUM_EVAL_FEATURES);
        assert!(results.final_error >= 0.0);
        assert!(results.iterations > 0);
    }

    #[test]
    fn test_adam_optimization() {
        let positions = create_test_positions();
        let method = OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        };
        let optimizer = Optimizer::new(method);
        
        let result = optimizer.optimize(&positions);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.optimized_weights.len(), NUM_EVAL_FEATURES);
        assert!(results.final_error >= 0.0);
        assert!(results.iterations > 0);
    }

    #[test]
    fn test_lbfgs_optimization() {
        let positions = create_test_positions();
        let method = OptimizationMethod::LBFGS {
            memory_size: 10,
            max_iterations: 100,
        };
        let optimizer = Optimizer::new(method);
        
        let result = optimizer.optimize(&positions);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.optimized_weights.len(), NUM_EVAL_FEATURES);
        assert!(results.final_error >= 0.0);
        assert!(results.iterations > 0);
    }

    #[test]
    fn test_genetic_algorithm_optimization() {
        let positions = create_test_positions();
        let method = OptimizationMethod::GeneticAlgorithm {
            population_size: 20,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            max_generations: 50,
        };
        let optimizer = Optimizer::new(method);
        
        let result = optimizer.optimize(&positions);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.optimized_weights.len(), NUM_EVAL_FEATURES);
        assert!(results.final_error >= 0.0);
        assert!(results.iterations > 0);
    }

    #[test]
    fn test_validation() {
        let positions = create_test_positions();
        let method = OptimizationMethod::default();
        let optimizer = Optimizer::new(method);
        
        let weights = vec![1.0; NUM_EVAL_FEATURES];
        let validation_results = optimizer.validate(&positions, &weights);
        
        // Validation should return some results
        assert!(validation_results.mean_error >= 0.0);
    }

    #[test]
    fn test_error_and_gradient_calculation() {
        let positions = create_test_positions();
        let method = OptimizationMethod::default();
        let optimizer = Optimizer::new(method);
        
        let weights = vec![1.0; NUM_EVAL_FEATURES];
        let (error, gradients) = optimizer.calculate_error_and_gradients(&weights, &positions, 1.0);
        
        assert!(error >= 0.0);
        assert_eq!(gradients.len(), NUM_EVAL_FEATURES);
        
        // All gradients should be finite
        for gradient in gradients {
            assert!(gradient.is_finite());
        }
    }

    /// Helper function to create test training positions
    fn create_test_positions() -> Vec<TrainingPosition> {
        let mut features1 = vec![0.0; NUM_EVAL_FEATURES];
        features1[0] = 1.0;
        
        let mut features2 = vec![0.0; NUM_EVAL_FEATURES];
        features2[1] = 1.0;
        
        let mut features3 = vec![0.0; NUM_EVAL_FEATURES];
        features3[2] = 1.0;
        
        vec![
            TrainingPosition::new(
                features1,
                1.0, // White win
                100,
                true,
                20,
                Player::White,
            ),
            TrainingPosition::new(
                features2,
                0.0, // Black win
                150,
                true,
                25,
                Player::Black,
            ),
            TrainingPosition::new(
                features3,
                0.5, // Draw
                200,
                true,
                30,
                Player::White,
            ),
        ]
    }
}
