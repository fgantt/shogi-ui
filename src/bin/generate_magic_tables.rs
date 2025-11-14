//! Build-time tool to generate precomputed magic bitboard tables
//!
//! This tool generates magic tables and saves them to disk for fast loading at runtime.
//! Usage:
//!   cargo run --bin generate_magic_tables [--output <path>]
//!
//! If no output path is specified, defaults to `resources/magic_tables/magic_table.bin`

use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse output path from command line arguments
    let output_path = if let Some(output_idx) = args.iter().position(|a| a == "--output" || a == "-o") {
        if output_idx + 1 < args.len() {
            PathBuf::from(&args[output_idx + 1])
        } else {
            eprintln!("Error: --output requires a path argument");
            std::process::exit(1);
        }
    } else {
        // Default to resources/magic_tables/magic_table.bin
        let mut path = PathBuf::from("resources");
        path.push("magic_tables");
        path.push("magic_table.bin");
        path
    };

    println!("Generating magic bitboard tables...");
    println!("Output path: {}", output_path.display());
    
    let start_time = Instant::now();
    
    // Generate the magic table
    use shogi_engine::types::MagicTable;
    let table = MagicTable::new()
        .map_err(|e| format!("Failed to generate magic table: {}", e))?;
    
    let generation_time = start_time.elapsed();
    println!("Magic table generation completed in {:?}", generation_time);
    
    // Validate the generated table
    println!("Validating generated table...");
    table.validate()
        .map_err(|e| format!("Generated table validation failed: {}", e))?;
    println!("Table validation passed");
    
    // Save to file
    println!("Saving table to {}...", output_path.display());
    let save_start = Instant::now();
    table.save_to_file(&output_path)
        .map_err(|e| format!("Failed to save magic table: {}", e))?;
    let save_time = save_start.elapsed();
    
    // Get file size
    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    println!("Table saved successfully in {:?}", save_time);
    println!("File size: {} bytes ({:.2} MB)", file_size, file_size as f64 / 1_048_576.0);
    
    // Print statistics
    let stats = table.memory_stats();
    println!("\nTable Statistics:");
    println!("  Total attack patterns: {}", stats.total_attack_patterns);
    println!("  Memory usage: {} bytes ({:.2} MB)", 
             stats.memory_usage_bytes, 
             stats.memory_usage_bytes as f64 / 1_048_576.0);
    
    let perf_stats = table.performance_stats();
    println!("  Total rook entries: {}", perf_stats.total_rook_entries);
    println!("  Total bishop entries: {}", perf_stats.total_bishop_entries);
    println!("  Memory efficiency: {:.2}%", perf_stats.memory_efficiency * 100.0);
    
    println!("\n✅ Magic table generation complete!");
    println!("   You can now use this precomputed table for fast initialization.");
    
    Ok(())
}

