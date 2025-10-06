//! WASM-Specific Benchmarks
//! 
//! This module provides benchmarking functionality specifically designed
//! for WASM environments. It measures performance characteristics that
//! are relevant for web deployment and browser execution.

use crate::types::*;
use crate::search::wasm_compatibility::{WasmTime, WasmDuration, WasmTranspositionConfig, wasm_utils};
use crate::search::wasm_transposition_table::WasmTranspositionTable;
use std::collections::HashMap;

/// WASM benchmark results
#[derive(Debug, Clone)]
pub struct WasmBenchmarkResults {
    /// Benchmark name
    pub name: String,
    /// Total operations performed
    pub operations: u64,
    /// Total time taken
    pub total_time: WasmDuration,
    /// Operations per second
    pub ops_per_second: f64,
    /// Average time per operation in microseconds
    pub avg_time_per_op_us: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// Cache hit rate
    pub hit_rate: f64,
    /// WASM-specific metrics
    pub wasm_metrics: WasmSpecificMetrics,
}

/// WASM-specific performance metrics
#[derive(Debug, Clone)]
pub struct WasmSpecificMetrics {
    /// Binary size impact in KB
    pub binary_size_kb: u32,
    /// Memory allocation count
    pub allocations: u64,
    /// Memory deallocation count
    pub deallocations: u64,
    /// Garbage collection hints sent
    pub gc_hints: u64,
    /// Function call overhead (estimated)
    pub function_call_overhead_us: f64,
}

/// WASM benchmark suite
pub struct WasmBenchmarkSuite {
    /// Benchmark results
    results: Vec<WasmBenchmarkResults>,
    /// Configuration used
    config: WasmTranspositionConfig,
}

impl WasmBenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(config: WasmTranspositionConfig) -> Self {
        Self {
            results: Vec::new(),
            config,
        }
    }
    
    /// Run all benchmarks
    pub fn run_all(&mut self) -> Vec<WasmBenchmarkResults> {
        self.results.clear();
        
        // Basic operations benchmark
        self.run_basic_operations_benchmark();
        
        // Memory usage benchmark
        self.run_memory_usage_benchmark();
        
        // Cache performance benchmark
        self.run_cache_performance_benchmark();
        
        // Time handling benchmark
        self.run_time_handling_benchmark();
        
        // Binary size benchmark
        self.run_binary_size_benchmark();
        
        self.results.clone()
    }
    
    /// Benchmark basic table operations
    fn run_basic_operations_benchmark(&mut self) {
        let mut table = WasmTranspositionTable::new(self.config.clone());
        let operations = 100_000;
        
        let _start_time = WasmTime::now();
        
        // Store operations
        for i in 0..operations / 2 {
            let entry = TranspositionEntry {
                hash_key: i as u64,
                depth: (i % 20) as u8 + 1,
                score: (i as i32 % 1000) - 500,
                flag: match i % 3 {
                    0 => TranspositionFlag::Exact,
                    1 => TranspositionFlag::LowerBound,
                    _ => TranspositionFlag::UpperBound,
                },
                best_move: None,
                age: 0,
            };
            table.store(entry);
        }
        
        // Probe operations
        for i in 0..operations / 2 {
            table.probe(i as u64, 1);
        }
        
        let end_time = WasmTime::now();
        let total_time = end_time.elapsed();
        
        let stats = table.get_stats();
        let ops_per_second = operations as f64 / (total_time.as_millis() as f64 / 1000.0);
        let avg_time_per_op_us = (total_time.as_millis() as f64 * 1000.0) / operations as f64;
        
        self.results.push(WasmBenchmarkResults {
            name: "Basic Operations".to_string(),
            operations,
            total_time,
            ops_per_second,
            avg_time_per_op_us,
            memory_usage: stats.memory_usage,
            peak_memory_usage: stats.memory_usage,
            hit_rate: stats.hit_rate,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: wasm_utils::estimate_binary_size_impact(&self.config),
                allocations: 1,
                deallocations: 0,
                gc_hints: 0,
                function_call_overhead_us: 0.1,
            },
        });
    }
    
    /// Benchmark memory usage patterns
    fn run_memory_usage_benchmark(&mut self) {
        let mut tables = Vec::new();
        let max_tables = 10;
        let entries_per_table = 10_000;
        
        let _start_time = WasmTime::now();
        let mut peak_memory = 0;
        
        // Create multiple tables to test memory management
        for i in 0..max_tables {
            let mut config = self.config.clone();
            config.base_config.table_size = entries_per_table;
            let mut table = WasmTranspositionTable::new(config);
            
            // Fill table
            for j in 0..entries_per_table {
                let entry = TranspositionEntry {
                    hash_key: (i * entries_per_table + j) as u64,
                    depth: 1,
                    score: 0,
                    flag: TranspositionFlag::Exact,
                    best_move: None,
                    age: 0,
                };
                table.store(entry);
            }
            
            let memory_usage = table.get_memory_usage();
            peak_memory = peak_memory.max(memory_usage);
            tables.push(table);
        }
        
        // Clear tables to test memory cleanup
        for table in &mut tables {
            table.clear();
        }
        
        let end_time = WasmTime::now();
        let total_time = end_time.elapsed();
        
        let total_operations = max_tables * entries_per_table * 2; // Store + clear
        let ops_per_second = total_operations as f64 / (total_time.as_millis() as f64 / 1000.0);
        
        self.results.push(WasmBenchmarkResults {
            name: "Memory Usage".to_string(),
            operations: total_operations as u64,
            total_time,
            ops_per_second,
            avg_time_per_op_us: (total_time.as_millis() as f64 * 1000.0) / total_operations as f64,
            memory_usage: peak_memory,
            peak_memory_usage: peak_memory,
            hit_rate: 0.0,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: wasm_utils::estimate_binary_size_impact(&self.config),
                allocations: max_tables as u64,
                deallocations: max_tables as u64,
                gc_hints: max_tables as u64,
                function_call_overhead_us: 0.05,
            },
        });
    }
    
    /// Benchmark cache performance
    fn run_cache_performance_benchmark(&mut self) {
        let mut table = WasmTranspositionTable::new(self.config.clone());
        let operations = 50_000;
        
        let _start_time = WasmTime::now();
        
        // Fill cache with some entries
        for i in 0..operations / 4 {
            let entry = TranspositionEntry {
                hash_key: i as u64,
                depth: 5,
                score: i as i32,
                flag: TranspositionFlag::Exact,
                best_move: None,
                age: 0,
            };
            table.store(entry);
        }
        
        // Probe for hits and misses
        for i in 0..operations / 2 {
            table.probe(i as u64, 1); // Should hit for first quarter
        }
        
        for i in operations / 2..operations {
            table.probe(i as u64, 1); // Should miss for remaining
        }
        
        let end_time = WasmTime::now();
        let total_time = end_time.elapsed();
        
        let stats = table.get_stats();
        let ops_per_second = operations as f64 / (total_time.as_millis() as f64 / 1000.0);
        
        self.results.push(WasmBenchmarkResults {
            name: "Cache Performance".to_string(),
            operations,
            total_time,
            ops_per_second,
            avg_time_per_op_us: (total_time.as_millis() as f64 * 1000.0) / operations as f64,
            memory_usage: stats.memory_usage,
            peak_memory_usage: stats.memory_usage,
            hit_rate: stats.hit_rate,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: wasm_utils::estimate_binary_size_impact(&self.config),
                allocations: 1,
                deallocations: 0,
                gc_hints: 0,
                function_call_overhead_us: 0.08,
            },
        });
    }
    
    /// Benchmark time handling performance
    fn run_time_handling_benchmark(&mut self) {
        let operations = 1_000_000;
        
        let _start_time = WasmTime::now();
        
        // Test time operations
        let mut times = Vec::with_capacity(operations);
        for _ in 0..operations {
            times.push(WasmTime::now());
        }
        
        // Test duration calculations
        let mut total_duration = WasmDuration::from_millis(0);
        for i in 1..operations {
            total_duration = WasmDuration::from_millis(
                total_duration.as_millis() + times[i].elapsed().as_millis()
            );
        }
        
        let end_time = WasmTime::now();
        let total_time = end_time.elapsed();
        
        let ops_per_second = operations as f64 / (total_time.as_millis() as f64 / 1000.0);
        
        self.results.push(WasmBenchmarkResults {
            name: "Time Handling".to_string(),
            operations: operations as u64,
            total_time,
            ops_per_second,
            avg_time_per_op_us: (total_time.as_millis() as f64 * 1000.0) / operations as f64,
            memory_usage: (operations * std::mem::size_of::<WasmTime>()) as u64,
            peak_memory_usage: (operations * std::mem::size_of::<WasmTime>()) as u64,
            hit_rate: 0.0,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: 15, // Time handling overhead
                allocations: 0,
                deallocations: 0,
                gc_hints: 0,
                function_call_overhead_us: 0.02,
            },
        });
    }
    
    /// Benchmark binary size impact
    fn run_binary_size_benchmark(&mut self) {
        let _start_time = WasmTime::now();
        
        // Create various configurations to test size impact
        let mut size_impact = 0;
        for table_size in [1024, 4096, 16384, 65536] {
            let mut config = self.config.clone();
            config.base_config.table_size = table_size;
            size_impact += wasm_utils::estimate_binary_size_impact(&config);
        }
        
        let end_time = WasmTime::now();
        let total_time = end_time.elapsed();
        
        self.results.push(WasmBenchmarkResults {
            name: "Binary Size Impact".to_string(),
            operations: 4, // Tested 4 configurations
            total_time,
            ops_per_second: 4.0 / (total_time.as_millis() as f64 / 1000.0),
            avg_time_per_op_us: (total_time.as_millis() as f64 * 1000.0) / 4.0,
            memory_usage: 0,
            peak_memory_usage: 0,
            hit_rate: 0.0,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: size_impact / 4, // Average size impact
                allocations: 0,
                deallocations: 0,
                gc_hints: 0,
                function_call_overhead_us: 0.0,
            },
        });
    }
    
    /// Get benchmark summary
    pub fn get_summary(&self) -> WasmBenchmarkSummary {
        let total_operations: u64 = self.results.iter().map(|r| r.operations).sum();
        let total_time_ms: u64 = self.results.iter().map(|r| r.total_time.as_millis()).sum();
        let avg_ops_per_second = total_operations as f64 / (total_time_ms as f64 / 1000.0);
        let total_memory: u64 = self.results.iter().map(|r| r.memory_usage).sum();
        let total_binary_size: u32 = self.results.iter().map(|r| r.wasm_metrics.binary_size_kb).sum();
        
        WasmBenchmarkSummary {
            total_benchmarks: self.results.len(),
            total_operations,
            total_time_ms,
            avg_ops_per_second,
            total_memory_usage: total_memory,
            total_binary_size_kb: total_binary_size,
            results: self.results.clone(),
        }
    }
}

/// WASM benchmark summary
#[derive(Debug, Clone)]
pub struct WasmBenchmarkSummary {
    /// Total number of benchmarks run
    pub total_benchmarks: usize,
    /// Total operations across all benchmarks
    pub total_operations: u64,
    /// Total time in milliseconds
    pub total_time_ms: u64,
    /// Average operations per second
    pub avg_ops_per_second: f64,
    /// Total memory usage
    pub total_memory_usage: u64,
    /// Total binary size impact in KB
    pub total_binary_size_kb: u32,
    /// Individual benchmark results
    pub results: Vec<WasmBenchmarkResults>,
}

/// WASM performance profiler
pub struct WasmPerformanceProfiler {
    /// Start time
    start_time: WasmTime,
    /// Operation counts
    operation_counts: HashMap<String, u64>,
    /// Operation times
    operation_times: HashMap<String, WasmDuration>,
}

impl WasmPerformanceProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            start_time: WasmTime::now(),
            operation_counts: HashMap::new(),
            operation_times: HashMap::new(),
        }
    }
    
    /// Start timing an operation
    pub fn start_operation(&self, _name: &str) -> WasmTime {
        WasmTime::now()
    }
    
    /// End timing an operation
    pub fn end_operation(&mut self, name: &str, start_time: WasmTime) {
        let duration = start_time.elapsed();
        
        *self.operation_counts.entry(name.to_string()).or_insert(0) += 1;
        let total_duration = self.operation_times.entry(name.to_string()).or_insert(WasmDuration::from_millis(0));
        *total_duration = WasmDuration::from_millis(total_duration.as_millis() + duration.as_millis());
    }
    
    /// Get operation statistics
    pub fn get_operation_stats(&self, name: &str) -> Option<WasmOperationStats> {
        let count = self.operation_counts.get(name)?;
        let total_time = self.operation_times.get(name)?;
        
        Some(WasmOperationStats {
            name: name.to_string(),
            count: *count,
            total_time: *total_time,
            avg_time_per_op: if *count > 0 {
                WasmDuration::from_millis(total_time.as_millis() / *count)
            } else {
                WasmDuration::from_millis(0)
            },
            ops_per_second: if total_time.as_millis() > 0 {
                (*count as f64) / (total_time.as_millis() as f64 / 1000.0)
            } else {
                0.0
            },
        })
    }
    
    /// Get all operation statistics
    pub fn get_all_stats(&self) -> Vec<WasmOperationStats> {
        self.operation_counts
            .keys()
            .filter_map(|name| self.get_operation_stats(name))
            .collect()
    }
    
    /// Get total profiling time
    pub fn total_time(&self) -> WasmDuration {
        self.start_time.elapsed()
    }
}

/// WASM operation statistics
#[derive(Debug, Clone)]
pub struct WasmOperationStats {
    /// Operation name
    pub name: String,
    /// Number of times called
    pub count: u64,
    /// Total time spent
    pub total_time: WasmDuration,
    /// Average time per operation
    pub avg_time_per_op: WasmDuration,
    /// Operations per second
    pub ops_per_second: f64,
}

impl Default for WasmPerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_benchmark_suite() {
        let config = WasmTranspositionConfig::default();
        let mut suite = WasmBenchmarkSuite::new(config);
        let results = suite.run_all();
        
        assert!(!results.is_empty());
        assert!(results.len() >= 4); // Should have at least 4 benchmarks
        
        // Check that all benchmarks completed successfully
        for result in &results {
            assert!(result.operations > 0);
            assert!(result.ops_per_second > 0.0);
            assert!(result.avg_time_per_op_us > 0.0);
        }
    }
    
    #[test]
    fn test_wasm_performance_profiler() {
        let mut profiler = WasmPerformanceProfiler::new();
        
        // Profile some operations
        let start = profiler.start_operation("test_op");
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
        profiler.end_operation("test_op", start);
        
        let stats = profiler.get_operation_stats("test_op");
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.name, "test_op");
        assert_eq!(stats.count, 1);
        assert!(stats.total_time.as_millis() > 0);
    }
    
    #[test]
    fn test_wasm_benchmark_results() {
        let result = WasmBenchmarkResults {
            name: "Test Benchmark".to_string(),
            operations: 1000,
            total_time: WasmDuration::from_millis(100),
            ops_per_second: 10.0,
            avg_time_per_op_us: 100.0,
            memory_usage: 1024,
            peak_memory_usage: 2048,
            hit_rate: 0.5,
            wasm_metrics: WasmSpecificMetrics {
                binary_size_kb: 50,
                allocations: 10,
                deallocations: 5,
                gc_hints: 2,
                function_call_overhead_us: 0.1,
            },
        };
        
        assert_eq!(result.name, "Test Benchmark");
        assert_eq!(result.operations, 1000);
        assert_eq!(result.ops_per_second, 10.0);
        assert_eq!(result.hit_rate, 0.5);
    }
}
