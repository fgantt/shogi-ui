// Debug logging utilities for standalone environments

use std::collections::HashMap;
use std::sync::Mutex;

// Global debug flag - set to false to disable debug logging by default
// Can be enabled via the USI "debug on" command
static DEBUG_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// Macros for lightweight debug logging that check runtime flag BEFORE string formatting
// This prevents expensive string formatting when debug is disabled
// When verbose-debug feature is disabled, these macros expand to nothing (zero overhead)

/// Lightweight trace logging macro - checks runtime flag before string formatting
/// Usage: trace_log_fast!("FEATURE", &format!(...))
/// The string formatting is only evaluated if debug is enabled
#[macro_export]
#[cfg(feature = "verbose-debug")]
macro_rules! trace_log_fast {
    ($feature:expr, $message:expr) => {
        // Check runtime flag first - this prevents string formatting when disabled
        if crate::debug_utils::is_debug_enabled() {
            crate::debug_utils::trace_log($feature, $message);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "verbose-debug"))]
macro_rules! trace_log_fast {
    ($feature:expr, $message:expr) => {
        // Compile out completely when verbose-debug feature is disabled (zero overhead)
    };
}

/// Lightweight debug logging macro - checks runtime flag before string formatting
#[macro_export]
#[cfg(feature = "verbose-debug")]
macro_rules! debug_log_fast {
    ($message:expr) => {
        if crate::debug_utils::is_debug_enabled() {
            crate::debug_utils::debug_log($message);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "verbose-debug"))]
macro_rules! debug_log_fast {
    ($message:expr) => {
        // Compile out completely when verbose-debug feature is disabled (zero overhead)
    };
}

/// Lightweight decision logging macro - checks runtime flag before string formatting
#[macro_export]
#[cfg(feature = "verbose-debug")]
macro_rules! log_decision_fast {
    ($feature:expr, $decision:expr, $reason:expr, $value:expr) => {
        if crate::debug_utils::is_debug_enabled() {
            crate::debug_utils::log_decision($feature, $decision, $reason, $value);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "verbose-debug"))]
macro_rules! log_decision_fast {
    ($feature:expr, $decision:expr, $reason:expr, $value:expr) => {
        // Compile out completely when verbose-debug feature is disabled (zero overhead)
    };
}

/// Lightweight move evaluation logging macro
#[macro_export]
#[cfg(feature = "verbose-debug")]
macro_rules! log_move_eval_fast {
    ($feature:expr, $move_str:expr, $score:expr, $reason:expr) => {
        if crate::debug_utils::is_debug_enabled() {
            crate::debug_utils::log_move_eval($feature, $move_str, $score, $reason);
        }
    };
}

#[macro_export]
#[cfg(not(feature = "verbose-debug"))]
macro_rules! log_move_eval_fast {
    ($feature:expr, $move_str:expr, $score:expr, $reason:expr) => {
        // Compile out completely when verbose-debug feature is disabled (zero overhead)
    };
}

// Note: The macros above still evaluate their arguments (including format! strings)
// even when debug is disabled, because Rust evaluates macro arguments eagerly.
// For maximum performance, use lazy evaluation with closures:
// trace_log_fast!("FEATURE", || format!(...))

/// Lightweight trace logging with lazy evaluation - only formats string if debug is enabled
/// Usage: trace_log_lazy!("FEATURE", || format!(...))
#[macro_export]
#[cfg(feature = "verbose-debug")]
macro_rules! trace_log_lazy {
    ($feature:expr, $message_fn:expr) => {
        if crate::debug_utils::is_debug_enabled() {
            crate::debug_utils::trace_log($feature, &($message_fn)());
        }
    };
}

#[macro_export]
#[cfg(not(feature = "verbose-debug"))]
macro_rules! trace_log_lazy {
    ($feature:expr, $message_fn:expr) => {
        // Compile out completely when verbose-debug feature is disabled (zero overhead)
    };
}

// Global timing context for tracking function execution times
lazy_static::lazy_static! {
    static ref TIMING_CONTEXT: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
    static ref SEARCH_START_TIME: Mutex<Option<f64>> = Mutex::new(None);
}

/// Get current time in milliseconds
fn get_current_time_ms() -> f64 {
    // Delegate to centralized utils::time for consistency
    crate::utils::time::current_time_ms() as f64
}

/// Enable or disable debug logging
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Set the start time for the entire search operation
pub fn set_search_start_time() {
    if is_debug_enabled() {
        if let Ok(mut start_time) = SEARCH_START_TIME.lock() {
            *start_time = Some(get_current_time_ms());
        }
    }
}

/// Get elapsed time since search start
pub fn get_search_elapsed_ms() -> u64 {
    if let Ok(start_time) = SEARCH_START_TIME.lock() {
        if let Some(start) = *start_time {
            (get_current_time_ms() - start) as u64
        } else {
            0
        }
    } else {
        0
    }
}

/// Start timing a specific function or feature
pub fn start_timing(key: &str) {
    if is_debug_enabled() {
        if let Ok(mut context) = TIMING_CONTEXT.lock() {
            context.insert(key.to_string(), get_current_time_ms());
        }
    }
}

/// End timing and log the duration
pub fn end_timing(key: &str, feature: &str) {
    if is_debug_enabled() {
        if let Ok(mut context) = TIMING_CONTEXT.lock() {
            if let Some(start_time) = context.remove(key) {
                let elapsed_ms = (get_current_time_ms() - start_time) as u64;
                let search_elapsed = get_search_elapsed_ms();
                trace_log(
                    feature,
                    &format!(
                        "{} completed in {}ms (total: {}ms)",
                        key, elapsed_ms, search_elapsed
                    ),
                );
            }
        }
    }
}

/// Log with feature context and timing information
/// Optimized: checks debug flag first to avoid unnecessary string formatting
#[cfg(feature = "verbose-debug")]
#[inline]
pub fn trace_log(feature: &str, message: &str) {
    if !is_debug_enabled() {
        return;
    }

    let search_elapsed = get_search_elapsed_ms();
    let formatted_message = format!("[{}] [{}ms] {}", feature, search_elapsed, message);

    debug_log(&formatted_message);
}

#[cfg(not(feature = "verbose-debug"))]
#[inline]
pub fn trace_log(_: &str, _: &str) {}

/// Debug logging for standalone environments
/// Optimized: checks debug flag first to avoid unnecessary string formatting
#[cfg(feature = "verbose-debug")]
#[inline]
pub fn debug_log(message: &str) {
    if !is_debug_enabled() {
        return;
    }

    let search_elapsed = get_search_elapsed_ms();
    let formatted_message = format!("[{}ms] {}", search_elapsed, message);

    eprintln!("DEBUG: {}", formatted_message);
}

#[cfg(not(feature = "verbose-debug"))]
#[inline]
pub fn debug_log(_: &str) {}

/// Log decision points with context
/// Optimized: checks debug flag first to avoid unnecessary string formatting
#[cfg(feature = "verbose-debug")]
#[inline]
pub fn log_decision(feature: &str, decision: &str, reason: &str, value: Option<i32>) {
    if !is_debug_enabled() {
        return;
    }

    let value_str = if let Some(v) = value {
        format!(" (value: {})", v)
    } else {
        String::new()
    };

    trace_log(
        feature,
        &format!("DECISION: {} - {} {}", decision, reason, value_str),
    );
}

#[cfg(not(feature = "verbose-debug"))]
#[inline]
pub fn log_decision(_: &str, _: &str, _: &str, _: Option<i32>) {}

/// Log move evaluation with context
/// Optimized: checks debug flag first to avoid unnecessary string formatting
#[cfg(feature = "verbose-debug")]
#[inline]
pub fn log_move_eval(feature: &str, move_str: &str, score: i32, reason: &str) {
    if !is_debug_enabled() {
        return;
    }

    trace_log(
        feature,
        &format!("MOVE_EVAL: {} -> {} ({})", move_str, score, reason),
    );
}

#[cfg(not(feature = "verbose-debug"))]
#[inline]
pub fn log_move_eval(_: &str, _: &str, _: i32, _: &str) {}

/// Log search statistics
/// Optimized: checks debug flag first to avoid unnecessary string formatting
#[cfg(feature = "verbose-debug")]
#[inline]
pub fn log_search_stats(feature: &str, depth: u8, nodes: u64, score: i32, pv: &str) {
    if !is_debug_enabled() {
        return;
    }

    trace_log(
        feature,
        &format!(
            "SEARCH_STATS: depth={} nodes={} score={} pv={}",
            depth, nodes, score, pv
        ),
    );
}

#[cfg(not(feature = "verbose-debug"))]
#[inline]
pub fn log_search_stats(_: &str, _: u8, _: u64, _: i32, _: &str) {}
