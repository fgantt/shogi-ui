//! Integration tests for TranspositionTableTrait
//!
//! Tests for Task 3.0 - Task 3.30: Verify transposition table trait works with all table implementations

use shogi_engine::search::transposition_config::TranspositionConfig;
use shogi_engine::search::transposition_table_config::{
    create_transposition_table, TranspositionTableConfig,
};
use shogi_engine::search::transposition_table_trait::TranspositionTableTrait;
use shogi_engine::types::search::TranspositionFlag;
use shogi_engine::types::transposition::TranspositionEntry;
use std::cell::RefCell;

fn create_test_entry(hash_key: u64, depth: u8, score: i32) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        TranspositionFlag::Exact,
        None,
        hash_key,
        0,
        shogi_engine::types::search::EntrySource::MainSearch,
    )
}

#[test]
fn test_basic_table_trait() {
    // Test basic TranspositionTable wrapped in RefCell
    let config = shogi_engine::search::transposition_table::TranspositionTableConfig::default();
    let tt_config = TranspositionTableConfig::basic(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0x1234, 5, 100);

    // Test store
    table.store(entry.clone());

    // Test probe
    let retrieved = table.probe(0x1234, 5);
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test size
    let size = table.size();
    assert!(size > 0);

    // Test clear
    table.clear();
    let after_clear = table.probe(0x1234, 5);
    assert!(after_clear.is_none());

    // Test hit_rate (default implementation returns 0.0)
    let hit_rate = table.hit_rate();
    assert_eq!(hit_rate, 0.0);
}

#[test]
fn test_thread_safe_table_trait() {
    // Test ThreadSafeTranspositionTable
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0x5678, 6, 200);

    // Test store
    table.store(entry.clone());

    // Test probe
    let retrieved = table.probe(0x5678, 6);
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test probe_with_prefetch
    let prefetch_retrieved = table.probe_with_prefetch(0x5678, 6, Some(0x9ABC));
    assert!(prefetch_retrieved.is_some());

    // Test size
    let size = table.size();
    assert!(size > 0);

    // Test hit_rate (ThreadSafeTranspositionTable tracks statistics)
    let hit_rate = table.hit_rate();
    // Hit rate should be calculated (may be 0.0 if statistics not enabled, but method should work)
    assert!(hit_rate >= 0.0 && hit_rate <= 100.0);

    // Test clear
    table.clear();
    let after_clear = table.probe(0x5678, 6);
    assert!(after_clear.is_none());
}

#[test]
fn test_multi_level_table_trait() {
    // Test MultiLevelTranspositionTable
    let config = shogi_engine::search::multi_level_transposition_table::MultiLevelConfig::default();
    let tt_config = TranspositionTableConfig::multi_level(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0xABCD, 7, 300);

    // Test store
    table.store(entry.clone());

    // Test probe
    let retrieved = table.probe(0xABCD, 7);
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test size
    let size = table.size();
    assert!(size > 0);

    // Test clear
    table.clear();
    let after_clear = table.probe(0xABCD, 7);
    assert!(after_clear.is_none());
}

#[cfg(feature = "hierarchical-tt")]
#[test]
fn test_hierarchical_table_trait() {
    // Test HierarchicalTranspositionTable
    let config = shogi_engine::search::hierarchical_transposition_table::HierarchicalTranspositionConfig::default();
    let tt_config = TranspositionTableConfig::hierarchical(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0xEF12, 8, 400);

    // Test store
    table.store(entry.clone());

    // Test probe (hierarchical returns (entry, level), trait adapts to just entry)
    let retrieved = table.probe(0xEF12, 8);
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test size
    let size = table.size();
    assert!(size > 0);

    // Test clear
    table.clear();
    let after_clear = table.probe(0xEF12, 8);
    assert!(after_clear.is_none());
}

#[cfg(feature = "hierarchical-tt")]
#[test]
fn test_compressed_table_trait() {
    // Test CompressedTranspositionTable
    let config = shogi_engine::search::compressed_transposition_table::CompressedTranspositionTableConfig::default();
    let tt_config = TranspositionTableConfig::compressed(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0x3456, 9, 500);

    // Test store
    table.store(entry.clone());

    // Test probe
    let retrieved = table.probe(0x3456, 9);
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.score, entry.score);
    assert_eq!(retrieved_entry.depth, entry.depth);

    // Test size
    let size = table.size();
    assert!(size > 0);

    // Test clear
    table.clear();
    let after_clear = table.probe(0x3456, 9);
    assert!(after_clear.is_none());
}

#[test]
fn test_trait_probe_with_prefetch_default() {
    // Test that probe_with_prefetch default implementation works
    // (calls probe ignoring prefetch hint)
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let table = create_transposition_table(tt_config);

    // Test that probe_with_prefetch works even without implementation
    // (should fall back to probe)
    let result = table.probe_with_prefetch(0x9999, 3, Some(0x8888));
    // Should return None (no entry stored)
    assert!(result.is_none());
}

#[test]
fn test_trait_polymorphism() {
    // Test that we can use different implementations through the trait
    let configs = vec![
        TranspositionTableConfig::basic(
            shogi_engine::search::transposition_table::TranspositionTableConfig::default(),
        ),
        TranspositionTableConfig::thread_safe(TranspositionConfig::default()),
        TranspositionTableConfig::multi_level(
            shogi_engine::search::multi_level_transposition_table::MultiLevelConfig::default(),
        ),
    ];

    for (i, tt_config) in configs.iter().enumerate() {
        let mut table = create_transposition_table(tt_config.clone());
        let hash_key = 0x1000 + i as u64;
        let entry = create_test_entry(hash_key, 5, 100 + i as i32);

        // All should support basic trait operations
        table.store(entry.clone());
        let retrieved = table.probe(hash_key, 5);
        assert!(retrieved.is_some(), "Config {} failed probe", i);
        assert_eq!(retrieved.unwrap().score, entry.score);

        // All should support clear
        table.clear();
        let after_clear = table.probe(hash_key, 5);
        assert!(after_clear.is_none(), "Config {} failed clear", i);

        // All should support size
        let size = table.size();
        assert!(size > 0, "Config {} has invalid size", i);
    }
}

#[test]
fn test_trait_depth_requirement() {
    // Test that probe respects depth requirements
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let mut table = create_transposition_table(tt_config);

    let entry = create_test_entry(0x7777, 5, 777);

    // Store entry at depth 5
    table.store(entry);

    // Probe with depth 5 - should find
    let result5 = table.probe(0x7777, 5);
    assert!(result5.is_some());

    // Probe with depth 4 - should find (entry depth >= required depth)
    let result4 = table.probe(0x7777, 4);
    assert!(result4.is_some());

    // Probe with depth 6 - should not find (entry depth < required depth)
    let result6 = table.probe(0x7777, 6);
    assert!(result6.is_none());
}

#[test]
fn test_trait_entry_replacement() {
    // Test that multiple stores replace entries correctly
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let mut table = create_transposition_table(tt_config);

    let hash_key = 0xAAAA;

    // Store first entry
    let entry1 = create_test_entry(hash_key, 5, 100);
    table.store(entry1.clone());
    let result1 = table.probe(hash_key, 5);
    assert!(result1.is_some());
    assert_eq!(result1.unwrap().score, 100);

    // Store second entry (should replace)
    let entry2 = create_test_entry(hash_key, 6, 200);
    table.store(entry2.clone());
    let result2 = table.probe(hash_key, 6);
    assert!(result2.is_some());
    assert_eq!(result2.unwrap().score, 200);
}

#[test]
fn test_trait_validation() {
    // Test that table configurations validate correctly
    let basic_config = shogi_engine::search::transposition_table::TranspositionTableConfig::default();
    let tt_config = TranspositionTableConfig::basic(basic_config);
    assert!(tt_config.validate().is_ok());

    let thread_safe_config = TranspositionConfig::default();
    let tt_config2 = TranspositionTableConfig::thread_safe(thread_safe_config);
    assert!(tt_config2.validate().is_ok());

    let multi_level_config = shogi_engine::search::multi_level_transposition_table::MultiLevelConfig::default();
    let tt_config3 = TranspositionTableConfig::multi_level(multi_level_config);
    assert!(tt_config3.validate().is_ok());
}

#[test]
fn test_trait_size_consistency() {
    // Test that size() returns consistent values
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let table = create_transposition_table(tt_config);

    let size1 = table.size();
    let size2 = table.size();
    assert_eq!(size1, size2, "Size should be consistent");
}

#[test]
fn test_trait_clear_preserves_size() {
    // Test that clear() doesn't change table size
    let config = TranspositionConfig::default();
    let tt_config = TranspositionTableConfig::thread_safe(config);
    let mut table = create_transposition_table(tt_config);

    let size_before = table.size();
    
    // Store some entries
    for i in 0..10 {
        let entry = create_test_entry(0x1000 + i, 5, 100 + i as i32);
        table.store(entry);
    }

    // Clear
    table.clear();

    // Size should remain the same (capacity unchanged)
    let size_after = table.size();
    assert_eq!(size_before, size_after, "Clear should not change table capacity");
}

