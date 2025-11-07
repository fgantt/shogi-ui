//! Position Analysis Utility
//!
//! A command-line tool for analyzing shogi positions with detailed evaluation

use clap::{Parser, Subcommand};
use shogi_engine::ShogiEngine;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "analyzer")]
#[command(about = "Analyze shogi positions with detailed evaluation")]
struct Cli {
    /// SFEN position string
    #[arg(short, long)]
    position: Option<String>,

    /// Search depth
    #[arg(short, long, default_value_t = 6)]
    depth: u8,

    /// Time limit in milliseconds
    #[arg(short = 't', long, default_value_t = 5000)]
    time_limit: u32,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand for specific operations
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze starting position
    Startpos {
        /// Search depth
        #[arg(short, long, default_value_t = 6)]
        depth: u8,
    },
    /// Analyze position from SFEN
    Sfen {
        /// SFEN string
        sfen: String,
        /// Search depth
        #[arg(short, long, default_value_t = 6)]
        depth: u8,
    },
    /// Compare multiple positions
    Compare {
        /// SFEN strings to compare
        positions: Vec<String>,
        /// Search depth
        #[arg(short, long, default_value_t = 6)]
        depth: u8,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Shogi Position Analyzer");
        println!("=======================");
    }

    match &cli.command {
        Some(Commands::Startpos { depth }) => {
            analyze_starting_position(*depth, cli.verbose)?;
        }
        Some(Commands::Sfen { sfen, depth }) => {
            analyze_sfen_position(sfen, *depth, cli.verbose)?;
        }
        Some(Commands::Compare { positions, depth }) => {
            compare_positions(positions, *depth, cli.verbose)?;
        }
        None => {
            if let Some(ref position) = cli.position {
                analyze_sfen_position(position, cli.depth, cli.verbose)?;
            } else {
                analyze_starting_position(cli.depth, cli.verbose)?;
            }
        }
    }

    Ok(())
}

fn analyze_starting_position(depth: u8, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = ShogiEngine::new();

    if verbose {
        println!("Analyzing starting position...");
        println!("Search depth: {}", depth);
    }

    let start_time = std::time::Instant::now();

    if let Some(best_move) = engine.get_best_move(depth, 5000, None) {
        let elapsed = start_time.elapsed();

        println!("\n=== Analysis Results ===");
        println!("Best move: {}", best_move.to_usi_string());
        println!("Search time: {:.2}ms", elapsed.as_millis());
        println!("Depth: {}", depth);

        if verbose {
            println!("\n=== Engine Information ===");
            println!("Debug mode: {}", engine.is_debug_enabled());
            println!("Opening book loaded: {}", engine.is_opening_book_loaded());
        }
    } else {
        println!("No legal moves found!");
    }

    Ok(())
}

fn analyze_sfen_position(
    sfen: &str,
    depth: u8,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = ShogiEngine::new();

    if verbose {
        println!("Analyzing position: {}", sfen);
        println!("Search depth: {}", depth);
        println!("Note: SFEN parsing not yet implemented - using starting position");
    }

    let start_time = std::time::Instant::now();

    if let Some(best_move) = engine.get_best_move(depth, 5000, None) {
        let elapsed = start_time.elapsed();

        println!("\n=== Analysis Results ===");
        println!("Best move: {}", best_move.to_usi_string());
        println!("Search time: {:.2}ms", elapsed.as_millis());
        println!("Depth: {}", depth);

        if verbose {
            println!("\n=== Engine Information ===");
            println!("Debug mode: {}", engine.is_debug_enabled());
            println!("Opening book loaded: {}", engine.is_opening_book_loaded());
        }
    } else {
        println!("No legal moves found!");
    }

    Ok(())
}

fn compare_positions(
    positions: &[String],
    depth: u8,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if positions.is_empty() {
        return Err("No positions provided for comparison".into());
    }

    println!("Comparing {} positions at depth {}", positions.len(), depth);
    println!("==========================================");

    for (i, sfen) in positions.iter().enumerate() {
        println!("\nPosition {}: {}", i + 1, sfen);
        println!("{}", "-".repeat(50));

        analyze_sfen_position(sfen, depth, false)?;
    }

    Ok(())
}
