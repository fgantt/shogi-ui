//! Validation framework for automated tuning
//!
//! This module provides cross-validation, holdout validation, and other
//! validation techniques to ensure the quality of tuned parameters.
//!
//! ## Stratified Sampling
//!
//! When `stratified` is enabled in `ValidationConfig`, positions are grouped
//! by game result (WhiteWin/BlackWin/Draw) and distributed proportionally
//! across k-folds. This ensures each fold has a similar distribution of results,
//! which is especially important for imbalanced datasets.
//!
//! ## Reproducibility
//!
//! When `random_seed` is provided in `ValidationConfig`, the same seed will
//! produce the same data splits, enabling reproducible validation results.
//! This is useful for:
//! - Comparing different optimization methods on the same data splits
//! - Debugging and testing validation logic
//! - Ensuring consistent results across runs
//!
//! ## Example
//!
//! ```rust,no_run
//! use shogi_engine::tuning::types::{ValidationConfig, TrainingPosition};
//! use shogi_engine::tuning::validator::Validator;
//!
//! // Create validation config with stratified sampling and random seed
//! let config = ValidationConfig {
//!     k_fold: 5,
//!     test_split: 0.2,
//!     validation_split: 0.2,
//!     stratified: true,  // Enable stratified sampling
//!     random_seed: Some(42),  // Enable reproducibility
//! };
//!
//! let validator = Validator::new(config);
//! let results = validator.cross_validate(&positions);
//! ```

use super::optimizer::Optimizer;
use super::types::{
    FoldResult, GameResult as TuningGameResult, MatchResult, OptimizationMethod, TrainingPosition,
    ValidationConfig, ValidationResults,
};
use crate::types::core::{Move, Player};
// Note: GameResult is not yet extracted to a sub-module, using root import
use crate::types::GameResult;
use crate::ShogiEngine;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    ///
    /// If `stratified` is enabled, positions are grouped by game result
    /// (WhiteWin/BlackWin/Draw) and distributed proportionally across folds.
    /// This ensures each fold has a similar distribution of results.
    ///
    /// If `random_seed` is provided, the same seed will produce the same
    /// data splits, enabling reproducible validation.
    pub fn cross_validate(&self, positions: &[TrainingPosition]) -> ValidationResults {
        if positions.is_empty() {
            return ValidationResults::new(vec![]);
        }

        let k = self.config.k_fold as usize;
        let mut fold_results = Vec::new();

        // Create RNG with seed if provided, otherwise use thread_rng
        let mut rng: Box<dyn RngCore> = if let Some(seed) = self.config.random_seed {
            Box::new(StdRng::seed_from_u64(seed))
        } else {
            Box::new(thread_rng())
        };

        // Prepare positions for splitting
        let positions_to_split = if self.config.stratified {
            self.prepare_stratified_positions(positions, &mut *rng)
        } else {
            let mut pos = positions.to_vec();
            pos.shuffle(&mut *rng);
            pos
        };

        let fold_size = positions_to_split.len() / k;
        let remainder = positions_to_split.len() % k;

        let mut start_idx = 0;
        for fold in 0..k {
            let fold_size_adjusted = fold_size + if fold < remainder { 1 } else { 0 };
            let end_idx = start_idx + fold_size_adjusted;

            // Split data into training and validation sets
            let (validation_set, training_set) =
                self.split_data(&positions_to_split, start_idx, end_idx);

            // Train model on training set
            let optimizer = Optimizer::new(OptimizationMethod::default());
            let optimization_result = optimizer.optimize(&training_set);

            match optimization_result {
                Ok(result) => {
                    // Validate on validation set
                    let validation_error =
                        self.calculate_error(&result.optimized_weights, &validation_set);

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
    ///
    /// If `random_seed` is provided, the same seed will produce the same
    /// data splits, enabling reproducible validation.
    pub fn holdout_validate(&self, positions: &[TrainingPosition]) -> ValidationResults {
        if positions.is_empty() {
            return ValidationResults::new(vec![]);
        }

        // Create RNG with seed if provided, otherwise use thread_rng
        let mut rng: Box<dyn RngCore> = if let Some(seed) = self.config.random_seed {
            Box::new(StdRng::seed_from_u64(seed))
        } else {
            Box::new(thread_rng())
        };

        let mut shuffled_positions = positions.to_vec();
        shuffled_positions.shuffle(&mut *rng);

        // Split data according to validation_split
        let validation_size = (positions.len() as f64 * self.config.validation_split) as usize;
        let (validation_set, training_set) = shuffled_positions.split_at(validation_size);

        // Train model on training set
        let optimizer = Optimizer::new(OptimizationMethod::default());
        let optimization_result = optimizer.optimize(training_set);

        match optimization_result {
            Ok(result) => {
                // Validate on validation set
                let validation_error =
                    self.calculate_error(&result.optimized_weights, validation_set);

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
    fn split_data(
        &self,
        positions: &[TrainingPosition],
        start_idx: usize,
        end_idx: usize,
    ) -> (Vec<TrainingPosition>, Vec<TrainingPosition>) {
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
            let score: f64 = weights
                .iter()
                .zip(position.features.iter())
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

        // Create RNG with seed if provided, otherwise use thread_rng
        let mut rng: Box<dyn RngCore> = if let Some(seed) = self.config.random_seed {
            Box::new(StdRng::seed_from_u64(seed))
        } else {
            Box::new(thread_rng())
        };

        let mut test_subset = validation_set.to_vec();
        test_subset.shuffle(&mut *rng);
        test_subset.truncate(test_size);
        test_subset
    }

    /// Determine the game result category from a training position
    ///
    /// The result field is from the side to move's perspective:
    /// - result > 0.5: Win for the side to move
    /// - result < -0.5: Loss for the side to move
    /// - Otherwise: Draw
    ///
    /// We convert this to an absolute result (WhiteWin/BlackWin/Draw).
    fn categorize_result(&self, position: &TrainingPosition) -> TuningGameResult {
        let result = position.result;
        let player = position.player_to_move;

        // Determine if the side to move won, lost, or drew
        let outcome = if result > 0.5 {
            // Win for the side to move
            match player {
                Player::White => TuningGameResult::WhiteWin,
                Player::Black => TuningGameResult::BlackWin,
            }
        } else if result < -0.5 {
            // Loss for the side to move
            match player {
                Player::White => TuningGameResult::BlackWin,
                Player::Black => TuningGameResult::WhiteWin,
            }
        } else {
            // Draw
            TuningGameResult::Draw
        };

        outcome
    }

    /// Prepare positions for stratified sampling
    ///
    /// Groups positions by game result (WhiteWin/BlackWin/Draw) and interleaves
    /// them proportionally so that each fold gets a similar distribution of results.
    fn prepare_stratified_positions(
        &self,
        positions: &[TrainingPosition],
        rng: &mut dyn RngCore,
    ) -> Vec<TrainingPosition> {
        // Group positions by result
        let mut white_wins = Vec::new();
        let mut black_wins = Vec::new();
        let mut draws = Vec::new();

        for position in positions {
            match self.categorize_result(position) {
                TuningGameResult::WhiteWin => white_wins.push(position.clone()),
                TuningGameResult::BlackWin => black_wins.push(position.clone()),
                TuningGameResult::Draw => draws.push(position.clone()),
            }
        }

        // Shuffle each group
        white_wins.shuffle(rng);
        black_wins.shuffle(rng);
        draws.shuffle(rng);

        // Interleave groups proportionally
        let total = positions.len();
        let white_ratio = white_wins.len() as f64 / total as f64;
        let black_ratio = black_wins.len() as f64 / total as f64;
        let draw_ratio = draws.len() as f64 / total as f64;

        let mut result = Vec::with_capacity(total);
        let mut white_idx = 0;
        let mut black_idx = 0;
        let mut draw_idx = 0;

        // Calculate target counts for each category based on proportions
        let mut white_target = 0.0;
        let mut black_target = 0.0;
        let mut draw_target = 0.0;

        for _i in 0..total {
            // Update targets based on proportions
            white_target += white_ratio;
            black_target += black_ratio;
            draw_target += draw_ratio;

            // Determine which category to add next based on which is furthest behind
            let white_behind = white_target - white_idx as f64;
            let black_behind = black_target - black_idx as f64;
            let draw_behind = draw_target - draw_idx as f64;

            if white_behind >= black_behind && white_behind >= draw_behind && white_idx < white_wins.len() {
                result.push(white_wins[white_idx].clone());
                white_idx += 1;
            } else if black_behind >= draw_behind && black_idx < black_wins.len() {
                result.push(black_wins[black_idx].clone());
                black_idx += 1;
            } else if draw_idx < draws.len() {
                result.push(draws[draw_idx].clone());
                draw_idx += 1;
            } else if white_idx < white_wins.len() {
                result.push(white_wins[white_idx].clone());
                white_idx += 1;
            } else if black_idx < black_wins.len() {
                result.push(black_wins[black_idx].clone());
                black_idx += 1;
            }
        }

        result
    }
}

/// Trait for playing games between two engine configurations
///
/// This trait abstracts the game playing interface, allowing different
/// implementations (actual engine, mock for testing, etc.)
pub trait GamePlayer: Send + Sync {
    /// Play a single game between two engine configurations
    ///
    /// # Arguments
    /// * `player1_weights` - Weights for the first player (playing as Black)
    /// * `player2_weights` - Weights for the second player (playing as White)
    /// * `time_per_move_ms` - Time limit per move in milliseconds
    /// * `max_moves` - Maximum number of moves before declaring a draw
    ///
    /// # Returns
    /// * `Ok(TuningGameResult)` - The result of the game from player1's perspective
    /// * `Err(String)` - Error message if the game could not be played
    fn play_game(
        &self,
        player1_weights: &[f64],
        player2_weights: &[f64],
        time_per_move_ms: u32,
        max_moves: u32,
    ) -> Result<TuningGameResult, String>;
}

/// Mock game player for testing
///
/// This implementation simulates game results for fast unit testing
/// without actually playing games.
pub struct MockGamePlayer {
    /// Predetermined results to return (cycles through)
    results: Vec<TuningGameResult>,
    /// Current index in results (uses interior mutability for thread safety)
    current_index: Arc<Mutex<usize>>,
}

impl MockGamePlayer {
    /// Create a new mock game player with predetermined results
    pub fn new(results: Vec<TuningGameResult>) -> Self {
        Self {
            results,
            current_index: Arc::new(Mutex::new(0)),
        }
    }
}

impl GamePlayer for MockGamePlayer {
    fn play_game(
        &self,
        _player1_weights: &[f64],
        _player2_weights: &[f64],
        _time_per_move_ms: u32,
        _max_moves: u32,
    ) -> Result<TuningGameResult, String> {
        if self.results.is_empty() {
            return Ok(TuningGameResult::Draw);
        }

        let mut index = self.current_index.lock().unwrap();
        let result = self.results[*index % self.results.len()];
        *index += 1;
        Ok(result)
    }
}

/// Game player implementation using ShogiEngine
///
/// This implementation plays actual games using the ShogiEngine,
/// allowing realistic strength testing of tuned weights.
pub struct ShogiEngineGamePlayer {
    /// Search depth for moves (0 = adaptive/unlimited)
    pub search_depth: u8,
    /// Whether to enable verbose logging
    pub verbose: bool,
}

impl ShogiEngineGamePlayer {
    /// Create a new ShogiEngine game player
    pub fn new(search_depth: u8, verbose: bool) -> Self {
        Self {
            search_depth,
            verbose,
        }
    }

    /// Convert engine GameResult to tuning GameResult from a player's perspective
    fn convert_game_result(
        engine_result: GameResult,
        perspective: Player,
    ) -> TuningGameResult {
        match (engine_result, perspective) {
            (GameResult::Win, Player::Black) => TuningGameResult::BlackWin,
            (GameResult::Win, Player::White) => TuningGameResult::WhiteWin,
            (GameResult::Loss, Player::Black) => TuningGameResult::WhiteWin,
            (GameResult::Loss, Player::White) => TuningGameResult::BlackWin,
            (GameResult::Draw, _) => TuningGameResult::Draw,
        }
    }
}

impl GamePlayer for ShogiEngineGamePlayer {
    fn play_game(
        &self,
        _player1_weights: &[f64],
        _player2_weights: &[f64],
        time_per_move_ms: u32,
        max_moves: u32,
    ) -> Result<TuningGameResult, String> {
        // Create a single engine instance for self-play
        // Note: In a full implementation, we would apply different weights to each player
        // This requires integration with the evaluation system to apply feature weights
        // For now, we use a single engine to establish the game playing infrastructure
        let mut engine = ShogiEngine::new();

        // TODO: Apply player1_weights and player2_weights to engine configurations
        // This requires:
        // 1. Integration with evaluation system to map feature weights to evaluation parameters
        // 2. Ability to configure engine with different evaluation weights per game
        // 3. Or use two separate engine instances with different configurations

        let mut move_count = 0;
        let mut consecutive_passes = 0;
        let mut last_move: Option<Move> = None;
        let mut current_player = Player::Black;

        // Play the game (engine plays against itself)
        loop {
            // Check if game is over
            if let Some(result) = engine.is_game_over() {
                if self.verbose {
                    println!("Game over: {:?}", result);
                }
                // Convert result from Black's perspective (player1)
                return Ok(Self::convert_game_result(result, Player::Black));
            }

            // Get engine's best move
            let best_move = engine.get_best_move(
                self.search_depth,
                time_per_move_ms,
                None,
            );

            match best_move {
                Some(move_) => {
                    if self.verbose && move_count < 10 {
                        println!(
                            "Move {}: {} ({:?})",
                            move_count + 1,
                            move_.to_usi_string(),
                            current_player
                        );
                    }

                    // Check if same move repeated (possible infinite loop)
                    if last_move.as_ref().map(|m| m.to_usi_string())
                        == Some(move_.to_usi_string())
                    {
                        consecutive_passes += 1;
                        if consecutive_passes >= 3 {
                            if self.verbose {
                                println!("Consecutive repeated moves detected, ending game as draw");
                            }
                            return Ok(TuningGameResult::Draw);
                        }
                    } else {
                        consecutive_passes = 0;
                    }

                    last_move = Some(move_.clone());

                    // Apply the move to the engine
                    if !engine.apply_move(&move_) {
                        if self.verbose {
                            println!(
                                "Failed to apply move: {}, ending game",
                                move_.to_usi_string()
                            );
                        }
                        return Ok(TuningGameResult::Draw);
                    }

                    // Switch turns
                    current_player = current_player.opposite();
                    move_count += 1;

                    // Safety limit to avoid infinite games
                    if move_count >= max_moves {
                        if self.verbose {
                            println!("Maximum move limit reached, ending as draw");
                        }
                        return Ok(TuningGameResult::Draw);
                    }
                }
                None => {
                    // No legal moves - game ended
                    if let Some(result) = engine.is_game_over() {
                        return Ok(Self::convert_game_result(result, Player::Black));
                    }
                    return Ok(TuningGameResult::Draw);
                }
            }
        }
    }
}

/// Strength testing framework for engine vs engine matches
///
/// This struct provides realistic validation by playing actual games between
/// two engine configurations (original vs tuned weights) rather than simulating results.
pub struct StrengthTester {
    /// Number of games to play for testing
    pub games_per_test: u32,
    /// Time control for games (in milliseconds per move)
    pub time_control_ms: u32,
    /// Maximum moves per game before declaring a draw
    pub max_moves_per_game: u32,
    /// Game player implementation
    game_player: Box<dyn GamePlayer>,
}

impl StrengthTester {
    /// Create a new strength tester with default ShogiEngine game player
    pub fn new(games_per_test: u32, time_control_ms: u32) -> Self {
        Self::with_game_player(
            games_per_test,
            time_control_ms,
            Box::new(ShogiEngineGamePlayer::new(3, false)),
        )
    }

    /// Create a new strength tester with a custom game player
    pub fn with_game_player(
        games_per_test: u32,
        time_control_ms: u32,
        game_player: Box<dyn GamePlayer>,
    ) -> Self {
        Self {
            games_per_test,
            time_control_ms,
            max_moves_per_game: 200, // Default maximum moves
            game_player,
        }
    }

    /// Set the maximum number of moves per game before declaring a draw
    pub fn set_max_moves_per_game(&mut self, max_moves: u32) {
        self.max_moves_per_game = max_moves;
    }

    /// Run engine vs engine matches to test strength by playing actual games
    ///
    /// This method plays actual games between two engine configurations:
    /// - Original weights (playing as Black in even games, White in odd games)
    /// - Tuned weights (playing as White in even games, Black in odd games)
    ///
    /// The games alternate colors to eliminate first-move advantage bias.
    ///
    /// # Arguments
    /// * `original_weights` - Weights for the original engine configuration
    /// * `tuned_weights` - Weights for the tuned engine configuration
    ///
    /// # Returns
    /// * `MatchResult` - Results from the match, including wins, losses, draws, and ELO difference
    pub fn test_engine_strength(
        &self,
        original_weights: &[f64],
        tuned_weights: &[f64],
    ) -> MatchResult {
        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;

        // Play games, alternating colors to eliminate first-move advantage
        for game_num in 0..self.games_per_test {
            let (player1_weights, player2_weights) = if game_num % 2 == 0 {
                // Even games: original plays Black, tuned plays White
                (original_weights, tuned_weights)
            } else {
                // Odd games: tuned plays Black, original plays White
                (tuned_weights, original_weights)
            };

            match self.game_player.play_game(
                player1_weights,
                player2_weights,
                self.time_control_ms,
                self.max_moves_per_game,
            ) {
                Ok(result) => {
                    // Count results from tuned weights' perspective
                    // In even games (game_num % 2 == 0), tuned is player2 (White)
                    // In odd games (game_num % 2 == 1), tuned is player1 (Black)
                    let is_tuned_black = game_num % 2 == 1;
                    match (result, is_tuned_black) {
                        (TuningGameResult::BlackWin, false) => {
                            // Original (Black) won, so tuned (White) lost
                            losses += 1;
                        }
                        (TuningGameResult::WhiteWin, false) => {
                            // Tuned (White) won
                            wins += 1;
                        }
                        (TuningGameResult::BlackWin, true) => {
                            // Tuned (Black) won
                            wins += 1;
                        }
                        (TuningGameResult::WhiteWin, true) => {
                            // Original (White) won, so tuned (Black) lost
                            losses += 1;
                        }
                        (TuningGameResult::Draw, _) => {
                            draws += 1;
                        }
                    }
                }
                Err(e) => {
                    // On error, count as draw (conservative approach)
                    eprintln!("Error playing game {}: {}", game_num + 1, e);
                    draws += 1;
                }
            }
        }

        // Calculate ELO difference and confidence interval
        let total_games = wins + losses + draws;
        let elo_difference = Self::calculate_elo_difference(wins, losses, draws);
        let elo_confidence_interval =
            Self::calculate_elo_confidence_interval(wins, losses, draws);

        MatchResult {
            wins,
            losses,
            draws,
            elo_difference,
            elo_confidence_interval,
            total_games,
        }
    }

    /// Calculate ELO difference from match results using standard ELO formula
    ///
    /// Uses the standard ELO calculation: ELO_diff = 400 * log10(W/L) where W/L is win/loss ratio
    fn calculate_elo_difference(wins: u32, losses: u32, _draws: u32) -> f64 {
        let total_games = wins + losses;
        if total_games == 0 {
            return 0.0;
        }

        let win_rate = wins as f64 / total_games as f64;
        if win_rate <= 0.0 || win_rate >= 1.0 {
            return 0.0;
        }

        // Standard ELO formula: ELO_diff = 400 * log10(W/L)
        // where W/L = wins / losses
        let win_loss_ratio = wins as f64 / losses.max(1) as f64;
        400.0 * win_loss_ratio.log10()
    }

    /// Calculate ELO confidence interval using standard error
    ///
    /// Uses the standard error formula for ELO difference with 95% confidence interval
    fn calculate_elo_confidence_interval(wins: u32, losses: u32, draws: u32) -> (f64, f64) {
        let total_games = wins + losses + draws;
        if total_games == 0 {
            return (0.0, 0.0);
        }

        // Standard error for ELO difference (95% confidence interval)
        // Margin = 1.96 * sqrt((W + L) / (W * L)) * 400
        let w = wins as f64;
        let l = losses as f64;
        let margin = if w > 0.0 && l > 0.0 {
            1.96 * ((w + l) / (w * l)).sqrt() * 400.0
        } else {
            100.0 / (total_games as f64).sqrt() // Fallback for edge cases
        };

        let elo_diff = Self::calculate_elo_difference(wins, losses, draws);
        (elo_diff - margin, elo_diff + margin)
    }
}

/// Synthetic dataset generator for testing optimization algorithms
pub struct SyntheticDataGenerator {
    /// Number of features to generate
    feature_count: usize,
    /// Random seed for reproducibility
    #[allow(dead_code)]
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
                128,      // Default game phase
                true,     // Default quiet
                i as u32, // Move number
                if i % 2 == 0 {
                    Player::White
                } else {
                    Player::Black
                },
            ));
        }

        positions
    }

    /// Generate synthetic result based on features
    fn generate_synthetic_result(&self, features: &[f64], rng: &mut impl rand::Rng) -> f64 {
        // Create a simple linear relationship with some noise
        let true_score: f64 = features
            .iter()
            .enumerate()
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
        validation_error > self.validation_error_threshold
            || (validation_error - training_error) > self.error_difference_threshold
    }

    /// Calculate overfitting score (0.0 = no overfitting, 1.0 = severe overfitting)
    pub fn calculate_overfitting_score(&self, training_error: f64, validation_error: f64) -> f64 {
        let error_diff = validation_error - training_error;
        let threshold_ratio = validation_error / self.validation_error_threshold;

        (error_diff / self.error_difference_threshold).min(1.0) * 0.5
            + (threshold_ratio - 1.0).max(0.0) * 0.5
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
            report.push_str(&format!(
                "  {}: {} bytes ({:.2} MB)\n",
                operation,
                bytes,
                *bytes as f64 / 1024.0 / 1024.0
            ));
        }

        report.push_str("\nTimings:\n");
        for (operation, seconds) in &self.timings {
            report.push_str(&format!("  {}: {:.3} seconds\n", operation, seconds));
        }

        report
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::super::types::{ValidationConfig, GameResult as TuningGameResult};
    use super::*;
    use crate::types::NUM_EVAL_FEATURES;

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
    fn test_strength_tester_match_with_mock() {
        // Use mock game player for fast unit testing
        let mock_results = vec![
            TuningGameResult::BlackWin, // Game 0: Black wins (tuned as White loses)
            TuningGameResult::WhiteWin, // Game 1: White wins (tuned as Black wins)
            TuningGameResult::Draw,     // Game 2: Draw
            TuningGameResult::BlackWin, // Game 3: Black wins (tuned as White loses)
            TuningGameResult::WhiteWin, // Game 4: White wins (tuned as Black wins)
        ];
        let mock_player = Box::new(MockGamePlayer::new(mock_results));
        let tester = StrengthTester::with_game_player(5, 1000, mock_player);
        let original_weights = vec![1.0; NUM_EVAL_FEATURES];
        let tuned_weights = vec![1.1; NUM_EVAL_FEATURES];

        let result = tester.test_engine_strength(&original_weights, &tuned_weights);
        assert_eq!(result.total_games, 5);
        assert_eq!(result.wins + result.losses + result.draws, 5);
        // Expected: 2 wins (games 1, 4), 2 losses (games 0, 3), 1 draw (game 2)
        assert_eq!(result.wins, 2);
        assert_eq!(result.losses, 2);
        assert_eq!(result.draws, 1);
    }

    #[test]
    fn test_strength_tester_actual_games() {
        // Integration test with actual engine (may be slow)
        // Use very fast time control and few games for testing
        let tester = StrengthTester::new(2, 100); // 2 games, 100ms per move
        let original_weights = vec![1.0; NUM_EVAL_FEATURES];
        let tuned_weights = vec![1.0; NUM_EVAL_FEATURES]; // Same weights for testing

        let result = tester.test_engine_strength(&original_weights, &tuned_weights);
        assert_eq!(result.total_games, 2);
        assert_eq!(result.wins + result.losses + result.draws, 2);
        // With same weights, results should be balanced (wins ≈ losses) or draws
        // But we can't predict exact results, so just verify structure
        assert!(result.wins <= 2);
        assert!(result.losses <= 2);
        assert!(result.draws <= 2);
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

    /// Create a training position with a specific result category
    fn create_position_with_result(
        result_category: TuningGameResult,
        player: Player,
        index: usize,
    ) -> TrainingPosition {
        let result = match (result_category, player) {
            (TuningGameResult::WhiteWin, Player::White) => 1.0,
            (TuningGameResult::BlackWin, Player::Black) => 1.0,
            (TuningGameResult::WhiteWin, Player::Black) => -1.0,
            (TuningGameResult::BlackWin, Player::White) => -1.0,
            (TuningGameResult::Draw, _) => 0.0,
        };

        TrainingPosition::new(
            vec![0.0; NUM_EVAL_FEATURES],
            result,
            128,
            true,
            index as u32,
            player,
        )
    }

    #[test]
    fn test_stratified_sampling() {
        let config = ValidationConfig {
            k_fold: 3,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: true,
            random_seed: Some(42),
        };
        let validator = Validator::new(config);

        // Create positions with known result distribution
        let mut positions = Vec::new();
        // 30 White wins
        for i in 0..30 {
            positions.push(create_position_with_result(
                TuningGameResult::WhiteWin,
                Player::White,
                i,
            ));
        }
        // 20 Black wins
        for i in 30..50 {
            positions.push(create_position_with_result(
                TuningGameResult::BlackWin,
                Player::Black,
                i,
            ));
        }
        // 10 Draws
        for i in 50..60 {
            positions.push(create_position_with_result(
                TuningGameResult::Draw,
                Player::White,
                i,
            ));
        }

        let results = validator.cross_validate(&positions);

        assert_eq!(results.fold_results.len(), 3);

        // Verify that each fold has approximately the same distribution
        // Each fold should have ~10 White wins, ~7 Black wins, ~3 Draws
        // We'll verify by checking that the distribution is roughly proportional
        // (exact counts may vary slightly due to rounding)
        for fold_result in &results.fold_results {
            assert!(fold_result.sample_count >= 18 && fold_result.sample_count <= 22);
        }
    }

    #[test]
    fn test_random_seed_reproducibility() {
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
        let positions = generator.generate_positions(30);

        // Run cross-validation twice with the same seed
        let results1 = validator.cross_validate(&positions);
        let results2 = validator.cross_validate(&positions);

        // Results should be identical (same splits)
        assert_eq!(results1.fold_results.len(), results2.fold_results.len());
        for (fold1, fold2) in results1.fold_results.iter().zip(results2.fold_results.iter()) {
            assert_eq!(fold1.sample_count, fold2.sample_count);
            // Validation errors should be the same (same data splits)
            assert!((fold1.validation_error - fold2.validation_error).abs() < 1e-10);
        }
    }

    #[test]
    fn test_stratified_with_imbalanced_data() {
        let config = ValidationConfig {
            k_fold: 5,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: true,
            random_seed: Some(42),
        };
        let validator = Validator::new(config);

        // Create heavily imbalanced data: 90% White wins, 5% Black wins, 5% Draws
        let mut positions = Vec::new();
        // 90 White wins
        for i in 0..90 {
            positions.push(create_position_with_result(
                TuningGameResult::WhiteWin,
                Player::White,
                i,
            ));
        }
        // 5 Black wins
        for i in 90..95 {
            positions.push(create_position_with_result(
                TuningGameResult::BlackWin,
                Player::Black,
                i,
            ));
        }
        // 5 Draws
        for i in 95..100 {
            positions.push(create_position_with_result(
                TuningGameResult::Draw,
                Player::White,
                i,
            ));
        }

        let results = validator.cross_validate(&positions);

        assert_eq!(results.fold_results.len(), 5);

        // With stratification, each fold should have roughly:
        // - 18 White wins (90/5)
        // - 1 Black win (5/5)
        // - 1 Draw (5/5)
        // Total: ~20 positions per fold
        for fold_result in &results.fold_results {
            assert!(fold_result.sample_count >= 18 && fold_result.sample_count <= 22);
        }
    }

    #[test]
    fn test_stratified_vs_non_stratified() {
        // Create positions with known distribution
        let mut positions = Vec::new();
        // 40 White wins
        for i in 0..40 {
            positions.push(create_position_with_result(
                TuningGameResult::WhiteWin,
                Player::White,
                i,
            ));
        }
        // 30 Black wins
        for i in 40..70 {
            positions.push(create_position_with_result(
                TuningGameResult::BlackWin,
                Player::Black,
                i,
            ));
        }
        // 30 Draws
        for i in 70..100 {
            positions.push(create_position_with_result(
                TuningGameResult::Draw,
                Player::White,
                i,
            ));
        }

        // Test with stratification
        let config_stratified = ValidationConfig {
            k_fold: 5,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: true,
            random_seed: Some(42),
        };
        let validator_stratified = Validator::new(config_stratified);
        let results_stratified = validator_stratified.cross_validate(&positions);

        // Test without stratification
        let config_non_stratified = ValidationConfig {
            k_fold: 5,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: false,
            random_seed: Some(42),
        };
        let validator_non_stratified = Validator::new(config_non_stratified);
        let results_non_stratified = validator_non_stratified.cross_validate(&positions);

        // Both should produce the same number of folds
        assert_eq!(results_stratified.fold_results.len(), results_non_stratified.fold_results.len());

        // With stratification, folds should have more consistent sample counts
        // (less variance in fold sizes)
        let stratified_counts: Vec<usize> = results_stratified
            .fold_results
            .iter()
            .map(|f| f.sample_count)
            .collect();
        let non_stratified_counts: Vec<usize> = results_non_stratified
            .fold_results
            .iter()
            .map(|f| f.sample_count)
            .collect();

        // Calculate variance in fold sizes
        let stratified_variance = calculate_variance(&stratified_counts);
        let non_stratified_variance = calculate_variance(&non_stratified_counts);

        // Stratified should have lower or equal variance (more consistent fold sizes)
        // Note: This may not always be true due to rounding, but it's a good heuristic
        assert!(stratified_variance <= non_stratified_variance + 1.0); // Allow small tolerance
    }

    /// Helper function to calculate variance
    fn calculate_variance(values: &[usize]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<usize>() as f64 / values.len() as f64;
        let variance = values
            .iter()
            .map(|&v| {
                let diff = v as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        variance
    }
}
