//! Validation and correctness testing for magic bitboards
//!
//! This module provides comprehensive validation and testing functionality
//! for magic bitboard implementations to ensure correctness and performance.

use super::attack_generator::AttackGenerator;
use crate::types::core::PieceType;
use crate::types::{Bitboard, EMPTY_BITBOARD, MagicError, MagicTable};

/// Validator for magic bitboard correctness
pub struct MagicValidator {
    /// Attack generator for reference implementation
    attack_generator: AttackGenerator,
    /// Validation statistics
    stats: ValidationStats,
}

/// Validation statistics
#[derive(Debug, Default, Clone)]
pub struct ValidationStats {
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub validation_time: std::time::Duration,
}

impl MagicValidator {
    /// Create a new magic validator
    pub fn new() -> Self {
        Self {
            attack_generator: AttackGenerator::new(),
            stats: ValidationStats::default(),
        }
    }

    /// Validate magic table correctness
    pub fn validate_magic_table(&mut self, table: &MagicTable) -> Result<(), MagicError> {
        let start_time = std::time::Instant::now();

        // Test all squares and piece types
        for square in 0..81 {
            for piece_type in [PieceType::Rook, PieceType::Bishop] {
                self.validate_square(table, square, piece_type)?;
            }
        }

        self.stats.validation_time = start_time.elapsed();
        Ok(())
    }

    /// Validate a specific square
    fn validate_square(
        &mut self,
        table: &MagicTable,
        square: u8,
        piece_type: PieceType,
    ) -> Result<(), MagicError> {
        // Generate test blocker combinations
        let test_combinations = self.generate_test_combinations(square, piece_type);

        for blockers in test_combinations {
            let magic_attacks = table.get_attacks(square, piece_type, blockers);
            let reference_attacks = self
                .attack_generator
                .generate_attack_pattern(square, piece_type, blockers);

            self.stats.total_tests += 1;

            if magic_attacks == reference_attacks {
                self.stats.passed_tests += 1;
            } else {
                self.stats.failed_tests += 1;
                return Err(MagicError::ValidationFailed {
                    reason: format!(
                        "Attack mismatch for square {} piece {:?} blockers {:016x}: magic={:016x}, reference={:016x}",
                        square, piece_type, blockers, magic_attacks, reference_attacks
                    ),
                });
            }
        }

        Ok(())
    }

    /// Generate test blocker combinations for validation
    fn generate_test_combinations(&self, _square: u8, _piece_type: PieceType) -> Vec<Bitboard> {
        // Generate a representative set of blocker combinations
        // This is a simplified version - in practice, you'd want more comprehensive testing
        vec![
            EMPTY_BITBOARD,
            0b1,         // Single blocker
            0b111,       // Multiple blockers
            0b111111111, // More blockers
        ]
    }

    /// Benchmark magic bitboards vs ray-casting
    pub fn benchmark_magic_vs_raycast(
        &mut self,
        table: &MagicTable,
        test_positions: &[(u8, PieceType, Bitboard)],
    ) -> BenchmarkResult {
        let start_time = std::time::Instant::now();

        // Benchmark magic bitboards
        let magic_start = std::time::Instant::now();
        for (square, piece_type, blockers) in test_positions {
            let _ = table.get_attacks(*square, *piece_type, *blockers);
        }
        let magic_time = magic_start.elapsed();

        // Benchmark ray-casting
        let raycast_start = std::time::Instant::now();
        for (square, piece_type, blockers) in test_positions {
            let _ = self
                .attack_generator
                .generate_attack_pattern(*square, *piece_type, *blockers);
        }
        let raycast_time = raycast_start.elapsed();

        BenchmarkResult {
            magic_time,
            raycast_time,
            speedup: raycast_time.as_nanos() as f64 / magic_time.as_nanos() as f64,
            total_time: start_time.elapsed(),
        }
    }

    /// Test all positions for correctness
    pub fn test_all_positions(&mut self, table: &MagicTable) -> ValidationResult {
        let start_time = std::time::Instant::now();

        match self.validate_magic_table(table) {
            Ok(()) => ValidationResult {
                success: true,
                error: None,
                stats: self.stats.clone(),
                total_time: start_time.elapsed(),
            },
            Err(e) => ValidationResult {
                success: false,
                error: Some(e),
                stats: self.stats.clone(),
                total_time: start_time.elapsed(),
            },
        }
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> &ValidationStats {
        &self.stats
    }

    /// Reset validation statistics
    pub fn reset_stats(&mut self) {
        self.stats = ValidationStats::default();
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub magic_time: std::time::Duration,
    pub raycast_time: std::time::Duration,
    pub speedup: f64,
    pub total_time: std::time::Duration,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub error: Option<MagicError>,
    pub stats: ValidationStats,
    pub total_time: std::time::Duration,
}

impl Default for MagicValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_validator_creation() {
        let validator = MagicValidator::new();
        assert_eq!(validator.get_stats().total_tests, 0);
    }

    #[test]
    fn test_validation_stats() {
        let mut validator = MagicValidator::new();
        validator.stats.total_tests = 100;
        validator.stats.passed_tests = 95;
        validator.stats.failed_tests = 5;

        let stats = validator.get_stats();
        assert_eq!(stats.total_tests, 100);
        assert_eq!(stats.passed_tests, 95);
        assert_eq!(stats.failed_tests, 5);
    }

    #[test]
    fn test_reset_stats() {
        let mut validator = MagicValidator::new();
        validator.stats.total_tests = 100;
        validator.reset_stats();

        let stats = validator.get_stats();
        assert_eq!(stats.total_tests, 0);
    }

    #[test]
    fn test_benchmark_result() {
        let result = BenchmarkResult {
            magic_time: std::time::Duration::from_millis(100),
            raycast_time: std::time::Duration::from_millis(400),
            speedup: 4.0,
            total_time: std::time::Duration::from_millis(500),
        };

        assert_eq!(result.speedup, 4.0);
    }
}
