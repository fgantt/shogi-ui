//! Compressed Entry Storage Example
//!
//! This example demonstrates how to use the compressed entry storage system
//! to significantly reduce memory usage in transposition tables while
//! maintaining fast access times.

use shogi_engine::search::{CompressedEntryStorage, CompressionAlgorithm, CompressionConfig};
use shogi_engine::types::{
    EntrySource, Move, PieceType, Player, Position, TranspositionEntry, TranspositionFlag,
};

fn build_entry(
    score: i32,
    depth: u8,
    flag: TranspositionFlag,
    best_move: Option<Move>,
    hash_key: u64,
    age: u32,
) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        flag,
        best_move,
        hash_key,
        age,
        EntrySource::MainSearch,
    )
}

fn rle_config() -> CompressionConfig {
    CompressionConfig {
        algorithm: CompressionAlgorithm::Rle,
        level: 1,
        adaptive: false,
        min_ratio: 0.8,
        cache_size: 500,
        use_dictionary: false,
    }
}

fn main() {
    println!("Compressed Entry Storage Example");
    println!("===============================");

    // Demonstrate different compression algorithms
    demonstrate_compression_algorithms();

    // Demonstrate compression benefits
    demonstrate_compression_benefits();

    // Demonstrate caching functionality
    demonstrate_caching();

    // Demonstrate adaptive compression
    demonstrate_adaptive_compression();

    // Demonstrate dictionary compression
    demonstrate_dictionary_compression();

    println!("\nCompressed Entry Storage Example completed successfully!");
}

fn demonstrate_compression_algorithms() {
    println!("\n--- Compression Algorithms Demo ---");

    let algorithms = [
        ("LZ4 Fast", CompressionConfig::lz4_fast()),
        ("LZ4 High", CompressionConfig::lz4_high()),
        ("Huffman", CompressionConfig::huffman()),
        ("Bit Packing", CompressionConfig::bit_packing()),
        ("Run-Length Encoding", rle_config()),
    ];

    let entry = build_entry(
        150,
        8,
        TranspositionFlag::Exact,
        None,
        0x123456789ABCDEF0,
        25,
    );

    for (name, config) in algorithms {
        let mut storage = CompressedEntryStorage::new(config);

        let start_time = std::time::Instant::now();
        let compressed = storage.compress_entry(&entry);
        let compression_time = start_time.elapsed().as_micros();

        let start_time = std::time::Instant::now();
        let decompressed = storage.decompress_entry(&compressed);
        let decompression_time = start_time.elapsed().as_micros();

        let stats = storage.get_stats();
        let compression_ratio = compressed.metadata.ratio;
        let savings = (1.0 - compression_ratio) * 100.0;

        println!(
            "{}: ratio={:.2}, savings={:.1}%, compress={}μs, decompress={}μs",
            name, compression_ratio, savings, compression_time, decompression_time
        );

        // Verify correctness
        assert_eq!(decompressed.hash_key, entry.hash_key);
        assert_eq!(decompressed.depth, entry.depth);
        assert_eq!(decompressed.score, entry.score);
        assert_eq!(decompressed.flag, entry.flag);
        assert_eq!(decompressed.age, entry.age);
    }
}

fn demonstrate_compression_benefits() {
    println!("\n--- Compression Benefits Demo ---");

    let mut storage = CompressedEntryStorage::new(CompressionConfig::lz4_fast());

    // Create multiple entries with different characteristics
    let entries = vec![
        // Entry with no best move
        build_entry(
            50,
            3,
            TranspositionFlag::Exact,
            None,
            0x1111111111111111,
            10,
        ),
        // Entry with best move
        build_entry(
            -75,
            6,
            TranspositionFlag::LowerBound,
            Some(create_sample_move()),
            0x2222222222222222,
            15,
        ),
        // Deep entry
        build_entry(
            200,
            12,
            TranspositionFlag::UpperBound,
            None,
            0x3333333333333333,
            20,
        ),
    ];

    let mut total_original_size = 0;
    let mut total_compressed_size = 0;

    for entry in &entries {
        let compressed = storage.compress_entry(entry);
        total_original_size += compressed.original_size;
        total_compressed_size += compressed.data.len();

        println!(
            "Entry depth {}: original={} bytes, compressed={} bytes, ratio={:.2}",
            entry.depth,
            compressed.original_size,
            compressed.data.len(),
            compressed.metadata.ratio
        );
    }

    let overall_ratio = total_compressed_size as f64 / total_original_size as f64;
    let overall_savings = (1.0 - overall_ratio) * 100.0;

    println!(
        "Overall: {} original bytes -> {} compressed bytes",
        total_original_size, total_compressed_size
    );
    println!(
        "Overall compression ratio: {:.2} ({:.1}% savings)",
        overall_ratio, overall_savings
    );

    let stats = storage.get_stats();
    println!(
        "Statistics: {} compressed, {} decompressed, {:.2} avg ratio",
        stats.total_compressed, stats.total_decompressed, stats.avg_compression_ratio
    );
}

fn demonstrate_caching() {
    println!("\n--- Caching Demo ---");

    let mut storage = CompressedEntryStorage::new(CompressionConfig::lz4_fast());

    let entry = build_entry(
        100,
        5,
        TranspositionFlag::Exact,
        None,
        0x4444444444444444,
        12,
    );

    let compressed = storage.compress_entry(&entry);

    // First decompression (should be cache miss)
    let start_time = std::time::Instant::now();
    let _decompressed1 = storage.decompress_entry(&compressed);
    let first_time = start_time.elapsed().as_micros();

    println!("First decompression: {}μs (cache miss)", first_time);

    // Second decompression (should be cache hit)
    let start_time = std::time::Instant::now();
    let _decompressed2 = storage.decompress_entry(&compressed);
    let second_time = start_time.elapsed().as_micros();

    println!("Second decompression: {}μs (cache hit)", second_time);

    // Estimate improvement based on timing
    if second_time > 0 {
        println!(
            "Cache hit was {:.2}x faster",
            first_time as f64 / second_time as f64
        );
    }
}

fn demonstrate_adaptive_compression() {
    println!("\n--- Adaptive Compression Demo ---");

    let mut storage = CompressedEntryStorage::new(CompressionConfig::adaptive());

    // Create entries with different data characteristics
    let entries = vec![
        // Low entropy entry (repeated patterns)
        build_entry(0, 1, TranspositionFlag::Exact, None, 0xAAAAAAAAAAAAAAAA, 1),
        // High entropy entry (random-like)
        build_entry(
            150,
            8,
            TranspositionFlag::Exact,
            Some(create_sample_move()),
            0x123456789ABCDEF0,
            25,
        ),
        // Medium entropy entry
        build_entry(
            75,
            4,
            TranspositionFlag::LowerBound,
            None,
            0x5555555555555555,
            10,
        ),
    ];

    for (i, entry) in entries.iter().enumerate() {
        let compressed = storage.compress_entry(entry);

        println!(
            "Entry {}: algorithm={:?}, ratio={:.2}, beneficial={}",
            i + 1,
            compressed.algorithm,
            compressed.metadata.ratio,
            compressed.metadata.beneficial
        );

        // Verify adaptive selection worked
        if compressed.metadata.beneficial {
            assert!(compressed.metadata.ratio < 1.0);
        }
    }
}

fn demonstrate_dictionary_compression() {
    println!("\n--- Dictionary Compression Demo ---");

    let mut storage = CompressedEntryStorage::new(CompressionConfig::huffman());

    // Create entries with repeated patterns
    let entries = vec![
        build_entry(
            50,
            3,
            TranspositionFlag::Exact,
            None,
            0x1111111111111111,
            10,
        ),
        build_entry(
            60,
            4,
            TranspositionFlag::Exact,
            None,
            0x1111111111111112,
            11,
        ),
        build_entry(
            70,
            5,
            TranspositionFlag::Exact,
            None,
            0x1111111111111113,
            12,
        ),
    ];

    // Update dictionary with common patterns
    let patterns = vec![
        vec![0x11, 0x11, 0x11, 0x11],         // Common hash pattern
        vec![TranspositionFlag::Exact as u8], // Common flag
    ];
    storage.update_dictionary(&patterns);

    println!("Updated dictionary with {} patterns", patterns.len());

    let mut total_original = 0;
    let mut total_compressed = 0;

    for entry in &entries {
        let compressed = storage.compress_entry(entry);
        total_original += compressed.original_size;
        total_compressed += compressed.data.len();

        println!(
            "Entry: ratio={:.2}, beneficial={}",
            compressed.metadata.ratio, compressed.metadata.beneficial
        );
    }

    let overall_ratio = total_compressed as f64 / total_original as f64;
    println!("Dictionary compression overall ratio: {:.2}", overall_ratio);
}

fn create_sample_move() -> Move {
    Move {
        from: Some(Position::from_u8(15)),
        to: Position::from_u8(25),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: true,
        is_capture: false,
        captured_piece: None,
        gives_check: true,
        is_recapture: false,
    }
}
