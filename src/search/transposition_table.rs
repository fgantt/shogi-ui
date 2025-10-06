use crate::types::*;

/// Basic transposition table for caching search results
/// 
/// This struct provides a hash table-based cache for storing and retrieving
/// transposition table entries. It supports configurable size, replacement policies,
/// and comprehensive statistics tracking.
pub struct TranspositionTable {
    /// The actual hash table storing entries
    entries: Vec<Option<TranspositionEntry>>,
    /// Size of the table (number of slots)
    size: usize,
    /// Current age counter for replacement policies
    age: u32,
    /// Hit counter for statistics
    hits: u64,
    /// Miss counter for statistics
    misses: u64,
    /// Memory usage in bytes
    memory_usage: usize,
    /// Configuration for the table
    config: TranspositionTableConfig,
}

/// Configuration options for the transposition table
#[derive(Debug, Clone)]
pub struct TranspositionTableConfig {
    /// Maximum number of entries in the table
    pub max_entries: usize,
    /// Replacement policy to use
    pub replacement_policy: ReplacementPolicy,
    /// Whether to enable memory usage tracking
    pub track_memory: bool,
    /// Whether to enable hit/miss statistics
    pub track_statistics: bool,
}

/// Replacement policies for the transposition table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementPolicy {
    /// Always replace (simple replacement)
    AlwaysReplace,
    /// Replace based on depth preference
    DepthPreferred,
    /// Replace based on age (newest wins)
    AgeBased,
    /// Replace based on combination of depth and age
    DepthAgeCombined,
}

impl Default for TranspositionTableConfig {
    fn default() -> Self {
        Self {
            max_entries: 1_000_000, // 1 million entries by default
            replacement_policy: ReplacementPolicy::DepthPreferred,
            track_memory: true,
            track_statistics: true,
        }
    }
}

impl TranspositionTable {
    /// Create a new transposition table with default configuration
    pub fn new() -> Self {
        Self::with_config(TranspositionTableConfig::default())
    }
    
    /// Create a new transposition table with specified size
    pub fn with_size(size: usize) -> Self {
        let mut config = TranspositionTableConfig::default();
        config.max_entries = size;
        Self::with_config(config)
    }
    
    /// Create a new transposition table with custom configuration
    pub fn with_config(config: TranspositionTableConfig) -> Self {
        let size = config.max_entries;
        let memory_usage = size * std::mem::size_of::<Option<TranspositionEntry>>();
        
        Self {
            entries: vec![None; size],
            size,
            age: 0,
            hits: 0,
            misses: 0,
            memory_usage,
            config,
        }
    }
    
    /// Probe the table for an entry with the given hash key
    pub fn probe(&mut self, hash_key: u64, depth: u8) -> Option<TranspositionEntry> {
        let index = self.hash_to_index(hash_key);
        
        if let Some(entry) = &self.entries[index] {
            // Check if the entry matches our hash key and is valid for the depth
            if entry.matches_hash(hash_key) && entry.is_valid_for_depth(depth) {
                if self.config.track_statistics {
                    self.hits += 1;
                }
                return Some(entry.clone());
            }
        }
        
        if self.config.track_statistics {
            self.misses += 1;
        }
        None
    }
    
    /// Store an entry in the transposition table
    pub fn store(&mut self, mut entry: TranspositionEntry) {
        // Update the entry's age and hash key
        entry.age = self.age;
        entry.hash_key = self.get_hash_key(&entry);
        
        let index = self.hash_to_index(entry.hash_key);
        
        // Apply replacement policy
        if let Some(existing_entry) = &self.entries[index] {
            if !self.should_replace(existing_entry, &entry) {
                return; // Don't replace
            }
        }
        
        self.entries[index] = Some(entry);
    }
    
    /// Store an entry with explicit hash key
    pub fn store_with_hash(&mut self, hash_key: u64, entry: TranspositionEntry) {
        let mut entry = entry;
        entry.hash_key = hash_key;
        entry.age = self.age;
        self.store(entry);
    }
    
    /// Clear all entries from the table
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.age = 0;
        if self.config.track_statistics {
            self.hits = 0;
            self.misses = 0;
        }
    }
    
    /// Increment the age counter
    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
    
    /// Get the current age
    pub fn get_age(&self) -> u32 {
        self.age
    }
    
    /// Get hit rate as a percentage
    pub fn get_hit_rate(&self) -> f64 {
        if self.config.track_statistics {
            let total = self.hits + self.misses;
            if total > 0 {
                (self.hits as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Get hit and miss counts
    pub fn get_statistics(&self) -> (u64, u64, f64) {
        if self.config.track_statistics {
            (self.hits, self.misses, self.get_hit_rate())
        } else {
            (0, 0, 0.0)
        }
    }
    
    /// Get memory usage in bytes
    pub fn get_memory_usage(&self) -> usize {
        if self.config.track_memory {
            self.memory_usage
        } else {
            0
        }
    }
    
    /// Get the number of entries currently stored
    pub fn get_entry_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_some()).count()
    }
    
    /// Get the table size
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    /// Check if the table is full (all slots occupied)
    pub fn is_full(&self) -> bool {
        self.get_entry_count() >= self.size
    }
    
    /// Get the fill percentage
    pub fn get_fill_percentage(&self) -> f64 {
        (self.get_entry_count() as f64 / self.size as f64) * 100.0
    }
    
    /// Resize the table to a new size
    pub fn resize(&mut self, new_size: usize) {
        let mut new_config = self.config.clone();
        new_config.max_entries = new_size;
        
        // Create new table with new size
        let mut new_table = Self::with_config(new_config);
        new_table.age = self.age;
        
        // Copy all valid entries to the new table
        for entry in self.entries.iter().flatten() {
            new_table.store(entry.clone());
        }
        
        // Replace current table with new one
        *self = new_table;
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &TranspositionTableConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, new_config: TranspositionTableConfig) {
        self.config = new_config;
        if !self.config.track_statistics {
            self.hits = 0;
            self.misses = 0;
        }
        if !self.config.track_memory {
            self.memory_usage = 0;
        }
    }
    
    // Private helper methods
    
    /// Convert hash key to table index using fast modulo
    fn hash_to_index(&self, hash_key: u64) -> usize {
        // Use bit masking for power-of-2 sizes, otherwise use modulo
        if self.size.is_power_of_two() {
            (hash_key as usize) & (self.size - 1)
        } else {
            (hash_key as usize) % self.size
        }
    }
    
    /// Get hash key for an entry (placeholder - will be enhanced in later tasks)
    fn get_hash_key(&self, _entry: &TranspositionEntry) -> u64 {
        // For now, just use the entry's existing hash key
        // This will be enhanced when we integrate with Zobrist hashing
        0
    }
    
    /// Determine if an existing entry should be replaced
    fn should_replace(&self, existing: &TranspositionEntry, new: &TranspositionEntry) -> bool {
        match self.config.replacement_policy {
            ReplacementPolicy::AlwaysReplace => true,
            ReplacementPolicy::DepthPreferred => new.depth >= existing.depth,
            ReplacementPolicy::AgeBased => new.age > existing.age,
            ReplacementPolicy::DepthAgeCombined => {
                // Prefer deeper searches, then newer entries
                if new.depth > existing.depth {
                    true
                } else if new.depth == existing.depth {
                    new.age > existing.age
                } else {
                    false
                }
            }
        }
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transposition_table_creation() {
        let table = TranspositionTable::new();
        assert_eq!(table.get_size(), 1_000_000);
        assert_eq!(table.get_entry_count(), 0);
        assert!(!table.is_full());
        assert_eq!(table.get_fill_percentage(), 0.0);
    }
    
    #[test]
    fn test_transposition_table_with_size() {
        let table = TranspositionTable::with_size(1000);
        assert_eq!(table.get_size(), 1000);
        assert_eq!(table.get_entry_count(), 0);
    }
    
    #[test]
    fn test_transposition_table_with_config() {
        let mut config = TranspositionTableConfig::default();
        config.max_entries = 500;
        config.replacement_policy = ReplacementPolicy::AgeBased;
        
        let table = TranspositionTable::with_config(config);
        assert_eq!(table.get_size(), 500);
        assert_eq!(table.get_config().replacement_policy, ReplacementPolicy::AgeBased);
    }
    
    #[test]
    fn test_probe_empty_table() {
        let mut table = TranspositionTable::with_size(100);
        let result = table.probe(0x1234567890ABCDEF, 5);
        assert!(result.is_none());
        assert_eq!(table.get_statistics(), (0, 1, 0.0));
    }
    
    #[test]
    fn test_store_and_probe() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry.clone());
        assert_eq!(table.get_entry_count(), 1);
        
        let retrieved = table.probe(0x1234567890ABCDEF, 5);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.score, 100);
        assert_eq!(retrieved.depth, 5);
        assert_eq!(retrieved.flag, TranspositionFlag::Exact);
        
        assert_eq!(table.get_statistics(), (1, 0, 100.0));
    }
    
    #[test]
    fn test_probe_with_insufficient_depth() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 3, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        assert_eq!(table.get_entry_count(), 1);
        
        // Probe with higher depth requirement - should not find
        let result = table.probe(0x1234567890ABCDEF, 5);
        assert!(result.is_none());
        assert_eq!(table.get_statistics(), (0, 1, 0.0));
        
        // Probe with same or lower depth - should find
        let result = table.probe(0x1234567890ABCDEF, 3);
        assert!(result.is_some());
        assert_eq!(table.get_statistics(), (1, 1, 50.0));
    }
    
    #[test]
    fn test_probe_with_hash_mismatch() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        
        // Probe with different hash key - should not find
        let result = table.probe(0xFEDCBA0987654321, 5);
        assert!(result.is_none());
        assert_eq!(table.get_statistics(), (0, 1, 0.0));
    }
    
    #[test]
    fn test_store_with_hash() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0
        );
        
        table.store_with_hash(0x1234567890ABCDEF, entry);
        
        let retrieved = table.probe(0x1234567890ABCDEF, 5);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.score, 100);
    }
    
    #[test]
    fn test_clear() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        assert_eq!(table.get_entry_count(), 1);
        
        table.clear();
        assert_eq!(table.get_entry_count(), 0);
        assert_eq!(table.get_age(), 0);
        assert_eq!(table.get_statistics(), (0, 0, 0.0));
    }
    
    #[test]
    fn test_increment_age() {
        let mut table = TranspositionTable::new();
        
        assert_eq!(table.get_age(), 0);
        table.increment_age();
        assert_eq!(table.get_age(), 1);
        table.increment_age();
        assert_eq!(table.get_age(), 2);
    }
    
    #[test]
    fn test_replacement_policies() {
        // Test AlwaysReplace
        let mut config = TranspositionTableConfig::default();
        config.max_entries = 1; // Force collision
        config.replacement_policy = ReplacementPolicy::AlwaysReplace;
        
        let mut table = TranspositionTable::with_config(config);
        
        let entry1 = TranspositionEntry::new_with_age(
            100, 3, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        let entry2 = TranspositionEntry::new_with_age(
            200, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry1);
        table.store(entry2);
        
        let result = table.probe(0x1234567890ABCDEF, 3);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.score, 200); // Should be the second entry
    }
    
    #[test]
    fn test_depth_preferred_replacement() {
        let mut config = TranspositionTableConfig::default();
        config.max_entries = 1; // Force collision
        config.replacement_policy = ReplacementPolicy::DepthPreferred;
        
        let mut table = TranspositionTable::with_config(config);
        
        let entry1 = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        let entry2 = TranspositionEntry::new_with_age(
            200, 3, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry1.clone());
        table.store(entry2);
        
        let result = table.probe(0x1234567890ABCDEF, 3);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.score, 100); // Should keep the deeper entry
    }
    
    #[test]
    fn test_memory_usage_tracking() {
        let table = TranspositionTable::with_size(1000);
        let memory_usage = table.get_memory_usage();
        assert!(memory_usage > 0);
        assert!(memory_usage < 1_000_000); // Should be reasonable
    }
    
    #[test]
    fn test_resize() {
        let mut table = TranspositionTable::with_size(100);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        assert_eq!(table.get_size(), 100);
        assert_eq!(table.get_entry_count(), 1);
        
        table.resize(200);
        assert_eq!(table.get_size(), 200);
        assert_eq!(table.get_entry_count(), 1);
        
        // Verify entry is still accessible
        let result = table.probe(0x1234567890ABCDEF, 5);
        assert!(result.is_some());
    }
    
    #[test]
    fn test_hash_to_index_power_of_two() {
        let table = TranspositionTable::with_size(1024); // Power of 2
        
        let hash1 = 0x1234567890ABCDEF;
        let hash2 = 0x1234567890ABCDEF + 1024; // Same index after masking
        
        let index1 = table.hash_to_index(hash1);
        let index2 = table.hash_to_index(hash2);
        
        assert_eq!(index1, index2); // Should map to same index
    }
    
    #[test]
    fn test_hash_to_index_non_power_of_two() {
        let table = TranspositionTable::with_size(1000); // Not power of 2
        
        let hash1 = 0x1234567890ABCDEF;
        let index1 = table.hash_to_index(hash1);
        
        assert!(index1 < 1000); // Should be within bounds
    }
    
    #[test]
    fn test_fill_percentage() {
        let mut table = TranspositionTable::with_size(10);
        
        assert_eq!(table.get_fill_percentage(), 0.0);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        assert_eq!(table.get_fill_percentage(), 10.0);
        
        let entry2 = TranspositionEntry::new_with_age(
            200, 5, TranspositionFlag::Exact, None, 0xFEDCBA0987654321
        );
        
        table.store(entry2);
        assert_eq!(table.get_fill_percentage(), 20.0);
    }
    
    #[test]
    fn test_statistics_tracking_disabled() {
        let mut config = TranspositionTableConfig::default();
        config.track_statistics = false;
        
        let mut table = TranspositionTable::with_config(config);
        
        let entry = TranspositionEntry::new_with_age(
            100, 5, TranspositionFlag::Exact, None, 0x1234567890ABCDEF
        );
        
        table.store(entry);
        table.probe(0x1234567890ABCDEF, 5);
        
        assert_eq!(table.get_statistics(), (0, 0, 0.0));
    }
    
    #[test]
    fn test_configuration_update() {
        let mut table = TranspositionTable::new();
        
        let mut new_config = TranspositionTableConfig::default();
        new_config.replacement_policy = ReplacementPolicy::AgeBased;
        new_config.track_statistics = false;
        
        table.update_config(new_config);
        
        assert_eq!(table.get_config().replacement_policy, ReplacementPolicy::AgeBased);
        assert!(!table.get_config().track_statistics);
        assert_eq!(table.get_statistics(), (0, 0, 0.0));
    }
}
