//! Predictive Prefetching Example
//!
//! This example demonstrates how to use the predictive prefetching system
//! to improve transposition table performance by predicting and prefetching
//! likely next accesses.

use shogi_engine::search::{PredictivePrefetcher, PrefetchConfig};
use shogi_engine::types::{Move, Piece, PieceType, Player, Position};

fn main() {
    println!("Predictive Prefetching Example");
    println!("==============================");

    // Demonstrate different prefetching strategies
    demonstrate_prefetching_strategies();

    // Demonstrate learning and adaptation
    demonstrate_learning_adaptation();

    // Demonstrate performance benefits
    demonstrate_performance_benefits();

    // Demonstrate pattern recognition
    demonstrate_pattern_recognition();

    // Demonstrate cache efficiency
    demonstrate_cache_efficiency();

    println!("\nPredictive Prefetching Example completed successfully!");
}

fn demonstrate_prefetching_strategies() {
    println!("\n--- Prefetching Strategies Demo ---");

    let strategies = [
        ("Move-Based", PrefetchConfig::move_based()),
        ("Depth-Based", PrefetchConfig::depth_based()),
        ("Pattern-Based", PrefetchConfig::pattern_based()),
        ("Hybrid", PrefetchConfig::hybrid()),
        ("Adaptive", PrefetchConfig::adaptive()),
    ];

    let current_hash = 0x123456789ABCDEF0;
    let current_move = create_sample_move();

    for (name, config) in strategies {
        let mut prefetcher = PredictivePrefetcher::new(config);

        let start_time = std::time::Instant::now();
        let prediction = prefetcher.predict_next_accesses(current_hash, Some(current_move.clone()));
        let prediction_time = start_time.elapsed().as_micros();

        println!(
            "{}: {} predictions in {}μs, avg confidence {:.2}",
            name,
            prediction.predicted_hashes.len(),
            prediction_time,
            prediction.confidence_scores.iter().sum::<f64>()
                / prediction.confidence_scores.len() as f64
        );

        // Demonstrate prefetching
        for &predicted_hash in &prediction.predicted_hashes {
            prefetcher.prefetch_entry(predicted_hash);
        }

        let stats = prefetcher.get_stats();
        println!(
            "  Prefetches: {}, Strategy: {:?}",
            stats.total_prefetches, prediction.strategy_used
        );
    }
}

fn demonstrate_learning_adaptation() {
    println!("\n--- Learning and Adaptation Demo ---");

    let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::adaptive());

    // Simulate a game sequence with predictable patterns
    let game_sequence = vec![
        (0x1111, create_pawn_move()),
        (0x2222, create_knight_move()),
        (0x3333, create_bishop_move()),
        (0x4444, create_pawn_move()),
        (0x5555, create_knight_move()),
    ];

    println!("Simulating game sequence with predictable patterns...");

    // First pass: record accesses without prefetching
    for (hash, move_) in &game_sequence {
        prefetcher.record_access(*hash, false, Some(move_.clone()));
    }

    // Update learning
    prefetcher.update_learning();

    let initial_stats = prefetcher.get_stats().clone();
    println!(
        "Initial stats: {} predictions recorded, avg hit rate {:.2}",
        initial_stats.total_predictions, initial_stats.avg_hit_rate
    );

    // Second pass: use predictions
    let mut successful_predictions = 0;
    for (hash, move_) in &game_sequence {
        let prediction = prefetcher.predict_next_accesses(*hash, Some(move_.clone()));

        // Simulate prefetching and record results
        for &predicted_hash in &prediction.predicted_hashes {
            prefetcher.prefetch_entry(predicted_hash);

            // Simulate hit if prediction was correct
            let was_hit = game_sequence.iter().any(|(h, _)| *h == predicted_hash);
            if was_hit {
                successful_predictions += 1;
            }

            prefetcher.record_access(predicted_hash, was_hit, None);
        }
    }

    let final_stats = prefetcher.get_stats();
    if final_stats.total_predictions > 0 {
        println!(
            "Final stats: {} predictions, {:.1}% accuracy",
            final_stats.total_predictions,
            (successful_predictions as f64 / final_stats.total_predictions as f64) * 100.0
        );
    } else {
        println!("Final stats: no predictions recorded yet");
    }

    println!(
        "Hit rate improved from {:.2} to {:.2}",
        initial_stats.avg_hit_rate, final_stats.avg_hit_rate
    );
}

fn demonstrate_performance_benefits() {
    println!("\n--- Performance Benefits Demo ---");

    let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::hybrid());

    // Simulate intensive search with prefetching
    let search_hashes = (0..100).map(|i| 0x1000 + i).collect::<Vec<_>>();

    println!(
        "Simulating intensive search with {} positions...",
        search_hashes.len()
    );

    let start_time = std::time::Instant::now();

    for (i, &hash) in search_hashes.iter().enumerate() {
        let move_info = if i % 3 == 0 {
            Some(create_sample_move())
        } else {
            None
        };

        // Predict and prefetch
        let prediction = prefetcher.predict_next_accesses(hash, move_info.clone());

        for &predicted_hash in &prediction.predicted_hashes {
            prefetcher.prefetch_entry(predicted_hash);
        }

        // Simulate actual access (with some hits)
        let was_prefetched = i % 4 == 0; // 25% hit rate simulation
        prefetcher.record_access(hash, was_prefetched, move_info);
    }

    let total_time = start_time.elapsed();

    let stats = prefetcher.get_stats();
    println!("Total time: {:?}", total_time);
    println!(
        "Predictions: {}, Prefetches: {}",
        stats.total_predictions, stats.total_prefetches
    );
    println!("Hit rate: {:.1}%", stats.avg_hit_rate * 100.0);
    println!("Avg prediction time: {:.1}μs", stats.avg_prediction_time_us);

    // Calculate time saved (simulated)
    let time_per_access = 10; // microseconds per cache miss
    let time_saved = stats.prefetch_hits * time_per_access;
    println!(
        "Estimated time saved: {}μs ({:.1}% improvement)",
        time_saved,
        (time_saved as f64 / total_time.as_micros() as f64) * 100.0
    );
}

fn demonstrate_pattern_recognition() {
    println!("\n--- Pattern Recognition Demo ---");

    let mut prefetcher = PredictivePrefetcher::new(PrefetchConfig::pattern_based());

    // Create a repeating pattern
    let pattern1 = vec![0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD];
    let pattern2 = vec![0x1111, 0x2222, 0x3333];

    let moves = vec![create_sample_move(); 10];

    println!("Creating repeating patterns...");

    // Repeat patterns multiple times
    for repetition in 0..3 {
        println!("Repetition {}", repetition + 1);

        for (hash, move_) in pattern1.iter().zip(moves.iter()) {
            prefetcher.record_access(*hash, false, Some(move_.clone()));
        }

        for (hash, move_) in pattern2.iter().zip(moves.iter()) {
            prefetcher.record_access(*hash, false, Some(move_.clone()));
        }

        // Update learning after each repetition
        prefetcher.update_learning();
    }

    let stats_after_training = prefetcher.get_stats().clone();
    println!(
        "Patterns analyzed across predictions: {} (avg hit rate {:.2})",
        stats_after_training.total_predictions, stats_after_training.avg_hit_rate
    );

    // Test pattern recognition
    let test_hash = 0xAAAA;
    let prediction = prefetcher.predict_next_accesses(test_hash, Some(create_sample_move()));

    println!(
        "Prediction for {}: {} hashes predicted",
        test_hash,
        prediction.predicted_hashes.len()
    );

    // Check if predicted hashes match the pattern
    let pattern_matches = prediction
        .predicted_hashes
        .iter()
        .filter(|&&h| pattern1.contains(&h))
        .count();

    if !prediction.predicted_hashes.is_empty() {
        println!(
            "Pattern matches: {}/{}",
            pattern_matches,
            prediction.predicted_hashes.len()
        );
    } else {
        println!("Pattern matches: 0/0 (no predictions yet)");
    }
}

fn demonstrate_cache_efficiency() {
    println!("\n--- Cache Efficiency Demo ---");

    let config = PrefetchConfig::move_based();
    let cache_capacity = config.prediction_cache_size;
    let mut prefetcher = PredictivePrefetcher::new(config.clone());

    // Test cache hit rates with repeated predictions
    let test_hashes = vec![0x1111, 0x2222, 0x3333, 0x4444, 0x5555];

    println!(
        "Testing cache efficiency with {} test hashes...",
        test_hashes.len()
    );

    let mut first_pass_metadata = Vec::new();
    for &hash in &test_hashes {
        let prediction = prefetcher.predict_next_accesses(hash, Some(create_sample_move()));
        println!(
            "First prediction for {:X}: {} results (cache hit rate {:.1}%)",
            hash,
            prediction.predicted_hashes.len(),
            prediction.metadata.cache_hit_rate * 100.0
        );
        first_pass_metadata.push(prediction.metadata);
    }

    // Second pass: should hit cache
    for &hash in &test_hashes {
        let start_time = std::time::Instant::now();
        let prediction = prefetcher.predict_next_accesses(hash, Some(create_sample_move()));
        let prediction_time = start_time.elapsed().as_micros();

        println!(
            "Cached prediction for {:X}: {}μs (cache hit rate {:.1}%)",
            hash,
            prediction_time,
            prediction.metadata.cache_hit_rate * 100.0
        );
    }

    // Third pass: test cache utilization
    let average_cache_hit_rate = prefetcher.get_stats().avg_hit_rate.max(0.0);
    println!(
        "Average cache hit rate across predictions: {:.1}%",
        average_cache_hit_rate * 100.0
    );
    println!("Configured cache capacity: {} entries", cache_capacity);

    // Test cache eviction
    println!("\nTesting cache eviction...");
    let large_sequence: Vec<u64> = (0..200).map(|i| 0x10000 + i).collect();

    for &hash in &large_sequence {
        let _prediction = prefetcher.predict_next_accesses(hash, Some(create_sample_move()));
    }

    let stats = prefetcher.get_stats();
    println!(
        "Cache memory overhead: {} bytes, total predictions: {}",
        stats.memory_overhead_bytes, stats.total_predictions
    );
}

fn create_sample_move() -> Move {
    Move {
        from: Some(Position::from_u8(15)),
        to: Position::from_u8(25),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: true,
        is_capture: false,
        captured_piece: None,
        gives_check: true,
        is_recapture: false,
    }
}

fn create_pawn_move() -> Move {
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

fn create_knight_move() -> Move {
    Move {
        from: Some(Position::from_u8(11)),
        to: Position::from_u8(21),
        piece_type: PieceType::Knight,
        player: Player::Black,
        is_promotion: false,
        is_capture: true,
        captured_piece: Some(Piece {
            piece_type: PieceType::Pawn,
            player: Player::White,
        }),
        gives_check: false,
        is_recapture: false,
    }
}

fn create_bishop_move() -> Move {
    Move {
        from: Some(Position::from_u8(12)),
        to: Position::from_u8(22),
        piece_type: PieceType::Bishop,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: true,
        is_recapture: false,
    }
}
