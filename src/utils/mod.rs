//! Centralized engine utilities.
//!
//! This module provides a single, documented entry point for shared helpers used
//! across the engine. Prefer adding new reusable helpers here rather than
//! sprinkling small utilities across feature modules.
//!
//! Submodules:
//! - time: time sources and convenience helpers
//! - telemetry: lightweight debug/trace logging integration points
//! - common: small general-purpose helpers

pub mod time;
pub mod telemetry;
pub mod common;


