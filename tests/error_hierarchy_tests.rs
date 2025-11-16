//! Tests for error type hierarchy and error propagation
//!
//! Tests for Task 4.0 - Tasks 4.31: Verify error type hierarchy and error propagation

use shogi_engine::error::*;

#[test]
fn test_search_error_variants() {
    // Test SearchError variants
    let timeout_err = SearchError::timeout("Test timeout");
    assert!(matches!(timeout_err, SearchError::Timeout { .. }));

    let depth_err = SearchError::invalid_depth(100, 20);
    assert!(matches!(depth_err, SearchError::InvalidDepth { depth: 100, max_depth: 20 }));

    let invalid_pos = SearchError::InvalidPosition {
        message: "Invalid position".to_string(),
    };
    assert!(matches!(invalid_pos, SearchError::InvalidPosition { .. }));
}

#[test]
fn test_evaluation_error_variants() {
    // Test EvaluationError variants
    let component_err = EvaluationError::component_failure("material", "Failed");
    assert!(matches!(component_err, EvaluationError::ComponentFailure { .. }));

    let invalid_pos = EvaluationError::InvalidPosition {
        message: "Invalid position".to_string(),
    };
    assert!(matches!(invalid_pos, EvaluationError::InvalidPosition { .. }));
}

#[test]
fn test_transposition_table_error_variants() {
    // Test TranspositionTableError variants
    let size_err = TranspositionTableError::invalid_size(0);
    assert!(matches!(size_err, TranspositionTableError::InvalidSize { size: 0 }));

    let probe_err = TranspositionTableError::probe_failed("Probe failed");
    assert!(matches!(probe_err, TranspositionTableError::ProbeFailed { .. }));

    let store_err = TranspositionTableError::store_failed("Store failed");
    assert!(matches!(store_err, TranspositionTableError::StoreFailed { .. }));
}

#[test]
fn test_configuration_error_variants() {
    // Test ConfigurationError variants
    let invalid_val = ConfigurationError::invalid_value("field", "value", "expected");
    assert!(matches!(invalid_val, ConfigurationError::InvalidValue { .. }));

    let missing = ConfigurationError::missing_field("field");
    assert!(matches!(missing, ConfigurationError::MissingField { field: _ }));

    let not_found = ConfigurationError::file_not_found("path");
    assert!(matches!(not_found, ConfigurationError::FileNotFound { .. }));

    let parse_err = ConfigurationError::parse_error("path", "message");
    assert!(matches!(parse_err, ConfigurationError::ParseError { .. }));

    let validation_err = ConfigurationError::validation_failed("message");
    assert!(matches!(validation_err, ConfigurationError::ValidationFailed { .. }));

    let serialization_err = ConfigurationError::serialization_failed("message");
    assert!(matches!(serialization_err, ConfigurationError::SerializationFailed { .. }));
}

#[test]
fn test_error_from_conversions() {
    // Test that From trait conversions work correctly
    // Task 4.0 (Task 4.7)

    // SearchError -> ShogiEngineError
    let search_err = SearchError::timeout("Timeout");
    let engine_err: ShogiEngineError = search_err.into();
    assert!(matches!(engine_err, ShogiEngineError::Search(_)));

    // EvaluationError -> ShogiEngineError
    let eval_err = EvaluationError::InvalidPosition {
        message: "Invalid".to_string(),
    };
    let engine_err: ShogiEngineError = eval_err.into();
    assert!(matches!(engine_err, ShogiEngineError::Evaluation(_)));

    // TranspositionTableError -> ShogiEngineError
    let tt_err = TranspositionTableError::invalid_size(0);
    let engine_err: ShogiEngineError = tt_err.into();
    assert!(matches!(engine_err, ShogiEngineError::TranspositionTable(_)));

    // MoveGenerationError -> ShogiEngineError
    let move_err = MoveGenerationError::InvalidPosition {
        message: "Invalid".to_string(),
    };
    let engine_err: ShogiEngineError = move_err.into();
    assert!(matches!(engine_err, ShogiEngineError::MoveGeneration(_)));

    // ConfigurationError -> ShogiEngineError
    let config_err = ConfigurationError::missing_field("field");
    let engine_err: ShogiEngineError = config_err.into();
    assert!(matches!(engine_err, ShogiEngineError::Configuration(_)));
}

#[test]
fn test_error_display() {
    // Test that errors implement Display correctly
    let search_err = SearchError::timeout("Test timeout");
    let display = format!("{}", search_err);
    // Error message is "Search timed out: {message}" (lowercase "timed out")
    assert!(display.contains("timed out") || display.contains("Timeout"), "Display: '{}'", display);
    assert!(display.contains("Test timeout"), "Display: '{}'", display);

    let config_err = ConfigurationError::invalid_value("field", "value", "expected");
    let display = format!("{}", config_err);
    assert!(display.contains("field"));
    assert!(display.contains("value"));
    assert!(display.contains("expected"));
}

#[test]
fn test_error_propagation() {
    // Test error propagation through Result types
    fn function_that_fails() -> Result<()> {
        Err(SearchError::timeout("Timeout").into())
    }

    let result = function_that_fails();
    assert!(result.is_err());
    if let Err(ShogiEngineError::Search(SearchError::Timeout { .. })) = result {
        // Correct error type propagated
    } else {
        panic!("Error not propagated correctly");
    }
}

