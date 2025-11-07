//! Bit manipulation utilities for bitboard operations
//!
//! This module provides a comprehensive collection of bit manipulation utilities
//! for bitboard operations. These utilities are optimized for performance and
//! provide essential building blocks for higher-level bitboard operations.

use crate::bitboards::integration::GlobalOptimizer;
use crate::types::Bitboard;

/// Isolate the least significant bit (LSB) in a bitboard
///
/// This function returns a bitboard containing only the least significant
/// set bit from the input bitboard. All other bits are cleared.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard containing only the LSB, or 0 if no bits are set
///
/// # Performance
/// This operation is typically implemented as a single instruction on most
/// modern processors using the formula: `bb & (!bb + 1)`
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::isolate_lsb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let lsb = isolate_lsb(bb);
/// assert_eq!(lsb, 0b0010); // Only bit 1 remains
///
/// let bb = 0b1000; // Only bit 3
/// let lsb = isolate_lsb(bb);
/// assert_eq!(lsb, 0b1000); // Same as input
/// ```
pub fn isolate_lsb(bb: Bitboard) -> Bitboard {
    if bb == 0 {
        0
    } else {
        bb & (!bb + 1)
    }
}

/// Isolate the most significant bit (MSB) in a bitboard
///
/// This function returns a bitboard containing only the most significant
/// set bit from the input bitboard. All other bits are cleared.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard containing only the MSB, or 0 if no bits are set
///
/// # Performance
/// This operation uses bit scanning to find the MSB position, then
/// creates a mask with only that bit set.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::isolate_msb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let msb = isolate_msb(bb);
/// assert_eq!(msb, 0b1000); // Only bit 3 remains
///
/// let bb = 0b0001; // Only bit 0
/// let msb = isolate_msb(bb);
/// assert_eq!(msb, 0b0001); // Same as input
/// ```
pub fn isolate_msb(bb: Bitboard) -> Bitboard {
    if bb == 0 {
        0
    } else if let Some(pos) = GlobalOptimizer::bit_scan_reverse(bb) {
        1u128 << pos
    } else {
        0
    }
}

/// Clear the least significant bit (LSB) in a bitboard
///
/// This function returns a bitboard with the least significant set bit
/// cleared. All other bits remain unchanged.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard with the LSB cleared
///
/// # Performance
/// This operation is typically implemented as a single instruction on most
/// modern processors using the formula: `bb & (bb - 1)`
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::clear_lsb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let cleared = clear_lsb(bb);
/// assert_eq!(cleared, 0b1000); // Bit 1 cleared, bit 3 remains
///
/// let bb = 0b1000; // Only bit 3
/// let cleared = clear_lsb(bb);
/// assert_eq!(cleared, 0b0000); // Bit 3 cleared
/// ```
pub fn clear_lsb(bb: Bitboard) -> Bitboard {
    bb & (bb - 1)
}

/// Clear the most significant bit (MSB) in a bitboard
///
/// This function returns a bitboard with the most significant set bit
/// cleared. All other bits remain unchanged.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard with the MSB cleared
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::clear_msb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let cleared = clear_msb(bb);
/// assert_eq!(cleared, 0b0010); // Bit 3 cleared, bit 1 remains
///
/// let bb = 0b0001; // Only bit 0
/// let cleared = clear_msb(bb);
/// assert_eq!(cleared, 0b0000); // Bit 0 cleared
/// ```
pub fn clear_msb(bb: Bitboard) -> Bitboard {
    if bb == 0 {
        0
    } else if let Some(pos) = GlobalOptimizer::bit_scan_reverse(bb) {
        bb & !(1u128 << pos)
    } else {
        0
    }
}

/// Get all bit positions in a bitboard
///
/// This function returns a vector containing all bit positions where
/// bits are set, ordered from least significant to most significant.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A vector containing all set bit positions
///
/// # Performance
/// This function uses the optimized bit position enumeration from
/// the integration module for best performance.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::bit_positions;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let positions = bit_positions(bb);
/// assert_eq!(positions, vec![1, 3]);
///
/// let bb = 0b0000; // No bits set
/// let positions = bit_positions(bb);
/// assert_eq!(positions, vec![]);
/// ```
pub fn bit_positions(bb: Bitboard) -> Vec<u8> {
    GlobalOptimizer::get_all_bit_positions(bb)
}

/// Extract and clear the least significant bit (LSB)
///
/// This function returns both the isolated LSB and the bitboard with
/// the LSB cleared. This is useful for iterating through bits while
/// modifying the bitboard.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A tuple containing (isolated_lsb, cleared_bitboard)
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::extract_lsb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let (lsb, cleared) = extract_lsb(bb);
/// assert_eq!(lsb, 0b0010); // Isolated LSB
/// assert_eq!(cleared, 0b1000); // Bitboard with LSB cleared
/// ```
pub fn extract_lsb(bb: Bitboard) -> (Bitboard, Bitboard) {
    let lsb = isolate_lsb(bb);
    let cleared = bb & (bb - 1);
    (lsb, cleared)
}

/// Extract and clear the most significant bit (MSB)
///
/// This function returns both the isolated MSB and the bitboard with
/// the MSB cleared. This is useful for iterating through bits while
/// modifying the bitboard.
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A tuple containing (isolated_msb, cleared_bitboard)
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::extract_msb;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let (msb, cleared) = extract_msb(bb);
/// assert_eq!(msb, 0b1000); // Isolated MSB
/// assert_eq!(cleared, 0b0010); // Bitboard with MSB cleared
/// ```
pub fn extract_msb(bb: Bitboard) -> (Bitboard, Bitboard) {
    let msb = isolate_msb(bb);
    let cleared = if msb != 0 { bb & !msb } else { 0 };
    (msb, cleared)
}

/// Check if a bitboard has exactly one bit set
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// True if exactly one bit is set, false otherwise
///
/// # Performance
/// This operation uses the efficient formula: `bb != 0 && (bb & (bb - 1)) == 0`
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::is_single_bit;
///
/// assert!(is_single_bit(0b0001));
/// assert!(is_single_bit(0b1000));
/// assert!(!is_single_bit(0b0000));
/// assert!(!is_single_bit(0b1010));
/// ```
pub fn is_single_bit(bb: Bitboard) -> bool {
    bb != 0 && (bb & (bb - 1)) == 0
}

/// Check if a bitboard has multiple bits set
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// True if more than one bit is set, false otherwise
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::is_multiple_bits;
///
/// assert!(is_multiple_bits(0b1010));
/// assert!(is_multiple_bits(0b1111));
/// assert!(!is_multiple_bits(0b0000));
/// assert!(!is_multiple_bits(0b1000));
/// ```
pub fn is_multiple_bits(bb: Bitboard) -> bool {
    bb != 0 && (bb & (bb - 1)) != 0
}

/// Check if a bitboard is empty (no bits set)
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// True if no bits are set, false otherwise
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::is_empty;
///
/// assert!(is_empty(0b0000));
/// assert!(!is_empty(0b0001));
/// assert!(!is_empty(0b1010));
/// ```
pub fn is_empty(bb: Bitboard) -> bool {
    bb == 0
}

/// Count the number of set bits in a bitboard
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// The number of set bits
///
/// # Performance
/// This function uses the optimized population count from the integration module.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::popcount;
///
/// assert_eq!(popcount(0b0000), 0);
/// assert_eq!(popcount(0b0001), 1);
/// assert_eq!(popcount(0b1010), 2);
/// assert_eq!(popcount(0b1111), 4);
/// ```
pub fn popcount(bb: Bitboard) -> u32 {
    GlobalOptimizer::popcount(bb)
}

/// Get the position of the least significant bit
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// The position of the LSB (0-based), or None if no bits are set
///
/// # Performance
/// This function uses the optimized bit scan forward from the integration module.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::lsb_position;
///
/// assert_eq!(lsb_position(0b0001), Some(0));
/// assert_eq!(lsb_position(0b1010), Some(1));
/// assert_eq!(lsb_position(0b1000), Some(3));
/// assert_eq!(lsb_position(0b0000), None);
/// ```
pub fn lsb_position(bb: Bitboard) -> Option<u8> {
    GlobalOptimizer::bit_scan_forward(bb)
}

/// Get the position of the most significant bit
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// The position of the MSB (0-based), or None if no bits are set
///
/// # Performance
/// This function uses the optimized bit scan reverse from the integration module.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::msb_position;
///
/// assert_eq!(msb_position(0b0001), Some(0));
/// assert_eq!(msb_position(0b1010), Some(3));
/// assert_eq!(msb_position(0b1000), Some(3));
/// assert_eq!(msb_position(0b0000), None);
/// ```
pub fn msb_position(bb: Bitboard) -> Option<u8> {
    GlobalOptimizer::bit_scan_reverse(bb)
}

/// Rotate a bitboard left by a specified number of positions
///
/// # Arguments
/// * `bb` - The input bitboard
/// * `amount` - Number of positions to rotate left (0-127)
///
/// # Returns
/// The rotated bitboard
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::rotate_left;
///
/// let bb = 0b0001; // Bit 0
/// let rotated = rotate_left(bb, 1);
/// assert_eq!(rotated, 0b0010); // Bit 1
///
/// let bb = 0b1000; // Bit 3
/// let rotated = rotate_left(bb, 1);
/// assert_eq!(rotated, 0b10000); // Bit 4
/// ```
pub fn rotate_left(bb: Bitboard, amount: u8) -> Bitboard {
    if amount == 0 {
        bb
    } else {
        let amount = amount as u32;
        (bb << amount) | (bb >> (128 - amount))
    }
}

/// Rotate a bitboard right by a specified number of positions
///
/// # Arguments
/// * `bb` - The input bitboard
/// * `amount` - Number of positions to rotate right (0-127)
///
/// # Returns
/// The rotated bitboard
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::rotate_right;
///
/// let bb = 0b0010; // Bit 1
/// let rotated = rotate_right(bb, 1);
/// assert_eq!(rotated, 0b0001); // Bit 0
///
/// let bb = 0b10000; // Bit 4
/// let rotated = rotate_right(bb, 1);
/// assert_eq!(rotated, 0b1000); // Bit 3
/// ```
pub fn rotate_right(bb: Bitboard, amount: u8) -> Bitboard {
    if amount == 0 {
        bb
    } else {
        let amount = amount as u32;
        (bb >> amount) | (bb << (128 - amount))
    }
}

/// Reverse the order of bits in a bitboard
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard with the bit order reversed
///
/// # Performance
/// This operation uses a combination of bit manipulation techniques
/// to reverse the bit order efficiently.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::reverse_bits;
///
/// let bb = 0b1000; // Bit 3
/// let reversed = reverse_bits(bb);
/// assert_eq!(reversed, 0b0001); // Bit 0
///
/// let bb = 0b1010; // Bits 1 and 3
/// let reversed = reverse_bits(bb);
/// assert_eq!(reversed, 0b0101); // Bits 0 and 2
/// ```
pub fn reverse_bits(bb: Bitboard) -> Bitboard {
    // For 128-bit values, we need to reverse both halves and swap them
    let low = bb as u64;
    let high = (bb >> 64) as u64;

    // Reverse each 64-bit half
    let reversed_low = reverse_bits_64(low);
    let reversed_high = reverse_bits_64(high);

    // Combine with swapped positions
    (reversed_low as u128) << 64 | (reversed_high as u128)
}

/// Helper function to reverse bits in a 64-bit value
fn reverse_bits_64(mut x: u64) -> u64 {
    // Reverse bits using bit manipulation tricks
    x = ((x & 0x5555555555555555) << 1) | ((x & 0xAAAAAAAAAAAAAAAA) >> 1);
    x = ((x & 0x3333333333333333) << 2) | ((x & 0xCCCCCCCCCCCCCCCC) >> 2);
    x = ((x & 0x0F0F0F0F0F0F0F0F) << 4) | ((x & 0xF0F0F0F0F0F0F0F0) >> 4);
    x = ((x & 0x00FF00FF00FF00FF) << 8) | ((x & 0xFF00FF00FF00FF00) >> 8);
    x = ((x & 0x0000FFFF0000FFFF) << 16) | ((x & 0xFFFF0000FFFF0000) >> 16);
    x = (x << 32) | (x >> 32);
    x
}

/// Check if two bitboards have any bits in common
///
/// # Arguments
/// * `bb1` - First bitboard
/// * `bb2` - Second bitboard
///
/// # Returns
/// True if the bitboards have any bits in common, false otherwise
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::overlaps;
///
/// assert!(overlaps(0b1010, 0b0101)); // Bits 1 and 2 overlap
/// assert!(overlaps(0b1010, 0b1000)); // Bit 3 overlaps
/// assert!(!overlaps(0b1010, 0b0001)); // No overlap
/// ```
pub fn overlaps(bb1: Bitboard, bb2: Bitboard) -> bool {
    (bb1 & bb2) != 0
}

/// Check if one bitboard is a subset of another
///
/// # Arguments
/// * `subset` - The potential subset bitboard
/// * `superset` - The potential superset bitboard
///
/// # Returns
/// True if all bits in subset are also in superset, false otherwise
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::is_subset;
///
/// assert!(is_subset(0b0010, 0b1010)); // Bit 1 is subset of bits 1,3
/// assert!(is_subset(0b1010, 0b1111)); // Bits 1,3 are subset of bits 0,1,2,3
/// assert!(!is_subset(0b1001, 0b1010)); // Bit 0 is not in bits 1,3
/// ```
pub fn is_subset(subset: Bitboard, superset: Bitboard) -> bool {
    (subset & superset) == subset
}

/// Get the intersection of two bitboards
///
/// # Arguments
/// * `bb1` - First bitboard
/// * `bb2` - Second bitboard
///
/// # Returns
/// A bitboard containing only the bits that are set in both input bitboards
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::intersection;
///
/// let bb1 = 0b1010; // Bits 1 and 3
/// let bb2 = 0b0110; // Bits 1 and 2
/// let result = intersection(bb1, bb2);
/// assert_eq!(result, 0b0010); // Only bit 1 is common
/// ```
pub fn intersection(bb1: Bitboard, bb2: Bitboard) -> Bitboard {
    bb1 & bb2
}

/// Get the union of two bitboards
///
/// # Arguments
/// * `bb1` - First bitboard
/// * `bb2` - Second bitboard
///
/// # Returns
/// A bitboard containing all bits that are set in either input bitboard
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::union;
///
/// let bb1 = 0b1010; // Bits 1 and 3
/// let bb2 = 0b0110; // Bits 1 and 2
/// let result = union(bb1, bb2);
/// assert_eq!(result, 0b1110); // Bits 1, 2, and 3
/// ```
pub fn union(bb1: Bitboard, bb2: Bitboard) -> Bitboard {
    bb1 | bb2
}

/// Get the symmetric difference (XOR) of two bitboards
///
/// # Arguments
/// * `bb1` - First bitboard
/// * `bb2` - Second bitboard
///
/// # Returns
/// A bitboard containing bits that are set in exactly one of the input bitboards
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::symmetric_difference;
///
/// let bb1 = 0b1010; // Bits 1 and 3
/// let bb2 = 0b0110; // Bits 1 and 2
/// let result = symmetric_difference(bb1, bb2);
/// assert_eq!(result, 0b1100); // Bits 2 and 3 (not bit 1)
/// ```
pub fn symmetric_difference(bb1: Bitboard, bb2: Bitboard) -> Bitboard {
    bb1 ^ bb2
}

/// Get the complement (NOT) of a bitboard
///
/// # Arguments
/// * `bb` - The input bitboard
///
/// # Returns
/// A bitboard with all bits flipped
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::complement;
///
/// let bb = 0b1010; // Bits 1 and 3
/// let result = complement(bb);
/// assert_eq!(result, !0b1010); // All bits except 1 and 3
/// ```
pub fn complement(bb: Bitboard) -> Bitboard {
    !bb
}

/// Get the difference (subtraction) of two bitboards
///
/// # Arguments
/// * `bb1` - The bitboard to subtract from
/// * `bb2` - The bitboard to subtract
///
/// # Returns
/// A bitboard containing bits that are in bb1 but not in bb2
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_utils::difference;
///
/// let bb1 = 0b1010; // Bits 1 and 3
/// let bb2 = 0b0110; // Bits 1 and 2
/// let result = difference(bb1, bb2);
/// assert_eq!(result, 0b1000); // Only bit 3 (bit 1 is removed)
/// ```
pub fn difference(bb1: Bitboard, bb2: Bitboard) -> Bitboard {
    bb1 & !bb2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolate_lsb() {
        assert_eq!(isolate_lsb(0b1010), 0b0010);
        assert_eq!(isolate_lsb(0b1000), 0b1000);
        assert_eq!(isolate_lsb(0b0001), 0b0001);
        assert_eq!(isolate_lsb(0b0000), 0b0000);
    }

    #[test]
    fn test_isolate_msb() {
        assert_eq!(isolate_msb(0b1010), 0b1000);
        assert_eq!(isolate_msb(0b1000), 0b1000);
        assert_eq!(isolate_msb(0b0001), 0b0001);
        assert_eq!(isolate_msb(0b0000), 0b0000);
    }

    #[test]
    fn test_clear_lsb() {
        assert_eq!(clear_lsb(0b1010), 0b1000);
        assert_eq!(clear_lsb(0b1000), 0b0000);
        assert_eq!(clear_lsb(0b0001), 0b0000);
        assert_eq!(clear_lsb(0b0000), 0b0000);
    }

    #[test]
    fn test_clear_msb() {
        assert_eq!(clear_msb(0b1010), 0b0010);
        assert_eq!(clear_msb(0b1000), 0b0000);
        assert_eq!(clear_msb(0b0001), 0b0000);
        assert_eq!(clear_msb(0b0000), 0b0000);
    }

    #[test]
    fn test_bit_positions() {
        assert_eq!(bit_positions(0b1010), vec![1, 3]);
        assert_eq!(bit_positions(0b1000), vec![3]);
        assert_eq!(bit_positions(0b0001), vec![0]);
        assert_eq!(bit_positions(0b0000), Vec::<u8>::new());
    }

    #[test]
    fn test_extract_lsb() {
        let (lsb, cleared) = extract_lsb(0b1010);
        assert_eq!(lsb, 0b0010);
        assert_eq!(cleared, 0b1000);

        let (lsb, cleared) = extract_lsb(0b1000);
        assert_eq!(lsb, 0b1000);
        assert_eq!(cleared, 0b0000);
    }

    #[test]
    fn test_extract_msb() {
        let (msb, cleared) = extract_msb(0b1010);
        assert_eq!(msb, 0b1000);
        assert_eq!(cleared, 0b0010);

        let (msb, cleared) = extract_msb(0b0001);
        assert_eq!(msb, 0b0001);
        assert_eq!(cleared, 0b0000);
    }

    #[test]
    fn test_is_single_bit() {
        assert!(is_single_bit(0b0001));
        assert!(is_single_bit(0b1000));
        assert!(!is_single_bit(0b0000));
        assert!(!is_single_bit(0b1010));
    }

    #[test]
    fn test_is_multiple_bits() {
        assert!(is_multiple_bits(0b1010));
        assert!(is_multiple_bits(0b1111));
        assert!(!is_multiple_bits(0b0000));
        assert!(!is_multiple_bits(0b1000));
    }

    #[test]
    fn test_is_empty() {
        assert!(is_empty(0b0000));
        assert!(!is_empty(0b0001));
        assert!(!is_empty(0b1010));
    }

    #[test]
    fn test_popcount() {
        assert_eq!(popcount(0b0000), 0);
        assert_eq!(popcount(0b0001), 1);
        assert_eq!(popcount(0b1010), 2);
        assert_eq!(popcount(0b1111), 4);
    }

    #[test]
    fn test_lsb_position() {
        assert_eq!(lsb_position(0b0001), Some(0));
        assert_eq!(lsb_position(0b1010), Some(1));
        assert_eq!(lsb_position(0b1000), Some(3));
        assert_eq!(lsb_position(0b0000), None);
    }

    #[test]
    fn test_msb_position() {
        assert_eq!(msb_position(0b0001), Some(0));
        assert_eq!(msb_position(0b1010), Some(3));
        assert_eq!(msb_position(0b1000), Some(3));
        assert_eq!(msb_position(0b0000), None);
    }

    #[test]
    fn test_rotate_left() {
        assert_eq!(rotate_left(0b0001, 1), 0b0010);
        assert_eq!(rotate_left(0b1000, 1), 0b10000);
        assert_eq!(rotate_left(0b1010, 0), 0b1010);
    }

    #[test]
    fn test_rotate_right() {
        assert_eq!(rotate_right(0b0010, 1), 0b0001);
        assert_eq!(rotate_right(0b10000, 1), 0b1000);
        assert_eq!(rotate_right(0b1010, 0), 0b1010);
    }

    #[test]
    fn test_reverse_bits() {
        assert_eq!(reverse_bits(0b1000), 0b0001);
        assert_eq!(reverse_bits(0b1010), 0b0101);
        assert_eq!(reverse_bits(0b0000), 0b0000);
    }

    #[test]
    fn test_overlaps() {
        assert!(overlaps(0b1010, 0b0101));
        assert!(overlaps(0b1010, 0b1000));
        assert!(!overlaps(0b1010, 0b0001));
    }

    #[test]
    fn test_is_subset() {
        assert!(is_subset(0b0010, 0b1010));
        assert!(is_subset(0b1010, 0b1111));
        assert!(!is_subset(0b1001, 0b1010));
    }

    #[test]
    fn test_intersection() {
        let bb1 = 0b1010;
        let bb2 = 0b0110;
        assert_eq!(intersection(bb1, bb2), 0b0010);
    }

    #[test]
    fn test_union() {
        let bb1 = 0b1010;
        let bb2 = 0b0110;
        assert_eq!(union(bb1, bb2), 0b1110);
    }

    #[test]
    fn test_symmetric_difference() {
        let bb1 = 0b1010;
        let bb2 = 0b0110;
        assert_eq!(symmetric_difference(bb1, bb2), 0b1100);
    }

    #[test]
    fn test_complement() {
        let bb = 0b1010;
        assert_eq!(complement(bb), !0b1010);
    }

    #[test]
    fn test_difference() {
        let bb1 = 0b1010;
        let bb2 = 0b0110;
        assert_eq!(difference(bb1, bb2), 0b1000);
    }

    #[test]
    fn test_edge_cases() {
        // Test with empty bitboards
        assert_eq!(isolate_lsb(0), 0);
        assert_eq!(isolate_msb(0), 0);
        assert_eq!(clear_lsb(0), 0);
        assert_eq!(clear_msb(0), 0);
        assert_eq!(bit_positions(0), Vec::<u8>::new());

        // Test with single bit
        assert_eq!(isolate_lsb(1), 1);
        assert_eq!(isolate_msb(1), 1);
        assert_eq!(clear_lsb(1), 0);
        assert_eq!(clear_msb(1), 0);

        // Test with all bits set (for u128, this would be 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
        let all_bits = !0u128;
        assert_eq!(popcount(all_bits), 128);
    }

    #[test]
    fn test_performance_consistency() {
        // Test that utility functions produce consistent results
        let test_cases = [
            0u128,
            1u128,
            0xFFu128,
            0x8000000000000000u128,
            0x10000000000000000u128,
            0x5555555555555555u128,
            0xAAAAAAAAAAAAAAAAu128,
            0x123456789ABCDEF0u128,
        ];

        for bb in test_cases {
            // Test that isolate_lsb and clear_lsb work together
            let lsb = isolate_lsb(bb);
            let cleared = clear_lsb(bb);
            if bb != 0 {
                assert_eq!(lsb | cleared, bb);
            }

            // Test that bit_positions and popcount are consistent
            let positions = bit_positions(bb);
            assert_eq!(positions.len(), popcount(bb) as usize);

            // Test that lsb_position and msb_position are consistent with positions
            if !positions.is_empty() {
                assert_eq!(lsb_position(bb), Some(positions[0]));
                assert_eq!(msb_position(bb), Some(positions[positions.len() - 1]));
            }
        }
    }
}
