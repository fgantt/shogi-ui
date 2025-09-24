// Time utilities that work in both WASM and standalone environments



/// A time source that works in both WASM and standalone environments
pub struct TimeSource {
    #[cfg(target_arch = "wasm32")]
    start_time: f64,
    #[cfg(not(target_arch = "wasm32"))]
    start_time: std::time::Instant,
}

impl TimeSource {
    /// Create a new time source with the current time
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                start_time: js_sys::Date::now(),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                start_time: std::time::Instant::now(),
            }
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u32 {
        #[cfg(target_arch = "wasm32")]
        {
            (js_sys::Date::now() - self.start_time) as u32
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start_time.elapsed().as_millis() as u32
        }
    }

    /// Check if the time limit has been exceeded
    pub fn has_exceeded_limit(&self, time_limit_ms: u32) -> bool {
        self.elapsed_ms() >= time_limit_ms
    }
}

/// Get current time in milliseconds (for compatibility with existing code)
pub fn current_time_ms() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u32
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32
    }
}
