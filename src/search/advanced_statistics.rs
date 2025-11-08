//! Advanced statistics for transposition table operations
//!
//! This module provides detailed cache statistics, hit rate tracking by depth,
//! collision monitoring, statistics export, visualization, and performance
//! trend analysis for comprehensive transposition table performance monitoring.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Detailed cache statistics
#[derive(Debug, Clone)]
pub struct DetailedCacheStats {
    /// Total number of probes
    pub total_probes: u64,
    /// Total number of hits
    pub total_hits: u64,
    /// Total number of stores
    pub total_stores: u64,
    /// Total number of replacements
    pub total_replacements: u64,
    /// Total number of collisions
    pub total_collisions: u64,
    /// Current table occupancy (0.0 to 1.0)
    pub occupancy_rate: f64,
    /// Average probe time (microseconds)
    pub avg_probe_time_us: f64,
    /// Average store time (microseconds)
    pub avg_store_time_us: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
    /// Hash distribution quality (0.0 to 1.0)
    pub hash_distribution_quality: f64,
}

impl Default for DetailedCacheStats {
    fn default() -> Self {
        Self {
            total_probes: 0,
            total_hits: 0,
            total_stores: 0,
            total_replacements: 0,
            total_collisions: 0,
            occupancy_rate: 0.0,
            avg_probe_time_us: 0.0,
            avg_store_time_us: 0.0,
            memory_usage_bytes: 0,
            hash_distribution_quality: 1.0,
        }
    }
}

/// Hit rate statistics by depth
#[derive(Debug)]
pub struct HitRateByDepth {
    /// Maximum depth to track
    max_depth: u8,
    /// Hit counts by depth
    hit_counts: Vec<AtomicU64>,
    /// Probe counts by depth
    probe_counts: Vec<AtomicU64>,
    /// Hit rates by depth (cached)
    hit_rates: Vec<f64>,
}

impl HitRateByDepth {
    /// Create a new hit rate tracker
    pub fn new(max_depth: u8) -> Self {
        Self {
            max_depth,
            hit_counts: (0..=max_depth as usize)
                .map(|_| AtomicU64::new(0))
                .collect(),
            probe_counts: (0..=max_depth as usize)
                .map(|_| AtomicU64::new(0))
                .collect(),
            hit_rates: vec![0.0; max_depth as usize + 1],
        }
    }

    /// Record a probe at the given depth
    pub fn record_probe(&self, depth: u8, hit: bool) {
        let depth_idx = (depth.min(self.max_depth)) as usize;
        self.probe_counts[depth_idx].fetch_add(1, Ordering::Relaxed);
        if hit {
            self.hit_counts[depth_idx].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Update cached hit rates
    pub fn update_hit_rates(&mut self) {
        for depth in 0..=self.max_depth as usize {
            let probes = self.probe_counts[depth].load(Ordering::Acquire);
            let hits = self.hit_counts[depth].load(Ordering::Acquire);

            self.hit_rates[depth] = if probes > 0 {
                hits as f64 / probes as f64
            } else {
                0.0
            };
        }
    }

    /// Get hit rate for a specific depth
    pub fn get_hit_rate(&self, depth: u8) -> f64 {
        let depth_idx = depth.min(self.max_depth) as usize;
        self.hit_rates[depth_idx]
    }

    /// Get all hit rates
    pub fn get_all_hit_rates(&self) -> Vec<(u8, f64)> {
        (0..=self.max_depth)
            .map(|depth| (depth, self.get_hit_rate(depth)))
            .collect()
    }

    /// Get total hit rate across all depths
    pub fn get_total_hit_rate(&self) -> f64 {
        let total_hits: u64 = self
            .hit_counts
            .iter()
            .map(|c| c.load(Ordering::Acquire))
            .sum();
        let total_probes: u64 = self
            .probe_counts
            .iter()
            .map(|c| c.load(Ordering::Acquire))
            .sum();

        if total_probes > 0 {
            total_hits as f64 / total_probes as f64
        } else {
            0.0
        }
    }
}

/// Collision rate monitoring
#[derive(Debug)]
pub struct CollisionMonitor {
    /// Maximum number of collision entries to track
    max_entries: usize,
    /// Collision entries (hash -> collision count)
    collision_map: Arc<Mutex<HashMap<u64, u32>>>,
    /// Total collision count
    total_collisions: AtomicU64,
    /// Hash distribution histogram
    hash_histogram: Arc<Mutex<Vec<u32>>>,
    /// Table size for histogram
    table_size: usize,
}

impl CollisionMonitor {
    /// Create a new collision monitor
    pub fn new(table_size: usize, max_entries: usize) -> Self {
        Self {
            max_entries,
            collision_map: Arc::new(Mutex::new(HashMap::new())),
            total_collisions: AtomicU64::new(0),
            hash_histogram: Arc::new(Mutex::new(vec![0; table_size])),
            table_size,
        }
    }

    /// Record a collision for a hash
    pub fn record_collision(&self, hash: u64, index: usize) {
        self.total_collisions.fetch_add(1, Ordering::Relaxed);

        // Update collision map
        {
            let mut map = self.collision_map.lock().unwrap();
            if map.len() >= self.max_entries {
                // Remove oldest entry (simple FIFO)
                if let Some(oldest_key) = map.keys().next().copied() {
                    map.remove(&oldest_key);
                }
            }
            *map.entry(hash).or_insert(0) += 1;
        }

        // Update histogram
        if index < self.table_size {
            let mut histogram = self.hash_histogram.lock().unwrap();
            histogram[index] = histogram[index].saturating_add(1);
        }
    }

    /// Get collision statistics
    pub fn get_collision_stats(&self) -> CollisionStats {
        let total_collisions = self.total_collisions.load(Ordering::Acquire);
        let collision_map = self.collision_map.lock().unwrap();
        let histogram = self.hash_histogram.lock().unwrap();

        // Calculate hash distribution quality
        let total_entries: u64 = histogram.iter().map(|&count| count as u64).sum();
        let avg_entries_per_slot = if self.table_size > 0 {
            total_entries as f64 / self.table_size as f64
        } else {
            0.0
        };

        let variance = histogram
            .iter()
            .map(|&count| (count as f64 - avg_entries_per_slot).powi(2))
            .sum::<f64>()
            / self.table_size as f64;

        let distribution_quality = if avg_entries_per_slot > 0.0 {
            1.0 / (1.0 + variance.sqrt() / avg_entries_per_slot)
        } else {
            1.0
        };

        CollisionStats {
            total_collisions,
            unique_collision_hashes: collision_map.len() as u64,
            max_collisions_per_hash: collision_map.values().max().copied().unwrap_or(0) as u64,
            hash_distribution_quality: distribution_quality,
            most_collided_hashes: collision_map
                .iter()
                .map(|(hash, &count)| (*hash, count as u64))
                .collect(),
        }
    }

    /// Clear collision data
    pub fn clear(&self) {
        self.total_collisions.store(0, Ordering::Release);
        {
            let mut map = self.collision_map.lock().unwrap();
            map.clear();
        }
        {
            let mut histogram = self.hash_histogram.lock().unwrap();
            histogram.fill(0);
        }
    }
}

/// Collision statistics
#[derive(Debug, Clone)]
pub struct CollisionStats {
    pub total_collisions: u64,
    pub unique_collision_hashes: u64,
    pub max_collisions_per_hash: u64,
    pub hash_distribution_quality: f64,
    pub most_collided_hashes: Vec<(u64, u64)>,
}

/// Statistics export format
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Human-readable text
    Text,
    /// Binary format for fast loading
    Binary,
}

/// Statistics exporter
pub struct StatisticsExporter {
    /// Export format
    format: ExportFormat,
    /// Include detailed data
    include_details: bool,
}

impl StatisticsExporter {
    /// Create a new statistics exporter
    pub fn new(format: ExportFormat, include_details: bool) -> Self {
        Self {
            format,
            include_details,
        }
    }

    /// Export detailed cache statistics
    pub fn export_cache_stats(&self, stats: &DetailedCacheStats) -> String {
        match self.format {
            ExportFormat::Json => self.export_json(stats),
            ExportFormat::Csv => self.export_csv(stats),
            ExportFormat::Text => self.export_text(stats),
            ExportFormat::Binary => self.export_binary(stats),
        }
    }

    /// Export hit rate statistics
    pub fn export_hit_rates(&self, hit_rates: &HitRateByDepth) -> String {
        match self.format {
            ExportFormat::Json => self.export_hit_rates_json(hit_rates),
            ExportFormat::Csv => self.export_hit_rates_csv(hit_rates),
            ExportFormat::Text => self.export_hit_rates_text(hit_rates),
            ExportFormat::Binary => self.export_hit_rates_binary(hit_rates),
        }
    }

    /// Export collision statistics
    pub fn export_collision_stats(&self, stats: &CollisionStats) -> String {
        match self.format {
            ExportFormat::Json => self.export_collision_json(stats),
            ExportFormat::Csv => self.export_collision_csv(stats),
            ExportFormat::Text => self.export_collision_text(stats),
            ExportFormat::Binary => self.export_collision_binary(stats),
        }
    }

    fn export_json(&self, stats: &DetailedCacheStats) -> String {
        format!(
            r#"{{
    "total_probes": {},
    "total_hits": {},
    "total_stores": {},
    "total_replacements": {},
    "total_collisions": {},
    "occupancy_rate": {:.6},
    "avg_probe_time_us": {:.6},
    "avg_store_time_us": {:.6},
    "memory_usage_bytes": {},
    "hash_distribution_quality": {:.6}
}}"#,
            stats.total_probes,
            stats.total_hits,
            stats.total_stores,
            stats.total_replacements,
            stats.total_collisions,
            stats.occupancy_rate,
            stats.avg_probe_time_us,
            stats.avg_store_time_us,
            stats.memory_usage_bytes,
            stats.hash_distribution_quality
        )
    }

    fn export_csv(&self, stats: &DetailedCacheStats) -> String {
        format!(
            "total_probes,total_hits,total_stores,total_replacements,total_collisions,occupancy_rate,avg_probe_time_us,avg_store_time_us,memory_usage_bytes,hash_distribution_quality\n{},{},{},{},{},{:.6},{:.6},{:.6},{},{:.6}",
            stats.total_probes, stats.total_hits, stats.total_stores, stats.total_replacements,
            stats.total_collisions, stats.occupancy_rate, stats.avg_probe_time_us,
            stats.avg_store_time_us, stats.memory_usage_bytes, stats.hash_distribution_quality
        )
    }

    fn export_text(&self, stats: &DetailedCacheStats) -> String {
        if self.include_details {
            format!(
                "=== Detailed Cache Statistics ===\n\
                Total Probes: {}\n\
                Total Hits: {}\n\
                Total Stores: {}\n\
                Total Replacements: {}\n\
                Total Collisions: {}\n\
                Occupancy Rate: {:.2}%\n\
                Average Probe Time: {:.2} μs\n\
                Average Store Time: {:.2} μs\n\
                Memory Usage: {} bytes\n\
                Hash Distribution Quality: {:.4}",
                stats.total_probes,
                stats.total_hits,
                stats.total_stores,
                stats.total_replacements,
                stats.total_collisions,
                stats.occupancy_rate * 100.0,
                stats.avg_probe_time_us,
                stats.avg_store_time_us,
                stats.memory_usage_bytes,
                stats.hash_distribution_quality
            )
        } else {
            format!(
                "=== Cache Statistics ===\n\
                Total Probes: {}\n\
                Total Hits: {}\n\
                Hit Rate: {:.2}%\n\
                Memory Usage: {} bytes",
                stats.total_probes,
                stats.total_hits,
                if stats.total_probes > 0 {
                    stats.total_hits as f64 / stats.total_probes as f64 * 100.0
                } else {
                    0.0
                },
                stats.memory_usage_bytes
            )
        }
    }

    fn export_binary(&self, _stats: &DetailedCacheStats) -> String {
        // In a real implementation, this would serialize to binary format
        // For now, return a placeholder
        "Binary export not implemented".to_string()
    }

    fn export_hit_rates_json(&self, hit_rates: &HitRateByDepth) -> String {
        let data: Vec<(u8, f64)> = hit_rates.get_all_hit_rates();
        let json_entries: Vec<String> = data
            .iter()
            .map(|(depth, rate)| format!(r#"{{"depth": {}, "hit_rate": {:.6}}}"#, depth, rate))
            .collect();
        format!("[\n  {}\n]", json_entries.join(",\n  "))
    }

    fn export_hit_rates_csv(&self, hit_rates: &HitRateByDepth) -> String {
        let mut csv = String::from("depth,hit_rate\n");
        for (depth, rate) in hit_rates.get_all_hit_rates() {
            csv.push_str(&format!("{},{:.6}\n", depth, rate));
        }
        csv
    }

    fn export_hit_rates_text(&self, hit_rates: &HitRateByDepth) -> String {
        let mut text = String::from("=== Hit Rates by Depth ===\n");
        text.push_str(&format!(
            "Total Hit Rate: {:.2}%\n\n",
            hit_rates.get_total_hit_rate() * 100.0
        ));

        if self.include_details {
            for (depth, rate) in hit_rates.get_all_hit_rates() {
                text.push_str(&format!("Depth {}: {:.2}%\n", depth, rate * 100.0));
            }
        } else {
            // Show only summary for non-detailed export
            let rates = hit_rates.get_all_hit_rates();
            if !rates.is_empty() {
                let avg_rate =
                    rates.iter().map(|(_, rate)| *rate).sum::<f64>() / rates.len() as f64;
                let min_rate = rates
                    .iter()
                    .map(|(_, rate)| *rate)
                    .fold(f64::INFINITY, |a, b| a.min(b));
                let max_rate = rates
                    .iter()
                    .map(|(_, rate)| *rate)
                    .fold(0.0f64, |a, b| a.max(b));
                text.push_str(&format!(
                    "Average: {:.2}%, Min: {:.2}%, Max: {:.2}%\n",
                    avg_rate * 100.0,
                    min_rate * 100.0,
                    max_rate * 100.0
                ));
            }
        }
        text
    }

    fn export_hit_rates_binary(&self, _hit_rates: &HitRateByDepth) -> String {
        "Binary export not implemented".to_string()
    }

    fn export_collision_json(&self, stats: &CollisionStats) -> String {
        format!(
            r#"{{
    "total_collisions": {},
    "unique_collision_hashes": {},
    "max_collisions_per_hash": {},
    "hash_distribution_quality": {:.6},
    "most_collided_hashes": [{}]
}}"#,
            stats.total_collisions,
            stats.unique_collision_hashes,
            stats.max_collisions_per_hash,
            stats.hash_distribution_quality,
            stats
                .most_collided_hashes
                .iter()
                .map(|(hash, count)| format!(r#"{{"hash": {}, "count": {}}}"#, hash, count))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn export_collision_csv(&self, stats: &CollisionStats) -> String {
        format!(
            "total_collisions,unique_collision_hashes,max_collisions_per_hash,hash_distribution_quality\n{},{},{},{:.6}",
            stats.total_collisions, stats.unique_collision_hashes, stats.max_collisions_per_hash, stats.hash_distribution_quality
        )
    }

    fn export_collision_text(&self, stats: &CollisionStats) -> String {
        if self.include_details {
            format!(
                "=== Collision Statistics ===\n\
                Total Collisions: {}\n\
                Unique Collision Hashes: {}\n\
                Max Collisions per Hash: {}\n\
                Hash Distribution Quality: {:.4}\n\n\
                Most Collided Hashes:\n{}",
                stats.total_collisions,
                stats.unique_collision_hashes,
                stats.max_collisions_per_hash,
                stats.hash_distribution_quality,
                stats
                    .most_collided_hashes
                    .iter()
                    .take(10)
                    .map(|(hash, count)| format!("  0x{:016x}: {} collisions", hash, count))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            format!(
                "=== Collision Statistics ===\n\
                Total Collisions: {}\n\
                Unique Collision Hashes: {}\n\
                Hash Distribution Quality: {:.4}",
                stats.total_collisions,
                stats.unique_collision_hashes,
                stats.hash_distribution_quality
            )
        }
    }

    fn export_collision_binary(&self, _stats: &CollisionStats) -> String {
        "Binary export not implemented".to_string()
    }
}

/// Performance trend analysis
pub struct PerformanceTrendAnalyzer {
    /// Historical performance data points
    data_points: Arc<Mutex<VecDeque<PerformanceDataPoint>>>,
    /// Maximum number of data points to keep
    max_data_points: usize,
    /// Time window for trend analysis (seconds)
    time_window_seconds: u64,
}

/// Performance data point
#[derive(Debug, Clone)]
pub struct PerformanceDataPoint {
    /// Timestamp
    pub timestamp: u64,
    /// Hit rate
    pub hit_rate: f64,
    /// Average probe time (microseconds)
    pub avg_probe_time_us: f64,
    /// Average store time (microseconds)
    pub avg_store_time_us: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
    /// Occupancy rate
    pub occupancy_rate: f64,
}

impl PerformanceTrendAnalyzer {
    /// Create a new performance trend analyzer
    pub fn new(max_data_points: usize, time_window_seconds: u64) -> Self {
        Self {
            data_points: Arc::new(Mutex::new(VecDeque::new())),
            max_data_points,
            time_window_seconds,
        }
    }

    /// Add a new performance data point
    pub fn add_data_point(&self, data_point: PerformanceDataPoint) {
        let mut points = self.data_points.lock().unwrap();

        // Remove old data points outside the time window
        let current_time = data_point.timestamp;

        while let Some(front) = points.front() {
            if current_time - front.timestamp > self.time_window_seconds {
                points.pop_front();
            } else {
                break;
            }
        }

        // Add new data point
        points.push_back(data_point);

        // Maintain maximum data points
        while points.len() > self.max_data_points {
            points.pop_front();
        }
    }

    /// Analyze performance trends
    pub fn analyze_trends(&self) -> PerformanceTrends {
        let points = self.data_points.lock().unwrap();

        if points.len() < 2 {
            return PerformanceTrends::default();
        }

        let recent_points: Vec<&PerformanceDataPoint> = points.iter().collect();
        let n = recent_points.len();

        // Calculate linear regression for hit rate trend
        let hit_rate_trend = self.calculate_trend(&recent_points, |p| p.hit_rate);
        let probe_time_trend = self.calculate_trend(&recent_points, |p| p.avg_probe_time_us);
        let store_time_trend = self.calculate_trend(&recent_points, |p| p.avg_store_time_us);
        let occupancy_trend = self.calculate_trend(&recent_points, |p| p.occupancy_rate);

        // Calculate volatility (standard deviation)
        let hit_rate_volatility = self.calculate_volatility(&recent_points, |p| p.hit_rate);
        let probe_time_volatility =
            self.calculate_volatility(&recent_points, |p| p.avg_probe_time_us);

        // Calculate performance score (higher is better)
        let recent_avg_hit_rate = recent_points.iter().map(|p| p.hit_rate).sum::<f64>() / n as f64;
        let recent_avg_probe_time = recent_points
            .iter()
            .map(|p| p.avg_probe_time_us)
            .sum::<f64>()
            / n as f64;
        let performance_score = (recent_avg_hit_rate * 100.0) - (recent_avg_probe_time / 10.0);

        PerformanceTrends {
            hit_rate_trend,
            probe_time_trend,
            store_time_trend,
            occupancy_trend,
            hit_rate_volatility,
            probe_time_volatility,
            performance_score,
            data_points_count: n,
            time_span_seconds: if n > 1 {
                recent_points[n - 1].timestamp - recent_points[0].timestamp
            } else {
                0
            },
        }
    }

    /// Calculate trend using linear regression
    fn calculate_trend<F>(&self, points: &[&PerformanceDataPoint], get_value: F) -> f64
    where
        F: Fn(&PerformanceDataPoint) -> f64,
    {
        let n = points.len() as f64;
        let sum_x: f64 = (0..points.len()).map(|i| i as f64).sum();
        let sum_y: f64 = points.iter().map(|p| get_value(p)).sum();
        let sum_xy: f64 = points
            .iter()
            .enumerate()
            .map(|(i, p)| i as f64 * get_value(p))
            .sum();
        let sum_x2: f64 = (0..points.len()).map(|i| (i as f64).powi(2)).sum();

        // Linear regression slope
        if n * sum_x2 - sum_x * sum_x != 0.0 {
            (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        } else {
            0.0
        }
    }

    /// Calculate volatility (standard deviation)
    fn calculate_volatility<F>(&self, points: &[&PerformanceDataPoint], get_value: F) -> f64
    where
        F: Fn(&PerformanceDataPoint) -> f64,
    {
        let n = points.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }

        let mean = points.iter().map(|p| get_value(p)).sum::<f64>() / n;
        let variance = points
            .iter()
            .map(|p| (get_value(p) - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0);

        variance.sqrt()
    }

    /// Get all data points
    pub fn get_data_points(&self) -> Vec<PerformanceDataPoint> {
        self.data_points.lock().unwrap().iter().cloned().collect()
    }

    /// Clear all data points
    pub fn clear(&self) {
        self.data_points.lock().unwrap().clear();
    }
}

/// Performance trends analysis result
#[derive(Debug, Clone, Default)]
pub struct PerformanceTrends {
    /// Hit rate trend (positive = improving, negative = declining)
    pub hit_rate_trend: f64,
    /// Probe time trend (positive = getting slower, negative = getting faster)
    pub probe_time_trend: f64,
    /// Store time trend
    pub store_time_trend: f64,
    /// Occupancy trend
    pub occupancy_trend: f64,
    /// Hit rate volatility (lower is better)
    pub hit_rate_volatility: f64,
    /// Probe time volatility (lower is better)
    pub probe_time_volatility: f64,
    /// Overall performance score
    pub performance_score: f64,
    /// Number of data points analyzed
    pub data_points_count: usize,
    /// Time span of analysis (seconds)
    pub time_span_seconds: u64,
}

/// Comprehensive statistics manager
pub struct AdvancedStatisticsManager {
    /// Detailed cache statistics
    cache_stats: Arc<Mutex<DetailedCacheStats>>,
    /// Hit rate by depth tracker
    hit_rate_tracker: Arc<Mutex<HitRateByDepth>>,
    /// Collision monitor
    collision_monitor: Arc<CollisionMonitor>,
    /// Performance trend analyzer
    trend_analyzer: Arc<PerformanceTrendAnalyzer>,
    /// Statistics exporter
    exporter: StatisticsExporter,
}

impl AdvancedStatisticsManager {
    /// Create a new advanced statistics manager
    pub fn new(table_size: usize, max_depth: u8) -> Self {
        Self {
            cache_stats: Arc::new(Mutex::new(DetailedCacheStats::default())),
            hit_rate_tracker: Arc::new(Mutex::new(HitRateByDepth::new(max_depth))),
            collision_monitor: Arc::new(CollisionMonitor::new(table_size, 1000)),
            trend_analyzer: Arc::new(PerformanceTrendAnalyzer::new(1000, 3600)), // 1 hour window
            exporter: StatisticsExporter::new(ExportFormat::Text, true),
        }
    }

    /// Record a probe operation
    pub fn record_probe(&self, depth: u8, hit: bool, probe_time_us: f64) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.total_probes += 1;
        if hit {
            stats.total_hits += 1;
        }

        // Update average probe time
        let total_probes = stats.total_probes as f64;
        stats.avg_probe_time_us =
            (stats.avg_probe_time_us * (total_probes - 1.0) + probe_time_us) / total_probes;

        // Record in hit rate tracker
        self.hit_rate_tracker
            .lock()
            .unwrap()
            .record_probe(depth, hit);
    }

    /// Record a store operation
    pub fn record_store(&self, store_time_us: f64, replacement: bool) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.total_stores += 1;
        if replacement {
            stats.total_replacements += 1;
        }

        // Update average store time
        let total_stores = stats.total_stores as f64;
        stats.avg_store_time_us =
            (stats.avg_store_time_us * (total_stores - 1.0) + store_time_us) / total_stores;
    }

    /// Record a collision
    pub fn record_collision(&self, hash: u64, index: usize) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.total_collisions += 1;

        self.collision_monitor.record_collision(hash, index);
    }

    /// Update occupancy rate
    pub fn update_occupancy(&self, occupied_entries: usize, total_entries: usize) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.occupancy_rate = if total_entries > 0 {
            occupied_entries as f64 / total_entries as f64
        } else {
            0.0
        };
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, memory_usage_bytes: usize) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.memory_usage_bytes = memory_usage_bytes;
    }

    /// Update hash distribution quality
    pub fn update_hash_distribution_quality(&self, quality: f64) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.hash_distribution_quality = quality;
    }

    /// Add performance data point for trend analysis
    pub fn add_performance_data_point(&self) {
        let cache_stats = self.cache_stats.lock().unwrap();
        let hit_rate_tracker = self.hit_rate_tracker.lock().unwrap();

        let data_point = PerformanceDataPoint {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            hit_rate: hit_rate_tracker.get_total_hit_rate(),
            avg_probe_time_us: cache_stats.avg_probe_time_us,
            avg_store_time_us: cache_stats.avg_store_time_us,
            memory_usage_bytes: cache_stats.memory_usage_bytes,
            occupancy_rate: cache_stats.occupancy_rate,
        };

        self.trend_analyzer.add_data_point(data_point);
    }

    /// Get comprehensive statistics report
    pub fn get_comprehensive_report(&self) -> ComprehensiveStatisticsReport {
        let cache_stats = self.cache_stats.lock().unwrap().clone();
        let mut hit_rate_tracker = self.hit_rate_tracker.lock().unwrap();
        hit_rate_tracker.update_hit_rates();
        let hit_rates = hit_rate_tracker.get_all_hit_rates();
        let collision_stats = self.collision_monitor.get_collision_stats();
        let trends = self.trend_analyzer.analyze_trends();

        ComprehensiveStatisticsReport {
            cache_stats,
            hit_rates,
            collision_stats,
            trends,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Export statistics in the configured format
    pub fn export_statistics(&self) -> ExportedStatistics {
        let report = self.get_comprehensive_report();

        ExportedStatistics {
            cache_stats: self.exporter.export_cache_stats(&report.cache_stats),
            hit_rates: self.exporter.export_hit_rates(&HitRateByDepth::new(20)), // Dummy for export
            collision_stats: self
                .exporter
                .export_collision_stats(&report.collision_stats),
            trends: format!(
                r#"{{
    "hit_rate_trend": {:.6},
    "probe_time_trend": {:.6},
    "store_time_trend": {:.6},
    "occupancy_trend": {:.6},
    "hit_rate_volatility": {:.6},
    "probe_time_volatility": {:.6},
    "performance_score": {:.6},
    "data_points_count": {},
    "time_span_seconds": {}
}}"#,
                report.trends.hit_rate_trend,
                report.trends.probe_time_trend,
                report.trends.store_time_trend,
                report.trends.occupancy_trend,
                report.trends.hit_rate_volatility,
                report.trends.probe_time_volatility,
                report.trends.performance_score,
                report.trends.data_points_count,
                report.trends.time_span_seconds
            ),
            format: self.exporter.format.clone(),
        }
    }

    /// Clear all statistics
    pub fn clear_all(&self) {
        {
            let mut stats = self.cache_stats.lock().unwrap();
            *stats = DetailedCacheStats::default();
        }
        {
            let mut hit_rate_tracker = self.hit_rate_tracker.lock().unwrap();
            *hit_rate_tracker = HitRateByDepth::new(20);
        }
        self.collision_monitor.clear();
        self.trend_analyzer.clear();
    }
}

/// Comprehensive statistics report
#[derive(Debug, Clone)]
pub struct ComprehensiveStatisticsReport {
    pub cache_stats: DetailedCacheStats,
    pub hit_rates: Vec<(u8, f64)>,
    pub collision_stats: CollisionStats,
    pub trends: PerformanceTrends,
    pub timestamp: u64,
}

/// Exported statistics
#[derive(Debug, Clone)]
pub struct ExportedStatistics {
    pub cache_stats: String,
    pub hit_rates: String,
    pub collision_stats: String,
    pub trends: String,
    pub format: ExportFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_rate_by_depth() {
        let mut tracker = HitRateByDepth::new(10);

        // Record some probes
        tracker.record_probe(0, true);
        tracker.record_probe(0, false);
        tracker.record_probe(1, true);
        tracker.record_probe(1, true);

        tracker.update_hit_rates();

        assert_eq!(tracker.get_hit_rate(0), 0.5); // 1 hit out of 2 probes
        assert_eq!(tracker.get_hit_rate(1), 1.0); // 2 hits out of 2 probes
        assert_eq!(tracker.get_total_hit_rate(), 0.75); // 3 hits out of 4 total probes
    }

    #[test]
    fn test_collision_monitor() {
        let monitor = CollisionMonitor::new(1000, 100);

        // Record some collisions
        monitor.record_collision(0x123, 100);
        monitor.record_collision(0x123, 100);
        monitor.record_collision(0x456, 200);

        let stats = monitor.get_collision_stats();
        assert_eq!(stats.total_collisions, 3);
        assert_eq!(stats.unique_collision_hashes, 2);
        assert_eq!(stats.max_collisions_per_hash, 2);
    }

    #[test]
    fn test_statistics_exporter() {
        let exporter = StatisticsExporter::new(ExportFormat::Text, true);
        let stats = DetailedCacheStats::default();

        let export = exporter.export_cache_stats(&stats);
        assert!(export.contains("Detailed Cache Statistics"));
        assert!(export.contains("Total Probes: 0"));
    }

    #[test]
    fn test_performance_trend_analyzer() {
        let analyzer = PerformanceTrendAnalyzer::new(100, 3600);

        // Add some data points
        for i in 0..5 {
            let data_point = PerformanceDataPoint {
                timestamp: i,
                hit_rate: i as f64 * 0.1,
                avg_probe_time_us: 10.0 - i as f64,
                avg_store_time_us: 5.0,
                memory_usage_bytes: 1000,
                occupancy_rate: 0.5,
            };
            analyzer.add_data_point(data_point);
        }

        let trends = analyzer.analyze_trends();
        assert_eq!(trends.data_points_count, 5);
        assert!(trends.hit_rate_trend > 0.0); // Should be positive trend
        assert!(trends.probe_time_trend < 0.0); // Should be negative trend (getting faster)
    }

    #[test]
    fn test_advanced_statistics_manager() {
        let manager = AdvancedStatisticsManager::new(1000, 20);

        // Record some operations
        manager.record_probe(0, true, 10.0);
        manager.record_probe(0, false, 15.0);
        manager.record_store(5.0, false);
        manager.record_collision(0x123, 100);

        let report = manager.get_comprehensive_report();
        assert_eq!(report.cache_stats.total_probes, 2);
        assert_eq!(report.cache_stats.total_hits, 1);
        assert_eq!(report.cache_stats.total_stores, 1);
        assert_eq!(report.cache_stats.total_collisions, 1);
    }
}
