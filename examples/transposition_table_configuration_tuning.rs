#[cfg(feature = "tt-config-tuning")]
use shogi_engine::bitboards::*;
#[cfg(feature = "tt-config-tuning")]
use shogi_engine::search::*;
#[cfg(feature = "tt-config-tuning")]
use shogi_engine::types::*;
#[cfg(feature = "tt-config-tuning")]
use std::time::Instant;

#[cfg(not(feature = "tt-config-tuning"))]
fn main() {
    eprintln!(
        "Enable the `tt-config-tuning` feature to build the transposition_table_configuration_tuning example."
    );
}

#[cfg(feature = "tt-config-tuning")]
fn basic_configuration_management() {
    println!("Creating runtime configuration manager...");

    // Create initial configuration
    let initial_config = TranspositionConfig::default();
    let mut runtime_manager = RuntimeConfigurationManager::new(initial_config);

    println!("  ✅ Runtime configuration manager created");
    println!("  Initial configuration:");
    let config = runtime_manager.get_active_config();
    println!("    Table size: {}", config.table_size);
    println!("    Replacement policy: {:?}", config.replacement_policy);
    println!("    Enable statistics: {}", config.enable_statistics);

    // List available templates
    println!("  Available templates:");
    for template_name in runtime_manager.list_templates() {
        println!("    • {}", template_name);
    }

    // Get a template
    if let Some(template) = runtime_manager.get_template("performance") {
        println!("  Performance template:");
        println!("    Table size: {}", template.table_size);
        println!("    Replacement policy: {:?}", template.replacement_policy);
    }
}

#[cfg(feature = "tt-config-tuning")]
fn runtime_configuration_updates() {
    println!("Demonstrating runtime configuration updates...");

    let initial_config = TranspositionConfig::default();
    let mut runtime_manager = RuntimeConfigurationManager::new(initial_config);

    // Update configuration using builder
    println!("  Creating configuration with builder...");
    let new_config = ConfigurationBuilder::new()
        .table_size(65536)
        .replacement_policy(ReplacementPolicy::DepthPreferred)
        .enable_statistics(true)
        .enable_cache_line_alignment(true)
        .build();

    println!("  New configuration:");
    println!("    Table size: {}", new_config.table_size);
    println!(
        "    Replacement policy: {:?}",
        new_config.replacement_policy
    );
    println!("    Enable statistics: {}", new_config.enable_statistics);

    // Apply immediate update
    println!("  Applying immediate configuration update...");
    match runtime_manager.update_config(new_config.clone(), ConfigurationUpdateStrategy::Immediate)
    {
        Ok(_) => println!("    ✅ Configuration updated successfully"),
        Err(e) => println!("    ❌ Configuration update failed: {}", e),
    }

    // Validate configuration
    println!("  Validating configuration...");
    let validation = runtime_manager.validate_configuration(&new_config);
    if validation.is_valid {
        println!("    ✅ Configuration is valid");
    } else {
        println!(
            "    ❌ Configuration validation failed: {:?}",
            validation.errors
        );
    }

    if !validation.warnings.is_empty() {
        println!("    ⚠️  Warnings: {:?}", validation.warnings);
    }

    // Demonstrate gradual update
    println!("  Applying gradual configuration update...");
    let target_config = TranspositionConfig::performance_optimized();
    match runtime_manager.update_config(
        target_config,
        ConfigurationUpdateStrategy::Gradual {
            steps: 3,
            step_duration_ms: 100,
        },
    ) {
        Ok(_) => println!("    ✅ Gradual update completed"),
        Err(e) => println!("    ❌ Gradual update failed: {}", e),
    }

    // Show configuration history
    println!("  Configuration history:");
    for (i, config) in runtime_manager.get_config_history().iter().enumerate() {
        println!("    {}: Table size = {}", i, config.table_size);
    }
}

#[cfg(feature = "tt-config-tuning")]
fn adaptive_configuration_demo() {
    println!("Demonstrating adaptive configuration...");

    let initial_config = TranspositionConfig::default();
    let mut adaptive_manager = AdaptiveConfigurationManager::new(initial_config);

    // Set adaptation mode
    println!("  Setting adaptation mode to Balanced...");
    adaptive_manager.set_adaptation_mode(AdaptationMode::Balanced);

    // Get current adaptation state
    let state = adaptive_manager.get_adaptation_state();
    println!("  Adaptation state:");
    println!("    Enabled: {}", state.enabled);
    println!("    Mode: {:?}", state.mode);
    println!("    Adaptation count: {}", state.adaptation_count);

    // List adaptation rules
    println!("  Adaptation rules:");
    for rule in adaptive_manager.get_adaptation_rules() {
        println!(
            "    • {} (priority: {}, enabled: {})",
            rule.name, rule.priority, rule.enabled
        );
    }

    // Simulate performance metrics updates
    println!("  Simulating performance metrics updates...");
    let mut metrics = PerformanceMetrics::default();

    // Simulate low hit rate scenario
    metrics.hit_rate = 0.15; // Below threshold
    metrics.avg_operation_time_us = 80.0;
    metrics.memory_usage_bytes = 1000000;
    metrics.collision_rate = 0.12;
    metrics.system_load = 0.6;
    metrics.available_memory_bytes = 1000000000;

    println!("    Updating metrics with low hit rate (15%)...");
    match adaptive_manager.update_performance_metrics(metrics.clone()) {
        Ok(_) => println!("      ✅ Metrics updated"),
        Err(e) => println!("      ❌ Metrics update failed: {}", e),
    }

    // Simulate high memory usage scenario
    metrics.memory_usage_bytes = 150000000; // Above threshold
    metrics.hit_rate = 0.35; // Good hit rate

    println!("    Updating metrics with high memory usage...");
    match adaptive_manager.update_performance_metrics(metrics) {
        Ok(_) => println!("      ✅ Metrics updated"),
        Err(e) => println!("      ❌ Metrics update failed: {}", e),
    }

    // Check if adaptations were triggered
    let final_state = adaptive_manager.get_adaptation_state();
    println!("  Final adaptation state:");
    println!("    Adaptation count: {}", final_state.adaptation_count);
    println!("    Performance trend: {:?}", final_state.performance_trend);

    // Export adaptation state
    match adaptive_manager.export_adaptation_state() {
        Ok(json) => println!("  Adaptation state exported ({} characters)", json.len()),
        Err(e) => println!("  Export failed: {}", e),
    }
}

#[cfg(feature = "tt-config-tuning")]
fn performance_tuning_demo() {
    println!("Demonstrating performance tuning...");

    let initial_config = TranspositionConfig::default();
    let mut tuning_manager = PerformanceTuningManager::new(initial_config);

    // Start tuning session
    println!("  Starting tuning session...");
    let session_id = tuning_manager.start_tuning_session();
    println!("    Session ID: {}", session_id);

    // Get initial recommendations
    println!("  Initial recommendations:");
    for recommendation in tuning_manager.get_recommendations() {
        println!(
            "    • {} (priority: {}, confidence: {:.1})",
            recommendation.title, recommendation.priority, recommendation.confidence
        );
    }

    // Apply a recommendation
    if let Some(first_recommendation) = tuning_manager.get_recommendations().first() {
        println!("  Applying recommendation: {}", first_recommendation.title);
        match tuning_manager.apply_recommendation(&first_recommendation.id) {
            Ok(_) => println!("    ✅ Recommendation applied successfully"),
            Err(e) => println!("    ❌ Recommendation application failed: {}", e),
        }
    }

    // Generate performance-based recommendations
    println!("  Generating performance-based recommendations...");
    let perf_recommendations = tuning_manager.generate_performance_recommendations();
    println!(
        "    Generated {} performance-based recommendations",
        perf_recommendations.len()
    );

    for recommendation in perf_recommendations.iter().take(3) {
        println!(
            "      • {} (expected improvement: {:.1}%)",
            recommendation.title, recommendation.expected_improvement
        );
    }

    // Get performance profiler
    let profiler = tuning_manager.get_profiler();
    let mut profiler = profiler.lock().unwrap();
    profiler.set_enabled(true);

    // Simulate some operations
    println!("  Profiling operations...");
    profiler.record_operation("store", 45);
    profiler.record_operation("probe", 25);
    profiler.record_operation("store", 50);
    profiler.record_operation("probe", 30);

    profiler.increment_counter("cache_hits");
    profiler.increment_counter("cache_hits");
    profiler.increment_counter("cache_misses");

    // Get profiling results
    let counters = profiler.get_performance_counters();
    println!("  Performance counters:");
    println!("    Total operations: {}", counters.total_operations);
    println!("    Cache hits: {}", counters.cache_hits);
    println!("    Cache misses: {}", counters.cache_misses);

    if let Some(avg_time) = profiler.get_average_operation_time("store") {
        println!("    Average store time: {:.1}μs", avg_time);
    }

    // End tuning session
    println!("  Ending tuning session...");
    match tuning_manager.end_tuning_session(&session_id) {
        Ok(improvement) => println!("    ✅ Session ended with {:.1}% improvement", improvement),
        Err(e) => println!("    ❌ Session end failed: {}", e),
    }

    // Export tuning report
    match tuning_manager.export_tuning_report() {
        Ok(report) => println!("  Tuning report exported ({} characters)", report.len()),
        Err(e) => println!("  Report export failed: {}", e),
    }
}

#[cfg(feature = "tt-config-tuning")]
fn configuration_templates_demo() {
    println!("Demonstrating configuration templates...");

    let mut template_manager = ConfigurationTemplateManager::new();

    // List all templates
    println!("  Available templates:");
    for template_name in template_manager.list_templates() {
        if let Some(template) = template_manager.get_template(&template_name) {
            println!(
                "    • {} ({:?}) - {}",
                template.name, template.category, template.description
            );
        }
    }

    // Get templates by category
    println!("  Performance templates:");
    for template in template_manager.get_templates_by_category(&TemplateCategory::Performance) {
        println!(
            "    • {} - Performance rating: {}",
            template.name, template.performance_profile.performance_rating
        );
    }

    // Get templates by tags
    println!("  Templates tagged with 'memory':");
    for template in template_manager.get_templates_by_tags(&["memory".to_string()]) {
        println!(
            "    • {} - Memory efficiency rating: {}",
            template.name, template.performance_profile.memory_efficiency_rating
        );
    }

    // Create custom template
    println!("  Creating custom template...");
    let custom_template = ConfigurationTemplate {
        name: "custom_balanced".to_string(),
        description: "Custom balanced configuration for specific use case".to_string(),
        config: TranspositionConfig {
            table_size: 32768,
            replacement_policy: ReplacementPolicy::DepthPreferred,
            enable_statistics: true,
            enable_cache_line_alignment: true,
            enable_prefetching: false,
        },
        category: TemplateCategory::Custom,
        tags: vec!["custom".to_string(), "balanced".to_string()],
        performance_profile: PerformanceProfile {
            hit_rate_range: (0.25, 0.40),
            operation_time_range: (35.0, 65.0),
            memory_usage_range: (524288, 2097152),
            collision_rate_range: (0.05, 0.15),
            performance_rating: 7,
            memory_efficiency_rating: 8,
        },
        memory_requirements: MemoryRequirements {
            minimum_memory_bytes: 524288,
            recommended_memory_bytes: 2097152,
            maximum_memory_bytes: 4194304,
            memory_growth_rate_bytes_per_op: 0.1,
        },
    };

    match template_manager.add_custom_template(custom_template.clone()) {
        Ok(_) => println!("    ✅ Custom template added successfully"),
        Err(e) => println!("    ❌ Custom template addition failed: {}", e),
    }

    // Update template usage
    template_manager.update_template_usage("custom_balanced");

    // Rate template
    template_manager
        .rate_template("custom_balanced", 4.5)
        .unwrap();

    // Get template metadata
    if let Some(metadata) = template_manager.get_template_metadata("custom_balanced") {
        println!("  Custom template metadata:");
        println!("    Usage count: {}", metadata.usage_count);
        println!("    User rating: {:?}", metadata.user_rating);
        println!("    Author: {}", metadata.author);
    }

    // Export templates
    match template_manager.export_templates(Some(TemplateCategory::Custom)) {
        Ok(json) => println!("  Custom templates exported ({} characters)", json.len()),
        Err(e) => println!("  Template export failed: {}", e),
    }

    // Remove custom template
    if template_manager.remove_custom_template("custom_balanced") {
        println!("    ✅ Custom template removed successfully");
    }
}

#[cfg(feature = "tt-config-tuning")]
fn configuration_validation_demo() {
    println!("Demonstrating configuration validation...");

    let validator = ConfigurationValidator::new();

    // Validate valid configuration
    println!("  Validating default configuration...");
    let default_config = TranspositionConfig::default();
    let results = validator.validate_configuration(&default_config);

    for result in results {
        if result.is_valid {
            println!("    ✅ Validation passed");
        } else {
            println!("    ❌ Validation failed: {:?}", result.error_message);
        }

        if let Some(warning) = result.warning_message {
            println!("    ⚠️  Warning: {}", warning);
        }

        if let Some(suggestion) = result.suggestion {
            println!("    💡 Suggestion: {}", suggestion);
        }
    }

    // Validate invalid configuration
    println!("  Validating invalid configuration (zero table size)...");
    let invalid_config = TranspositionConfig {
        table_size: 0,
        ..TranspositionConfig::default()
    };
    let results = validator.validate_configuration(&invalid_config);

    for result in results {
        if !result.is_valid {
            println!("    ❌ Validation failed: {:?}", result.error_message);
        }
    }

    // Validate configuration with warnings
    println!("  Validating configuration with warnings (non-power-of-two size)...");
    let warning_config = TranspositionConfig {
        table_size: 50000, // Not a power of two
        ..TranspositionConfig::default()
    };
    let results = validator.validate_configuration(&warning_config);

    for result in results {
        if let Some(warning) = result.warning_message {
            println!("    ⚠️  Warning: {}", warning);
        }
    }

    // Benchmark configuration
    println!("  Benchmarking configuration...");
    let benchmark_results = validator.benchmark_configuration(&default_config);
    println!(
        "    Average operation time: {:.1}μs",
        benchmark_results.avg_operation_time_us
    );
    println!(
        "    Expected hit rate: {:.1}%",
        benchmark_results.hit_rate_percentage
    );
    println!(
        "    Expected memory usage: {:.1} KB",
        benchmark_results.memory_usage_bytes as f64 / 1024.0
    );
    println!(
        "    Expected collision rate: {:.1}%",
        benchmark_results.collision_rate_percentage
    );
    println!(
        "    Expected throughput: {:.0} ops/sec",
        benchmark_results.throughput_ops_per_sec
    );
}

#[cfg(feature = "tt-config-tuning")]
fn advanced_tuning_scenarios() {
    println!("Demonstrating advanced tuning scenarios...");

    // Scenario 1: Low memory environment
    println!("  Scenario 1: Low memory environment");
    let low_memory_config = TranspositionConfig::memory_optimized();
    let mut runtime_manager = RuntimeConfigurationManager::new(low_memory_config);

    let mut metrics = PerformanceMetrics::default();
    metrics.available_memory_bytes = 50000000; // 50MB
    metrics.memory_usage_bytes = 10000000; // 10MB
    metrics.system_load = 0.8;

    runtime_manager.update_performance_metrics(metrics);

    // Generate recommendations for low memory
    let mut tuning_manager = PerformanceTuningManager::new(runtime_manager.get_active_config());
    let recommendations = tuning_manager.generate_performance_recommendations();

    println!(
        "    Generated {} recommendations for low memory scenario",
        recommendations.len()
    );
    for recommendation in recommendations.iter().take(2) {
        println!("      • {}", recommendation.title);
    }

    // Scenario 2: High performance requirements
    println!("  Scenario 2: High performance requirements");
    let high_perf_config = TranspositionConfig::performance_optimized();
    let mut runtime_manager = RuntimeConfigurationManager::new(high_perf_config);

    let mut metrics = PerformanceMetrics::default();
    metrics.hit_rate = 0.25; // Below target
    metrics.avg_operation_time_us = 80.0; // Above target
    metrics.available_memory_bytes = 500000000; // 500MB
    metrics.system_load = 0.3;

    runtime_manager.update_performance_metrics(metrics);

    let mut tuning_manager = PerformanceTuningManager::new(runtime_manager.get_active_config());
    let recommendations = tuning_manager.generate_performance_recommendations();

    println!(
        "    Generated {} recommendations for high performance scenario",
        recommendations.len()
    );
    for recommendation in recommendations.iter().take(2) {
        println!(
            "      • {} (expected improvement: {:.1}%)",
            recommendation.title, recommendation.expected_improvement
        );
    }

    // Scenario 3: Adaptive configuration with changing conditions
    println!("  Scenario 3: Adaptive configuration with changing conditions");
    let initial_config = TranspositionConfig::default();
    let mut adaptive_manager = AdaptiveConfigurationManager::new(initial_config);

    // Start with low load
    let mut metrics = PerformanceMetrics::default();
    metrics.system_load = 0.2;
    metrics.available_memory_bytes = 1000000000;

    adaptive_manager
        .update_performance_metrics(metrics)
        .unwrap();

    // Simulate load increase
    let mut metrics = PerformanceMetrics::default();
    metrics.system_load = 0.9; // High load
    metrics.available_memory_bytes = 50000000; // Reduced memory

    adaptive_manager
        .update_performance_metrics(metrics)
        .unwrap();

    let state = adaptive_manager.get_adaptation_state();
    println!("    Adaptation count: {}", state.adaptation_count);
    println!("    Performance trend: {:?}", state.performance_trend);

    // Scenario 4: Configuration rollback
    println!("  Scenario 4: Configuration rollback");
    let mut runtime_manager = RuntimeConfigurationManager::new(TranspositionConfig::default());

    // Apply a configuration
    let new_config = TranspositionConfig::performance_optimized();
    runtime_manager
        .update_config(new_config, ConfigurationUpdateStrategy::Immediate)
        .unwrap();

    // Rollback
    match runtime_manager.rollback_config() {
        Ok(_) => println!("    ✅ Configuration rolled back successfully"),
        Err(e) => println!("    ❌ Rollback failed: {}", e),
    }

    let current_config = runtime_manager.get_active_config();
    println!("    Current table size: {}", current_config.table_size);

    println!("  All advanced tuning scenarios completed successfully!");
}

#[cfg(feature = "tt-config-tuning")]
fn main() {
    println!("⚙️ Transposition Table Configuration and Tuning Example");
    println!("=====================================================");

    // 1. Basic configuration management
    println!("\n📋 Basic Configuration Management");
    println!("----------------------------------");
    basic_configuration_management();

    // 2. Runtime configuration updates
    println!("\n🔄 Runtime Configuration Updates");
    println!("---------------------------------");
    runtime_configuration_updates();

    // 3. Adaptive configuration
    println!("\n🤖 Adaptive Configuration");
    println!("-------------------------");
    adaptive_configuration_demo();

    // 4. Performance tuning
    println!("\n⚡ Performance Tuning");
    println!("---------------------");
    performance_tuning_demo();

    // 5. Configuration templates
    println!("\n📝 Configuration Templates");
    println!("---------------------------");
    configuration_templates_demo();

    // 6. Configuration validation
    println!("\n✅ Configuration Validation");
    println!("---------------------------");
    configuration_validation_demo();

    // 7. Advanced tuning scenarios
    println!("\n🎯 Advanced Tuning Scenarios");
    println!("-----------------------------");
    advanced_tuning_scenarios();

    println!("\n🎉 Configuration and tuning example completed!");
    println!("\n📚 Key Features Demonstrated:");
    println!("   • Runtime configuration management and updates");
    println!("   • Adaptive configuration based on performance metrics");
    println!("   • Performance tuning with recommendations");
    println!("   • Configuration templates for common use cases");
    println!("   • Comprehensive configuration validation");
    println!("   • Advanced tuning scenarios and optimization");
}
