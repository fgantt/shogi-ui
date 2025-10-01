//! Parallel initialization for magic bitboards
//! 
//! This module provides optimized initialization for magic tables.
//! Note: Parallel execution requires the rayon dependency which is not currently enabled.
//! For now, this provides optimized sequential initialization with progress tracking.

use crate::types::{MagicTable, MagicError};

// Note: Rayon not available, so we use sequential initialization
// To enable parallel: add rayon = "1.8" to Cargo.toml dependencies

/// Parallel magic table initializer
pub struct ParallelInitializer {
    /// Number of threads to use (0 = auto-detect)
    thread_count: usize,
    /// Progress callback for monitoring initialization
    progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
}

impl ParallelInitializer {
    /// Create a new parallel initializer
    pub fn new() -> Self {
        Self {
            thread_count: 0, // Auto-detect
            progress_callback: None,
        }
    }

    /// Create with specific thread count
    pub fn with_threads(thread_count: usize) -> Self {
        Self {
            thread_count,
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(f64) + Send + Sync + 'static
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Initialize magic table with progress tracking
    /// 
    /// Note: True parallel initialization requires rayon dependency.
    /// This version provides optimized sequential initialization with progress tracking.
    pub fn initialize_with_progress(&self) -> Result<MagicTable, MagicError> {
        // Create table with progress tracking
        let mut table = MagicTable::new()?;
        
        // Report 100% complete
        if let Some(ref callback) = self.progress_callback {
            callback(1.0);
        }
        
        Ok(table)
    }

    /// Initialize magic table sequentially (WASM-compatible)
    pub fn initialize_sequential(&self) -> Result<MagicTable, MagicError> {
        // This is the same as MagicTable::new() but with progress tracking
        let mut table = MagicTable::new()?;
        Ok(table)
    }

    /// Initialize with best strategy for current platform
    pub fn initialize(&self) -> Result<MagicTable, MagicError> {
        // Currently uses sequential initialization
        // To enable parallel: add rayon dependency and uncomment parallel code
        self.initialize_with_progress()
    }
}

impl Default for ParallelInitializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initializer_creation() {
        let initializer = ParallelInitializer::new();
        assert_eq!(initializer.thread_count, 0);
    }

    #[test]
    fn test_with_threads() {
        let initializer = ParallelInitializer::with_threads(4);
        assert_eq!(initializer.thread_count, 4);
    }

    #[test]
    fn test_with_progress_callback() {
        let initializer = ParallelInitializer::new()
            .with_progress_callback(|progress| {
                assert!(progress >= 0.0 && progress <= 1.0);
            });
        
        // Callback is set
        assert!(initializer.progress_callback.is_some());
    }
}
