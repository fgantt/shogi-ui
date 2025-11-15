//! Weight Management System for Shogi Engine
//!
//! This module provides functionality for loading, managing, and applying
//! tuned evaluation weights to the engine.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::types::evaluation::{NUM_EG_FEATURES, NUM_EVAL_FEATURES, NUM_MG_FEATURES};

/// Weight file format version for compatibility checking
pub const WEIGHT_FILE_VERSION: u32 = 1;

/// Magic number for weight file identification
pub const WEIGHT_FILE_MAGIC: &[u8] = b"SHOGI_WEIGHTS_V1";

/// Header for weight files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightFileHeader {
    /// Magic number for file identification
    pub magic: [u8; 16],
    /// File format version
    pub version: u32,
    /// Number of features (should match NUM_EVAL_FEATURES)
    pub num_features: usize,
    /// Number of middlegame features
    pub num_mg_features: usize,
    /// Number of endgame features
    pub num_eg_features: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Tuning method used
    pub tuning_method: String,
    /// Validation error from tuning
    pub validation_error: f64,
    /// Number of training positions used
    pub training_positions: usize,
    /// Checksum for integrity verification
    pub checksum: u64,
}

/// Complete weight file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightFile {
    /// File header with metadata
    pub header: WeightFileHeader,
    /// Weight values (NUM_EVAL_FEATURES elements)
    pub weights: Vec<f64>,
}

/// Weight application statistics
#[derive(Debug, Clone, Default)]
pub struct WeightStats {
    /// Number of times weights were applied
    pub applications: u64,
    /// Total time spent applying weights (microseconds)
    pub total_time_us: u64,
    /// Average time per application (microseconds)
    pub avg_time_us: u64,
    /// Last application timestamp
    pub last_application: Option<Instant>,
}

impl WeightStats {
    /// Update statistics after a weight application
    pub fn record_application(&mut self, duration: Duration) {
        self.applications += 1;
        self.total_time_us += duration.as_micros() as u64;
        self.avg_time_us = self.total_time_us / self.applications;
        self.last_application = Some(Instant::now());
    }

    /// Get average time per application in microseconds
    pub fn get_avg_time_us(&self) -> u64 {
        self.avg_time_us
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.applications = 0;
        self.total_time_us = 0;
        self.avg_time_us = 0;
        self.last_application = None;
    }
}

/// Weight manager for loading and applying tuned weights
#[derive(Debug)]
pub struct WeightManager {
    /// Current weights (None if using default weights)
    weights: Option<Vec<f64>>,
    /// Default weights (fallback)
    default_weights: Vec<f64>,
    /// Weight file metadata
    metadata: Option<WeightFileHeader>,
    /// Performance statistics
    stats: WeightStats,
    /// Whether tuned weights are enabled
    enabled: bool,
}

impl WeightManager {
    /// Create a new weight manager with default weights
    pub fn new() -> Self {
        Self {
            weights: None,
            default_weights: Self::create_default_weights(),
            metadata: None,
            stats: WeightStats::default(),
            enabled: false,
        }
    }

    /// Load weights from a file
    pub fn load_weights<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WeightError> {
        let path = path.as_ref();

        // Try to load the weight file
        match self.load_weight_file(path) {
            Ok(weight_file) => {
                // Validate the weight file
                self.validate_weight_file(&weight_file)?;

                // Set the weights
                self.weights = Some(weight_file.weights);
                self.metadata = Some(weight_file.header);
                self.enabled = true;

                Ok(())
            }
            Err(e) => {
                // Log the error and fall back to default weights
                eprintln!("Failed to load weights from {:?}: {}", path, e);
                self.fallback_to_default();
                Err(e)
            }
        }
    }

    /// Load weight file from path
    fn load_weight_file<P: AsRef<Path>>(&self, path: P) -> Result<WeightFile, WeightError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Try to parse as JSON first
        match serde_json::from_reader(reader) {
            Ok(weight_file) => Ok(weight_file),
            Err(_) => {
                // If JSON fails, try binary format (for future compatibility)
                Err(WeightError::UnsupportedFormat)
            }
        }
    }

    /// Validate a weight file for compatibility
    fn validate_weight_file(&self, weight_file: &WeightFile) -> Result<(), WeightError> {
        // Check magic number
        if weight_file.header.magic != Self::get_magic_bytes() {
            return Err(WeightError::InvalidMagic);
        }

        // Check version compatibility
        if weight_file.header.version != WEIGHT_FILE_VERSION {
            return Err(WeightError::VersionMismatch {
                file_version: weight_file.header.version,
                supported_version: WEIGHT_FILE_VERSION,
            });
        }

        // Check feature count
        if weight_file.header.num_features != NUM_EVAL_FEATURES {
            return Err(WeightError::FeatureCountMismatch {
                file_features: weight_file.header.num_features,
                expected_features: NUM_EVAL_FEATURES,
            });
        }

        // Check weight count
        if weight_file.weights.len() != NUM_EVAL_FEATURES {
            return Err(WeightError::WeightCountMismatch {
                file_weights: weight_file.weights.len(),
                expected_weights: NUM_EVAL_FEATURES,
            });
        }

        // Validate weight values
        for (i, &weight) in weight_file.weights.iter().enumerate() {
            if !weight.is_finite() {
                return Err(WeightError::InvalidWeight {
                    index: i,
                    value: weight,
                });
            }
        }

        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&weight_file.weights);
        if calculated_checksum != weight_file.header.checksum {
            return Err(WeightError::ChecksumMismatch {
                file_checksum: weight_file.header.checksum,
                calculated_checksum,
            });
        }

        Ok(())
    }

    /// Apply weights to evaluation features
    pub fn apply_weights(&mut self, features: &[f64], game_phase: i32) -> Result<i32, WeightError> {
        let start_time = Instant::now();

        // Ensure features match expected count
        if features.len() != NUM_EVAL_FEATURES {
            return Err(WeightError::FeatureCountMismatch {
                file_features: features.len(),
                expected_features: NUM_EVAL_FEATURES,
            });
        }

        // Get the weights to use (tuned or default)
        let weights = if self.enabled {
            self.weights.as_ref().unwrap()
        } else {
            &self.default_weights
        };

        // Apply phase-dependent weighting
        let phase_weight = game_phase as f64 / 100.0; // Assuming GAME_PHASE_MAX = 100

        let mut mg_score = 0.0;
        let mut eg_score = 0.0;

        for (i, &feature) in features.iter().enumerate() {
            if i < NUM_MG_FEATURES {
                mg_score += feature * weights[i];
            } else {
                eg_score += feature * weights[i];
            }
        }

        // Interpolate based on game phase
        let final_score = mg_score * phase_weight + eg_score * (1.0 - phase_weight);

        // Record performance statistics
        let duration = start_time.elapsed();
        self.stats.record_application(duration);

        Ok(final_score as i32)
    }

    /// Save weights to a file
    pub fn save_weights<P: AsRef<Path>>(
        &self,
        path: P,
        tuning_method: String,
        validation_error: f64,
        training_positions: usize,
    ) -> Result<(), WeightError> {
        let weights = self.weights.as_ref().ok_or(WeightError::NoWeightsLoaded)?;

        let weight_file = WeightFile {
            header: WeightFileHeader {
                magic: Self::get_magic_bytes(),
                version: WEIGHT_FILE_VERSION,
                num_features: NUM_EVAL_FEATURES,
                num_mg_features: NUM_MG_FEATURES,
                num_eg_features: NUM_EG_FEATURES,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tuning_method,
                validation_error,
                training_positions,
                checksum: self.calculate_checksum(weights),
            },
            weights: weights.clone(),
        };

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &weight_file)?;

        Ok(())
    }

    /// Enable or disable tuned weights
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if tuned weights are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if weights are loaded
    pub fn has_weights(&self) -> bool {
        self.weights.is_some()
    }

    /// Get weight metadata
    pub fn get_metadata(&self) -> Option<&WeightFileHeader> {
        self.metadata.as_ref()
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &WeightStats {
        &self.stats
    }

    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }

    /// Fall back to default weights
    fn fallback_to_default(&mut self) {
        self.weights = None;
        self.metadata = None;
        self.enabled = false;
        eprintln!("Falling back to default evaluation weights");
    }

    /// Create default weights (all 1.0)
    fn create_default_weights() -> Vec<f64> {
        vec![1.0; NUM_EVAL_FEATURES]
    }

    /// Get magic bytes for file identification
    fn get_magic_bytes() -> [u8; 16] {
        let mut magic = [0u8; 16];
        let magic_str = WEIGHT_FILE_MAGIC;
        for (i, &byte) in magic_str.iter().enumerate() {
            if i < 16 {
                magic[i] = byte;
            }
        }
        magic
    }

    /// Calculate checksum for weight validation
    fn calculate_checksum(&self, weights: &[f64]) -> u64 {
        let mut checksum = 0u64;
        for &weight in weights {
            // Convert f64 to bits for checksum
            let bits = weight.to_bits();
            checksum = checksum.wrapping_add(bits);
        }
        checksum
    }
}

impl Default for WeightManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during weight management
#[derive(Debug, thiserror::Error)]
pub enum WeightError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid magic number in weight file")]
    InvalidMagic,

    #[error(
        "Version mismatch: file version {file_version}, supported version {supported_version}"
    )]
    VersionMismatch {
        file_version: u32,
        supported_version: u32,
    },

    #[error("Feature count mismatch: file has {file_features}, expected {expected_features}")]
    FeatureCountMismatch {
        file_features: usize,
        expected_features: usize,
    },

    #[error("Weight count mismatch: file has {file_weights}, expected {expected_weights}")]
    WeightCountMismatch {
        file_weights: usize,
        expected_weights: usize,
    },

    #[error("Invalid weight at index {index}: {value}")]
    InvalidWeight { index: usize, value: f64 },

    #[error("Checksum mismatch: file checksum {file_checksum}, calculated {calculated_checksum}")]
    ChecksumMismatch {
        file_checksum: u64,
        calculated_checksum: u64,
    },

    #[error("Unsupported file format")]
    UnsupportedFormat,

    #[error("No weights loaded")]
    NoWeightsLoaded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_weight_manager_creation() {
        let manager = WeightManager::new();
        assert!(!manager.is_enabled());
        assert!(!manager.has_weights());
        assert!(manager.get_metadata().is_none());
    }

    #[test]
    fn test_default_weights() {
        let manager = WeightManager::new();
        assert_eq!(manager.default_weights.len(), NUM_EVAL_FEATURES);
        for &weight in &manager.default_weights {
            assert_eq!(weight, 1.0);
        }
    }

    #[test]
    fn test_weight_application() {
        let mut manager = WeightManager::new();
        let features = vec![1.0; NUM_EVAL_FEATURES];
        let game_phase = 50;

        let result = manager.apply_weights(&features, game_phase);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score != i32::MIN && score != i32::MAX);
    }

    #[test]
    fn test_weight_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let weight_file = temp_dir.path().join("test_weights.json");

        // Create a weight manager with dummy weights
        let mut manager = WeightManager::new();
        manager.weights = Some(vec![0.5; NUM_EVAL_FEATURES]);

        // Save weights
        let save_result = manager.save_weights(&weight_file, "test_method".to_string(), 0.1, 1000);
        assert!(save_result.is_ok());

        // Create a new manager and load weights
        let mut new_manager = WeightManager::new();
        let load_result = new_manager.load_weights(&weight_file);
        assert!(load_result.is_ok());

        assert!(new_manager.has_weights());
        assert!(new_manager.is_enabled());

        let metadata = new_manager.get_metadata().unwrap();
        assert_eq!(metadata.tuning_method, "test_method");
        assert_eq!(metadata.validation_error, 0.1);
        assert_eq!(metadata.training_positions, 1000);
    }

    #[test]
    fn test_weight_validation() {
        let mut manager = WeightManager::new();

        // Test with invalid feature count
        let invalid_features = vec![1.0; NUM_EVAL_FEATURES - 1];
        let result = manager.apply_weights(&invalid_features, 50);
        assert!(result.is_err());

        match result.unwrap_err() {
            WeightError::FeatureCountMismatch { .. } => {}
            _ => panic!("Expected FeatureCountMismatch error"),
        }
    }

    #[test]
    fn test_performance_stats() {
        let mut manager = WeightManager::new();
        let features = vec![1.0; NUM_EVAL_FEATURES];

        // Apply weights multiple times
        for _ in 0..10 {
            manager.apply_weights(&features, 50).unwrap();
        }

        let stats = manager.get_stats();
        assert_eq!(stats.applications, 10);
        assert!(stats.avg_time_us > 0);
        assert!(stats.last_application.is_some());

        // Reset stats
        manager.reset_stats();
        let stats = manager.get_stats();
        assert_eq!(stats.applications, 0);
        assert_eq!(stats.avg_time_us, 0);
        assert!(stats.last_application.is_none());
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = WeightManager::new();

        // Initially disabled
        assert!(!manager.is_enabled());

        // Enable
        manager.set_enabled(true);
        assert!(manager.is_enabled());

        // Disable
        manager.set_enabled(false);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_checksum_calculation() {
        let manager = WeightManager::new();
        let weights1 = vec![1.0, 2.0, 3.0];
        let weights2 = vec![1.0, 2.0, 3.0];
        let weights3 = vec![1.0, 2.0, 4.0];

        let checksum1 = manager.calculate_checksum(&weights1);
        let checksum2 = manager.calculate_checksum(&weights2);
        let checksum3 = manager.calculate_checksum(&weights3);

        assert_eq!(checksum1, checksum2);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_magic_bytes() {
        let magic = WeightManager::get_magic_bytes();
        assert_eq!(&magic[..WEIGHT_FILE_MAGIC.len()], WEIGHT_FILE_MAGIC);
    }
}
