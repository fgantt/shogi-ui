//! Search helpers and integration submodules.
//!
//! This namespace aggregates helper types and utilities used across search.
//! It provides a stable import surface while allowing internal code to be
//! organized without breaking external imports.
//!
//! - time: time management utilities for search cadence and limits
//! - parallel: work queue helpers and parallel search configuration/types
//!
//! Re-exports keep public API stable.

/// Time management helpers for search.
pub use crate::search::time_management::TimeManager;

/// Parallel search helpers and types.
pub use crate::search::parallel_search::{
    ParallelSearchConfig, WorkDistributionRecorder, WorkDistributionStats, WorkMetricsMode,
    WorkQueueSnapshot, WorkStealingQueue,
};


