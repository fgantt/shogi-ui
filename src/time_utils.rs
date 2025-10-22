// Time utilities for standalone environments

/// A time source for standalone environments
pub struct TimeSource {
    start_time: std::time::Instant,
}

impl TimeSource {
    /// Create a new time source with the current time
    pub fn now() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u32 {
        self.start_time.elapsed().as_millis() as u32
    }

    /// Check if the time limit has been exceeded
    pub fn has_exceeded_limit(&self, time_limit_ms: u32) -> bool {
        self.elapsed_ms() >= time_limit_ms
    }
}

/// Get current time in milliseconds (for compatibility with existing code)
pub fn current_time_ms() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32
}
