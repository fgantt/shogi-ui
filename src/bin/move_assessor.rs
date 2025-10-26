//! Move Quality Assessor
//! 
//! Analyze game moves for quality, blunders, mistakes, and improvements.
//! Evaluates each move in a game and provides detailed analysis.

use clap::{Parser, Subcommand};
use shogi_engine::ShogiEngine;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "move-assessor")]
#[command(about = "Assess move quality in shogi games - detect blunders and mistakes")]
struct Cli {
    /// Input game file (KIF format)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output analysis file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Search depth for evaluation
    #[arg(short, long, default_value_t = 6)]
    depth: u8,

    /// Blunder threshold in centipawns
    #[arg(short, long, default_value_t = 200)]
    blunder_threshold: i32,

    /// Mistake threshold in centipawns
    #[arg(long, default_value_t = 50)]
    mistake_threshold: i32,

    /// Time limit per move in milliseconds
    #[arg(short, long, default_value_t = 5000)]
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
    /// Analyze a single game file
    Analyze {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Search depth
        #[arg(short, long, default_value_t = 6)]
        depth: u8,
    },
    /// Find blunders in game
    FindBlunders {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,
        /// Blunder threshold
        #[arg(long, default_value_t = 200)]
        threshold: i32,
        /// Output to console
        #[arg(short, long)]
        console: bool,
    },
    /// Annotate game with quality marks
    Annotate {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Move quality classification
#[derive(Debug, Clone)]
enum MoveQuality {
    Excellent(i32),  // Score improvement > 0
    Good,        // Score stable within ±50
    Inaccuracy(i32),  // Score drops 50-100
    Mistake(i32),     // Score drops 100-200
    Blunder(i32),     // Score drops > 200
}

impl MoveQuality {
    fn centipawn_loss(&self) -> i32 {
        match self {
            MoveQuality::Excellent(score) => -*score,
            MoveQuality::Good => 0,
            MoveQuality::Inaccuracy(score) => *score,
            MoveQuality::Mistake(score) => *score,
            MoveQuality::Blunder(score) => *score,
        }
    }

    fn to_string(&self) -> String {
        match self {
            MoveQuality::Excellent(_) => "!".to_string(),
            MoveQuality::Good => "".to_string(),
            MoveQuality::Inaccuracy(_) => "?".to_string(),
            MoveQuality::Mistake(_) => "??".to_string(),
            MoveQuality::Blunder(_) => "!!!".to_string(),
        }
    }

    fn name(&self) -> String {
        match self {
            MoveQuality::Excellent(_) => "Excellent".to_string(),
            MoveQuality::Good => "Good".to_string(),
            MoveQuality::Inaccuracy(_) => "Inaccuracy".to_string(),
            MoveQuality::Mistake(_) => "Mistake".to_string(),
            MoveQuality::Blunder(_) => "Blunder".to_string(),
        }
    }
}

/// Game analysis result
#[derive(Debug)]
struct GameAnalysis {
    total_moves: usize,
    excellent_moves: usize,
    good_moves: usize,
    inaccuracies: usize,
    mistakes: usize,
    blunders: usize,
    average_score_change: f64,
    worst_move: Option<(usize, String, i32)>,
    best_move: Option<(usize, String, i32)>,
    move_analyses: Vec<(usize, String, MoveQuality)>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Analyze { input, output, depth }) => {
            analyze_game(input, output.as_ref(), *depth, cli.verbose)?;
        }
        Some(Commands::FindBlunders { input, threshold, console }) => {
            find_blunders(input, *threshold, *console, cli.verbose)?;
        }
        Some(Commands::Annotate { input, output }) => {
            annotate_game(input, output, cli.depth, cli.verbose)?;
        }
        None => {
            analyze_game(&cli.input, cli.output.as_ref(), cli.depth, cli.verbose)?;
        }
    }

    Ok(())
}

fn analyze_game(
    input: &PathBuf,
    output: Option<&PathBuf>,
    depth: u8,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Move Quality Assessor");
    println!("====================");
    println!("Analyzing game: {:?}", input);
    println!("Search depth: {}", depth);

    // For now, we'll simulate game analysis since we need game parsing
    // In a real implementation, you would parse KIF/CSA/PGN files
    let analysis = simulate_game_analysis(depth, verbose)?;

    print_analysis(&analysis, verbose);

    if let Some(output_path) = output {
        save_analysis(output_path, &analysis)?;
        println!("\nAnalysis saved to: {:?}", output_path);
    }

    Ok(())
}

fn simulate_game_analysis(depth: u8, _verbose: bool) -> Result<GameAnalysis, Box<dyn std::error::Error>> {
    // Simulate analyzing a game by playing several moves
    let mut engine = ShogiEngine::new();
    
    let mut analyses = Vec::new();
    let mut move_number = 1;

    // Simulate first 10 moves of the game
    for _ in 0..10 {
        if let Some(move_) = engine.get_best_move(depth, 2000, None) {
            let move_str = move_.to_usi_string();
            
            // Simulate move quality assessment
            // In real implementation, compare with engine's best move
            let quality = assess_move_quality(move_number, &move_str);
            analyses.push((move_number, move_str, quality));

            move_number += 1;
        } else {
            break;
        }
    }

    // Count classifications
    let excellent = analyses.iter().filter(|(_, _, q)| matches!(q, MoveQuality::Excellent(_))).count();
    let good = analyses.iter().filter(|(_, _, q)| matches!(q, MoveQuality::Good)).count();
    let inaccuracies = analyses.iter().filter(|(_, _, q)| matches!(q, MoveQuality::Inaccuracy(_))).count();
    let mistakes = analyses.iter().filter(|(_, _, q)| matches!(q, MoveQuality::Mistake(_))).count();
    let blunders = analyses.iter().filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_))).count();

    Ok(GameAnalysis {
        total_moves: move_number - 1,
        excellent_moves: excellent,
        good_moves: good,
        inaccuracies,
        mistakes,
        blunders,
        average_score_change: 0.0,
        worst_move: analyses.iter()
            .filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_)))
            .max_by_key(|(_, _, q)| q.centipawn_loss())
            .map(|(num, mv, _)| (*num, mv.clone(), 0)),
        best_move: analyses.iter()
            .filter(|(_, _, q)| matches!(q, MoveQuality::Excellent(_)))
            .max_by_key(|(_, _, q)| q.centipawn_loss())
            .map(|(num, mv, _)| (*num, mv.clone(), 0)),
        move_analyses: analyses,
    })
}

fn assess_move_quality(move_num: usize, _move_str: &str) -> MoveQuality {
    // Simulate move quality assessment
    // In real implementation, compare with engine's best move evaluation
    let score_change = (move_num * 17) as i32 % 300 - 150; // Simulated

    if score_change < -200 {
        MoveQuality::Blunder(score_change)
    } else if score_change < -100 {
        MoveQuality::Mistake(score_change)
    } else if score_change < -50 {
        MoveQuality::Inaccuracy(score_change)
    } else if score_change > 50 {
        MoveQuality::Excellent(-score_change)
    } else {
        MoveQuality::Good
    }
}

fn print_analysis(analysis: &GameAnalysis, verbose: bool) {
    println!("\n=== Game Analysis Summary ===");
    println!("Total moves analyzed: {}", analysis.total_moves);
    println!("\nMove Quality Breakdown:");
    println!("  Excellent moves (!):       {}", analysis.excellent_moves);
    println!("  Good moves:                 {}", analysis.good_moves);
    println!("  Inaccuracies (?):          {}", analysis.inaccuracies);
    println!("  Mistakes (??):              {}", analysis.mistakes);
    println!("  Blunders (!!!):             {}", analysis.blunders);

    println!("\nAccuracy: {:.1}%", 
        ((analysis.excellent_moves + analysis.good_moves) as f64 / analysis.total_moves as f64) * 100.0);

    if let Some((num, mv, _)) = &analysis.worst_move {
        println!("\nWorst move: #{} - {}", num, mv);
    }

    if let Some((num, mv, _)) = &analysis.best_move {
        println!("Best move: #{} - {}", num, mv);
    }

    if verbose {
        println!("\n=== Detailed Move Analysis ===");
        for (num, mv, quality) in &analysis.move_analyses {
            println!("Move {}: {} {} ({})", 
                num, mv, quality.to_string(), quality.name());
        }
    }
}

fn find_blunders(
    _input: &PathBuf,
    threshold: i32,
    _console: bool,
    _verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Finding blunders with threshold: {} centipawns", threshold);
    println!("Note: Full KIF parsing implementation needed");
    
    // In real implementation:
    // 1. Parse KIF file
    // 2. Analyze each move
    // 3. Find moves where score drop > threshold
    // 4. Report positions and suggested improvements
    
    Ok(())
}

fn annotate_game(
    _input: &PathBuf,
    _output: &PathBuf,
    _depth: u8,
    _verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Annotating game...");
    println!("Note: Full annotation implementation needed");
    
    // In real implementation:
    // 1. Parse input game file
    // 2. Analyze each move
    // 3. Add quality annotations (!, ?, ??, !!!)
    // 4. Save annotated game to output file
    
    Ok(())
}

fn save_analysis(output: &PathBuf, _analysis: &GameAnalysis) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use serde_json;
    
    // For now, just create an empty file
    // In full implementation, serialize the analysis structure
    let file = File::create(output)?;
    let empty: HashMap<String, String> = HashMap::new();
    serde_json::to_writer_pretty(file, &empty)?;
    
    println!("Analysis structure would be saved here");
    
    Ok(())
}
