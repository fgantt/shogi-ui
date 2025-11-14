//! Example: Telemetry-to-Tuning Pipeline (Task 20.0 - Task 4.16)
//!
//! This example demonstrates how to collect telemetry from evaluations and
//! use it to tune weights automatically.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use shogi_engine::evaluation::statistics::EvaluationTelemetry;
use shogi_engine::types::{CapturedPieces, Player};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Telemetry-to-Tuning Pipeline Example ===");
    println!();

    // Create an evaluator with statistics enabled
    let mut evaluator = IntegratedEvaluator::new();
    evaluator.enable_statistics();

    println!("1. Collecting telemetry from evaluations...");
    let mut telemetry_collection = Vec::new();

    // Evaluate multiple positions and collect telemetry
    for i in 0..50 {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = if i % 2 == 0 {
            Player::Black
        } else {
            Player::White
        };

        // Evaluate position
        evaluator.evaluate(&board, player, &captured_pieces);

        // Get telemetry snapshot
        if let Some(telemetry) = evaluator.telemetry_snapshot() {
            // In a real scenario, expected_score would come from game results
            let expected_score = if i % 3 == 0 {
                0.0 // Draw
            } else if i % 3 == 1 {
                0.3 // Advantage
            } else {
                -0.3 // Disadvantage
            };

            telemetry_collection.push((
                board.clone(),
                captured_pieces.clone(),
                player,
                telemetry,
                expected_score,
            ));
        }
    }

    println!(
        "   Collected telemetry from {} positions",
        telemetry_collection.len()
    );
    println!();

    // Convert telemetry to tuning position set
    println!("2. Converting telemetry to tuning position set...");
    let position_set = evaluator.telemetry_to_tuning_pipeline(&telemetry_collection);
    println!("   Created {} tuning positions", position_set.len());
    println!();

    // Analyze telemetry contributions
    println!("3. Analyzing weight contributions from telemetry...");
    let mut aggregated_contributions: HashMap<String, f32> = HashMap::new();

    for (_, _, _, telemetry, _) in &telemetry_collection {
        for (component, contribution) in &telemetry.weight_contributions {
            *aggregated_contributions
                .entry(component.clone())
                .or_insert(0.0) += contribution;
        }
    }

    // Average contributions
    let count = telemetry_collection.len() as f32;
    for contribution in aggregated_contributions.values_mut() {
        *contribution /= count;
    }

    println!("   Component contributions (averaged):");
    for (component, contribution) in &aggregated_contributions {
        println!("     {}: {:.2}%", component, contribution * 100.0);
    }
    println!();

    // Option 1: Use tune_from_telemetry() for quick adjustments
    println!("4. Option 1: Quick weight adjustment from telemetry...");
    let telemetry_set: Vec<EvaluationTelemetry> = telemetry_collection
        .iter()
        .map(|(_, _, _, telemetry, _)| telemetry.clone())
        .collect();

    // Define target contributions (optional - can be None for automatic)
    let mut target_contributions = HashMap::new();
    target_contributions.insert("material".to_string(), 0.25);
    target_contributions.insert("position".to_string(), 0.20);
    target_contributions.insert("tactical".to_string(), 0.15);
    target_contributions.insert("positional".to_string(), 0.15);

    let adjusted_weights =
        evaluator.tune_from_telemetry(&telemetry_set, Some(&target_contributions), 0.01)?;

    println!("   Adjusted weights:");
    println!("     Material: {:.3}", adjusted_weights.material_weight);
    println!("     Position: {:.3}", adjusted_weights.position_weight);
    println!("     Tactical: {:.3}", adjusted_weights.tactical_weight);
    println!("     Positional: {:.3}", adjusted_weights.positional_weight);
    println!();

    // Option 2: Use full tune_weights() with position set for thorough optimization
    println!("5. Option 2: Full weight tuning from position set...");
    println!("   (This would use tune_weights() for more thorough optimization)");
    println!("   See weight_tuning_example.rs for details");
    println!();

    // Export telemetry data for analysis
    println!("6. Exporting telemetry data for analysis...");
    if let Some(first_telemetry) = telemetry_collection.first() {
        let exported_data = first_telemetry.3.export_for_tuning();
        println!("   Exported {} data points", exported_data.len());
        println!(
            "   Sample keys: {:?}",
            exported_data.keys().take(5).collect::<Vec<_>>()
        );
    }
    println!();

    println!("=== Telemetry Pipeline Complete ===");
    println!();
    println!("Next steps:");
    println!("  - Use tune_weights() with the position set for full optimization");
    println!("  - Analyze exported telemetry data to understand component contributions");
    println!("  - Adjust target_contributions based on desired play style");

    Ok(())
}
