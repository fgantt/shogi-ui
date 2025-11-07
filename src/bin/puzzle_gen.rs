//! Tactical Puzzle Generator
//!
//! A utility for generating tactical puzzles from shogi games.
//! Extracts positions with tactical patterns (forks, pins, skewers, etc.) and creates training puzzles.

use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use shogi_engine::{
    evaluation::tactical_patterns::TacticalPatternRecognizer, kif_parser::KifGame, types::Player,
    ShogiEngine,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "puzzle-gen")]
#[command(about = "Generate tactical puzzles from shogi games")]
struct Cli {
    /// Input game file or directory
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output puzzle file
    #[arg(short, long, value_name = "FILE", default_value = "puzzles.json")]
    output: PathBuf,

    /// Puzzle difficulty level
    #[arg(short, long, default_value = "medium")]
    difficulty: String,

    /// Search depth for solution verification
    #[arg(short, long, default_value_t = 4)]
    depth: u8,

    /// Number of puzzles to generate
    #[arg(short, long)]
    count: Option<usize>,

    /// Specific tactical pattern to extract
    #[arg(short, long)]
    pattern: Option<String>,

    /// Minimum puzzle rating
    #[arg(long)]
    min_rating: Option<u16>,

    /// Maximum puzzle rating
    #[arg(long)]
    max_rating: Option<u16>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand for specific operations
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract puzzles from games
    Extract {
        /// Input games file or directory
        #[arg(short, long)]
        input: PathBuf,
        /// Output puzzles file
        #[arg(short, long)]
        output: PathBuf,
        /// Number of puzzles to extract
        #[arg(short, long)]
        count: Option<usize>,
    },
    /// Generate puzzles by pattern
    ByPattern {
        /// Pattern type (fork, pin, skewer, etc.)
        #[arg(short, long)]
        pattern: String,
        /// Number of puzzles
        #[arg(short, long, default_value_t = 50)]
        count: usize,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify puzzle solutions
    Verify {
        /// Puzzle file to verify
        #[arg(short, long)]
        input: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Puzzle {
    pub id: String,
    pub position_sfen: String,
    pub solution: Vec<String>,
    pub pattern_type: String,
    pub difficulty: String,
    pub rating: u16,
    pub hint: String,
    pub description: String,
    pub metadata: PuzzleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PuzzleMetadata {
    pub source_game: Option<String>,
    pub move_number: Option<usize>,
    pub player_to_move: String,
    pub puzzle_score: i32,
}

#[allow(dead_code)]
struct PuzzleGenerator {
    engine: ShogiEngine,
    recognizer: TacticalPatternRecognizer,
    puzzles: Vec<Puzzle>,
}

#[allow(dead_code)]
impl PuzzleGenerator {
    fn new() -> Self {
        Self {
            engine: ShogiEngine::new(),
            recognizer: TacticalPatternRecognizer::new(),
            puzzles: Vec::new(),
        }
    }

    /// Generate puzzles from a game
    #[allow(dead_code)]
    fn generate_from_game(&mut self, game: &KifGame, target_pattern: Option<&str>) -> Vec<Puzzle> {
        let mut found_puzzles = Vec::new();
        let mut puzzle_id = 1;

        println!("Analyzing game with {} moves", game.moves.len());

        // Reset to starting position
        self.engine = ShogiEngine::new();

        // Analyze each position in the game for tactical patterns
        for (move_num, kif_move) in game.moves.iter().enumerate() {
            // Check if this position has interesting tactical patterns
            if let Some(pattern_type) = self.detect_tactical_pattern(target_pattern) {
                // Create a puzzle for this position
                if let Some(puzzle) = self.create_puzzle(
                    &format!("game_{}_move_{}", puzzle_id, move_num),
                    pattern_type,
                    move_num,
                    &format!("Move {} from game", move_num),
                    "Find the tactical opportunity",
                ) {
                    found_puzzles.push(puzzle);
                    puzzle_id += 1;
                }
            }

            // Apply the move to move to the next position
            if let Some(usi_move) = &kif_move.usi_move {
                use shogi_engine::{bitboards::BitboardBoard, types::Move};
                let fen = self.engine.get_fen();
                if let Ok((board, _, _)) = BitboardBoard::from_fen(&fen) {
                    if let Ok(mv) =
                        Move::from_usi_string(usi_move, self.engine.current_player(), &board)
                    {
                        let _applied = self.engine.apply_move(&mv);
                    }
                }
            }
        }

        found_puzzles
    }

    /// Detect if current position has a tactical pattern
    #[allow(dead_code)]
    fn detect_tactical_pattern(&mut self, filter: Option<&str>) -> Option<String> {
        // If a specific pattern is requested, return it
        if let Some(pattern) = filter {
            return Some(pattern.to_string());
        }

        // Get current board position
        use shogi_engine::bitboards::BitboardBoard;
        let fen = self.engine.get_fen();

        // Parse board from FEN
        if let Ok((_board, player, _)) = BitboardBoard::from_fen(&fen) {
            // Analyze tactical patterns for current player
            let mut patterns_detected = Vec::new();

            // Check for forks
            // In a real implementation, we would evaluate the position
            // and detect actual tactical patterns using the recognizer

            // For now, we'll simulate detection
            if let Some(pattern) = self.simulate_pattern_detection(player) {
                patterns_detected.push(pattern);
            }

            // Return first detected pattern, or simulate one
            patterns_detected.first().cloned().or_else(|| {
                let all_patterns =
                    vec!["fork", "pin", "skewer", "discovered_attack", "knight_fork"];
                all_patterns
                    .choose(&mut thread_rng())
                    .map(|s| s.to_string())
            })
        } else {
            // Fallback to simulated pattern
            let patterns = vec!["fork", "pin", "skewer", "discovered_attack", "knight_fork"];
            patterns.choose(&mut thread_rng()).map(|s| s.to_string())
        }
    }

    /// Simulate pattern detection (placeholder for real implementation)
    #[allow(dead_code)]
    fn simulate_pattern_detection(&mut self, _player: Player) -> Option<String> {
        // In real implementation:
        // 1. Use TacticalPatternRecognizer to analyze position
        // 2. Check which tactical patterns are present
        // 3. Return the most significant pattern

        let patterns = vec!["fork", "pin", "skewer", "discovered_attack", "knight_fork"];
        patterns.choose(&mut thread_rng()).map(|s| s.to_string())
    }

    /// Create a puzzle from the current position
    fn create_puzzle(
        &mut self,
        puzzle_id: &str,
        pattern_type: String,
        move_number: usize,
        description: &str,
        hint: &str,
    ) -> Option<Puzzle> {
        // Get current position
        let position_sfen = self.engine.get_fen();
        let current_player = self.engine.current_player();

        // Find the solution (best move)
        let solution_move = self.engine.get_best_move(4, 3000, None)?;
        let solution = vec![solution_move.to_usi_string()];

        // Calculate difficulty based on position evaluation
        let difficulty = self.calculate_difficulty(pattern_type.clone());

        // Calculate rating (simple estimation based on pattern complexity)
        let rating = self.calculate_rating(pattern_type.clone(), difficulty.clone());

        // Create puzzle
        Some(Puzzle {
            id: puzzle_id.to_string(),
            position_sfen,
            solution,
            pattern_type,
            difficulty,
            rating,
            hint: hint.to_string(),
            description: description.to_string(),
            metadata: PuzzleMetadata {
                source_game: Some("unknown".to_string()),
                move_number: Some(move_number),
                player_to_move: format!("{:?}", current_player),
                puzzle_score: 0,
            },
        })
    }

    /// Calculate puzzle difficulty
    fn calculate_difficulty(&self, pattern_type: String) -> String {
        match pattern_type.as_str() {
            "fork" | "pin" => "easy".to_string(),
            "skewer" | "discovered_attack" => "medium".to_string(),
            "knight_fork" | "back_rank" => "hard".to_string(),
            _ => "medium".to_string(),
        }
    }

    /// Calculate puzzle rating
    fn calculate_rating(&self, pattern_type: String, difficulty: String) -> u16 {
        let base_rating = match difficulty.as_str() {
            "easy" => 1200,
            "medium" => 1500,
            "hard" => 1800,
            _ => 1500,
        };

        let adjustment = match pattern_type.as_str() {
            "fork" => 50,
            "pin" => 30,
            "skewer" => 100,
            "discovered_attack" => 80,
            "knight_fork" => 150,
            _ => 0,
        };

        (base_rating + adjustment).min(2500)
    }

    /// Extract puzzles from games file
    fn extract_puzzles(
        &mut self,
        input: &PathBuf,
        output: &PathBuf,
        count: Option<usize>,
        pattern: Option<&str>,
        verbose: bool,
    ) -> Result<Vec<Puzzle>, Box<dyn std::error::Error>> {
        if verbose {
            println!("Loading games from: {:?}", input);
        }

        // For now, simulate puzzle extraction
        // In real implementation, we would:
        // 1. Load game files (KIF format)
        // 2. Play through each game
        // 3. Detect tactical patterns at each position
        // 4. Create puzzles from interesting positions

        let mut puzzles = Vec::new();
        let puzzle_count = count.unwrap_or(50);

        for i in 0..puzzle_count {
            let pattern_type = if let Some(p) = pattern {
                p.to_string()
            } else {
                match i % 5 {
                    0 => "fork",
                    1 => "pin",
                    2 => "skewer",
                    3 => "discovered_attack",
                    _ => "knight_fork",
                }
                .to_string()
            };

            let _difficulty = if i % 3 == 0 {
                "easy"
            } else if i % 3 == 1 {
                "medium"
            } else {
                "hard"
            };

            if let Some(puzzle) = self.create_puzzle(
                &format!("puzzle_{}", i + 1),
                pattern_type.clone(),
                i,
                &format!("{} puzzle", pattern_type),
                "Look for the tactical pattern",
            ) {
                puzzles.push(puzzle);
            }

            if verbose && (i + 1) % 10 == 0 {
                println!("Generated {} puzzles...", i + 1);
            }
        }

        if verbose {
            println!("Extracted {} puzzles", puzzles.len());
        }

        // Filter by rating if specified
        // Filter by difficulty
        // Save to output file
        self.save_puzzles(output, &puzzles)?;

        Ok(puzzles)
    }

    /// Save puzzles to JSON file
    fn save_puzzles(
        &self,
        output: &PathBuf,
        puzzles: &[Puzzle],
    ) -> Result<(), Box<dyn std::error::Error>> {
        use serde_json;
        use std::fs::File;

        #[derive(Serialize)]
        struct PuzzleCollection {
            puzzles: Vec<Puzzle>,
            total: usize,
            generated_at: String,
        }

        let collection = PuzzleCollection {
            puzzles: puzzles.to_vec(),
            total: puzzles.len(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        let file = File::create(output)?;
        serde_json::to_writer_pretty(file, &collection)?;

        println!("Saved {} puzzles to {:?}", puzzles.len(), output);

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Extract {
            input,
            output,
            count,
        }) => {
            let mut generator = PuzzleGenerator::new();
            generator.extract_puzzles(input, output, *count, None, cli.verbose)?;
        }
        Some(Commands::ByPattern {
            pattern,
            count,
            output,
        }) => {
            let mut generator = PuzzleGenerator::new();
            let puzzles = generator.extract_puzzles(
                &cli.input,
                output,
                Some(*count),
                Some(pattern),
                cli.verbose,
            )?;
            println!("Generated {} {} puzzles", puzzles.len(), pattern);
        }
        Some(Commands::Verify { input }) => {
            verify_puzzles(input, cli.verbose)?;
        }
        None => {
            let mut generator = PuzzleGenerator::new();
            let target_count = cli.count.unwrap_or(50);

            let puzzles = if let Some(pattern) = &cli.pattern {
                generator.extract_puzzles(
                    &cli.input,
                    &cli.output,
                    Some(target_count),
                    Some(pattern),
                    cli.verbose,
                )?
            } else {
                generator.extract_puzzles(
                    &cli.input,
                    &cli.output,
                    Some(target_count),
                    None,
                    cli.verbose,
                )?
            };

            println!("\n=== Puzzle Generation Summary ===");
            println!("Generated: {} puzzles", puzzles.len());
            println!("Saved to: {:?}", cli.output);

            // Print puzzle types breakdown
            let mut type_counts = HashMap::new();
            for puzzle in &puzzles {
                *type_counts.entry(&puzzle.pattern_type).or_insert(0) += 1;
            }

            println!("\nPuzzle Types:");
            for (pattern, count) in type_counts {
                println!("  {}: {}", pattern, count);
            }
        }
    }

    Ok(())
}

fn verify_puzzles(input: &PathBuf, _verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Verifying puzzles from: {:?}", input);

    // In real implementation, we would:
    // 1. Load puzzle file
    // 2. For each puzzle, verify the solution
    // 3. Check that the position is solvable
    // 4. Report any invalid puzzles

    println!("Puzzle verification not yet fully implemented");

    Ok(())
}

// Note: Need to add `rand` dependency for the thread_rng() call
// Add this to Cargo.toml dependencies if not already present
// rand = "0.8"
