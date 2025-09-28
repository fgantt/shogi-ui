# Code Examples for Extending the Tuning System

Examples for extending and customizing the automated tuning system.

## Table of Contents

1. [Custom Optimization Algorithms](#custom-optimization-algorithms)
2. [Custom Feature Extractors](#custom-feature-extractors)
3. [Custom Validation Methods](#custom-validation-methods)
4. [Custom Data Processors](#custom-data-processors)
5. [Performance Extensions](#performance-extensions)
6. [Integration Examples](#integration-examples)

## Custom Optimization Algorithms

### Implementing a New Optimizer

```rust
use crate::tuning::types::{TrainingPosition, OptimizationResults, ConvergenceReason};
use crate::tuning::optimizer::Optimizer;

pub struct CustomOptimizer {
    learning_rate: f64,
    momentum: f64,
    decay: f64,
}

impl CustomOptimizer {
    pub fn new(learning_rate: f64, momentum: f64, decay: f64) -> Self {
        Self { learning_rate, momentum, decay }
    }
    
    pub fn optimize(&mut self, positions: &[TrainingPosition], iterations: usize) -> OptimizationResults {
        let mut weights = vec![0.0; 2000]; // Initialize weights
        let mut velocity = vec![0.0; 2000]; // Momentum
        let mut error_history = Vec::new();
        
        for iteration in 0..iterations {
            let (error, gradients) = self.calculate_error_and_gradients(&positions, &weights);
            error_history.push(error);
            
            // Update weights with momentum and decay
            for (i, gradient) in gradients.iter().enumerate() {
                velocity[i] = self.momentum * velocity[i] + self.learning_rate * gradient;
                weights[i] -= velocity[i];
                weights[i] *= (1.0 - self.decay); // Weight decay
            }
            
            // Check convergence
            if self.check_convergence(&error_history) {
                return OptimizationResults {
                    final_weights: weights,
                    final_error: error,
                    iterations_completed: iteration + 1,
                    convergence_reason: ConvergenceReason::ConvergenceThresholdMet,
                    error_history,
                    validation_results: None,
                    performance_metrics: Default::default(),
                };
            }
        }
        
        OptimizationResults {
            final_weights: weights,
            final_error: error_history.last().copied().unwrap_or(0.0),
            iterations_completed: iterations,
            convergence_reason: ConvergenceReason::MaxIterationsReached,
            error_history,
            validation_results: None,
            performance_metrics: Default::default(),
        }
    }
    
    fn calculate_error_and_gradients(&self, positions: &[TrainingPosition], weights: &[f64]) -> (f64, Vec<f64>) {
        let mut total_error = 0.0;
        let mut gradients = vec![0.0; weights.len()];
        
        for position in positions {
            let predicted = self.predict_score(position, weights);
            let actual = position.result.to_score();
            let error = predicted - actual;
            
            total_error += error * error;
            
            // Calculate gradients
            for (i, feature) in position.features.iter().enumerate() {
                gradients[i] += 2.0 * error * feature;
            }
        }
        
        // Normalize
        let count = positions.len() as f64;
        total_error /= count;
        for gradient in &mut gradients {
            *gradient /= count;
        }
        
        (total_error, gradients)
    }
    
    fn predict_score(&self, position: &TrainingPosition, weights: &[f64]) -> f64 {
        let mut score = 0.0;
        for (feature, weight) in position.features.iter().zip(weights.iter()) {
            score += feature * weight;
        }
        self.sigmoid(score)
    }
    
    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
    
    fn check_convergence(&self, error_history: &[f64]) -> bool {
        if error_history.len() < 10 {
            return false;
        }
        
        let recent = &error_history[error_history.len()-10..];
        let improvement = recent[0] - recent[recent.len()-1];
        improvement.abs() < 1e-6
    }
}
```

## Custom Feature Extractors

### Adding New Evaluation Features

```rust
use crate::tuning::feature_extractor::FeatureExtractor;
use crate::types::{BitboardBoard, Player, CapturedPieces};

pub struct ExtendedFeatureExtractor {
    base_extractor: FeatureExtractor,
    custom_features: Vec<CustomFeature>,
}

impl ExtendedFeatureExtractor {
    pub fn new() -> Self {
        Self {
            base_extractor: FeatureExtractor::new(),
            custom_features: Vec::new(),
        }
    }
    
    pub fn extract_all_features(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<f64> {
        let mut features = self.base_extractor.extract_features(board, player, captured_pieces);
        
        // Add custom features
        for custom_feature in &self.custom_features {
            let value = custom_feature.extract(board, player, captured_pieces);
            features.push(value);
        }
        
        features
    }
    
    pub fn add_custom_feature(&mut self, feature: CustomFeature) {
        self.custom_features.push(feature);
    }
}

pub trait CustomFeature {
    fn extract(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> f64;
    fn name(&self) -> &str;
}

pub struct PieceActivityFeature {
    threshold: f64,
}

impl CustomFeature for PieceActivityFeature {
    fn extract(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> f64 {
        let mut activity = 0.0;
        
        // Count active pieces (pieces that can move)
        for square in 0..81 {
            if let Some(piece) = board.get_piece_at(square) {
                if piece.player == player {
                    let moves = board.get_pseudo_legal_moves(square);
                    if moves.len() > 0 {
                        activity += 1.0;
                    }
                }
            }
        }
        
        // Normalize by total pieces
        let total_pieces = board.count_pieces(player);
        if total_pieces > 0 {
            activity / total_pieces as f64
        } else {
            0.0
        }
    }
    
    fn name(&self) -> &str {
        "piece_activity"
    }
}

pub struct KingMobilityFeature;

impl CustomFeature for KingMobilityFeature {
    fn extract(&self, board: &BitboardBoard, player: Player, _captured_pieces: &CapturedPieces) -> f64 {
        if let Some(king_pos) = board.find_king(player) {
            let moves = board.get_pseudo_legal_moves(king_pos);
            moves.len() as f64 / 8.0 // Normalize by maximum king moves
        } else {
            0.0
        }
    }
    
    fn name(&self) -> &str {
        "king_mobility"
    }
}
```

## Custom Validation Methods

### Custom Validation Metrics

```rust
use crate::tuning::validator::Validator;
use crate::tuning::types::{TrainingPosition, ValidationResults};

pub struct ExtendedValidator {
    base_validator: Validator,
    custom_metrics: Vec<Box<dyn ValidationMetric>>,
}

impl ExtendedValidator {
    pub fn new() -> Self {
        Self {
            base_validator: Validator::new(Default::default()),
            custom_metrics: Vec::new(),
        }
    }
    
    pub fn validate_with_custom_metrics(&self, positions: &[TrainingPosition], weights: &[f64]) -> ExtendedValidationResults {
        let base_results = self.base_validator.validate(positions, weights);
        
        let mut custom_results = Vec::new();
        for metric in &self.custom_metrics {
            let value = metric.calculate(positions, weights);
            custom_results.push(CustomMetricResult {
                name: metric.name().to_string(),
                value,
            });
        }
        
        ExtendedValidationResults {
            base_results,
            custom_results,
        }
    }
    
    pub fn add_custom_metric(&mut self, metric: Box<dyn ValidationMetric>) {
        self.custom_metrics.push(metric);
    }
}

pub trait ValidationMetric {
    fn calculate(&self, positions: &[TrainingPosition], weights: &[f64]) -> f64;
    fn name(&self) -> &str;
}

pub struct PositionAccuracyMetric {
    threshold: f64,
}

impl ValidationMetric for PositionAccuracyMetric {
    fn calculate(&self, positions: &[TrainingPosition], weights: &[f64]) -> f64 {
        let mut correct = 0;
        let mut total = 0;
        
        for position in positions {
            let predicted = self.predict_score(position, weights);
            let actual = position.result.to_score();
            
            if (predicted - actual).abs() < self.threshold {
                correct += 1;
            }
            total += 1;
        }
        
        correct as f64 / total as f64
    }
    
    fn name(&self) -> &str {
        "position_accuracy"
    }
}

impl PositionAccuracyMetric {
    fn predict_score(&self, position: &TrainingPosition, weights: &[f64]) -> f64 {
        let mut score = 0.0;
        for (feature, weight) in position.features.iter().zip(weights.iter()) {
            score += feature * weight;
        }
        1.0 / (1.0 + (-score).exp()) // Sigmoid
    }
}

#[derive(Debug)]
pub struct ExtendedValidationResults {
    pub base_results: ValidationResults,
    pub custom_results: Vec<CustomMetricResult>,
}

#[derive(Debug)]
pub struct CustomMetricResult {
    pub name: String,
    pub value: f64,
}
```

## Custom Data Processors

### Custom Game Filters

```rust
use crate::tuning::data_processor::{DataProcessor, PositionFilter};
use crate::tuning::types::{GameRecord, TrainingPosition};

pub struct AdvancedDataProcessor {
    base_processor: DataProcessor,
    custom_filters: Vec<Box<dyn GameFilter>>,
}

impl AdvancedDataProcessor {
    pub fn new() -> Self {
        Self {
            base_processor: DataProcessor::new(Default::default()),
            custom_filters: Vec::new(),
        }
    }
    
    pub fn process_game_with_filters(&self, game: &GameRecord) -> Vec<TrainingPosition> {
        // Apply custom filters first
        for filter in &self.custom_filters {
            if !filter.should_include(game) {
                return Vec::new(); // Skip game
            }
        }
        
        // Use base processor
        self.base_processor.process_game(game)
    }
    
    pub fn add_custom_filter(&mut self, filter: Box<dyn GameFilter>) {
        self.custom_filters.push(filter);
    }
}

pub trait GameFilter {
    fn should_include(&self, game: &GameRecord) -> bool;
    fn name(&self) -> &str;
}

pub struct RatingDifferenceFilter {
    max_difference: u16,
}

impl GameFilter for RatingDifferenceFilter {
    fn should_include(&self, game: &GameRecord) -> bool {
        if let (Some(white_rating), Some(black_rating)) = (game.white_rating, game.black_rating) {
            let difference = (white_rating as i32 - black_rating as i32).abs() as u16;
            difference <= self.max_difference
        } else {
            true // Include if ratings unknown
        }
    }
    
    fn name(&self) -> &str {
        "rating_difference"
    }
}

pub struct OpeningVarietyFilter {
    opening_counts: std::collections::HashMap<String, usize>,
    max_per_opening: usize,
}

impl GameFilter for OpeningVarietyFilter {
    fn should_include(&self, game: &GameRecord) -> bool {
        let opening = self.extract_opening(game);
        let count = self.opening_counts.get(&opening).copied().unwrap_or(0);
        count < self.max_per_opening
    }
    
    fn name(&self) -> &str {
        "opening_variety"
    }
}

impl OpeningVarietyFilter {
    fn extract_opening(&self, game: &GameRecord) -> String {
        // Extract first few moves as opening signature
        game.moves.iter().take(6).collect::<Vec<_>>().join(" ")
    }
}
```

## Performance Extensions

### Parallel Feature Extraction

```rust
use rayon::prelude::*;
use std::sync::Arc;

pub struct ParallelFeatureExtractor {
    extractor: Arc<FeatureExtractor>,
    thread_pool: rayon::ThreadPool,
}

impl ParallelFeatureExtractor {
    pub fn new(num_threads: usize) -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
            
        Self {
            extractor: Arc::new(FeatureExtractor::new()),
            thread_pool,
        }
    }
    
    pub fn extract_features_parallel(&self, positions: &[(BitboardBoard, Player, CapturedPieces)]) -> Vec<Vec<f64>> {
        self.thread_pool.install(|| {
            positions.par_iter().map(|(board, player, captured_pieces)| {
                self.extractor.extract_features(board, *player, captured_pieces)
            }).collect()
        })
    }
    
    pub fn process_games_parallel(&self, games: &[GameRecord]) -> Vec<TrainingPosition> {
        self.thread_pool.install(|| {
            games.par_iter().flat_map(|game| {
                self.extractor.process_game(game)
            }).collect()
        })
    }
}
```

### Memory-Efficient Processing

```rust
pub struct StreamingDataProcessor {
    chunk_size: usize,
    extractor: FeatureExtractor,
}

impl StreamingDataProcessor {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            extractor: FeatureExtractor::new(),
        }
    }
    
    pub fn process_stream<R: std::io::Read>(&self, mut reader: R) -> impl Iterator<Item = TrainingPosition> + '_ {
        let mut buffer = String::new();
        let mut chunk = Vec::new();
        
        std::iter::from_fn(move || {
            loop {
                if chunk.len() >= self.chunk_size {
                    if let Some(position) = chunk.pop() {
                        return Some(position);
                    }
                }
                
                if reader.read_to_string(&mut buffer).unwrap() == 0 {
                    return chunk.pop();
                }
                
                // Process buffer and add to chunk
                let new_positions = self.process_buffer(&buffer);
                chunk.extend(new_positions);
                buffer.clear();
            }
        })
    }
    
    fn process_buffer(&self, buffer: &str) -> Vec<TrainingPosition> {
        // Parse and extract features from buffer
        Vec::new() // Implementation depends on format
    }
}
```

## Integration Examples

### Engine Integration

```rust
use crate::evaluation::PositionEvaluator;
use crate::weights::WeightManager;

pub struct TunedEngine {
    evaluator: PositionEvaluator,
    custom_features: ExtendedFeatureExtractor,
}

impl TunedEngine {
    pub fn new(weights_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut evaluator = PositionEvaluator::new();
        evaluator.load_tuned_weights(weights_path)?;
        evaluator.set_use_tuned_weights(true);
        
        let mut custom_features = ExtendedFeatureExtractor::new();
        custom_features.add_custom_feature(Box::new(PieceActivityFeature { threshold: 0.5 }));
        custom_features.add_custom_feature(Box::new(KingMobilityFeature));
        
        Ok(Self {
            evaluator,
            custom_features,
        })
    }
    
    pub fn evaluate_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        // Extract custom features
        let custom_features = self.custom_features.extract_all_features(board, player, captured_pieces);
        
        // Use tuned evaluation
        self.evaluator.evaluate_with_tuned_weights(board, player, captured_pieces)
    }
    
    pub fn analyze_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> PositionAnalysis {
        let features = self.custom_features.extract_all_features(board, player, captured_pieces);
        let score = self.evaluate_position(board, player, captured_pieces);
        
        PositionAnalysis {
            score,
            features,
            custom_feature_count: features.len() - 2000, // Subtract base features
        }
    }
}

#[derive(Debug)]
pub struct PositionAnalysis {
    pub score: i32,
    pub features: Vec<f64>,
    pub custom_feature_count: usize,
}
```

### Custom Tuning Pipeline

```rust
pub struct CustomTuningPipeline {
    data_processor: AdvancedDataProcessor,
    feature_extractor: ExtendedFeatureExtractor,
    optimizer: CustomOptimizer,
    validator: ExtendedValidator,
}

impl CustomTuningPipeline {
    pub fn new() -> Self {
        let mut data_processor = AdvancedDataProcessor::new();
        data_processor.add_custom_filter(Box::new(RatingDifferenceFilter { max_difference: 200 }));
        data_processor.add_custom_filter(Box::new(OpeningVarietyFilter {
            opening_counts: std::collections::HashMap::new(),
            max_per_opening: 100,
        }));
        
        let mut feature_extractor = ExtendedFeatureExtractor::new();
        feature_extractor.add_custom_feature(Box::new(PieceActivityFeature { threshold: 0.5 }));
        
        let mut validator = ExtendedValidator::new();
        validator.add_custom_metric(Box::new(PositionAccuracyMetric { threshold: 0.1 }));
        
        Self {
            data_processor,
            feature_extractor,
            optimizer: CustomOptimizer::new(0.01, 0.9, 0.0001),
            validator,
        }
    }
    
    pub fn run_tuning(&mut self, games: &[GameRecord], iterations: usize) -> TuningResults {
        // Process data
        let positions: Vec<_> = games.iter()
            .flat_map(|game| self.data_processor.process_game_with_filters(game))
            .collect();
        
        // Run optimization
        let optimization_results = self.optimizer.optimize(&positions, iterations);
        
        // Validate results
        let validation_results = self.validator.validate_with_custom_metrics(&positions, &optimization_results.final_weights);
        
        TuningResults {
            weights: optimization_results.final_weights,
            final_error: optimization_results.final_error,
            validation_results,
            iterations: optimization_results.iterations_completed,
        }
    }
}

#[derive(Debug)]
pub struct TuningResults {
    pub weights: Vec<f64>,
    pub final_error: f64,
    pub validation_results: ExtendedValidationResults,
    pub iterations: usize,
}
```

## Next Steps

- [API Documentation](API_DOCUMENTATION.md) for detailed API reference
- [Integration Guide](INTEGRATION_GUIDE.md) for engine integration
- [User Guide](USER_GUIDE.md) for usage instructions
