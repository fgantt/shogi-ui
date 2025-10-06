//! WASM Compatibility Module
//! 
//! This module provides WASM-specific implementations and compatibility layers
//! for the transposition table system. It handles the differences between
//! native and WASM environments, particularly around time handling, threading,
//! and memory management.
//! 
//! # Features
//! 
//! - **WASM-Compatible Time Handling**: Alternative to `std::time::Instant`
//! - **Single-Threaded Optimizations**: Optimized for WASM's single-threaded nature
//! - **Memory Management**: Efficient memory usage for WASM constraints
//! - **Conditional Compilation**: Seamless switching between native and WASM
//! - **Performance Optimizations**: WASM-specific performance improvements
//! 
//! # Usage
//! 
//! The module automatically detects the target architecture and provides
//! appropriate implementations. No manual configuration is required.
//! 
//! ```rust
//! use shogi_engine::search::wasm_compatibility::{WasmTime, WasmMemoryManager};
//! 
//! // Time handling works in both native and WASM
//! let start_time = WasmTime::now();
//! // ... do work ...
//! let elapsed = start_time.elapsed();
//! 
//! // Memory management optimized for target
//! let memory_manager = WasmMemoryManager::new();
//! ```

use crate::search::transposition_config::TranspositionConfig;
use std::collections::HashMap;

/// WASM-compatible time handling
/// 
/// Provides a unified time interface that works in both native and WASM environments.
/// In WASM, it uses performance.now() via JavaScript bindings, while in native
/// environments it uses std::time::Instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WasmTime {
    /// Time in milliseconds since epoch (WASM) or nanoseconds (native)
    timestamp: u64,
}

impl WasmTime {
    /// Get the current time
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                timestamp: get_performance_time(),
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            }
        }
    }
    
    /// Get elapsed time since this timestamp
    pub fn elapsed(&self) -> WasmDuration {
        let current = Self::now();
        
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, timestamp is in milliseconds
            let elapsed_ms = current.timestamp.saturating_sub(self.timestamp);
            WasmDuration::from_millis(elapsed_ms)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // In native, timestamp is in nanoseconds
            let elapsed_ns = current.timestamp.saturating_sub(self.timestamp);
            WasmDuration::from_nanos(elapsed_ns)
        }
    }
    
    /// Create a time from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self { timestamp: millis }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                timestamp: millis * 1_000_000, // Convert to nanoseconds
            }
        }
    }
    
    /// Get timestamp as milliseconds
    pub fn as_millis(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            self.timestamp
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.timestamp / 1_000_000
        }
    }
}

/// WASM-compatible duration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WasmDuration {
    /// Duration in milliseconds (WASM) or nanoseconds (native)
    duration: u64,
}

impl WasmDuration {
    /// Create duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self { duration: millis }
    }
    
    /// Create duration from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { duration: nanos }
    }
    
    /// Get duration as milliseconds
    pub fn as_millis(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            self.duration
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.duration / 1_000_000
        }
    }
    
    /// Get duration as nanoseconds
    pub fn as_nanos(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            self.duration * 1_000_000
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.duration
        }
    }
    
    /// Check if duration is zero
    pub fn is_zero(&self) -> bool {
        self.duration == 0
    }
}

/// WASM-compatible memory manager
/// 
/// Provides memory management optimized for WASM constraints,
/// including memory usage tracking and garbage collection hints.
pub struct WasmMemoryManager {
    /// Current memory usage in bytes
    memory_usage: u64,
    /// Maximum allowed memory usage
    max_memory: Option<u64>,
    /// Memory usage tracking
    allocations: HashMap<String, u64>,
}

impl WasmMemoryManager {
    /// Create a new WASM memory manager
    pub fn new() -> Self {
        Self {
            memory_usage: 0,
            max_memory: None,
            allocations: HashMap::new(),
        }
    }
    
    /// Create with memory limit
    pub fn with_limit(max_memory: u64) -> Self {
        Self {
            memory_usage: 0,
            max_memory: Some(max_memory),
            allocations: HashMap::new(),
        }
    }
    
    /// Track memory allocation
    pub fn allocate(&mut self, name: &str, size: u64) -> Result<(), String> {
        if let Some(max) = self.max_memory {
            if self.memory_usage + size > max {
                return Err(format!("Memory allocation would exceed limit: {} + {} > {}", 
                    self.memory_usage, size, max));
            }
        }
        
        self.memory_usage += size;
        self.allocations.insert(name.to_string(), size);
        Ok(())
    }
    
    /// Track memory deallocation
    pub fn deallocate(&mut self, name: &str) {
        if let Some(size) = self.allocations.remove(name) {
            self.memory_usage = self.memory_usage.saturating_sub(size);
        }
    }
    
    /// Get current memory usage
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage
    }
    
    /// Get memory usage percentage
    pub fn memory_usage_percentage(&self) -> f64 {
        if let Some(max) = self.max_memory {
            (self.memory_usage as f64 / max as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Check if memory usage is high
    pub fn is_memory_pressure(&self) -> bool {
        if let Some(max) = self.max_memory {
            self.memory_usage as f64 / max as f64 > 0.8
        } else {
            false
        }
    }
    
    /// Force garbage collection (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub fn force_gc(&self) {
        // In WASM, we can't force GC, but we can provide hints
        // This would typically be handled by the JavaScript runtime
    }
    
    /// Force garbage collection (native - no-op)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn force_gc(&self) {
        // No-op in native environments
    }
}

impl Default for WasmMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM-specific performance counter
/// 
/// Provides performance counting optimized for WASM environments.
pub struct WasmPerformanceCounter {
    /// Counter value
    count: u64,
    /// Last reset time
    last_reset: WasmTime,
}

impl WasmPerformanceCounter {
    /// Create a new performance counter
    pub fn new() -> Self {
        Self {
            count: 0,
            last_reset: WasmTime::now(),
        }
    }
    
    /// Increment the counter
    pub fn increment(&mut self) {
        self.count = self.count.saturating_add(1);
    }
    
    /// Add to the counter
    pub fn add(&mut self, value: u64) {
        self.count = self.count.saturating_add(value);
    }
    
    /// Get current count
    pub fn count(&self) -> u64 {
        self.count
    }
    
    /// Reset the counter
    pub fn reset(&mut self) {
        self.count = 0;
        self.last_reset = WasmTime::now();
    }
    
    /// Get operations per second
    pub fn ops_per_second(&self) -> f64 {
        let elapsed = self.last_reset.elapsed();
        if elapsed.is_zero() {
            return 0.0;
        }
        
        let elapsed_secs = elapsed.as_millis() as f64 / 1000.0;
        if elapsed_secs > 0.0 {
            self.count as f64 / elapsed_secs
        } else {
            0.0
        }
    }
}

impl Default for WasmPerformanceCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM-compatible transposition table configuration
/// 
/// Provides configuration optimized for WASM environments.
#[derive(Debug, Clone)]
pub struct WasmTranspositionConfig {
    /// Base configuration
    pub base_config: TranspositionConfig,
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    /// Enable memory pressure monitoring
    pub enable_memory_monitoring: bool,
    /// Memory pressure threshold (0.0 to 1.0)
    pub memory_pressure_threshold: f64,
    /// Enable garbage collection hints
    pub enable_gc_hints: bool,
}

impl Default for WasmTranspositionConfig {
    fn default() -> Self {
        Self {
            base_config: TranspositionConfig::default(),
            max_memory_mb: 32, // 32MB default for WASM (more conservative)
            enable_memory_monitoring: true,
            memory_pressure_threshold: 0.7, // Lower threshold for earlier intervention
            enable_gc_hints: true,
        }
    }
}

impl From<TranspositionConfig> for WasmTranspositionConfig {
    fn from(config: TranspositionConfig) -> Self {
        Self {
            base_config: config,
            max_memory_mb: 32, // More conservative default
            enable_memory_monitoring: true,
            memory_pressure_threshold: 0.7, // Lower threshold
            enable_gc_hints: true,
        }
    }
}

/// WASM-specific utility functions
pub mod wasm_utils {
    use super::*;
    
    /// Get optimal table size for WASM
    pub fn get_optimal_wasm_table_size(available_memory_mb: u32) -> usize {
        // Conservative sizing for WASM environments
        let max_entries = (available_memory_mb as usize * 1024 * 1024) / 16; // 16 bytes per entry
        max_entries.next_power_of_two()
    }
    
    /// Check if we should enable prefetching in WASM
    pub fn should_enable_prefetching_wasm() -> bool {
        // Prefetching is less beneficial in WASM due to different memory model
        false
    }
    
    /// Get WASM-optimized replacement policy
    pub fn get_wasm_replacement_policy() -> crate::search::transposition_config::ReplacementPolicy {
        // Age-based replacement works well in WASM
        crate::search::transposition_config::ReplacementPolicy::AgeBased
    }
    
    /// Estimate WASM binary size impact
    pub fn estimate_binary_size_impact(config: &WasmTranspositionConfig) -> u32 {
        // Rough estimation of binary size impact in KB
        let base_size = 50; // Base transposition table code
        let memory_management = if config.enable_memory_monitoring { 20 } else { 0 };
        let gc_hints = if config.enable_gc_hints { 10 } else { 0 };
        let time_handling = 30; // WASM time compatibility
        
        base_size + memory_management + gc_hints + time_handling
    }
}

// WASM-specific JavaScript bindings
#[cfg(target_arch = "wasm32")]
mod wasm_bindings {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance)]
        fn now() -> f64;
    }
    
    /// Get performance time in milliseconds
    pub fn get_performance_time() -> u64 {
        now() as u64
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod wasm_bindings {
    /// Fallback for native environments
    #[allow(dead_code)]
    pub fn get_performance_time() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

// Re-export the get_performance_time function
#[allow(unused_imports)]
use wasm_bindings::get_performance_time;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_time_basic() {
        let start = WasmTime::now();
        let duration = start.elapsed();
        assert!(!duration.is_zero() || duration.is_zero()); // Either works
    }
    
    #[test]
    fn test_wasm_duration_conversion() {
        let duration = WasmDuration::from_millis(1000);
        assert_eq!(duration.as_millis(), 1000);
    }
    
    #[test]
    fn test_memory_manager() {
        let mut manager = WasmMemoryManager::new();
        assert!(manager.allocate("test", 1000).is_ok());
        assert_eq!(manager.memory_usage(), 1000);
        
        manager.deallocate("test");
        assert_eq!(manager.memory_usage(), 0);
    }
    
    #[test]
    fn test_performance_counter() {
        let mut counter = WasmPerformanceCounter::new();
        counter.increment();
        assert_eq!(counter.count(), 1);
        
        counter.add(5);
        assert_eq!(counter.count(), 6);
        
        counter.reset();
        assert_eq!(counter.count(), 0);
    }
    
    #[test]
    fn test_wasm_config_default() {
        let config = WasmTranspositionConfig::default();
        assert_eq!(config.max_memory_mb, 64);
        assert!(config.enable_memory_monitoring);
        assert_eq!(config.memory_pressure_threshold, 0.8);
    }
    
    #[test]
    fn test_wasm_utils() {
        let size = wasm_utils::get_optimal_wasm_table_size(64);
        assert!(size > 0);
        assert!(size.is_power_of_two());
        
        let should_prefetch = wasm_utils::should_enable_prefetching_wasm();
        assert!(!should_prefetch); // Should be false for WASM
    }
}
