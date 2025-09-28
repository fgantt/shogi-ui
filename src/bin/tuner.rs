//! Shogi Engine Tuning Binary
//! 
//! This binary provides a command-line interface for automated evaluation tuning
//! using various optimization algorithms and validation methods.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;
use shogi_engine::tuning::{
    types::{
        TuningConfig, OptimizationMethod, ValidationConfig, 
        PositionFilter, PerformanceConfig, TuningResults
    },
    data_processor::DataProcessor,
    optimizer::Optimizer,
    validator::Validator,
};

/// Shogi Engine Automated Tuning Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "tuner")]
#[command(about = "Automated evaluation tuning for Shogi engine")]
struct Cli {
    /// Dataset file path (JSON, KIF, CSA, or PGN format)
    #[arg(short, long, value_name = "FILE")]
    dataset: PathBuf,

    /// Output weights file path
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,

    /// Optimization method to use
    #[arg(short, long, default_value = "adam")]
    #[arg(value_parser = ["gradient-descent", "adam", "lbfgs", "genetic"])]
    method: String,

    /// Maximum number of optimization iterations
    #[arg(short, long, default_value_t = 1000)]
    iterations: u32,

    /// Number of folds for k-fold cross-validation
    #[arg(short = 'k', long, default_value_t = 5)]
    k_fold: u32,

    /// Percentage of data to use for testing (0.0 to 1.0)
    #[arg(long, default_value_t = 0.2)]
    test_split: f64,

    /// L2 regularization strength
    #[arg(long, default_value_t = 0.01)]
    regularization: f64,

    /// Quiet move threshold for position filtering
    #[arg(long, default_value_t = 3)]
    quiet_threshold: u32,

    /// Minimum player rating for game filtering
    #[arg(long, default_value_t = 1800)]
    min_rating: u32,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable progress reporting
    #[arg(short, long)]
    progress: bool,

    /// Subcommand for specific operations
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Subcommands for specific tuning operations
#[derive(Subcommand, Debug)]
enum Commands {
    /// Run cross-validation on the dataset
    Validate {
        /// Number of folds for cross-validation
        #[arg(short, long, default_value_t = 5)]
        folds: u32,
    },
    /// Generate synthetic test data
    Generate {
        /// Number of positions to generate
        #[arg(short, long, default_value_t = 1000)]
        count: usize,
        /// Output file for synthetic data
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },
    /// Benchmark optimization algorithms
    Benchmark {
        /// Number of iterations for benchmarking
        #[arg(short, long, default_value_t = 100)]
        iterations: u32,
    },
}

/// Main function for the tuning binary
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging if verbose mode is enabled
    if cli.verbose {
        println!("Starting Shogi Engine Tuning Tool");
        println!("Dataset: {:?}", cli.dataset);
        println!("Output: {:?}", cli.output);
        println!("Method: {}", cli.method);
        println!("Iterations: {}", cli.iterations);
    }

    // Handle subcommands
    if let Some(ref command) = cli.command {
        match command {
            Commands::Validate { folds } => {
                run_validation(&cli, *folds)?;
            }
            Commands::Generate { count, output } => {
                generate_synthetic_data(*count, output.clone())?;
            }
            Commands::Benchmark { iterations } => {
                run_benchmark(&cli, *iterations)?;
            }
        }
        return Ok(());
    }

    // Run main tuning process
    run_tuning(&cli)?;

    Ok(())
}

/// Run the main tuning process
fn run_tuning(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    if cli.verbose {
        println!("Loading dataset...");
    }

    // Create tuning configuration
    let config = create_tuning_config(cli)?;

    // Load and process dataset
    let data_processor = DataProcessor::new(config.position_filter.clone());
    let positions = load_dataset(&cli.dataset, &data_processor)?;

    if cli.verbose {
        println!("Loaded {} training positions", positions.len());
    }

    if positions.is_empty() {
        return Err("No training positions found in dataset".into());
    }

    // Create optimizer
    let optimizer = Optimizer::new(config.optimization_method.clone());

    if cli.verbose {
        println!("Starting optimization with {} method...", cli.method);
    }

    // Run optimization
    let optimization_result = optimizer.optimize(&positions)?;

    if cli.verbose {
        println!("Optimization completed in {:.2} seconds", 
                optimization_result.optimization_time.as_secs_f64());
        println!("Final error: {:.6}", optimization_result.final_error);
        println!("Iterations: {}", optimization_result.iterations);
    }

    // Run validation
    if cli.verbose {
        println!("Running cross-validation...");
    }

    let validator = Validator::new(config.validation_config.clone());
    let validation_results = validator.cross_validate(&positions);

    if cli.verbose {
        println!("Validation completed");
        println!("Mean validation error: {:.6}", validation_results.mean_error);
        println!("Standard deviation: {:.6}", validation_results.std_error);
    }

    // Create tuning results
    let tuning_results = TuningResults::new(
        optimization_result.optimized_weights,
        validation_results.clone(),
        config,
        start_time.elapsed().as_secs_f64(),
        optimization_result.iterations,
        optimization_result.final_error,
        matches!(optimization_result.convergence_reason, shogi_engine::tuning::optimizer::ConvergenceReason::Converged),
    );

    // Save results
    save_results(&cli.output, &tuning_results)?;

    if cli.verbose {
        println!("Results saved to {:?}", cli.output);
    }

    println!("Tuning completed successfully!");
    println!("Total time: {:.2} seconds", start_time.elapsed().as_secs_f64());
    println!("Final error: {:.6}", optimization_result.final_error);
    println!("Validation error: {:.6}", validation_results.mean_error);

    Ok(())
}

/// Run validation-only mode
fn run_validation(cli: &Cli, folds: u32) -> Result<(), Box<dyn std::error::Error>> {
    if cli.verbose {
        println!("Running validation with {} folds...", folds);
    }

    let config = create_tuning_config(cli)?;
    let data_processor = DataProcessor::new(config.position_filter.clone());
    let positions = load_dataset(&cli.dataset, &data_processor)?;

    let validator = Validator::new(config.validation_config.clone());
    let validation_results = validator.cross_validate(&positions);

    println!("Validation Results:");
    println!("  Mean error: {:.6}", validation_results.mean_error);
    println!("  Standard deviation: {:.6}", validation_results.std_error);
    println!("  Best fold: {:?}", validation_results.best_fold);
    println!("  Worst fold: {:?}", validation_results.worst_fold);

    if cli.verbose {
        println!("  Fold details:");
        for fold in &validation_results.fold_results {
            println!("    Fold {}: error={:.6}, samples={}", 
                    fold.fold_number, fold.validation_error, fold.sample_count);
        }
    }

    Ok(())
}

/// Generate synthetic test data
fn generate_synthetic_data(count: usize, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use shogi_engine::tuning::validator::SyntheticDataGenerator;
    use shogi_engine::types::NUM_EVAL_FEATURES;
    use std::fs::File;
    use serde_json;

    if count == 0 {
        return Err("Count must be greater than 0".into());
    }

    println!("Generating {} synthetic training positions...", count);

    let generator = SyntheticDataGenerator::new(NUM_EVAL_FEATURES, 42);
    let positions = generator.generate_positions(count);

    let file = File::create(&output)?;
    serde_json::to_writer_pretty(file, &positions)?;

    println!("Synthetic data saved to {:?}", output);

    Ok(())
}

/// Run optimization algorithm benchmark
fn run_benchmark(_cli: &Cli, iterations: u32) -> Result<(), Box<dyn std::error::Error>> {
    use shogi_engine::tuning::validator::{SyntheticDataGenerator, PerformanceBenchmark};
    use shogi_engine::types::NUM_EVAL_FEATURES;

    println!("Running optimization benchmark...");

    // Generate synthetic data for benchmarking
    let generator = SyntheticDataGenerator::new(NUM_EVAL_FEATURES, 42);
    let positions = generator.generate_positions(1000);

    let mut benchmark = PerformanceBenchmark::new();

    // Test different optimization methods
    let methods = [
        ("Gradient Descent", OptimizationMethod::GradientDescent { learning_rate: 0.01 }),
        ("Adam", OptimizationMethod::Adam { learning_rate: 0.001, beta1: 0.9, beta2: 0.999, epsilon: 1e-8 }),
        ("LBFGS", OptimizationMethod::LBFGS { memory_size: 10, max_iterations: iterations as usize }),
        ("Genetic Algorithm", OptimizationMethod::GeneticAlgorithm { population_size: 50, mutation_rate: 0.1, crossover_rate: 0.8, max_generations: iterations as usize }),
    ];

    for (name, method) in methods.iter() {
        println!("Benchmarking {}...", name);
        
        let start_time = Instant::now();
        let optimizer = Optimizer::new(method.clone());
        
        match optimizer.optimize(&positions) {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                benchmark.record_timing(name, elapsed.as_secs_f64());
                
                println!("  {}: {:.3}s, error: {:.6}", name, elapsed.as_secs_f64(), result.final_error);
            }
            Err(e) => {
                println!("  {}: Failed - {}", name, e);
            }
        }
    }

    println!("\nBenchmark Report:");
    println!("{}", benchmark.generate_report());

    Ok(())
}

/// Create tuning configuration from CLI arguments
fn create_tuning_config(cli: &Cli) -> Result<TuningConfig, Box<dyn std::error::Error>> {
    // Parse optimization method
    let optimization_method = match cli.method.as_str() {
        "gradient-descent" => OptimizationMethod::GradientDescent { learning_rate: 0.01 },
        "adam" => OptimizationMethod::Adam { 
            learning_rate: 0.001, 
            beta1: 0.9, 
            beta2: 0.999, 
            epsilon: 1e-8 
        },
        "lbfgs" => OptimizationMethod::LBFGS { 
            memory_size: 10, 
            max_iterations: cli.iterations as usize 
        },
        "genetic" => OptimizationMethod::GeneticAlgorithm { 
            population_size: 50, 
            mutation_rate: 0.1, 
            crossover_rate: 0.8, 
            max_generations: cli.iterations as usize 
        },
        _ => return Err(format!("Unknown optimization method: {}", cli.method).into()),
    };

    // Create position filter
    let position_filter = PositionFilter {
        quiet_move_threshold: cli.quiet_threshold,
        min_rating: Some(cli.min_rating as u16),
        max_rating: None,
        min_move_number: 10,
        max_move_number: 1000,
        max_positions_per_game: Some(100),
        quiet_only: true,
        high_rated_only: false,
    };

    // Create validation config
    let validation_config = ValidationConfig {
        k_fold: cli.k_fold,
        test_split: cli.test_split,
        validation_split: 0.2,
        stratified: false,
        random_seed: Some(42),
    };

    // Create performance config
    let performance_config = PerformanceConfig {
        memory_limit_mb: 1024,
        thread_count: num_cpus::get(),
        checkpoint_frequency: 100,
        enable_logging: cli.progress,
        max_batch_size: 1000,
    };

    Ok(TuningConfig {
        dataset_path: cli.dataset.to_string_lossy().to_string(),
        output_path: cli.output.to_string_lossy().to_string(),
        checkpoint_path: Some("checkpoints/".to_string()),
        optimization_method,
        max_iterations: cli.iterations as usize,
        convergence_threshold: 1e-6,
        position_filter,
        validation_config,
        performance_config,
    })
}

/// Load dataset from file
fn load_dataset(
    dataset_path: &PathBuf, 
    _data_processor: &DataProcessor
) -> Result<Vec<shogi_engine::tuning::types::TrainingPosition>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(dataset_path)?;
    let reader = BufReader::new(file);

    // Try to determine file format from extension
    let extension = dataset_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "json" => {
            // Load as JSON format
            let positions: Vec<shogi_engine::tuning::types::TrainingPosition> = 
                serde_json::from_reader(reader)?;
            Ok(positions)
        }
        "kif" | "csa" | "pgn" => {
            // For now, return an error for unsupported formats
            // In a real implementation, these would be parsed
            Err(format!("Format '{}' not yet supported", extension).into())
        }
        _ => {
            Err(format!("Unknown file format: {}", extension).into())
        }
    }
}

/// Save tuning results to file
fn save_results(
    output_path: &PathBuf, 
    results: &TuningResults
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use serde_json;

    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, results)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "tuner",
            "--dataset", "test.json",
            "--output", "weights.json",
            "--method", "adam",
            "--iterations", "100",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.dataset, PathBuf::from("test.json"));
        assert_eq!(cli.output, PathBuf::from("weights.json"));
        assert_eq!(cli.method, "adam");
        assert_eq!(cli.iterations, 100);
    }

    #[test]
    fn test_config_creation() {
        let cli = Cli {
            dataset: PathBuf::from("test.json"),
            output: PathBuf::from("weights.json"),
            method: "adam".to_string(),
            iterations: 100,
            k_fold: 5,
            test_split: 0.2,
            regularization: 0.01,
            quiet_threshold: 3,
            min_rating: 1800,
            verbose: false,
            progress: false,
            command: None,
        };

        let config = create_tuning_config(&cli).unwrap();
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.position_filter.min_rating, Some(1800));
        assert_eq!(config.validation_config.k_fold, 5);
    }

    #[test]
    fn test_validation_command() {
        let args = vec![
            "tuner",
            "--dataset", "test.json",
            "--output", "weights.json",
            "validate",
            "--folds", "3",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Some(Commands::Validate { folds: 3 })));
    }

    #[test]
    fn test_generate_command() {
        let args = vec![
            "tuner",
            "--dataset", "test.json",
            "--output", "weights.json",
            "generate",
            "--count", "500",
            "--output", "synthetic.json",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Generate { count, output }) => {
                assert_eq!(count, 500);
                assert_eq!(output, PathBuf::from("synthetic.json"));
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_benchmark_command() {
        let args = vec![
            "tuner",
            "--dataset", "test.json",
            "--output", "weights.json",
            "benchmark",
            "--iterations", "50",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Some(Commands::Benchmark { iterations: 50 })));
    }

    #[test]
    fn test_synthetic_data_generation() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("synthetic.json");

        // Test synthetic data generation
        let result = generate_synthetic_data(10, output_path.clone());
        // Should succeed now that we have implementation
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_config_defaults() {
        let cli = Cli {
            dataset: PathBuf::from("test.json"),
            output: PathBuf::from("weights.json"),
            method: "adam".to_string(),
            iterations: 1000,
            k_fold: 5,
            test_split: 0.2,
            regularization: 0.01,
            quiet_threshold: 3,
            min_rating: 1800,
            verbose: false,
            progress: false,
            command: None,
        };

        let config = create_tuning_config(&cli).unwrap();
        
        // Test default values
        assert_eq!(config.validation_config.validation_split, 0.2);
        assert_eq!(config.validation_config.stratified, false);
        assert_eq!(config.validation_config.random_seed, Some(42));
        
        // Test position filter defaults
        assert_eq!(config.position_filter.min_rating, Some(1800));
        assert_eq!(config.position_filter.min_move_number, 10);
        assert_eq!(config.position_filter.max_positions_per_game, Some(100));
        assert_eq!(config.position_filter.quiet_only, true);
    }
}
