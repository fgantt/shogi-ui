//! Advanced replacement policies for transposition table
//!
//! This module provides sophisticated replacement policies for the transposition table,
//! including depth-preferred, age-based, and combined strategies that optimize
//! hit rates and search performance.

use crate::search::cache_management::AgeCounter;
use crate::search::transposition_config::{ReplacementPolicy, TranspositionConfig};
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;

/// Replacement decision result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementDecision {
    /// Replace the existing entry
    Replace,
    /// Keep the existing entry
    Keep,
    /// Replace only if new entry is exact and existing is not
    ReplaceIfExact,
}

/// Advanced replacement policy handler
///
/// This struct provides sophisticated replacement logic based on various
/// criteria including depth, age, score bounds, and hash quality.
pub struct ReplacementPolicyHandler {
    /// Current replacement policy
    policy: ReplacementPolicy,
    /// Configuration for the policy
    config: TranspositionConfig,
    /// Statistics for policy performance
    stats: ReplacementStats,
}

/// Statistics for replacement policy performance
#[derive(Debug, Clone, Default)]
pub struct ReplacementStats {
    /// Number of replacement decisions made
    pub decisions_made: u64,
    /// Number of entries replaced
    pub entries_replaced: u64,
    /// Number of entries kept
    pub entries_kept: u64,
    /// Number of depth-preferred replacements
    pub depth_preferred_replacements: u64,
    /// Number of age-based replacements
    pub age_based_replacements: u64,
    /// Number of exact score replacements
    pub exact_replacements: u64,
    /// Number of bound replacements
    pub bound_replacements: u64,
}

impl ReplacementStats {
    /// Get the replacement rate (entries replaced / total decisions)
    pub fn replacement_rate(&self) -> f64 {
        if self.decisions_made == 0 {
            0.0
        } else {
            self.entries_replaced as f64 / self.decisions_made as f64
        }
    }

    /// Get the depth-preferred replacement rate
    pub fn depth_preferred_rate(&self) -> f64 {
        if self.entries_replaced == 0 {
            0.0
        } else {
            self.depth_preferred_replacements as f64 / self.entries_replaced as f64
        }
    }

    /// Get the age-based replacement rate
    pub fn age_based_rate(&self) -> f64 {
        if self.entries_replaced == 0 {
            0.0
        } else {
            self.age_based_replacements as f64 / self.entries_replaced as f64
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl ReplacementPolicyHandler {
    /// Create a new replacement policy handler
    pub fn new(config: TranspositionConfig) -> Self {
        Self {
            policy: config.replacement_policy,
            config,
            stats: ReplacementStats::default(),
        }
    }

    /// Determine whether to replace an existing entry with a new one
    ///
    /// This is the main method for making replacement decisions based on
    /// the configured policy and various criteria.
    pub fn should_replace_entry(
        &mut self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
        current_age: u32,
    ) -> ReplacementDecision {
        self.stats.decisions_made += 1;

        let decision = match self.policy {
            ReplacementPolicy::AlwaysReplace => ReplacementDecision::Replace,
            ReplacementPolicy::DepthPreferred => {
                self.should_replace_depth_preferred(existing, new_entry)
            }
            ReplacementPolicy::AgeBased => {
                self.should_replace_age_based(existing, new_entry, current_age)
            }
            ReplacementPolicy::DepthAndAge => {
                self.should_replace_depth_and_age(existing, new_entry, current_age)
            }
            ReplacementPolicy::ExactPreferred => {
                self.should_replace_exact_preferred(existing, new_entry)
            }
        };

        // Update statistics
        match decision {
            ReplacementDecision::Replace => {
                self.stats.entries_replaced += 1;
                self.update_replacement_stats(existing, new_entry);
            }
            ReplacementDecision::Keep => {
                self.stats.entries_kept += 1;
            }
            ReplacementDecision::ReplaceIfExact => {
                // This will be handled by the caller
                self.stats.entries_kept += 1;
            }
        }

        decision
    }

    /// Depth-preferred replacement logic
    ///
    /// Replace if the new entry has higher depth, or if depths are equal
    /// but the new entry has a better score bound.
    fn should_replace_depth_preferred(
        &mut self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
    ) -> ReplacementDecision {
        // Higher depth always wins
        if new_entry.depth > existing.depth {
            self.stats.depth_preferred_replacements += 1;
            return ReplacementDecision::Replace;
        }

        // Lower depth never wins
        if new_entry.depth < existing.depth {
            return ReplacementDecision::Keep;
        }

        // Same depth - compare score bounds
        if new_entry.depth == existing.depth {
            let new_bound_quality = self.get_bound_quality(new_entry);
            let existing_bound_quality = self.get_bound_quality(existing);

            if new_bound_quality > existing_bound_quality {
                self.stats.bound_replacements += 1;
                return ReplacementDecision::Replace;
            }
        }

        ReplacementDecision::Keep
    }

    /// Age-based replacement logic
    ///
    /// Replace if the new entry is significantly newer, or if the existing
    /// entry is very old.
    fn should_replace_age_based(
        &mut self,
        existing: &TranspositionEntry,
        _new_entry: &TranspositionEntry,
        current_age: u32,
    ) -> ReplacementDecision {
        let age_difference = self.age_delta(current_age, existing.age);

        // Replace very old entries
        if age_difference > self.config.max_age {
            self.stats.age_based_replacements += 1;
            return ReplacementDecision::Replace;
        }

        // Replace if new entry is significantly newer (more than half max_age)
        if age_difference > self.config.max_age / 2 {
            self.stats.age_based_replacements += 1;
            return ReplacementDecision::Replace;
        }

        // Keep recent entries
        ReplacementDecision::Keep
    }

    /// Combined depth and age replacement logic
    ///
    /// This is the most sophisticated policy that considers both depth
    /// and age with weighted scoring.
    fn should_replace_depth_and_age(
        &mut self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
        current_age: u32,
    ) -> ReplacementDecision {
        let existing_score = self.calculate_entry_score(existing, current_age);
        let new_score = self.calculate_entry_score(new_entry, current_age);

        if new_score > existing_score {
            // Determine which factor contributed most
            let depth_advantage = new_entry.depth as i32 - existing.depth as i32;
            let existing_age_gap = self.age_delta(current_age, existing.age) as i32;
            let new_age_gap = self.age_delta(current_age, new_entry.age) as i32;
            let age_advantage = existing_age_gap - new_age_gap;

            if depth_advantage > age_advantage {
                self.stats.depth_preferred_replacements += 1;
            } else {
                self.stats.age_based_replacements += 1;
            }

            ReplacementDecision::Replace
        } else {
            ReplacementDecision::Keep
        }
    }

    /// Exact score preferred replacement logic
    ///
    /// Replace only if the new entry is exact and the existing is not,
    /// or if both are exact but new has higher depth.
    fn should_replace_exact_preferred(
        &mut self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
    ) -> ReplacementDecision {
        let existing_is_exact = existing.is_exact();
        let new_is_exact = new_entry.is_exact();

        // Always replace non-exact with exact
        if !existing_is_exact && new_is_exact {
            return ReplacementDecision::Replace;
        }

        // Never replace exact with non-exact
        if existing_is_exact && !new_is_exact {
            return ReplacementDecision::Keep;
        }

        // Both exact - use depth preference
        if existing_is_exact && new_is_exact {
            if new_entry.depth > existing.depth {
                return ReplacementDecision::Replace;
            }
        }

        // Neither exact - use depth preference
        if !existing_is_exact && !new_is_exact {
            if new_entry.depth > existing.depth {
                self.stats.bound_replacements += 1;
                return ReplacementDecision::Replace;
            }
        }

        ReplacementDecision::Keep
    }

    /// Calculate a score for an entry based on depth and age
    ///
    /// Higher scores indicate better entries that should be kept.
    fn calculate_entry_score(&self, entry: &TranspositionEntry, current_age: u32) -> i32 {
        let depth_score = entry.depth as i32 * 1000;
        let age_gap = self.age_delta(current_age, entry.age) as i32;
        let age_score = (self.config.max_age as i32 - age_gap).max(0);
        let bound_score = self.get_bound_quality(entry) as i32 * 100;

        depth_score + age_score + bound_score
    }

    /// Get the quality of a bound (higher is better)
    fn get_bound_quality(&self, entry: &TranspositionEntry) -> u8 {
        match entry.flag {
            TranspositionFlag::Exact => 3,
            TranspositionFlag::LowerBound => 2,
            TranspositionFlag::UpperBound => 1,
        }
    }

    /// Update replacement statistics based on the type of replacement
    fn update_replacement_stats(
        &mut self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
    ) {
        if new_entry.is_exact() {
            self.stats.exact_replacements += 1;
        }

        if new_entry.is_exact() && !existing.is_exact() {
            self.stats.bound_replacements += 1;
        }
    }

    fn age_delta(&self, current_stamp: u32, entry_stamp: u32) -> u32 {
        let (current_wrap, current_age) = AgeCounter::decode_stamp(current_stamp);
        let (entry_wrap, entry_age_raw) = AgeCounter::decode_stamp(entry_stamp);

        if current_wrap < entry_wrap {
            return 0;
        }

        let entry_age = entry_age_raw.min(self.config.max_age);
        let wrap_diff = current_wrap - entry_wrap;

        if wrap_diff == 0 {
            current_age.saturating_sub(entry_age)
        } else {
            let mut diff = self
                .config
                .max_age
                .saturating_sub(entry_age)
                .saturating_add(current_age);

            if wrap_diff > 1 {
                diff = diff.saturating_add((wrap_diff - 1) * self.config.max_age);
            }

            diff
        }
    }

    /// Store an entry with depth-preferred replacement
    ///
    /// This method implements the store_depth_preferred functionality
    /// from the task requirements.
    pub fn store_depth_preferred(
        &mut self,
        table: &mut Vec<Option<TranspositionEntry>>,
        index: usize,
        new_entry: TranspositionEntry,
    ) -> bool {
        match &table[index] {
            Some(existing) => {
                let decision = self.should_replace_depth_preferred(existing, &new_entry);
                match decision {
                    ReplacementDecision::Replace => {
                        table[index] = Some(new_entry);
                        true
                    }
                    _ => false,
                }
            }
            None => {
                table[index] = Some(new_entry);
                true
            }
        }
    }

    /// Get current replacement statistics
    pub fn get_stats(&self) -> &ReplacementStats {
        &self.stats
    }

    /// Reset replacement statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }

    /// Update the replacement policy
    pub fn set_policy(&mut self, policy: ReplacementPolicy) {
        self.policy = policy;
    }

    /// Get the current replacement policy
    pub fn get_policy(&self) -> ReplacementPolicy {
        self.policy
    }

    /// Check if the policy considers depth in decisions
    pub fn considers_depth(&self) -> bool {
        self.policy.considers_depth()
    }

    /// Check if the policy considers age in decisions
    pub fn considers_age(&self) -> bool {
        self.policy.considers_age()
    }
}

/// Optimized replacement decision maker
///
/// This struct provides fast, optimized replacement decisions
/// with minimal overhead for high-frequency operations.
pub struct OptimizedReplacementMaker {
    /// Pre-computed thresholds for fast decisions
    depth_threshold: u8,
    age_threshold: u32,
    /// Current policy
    policy: ReplacementPolicy,
    max_age: u32,
}

impl OptimizedReplacementMaker {
    /// Create a new optimized replacement maker
    pub fn new(config: &TranspositionConfig) -> Self {
        Self {
            depth_threshold: 3,                // Replace if depth difference > 3
            age_threshold: config.max_age / 4, // Replace if age difference > max_age/4
            policy: config.replacement_policy,
            max_age: config.max_age,
        }
    }

    /// Fast replacement decision with minimal overhead
    pub fn quick_replace_decision(
        &self,
        existing: &TranspositionEntry,
        new_entry: &TranspositionEntry,
        current_age: u32,
    ) -> bool {
        match self.policy {
            ReplacementPolicy::AlwaysReplace => true,
            ReplacementPolicy::DepthPreferred => {
                new_entry.depth > existing.depth
                    || (new_entry.depth == existing.depth
                        && self.get_bound_quality(new_entry) > self.get_bound_quality(existing))
                    || (new_entry.depth as i32 - existing.depth as i32).abs()
                        <= self.depth_threshold as i32
                        && self.get_bound_quality(new_entry) > self.get_bound_quality(existing)
            }
            ReplacementPolicy::AgeBased => {
                self.age_delta(current_age, existing.age) > self.age_threshold
            }
            ReplacementPolicy::DepthAndAge => {
                let depth_advantage = new_entry.depth as i32 - existing.depth as i32;
                let age_advantage = self.age_delta(current_age, existing.age) as i32
                    - self.age_delta(current_age, new_entry.age) as i32;
                depth_advantage > 0 || age_advantage > self.age_threshold as i32
            }
            ReplacementPolicy::ExactPreferred => {
                let existing_exact = existing.is_exact();
                let new_exact = new_entry.is_exact();
                (!existing_exact && new_exact)
                    || (existing_exact && new_exact && new_entry.depth > existing.depth)
            }
        }
    }

    /// Get bound quality (0-3, higher is better)
    fn get_bound_quality(&self, entry: &TranspositionEntry) -> u8 {
        match entry.flag {
            TranspositionFlag::Exact => 3,
            TranspositionFlag::LowerBound => 2,
            TranspositionFlag::UpperBound => 1,
        }
    }

    fn age_delta(&self, current_stamp: u32, entry_stamp: u32) -> u32 {
        let (current_wrap, current_age) = AgeCounter::decode_stamp(current_stamp);
        let (entry_wrap, entry_age_raw) = AgeCounter::decode_stamp(entry_stamp);

        if current_wrap < entry_wrap {
            return 0;
        }

        let entry_age = entry_age_raw.min(self.max_age);
        let wrap_diff = current_wrap - entry_wrap;

        if wrap_diff == 0 {
            current_age.saturating_sub(entry_age)
        } else {
            let mut diff = self
                .max_age
                .saturating_sub(entry_age)
                .saturating_add(current_age);

            if wrap_diff > 1 {
                diff = diff.saturating_add((wrap_diff - 1) * self.max_age);
            }

            diff
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::transposition_config::TranspositionConfig;

    fn create_test_entry(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        age: u32,
    ) -> TranspositionEntry {
        let mut entry = TranspositionEntry::new_with_age(score, depth, flag, None, 0);
        entry.age = age;
        entry
    }

    fn create_test_config(policy: ReplacementPolicy) -> TranspositionConfig {
        let mut config = TranspositionConfig::debug_config();
        config.replacement_policy = policy;
        config
    }

    #[test]
    fn test_always_replace_policy() {
        let config = create_test_config(ReplacementPolicy::AlwaysReplace);
        let mut handler = ReplacementPolicyHandler::new(config);

        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 3, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30);
        assert_eq!(decision, ReplacementDecision::Replace);

        let stats = handler.get_stats();
        assert_eq!(stats.entries_replaced, 1);
        assert_eq!(stats.entries_kept, 0);
    }

    #[test]
    fn test_depth_preferred_policy() {
        let config = create_test_config(ReplacementPolicy::DepthPreferred);
        let mut handler = ReplacementPolicyHandler::new(config);

        // Higher depth should replace
        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 7, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30);
        assert_eq!(decision, ReplacementDecision::Replace);

        // Lower depth should not replace
        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 3, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30);
        assert_eq!(decision, ReplacementDecision::Keep);

        let stats = handler.get_stats();
        assert_eq!(stats.depth_preferred_replacements, 1);
    }

    #[test]
    fn test_age_based_policy() {
        let config = create_test_config(ReplacementPolicy::AgeBased);
        let mut handler = ReplacementPolicyHandler::new(config);

        // Old entry should be replaced
        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 3, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 150); // 140 age difference
        assert_eq!(decision, ReplacementDecision::Replace);

        // Recent entry should be kept
        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 3, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30); // 20 age difference
        assert_eq!(decision, ReplacementDecision::Keep);

        let stats = handler.get_stats();
        assert_eq!(stats.age_based_replacements, 1);
    }

    #[test]
    fn test_exact_preferred_policy() {
        let config = create_test_config(ReplacementPolicy::ExactPreferred);
        let mut handler = ReplacementPolicyHandler::new(config);

        // Exact should replace non-exact
        let existing = create_test_entry(100, 5, TranspositionFlag::LowerBound, 10);
        let new_entry = create_test_entry(200, 3, TranspositionFlag::Exact, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30);
        assert_eq!(decision, ReplacementDecision::Replace);

        // Non-exact should not replace exact
        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 7, TranspositionFlag::LowerBound, 20);

        let decision = handler.should_replace_entry(&existing, &new_entry, 30);
        assert_eq!(decision, ReplacementDecision::Keep);

        let stats = handler.get_stats();
        assert_eq!(stats.exact_replacements, 1);
    }

    #[test]
    fn test_store_depth_preferred() {
        let config = create_test_config(ReplacementPolicy::DepthPreferred);
        let mut handler = ReplacementPolicyHandler::new(config);
        let mut table = vec![None; 1000];

        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        table[0] = Some(existing);

        // Higher depth should replace
        let new_entry = create_test_entry(200, 7, TranspositionFlag::LowerBound, 20);
        let replaced = handler.store_depth_preferred(&mut table, 0, new_entry);
        assert!(replaced);
        assert!(table[0].is_some());

        // Lower depth should not replace
        let new_entry = create_test_entry(300, 3, TranspositionFlag::UpperBound, 30);
        let replaced = handler.store_depth_preferred(&mut table, 0, new_entry);
        assert!(!replaced);
    }

    #[test]
    fn test_optimized_replacement_maker() {
        let config = create_test_config(ReplacementPolicy::DepthPreferred);
        let maker = OptimizedReplacementMaker::new(&config);

        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 7, TranspositionFlag::LowerBound, 20);

        let should_replace = maker.quick_replace_decision(&existing, &new_entry, 30);
        assert!(should_replace);

        let new_entry = create_test_entry(200, 3, TranspositionFlag::LowerBound, 20);
        let should_replace = maker.quick_replace_decision(&existing, &new_entry, 30);
        assert!(!should_replace);
    }

    #[test]
    fn test_replacement_stats() {
        let config = create_test_config(ReplacementPolicy::DepthPreferred);
        let mut handler = ReplacementPolicyHandler::new(config);

        let existing = create_test_entry(100, 5, TranspositionFlag::Exact, 10);
        let new_entry = create_test_entry(200, 7, TranspositionFlag::LowerBound, 20);

        handler.should_replace_entry(&existing, &new_entry, 30);

        let stats = handler.get_stats();
        assert_eq!(stats.decisions_made, 1);
        assert_eq!(stats.entries_replaced, 1);
        assert_eq!(stats.replacement_rate(), 1.0);
        assert_eq!(stats.depth_preferred_rate(), 1.0);

        // Reset and verify
        handler.reset_stats();
        let stats = handler.get_stats();
        assert_eq!(stats.decisions_made, 0);
    }
}
