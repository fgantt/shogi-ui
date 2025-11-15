//! Performance benchmarks for transposition table optimizations
//!
//! This module provides comprehensive benchmarking tools to measure and validate
//! the performance improvements from the optimization systems.

use crate::search::performance_optimization::*;
use crate::search::thread_safe_table::ThreadSafeTranspositionTable;
use crate::search::transposition_config::TranspositionConfig;
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::time::{Duration, Instant};

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Operation name
    pub operation: String,
    /// Number of operations performed
    pub operations: u64,
    /// Total time taken
    pub total_time: Duration,
    /// Average time per operation (nanoseconds)
    pub avg_time_ns: u64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Memory usage (bytes)
    pub memory_usage: usize,
    /// Cache hit rate (if applicable)
    pub cache_hit_rate: Option<f64>,
}

/// Performance benchmark suite
pub struct PerformanceBenchmarks {
    /// Table size for benchmarks
    table_size: usize,
    /// Number of test operations
    operation_count: u64,
}

impl PerformanceBenchmarks {
    /// Create a new benchmark suite
    pub fn new(table_size: usize, operation_count: u64) -> Self {
        Self {
            table_size,
            operation_count,
        }
    }

    /// Run hash mapping benchmark
    pub fn benchmark_hash_mapping(&self) -> BenchmarkResults {
        let mapper = OptimizedHashMapper::new(self.table_size);
        let test_hashes: Vec<u64> = (0..self.operation_count)
            .map(|i| (i as u64).wrapping_mul(0x9E3779B97F4A7C15))
            .collect();

        let start = Instant::now();

        for hash in &test_hashes {
            let _index = mapper.hash_to_index(*hash);
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();

        BenchmarkResults {
            operation: "Hash Mapping".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: 0, // Hash mapping doesn't use additional memory
            cache_hit_rate: None,
        }
    }

    /// Run entry packing benchmark
    pub fn benchmark_entry_packing(&self) -> BenchmarkResults {
        let test_entries: Vec<(i32, u8, TranspositionFlag)> = (0..self.operation_count)
            .map(|i| {
                (
                    (i as i32) % 1000 - 500, // Score range: -500 to 499
                    (i as u8) % 20,          // Depth range: 0 to 19
                    match i % 3 {
                        0 => TranspositionFlag::Exact,
                        1 => TranspositionFlag::LowerBound,
                        _ => TranspositionFlag::UpperBound,
                    },
                )
            })
            .collect();

        let start = Instant::now();

        for &(score, depth, flag) in &test_entries {
            let _packed = OptimizedEntryPacker::pack_entry_fast(score, depth, flag);
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();

        BenchmarkResults {
            operation: "Entry Packing".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: 0,
            cache_hit_rate: None,
        }
    }

    /// Run entry unpacking benchmark
    pub fn benchmark_entry_unpacking(&self) -> BenchmarkResults {
        let test_packed: Vec<u64> = (0..self.operation_count)
            .map(|i| {
                OptimizedEntryPacker::pack_entry_fast(
                    (i as i32) % 1000 - 500,
                    (i as u8) % 20,
                    match i % 3 {
                        0 => TranspositionFlag::Exact,
                        1 => TranspositionFlag::LowerBound,
                        _ => TranspositionFlag::UpperBound,
                    },
                )
            })
            .collect();

        let start = Instant::now();

        for packed in &test_packed {
            let _unpacked = OptimizedEntryPacker::unpack_entry_fast(*packed);
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();

        BenchmarkResults {
            operation: "Entry Unpacking".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: 0,
            cache_hit_rate: None,
        }
    }

    /// Run thread-safe table probe benchmark
    pub fn benchmark_table_probe(&self) -> BenchmarkResults {
        let config = TranspositionConfig::debug_config();
        let table = ThreadSafeTranspositionTable::new(config);

        // Pre-populate table with some entries
        for i in 0..1000 {
            let entry = TranspositionEntry::new_with_age(
                (i as i32) % 1000 - 500,
                (i as u8) % 20,
                match i % 3 {
                    0 => TranspositionFlag::Exact,
                    1 => TranspositionFlag::LowerBound,
                    _ => TranspositionFlag::UpperBound,
                },
                None,
                i as u64,
            );
            table.store(entry);
        }

        let test_hashes: Vec<u64> = (0..self.operation_count)
            .map(|i| (i as u64) % 1000)
            .collect();

        let mut hits = 0;
        let start = Instant::now();

        for hash in &test_hashes {
            if table.probe(*hash, 5).is_some() {
                hits += 1;
            }
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();
        let hit_rate = hits as f64 / self.operation_count as f64;

        BenchmarkResults {
            operation: "Table Probe".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: table.size() * std::mem::size_of::<TranspositionEntry>(),
            cache_hit_rate: Some(hit_rate),
        }
    }

    /// Run thread-safe table store benchmark
    pub fn benchmark_table_store(&self) -> BenchmarkResults {
        let config = TranspositionConfig::debug_config();
        let table = ThreadSafeTranspositionTable::new(config);

        let test_entries: Vec<TranspositionEntry> = (0..self.operation_count)
            .map(|i| {
                TranspositionEntry::new_with_age(
                    (i as i32) % 1000 - 500,
                    (i as u8) % 20,
                    match i % 3 {
                        0 => TranspositionFlag::Exact,
                        1 => TranspositionFlag::LowerBound,
                        _ => TranspositionFlag::UpperBound,
                    },
                    None,
                    i as u64,
                )
            })
            .collect();

        let start = Instant::now();

        for entry in &test_entries {
            table.store(entry.clone());
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();

        BenchmarkResults {
            operation: "Table Store".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: table.size() * std::mem::size_of::<TranspositionEntry>(),
            cache_hit_rate: None,
        }
    }

    /// Run hot path optimizer benchmark
    pub fn benchmark_hot_path(&self) -> BenchmarkResults {
        let mut optimizer = HotPathOptimizer::new(self.table_size);
        let mut entries =
            vec![AtomicPackedEntry::new(0, 0, TranspositionFlag::Exact, None); self.table_size];

        let test_entries: Vec<TranspositionEntry> = (0..self.operation_count)
            .map(|i| {
                TranspositionEntry::new_with_age(
                    (i as i32) % 1000 - 500,
                    (i as u8) % 20,
                    match i % 3 {
                        0 => TranspositionFlag::Exact,
                        1 => TranspositionFlag::LowerBound,
                        _ => TranspositionFlag::UpperBound,
                    },
                    None,
                    i as u64,
                )
            })
            .collect();

        // Store entries first
        for entry in &test_entries {
            optimizer.fast_store(entry.clone(), &mut entries);
        }

        let start = Instant::now();

        // Probe entries
        let mut hits = 0;
        for entry in &test_entries {
            if optimizer
                .fast_probe(entry.hash_key, entry.depth, &entries)
                .is_some()
            {
                hits += 1;
            }
        }

        let total_time = start.elapsed();
        let avg_time_ns = total_time.as_nanos() / self.operation_count as u128;
        let ops_per_second = self.operation_count as f64 / total_time.as_secs_f64();
        let hit_rate = hits as f64 / self.operation_count as f64;

        BenchmarkResults {
            operation: "Hot Path Optimizer".to_string(),
            operations: self.operation_count,
            total_time,
            avg_time_ns: avg_time_ns as u64,
            ops_per_second,
            memory_usage: self.table_size * std::mem::size_of::<AtomicPackedEntry>(),
            cache_hit_rate: Some(hit_rate),
        }
    }

    /// Run complete benchmark suite
    pub fn run_all_benchmarks(&self) -> Vec<BenchmarkResults> {
        vec![
            self.benchmark_hash_mapping(),
            self.benchmark_entry_packing(),
            self.benchmark_entry_unpacking(),
            self.benchmark_table_probe(),
            self.benchmark_table_store(),
            self.benchmark_hot_path(),
        ]
    }

    /// Compare benchmark results
    pub fn compare_results(&self, results: &[BenchmarkResults]) -> BenchmarkComparison {
        let mut comparison = BenchmarkComparison {
            operations: Vec::new(),
            avg_times: Vec::new(),
            ops_per_second: Vec::new(),
            memory_usage: Vec::new(),
            cache_hit_rates: Vec::new(),
        };

        for result in results {
            comparison.operations.push(result.operation.clone());
            comparison.avg_times.push(result.avg_time_ns);
            comparison.ops_per_second.push(result.ops_per_second);
            comparison.memory_usage.push(result.memory_usage);
            comparison.cache_hit_rates.push(result.cache_hit_rate);
        }

        comparison
    }
}

/// Benchmark comparison results
#[derive(Debug, Clone)]
pub struct BenchmarkComparison {
    pub operations: Vec<String>,
    pub avg_times: Vec<u64>,
    pub ops_per_second: Vec<f64>,
    pub memory_usage: Vec<usize>,
    pub cache_hit_rates: Vec<Option<f64>>,
}

impl BenchmarkComparison {
    /// Print benchmark results in a formatted table
    pub fn print_results(&self) {
        println!("┌─────────────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
        println!("│ Operation               │ Avg Time (ns)│ Ops/Second   │ Memory (MB)  │ Hit Rate (%) │");
        println!("├─────────────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");

        for i in 0..self.operations.len() {
            let operation = &self.operations[i];
            let avg_time = self.avg_times[i];
            let ops_per_sec = self.ops_per_second[i];
            let memory_mb = self.memory_usage[i] as f64 / (1024.0 * 1024.0);
            let hit_rate = self.cache_hit_rates[i]
                .map(|r| format!("{:.1}", r * 100.0))
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "│ {:23} │ {:12} │ {:12.0} │ {:12.2} │ {:12} │",
                operation, avg_time, ops_per_sec, memory_mb, hit_rate
            );
        }

        println!("└─────────────────────────┴──────────────┴──────────────┴──────────────┴──────────────┘");
    }

    /// Find the fastest operation
    pub fn fastest_operation(&self) -> Option<&String> {
        self.avg_times
            .iter()
            .enumerate()
            .min_by_key(|(_, &time)| time)
            .map(|(i, _)| &self.operations[i])
    }

    /// Find the slowest operation
    pub fn slowest_operation(&self) -> Option<&String> {
        self.avg_times
            .iter()
            .enumerate()
            .max_by_key(|(_, &time)| time)
            .map(|(i, _)| &self.operations[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_mapping_benchmark() {
        let benchmarks = PerformanceBenchmarks::new(1024, 10000);
        let results = benchmarks.benchmark_hash_mapping();

        assert_eq!(results.operation, "Hash Mapping");
        assert_eq!(results.operations, 10000);
        assert!(results.ops_per_second > 0.0);
    }

    #[test]
    fn test_entry_packing_benchmark() {
        let benchmarks = PerformanceBenchmarks::new(1024, 10000);
        let results = benchmarks.benchmark_entry_packing();

        assert_eq!(results.operation, "Entry Packing");
        assert_eq!(results.operations, 10000);
        assert!(results.ops_per_second > 0.0);
    }

    #[test]
    fn test_benchmark_comparison() {
        let benchmarks = PerformanceBenchmarks::new(1024, 1000);
        let results = benchmarks.run_all_benchmarks();
        let comparison = benchmarks.compare_results(&results);

        assert!(!comparison.operations.is_empty());
        assert_eq!(comparison.operations.len(), comparison.avg_times.len());

        // Print results for manual inspection
        comparison.print_results();
    }
}
