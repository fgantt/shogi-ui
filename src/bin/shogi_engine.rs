// Standalone USI-compliant Shogi Engine Binary
// This binary wraps the existing ShogiEngine with a USI protocol handler
// for communication via stdin/stdout.
//
// Usage:
//   shogi-engine
//
// The engine will read USI commands from stdin and write responses to stdout.
// It communicates using the Universal Shogi Interface (USI) protocol.

use shogi_engine::usi::run_usi_loop;

fn main() {
    // Initialize logging for debugging purposes (stderr only, stdout is for USI)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stderr)
        .init();
    
    log::info!("Shogi Engine starting...");
    log::info!("USI protocol handler initialized");
    
    // Run the USI command loop
    // This reads commands from stdin and writes responses to stdout
    run_usi_loop();
    
    log::info!("Shogi Engine shutting down");
}

