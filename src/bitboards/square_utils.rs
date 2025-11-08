//! Square coordinate conversion utilities for bitboard operations
//!
//! This module provides comprehensive utilities for converting between bit positions,
//! square coordinates, and various coordinate systems used in Shogi. It integrates
//! with the existing Position type and provides efficient conversion functions
//! optimized for bitboard operations.

use crate::types::{Bitboard, Position};

/// Convert a bit position to a Position struct
///
/// # Arguments
/// * `bit` - The bit position (0-80 for 9x9 board, 0-127 for extended bitboard)
///
/// # Returns
/// A Position struct with row and col coordinates
///
/// # Panics
/// This function will panic if the bit position is >= 128
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::bit_to_square;
///
/// let pos = bit_to_square(0);  // Top-left corner
/// assert_eq!(pos.row, 0);
/// assert_eq!(pos.col, 0);
///
/// let pos = bit_to_square(8);  // First square of second rank
/// assert_eq!(pos.row, 1);
/// assert_eq!(pos.col, 0);
///
/// let pos = bit_to_square(80); // Bottom-right corner of 9x9 board
/// assert_eq!(pos.row, 8);
/// assert_eq!(pos.col, 8);
/// ```
pub fn bit_to_square(bit: u8) -> Position {
    if bit >= 128 {
        panic!("Bit position {} is out of range (must be < 128)", bit);
    }

    let row = bit / 9;
    let col = bit % 9;

    // Clamp to 9x9 board for Shogi
    if row >= 9 {
        Position::new(8, 8) // Return bottom-right corner for extended positions
    } else {
        Position::new(row, col)
    }
}

/// Convert a Position to a bit position
///
/// # Arguments
/// * `square` - The Position struct containing row and col coordinates
///
/// # Returns
/// The corresponding bit position (0-80 for valid 9x9 squares)
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::square_to_bit;
/// use shogi_engine::types::Position;
///
/// let pos = Position::new(0, 0); // Top-left corner
/// assert_eq!(square_to_bit(pos), 0);
///
/// let pos = Position::new(1, 0); // First square of second rank
/// assert_eq!(square_to_bit(pos), 9);
///
/// let pos = Position::new(8, 8); // Bottom-right corner
/// assert_eq!(square_to_bit(pos), 80);
/// ```
pub fn square_to_bit(square: Position) -> u8 {
    square.row * 9 + square.col
}

/// Convert a bit position to (file, rank) coordinates
///
/// # Arguments
/// * `bit` - The bit position (0-80 for 9x9 board)
///
/// # Returns
/// A tuple (file, rank) where both are 0-based indices
///
/// # Panics
/// This function will panic if the bit position is >= 128
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::bit_to_coords;
///
/// let (file, rank) = bit_to_coords(0);  // Top-left corner
/// assert_eq!(file, 0);
/// assert_eq!(rank, 0);
///
/// let (file, rank) = bit_to_coords(8);  // First square of second rank
/// assert_eq!(file, 0);
/// assert_eq!(rank, 1);
///
/// let (file, rank) = bit_to_coords(80); // Bottom-right corner
/// assert_eq!(file, 8);
/// assert_eq!(rank, 8);
/// ```
pub fn bit_to_coords(bit: u8) -> (u8, u8) {
    if bit >= 128 {
        panic!("Bit position {} is out of range (must be < 128)", bit);
    }

    let file = bit % 9;
    let rank = bit / 9;

    (file, rank)
}

/// Convert (file, rank) coordinates to a bit position
///
/// # Arguments
/// * `file` - The file coordinate (0-8, left to right)
/// * `rank` - The rank coordinate (0-8, top to bottom)
///
/// # Returns
/// The corresponding bit position (0-80 for valid coordinates)
///
/// # Panics
/// This function will panic if either coordinate is >= 9
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::coords_to_bit;
///
/// assert_eq!(coords_to_bit(0, 0), 0);  // Top-left corner
/// assert_eq!(coords_to_bit(0, 1), 9);  // First square of second rank
/// assert_eq!(coords_to_bit(8, 8), 80); // Bottom-right corner
/// ```
pub fn coords_to_bit(file: u8, rank: u8) -> u8 {
    if file >= 9 || rank >= 9 {
        panic!(
            "Coordinates ({}, {}) are out of range (must be < 9)",
            file, rank
        );
    }

    rank * 9 + file
}

/// Convert a bit position to a square name in algebraic notation
///
/// # Arguments
/// * `bit` - The bit position (0-80 for 9x9 board)
///
/// # Returns
/// A string representing the square in algebraic notation (e.g., "9i", "5e")
///
/// # Panics
/// This function will panic if the bit position is >= 128
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::bit_to_square_name;
///
/// assert_eq!(bit_to_square_name(0), "1i");   // Top-left corner
/// assert_eq!(bit_to_square_name(4), "5i");   // Center of top rank
/// assert_eq!(bit_to_square_name(40), "5e");  // Center of board
/// assert_eq!(bit_to_square_name(80), "9a");  // Bottom-right corner
/// ```
pub fn bit_to_square_name(bit: u8) -> String {
    if bit >= 128 {
        panic!("Bit position {} is out of range (must be < 128)", bit);
    }

    let (file, rank) = bit_to_coords(bit);

    // Convert to Shogi algebraic notation
    // Files: 1-9 (left to right), Ranks: a-i (bottom to top)
    let file_name = (file + 1).to_string();
    let rank_name = match 8 - rank {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        8 => 'i',
        _ => 'a', // Fallback
    };

    format!("{}{}", file_name, rank_name)
}

/// Convert a square name in algebraic notation to a bit position
///
/// # Arguments
/// * `name` - The square name in algebraic notation (e.g., "9i", "5e")
///
/// # Returns
/// The corresponding bit position (0-80 for valid squares)
///
/// # Panics
/// This function will panic if the square name is invalid
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::square_name_to_bit;
///
/// assert_eq!(square_name_to_bit("1i"), 0);   // Top-left corner
/// assert_eq!(square_name_to_bit("5i"), 4);   // Center of top rank
/// assert_eq!(square_name_to_bit("5e"), 40);  // Center of board
/// assert_eq!(square_name_to_bit("9a"), 80);  // Bottom-right corner
/// ```
pub fn square_name_to_bit(name: &str) -> u8 {
    if name.len() != 2 {
        panic!("Invalid square name '{}': must be 2 characters", name);
    }

    let chars: Vec<char> = name.chars().collect();
    let file_char = chars[0];
    let rank_char = chars[1];

    // Parse file (1-9)
    let file = match file_char.to_digit(10) {
        Some(f) if f >= 1 && f <= 9 => (f - 1) as u8,
        _ => panic!(
            "Invalid file '{}' in square name '{}': must be 1-9",
            file_char, name
        ),
    };

    // Parse rank (a-i)
    let rank = match rank_char {
        'a' => 8,
        'b' => 7,
        'c' => 6,
        'd' => 5,
        'e' => 4,
        'f' => 3,
        'g' => 2,
        'h' => 1,
        'i' => 0,
        _ => panic!(
            "Invalid rank '{}' in square name '{}': must be a-i",
            rank_char, name
        ),
    };

    coords_to_bit(file, rank)
}

/// Check if a bit position is on a 9x9 Shogi board
///
/// # Arguments
/// * `bit` - The bit position to check
///
/// # Returns
/// True if the bit position corresponds to a valid square on a 9x9 board
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::is_valid_shogi_square;
///
/// assert!(is_valid_shogi_square(0));   // Top-left corner
/// assert!(is_valid_shogi_square(40));  // Center of board
/// assert!(is_valid_shogi_square(80));  // Bottom-right corner
/// assert!(!is_valid_shogi_square(81)); // Beyond 9x9 board
/// assert!(!is_valid_shogi_square(127)); // Extended bitboard position
/// ```
pub fn is_valid_shogi_square(bit: u8) -> bool {
    bit < 81 // 9x9 = 81 squares
}

/// Check if a bit position is in the promotion zone for a given player
///
/// # Arguments
/// * `bit` - The bit position to check
/// * `player` - The player (Black or White)
///
/// # Returns
/// True if the square is in the promotion zone for the given player
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::is_promotion_zone;
/// use shogi_engine::types::Player;
///
/// // Black's promotion zone (ranks 7, 8, 9)
/// assert!(is_promotion_zone(63, Player::Black)); // Rank 7
/// assert!(is_promotion_zone(72, Player::Black)); // Rank 8
/// assert!(is_promotion_zone(80, Player::Black)); // Rank 9
///
/// // White's promotion zone (ranks 1, 2, 3)
/// assert!(is_promotion_zone(0, Player::White));  // Rank 1
/// assert!(is_promotion_zone(9, Player::White));  // Rank 2
/// assert!(is_promotion_zone(18, Player::White)); // Rank 3
/// ```
pub fn is_promotion_zone(bit: u8, player: crate::types::Player) -> bool {
    if !is_valid_shogi_square(bit) {
        return false;
    }

    let (_, rank) = bit_to_coords(bit);

    match player {
        crate::types::Player::Black => rank >= 6, // Ranks 7, 8, 9 (0-based: 6, 7, 8)
        crate::types::Player::White => rank <= 2, // Ranks 1, 2, 3 (0-based: 0, 1, 2)
    }
}

/// Get the distance between two squares (Manhattan distance)
///
/// # Arguments
/// * `bit1` - First bit position
/// * `bit2` - Second bit position
///
/// # Returns
/// The Manhattan distance between the two squares
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::square_distance;
///
/// assert_eq!(square_distance(0, 0), 0);   // Same square
/// assert_eq!(square_distance(0, 1), 1);   // Adjacent horizontally
/// assert_eq!(square_distance(0, 9), 1);   // Adjacent vertically
/// assert_eq!(square_distance(0, 10), 2);  // Diagonal (1+1)
/// ```
pub fn square_distance(bit1: u8, bit2: u8) -> u8 {
    let (file1, rank1) = bit_to_coords(bit1);
    let (file2, rank2) = bit_to_coords(bit2);

    let file_diff = if file1 > file2 {
        file1 - file2
    } else {
        file2 - file1
    };
    let rank_diff = if rank1 > rank2 {
        rank1 - rank2
    } else {
        rank2 - rank1
    };

    file_diff + rank_diff
}

/// Get all squares on the same rank as the given square
///
/// # Arguments
/// * `bit` - The bit position
///
/// # Returns
/// A vector of all bit positions on the same rank
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::get_rank_squares;
///
/// let rank_squares = get_rank_squares(4); // Center of top rank
/// assert_eq!(rank_squares.len(), 9);
/// assert!(rank_squares.contains(&0)); // Left edge
/// assert!(rank_squares.contains(&4)); // Center
/// assert!(rank_squares.contains(&8)); // Right edge
/// ```
pub fn get_rank_squares(bit: u8) -> Vec<u8> {
    let (_, rank) = bit_to_coords(bit);
    (0..9).map(|file| coords_to_bit(file, rank)).collect()
}

/// Get all squares on the same file as the given square
///
/// # Arguments
/// * `bit` - The bit position
///
/// # Returns
/// A vector of all bit positions on the same file
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::get_file_squares;
///
/// let file_squares = get_file_squares(4); // Center of top rank
/// assert_eq!(file_squares.len(), 9);
/// assert!(file_squares.contains(&4));  // Top
/// assert!(file_squares.contains(&40)); // Center
/// assert!(file_squares.contains(&76)); // Bottom
/// ```
pub fn get_file_squares(bit: u8) -> Vec<u8> {
    let (file, _) = bit_to_coords(bit);
    (0..9).map(|rank| coords_to_bit(file, rank)).collect()
}

/// Get all squares on the same diagonal as the given square
///
/// # Arguments
/// * `bit` - The bit position
///
/// # Returns
/// A vector of all bit positions on the same diagonal
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::get_diagonal_squares;
///
/// let diag_squares = get_diagonal_squares(40); // Center of board
/// assert!(diag_squares.contains(&0));  // Top-left
/// assert!(diag_squares.contains(&40)); // Center
/// assert!(diag_squares.contains(&80)); // Bottom-right
/// ```
pub fn get_diagonal_squares(bit: u8) -> Vec<u8> {
    let (file, rank) = bit_to_coords(bit);
    let mut squares = Vec::new();

    // Main diagonal (top-left to bottom-right)
    for i in 0..9 {
        let f = file as i8 + (i as i8 - file as i8);
        let r = rank as i8 + (i as i8 - rank as i8);
        if f >= 0 && f < 9 && r >= 0 && r < 9 {
            squares.push(coords_to_bit(f as u8, r as u8));
        }
    }

    // Anti-diagonal (top-right to bottom-left)
    for i in 0..9 {
        let f = file as i8 + (rank as i8 - i as i8);
        let r = i as i8;
        if f >= 0 && f < 9 && r >= 0 && r < 9 {
            let square = coords_to_bit(f as u8, r as u8);
            if !squares.contains(&square) {
                squares.push(square);
            }
        }
    }

    squares.sort();
    squares
}

/// Create a bitboard mask for a specific rank
///
/// # Arguments
/// * `rank` - The rank number (0-8)
///
/// # Returns
/// A bitboard with bits set for all squares on the specified rank
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::rank_mask;
///
/// let rank_0 = rank_mask(0); // Top rank
/// assert_eq!(rank_0 & 0xFF, 0x1FF); // First 9 bits set
///
/// let rank_8 = rank_mask(8); // Bottom rank
/// assert_eq!(rank_8 >> 72, 0x1FF); // Last 9 bits set
/// ```
pub fn rank_mask(rank: u8) -> Bitboard {
    if rank >= 9 {
        return 0;
    }

    let start_bit = rank * 9;
    0x1FFu128 << start_bit
}

/// Create a bitboard mask for a specific file
///
/// # Arguments
/// * `file` - The file number (0-8)
///
/// # Returns
/// A bitboard with bits set for all squares on the specified file
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::file_mask;
///
/// let file_0 = file_mask(0); // Leftmost file
/// // Pattern: bit 0, 9, 18, 27, 36, 45, 54, 63, 72
///
/// let file_8 = file_mask(8); // Rightmost file
/// // Pattern: bit 8, 17, 26, 35, 44, 53, 62, 71, 80
/// ```
pub fn file_mask(file: u8) -> Bitboard {
    if file >= 9 {
        return 0;
    }

    let mut mask = 0u128;
    for rank in 0..9 {
        mask |= 1u128 << (rank * 9 + file);
    }
    mask
}

/// Create a bitboard mask for the promotion zone of a given player
///
/// # Arguments
/// * `player` - The player (Black or White)
///
/// # Returns
/// A bitboard with bits set for all squares in the player's promotion zone
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::promotion_zone_mask;
/// use shogi_engine::types::Player;
///
/// let black_zone = promotion_zone_mask(Player::Black);
/// // Should have bits set for ranks 6, 7, 8 (0-based)
///
/// let white_zone = promotion_zone_mask(Player::White);
/// // Should have bits set for ranks 0, 1, 2 (0-based)
/// ```
pub fn promotion_zone_mask(player: crate::types::Player) -> Bitboard {
    let mut mask = 0u128;

    for bit in 0..81 {
        if is_promotion_zone(bit, player) {
            mask |= 1u128 << bit;
        }
    }

    mask
}

/// Get the center squares of the board
///
/// # Returns
/// A vector of bit positions for the center squares (5e, 4e, 6e, 5d, 5f)
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::get_center_squares;
///
/// let center = get_center_squares();
/// assert_eq!(center.len(), 5);
/// assert!(center.contains(&40)); // 5e (center)
/// ```
pub fn get_center_squares() -> Vec<u8> {
    vec![
        40, // 5e (center)
        39, // 4e
        41, // 6e
        31, // 5d
        49, // 5f
    ]
}

/// Check if a square is in the center area of the board
///
/// # Arguments
/// * `bit` - The bit position
///
/// # Returns
/// True if the square is in the center area (3x3 center)
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::square_utils::is_center_square;
///
/// assert!(is_center_square(40));  // 5e (center)
/// assert!(is_center_square(31));  // 5d
/// assert!(!is_center_square(0));  // 9i (corner)
/// ```
pub fn is_center_square(bit: u8) -> bool {
    let (file, rank) = bit_to_coords(bit);
    file >= 3 && file <= 5 && rank >= 3 && rank <= 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Player;

    #[test]
    fn test_bit_to_square() {
        // Test corner cases
        assert_eq!(bit_to_square(0), Position::new(0, 0));
        assert_eq!(bit_to_square(8), Position::new(0, 8));
        assert_eq!(bit_to_square(72), Position::new(8, 0));
        assert_eq!(bit_to_square(80), Position::new(8, 8));

        // Test center
        assert_eq!(bit_to_square(40), Position::new(4, 4));

        // Test edge case (should clamp)
        assert_eq!(bit_to_square(127), Position::new(8, 8));
    }

    #[test]
    fn test_square_to_bit() {
        // Test corner cases
        assert_eq!(square_to_bit(Position::new(0, 0)), 0);
        assert_eq!(square_to_bit(Position::new(0, 8)), 8);
        assert_eq!(square_to_bit(Position::new(8, 0)), 72);
        assert_eq!(square_to_bit(Position::new(8, 8)), 80);

        // Test center
        assert_eq!(square_to_bit(Position::new(4, 4)), 40);
    }

    #[test]
    fn test_bit_to_coords() {
        // Test corner cases
        assert_eq!(bit_to_coords(0), (0, 0));
        assert_eq!(bit_to_coords(8), (8, 0));
        assert_eq!(bit_to_coords(72), (0, 8));
        assert_eq!(bit_to_coords(80), (8, 8));

        // Test center
        assert_eq!(bit_to_coords(40), (4, 4));
    }

    #[test]
    fn test_coords_to_bit() {
        // Test corner cases
        assert_eq!(coords_to_bit(0, 0), 0);
        assert_eq!(coords_to_bit(8, 0), 8);
        assert_eq!(coords_to_bit(0, 8), 72);
        assert_eq!(coords_to_bit(8, 8), 80);

        // Test center
        assert_eq!(coords_to_bit(4, 4), 40);
    }

    #[test]
    fn test_bit_to_square_name() {
        // Test corner cases
        assert_eq!(bit_to_square_name(0), "1i");
        assert_eq!(bit_to_square_name(8), "9i");
        assert_eq!(bit_to_square_name(72), "1a");
        assert_eq!(bit_to_square_name(80), "9a");

        // Test center
        assert_eq!(bit_to_square_name(40), "5e");

        // Test some other positions
        assert_eq!(bit_to_square_name(4), "5i");
        assert_eq!(bit_to_square_name(76), "5a");
    }

    #[test]
    fn test_square_name_to_bit() {
        // Test corner cases
        assert_eq!(square_name_to_bit("1i"), 0);
        assert_eq!(square_name_to_bit("9i"), 8);
        assert_eq!(square_name_to_bit("1a"), 72);
        assert_eq!(square_name_to_bit("9a"), 80);

        // Test center
        assert_eq!(square_name_to_bit("5e"), 40);

        // Test some other positions
        assert_eq!(square_name_to_bit("5i"), 4);
        assert_eq!(square_name_to_bit("5a"), 76);
    }

    #[test]
    fn test_is_valid_shogi_square() {
        // Valid squares
        assert!(is_valid_shogi_square(0));
        assert!(is_valid_shogi_square(40));
        assert!(is_valid_shogi_square(80));

        // Invalid squares
        assert!(!is_valid_shogi_square(81));
        assert!(!is_valid_shogi_square(127));
    }

    #[test]
    fn test_is_promotion_zone() {
        // Black's promotion zone (ranks 7, 8, 9)
        assert!(is_promotion_zone(63, Player::Black)); // Rank 7
        assert!(is_promotion_zone(72, Player::Black)); // Rank 8
        assert!(is_promotion_zone(80, Player::Black)); // Rank 9

        // White's promotion zone (ranks 1, 2, 3)
        assert!(is_promotion_zone(0, Player::White)); // Rank 1
        assert!(is_promotion_zone(9, Player::White)); // Rank 2
        assert!(is_promotion_zone(18, Player::White)); // Rank 3

        // Not in promotion zones
        assert!(!is_promotion_zone(40, Player::Black)); // Center
        assert!(!is_promotion_zone(40, Player::White)); // Center
    }

    #[test]
    fn test_square_distance() {
        assert_eq!(square_distance(0, 0), 0);
        assert_eq!(square_distance(0, 1), 1);
        assert_eq!(square_distance(0, 9), 1);
        assert_eq!(square_distance(0, 10), 2);
        assert_eq!(square_distance(0, 80), 16); // Corner to corner
    }

    #[test]
    fn test_get_rank_squares() {
        let rank_0 = get_rank_squares(4); // Center of top rank
        assert_eq!(rank_0.len(), 9);
        for i in 0..9 {
            assert!(rank_0.contains(&i));
        }

        let rank_4 = get_rank_squares(40); // Center of middle rank
        assert_eq!(rank_4.len(), 9);
        for i in 36..45 {
            assert!(rank_4.contains(&i));
        }
    }

    #[test]
    fn test_get_file_squares() {
        let file_4 = get_file_squares(40); // Center file
        assert_eq!(file_4.len(), 9);
        assert!(file_4.contains(&4)); // Top
        assert!(file_4.contains(&40)); // Center
        assert!(file_4.contains(&76)); // Bottom
    }

    #[test]
    fn test_get_diagonal_squares() {
        let diag = get_diagonal_squares(40); // Center
        assert!(diag.contains(&0)); // Top-left
        assert!(diag.contains(&40)); // Center
        assert!(diag.contains(&80)); // Bottom-right

        // Should include anti-diagonal too
        assert!(diag.contains(&8)); // Top-right
        assert!(diag.contains(&72)); // Bottom-left
    }

    #[test]
    fn test_rank_mask() {
        let rank_0 = rank_mask(0);
        assert_eq!(rank_0 & 0x1FF, 0x1FF);

        let rank_8 = rank_mask(8);
        assert_eq!(rank_8 >> 72, 0x1FF);

        // Test invalid rank
        assert_eq!(rank_mask(9), 0);
    }

    #[test]
    fn test_file_mask() {
        let file_0 = file_mask(0);
        // Should have bits 0, 9, 18, 27, 36, 45, 54, 63, 72
        for i in 0..9 {
            assert_ne!(file_0 & (1u128 << (i * 9)), 0);
        }

        let file_8 = file_mask(8);
        // Should have bits 8, 17, 26, 35, 44, 53, 62, 71, 80
        for i in 0..9 {
            assert_ne!(file_8 & (1u128 << (i * 9 + 8)), 0);
        }

        // Test invalid file
        assert_eq!(file_mask(9), 0);
    }

    #[test]
    fn test_promotion_zone_mask() {
        let black_zone = promotion_zone_mask(Player::Black);
        // Should have bits set for ranks 6, 7, 8
        for rank in 6..9 {
            for file in 0..9 {
                let bit = rank * 9 + file;
                assert_ne!(black_zone & (1u128 << bit), 0);
            }
        }

        let white_zone = promotion_zone_mask(Player::White);
        // Should have bits set for ranks 0, 1, 2
        for rank in 0..3 {
            for file in 0..9 {
                let bit = rank * 9 + file;
                assert_ne!(white_zone & (1u128 << bit), 0);
            }
        }
    }

    #[test]
    fn test_get_center_squares() {
        let center = get_center_squares();
        assert_eq!(center.len(), 5);
        assert!(center.contains(&40)); // 5e
        assert!(center.contains(&39)); // 4e
        assert!(center.contains(&41)); // 6e
        assert!(center.contains(&31)); // 5d
        assert!(center.contains(&49)); // 5f
    }

    #[test]
    fn test_is_center_square() {
        assert!(is_center_square(40)); // 5e (center)
        assert!(is_center_square(39)); // 4e
        assert!(is_center_square(31)); // 5d
        assert!(!is_center_square(0)); // 9i (corner)
        assert!(!is_center_square(80)); // 1a (corner)
    }

    #[test]
    fn test_round_trip_conversions() {
        // Test bit -> square -> bit
        for bit in 0..81 {
            let square = bit_to_square(bit);
            let converted_bit = square_to_bit(square);
            assert_eq!(bit, converted_bit);
        }

        // Test bit -> coords -> bit
        for bit in 0..81 {
            let (file, rank) = bit_to_coords(bit);
            let converted_bit = coords_to_bit(file, rank);
            assert_eq!(bit, converted_bit);
        }

        // Test bit -> square_name -> bit
        for bit in 0..81 {
            let name = bit_to_square_name(bit);
            let converted_bit = square_name_to_bit(&name);
            assert_eq!(bit, converted_bit);
        }
    }

    #[test]
    #[should_panic]
    fn test_bit_to_square_panic() {
        bit_to_square(128);
    }

    #[test]
    #[should_panic]
    fn test_bit_to_coords_panic() {
        bit_to_coords(128);
    }

    #[test]
    #[should_panic]
    fn test_coords_to_bit_panic() {
        coords_to_bit(9, 0);
    }

    #[test]
    #[should_panic]
    fn test_coords_to_bit_panic2() {
        coords_to_bit(0, 9);
    }

    #[test]
    #[should_panic]
    fn test_square_name_to_bit_invalid() {
        square_name_to_bit("x1");
    }

    #[test]
    #[should_panic]
    fn test_square_name_to_bit_invalid2() {
        square_name_to_bit("10");
    }
}
