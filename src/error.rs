//! Unified Error Handling for Shogi Engine
//!
//! This module provides a comprehensive error type hierarchy for all engine operations.
//! All errors use the `thiserror` crate for ergonomic error handling.
//!
//! # Task 4.0 (Tasks 4.1-4.7)
//!
//! # Examples
//!
//! ## Creating and handling errors
//!
//! ```rust,no_run
//! use shogi_engine::error::{SearchError, ShogiEngineError, Result};
//!
//! fn search_with_timeout() -> Result<()> {
//!     Err(SearchError::timeout("Search exceeded time limit").into())
//! }
//!
//! match search_with_timeout() {
//!     Ok(()) => println!("Search completed"),
//!     Err(ShogiEngineError::Search(SearchError::Timeout { message })) => {
//!         println!("Search timed out: {}", message);
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```
//!
//! ## Error propagation
//!
//! ```rust,no_run
//! use shogi_engine::error::{ConfigurationError, ShogiEngineError, Result};
//!
//! fn load_config(path: &str) -> Result<()> {
//!     // Errors automatically convert to ShogiEngineError via From trait
//!     Err(ConfigurationError::file_not_found(path).into())
//! }
//! ```
//!
//! # Error Types
//!
//! The error hierarchy consists of:
//! - [`ShogiEngineError`]: Root error type for all engine operations
//! - [`SearchError`]: Search-related errors (timeout, invalid depth, etc.)
//! - [`EvaluationError`]: Evaluation-related errors (invalid position, component failure, etc.)
//! - [`TranspositionTableError`]: Transposition table errors (invalid size, probe failure, etc.)
//! - [`MoveGenerationError`]: Move generation errors
//! - [`ConfigurationError`]: Configuration validation and loading errors

use thiserror::Error;

/// Root error type for all engine operations
///
/// This enum serves as the single error type for the entire engine.
/// All module-specific errors are converted to this type using `From` trait implementations.
///
/// # Task 4.0 (Task 4.1)
#[derive(Error, Debug)]
pub enum ShogiEngineError {
    /// Search-related errors
    #[error("Search error: {0}")]
    Search(#[from] SearchError),

    /// Evaluation-related errors
    #[error("Evaluation error: {0}")]
    Evaluation(#[from] EvaluationError),

    /// Transposition table errors
    #[error("Transposition table error: {0}")]
    TranspositionTable(#[from] TranspositionTableError),

    /// Move generation errors
    #[error("Move generation error: {0}")]
    MoveGeneration(#[from] MoveGenerationError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
}

/// Search-related errors
///
/// Errors that can occur during search operations.
///
/// # Task 4.0 (Task 4.2)
#[derive(Error, Debug)]
pub enum SearchError {
    /// Search timed out before completing
    #[error("Search timed out: {message}")]
    Timeout { message: String },

    /// Invalid search depth specified
    #[error("Invalid search depth: {depth} (valid range: 1-{max_depth})")]
    InvalidDepth { depth: u8, max_depth: u8 },

    /// Invalid time limit specified
    #[error("Invalid time limit: {limit_ms}ms (must be > 0)")]
    InvalidTimeLimit { limit_ms: u64 },

    /// Invalid position for search
    #[error("Invalid position for search: {message}")]
    InvalidPosition { message: String },

    /// Search was interrupted (e.g., by user)
    #[error("Search interrupted: {message}")]
    Interrupted { message: String },

    /// Internal search error
    #[error("Internal search error: {message}")]
    Internal { message: String },
}

/// Evaluation-related errors
///
/// Errors that can occur during evaluation operations.
///
/// # Task 4.0 (Task 4.3)
#[derive(Error, Debug)]
pub enum EvaluationError {
    /// Invalid position for evaluation
    #[error("Invalid position for evaluation: {message}")]
    InvalidPosition { message: String },

    /// Evaluation component failed
    #[error("Evaluation component '{component}' failed: {message}")]
    ComponentFailure { component: String, message: String },

    /// Configuration error for evaluation
    #[error("Evaluation configuration error: {message}")]
    ConfigurationError { message: String },

    /// Cache operation failed
    #[error("Evaluation cache operation failed: {message}")]
    CacheError { message: String },

    /// Internal evaluation error
    #[error("Internal evaluation error: {message}")]
    Internal { message: String },
}

/// Transposition table errors
///
/// Errors that can occur during transposition table operations.
///
/// # Task 4.0 (Task 4.4)
#[derive(Error, Debug)]
pub enum TranspositionTableError {
    /// Invalid table size specified
    #[error("Invalid transposition table size: {size} (must be > 0)")]
    InvalidSize { size: usize },

    /// Probe operation failed
    #[error("Transposition table probe failed: {message}")]
    ProbeFailed { message: String },

    /// Store operation failed
    #[error("Transposition table store failed: {message}")]
    StoreFailed { message: String },

    /// Memory allocation failed
    #[error("Memory allocation failed for transposition table: {message}")]
    MemoryAllocationFailed { message: String },

    /// Thread safety violation
    #[error("Thread safety violation in transposition table: {message}")]
    ThreadSafetyViolation { message: String },

    /// Data corruption detected
    #[error("Data corruption detected in transposition table: {message}")]
    DataCorruption { message: String },

    /// Configuration error for transposition table
    #[error("Transposition table configuration error: {message}")]
    ConfigurationError { message: String },
}

/// Move generation errors
///
/// Errors that can occur during move generation.
///
/// # Task 4.0 (Task 4.5)
#[derive(Error, Debug)]
pub enum MoveGenerationError {
    /// Invalid position for move generation
    #[error("Invalid position for move generation: {message}")]
    InvalidPosition { message: String },

    /// Invalid piece or square
    #[error("Invalid piece or square: {message}")]
    InvalidPieceOrSquare { message: String },

    /// Move generation failed
    #[error("Move generation failed: {message}")]
    GenerationFailed { message: String },

    /// Internal move generation error
    #[error("Internal move generation error: {message}")]
    Internal { message: String },
}

/// Configuration errors
///
/// Errors that can occur during configuration validation or loading.
///
/// # Task 4.0 (Task 4.6)
#[derive(Error, Debug)]
pub enum ConfigurationError {
    /// Invalid configuration value
    #[error("Invalid configuration value for '{field}': {value} (expected: {expected})")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },

    /// Missing required configuration field
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },

    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Failed to parse configuration file
    #[error("Failed to parse configuration file '{path}': {message}")]
    ParseError { path: String, message: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {message}")]
    ValidationFailed { message: String },

    /// Configuration serialization failed
    #[error("Configuration serialization failed: {message}")]
    SerializationFailed { message: String },

    /// Configuration deserialization failed
    #[error("Configuration deserialization failed: {message}")]
    DeserializationFailed { message: String },
}

// Task 4.0 (Task 4.7): From trait conversions are automatically provided by #[from] attributes

impl TranspositionTableError {
    /// Create an invalid size error
    pub fn invalid_size(size: usize) -> Self {
        Self::InvalidSize { size }
    }

    /// Create a probe failed error
    pub fn probe_failed<S: Into<String>>(message: S) -> Self {
        Self::ProbeFailed {
            message: message.into(),
        }
    }

    /// Create a store failed error
    pub fn store_failed<S: Into<String>>(message: S) -> Self {
        Self::StoreFailed {
            message: message.into(),
        }
    }
}

impl SearchError {
    /// Create a timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create an invalid depth error
    pub fn invalid_depth(depth: u8, max_depth: u8) -> Self {
        Self::InvalidDepth { depth, max_depth }
    }
}

impl EvaluationError {
    /// Create a component failure error
    pub fn component_failure<S1: Into<String>, S2: Into<String>>(
        component: S1,
        message: S2,
    ) -> Self {
        Self::ComponentFailure {
            component: component.into(),
            message: message.into(),
        }
    }
}

impl ConfigurationError {
    /// Create an invalid value error
    pub fn invalid_value<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        field: S1,
        value: S2,
        expected: S3,
    ) -> Self {
        Self::InvalidValue {
            field: field.into(),
            value: value.into(),
            expected: expected.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field<S: Into<String>>(field: S) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create a file not found error
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        Self::FileNotFound {
            path: path.into(),
        }
    }

    /// Create a parse error
    pub fn parse_error<S1: Into<String>, S2: Into<String>>(path: S1, message: S2) -> Self {
        Self::ParseError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a validation failed error
    pub fn validation_failed<S: Into<String>>(message: S) -> Self {
        Self::ValidationFailed {
            message: message.into(),
        }
    }

    /// Create a serialization failed error
    pub fn serialization_failed<S: Into<String>>(message: S) -> Self {
        Self::SerializationFailed {
            message: message.into(),
        }
    }
}

/// Convenience type alias for Result with ShogiEngineError
pub type Result<T, E = ShogiEngineError> = std::result::Result<T, E>;

