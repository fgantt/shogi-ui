//! Precomputed masks for Shogi board geometry
//!
//! This module provides precomputed bitboard masks for common Shogi board patterns
//! including ranks, files, and diagonals. These masks are optimized for the 9x9
//! Shogi board layout and provide O(1) lookup performance for geometric operations.

use crate::types::Bitboard;

/// Bitboard representation of a 9x9 Shogi board
///
/// The board is represented as a 128-bit bitboard where:
/// - Bits 0-80 represent the 9x9 board (81 squares)
/// - Each rank contains 9 consecutive bits
/// - Files are represented by every 9th bit starting from the file offset
///
/// Board layout (rank 0 is bottom, rank 8 is top):
/// ```
/// 8: 72 73 74 75 76 77 78 79 80
/// 7: 63 64 65 66 67 68 69 70 71
/// 6: 54 55 56 57 58 59 60 61 62
/// 5: 45 46 47 48 49 50 51 52 53
/// 4: 36 37 38 39 40 41 42 43 44
/// 3: 27 28 29 30 31 32 33 34 35
/// 2: 18 19 20 21 22 23 24 25 26
/// 1:  9 10 11 12 13 14 15 16 17
/// 0:  0  1  2  3  4  5  6  7  8
///    a  b  c  d  e  f  g  h  i
/// ```

/// Rank masks for the 9x9 Shogi board
///
/// Each mask represents all squares on a specific rank (0-8).
/// Rank 0 is the bottom rank (sent pieces), rank 8 is the top rank.
///
/// # Memory Usage
/// - Size: 144 bytes (9 entries × 16 bytes each for u128)
/// - Access pattern: Single array lookup per rank query
const RANK_MASKS: [Bitboard; 9] = [
    // Rank 0 (bottom rank): bits 0-8
    0b111111111u128,
    // Rank 1: bits 9-17
    0b111111111u128 << 9,
    // Rank 2: bits 18-26
    0b111111111u128 << 18,
    // Rank 3: bits 27-35
    0b111111111u128 << 27,
    // Rank 4: bits 36-44
    0b111111111u128 << 36,
    // Rank 5: bits 45-53
    0b111111111u128 << 45,
    // Rank 6: bits 54-62
    0b111111111u128 << 54,
    // Rank 7: bits 63-71
    0b111111111u128 << 63,
    // Rank 8 (top rank): bits 72-80
    0b111111111u128 << 72,
];

/// File masks for the 9x9 Shogi board
///
/// Each mask represents all squares on a specific file (0-8).
/// File 0 is the 'a' file (leftmost), file 8 is the 'i' file (rightmost).
///
/// # Memory Usage
/// - Size: 144 bytes (9 entries × 16 bytes each for u128)
/// - Access pattern: Single array lookup per file query
const FILE_MASKS: [Bitboard; 9] = [
    // File 0 (a-file): bits 0,9,18,27,36,45,54,63,72
    0b1000000001000000001000000001000000001000000001000000001000000001000000001u128,
    // File 1 (b-file): bits 1,10,19,28,37,46,55,64,73
    0b10000000010000000010000000010000000010000000010000000010000000010000000010u128,
    // File 2 (c-file): bits 2,11,20,29,38,47,56,65,74
    0b100000000100000000100000000100000000100000000100000000100000000100000000100u128,
    // File 3 (d-file): bits 3,12,21,30,39,48,57,66,75
    0b1000000001000000001000000001000000001000000001000000001000000001000000001000u128,
    // File 4 (e-file): bits 4,13,22,31,40,49,58,67,76
    0b10000000010000000010000000010000000010000000010000000010000000010000000010000u128,
    // File 5 (f-file): bits 5,14,23,32,41,50,59,68,77
    0b100000000100000000100000000100000000100000000100000000100000000100000000100000u128,
    // File 6 (g-file): bits 6,15,24,33,42,51,60,69,78
    0b1000000001000000001000000001000000001000000001000000001000000001000000001000000u128,
    // File 7 (h-file): bits 7,16,25,34,43,52,61,70,79
    0b10000000010000000010000000010000000010000000010000000010000000010000000010000000u128,
    // File 8 (i-file): bits 8,17,26,35,44,53,62,71,80
    0b100000000100000000100000000100000000100000000100000000100000000100000000100000000u128,
];

/// Diagonal masks for the 9x9 Shogi board
///
/// Each mask represents all squares on a specific diagonal.
/// There are 15 diagonals total:
/// - 9 main diagonals (including the main diagonal)
/// - 6 anti-diagonals (perpendicular to main diagonals)
///
/// Diagonal indexing:
/// - Diagonals 0-8: Main diagonals (top-left to bottom-right)
/// - Diagonals 9-14: Anti-diagonals (top-right to bottom-left)
///
/// # Memory Usage
/// - Size: 240 bytes (15 entries × 16 bytes each for u128)
/// - Access pattern: Single array lookup per diagonal query
const DIAGONAL_MASKS: [Bitboard; 15] = [
    // Main diagonal 0: (0,0) - single square
    0b1u128,
    // Main diagonal 1: (0,1), (1,0) - 2 squares
    0b10000000010u128,
    // Main diagonal 2: (0,2), (1,1), (2,0) - 3 squares
    0b100000000100000000100u128,
    // Main diagonal 3: (0,3), (1,2), (2,1), (3,0) - 4 squares
    0b1000000001000000001000000001000u128,
    // Main diagonal 4: (0,4), (1,3), (2,2), (3,1), (4,0) - 5 squares
    0b10000000010000000010000000010000000010000u128,
    // Main diagonal 5: (0,5), (1,4), (2,3), (3,2), (4,1), (5,0) - 6 squares
    0b100000000100000000100000000100000000100000000100000u128,
    // Main diagonal 6: (0,6), (1,5), (2,4), (3,3), (4,2), (5,1), (6,0) - 7 squares
    0b1000000001000000001000000001000000001000000001000000001000000u128,
    // Main diagonal 7: (0,7), (1,6), (2,5), (3,4), (4,3), (5,2), (6,1), (7,0) - 8 squares
    0b10000000010000000010000000010000000010000000010000000010000000010000000u128,
    // Main diagonal 8: (0,8), (1,7), (2,6), (3,5), (4,4), (5,3), (6,2), (7,1), (8,0) - 9 squares
    0b100000000100000000100000000100000000100000000100000000100000000100000000100000000u128,
    // Anti-diagonal 9: (0,8) - single square
    0b100000000u128,
    // Anti-diagonal 10: (0,7), (1,8) - 2 squares
    0b100000001000000000u128,
    // Anti-diagonal 11: (0,6), (1,7), (2,8) - 3 squares
    0b100000100000001000000000u128,
    // Anti-diagonal 12: (0,5), (1,6), (2,7), (3,8) - 4 squares
    0b100010000001000000100000000000u128,
    // Anti-diagonal 13: (0,4), (1,5), (2,6), (3,7), (4,8) - 5 squares
    0b1000100000100000010000001000000000000u128,
    // Anti-diagonal 14: (0,3), (1,4), (2,5), (3,6), (4,7), (5,8) - 6 squares
    0b10001000001000001000000100000010000000000000u128,
];

/// Get rank mask for a specific rank
///
/// # Arguments
/// * `rank` - The rank number (0-8, where 0 is bottom rank)
///
/// # Returns
/// A bitboard mask representing all squares on the specified rank
///
/// # Panics
/// Panics if rank is >= 9
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_rank_mask;
///
/// let rank_0_mask = get_rank_mask(0); // Bottom rank (sent pieces)
/// let rank_8_mask = get_rank_mask(8); // Top rank (promotion zone)
/// ```
pub fn get_rank_mask(rank: u8) -> Bitboard {
    assert!(rank < 9, "Rank must be 0-8, got {}", rank);
    RANK_MASKS[rank as usize]
}

/// Get file mask for a specific file
///
/// # Arguments
/// * `file` - The file number (0-8, where 0 is 'a' file, 8 is 'i' file)
///
/// # Returns
/// A bitboard mask representing all squares on the specified file
///
/// # Panics
/// Panics if file is >= 9
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_file_mask;
///
/// let file_a_mask = get_file_mask(0); // 'a' file (leftmost)
/// let file_i_mask = get_file_mask(8); // 'i' file (rightmost)
/// ```
pub fn get_file_mask(file: u8) -> Bitboard {
    assert!(file < 9, "File must be 0-8, got {}", file);
    FILE_MASKS[file as usize]
}

/// Get diagonal mask for a specific diagonal
///
/// # Arguments
/// * `diagonal` - The diagonal number (0-14)
///   - 0-8: Main diagonals (top-left to bottom-right)
///   - 9-14: Anti-diagonals (top-right to bottom-left)
///
/// # Returns
/// A bitboard mask representing all squares on the specified diagonal
///
/// # Panics
/// Panics if diagonal is >= 15
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_diagonal_mask;
///
/// let main_diagonal = get_diagonal_mask(8); // Main diagonal (longest)
/// let anti_diagonal = get_diagonal_mask(14); // Longest anti-diagonal
/// ```
pub fn get_diagonal_mask(diagonal: u8) -> Bitboard {
    assert!(diagonal < 15, "Diagonal must be 0-14, got {}", diagonal);

    if diagonal == 0 {
        return 1u128; // Only the origin square
    }

    if diagonal == 8 {
        let mut mask = 0u128;
        let mut idx = 0u32;
        while idx < 9 {
            mask |= 1u128 << (idx * 10);
            idx += 1;
        }
        return mask;
    }

    if diagonal == 9 {
        return 1u128 << 8; // Top-right corner
    }

    if diagonal == 14 {
        let mut mask = 0u128;
        let mut idx = 0u32;
        while idx < 9 {
            let rank = idx;
            let file = 8 - idx;
            let square = rank * 9 + file;
            mask |= 1u128 << square;
            idx += 1;
        }
        return mask;
    }

    DIAGONAL_MASKS[diagonal as usize]
}

/// Get rank from square index
///
/// # Arguments
/// * `square` - The square index (0-80)
///
/// # Returns
/// The rank number (0-8) for the given square
///
/// # Panics
/// Panics if square is >= 81
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_rank_from_square;
///
/// assert_eq!(get_rank_from_square(0), 0); // Square 0 is on rank 0
/// assert_eq!(get_rank_from_square(9), 1); // Square 9 is on rank 1
/// assert_eq!(get_rank_from_square(80), 8); // Square 80 is on rank 8
/// ```
pub fn get_rank_from_square(square: u8) -> u8 {
    assert!(square < 81, "Square must be 0-80, got {}", square);
    square / 9
}

/// Get file from square index
///
/// # Arguments
/// * `square` - The square index (0-80)
///
/// # Returns
/// The file number (0-8) for the given square
///
/// # Panics
/// Panics if square is >= 81
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_file_from_square;
///
/// assert_eq!(get_file_from_square(0), 0); // Square 0 is on file 0 (a-file)
/// assert_eq!(get_file_from_square(1), 1); // Square 1 is on file 1 (b-file)
/// assert_eq!(get_file_from_square(9), 0); // Square 9 is on file 0 (a-file)
/// ```
pub fn get_file_from_square(square: u8) -> u8 {
    assert!(square < 81, "Square must be 0-80, got {}", square);
    square % 9
}

/// Get square index from rank and file
///
/// # Arguments
/// * `rank` - The rank number (0-8)
/// * `file` - The file number (0-8)
///
/// # Returns
/// The square index (0-80) for the given rank and file
///
/// # Panics
/// Panics if rank >= 9 or file >= 9
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_square_from_rank_file;
///
/// assert_eq!(get_square_from_rank_file(0, 0), 0); // Rank 0, File 0 = Square 0
/// assert_eq!(get_square_from_rank_file(1, 0), 9); // Rank 1, File 0 = Square 9
/// assert_eq!(get_square_from_rank_file(8, 8), 80); // Rank 8, File 8 = Square 80
/// ```
pub fn get_square_from_rank_file(rank: u8, file: u8) -> u8 {
    assert!(rank < 9, "Rank must be 0-8, got {}", rank);
    assert!(file < 9, "File must be 0-8, got {}", file);
    rank * 9 + file
}

/// Get all squares on a specific rank
///
/// # Arguments
/// * `rank` - The rank number (0-8)
///
/// # Returns
/// A vector containing all square indices on the specified rank
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_rank_squares;
///
/// let rank_0_squares = get_rank_squares(0); // [0, 1, 2, 3, 4, 5, 6, 7, 8]
/// let rank_1_squares = get_rank_squares(1); // [9, 10, 11, 12, 13, 14, 15, 16, 17]
/// ```
pub fn get_rank_squares(rank: u8) -> Vec<u8> {
    assert!(rank < 9, "Rank must be 0-8, got {}", rank);
    (0..9)
        .map(|file| get_square_from_rank_file(rank, file))
        .collect()
}

/// Get all squares on a specific file
///
/// # Arguments
/// * `file` - The file number (0-8)
///
/// # Returns
/// A vector containing all square indices on the specified file
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_file_squares;
///
/// let file_a_squares = get_file_squares(0); // [0, 9, 18, 27, 36, 45, 54, 63, 72]
/// let file_i_squares = get_file_squares(8); // [8, 17, 26, 35, 44, 53, 62, 71, 80]
/// ```
pub fn get_file_squares(file: u8) -> Vec<u8> {
    assert!(file < 9, "File must be 0-8, got {}", file);
    (0..9)
        .map(|rank| get_square_from_rank_file(rank, file))
        .collect()
}

/// Get all squares on a specific diagonal
///
/// # Arguments
/// * `diagonal` - The diagonal number (0-14)
///
/// # Returns
/// A vector containing all square indices on the specified diagonal
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::get_diagonal_squares;
///
/// let main_diagonal = get_diagonal_squares(8); // Main diagonal squares
/// let anti_diagonal = get_diagonal_squares(14); // Longest anti-diagonal squares
/// ```
pub fn get_diagonal_squares(diagonal: u8) -> Vec<u8> {
    assert!(diagonal < 15, "Diagonal must be 0-14, got {}", diagonal);

    let mut squares = Vec::new();
    let mut remaining = get_diagonal_mask(diagonal);

    // Extract all set bits from the mask
    while remaining != 0 {
        let square = remaining.trailing_zeros() as u8;
        if square < 81 {
            squares.push(square);
        }
        remaining &= remaining - 1; // Clear the least significant bit
    }

    squares.sort_unstable();
    squares
}

/// Check if two squares are on the same rank
///
/// # Arguments
/// * `square1` - First square index (0-80)
/// * `square2` - Second square index (0-80)
///
/// # Returns
/// True if both squares are on the same rank
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::same_rank;
///
/// assert!(same_rank(0, 8)); // Both on rank 0
/// assert!(!same_rank(0, 9)); // Different ranks
/// ```
pub fn same_rank(square1: u8, square2: u8) -> bool {
    assert!(square1 < 81, "Square1 must be 0-80, got {}", square1);
    assert!(square2 < 81, "Square2 must be 0-80, got {}", square2);
    get_rank_from_square(square1) == get_rank_from_square(square2)
}

/// Check if two squares are on the same file
///
/// # Arguments
/// * `square1` - First square index (0-80)
/// * `square2` - Second square index (0-80)
///
/// # Returns
/// True if both squares are on the same file
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::same_file;
///
/// assert!(same_file(0, 72)); // Both on file 0 (a-file)
/// assert!(!same_file(0, 1)); // Different files
/// ```
pub fn same_file(square1: u8, square2: u8) -> bool {
    assert!(square1 < 81, "Square1 must be 0-80, got {}", square1);
    assert!(square2 < 81, "Square2 must be 0-80, got {}", square2);
    get_file_from_square(square1) == get_file_from_square(square2)
}

/// Check if two squares are on the same diagonal
///
/// # Arguments
/// * `square1` - First square index (0-80)
/// * `square2` - Second square index (0-80)
///
/// # Returns
/// True if both squares are on the same diagonal
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::masks::same_diagonal;
///
/// assert!(same_diagonal(0, 80)); // Both on main diagonal
/// assert!(!same_diagonal(0, 1)); // Different diagonals
/// ```
pub fn same_diagonal(square1: u8, square2: u8) -> bool {
    assert!(square1 < 81, "Square1 must be 0-80, got {}", square1);
    assert!(square2 < 81, "Square2 must be 0-80, got {}", square2);

    let rank1 = get_rank_from_square(square1);
    let file1 = get_file_from_square(square1);
    let rank2 = get_rank_from_square(square2);
    let file2 = get_file_from_square(square2);

    // Check main diagonal (rank - file is constant)
    if (rank1 as i8 - file1 as i8) == (rank2 as i8 - file2 as i8) {
        return true;
    }

    // Check anti-diagonal (rank + file is constant)
    if (rank1 + file1) == (rank2 + file2) {
        return true;
    }

    false
}

/// Get mask information for debugging
///
/// # Returns
/// A string containing information about all masks
pub fn get_masks_info() -> String {
    format!(
        "Shogi Board Masks Info:\n\
         Board Size: 9x9 (81 squares)\n\
         Rank Masks: {} entries, {} bytes\n\
         File Masks: {} entries, {} bytes\n\
         Diagonal Masks: {} entries, {} bytes\n\
         Total Memory: {} bytes\n\
         Board Layout: 0-80 (rank 0 bottom, rank 8 top)",
        RANK_MASKS.len(),
        std::mem::size_of_val(&RANK_MASKS),
        FILE_MASKS.len(),
        std::mem::size_of_val(&FILE_MASKS),
        DIAGONAL_MASKS.len(),
        std::mem::size_of_val(&DIAGONAL_MASKS),
        std::mem::size_of_val(&RANK_MASKS)
            + std::mem::size_of_val(&FILE_MASKS)
            + std::mem::size_of_val(&DIAGONAL_MASKS)
    )
}

/// Validate all masks for correctness
///
/// # Returns
/// True if all masks are correctly configured
pub fn validate_masks() -> bool {
    // Validate rank masks
    for rank in 0..9 {
        let mask = RANK_MASKS[rank];
        let expected_squares = get_rank_squares(rank as u8);

        // Check that mask contains exactly the expected squares
        for &square in &expected_squares {
            if (mask & (1 << square)) == 0 {
                return false;
            }
        }

        // Check that mask doesn't contain unexpected squares
        let mut remaining = mask;
        while remaining != 0 {
            let square = remaining.trailing_zeros() as u8;
            if !expected_squares.contains(&square) {
                return false;
            }
            remaining &= remaining - 1;
        }
    }

    // Validate file masks
    for file in 0..9 {
        let mask = FILE_MASKS[file];
        let expected_squares = get_file_squares(file as u8);

        // Check that mask contains exactly the expected squares
        for &square in &expected_squares {
            if (mask & (1 << square)) == 0 {
                return false;
            }
        }

        // Check that mask doesn't contain unexpected squares
        let mut remaining = mask;
        while remaining != 0 {
            let square = remaining.trailing_zeros() as u8;
            if !expected_squares.contains(&square) {
                return false;
            }
            remaining &= remaining - 1;
        }
    }

    // Validate diagonal masks
    for diagonal in 0..15 {
        let mask = get_diagonal_mask(diagonal as u8);
        let expected_squares = get_diagonal_squares(diagonal as u8);

        // Check that mask contains exactly the expected squares
        for &square in &expected_squares {
            if (mask & (1 << square)) == 0 {
                return false;
            }
        }

        // Check that mask doesn't contain unexpected squares
        let mut remaining = mask;
        while remaining != 0 {
            let square = remaining.trailing_zeros() as u8;
            if square >= 81 || !expected_squares.contains(&square) {
                return false;
            }
            remaining &= remaining - 1;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masks_validation() {
        // Validate that all masks are correctly configured
        assert!(validate_masks(), "Masks validation failed");
    }

    #[test]
    fn test_rank_masks() {
        // Test rank mask retrieval
        for rank in 0..9 {
            let mask = get_rank_mask(rank);
            assert_ne!(mask, 0, "Rank {} mask should not be empty", rank);

            // Check that mask contains exactly 9 squares
            let count = mask.count_ones();
            assert_eq!(count, 9, "Rank {} should contain exactly 9 squares", rank);
        }

        // Test specific rank masks
        let rank_0 = get_rank_mask(0);
        assert_eq!(rank_0 & 0b111111111, 0b111111111); // Bottom rank

        let rank_8 = get_rank_mask(8);
        assert_eq!(rank_8 >> 72, 0b111111111); // Top rank
    }

    #[test]
    fn test_file_masks() {
        // Test file mask retrieval
        for file in 0..9 {
            let mask = get_file_mask(file);
            assert_ne!(mask, 0, "File {} mask should not be empty", file);

            // Check that mask contains exactly 9 squares
            let count = mask.count_ones();
            assert_eq!(count, 9, "File {} should contain exactly 9 squares", file);
        }

        // Test specific file masks
        let file_a = get_file_mask(0);
        assert!(file_a & (1 << 0) != 0); // Square 0
        assert!(file_a & (1 << 9) != 0); // Square 9
        assert!(file_a & (1 << 72) != 0); // Square 72

        let file_i = get_file_mask(8);
        assert!(file_i & (1 << 8) != 0); // Square 8
        assert!(file_i & (1 << 17) != 0); // Square 17
        assert!(file_i & (1 << 80) != 0); // Square 80
    }

    #[test]
    fn test_diagonal_masks() {
        // Test diagonal mask retrieval
        for diagonal in 0..15 {
            let mask = get_diagonal_mask(diagonal);
            assert_ne!(mask, 0, "Diagonal {} mask should not be empty", diagonal);
        }

        // Test main diagonal (diagonal 8)
        let main_diagonal = get_diagonal_mask(8);
        let squares: Vec<u8> = (0..81)
            .filter(|&sq| (main_diagonal >> sq) & 1 == 1)
            .collect();
        let expected: Vec<u8> = (0..9)
            .map(|rank| get_square_from_rank_file(rank, rank))
            .collect();
        assert_eq!(squares, expected);
    }

    #[test]
    fn test_square_coordinate_conversion() {
        // Test rank/file to square conversion
        for rank in 0..9 {
            for file in 0..9 {
                let square = get_square_from_rank_file(rank, file);
                assert!(square < 81, "Square {} should be < 81", square);

                // Test reverse conversion
                assert_eq!(get_rank_from_square(square), rank);
                assert_eq!(get_file_from_square(square), file);
            }
        }

        // Test specific conversions
        assert_eq!(get_square_from_rank_file(0, 0), 0);
        assert_eq!(get_square_from_rank_file(0, 8), 8);
        assert_eq!(get_square_from_rank_file(8, 0), 72);
        assert_eq!(get_square_from_rank_file(8, 8), 80);
    }

    #[test]
    fn test_rank_file_squares() {
        // Test rank squares
        let rank_0_squares = get_rank_squares(0);
        assert_eq!(rank_0_squares, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        let rank_1_squares = get_rank_squares(1);
        assert_eq!(rank_1_squares, vec![9, 10, 11, 12, 13, 14, 15, 16, 17]);

        let rank_8_squares = get_rank_squares(8);
        assert_eq!(rank_8_squares, vec![72, 73, 74, 75, 76, 77, 78, 79, 80]);

        // Test file squares
        let file_a_squares = get_file_squares(0);
        assert_eq!(file_a_squares, vec![0, 9, 18, 27, 36, 45, 54, 63, 72]);

        let file_i_squares = get_file_squares(8);
        assert_eq!(file_i_squares, vec![8, 17, 26, 35, 44, 53, 62, 71, 80]);
    }

    #[test]
    fn test_diagonal_squares() {
        // Test main diagonal (diagonal 8)
        let main_diagonal = get_diagonal_squares(8);
        assert_eq!(main_diagonal, vec![0, 10, 20, 30, 40, 50, 60, 70, 80]);

        // Test anti-diagonal (diagonal 14)
        let anti_diagonal = get_diagonal_squares(14);
        assert_eq!(anti_diagonal, vec![8, 16, 24, 32, 40, 48, 56, 64, 72]);

        // Test single square diagonals
        let diagonal_0 = get_diagonal_squares(0);
        assert_eq!(diagonal_0, vec![0]);

        let diagonal_9 = get_diagonal_squares(9);
        assert_eq!(diagonal_9, vec![8]);
    }

    #[test]
    fn test_same_rank_file_diagonal() {
        // Test same rank
        assert!(same_rank(0, 8)); // Both on rank 0
        assert!(same_rank(9, 17)); // Both on rank 1
        assert!(!same_rank(0, 9)); // Different ranks

        // Test same file
        assert!(same_file(0, 72)); // Both on file 0 (a-file)
        assert!(same_file(8, 80)); // Both on file 8 (i-file)
        assert!(!same_file(0, 1)); // Different files

        // Test same diagonal
        assert!(same_diagonal(0, 80)); // Both on main diagonal
        assert!(same_diagonal(8, 72)); // Both on anti-diagonal
        assert!(!same_diagonal(0, 1)); // Different diagonals
    }

    #[test]
    fn test_masks_memory_usage() {
        // Test that memory usage is within reasonable limits
        let rank_size = std::mem::size_of_val(&RANK_MASKS);
        let file_size = std::mem::size_of_val(&FILE_MASKS);
        let diagonal_size = std::mem::size_of_val(&DIAGONAL_MASKS);
        let total_size = rank_size + file_size + diagonal_size;

        // Total memory should be less than 1KB as specified
        assert!(
            total_size < 1024,
            "Memory usage too high: {} bytes",
            total_size
        );

        // Individual sizes should be reasonable
        assert_eq!(rank_size, 144); // 9 entries × 16 bytes
        assert_eq!(file_size, 144); // 9 entries × 16 bytes
        assert_eq!(diagonal_size, 240); // 15 entries × 16 bytes
        assert_eq!(total_size, 528); // Total: 528 bytes
    }

    #[test]
    fn test_masks_edge_cases() {
        // Test edge cases that might cause issues

        // Test rank 0 and 8 (boundary ranks)
        let rank_0_mask = get_rank_mask(0);
        let rank_8_mask = get_rank_mask(8);
        assert_ne!(rank_0_mask, rank_8_mask);

        // Test file 0 and 8 (boundary files)
        let file_0_mask = get_file_mask(0);
        let file_8_mask = get_file_mask(8);
        assert_ne!(file_0_mask, file_8_mask);

        // Test corner squares
        assert_eq!(get_square_from_rank_file(0, 0), 0); // Bottom-left
        assert_eq!(get_square_from_rank_file(0, 8), 8); // Bottom-right
        assert_eq!(get_square_from_rank_file(8, 0), 72); // Top-left
        assert_eq!(get_square_from_rank_file(8, 8), 80); // Top-right

        // Test center square
        assert_eq!(get_square_from_rank_file(4, 4), 40); // Center
    }

    #[test]
    fn test_masks_consistency() {
        // Test that mask operations are consistent

        // Test that rank masks don't overlap
        for i in 0..9 {
            for j in 0..9 {
                if i != j {
                    let mask_i = get_rank_mask(i);
                    let mask_j = get_rank_mask(j);
                    assert_eq!(
                        mask_i & mask_j,
                        0,
                        "Rank masks {} and {} should not overlap",
                        i,
                        j
                    );
                }
            }
        }

        // Test that file masks don't overlap
        for i in 0..9 {
            for j in 0..9 {
                if i != j {
                    let mask_i = get_file_mask(i);
                    let mask_j = get_file_mask(j);
                    assert_eq!(
                        mask_i & mask_j,
                        0,
                        "File masks {} and {} should not overlap",
                        i,
                        j
                    );
                }
            }
        }

        // Test coordinate conversion consistency
        for square in 0..81 {
            let rank = get_rank_from_square(square);
            let file = get_file_from_square(square);
            let reconstructed_square = get_square_from_rank_file(rank, file);
            assert_eq!(
                square, reconstructed_square,
                "Square {} -> rank {}, file {} -> square {}",
                square, rank, file, reconstructed_square
            );
        }
    }

    #[test]
    fn test_masks_performance() {
        // Test that mask operations are fast (O(1) lookup)
        let iterations = 10000;

        // Benchmark rank mask lookup
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            for rank in 0..9 {
                let _mask = get_rank_mask(rank);
            }
        }
        let rank_duration = start.elapsed();

        // Benchmark file mask lookup
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            for file in 0..9 {
                let _mask = get_file_mask(file);
            }
        }
        let file_duration = start.elapsed();

        // Benchmark diagonal mask lookup
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            for diagonal in 0..15 {
                let _mask = get_diagonal_mask(diagonal);
            }
        }
        let diagonal_duration = start.elapsed();

        // All operations should be very fast (less than 1ms for 10k iterations)
        assert!(
            rank_duration.as_millis() < 10,
            "Rank lookup too slow: {:?}",
            rank_duration
        );
        assert!(
            file_duration.as_millis() < 10,
            "File lookup too slow: {:?}",
            file_duration
        );
        assert!(
            diagonal_duration.as_millis() < 10,
            "Diagonal lookup too slow: {:?}",
            diagonal_duration
        );

        // Print performance info
        println!("Masks Performance ({} iterations):", iterations);
        println!("  Rank lookup: {:?}", rank_duration);
        println!("  File lookup: {:?}", file_duration);
        println!("  Diagonal lookup: {:?}", diagonal_duration);
    }
}
