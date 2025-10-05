//! Cache optimization module for bit-scanning operations
//! 
//! This module provides cache-aware optimizations for bitboard operations,
//! including memory alignment, prefetching, and cache-friendly data structures.
//! These optimizations are crucial for achieving maximum performance in
//! memory-intensive bit-scanning operations.

use crate::types::Bitboard;
// Removed unused imports
use std::sync::atomic::{AtomicBool, Ordering};

/// Cache line size in bytes (64 bytes for most modern processors)
pub const CACHE_LINE_SIZE: usize = 64;

/// Memory alignment for cache optimization
pub const CACHE_ALIGNED_SIZE: usize = 64;

/// Global flag to enable/disable prefetching optimizations
static PREFETCH_ENABLED: AtomicBool = AtomicBool::new(true);

/// Cache-aligned lookup table for 4-bit population counts
/// 
/// This structure ensures that the lookup table is aligned to cache lines
/// for optimal memory access performance.
#[repr(align(64))]
pub struct CacheAlignedPopcountTable {
    /// 4-bit population count lookup table
    pub table: [u8; 16],
    /// Padding to ensure cache line alignment
    _padding: [u8; 48],
}

impl CacheAlignedPopcountTable {
    /// Create a new cache-aligned population count table
    pub fn new() -> Self {
        Self {
            table: [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4],
            _padding: [0; 48],
        }
    }
    
    /// Get population count for a 4-bit value
    pub fn get_popcount(&self, value: u8) -> u8 {
        self.table[value as usize & 0xF]
    }
}

/// Cache-aligned lookup table for bit positions
/// 
/// This structure provides cache-aligned storage for bit position lookup tables.
#[repr(align(64))]
pub struct CacheAlignedBitPositionTable {
    /// Bit positions for each 4-bit pattern
    pub positions: [[u8; 4]; 16],
    /// Padding to ensure cache line alignment
    _padding: [u8; 0],
}

impl CacheAlignedBitPositionTable {
    /// Create a new cache-aligned bit position table
    pub fn new() -> Self {
        Self {
            positions: [
                [0, 0, 0, 0], // 0000
                [0, 0, 0, 0], // 0001
                [1, 0, 0, 0], // 0010
                [0, 1, 0, 0], // 0011
                [2, 0, 0, 0], // 0100
                [0, 2, 0, 0], // 0101
                [1, 2, 0, 0], // 0110
                [0, 1, 2, 0], // 0111
                [3, 0, 0, 0], // 1000
                [0, 3, 0, 0], // 1001
                [1, 3, 0, 0], // 1010
                [0, 1, 3, 0], // 1011
                [2, 3, 0, 0], // 1100
                [0, 2, 3, 0], // 1101
                [1, 2, 3, 0], // 1110
                [0, 1, 2, 3], // 1111
            ],
            _padding: [],
        }
    }
    
    /// Get bit positions for a 4-bit value
    pub fn get_positions(&self, value: u8) -> &[u8; 4] {
        &self.positions[value as usize & 0xF]
    }
}

/// Cache-aligned rank mask table for Shogi board
/// 
/// This structure provides cache-aligned storage for rank masks.
#[repr(align(64))]
pub struct CacheAlignedRankMasks {
    /// Rank masks for each rank (0-8)
    pub masks: [Bitboard; 9],
    /// Padding to ensure cache line alignment
    _padding: [u8; 56],
}

impl CacheAlignedRankMasks {
    /// Create a new cache-aligned rank mask table
    pub fn new() -> Self {
        let mut masks = [0u128; 9];
        
        // Generate rank masks for 9x9 board
        for rank in 0..9 {
            let mut mask = 0u128;
            for file in 0..9 {
                let bit_pos = rank * 9 + file;
                mask |= 1u128 << bit_pos;
            }
            masks[rank] = mask;
        }
        
        Self {
            masks,
            _padding: [0; 56],
        }
    }
    
    /// Get rank mask for a specific rank
    pub fn get_rank_mask(&self, rank: u8) -> Bitboard {
        if rank < 9 {
            self.masks[rank as usize]
        } else {
            0
        }
    }
}

/// Cache-aligned file mask table for Shogi board
/// 
/// This structure provides cache-aligned storage for file masks.
#[repr(align(64))]
pub struct CacheAlignedFileMasks {
    /// File masks for each file (0-8)
    pub masks: [Bitboard; 9],
    /// Padding to ensure cache line alignment
    _padding: [u8; 56],
}

impl CacheAlignedFileMasks {
    /// Create a new cache-aligned file mask table
    pub fn new() -> Self {
        let mut masks = [0u128; 9];
        
        // Generate file masks for 9x9 board
        for file in 0..9 {
            let mut mask = 0u128;
            for rank in 0..9 {
                let bit_pos = rank * 9 + file;
                mask |= 1u128 << bit_pos;
            }
            masks[file] = mask;
        }
        
        Self {
            masks,
            _padding: [0; 56],
        }
    }
    
    /// Get file mask for a specific file
    pub fn get_file_mask(&self, file: u8) -> Bitboard {
        if file < 9 {
            self.masks[file as usize]
        } else {
            0
        }
    }
}

lazy_static::lazy_static! {
    static ref CACHE_ALIGNED_POPCOUNT_TABLE: CacheAlignedPopcountTable = 
        CacheAlignedPopcountTable::new();
    
    static ref CACHE_ALIGNED_BIT_POSITION_TABLE: CacheAlignedBitPositionTable = 
        CacheAlignedBitPositionTable::new();
    
    static ref CACHE_ALIGNED_RANK_MASKS: CacheAlignedRankMasks = 
        CacheAlignedRankMasks::new();
    
    static ref CACHE_ALIGNED_FILE_MASKS: CacheAlignedFileMasks = 
        CacheAlignedFileMasks::new();
}

/// Prefetch a bitboard into CPU cache
/// 
/// This function provides hints to the CPU to prefetch the bitboard data
/// into cache before it's needed, reducing memory access latency.
/// 
/// # Arguments
/// * `bb` - The bitboard to prefetch
/// 
/// # Safety
/// This function is safe as it only provides prefetch hints and doesn't
/// modify memory or cause undefined behavior.
#[cfg(all(not(target_arch = "wasm32"), target_arch = "x86_64"))]
pub unsafe fn prefetch_bitboard(bb: Bitboard) {
    if PREFETCH_ENABLED.load(Ordering::Relaxed) {
        // Prefetch for read access
        std::arch::x86_64::_mm_prefetch(
            &bb as *const _ as *const i8,
            std::arch::x86_64::_MM_HINT_T0,
        );
    }
}

/// Generic prefetch function for non-x86_64 platforms (no-op)
#[cfg(not(all(not(target_arch = "wasm32"), target_arch = "x86_64")))]
pub unsafe fn prefetch_bitboard(_bb: Bitboard) {
    // No-op for non-x86_64 platforms - prefetching not available
}

/// Prefetch multiple bitboards into CPU cache
/// 
/// This function prefetches a sequence of bitboards to improve
/// memory access patterns for batch operations.
/// 
/// # Arguments
/// * `bitboards` - Slice of bitboards to prefetch
/// 
/// # Safety
/// This function is safe as it only provides prefetch hints.
pub unsafe fn prefetch_bitboard_sequence(bitboards: &[Bitboard]) {
    if PREFETCH_ENABLED.load(Ordering::Relaxed) {
        // Prefetch up to 4 bitboards ahead
        let prefetch_limit = std::cmp::min(bitboards.len(), 4);
        for i in 0..prefetch_limit {
            if let Some(&bb) = bitboards.get(i) {
                prefetch_bitboard(bb);
            }
        }
    }
}

/// Process a sequence of bitboards with cache optimization
/// 
/// This function processes multiple bitboards with prefetching and
/// cache-friendly access patterns for optimal performance.
/// 
/// # Arguments
/// * `bitboards` - Slice of bitboards to process
/// 
/// # Returns
/// Vector of population counts for each bitboard
/// 
/// # Examples
/// ```
/// use shogi_engine::bitboards::cache_opt::process_bitboard_sequence;
/// 
/// let bitboards = vec![0b1010, 0b1100, 0b1111];
/// let counts = unsafe { process_bitboard_sequence(&bitboards) };
/// assert_eq!(counts, vec![2, 2, 4]);
/// ```
pub unsafe fn process_bitboard_sequence(bitboards: &[Bitboard]) -> Vec<u32> {
    let mut results = Vec::with_capacity(bitboards.len());
    
    for (i, &bb) in bitboards.iter().enumerate() {
        // Prefetch next bitboards
        if i + 4 < bitboards.len() {
            prefetch_bitboard_sequence(&bitboards[i + 1..i + 5]);
        }
        
        // Process current bitboard
        let count = popcount_cache_optimized(bb);
        results.push(count);
    }
    
    results
}

/// Cache-optimized population count using aligned lookup table
/// 
/// This function uses the cache-aligned lookup table for improved
/// memory access performance.
/// 
/// # Arguments
/// * `bb` - The bitboard to count
/// 
/// # Returns
/// The number of set bits in the bitboard
pub fn popcount_cache_optimized(bb: Bitboard) -> u32 {
    let mut count = 0u32;
    
    // Process 64-bit chunks with cache-aligned lookup
    let low = bb as u64;
    let high = (bb >> 64) as u64;
    
    // Process low 64 bits in 4-bit chunks
    for i in 0..16 {
        let chunk = ((low >> (i * 4)) & 0xF) as u8;
        count += CACHE_ALIGNED_POPCOUNT_TABLE.get_popcount(chunk) as u32;
    }
    
    // Process high 64 bits in 4-bit chunks
    for i in 0..16 {
        let chunk = ((high >> (i * 4)) & 0xF) as u8;
        count += CACHE_ALIGNED_POPCOUNT_TABLE.get_popcount(chunk) as u32;
    }
    
    count
}

/// Cache-optimized bit position enumeration
/// 
/// This function uses cache-aligned lookup tables for improved
/// performance when enumerating bit positions.
/// 
/// # Arguments
/// * `bb` - The bitboard to analyze
/// 
/// # Returns
/// Vector of bit positions
pub fn get_bit_positions_cache_optimized(bb: Bitboard) -> Vec<u8> {
    let mut positions = Vec::new();
    
    // Process 64-bit chunks
    let low = bb as u64;
    let high = (bb >> 64) as u64;
    
    // Process low 64 bits
    for i in 0..16 {
        let chunk = ((low >> (i * 4)) & 0xF) as u8;
        if chunk != 0 {
            let chunk_positions = CACHE_ALIGNED_BIT_POSITION_TABLE.get_positions(chunk);
            for &pos in chunk_positions.iter() {
                if pos < 4 {
                    positions.push((i * 4 + pos) as u8);
                }
            }
        }
    }
    
    // Process high 64 bits
    for i in 0..16 {
        let chunk = ((high >> (i * 4)) & 0xF) as u8;
        if chunk != 0 {
            let chunk_positions = CACHE_ALIGNED_BIT_POSITION_TABLE.get_positions(chunk);
            for &pos in chunk_positions.iter() {
                if pos < 4 {
                    positions.push((64 + i * 4 + pos) as u8);
                }
            }
        }
    }
    
    positions
}

/// Memory layout optimization utilities
pub mod layout {
    use super::*;
    
    /// Align a value to cache line boundary
    /// 
    /// # Arguments
    /// * `value` - The value to align
    /// 
    /// # Returns
    /// The aligned value
    pub fn align_to_cache_line(value: usize) -> usize {
        (value + CACHE_LINE_SIZE - 1) & !(CACHE_LINE_SIZE - 1)
    }
    
    /// Check if a value is cache-aligned
    /// 
    /// # Arguments
    /// * `value` - The value to check
    /// 
    /// # Returns
    /// True if the value is cache-aligned
    pub fn is_cache_aligned(value: usize) -> bool {
        value & (CACHE_LINE_SIZE - 1) == 0
    }
    
    /// Get the cache line offset of a value
    /// 
    /// # Arguments
    /// * `value` - The value to analyze
    /// 
    /// # Returns
    /// The offset within the cache line (0-63)
    pub fn cache_line_offset(value: usize) -> usize {
        value & (CACHE_LINE_SIZE - 1)
    }
}

/// Prefetch control functions
pub mod prefetch {
    use super::*;
    
    /// Enable prefetching optimizations
    pub fn enable_prefetch() {
        PREFETCH_ENABLED.store(true, Ordering::Relaxed);
    }
    
    /// Disable prefetching optimizations
    pub fn disable_prefetch() {
        PREFETCH_ENABLED.store(false, Ordering::Relaxed);
    }
    
    /// Check if prefetching is enabled
    pub fn is_prefetch_enabled() -> bool {
        PREFETCH_ENABLED.load(Ordering::Relaxed)
    }
}

/// Performance benchmarking for cache optimizations
pub mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark cache-optimized vs standard population count
    /// 
    /// # Arguments
    /// * `bitboards` - Test bitboards
    /// * `iterations` - Number of iterations
    /// 
    /// # Returns
    /// Tuple of (cache_optimized_time, standard_time)
    pub fn benchmark_popcount(bitboards: &[Bitboard], iterations: usize) -> (u64, u64) {
        // Benchmark cache-optimized version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in bitboards {
                let _ = popcount_cache_optimized(bb);
            }
        }
        let cache_optimized_time = start.elapsed().as_nanos() as u64;
        
        // Benchmark standard version (simplified)
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in bitboards {
                let _ = bb.count_ones();
            }
        }
        let standard_time = start.elapsed().as_nanos() as u64;
        
        (cache_optimized_time, standard_time)
    }
    
    /// Benchmark prefetching effectiveness
    /// 
    /// # Arguments
    /// * `bitboards` - Test bitboards
    /// * `iterations` - Number of iterations
    /// 
    /// # Returns
    /// Tuple of (with_prefetch_time, without_prefetch_time)
    pub fn benchmark_prefetching(bitboards: &[Bitboard], iterations: usize) -> (u64, u64) {
        // Benchmark with prefetching
        prefetch::enable_prefetch();
        let start = Instant::now();
        for _ in 0..iterations {
            unsafe {
                let _ = process_bitboard_sequence(bitboards);
            }
        }
        let with_prefetch_time = start.elapsed().as_nanos() as u64;
        
        // Benchmark without prefetching
        prefetch::disable_prefetch();
        let start = Instant::now();
        for _ in 0..iterations {
            unsafe {
                let _ = process_bitboard_sequence(bitboards);
            }
        }
        let without_prefetch_time = start.elapsed().as_nanos() as u64;
        
        // Re-enable prefetching
        prefetch::enable_prefetch();
        
        (with_prefetch_time, without_prefetch_time)
    }
}

/// Validation functions for cache optimizations
pub mod validation {
    use super::*;
    
    /// Validate that all cache-aligned tables are properly aligned
    /// 
    /// # Returns
    /// True if all tables are properly cache-aligned
    pub fn validate_cache_alignment() -> bool {
        let popcount_addr = &*CACHE_ALIGNED_POPCOUNT_TABLE as *const _ as usize;
        let bitpos_addr = &*CACHE_ALIGNED_BIT_POSITION_TABLE as *const _ as usize;
        let rank_addr = &*CACHE_ALIGNED_RANK_MASKS as *const _ as usize;
        let file_addr = &*CACHE_ALIGNED_FILE_MASKS as *const _ as usize;
        
        layout::is_cache_aligned(popcount_addr) &&
        layout::is_cache_aligned(bitpos_addr) &&
        layout::is_cache_aligned(rank_addr) &&
        layout::is_cache_aligned(file_addr)
    }
    
    /// Validate cache-aligned lookup table correctness
    /// 
    /// # Returns
    /// True if all lookup tables are correct
    pub fn validate_lookup_tables() -> bool {
        // Validate popcount table
        let popcount_table = &*CACHE_ALIGNED_POPCOUNT_TABLE;
        for i in 0..16 {
            let expected = (i as u8).count_ones();
            let actual = popcount_table.get_popcount(i as u8) as u32;
            if expected != actual {
                return false;
            }
        }
        
        // Validate bit position table
        let bitpos_table = &*CACHE_ALIGNED_BIT_POSITION_TABLE;
        for i in 0..16 {
            let positions = bitpos_table.get_positions(i as u8);
            let expected_positions = get_expected_bit_positions(i as u8);
            
            // Check that positions match expected pattern
            for (j, &pos) in positions.iter().enumerate() {
                if j < expected_positions.len() {
                    if pos != expected_positions[j] {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Get expected bit positions for a 4-bit value
    fn get_expected_bit_positions(value: u8) -> Vec<u8> {
        let mut positions = Vec::new();
        for i in 0..4 {
            if value & (1 << i) != 0 {
                positions.push(i);
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_alignment() {
        assert!(validation::validate_cache_alignment());
    }
    
    #[test]
    fn test_lookup_table_correctness() {
        assert!(validation::validate_lookup_tables());
    }
    
    #[test]
    fn test_popcount_cache_optimized() {
        let test_cases = vec![
            (0b0000, 0),
            (0b0001, 1),
            (0b0010, 1),
            (0b0011, 2),
            (0b1111, 4),
            (0b1010_1010, 4),
            (0b1111_1111_1111_1111, 16),
        ];
        
        for (bb, expected) in test_cases {
            assert_eq!(popcount_cache_optimized(bb), expected);
        }
    }
    
    #[test]
    fn test_bit_positions_cache_optimized() {
        let bb = 0b1010; // Bits at positions 1 and 3
        let positions = get_bit_positions_cache_optimized(bb);
        assert_eq!(positions, vec![1, 3]);
    }
    
    #[test]
    fn test_process_bitboard_sequence() {
        let bitboards = vec![0b1010, 0b1100, 0b1111];
        let counts = unsafe { process_bitboard_sequence(&bitboards) };
        assert_eq!(counts, vec![2, 2, 4]);
    }
    
    #[test]
    fn test_rank_masks() {
        let rank_masks = &*CACHE_ALIGNED_RANK_MASKS;
        let rank_0_mask = rank_masks.get_rank_mask(0);
        assert_eq!(rank_0_mask & 0x1FF, 0x1FF); // First 9 bits set
    }
    
    #[test]
    fn test_file_masks() {
        let file_masks = &*CACHE_ALIGNED_FILE_MASKS;
        let file_0_mask = file_masks.get_file_mask(0);
        // File 0 should have bits at positions 0, 9, 18, 27, 36, 45, 54, 63, 72
        assert!(file_0_mask & (1 << 0) != 0);
        assert!(file_0_mask & (1 << 9) != 0);
        assert!(file_0_mask & (1 << 18) != 0);
    }
    
    #[test]
    fn test_layout_utilities() {
        assert!(layout::is_cache_aligned(0));
        assert!(layout::is_cache_aligned(64));
        assert!(layout::is_cache_aligned(128));
        assert!(!layout::is_cache_aligned(1));
        assert!(!layout::is_cache_aligned(63));
        
        assert_eq!(layout::align_to_cache_line(0), 0);
        assert_eq!(layout::align_to_cache_line(1), 64);
        assert_eq!(layout::align_to_cache_line(64), 64);
        assert_eq!(layout::align_to_cache_line(65), 128);
        
        assert_eq!(layout::cache_line_offset(0), 0);
        assert_eq!(layout::cache_line_offset(63), 63);
        assert_eq!(layout::cache_line_offset(64), 0);
        assert_eq!(layout::cache_line_offset(127), 63);
    }
    
    #[test]
    fn test_prefetch_control() {
        assert!(prefetch::is_prefetch_enabled());
        
        prefetch::disable_prefetch();
        assert!(!prefetch::is_prefetch_enabled());
        
        prefetch::enable_prefetch();
        assert!(prefetch::is_prefetch_enabled());
    }
}
