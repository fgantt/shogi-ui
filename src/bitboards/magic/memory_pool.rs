//! Memory pool for efficient allocation of attack tables in magic bitboards
//! 
//! This module provides a memory pool implementation optimized for allocating
//! attack pattern tables for magic bitboards. It uses pre-allocated blocks
//! to reduce memory fragmentation and improve cache locality.

use crate::types::{Bitboard, MemoryPool, MagicError, EMPTY_BITBOARD};

impl MemoryPool {
    /// Create a new memory pool with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new memory pool with custom block size
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            blocks: Vec::new(),
            current_block: 0,
            current_offset: 0,
            block_size,
        }
    }

    /// Allocate memory for attack table
    /// 
    /// Returns the base index where the allocated memory starts
    pub fn allocate(&mut self, size: usize) -> Result<usize, MagicError> {
        // Check if current block has enough space
        if self.current_offset + size <= self.block_size {
            let offset = self.current_offset;
            self.current_offset += size;
            return Ok(offset);
        }

        // Allocate new block
        self.allocate_new_block()?;
        self.current_offset = size;
        Ok(0)
    }

    /// Allocate new memory block
    fn allocate_new_block(&mut self) -> Result<(), MagicError> {
        let block = vec![EMPTY_BITBOARD; self.block_size];
        self.blocks.push(block);
        self.current_block = self.blocks.len() - 1;
        Ok(())
    }

    /// Get the total number of allocated blocks
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Get the total allocated memory in bytes
    pub fn total_allocated_bytes(&self) -> usize {
        self.blocks.len() * self.block_size * std::mem::size_of::<Bitboard>()
    }

    /// Get the current block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get the current offset within the current block
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Clear all allocated memory
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.current_block = 0;
        self.current_offset = 0;
    }

    /// Reserve space for a specific number of attack patterns
    /// 
    /// This is useful for pre-allocating space when the total size is known
    pub fn reserve(&mut self, total_patterns: usize) -> Result<(), MagicError> {
        let blocks_needed = (total_patterns + self.block_size - 1) / self.block_size;
        
        for _ in 0..blocks_needed {
            self.allocate_new_block()?;
        }
        
        Ok(())
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_blocks: self.blocks.len(),
            current_block: self.current_block,
            current_offset: self.current_offset,
            block_size: self.block_size,
            total_allocated_bytes: self.total_allocated_bytes(),
            utilization_percentage: if self.blocks.is_empty() {
                0.0
            } else {
                (self.current_offset as f64 / self.block_size as f64) * 100.0
            },
        }
    }
}

/// Memory usage statistics for the memory pool
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_blocks: usize,
    pub current_block: usize,
    pub current_offset: usize,
    pub block_size: usize,
    pub total_allocated_bytes: usize,
    pub utilization_percentage: f64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            current_block: 0,
            current_offset: 0,
            block_size: 0,
            total_allocated_bytes: 0,
            utilization_percentage: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.block_count(), 0);
        assert_eq!(pool.total_allocated_bytes(), 0);
    }

    #[test]
    fn test_memory_pool_custom_block_size() {
        let pool = MemoryPool::with_block_size(1024);
        assert_eq!(pool.block_size(), 1024);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_memory_allocation() {
        let mut pool = MemoryPool::with_block_size(100);
        
        // Allocate within first block
        let offset1 = pool.allocate(50).unwrap();
        assert_eq!(offset1, 0);
        assert_eq!(pool.current_offset(), 50);
        assert_eq!(pool.block_count(), 1);

        // Allocate more within first block
        let offset2 = pool.allocate(30).unwrap();
        assert_eq!(offset2, 50);
        assert_eq!(pool.current_offset(), 80);
        assert_eq!(pool.block_count(), 1);

        // Allocate beyond first block
        let offset3 = pool.allocate(50).unwrap();
        assert_eq!(offset3, 0); // New block starts at 0
        assert_eq!(pool.current_offset(), 50);
        assert_eq!(pool.block_count(), 2);
    }

    #[test]
    fn test_memory_pool_clear() {
        let mut pool = MemoryPool::with_block_size(100);
        pool.allocate(50).unwrap();
        assert_eq!(pool.block_count(), 1);
        
        pool.clear();
        assert!(pool.is_empty());
        assert_eq!(pool.block_count(), 0);
    }

    #[test]
    fn test_memory_stats() {
        let mut pool = MemoryPool::with_block_size(100);
        pool.allocate(75).unwrap();
        
        let stats = pool.memory_stats();
        assert_eq!(stats.total_blocks, 1);
        assert_eq!(stats.current_offset, 75);
        assert_eq!(stats.block_size, 100);
        assert_eq!(stats.utilization_percentage, 75.0);
    }

    #[test]
    fn test_reserve() {
        let mut pool = MemoryPool::with_block_size(100);
        pool.reserve(250).unwrap();
        
        assert_eq!(pool.block_count(), 3); // 250 / 100 = 3 blocks
        assert_eq!(pool.current_offset(), 0); // No allocation yet
    }
}
