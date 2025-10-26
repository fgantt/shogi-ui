//! Engine Strength Tester Utility
//!
//! A command-line tool for testing the strength of the shogi engine.

use clap::{Parser, Subcommand};
use shogi_engine::types::{GameResult, Player};
use std::io::{BufRead, BufReader, Write, Read};
use std::process::{Command, Stdio};

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
        let result = play_game_as_process(depth, verbose)?;
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

fn play_game_as_process(depth: u8, verbose: bool) -> Result<GameResult, Box<dyn std::error::Error>> {
    let mut child = Command::new("./target/debug/usi-engine")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // USI handshake
    stdin.write_all(b"usi\n")?;
    wait_for_line(&mut reader, "usiok")?;
    stdin.write_all(b"isready\n")?;
    wait_for_line(&mut reader, "readyok")?;

    let mut move_count = 0;
    let mut current_player = Player::Black;
    let mut moves = String::new();

    loop {
        let position_cmd = if moves.is_empty() {
            "position startpos\n".to_string()
        } else {
            format!("position startpos moves {}\n", moves.trim())
        };
        stdin.write_all(position_cmd.as_bytes())?;
        stdin.write_all(format!("go depth {}\n", depth).as_bytes())?;

        let best_move = wait_for_bestmove(&mut reader)?;

        if best_move == "(none)" {
            return Ok(if current_player == Player::Black { GameResult::Loss } else { GameResult::Win });
        }

        if verbose {
            println!("Move {}: {} (Player: {:?})", move_count + 1, best_move, current_player);
        }

        moves.push_str(&best_move);
        moves.push(' ');
        move_count += 1;
        current_player = current_player.opposite();

        if move_count > 200 {
            child.kill()?;
            return Ok(GameResult::Draw);
        }
    }
}

fn wait_for_line(reader: &mut BufReader<impl Read>, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut line = String::new();
    loop {
        if reader.read_line(&mut line)? == 0 {
            return Err("Engine process closed unexpectedly".into());
        }
        if line.trim() == expected {
            return Ok(());
        }
        line.clear();
    }
}

fn wait_for_bestmove(reader: &mut BufReader<impl Read>) -> Result<String, Box<dyn std::error::Error>> {
    let mut line = String::new();
    loop {
        if reader.read_line(&mut line)? == 0 {
            return Err("Engine process closed unexpectedly while waiting for bestmove".into());
        }
        if let Some(mv) = line.strip_prefix("bestmove ") {
            return Ok(mv.trim().to_string());
        }
        line.clear();
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
