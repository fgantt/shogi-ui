//! Castle pattern definitions and recognition logic
//! 
//! This module contains the specific castle patterns used in Shogi,
//! including Mino, Anaguma, and Yagura formations.

pub mod mino;
pub mod anaguma;
pub mod yagura;
pub mod common;

// Re-export the main pattern types
pub use mino::*;
pub use anaguma::*;
pub use yagura::*;
pub use common::*;
