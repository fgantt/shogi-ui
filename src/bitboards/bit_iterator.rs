//! Efficient bit iterator for bitboard operations
//!
//! This module provides an efficient iterator for traversing set bits in a bitboard.
//! The iterator is designed for zero-allocation performance and optimal memory usage,
//! making it suitable for high-performance bitboard operations in Shogi engines.

use crate::bitboards::integration::GlobalOptimizer;
use crate::types::Bitboard;

/// Efficient iterator over set bits in a bitboard
///
/// This iterator traverses all set bits in a bitboard, yielding their positions
/// in order from least significant to most significant bit. The iterator is
/// designed for zero-allocation performance and optimal memory usage.
///
/// # Performance
/// - Zero heap allocations during iteration
/// - O(k) time complexity where k is the number of set bits
/// - Uses the fastest available bit scanning algorithm for the platform
/// - Supports size_hint for iterator optimization
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_iterator::BitIterator;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let positions: Vec<u8> = BitIterator::new(bb).collect();
/// assert_eq!(positions, vec![1, 3]);
/// ```
#[derive(Debug, Clone)]
pub struct BitIterator {
    /// The bitboard being iterated over
    bits: Bitboard,
    /// Current bit position (None when iteration is complete)
    current: Option<u8>,
}

impl BitIterator {
    /// Create a new bit iterator for the given bitboard
    ///
    /// # Arguments
    /// * `bits` - The bitboard to iterate over
    ///
    /// # Returns
    /// A new iterator that will yield all set bit positions
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::bit_iterator::BitIterator;
    ///
    /// let bb = 0b1010;
    /// let mut iter = BitIterator::new(bb);
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn new(bits: Bitboard) -> Self {
        // Find the first set bit to initialize current position
        let current = if bits == 0 {
            None
        } else {
            GlobalOptimizer::bit_scan_forward(bits)
        };

        Self { bits, current }
    }

    /// Create an iterator that starts from a specific position
    ///
    /// # Arguments
    /// * `bits` - The bitboard to iterate over
    /// * `start_pos` - The position to start iteration from (inclusive)
    ///
    /// # Returns
    /// A new iterator that starts from the specified position
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::bit_iterator::BitIterator;
    ///
    /// let bb = 0b1110; // Bits at positions 1, 2, 3
    /// let mut iter = BitIterator::from_position(bb, 2);
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn from_position(bits: Bitboard, start_pos: u8) -> Self {
        if start_pos >= 128 {
            return Self {
                bits: 0,
                current: None,
            };
        }

        // Create a mask to clear bits before start_pos
        let mask = if start_pos == 0 {
            bits
        } else {
            // Clear all bits before start_pos by shifting left then right
            (bits << (128 - start_pos)) >> (128 - start_pos)
        };

        let current = if mask == 0 {
            None
        } else {
            GlobalOptimizer::bit_scan_forward(mask)
        };

        Self {
            bits: mask,
            current,
        }
    }

    /// Get the remaining bits in the bitboard
    ///
    /// # Returns
    /// The bitboard containing all remaining bits to be iterated
    pub fn remaining_bits(&self) -> Bitboard {
        self.bits
    }

    /// Skip the next n bits
    ///
    /// # Arguments
    /// * `n` - Number of bits to skip
    ///
    /// # Returns
    /// The number of bits actually skipped
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::bit_iterator::BitIterator;
    ///
    /// let bb = 0b1111; // Bits at positions 0, 1, 2, 3
    /// let mut iter = BitIterator::new(bb);
    /// assert_eq!(iter.skip(2), 2);
    /// assert_eq!(iter.next(), Some(2));
    /// ```
    pub fn skip(&mut self, mut n: usize) -> usize {
        let mut skipped = 0;

        while n > 0 && self.current.is_some() {
            if let Some(_pos) = self.current {
                // Clear the current bit and find the next one
                self.bits &= self.bits - 1;
                self.current = if self.bits == 0 {
                    None
                } else {
                    GlobalOptimizer::bit_scan_forward(self.bits)
                };
                skipped += 1;
                n -= 1;
            } else {
                break;
            }
        }

        skipped
    }

    /// Peek at the next bit position without consuming it
    ///
    /// # Returns
    /// The next bit position, or None if iteration is complete
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::bit_iterator::BitIterator;
    ///
    /// let bb = 0b1010; // Bits at positions 1 and 3
    /// let mut iter = BitIterator::new(bb);
    /// assert_eq!(iter.peek(), Some(1));
    /// assert_eq!(iter.next(), Some(1)); // Still returns 1
    /// ```
    pub fn peek(&self) -> Option<u8> {
        self.current
    }

    /// Check if the iterator is empty (no more bits to iterate)
    ///
    /// # Returns
    /// True if there are no more bits to iterate
    pub fn is_empty(&self) -> bool {
        self.current.is_none()
    }

    /// Count the total number of bits that will be yielded
    ///
    /// # Returns
    /// The total number of set bits in the bitboard
    pub fn count(&self) -> u32 {
        GlobalOptimizer::popcount(self.bits)
    }

    /// Convert the iterator to a vector of bit positions
    ///
    /// # Returns
    /// A vector containing all bit positions in order
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::bit_iterator::BitIterator;
    ///
    /// let bb = 0b1010; // Bits at positions 1 and 3
    /// let positions = BitIterator::new(bb).to_vec();
    /// assert_eq!(positions, vec![1, 3]);
    /// ```
    pub fn to_vec(self) -> Vec<u8> {
        self.collect()
    }
}

impl Iterator for BitIterator {
    type Item = u8;

    /// Get the next bit position
    ///
    /// # Returns
    /// The next bit position, or None if iteration is complete
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.current {
            // Clear the current bit and find the next one
            self.bits &= self.bits - 1;
            self.current = if self.bits == 0 {
                None
            } else {
                GlobalOptimizer::bit_scan_forward(self.bits)
            };
            Some(pos)
        } else {
            None
        }
    }

    /// Provide a size hint for iterator optimization
    ///
    /// # Returns
    /// A tuple containing (lower_bound, upper_bound) for the number of remaining items
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count() as usize;
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for BitIterator {
    /// Get the next bit position from the end
    ///
    /// # Returns
    /// The next bit position from the end, or None if iteration is complete
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        // Find the most significant bit
        if let Some(pos) = GlobalOptimizer::bit_scan_reverse(self.bits) {
            // Clear the MSB
            self.bits &= !(1u128 << pos);
            Some(pos)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for BitIterator {
    /// Get the exact number of remaining items
    ///
    /// # Returns
    /// The exact number of remaining bit positions
    fn len(&self) -> usize {
        self.count() as usize
    }
}

/// Convenience function to create a bit iterator
///
/// # Arguments
/// * `bits` - The bitboard to iterate over
///
/// # Returns
/// A new bit iterator
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::bit_iterator::bits;
///
/// let bb = 0b1010;
/// let positions: Vec<u8> = bits(bb).collect();
/// assert_eq!(positions, vec![1, 3]);
/// ```
pub fn bits(bits: Bitboard) -> BitIterator {
    BitIterator::new(bits)
}

/// Convenience function to create a bit iterator from a specific position
///
/// # Arguments
/// * `bits` - The bitboard to iterate over
/// * `start_pos` - The position to start iteration from
///
/// # Returns
/// A new bit iterator starting from the specified position
pub fn bits_from(bits: Bitboard, start_pos: u8) -> BitIterator {
    BitIterator::from_position(bits, start_pos)
}

/// Extension trait for bitboards to add iterator functionality
///
/// This trait provides convenient methods for creating iterators directly
/// from bitboard values.
pub trait BitIteratorExt {
    /// Create an iterator over all set bits
    ///
    /// # Returns
    /// A new bit iterator
    fn bits(self) -> BitIterator;

    /// Create an iterator starting from a specific position
    ///
    /// # Arguments
    /// * `start_pos` - The position to start iteration from
    ///
    /// # Returns
    /// A new bit iterator starting from the specified position
    fn bits_from(self, start_pos: u8) -> BitIterator;
}

impl BitIteratorExt for Bitboard {
    fn bits(self) -> BitIterator {
        BitIterator::new(self)
    }

    fn bits_from(self, start_pos: u8) -> BitIterator {
        BitIterator::from_position(self, start_pos)
    }
}

/// Iterator that yields bit positions in reverse order (MSB to LSB)
///
/// This iterator traverses all set bits in a bitboard, yielding their positions
/// in order from most significant to least significant bit.
#[derive(Debug, Clone)]
pub struct ReverseBitIterator {
    bits: Bitboard,
}

impl ReverseBitIterator {
    /// Create a new reverse bit iterator
    ///
    /// # Arguments
    /// * `bits` - The bitboard to iterate over
    ///
    /// # Returns
    /// A new reverse iterator
    pub fn new(bits: Bitboard) -> Self {
        Self { bits }
    }
}

impl Iterator for ReverseBitIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        // Find the most significant bit
        if let Some(pos) = GlobalOptimizer::bit_scan_reverse(self.bits) {
            // Clear the MSB
            self.bits &= !(1u128 << pos);
            Some(pos)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = GlobalOptimizer::popcount(self.bits) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ReverseBitIterator {
    fn len(&self) -> usize {
        GlobalOptimizer::popcount(self.bits) as usize
    }
}

/// Extension trait for reverse iteration
pub trait ReverseBitIteratorExt {
    /// Create a reverse iterator over all set bits
    ///
    /// # Returns
    /// A new reverse bit iterator
    fn bits_rev(self) -> ReverseBitIterator;
}

impl ReverseBitIteratorExt for Bitboard {
    fn bits_rev(self) -> ReverseBitIterator {
        ReverseBitIterator::new(self)
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_bit_iterator_basic() {
        let bb = 0b1010; // Bits at positions 1 and 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_bit_iterator_empty() {
        let bb = 0u128;
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.next(), None);
        assert_eq!(iter.peek(), None);
        assert!(iter.is_empty());
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_bit_iterator_single_bit() {
        let bb = 1u128; // Only bit 0
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_bit_iterator_all_bits() {
        let bb = 0b1111u128; // Bits 0, 1, 2, 3
        let positions: Vec<u8> = BitIterator::new(bb).collect();

        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_bit_iterator_from_position() {
        let bb = 0b1111u128; // Bits 0, 1, 2, 3
        let mut iter = BitIterator::from_position(bb, 2);

        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_bit_iterator_skip() {
        let bb = 0b1111u128; // Bits 0, 1, 2, 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.skip(2), 2);
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_bit_iterator_peek() {
        let bb = 0b1010u128; // Bits 1 and 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.peek(), Some(1));
        assert_eq!(iter.peek(), Some(1)); // Peek doesn't consume
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.peek(), Some(3));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.peek(), None);
    }

    #[test]
    fn test_bit_iterator_size_hint() {
        let bb = 0b1010u128; // Bits 1 and 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_bit_iterator_exact_size() {
        let bb = 0b1010u128; // Bits 1 and 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn test_bit_iterator_double_ended() {
        let bb = 0b1111u128; // Bits 0, 1, 2, 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_bit_iterator_convenience_functions() {
        let bb = 0b1010u128;

        // Test bits() function
        let positions1: Vec<u8> = bits(bb).collect();
        assert_eq!(positions1, vec![1, 3]);

        // Test bits_from() function
        let positions2: Vec<u8> = bits_from(bb, 2).collect();
        assert_eq!(positions2, vec![3]);

        // Test extension trait
        let positions3: Vec<u8> = bb.bits().collect();
        assert_eq!(positions3, vec![1, 3]);

        let positions4: Vec<u8> = bb.bits_from(2).collect();
        assert_eq!(positions4, vec![3]);
    }

    #[test]
    fn test_reverse_bit_iterator() {
        let bb = 0b1111u128; // Bits 0, 1, 2, 3
        let positions: Vec<u8> = ReverseBitIterator::new(bb).collect();

        assert_eq!(positions, vec![3, 2, 1, 0]);
    }

    #[test]
    fn test_reverse_bit_iterator_extension() {
        let bb = 0b1010u128; // Bits 1 and 3
        let positions: Vec<u8> = bb.bits_rev().collect();

        assert_eq!(positions, vec![3, 1]);
    }

    #[test]
    fn test_bit_iterator_to_vec() {
        let bb = 0b1010u128;
        let positions = BitIterator::new(bb).to_vec();

        assert_eq!(positions, vec![1, 3]);
    }

    #[test]
    fn test_bit_iterator_remaining_bits() {
        let bb = 0b1110u128; // Bits 1, 2, 3
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.remaining_bits(), bb);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.remaining_bits(), 0b1100u128); // Bits 2, 3
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.remaining_bits(), 0b1000u128); // Bit 3
    }

    #[test]
    fn test_bit_iterator_large_bitboard() {
        // Test with a larger bitboard to ensure it works with u128
        let bb = 0x80000000000000000000000000000000u128; // Bit 127
        let mut iter = BitIterator::new(bb);

        assert_eq!(iter.next(), Some(127));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_bit_iterator_performance_consistency() {
        // Test that iterator produces same results as manual scanning
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
            let iterator_positions: Vec<u8> = BitIterator::new(bb).collect();
            let manual_positions = GlobalOptimizer::get_all_bit_positions(bb);

            assert_eq!(
                iterator_positions, manual_positions,
                "Iterator inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_bit_iterator_edge_cases() {
        // Test edge cases that might cause issues

        // Test from_position with position >= 128
        let bb = 0b1111u128;
        let mut iter = BitIterator::from_position(bb, 128);
        assert_eq!(iter.next(), None);

        // Test from_position with position 0
        let bb = 0b1111u128;
        let positions: Vec<u8> = BitIterator::from_position(bb, 0).collect();
        assert_eq!(positions, vec![0, 1, 2, 3]);

        // Test skip with more than available bits
        let bb = 0b1010u128; // Only 2 bits
        let mut iter = BitIterator::new(bb);
        assert_eq!(iter.skip(5), 2); // Should only skip 2
        assert_eq!(iter.next(), None);
    }
}
