//! Compressed Entry Storage Example
//!
//! This example demonstrates how to use the compressed entry storage system
//! to significantly reduce memory usage in transposition tables while
//! maintaining fast access times.

use shogi_engine::search::{
    CompressedEntryStorage, CompressionAlgorithm, CompressionConfig, TranspositionEntry,
    TranspositionFlag,
};

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
        ("Run-Length Encoding", CompressionConfig::rle()),
    ];

    let entry = TranspositionEntry {
        hash_key: 0x123456789ABCDEF0,
        depth: 8,
        score: 150,
        flag: TranspositionFlag::Exact,
        best_move: None,
        age: 25,
    };

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
        TranspositionEntry {
            hash_key: 0x1111111111111111,
            depth: 3,
            score: 50,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
        },
        // Entry with best move
        TranspositionEntry {
            hash_key: 0x2222222222222222,
            depth: 6,
            score: -75,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(create_sample_move()),
            age: 15,
        },
        // Deep entry
        TranspositionEntry {
            hash_key: 0x3333333333333333,
            depth: 12,
            score: 200,
            flag: TranspositionFlag::UpperBound,
            best_move: None,
            age: 20,
        },
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

    let entry = TranspositionEntry {
        hash_key: 0x4444444444444444,
        depth: 5,
        score: 100,
        flag: TranspositionFlag::Exact,
        best_move: None,
        age: 12,
    };

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

    println!(
        "Cache hits: {}, cache misses: {}",
        storage.cache_hits, storage.cache_misses
    );

    // Verify cache hit was faster
    if storage.cache_hits > 0 {
        println!(
            "Cache hit was {}x faster",
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
        TranspositionEntry {
            hash_key: 0xAAAAAAAAAAAAAAAA,
            depth: 1,
            score: 0,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 1,
        },
        // High entropy entry (random-like)
        TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 8,
            score: 150,
            flag: TranspositionFlag::Exact,
            best_move: Some(create_sample_move()),
            age: 25,
        },
        // Medium entropy entry
        TranspositionEntry {
            hash_key: 0x5555555555555555,
            depth: 4,
            score: 75,
            flag: TranspositionFlag::LowerBound,
            best_move: None,
            age: 10,
        },
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
        TranspositionEntry {
            hash_key: 0x1111111111111111,
            depth: 3,
            score: 50,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
        },
        TranspositionEntry {
            hash_key: 0x1111111111111112,
            depth: 4,
            score: 60,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 11,
        },
        TranspositionEntry {
            hash_key: 0x1111111111111113,
            depth: 5,
            score: 70,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 12,
        },
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

// Helper function to create RLE configuration
impl CompressionConfig {
    fn rle() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Rle,
            level: 1,
            adaptive: false,
            min_ratio: 0.8,
            cache_size: 500,
            use_dictionary: false,
        }
    }
}
