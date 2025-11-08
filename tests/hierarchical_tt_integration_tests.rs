#![cfg(feature = "hierarchical-tt")]

use shogi_engine::search::{
    CompressedTranspositionTableConfig, HierarchicalTranspositionConfig,
    HierarchicalTranspositionTable, HitLevel,
};
use shogi_engine::types::{EntrySource, TranspositionEntry, TranspositionFlag};

fn make_entry(hash: u64, depth: u8, age: u32, score: i32) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        TranspositionFlag::Exact,
        None,
        hash,
        age,
        EntrySource::MainSearch,
    )
}

fn small_config() -> HierarchicalTranspositionConfig {
    HierarchicalTranspositionConfig::default()
        .with_statistics_enabled(true)
        .with_l1_table_size(8)
        .with_promotion_depth(4)
        .with_demotion_age(2)
        .with_l2_config(
            CompressedTranspositionTableConfig::default()
                .with_max_entries(128)
                .with_segment_count(8)
                .with_target_compression_ratio(0.5)
                .with_max_maintenance_backlog(128),
        )
}

#[test]
fn demotes_low_depth_entries_into_l2() {
    let mut table = HierarchicalTranspositionTable::new(small_config());

    for i in 0..16 {
        let entry = make_entry(0xAB00 + i as u64, 1, 0, 50);
        table.store(entry);
    }

    let snapshot = table.snapshot();
    assert!(
        snapshot.stats.demotions >= 8,
        "expected demotions when L1 capacity exceeded"
    );
    assert!(
        snapshot.l2_stats.stored_entries > 0,
        "compressed L2 should contain demoted entries"
    );
}

#[test]
fn probing_l2_entry_promotes_back_to_l1() {
    let config = small_config().with_promotion_depth(6);
    let mut table = HierarchicalTranspositionTable::new(config);

    let hash = 0xDEADBEEF_u64;
    let entry = make_entry(hash, 2, 5, 120);
    table.store(entry.clone());

    // Overwrite the L1 slot with a colliding hash so subsequent probes require L2.
    let collision_hash = hash.wrapping_add(8);
    let collision_entry = make_entry(collision_hash, 6, 0, 60);
    table.store(collision_entry);

    let (retrieved, level) = table.probe(hash, 1).expect("entry missing");
    assert_eq!(level, HitLevel::L2);
    assert_eq!(retrieved.score, 120);

    let snapshot_after_probe = table.snapshot();
    assert_eq!(snapshot_after_probe.stats.l2_hits, 1);
    assert_eq!(snapshot_after_probe.stats.promotions, 1);

    let final_snapshot = table.snapshot();
    assert!(
        final_snapshot.stats.l2_hits >= 1,
        "expected at least one L2 hit recorded"
    );
    assert!(
        final_snapshot.stats.promotions >= 1,
        "expected promotion counter to increase after probe"
    );
    assert!(
        final_snapshot.stats.demotions >= 1,
        "demotions should be recorded from initial store"
    );
}
