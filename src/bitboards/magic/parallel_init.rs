//! Parallel initialization for magic bitboards
//!
//! This module provides optimized initialization for magic tables with support for:
//! - Parallel initialization using rayon
//! - Progress tracking and reporting
//! - Configurable thread count

use crate::types::{MagicError, MagicTable, PieceType};
use crate::bitboards::magic::attack_generator::AttackGenerator;
use crate::bitboards::magic::magic_finder::MagicFinder;
use crate::types::{Bitboard, EMPTY_BITBOARD};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

/// Parallel magic table initializer
pub struct ParallelInitializer {
    /// Number of threads to use (0 = auto-detect)
    thread_count: usize,
    /// Progress callback for monitoring initialization
    progress_callback: Option<Arc<Mutex<Box<dyn Fn(f64) + Send + Sync>>>>,
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
        F: Fn(f64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }

    /// Report progress (thread-safe)
    fn report_progress(&self, progress: f64) {
        if let Some(ref callback) = self.progress_callback {
            if let Ok(cb) = callback.lock() {
                cb(progress);
            }
        }
    }

    /// Initialize magic table with progress tracking
    ///
    /// Uses parallel initialization when rayon is available.
    pub fn initialize_with_progress(&self) -> Result<MagicTable, MagicError> {
        self.initialize()
    }

    /// Initialize magic table sequentially
    pub fn initialize_sequential(&self) -> Result<MagicTable, MagicError> {
        let mut table = MagicTable::default();
        let total_squares = 162;
        let mut completed = 0;

        // Initialize rook tables
        for square in 0..81 {
            table.initialize_rook_square(square)?;
            completed += 1;
            self.report_progress(completed as f64 / total_squares as f64);
        }

        // Initialize bishop tables
        for square in 0..81 {
            table.initialize_bishop_square(square)?;
            completed += 1;
            self.report_progress(completed as f64 / total_squares as f64);
        }

        Ok(table)
    }

    /// Initialize with best strategy for current platform
    ///
    /// Uses parallel initialization with rayon when available.
    pub fn initialize(&self) -> Result<MagicTable, MagicError> {
        // Configure rayon thread pool if custom thread count specified
        if self.thread_count > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(self.thread_count)
                .build_global()
                .ok(); // Ignore errors (pool may already be initialized)
        }

        // Use parallel initialization
        self.initialize_parallel()
    }

    /// Initialize magic table in parallel using rayon
    fn initialize_parallel(&self) -> Result<MagicTable, MagicError> {
        let mut table = MagicTable::default();
        
        // Pre-allocate storage
        let estimated_size = 1024 * 162; // Conservative estimate
        table.attack_storage.reserve(estimated_size);

        // Shared state for progress tracking
        let progress = Arc::new(Mutex::new(0usize));
        let total_squares = 162;

        // Initialize rook tables in parallel
        // We generate magic numbers and patterns in parallel, then apply sequentially
        let rook_results: Result<Vec<_>, _> = (0..81)
            .into_par_iter()
            .map(|square| {
                let mut finder = MagicFinder::new();
                let magic_result = finder.find_magic_number(square, PieceType::Rook)?;
                
                // Generate attack patterns
                let mut generator = AttackGenerator::new();
                let mask = magic_result.mask;
                let combinations = generator.generate_all_blocker_combinations(mask);
                
                let patterns: Vec<(usize, Bitboard)> = combinations
                    .iter()
                    .map(|&blockers| {
                        let attack = generator.generate_attack_pattern(square, PieceType::Rook, blockers);
                        let hash = (blockers.wrapping_mul(magic_result.magic_number as u128)) >> magic_result.shift;
                        (hash as usize, attack)
                    })
                    .collect();

                Ok((square, magic_result, patterns))
            })
            .collect();

        // Process rook results and update table sequentially (to avoid mutability issues)
        for result in rook_results? {
            let (square, magic_result, patterns) = result;
            let attack_base = table.memory_pool.allocate(magic_result.table_size)?;
            
            // Store patterns
            for (hash, attack) in patterns {
                let index = attack_base + hash;
                if index >= table.attack_storage.len() {
                    table.attack_storage.resize(index + 1, EMPTY_BITBOARD);
                }
                table.attack_storage[index] = attack;
            }

            table.rook_magics[square as usize] = crate::types::MagicBitboard {
                magic_number: magic_result.magic_number,
                mask: magic_result.mask,
                shift: magic_result.shift,
                attack_base,
                table_size: magic_result.table_size,
            };

            // Update progress
            {
                let mut prog = progress.lock().unwrap();
                *prog += 1;
                self.report_progress(*prog as f64 / total_squares as f64);
            }
        }

        // Initialize bishop tables in parallel
        let bishop_results: Result<Vec<_>, _> = (0..81)
            .into_par_iter()
            .map(|square| {
                let mut finder = MagicFinder::new();
                let magic_result = finder.find_magic_number(square, PieceType::Bishop)?;
                
                // Generate attack patterns
                let mut generator = AttackGenerator::new();
                let mask = magic_result.mask;
                let combinations = generator.generate_all_blocker_combinations(mask);
                
                let patterns: Vec<(usize, Bitboard)> = combinations
                    .iter()
                    .map(|&blockers| {
                        let attack = generator.generate_attack_pattern(square, PieceType::Bishop, blockers);
                        let hash = (blockers.wrapping_mul(magic_result.magic_number as u128)) >> magic_result.shift;
                        (hash as usize, attack)
                    })
                    .collect();

                Ok((square, magic_result, patterns))
            })
            .collect();

        // Process bishop results and update table sequentially
        for result in bishop_results? {
            let (square, magic_result, patterns) = result;
            let attack_base = table.memory_pool.allocate(magic_result.table_size)?;
            
            // Store patterns
            for (hash, attack) in patterns {
                let index = attack_base + hash;
                if index >= table.attack_storage.len() {
                    table.attack_storage.resize(index + 1, EMPTY_BITBOARD);
                }
                table.attack_storage[index] = attack;
            }

            table.bishop_magics[square as usize] = crate::types::MagicBitboard {
                magic_number: magic_result.magic_number,
                mask: magic_result.mask,
                shift: magic_result.shift,
                attack_base,
                table_size: magic_result.table_size,
            };

            // Update progress
            {
                let mut prog = progress.lock().unwrap();
                *prog += 1;
                self.report_progress(*prog as f64 / total_squares as f64);
            }
        }

        Ok(table)
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
        let initializer = ParallelInitializer::new().with_progress_callback(|progress| {
            assert!(progress >= 0.0 && progress <= 1.0);
        });

        // Callback is set
        assert!(initializer.progress_callback.is_some());
    }

    #[test]
    #[ignore] // Long-running test
    fn test_parallel_initialization() {
        let initializer = ParallelInitializer::new();
        let result = initializer.initialize();
        assert!(result.is_ok());
        
        let table = result.unwrap();
        assert!(table.is_fully_initialized());
    }

    #[test]
    #[ignore] // Long-running test
    fn test_sequential_initialization() {
        let initializer = ParallelInitializer::new();
        let result = initializer.initialize_sequential();
        assert!(result.is_ok());
        
        let table = result.unwrap();
        assert!(table.is_fully_initialized());
    }

    #[test]
    #[ignore] // Long-running test
    fn test_progress_callback() {
        let mut progress_values = Vec::new();
        let initializer = ParallelInitializer::new()
            .with_progress_callback(move |progress| {
                progress_values.push(progress);
            });

        let _table = initializer.initialize().unwrap();
        
        // Verify progress was reported
        assert!(!progress_values.is_empty());
        assert!(progress_values[0] >= 0.0);
        assert!(*progress_values.last().unwrap() >= 1.0);
    }
}
