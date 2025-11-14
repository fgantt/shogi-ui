/// Unified statistics API for opening book
///
/// This module aggregates statistics from various sources:
/// - Opening book operations (migration, memory usage)
/// - Opening principles integration (book move evaluation)
/// - Move ordering integration (opening book move prioritization)
use super::{HashCollisionStats, MemoryUsageStats};
use crate::evaluation::opening_principles::OpeningPrincipleStats;
use crate::opening_book_converter::MigrationStats;
use crate::search::move_ordering::AdvancedIntegrationStats;

/// Unified statistics structure for opening book
#[derive(Debug, Clone, Default)]
pub struct BookStatistics {
    /// Migration statistics (from JSON to binary conversion)
    pub migration: Option<MigrationStats>,
    /// Memory usage statistics
    pub memory: Option<MemoryUsageStats>,
    /// Hash collision statistics
    pub hash_collisions: Option<HashCollisionStats>,
    /// Opening principles integration statistics
    pub opening_principles: OpeningPrincipleBookStats,
    /// Move ordering integration statistics
    pub move_ordering: MoveOrderingBookStats,
}

/// Opening principles statistics related to opening book
#[derive(Debug, Clone, Default)]
pub struct OpeningPrincipleBookStats {
    /// Number of book moves evaluated using opening principles
    pub book_moves_evaluated: u64,
    /// Number of book moves prioritized by opening principles
    pub book_moves_prioritized: u64,
    /// Number of book moves validated (checked for violations)
    pub book_moves_validated: u64,
    /// Sum of book move quality scores (for average calculation)
    pub book_move_quality_scores: i64,
}

/// Move ordering statistics related to opening book
#[derive(Debug, Clone, Default)]
pub struct MoveOrderingBookStats {
    /// Number of opening book integrations in move ordering
    pub opening_book_integrations: u64,
}

impl BookStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update from opening principles stats
    pub fn update_from_opening_principles(&mut self, stats: &OpeningPrincipleStats) {
        self.opening_principles.book_moves_evaluated = stats.book_moves_evaluated;
        self.opening_principles.book_moves_prioritized = stats.book_moves_prioritized;
        self.opening_principles.book_moves_validated = stats.book_moves_validated;
        self.opening_principles.book_move_quality_scores = stats.book_move_quality_scores;
    }

    /// Update from move ordering integration stats
    pub fn update_from_move_ordering(&mut self, stats: &AdvancedIntegrationStats) {
        self.move_ordering.opening_book_integrations = stats.opening_book_integrations;
    }

    /// Update migration statistics
    pub fn set_migration_stats(&mut self, stats: MigrationStats) {
        self.migration = Some(stats);
    }

    /// Update memory usage statistics
    pub fn set_memory_stats(&mut self, stats: MemoryUsageStats) {
        self.memory = Some(stats);
    }

    /// Update hash collision statistics
    pub fn set_hash_collision_stats(&mut self, stats: HashCollisionStats) {
        self.hash_collisions = Some(stats);
    }

    /// Calculate average book move quality score
    pub fn average_book_move_quality(&self) -> f64 {
        if self.opening_principles.book_moves_evaluated > 0 {
            self.opening_principles.book_move_quality_scores as f64
                / self.opening_principles.book_moves_evaluated as f64
        } else {
            0.0
        }
    }
}

impl From<&OpeningPrincipleStats> for OpeningPrincipleBookStats {
    fn from(stats: &OpeningPrincipleStats) -> Self {
        Self {
            book_moves_evaluated: stats.book_moves_evaluated,
            book_moves_prioritized: stats.book_moves_prioritized,
            book_moves_validated: stats.book_moves_validated,
            book_move_quality_scores: stats.book_move_quality_scores,
        }
    }
}

impl From<&AdvancedIntegrationStats> for MoveOrderingBookStats {
    fn from(stats: &AdvancedIntegrationStats) -> Self {
        Self {
            opening_book_integrations: stats.opening_book_integrations,
        }
    }
}
