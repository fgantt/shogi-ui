//! Machine Learning Replacement Policies Example
//!
//! This example demonstrates how to use machine learning-based replacement
//! policies for transposition tables, showing different algorithms and
//! their learning capabilities.

use shogi_engine::search::ml_replacement_policies::ReplacementOutcome;
use shogi_engine::search::{
    AccessPatternInfo, MLAlgorithm, MLReplacementConfig, MLReplacementContext,
    MLReplacementDecision, MLReplacementPolicy, PositionFeatures, ReplacementAction, TemporalInfo,
};
use shogi_engine::types::{EntrySource, TranspositionEntry, TranspositionFlag};

fn main() {
    println!("Machine Learning Replacement Policies Example");
    println!("=============================================");

    // Demonstrate different ML algorithms
    demonstrate_ml_algorithms();

    // Demonstrate feature extraction
    demonstrate_feature_extraction();

    // Demonstrate training and learning
    demonstrate_training_learning();

    // Demonstrate performance monitoring
    demonstrate_performance_monitoring();

    // Demonstrate real-time adaptation
    demonstrate_realtime_adaptation();

    println!("\nMachine Learning Replacement Policies Example completed successfully!");
}

fn demonstrate_ml_algorithms() {
    println!("\n--- ML Algorithms Demo ---");

    let algorithms = [
        (
            "Linear Regression",
            MLReplacementConfig::linear_regression(),
        ),
        ("Neural Network", MLReplacementConfig::neural_network()),
        (
            "Reinforcement Learning",
            MLReplacementConfig::reinforcement_learning(),
        ),
    ];

    let context = create_sample_context();

    for (name, config) in algorithms {
        let mut policy = MLReplacementPolicy::new(config);

        println!("Testing {} algorithm:", name);

        // Make multiple decisions to see variation
        for i in 0..5 {
            let decision = policy.decide_replacement(&context);

            println!(
                "  Decision {}: {:?} (confidence: {:.2})",
                i + 1,
                decision.decision,
                decision.confidence
            );

            // Simulate outcome
            let outcome = create_sample_outcome(i % 2 == 0);
            policy.record_outcome(&decision, outcome);
        }

        let stats = policy.get_performance_stats();
        println!(
            "  Stats: {} decisions, {:.1}% accuracy, {:.2} avg confidence",
            stats.total_decisions,
            (stats.correct_decisions as f64 / stats.total_decisions as f64) * 100.0,
            stats.avg_confidence
        );
    }
}

fn demonstrate_feature_extraction() {
    println!("\n--- Feature Extraction Demo ---");

    let config = MLReplacementConfig::neural_network();
    let mut policy = MLReplacementPolicy::new(config);

    // Test different contexts with varying features
    let contexts = vec![
        ("Simple position", create_simple_context()),
        ("Complex tactical position", create_complex_context()),
        ("Endgame position", create_endgame_context()),
        ("Opening position", create_opening_context()),
    ];

    for (name, context) in contexts {
        let decision = policy.decide_replacement(&context);

        println!("{}:", name);
        println!("  Features extracted: {}", decision.features.len());
        println!("  Decision: {:?}", decision.decision);
        println!("  Confidence: {:.2}", decision.confidence);

        // Show feature values (first few)
        if !decision.features.is_empty() {
            println!(
                "  Sample features: [{:.2}, {:.2}, {:.2}, ...]",
                decision.features[0], decision.features[1], decision.features[2]
            );
        }
    }
}

fn demonstrate_training_learning() {
    println!("\n--- Training and Learning Demo ---");

    let config = MLReplacementConfig::linear_regression();
    let mut policy = MLReplacementPolicy::new(config);

    println!("Training ML model with simulated game data...");

    // Simulate a sequence of replacement decisions
    for game_phase in 0..3 {
        println!(
            "Game phase {}: Training on {} decisions",
            game_phase + 1,
            20
        );

        for decision_num in 0..20 {
            let context = create_varying_context(decision_num);
            let decision = policy.decide_replacement(&context);

            // Simulate outcome based on some heuristic
            let beneficial = decision.confidence > 0.6 || decision_num % 3 == 0;
            let outcome = create_sample_outcome(beneficial);

            policy.record_outcome(&decision, outcome);

            if decision_num % 5 == 0 {
                println!(
                    "  Decision {}: {:?} -> {}",
                    decision_num,
                    decision.decision,
                    if beneficial { "Good" } else { "Poor" }
                );
            }
        }

        let stats = policy.get_performance_stats();
        let accuracy = if stats.total_decisions > 0 {
            stats.correct_decisions as f64 / stats.total_decisions as f64
        } else {
            0.0
        };

        println!(
            "  Phase {} accuracy: {:.1}%",
            game_phase + 1,
            accuracy * 100.0
        );
    }

    let final_stats = policy.get_performance_stats();
    println!(
        "Final performance: {:.1}% accuracy over {} decisions",
        (final_stats.correct_decisions as f64 / final_stats.total_decisions as f64) * 100.0,
        final_stats.total_decisions
    );
}

fn demonstrate_performance_monitoring() {
    println!("\n--- Performance Monitoring Demo ---");

    let config = MLReplacementConfig::neural_network();
    let mut policy = MLReplacementPolicy::new(config);

    println!("Monitoring ML policy performance...");

    let mut decision_history = Vec::new();

    // Simulate decision making with performance tracking
    for i in 0..50 {
        let context = create_sample_context();
        let decision = policy.decide_replacement(&context);

        // Simulate performance impact
        let performance_impact = match decision.decision {
            ReplacementAction::ReplaceWithNew => 0.1,
            ReplacementAction::KeepExisting => 0.05,
            ReplacementAction::StoreNewElsewhere => 0.08,
        };

        let outcome = ReplacementOutcome {
            beneficial: decision.confidence > 0.7,
            performance_impact,
            cache_efficiency: 0.8 + (decision.confidence * 0.2),
            access_frequency: 0.5 + (decision.confidence * 0.3),
        };

        policy.record_outcome(&decision, outcome);
        decision_history.push((decision.confidence, outcome.beneficial));

        if i % 10 == 0 {
            let stats = policy.get_performance_stats();
            println!(
                "  After {} decisions: {:.1}% accuracy, {:.2} avg confidence, {:.3} avg impact",
                stats.total_decisions,
                (stats.correct_decisions as f64 / stats.total_decisions as f64) * 100.0,
                stats.avg_confidence,
                stats.avg_performance_impact
            );
        }
    }

    // Analyze decision quality over time
    let high_confidence_decisions: usize = decision_history
        .iter()
        .filter(|(confidence, _)| *confidence > 0.8)
        .count();

    let beneficial_high_confidence: usize = decision_history
        .iter()
        .filter(|(confidence, beneficial)| *confidence > 0.8 && *beneficial)
        .count();

    println!(
        "High confidence decisions: {}/{} ({:.1}%)",
        high_confidence_decisions,
        decision_history.len(),
        (high_confidence_decisions as f64 / decision_history.len() as f64) * 100.0
    );

    if high_confidence_decisions > 0 {
        println!(
            "High confidence accuracy: {:.1}%",
            (beneficial_high_confidence as f64 / high_confidence_decisions as f64) * 100.0
        );
    }
}

fn demonstrate_realtime_adaptation() {
    println!("\n--- Real-time Adaptation Demo ---");

    let config = MLReplacementConfig::reinforcement_learning();
    let mut policy = MLReplacementPolicy::new(config);

    println!("Demonstrating real-time learning adaptation...");

    // Simulate changing game conditions
    let phases = [("Opening", 0.3), ("Middlegame", 0.6), ("Endgame", 0.9)];

    for (phase_name, complexity) in phases {
        println!(
            "Adapting to {} phase (complexity: {:.1})",
            phase_name, complexity
        );

        let mut phase_decisions = 0;
        let mut phase_correct = 0;

        for i in 0..15 {
            let context = create_context_with_complexity(complexity);
            let decision = policy.decide_replacement(&context);

            // Simulate outcome based on complexity and decision quality
            let expected_decision = if complexity > 0.7 {
                ReplacementAction::KeepExisting // Prefer stability in endgame
            } else {
                ReplacementAction::ReplaceWithNew // More exploration in opening
            };

            let beneficial = decision.decision == expected_decision;
            if beneficial {
                phase_correct += 1;
            }

            let outcome = create_sample_outcome(beneficial);
            policy.record_outcome(&decision, outcome);

            phase_decisions += 1;

            if i % 5 == 0 {
                println!(
                    "  {} decision: {:?} (expected: {:?}) -> {}",
                    phase_name,
                    decision.decision,
                    expected_decision,
                    if beneficial { "Correct" } else { "Incorrect" }
                );
            }
        }

        let phase_accuracy = phase_correct as f64 / phase_decisions as f64;
        println!("  {} accuracy: {:.1}%", phase_name, phase_accuracy * 100.0);

        let stats = policy.get_performance_stats();
        println!(
            "  Overall accuracy: {:.1}%",
            (stats.correct_decisions as f64 / stats.total_decisions as f64) * 100.0
        );
    }

    println!("Adaptation complete - policy learned phase-specific behavior");
}

// Helper functions to create sample data
fn create_sample_context() -> MLReplacementContext {
    MLReplacementContext {
        current_hash: 0x123456789ABCDEF0,
        existing_entry: Some(TranspositionEntry::new(
            100,
            5,
            TranspositionFlag::Exact,
            None,
            0x123456789ABCDEF0,
            10,
            EntrySource::MainSearch,
        )),
        new_entry: TranspositionEntry::new(
            120,
            6,
            TranspositionFlag::LowerBound,
            None,
            0x123456789ABCDEF0,
            1,
            EntrySource::MainSearch,
        ),
        access_pattern: AccessPatternInfo {
            recent_frequency: 0.7,
            depth_pattern: 0.5,
            sibling_access: 0.3,
            parent_access: 0.2,
        },
        position_features: PositionFeatures {
            complexity: 0.6,
            tactical_score: 0.4,
            positional_score: 0.8,
            material_balance: 0.1,
        },
        temporal_info: TemporalInfo {
            timestamp: std::time::Instant::now(),
            existing_age: std::time::Duration::from_secs(30),
            time_since_access: std::time::Duration::from_secs(5),
        },
    }
}

fn create_simple_context() -> MLReplacementContext {
    let mut context = create_sample_context();
    context.position_features.complexity = 0.2;
    context.position_features.tactical_score = 0.1;
    context.access_pattern.recent_frequency = 0.3;
    context
}

fn create_complex_context() -> MLReplacementContext {
    let mut context = create_sample_context();
    context.position_features.complexity = 0.9;
    context.position_features.tactical_score = 0.8;
    context.access_pattern.recent_frequency = 0.9;
    context
}

fn create_endgame_context() -> MLReplacementContext {
    let mut context = create_sample_context();
    context.position_features.material_balance = 0.9;
    context.position_features.tactical_score = 0.2;
    context.position_features.positional_score = 0.1;
    context
}

fn create_opening_context() -> MLReplacementContext {
    let mut context = create_sample_context();
    context.position_features.complexity = 0.4;
    context.position_features.positional_score = 0.6;
    context.access_pattern.depth_pattern = 0.8;
    context
}

fn create_varying_context(decision_num: usize) -> MLReplacementContext {
    let mut context = create_sample_context();

    // Vary features based on decision number
    context.position_features.complexity = 0.3 + (decision_num as f64 * 0.05) % 0.7;
    context.access_pattern.recent_frequency = 0.2 + (decision_num as f64 * 0.03) % 0.8;
    context.temporal_info.existing_age = std::time::Duration::from_secs(5 + decision_num as u64);

    context
}

fn create_context_with_complexity(complexity: f64) -> MLReplacementContext {
    let mut context = create_sample_context();
    context.position_features.complexity = complexity;
    context.position_features.tactical_score = complexity * 0.8;
    context.access_pattern.recent_frequency = complexity;
    context
}

fn create_sample_outcome(beneficial: bool) -> ReplacementOutcome {
    ReplacementOutcome {
        beneficial,
        performance_impact: if beneficial { 0.1 } else { -0.05 },
        cache_efficiency: if beneficial { 0.85 } else { 0.75 },
        access_frequency: if beneficial { 0.7 } else { 0.4 },
    }
}
