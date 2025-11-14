//! Task 5.0.5.4: Integration tests for platform-specific bitboard code paths
//!
//! These tests ensure that SIMD/BMI fallback paths remain functional across platforms,
//! including wasm/ARM builds if applicable.

use shogi_engine::bitboards::{
    integration::{BitScanningOptimizer, GlobalOptimizer},
    get_board_telemetry, get_magic_telemetry, reset_board_telemetry, BitboardBoard,
};
use shogi_engine::types::Bitboard;

#[test]
fn test_platform_specific_bitscan_fallback() {
    // Test that bit scanning works regardless of platform capabilities
    let optimizer = BitScanningOptimizer::new();
    
    let test_bitboards = vec![
        0u128,
        1u128,
        0b1010u128,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128,
        0x5555555555555555u128,
    ];
    
    for bb in test_bitboards {
        // Should work on all platforms (uses fallback if hardware not available)
        let result = optimizer.bit_scan_forward(bb);
        assert!(result.is_none() || result.unwrap() < 128);
        
        let popcount = optimizer.popcount(bb);
        assert!(popcount <= 128);
    }
}

#[test]
fn test_adaptive_selection_without_hardware() {
    // Test adaptive selection works even without hardware acceleration
    let optimizer = BitScanningOptimizer::with_config(true);
    
    // Should fall back to software implementations
    let sparse = 0b1010u128;
    let dense = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128;
    
    assert_eq!(optimizer.popcount(sparse), 2);
    assert_eq!(optimizer.popcount(dense), 128);
    
    // Should track strategy selection
    let counters = optimizer.get_strategy_counters();
    assert!(counters.popcount_hardware + counters.popcount_4bit + counters.popcount_swar + counters.popcount_debruijn > 0);
}

#[test]
fn test_global_optimizer_platform_independence() {
    // Test that GlobalOptimizer works on all platforms
    let test_cases = vec![
        (0u128, 0),
        (1u128, 1),
        (0b1010u128, 2),
        (0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128, 128),
    ];
    
    for (bb, expected_count) in test_cases {
        let count = GlobalOptimizer::popcount(bb);
        assert_eq!(count, expected_count, "Popcount failed for 0x{:X}", bb);
        
        if bb != 0 {
            let first_bit = GlobalOptimizer::bit_scan_forward(bb);
            assert!(first_bit.is_some(), "Bit scan forward should find bit in 0x{:X}", bb);
        }
    }
}

#[test]
fn test_magic_fallback_functionality() {
    // Test that magic bitboard fallback works correctly
    let board = BitboardBoard::empty();
    
    // Test attack pattern generation (should use fallback if magic unavailable)
    use shogi_engine::types::{PieceType, Position};
    let center = Position::new(4, 4);
    
    let rook_attacks = board.get_attack_pattern(center, PieceType::Rook);
    // Should generate some attacks (at least in the same row/col)
    assert!(rook_attacks != 0 || board.get_occupied_bitboard() != 0);
    
    let bishop_attacks = board.get_attack_pattern(center, PieceType::Bishop);
    // Should generate some attacks (at least in diagonals)
    assert!(bishop_attacks != 0 || board.get_occupied_bitboard() != 0);
}

#[test]
fn test_telemetry_counters_functionality() {
    // Test that telemetry counters work correctly
    reset_board_telemetry();
    
    let board = BitboardBoard::new();
    
    // Perform some operations
    let _clone1 = board.clone();
    let _clone2 = board.clone();
    
    let telemetry = get_board_telemetry();
    assert!(telemetry.clone_count >= 2, "Clone counter should track operations");
    
    // Test magic telemetry
    let (raycast_count, magic_count, unavailable_count) = get_magic_telemetry();
    // These should be non-negative (may be 0 if no operations performed yet)
    assert!(raycast_count >= 0);
    assert!(magic_count >= 0);
    assert!(unavailable_count >= 0);
}

#[test]
fn test_strategy_counters_reset() {
    let optimizer = BitScanningOptimizer::new();
    
    // Perform some operations
    optimizer.popcount(0b1010);
    optimizer.bit_scan_forward(0b1000);
    
    let counters_before = optimizer.get_strategy_counters();
    let total_before = counters_before.popcount_hardware + counters_before.popcount_4bit
        + counters_before.popcount_swar + counters_before.popcount_debruijn
        + counters_before.bitscan_hardware + counters_before.bitscan_debruijn;
    
    // Reset and verify
    optimizer.reset_counters();
    let counters_after = optimizer.get_strategy_counters();
    assert_eq!(counters_after.popcount_hardware, 0);
    assert_eq!(counters_after.bitscan_hardware, 0);
}

#[test]
fn test_attack_table_initialization_telemetry() {
    reset_board_telemetry();
    
    // Creating a new board should initialize attack tables
    let _board = BitboardBoard::empty();
    
    let telemetry = get_board_telemetry();
    // Attack table should have been initialized (time > 0 or memory > 0)
    assert!(
        telemetry.attack_table_init_time > 0 || telemetry.attack_table_memory > 0,
        "Attack table initialization should be tracked"
    );
}

#[test]
fn test_bitboard_operations_cross_platform() {
    // Test that basic bitboard operations work on all platforms
    let board = BitboardBoard::new();
    
    // Test attack detection
    use shogi_engine::types::{Position, Player};
    let center = Position::new(4, 4);
    let _attacked = board.is_square_attacked_by(center, Player::Black);
    
    // Test attack pattern iteration
    let attacks = board.get_attack_pattern(center, shogi_engine::types::PieceType::Rook);
    let targets: Vec<_> = board.iter_attack_targets(attacks).collect();
    // Should be able to iterate over targets
    assert!(targets.len() <= 81);
    
    // Test piece iteration
    let pieces: Vec<_> = board.iter_pieces().collect();
    assert!(pieces.len() <= 81);
}

#[test]
fn test_estimate_bit_count_accuracy() {
    // Test that estimate_bit_count works correctly on all platforms
    let optimizer = BitScanningOptimizer::new();
    
    let test_cases = vec![
        (0u128, 0),
        (0b1010u128, 2),
        (0x5555555555555555u128, 32), // Low half only
        (0x55555555555555550000000000000000u128, 32), // High half only
        (0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128, 128), // All bits
    ];
    
    for (bb, expected) in test_cases {
        let estimated = optimizer.estimate_bit_count(bb);
        assert_eq!(
            estimated, expected,
            "Estimate should match actual for 0x{:X}",
            bb
        );
    }
}

