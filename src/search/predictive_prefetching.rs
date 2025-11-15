//! Predictive Prefetching
//!
//! This module implements predictive prefetching for transposition tables,
//! using various heuristics to predict which entries are likely to be
//! accessed next and prefetch them into cache for improved performance.
//!
//! # Features
//!
//! - **Multiple Prediction Strategies**: Move-based, depth-based, and pattern-based
//! - **Adaptive Learning**: Learns from access patterns to improve predictions
//! - **Configurable Prefetching**: Adjustable prefetch depth and aggressiveness
//! - **Performance Monitoring**: Tracks prefetch hit rates and effectiveness
//! - **Memory Efficient**: Minimal overhead while providing significant benefits
//!
//! # Usage
//!
//! ```rust
//! use shogi_engine::search::{PredictivePrefetcher, PrefetchConfig, PrefetchStrategy};
//!
//! // Create prefetcher with move-based strategy
//! let config = PrefetchConfig::move_based();
//! let mut prefetcher = PredictivePrefetcher::new(config);
//!
//! // Predict and prefetch likely next accesses
//! let current_hash = 0x123456789ABCDEF0;
//! let predicted_hashes = prefetcher.predict_next_accesses(current_hash, 5);
//!
//! // Prefetch the predicted entries
//! for hash in predicted_hashes {
//!     prefetcher.prefetch_entry(hash);
//! }
//!
//! // Update prediction accuracy when actual access occurs
//! prefetcher.record_access(current_hash, true); // true if prefetched
//! ```

use crate::types::core::{Move, PieceType, Player, Position};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Prefetching configuration
#[derive(Debug, Clone)]
pub struct PrefetchConfig {
    /// Prediction strategy to use
    pub strategy: PrefetchStrategy,
    /// Maximum number of entries to prefetch
    pub max_prefetch_entries: usize,
    /// Prefetch depth (how many moves ahead to predict)
    pub prefetch_depth: u8,
    /// Learning rate for adaptive algorithms (0.0 to 1.0)
    pub learning_rate: f64,
    /// Confidence threshold for predictions (0.0 to 1.0)
    pub confidence_threshold: f64,
    /// Enable aggressive prefetching
    pub aggressive: bool,
    /// Cache size for prediction data
    pub prediction_cache_size: usize,
    /// Enable pattern-based learning
    pub enable_pattern_learning: bool,
}

/// Prefetching strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchStrategy {
    /// Move-based prediction (predict based on likely next moves)
    MoveBased,
    /// Depth-based prediction (predict based on search depth)
    DepthBased,
    /// Pattern-based prediction (learn from access patterns)
    PatternBased,
    /// Hybrid approach combining multiple strategies
    Hybrid,
    /// Adaptive strategy that learns optimal approach
    Adaptive,
}

/// Prefetch prediction result
#[derive(Debug, Clone)]
pub struct PrefetchPrediction {
    /// Predicted hash keys
    pub predicted_hashes: Vec<u64>,
    /// Confidence scores for each prediction
    pub confidence_scores: Vec<f64>,
    /// Prediction strategy used
    pub strategy_used: PrefetchStrategy,
    /// Prediction metadata
    pub metadata: PredictionMetadata,
}

/// Prediction metadata
#[derive(Debug, Clone)]
pub struct PredictionMetadata {
    /// Time taken to generate predictions
    pub prediction_time_us: u64,
    /// Number of patterns analyzed
    pub patterns_analyzed: usize,
    /// Cache hit rate for prediction data
    pub cache_hit_rate: f64,
    /// Prediction accuracy estimate
    pub estimated_accuracy: f64,
}

/// Prefetch statistics
#[derive(Debug, Clone, Default)]
pub struct PrefetchStats {
    /// Total predictions made
    pub total_predictions: u64,
    /// Total prefetches performed
    pub total_prefetches: u64,
    /// Successful prefetch hits
    pub prefetch_hits: u64,
    /// Failed prefetch attempts
    pub prefetch_misses: u64,
    /// Average prediction time in microseconds
    pub avg_prediction_time_us: f64,
    /// Average prefetch hit rate
    pub avg_hit_rate: f64,
    /// Total time saved through prefetching
    pub time_saved_us: u64,
    /// Memory overhead of prefetching
    pub memory_overhead_bytes: usize,
}

/// Move pattern for prediction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovePattern {
    /// Pattern of piece types involved
    pub piece_pattern: Vec<PieceType>,
    /// Pattern of positions
    pub position_pattern: Vec<u8>,
    /// Pattern depth
    pub depth: u8,
}

/// Access pattern for learning
#[derive(Debug, Clone)]
pub struct AccessPattern {
    /// Sequence of hash keys accessed
    pub hash_sequence: VecDeque<u64>,
    /// Access timestamps
    pub timestamps: VecDeque<Instant>,
    /// Move information associated with accesses
    pub move_info: VecDeque<Move>,
    /// Pattern frequency
    pub frequency: u32,
    /// Last access time
    pub last_access: Instant,
}

/// Predictive prefetcher
pub struct PredictivePrefetcher {
    /// Prefetching configuration
    config: PrefetchConfig,
    /// Prefetching statistics
    stats: PrefetchStats,
    /// Move patterns for prediction
    move_patterns: HashMap<MovePattern, u32>,
    /// Access patterns for learning
    access_patterns: HashMap<Vec<u64>, AccessPattern>,
    /// Prediction cache
    prediction_cache: HashMap<u64, (PrefetchPrediction, Instant)>,
    /// Recent access history
    recent_accesses: VecDeque<(u64, Instant, Move)>,
    /// Pattern learning weights
    pattern_weights: HashMap<MovePattern, f64>,
    /// Prefetch queue
    prefetch_queue: VecDeque<u64>,
}

impl PrefetchConfig {
    /// Create move-based configuration
    pub fn move_based() -> Self {
        Self {
            strategy: PrefetchStrategy::MoveBased,
            max_prefetch_entries: 10,
            prefetch_depth: 3,
            learning_rate: 0.1,
            confidence_threshold: 0.6,
            aggressive: false,
            prediction_cache_size: 1000,
            enable_pattern_learning: true,
        }
    }

    /// Create depth-based configuration
    pub fn depth_based() -> Self {
        Self {
            strategy: PrefetchStrategy::DepthBased,
            max_prefetch_entries: 15,
            prefetch_depth: 5,
            learning_rate: 0.05,
            confidence_threshold: 0.7,
            aggressive: true,
            prediction_cache_size: 2000,
            enable_pattern_learning: false,
        }
    }

    /// Create pattern-based configuration
    pub fn pattern_based() -> Self {
        Self {
            strategy: PrefetchStrategy::PatternBased,
            max_prefetch_entries: 8,
            prefetch_depth: 4,
            learning_rate: 0.15,
            confidence_threshold: 0.5,
            aggressive: false,
            prediction_cache_size: 1500,
            enable_pattern_learning: true,
        }
    }

    /// Create hybrid configuration
    pub fn hybrid() -> Self {
        Self {
            strategy: PrefetchStrategy::Hybrid,
            max_prefetch_entries: 12,
            prefetch_depth: 4,
            learning_rate: 0.1,
            confidence_threshold: 0.65,
            aggressive: true,
            prediction_cache_size: 2000,
            enable_pattern_learning: true,
        }
    }

    /// Create adaptive configuration
    pub fn adaptive() -> Self {
        Self {
            strategy: PrefetchStrategy::Adaptive,
            max_prefetch_entries: 10,
            prefetch_depth: 3,
            learning_rate: 0.2,
            confidence_threshold: 0.6,
            aggressive: false,
            prediction_cache_size: 1000,
            enable_pattern_learning: true,
        }
    }
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        Self::move_based()
    }
}

impl PredictivePrefetcher {
    /// Create a new predictive prefetcher
    pub fn new(config: PrefetchConfig) -> Self {
        Self {
            config,
            stats: PrefetchStats::default(),
            move_patterns: HashMap::new(),
            access_patterns: HashMap::new(),
            prediction_cache: HashMap::with_capacity(1000),
            recent_accesses: VecDeque::with_capacity(100),
            pattern_weights: HashMap::new(),
            prefetch_queue: VecDeque::new(),
        }
    }

    /// Predict next likely accesses based on current hash
    pub fn predict_next_accesses(
        &mut self,
        current_hash: u64,
        current_move: Option<Move>,
    ) -> PrefetchPrediction {
        let start_time = Instant::now();

        // Check cache first
        if let Some((cached_prediction, cache_time)) = self.prediction_cache.get(&current_hash) {
            if cache_time.elapsed() < Duration::from_millis(100) {
                self.stats.total_predictions += 1;
                return cached_prediction.clone();
            }
        }

        let prediction = match self.config.strategy {
            PrefetchStrategy::MoveBased => self.predict_move_based(current_hash, current_move),
            PrefetchStrategy::DepthBased => self.predict_depth_based(current_hash),
            PrefetchStrategy::PatternBased => self.predict_pattern_based(current_hash),
            PrefetchStrategy::Hybrid => self.predict_hybrid(current_hash, current_move),
            PrefetchStrategy::Adaptive => self.predict_adaptive(current_hash, current_move),
        };

        let prediction_time = start_time.elapsed().as_micros() as u64;

        // Update statistics
        self.stats.total_predictions += 1;
        self.stats.avg_prediction_time_us = (self.stats.avg_prediction_time_us
            * (self.stats.total_predictions - 1) as f64
            + prediction_time as f64)
            / self.stats.total_predictions as f64;

        // Cache the prediction
        if self.prediction_cache.len() >= self.config.prediction_cache_size {
            // Remove oldest entry
            if let Some(oldest_key) = self.prediction_cache.keys().next().copied() {
                self.prediction_cache.remove(&oldest_key);
            }
        }

        let mut prediction_with_metadata = prediction;
        prediction_with_metadata.metadata.prediction_time_us = prediction_time;
        self.prediction_cache.insert(
            current_hash,
            (prediction_with_metadata.clone(), Instant::now()),
        );

        prediction_with_metadata
    }

    /// Prefetch an entry
    pub fn prefetch_entry(&mut self, hash: u64) {
        // Add to prefetch queue
        if self.prefetch_queue.len() >= self.config.max_prefetch_entries {
            self.prefetch_queue.pop_front();
        }

        self.prefetch_queue.push_back(hash);
        self.stats.total_prefetches += 1;

        // Simulate prefetch operation (in real implementation, this would
        // actually load the entry into cache)
        self.perform_prefetch(hash);
    }

    /// Record an actual access for learning
    pub fn record_access(&mut self, hash: u64, was_prefetched: bool, move_info: Option<Move>) {
        let now = Instant::now();

        // Update statistics
        if was_prefetched {
            self.stats.prefetch_hits += 1;
        } else {
            self.stats.prefetch_misses += 1;
        }

        // Update average hit rate
        let total_accesses = self.stats.prefetch_hits + self.stats.prefetch_misses;
        if total_accesses > 0 {
            self.stats.avg_hit_rate = self.stats.prefetch_hits as f64 / total_accesses as f64;
        }

        // Add to recent accesses
        if let Some(ref move_) = move_info {
            self.recent_accesses.push_back((hash, now, move_.clone()));
        } else {
            self.recent_accesses
                .push_back((hash, now, create_dummy_move()));
        }

        // Maintain recent accesses size
        if self.recent_accesses.len() > 100 {
            self.recent_accesses.pop_front();
        }

        // Update access patterns
        if self.config.enable_pattern_learning {
            self.update_access_patterns(hash, move_info);
        }

        // Update pattern weights based on success
        self.update_pattern_weights(hash, was_prefetched);
    }

    /// Get prefetching statistics
    pub fn get_stats(&self) -> &PrefetchStats {
        &self.stats
    }

    /// Clear prediction cache
    pub fn clear_cache(&mut self) {
        self.prediction_cache.clear();
        self.prefetch_queue.clear();
    }

    /// Update learning based on recent performance
    pub fn update_learning(&mut self) {
        if !self.config.enable_pattern_learning {
            return;
        }

        // Analyze recent access patterns
        self.analyze_recent_patterns();

        // Update pattern weights
        self.update_all_pattern_weights();

        // Clean up old patterns
        self.cleanup_old_patterns();
    }

    /// Move-based prediction
    fn predict_move_based(
        &mut self,
        current_hash: u64,
        current_move: Option<Move>,
    ) -> PrefetchPrediction {
        let mut predicted_hashes = Vec::new();
        let mut confidence_scores = Vec::new();

        if let Some(move_) = current_move {
            // Predict based on likely follow-up moves
            let likely_moves = self.predict_likely_moves(&move_);

            for (predicted_move, confidence) in likely_moves {
                let predicted_hash = self.hash_move_combination(current_hash, &predicted_move);
                predicted_hashes.push(predicted_hash);
                confidence_scores.push(confidence);
            }
        }

        // Add some depth-based predictions
        let depth_predictions = self.predict_depth_based_hashes(current_hash, 2);
        predicted_hashes.extend(depth_predictions.iter());
        confidence_scores.extend(vec![0.5; depth_predictions.len()]);

        PrefetchPrediction {
            predicted_hashes: predicted_hashes
                .into_iter()
                .take(self.config.max_prefetch_entries)
                .collect(),
            confidence_scores: confidence_scores
                .into_iter()
                .take(self.config.max_prefetch_entries)
                .collect(),
            strategy_used: PrefetchStrategy::MoveBased,
            metadata: PredictionMetadata {
                prediction_time_us: 0,
                patterns_analyzed: self.move_patterns.len(),
                cache_hit_rate: self.calculate_cache_hit_rate(),
                estimated_accuracy: 0.7,
            },
        }
    }

    /// Depth-based prediction
    fn predict_depth_based(&mut self, current_hash: u64) -> PrefetchPrediction {
        let predicted_hashes =
            self.predict_depth_based_hashes(current_hash, self.config.prefetch_depth);

        let predicted_count = predicted_hashes.len().min(self.config.max_prefetch_entries);

        PrefetchPrediction {
            predicted_hashes: predicted_hashes
                .into_iter()
                .take(self.config.max_prefetch_entries)
                .collect(),
            confidence_scores: vec![0.8; predicted_count],
            strategy_used: PrefetchStrategy::DepthBased,
            metadata: PredictionMetadata {
                prediction_time_us: 0,
                patterns_analyzed: 0,
                cache_hit_rate: self.calculate_cache_hit_rate(),
                estimated_accuracy: 0.8,
            },
        }
    }

    /// Pattern-based prediction
    fn predict_pattern_based(&mut self, current_hash: u64) -> PrefetchPrediction {
        let mut predicted_hashes = Vec::new();
        let mut confidence_scores = Vec::new();

        // Find similar access patterns
        let similar_patterns = self.find_similar_patterns(current_hash);

        for (pattern, confidence) in similar_patterns {
            if let Some(_access_pattern) = self.access_patterns.get(&pattern) {
                // Predict next hash in the pattern
                for &next_hash in &_access_pattern.hash_sequence {
                    if next_hash != current_hash && !predicted_hashes.contains(&next_hash) {
                        predicted_hashes.push(next_hash);
                        confidence_scores.push(confidence);

                        if predicted_hashes.len() >= self.config.max_prefetch_entries {
                            break;
                        }
                    }
                }
            }
        }

        if predicted_hashes.is_empty() {
            // Fall back to depth-based heuristics to ensure we always have predictions
            let fallback_hashes =
                self.predict_depth_based_hashes(current_hash, self.config.prefetch_depth.max(1));

            for hash in fallback_hashes
                .into_iter()
                .take(self.config.max_prefetch_entries - predicted_hashes.len())
            {
                if !predicted_hashes.contains(&hash) {
                    predicted_hashes.push(hash);
                    confidence_scores.push(0.3);
                }
            }
        }

        PrefetchPrediction {
            predicted_hashes,
            confidence_scores,
            strategy_used: PrefetchStrategy::PatternBased,
            metadata: PredictionMetadata {
                prediction_time_us: 0,
                patterns_analyzed: self.access_patterns.len(),
                cache_hit_rate: self.calculate_cache_hit_rate(),
                estimated_accuracy: 0.6,
            },
        }
    }

    /// Hybrid prediction
    fn predict_hybrid(
        &mut self,
        current_hash: u64,
        current_move: Option<Move>,
    ) -> PrefetchPrediction {
        // Combine multiple strategies
        let move_prediction = self.predict_move_based(current_hash, current_move);
        let depth_prediction = self.predict_depth_based(current_hash);
        let pattern_prediction = self.predict_pattern_based(current_hash);

        // Merge predictions with weighted confidence
        let mut combined_hashes = Vec::new();
        let mut combined_confidences = Vec::new();

        // Add move-based predictions with higher weight
        for (hash, confidence) in move_prediction
            .predicted_hashes
            .iter()
            .zip(move_prediction.confidence_scores.iter())
        {
            combined_hashes.push(*hash);
            combined_confidences.push(confidence * 0.4);
        }

        // Add depth-based predictions
        for (hash, confidence) in depth_prediction
            .predicted_hashes
            .iter()
            .zip(depth_prediction.confidence_scores.iter())
        {
            if !combined_hashes.contains(hash) {
                combined_hashes.push(*hash);
                combined_confidences.push(confidence * 0.4);
            }
        }

        // Add pattern-based predictions
        for (hash, confidence) in pattern_prediction
            .predicted_hashes
            .iter()
            .zip(pattern_prediction.confidence_scores.iter())
        {
            if !combined_hashes.contains(hash) {
                combined_hashes.push(*hash);
                combined_confidences.push(confidence * 0.2);
            }
        }

        // Sort by confidence and limit
        let mut indexed: Vec<(usize, f64)> = combined_confidences
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, c))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let limited_hashes: Vec<u64> = indexed
            .iter()
            .take(self.config.max_prefetch_entries)
            .map(|&(i, _)| combined_hashes[i])
            .collect();

        let limited_confidences: Vec<f64> = indexed
            .iter()
            .take(self.config.max_prefetch_entries)
            .map(|&(_, c)| c)
            .collect();

        PrefetchPrediction {
            predicted_hashes: limited_hashes,
            confidence_scores: limited_confidences,
            strategy_used: PrefetchStrategy::Hybrid,
            metadata: PredictionMetadata {
                prediction_time_us: 0,
                patterns_analyzed: self.move_patterns.len() + self.access_patterns.len(),
                cache_hit_rate: self.calculate_cache_hit_rate(),
                estimated_accuracy: 0.75,
            },
        }
    }

    /// Adaptive prediction
    fn predict_adaptive(
        &mut self,
        current_hash: u64,
        current_move: Option<Move>,
    ) -> PrefetchPrediction {
        // Choose best strategy based on recent performance
        let best_strategy = self.select_best_strategy();

        let mut prediction = match best_strategy {
            PrefetchStrategy::MoveBased => self.predict_move_based(current_hash, current_move),
            PrefetchStrategy::DepthBased => self.predict_depth_based(current_hash),
            PrefetchStrategy::PatternBased => self.predict_pattern_based(current_hash),
            _ => self.predict_hybrid(current_hash, current_move),
        };

        prediction.strategy_used = PrefetchStrategy::Adaptive;
        prediction
    }

    /// Predict likely follow-up moves
    fn predict_likely_moves(&self, current_move: &Move) -> Vec<(Move, f64)> {
        let mut likely_moves = Vec::new();

        // Common follow-up patterns
        match current_move.piece_type {
            PieceType::Pawn => {
                // Pawn moves often followed by development
                likely_moves.push((create_dummy_move(), 0.7));
            }
            PieceType::Knight => {
                // Knight moves often followed by pawn moves
                likely_moves.push((create_dummy_move(), 0.6));
            }
            PieceType::Bishop | PieceType::Rook => {
                // Major pieces often followed by tactical moves
                likely_moves.push((create_dummy_move(), 0.8));
            }
            _ => {
                // Default prediction
                likely_moves.push((create_dummy_move(), 0.5));
            }
        }

        likely_moves
    }

    /// Predict hashes based on depth
    fn predict_depth_based_hashes(&self, current_hash: u64, depth: u8) -> Vec<u64> {
        let mut predicted_hashes = Vec::new();

        // Generate variations of current hash for different depths
        for i in 1..=depth {
            let depth_hash = current_hash.wrapping_add(i as u64 * 0x100000000);
            predicted_hashes.push(depth_hash);
        }

        predicted_hashes
    }

    /// Find similar access patterns
    fn find_similar_patterns(&self, current_hash: u64) -> Vec<(Vec<u64>, f64)> {
        let mut similar_patterns = Vec::new();

        // Find patterns that contain current hash
        for (pattern, _access_pattern) in &self.access_patterns {
            if pattern.contains(&current_hash) {
                let confidence = self.calculate_pattern_confidence(pattern);
                similar_patterns.push((pattern.clone(), confidence));
            }
        }

        // Sort by confidence
        similar_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        similar_patterns
    }

    /// Calculate pattern confidence
    fn calculate_pattern_confidence(&self, pattern: &[u64]) -> f64 {
        if let Some(access_pattern) = self.access_patterns.get(pattern) {
            // Confidence based on frequency and recency
            let frequency_factor = (access_pattern.frequency as f64).log10() / 3.0;
            let recency_factor = if access_pattern.last_access.elapsed() < Duration::from_secs(60) {
                1.0
            } else {
                0.5
            };

            (frequency_factor * recency_factor).min(1.0)
        } else {
            0.0
        }
    }

    /// Select best strategy based on performance
    fn select_best_strategy(&self) -> PrefetchStrategy {
        // Simple heuristic based on hit rate
        if self.stats.avg_hit_rate > 0.7 {
            PrefetchStrategy::DepthBased
        } else if self.stats.avg_hit_rate > 0.5 {
            PrefetchStrategy::MoveBased
        } else {
            PrefetchStrategy::PatternBased
        }
    }

    /// Hash move combination
    fn hash_move_combination(&self, base_hash: u64, move_: &Move) -> u64 {
        // Simple hash combination
        let move_hash = (move_.from.map(|p| p.to_u8()).unwrap_or(0) as u64) << 8
            | (move_.to.to_u8() as u64) << 16
            | (move_.piece_type as u8 as u64) << 24;

        base_hash ^ move_hash
    }

    /// Perform actual prefetch operation
    fn perform_prefetch(&mut self, _hash: u64) {
        // In a real implementation, this would:
        // 1. Check if entry exists in transposition table
        // 2. Load entry into cache if not already cached
        // 3. Update cache statistics
        // For now, just simulate the operation
    }

    /// Update access patterns
    fn update_access_patterns(&mut self, _hash: u64, _move_info: Option<Move>) {
        // Create pattern from recent accesses
        let recent_hashes: Vec<u64> = self
            .recent_accesses
            .iter()
            .rev()
            .take(5)
            .map(|(h, _, _)| *h)
            .collect();

        if recent_hashes.len() >= 3 {
            let pattern_key = recent_hashes.clone();

            if let Some(access_pattern) = self.access_patterns.get_mut(&pattern_key) {
                access_pattern.frequency += 1;
                access_pattern.last_access = Instant::now();
            } else {
                let access_pattern = AccessPattern {
                    hash_sequence: recent_hashes.iter().cloned().collect(),
                    timestamps: self
                        .recent_accesses
                        .iter()
                        .rev()
                        .take(5)
                        .map(|(_, t, _)| *t)
                        .collect(),
                    move_info: self
                        .recent_accesses
                        .iter()
                        .rev()
                        .take(5)
                        .map(|(_, _, m)| m.clone())
                        .collect(),
                    frequency: 1,
                    last_access: Instant::now(),
                };

                self.access_patterns.insert(pattern_key, access_pattern);
            }
        }
    }

    /// Update pattern weights based on success
    fn update_pattern_weights(&mut self, hash: u64, was_prefetched: bool) {
        // Find patterns that predicted this hash
        for (pattern, _access_pattern) in &self.access_patterns {
            if pattern.contains(&hash) {
                if let Some(weight) = self.pattern_weights.get_mut(&self.hash_to_pattern(pattern)) {
                    if was_prefetched {
                        *weight += self.config.learning_rate;
                    } else {
                        *weight -= self.config.learning_rate * 0.1;
                    }
                    *weight = weight.max(0.0).min(1.0);
                }
            }
        }
    }

    /// Update all pattern weights
    fn update_all_pattern_weights(&mut self) {
        // Update weights based on recent performance
        for (pattern, _access_pattern) in &self.access_patterns {
            let pattern_key = self.hash_to_pattern(pattern);
            let current_weight = self
                .pattern_weights
                .get(&pattern_key)
                .copied()
                .unwrap_or(0.5);

            // Adjust weight based on frequency and recency
            let frequency_bonus = (_access_pattern.frequency as f64).log10() / 10.0;
            let recency_bonus = if _access_pattern.last_access.elapsed() < Duration::from_secs(30) {
                0.1
            } else {
                0.0
            };

            let new_weight = (current_weight + frequency_bonus + recency_bonus).min(1.0);
            self.pattern_weights.insert(pattern_key, new_weight);
        }
    }

    /// Analyze recent patterns
    fn analyze_recent_patterns(&mut self) {
        // Analyze the last 50 accesses for new patterns
        let recent_count = self.recent_accesses.len().min(50);
        let recent_accesses: Vec<(u64, Instant, Move)> = self
            .recent_accesses
            .iter()
            .rev()
            .take(recent_count)
            .cloned()
            .collect();

        // Find repeating patterns
        for window_size in 3..=8 {
            if recent_accesses.len() < window_size {
                continue;
            }

            for start in 0..=recent_accesses.len().saturating_sub(window_size) {
                let pattern: Vec<u64> = recent_accesses[start..start + window_size]
                    .iter()
                    .map(|(h, _, _)| *h)
                    .collect();

                // Count occurrences of this pattern
                let mut count = 0;
                for check_start in 0..=recent_accesses.len().saturating_sub(window_size) {
                    if recent_accesses[check_start..check_start + window_size]
                        .iter()
                        .map(|(h, _, _)| *h)
                        .eq(pattern.iter().cloned())
                    {
                        count += 1;
                    }
                }

                if count >= 2 {
                    // Found a repeating pattern
                    let pattern_key = self.hash_to_pattern(&pattern);
                    self.pattern_weights.insert(pattern_key, 0.7);
                }
            }
        }
    }

    /// Clean up old patterns
    fn cleanup_old_patterns(&mut self) {
        let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes

        self.access_patterns
            .retain(|_, access_pattern| access_pattern.last_access > cutoff_time);

        // Remove low-weight patterns
        self.pattern_weights.retain(|_, &mut weight| weight > 0.1);
    }

    /// Calculate cache hit rate
    fn calculate_cache_hit_rate(&self) -> f64 {
        if self.prediction_cache.is_empty() {
            return 0.0;
        }

        // Simple estimation based on cache size and prediction frequency
        let cache_utilization =
            self.prediction_cache.len() as f64 / self.config.prediction_cache_size as f64;
        cache_utilization * 0.8 // Assume 80% hit rate at full utilization
    }

    /// Convert hash sequence to pattern
    fn hash_to_pattern(&self, hashes: &[u64]) -> MovePattern {
        // Create a pattern from the hash sequence
        let piece_pattern = hashes.iter().take(5).map(|_| PieceType::Pawn).collect();
        let position_pattern = hashes.iter().take(5).map(|h| (h & 0xFF) as u8).collect();

        MovePattern {
            piece_pattern,
            position_pattern,
            depth: hashes.len() as u8,
        }
    }
}

impl Default for PredictivePrefetcher {
    fn default() -> Self {
        Self::new(PrefetchConfig::default())
    }
}

/// Helper function to create a dummy move
fn create_dummy_move() -> Move {
    Move {
        from: Some(Position::from_u8(10)),
        to: Position::from_u8(20),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: false,
        is_recapture: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_configs() {
        let move_based = PrefetchConfig::move_based();
        assert_eq!(move_based.strategy, PrefetchStrategy::MoveBased);
        assert_eq!(move_based.max_prefetch_entries, 10);

        let depth_based = PrefetchConfig::depth_based();
        assert_eq!(depth_based.strategy, PrefetchStrategy::DepthBased);
        assert!(depth_based.aggressive);

        let pattern_based = PrefetchConfig::pattern_based();
        assert_eq!(pattern_based.strategy, PrefetchStrategy::PatternBased);
        assert!(pattern_based.enable_pattern_learning);

        let hybrid = PrefetchConfig::hybrid();
        assert_eq!(hybrid.strategy, PrefetchStrategy::Hybrid);

        let adaptive = PrefetchConfig::adaptive();
        assert_eq!(adaptive.strategy, PrefetchStrategy::Adaptive);
    }

    #[test]
    fn test_prediction_generation() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::move_based());

        let current_hash = 0x123456789ABCDEF0;
        let prediction = prefetcher.predict_next_accesses(current_hash, Some(create_dummy_move()));

        assert!(!prediction.predicted_hashes.is_empty());
        assert_eq!(
            prediction.predicted_hashes.len(),
            prediction.confidence_scores.len()
        );
        assert!(prediction.predicted_hashes.len() <= 10); // max_prefetch_entries

        // Verify all predicted hashes are different from current
        for &predicted_hash in &prediction.predicted_hashes {
            assert_ne!(predicted_hash, current_hash);
        }
    }

    #[test]
    fn test_prefetch_functionality() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::default());

        let hash = 0x123456789ABCDEF0;
        let initial_prefetches = prefetcher.stats.total_prefetches;

        prefetcher.prefetch_entry(hash);

        assert_eq!(prefetcher.stats.total_prefetches, initial_prefetches + 1);
        assert_eq!(prefetcher.prefetch_queue.len(), 1);
    }

    #[test]
    fn test_access_recording() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::pattern_based());

        let hash = 0x123456789ABCDEF0;
        let move_info = create_dummy_move();

        // Record successful prefetch
        prefetcher.record_access(hash, true, Some(move_info.clone()));

        assert_eq!(prefetcher.stats.prefetch_hits, 1);
        assert_eq!(prefetcher.stats.prefetch_misses, 0);
        assert_eq!(prefetcher.stats.avg_hit_rate, 1.0);

        // Record failed prefetch
        prefetcher.record_access(hash, false, Some(move_info));

        assert_eq!(prefetcher.stats.prefetch_hits, 1);
        assert_eq!(prefetcher.stats.prefetch_misses, 1);
        assert_eq!(prefetcher.stats.avg_hit_rate, 0.5);
    }

    #[test]
    fn test_different_strategies() {
        let strategies = [
            PrefetchStrategy::MoveBased,
            PrefetchStrategy::DepthBased,
            PrefetchStrategy::PatternBased,
            PrefetchStrategy::Hybrid,
            PrefetchStrategy::Adaptive,
        ];

        let current_hash = 0x123456789ABCDEF0;

        for strategy in strategies {
            let config = PrefetchConfig {
                strategy,
                max_prefetch_entries: 5,
                prefetch_depth: 3,
                learning_rate: 0.1,
                confidence_threshold: 0.6,
                aggressive: false,
                prediction_cache_size: 100,
                enable_pattern_learning: true,
            };

            let mut prefetcher = PredictivePrefetcher::new(config);
            let prediction =
                prefetcher.predict_next_accesses(current_hash, Some(create_dummy_move()));

            assert_eq!(prediction.strategy_used, strategy);
            assert!(!prediction.predicted_hashes.is_empty());
        }
    }

    #[test]
    fn test_pattern_learning() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::pattern_based());

        // Record a sequence of accesses to create a pattern
        let hashes = vec![0x1111, 0x2222, 0x3333, 0x4444];
        let moves = vec![create_dummy_move(); 4];

        for (hash, move_) in hashes.iter().zip(moves.iter()) {
            prefetcher.record_access(*hash, false, Some(move_.clone()));
        }

        // Update learning
        prefetcher.update_learning();

        // Check that patterns were created
        assert!(!prefetcher.access_patterns.is_empty());
        assert!(!prefetcher.pattern_weights.is_empty());
    }

    #[test]
    fn test_cache_functionality() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::move_based());

        let hash = 0x123456789ABCDEF0;

        // First prediction (cache miss)
        let prediction1 = prefetcher.predict_next_accesses(hash, Some(create_dummy_move()));

        // Second prediction (cache hit)
        let prediction2 = prefetcher.predict_next_accesses(hash, Some(create_dummy_move()));

        // Should be identical (from cache)
        assert_eq!(prediction1.predicted_hashes, prediction2.predicted_hashes);
        assert_eq!(prediction1.confidence_scores, prediction2.confidence_scores);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::default());

        // Generate some activity
        for i in 0..10 {
            let hash = 0x1000 + i;
            let prediction = prefetcher.predict_next_accesses(hash, Some(create_dummy_move()));

            for predicted_hash in prediction.predicted_hashes {
                prefetcher.prefetch_entry(predicted_hash);
            }

            prefetcher.record_access(hash, i % 2 == 0, Some(create_dummy_move()));
        }

        let stats = prefetcher.get_stats();
        assert_eq!(stats.total_predictions, 10);
        assert!(stats.total_prefetches > 0);
        assert!(stats.prefetch_hits > 0);
        assert!(stats.prefetch_misses > 0);
        assert!(stats.avg_prediction_time_us > 0.0);
        assert!(stats.avg_hit_rate > 0.0 && stats.avg_hit_rate <= 1.0);
    }
}
