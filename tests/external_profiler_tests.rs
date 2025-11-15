//! Tests for external profiler integration (Task 26.0 - Task 8.0)

use crate::search::performance_tuning::{ExternalProfiler, InstrumentsProfiler, PerfProfiler, ProfilerMarker, MarkerType};
use crate::search::search_engine::SearchEngine;
use crate::types::*;
use std::sync::Arc;

#[test]
fn test_external_profiler_markers() {
    let mut perf_profiler = PerfProfiler::new();
    perf_profiler.enable();
    
    let mut engine = SearchEngine::new(None, 16);
    engine.enable_external_profiling(Arc::new(perf_profiler));
    
    // Run a short search to generate markers
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Export markers
    let markers_result = engine.export_profiling_markers();
    assert!(markers_result.is_ok());
    
    let markers_json = markers_result.unwrap();
    assert!(markers_json.get("profiler").is_some());
    assert!(markers_json.get("markers").is_some());
    assert!(markers_json.get("total_markers").is_some());
    
    // Verify markers were created
    let total_markers = markers_json.get("total_markers").unwrap().as_u64().unwrap();
    assert!(total_markers > 0);
}

#[test]
fn test_perf_profiler_region_markers() {
    let mut profiler = PerfProfiler::new();
    profiler.enable();
    
    profiler.start_region("test_region");
    profiler.end_region("test_region");
    
    let markers = profiler.get_markers();
    assert_eq!(markers.len(), 2);
    assert_eq!(markers[0].name, "test_region");
    assert_eq!(markers[0].marker_type, MarkerType::RegionStart);
    assert_eq!(markers[1].name, "test_region");
    assert_eq!(markers[1].marker_type, MarkerType::RegionEnd);
}

#[test]
fn test_instruments_profiler_point_markers() {
    let mut profiler = InstrumentsProfiler::new();
    profiler.enable();
    
    profiler.mark("test_point");
    profiler.mark("another_point");
    
    let markers = profiler.get_markers();
    assert_eq!(markers.len(), 2);
    assert_eq!(markers[0].name, "test_point");
    assert_eq!(markers[0].marker_type, MarkerType::Point);
    assert_eq!(markers[1].name, "another_point");
    assert_eq!(markers[1].marker_type, MarkerType::Point);
}

#[test]
fn test_profiler_disabled() {
    let profiler = PerfProfiler::new();
    
    // Profiler is disabled by default
    assert!(!profiler.is_enabled());
    
    // Markers should not be created when disabled
    profiler.start_region("test");
    profiler.end_region("test");
    profiler.mark("point");
    
    let markers = profiler.get_markers();
    assert_eq!(markers.len(), 0);
}

#[test]
fn test_profiler_export_markers() {
    let mut profiler = PerfProfiler::new();
    profiler.enable();
    
    profiler.start_region("region1");
    profiler.mark("point1");
    profiler.end_region("region1");
    
    let export_result = profiler.export_markers();
    assert!(export_result.is_ok());
    
    let json = export_result.unwrap();
    assert_eq!(json.get("profiler").unwrap().as_str().unwrap(), "perf");
    assert_eq!(json.get("total_markers").unwrap().as_u64().unwrap(), 3);
    
    let markers_array = json.get("markers").unwrap().as_array().unwrap();
    assert_eq!(markers_array.len(), 3);
}

#[test]
fn test_instruments_profiler_export() {
    let mut profiler = InstrumentsProfiler::new();
    profiler.enable();
    
    profiler.start_region("test");
    profiler.end_region("test");
    
    let export_result = profiler.export_markers();
    assert!(export_result.is_ok());
    
    let json = export_result.unwrap();
    assert_eq!(json.get("profiler").unwrap().as_str().unwrap(), "instruments");
    assert_eq!(json.get("total_markers").unwrap().as_u64().unwrap(), 2);
}

#[test]
fn test_external_profiler_disabled() {
    let mut engine = SearchEngine::new(None, 16);
    
    // Try to export markers when profiler is not enabled
    let result = engine.export_profiling_markers();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not enabled"));
}

#[test]
fn test_profiler_marker_timestamps() {
    let mut profiler = PerfProfiler::new();
    profiler.enable();
    
    profiler.start_region("region1");
    std::thread::sleep(std::time::Duration::from_millis(10));
    profiler.end_region("region1");
    
    let markers = profiler.get_markers();
    assert_eq!(markers.len(), 2);
    
    // End timestamp should be greater than start timestamp
    assert!(markers[1].timestamp_ns > markers[0].timestamp_ns);
}

