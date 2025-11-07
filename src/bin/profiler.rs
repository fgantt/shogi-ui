//! Performance Profiler Utility
//!
//! A command-line tool for profiling the shogi engine's performance.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use shogi_engine::ShogiEngine;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

// Access global search metrics exposed by the engine
use shogi_engine::search::search_engine::{
    snapshot_and_reset_metrics as snapshot_tt_ybwc_metrics, GLOBAL_NODES_SEARCHED,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "profiler")]
#[command(about = "Profile engine performance, cache efficiency, and search metrics")]
struct Cli {
    /// SFEN position string; if omitted, uses starting position
    #[arg(short, long)]
    position: Option<String>,

    /// Search depth
    #[arg(short, long, default_value_t = 8)]
    depth: u8,

    /// Time limit in milliseconds
    #[arg(short = 't', long, default_value_t = 5000)]
    time_limit: u32,

    /// Output profile JSON file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compare two profiler JSON reports and print a summary
    Compare {
        /// Path to first profile JSON (alias: --config1 to match docs)
        #[arg(long, alias = "config1", value_name = "FILE")]
        file1: PathBuf,
        /// Path to second profile JSON (alias: --config2 to match docs)
        #[arg(long, alias = "config2", value_name = "FILE")]
        file2: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ProfilerReport {
    // Inputs
    position: String,
    depth: u8,
    time_ms: u64,

    // Core performance
    nodes: u64,
    nps: u64,

    // TT/YBWC metrics (snapshot)
    tt_try_reads: u64,
    tt_try_read_successes: u64,
    tt_try_read_fails: u64,
    tt_try_writes: u64,
    tt_try_write_successes: u64,
    tt_try_write_fails: u64,
    ybwc_sibling_batches: u64,
    ybwc_siblings_evaluated: u64,
    ybwc_trigger_opportunities: u64,
    ybwc_trigger_eligible_depth: u64,
    ybwc_trigger_eligible_branch: u64,
    ybwc_triggered: u64,

    // Derived
    cache_hit_rate: f64,

    // Suggestions
    recommendations: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Compare { file1, file2 }) => {
            compare_reports(file1, file2)?;
            return Ok(());
        }
        None => {}
    }

    if cli.verbose {
        println!("Shogi Engine Performance Profiler");
        println!("================================");
    }

    // Prepare engine
    let mut engine = ShogiEngine::new();

    // Note: SFEN parsing is not implemented yet in engine utilities.
    // We keep API parity with other tools: accept --position but use startpos for now.
    let position = cli
        .position
        .clone()
        .unwrap_or_else(|| "startpos".to_string());

    // Reset global counters before profiling
    GLOBAL_NODES_SEARCHED.store(0, std::sync::atomic::Ordering::Relaxed);
    let _ = snapshot_tt_ybwc_metrics(); // clear previous snapshot state

    if cli.verbose {
        println!("Profiling position: {}", position);
        println!("Depth: {}", cli.depth);
        println!("Time limit: {}ms", cli.time_limit);
    }

    let start = Instant::now();
    // Execute a single best-move search; this drives the metrics
    let _ = engine.get_best_move(cli.depth, cli.time_limit, None);
    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Capture metrics
    let nodes = GLOBAL_NODES_SEARCHED.load(std::sync::atomic::Ordering::Relaxed);
    let nps = if elapsed_ms > 0 {
        nodes.saturating_mul(1000) / elapsed_ms
    } else {
        0
    };
    let m = snapshot_tt_ybwc_metrics();

    let cache_hit_rate = if m.tt_try_reads > 0 {
        m.tt_try_read_successes as f64 / m.tt_try_reads as f64
    } else {
        0.0
    };

    let mut recommendations: Vec<String> = Vec::new();
    if cache_hit_rate < 0.20 {
        recommendations
            .push("Low TT hit rate detected (<20%). Consider increasing hash size.".to_string());
    }
    if nps < 50_000 {
        recommendations
            .push("Low nodes-per-second. Reduce depth or adjust pruning parameters.".to_string());
    }
    if m.ybwc_trigger_opportunities > 0 && m.ybwc_triggered == 0 {
        recommendations.push(
            "YBWC opportunities observed but not triggered; review YBWC thresholds.".to_string(),
        );
    }

    let report = ProfilerReport {
        position,
        depth: cli.depth,
        time_ms: elapsed_ms,
        nodes,
        nps,
        tt_try_reads: m.tt_try_reads,
        tt_try_read_successes: m.tt_try_read_successes,
        tt_try_read_fails: m.tt_try_read_fails,
        tt_try_writes: m.tt_try_writes,
        tt_try_write_successes: m.tt_try_write_successes,
        tt_try_write_fails: m.tt_try_write_fails,
        ybwc_sibling_batches: m.ybwc_sibling_batches,
        ybwc_siblings_evaluated: m.ybwc_siblings_evaluated,
        ybwc_trigger_opportunities: m.ybwc_trigger_opportunities,
        ybwc_trigger_eligible_depth: m.ybwc_trigger_eligible_depth,
        ybwc_trigger_eligible_branch: m.ybwc_trigger_eligible_branch,
        ybwc_triggered: m.ybwc_triggered,
        cache_hit_rate,
        recommendations,
    };

    // Output
    if cli.verbose {
        println!("\n=== Profile Summary ===");
        println!("Time: {} ms", report.time_ms);
        println!("Nodes: {}", report.nodes);
        println!("NPS: {}", report.nps);
        println!("TT hit rate: {:.2}%", report.cache_hit_rate * 100.0);
        if !report.recommendations.is_empty() {
            println!("Recommendations:");
            for r in &report.recommendations {
                println!("- {}", r);
            }
        }
    }

    if let Some(path) = cli.output {
        let mut file = File::create(&path)?;
        let json = serde_json::to_string_pretty(&report)?;
        file.write_all(json.as_bytes())?;
        if cli.verbose {
            println!("Saved profile to {}", path.display());
        }
    }

    Ok(())
}

fn compare_reports(file1: &PathBuf, file2: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let r1: ProfilerReport = serde_json::from_slice(&std::fs::read(file1)?)?;
    let r2: ProfilerReport = serde_json::from_slice(&std::fs::read(file2)?)?;

    println!(
        "Comparing profiles:\n  A: {}\n  B: {}",
        file1.display(),
        file2.display()
    );
    println!("================================================");
    println!(
        "Time (ms):       {:>10}  -> {:>10}  ({:+})",
        r1.time_ms,
        r2.time_ms,
        (r2.time_ms as i64 - r1.time_ms as i64)
    );
    println!(
        "Nodes:           {:>10}  -> {:>10}  ({:+})",
        r1.nodes,
        r2.nodes,
        (r2.nodes as i64 - r1.nodes as i64)
    );
    println!(
        "NPS:             {:>10}  -> {:>10}  ({:+})",
        r1.nps,
        r2.nps,
        (r2.nps as i64 - r1.nps as i64)
    );
    println!(
        "TT hit rate (%): {:>10.2} -> {:>10.2} ({:+.2})",
        r1.cache_hit_rate * 100.0,
        r2.cache_hit_rate * 100.0,
        (r2.cache_hit_rate - r1.cache_hit_rate) * 100.0
    );

    Ok(())
}
