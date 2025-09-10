// Debug logging utilities that work in both WASM and standalone environments

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Debug logging that works in both WASM and standalone environments
pub fn debug_log(message: &str) {
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
