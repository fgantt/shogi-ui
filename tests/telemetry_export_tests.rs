//! Tests for telemetry export functionality (Task 26.0 - Task 7.0)

use crate::search::performance_tuning::TelemetryExporter;
use crate::search::search_engine::SearchEngine;
use crate::types::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_telemetry_json_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let exporter = TelemetryExporter::new(export_path);
    let mut engine = SearchEngine::new();
    
    // Run a short search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Export to JSON
    let result = exporter.export_performance_metrics_to_json(&engine, "test_metrics.json");
    assert!(result.is_ok());
    
    let file_path = result.unwrap();
    assert!(file_path.exists());
    
    // Verify JSON content
    let content = fs::read_to_string(&file_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert!(json.get("timestamp").is_some());
    assert!(json.get("performance_metrics").is_some());
    assert!(json.get("baseline_metrics").is_some());
}

#[test]
fn test_telemetry_csv_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let exporter = TelemetryExporter::new(export_path);
    let mut engine = SearchEngine::new();
    
    // Run a short search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Export to CSV
    let result = exporter.export_performance_metrics_to_csv(&engine, "test_metrics.csv");
    assert!(result.is_ok());
    
    let file_path = result.unwrap();
    assert!(file_path.exists());
    
    // Verify CSV content
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Metric,Value"));
    assert!(content.contains("nodes_per_second"));
    assert!(content.contains("health_score"));
}

#[test]
fn test_telemetry_markdown_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let exporter = TelemetryExporter::new(export_path);
    let mut engine = SearchEngine::new();
    
    // Run a short search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Export to Markdown
    let result = exporter.export_performance_metrics_to_markdown(&engine, "test_metrics.md");
    assert!(result.is_ok());
    
    let file_path = result.unwrap();
    assert!(file_path.exists());
    
    // Verify Markdown content
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("# Performance Metrics Report"));
    assert!(content.contains("## Performance Metrics"));
    assert!(content.contains("## Search Metrics"));
}

#[test]
fn test_telemetry_efficiency_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let exporter = TelemetryExporter::new(export_path);
    let mut engine = SearchEngine::new();
    
    // Run a short search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Export efficiency metrics
    let result = exporter.export_efficiency_metrics(&engine, "efficiency.json");
    assert!(result.is_ok());
    
    let file_path = result.unwrap();
    assert!(file_path.exists());
    
    // Verify JSON content
    let content = fs::read_to_string(&file_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert!(json.get("timestamp").is_some());
    assert!(json.get("iid_metrics").is_some());
    assert!(json.get("lmr_metrics").is_some());
}

#[test]
fn test_telemetry_export_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let mut exporter = TelemetryExporter::with_enabled(export_path, false);
    let mut engine = SearchEngine::new();
    
    // Run a short search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 4, None);
    
    // Try to export - should fail
    let result = exporter.export_performance_metrics_to_json(&engine, "test.json");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("disabled"));
}

#[test]
fn test_telemetry_export_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path();
    
    let exporter = TelemetryExporter::new(export_path);
    let mut engine = SearchEngine::new();
    
    // Run a search to generate metrics
    let board = BitboardBoard::starting_position();
    let _ = engine.search(&board, 5, None);
    
    // Export all telemetry formats
    let json_result = exporter.export_performance_metrics_to_json(&engine, "metrics.json");
    let csv_result = exporter.export_performance_metrics_to_csv(&engine, "metrics.csv");
    let md_result = exporter.export_performance_metrics_to_markdown(&engine, "metrics.md");
    let efficiency_result = exporter.export_efficiency_metrics(&engine, "efficiency.json");
    let tt_result = exporter.export_tt_entry_quality_distribution(&engine, "tt_quality.json");
    let hit_rate_result = exporter.export_hit_rate_by_depth(&engine, "hit_rate.json");
    let scalability_result = exporter.export_scalability_metrics(&engine, "scalability.json");
    let cache_result = exporter.export_cache_effectiveness(&engine, "cache.json");
    
    // Verify all exports succeeded
    assert!(json_result.is_ok());
    assert!(csv_result.is_ok());
    assert!(md_result.is_ok());
    assert!(efficiency_result.is_ok());
    assert!(tt_result.is_ok());
    assert!(hit_rate_result.is_ok());
    assert!(scalability_result.is_ok());
    assert!(cache_result.is_ok());
    
    // Verify all files exist
    assert!(json_result.unwrap().exists());
    assert!(csv_result.unwrap().exists());
    assert!(md_result.unwrap().exists());
    assert!(efficiency_result.unwrap().exists());
    assert!(tt_result.unwrap().exists());
    assert!(hit_rate_result.unwrap().exists());
    assert!(scalability_result.unwrap().exists());
    assert!(cache_result.unwrap().exists());
}

