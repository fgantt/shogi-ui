//! Castle pattern definitions and recognition logic
//!
//! This module contains the specific castle patterns used in Shogi,
//! including Mino, Anaguma, and Yagura formations.

pub mod anaguma;
pub mod common;
pub mod mino;
pub mod yagura;

// Re-export the main pattern types
pub use anaguma::*;
pub use common::*;
pub use mino::*;
pub use yagura::*;
