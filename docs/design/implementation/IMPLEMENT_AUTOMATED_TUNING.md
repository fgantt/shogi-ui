# Implementation Plan: Automated Evaluation Tuning

## 1. Objective

To establish a framework for automatically tuning the weights of the engine's evaluation function. This data-driven approach will replace manual tuning, leading to a more accurate, balanced, and ultimately stronger engine by optimizing the evaluation parameters against a large dataset of real-world game outcomes.

## 2. Background

A shogi engine's evaluation function contains hundreds of parameters (material values, piece-square table entries, king safety bonuses, etc.). Manually finding the optimal balance for these weights is practically impossible. Automated tuning solves this by using statistical methods to find the set of weights that best predicts the outcomes of actual games.

The standard approach, often called **Texel's Tuning Method**, uses logistic regression. It works by:
1.  Collecting a massive number of quiet positions from high-quality games, each labeled with the final game result (Win, Loss, or Draw).
2.  Defining a function that maps the engine's evaluation score to a predicted win probability.
3.  Using a numerical optimization algorithm to adjust the evaluation weights until the predicted outcomes are as close as possible to the actual game results.

This process is performed **offline** by a separate tuning program, not by the engine itself during a game.

## 3. Core Logic and Implementation Plan

### Step 1: Refactor the Evaluation Function for Tunability

This is the most critical prerequisite. The existing `evaluate` function, which returns a single `i32`, must be refactored to output a vector of its individual, unweighted components.

**File:** `src/evaluation.rs`

```rust
// The evaluation function needs to be changed to return a vector of feature values.
// The weights will be applied separately.
pub fn get_evaluation_features(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<f64> {
    let mut features = vec![0.0; NUM_EVAL_FEATURES]; // NUM_EVAL_FEATURES is a constant we define

    // For each term, calculate its raw value and add it to the feature vector.
    // Example: Material
    let (black_material, white_material) = self.calculate_material(board);
    features[MATERIAL_PAWN_INDEX] = (black_pawn_count - white_pawn_count) as f64;
    features[MATERIAL_ROOK_INDEX] = (black_rook_count - white_rook_count) as f64;
    // ... and so on for every piece type and every evaluation term.

    // Example: King Safety
    features[KING_SAFETY_MG_INDEX] = self.calculate_king_safety_mg(board, player) as f64;
    features[KING_SAFETY_EG_INDEX] = self.calculate_king_safety_eg(board, player) as f64;

    features
}
```

### Step 2: Create a Standalone Tuner Binary

The tuning process will live in a separate executable.

**File:** `Cargo.toml`
```toml
[[bin]]
name = "tuner"
path = "src/bin/tuner.rs"
```

**File:** `src/bin/tuner.rs`
This file will contain the main logic for the tuning program.

### Step 3: Data Preparation

The tuner needs to read a large dataset of games (e.g., in KIF format).

```rust
// In tuner.rs

struct TrainingPosition {
    features: Vec<f64>,
    result: f64, // 1.0 for Win, 0.5 for Draw, 0.0 for Loss
}

fn load_positions_from_kif(file_path: &str) -> Vec<TrainingPosition> {
    // 1. Read and parse the KIF file.
    // 2. For each game, iterate through the moves.
    // 3. Select only "quiet" positions (e.g., where the last move was not a capture).
    // 4. For each selected position, use the engine's `get_evaluation_features` to get the feature vector.
    // 5. Store the feature vector along with the final game result.
    // ...
}
```

### Step 4: Implement the Optimization Algorithm

The core of the tuner is the optimization loop that minimizes the error between the model's predictions and the actual results.

```rust
// In tuner.rs

// The logistic function maps an evaluation score E to a win probability.
fn predict_win_prob(eval: f64, k: f64) -> f64 {
    1.0 / (1.0 + 10.0f64.powf(-k * eval / 400.0))
}

// The error function to minimize (mean squared error).
fn calculate_error(positions: &[TrainingPosition], weights: &[f64], k: f64) -> f64 {
    let mut total_error = 0.0;
    for pos in positions {
        let eval = dot_product(&pos.features, weights);
        let prediction = predict_win_prob(eval, k);
        total_error += (pos.result - prediction).powi(2);
    }
    total_error / positions.len() as f64
}

fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn main() {
    // 1. Load data.
    let training_data = load_positions_from_kif("path/to/games.kif");

    // 2. Initialize weights (e.g., to current manual values or all 1.0).
    let mut weights = vec![1.0; NUM_EVAL_FEATURES];

    // 3. Use an optimization algorithm (like simple gradient descent or a library like `argmin`)
    //    to iteratively adjust `weights` to minimize `calculate_error`.
    // ... optimization loop ...

    // 4. Print the final, optimized weights.
    println!("const OPTIMIZED_WEIGHTS: [f64; {}] = {:?};", NUM_EVAL_FEATURES, weights);
}
```

### Step 5: Use the Tuned Weights in the Engine

The output of the tuner is a Rust array of weights. This can be saved to a file (e.g., `src/tuned_weights.rs`) and compiled into the engine. The main `evaluate` function will then compute the final score by taking the dot product of the feature vector and the tuned weight vector.

## 4. Dependencies and Considerations

*   **Evaluation Refactoring:** This is a major prerequisite. The evaluation function *must* be cleanly separated into unweighted feature calculation and weighted summation.
*   **Dataset:** A large, high-quality dataset of several hundred thousand games is required for good results.
*   **Computational Cost:** The tuning process is very computationally expensive and may take hours or days to run.
*   **Numerical Optimization:** Implementing gradient-based optimization correctly is complex. Using a crate like `argmin` is highly recommended to handle the optimization algorithm.

## 5. Verification Plan

1.  **Tuner Sanity Check:** Create a small, synthetic dataset with known weights. Run the tuner on this dataset and verify that it can successfully recover the original weights. This confirms the optimization logic is working.
2.  **Strength Testing (Primary):** This is the ultimate validation. Compile the engine with the newly tuned weights. Play a match of several hundred games against the version with the old, hand-tuned weights. A successful tuning process should result in a significantly higher win rate.
3.  **Error Minimization:** Monitor the error value produced by the tuner. A lower final error value indicates a better fit to the data and should correlate with a stronger engine.

