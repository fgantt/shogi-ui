// Debug logging utilities that work in both WASM and standalone environments

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Global debug flag - set to false to disable debug logging
static DEBUG_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Enable or disable debug logging
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Debug logging that works in both WASM and standalone environments
pub fn debug_log(message: &str) {
    if !is_debug_enabled() {
        return;
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::console;
        console::log(&js_sys::Array::of1(&JsValue::from_str(message)));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("DEBUG: {}", message);
    }
}
