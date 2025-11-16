//! Integration tests for TranspositionTableTrait
//!
//! Tests for Task 3.0 - Task 3.30: Verify transposition table trait works with all table implementations

use shogi_engine::search::transposition_table_config::create_transposition_table;
use shogi_engine::search::transposition_table_config::TranspositionTableConfig;
use shogi_engine::search::TranspositionConfig;
use shogi_engine::types::*;

/// Create a test transposition entry
fn create_test_entry(hash_key: u64, depth: u8, score: i32) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        TranspositionFlag::Exact,
        None,
        hash_key,
        0,
        EntrySource::MainSearch,
    )
}

#[test]
fn test_basic_transposition_table_trait() {
    // Test that basic TranspositionTable works with the trait
    let table_config = TranspositionTableConfig::Basic {
        config: shogi_engine::search::transposition_table::TranspositionTableConfig {
            max_entries: 1024,
            replacement_policy: shogi_engine::search::transposition_table::ReplacementPolicy::DepthPreferred,
            track_memory: false,
            track_statistics: false,
        },
    };

    let mut table = create_transposition_table(table_config);

    // Test probe, store, clear, size
    let entry = create_test_entry(0x1234, 5, 100);
    table.store(entry.clone());

    let retrieved = table.probe(0x1234, 5);
    assert!(retrieved.is_some(), "Should retrieve stored entry");
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    assert!(table.size() > 0, "Table should have a size");

    table.clear();
    let after_clear = table.probe(0x1234, 5);
    assert!(after_clear.is_none(), "Entry should be cleared");
}

#[test]
fn test_thread_safe_transposition_table_trait() {
    // Test that ThreadSafeTranspositionTable works with the trait
    let config = TranspositionConfig {
        table_size: 1024,
        enable_statistics: true,
        ..TranspositionConfig::default()
    };

    let table_config = TranspositionTableConfig::ThreadSafe { config };
    let mut table = create_transposition_table(table_config);

    // Test probe, store, clear, size
    let entry = create_test_entry(0x5678, 6, 200);
    table.store(entry.clone());

    let retrieved = table.probe(0x5678, 6);
    assert!(retrieved.is_some(), "Should retrieve stored entry");
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test hit_rate (ThreadSafeTranspositionTable tracks statistics)
    let hit_rate = table.hit_rate();
    assert!(hit_rate >= 0.0 && hit_rate <= 100.0, "Hit rate should be in valid range");

    assert!(table.size() > 0, "Table should have a size");

    table.clear();
    let after_clear = table.probe(0x5678, 6);
    assert!(after_clear.is_none(), "Entry should be cleared");
}

#[test]
fn test_probe_with_prefetch() {
    // Test that probe_with_prefetch works (default implementation just calls probe)
    let config = TranspositionConfig {
        table_size: 1024,
        ..TranspositionConfig::default()
    };

    let table_config = TranspositionTableConfig::ThreadSafe { config };
    let mut table = create_transposition_table(table_config);

    let entry = create_test_entry(0xABCD, 7, 300);
    table.store(entry.clone());

    // Test probe_with_prefetch with next_hash hint
    let retrieved = table.probe_with_prefetch(0xABCD, 7, Some(0xEF00));
    assert!(retrieved.is_some(), "Should retrieve stored entry with prefetch");
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);

    // Test probe_with_prefetch without next_hash hint
    let retrieved2 = table.probe_with_prefetch(0xABCD, 7, None);
    assert!(retrieved2.is_some(), "Should retrieve stored entry without prefetch");
}

#[test]
fn test_prefill_from_book() {
    // Test that prefill_from_book works (if supported by implementation)
    let config = TranspositionConfig {
        table_size: 1024,
        ..TranspositionConfig::default()
    };

    let table_config = TranspositionTableConfig::ThreadSafe { config };
    let mut table = create_transposition_table(table_config);

    // Create a minimal opening book for testing
    use shogi_engine::opening_book::{BookMove, OpeningBookBuilder};
    let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    let book_move = BookMove::new(
        Some(shogi_engine::types::core::Position::new(6, 4)),
        shogi_engine::types::core::Position::new(5, 4),
        shogi_engine::types::core::PieceType::Pawn,
        false,
        false,
        1000,
        60,
    );

    let mut book = OpeningBookBuilder::new()
        .add_position(fen.to_string(), vec![book_move])
        .mark_loaded()
        .build();

    // Test prefill_from_book
    let inserted = table.prefill_from_book(&mut book, 4);
    assert!(inserted > 0, "Should insert entries from opening book");
}

#[test]
fn test_trait_polymorphism() {
    // Test that we can use different table types polymorphically via the trait
    let test_configs = vec![
        ("Basic", TranspositionTableConfig::Basic {
            config: shogi_engine::search::transposition_table::TranspositionTableConfig {
                max_entries: 512,
                replacement_policy: shogi_engine::search::transposition_table::ReplacementPolicy::DepthPreferred,
                track_memory: false,
                track_statistics: false,
            },
        }),
        ("ThreadSafe", TranspositionTableConfig::ThreadSafe {
            config: TranspositionConfig {
                table_size: 512,
                ..TranspositionConfig::default()
            },
        }),
    ];

    for (name, config) in test_configs {
        let mut table = create_transposition_table(config);

        // Test basic operations work for all implementations
        let entry = create_test_entry(0x1111, 3, 50);
        table.store(entry.clone());

        let retrieved = table.probe(0x1111, 3);
        assert!(
            retrieved.is_some(),
            "{} table should retrieve stored entry",
            name
        );
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(
            retrieved_entry.score, entry.score,
            "{} table should return correct score",
            name
        );

        assert!(table.size() > 0, "{} table should have a size", name);

        table.clear();
        let after_clear = table.probe(0x1111, 3);
        assert!(
            after_clear.is_none(),
            "{} table should clear entries",
            name
        );
    }
}

#[test]
fn test_trait_multiple_entries() {
    // Test storing and retrieving multiple entries
    let config = TranspositionConfig {
        table_size: 1024,
        ..TranspositionConfig::default()
    };

    let table_config = TranspositionTableConfig::ThreadSafe { config };
    let mut table = create_transposition_table(table_config);

    // Store multiple entries
    let entries = vec![
        create_test_entry(0x1000, 3, 100),
        create_test_entry(0x2000, 4, 200),
        create_test_entry(0x3000, 5, 300),
    ];

    for entry in &entries {
        table.store(entry.clone());
    }

    // Retrieve all entries
    for entry in &entries {
        let retrieved = table.probe(entry.hash_key, entry.depth);
        assert!(
            retrieved.is_some(),
            "Should retrieve entry with hash 0x{:X}",
            entry.hash_key
        );
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.score, entry.score);
        assert_eq!(retrieved_entry.depth, entry.depth);
    }
}

#[test]
fn test_trait_depth_requirement() {
    // Test that probe respects depth requirement
    let config = TranspositionConfig {
        table_size: 1024,
        ..TranspositionConfig::default()
    };

    let table_config = TranspositionTableConfig::ThreadSafe { config };
    let mut table = create_transposition_table(table_config);

    // Store entry with depth 5
    let entry = create_test_entry(0x4000, 5, 400);
    table.store(entry);

    // Probe with depth 5 should succeed
    let retrieved = table.probe(0x4000, 5);
    assert!(retrieved.is_some(), "Should retrieve entry at same depth");

    // Probe with depth 6 (requires deeper entry) should fail
    let retrieved_deeper = table.probe(0x4000, 6);
    assert!(
        retrieved_deeper.is_none(),
        "Should not retrieve entry when requiring deeper depth"
    );

    // Probe with depth 4 (shallower requirement) should succeed
    let retrieved_shallow = table.probe(0x4000, 4);
    assert!(
        retrieved_shallow.is_some(),
        "Should retrieve entry when requiring shallower depth"
    );
}

#[test]
fn test_trait_all_implementations() {
    // Test that all table implementations work with the trait
    // This verifies that the trait abstraction works correctly

    // Test configurations for all supported table types
    let configs = vec![
        ("Basic", TranspositionTableConfig::Basic {
            config: shogi_engine::search::transposition_table::TranspositionTableConfig {
                max_entries: 256,
                replacement_policy: shogi_engine::search::transposition_table::ReplacementPolicy::DepthPreferred,
                track_memory: false,
                track_statistics: false,
            },
        }),
        ("ThreadSafe", TranspositionTableConfig::ThreadSafe {
            config: TranspositionConfig {
                table_size: 256,
                enable_statistics: true,
                ..TranspositionConfig::default()
            },
        }),
    ];

    for (name, config) in configs {
        let mut table = create_transposition_table(config);

        // Test all trait methods
        let entry = create_test_entry(0x9999, 8, 999);
        table.store(entry.clone());

        // probe
        let retrieved = table.probe(0x9999, 8);
        assert!(retrieved.is_some(), "{}: probe should work", name);

        // probe_with_prefetch
        let retrieved_prefetch = table.probe_with_prefetch(0x9999, 8, Some(0xAAAA));
        assert!(
            retrieved_prefetch.is_some(),
            "{}: probe_with_prefetch should work",
            name
        );

        // size
        let size = table.size();
        assert!(size > 0, "{}: size should return positive value", name);

        // hit_rate (may return 0.0 for implementations without statistics)
        let hit_rate = table.hit_rate();
        assert!(
            hit_rate >= 0.0 && hit_rate <= 100.0,
            "{}: hit_rate should be in valid range",
            name
        );

        // clear
        table.clear();
        let after_clear = table.probe(0x9999, 8);
        assert!(
            after_clear.is_none(),
            "{}: clear should remove entries",
            name
        );
    }
}

