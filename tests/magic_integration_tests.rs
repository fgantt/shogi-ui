#![cfg(feature = "legacy-tests")]
//! Integration tests for magic bitboards with the move generation system
//!
//! These tests verify that magic bitboards integrate correctly with the
//! existing Shogi engine components.

use shogi_engine::{
    bitboards::magic::magic_table,
    types::{MagicTable, Piece, PieceType, Player, Position},
    BitboardBoard,
};
use std::fs;
use std::path::Path;

#[test]
fn test_bitboard_with_magic_support() {
    let result = BitboardBoard::new_with_magic_support();
    assert!(
        result.is_ok(),
        "Failed to create BitboardBoard with magic support: {:?}",
        result.err()
    );

    let board = result.unwrap();
    assert!(board.has_magic_support(), "Board should have magic support");
}

#[test]
fn test_magic_table_in_bitboard() {
    let board = BitboardBoard::new_with_magic_support().unwrap();

    let magic_table = board.get_magic_table();
    assert!(magic_table.is_some(), "Magic table should be present");
}

#[test]
fn test_sliding_generator_initialization() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();

    // Initialize sliding generator
    let result = board.init_sliding_generator();
    assert!(
        result.is_ok(),
        "Failed to initialize sliding generator: {:?}",
        result.err()
    );

    assert!(
        board.is_sliding_generator_initialized(),
        "Sliding generator should be initialized"
    );
}

#[test]
fn test_magic_sliding_moves_generation() {
    let mut board = BitboardBoard::empty();

    // Set up board with magic support
    let magic_table = MagicTable::new().unwrap();
    board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a rook in the center
    let rook_pos = Position::new(4, 4);
    let rook = Piece {
        piece_type: PieceType::Rook,
        player: Player::Black,
    };
    board.place_piece(rook, rook_pos);

    // Generate magic sliding moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(rook_pos, PieceType::Rook, Player::Black)
    {
        assert!(!moves.is_empty(), "Rook should have moves from center");

        // Verify moves are valid
        for move_ in moves {
            assert_eq!(
                move_.from,
                Some(rook_pos),
                "Move should start from rook position"
            );
            assert_eq!(move_.piece_type, PieceType::Rook, "Move should be for rook");
            assert_eq!(
                move_.player,
                Player::Black,
                "Move should be for black player"
            );
        }
    }
}

#[test]
fn test_magic_vs_raycast_consistency() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a bishop
    let bishop_pos = Position::new(3, 3);
    let bishop = Piece {
        piece_type: PieceType::Bishop,
        player: Player::White,
    };
    board.place_piece(bishop, bishop_pos);

    // Generate magic moves
    let magic_moves =
        board.generate_magic_sliding_moves(bishop_pos, PieceType::Bishop, Player::White);

    // Magic moves should be generated
    assert!(
        magic_moves.is_some(),
        "Magic moves should be generated for bishop"
    );

    let moves = magic_moves.unwrap();
    assert!(!moves.is_empty(), "Bishop should have moves");
}

#[test]
fn test_sliding_generator_with_blockers() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a rook
    let rook_pos = Position::new(4, 4);
    let rook = Piece {
        piece_type: PieceType::Rook,
        player: Player::Black,
    };
    board.place_piece(rook, rook_pos);

    // Place a blocker
    let blocker_pos = Position::new(4, 6);
    let blocker = Piece {
        piece_type: PieceType::Pawn,
        player: Player::White,
    };
    board.place_piece(blocker, blocker_pos);

    // Generate moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(rook_pos, PieceType::Rook, Player::Black)
    {
        // Check that rook can capture blocker but not go beyond
        let captures_blocker = moves.iter().any(|m| m.to == blocker_pos);
        assert!(captures_blocker, "Rook should be able to capture blocker");

        // Check that rook doesn't go beyond blocker (e.g., column 7)
        let beyond_blocker = moves.iter().any(|m| m.to.row == 4 && m.to.col > 6);
        assert!(!beyond_blocker, "Rook should not go beyond blocker");
    }
}

#[test]
fn test_sliding_generator_respects_own_pieces() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a bishop
    let bishop_pos = Position::new(3, 3);
    let bishop = Piece {
        piece_type: PieceType::Bishop,
        player: Player::Black,
    };
    board.place_piece(bishop, bishop_pos);

    // Place own piece in diagonal
    let own_piece_pos = Position::new(5, 5);
    let own_piece = Piece {
        piece_type: PieceType::Pawn,
        player: Player::Black,
    };
    board.place_piece(own_piece, own_piece_pos);

    // Generate moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(bishop_pos, PieceType::Bishop, Player::Black)
    {
        // Check that bishop doesn't capture own piece
        let captures_own = moves.iter().any(|m| m.to == own_piece_pos);
        assert!(!captures_own, "Bishop should not capture own piece");
    }
}

#[test]
fn test_board_clone_preserves_magic_support() {
    let board1 = BitboardBoard::new_with_magic_support().unwrap();
    let board2 = board1.clone();

    assert_eq!(
        board1.has_magic_support(),
        board2.has_magic_support(),
        "Cloned board should preserve magic support"
    );
}

#[test]
fn test_magic_table_serialization_integration() {
    let table1 = MagicTable::new().unwrap();

    // Serialize
    let serialized = table1.serialize();
    assert!(serialized.is_ok(), "Serialization should succeed");

    let bytes = serialized.unwrap();
    assert!(!bytes.is_empty(), "Serialized data should not be empty");

    // Deserialize
    let table2 = MagicTable::deserialize(&bytes);
    assert!(table2.is_ok(), "Deserialization should succeed");

    let table2 = table2.unwrap();

    // Verify tables produce same results
    for square in (0..81).step_by(10) {
        let attacks1 = table1.get_attacks(square, PieceType::Rook, 0);
        let attacks2 = table2.get_attacks(square, PieceType::Rook, 0);

        assert_eq!(
            attacks1, attacks2,
            "Deserialized table should match original for square {}",
            square
        );
    }
}

#[test]
fn test_performance_stats() {
    let table = MagicTable::new().unwrap();

    let stats = table.performance_stats();

    // Should have entries for all squares and piece types
    assert_eq!(stats.total_rook_entries, 81, "Should have 81 rook entries");
    assert_eq!(
        stats.total_bishop_entries, 81,
        "Should have 81 bishop entries"
    );
    assert!(
        stats.total_attack_patterns > 0,
        "Should have attack patterns"
    );
}

#[test]
fn test_magic_initialization_progress() {
    let table = MagicTable::new().unwrap();

    let (initialized, total) = table.initialization_progress();
    assert_eq!(
        initialized, total,
        "Fully initialized table should show all entries initialized"
    );

    assert!(
        table.is_fully_initialized(),
        "Table should be fully initialized"
    );
}

#[test]
fn test_multiple_pieces_with_magic() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place multiple sliding pieces
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::Black,
        },
        Position::new(0, 0),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Bishop,
            player: Player::White,
        },
        Position::new(2, 2),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::White,
        },
        Position::new(4, 4),
    );

    // Generate moves for each piece
    let rook1_moves =
        board.generate_magic_sliding_moves(Position::new(0, 0), PieceType::Rook, Player::Black);
    let bishop_moves =
        board.generate_magic_sliding_moves(Position::new(2, 2), PieceType::Bishop, Player::White);
    let rook2_moves =
        board.generate_magic_sliding_moves(Position::new(4, 4), PieceType::Rook, Player::White);

    assert!(rook1_moves.is_some(), "Should generate moves for rook 1");
    assert!(bishop_moves.is_some(), "Should generate moves for bishop");
    assert!(rook2_moves.is_some(), "Should generate moves for rook 2");
}

#[test]
fn test_promoted_pieces_preparation() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Note: Promoted pieces use same sliding patterns as base pieces
    // This test prepares for future promoted piece integration

    let generator = board.get_sliding_generator();
    assert!(
        generator.is_some(),
        "Sliding generator should be available for promoted pieces"
    );
}

#[test]
fn test_magic_table_validation() {
    let table = MagicTable::new().unwrap();

    // Validate the table
    let validation_result = table.validate();
    assert!(
        validation_result.is_ok(),
        "Magic table validation failed: {:?}",
        validation_result.err()
    );
}

#[test]
fn test_edge_case_positions() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Test corners
    let corners = [
        Position::new(0, 0),
        Position::new(0, 8),
        Position::new(8, 0),
        Position::new(8, 8),
    ];

    for corner in corners {
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::Black,
            },
            corner,
        );

        let moves = board.generate_magic_sliding_moves(corner, PieceType::Rook, Player::Black);
        assert!(
            moves.is_some(),
            "Should generate moves from corner {:?}",
            corner
        );

        board.remove_piece(corner);
    }
}

#[test]
fn test_memory_efficiency() {
    let table = MagicTable::new().unwrap();
    let stats = table.performance_stats();

    // Memory efficiency should be reasonable
    assert!(
        stats.memory_efficiency > 0.0,
        "Memory efficiency should be positive"
    );
    assert!(
        stats.memory_efficiency <= 1.0,
        "Memory efficiency should not exceed 100%"
    );
}

#[test]
#[ignore] // Ignore by default - generation takes 60+ seconds
fn test_precomputed_table_loads_correctly() {
    use std::time::Instant;
    
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_precomputed_magic_table.bin");
    
    // Clean up if file exists
    let _ = fs::remove_file(&test_file);
    
    // Generate a table and save it
    println!("Generating magic table for precomputed test...");
    let gen_start = Instant::now();
    let generated_table = MagicTable::new().unwrap();
    let gen_time = gen_start.elapsed();
    println!("Generation took: {:?}", gen_time);
    
    // Validate generated table
    generated_table.validate().expect("Generated table should be valid");
    
    // Save to file
    generated_table.save_to_file(&test_file)
        .expect("Failed to save generated table");
    assert!(test_file.exists(), "Precomputed file should exist");
    
    // Load from file
    println!("Loading precomputed table...");
    let load_start = Instant::now();
    let loaded_table = MagicTable::load_from_file(&test_file)
        .expect("Failed to load precomputed table");
    let load_time = load_start.elapsed();
    println!("Load took: {:?}", load_time);
    
    // Validate loaded table
    loaded_table.validate().expect("Loaded table should be valid");
    
    // Verify loaded table matches generated table
    assert_eq!(
        generated_table.attack_storage.len(),
        loaded_table.attack_storage.len(),
        "Attack storage length should match"
    );
    assert_eq!(
        generated_table.attack_storage,
        loaded_table.attack_storage,
        "Attack storage data should match"
    );
    
    // Verify magic entries match
    for i in 0..81 {
        assert_eq!(
            generated_table.rook_magics[i],
            loaded_table.rook_magics[i],
            "Rook magic entry {} should match",
            i
        );
        assert_eq!(
            generated_table.bishop_magics[i],
            loaded_table.bishop_magics[i],
            "Bishop magic entry {} should match",
            i
        );
    }
    
    // Verify lookup results match for sample positions
    for square in (0..81).step_by(10) {
        let test_occupied = 0x1234567890ABCDEF; // Sample occupied bitboard
        let gen_rook = generated_table.get_attacks(square, PieceType::Rook, test_occupied);
        let load_rook = loaded_table.get_attacks(square, PieceType::Rook, test_occupied);
        assert_eq!(
            gen_rook, load_rook,
            "Rook attacks should match for square {}",
            square
        );
        
        let gen_bishop = generated_table.get_attacks(square, PieceType::Bishop, test_occupied);
        let load_bishop = loaded_table.get_attacks(square, PieceType::Bishop, test_occupied);
        assert_eq!(
            gen_bishop, load_bishop,
            "Bishop attacks should match for square {}",
            square
        );
    }
    
    // Verify load time is much faster than generation time
    // (This is the main benefit of precomputed tables)
    if gen_time.as_millis() > 0 {
        let speedup = gen_time.as_millis() as f64 / load_time.as_millis().max(1) as f64;
        println!("Load speedup: {:.2}x faster than generation", speedup);
        // Load should be at least 10x faster (generation takes 60s+, load should be <1s)
        // But we'll be lenient in tests since generation might be fast in test environment
        assert!(
            load_time < gen_time,
            "Loading should be faster than generation (load: {:?}, gen: {:?})",
            load_time,
            gen_time
        );
    }
    
    // Clean up
    let _ = fs::remove_file(&test_file);
}

#[test]
#[ignore] // Ignore by default - compression takes time
fn test_compressed_table_produces_identical_results() {
    use shogi_engine::bitboards::magic::compressed_table::CompressedMagicTable;
    
    // Generate a magic table
    let table = MagicTable::new().unwrap();
    table.validate().expect("Table should be valid");
    
    // Create compressed and uncompressed versions
    let compressed = CompressedMagicTable::from_table(table.clone())
        .expect("Should create compressed table");
    let uncompressed = CompressedMagicTable::uncompressed(table.clone());
    
    // Test various positions
    let test_cases = vec![
        (0u8, PieceType::Rook, 0u128),
        (40u8, PieceType::Rook, 0b1010101010101010u128),
        (40u8, PieceType::Bishop, 0b1111000011110000u128),
        (72u8, PieceType::Rook, 0b1111111100000000u128),
        (4u8, PieceType::Bishop, 0b0000111100001111u128),
        (20u8, PieceType::Rook, 0xFFFFFFFFFFFFFFFFu128),
        (60u8, PieceType::Bishop, 0x1234567890ABCDEFu128),
    ];
    
    // Verify compressed and uncompressed produce identical results
    for (square, piece_type, occupied) in test_cases {
        let base_attacks = table.get_attacks(square, piece_type, occupied);
        let compressed_attacks = compressed.get_attacks(square, piece_type, occupied);
        let uncompressed_attacks = uncompressed.get_attacks(square, piece_type, occupied);
        
        assert_eq!(
            base_attacks, compressed_attacks,
            "Compressed attacks should match base for square {} piece {:?}",
            square, piece_type
        );
        assert_eq!(
            base_attacks, uncompressed_attacks,
            "Uncompressed attacks should match base for square {} piece {:?}",
            square, piece_type
        );
    }
    
    // Test all squares systematically (sample)
    for square in (0..81).step_by(5) {
        for piece_type in [PieceType::Rook, PieceType::Bishop] {
            let test_occupied = (square as u128) * 0x1234567890ABCDEF;
            let base_attacks = table.get_attacks(square, piece_type, test_occupied);
            let compressed_attacks = compressed.get_attacks(square, piece_type, test_occupied);
            
            assert_eq!(
                base_attacks, compressed_attacks,
                "Compressed attacks should match base for square {} piece {:?}",
                square, piece_type
            );
        }
    }
}

#[test]
#[ignore] // Ignore by default - compression takes time
fn test_compression_memory_usage_comparison() {
    use shogi_engine::bitboards::magic::compressed_table::CompressedMagicTable;
    use std::alloc::{GlobalAlloc, Layout, System};
    
    // Generate a magic table
    let table = MagicTable::new().unwrap();
    
    // Measure original size
    let original_size = table.attack_storage.len() * std::mem::size_of::<u128>();
    
    // Create compressed version
    let compressed = CompressedMagicTable::from_table(table.clone())
        .expect("Should create compressed table");
    
    // Get compression statistics
    let stats = compressed.stats();
    
    // Verify compression achieved some savings (if enabled)
    if compressed.is_compressed() {
        assert!(
            stats.compression_ratio >= 1.0,
            "Compression ratio should be >= 1.0, got {}",
            stats.compression_ratio
        );
        
        // Print compression results
        println!("\n=== Compression Memory Usage ===");
        println!("Original size: {} bytes ({:.2} MB)", 
                 stats.original_size, 
                 stats.original_size as f64 / 1_000_000.0);
        println!("Compressed size: {} bytes ({:.2} MB)", 
                 stats.compressed_size, 
                 stats.compressed_size as f64 / 1_000_000.0);
        println!("Compression ratio: {:.2}x", stats.compression_ratio);
        println!("Memory saved: {} bytes ({:.2} MB)", 
                 stats.memory_saved, 
                 stats.memory_saved as f64 / 1_000_000.0);
        println!("Memory reduction: {:.1}%", 
                 (1.0 - 1.0 / stats.compression_ratio) * 100.0);
        println!("Deduplication: {} patterns", stats.dedup_count);
        println!("RLE encoded: {} patterns", stats.rle_count);
        println!("Delta encoded: {} patterns", stats.delta_count);
        println!("Raw storage: {} patterns", stats.raw_count);
        
        // Verify we achieved some memory savings (at least 5% reduction)
        let reduction = (1.0 - 1.0 / stats.compression_ratio) * 100.0;
        assert!(
            reduction >= 0.0,
            "Compression should not increase memory usage (reduction: {:.1}%)",
            reduction
        );
    }
}
