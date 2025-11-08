//! Hierarchical transposition table facade (L1 + compressed L2).
//!
//! This module coordinates a fast, uncompressed L1 transposition table with the
//! compressed L2 backing store, handling promotion / demotion heuristics and
//! exposing a single probe/store API for the search engine.

use crate::search::compressed_transposition_table::{
    CompressedTranspositionStats, CompressedTranspositionTable, CompressedTranspositionTableConfig,
};
use crate::search::thread_safe_table::ThreadSafeTranspositionTable;
use crate::search::transposition_config::TranspositionConfig;
use crate::types::TranspositionEntry;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Hierarchical table configuration.
#[derive(Debug, Clone)]
pub struct HierarchicalTranspositionConfig {
    /// Configuration for the L1 (fast) table.
    pub l1_config: TranspositionConfig,
    /// Configuration for the L2 (compressed) table.
    pub l2_config: CompressedTranspositionTableConfig,
    /// Minimum depth to promote entries from L2 back into L1.
    pub promotion_depth: u8,
    /// Age threshold for demoting L1 entries into L2.
    pub demotion_age: u32,
    /// Whether statistics collection is enabled.
    pub enable_statistics: bool,
    /// Background maintenance configuration.
    pub maintenance: HierarchicalMaintenanceConfig,
}

/// Scheduling behaviour for optional maintenance worker.
#[derive(Debug, Clone)]
pub enum HierarchicalMaintenanceMode {
    /// Background maintenance disabled.
    Off,
    /// Run maintenance periodically every `seconds`.
    Interval { seconds: u64 },
    /// Trigger maintenance when L2 fill ratio exceeds threshold (0.0–1.0).
    LoadTriggered { fill_ratio: f64 },
}

impl Default for HierarchicalMaintenanceMode {
    fn default() -> Self {
        HierarchicalMaintenanceMode::Off
    }
}

/// Maintenance configuration (placeholder for future background worker).
#[derive(Debug, Clone)]
pub struct HierarchicalMaintenanceConfig {
    /// Scheduling mode.
    pub mode: HierarchicalMaintenanceMode,
    /// Optional limit on maintenance sweep duration (milliseconds).
    pub max_sweep_ms: Option<u64>,
}

impl Default for HierarchicalMaintenanceConfig {
    fn default() -> Self {
        Self {
            mode: HierarchicalMaintenanceMode::Off,
            max_sweep_ms: None,
        }
    }
}

impl Default for HierarchicalTranspositionConfig {
    fn default() -> Self {
        Self {
            l1_config: TranspositionConfig {
                table_size: 1 << 20, // 1M entries (~64 MB)
                enable_statistics: false,
                clear_between_games: false,
                ..TranspositionConfig::default()
            },
            l2_config: CompressedTranspositionTableConfig {
                max_entries: 8 * 1_000_000,
                segment_count: 1024,
                target_compression_ratio: 0.5,
                max_maintenance_backlog: 10_000,
            },
            promotion_depth: 6,
            demotion_age: 4,
            enable_statistics: false,
            maintenance: HierarchicalMaintenanceConfig::default(),
        }
    }
}

impl HierarchicalTranspositionConfig {
    /// Set the desired number of entries in the L1 table (will be rounded to next power of two).
    pub fn with_l1_table_size(mut self, entries: usize) -> Self {
        self.l1_config.table_size = entries.max(1);
        self
    }

    /// Override the compressed L2 configuration.
    pub fn with_l2_config(mut self, config: CompressedTranspositionTableConfig) -> Self {
        self.l2_config = config;
        self
    }

    /// Set the promotion depth threshold.
    pub fn with_promotion_depth(mut self, depth: u8) -> Self {
        self.promotion_depth = depth;
        self
    }

    /// Set the demotion age threshold.
    pub fn with_demotion_age(mut self, age: u32) -> Self {
        self.demotion_age = age;
        self
    }

    /// Enable or disable statistics collection for both tiers.
    pub fn with_statistics_enabled(mut self, enable: bool) -> Self {
        self.enable_statistics = enable;
        self.l1_config.enable_statistics = enable;
        self
    }

    /// Configure background maintenance behaviour.
    pub fn with_maintenance(mut self, maintenance: HierarchicalMaintenanceConfig) -> Self {
        self.maintenance = maintenance;
        self
    }

    /// Disable background maintenance explicitly.
    pub fn disable_maintenance(mut self) -> Self {
        self.maintenance = HierarchicalMaintenanceConfig::default();
        self
    }
}

/// Hierarchical hit metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitLevel {
    /// Entry retrieved from the L1 table.
    L1,
    /// Entry retrieved from the L2 table and promoted into L1.
    L2,
}

/// Statistics recorded by the hierarchical table.
#[derive(Debug, Default, Clone)]
pub struct HierarchicalStats {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub stores: u64,
    pub l1_misses: u64,
    pub l2_misses: u64,
}

/// Public snapshot of the hierarchical table state.
#[derive(Debug, Clone)]
pub struct HierarchicalSnapshot {
    pub stats: HierarchicalStats,
    pub l2_stats: CompressedTranspositionStats,
    pub maintenance: HierarchicalMaintenanceConfig,
}

/// Composite transposition table coordinating L1 and L2 tiers.
pub struct HierarchicalTranspositionTable {
    l1: ThreadSafeTranspositionTable,
    l2: Arc<Mutex<CompressedTranspositionTable>>,
    config: HierarchicalTranspositionConfig,
    stats: HierarchicalStats,
    maintenance_handle: Option<MaintenanceHandle>,
}

/// Background maintenance worker handle.
struct MaintenanceHandle {
    shutdown: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl HierarchicalTranspositionTable {
    /// Create a new hierarchical transposition table.
    pub fn new(config: HierarchicalTranspositionConfig) -> Self {
        let mut l1_config = config.l1_config.clone();
        l1_config.enable_statistics = config.enable_statistics;

        let l2 = Arc::new(Mutex::new(CompressedTranspositionTable::new(
            config.l2_config.clone(),
        )));
        let maintenance_handle =
            Self::spawn_maintenance_worker(&config.maintenance, Arc::clone(&l2));

        Self {
            l1: ThreadSafeTranspositionTable::new(l1_config),
            l2,
            config,
            stats: HierarchicalStats::default(),
            maintenance_handle,
        }
    }

    /// Probe the table hierarchy for a matching entry.
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<(TranspositionEntry, HitLevel)> {
        if let Some(entry) = self.l1.probe(hash, depth) {
            self.stats.l1_hits += 1;
            return Some((entry, HitLevel::L1));
        }

        self.stats.l1_misses += 1;
        let l2_hit = {
            let mut l2 = self.l2.lock().unwrap();
            l2.probe(hash, depth)
        };
        if let Some(entry) = l2_hit {
            self.stats.l2_hits += 1;
            self.promote_to_l1(&entry);
            self.stats.promotions += 1;
            return Some((entry, HitLevel::L2));
        }

        self.stats.l2_misses += 1;
        None
    }

    /// Store a new entry, first placing it in L1 and optionally demoting to L2.
    pub fn store(&mut self, entry: TranspositionEntry) {
        self.l1.store(entry.clone());
        self.stats.stores += 1;

        if entry.age >= self.config.demotion_age || entry.depth < self.config.promotion_depth {
            let mut l2 = self.l2.lock().unwrap();
            l2.store(&entry);
            self.stats.demotions += 1;
        }
    }

    /// Clear the L1 and L2 tables.
    pub fn clear(&mut self) {
        self.l1.clear();
        self.l2.lock().unwrap().clear();
        self.stats = HierarchicalStats::default();
    }

    /// Snapshot statistics and compressed L2 metrics.
    pub fn snapshot(&self) -> HierarchicalSnapshot {
        HierarchicalSnapshot {
            stats: self.stats.clone(),
            l2_stats: self.l2.lock().unwrap().stats().clone(),
            maintenance: self.config.maintenance.clone(),
        }
    }

    /// Access statistics (mutable) for advanced instrumentation.
    pub fn stats_mut(&mut self) -> &mut HierarchicalStats {
        &mut self.stats
    }

    /// Access the compressed table statistics.
    pub fn l2_stats(&self) -> CompressedTranspositionStats {
        self.l2.lock().unwrap().stats().clone()
    }

    /// L2 reference for testing/diagnostics.
    #[cfg(test)]
    pub(crate) fn l2_mut(&self) -> std::sync::MutexGuard<'_, CompressedTranspositionTable> {
        self.l2.lock().unwrap()
    }

    fn promote_to_l1(&mut self, entry: &TranspositionEntry) {
        self.l1.store(entry.clone());
    }

    fn spawn_maintenance_worker(
        maintenance: &HierarchicalMaintenanceConfig,
        l2: Arc<Mutex<CompressedTranspositionTable>>,
    ) -> Option<MaintenanceHandle> {
        match maintenance.mode {
            HierarchicalMaintenanceMode::Off => None,
            _ => {
                let shutdown_flag = Arc::new(AtomicBool::new(false));
                let maintenance_clone = maintenance.clone();
                let shutdown_clone = Arc::clone(&shutdown_flag);
                let l2_clone = Arc::clone(&l2);
                let handle = thread::Builder::new()
                    .name("hierarchical-tt-maint".into())
                    .spawn(move || {
                        maintenance_loop(l2_clone, maintenance_clone, shutdown_clone);
                    })
                    .expect("failed to spawn hierarchical maintenance worker");
                Some(MaintenanceHandle::new(handle, shutdown_flag))
            }
        }
    }
}

impl Drop for HierarchicalTranspositionTable {
    fn drop(&mut self) {
        if let Some(mut handle) = self.maintenance_handle.take() {
            handle.shutdown();
        }
    }
}

impl MaintenanceHandle {
    fn new(join_handle: thread::JoinHandle<()>, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            shutdown,
            join_handle: Some(join_handle),
        }
    }

    fn shutdown(&mut self) {
        self.shutdown.store(true, AtomicOrdering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MaintenanceHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn maintenance_loop(
    l2: Arc<Mutex<CompressedTranspositionTable>>,
    maintenance: HierarchicalMaintenanceConfig,
    shutdown: Arc<AtomicBool>,
) {
    match maintenance.mode {
        HierarchicalMaintenanceMode::Off => return,
        HierarchicalMaintenanceMode::Interval { seconds } => {
            let sleep_duration = if seconds == 0 {
                Duration::from_millis(50)
            } else {
                Duration::from_secs(seconds)
            };
            while !shutdown.load(AtomicOrdering::Relaxed) {
                perform_maintenance_cycle(&l2, &maintenance);
                if shutdown.load(AtomicOrdering::Relaxed) {
                    break;
                }
                thread::sleep(sleep_duration);
            }
        }
        HierarchicalMaintenanceMode::LoadTriggered { .. } => {
            let sleep_duration = Duration::from_millis(200);
            while !shutdown.load(AtomicOrdering::Relaxed) {
                let did_work = perform_maintenance_cycle(&l2, &maintenance);
                if shutdown.load(AtomicOrdering::Relaxed) {
                    break;
                }
                if did_work {
                    thread::sleep(Duration::from_millis(10));
                } else {
                    thread::sleep(sleep_duration);
                }
            }
        }
    }
}

fn perform_maintenance_cycle(
    l2: &Arc<Mutex<CompressedTranspositionTable>>,
    maintenance: &HierarchicalMaintenanceConfig,
) -> bool {
    match maintenance.mode {
        HierarchicalMaintenanceMode::Off => false,
        HierarchicalMaintenanceMode::Interval { .. } => {
            run_maintenance_sweep(l2, maintenance);
            true
        }
        HierarchicalMaintenanceMode::LoadTriggered { fill_ratio } => {
            let should_run = {
                let l2_guard = l2.lock().unwrap();
                l2_guard.fill_ratio() >= fill_ratio
            };
            if should_run {
                run_maintenance_sweep(l2, maintenance);
                true
            } else {
                false
            }
        }
    }
}

fn run_maintenance_sweep(
    l2: &Arc<Mutex<CompressedTranspositionTable>>,
    maintenance: &HierarchicalMaintenanceConfig,
) -> usize {
    let mut l2_guard = l2.lock().unwrap();
    let max_backlog = l2_guard.config().max_maintenance_backlog;
    let duration = maintenance.max_sweep_ms.map(Duration::from_millis);
    l2_guard.maintenance_sweep(max_backlog, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntrySource, Move, PieceType, Player, Position, TranspositionFlag};
    use std::thread;
    use std::time::Duration;

    fn make_entry(hash: u64, depth: u8, score: i32, age: u32) -> TranspositionEntry {
        let mv = Move::new_move(
            Position::new(4, 4),
            Position::new(4, 5),
            PieceType::Silver,
            Player::Black,
            false,
        );
        TranspositionEntry {
            score,
            depth,
            flag: TranspositionFlag::Exact,
            best_move: Some(mv),
            hash_key: hash,
            age,
            source: EntrySource::MainSearch,
        }
    }

    fn test_config() -> HierarchicalTranspositionConfig {
        let mut l1_config = TranspositionConfig::debug_config();
        l1_config.table_size = 512;
        l1_config.enable_statistics = false;

        HierarchicalTranspositionConfig {
            l1_config,
            l2_config: CompressedTranspositionTableConfig {
                max_entries: 2048,
                segment_count: 32,
                target_compression_ratio: 0.5,
                max_maintenance_backlog: 10_000,
            },
            promotion_depth: 4,
            demotion_age: 3,
            enable_statistics: false,
            maintenance: HierarchicalMaintenanceConfig::default(),
        }
    }

    #[test]
    fn probe_hits_l1_first() {
        let mut table = HierarchicalTranspositionTable::new(test_config());
        let entry = make_entry(0xDEADBEEF, 6, 120, 1);
        table.store(entry.clone());

        let result = table.probe(0xDEADBEEF, 4).expect("entry missing");
        assert_eq!(result.0.score, 120);
        assert_eq!(result.1, HitLevel::L1);
    }

    #[test]
    fn l2_hit_promotes_to_l1() {
        let mut table = HierarchicalTranspositionTable::new(test_config());
        let entry = make_entry(0xABCD1234, 6, 90, 2);
        {
            let mut l2 = table.l2_mut();
            l2.store(&entry);
        }

        let (retrieved, level) = table.probe(0xABCD1234, 4).expect("entry missing");
        assert_eq!(level, HitLevel::L2);
        assert_eq!(retrieved.score, 90);

        // Subsequent probe should hit L1 due to promotion.
        let (retrieved_again, level_again) = table
            .probe(0xABCD1234, 4)
            .expect("entry missing after promotion");
        assert_eq!(level_again, HitLevel::L1);
        assert_eq!(retrieved_again.score, 90);
    }

    #[test]
    fn demotion_stores_entry_in_l2() {
        let mut table = HierarchicalTranspositionTable::new(test_config());
        let entry = make_entry(0xCAFEBABE, 2, 40, 5);
        table.store(entry.clone());

        let snapshot = table.snapshot();
        assert!(snapshot.l2_stats.stored_entries > 0);
    }

    #[test]
    fn configuration_builder_helpers_adjust_fields() {
        let maintenance = HierarchicalMaintenanceConfig {
            mode: HierarchicalMaintenanceMode::Interval { seconds: 30 },
            max_sweep_ms: Some(250),
        };

        let config = HierarchicalTranspositionConfig::default()
            .with_l1_table_size(2048)
            .with_l2_config(
                CompressedTranspositionTableConfig::default()
                    .with_max_entries(5000)
                    .with_segment_count(64)
                    .with_target_compression_ratio(0.6)
                    .with_max_maintenance_backlog(7),
            )
            .with_promotion_depth(8)
            .with_demotion_age(6)
            .with_statistics_enabled(true)
            .with_maintenance(maintenance.clone());

        assert_eq!(config.l1_config.table_size, 2048);
        assert_eq!(config.l2_config.max_entries, 5000);
        assert_eq!(config.l2_config.segment_count, 64);
        assert!((config.l2_config.target_compression_ratio - 0.6).abs() < f64::EPSILON);
        assert_eq!(config.promotion_depth, 8);
        assert_eq!(config.demotion_age, 6);
        assert!(config.enable_statistics);
        assert!(config.l1_config.enable_statistics);
        matches!(
            &config.maintenance.mode,
            HierarchicalMaintenanceMode::Interval { .. }
        );
        assert_eq!(config.maintenance.max_sweep_ms, Some(250));
        assert_eq!(config.l2_config.max_maintenance_backlog, 7);

        let disabled = config.disable_maintenance();
        matches!(disabled.maintenance.mode, HierarchicalMaintenanceMode::Off);
        assert!(disabled.maintenance.max_sweep_ms.is_none());
    }

    #[test]
    fn maintenance_sweep_respects_backlog() {
        let mut table = CompressedTranspositionTable::new(
            CompressedTranspositionTableConfig::default()
                .with_max_entries(50)
                .with_max_maintenance_backlog(5),
        );

        for i in 0..20 {
            let entry = make_entry(0x1000 + i as u64, 3, i as i32, 10);
            table.store(&entry);
        }

        assert!(table.len() > 5);
        let removed = table.maintenance_sweep(5, Some(Duration::from_millis(10)));
        assert!(removed > 0);
        assert!(table.len() <= 5);
    }

    #[test]
    fn interval_maintenance_worker_runs() {
        let maintenance = HierarchicalMaintenanceConfig {
            mode: HierarchicalMaintenanceMode::Interval { seconds: 0 },
            max_sweep_ms: Some(50),
        };

        let config = HierarchicalTranspositionConfig::default()
            .with_l1_table_size(128)
            .with_l2_config(
                CompressedTranspositionTableConfig::default()
                    .with_max_entries(40)
                    .with_max_maintenance_backlog(5),
            )
            .with_maintenance(maintenance);

        let mut table = HierarchicalTranspositionTable::new(config);

        {
            let mut l2 = table.l2_mut();
            for i in 0..25 {
                l2.store(&make_entry(0x2000 + i as u64, 2, i as i32, 8));
            }
        }

        let before = table.snapshot().l2_stats.stored_entries;
        assert!(before > 5);

        thread::sleep(Duration::from_millis(150));

        let after = table.snapshot().l2_stats.stored_entries;
        assert!(after <= 5);
    }
}
