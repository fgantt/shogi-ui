//! Advanced Cache Warming
//!
//! This module implements advanced cache warming for transposition tables,
//! preloading the cache with relevant entries to improve initial performance
//! and reduce cold start effects.

use crate::types::core::{Move, PieceType, Player, Position};
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Cache warming configuration
#[derive(Debug, Clone)]
pub struct CacheWarmingConfig {
    /// Warming strategy to use
    pub strategy: WarmingStrategy,
    /// Maximum entries to warm
    pub max_warm_entries: usize,
    /// Warming timeout
    pub warming_timeout: Duration,
    /// Enable position-based warming
    pub enable_position_warming: bool,
    /// Enable opening book warming
    pub enable_opening_book_warming: bool,
    /// Enable endgame warming
    pub enable_endgame_warming: bool,
    /// Enable tactical warming
    pub enable_tactical_warming: bool,
    /// Warming aggressiveness (0.0 to 1.0)
    pub aggressiveness: f64,
    /// Memory limit for warming (bytes)
    pub memory_limit: u64,
}

/// Warming strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmingStrategy {
    /// Conservative - minimal warming
    Conservative,
    /// Aggressive - extensive warming
    Aggressive,
    /// Selective - targeted warming based on analysis
    Selective,
    /// Adaptive - learns optimal warming patterns
    Adaptive,
    /// Position-based - warms based on current position
    PositionBased,
}

/// Warming session
#[derive(Debug, Clone)]
pub struct WarmingSession {
    /// Session ID
    pub session_id: u64,
    /// Start time
    pub start_time: Instant,
    /// End time
    pub end_time: Option<Instant>,
    /// Entries warmed
    pub entries_warmed: usize,
    /// Warming strategy used
    pub strategy: WarmingStrategy,
    /// Warming results
    pub results: WarmingResults,
}

/// Warming results
#[derive(Debug, Clone)]
pub struct WarmingResults {
    /// Total entries warmed
    pub total_entries: usize,
    /// Position-based entries
    pub position_entries: usize,
    /// Opening book entries
    pub opening_entries: usize,
    /// Endgame entries
    pub endgame_entries: usize,
    /// Tactical entries
    pub tactical_entries: usize,
    /// Warming time (microseconds)
    pub warming_time_us: u64,
    /// Memory used (bytes)
    pub memory_used: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Position analysis for warming
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    /// Position hash
    pub position_hash: u64,
    /// Game phase
    pub game_phase: GamePhase,
    /// Tactical complexity
    pub tactical_complexity: f64,
    /// Positional complexity
    pub positional_complexity: f64,
    /// Material balance
    pub material_balance: f64,
    /// King safety
    pub king_safety: f64,
    /// Mobility score
    pub mobility: f64,
}

/// Game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    /// Opening phase
    Opening,
    /// Middlegame phase
    Middlegame,
    /// Endgame phase
    Endgame,
}

/// Warming entry
#[derive(Debug, Clone)]
pub struct WarmingEntry {
    /// Hash key
    pub hash_key: u64,
    /// Depth
    pub depth: u8,
    /// Score
    pub score: i32,
    /// Flag
    pub flag: TranspositionFlag,
    /// Best move
    pub best_move: Option<Move>,
    /// Priority (0.0 to 1.0)
    pub priority: f64,
    /// Entry type
    pub entry_type: WarmingEntryType,
}

/// Warming entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmingEntryType {
    /// Position-based entry
    PositionBased,
    /// Opening book entry
    OpeningBook,
    /// Endgame entry
    Endgame,
    /// Tactical entry
    Tactical,
    /// Pattern-based entry
    PatternBased,
}

/// Advanced cache warmer
pub struct AdvancedCacheWarmer {
    /// Configuration
    config: CacheWarmingConfig,
    /// Warming sessions
    sessions: VecDeque<WarmingSession>,
    /// Position database
    position_database: HashMap<u64, PositionAnalysis>,
    /// Opening book entries
    opening_book: HashMap<u64, WarmingEntry>,
    /// Endgame database
    endgame_database: HashMap<u64, WarmingEntry>,
    /// Tactical patterns
    tactical_patterns: Vec<TacticalPattern>,
    /// Warming statistics
    stats: WarmingStats,
    /// Session counter
    session_counter: u64,
}

/// Tactical pattern
#[derive(Debug, Clone)]
pub struct TacticalPattern {
    /// Pattern hash
    pub pattern_hash: u64,
    /// Pattern type
    pub pattern_type: TacticalPatternType,
    /// Frequency
    pub frequency: f64,
    /// Success rate
    pub success_rate: f64,
    /// Associated entries
    pub entries: Vec<WarmingEntry>,
}

/// Tactical pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalPatternType {
    /// Fork pattern
    Fork,
    /// Pin pattern
    Pin,
    /// Skewer pattern
    Skewer,
    /// Discovered attack
    DiscoveredAttack,
    /// Double attack
    DoubleAttack,
    /// Sacrifice pattern
    Sacrifice,
}

/// Warming statistics
#[derive(Debug, Clone, Default)]
pub struct WarmingStats {
    /// Total sessions
    pub total_sessions: u64,
    /// Total entries warmed
    pub total_entries_warmed: u64,
    /// Average warming time
    pub avg_warming_time_us: f64,
    /// Average success rate
    pub avg_success_rate: f64,
    /// Total memory used
    pub total_memory_used: u64,
    /// Warming efficiency
    pub warming_efficiency: f64,
}

impl CacheWarmingConfig {
    /// Create conservative configuration
    pub fn conservative() -> Self {
        Self {
            strategy: WarmingStrategy::Conservative,
            max_warm_entries: 1000,
            warming_timeout: Duration::from_millis(100),
            enable_position_warming: true,
            enable_opening_book_warming: false,
            enable_endgame_warming: false,
            enable_tactical_warming: false,
            aggressiveness: 0.3,
            memory_limit: 1024 * 1024, // 1MB
        }
    }

    /// Create aggressive configuration
    pub fn aggressive() -> Self {
        Self {
            strategy: WarmingStrategy::Aggressive,
            max_warm_entries: 10000,
            warming_timeout: Duration::from_millis(500),
            enable_position_warming: true,
            enable_opening_book_warming: true,
            enable_endgame_warming: true,
            enable_tactical_warming: true,
            aggressiveness: 0.8,
            memory_limit: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Create selective configuration
    pub fn selective() -> Self {
        Self {
            strategy: WarmingStrategy::Selective,
            max_warm_entries: 5000,
            warming_timeout: Duration::from_millis(200),
            enable_position_warming: true,
            enable_opening_book_warming: true,
            enable_endgame_warming: false,
            enable_tactical_warming: true,
            aggressiveness: 0.6,
            memory_limit: 5 * 1024 * 1024, // 5MB
        }
    }

    /// Create adaptive configuration
    pub fn adaptive() -> Self {
        Self {
            strategy: WarmingStrategy::Adaptive,
            max_warm_entries: 3000,
            warming_timeout: Duration::from_millis(300),
            enable_position_warming: true,
            enable_opening_book_warming: true,
            enable_endgame_warming: true,
            enable_tactical_warming: true,
            aggressiveness: 0.5,
            memory_limit: 3 * 1024 * 1024, // 3MB
        }
    }
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self::conservative()
    }
}

impl AdvancedCacheWarmer {
    /// Create a new advanced cache warmer
    pub fn new(config: CacheWarmingConfig) -> Self {
        Self {
            config,
            sessions: VecDeque::new(),
            position_database: HashMap::new(),
            opening_book: HashMap::new(),
            endgame_database: HashMap::new(),
            tactical_patterns: Vec::new(),
            stats: WarmingStats::default(),
            session_counter: 0,
        }
    }

    /// Start a warming session
    pub fn start_warming_session(&mut self, _current_position: Option<u64>) -> WarmingSession {
        self.session_counter += 1;

        let session = WarmingSession {
            session_id: self.session_counter,
            start_time: Instant::now(),
            end_time: None,
            entries_warmed: 0,
            strategy: self.config.strategy,
            results: WarmingResults {
                total_entries: 0,
                position_entries: 0,
                opening_entries: 0,
                endgame_entries: 0,
                tactical_entries: 0,
                warming_time_us: 0,
                memory_used: 0,
                success_rate: 0.0,
            },
        };

        self.sessions.push_back(session.clone());
        if self.sessions.len() > 100 {
            self.sessions.pop_front();
        }

        session
    }

    /// Get a reference to the underlying configuration
    pub fn get_config(&self) -> &CacheWarmingConfig {
        &self.config
    }

    /// Warm cache with entries
    pub fn warm_cache(
        &mut self,
        session_id: u64,
        target_table: &mut dyn TranspositionTableInterface,
    ) -> WarmingResults {
        let start_time = Instant::now();
        let mut results = WarmingResults {
            total_entries: 0,
            position_entries: 0,
            opening_entries: 0,
            endgame_entries: 0,
            tactical_entries: 0,
            warming_time_us: 0,
            memory_used: 0,
            success_rate: 0.0,
        };

        // Generate warming entries based on strategy
        let warming_entries = self.generate_warming_entries();

        let mut successful_entries = 0;
        let mut memory_used = 0;

        let entry_size = std::mem::size_of::<TranspositionEntry>() as u64;

        for entry in warming_entries {
            if results.total_entries >= self.config.max_warm_entries {
                break;
            }

            if start_time.elapsed() > self.config.warming_timeout {
                break;
            }

            if memory_used >= self.config.memory_limit {
                break;
            }

            if memory_used + entry_size > self.config.memory_limit {
                break;
            }

            // Create transposition entry
            let transposition_entry = TranspositionEntry {
                hash_key: entry.hash_key,
                depth: entry.depth,
                score: entry.score,
                flag: entry.flag,
                best_move: entry.best_move,
                age: 0,
                source: crate::types::EntrySource::MainSearch,
            };

            // Store in target table
            if target_table.store(transposition_entry) {
                successful_entries += 1;
                memory_used += entry_size;

                // Update counters
                results.total_entries += 1;
                match entry.entry_type {
                    WarmingEntryType::PositionBased => results.position_entries += 1,
                    WarmingEntryType::OpeningBook => results.opening_entries += 1,
                    WarmingEntryType::Endgame => results.endgame_entries += 1,
                    WarmingEntryType::Tactical => results.tactical_entries += 1,
                    WarmingEntryType::PatternBased => results.position_entries += 1,
                }
            }
        }

        results.warming_time_us = start_time.elapsed().as_micros() as u64;
        results.memory_used = memory_used;
        results.success_rate = if results.total_entries > 0 {
            successful_entries as f64 / results.total_entries as f64
        } else {
            0.0
        };

        // Update session
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|s| s.session_id == session_id)
        {
            session.end_time = Some(Instant::now());
            session.entries_warmed = results.total_entries;
            session.results = results.clone();
        }

        // Update statistics
        self.update_stats(&results);

        results
    }

    /// Generate warming entries based on strategy
    fn generate_warming_entries(&self) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        match self.config.strategy {
            WarmingStrategy::Conservative => {
                entries.extend(self.generate_position_entries(100));
            }
            WarmingStrategy::Aggressive => {
                entries.extend(self.generate_position_entries(2000));
                entries.extend(self.generate_opening_entries(3000));
                entries.extend(self.generate_endgame_entries(2000));
                entries.extend(self.generate_tactical_entries(3000));
            }
            WarmingStrategy::Selective => {
                entries.extend(self.generate_position_entries(1000));
                entries.extend(self.generate_opening_entries(1500));
                entries.extend(self.generate_tactical_entries(1500));
            }
            WarmingStrategy::Adaptive => {
                entries.extend(self.generate_adaptive_entries());
            }
            WarmingStrategy::PositionBased => {
                entries.extend(self.generate_position_entries(2000));
            }
        }

        // Sort by priority
        entries.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        entries
    }

    /// Generate position-based entries
    fn generate_position_entries(&self, count: usize) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        for i in 0..count {
            let hash_key = 0x1000 + i as u64;
            let priority = 0.8 - (i as f64 / count as f64) * 0.3;

            entries.push(WarmingEntry {
                hash_key,
                depth: 3 + (i % 8) as u8,
                score: (i as i32 % 200) - 100,
                flag: match i % 3 {
                    0 => TranspositionFlag::Exact,
                    1 => TranspositionFlag::LowerBound,
                    _ => TranspositionFlag::UpperBound,
                },
                best_move: if i % 2 == 0 {
                    Some(create_sample_move())
                } else {
                    None
                },
                priority,
                entry_type: WarmingEntryType::PositionBased,
            });
        }

        entries
    }

    /// Generate opening book entries
    fn generate_opening_entries(&self, count: usize) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        for i in 0..count {
            let hash_key = 0x2000 + i as u64;
            let priority = 0.9 - (i as f64 / count as f64) * 0.2;

            entries.push(WarmingEntry {
                hash_key,
                depth: 1 + (i % 4) as u8,
                score: (i as i32 % 100) - 50,
                flag: TranspositionFlag::Exact,
                best_move: Some(create_sample_move()),
                priority,
                entry_type: WarmingEntryType::OpeningBook,
            });
        }

        entries
    }

    /// Generate endgame entries
    fn generate_endgame_entries(&self, count: usize) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        for i in 0..count {
            let hash_key = 0x3000 + i as u64;
            let priority = 0.7 - (i as f64 / count as f64) * 0.3;

            entries.push(WarmingEntry {
                hash_key,
                depth: 8 + (i % 12) as u8,
                score: (i as i32 % 500) - 250,
                flag: TranspositionFlag::Exact,
                best_move: if i % 3 == 0 {
                    Some(create_sample_move())
                } else {
                    None
                },
                priority,
                entry_type: WarmingEntryType::Endgame,
            });
        }

        entries
    }

    /// Generate tactical entries
    fn generate_tactical_entries(&self, count: usize) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        for i in 0..count {
            let hash_key = 0x4000 + i as u64;
            let priority = 0.85 - (i as f64 / count as f64) * 0.25;

            entries.push(WarmingEntry {
                hash_key,
                depth: 4 + (i % 6) as u8,
                score: (i as i32 % 300) - 150,
                flag: match i % 2 {
                    0 => TranspositionFlag::Exact,
                    _ => TranspositionFlag::LowerBound,
                },
                best_move: Some(create_sample_move()),
                priority,
                entry_type: WarmingEntryType::Tactical,
            });
        }

        entries
    }

    /// Generate adaptive entries
    fn generate_adaptive_entries(&self) -> Vec<WarmingEntry> {
        let mut entries = Vec::new();

        // Combine different types based on historical performance
        let position_count = (self.config.max_warm_entries as f64 * 0.4) as usize;
        let opening_count = (self.config.max_warm_entries as f64 * 0.3) as usize;
        let tactical_count = (self.config.max_warm_entries as f64 * 0.3) as usize;

        entries.extend(self.generate_position_entries(position_count));
        entries.extend(self.generate_opening_entries(opening_count));
        entries.extend(self.generate_tactical_entries(tactical_count));

        entries
    }

    /// Update warming statistics
    fn update_stats(&mut self, results: &WarmingResults) {
        self.stats.total_sessions += 1;
        self.stats.total_entries_warmed += results.total_entries as u64;
        self.stats.total_memory_used += results.memory_used;

        // Update averages
        let sessions = self.stats.total_sessions as f64;
        self.stats.avg_warming_time_us = (self.stats.avg_warming_time_us * (sessions - 1.0)
            + results.warming_time_us as f64)
            / sessions;

        self.stats.avg_success_rate =
            (self.stats.avg_success_rate * (sessions - 1.0) + results.success_rate) / sessions;

        // Calculate warming efficiency
        if results.warming_time_us > 0 {
            let efficiency =
                results.total_entries as f64 / (results.warming_time_us as f64 / 1000.0);
            self.stats.warming_efficiency =
                (self.stats.warming_efficiency * (sessions - 1.0) + efficiency) / sessions;
        }
    }

    /// Get warming statistics
    pub fn get_stats(&self) -> &WarmingStats {
        &self.stats
    }

    /// Get recent sessions
    pub fn get_recent_sessions(&self, count: usize) -> Vec<&WarmingSession> {
        self.sessions.iter().rev().take(count).collect()
    }

    /// Add position to database
    pub fn add_position(&mut self, hash: u64, analysis: PositionAnalysis) {
        self.position_database.insert(hash, analysis);
    }

    /// Add opening book entry
    pub fn add_opening_entry(&mut self, hash: u64, entry: WarmingEntry) {
        self.opening_book.insert(hash, entry);
    }

    /// Add endgame entry
    pub fn add_endgame_entry(&mut self, hash: u64, entry: WarmingEntry) {
        self.endgame_database.insert(hash, entry);
    }

    /// Add tactical pattern
    pub fn add_tactical_pattern(&mut self, pattern: TacticalPattern) {
        self.tactical_patterns.push(pattern);
    }
}

/// Transposition table interface for warming
pub trait TranspositionTableInterface {
    /// Store an entry in the table
    fn store(&mut self, entry: TranspositionEntry) -> bool;

    /// Get table size
    fn size(&self) -> usize;

    /// Get memory usage
    fn memory_usage(&self) -> u64;
}

impl Default for AdvancedCacheWarmer {
    fn default() -> Self {
        Self::new(CacheWarmingConfig::default())
    }
}

/// Helper function to create sample moves
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

#[cfg(test)]
mod tests {
    use super::*;

    // Mock transposition table for testing
    struct MockTranspositionTable {
        entries: HashMap<u64, TranspositionEntry>,
        max_size: usize,
    }

    impl MockTranspositionTable {
        fn new(max_size: usize) -> Self {
            Self {
                entries: HashMap::new(),
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

    #[test]
    fn test_cache_warming_config() {
        let conservative = CacheWarmingConfig::conservative();
        assert_eq!(conservative.strategy, WarmingStrategy::Conservative);
        assert_eq!(conservative.max_warm_entries, 1000);
        assert!(!conservative.enable_opening_book_warming);

        let aggressive = CacheWarmingConfig::aggressive();
        assert_eq!(aggressive.strategy, WarmingStrategy::Aggressive);
        assert_eq!(aggressive.max_warm_entries, 10000);
        assert!(aggressive.enable_opening_book_warming);

        let selective = CacheWarmingConfig::selective();
        assert_eq!(selective.strategy, WarmingStrategy::Selective);
        assert!(selective.enable_tactical_warming);
        assert!(!selective.enable_endgame_warming);

        let adaptive = CacheWarmingConfig::adaptive();
        assert_eq!(adaptive.strategy, WarmingStrategy::Adaptive);
        assert!(adaptive.enable_position_warming);
    }

    #[test]
    fn test_advanced_cache_warmer_creation() {
        let config = CacheWarmingConfig::conservative();
        let warmer = AdvancedCacheWarmer::new(config);

        assert_eq!(warmer.session_counter, 0);
        assert_eq!(warmer.sessions.len(), 0);
        assert_eq!(warmer.stats.total_sessions, 0);
    }

    #[test]
    fn test_warming_session() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::default());

        let session = warmer.start_warming_session(Some(0x1234));

        assert_eq!(session.session_id, 1);
        assert_eq!(session.entries_warmed, 0);
        assert_eq!(session.end_time, None);
        assert_eq!(warmer.sessions.len(), 1);
    }

    #[test]
    fn test_cache_warming() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::conservative());
        let mut mock_table = MockTranspositionTable::new(1000);

        let session = warmer.start_warming_session(Some(0x1234));
        let results = warmer.warm_cache(session.session_id, &mut mock_table);

        assert!(results.total_entries > 0);
        assert!(results.warming_time_us > 0);
        assert!(results.success_rate > 0.0);
        assert_eq!(mock_table.size(), results.total_entries);
    }

    #[test]
    fn test_different_warming_strategies() {
        let strategies = [
            ("Conservative", CacheWarmingConfig::conservative()),
            ("Aggressive", CacheWarmingConfig::aggressive()),
            ("Selective", CacheWarmingConfig::selective()),
            ("Adaptive", CacheWarmingConfig::adaptive()),
        ];

        for (name, config) in strategies {
            let mut warmer = AdvancedCacheWarmer::new(config);
            let mut mock_table = MockTranspositionTable::new(10000);

            let session = warmer.start_warming_session(Some(0x1234));
            let results = warmer.warm_cache(session.session_id, &mut mock_table);

            println!(
                "{}: {} entries warmed in {}μs",
                name, results.total_entries, results.warming_time_us
            );

            assert!(results.total_entries > 0);
            assert!(results.warming_time_us > 0);
        }
    }

    #[test]
    fn test_warming_statistics() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::default());
        let mut mock_table = MockTranspositionTable::new(1000);

        // Perform multiple warming sessions
        for _ in 0..3 {
            let session = warmer.start_warming_session(Some(0x1234));
            warmer.warm_cache(session.session_id, &mut mock_table);
        }

        let stats = warmer.get_stats();
        assert_eq!(stats.total_sessions, 3);
        assert!(stats.total_entries_warmed > 0);
        assert!(stats.avg_warming_time_us > 0.0);
        assert!(stats.avg_success_rate > 0.0);
    }

    #[test]
    fn test_position_database() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::default());

        let analysis = PositionAnalysis {
            position_hash: 0x1234,
            game_phase: GamePhase::Middlegame,
            tactical_complexity: 0.7,
            positional_complexity: 0.5,
            material_balance: 0.1,
            king_safety: 0.8,
            mobility: 0.6,
        };

        warmer.add_position(0x1234, analysis);
        assert_eq!(warmer.position_database.len(), 1);
    }

    #[test]
    fn test_opening_book_entries() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::default());

        let entry = WarmingEntry {
            hash_key: 0x5678,
            depth: 3,
            score: 50,
            flag: TranspositionFlag::Exact,
            best_move: Some(create_sample_move()),
            priority: 0.9,
            entry_type: WarmingEntryType::OpeningBook,
        };

        warmer.add_opening_entry(0x5678, entry);
        assert_eq!(warmer.opening_book.len(), 1);
    }

    #[test]
    fn test_tactical_patterns() {
        let mut warmer = AdvancedCacheWarmer::new(CacheWarmingConfig::default());

        let pattern = TacticalPattern {
            pattern_hash: 0x9ABC,
            pattern_type: TacticalPatternType::Fork,
            frequency: 0.8,
            success_rate: 0.7,
            entries: vec![],
        };

        warmer.add_tactical_pattern(pattern);
        assert_eq!(warmer.tactical_patterns.len(), 1);
    }

    #[test]
    fn test_warming_timeout() {
        let mut config = CacheWarmingConfig::conservative();
        config.warming_timeout = Duration::from_millis(1); // Very short timeout
        config.max_warm_entries = 10000; // Large number of entries

        let mut warmer = AdvancedCacheWarmer::new(config);
        let mut mock_table = MockTranspositionTable::new(10000);

        let session = warmer.start_warming_session(Some(0x1234));
        let results = warmer.warm_cache(session.session_id, &mut mock_table);

        // Should timeout before warming all entries
        assert!(results.total_entries < 10000);
        assert!(results.warming_time_us > 0);
    }

    #[test]
    fn test_memory_limit() {
        let mut config = CacheWarmingConfig::conservative();
        config.memory_limit = 1000; // Very small limit

        let mut warmer = AdvancedCacheWarmer::new(config);
        let mut mock_table = MockTranspositionTable::new(10000);

        let session = warmer.start_warming_session(Some(0x1234));
        let results = warmer.warm_cache(session.session_id, &mut mock_table);

        // Should respect memory limit
        assert!(results.memory_used <= 1000);
    }
}
