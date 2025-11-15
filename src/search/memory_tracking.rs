//! Memory Usage Tracking Module (Task 26.0 - Task 4.0)
//!
//! This module provides actual RSS (Resident Set Size) memory tracking
//! using the sysinfo crate for cross-platform support.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::{Pid, System, SystemExt, ProcessExt};

/// Memory tracker for actual RSS tracking (Task 26.0 - Task 4.0)
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    /// System instance for memory queries
    system: Arc<Mutex<System>>,
    /// Peak RSS in bytes
    peak_rss_bytes: Arc<Mutex<u64>>,
    /// Initial RSS in bytes (for growth tracking)
    initial_rss_bytes: Arc<Mutex<u64>>,
    /// Memory leak detection threshold (percentage)
    leak_threshold_percent: f64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        let mut system = System::new();
        let pid = Pid::from(std::process::id() as usize);
        system.refresh_process(pid);

        let current_rss = Self::get_current_rss_internal(&system, pid);
        let initial_rss = current_rss;

        Self {
            system: Arc::new(Mutex::new(system)),
            peak_rss_bytes: Arc::new(Mutex::new(current_rss)),
            initial_rss_bytes: Arc::new(Mutex::new(initial_rss)),
            leak_threshold_percent: 50.0, // Default: 50% growth threshold
        }
    }

    /// Create a new memory tracker with custom leak threshold
    pub fn with_leak_threshold(leak_threshold_percent: f64) -> Self {
        let mut tracker = Self::new();
        tracker.leak_threshold_percent = leak_threshold_percent;
        tracker
    }

    /// Get current RSS in bytes (Task 4.0)
    pub fn get_current_rss(&self) -> u64 {
        let mut system = self.system.lock().unwrap();
        let pid = Pid::from(std::process::id() as usize);
        system.refresh_process(pid);
        Self::get_current_rss_internal(&system, pid)
    }

    /// Internal helper to get RSS from system
    fn get_current_rss_internal(system: &System, pid: Pid) -> u64 {
        if let Some(process) = system.process(pid) {
            process.memory() * 1024 // sysinfo returns KB, convert to bytes
        } else {
            0
        }
    }

    /// Get peak RSS in bytes (Task 4.0)
    pub fn get_peak_rss(&self) -> u64 {
        *self.peak_rss_bytes.lock().unwrap()
    }

    /// Update peak RSS if current RSS is higher
    pub fn update_peak_rss(&self) {
        let current_rss = self.get_current_rss();
        let mut peak = self.peak_rss_bytes.lock().unwrap();
        if current_rss > *peak {
            *peak = current_rss;
        }
    }

    /// Get memory growth since initialization in bytes
    pub fn get_memory_growth(&self) -> u64 {
        let current_rss = self.get_current_rss();
        let initial_rss = *self.initial_rss_bytes.lock().unwrap();
        current_rss.saturating_sub(initial_rss)
    }

    /// Get memory growth percentage since initialization
    pub fn get_memory_growth_percentage(&self) -> f64 {
        let initial_rss = *self.initial_rss_bytes.lock().unwrap();
        if initial_rss == 0 {
            return 0.0;
        }
        let growth = self.get_memory_growth();
        (growth as f64 / initial_rss as f64) * 100.0
    }

    /// Check for memory leak (Task 4.0)
    /// Returns true if memory growth exceeds threshold
    pub fn check_for_leak(&self) -> bool {
        self.get_memory_growth_percentage() > self.leak_threshold_percent
    }

    /// Get memory breakdown combining RSS with component estimates (Task 4.0)
    pub fn get_memory_breakdown(&self, component_estimates: &MemoryBreakdown) -> MemoryBreakdownWithRSS {
        let current_rss = self.get_current_rss();
        let peak_rss = self.get_peak_rss();
        let growth = self.get_memory_growth();

        MemoryBreakdownWithRSS {
            current_rss_bytes: current_rss,
            peak_rss_bytes: peak_rss,
            memory_growth_bytes: growth,
            memory_growth_percentage: self.get_memory_growth_percentage(),
            component_breakdown: component_estimates.clone(),
            leak_detected: self.check_for_leak(),
        }
    }

    /// Reset peak RSS tracking
    pub fn reset_peak(&self) {
        let current_rss = self.get_current_rss();
        *self.peak_rss_bytes.lock().unwrap() = current_rss;
        *self.initial_rss_bytes.lock().unwrap() = current_rss;
    }

    /// Set leak detection threshold
    pub fn set_leak_threshold(&mut self, threshold_percent: f64) {
        self.leak_threshold_percent = threshold_percent;
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory breakdown by component (Task 4.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBreakdown {
    /// Transposition table memory in bytes
    pub tt_memory_bytes: u64,
    /// Cache memory in bytes
    pub cache_memory_bytes: u64,
    /// Move ordering memory in bytes
    pub move_ordering_memory_bytes: u64,
    /// Other component memory in bytes
    pub other_memory_bytes: u64,
    /// Total estimated component memory
    pub total_component_bytes: u64,
}

impl Default for MemoryBreakdown {
    fn default() -> Self {
        Self {
            tt_memory_bytes: 0,
            cache_memory_bytes: 0,
            move_ordering_memory_bytes: 0,
            other_memory_bytes: 0,
            total_component_bytes: 0,
        }
    }
}

impl MemoryBreakdown {
    /// Calculate total component bytes
    pub fn calculate_total(&mut self) {
        self.total_component_bytes = self.tt_memory_bytes
            + self.cache_memory_bytes
            + self.move_ordering_memory_bytes
            + self.other_memory_bytes;
    }
}

/// Memory breakdown with RSS tracking (Task 4.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBreakdownWithRSS {
    /// Current RSS in bytes
    pub current_rss_bytes: u64,
    /// Peak RSS in bytes
    pub peak_rss_bytes: u64,
    /// Memory growth since initialization in bytes
    pub memory_growth_bytes: u64,
    /// Memory growth percentage
    pub memory_growth_percentage: f64,
    /// Component-level breakdown
    pub component_breakdown: MemoryBreakdown,
    /// Whether memory leak was detected
    pub leak_detected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();
        let rss = tracker.get_current_rss();
        assert!(rss > 0, "RSS should be greater than 0");
    }

    #[test]
    fn test_peak_rss_tracking() {
        let tracker = MemoryTracker::new();
        let initial_peak = tracker.get_peak_rss();
        tracker.update_peak_rss();
        let updated_peak = tracker.get_peak_rss();
        assert!(updated_peak >= initial_peak, "Peak should not decrease");
    }

    #[test]
    fn test_memory_breakdown() {
        let tracker = MemoryTracker::new();
        let breakdown = MemoryBreakdown {
            tt_memory_bytes: 1000,
            cache_memory_bytes: 500,
            move_ordering_memory_bytes: 200,
            other_memory_bytes: 300,
            total_component_bytes: 2000,
        };

        let with_rss = tracker.get_memory_breakdown(&breakdown);
        assert!(with_rss.current_rss_bytes > 0);
        assert!(with_rss.peak_rss_bytes > 0);
        assert_eq!(with_rss.component_breakdown.total_component_bytes, 2000);
    }

    #[test]
    fn test_memory_growth() {
        let tracker = MemoryTracker::new();
        let growth = tracker.get_memory_growth();
        let growth_pct = tracker.get_memory_growth_percentage();
        assert!(growth >= 0);
        assert!(growth_pct >= 0.0);
    }
}

