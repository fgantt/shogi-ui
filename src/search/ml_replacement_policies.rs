//! Machine Learning Replacement Policies
//!
//! This module implements machine learning-based replacement policies for
//! transposition tables, learning optimal replacement decisions based on
//! access patterns, game characteristics, and performance metrics.

use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// ML replacement policy configuration
#[derive(Debug, Clone)]
pub struct MLReplacementConfig {
    /// ML algorithm to use
    pub algorithm: MLAlgorithm,
    /// Learning rate for online learning
    pub learning_rate: f64,
    /// Feature extraction settings
    pub feature_settings: FeatureExtractionSettings,
    /// Training data collection settings
    pub training_settings: TrainingDataSettings,
    /// Performance monitoring settings
    pub performance_settings: PerformanceMonitoringSettings,
}

/// ML algorithms available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MLAlgorithm {
    /// Linear regression for simple patterns
    LinearRegression,
    /// Decision tree for interpretable decisions
    DecisionTree,
    /// Random forest for ensemble learning
    RandomForest,
    /// Neural network for complex patterns
    NeuralNetwork,
    /// Reinforcement learning for adaptive policies
    ReinforcementLearning,
}

/// Feature extraction settings
#[derive(Debug, Clone)]
pub struct FeatureExtractionSettings {
    /// Enable position-based features
    pub enable_position_features: bool,
    /// Enable access pattern features
    pub enable_access_pattern_features: bool,
    /// Enable entry characteristic features
    pub enable_entry_features: bool,
    /// Enable temporal features
    pub enable_temporal_features: bool,
    /// Feature normalization
    pub normalize_features: bool,
}

/// Training data collection settings
#[derive(Debug, Clone)]
pub struct TrainingDataSettings {
    /// Maximum training samples to keep
    pub max_training_samples: usize,
    /// Minimum samples before training
    pub min_samples_for_training: usize,
    /// Training frequency (every N decisions)
    pub training_frequency: usize,
    /// Enable online learning
    pub enable_online_learning: bool,
}

/// Performance monitoring settings
#[derive(Debug, Clone)]
pub struct PerformanceMonitoringSettings {
    /// Enable performance tracking
    pub enable_performance_tracking: bool,
    /// Performance evaluation window
    pub evaluation_window: Duration,
    /// Performance threshold for policy updates
    pub performance_threshold: f64,
}

/// ML replacement context
#[derive(Debug, Clone)]
pub struct MLReplacementContext {
    /// Current hash being accessed
    pub current_hash: u64,
    /// Existing entry in table
    pub existing_entry: Option<TranspositionEntry>,
    /// New entry to potentially store
    pub new_entry: TranspositionEntry,
    /// Access pattern information
    pub access_pattern: AccessPatternInfo,
    /// Position characteristics
    pub position_features: PositionFeatures,
    /// Temporal information
    pub temporal_info: TemporalInfo,
}

/// Access pattern information
#[derive(Debug, Clone)]
pub struct AccessPatternInfo {
    /// Recent access frequency
    pub recent_frequency: f64,
    /// Depth-based access pattern
    pub depth_pattern: f64,
    /// Sibling node access
    pub sibling_access: f64,
    /// Parent node access
    pub parent_access: f64,
}

/// Position features
#[derive(Debug, Clone)]
pub struct PositionFeatures {
    /// Board complexity
    pub complexity: f64,
    /// Tactical features
    pub tactical_score: f64,
    /// Positional features
    pub positional_score: f64,
    /// Material balance
    pub material_balance: f64,
}

/// Temporal information
#[derive(Debug, Clone)]
pub struct TemporalInfo {
    /// Current time
    pub timestamp: Instant,
    /// Age of existing entry
    pub existing_age: Duration,
    /// Time since last access
    pub time_since_access: Duration,
}

/// ML replacement decision
#[derive(Debug, Clone)]
pub struct MLReplacementDecision {
    /// Decision (keep existing, replace, or store new)
    pub decision: ReplacementAction,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Features used for decision
    pub features: Vec<f64>,
    /// Algorithm used
    pub algorithm_used: MLAlgorithm,
}

/// Replacement action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementAction {
    /// Keep existing entry
    KeepExisting,
    /// Replace with new entry
    ReplaceWithNew,
    /// Store new entry in different slot
    StoreNewElsewhere,
}

/// ML replacement policy
pub struct MLReplacementPolicy {
    /// Configuration
    config: MLReplacementConfig,
    /// Feature extractors
    feature_extractors: FeatureExtractors,
    /// ML models
    models: HashMap<MLAlgorithm, Box<dyn MLModel>>,
    /// Training data collector
    training_collector: TrainingDataCollector,
    /// Performance monitor
    performance_monitor: MLPerformanceMonitor,
    /// Feature scaler
    feature_scaler: FeatureScaler,
}

/// Feature extractors
#[derive(Debug)]
pub struct FeatureExtractors {
    /// Position feature extractor
    pub position_extractor: PositionFeatureExtractor,
    /// Access pattern extractor
    pub access_pattern_extractor: AccessPatternExtractor,
    /// Entry feature extractor
    pub entry_extractor: EntryFeatureExtractor,
    /// Temporal feature extractor
    pub temporal_extractor: TemporalFeatureExtractor,
}

/// ML model trait
pub trait MLModel {
    /// Predict replacement decision
    fn predict(&self, features: &[f64]) -> MLReplacementDecision;
    /// Train the model
    fn train(&mut self, training_data: &TrainingDataset);
    /// Update model with new sample
    fn update(&mut self, features: &[f64], target: &MLReplacementDecision);
}

/// Training dataset
#[derive(Debug)]
pub struct TrainingDataset {
    /// Feature vectors
    pub features: Vec<Vec<f64>>,
    /// Target decisions
    pub targets: Vec<MLReplacementDecision>,
    /// Sample weights
    pub weights: Vec<f64>,
}

/// Training data collector
pub struct TrainingDataCollector {
    /// Collected training samples
    samples: VecDeque<TrainingSample>,
    /// Sample counter
    sample_count: u64,
    /// Last training time
    last_training: Instant,
}

/// Training sample
#[derive(Debug, Clone)]
pub struct TrainingSample {
    /// Features
    pub features: Vec<f64>,
    /// Target decision
    pub target: MLReplacementDecision,
    /// Outcome (was decision beneficial)
    pub outcome: ReplacementOutcome,
    /// Timestamp
    pub timestamp: Instant,
}

/// Replacement outcome
#[derive(Debug, Clone)]
pub struct ReplacementOutcome {
    /// Was the decision beneficial
    pub beneficial: bool,
    /// Performance impact
    pub performance_impact: f64,
    /// Cache efficiency after decision
    pub cache_efficiency: f64,
    /// Access frequency after decision
    pub access_frequency: f64,
}

/// ML performance monitor
pub struct MLPerformanceMonitor {
    /// Performance metrics
    metrics: MLPerformanceMetrics,
    /// Performance history
    _history: VecDeque<PerformanceSnapshot>,
    /// Last evaluation time
    _last_evaluation: Instant,
}

/// ML performance metrics
#[derive(Debug, Clone, Default)]
pub struct MLPerformanceMetrics {
    /// Total decisions made
    pub total_decisions: u64,
    /// Correct decisions
    pub correct_decisions: u64,
    /// Average confidence
    pub avg_confidence: f64,
    /// Average performance impact
    pub avg_performance_impact: f64,
    /// Cache hit rate improvement
    pub cache_hit_improvement: f64,
}

/// Performance snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Metrics at this point
    pub metrics: MLPerformanceMetrics,
    /// Timestamp
    pub timestamp: Instant,
}

/// Feature scaler
pub struct FeatureScaler {
    /// Mean values for normalization
    means: Vec<f64>,
    /// Standard deviations for normalization
    std_devs: Vec<f64>,
    /// Whether scaling is enabled
    enabled: bool,
}

impl MLReplacementConfig {
    /// Create linear regression configuration
    pub fn linear_regression() -> Self {
        Self {
            algorithm: MLAlgorithm::LinearRegression,
            learning_rate: 0.01,
            feature_settings: FeatureExtractionSettings {
                enable_position_features: true,
                enable_access_pattern_features: true,
                enable_entry_features: true,
                enable_temporal_features: false,
                normalize_features: true,
            },
            training_settings: TrainingDataSettings {
                max_training_samples: 10000,
                min_samples_for_training: 100,
                training_frequency: 50,
                enable_online_learning: true,
            },
            performance_settings: PerformanceMonitoringSettings {
                enable_performance_tracking: true,
                evaluation_window: Duration::from_secs(60),
                performance_threshold: 0.05,
            },
        }
    }

    /// Create neural network configuration
    pub fn neural_network() -> Self {
        Self {
            algorithm: MLAlgorithm::NeuralNetwork,
            learning_rate: 0.001,
            feature_settings: FeatureExtractionSettings {
                enable_position_features: true,
                enable_access_pattern_features: true,
                enable_entry_features: true,
                enable_temporal_features: true,
                normalize_features: true,
            },
            training_settings: TrainingDataSettings {
                max_training_samples: 50000,
                min_samples_for_training: 1000,
                training_frequency: 100,
                enable_online_learning: true,
            },
            performance_settings: PerformanceMonitoringSettings {
                enable_performance_tracking: true,
                evaluation_window: Duration::from_secs(30),
                performance_threshold: 0.03,
            },
        }
    }

    /// Create reinforcement learning configuration
    pub fn reinforcement_learning() -> Self {
        Self {
            algorithm: MLAlgorithm::ReinforcementLearning,
            learning_rate: 0.1,
            feature_settings: FeatureExtractionSettings {
                enable_position_features: true,
                enable_access_pattern_features: true,
                enable_entry_features: true,
                enable_temporal_features: true,
                normalize_features: true,
            },
            training_settings: TrainingDataSettings {
                max_training_samples: 100000,
                min_samples_for_training: 500,
                training_frequency: 25,
                enable_online_learning: true,
            },
            performance_settings: PerformanceMonitoringSettings {
                enable_performance_tracking: true,
                evaluation_window: Duration::from_secs(15),
                performance_threshold: 0.02,
            },
        }
    }
}

impl Default for MLReplacementConfig {
    fn default() -> Self {
        Self::linear_regression()
    }
}

impl MLReplacementPolicy {
    /// Create a new ML replacement policy
    pub fn new(config: MLReplacementConfig) -> Self {
        let mut models = HashMap::new();

        // Initialize models based on configuration
        match config.algorithm {
            MLAlgorithm::LinearRegression => {
                models.insert(
                    MLAlgorithm::LinearRegression,
                    Box::new(LinearRegressionModel::new()) as Box<dyn MLModel>,
                );
            }
            MLAlgorithm::DecisionTree => {
                models.insert(
                    MLAlgorithm::DecisionTree,
                    Box::new(DecisionTreeModel::new()) as Box<dyn MLModel>,
                );
            }
            MLAlgorithm::RandomForest => {
                models.insert(
                    MLAlgorithm::RandomForest,
                    Box::new(RandomForestModel::new()) as Box<dyn MLModel>,
                );
            }
            MLAlgorithm::NeuralNetwork => {
                models.insert(
                    MLAlgorithm::NeuralNetwork,
                    Box::new(NeuralNetworkModel::new()) as Box<dyn MLModel>,
                );
            }
            MLAlgorithm::ReinforcementLearning => {
                models.insert(
                    MLAlgorithm::ReinforcementLearning,
                    Box::new(RLModel::new()) as Box<dyn MLModel>,
                );
            }
        }

        Self {
            config,
            feature_extractors: FeatureExtractors::new(),
            models,
            training_collector: TrainingDataCollector::new(),
            performance_monitor: MLPerformanceMonitor::new(),
            feature_scaler: FeatureScaler::new(),
        }
    }

    /// Make a replacement decision using ML
    pub fn decide_replacement(&mut self, context: &MLReplacementContext) -> MLReplacementDecision {
        // Extract features
        let features = self.extract_features(context);

        // Scale features if enabled
        let scaled_features = if self.config.feature_settings.normalize_features {
            self.feature_scaler.scale(&features)
        } else {
            features.clone()
        };

        // Get model prediction
        let model = self.models.get(&self.config.algorithm).unwrap();
        let mut decision = model.predict(&scaled_features);

        // Add feature information
        decision.features = features;
        decision.algorithm_used = self.config.algorithm;

        // Update performance monitoring
        if self.config.performance_settings.enable_performance_tracking {
            self.performance_monitor.record_decision(&decision);
        }

        decision
    }

    /// Record outcome for learning
    pub fn record_outcome(
        &mut self,
        decision: &MLReplacementDecision,
        outcome: ReplacementOutcome,
    ) {
        // Create training sample
        let sample = TrainingSample {
            features: decision.features.clone(),
            target: decision.clone(),
            outcome: outcome.clone(),
            timestamp: Instant::now(),
        };

        // Add to training data
        self.training_collector.add_sample(sample);

        // Check if we should train
        if self.should_train() {
            self.train_models();
        }

        // Update online learning
        if self.config.training_settings.enable_online_learning {
            self.update_online_learning(decision, &outcome);
        }
    }

    /// Extract features from context
    fn extract_features(&self, context: &MLReplacementContext) -> Vec<f64> {
        let mut features = Vec::new();

        if self.config.feature_settings.enable_position_features {
            features.extend(
                self.feature_extractors
                    .position_extractor
                    .extract(&context.position_features),
            );
        }

        if self.config.feature_settings.enable_access_pattern_features {
            features.extend(
                self.feature_extractors
                    .access_pattern_extractor
                    .extract(&context.access_pattern),
            );
        }

        if self.config.feature_settings.enable_entry_features {
            features.extend(
                self.feature_extractors
                    .entry_extractor
                    .extract(&context.new_entry),
            );
            if let Some(ref existing) = context.existing_entry {
                features.extend(self.feature_extractors.entry_extractor.extract(existing));
            } else {
                features.extend(vec![0.0; 5]); // Default values
            }
        }

        if self.config.feature_settings.enable_temporal_features {
            features.extend(
                self.feature_extractors
                    .temporal_extractor
                    .extract(&context.temporal_info),
            );
        }

        features
    }

    /// Check if models should be trained
    fn should_train(&self) -> bool {
        let sample_count = self.training_collector.sample_count;
        let last_training = self.training_collector.last_training;

        sample_count >= self.config.training_settings.min_samples_for_training as u64
            && (sample_count % self.config.training_settings.training_frequency as u64 == 0
                || last_training.elapsed() > Duration::from_secs(300)) // 5 minutes
    }

    /// Train models with collected data
    fn train_models(&mut self) {
        if let Some(dataset) = self.training_collector.create_dataset() {
            // Update feature scaler
            if self.config.feature_settings.normalize_features {
                self.feature_scaler.fit(&dataset.features);
            }

            // Scale features
            let scaled_features = if self.config.feature_settings.normalize_features {
                dataset
                    .features
                    .iter()
                    .map(|f| self.feature_scaler.scale(f))
                    .collect()
            } else {
                dataset.features
            };

            let scaled_dataset = TrainingDataset {
                features: scaled_features,
                targets: dataset.targets,
                weights: dataset.weights,
            };

            // Train the active model
            if let Some(model) = self.models.get_mut(&self.config.algorithm) {
                model.train(&scaled_dataset);
            }

            self.training_collector.last_training = Instant::now();
        }
    }

    /// Update online learning
    fn update_online_learning(
        &mut self,
        decision: &MLReplacementDecision,
        _outcome: &ReplacementOutcome,
    ) {
        if let Some(model) = self.models.get_mut(&self.config.algorithm) {
            let scaled_features = if self.config.feature_settings.normalize_features {
                self.feature_scaler.scale(&decision.features)
            } else {
                decision.features.clone()
            };

            model.update(&scaled_features, decision);
        }
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &MLPerformanceMetrics {
        &self.performance_monitor.metrics
    }
}

// Feature extractor implementations
#[derive(Debug)]
pub struct PositionFeatureExtractor;

impl PositionFeatureExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract(&self, features: &PositionFeatures) -> Vec<f64> {
        vec![
            features.complexity,
            features.tactical_score,
            features.positional_score,
            features.material_balance,
        ]
    }
}

#[derive(Debug)]
pub struct AccessPatternExtractor;

impl AccessPatternExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract(&self, pattern: &AccessPatternInfo) -> Vec<f64> {
        vec![
            pattern.recent_frequency,
            pattern.depth_pattern,
            pattern.sibling_access,
            pattern.parent_access,
        ]
    }
}

#[derive(Debug)]
pub struct EntryFeatureExtractor;

impl EntryFeatureExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract(&self, entry: &TranspositionEntry) -> Vec<f64> {
        vec![
            entry.depth as f64,
            entry.score as f64,
            entry.age as f64,
            match entry.flag {
                TranspositionFlag::Exact => 1.0,
                TranspositionFlag::LowerBound => 0.5,
                TranspositionFlag::UpperBound => 0.0,
            },
            if entry.best_move.is_some() { 1.0 } else { 0.0 },
        ]
    }
}

#[derive(Debug)]
pub struct TemporalFeatureExtractor;

impl TemporalFeatureExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract(&self, info: &TemporalInfo) -> Vec<f64> {
        vec![
            info.existing_age.as_secs() as f64,
            info.time_since_access.as_secs() as f64,
        ]
    }
}

impl FeatureExtractors {
    pub fn new() -> Self {
        Self {
            position_extractor: PositionFeatureExtractor::new(),
            access_pattern_extractor: AccessPatternExtractor::new(),
            entry_extractor: EntryFeatureExtractor::new(),
            temporal_extractor: TemporalFeatureExtractor::new(),
        }
    }
}

// ML model implementations
pub struct LinearRegressionModel {
    weights: Vec<f64>,
    bias: f64,
}

impl LinearRegressionModel {
    pub fn new() -> Self {
        Self {
            weights: Vec::new(),
            bias: 0.0,
        }
    }
}

impl MLModel for LinearRegressionModel {
    fn predict(&self, features: &[f64]) -> MLReplacementDecision {
        if self.weights.is_empty() {
            // Default decision if not trained
            return MLReplacementDecision {
                decision: ReplacementAction::KeepExisting,
                confidence: 0.5,
                features: features.to_vec(),
                algorithm_used: MLAlgorithm::LinearRegression,
            };
        }

        let score = features
            .iter()
            .zip(&self.weights)
            .map(|(f, w)| f * w)
            .sum::<f64>()
            + self.bias;
        let confidence = (score / (score.abs() + 1.0) + 1.0) / 2.0; // Sigmoid-like

        let decision = if score > 0.0 {
            ReplacementAction::ReplaceWithNew
        } else {
            ReplacementAction::KeepExisting
        };

        MLReplacementDecision {
            decision,
            confidence,
            features: features.to_vec(),
            algorithm_used: MLAlgorithm::LinearRegression,
        }
    }

    fn train(&mut self, training_data: &TrainingDataset) {
        // Simple linear regression training
        if training_data.features.is_empty() {
            return;
        }

        let feature_count = training_data.features[0].len();
        self.weights = vec![0.0; feature_count];

        // Simple gradient descent
        let learning_rate = 0.01;
        let iterations = 100;

        for _ in 0..iterations {
            for (features, target) in training_data.features.iter().zip(&training_data.targets) {
                let prediction = self.predict(features).decision as i32 as f64;
                let actual = target.decision as i32 as f64;
                let error = actual - prediction;

                // Update weights
                for (weight, feature) in self.weights.iter_mut().zip(features) {
                    *weight += learning_rate * error * feature;
                }
                self.bias += learning_rate * error;
            }
        }
    }

    fn update(&mut self, features: &[f64], target: &MLReplacementDecision) {
        if self.weights.is_empty() {
            self.weights = vec![0.0; features.len()];
        }

        let prediction = self.predict(features).decision as i32 as f64;
        let actual = target.decision as i32 as f64;
        let error = actual - prediction;
        let learning_rate = 0.01;

        // Update weights
        for (weight, feature) in self.weights.iter_mut().zip(features) {
            *weight += learning_rate * error * feature;
        }
        self.bias += learning_rate * error;
    }
}

// Placeholder implementations for other models
pub struct DecisionTreeModel;
pub struct RandomForestModel;
pub struct NeuralNetworkModel;
pub struct RLModel;

impl DecisionTreeModel {
    pub fn new() -> Self {
        Self
    }
}

impl RandomForestModel {
    pub fn new() -> Self {
        Self
    }
}

impl NeuralNetworkModel {
    pub fn new() -> Self {
        Self
    }
}

impl RLModel {
    pub fn new() -> Self {
        Self
    }
}

impl MLModel for DecisionTreeModel {
    fn predict(&self, features: &[f64]) -> MLReplacementDecision {
        // Simplified decision tree
        let score = if features.len() > 0 { features[0] } else { 0.0 };
        let decision = if score > 0.5 {
            ReplacementAction::ReplaceWithNew
        } else {
            ReplacementAction::KeepExisting
        };

        MLReplacementDecision {
            decision,
            confidence: 0.7,
            features: features.to_vec(),
            algorithm_used: MLAlgorithm::DecisionTree,
        }
    }

    fn train(&mut self, _training_data: &TrainingDataset) {
        // Placeholder implementation
    }

    fn update(&mut self, _features: &[f64], _target: &MLReplacementDecision) {
        // Placeholder implementation
    }
}

impl MLModel for RandomForestModel {
    fn predict(&self, features: &[f64]) -> MLReplacementDecision {
        // Simplified random forest
        let avg_score = features.iter().sum::<f64>() / features.len() as f64;
        let decision = if avg_score > 0.0 {
            ReplacementAction::ReplaceWithNew
        } else {
            ReplacementAction::KeepExisting
        };

        MLReplacementDecision {
            decision,
            confidence: 0.8,
            features: features.to_vec(),
            algorithm_used: MLAlgorithm::RandomForest,
        }
    }

    fn train(&mut self, _training_data: &TrainingDataset) {
        // Placeholder implementation
    }

    fn update(&mut self, _features: &[f64], _target: &MLReplacementDecision) {
        // Placeholder implementation
    }
}

impl MLModel for NeuralNetworkModel {
    fn predict(&self, features: &[f64]) -> MLReplacementDecision {
        // Simplified neural network
        let score = features.iter().map(|f| f.tanh()).sum::<f64>() / features.len() as f64;
        let decision = if score > 0.0 {
            ReplacementAction::ReplaceWithNew
        } else {
            ReplacementAction::KeepExisting
        };

        MLReplacementDecision {
            decision,
            confidence: 0.9,
            features: features.to_vec(),
            algorithm_used: MLAlgorithm::NeuralNetwork,
        }
    }

    fn train(&mut self, _training_data: &TrainingDataset) {
        // Placeholder implementation
    }

    fn update(&mut self, _features: &[f64], _target: &MLReplacementDecision) {
        // Placeholder implementation
    }
}

impl MLModel for RLModel {
    fn predict(&self, features: &[f64]) -> MLReplacementDecision {
        // Simplified reinforcement learning
        let q_value = features.iter().sum::<f64>();
        let decision = if q_value > 0.5 {
            ReplacementAction::ReplaceWithNew
        } else {
            ReplacementAction::KeepExisting
        };

        MLReplacementDecision {
            decision,
            confidence: 0.6,
            features: features.to_vec(),
            algorithm_used: MLAlgorithm::ReinforcementLearning,
        }
    }

    fn train(&mut self, _training_data: &TrainingDataset) {
        // Placeholder implementation
    }

    fn update(&mut self, _features: &[f64], _target: &MLReplacementDecision) {
        // Placeholder implementation
    }
}

// Training data collector implementation
impl TrainingDataCollector {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            sample_count: 0,
            last_training: Instant::now(),
        }
    }

    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.samples.push_back(sample);
        self.sample_count += 1;

        // Maintain maximum sample count
        while self.samples.len() > 10000 {
            self.samples.pop_front();
        }
    }

    pub fn create_dataset(&self) -> Option<TrainingDataset> {
        if self.samples.len() < 10 {
            return None;
        }

        let features: Vec<Vec<f64>> = self.samples.iter().map(|s| s.features.clone()).collect();
        let targets: Vec<MLReplacementDecision> =
            self.samples.iter().map(|s| s.target.clone()).collect();
        let weights: Vec<f64> = self
            .samples
            .iter()
            .map(|s| if s.outcome.beneficial { 1.0 } else { 0.5 })
            .collect();

        Some(TrainingDataset {
            features,
            targets,
            weights,
        })
    }
}

// Performance monitor implementation
impl MLPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: MLPerformanceMetrics::default(),
            _history: VecDeque::new(),
            _last_evaluation: Instant::now(),
        }
    }

    pub fn record_decision(&mut self, decision: &MLReplacementDecision) {
        self.metrics.total_decisions += 1;
        self.metrics.avg_confidence = (self.metrics.avg_confidence
            * (self.metrics.total_decisions - 1) as f64
            + decision.confidence)
            / self.metrics.total_decisions as f64;
    }

    pub fn record_outcome(&mut self, outcome: &ReplacementOutcome) {
        if outcome.beneficial {
            self.metrics.correct_decisions += 1;
        }

        self.metrics.avg_performance_impact = (self.metrics.avg_performance_impact
            * (self.metrics.total_decisions - 1) as f64
            + outcome.performance_impact)
            / self.metrics.total_decisions as f64;
    }
}

// Feature scaler implementation
impl FeatureScaler {
    pub fn new() -> Self {
        Self {
            means: Vec::new(),
            std_devs: Vec::new(),
            enabled: false,
        }
    }

    pub fn fit(&mut self, features: &[Vec<f64>]) {
        if features.is_empty() {
            return;
        }

        let feature_count = features[0].len();
        self.means = vec![0.0; feature_count];
        self.std_devs = vec![0.0; feature_count];

        // Calculate means
        for feature_vector in features {
            for (i, &value) in feature_vector.iter().enumerate() {
                self.means[i] += value;
            }
        }

        for mean in &mut self.means {
            *mean /= features.len() as f64;
        }

        // Calculate standard deviations
        for feature_vector in features {
            for (i, &value) in feature_vector.iter().enumerate() {
                let diff = value - self.means[i];
                self.std_devs[i] += diff * diff;
            }
        }

        for std_dev in &mut self.std_devs {
            *std_dev = (*std_dev / features.len() as f64).sqrt();
            if *std_dev == 0.0 {
                *std_dev = 1.0; // Avoid division by zero
            }
        }

        self.enabled = true;
    }

    pub fn scale(&self, features: &[f64]) -> Vec<f64> {
        if !self.enabled || self.means.is_empty() {
            return features.to_vec();
        }

        features
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                if i < self.means.len() && i < self.std_devs.len() {
                    (value - self.means[i]) / self.std_devs[i]
                } else {
                    value
                }
            })
            .collect()
    }
}

impl Default for MLReplacementPolicy {
    fn default() -> Self {
        Self::new(MLReplacementConfig::default())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_ml_replacement_config() {
        let linear_config = MLReplacementConfig::linear_regression();
        assert_eq!(linear_config.algorithm, MLAlgorithm::LinearRegression);
        assert!(linear_config.feature_settings.enable_position_features);

        let nn_config = MLReplacementConfig::neural_network();
        assert_eq!(nn_config.algorithm, MLAlgorithm::NeuralNetwork);
        assert!(nn_config.feature_settings.enable_temporal_features);

        let rl_config = MLReplacementConfig::reinforcement_learning();
        assert_eq!(rl_config.algorithm, MLAlgorithm::ReinforcementLearning);
        assert!(rl_config.training_settings.enable_online_learning);
    }

    #[test]
    fn test_feature_extraction() {
        let extractors = FeatureExtractors::new();

        let position_features = PositionFeatures {
            complexity: 0.5,
            tactical_score: 0.3,
            positional_score: 0.7,
            material_balance: 0.1,
        };

        let features = extractors.position_extractor.extract(&position_features);
        assert_eq!(features.len(), 4);
        assert_eq!(features[0], 0.5);

        let entry = TranspositionEntry {
            hash_key: 0x1234,
            depth: 5,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
        };

        let entry_features = extractors.entry_extractor.extract(&entry);
        assert_eq!(entry_features.len(), 5);
        assert_eq!(entry_features[0], 5.0); // depth
        assert_eq!(entry_features[3], 1.0); // exact flag
    }

    #[test]
    fn test_ml_replacement_policy() {
        let config = MLReplacementConfig::linear_regression();
        let mut policy = MLReplacementPolicy::new(config);

        let context = MLReplacementContext {
            current_hash: 0x1234,
            existing_entry: None,
            new_entry: TranspositionEntry {
                hash_key: 0x1234,
                depth: 3,
                score: 50,
                flag: TranspositionFlag::Exact,
                best_move: None,
                age: 5,
            },
            access_pattern: AccessPatternInfo {
                recent_frequency: 0.5,
                depth_pattern: 0.3,
                sibling_access: 0.2,
                parent_access: 0.1,
            },
            position_features: PositionFeatures {
                complexity: 0.4,
                tactical_score: 0.6,
                positional_score: 0.3,
                material_balance: 0.1,
            },
            temporal_info: TemporalInfo {
                timestamp: Instant::now(),
                existing_age: Duration::from_secs(10),
                time_since_access: Duration::from_secs(5),
            },
        };

        let decision = policy.decide_replacement(&context);
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
        assert!(!decision.features.is_empty());
        assert_eq!(decision.algorithm_used, MLAlgorithm::LinearRegression);
    }

    #[test]
    fn test_training_data_collection() {
        let mut collector = TrainingDataCollector::new();

        let sample = TrainingSample {
            features: vec![0.1, 0.2, 0.3],
            target: MLReplacementDecision {
                decision: ReplacementAction::KeepExisting,
                confidence: 0.7,
                features: vec![0.1, 0.2, 0.3],
                algorithm_used: MLAlgorithm::LinearRegression,
            },
            outcome: ReplacementOutcome {
                beneficial: true,
                performance_impact: 0.1,
                cache_efficiency: 0.8,
                access_frequency: 0.6,
            },
            timestamp: Instant::now(),
        };

        collector.add_sample(sample);
        assert_eq!(collector.sample_count, 1);

        if let Some(dataset) = collector.create_dataset() {
            assert_eq!(dataset.features.len(), 1);
            assert_eq!(dataset.targets.len(), 1);
            assert_eq!(dataset.weights.len(), 1);
        }
    }

    #[test]
    fn test_feature_scaling() {
        let mut scaler = FeatureScaler::new();

        let features = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 6.0],
            vec![3.0, 6.0, 9.0],
        ];

        scaler.fit(&features);

        let scaled = scaler.scale(&vec![2.0, 4.0, 6.0]);
        assert_eq!(scaled.len(), 3);
        // The middle value should be close to 0 after scaling
        assert!(scaled[0].abs() < 0.1);
    }

    #[test]
    fn test_ml_models() {
        let mut linear_model = LinearRegressionModel::new();

        let features = vec![0.1, 0.2, 0.3];
        let decision = linear_model.predict(&features);
        assert_eq!(decision.decision, ReplacementAction::KeepExisting);
        assert_eq!(decision.confidence, 0.5);

        let training_data = TrainingDataset {
            features: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            targets: vec![
                MLReplacementDecision {
                    decision: ReplacementAction::ReplaceWithNew,
                    confidence: 0.8,
                    features: vec![1.0, 0.0],
                    algorithm_used: MLAlgorithm::LinearRegression,
                },
                MLReplacementDecision {
                    decision: ReplacementAction::KeepExisting,
                    confidence: 0.6,
                    features: vec![0.0, 1.0],
                    algorithm_used: MLAlgorithm::LinearRegression,
                },
            ],
            weights: vec![1.0, 1.0],
        };

        linear_model.train(&training_data);

        let new_decision = linear_model.predict(&vec![1.0, 0.0]);
        assert!(new_decision.confidence != 0.5); // Should have learned something
    }
}
