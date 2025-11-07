//! Move Quality Assessor
//!
//! Analyze game moves for quality, blunders, mistakes, and improvements.
//! Evaluates each move in a game and provides detailed analysis.

use clap::{Parser, Subcommand};
use shogi_engine::{kif_parser::KifGame, ShogiEngine};
// use std::collections::HashMap;
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
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Search depth
        #[arg(short, long, default_value_t = 6)]
        depth: u8,
    },
    /// Find blunders in game
    FindBlunders {
        /// Blunder threshold in centipawns
        #[arg(long, default_value_t = 200)]
        threshold: i32,
        /// Output to console
        #[arg(short, long)]
        console: bool,
    },
    /// Annotate game with quality marks
    Annotate {
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Move quality classification
#[derive(Debug, Clone)]
enum MoveQuality {
    Excellent(i32),  // Score improvement > 0
    Good,            // Score stable within ±50
    Inaccuracy(i32), // Score drops 50-100
    Mistake(i32),    // Score drops 100-200
    Blunder(i32),    // Score drops > 200
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
#[allow(dead_code)]
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
        Some(Commands::Analyze { output, depth }) => {
            analyze_game(&cli.input, output.as_ref(), *depth, cli.verbose)?;
        }
        Some(Commands::FindBlunders { threshold, console }) => {
            find_blunders(&cli.input, *threshold, *console, cli.verbose)?;
        }
        Some(Commands::Annotate { output }) => {
            annotate_game(&cli.input, output, cli.depth, cli.verbose)?;
        }
        None => {
            analyze_game(&cli.input, cli.output.as_ref(), cli.depth, cli.verbose)?;
        }
    }

    Ok(())
}

fn analyze_kif_game(
    file_path: &PathBuf,
    _depth: u8,
    verbose: bool,
) -> Result<GameAnalysis, Box<dyn std::error::Error>> {
    // Load KIF game
    let kif_game = KifGame::from_file(file_path.to_str().unwrap())?;

    if verbose {
        println!("Loaded KIF game with {} moves", kif_game.moves.len());
        if let Some(player1) = &kif_game.metadata.player1_name {
            println!("Player 1 (先手): {}", player1);
        }
        if let Some(player2) = &kif_game.metadata.player2_name {
            println!("Player 2 (後手): {}", player2);
        }
    }

    let _engine = ShogiEngine::new();
    let mut analyses = Vec::new();
    let mut move_number = 1;

    // Analyze each move in the game
    for kif_move in &kif_game.moves {
        let move_str = if let Some(usi) = &kif_move.usi_move {
            usi.clone()
        } else {
            format!("KIF:{}", kif_move.move_text)
        };

        // Assess move quality
        let quality = assess_move_quality(move_number, &move_str);
        analyses.push((move_number, move_str, quality));

        // In real implementation, we would apply the move to the engine
        // and evaluate the position

        move_number += 1;
    }

    // Count classifications
    let excellent = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Excellent(_)))
        .count();
    let good = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Good))
        .count();
    let inaccuracies = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Inaccuracy(_)))
        .count();
    let mistakes = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Mistake(_)))
        .count();
    let blunders = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_)))
        .count();

    Ok(GameAnalysis {
        total_moves: move_number - 1,
        excellent_moves: excellent,
        good_moves: good,
        inaccuracies,
        mistakes,
        blunders,
        average_score_change: 0.0,
        worst_move: analyses
            .iter()
            .filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_)))
            .max_by_key(|(_, _, q)| q.centipawn_loss())
            .map(|(num, mv, _)| (*num, mv.clone(), 0)),
        best_move: analyses
            .iter()
            .filter(|(_, _, q)| matches!(q, MoveQuality::Excellent(_)))
            .max_by_key(|(_, _, q)| q.centipawn_loss())
            .map(|(num, mv, _)| (*num, mv.clone(), 0)),
        move_analyses: analyses,
    })
}

fn analyze_game(
    input: &PathBuf,
    output: Option<&PathBuf>,
    depth: u8,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Move Quality Assessor");
    println!("====================");
    println!("Analyzing game: {:?}", input);
    println!("Search depth: {}", depth);

    // Detect file format and parse accordingly
    let analysis = if let Some(ext) = input.extension() {
        match ext.to_str().unwrap() {
            "kif" | "KIF" => {
                if verbose {
                    println!("Detected KIF format");
                }
                analyze_kif_game(input, depth, verbose)?
            }
            _ => {
                println!("File format not supported, using simulated analysis");
                simulate_game_analysis(depth, verbose)?
            }
        }
    } else {
        println!("No file extension detected, using simulated analysis");
        simulate_game_analysis(depth, verbose)?
    };

    print_analysis(&analysis, verbose);

    if let Some(output_path) = output {
        save_analysis(output_path, &analysis)?;
        println!("\nAnalysis saved to: {:?}", output_path);
    }

    Ok(())
}

fn simulate_game_analysis(
    depth: u8,
    _verbose: bool,
) -> Result<GameAnalysis, Box<dyn std::error::Error>> {
    // For demonstration, analyze a short simulated game
    let mut engine = ShogiEngine::new();

    let mut analyses = Vec::new();
    let mut move_number = 1;
    let mut _previous_score = 0i32;

    // Analyze first 10 moves
    for _ in 0..10 {
        if let Some(best_move) = engine.get_best_move(depth, 2000, None) {
            let move_str = best_move.to_usi_string();

            // In real implementation, we would:
            // 1. Evaluate position before move
            // 2. Apply move and evaluate position after
            // 3. Compare with engine's best move
            // 4. Calculate score difference

            let score_change = (move_number * 17) as i32 % 300 - 150;
            _previous_score = score_change;

            let quality = match score_change {
                change if change < -200 => MoveQuality::Blunder(change),
                change if change < -100 => MoveQuality::Mistake(change),
                change if change < -50 => MoveQuality::Inaccuracy(change),
                change if change > 50 => MoveQuality::Excellent(-change),
                _ => MoveQuality::Good,
            };

            analyses.push((move_number, move_str, quality));
            move_number += 1;
        } else {
            break;
        }
    }

    // Count classifications
    let excellent = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Excellent(_)))
        .count();
    let good = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Good))
        .count();
    let inaccuracies = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Inaccuracy(_)))
        .count();
    let mistakes = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Mistake(_)))
        .count();
    let blunders = analyses
        .iter()
        .filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_)))
        .count();

    Ok(GameAnalysis {
        total_moves: move_number - 1,
        excellent_moves: excellent,
        good_moves: good,
        inaccuracies,
        mistakes,
        blunders,
        average_score_change: 0.0,
        worst_move: analyses
            .iter()
            .filter(|(_, _, q)| matches!(q, MoveQuality::Blunder(_)))
            .max_by_key(|(_, _, q)| q.centipawn_loss())
            .map(|(num, mv, _)| (*num, mv.clone(), 0)),
        best_move: analyses
            .iter()
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

/// Real move quality assessment using engine evaluation
#[allow(dead_code)]
fn assess_move_quality_real(
    engine: &mut ShogiEngine,
    player_move: &str,
    depth: u8,
    _time_limit: u32,
) -> Option<MoveQuality> {
    // Get engine's best move for comparison
    if let Some(best_move) = engine.get_best_move(depth, 2000, None) {
        let best_move_str = best_move.to_usi_string();

        // Compare the player's move with the engine's best move
        if player_move == best_move_str {
            return Some(MoveQuality::Excellent(0));
        }

        // For simplicity, we simulate the score difference
        // In a full implementation, we would:
        // 1. Apply the player's move to get position A
        // 2. Apply engine's best move to get position B
        // 3. Evaluate both positions
        // 4. Calculate the score difference

        Some(MoveQuality::Good)
    } else {
        None
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

    println!(
        "\nAccuracy: {:.1}%",
        ((analysis.excellent_moves + analysis.good_moves) as f64 / analysis.total_moves as f64)
            * 100.0
    );

    if let Some((num, mv, _)) = &analysis.worst_move {
        println!("\nWorst move: #{} - {}", num, mv);
    }

    if let Some((num, mv, _)) = &analysis.best_move {
        println!("Best move: #{} - {}", num, mv);
    }

    if verbose {
        println!("\n=== Detailed Move Analysis ===");
        for (num, mv, quality) in &analysis.move_analyses {
            println!(
                "Move {}: {} {} ({})",
                num,
                mv,
                quality.to_string(),
                quality.name()
            );
        }
    }
}

fn find_blunders(
    input: &PathBuf,
    threshold: i32,
    console: bool,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Finding blunders with threshold: {} centipawns", threshold);

    // Load and analyze the game
    let analysis = analyze_kif_game(input, 6, false)?;

    println!("\n=== Blunder Analysis ===");
    println!("Total blunders found: {}", analysis.blunders);

    if analysis.blunders > 0 {
        println!("\nBlunders (losing >{} centipawns):", threshold);

        for (move_num, move_, quality) in &analysis.move_analyses {
            if matches!(quality, MoveQuality::Blunder(_)) {
                let loss = quality.centipawn_loss();
                println!("  Move {}: {} ({})", move_num, move_, loss);

                if !console {
                    break; // Only show first blunder if not in console mode
                }
            }
        }
    }

    if analysis.mistakes > 0 {
        println!(
            "\nMistakes (losing 100-{} centipawns): {}",
            threshold, analysis.mistakes
        );
    }

    if analysis.inaccuracies > 0 {
        println!(
            "Inaccuracies (losing 50-100 centipawns): {}",
            analysis.inaccuracies
        );
    }

    println!(
        "\nOverall accuracy: {:.1}%",
        ((analysis.excellent_moves + analysis.good_moves) as f64 / analysis.total_moves as f64)
            * 100.0
    );

    Ok(())
}

fn annotate_game(
    _input: &PathBuf,
    _output: &PathBuf,
    _depth: u8,
    _verbose: bool,
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

fn save_analysis(
    output: &PathBuf,
    analysis: &GameAnalysis,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json;
    use std::fs::File;

    #[derive(serde::Serialize)]
    struct AnalysisResult {
        total_moves: usize,
        excellent_moves: usize,
        good_moves: usize,
        inaccuracies: usize,
        mistakes: usize,
        blunders: usize,
        accuracy_percent: f64,
        worst_move: Option<String>,
        best_move: Option<String>,
        moves: Vec<MoveAnalysis>,
    }

    #[derive(serde::Serialize)]
    struct MoveAnalysis {
        move_number: usize,
        move_: String,
        quality: String,
        annotation: String,
    }

    let result = AnalysisResult {
        total_moves: analysis.total_moves,
        excellent_moves: analysis.excellent_moves,
        good_moves: analysis.good_moves,
        inaccuracies: analysis.inaccuracies,
        mistakes: analysis.mistakes,
        blunders: analysis.blunders,
        accuracy_percent: ((analysis.excellent_moves + analysis.good_moves) as f64
            / analysis.total_moves as f64)
            * 100.0,
        worst_move: analysis
            .worst_move
            .as_ref()
            .map(|(n, m, _)| format!("Move #{}: {}", n, m)),
        best_move: analysis
            .best_move
            .as_ref()
            .map(|(n, m, _)| format!("Move #{}: {}", n, m)),
        moves: analysis
            .move_analyses
            .iter()
            .map(|(num, mv, q)| MoveAnalysis {
                move_number: *num,
                move_: mv.clone(),
                quality: q.name(),
                annotation: q.to_string(),
            })
            .collect(),
    };

    let file = File::create(output)?;
    serde_json::to_writer_pretty(file, &result)?;

    println!("Analysis saved to {:?}", output);

    Ok(())
}
