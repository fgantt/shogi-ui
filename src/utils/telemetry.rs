//! Telemetry utilities: thin, centralized surface for debug/trace logging.
//!
//! This re-exports existing debug utilities behind a stable path and can be
//! extended with higher-level helpers without touching call sites.

pub use crate::debug_utils::{is_debug_enabled, set_debug_enabled, trace_log, debug_log};

/// Convenience helper to trace with lazy formatting.
#[inline]
pub fn tracef<F: FnOnce() -> String>(feature: &str, msg: F) {
    if is_debug_enabled() {
        trace_log(feature, &msg());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_no_panic_when_disabled() {
        set_debug_enabled(false);
        tracef("TEST", || "ok".to_string());
        debug_log("ok");
    }
}


