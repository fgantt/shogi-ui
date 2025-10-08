pub mod zobrist;
pub mod transposition_table;
pub mod search_engine;
pub mod board_trait;
pub mod shogi_hash;
pub mod shogi_position_tests;
pub mod transposition_config;
pub mod replacement_policies;
pub mod cache_management;
pub mod thread_safe_table;
pub mod performance_optimization;
pub mod performance_benchmarks;
pub mod error_handling;
pub mod advanced_statistics;
pub mod search_integration;
pub mod move_ordering;
pub mod move_ordering_integration;
pub mod move_ordering_tests;
pub mod comprehensive_tests;
pub mod test_runner;
pub mod search_integration_tests;

// Configuration and tuning modules
pub mod runtime_configuration;
pub mod adaptive_configuration;
pub mod performance_tuning;
pub mod configuration_templates;

// WASM compatibility modules
pub mod wasm_compatibility;
pub mod wasm_transposition_table;
pub mod wasm_benchmarks;

// Advanced features modules
pub mod multi_level_transposition_table;
pub mod compressed_entry_storage;
pub mod predictive_prefetching;
pub mod ml_replacement_policies;
pub mod dynamic_table_sizing;
pub mod advanced_cache_warming;

// Re-export commonly used types and functions
pub use zobrist::*;
pub use transposition_table::TranspositionTable;
pub use search_engine::*;
pub use board_trait::*;
pub use shogi_hash::*;
pub use shogi_position_tests::*;
pub use transposition_config::*;
pub use replacement_policies::*;
pub use cache_management::*;
pub use thread_safe_table::{ThreadSafeTranspositionTable, ThreadSafetyMode, ThreadSafeStatsSnapshot};
pub use performance_optimization::{OptimizedHashMapper, CacheAlignedAllocator, PrefetchManager, OptimizedEntryPacker, HotPathOptimizer};
pub use performance_benchmarks::{PerformanceBenchmarks, BenchmarkResults as PerformanceBenchmarkResults, BenchmarkComparison};
pub use error_handling::{TranspositionError, TranspositionResult, ComprehensiveErrorHandler, ErrorLogger, GracefulDegradationHandler, ErrorRecoveryManager};
pub use advanced_statistics::{AdvancedStatisticsManager, DetailedCacheStats, HitRateByDepth, CollisionMonitor, StatisticsExporter, PerformanceTrendAnalyzer};
pub use search_integration::{EnhancedSearchEngine, SearchStats};
pub use move_ordering::{
    MoveOrdering, OrderingStats, OrderingWeights, MemoryUsage, MoveOrderingConfig,
    CacheConfig, KillerConfig, HistoryConfig, PerformanceConfig, DebugConfig,
    HotPathStats, PerformanceStats, CacheSizes, BottleneckAnalysis, Bottleneck,
    BottleneckCategory, BottleneckSeverity, HeuristicStats, HeuristicPerformance,
    TimingStats, OperationTiming, MemoryStats, MemoryBreakdown, AllocationStats,
    FragmentationStats, CacheStats, CachePerformance, StatisticsExport,
    PerformanceSummary, PerformanceChartData, CacheHitRates, HeuristicEffectiveness,
    MemoryUsageTrend, TimingBreakdown, PerformanceTrendAnalysis, TrendAnalysis,
    TrendDirection, MoveOrderingError, MoveOrderingResult, ErrorSeverity,
    ErrorLogEntry, ErrorHandler, MemoryPool, MemoryPoolSizes, MemoryTracker,
    MemoryUsageBreakdown, AllocationEvent, AllocationType, MemoryThresholds,
    MemoryThresholdStatus, MemoryLeakWarning, MemoryLeakReport, MemoryCleanupReport,
    MemoryPressureLevel, AdvancedFeatures, PositionSpecificStrategies, OrderingStrategy,
    PriorityAdjustments, HeuristicPreferences, GamePhase, MachineLearningModel,
    MLModelType, MLParameters, TrainingExample, PositionContext, DynamicWeightAdjuster,
    WeightAdjustment, PerformanceTracker, PerformanceTrend, ThreadingSupport,
    PredictiveOrdering, PredictionModel, PredictionModelType, PredictionParameters,
    PredictionExample, MovePattern, AdvancedCacheWarming, CacheWarmingStrategy,
    CacheWarmingType, CacheWarmingParameters, CacheWarmingPerformance,
    AdvancedFeatureFlags, AdvancedFeatureStatus
};
pub use move_ordering_integration::{TranspositionMoveOrderer, MoveOrderingStats, MoveOrderingHints};
pub use move_ordering_tests::{MoveOrderingTestSuite, MoveOrderingBenchmarks, TestResults};
pub use comprehensive_tests::{ComprehensiveTestSuite, ComprehensiveTestResults, TestConfig, PerformanceTargets, KnownPosition};
pub use test_runner::{TestRunner, TestRunnerConfig, TestCategory, OutputFormat, TestExecutionResult, run_all_tests, run_test_categories};

// Configuration and tuning re-exports
pub use runtime_configuration::{
    RuntimeConfigurationManager, PerformanceMetrics, ConfigurationUpdateStrategy,
    ConfigurationValidationResult, PerformanceImpact, ConfigurationBuilder
};

pub use adaptive_configuration::{
    AdaptiveConfigurationManager, AdaptationRule, AdaptationCondition, AdaptationAction,
    AdaptationState, AdaptationMode
};

pub use performance_tuning::{
    PerformanceTuningManager, PerformanceProfiler, PerformanceCounters, MemorySnapshot,
    PerformanceTargets as TuningPerformanceTargets, TuningRecommendation, TuningAction, TuningSession, TuningReport
};

pub use configuration_templates::{
    ConfigurationTemplateManager, ConfigurationTemplate, TemplateMetadata, TemplateCategory,
    PerformanceProfile, MemoryRequirements, ConfigurationValidator, ValidationRule,
    ValidationResult, ValidationSeverity, PerformanceBenchmark, BenchmarkResults
};

// WASM compatibility re-exports
pub use wasm_compatibility::{
    WasmTime, WasmDuration, WasmMemoryManager, WasmPerformanceCounter, 
    WasmTranspositionConfig, wasm_utils
};

pub use wasm_transposition_table::{
    WasmTranspositionTable, WasmTranspositionEntry, WasmTranspositionStats
};

pub use wasm_benchmarks::{
    WasmBenchmarkSuite, WasmBenchmarkResults, WasmBenchmarkSummary,
    WasmPerformanceProfiler, WasmOperationStats, WasmSpecificMetrics
};

// Advanced features re-exports
pub use multi_level_transposition_table::{
    MultiLevelTranspositionTable, MultiLevelConfig, MultiLevelStats, LevelStats,
    LevelConfig, MemoryAllocationStrategy
};

pub use compressed_entry_storage::{
    CompressedEntryStorage, CompressionConfig, CompressionStats, CompressedEntry,
    CompressionMetadata, CompressionAlgorithm
};

pub use predictive_prefetching::{
    PredictivePrefetcher, PrefetchConfig, PrefetchStats, PrefetchPrediction,
    PredictionMetadata, PrefetchStrategy
};

pub use ml_replacement_policies::{
    MLReplacementPolicy, MLReplacementConfig, MLReplacementContext, MLReplacementDecision,
    MLAlgorithm, ReplacementAction, PositionFeatures, AccessPatternInfo, TemporalInfo
};

pub use dynamic_table_sizing::{
    DynamicTableSizer, DynamicSizingConfig, ResizeDecision, ResizeReason,
    AccessPatternAnalysis, DynamicSizingStats
};

pub use advanced_cache_warming::{
    AdvancedCacheWarmer, CacheWarmingConfig, WarmingSession, WarmingResults,
    WarmingStrategy, WarmingEntry, WarmingEntryType, PositionAnalysis
};
