//! WASM Compatibility Module for Tapered Evaluation
//!
//! This module provides WASM-specific implementations and optimizations for
//! the tapered evaluation system. It handles differences between native and
//! WASM environments, particularly around memory management, timing, and
//! binary size optimization.
//!
//! # Features
//!
//! - **WASM-Optimized Configuration**: Reduced memory footprint
//! - **Conditional Compilation**: Seamless native/WASM switching
//! - **Memory Management**: Optimized for browser constraints
//! - **Binary Size Optimization**: Minimal WASM binary impact
//! - **Performance Tuning**: WASM-specific optimizations
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::evaluation::wasm_compatibility::WasmEvaluatorConfig;
//!
//! #[cfg(target_arch = "wasm32")]
//! let config = WasmEvaluatorConfig::wasm_optimized();
//!
//! #[cfg(not(target_arch = "wasm32"))]
//! let config = WasmEvaluatorConfig::native_optimized();
//!
//! let evaluator = create_evaluator_with_wasm_config(config);
//! ```

use crate::evaluation::integration::{IntegratedEvaluationConfig, ComponentFlags};
use serde::{Deserialize, Serialize};

/// WASM-specific configuration for tapered evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEvaluatorConfig {
    /// Enable WASM-specific optimizations
    pub enable_wasm_optimizations: bool,
    /// Maximum memory usage in bytes (for WASM)
    pub max_wasm_memory: usize,
    /// Use compact data structures
    pub use_compact_structures: bool,
    /// Disable heavy features in WASM
    pub disable_heavy_features: bool,
    /// Reduce cache sizes for WASM
    pub reduce_cache_sizes: bool,
    /// Phase cache size for WASM
    pub wasm_phase_cache_size: usize,
    /// Evaluation cache size for WASM
    pub wasm_eval_cache_size: usize,
    /// Disable statistics in WASM (saves memory)
    pub disable_statistics_wasm: bool,
}

impl WasmEvaluatorConfig {
    /// Create WASM-optimized configuration
    pub fn wasm_optimized() -> Self {
        Self {
            enable_wasm_optimizations: true,
            max_wasm_memory: 16 * 1024 * 1024, // 16MB
            use_compact_structures: true,
            disable_heavy_features: false,
            reduce_cache_sizes: true,
            wasm_phase_cache_size: 1000,
            wasm_eval_cache_size: 2000,
            disable_statistics_wasm: true,
        }
    }

    /// Create memory-constrained WASM configuration
    pub fn wasm_memory_constrained() -> Self {
        Self {
            enable_wasm_optimizations: true,
            max_wasm_memory: 4 * 1024 * 1024, // 4MB
            use_compact_structures: true,
            disable_heavy_features: true,
            reduce_cache_sizes: true,
            wasm_phase_cache_size: 500,
            wasm_eval_cache_size: 1000,
            disable_statistics_wasm: true,
        }
    }

    /// Create native-optimized configuration
    pub fn native_optimized() -> Self {
        Self {
            enable_wasm_optimizations: false,
            max_wasm_memory: 100 * 1024 * 1024, // 100MB
            use_compact_structures: false,
            disable_heavy_features: false,
            reduce_cache_sizes: false,
            wasm_phase_cache_size: 10000,
            wasm_eval_cache_size: 10000,
            disable_statistics_wasm: false,
        }
    }

    /// Create platform-optimized configuration (auto-detects)
    pub fn platform_optimized() -> Self {
        if Self::is_wasm_environment() {
            Self::wasm_optimized()
        } else {
            Self::native_optimized()
        }
    }

    /// Check if running in WASM environment
    pub fn is_wasm_environment() -> bool {
        cfg!(target_arch = "wasm32")
    }

    /// Convert to IntegratedEvaluationConfig
    pub fn to_integrated_config(&self) -> IntegratedEvaluationConfig {
        let mut config = IntegratedEvaluationConfig::default();

        // Adjust based on WASM settings
        if self.enable_wasm_optimizations {
            config.max_cache_size = self.wasm_eval_cache_size;
            
            if self.disable_heavy_features {
                // Disable expensive components in WASM
                config.components.position_features = false;
                config.components.opening_principles = false;
                config.components.endgame_patterns = false;
            } else {
                config.components = ComponentFlags::all_enabled();
            }

            if self.reduce_cache_sizes {
                config.max_cache_size = self.wasm_eval_cache_size;
            }
        }

        config
    }

    /// Estimate memory usage
    pub fn estimate_memory_usage(&self) -> usize {
        let base_evaluator = 1024; // ~1KB
        let phase_cache = self.wasm_phase_cache_size * 16; // 16 bytes per entry
        let eval_cache = self.wasm_eval_cache_size * 32; // 32 bytes per entry
        let statistics = if self.disable_statistics_wasm { 0 } else { 3 * 1024 }; // ~3KB
        
        base_evaluator + phase_cache + eval_cache + statistics
    }

    /// Estimate binary size impact (in KB)
    pub fn estimate_binary_size_impact(&self) -> u32 {
        let base_tapered = 40; // Base tapered evaluation code
        let material = 10;
        let pst = 15;
        let phase_transition = 8;
        
        let position_features = if self.disable_heavy_features { 0 } else { 20 };
        let opening_principles = if self.disable_heavy_features { 0 } else { 15 };
        let endgame_patterns = if self.disable_heavy_features { 0 } else { 15 };
        
        let statistics = if self.disable_statistics_wasm { 0 } else { 12 };
        let advanced = if self.disable_heavy_features { 0 } else { 10 };
        
        base_tapered + material + pst + phase_transition + 
        position_features + opening_principles + endgame_patterns +
        statistics + advanced
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_wasm_memory == 0 {
            return Err("Max WASM memory must be greater than 0".to_string());
        }
        
        if self.max_wasm_memory > 1024 * 1024 * 1024 {
            return Err("Max WASM memory should not exceed 1GB".to_string());
        }
        
        let estimated = self.estimate_memory_usage();
        if estimated > self.max_wasm_memory {
            return Err(format!(
                "Estimated memory usage ({} bytes) exceeds limit ({} bytes)",
                estimated, self.max_wasm_memory
            ));
        }
        
        Ok(())
    }
}

impl Default for WasmEvaluatorConfig {
    fn default() -> Self {
        Self::platform_optimized()
    }
}

/// WASM-specific utility functions
pub mod wasm_utils {
    use super::*;

    /// Get optimal cache size for WASM
    pub fn get_optimal_wasm_cache_size(available_memory_mb: u32) -> usize {
        // Conservative sizing for WASM environments
        let max_entries = (available_memory_mb as usize * 1024 * 1024) / 32;
        max_entries.min(5000) // Cap at 5000 entries
    }

    /// Get optimal component configuration for WASM
    pub fn get_wasm_components(memory_constrained: bool) -> ComponentFlags {
        if memory_constrained {
            ComponentFlags::minimal()
        } else {
            ComponentFlags {
                material: true,
                piece_square_tables: true,
                position_features: true,
                opening_principles: false, // Disable in WASM for size
                endgame_patterns: false,   // Disable in WASM for size
                tactical_patterns: false,  // Disable in WASM for size
                positional_patterns: false, // Disable in WASM for size
            }
        }
    }

    /// Check if feature should be enabled in WASM
    pub fn should_enable_feature_wasm(feature: &str) -> bool {
        match feature {
            "material" => true,
            "piece_square_tables" => true,
            "position_features" => !cfg!(target_arch = "wasm32"),
            "opening_principles" => false,
            "endgame_patterns" => false,
            "statistics" => false,
            "advanced_interpolation" => false,
            _ => true,
        }
    }

    /// Estimate total WASM binary size impact (KB)
    pub fn estimate_total_binary_impact() -> u32 {
        let config = WasmEvaluatorConfig::wasm_optimized();
        config.estimate_binary_size_impact()
    }
}

/// Create WASM-optimized integrated evaluator
#[cfg(target_arch = "wasm32")]
pub fn create_wasm_evaluator() -> crate::evaluation::integration::IntegratedEvaluator {
    use crate::evaluation::integration::IntegratedEvaluator;
    
    let wasm_config = WasmEvaluatorConfig::wasm_optimized();
    let config = wasm_config.to_integrated_config();
    
    IntegratedEvaluator::with_config(config)
}

/// Create native-optimized integrated evaluator
#[cfg(not(target_arch = "wasm32"))]
pub fn create_wasm_evaluator() -> crate::evaluation::integration::IntegratedEvaluator {
    use crate::evaluation::integration::IntegratedEvaluator;
    
    IntegratedEvaluator::new()
}

/// Platform-agnostic evaluator creation
pub fn create_platform_evaluator() -> crate::evaluation::integration::IntegratedEvaluator {
    create_wasm_evaluator()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_config_creation() {
        let config = WasmEvaluatorConfig::wasm_optimized();
        assert!(config.enable_wasm_optimizations);
        assert_eq!(config.max_wasm_memory, 16 * 1024 * 1024);
    }

    #[test]
    fn test_memory_constrained_config() {
        let config = WasmEvaluatorConfig::wasm_memory_constrained();
        assert_eq!(config.max_wasm_memory, 4 * 1024 * 1024);
        assert!(config.disable_heavy_features);
    }

    #[test]
    fn test_native_config() {
        let config = WasmEvaluatorConfig::native_optimized();
        assert!(!config.enable_wasm_optimizations);
        assert_eq!(config.max_wasm_memory, 100 * 1024 * 1024);
    }

    #[test]
    fn test_platform_detection() {
        let config = WasmEvaluatorConfig::platform_optimized();
        
        #[cfg(target_arch = "wasm32")]
        assert!(config.enable_wasm_optimizations);
        
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!config.enable_wasm_optimizations);
    }

    #[test]
    fn test_memory_estimation() {
        let config = WasmEvaluatorConfig::wasm_optimized();
        let memory = config.estimate_memory_usage();
        
        // Should be under max
        assert!(memory <= config.max_wasm_memory);
    }

    #[test]
    fn test_binary_size_estimation() {
        let config = WasmEvaluatorConfig::wasm_optimized();
        let size = config.estimate_binary_size_impact();
        
        // Should be reasonable (< 150KB)
        assert!(size < 150);
    }

    #[test]
    fn test_config_validation() {
        let config = WasmEvaluatorConfig::wasm_optimized();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = WasmEvaluatorConfig::wasm_optimized();
        config.max_wasm_memory = 0;
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_to_integrated_config() {
        let wasm_config = WasmEvaluatorConfig::wasm_optimized();
        let integrated = wasm_config.to_integrated_config();
        
        assert_eq!(integrated.max_cache_size, wasm_config.wasm_eval_cache_size);
    }

    #[test]
    fn test_wasm_utils_cache_size() {
        let cache_size = wasm_utils::get_optimal_wasm_cache_size(8);
        
        // Should be reasonable for 8MB
        assert!(cache_size <= 5000);
    }

    #[test]
    fn test_wasm_components() {
        let minimal = wasm_utils::get_wasm_components(true);
        assert!(minimal.material);
        assert!(!minimal.opening_principles);

        let full = wasm_utils::get_wasm_components(false);
        assert!(full.position_features);
    }

    #[test]
    fn test_feature_enablement() {
        assert!(wasm_utils::should_enable_feature_wasm("material"));
        assert!(wasm_utils::should_enable_feature_wasm("piece_square_tables"));
    }

    #[test]
    fn test_platform_evaluator_creation() {
        let evaluator = create_platform_evaluator();
        // Should create without errors
        drop(evaluator);
    }

    #[test]
    fn test_memory_constrained_validation() {
        let config = WasmEvaluatorConfig::wasm_memory_constrained();
        assert!(config.validate().is_ok());
        
        let memory = config.estimate_memory_usage();
        assert!(memory <= config.max_wasm_memory);
    }

    #[test]
    fn test_binary_size_with_heavy_features() {
        let mut config = WasmEvaluatorConfig::wasm_optimized();
        config.disable_heavy_features = false;
        
        let size_with = config.estimate_binary_size_impact();
        
        config.disable_heavy_features = true;
        let size_without = config.estimate_binary_size_impact();
        
        // Disabling heavy features should reduce binary size
        assert!(size_without < size_with);
    }
}

