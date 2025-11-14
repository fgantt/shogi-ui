//! Offline magic number optimization tool
//!
//! This tool precomputes optimal magic numbers for all squares and piece types,
//! storing them in a resource file for use by the engine. This allows the engine
//! to use pre-optimized magic numbers instead of generating them at runtime.

use shogi_engine::bitboards::magic::magic_finder::MagicFinder;
use shogi_engine::types::{MagicGenerationResult, PieceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Optimized magic numbers database
#[derive(Debug, Serialize, Deserialize)]
struct OptimizedMagics {
    /// Magic numbers for each (square, piece_type) pair
    magics: HashMap<(u8, PieceType), MagicGenerationResult>,
    /// Generation timestamp
    generated_at: String,
    /// Total table size
    total_table_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting magic number optimization...");
    let start_time = Instant::now();

    let mut finder = MagicFinder::new();
    let mut magics = HashMap::new();
    let mut total_table_size = 0;

    // Optimize rook magics
    println!("Optimizing rook magic numbers...");
    for square in 0..81 {
        match finder.find_magic_number(square, PieceType::Rook) {
            Ok(result) => {
                magics.insert((square, PieceType::Rook), result);
                total_table_size += result.table_size;
                if square % 10 == 0 {
                    println!("  Square {}: table_size={}", square, result.table_size);
                }
            }
            Err(e) => {
                eprintln!("Failed to find magic for rook square {}: {:?}", square, e);
            }
        }
    }

    // Optimize bishop magics
    println!("Optimizing bishop magic numbers...");
    for square in 0..81 {
        match finder.find_magic_number(square, PieceType::Bishop) {
            Ok(result) => {
                magics.insert((square, PieceType::Bishop), result);
                total_table_size += result.table_size;
                if square % 10 == 0 {
                    println!("  Square {}: table_size={}", square, result.table_size);
                }
            }
            Err(e) => {
                eprintln!("Failed to find magic for bishop square {}: {:?}", square, e);
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("\nOptimization completed in {:?}", elapsed);
    println!("Total table size: {} entries", total_table_size);

    // Save to file
    let output_path = "resources/magic_tables/optimized_magics.json";
    std::fs::create_dir_all("resources/magic_tables")?;
    
    let optimized = OptimizedMagics {
        magics,
        generated_at: chrono::Utc::now().to_rfc3339(),
        total_table_size,
    };

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &optimized)?;
    writer.flush()?;

    println!("Saved optimized magic numbers to {}", output_path);

    Ok(())
}

