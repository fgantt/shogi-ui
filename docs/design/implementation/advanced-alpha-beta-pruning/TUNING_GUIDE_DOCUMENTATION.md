# Pruning Parameters Tuning Guide

## Overview

This document provides a comprehensive guide for tuning the advanced alpha-beta pruning parameters in the Shogi engine. Proper parameter tuning is crucial for achieving optimal performance while maintaining search correctness and tactical accuracy.

## Table of Contents

1. [Parameter Overview](#parameter-overview)
2. [Tuning Methodology](#tuning-methodology)
3. [Individual Parameter Tuning](#individual-parameter-tuning)
4. [Parameter Interactions](#parameter-interactions)
5. [Position-Specific Tuning](#position-specific-tuning)
6. [Automated Tuning](#automated-tuning)
7. [Validation and Testing](#validation-and-testing)
8. [Performance Monitoring](#performance-monitoring)
9. [Troubleshooting Common Issues](#troubleshooting-common-issues)

## Parameter Overview

### Core Parameters Structure

```rust
pub struct PruningParameters {
    // Futility pruning parameters
    pub futility_margin: [i32; 8],        // Margins by depth
    pub futility_depth_limit: u8,         // Maximum depth for futility
    pub extended_futility_depth: u8,      // Depth for extended futility
    
    // Late move reduction parameters
    pub lmr_base_reduction: u8,           // Base reduction amount
    pub lmr_move_threshold: u8,           // Move index threshold
    pub lmr_depth_threshold: u8,          // Minimum depth for LMR
    pub lmr_max_reduction: u8,            // Maximum reduction allowed
    
    // Delta pruning parameters
    pub delta_margin: i32,                // Safety margin for captures
    pub delta_depth_limit: u8,            // Maximum depth for delta
    
    // Razoring parameters
    pub razoring_depth_limit: u8,         // Maximum depth for razoring
    pub razoring_margin: i32,             // Margin for middlegame
    pub razoring_margin_endgame: i32,     // Margin for endgame
    
    // Multi-cut pruning parameters
    pub multi_cut_threshold: u8,          // Fail-high moves needed
    pub multi_cut_depth_limit: u8,        // Minimum depth for multi-cut
    
    // Adaptive parameters
    pub adaptive_enabled: bool,           // Enable adaptive tuning
    pub position_dependent_margins: bool, // Position-dependent margins
}
```

### Default Parameters

```rust
impl Default for PruningParameters {
    fn default() -> Self {
        Self {
            futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
            futility_depth_limit: 3,
            extended_futility_depth: 5,
            lmr_base_reduction: 1,
            lmr_move_threshold: 3,
            lmr_depth_threshold: 2,
            lmr_max_reduction: 3,
            delta_margin: 200,
            delta_depth_limit: 4,
            razoring_depth_limit: 3,
            razoring_margin: 300,
            razoring_margin_endgame: 200,
            multi_cut_threshold: 3,
            multi_cut_depth_limit: 4,
            adaptive_enabled: false,
            position_dependent_margins: false,
        }
    }
}
```

## Tuning Methodology

### 1. Systematic Approach

The tuning process follows a systematic approach:

1. **Baseline Establishment**: Establish baseline performance with default parameters
2. **Individual Parameter Tuning**: Tune each parameter individually
3. **Parameter Interaction Analysis**: Analyze interactions between parameters
4. **Position-Specific Tuning**: Tune for different position types
5. **Validation and Testing**: Validate tuned parameters
6. **Performance Monitoring**: Monitor performance in production

### 2. Tuning Tools

```rust
pub struct TuningTools {
    pub parameter_validator: ParameterValidator,
    pub performance_analyzer: PerformanceAnalyzer,
    pub position_tester: PositionTester,
    pub automated_tuner: AutomatedTuner,
}
```

### 3. Tuning Process

```rust
pub struct TuningProcess {
    pub step_1_baseline: BaselineMeasurement,
    pub step_2_individual: IndividualParameterTuning,
    pub step_3_interactions: ParameterInteractionAnalysis,
    pub step_4_position_specific: PositionSpecificTuning,
    pub step_5_validation: ParameterValidation,
    pub step_6_monitoring: PerformanceMonitoring,
}
```

## Individual Parameter Tuning

### Futility Pruning Parameters

#### futility_margin

**Purpose**: Controls how aggressive futility pruning is at each depth.

**Tuning Guidelines**:
- **Conservative**: [0, 80, 160, 240, 320, 400, 480, 560]
- **Default**: [0, 100, 200, 300, 400, 500, 600, 700]
- **Aggressive**: [0, 120, 240, 360, 480, 600, 720, 840]

**Tuning Process**:
```rust
// Test different margin values
let test_margins = vec![
    [0, 80, 160, 240, 320, 400, 480, 560],   // Conservative
    [0, 100, 200, 300, 400, 500, 600, 700],  // Default
    [0, 120, 240, 360, 480, 600, 720, 840],  // Aggressive
];

for margins in test_margins {
    let params = PruningParameters {
        futility_margin: margins,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Pruning rate: 8-15%
- Tactical accuracy: >99%
- Performance improvement: 10-20%

#### futility_depth_limit

**Purpose**: Maximum depth at which futility pruning is applied.

**Tuning Guidelines**:
- **Conservative**: 2
- **Default**: 3
- **Aggressive**: 4

**Tuning Process**:
```rust
// Test different depth limits
let test_limits = vec![2, 3, 4];

for limit in test_limits {
    let params = PruningParameters {
        futility_depth_limit: limit,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- No tactical losses at test positions
- Pruning rate appropriate for depth
- Performance improvement maintained

### Late Move Reduction Parameters

#### lmr_base_reduction

**Purpose**: Base reduction amount for LMR.

**Tuning Guidelines**:
- **Conservative**: 1
- **Default**: 1
- **Aggressive**: 2

**Tuning Process**:
```rust
// Test different base reductions
let test_reductions = vec![1, 2];

for reduction in test_reductions {
    let params = PruningParameters {
        lmr_base_reduction: reduction,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Re-search rate: <5%
- Tactical accuracy: >99.5%
- Performance improvement: 12-18%

#### lmr_move_threshold

**Purpose**: Move index threshold for LMR application.

**Tuning Guidelines**:
- **Conservative**: 2
- **Default**: 3
- **Aggressive**: 4

**Tuning Process**:
```rust
// Test different move thresholds
let test_thresholds = vec![2, 3, 4, 5];

for threshold in test_thresholds {
    let params = PruningParameters {
        lmr_move_threshold: threshold,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Best move preservation: >99%
- Performance improvement: 10-20%
- Re-search rate: <10%

#### lmr_max_reduction

**Purpose**: Maximum reduction allowed for LMR.

**Tuning Guidelines**:
- **Conservative**: 2
- **Default**: 3
- **Aggressive**: 4

**Tuning Process**:
```rust
// Test different max reductions
let test_max_reductions = vec![2, 3, 4, 5];

for max_reduction in test_max_reductions {
    let params = PruningParameters {
        lmr_max_reduction: max_reduction,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Re-search rate: <15%
- Tactical accuracy: >99%
- Performance improvement: 15-25%

### Delta Pruning Parameters

#### delta_margin

**Purpose**: Safety margin for delta pruning of captures.

**Tuning Guidelines**:
- **Conservative**: 150
- **Default**: 200
- **Aggressive**: 250

**Tuning Process**:
```rust
// Test different delta margins
let test_margins = vec![150, 200, 250, 300];

for margin in test_margins {
    let params = PruningParameters {
        delta_margin: margin,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Capture accuracy: >99.5%
- Pruning rate: 5-10%
- Performance improvement: 5-15%

#### delta_depth_limit

**Purpose**: Maximum depth for delta pruning.

**Tuning Guidelines**:
- **Conservative**: 3
- **Default**: 4
- **Aggressive**: 5

**Tuning Process**:
```rust
// Test different depth limits
let test_limits = vec![3, 4, 5];

for limit in test_limits {
    let params = PruningParameters {
        delta_depth_limit: limit,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- No tactical losses in captures
- Pruning rate appropriate for depth
- Performance improvement maintained

### Razoring Parameters

#### razoring_margin

**Purpose**: Margin for razoring in middlegame positions.

**Tuning Guidelines**:
- **Conservative**: 250
- **Default**: 300
- **Aggressive**: 350

**Tuning Process**:
```rust
// Test different razoring margins
let test_margins = vec![250, 300, 350, 400];

for margin in test_margins {
    let params = PruningParameters {
        razoring_margin: margin,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Quiet position accuracy: >99%
- Pruning rate: 6-12%
- Performance improvement: 8-18%

#### razoring_margin_endgame

**Purpose**: Margin for razoring in endgame positions.

**Tuning Guidelines**:
- **Conservative**: 150
- **Default**: 200
- **Aggressive**: 250

**Tuning Process**:
```rust
// Test different endgame margins
let test_margins = vec![150, 200, 250, 300];

for margin in test_margins {
    let params = PruningParameters {
        razoring_margin_endgame: margin,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Endgame accuracy: >99.5%
- Pruning rate: 8-15%
- Performance improvement: 10-20%

### Multi-cut Pruning Parameters

#### multi_cut_threshold

**Purpose**: Number of fail-high moves needed for multi-cut pruning.

**Tuning Guidelines**:
- **Conservative**: 2
- **Default**: 3
- **Aggressive**: 4

**Tuning Process**:
```rust
// Test different thresholds
let test_thresholds = vec![2, 3, 4, 5];

for threshold in test_thresholds {
    let params = PruningParameters {
        multi_cut_threshold: threshold,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- Multi-cut accuracy: >99%
- Pruning rate: 3-8%
- Performance improvement: 5-12%

#### multi_cut_depth_limit

**Purpose**: Minimum depth for multi-cut pruning.

**Tuning Guidelines**:
- **Conservative**: 3
- **Default**: 4
- **Aggressive**: 5

**Tuning Process**:
```rust
// Test different depth limits
let test_limits = vec![3, 4, 5];

for limit in test_limits {
    let params = PruningParameters {
        multi_cut_depth_limit: limit,
        ..Default::default()
    };
    
    let performance = test_parameters(params);
    analyze_performance(performance);
}
```

**Validation Criteria**:
- No tactical losses in multi-cut positions
- Pruning rate appropriate for depth
- Performance improvement maintained

## Parameter Interactions

### Interaction Analysis

Parameters interact with each other in complex ways:

```rust
pub struct ParameterInteractionAnalysis {
    pub futility_lmr_interaction: f64,      // How futility and LMR interact
    pub delta_razoring_interaction: f64,    // How delta and razoring interact
    pub multi_cut_effectiveness: f64,       // How multi-cut affects other techniques
    pub overall_synergy: f64,               // Overall parameter synergy
}
```

### Common Interactions

#### 1. Futility + LMR Interaction
- **Synergy**: High - both target quiet moves
- **Optimal Combination**: Conservative futility + aggressive LMR
- **Tuning Strategy**: Tune futility first, then LMR

#### 2. Delta + Razoring Interaction
- **Synergy**: Medium - both target different move types
- **Optimal Combination**: Balanced parameters
- **Tuning Strategy**: Tune independently, then optimize together

#### 3. Multi-cut + All Others
- **Synergy**: Low - multi-cut is independent
- **Optimal Combination**: Multi-cut as final optimization
- **Tuning Strategy**: Tune last, after other parameters

### Interaction Tuning Process

```rust
// Tune parameters in order of interaction
pub struct InteractionTuningProcess {
    pub step_1_futility: FutilityTuning,
    pub step_2_lmr: LMRTuning,
    pub step_3_delta: DeltaTuning,
    pub step_4_razoring: RazoringTuning,
    pub step_5_multi_cut: MultiCutTuning,
    pub step_6_optimization: OverallOptimization,
}
```

## Position-Specific Tuning

### Tuning by Game Phase

#### Opening Tuning
```rust
pub struct OpeningParameters {
    pub futility_margin: [0, 80, 160, 240, 320, 400, 480, 560],
    pub lmr_move_threshold: 2,
    pub delta_margin: 150,
    pub razoring_margin: 250,
}
```

#### Middlegame Tuning
```rust
pub struct MiddlegameParameters {
    pub futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
    pub lmr_move_threshold: 3,
    pub delta_margin: 200,
    pub razoring_margin: 300,
}
```

#### Endgame Tuning
```rust
pub struct EndgameParameters {
    pub futility_margin: [0, 120, 240, 360, 480, 600, 720, 840],
    pub lmr_move_threshold: 4,
    pub delta_margin: 250,
    pub razoring_margin: 200,
}
```

### Tuning by Position Type

#### Tactical Positions
```rust
pub struct TacticalParameters {
    pub futility_depth_limit: 2,        // More conservative
    pub lmr_max_reduction: 2,           // Less aggressive
    pub delta_margin: 150,              // More conservative
    pub razoring_margin: 250,           // More conservative
}
```

#### Quiet Positions
```rust
pub struct QuietParameters {
    pub futility_depth_limit: 4,        // More aggressive
    pub lmr_max_reduction: 4,           // More aggressive
    pub delta_margin: 250,              // More aggressive
    pub razoring_margin: 350,           // More aggressive
}
```

#### Complex Positions
```rust
pub struct ComplexParameters {
    pub futility_depth_limit: 3,        // Balanced
    pub lmr_max_reduction: 3,           // Balanced
    pub delta_margin: 200,              // Balanced
    pub razoring_margin: 300,           // Balanced
}
```

## Automated Tuning

### Automated Tuning System

```rust
pub struct AutomatedTuner {
    pub parameter_space: ParameterSpace,
    pub optimization_algorithm: OptimizationAlgorithm,
    pub fitness_function: FitnessFunction,
    pub validation_suite: ValidationSuite,
}
```

### Optimization Algorithms

#### 1. Genetic Algorithm
```rust
pub struct GeneticAlgorithm {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub selection_pressure: f64,
}
```

#### 2. Simulated Annealing
```rust
pub struct SimulatedAnnealing {
    pub initial_temperature: f64,
    pub cooling_rate: f64,
    pub final_temperature: f64,
    pub max_iterations: usize,
}
```

#### 3. Particle Swarm Optimization
```rust
pub struct ParticleSwarmOptimization {
    pub swarm_size: usize,
    pub max_iterations: usize,
    pub inertia_weight: f64,
    pub cognitive_weight: f64,
    pub social_weight: f64,
}
```

### Fitness Function

```rust
pub struct FitnessFunction {
    pub performance_weight: f64,        // 0.4
    pub accuracy_weight: f64,           // 0.4
    pub safety_weight: f64,             // 0.2
}
```

### Automated Tuning Process

```rust
// Automated tuning workflow
pub struct AutomatedTuningProcess {
    pub step_1_parameter_space: ParameterSpaceDefinition,
    pub step_2_optimization: ParameterOptimization,
    pub step_3_validation: ParameterValidation,
    pub step_4_analysis: PerformanceAnalysis,
    pub step_5_refinement: ParameterRefinement,
}
```

## Validation and Testing

### Validation Suite

```rust
pub struct ValidationSuite {
    pub tactical_tests: TacticalTestSuite,
    pub positional_tests: PositionalTestSuite,
    pub endgame_tests: EndgameTestSuite,
    pub performance_tests: PerformanceTestSuite,
}
```

### Test Categories

#### 1. Tactical Tests
- **Mate in N**: Forced mate sequences
- **Tactical Combinations**: Complex tactical sequences
- **Sacrifice Positions**: Sacrificial combinations
- **Defensive Tactics**: Defensive tactical sequences

#### 2. Positional Tests
- **Strategic Plans**: Long-term strategic plans
- **Positional Sacrifices**: Positional sacrifices
- **Endgame Technique**: Endgame technique tests
- **Opening Theory**: Opening theory positions

#### 3. Performance Tests
- **Speed Tests**: Search speed benchmarks
- **Memory Tests**: Memory usage benchmarks
- **Scalability Tests**: Performance at different depths
- **Stress Tests**: Performance under load

### Validation Criteria

```rust
pub struct ValidationCriteria {
    pub tactical_accuracy: f64,         // >99%
    pub positional_accuracy: f64,       // >98%
    pub endgame_accuracy: f64,          // >99.5%
    pub performance_improvement: f64,   // >30%
    pub memory_overhead: f64,           // <15%
}
```

## Performance Monitoring

### Real-time Monitoring

```rust
pub struct RealTimeMonitoring {
    pub performance_metrics: PerformanceMetrics,
    pub pruning_statistics: PruningStatistics,
    pub error_detection: ErrorDetection,
    pub alert_system: AlertSystem,
}
```

### Performance Alerts

```rust
pub enum PerformanceAlert {
    LowPruningRate(f64),               // Pruning rate < 20%
    HighMemoryUsage(u64),              // Memory usage > 100 MB
    LowCacheHitRate(f64),              // Cache hit rate < 70%
    SlowSearch(f64),                   // Nodes per second < 200,000
    PruningIneffectiveness(f64),       // Pruning effectiveness < 80%
    TacticalAccuracyDrop(f64),         // Tactical accuracy < 99%
}
```

### Performance Reports

```rust
pub struct PerformanceReport {
    pub timestamp: String,
    pub search_duration: u64,
    pub total_nodes: u64,
    pub pruning_statistics: PruningStatistics,
    pub performance_metrics: PerformanceMetrics,
    pub recommendations: Vec<String>,
}
```

## Troubleshooting Common Issues

### Common Tuning Problems

#### 1. Over-Pruning
**Symptoms**: Tactical losses, missed best moves
**Causes**: Too aggressive parameters
**Solutions**:
- Reduce futility margins
- Increase LMR thresholds
- Reduce delta margins
- Increase razoring margins

#### 2. Under-Pruning
**Symptoms**: Low pruning rate, slow search
**Causes**: Too conservative parameters
**Solutions**:
- Increase futility margins
- Decrease LMR thresholds
- Increase delta margins
- Decrease razoring margins

#### 3. Parameter Conflicts
**Symptoms**: Inconsistent performance, high re-search rate
**Causes**: Conflicting parameter values
**Solutions**:
- Analyze parameter interactions
- Tune parameters in correct order
- Use automated tuning
- Validate parameter combinations

#### 4. Memory Issues
**Symptoms**: High memory usage, cache inefficiency
**Causes**: Large cache sizes, memory leaks
**Solutions**:
- Reduce cache sizes
- Implement cache cleanup
- Monitor memory usage
- Optimize cache strategies

### Debugging Tools

```rust
pub struct DebuggingTools {
    pub parameter_analyzer: ParameterAnalyzer,
    pub performance_profiler: PerformanceProfiler,
    pub error_detector: ErrorDetector,
    pub optimization_suggestions: OptimizationSuggestions,
}
```

### Debugging Process

```rust
// Systematic debugging approach
pub struct DebuggingProcess {
    pub step_1_identify_issue: IssueIdentification,
    pub step_2_analyze_cause: CauseAnalysis,
    pub step_3_test_solution: SolutionTesting,
    pub step_4_validate_fix: FixValidation,
    pub step_5_monitor_performance: PerformanceMonitoring,
}
```

## Conclusion

Proper parameter tuning is essential for achieving optimal performance with the advanced alpha-beta pruning system. The systematic approach outlined in this guide ensures:

1. **Optimal Performance**: Maximum tree reduction and time improvement
2. **Search Correctness**: No tactical losses or missed best moves
3. **Stability**: Consistent performance across different positions
4. **Maintainability**: Easy to understand and modify parameters

The automated tuning tools and validation suite provide a robust framework for parameter optimization. Regular monitoring and performance analysis ensure that the tuned parameters continue to perform optimally over time.

Key recommendations:
- Start with default parameters as baseline
- Tune parameters systematically, one at a time
- Validate all parameter changes thoroughly
- Monitor performance continuously
- Use automated tools for complex optimizations
- Document all parameter changes and their rationale
