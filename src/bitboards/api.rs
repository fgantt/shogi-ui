//! Public API for bit-scanning optimizations
//!
//! This module provides a clean, organized public API for all bit-scanning
//! optimization features. It serves as the main entry point for users of the
//! bit-scanning system and ensures backward compatibility while providing
//! access to all new optimization features.

use crate::types::Bitboard;

/// Bit-scanning operations module
///
/// This module provides optimized bit-scanning operations including population count,
/// bit position finding, and bit iteration. All operations are optimized for both
/// native and WebAssembly environments.
pub mod bitscan {
    use super::*;

    // Re-export bit iterator functionality
    pub use crate::bitboards::bit_iterator::{
        bits, bits_from, BitIterator, BitIteratorExt, ReverseBitIterator, ReverseBitIteratorExt,
    };

    /// Get the optimal population count for a bitboard
    ///
    /// This function automatically selects the best available implementation
    /// based on platform capabilities (hardware acceleration, fallback paths, etc.).
    ///
    /// # Arguments
    /// * `bb` - The bitboard to count
    ///
    /// # Returns
    /// The number of set bits in the bitboard
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::bitscan::popcount;
    ///
    /// assert_eq!(popcount(0b1010), 2);
    /// assert_eq!(popcount(0b1111), 4);
    /// assert_eq!(popcount(0), 0);
    /// ```
    pub fn popcount(bb: Bitboard) -> u32 {
        crate::bitboards::integration::GlobalOptimizer::popcount(bb)
    }

    /// Find the least significant bit position
    ///
    /// This function automatically selects the best available implementation
    /// based on platform capabilities.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the least significant bit, or None if no bits are set
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::bitscan::bit_scan_forward;
    ///
    /// assert_eq!(bit_scan_forward(0b1000), Some(3));
    /// assert_eq!(bit_scan_forward(0b1010), Some(1));
    /// assert_eq!(bit_scan_forward(0), None);
    /// ```
    pub fn bit_scan_forward(bb: Bitboard) -> Option<u8> {
        crate::bitboards::integration::GlobalOptimizer::bit_scan_forward(bb)
    }

    /// Find the most significant bit position
    ///
    /// This function automatically selects the best available implementation
    /// based on platform capabilities.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the most significant bit, or None if no bits are set
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::bitscan::bit_scan_reverse;
    ///
    /// assert_eq!(bit_scan_reverse(0b1000), Some(3));
    /// assert_eq!(bit_scan_reverse(0b1010), Some(3));
    /// assert_eq!(bit_scan_reverse(0), None);
    /// ```
    pub fn bit_scan_reverse(bb: Bitboard) -> Option<u8> {
        crate::bitboards::integration::GlobalOptimizer::bit_scan_reverse(bb)
    }

    /// Get all bit positions in a bitboard
    ///
    /// This function returns all positions where bits are set, ordered from
    /// least significant to most significant.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to analyze
    ///
    /// # Returns
    /// A vector containing all set bit positions
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::bitscan::get_all_bit_positions;
    ///
    /// let positions = get_all_bit_positions(0b1010);
    /// assert_eq!(positions, vec![1, 3]);
    /// ```
    pub fn get_all_bit_positions(bb: Bitboard) -> Vec<u8> {
        crate::bitboards::integration::GlobalOptimizer::get_all_bit_positions(bb)
    }
}

/// Bit manipulation utilities module
///
/// This module provides utility functions for common bit manipulation operations
/// including bit isolation, clearing, and set operations.
pub mod utils {
    // Re-export bit manipulation utilities
    pub use crate::bitboards::bit_utils::{
        bit_positions, complement, difference, extract_lsb, extract_msb, intersection, is_subset,
        lsb_position, msb_position, overlaps, reverse_bits, rotate_left, rotate_right,
        symmetric_difference, union,
    };
}

/// Square coordinate conversion module
///
/// This module provides utilities for converting between bit positions,
/// coordinate systems, and algebraic notation used in Shogi.
pub mod squares {
    // Re-export square conversion utilities
    pub use crate::bitboards::square_utils::{
        bit_to_coords, bit_to_square, bit_to_square_name, coords_to_bit, get_center_squares,
        is_center_square, is_promotion_zone, is_valid_shogi_square, promotion_zone_mask,
        square_distance, square_name_to_bit, square_to_bit,
    };
}

/// Platform detection and optimization module
///
/// This module provides information about platform capabilities and
/// optimization settings for bit-scanning operations.
pub mod platform {
    // Re-export platform detection utilities
    pub use crate::bitboards::integration::{BitScanningOptimizer, GlobalOptimizer};
    pub use crate::bitboards::platform_detection::{
        get_best_bitscan_impl, get_best_popcount_impl, get_platform_capabilities,
    };

    /// Create an optimized bit-scanning optimizer
    ///
    /// # Returns
    /// A BitScanningOptimizer configured for the current platform
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::platform::create_optimizer;
    ///
    /// let optimizer = create_optimizer();
    /// let count = optimizer.popcount(0b1010);
    /// assert_eq!(count, 2);
    /// ```
    pub fn create_optimizer() -> crate::bitboards::integration::BitScanningOptimizer {
        crate::bitboards::integration::BitScanningOptimizer::new()
    }
}

/// Performance analysis module
///
/// This module provides utilities for analyzing the performance characteristics
/// of bit-scanning operations and geometric patterns in bitboards.
pub mod analysis {
    use crate::types::Bitboard;

    // Re-export analysis utilities
    pub use crate::bitboards::integration::GeometricAnalysis;

    /// Analyze geometric patterns in a bitboard
    ///
    /// # Arguments
    /// * `bb` - The bitboard to analyze
    ///
    /// # Returns
    /// A GeometricAnalysis containing rank, file, and diagonal information
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::analysis::analyze_geometry;
    ///
    /// let analysis = analyze_geometry(0b1010);
    /// println!("Rank distribution: {:?}", analysis.rank_counts);
    /// ```
    pub fn analyze_geometry(bb: Bitboard) -> crate::bitboards::integration::GeometricAnalysis {
        crate::bitboards::integration::GlobalOptimizer::analyze_geometry(bb)
    }
}

/// Backward compatibility layer
///
/// This module provides backward compatibility for existing code that uses
/// the old bit-scanning API. It re-exports the new optimized functions
/// with the same names and signatures.
pub mod compat {
    use super::*;

    // Re-export core functions with original names for backward compatibility
    // Note: GlobalOptimizer is a struct, so we access its methods directly

    // Re-export utility functions
    pub use crate::bitboards::bit_utils::{
        bit_positions, complement, difference, intersection, is_subset, lsb_position, msb_position,
        overlaps, rotate_left, rotate_right, symmetric_difference, union,
    };

    // Re-export square conversion functions
    pub use crate::bitboards::square_utils::{
        bit_to_coords, bit_to_square, bit_to_square_name, coords_to_bit, square_name_to_bit,
        square_to_bit,
    };

    // Re-export platform detection
    pub use crate::bitboards::platform_detection::{
        get_best_bitscan_impl, get_best_popcount_impl, get_platform_capabilities,
    };

    /// Legacy function name for population count
    ///
    /// # Arguments
    /// * `bb` - The bitboard to count
    ///
    /// # Returns
    /// The number of set bits in the bitboard
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::compat::count_bits;
    ///
    /// assert_eq!(count_bits(0b1010), 2);
    /// ```
    #[deprecated(note = "Use bitscan::popcount instead")]
    pub fn count_bits(bb: Bitboard) -> u32 {
        crate::bitboards::integration::GlobalOptimizer::popcount(bb)
    }

    /// Legacy function name for bit scan forward
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the least significant bit, or None if no bits are set
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::compat::find_first_bit;
    ///
    /// assert_eq!(find_first_bit(0b1000), Some(3));
    /// ```
    #[deprecated(note = "Use bitscan::bit_scan_forward instead")]
    pub fn find_first_bit(bb: Bitboard) -> Option<u8> {
        crate::bitboards::integration::GlobalOptimizer::bit_scan_forward(bb)
    }

    /// Legacy function name for bit scan reverse
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the most significant bit, or None if no bits are set
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::compat::find_last_bit;
    ///
    /// assert_eq!(find_last_bit(0b1000), Some(3));
    /// ```
    #[deprecated(note = "Use bitscan::bit_scan_reverse instead")]
    pub fn find_last_bit(bb: Bitboard) -> Option<u8> {
        crate::bitboards::integration::GlobalOptimizer::bit_scan_reverse(bb)
    }
}

/// Precomputed masks and lookup tables module
///
/// This module provides access to precomputed masks and lookup tables
/// for efficient bitboard operations.
pub mod lookup {
    // Re-export lookup table utilities
    pub use crate::bitboards::lookup_tables::{
        bit_positions_4bit_lookup, bit_positions_4bit_small, popcount_4bit_lookup,
        popcount_4bit_optimized, popcount_4bit_small, validate_4bit_lookup_tables,
    };

    // Re-export mask utilities
    pub use crate::bitboards::masks::{
        get_diagonal_mask, get_diagonal_squares, get_file_from_square, get_file_mask,
        get_file_squares, get_rank_from_square, get_rank_mask, get_rank_squares,
        get_square_from_rank_file, same_diagonal, same_file, same_rank, validate_masks,
    };

    // Re-export De Bruijn sequence utilities
    pub use crate::bitboards::debruijn::{
        bit_scan_forward_debruijn, bit_scan_reverse_debruijn, get_all_bit_positions_debruijn,
    };

    /// Validate all lookup tables for correctness
    ///
    /// # Returns
    /// True if all lookup tables are valid
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::api::lookup::validate_all_tables;
    ///
    /// assert!(validate_all_tables());
    /// ```
    pub fn validate_all_tables() -> bool {
        crate::bitboards::lookup_tables::validate_4bit_lookup_tables()
            && crate::bitboards::masks::validate_masks()
    }
}
