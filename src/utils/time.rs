//! Time utilities: re-exports common time sources and provides convenience helpers.

pub use crate::time_utils::TimeSource;

use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

/// Returns current time in milliseconds since UNIX epoch.
#[inline]
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

/// Simple stopwatch for scoped timing in tests or lightweight measurements.
#[derive(Debug)]
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    #[inline]
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopwatch_measures_time() {
        let sw = Stopwatch::start();
        std::thread::sleep(Duration::from_millis(5));
        assert!(sw.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn current_time_ms_monotonic_nonzero() {
        let a = current_time_ms();
        let b = current_time_ms();
        assert!(b >= a);
        assert!(a > 0);
    }
}


