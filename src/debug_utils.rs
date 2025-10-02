// Debug logging utilities that work in both WASM and standalone environments

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::time::Duration;
use std::sync::Mutex;
use std::collections::HashMap;

// Global debug flag - set to true to enable debug logging by default
static DEBUG_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

// Global timing context for tracking function execution times
lazy_static::lazy_static! {
    static ref TIMING_CONTEXT: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
    static ref SEARCH_START_TIME: Mutex<Option<f64>> = Mutex::new(None);
}

/// Get current time in milliseconds (WASM-compatible)
fn get_current_time_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::js_sys::Date;
        Date::new_0().get_time()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64
    }
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
                trace_log(feature, &format!("{} completed in {}ms (total: {}ms)", 
                    key, elapsed_ms, search_elapsed));
            }
        }
    }
}

/// Log with feature context and timing information
pub fn trace_log(feature: &str, message: &str) {
    if !is_debug_enabled() {
        return;
    }
    
    let search_elapsed = get_search_elapsed_ms();
    let formatted_message = format!("[{}] [{}ms] {}", feature, search_elapsed, message);
    
    // Use the existing debug_log function which already works in WASM
    debug_log(&formatted_message);
}

/// Debug logging that works in both WASM and standalone environments
pub fn debug_log(message: &str) {
    if !is_debug_enabled() {
        return;
    }
    
    let search_elapsed = get_search_elapsed_ms();
    let formatted_message = format!("[{}ms] {}", search_elapsed, message);
    
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::console;
        console::log(&js_sys::Array::of1(&JsValue::from_str(&formatted_message)));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("DEBUG: {}", formatted_message);
    }
}

/// Log decision points with context
pub fn log_decision(feature: &str, decision: &str, reason: &str, value: Option<i32>) {
    if !is_debug_enabled() {
        return;
    }
    
    let value_str = if let Some(v) = value {
        format!(" (value: {})", v)
    } else {
        String::new()
    };
    
    trace_log(feature, &format!("DECISION: {} - {} {}", decision, reason, value_str));
}

/// Log move evaluation with context
pub fn log_move_eval(feature: &str, move_str: &str, score: i32, reason: &str) {
    if !is_debug_enabled() {
        return;
    }
    
    trace_log(feature, &format!("MOVE_EVAL: {} -> {} ({})", move_str, score, reason));
}

/// Log search statistics
pub fn log_search_stats(feature: &str, depth: u8, nodes: u64, score: i32, pv: &str) {
    if !is_debug_enabled() {
        return;
    }
    
    trace_log(feature, &format!("SEARCH_STATS: depth={} nodes={} score={} pv={}", depth, nodes, score, pv));
}
