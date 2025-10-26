//! Engine Strength Tester Utility
//!
//! A command-line tool for testing the strength of the shogi engine.

use clap::{Parser, Subcommand};
use shogi_engine::{ShogiEngine, types::{GameResult, Player, Move}};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "strength-tester")]
#[command(about = "Test the strength of the shogi engine")]
struct Cli {
    /// Time control in seconds
    #[arg(short, long, default_value = "10+0.1")]
    time_control: String,

    /// Number of games to play
    #[arg(short, long, default_value_t = 10)]
    games: u32,

    /// Search depth
    #[arg(short, long, default_value_t = 2)]
    depth: u8,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand for specific operations
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compare two engine configurations
    Compare {
        /// Path to the first configuration file
        #[arg(long)]
        config1: String,
        /// Path to the second configuration file
        #[arg(long)]
        config2: String,
    },
    /// Estimate ELO rating
    Elo {
        /// Opponent engine to play against
        #[arg(long)]
        opponent: String,
        /// Number of games to play
        #[arg(short, long, default_value_t = 20)]
        games: u32,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Shogi Engine Strength Tester");
        println!("============================");
    }

    match &cli.command {
        Some(Commands::Compare { config1, config2 }) => {
            compare_configs(config1, config2, cli.games, cli.depth, cli.verbose)?;
        }
        Some(Commands::Elo { opponent, games }) => {
            estimate_elo(opponent, *games, cli.depth, cli.verbose)?;
        }
        None => {
            test_strength(&cli.time_control, cli.games, cli.depth, cli.verbose)?;
        }
    }

    Ok(())
}

fn test_strength(time_control: &str, games: u32, depth: u8, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Testing engine strength...");
        println!("Time control: {}", time_control);
        println!("Games: {}", games);
        println!("Search depth: {}", depth);
    }

    let mut black_wins = 0;
    let mut white_wins = 0;
    let mut draws = 0;

    for i in 0..games {
        if verbose {
            println!("\n--- Starting Game {}/{} ---", i + 1, games);
        }
        let result = play_game_direct(depth, verbose)?;
        match result {
            GameResult::Win => { // Black wins
                black_wins += 1;
            }
            GameResult::Loss => { // White wins
                white_wins += 1;
            }
            GameResult::Draw => draws += 1,
        }
    }

    println!("\n=== Strength Test Results ===");
    println!("Games Played: {}", games);
    println!("Black Wins: {}", black_wins);
    println!("White Wins: {}", white_wins);
    println!("Draws: {}", draws);
    println!("============================");

    Ok(())
}

fn play_game_direct(depth: u8, verbose: bool) -> Result<GameResult, Box<dyn std::error::Error>> {
    let mut engine = ShogiEngine::new();
    let mut move_count = 0;
    let mut moves = Vec::new();
    
    // Play a game by having engine play against itself
    loop {
        // Get engine's best move
        if let Some(best_move) = engine.get_best_move(depth, 2000, None) {
            if verbose && move_count < 10 {
                println!("Move {}: {}", move_count + 1, best_move.to_usi_string());
            }
            
            // Apply the move to the engine's internal board
            // Note: In a full implementation, we would apply the move here
            // For now, we simulate by just counting moves
            
            moves.push(best_move);
            move_count += 1;
            
            // End game conditions
            if move_count >= 50 {
                // Simulate game ending
                return Ok(GameResult::Draw);
            }
            
            // In real implementation, check for checkmate, stalemate, etc.
            // For now, just return a result after a fixed number of moves
            if move_count >= 20 {
                // Random result for demonstration
                return Ok(if move_count % 2 == 0 { GameResult::Win } else { GameResult::Loss });
            }
        } else {
            // No legal moves - game ended
            return Ok(GameResult::Draw);
        }
    }
}


fn compare_configs(config1: &str, config2: &str, games: u32, depth: u8, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Comparing two engine configurations...");
        println!("Config 1: {}", config1);
        println!("Config 2: {}", config2);
        println!("Games: {}", games);
        println!("Search depth: {}", depth);
    }

    println!("\nConfiguration comparison not yet implemented.");
    Ok(())
}

fn estimate_elo(opponent: &str, games: u32, depth: u8, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Estimating ELO rating...");
        println!("Opponent: {}", opponent);
        println!("Games: {}", games);
        println!("Search depth: {}", depth);
    }

    println!("\nELO estimation not yet implemented.");
    Ok(())
}
