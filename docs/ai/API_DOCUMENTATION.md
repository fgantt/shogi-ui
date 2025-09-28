# API Documentation

Comprehensive API reference for the automated tuning system components.

## Table of Contents

1. [Core Types](#core-types)
2. [Feature Extraction](#feature-extraction)
3. [Data Processing](#data-processing)
4. [Optimization](#optimization)
5. [Validation](#validation)
6. [Performance Monitoring](#performance-monitoring)
7. [Weight Management](#weight-management)
8. [CLI Interface](#cli-interface)

## Core Types

### GameRecord

Represents a single game with metadata and moves.

```rust
pub struct GameRecord {
    pub id: String,
    pub white_player: String,
    pub black_player: String,
    pub white_rating: Option<u16>,
    pub black_rating: Option<u16>,
    pub result: GameResult,
    pub time_control: Option<u32>,
    pub date: Option<DateTime<Utc>>,
    pub moves: Vec<String>,
    pub positions: Vec<TrainingPosition>,
}
```

**Methods:**
- `new()` - Create new game record
- `add_position(position: TrainingPosition)` - Add position to game
- `validate()` - Validate game data
- `to_json()` - Serialize to JSON
- `from_json(json: &str)` - Deserialize from JSON

### TrainingPosition

Represents a single training position with features and result.

```rust
pub struct TrainingPosition {
    pub features: Vec<f64>,
    pub result: GameResult,
    pub game_phase: GamePhase,
    pub move_number: u32,
    pub is_quiet: bool,
    pub fen: String,
}
```

**Methods:**
- `new(features: Vec<f64>, result: GameResult)` - Create new position
- `is_opening()` - Check if position is in opening
- `is_middlegame()` - Check if position is in middlegame
- `is_endgame()` - Check if position is in endgame
- `validate()` - Validate position data

### GameResult

Enum representing game outcomes.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
}
```

**Methods:**
- `to_score()` - Convert to numerical score (1.0, 0.0, 0.5)
- `from_score(score: f64)` - Create from numerical score

### TuningConfig

Configuration for the tuning process.

```rust
pub struct TuningConfig {
    pub dataset_path: PathBuf,
    pub output_path: PathBuf,
    pub optimization_method: OptimizationMethod,
    pub iterations: usize,
    pub learning_rate: f64,
    pub k_fold: usize,
    pub test_split: f64,
    pub regularization: f64,
    pub min_rating: Option<u16>,
    pub quiet_threshold: u32,
    pub performance_config: PerformanceConfig,
    pub validation_config: ValidationConfig,
}
```

**Methods:**
- `new()` - Create with default values
- `with_method(method: OptimizationMethod)` - Set optimization method
- `with_iterations(iterations: usize)` - Set iteration count
- `validate()` - Validate configuration
- `to_json()` - Serialize to JSON
- `from_json(json: &str)` - Deserialize from JSON

## Feature Extraction

### FeatureExtractor

Extracts evaluation features from board positions.

```rust
pub struct FeatureExtractor {
    evaluator: PositionEvaluator,
    king_safety_evaluator: KingSafetyEvaluator,
}
```

**Methods:**
- `new()` - Create new feature extractor
- `with_king_safety_config(config: KingSafetyConfig)` - Configure king safety
- `extract_features(board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<f64>` - Extract all features
- `extract_material_features(board: &BitboardBoard, captured_pieces: &CapturedPieces) -> Vec<f64>` - Extract material features
- `extract_positional_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract positional features
- `extract_king_safety_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract king safety features
- `extract_pawn_structure_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract pawn structure features
- `extract_mobility_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract mobility features
- `extract_coordination_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract coordination features
- `extract_center_control_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract center control features
- `extract_development_features(board: &BitboardBoard, player: Player) -> Vec<f64>` - Extract development features
- `normalize_features(features: &mut [f64])` - Normalize feature values
- `validate_features(features: &[f64]) -> bool` - Validate feature values
- `create_training_position(board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces, result: GameResult) -> TrainingPosition` - Create training position

**Constants:**
- `NUM_EVAL_FEATURES: usize = 2000` - Total number of features
- `NUM_MG_FEATURES: usize = 1000` - Middlegame features
- `NUM_EG_FEATURES: usize = 1000` - Endgame features

### Feature Indices

Predefined indices for different feature types:

```rust
// Material features (0-15)
pub const MATERIAL_PAWN_INDEX: usize = 0;
pub const MATERIAL_LANCE_INDEX: usize = 1;
pub const MATERIAL_KNIGHT_INDEX: usize = 2;
pub const MATERIAL_SILVER_INDEX: usize = 3;
pub const MATERIAL_GOLD_INDEX: usize = 4;
pub const MATERIAL_BISHOP_INDEX: usize = 5;
pub const MATERIAL_ROOK_INDEX: usize = 6;

// Positional features (16-815)
pub const POSITIONAL_PAWN_INDEX: usize = 16;
pub const POSITIONAL_LANCE_INDEX: usize = 176;
pub const POSITIONAL_KNIGHT_INDEX: usize = 336;
pub const POSITIONAL_SILVER_INDEX: usize = 496;
pub const POSITIONAL_GOLD_INDEX: usize = 656;

// King safety features (816-815)
pub const KING_SAFETY_CASTLE_INDEX: usize = 816;
pub const KING_SAFETY_ATTACK_INDEX: usize = 817;
pub const KING_SAFETY_THREAT_INDEX: usize = 818;

// Other feature indices...
```

## Data Processing

### DataProcessor

Processes game databases and extracts training positions.

```rust
pub struct DataProcessor {
    position_filter: PositionFilter,
    progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
}
```

**Methods:**
- `new(filter: PositionFilter)` - Create new processor
- `with_progress_callback(callback: Box<dyn Fn(f64) + Send + Sync>)` - Set progress callback
- `process_game(game: &GameRecord) -> Vec<TrainingPosition>` - Process single game
- `load_dataset<P: AsRef<Path>>(path: P) -> Result<Vec<TrainingPosition>, Error>` - Load dataset
- `save_training_data<P: AsRef<Path>>(positions: &[TrainingPosition], path: P)` - Save training data
- `load_training_data<P: AsRef<Path>>(path: P) -> Result<Vec<TrainingPosition>, Error>` - Load training data

### GameDatabase

Manages collections of games.

```rust
pub struct GameDatabase {
    games: Vec<GameRecord>,
    metadata: HashMap<String, String>,
}
```

**Methods:**
- `new()` - Create new database
- `add_games(games: Vec<GameRecord>)` - Add games to database
- `get_games() -> &Vec<GameRecord>` - Get all games
- `game_count() -> usize` - Get game count
- `total_positions() -> usize` - Get total position count
- `recalculate_stats()` - Recalculate statistics

### PositionSelector

Filters and selects training positions.

```rust
pub struct PositionSelector {
    filter: PositionFilter,
}
```

**Methods:**
- `new(filter: PositionFilter)` - Create new selector
- `select_positions(positions: &[TrainingPosition]) -> Vec<TrainingPosition>` - Select positions
- `passes_rating_filter(position: &TrainingPosition) -> bool` - Check rating filter
- `passes_move_number_filter(position: &TrainingPosition) -> bool` - Check move number filter
- `is_duplicate_position(position: &TrainingPosition, seen: &HashSet<String>) -> bool` - Check for duplicates

## Optimization

### Optimizer

Main optimization interface.

```rust
pub struct Optimizer {
    method: OptimizationMethod,
    config: TuningConfig,
}
```

**Methods:**
- `new(method: OptimizationMethod)` - Create new optimizer
- `with_config(config: TuningConfig)` - Set configuration
- `optimize(positions: &[TrainingPosition]) -> OptimizationResults` - Run optimization
- `validate(positions: &[TrainingPosition], weights: &[f64]) -> ValidationResults` - Validate weights

### TexelTuner

Implementation of Texel's tuning method.

```rust
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
```

**Methods:**
- `new(positions: Vec<TrainingPosition>)` - Create new tuner
- `with_params(k_factor: f64, learning_rate: f64, momentum: f64)` - Set parameters
- `optimize() -> OptimizationResults` - Run optimization
- `calculate_error_and_gradients() -> (f64, Vec<f64>)` - Calculate error and gradients
- `calculate_position_score(position: &TrainingPosition) -> f64` - Calculate position score
- `sigmoid(x: f64) -> f64` - Sigmoid function
- `sigmoid_derivative(x: f64) -> f64` - Sigmoid derivative
- `apply_regularization(gradients: &mut [f64])` - Apply regularization
- `get_weights() -> &Vec<f64>` - Get current weights
- `set_weights(weights: Vec<f64>)` - Set weights

### OptimizationMethod

Enum for different optimization algorithms.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OptimizationMethod {
    GradientDescent { learning_rate: f64 },
    Adam { learning_rate: f64, beta1: f64, beta2: f64, epsilon: f64 },
    LBFGS { memory: usize, tolerance: f64 },
    GeneticAlgorithm { population_size: usize, mutation_rate: f64, crossover_rate: f64 },
}
```

### OptimizationResults

Results from optimization process.

```rust
pub struct OptimizationResults {
    pub final_weights: Vec<f64>,
    pub final_error: f64,
    pub iterations_completed: usize,
    pub convergence_reason: ConvergenceReason,
    pub error_history: Vec<f64>,
    pub validation_results: Option<ValidationResults>,
    pub performance_metrics: PerformanceMetrics,
}
```

### ConvergenceReason

Reason for optimization termination.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConvergenceReason {
    MaxIterationsReached,
    ConvergenceThresholdMet,
    EarlyStopping,
    GradientTooSmall,
    ErrorIncreased,
}
```

## Validation

### Validator

Main validation interface.

```rust
pub struct Validator {
    config: ValidationConfig,
    strength_tester: StrengthTester,
    overfitting_detector: OverfittingDetector,
    performance_benchmark: PerformanceBenchmark,
    games_per_test: u32,
    time_control_ms: u32,
}
```

**Methods:**
- `new(config: ValidationConfig)` - Create new validator
- `cross_validate(positions: &[TrainingPosition]) -> ValidationResults` - Run cross-validation
- `holdout_validate(positions: &[TrainingPosition]) -> ValidationResults` - Run holdout validation
- `test_engine_strength(original_weights: &[f64], tuned_weights: &[f64]) -> MatchResult` - Test engine strength
- `detect_overfitting(training_error: f64, validation_error: f64) -> bool` - Detect overfitting
- `benchmark_performance(positions: &[TrainingPosition]) -> PerformanceMetrics` - Benchmark performance

### StrengthTester

Tests engine strength improvements.

```rust
pub struct StrengthTester {
    games_per_test: u32,
    time_control_ms: u32,
}
```

**Methods:**
- `new(games_per_test: u32, time_control_ms: u32)` - Create new tester
- `test_engine_strength(original_weights: &[f64], tuned_weights: &[f64]) -> MatchResult` - Test strength
- `calculate_weight_difference(weights1: &[f64], weights2: &[f64]) -> f64` - Calculate weight difference
- `estimate_strength_improvement(weight_difference: f64) -> f64` - Estimate strength improvement
- `simulate_match_results(strength_improvement: f64, games: u32) -> MatchResult` - Simulate match results
- `calculate_elo_difference(wins: u32, losses: u32, draws: u32) -> f64` - Calculate ELO difference
- `calculate_elo_confidence_interval(match_result: &MatchResult) -> (f64, f64)` - Calculate confidence interval

### ValidationResults

Results from validation process.

```rust
pub struct ValidationResults {
    pub mean_error: f64,
    pub std_error: f64,
    pub fold_results: Vec<FoldResult>,
    pub best_fold: Option<u32>,
    pub worst_fold: Option<u32>,
}
```

**Methods:**
- `new(fold_results: Vec<FoldResult>)` - Create new results
- `calculate_statistics()` - Calculate statistics
- `get_best_fold() -> Option<usize>` - Get best fold index
- `get_worst_fold() -> Option<usize>` - Get worst fold index
- `is_significant(threshold: f64) -> bool` - Check if results are significant

### MatchResult

Results from engine strength testing.

```rust
pub struct MatchResult {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub elo_difference: f64,
    pub elo_confidence_interval: (f64, f64),
    pub total_games: u32,
}
```

**Methods:**
- `new(wins: u32, losses: u32, draws: u32)` - Create new result
- `win_rate() -> f64` - Calculate win rate
- `elo_difference() -> f64` - Calculate ELO difference
- `is_significant(confidence_level: f64) -> bool` - Check if result is significant

## Performance Monitoring

### TuningProfiler

Comprehensive performance monitoring.

```rust
pub struct TuningProfiler {
    config: PerformanceConfig,
    start_time: Instant,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    error_history: Arc<Mutex<Vec<f64>>>,
    iteration_times: Arc<Mutex<Vec<Duration>>>,
    memory_snapshots: Arc<Mutex<Vec<(Instant, usize)>>>,
    checkpoint_frequency: usize,
    log_level: LogLevel,
}
```

**Methods:**
- `new(config: PerformanceConfig, log_level: LogLevel)` - Create new profiler
- `start_timer()` - Reset start time
- `record_memory_usage()` - Record current memory usage
- `record_iteration(iteration: usize, error: f64, iteration_time: Duration)` - Record iteration
- `log_iteration(iteration: usize, error: f64, iteration_time: Duration)` - Log iteration details
- `create_checkpoint(iteration: usize, error: f64, weights: Option<Vec<f64>>, validation_results: Option<ValidationResults>) -> Result<(), std::io::Error>` - Create checkpoint
- `load_latest_checkpoint() -> Option<CheckpointData>` - Load latest checkpoint
- `get_progress_info() -> ProgressInfo` - Get progress information
- `get_metrics() -> PerformanceMetrics` - Get performance metrics
- `generate_report(results: &TuningResults) -> String` - Generate performance report
- `plot_error_history(path: &Path)` - Plot error history
- `plot_weight_distribution(path: &Path, weights: &[f64])` - Plot weight distribution

### PerformanceMetrics

Performance statistics.

```rust
pub struct PerformanceMetrics {
    pub total_training_time: Duration,
    pub iterations_completed: usize,
    pub final_error: f64,
    pub peak_memory_mb: f64,
    pub avg_iteration_time_us: f64,
    pub avg_error_reduction: f64,
    pub convergence_rate: f64,
    pub checkpoints_saved: usize,
    pub last_checkpoint_time: Option<Instant>,
}
```

### ProgressInfo

Real-time progress information.

```rust
pub struct ProgressInfo {
    pub current_iteration: usize,
    pub total_iterations: Option<usize>,
    pub current_error: f64,
    pub eta_seconds: Option<f64>,
    pub elapsed_seconds: f64,
    pub memory_usage_mb: f64,
}
```

### CheckpointData

Checkpoint data for resuming optimization.

```rust
pub struct CheckpointData {
    pub timestamp: u64,
    pub iteration: usize,
    pub weights: Vec<f64>,
    pub current_error: f64,
    pub optimization_method: OptimizationMethod,
    pub metrics: PerformanceMetrics,
    pub validation_results: Option<ValidationResults>,
}
```

## Weight Management

### WeightManager

Manages tuned evaluation weights.

```rust
pub struct WeightManager {
    weights: Vec<f64>,
    enabled: bool,
    performance_stats: PerformanceStats,
}
```

**Methods:**
- `new() -> Self` - Create new weight manager
- `load_weights<P: AsRef<Path>>(path: P) -> Result<(), WeightError>` - Load weights from file
- `save_weights<P: AsRef<Path>>(path: P, tuning_method: String, validation_error: f64, training_positions: usize) -> Result<(), WeightError>` - Save weights to file
- `apply_weights(&mut self, features: &[f64], game_phase: i32) -> Result<i32, WeightError>` - Apply weights to features
- `get_weights() -> &Vec<f64>` - Get current weights
- `set_weights(weights: Vec<f64>)` - Set new weights
- `has_weights() -> bool` - Check if weights are loaded
- `is_enabled() -> bool` - Check if tuned weights are enabled
- `set_enabled(enabled: bool)` - Enable/disable tuned weights
- `get_stats() -> &PerformanceStats` - Get performance statistics
- `reset_stats()` - Reset performance statistics

### WeightError

Error type for weight management.

```rust
#[derive(Debug, thiserror::Error)]
pub enum WeightError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid magic number: expected {expected}, found {found}")]
    InvalidMagicNumber { expected: String, found: String },
    #[error("Invalid version: expected {expected}, found {found}")]
    InvalidVersion { expected: u32, found: u32 },
    #[error("Invalid feature count: expected {expected}, found {found}")]
    InvalidFeatureCount { expected: usize, found: usize },
    #[error("Invalid checksum: expected {expected}, found {found}")]
    InvalidChecksum { expected: String, found: String },
    #[error("Weights not loaded")]
    WeightsNotLoaded,
    #[error("Feature count mismatch: expected {expected}, found {found}")]
    FeatureCountMismatch { expected: usize, found: usize },
}
```

### WeightFileHeader

Header information for weight files.

```rust
pub struct WeightFileHeader {
    pub version: u32,
    pub magic_number: String,
    pub feature_count: usize,
    pub checksum: String,
    pub tuning_method: String,
    pub validation_error: f64,
    pub training_positions: usize,
    pub timestamp: u64,
}
```

### WeightFile

Complete weight file structure.

```rust
pub struct WeightFile {
    pub header: WeightFileHeader,
    pub weights: Vec<f64>,
}
```

## CLI Interface

### Main Commands

#### `tune` Command

```rust
struct TuneCommand {
    dataset: PathBuf,
    output: PathBuf,
    method: OptimizationMethod,
    iterations: usize,
    learning_rate: f64,
    k_fold: usize,
    test_split: f64,
    regularization: f64,
    min_rating: Option<u16>,
    quiet_threshold: u32,
    progress: bool,
    verbose: bool,
    checkpoint_frequency: usize,
    memory_limit: usize,
    threads: usize,
}
```

#### `validate` Command

```rust
struct ValidateCommand {
    weights: PathBuf,
    dataset: PathBuf,
    k_fold: usize,
    test_split: f64,
    games: u32,
    time_control: u32,
}
```

#### `benchmark` Command

```rust
struct BenchmarkCommand {
    weights: PathBuf,
    games: u32,
    time_control: u32,
    depth: Option<u8>,
    output: Option<PathBuf>,
}
```

#### `generate` Command

```rust
struct GenerateCommand {
    output: PathBuf,
    positions: usize,
    seed: Option<u64>,
    format: String,
}
```

#### `prepare-data` Command

```rust
struct PrepareDataCommand {
    input: PathBuf,
    output: PathBuf,
    format: String,
    filter_rating: bool,
    deduplicate: bool,
    quiet_only: bool,
    max_positions: Option<usize>,
}
```

### Configuration Files

#### JSON Configuration

```json
{
  "dataset": "games.json",
  "output": "weights.json",
  "method": {
    "type": "adam",
    "learning_rate": 0.01,
    "beta1": 0.9,
    "beta2": 0.999,
    "epsilon": 1e-8
  },
  "iterations": 5000,
  "k_fold": 5,
  "test_split": 0.2,
  "regularization": 0.001,
  "min_rating": 2000,
  "quiet_threshold": 3,
  "performance": {
    "memory_limit": 8192,
    "threads": 16,
    "checkpoint_frequency": 100,
    "enable_logging": true
  },
  "validation": {
    "k_fold": 5,
    "test_split": 0.2,
    "validation_split": 0.1,
    "stratified": true,
    "random_seed": 42
  }
}
```

#### YAML Configuration

```yaml
dataset: games.json
output: weights.json
method:
  type: adam
  learning_rate: 0.01
  beta1: 0.9
  beta2: 0.999
  epsilon: 1e-8
iterations: 5000
k_fold: 5
test_split: 0.2
regularization: 0.001
min_rating: 2000
quiet_threshold: 3
performance:
  memory_limit: 8192
  threads: 16
  checkpoint_frequency: 100
  enable_logging: true
validation:
  k_fold: 5
  test_split: 0.2
  validation_split: 0.1
  stratified: true
  random_seed: 42
```

## Error Handling

### Common Error Types

```rust
// Data processing errors
#[derive(Debug, thiserror::Error)]
pub enum DataProcessingError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("Invalid format: {format}")]
    InvalidFormat { format: String },
    #[error("Parse error: {message}")]
    ParseError { message: String },
    #[error("Insufficient data: {count} positions")]
    InsufficientData { count: usize },
}

// Optimization errors
#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("Convergence failure: {reason}")]
    ConvergenceFailure { reason: String },
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },
    #[error("Memory limit exceeded: {limit}MB")]
    MemoryLimitExceeded { limit: usize },
    #[error("Numerical instability: {details}")]
    NumericalInstability { details: String },
}

// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },
    #[error("Insufficient test data: {count} positions")]
    InsufficientTestData { count: usize },
    #[error("Overfitting detected: training={train}, validation={val}")]
    OverfittingDetected { train: f64, val: f64 },
}
```

## Constants

### Feature Constants

```rust
pub const NUM_EVAL_FEATURES: usize = 2000;
pub const NUM_MG_FEATURES: usize = 1000;
pub const NUM_EG_FEATURES: usize = 1000;

// Feature type counts
pub const NUM_MATERIAL_FEATURES: usize = 16;
pub const NUM_POSITIONAL_FEATURES: usize = 800;
pub const NUM_KING_SAFETY_FEATURES: usize = 32;
pub const NUM_PAWN_STRUCTURE_FEATURES: usize = 64;
pub const NUM_MOBILITY_FEATURES: usize = 32;
pub const NUM_COORDINATION_FEATURES: usize = 32;
pub const NUM_CENTER_CONTROL_FEATURES: usize = 16;
pub const NUM_DEVELOPMENT_FEATURES: usize = 8;
```

### File Format Constants

```rust
pub const WEIGHT_FILE_VERSION: u32 = 1;
pub const WEIGHT_FILE_MAGIC_NUMBER: &str = "SHOGI_WEIGHTS_V1";

// Supported formats
pub const SUPPORTED_FORMATS: &[&str] = &["kif", "csa", "pgn", "json"];
```

## Next Steps

- [User Guide](USER_GUIDE.md) for usage instructions
- [Optimization Examples](OPTIMIZATION_EXAMPLES.md) for configuration examples
- [Integration Guide](INTEGRATION_GUIDE.md) for engine integration
- [Code Examples](CODE_EXAMPLES.md) for extending the system
