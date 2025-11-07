//! Tuning module for automated evaluation parameter optimization
//!
//! This module provides the core data structures and functionality for implementing
//! Texel's tuning method and other optimization algorithms for the shogi engine's
//! evaluation function.
//!
//! The tuning system works by:
//! 1. Processing large datasets of real game positions
//! 2. Extracting feature vectors from each position
//! 3. Using machine learning algorithms to optimize evaluation weights
//! 4. Validating results through cross-validation and engine strength testing
//!
//! Key components:
//! - `types.rs`: Core data structures for games, positions, and configuration
//! - `feature_extractor.rs`: Feature extraction from positions
//! - `data_processor.rs`: Game database processing and position filtering
//! - `optimizer.rs`: Optimization algorithms (gradient descent, Adam, LBFGS, genetic)
//! - `validator.rs`: Validation framework and cross-validation
//! - `performance.rs`: Performance monitoring and analysis

pub mod data_processor;
pub mod feature_extractor;
pub mod optimizer;
pub mod performance;
pub mod types;
pub mod validator;

// Re-export commonly used types
pub use types::*;
