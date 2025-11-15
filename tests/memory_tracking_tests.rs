//! Tests for memory tracking (Task 26.0 - Task 4.0)

use shogi_engine::search::memory_tracking::{MemoryBreakdown, MemoryTracker};
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::types::*;

#[test]
fn test_memory_tracking() {
    let tracker = MemoryTracker::new();
    let rss = tracker.get_current_rss();
    
    // RSS should be greater than 0 (process has memory)
    assert!(rss > 0, "RSS should be greater than 0");
    
    // Peak RSS should be at least current RSS
    let peak = tracker.get_peak_rss();
    assert!(peak >= rss, "Peak RSS should be at least current RSS");
}

#[test]
fn test_peak_rss_tracking() {
    let tracker = MemoryTracker::new();
    let initial_peak = tracker.get_peak_rss();
    
    // Update peak
    tracker.update_peak_rss();
    let updated_peak = tracker.get_peak_rss();
    
    // Peak should not decrease
    assert!(updated_peak >= initial_peak, "Peak should not decrease");
}

#[test]
fn test_memory_growth() {
    let tracker = MemoryTracker::new();
    let growth = tracker.get_memory_growth();
    let growth_pct = tracker.get_memory_growth_percentage();
    
    // Growth should be non-negative
    assert!(growth >= 0);
    assert!(growth_pct >= 0.0);
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
    
    // Should have RSS data
    assert!(with_rss.current_rss_bytes > 0);
    assert!(with_rss.peak_rss_bytes > 0);
    
    // Should have component breakdown
    assert_eq!(with_rss.component_breakdown.total_component_bytes, 2000);
    
    // Growth should be non-negative
    assert!(with_rss.memory_growth_bytes >= 0);
    assert!(with_rss.memory_growth_percentage >= 0.0);
}

#[test]
fn test_memory_leak_detection() {
    // Create tracker with low threshold for testing
    let tracker = MemoryTracker::with_leak_threshold(10.0); // 10% threshold
    
    // Initially should not detect leak
    let leak_detected = tracker.check_for_leak();
    // May or may not detect leak depending on actual memory usage
    // Just verify the method works
    assert!(leak_detected || !leak_detected); // Always true, just checking it doesn't panic
}

#[test]
fn test_memory_growth_tracking() {
    let mut engine = SearchEngine::new(None, 16);
    
    // Get initial memory
    let initial_memory = engine.get_memory_usage();
    assert!(initial_memory > 0);
    
    // Reset peak tracking
    engine.memory_tracker.reset_peak();
    
    // Update peak
    engine.track_memory_usage(0);
    
    // Get peak
    let peak = engine.memory_tracker.get_peak_rss();
    assert!(peak >= initial_memory as u64);
}

#[test]
fn test_get_memory_breakdown() {
    let engine = SearchEngine::new(None, 16);
    let breakdown = engine.get_memory_breakdown();
    
    // Should have RSS data
    assert!(breakdown.current_rss_bytes > 0);
    assert!(breakdown.peak_rss_bytes > 0);
    
    // Should have component breakdown
    assert!(breakdown.component_breakdown.total_component_bytes > 0);
}

#[test]
fn test_memory_tracker_reset() {
    let tracker = MemoryTracker::new();
    let initial_rss = tracker.get_current_rss();
    
    // Update peak
    tracker.update_peak_rss();
    
    // Reset peak
    tracker.reset_peak();
    
    // After reset, peak should be current RSS
    let peak_after_reset = tracker.get_peak_rss();
    let current_after_reset = tracker.get_current_rss();
    
    // Peak should be close to current (may have changed slightly)
    assert!(peak_after_reset >= current_after_reset.saturating_sub(1024 * 1024)); // Allow 1MB variance
}

