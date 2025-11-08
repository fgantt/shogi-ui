//! Advanced Cache Warming Example
//!
//! This example demonstrates how to use advanced cache warming to preload
//! transposition tables with relevant entries to improve initial performance.

use shogi_engine::search::advanced_cache_warming::{
    GamePhase, TacticalPattern, TacticalPatternType, TranspositionTableInterface,
};
use shogi_engine::search::{
    AdvancedCacheWarmer, CacheWarmingConfig, PositionAnalysis, WarmingEntry, WarmingEntryType,
    WarmingResults, WarmingSession, WarmingStrategy,
};
use shogi_engine::types::{
    Move, PieceType, Player, Position, TranspositionEntry, TranspositionFlag,
};

fn main() {
    println!("Advanced Cache Warming Example");
    println!("=============================");

    // Demonstrate different warming strategies
    demonstrate_warming_strategies();

    // Demonstrate warming sessions
    demonstrate_warming_sessions();

    // Demonstrate position-based warming
    demonstrate_position_based_warming();

    // Demonstrate warming with different entry types
    demonstrate_entry_types();

    // Demonstrate warming performance
    demonstrate_warming_performance();

    println!("\nAdvanced Cache Warming Example completed successfully!");
}

fn demonstrate_warming_strategies() {
    println!("\n--- Warming Strategies Demo ---");

    let strategies = [
        ("Conservative", CacheWarmingConfig::conservative()),
        ("Aggressive", CacheWarmingConfig::aggressive()),
        ("Selective", CacheWarmingConfig::selective()),
        ("Adaptive", CacheWarmingConfig::adaptive()),
    ];

    for (name, config) in strategies {
        let mut warmer = AdvancedCacheWarmer::new(config);
        let mut mock_table = MockTranspositionTable::new(10000);

        let config_snapshot = warmer.get_config().clone();

        println!("{} Strategy:", name);
        println!("  Max entries: {}", config_snapshot.max_warm_entries);
        println!("  Timeout: {:?}", config_snapshot.warming_timeout);
        println!("  Aggressiveness: {:.1}", config_snapshot.aggressiveness);
        println!("  Memory limit: {} bytes", config_snapshot.memory_limit);

        let session = warmer.start_warming_session(Some(0x1234));
        let results = warmer.warm_cache(session.session_id, &mut mock_table);

        println!(
            "  Results: {} entries warmed in {}μs",
            results.total_entries, results.warming_time_us
        );
        println!("  Success rate: {:.1}%", results.success_rate * 100.0);
        println!("  Memory used: {} bytes", results.memory_used);
        println!("  Position entries: {}", results.position_entries);
        println!("  Opening entries: {}", results.opening_entries);
        println!("  Tactical entries: {}", results.tactical_entries);
    }
}

fn demonstrate_warming_sessions() {
    println!("\n--- Warming Sessions Demo ---");

    let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::selective());
    let mut mock_table = MockTranspositionTable::new(5000);

    println!("Performing multiple warming sessions...");

    // Session 1: Initial warming
    let session1 = warmer.start_warming_session(Some(0x1000));
    let results1 = warmer.warm_cache(session1.session_id, &mut mock_table);
    println!(
        "Session 1: {} entries warmed, {:.1}% success rate",
        results1.total_entries,
        results1.success_rate * 100.0
    );

    // Session 2: Additional warming
    let session2 = warmer.start_warming_session(Some(0x2000));
    let results2 = warmer.warm_cache(session2.session_id, &mut mock_table);
    println!(
        "Session 2: {} entries warmed, {:.1}% success rate",
        results2.total_entries,
        results2.success_rate * 100.0
    );

    // Session 3: Final warming
    let session3 = warmer.start_warming_session(Some(0x3000));
    let results3 = warmer.warm_cache(session3.session_id, &mut mock_table);
    println!(
        "Session 3: {} entries warmed, {:.1}% success rate",
        results3.total_entries,
        results3.success_rate * 100.0
    );

    // Show session history
    let recent_sessions = warmer.get_recent_sessions(3);
    println!("Recent sessions:");
    for session in recent_sessions {
        println!(
            "  Session {}: {} entries, {:?} strategy",
            session.session_id, session.entries_warmed, session.strategy
        );
    }

    // Show overall statistics
    let stats = warmer.get_stats();
    println!("Overall statistics:");
    println!("  Total sessions: {}", stats.total_sessions);
    println!("  Total entries warmed: {}", stats.total_entries_warmed);
    println!("  Average warming time: {:.1}μs", stats.avg_warming_time_us);
    println!(
        "  Average success rate: {:.1}%",
        stats.avg_success_rate * 100.0
    );
    println!(
        "  Warming efficiency: {:.1} entries/ms",
        stats.warming_efficiency
    );
}

fn demonstrate_position_based_warming() {
    println!("\n--- Position-Based Warming Demo ---");

    let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::selective());
    let mut mock_table = MockTranspositionTable::new(3000);

    // Add some position analyses
    let positions = vec![
        (0x1000, GamePhase::Opening, 0.3, 0.4),
        (0x2000, GamePhase::Middlegame, 0.7, 0.6),
        (0x3000, GamePhase::Endgame, 0.5, 0.8),
    ];

    for (hash, phase, tactical, positional) in positions {
        let analysis = PositionAnalysis {
            position_hash: hash,
            game_phase: phase,
            tactical_complexity: tactical,
            positional_complexity: positional,
            material_balance: 0.1,
            king_safety: 0.8,
            mobility: 0.6,
        };

        warmer.add_position(hash, analysis);
        println!(
            "Added position analysis for {:X} (phase: {:?})",
            hash, phase
        );
    }

    // Perform position-based warming
    let session = warmer.start_warming_session(Some(0x1000));
    let results = warmer.warm_cache(session.session_id, &mut mock_table);

    println!("Position-based warming results:");
    println!("  Total entries: {}", results.total_entries);
    println!("  Position entries: {}", results.position_entries);
    println!("  Warming time: {}μs", results.warming_time_us);
    println!("  Success rate: {:.1}%", results.success_rate * 100.0);
}

fn demonstrate_entry_types() {
    println!("\n--- Entry Types Demo ---");

    let config = CacheWarmingConfig::aggressive();
    let mut warmer = AdvancedCacheWarmer::new(config);
    let mut mock_table = MockTranspositionTable::new(8000);

    // Add different types of entries
    println!("Adding specialized entries...");

    // Opening book entries
    for i in 0..5 {
        let entry = WarmingEntry {
            hash_key: 0x5000 + i,
            depth: 2 + (i % 3) as u8,
            score: 20 + (i * 10) as i32,
            flag: TranspositionFlag::Exact,
            best_move: Some(create_sample_move()),
            priority: 0.9 - (i as f64 * 0.1),
            entry_type: WarmingEntryType::OpeningBook,
        };
        warmer.add_opening_entry(0x5000 + i, entry);
    }
    println!("  Added {} opening book entries", 5);

    // Endgame entries
    for i in 0..5 {
        let entry = WarmingEntry {
            hash_key: 0x6000 + i,
            depth: 10 + (i % 5) as u8,
            score: -100 + (i * 50) as i32,
            flag: TranspositionFlag::Exact,
            best_move: if i % 2 == 0 {
                Some(create_sample_move())
            } else {
                None
            },
            priority: 0.8 - (i as f64 * 0.1),
            entry_type: WarmingEntryType::Endgame,
        };
        warmer.add_endgame_entry(0x6000 + i, entry);
    }
    println!("  Added {} endgame entries", 5);

    // Tactical patterns
    let pattern_types = [
        TacticalPatternType::Fork,
        TacticalPatternType::Pin,
        TacticalPatternType::Skewer,
        TacticalPatternType::DiscoveredAttack,
    ];

    for (i, pattern_type) in pattern_types.iter().enumerate() {
        let pattern = TacticalPattern {
            pattern_hash: 0x7000 + i as u64,
            pattern_type: *pattern_type,
            frequency: 0.7 + (i as f64 * 0.05),
            success_rate: 0.6 + (i as f64 * 0.1),
            entries: vec![],
        };
        warmer.add_tactical_pattern(pattern);
    }
    println!("  Added {} tactical patterns", pattern_types.len());

    // Perform warming with all entry types
    let session = warmer.start_warming_session(Some(0x1000));
    let results = warmer.warm_cache(session.session_id, &mut mock_table);

    println!("Multi-type warming results:");
    println!("  Total entries: {}", results.total_entries);
    println!("  Position entries: {}", results.position_entries);
    println!("  Opening entries: {}", results.opening_entries);
    println!("  Endgame entries: {}", results.endgame_entries);
    println!("  Tactical entries: {}", results.tactical_entries);
    println!("  Warming time: {}μs", results.warming_time_us);
    println!("  Success rate: {:.1}%", results.success_rate * 100.0);
}

fn demonstrate_warming_performance() {
    println!("\n--- Warming Performance Demo ---");

    let configs = [
        ("Conservative", CacheWarmingConfig::conservative()),
        ("Selective", CacheWarmingConfig::selective()),
        ("Aggressive", CacheWarmingConfig::aggressive()),
    ];

    for (name, config) in configs {
        let mut warmer = AdvancedCacheWarmer::new(config);
        let mut mock_table = MockTranspositionTable::new(15000);

        println!("{} Performance Test:", name);

        // Measure warming performance
        let start_time = std::time::Instant::now();
        let session = warmer.start_warming_session(Some(0x1234));
        let results = warmer.warm_cache(session.session_id, &mut mock_table);
        let total_time = start_time.elapsed();

        // Calculate performance metrics
        let entries_per_ms = results.total_entries as f64 / total_time.as_millis() as f64;
        let memory_efficiency =
            results.total_entries as f64 / (results.memory_used as f64 / 1024.0);
        let time_efficiency =
            results.total_entries as f64 / (results.warming_time_us as f64 / 1000.0);

        println!("  Entries warmed: {}", results.total_entries);
        println!("  Total time: {:?}", total_time);
        println!("  Warming time: {}μs", results.warming_time_us);
        println!("  Entries per ms: {:.1}", entries_per_ms);
        println!("  Memory efficiency: {:.1} entries/KB", memory_efficiency);
        println!("  Time efficiency: {:.1} entries/ms", time_efficiency);
        println!("  Success rate: {:.1}%", results.success_rate * 100.0);

        // Test cache hit rate after warming
        let hit_rate = test_cache_hit_rate(&mock_table);
        println!("  Post-warming hit rate: {:.1}%", hit_rate * 100.0);
    }
}

// Helper functions and mock implementations
struct MockTranspositionTable {
    entries: std::collections::HashMap<u64, TranspositionEntry>,
    max_size: usize,
}

impl MockTranspositionTable {
    fn new(max_size: usize) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            max_size,
        }
    }
}

impl TranspositionTableInterface for MockTranspositionTable {
    fn store(&mut self, entry: TranspositionEntry) -> bool {
        if self.entries.len() < self.max_size {
            self.entries.insert(entry.hash_key, entry);
            true
        } else {
            false
        }
    }

    fn size(&self) -> usize {
        self.entries.len()
    }

    fn memory_usage(&self) -> u64 {
        self.entries.len() as u64 * std::mem::size_of::<TranspositionEntry>() as u64
    }
}

fn test_cache_hit_rate(table: &MockTranspositionTable) -> f64 {
    // Simulate some cache lookups
    let mut hits = 0;
    let total_lookups = 100;

    for i in 0..total_lookups {
        let hash = 0x1000 + (i % table.size() as u64);
        if table.entries.contains_key(&hash) {
            hits += 1;
        }
    }

    hits as f64 / total_lookups as f64
}

fn create_sample_move() -> Move {
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
