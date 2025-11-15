//! Core data structures for the automated tuning system
//!
//! This module defines all the essential types and structures used throughout
//! the tuning process, from game records to optimization configuration.

use crate::types::NUM_EVAL_FEATURES;
use crate::types::{Move, Player};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// GAME AND POSITION DATA STRUCTURES
// ============================================================================

/// Result of a completed game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    /// White player won
    WhiteWin,
    /// Black player won  
    BlackWin,
    /// Game ended in a draw
    Draw,
}

impl GameResult {
    /// Convert game result to a score from White's perspective
    /// Returns 1.0 for WhiteWin, 0.0 for Draw, -1.0 for BlackWin
    pub fn to_score(&self) -> f64 {
        match self {
            GameResult::WhiteWin => 1.0,
            GameResult::Draw => 0.0,
            GameResult::BlackWin => -1.0,
        }
    }

    /// Convert game result to a score from the specified player's perspective
    pub fn to_score_for_player(&self, player: Player) -> f64 {
        let base_score = self.to_score();
        match player {
            Player::White => base_score,
            Player::Black => -base_score,
        }
    }
}

/// Time control information for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeControl {
    /// Initial time in seconds
    pub initial_time: u32,
    /// Time increment per move in seconds
    pub increment: u32,
    /// Whether this is a blitz game (< 10 minutes)
    pub is_blitz: bool,
    /// Whether this is a bullet game (< 3 minutes)
    pub is_bullet: bool,
}

impl TimeControl {
    /// Create a new time control
    pub fn new(initial_time: u32, increment: u32) -> Self {
        let is_blitz = initial_time < 600; // Less than 10 minutes
        let is_bullet = initial_time < 180; // Less than 3 minutes

        Self {
            initial_time,
            increment,
            is_blitz,
            is_bullet,
        }
    }
}

/// Complete record of a game for tuning purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    /// List of moves played in the game
    pub moves: Vec<Move>,
    /// Final result of the game
    pub result: GameResult,
    /// White player's rating (if available)
    pub white_rating: Option<u16>,
    /// Black player's rating (if available)
    pub black_rating: Option<u16>,
    /// Time control used in the game
    pub time_control: TimeControl,
    /// Opening name or ECO code (if available)
    pub opening: Option<String>,
    /// Date the game was played (if available)
    pub date: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl GameRecord {
    /// Create a new game record
    pub fn new(moves: Vec<Move>, result: GameResult, time_control: TimeControl) -> Self {
        Self {
            moves,
            result,
            white_rating: None,
            black_rating: None,
            time_control,
            opening: None,
            date: None,
            metadata: HashMap::new(),
        }
    }

    /// Get the average rating of both players
    pub fn average_rating(&self) -> Option<u16> {
        match (self.white_rating, self.black_rating) {
            (Some(w), Some(b)) => Some((w + b) / 2),
            _ => None,
        }
    }

    /// Check if this is a high-rated game (both players > 2000)
    pub fn is_high_rated(&self) -> bool {
        self.average_rating().map_or(false, |rating| rating > 2000)
    }

    /// Get the number of moves in the game
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }
}

/// A position extracted from a game for training purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingPosition {
    /// Feature vector extracted from the position
    pub features: Vec<f64>,
    /// Game result from the side to move's perspective
    pub result: f64,
    /// Game phase (0 = endgame, GAME_PHASE_MAX = opening)
    pub game_phase: i32,
    /// Whether this position was reached after a quiet move
    pub is_quiet: bool,
    /// Move number in the game (1-indexed)
    pub move_number: u32,
    /// Player to move
    pub player_to_move: Player,
    /// FEN string of the position (if available)
    pub fen: Option<String>,
}

impl TrainingPosition {
    /// Create a new training position
    pub fn new(
        features: Vec<f64>,
        result: f64,
        game_phase: i32,
        is_quiet: bool,
        move_number: u32,
        player_to_move: Player,
    ) -> Self {
        assert_eq!(
            features.len(),
            NUM_EVAL_FEATURES,
            "Feature vector must have {} elements",
            NUM_EVAL_FEATURES
        );
        assert!(
            result >= -1.0 && result <= 1.0,
            "Result must be between -1.0 and 1.0"
        );

        Self {
            features,
            result,
            game_phase,
            is_quiet,
            move_number,
            player_to_move,
            fen: None,
        }
    }

    /// Check if this is an opening position (first 20 moves)
    pub fn is_opening(&self) -> bool {
        self.move_number <= 20
    }

    /// Check if this is an endgame position (after move 40)
    pub fn is_endgame(&self) -> bool {
        self.move_number > 40
    }

    /// Check if this is a middlegame position
    pub fn is_middlegame(&self) -> bool {
        !self.is_opening() && !self.is_endgame()
    }
}

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Filtering criteria for selecting training positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionFilter {
    /// Minimum number of quiet moves required before a position
    pub quiet_move_threshold: u32,
    /// Minimum rating for games to include
    pub min_rating: Option<u16>,
    /// Maximum rating for games to include  
    pub max_rating: Option<u16>,
    /// Minimum move number to include positions from
    pub min_move_number: u32,
    /// Maximum move number to include positions from
    pub max_move_number: u32,
    /// Maximum number of positions per game
    pub max_positions_per_game: Option<usize>,
    /// Whether to include only quiet positions
    pub quiet_only: bool,
    /// Whether to include only high-rated games
    pub high_rated_only: bool,
}

impl Default for PositionFilter {
    fn default() -> Self {
        Self {
            quiet_move_threshold: 3,
            min_rating: Some(1800),
            max_rating: None,
            min_move_number: 10,
            max_move_number: 80,
            max_positions_per_game: Some(5),
            quiet_only: true,
            high_rated_only: false,
        }
    }
}

/// Validation configuration for cross-validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Number of folds for k-fold cross-validation
    pub k_fold: u32,
    /// Percentage of data to use for testing (0.0 to 1.0)
    pub test_split: f64,
    /// Percentage of data to use for validation (0.0 to 1.0)
    pub validation_split: f64,
    /// Whether to use stratified sampling
    pub stratified: bool,
    /// Random seed for reproducible splits
    pub random_seed: Option<u64>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            k_fold: 5,
            test_split: 0.2,
            validation_split: 0.2,
            stratified: true,
            random_seed: Some(42),
        }
    }
}

/// Performance and resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum memory usage in MB
    pub memory_limit_mb: usize,
    /// Number of threads to use for parallel processing
    pub thread_count: usize,
    /// Frequency of checkpoint saves (in iterations)
    pub checkpoint_frequency: usize,
    /// Whether to enable progress logging
    pub enable_logging: bool,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Maximum number of iterations (None for unlimited)
    pub max_iterations: Option<usize>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 8192, // 8 GB
            thread_count: num_cpus::get(),
            checkpoint_frequency: 1000,
            enable_logging: true,
            max_batch_size: 10000,
            max_iterations: None,
        }
    }
}

/// Type of line search algorithm for LBFGS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineSearchType {
    /// Armijo condition line search (sufficient decrease)
    /// Ensures: f(x + αp) ≤ f(x) + c1 * α * ∇f(x)^T * p
    Armijo,
    /// Wolfe conditions (sufficient decrease + curvature condition)
    /// Not yet implemented
    #[allow(dead_code)]
    Wolfe,
}

impl Default for LineSearchType {
    fn default() -> Self {
        LineSearchType::Armijo
    }
}

/// Optimization algorithm to use
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OptimizationMethod {
    /// Gradient descent with fixed learning rate
    GradientDescent { learning_rate: f64 },
    /// Adam optimizer with adaptive learning rates
    ///
    /// All parameters (`beta1`, `beta2`, `epsilon`) are honored from the configuration
    /// and passed to the optimizer implementation. Default values are:
    /// - `beta1`: 0.9 (exponential decay rate for first moment estimates)
    /// - `beta2`: 0.999 (exponential decay rate for second moment estimates)
    /// - `epsilon`: 1e-8 (small constant for numerical stability)
    ///
    /// These parameters can be tuned to adapt the optimizer behavior to different
    /// datasets and optimization landscapes.
    Adam {
        learning_rate: f64,
        beta1: f64,
        beta2: f64,
        epsilon: f64,
    },
    /// Limited-memory BFGS quasi-Newton method with line search
    ///
    /// Line search ensures sufficient decrease in the objective function,
    /// preventing instability from fixed learning rates.
    ///
    /// Configuration parameters:
    /// - `line_search_type`: Type of line search (currently supports Armijo)
    /// - `initial_step_size`: Initial step size for line search (typically 1.0)
    /// - `max_line_search_iterations`: Maximum backtracking iterations (typically 20)
    /// - `armijo_constant`: Armijo condition constant c1 (typically 0.0001)
    /// - `step_size_reduction`: Step size reduction factor for backtracking (typically 0.5)
    LBFGS {
        memory_size: usize,
        max_iterations: usize,
        line_search_type: LineSearchType,
        initial_step_size: f64,
        max_line_search_iterations: usize,
        armijo_constant: f64,
        step_size_reduction: f64,
    },
    /// Genetic algorithm with population-based search
    GeneticAlgorithm {
        population_size: usize,
        mutation_rate: f64,
        crossover_rate: f64,
        max_generations: usize,
        /// Tournament size for tournament selection (default: 3)
        tournament_size: usize,
        /// Percentage of population to preserve as elite (0.0 to 1.0, default: 0.1 = 10%)
        elite_percentage: f64,
        /// Magnitude of mutation changes (default: 0.2)
        mutation_magnitude: f64,
        /// Bounds for mutation values (min, max) (default: (-10.0, 10.0))
        mutation_bounds: (f64, f64),
    },
}

impl Default for OptimizationMethod {
    fn default() -> Self {
        OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        }
    }
}

/// Main configuration for the tuning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Path to the dataset file or directory
    pub dataset_path: String,
    /// Path to save the tuned weights
    pub output_path: String,
    /// Path to save intermediate results and checkpoints
    pub checkpoint_path: Option<String>,
    /// Optimization method to use
    pub optimization_method: OptimizationMethod,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence threshold for early stopping
    pub convergence_threshold: f64,
    /// Position filtering criteria
    pub position_filter: PositionFilter,
    /// Validation configuration
    pub validation_config: ValidationConfig,
    /// Performance configuration
    pub performance_config: PerformanceConfig,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            dataset_path: "dataset.pgn".to_string(),
            output_path: "tuned_weights.json".to_string(),
            checkpoint_path: Some("checkpoints/".to_string()),
            optimization_method: OptimizationMethod::default(),
            max_iterations: 10000,
            convergence_threshold: 1e-6,
            position_filter: PositionFilter::default(),
            validation_config: ValidationConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

// ============================================================================
// RESULTS AND VALIDATION STRUCTURES
// ============================================================================

/// Results from a single validation fold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldResult {
    /// Fold number (1-indexed)
    pub fold_number: u32,
    /// Mean squared error on validation set
    pub validation_error: f64,
    /// Mean squared error on test set
    pub test_error: f64,
    /// Number of samples in this fold
    pub sample_count: usize,
}

/// Comprehensive validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Mean validation error across all folds
    pub mean_error: f64,
    /// Standard deviation of validation errors
    pub std_error: f64,
    /// Results for each individual fold
    pub fold_results: Vec<FoldResult>,
    /// Best performing fold
    pub best_fold: Option<u32>,
    /// Worst performing fold
    pub worst_fold: Option<u32>,
}

impl ValidationResults {
    /// Create new validation results
    pub fn new(fold_results: Vec<FoldResult>) -> Self {
        let errors: Vec<f64> = fold_results.iter().map(|f| f.validation_error).collect();
        let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
        let variance = errors
            .iter()
            .map(|&x| (x - mean_error).powi(2))
            .sum::<f64>()
            / errors.len() as f64;
        let std_error = variance.sqrt();

        let best_fold = fold_results
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1.validation_error
                    .partial_cmp(&b.1.validation_error)
                    .unwrap()
            })
            .map(|(i, _)| (i + 1) as u32);

        let worst_fold = fold_results
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.validation_error
                    .partial_cmp(&b.1.validation_error)
                    .unwrap()
            })
            .map(|(i, _)| (i + 1) as u32);

        Self {
            mean_error,
            std_error,
            fold_results,
            best_fold,
            worst_fold,
        }
    }
}

/// Results from engine strength testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Number of games won by the tuned engine
    pub wins: u32,
    /// Number of games lost by the tuned engine
    pub losses: u32,
    /// Number of games drawn
    pub draws: u32,
    /// ELO difference (positive means tuned engine is stronger)
    pub elo_difference: f64,
    /// Confidence interval for ELO difference
    pub elo_confidence_interval: (f64, f64),
    /// Total number of games played
    pub total_games: u32,
}

impl MatchResult {
    /// Create new match result
    pub fn new(wins: u32, losses: u32, draws: u32) -> Self {
        let total_games = wins + losses + draws;
        let elo_difference = Self::calculate_elo_difference(wins, losses, draws);
        let elo_confidence_interval =
            Self::calculate_confidence_interval(total_games, elo_difference);

        Self {
            wins,
            losses,
            draws,
            elo_difference,
            elo_confidence_interval,
            total_games,
        }
    }

    /// Calculate ELO difference using the standard formula
    fn calculate_elo_difference(wins: u32, losses: u32, _draws: u32) -> f64 {
        if wins + losses == 0 {
            return 0.0;
        }

        let win_rate = wins as f64 / (wins + losses) as f64;
        if win_rate <= 0.0 || win_rate >= 1.0 {
            return 0.0;
        }

        -400.0 * (1.0 / win_rate - 1.0).log10()
    }

    /// Calculate 95% confidence interval for ELO difference
    fn calculate_confidence_interval(total_games: u32, elo_diff: f64) -> (f64, f64) {
        if total_games < 30 {
            // Not enough games for reliable confidence interval
            return (elo_diff - 200.0, elo_diff + 200.0);
        }

        let margin = 1.96 * (400.0 / (total_games as f64).sqrt());
        (elo_diff - margin, elo_diff + margin)
    }

    /// Get the win rate of the tuned engine
    pub fn win_rate(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.wins as f64 / self.total_games as f64
    }

    /// Check if the tuned engine is significantly stronger
    pub fn is_significantly_stronger(&self) -> bool {
        self.elo_difference > self.elo_confidence_interval.0.max(10.0)
    }
}

/// Complete tuning results including validation and match results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResults {
    /// Final tuned weights
    pub weights: Vec<f64>,
    /// Validation results from cross-validation
    pub validation_results: ValidationResults,
    /// Match results from engine strength testing
    pub match_results: Option<MatchResult>,
    /// Configuration used for tuning
    pub config: TuningConfig,
    /// Total training time in seconds
    pub training_time_seconds: f64,
    /// Number of iterations completed
    pub iterations_completed: usize,
    /// Final loss value
    pub final_loss: f64,
    /// Whether tuning converged
    pub converged: bool,
}

impl TuningResults {
    /// Create new tuning results
    pub fn new(
        weights: Vec<f64>,
        validation_results: ValidationResults,
        config: TuningConfig,
        training_time_seconds: f64,
        iterations_completed: usize,
        final_loss: f64,
        converged: bool,
    ) -> Self {
        assert_eq!(
            weights.len(),
            NUM_EVAL_FEATURES,
            "Weights must have {} elements",
            NUM_EVAL_FEATURES
        );

        Self {
            weights,
            validation_results,
            match_results: None,
            config,
            training_time_seconds,
            iterations_completed,
            final_loss,
            converged,
        }
    }

    /// Add match results to the tuning results
    pub fn add_match_results(&mut self, match_results: MatchResult) {
        self.match_results = Some(match_results);
    }

    /// Get the improvement in validation error
    pub fn validation_improvement(&self) -> Option<f64> {
        // This would need to be compared with baseline results
        // For now, just return the mean error
        Some(self.validation_results.mean_error)
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculate the sigmoid function for probability conversion
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Calculate the derivative of the sigmoid function
pub fn sigmoid_derivative(x: f64) -> f64 {
    let s = sigmoid(x);
    s * (1.0 - s)
}

/// Calculate the mean squared error between predicted and actual values
pub fn mean_squared_error(predictions: &[f64], actual: &[f64]) -> f64 {
    assert_eq!(predictions.len(), actual.len());

    predictions
        .iter()
        .zip(actual.iter())
        .map(|(p, a)| (p - a).powi(2))
        .sum::<f64>()
        / predictions.len() as f64
}

/// Calculate the root mean squared error
pub fn root_mean_squared_error(predictions: &[f64], actual: &[f64]) -> f64 {
    mean_squared_error(predictions, actual).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Player;

    #[test]
    fn test_game_result_conversion() {
        assert_eq!(GameResult::WhiteWin.to_score(), 1.0);
        assert_eq!(GameResult::Draw.to_score(), 0.0);
        assert_eq!(GameResult::BlackWin.to_score(), -1.0);

        assert_eq!(GameResult::WhiteWin.to_score_for_player(Player::White), 1.0);
        assert_eq!(
            GameResult::WhiteWin.to_score_for_player(Player::Black),
            -1.0
        );
    }

    #[test]
    fn test_time_control_creation() {
        let tc = TimeControl::new(300, 5);
        assert_eq!(tc.initial_time, 300);
        assert_eq!(tc.increment, 5);
        assert!(tc.is_blitz);
        assert!(!tc.is_bullet);

        let tc2 = TimeControl::new(120, 0);
        assert!(tc2.is_bullet);
    }

    #[test]
    fn test_game_record_creation() {
        let moves = vec![];
        let result = GameResult::Draw;
        let time_control = TimeControl::new(600, 10);

        let record = GameRecord::new(moves, result, time_control);
        assert_eq!(record.result, GameResult::Draw);
        assert_eq!(record.move_count(), 0);
        assert_eq!(record.average_rating(), None);
        assert!(!record.is_high_rated());
    }

    #[test]
    fn test_training_position_creation() {
        let features = vec![0.0; NUM_EVAL_FEATURES];
        let result = 0.5;
        let game_phase = 100;
        let is_quiet = true;
        let move_number = 25; // Changed to 25 to be in middlegame range (20 < 25 <= 40)
        let player = Player::White;

        let position =
            TrainingPosition::new(features, result, game_phase, is_quiet, move_number, player);
        assert_eq!(position.result, 0.5);
        assert!(!position.is_opening()); // 25 > 20, so not opening
        assert!(!position.is_endgame()); // 25 <= 40, so not endgame
        assert!(position.is_middlegame()); // 20 < 25 <= 40, so is middlegame
    }

    #[test]
    fn test_match_result_calculation() {
        let result = MatchResult::new(10, 5, 5);
        assert_eq!(result.total_games, 20);
        assert_eq!(result.win_rate(), 0.5);
        // With 10 wins and 5 losses, win rate is 66.7%, which gives a significant ELO difference
        assert!(result.elo_difference > 0.0); // Should be positive (tuned engine is stronger)

        // Test with equal wins/losses for ~0 ELO difference
        let equal_result = MatchResult::new(10, 10, 0);
        assert_eq!(equal_result.win_rate(), 0.5);
        assert!(equal_result.elo_difference.abs() < 100.0); // Should be close to 0 for 50% win rate
    }

    #[test]
    fn test_validation_results_creation() {
        let fold_results = vec![
            FoldResult {
                fold_number: 1,
                validation_error: 0.1,
                test_error: 0.12,
                sample_count: 100,
            },
            FoldResult {
                fold_number: 2,
                validation_error: 0.15,
                test_error: 0.14,
                sample_count: 100,
            },
            FoldResult {
                fold_number: 3,
                validation_error: 0.08,
                test_error: 0.09,
                sample_count: 100,
            },
        ];

        let results = ValidationResults::new(fold_results);
        assert_eq!(results.mean_error, 0.11);
        assert!(results.std_error > 0.0);
        assert_eq!(results.best_fold, Some(3));
        assert_eq!(results.worst_fold, Some(2));
    }

    #[test]
    fn test_utility_functions() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!((sigmoid(100.0) - 1.0).abs() < 1e-10);
        assert!((sigmoid(-100.0) - 0.0).abs() < 1e-10);

        let predictions = vec![1.0, 2.0, 3.0];
        let actual = vec![1.1, 1.9, 3.1];
        let mse = mean_squared_error(&predictions, &actual);
        assert!(mse > 0.0);
        assert!(mse < 1.0);
    }
}
