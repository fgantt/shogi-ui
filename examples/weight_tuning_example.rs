//! Example: Weight Tuning with IntegratedEvaluator (Task 20.0 - Task 4.15)
//!
//! This example demonstrates how to use the tuning infrastructure to optimize
//! evaluation weights using training positions with expected scores.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::config::EvaluationWeights;
use shogi_engine::evaluation::integration::{
    IntegratedEvaluator, TuningConfig, TuningPosition, TuningPositionSet,
};
use shogi_engine::tuning::OptimizationMethod;
use shogi_engine::types::{CapturedPieces, Player};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Weight Tuning Example ===");
    println!();

    // Create an evaluator
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    println!("1. Creating training position set...");
    let mut position_set = TuningPositionSet::empty();

    // Create training positions from games
    // In a real scenario, you would load these from game databases
    for i in 0..100 {
        // Create positions (using initial position as example)
        let position = TuningPosition {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            player: if i % 2 == 0 {
                Player::Black
            } else {
                Player::White
            },
            expected_score: if i % 3 == 0 {
                0.0 // Draw
            } else if i % 3 == 1 {
                0.5 // White advantage
            } else {
                -0.5 // Black advantage
            },
            game_phase: if i < 33 {
                200 // Opening
            } else if i < 66 {
                128 // Middlegame
            } else {
                32 // Endgame
            },
            move_number: (i % 50) as u32 + 1,
        };
        position_set.add_position(position);
    }

    println!("   Created {} training positions", position_set.len());
    println!();

    // Configure tuning
    println!("2. Configuring tuning...");
    let tuning_config = TuningConfig {
        method: OptimizationMethod::Adam {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        },
        max_iterations: 100,
        convergence_threshold: 1e-6,
        learning_rate: 0.001,
        k_factor: 1.0,
    };

    println!("   Method: Adam optimizer");
    println!("   Max iterations: {}", tuning_config.max_iterations);
    println!("   Learning rate: {}", tuning_config.learning_rate);
    println!();

    // Get initial weights for comparison (access through config)
    // Note: In real usage, you would access weights through the config
    let initial_weights = EvaluationWeights::default(); // Simplified for example
    println!("3. Initial weights:");
    println!("   Material: {:.3}", initial_weights.material_weight);
    println!("   Position: {:.3}", initial_weights.position_weight);
    println!("   King Safety: {:.3}", initial_weights.king_safety_weight);
    println!();

    // Run tuning
    println!("4. Running weight tuning...");
    let result = evaluator.tune_weights(&position_set, &tuning_config)?;

    println!("   Tuning completed!");
    println!("   Iterations: {}", result.iterations);
    println!("   Final error: {:.6}", result.final_error);
    println!("   Convergence: {:?}", result.convergence_reason);
    println!(
        "   Optimization time: {:.2}s",
        result.optimization_time.as_secs_f64()
    );
    println!();

    // Display optimized weights
    println!("5. Optimized weights:");
    println!(
        "   Material: {:.3} (was {:.3})",
        result.optimized_weights.material_weight, initial_weights.material_weight
    );
    println!(
        "   Position: {:.3} (was {:.3})",
        result.optimized_weights.position_weight, initial_weights.position_weight
    );
    println!(
        "   King Safety: {:.3} (was {:.3})",
        result.optimized_weights.king_safety_weight, initial_weights.king_safety_weight
    );
    println!(
        "   Tactical: {:.3} (was {:.3})",
        result.optimized_weights.tactical_weight, initial_weights.tactical_weight
    );
    println!(
        "   Positional: {:.3} (was {:.3})",
        result.optimized_weights.positional_weight, initial_weights.positional_weight
    );
    println!();

    // Apply optimized weights
    println!("6. Applying optimized weights...");
    // Note: In real usage, you would update weights through the config
    // For example: evaluator.set_config(...) with updated weights
    println!("   Optimized weights available in result.optimized_weights");
    println!("   Apply weights using evaluator configuration update");
    println!();

    // Show error history (first and last few iterations)
    println!("7. Error history (sample):");
    if result.error_history.len() > 0 {
        println!("   Iteration 1: {:.6}", result.error_history[0]);
        if result.error_history.len() > 1 {
            println!(
                "   Iteration {}: {:.6}",
                result.error_history.len(),
                result.error_history[result.error_history.len() - 1]
            );
        }
    }
    println!();

    println!("=== Tuning Complete ===");
    Ok(())
}
