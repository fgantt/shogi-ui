# Design Document: Automated Evaluation Tuning System

## 1. Executive Summary

This document presents a comprehensive design for implementing an automated evaluation tuning system for the shogi engine using the Texel's Tuning Method. The system will automatically optimize evaluation function parameters by learning from large datasets of real game positions, replacing manual tuning with data-driven optimization.

## 2. Architecture Overview

### 2.1 System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Tuning System Architecture               │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ Data        │    │ Feature     │    │ Optimizer   │     │
│  │ Processor   │───▶│ Extractor   │───▶│ Engine      │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │          │
│         ▼                   ▼                   ▼          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ Game        │    │ Evaluation  │    │ Weight      │     │
│  │ Database    │    │ Features    │    │ Generator   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Key Design Principles

1. **Separation of Concerns**: Tuning is completely separate from the engine runtime
2. **Modular Feature System**: Each evaluation term is independently tunable
3. **Robust Optimization**: Multiple optimization algorithms and validation methods
4. **Incremental Tuning**: Support for partial tuning of specific feature groups
5. **Performance Monitoring**: Comprehensive logging and analysis capabilities

## 3. Detailed Component Design

### 3.1 Feature Extraction System

#### 3.1.1 Feature Vector Structure

The evaluation function must be refactored to output unweighted feature values:

```rust
// Feature indices for different evaluation terms
pub const NUM_EVAL_FEATURES: usize = 2000; // Estimated based on analysis

// Material features (14 piece types × 2 players = 28 features)
pub const MATERIAL_PAWN_INDEX: usize = 0;
pub const MATERIAL_LANCE_INDEX: usize = 1;
// ... etc for all piece types

// Positional features (piece-square tables)
pub const PST_PAWN_MG_START: usize = 28;
pub const PST_PAWN_EG_START: usize = 28 + 81;
// ... etc for all pieces and phases

// King safety features
pub const KING_SAFETY_CASTLE_INDEX: usize = 500;
pub const KING_SAFETY_ATTACK_INDEX: usize = 501;
// ... etc

// Other evaluation terms
pub const PAWN_STRUCTURE_INDEX: usize = 600;
pub const MOBILITY_INDEX: usize = 700;
pub const COORDINATION_INDEX: usize = 800;
```

#### 3.1.2 Refactored Evaluation Function

```rust
impl PositionEvaluator {
    /// Extract raw feature values for tuning
    pub fn get_evaluation_features(
        &self, 
        board: &BitboardBoard, 
        player: Player, 
        captured_pieces: &CapturedPieces
    ) -> Vec<f64> {
        let mut features = vec![0.0; NUM_EVAL_FEATURES];
        
        // Material features
        self.extract_material_features(&mut features, board, player);
        
        // Positional features
        self.extract_positional_features(&mut features, board, player);
        
        // King safety features
        self.extract_king_safety_features(&mut features, board, player);
        
        // Other evaluation terms
        self.extract_other_features(&mut features, board, player, captured_pieces);
        
        features
    }
    
    /// Apply tuned weights to features
    pub fn evaluate_with_weights(
        &self,
        features: &[f64],
        weights: &[f64],
        game_phase: i32
    ) -> i32 {
        // Apply phase-dependent weighting
        let phase_weight = game_phase as f64 / GAME_PHASE_MAX as f64;
        
        let mut mg_score = 0.0;
        let mut eg_score = 0.0;
        
        for (i, &feature) in features.iter().enumerate() {
            if i < NUM_MG_FEATURES {
                mg_score += feature * weights[i];
            } else {
                eg_score += feature * weights[i];
            }
        }
        
        // Interpolate based on game phase
        let final_score = mg_score * phase_weight + eg_score * (1.0 - phase_weight);
        final_score as i32
    }
}
```

### 3.2 Data Processing Pipeline

#### 3.2.1 Game Database Structure

```rust
#[derive(Debug, Clone)]
pub struct GameRecord {
    pub moves: Vec<Move>,
    pub result: GameResult, // Win/Loss/Draw from perspective of first player
    pub player_ratings: (u16, u16), // ELO ratings
    pub game_phase: GamePhase,
    pub time_control: Option<TimeControl>,
}

#[derive(Debug, Clone)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
}

#[derive(Debug, Clone)]
pub struct TrainingPosition {
    pub features: Vec<f64>,
    pub result: f64, // 1.0, 0.5, or 0.0
    pub game_phase: i32,
    pub is_quiet: bool,
    pub move_number: u32,
}
```

#### 3.2.2 Position Selection Criteria

```rust
pub struct PositionFilter {
    /// Only include quiet positions (no captures in last N moves)
    pub quiet_move_threshold: u32,
    /// Minimum rating for games to include
    pub min_rating: u16,
    /// Maximum rating for games to include
    pub max_rating: u16,
    /// Skip positions from first N moves
    pub skip_opening_moves: u32,
    /// Skip positions from last N moves
    pub skip_endgame_moves: u32,
    /// Maximum positions per game
    pub max_positions_per_game: usize,
}
```

### 3.3 Optimization Engine

#### 3.3.1 Texel's Tuning Method Implementation

```rust
pub struct TexelTuner {
    positions: Vec<TrainingPosition>,
    weights: Vec<f64>,
    k_factor: f64,
    learning_rate: f64,
    regularization: f64,
}

impl TexelTuner {
    /// Logistic function for win probability prediction
    fn sigmoid(&self, eval: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-self.k_factor * eval / 400.0))
    }
    
    /// Calculate mean squared error
    fn calculate_error(&self) -> f64 {
        let mut total_error = 0.0;
        
        for position in &self.positions {
            let eval = self.dot_product(&position.features, &self.weights);
            let prediction = self.sigmoid(eval);
            let error = (position.result - prediction).powi(2);
            total_error += error;
        }
        
        total_error / self.positions.len() as f64
    }
    
    /// Calculate gradients for gradient descent
    fn calculate_gradients(&self) -> Vec<f64> {
        let mut gradients = vec![0.0; self.weights.len()];
        
        for position in &self.positions {
            let eval = self.dot_product(&position.features, &self.weights);
            let prediction = self.sigmoid(eval);
            let error = position.result - prediction;
            
            for (i, &feature) in position.features.iter().enumerate() {
                gradients[i] += error * feature * self.sigmoid_derivative(eval);
            }
        }
        
        // Apply regularization
        for (i, gradient) in gradients.iter_mut().enumerate() {
            *gradient = *gradient / self.positions.len() as f64 + self.regularization * self.weights[i];
        }
        
        gradients
    }
}
```

#### 3.3.2 Advanced Optimization Algorithms

```rust
pub enum OptimizationMethod {
    GradientDescent {
        learning_rate: f64,
        momentum: f64,
        adaptive_rate: bool,
    },
    Adam {
        alpha: f64,
        beta1: f64,
        beta2: f64,
        epsilon: f64,
    },
    LBFGS {
        memory_size: usize,
        tolerance: f64,
    },
    GeneticAlgorithm {
        population_size: usize,
        mutation_rate: f64,
        crossover_rate: f64,
        generations: usize,
    },
}

pub struct OptimizationEngine {
    method: OptimizationMethod,
    max_iterations: usize,
    convergence_threshold: f64,
    early_stopping_patience: usize,
}
```

### 3.4 Validation and Testing Framework

#### 3.4.1 Cross-Validation System

```rust
pub struct ValidationFramework {
    k_fold: usize,
    test_split: f64,
    validation_split: f64,
}

impl ValidationFramework {
    pub fn cross_validate(&self, positions: &[TrainingPosition]) -> ValidationResults {
        let mut results = Vec::new();
        
        for fold in 0..self.k_fold {
            let (train_data, test_data) = self.split_data(positions, fold);
            let mut tuner = TexelTuner::new(train_data);
            tuner.optimize();
            
            let test_error = tuner.calculate_error_on_dataset(&test_data);
            results.push(test_error);
        }
        
        ValidationResults {
            mean_error: results.iter().sum::<f64>() / results.len() as f64,
            std_error: self.calculate_std_dev(&results),
            fold_results: results,
        }
    }
}
```

#### 3.4.2 Engine Strength Testing

```rust
pub struct StrengthTester {
    engine_a: Box<dyn ShogiEngine>,
    engine_b: Box<dyn ShogiEngine>,
    time_control: TimeControl,
    games_per_match: usize,
}

impl StrengthTester {
    pub fn run_match(&mut self) -> MatchResult {
        let mut results = Vec::new();
        
        for game in 0..self.games_per_match {
            let result = self.play_game();
            results.push(result);
        }
        
        MatchResult {
            wins: results.iter().filter(|&&r| r == GameResult::EngineAWin).count(),
            losses: results.iter().filter(|&&r| r == GameResult::EngineBWin).count(),
            draws: results.iter().filter(|&&r| r == GameResult::Draw).count(),
            elo_difference: self.calculate_elo_difference(&results),
        }
    }
}
```

## 4. Implementation Phases

### Phase 1: Foundation (Weeks 1-2)
- Refactor evaluation function for feature extraction
- Implement basic data structures and interfaces
- Create position filtering and selection logic
- Set up basic testing framework

### Phase 2: Core Tuning (Weeks 3-4)
- Implement Texel's tuning method
- Add gradient descent optimization
- Create weight management system
- Implement basic validation

### Phase 3: Advanced Features (Weeks 5-6)
- Add multiple optimization algorithms
- Implement cross-validation
- Create strength testing framework
- Add performance monitoring

### Phase 4: Integration and Testing (Weeks 7-8)
- Integrate with existing engine
- Comprehensive testing and validation
- Performance optimization
- Documentation and examples

## 5. Technical Considerations

### 5.1 Performance Requirements

- **Feature Extraction**: < 1ms per position
- **Optimization**: Handle 1M+ positions efficiently
- **Memory Usage**: < 8GB for typical datasets
- **Convergence**: < 1000 iterations for most cases

### 5.2 Data Requirements

- **Minimum Dataset**: 100,000 positions from 10,000+ games
- **Quality Requirements**: Games from 2000+ ELO players
- **Format Support**: KIF, CSA, PGN formats
- **Storage**: Compressed binary format for efficiency

### 5.3 Robustness Features

- **Overfitting Prevention**: Regularization and validation
- **Numerical Stability**: Careful handling of edge cases
- **Reproducibility**: Deterministic random seeds
- **Error Handling**: Graceful degradation on invalid data

## 6. Configuration and Usage

### 6.1 Tuning Configuration

```rust
pub struct TuningConfig {
    pub dataset_path: String,
    pub output_weights_path: String,
    pub optimization_method: OptimizationMethod,
    pub position_filter: PositionFilter,
    pub validation_config: ValidationConfig,
    pub performance_config: PerformanceConfig,
}

pub struct PerformanceConfig {
    pub max_memory_gb: usize,
    pub num_threads: usize,
    pub checkpoint_interval: usize,
    pub progress_reporting: bool,
}
```

### 6.2 Command Line Interface

```bash
# Basic tuning
cargo run --bin tuner -- \
    --dataset games.kif \
    --output weights.rs \
    --method gradient_descent \
    --iterations 1000

# Advanced tuning with validation
cargo run --bin tuner -- \
    --dataset games.kif \
    --output weights.rs \
    --method adam \
    --k-fold 5 \
    --test-split 0.2 \
    --regularization 0.001 \
    --quiet-threshold 2 \
    --min-rating 2000
```

## 7. Expected Outcomes

### 7.1 Performance Improvements

- **Evaluation Accuracy**: 15-25% reduction in prediction error
- **Engine Strength**: 50-100 ELO point improvement
- **Consistency**: More balanced evaluation across game phases
- **Robustness**: Better handling of edge cases and unusual positions

### 7.2 Development Benefits

- **Automated Optimization**: Eliminates manual parameter tuning
- **Data-Driven Decisions**: Objective optimization criteria
- **Scalability**: Easy to add new evaluation features
- **Reproducibility**: Consistent results across runs

## 8. Risk Mitigation

### 8.1 Technical Risks

- **Overfitting**: Mitigated by cross-validation and regularization
- **Numerical Instability**: Careful implementation and testing
- **Performance Impact**: Profiling and optimization
- **Data Quality**: Multiple validation layers

### 8.2 Implementation Risks

- **Complexity**: Phased implementation with clear milestones
- **Integration**: Extensive testing with existing engine
- **Maintenance**: Comprehensive documentation and examples
- **Scalability**: Modular design for future expansion

## 9. Future Enhancements

### 9.1 Advanced Features

- **Online Learning**: Continuous tuning during engine use
- **Multi-Objective Optimization**: Balance accuracy vs. speed
- **Feature Engineering**: Automatic feature discovery
- **Ensemble Methods**: Multiple evaluation functions

### 9.2 Integration Opportunities

- **Opening Book**: Tune evaluation for opening positions
- **Endgame Database**: Specialized endgame evaluation
- **Time Management**: Evaluation-based time allocation
- **Personality Profiles**: Different evaluation styles

This design provides a comprehensive framework for implementing automated evaluation tuning that will significantly improve the engine's playing strength while maintaining code quality and performance.
