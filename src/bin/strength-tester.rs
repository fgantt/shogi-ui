//! Engine Strength Tester Utility
//!
//! A command-line tool for testing the strength of the shogi engine.

use clap::{Parser, Subcommand};
use shogi_engine::{
    types::{GameResult, Move},
    ShogiEngine,
};
// use std::path::PathBuf;

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

fn test_strength(
    time_control: &str,
    games: u32,
    depth: u8,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
            GameResult::Win => {
                // Black wins
                black_wins += 1;
            }
            GameResult::Loss => {
                // White wins
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
    let mut consecutive_passes = 0;
    let mut last_move: Option<Move> = None;

    // Play a game by having engine play against itself
    loop {
        // Check if game is over
        if let Some(result) = engine.is_game_over() {
            if verbose {
                println!("Game over: {:?}", result);
            }
            return Ok(result);
        }

        // Get engine's best move
        if let Some(best_move) = engine.get_best_move(depth, 2000, None) {
            if verbose && move_count < 10 {
                println!("Move {}: {}", move_count + 1, best_move.to_usi_string());
            }

            // Check if same move repeated (possible infinite loop)
            if last_move.as_ref().map(|m| m.to_usi_string()) == Some(best_move.to_usi_string()) {
                consecutive_passes += 1;
                if consecutive_passes >= 3 {
                    if verbose {
                        println!("Consecutive repeated moves detected, ending game as draw");
                    }
                    return Ok(GameResult::Draw);
                }
            } else {
                consecutive_passes = 0;
            }

            last_move = Some(best_move.clone());

            // Apply the move to the engine
            if !engine.apply_move(&best_move) {
                if verbose {
                    println!(
                        "Failed to apply move: {}, ending game",
                        best_move.to_usi_string()
                    );
                }
                return Ok(GameResult::Draw);
            }

            move_count += 1;

            // Safety limit to avoid infinite games
            if move_count >= 200 {
                if verbose {
                    println!("Maximum move limit reached, ending as draw");
                }
                return Ok(GameResult::Draw);
            }
        } else {
            // No legal moves - game ended
            if let Some(result) = engine.is_game_over() {
                return Ok(result);
            }
            return Ok(GameResult::Draw);
        }
    }
}

fn compare_configs(
    config1: &str,
    config2: &str,
    games: u32,
    depth: u8,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn estimate_elo(
    opponent: &str,
    games: u32,
    depth: u8,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Estimating ELO rating...");
        println!("Opponent: {}", opponent);
        println!("Games: {}", games);
        println!("Search depth: {}", depth);
    }

    println!("\nELO estimation not yet implemented.");
    Ok(())
}
