pub mod board_trait;
pub mod iterative_deepening;
pub mod null_move;
pub mod parallel_search;
pub mod pvs;
pub mod quiescence;
pub mod reductions;
pub mod search_engine;
pub mod shogi_hash;
pub mod shogi_position_tests;
pub mod statistics;
pub mod time_management;
pub mod transposition_table;
pub mod zobrist;
pub use parallel_search::{
    ParallelSearchConfig, ParallelSearchEngine, ThreadLocalSearchContext, WorkDistributionStats,
    WorkStealingQueue, WorkUnit,
};
pub mod advanced_statistics;
pub mod cache_management;
pub mod comprehensive_tests;
pub mod error_handling;
pub mod move_ordering;
pub mod move_ordering_integration;
pub mod move_ordering_tests;
pub mod performance_benchmarks;
pub mod performance_optimization;
pub mod replacement_policies;
pub mod search_integration;
#[cfg(all(test, feature = "legacy-tests"))]
pub mod search_integration_tests;
pub mod tapered_search_integration;
pub mod test_runner;
pub mod thread_safe_table;
pub mod transposition_config;
pub mod transposition_table_trait;
pub mod transposition_table_config;

// Configuration and tuning modules
pub mod adaptive_configuration;
pub mod configuration_templates;
pub mod performance_tuning;
pub mod runtime_configuration;
pub mod memory_tracking;

// Web compatibility modules removed - no longer needed

// Advanced features modules
pub mod advanced_cache_warming;
pub mod compressed_entry_storage;
#[cfg(feature = "hierarchical-tt")]
pub mod compressed_transposition_table;
pub mod dynamic_table_sizing;
#[cfg(feature = "hierarchical-tt")]
pub mod hierarchical_transposition_table;
pub mod ml_replacement_policies;
pub mod multi_level_transposition_table;
pub mod predictive_prefetching;

// Re-export commonly used types and functions
pub use advanced_statistics::{
    AdvancedStatisticsManager, CollisionMonitor, DetailedCacheStats, HitRateByDepth,
    PerformanceTrendAnalyzer, StatisticsExporter,
};
pub use board_trait::*;
pub use cache_management::*;
pub use comprehensive_tests::{
    ComprehensiveTestResults, ComprehensiveTestSuite, KnownPosition, PerformanceTargets, TestConfig,
};
pub use error_handling::{
    ComprehensiveErrorHandler, ErrorLogger, ErrorRecoveryManager, GracefulDegradationHandler,
    TranspositionError, TranspositionResult,
};
pub use move_ordering::{
    AdvancedCacheWarming, AdvancedFeatureFlags, AdvancedFeatureStatus, AdvancedFeatures,
    AllocationEvent, AllocationStats, AllocationType, Bottleneck, BottleneckAnalysis,
    BottleneckCategory, BottleneckSeverity, CacheConfig, CacheHitRates, CachePerformance,
    CacheSizes, CacheStats, CacheWarmingParameters, CacheWarmingPerformance, CacheWarmingStrategy,
    CacheWarmingType, DebugConfig, DynamicWeightAdjuster, ErrorHandler, ErrorLogEntry,
    ErrorSeverity, FragmentationStats, GamePhase, HeuristicEffectiveness, HeuristicPerformance,
    HeuristicPreferences, HeuristicStats, HistoryConfig, HotPathStats, KillerConfig, MLModelType,
    MLParameters, MachineLearningModel, MemoryBreakdown, MemoryCleanupReport, MemoryLeakReport,
    MemoryLeakWarning, MemoryPool, MemoryPoolSizes, MemoryPressureLevel, MemoryStats,
    MemoryThresholdStatus, MemoryThresholds, MemoryTracker, MemoryUsage, MemoryUsageBreakdown,
    MemoryUsageTrend, MoveOrdering, MoveOrderingConfig, MoveOrderingError, MoveOrderingResult,
    MovePattern, OperationTiming, OrderingStats, OrderingStrategy, OrderingWeights,
    PerformanceChartData, PerformanceConfig, PerformanceStats, PerformanceSummary,
    PerformanceTracker, PerformanceTrend, PerformanceTrendAnalysis, PositionContext,
    PositionSpecificStrategies, PredictionExample, PredictionModel, PredictionModelType,
    PredictionParameters, PredictiveOrdering, PriorityAdjustments, StatisticsExport,
    ThreadingSupport, TimingBreakdown, TimingStats, TrainingExample, TrendAnalysis, TrendDirection,
    WeightAdjustment,
};
pub use move_ordering_integration::{
    MoveOrderingHints, MoveOrderingStats, TranspositionMoveOrderer,
};
pub use move_ordering_tests::{MoveOrderingBenchmarks, MoveOrderingTestSuite, TestResults};
pub use performance_benchmarks::{
    BenchmarkComparison, BenchmarkResults as PerformanceBenchmarkResults, PerformanceBenchmarks,
};
pub use performance_optimization::{
    CacheAlignedAllocator, HotPathOptimizer, OptimizedEntryPacker, OptimizedHashMapper,
    PrefetchManager,
};
pub use replacement_policies::*;
pub use search_engine::*;
pub use search_integration::{EnhancedSearchEngine, SearchStats};
pub use shogi_hash::*;
pub use shogi_position_tests::*;
pub use test_runner::{
    run_all_tests, run_test_categories, OutputFormat, TestCategory, TestExecutionResult,
    TestRunner, TestRunnerConfig,
};
pub use thread_safe_table::{
    ThreadSafeStatsSnapshot, ThreadSafeTranspositionTable, ThreadSafetyMode,
};
pub use transposition_config::*;
pub use transposition_table::TranspositionTable;
pub use zobrist::*;

// Configuration and tuning re-exports
pub use runtime_configuration::{
    ConfigurationBuilder, ConfigurationUpdateStrategy, ConfigurationValidationResult,
    PerformanceImpact, PerformanceMetrics, RuntimeConfigurationManager,
};

pub use adaptive_configuration::{
    AdaptationAction, AdaptationCondition, AdaptationMode, AdaptationRule, AdaptationState,
    AdaptiveConfigurationManager,
};

pub use performance_tuning::{
    MemorySnapshot, PerformanceCounters, PerformanceProfiler,
    PerformanceTargets as TuningPerformanceTargets, PerformanceTuningManager, TuningAction,
    TuningRecommendation, TuningReport, TuningSession,
};

pub use configuration_templates::{
    BenchmarkResults, ConfigurationTemplate, ConfigurationTemplateManager, ConfigurationValidator,
    MemoryRequirements, PerformanceBenchmark, PerformanceProfile, TemplateCategory,
    TemplateMetadata, ValidationResult, ValidationRule, ValidationSeverity,
};

// Web compatibility re-exports removed - no longer needed

// Advanced features re-exports
pub use multi_level_transposition_table::{
    LevelConfig, LevelStats, MemoryAllocationStrategy, MultiLevelConfig, MultiLevelStats,
    MultiLevelTranspositionTable,
};

pub use compressed_entry_storage::{
    CompressedEntry, CompressedEntryStorage, CompressionAlgorithm, CompressionConfig,
    CompressionMetadata, CompressionStats,
};
#[cfg(feature = "hierarchical-tt")]
pub use compressed_transposition_table::{
    CompressedTranspositionStats, CompressedTranspositionTable, CompressedTranspositionTableConfig,
};
#[cfg(feature = "hierarchical-tt")]
pub use hierarchical_transposition_table::{
    HierarchicalSnapshot, HierarchicalStats, HierarchicalTranspositionConfig,
    HierarchicalTranspositionTable, HitLevel,
};

pub use predictive_prefetching::{
    PredictionMetadata, PredictivePrefetcher, PrefetchConfig, PrefetchPrediction, PrefetchStats,
    PrefetchStrategy,
};

pub use ml_replacement_policies::{
    AccessPatternInfo, MLAlgorithm, MLReplacementConfig, MLReplacementContext,
    MLReplacementDecision, MLReplacementPolicy, PositionFeatures, ReplacementAction, TemporalInfo,
};

pub use dynamic_table_sizing::{
    AccessPatternAnalysis, DynamicSizingConfig, DynamicSizingStats, DynamicTableSizer,
    ResizeDecision, ResizeReason,
};

pub use advanced_cache_warming::{
    AdvancedCacheWarmer, CacheWarmingConfig, PositionAnalysis, WarmingEntry, WarmingEntryType,
    WarmingResults, WarmingSession, WarmingStrategy,
};
