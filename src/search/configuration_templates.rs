//! Configuration templates and validation system
//!
//! This module provides predefined configuration templates for common use cases
//! and comprehensive validation to ensure configurations are valid and optimal.

use crate::search::transposition_config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration template manager
pub struct ConfigurationTemplateManager {
    /// Built-in templates
    builtin_templates: HashMap<String, ConfigurationTemplate>,
    /// Custom templates
    custom_templates: HashMap<String, ConfigurationTemplate>,
    /// Template metadata
    template_metadata: HashMap<String, TemplateMetadata>,
}

/// Configuration template with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template configuration
    pub config: TranspositionConfig,
    /// Template category
    pub category: TemplateCategory,
    /// Use case tags
    pub tags: Vec<String>,
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
    /// Memory requirements
    pub memory_requirements: MemoryRequirements,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Creation date
    pub created_at: std::time::SystemTime,
    /// Last modified date
    pub last_modified: std::time::SystemTime,
    /// Usage count
    pub usage_count: u64,
    /// User rating (1-5)
    pub user_rating: Option<f64>,
    /// Template author
    pub author: String,
}

/// Template categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// General purpose templates
    General,
    /// Performance-optimized templates
    Performance,
    /// Memory-optimized templates
    Memory,
    /// Development/testing templates
    Development,
    /// Production templates
    Production,
    /// Custom templates
    Custom,
}

/// Performance profile characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Expected hit rate range
    pub hit_rate_range: (f64, f64),
    /// Expected operation time range (microseconds)
    pub operation_time_range: (f64, f64),
    /// Expected memory usage range (bytes)
    pub memory_usage_range: (u64, u64),
    /// Expected collision rate range
    pub collision_rate_range: (f64, f64),
    /// Performance rating (1-10)
    pub performance_rating: u8,
    /// Memory efficiency rating (1-10)
    pub memory_efficiency_rating: u8,
}

/// Memory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRequirements {
    /// Minimum memory required (bytes)
    pub minimum_memory_bytes: u64,
    /// Recommended memory (bytes)
    pub recommended_memory_bytes: u64,
    /// Maximum memory usage (bytes)
    pub maximum_memory_bytes: u64,
    /// Memory growth rate (bytes per operation)
    pub memory_growth_rate_bytes_per_op: f64,
}

/// Configuration validator
pub struct ConfigurationValidator {
    /// Validation rules
    validation_rules: Vec<ValidationRule>,
    /// Performance benchmarks (for future use)
    #[allow(dead_code)]
    performance_benchmarks: HashMap<String, PerformanceBenchmark>,
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Validation function
    pub validator: fn(&TranspositionConfig) -> ValidationResult,
    /// Rule severity
    pub severity: ValidationSeverity,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// Error message if validation failed
    pub error_message: Option<String>,
    /// Warning message if applicable
    pub warning_message: Option<String>,
    /// Suggestion for improvement
    pub suggestion: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    /// Error - configuration is invalid
    Error,
    /// Warning - configuration may not be optimal
    Warning,
    /// Info - informational message
    Info,
}

/// Performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    /// Benchmark name
    pub name: String,
    /// Benchmark configuration
    pub config: TranspositionConfig,
    /// Benchmark results
    pub results: BenchmarkResults,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Average operation time (microseconds)
    pub avg_operation_time_us: f64,
    /// Hit rate percentage
    pub hit_rate_percentage: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    /// Collision rate percentage
    pub collision_rate_percentage: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
}

impl ConfigurationTemplateManager {
    /// Create a new configuration template manager
    pub fn new() -> Self {
        let mut manager = Self {
            builtin_templates: HashMap::new(),
            custom_templates: HashMap::new(),
            template_metadata: HashMap::new(),
        };

        // Initialize built-in templates
        manager.initialize_builtin_templates();

        manager
    }

    /// Initialize built-in templates
    fn initialize_builtin_templates(&mut self) {
        // Default template
        let default_template = ConfigurationTemplate {
            name: "default".to_string(),
            description: "Balanced configuration suitable for general use".to_string(),
            config: TranspositionConfig::default(),
            category: TemplateCategory::General,
            tags: vec!["balanced".to_string(), "general".to_string()],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.25, 0.40),
                operation_time_range: (40.0, 80.0),
                memory_usage_range: (1048576, 4194304), // 1MB to 4MB
                collision_rate_range: (0.05, 0.15),
                performance_rating: 6,
                memory_efficiency_rating: 7,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 1048576,     // 1MB
                recommended_memory_bytes: 4194304, // 4MB
                maximum_memory_bytes: 8388608,     // 8MB
                memory_growth_rate_bytes_per_op: 0.1,
            },
        };

        // Performance template
        let performance_template = ConfigurationTemplate {
            name: "performance".to_string(),
            description: "High-performance configuration optimized for speed".to_string(),
            config: TranspositionConfig::performance_optimized(),
            category: TemplateCategory::Performance,
            tags: vec![
                "fast".to_string(),
                "speed".to_string(),
                "optimized".to_string(),
            ],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.35, 0.55),
                operation_time_range: (20.0, 50.0),
                memory_usage_range: (8388608, 67108864), // 8MB to 64MB
                collision_rate_range: (0.03, 0.10),
                performance_rating: 9,
                memory_efficiency_rating: 4,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 8388608,      // 8MB
                recommended_memory_bytes: 33554432, // 32MB
                maximum_memory_bytes: 134217728,    // 128MB
                memory_growth_rate_bytes_per_op: 0.2,
            },
        };

        // Memory template
        let memory_template = ConfigurationTemplate {
            name: "memory".to_string(),
            description: "Memory-efficient configuration for constrained environments".to_string(),
            config: TranspositionConfig::memory_optimized(),
            category: TemplateCategory::Memory,
            tags: vec![
                "memory".to_string(),
                "efficient".to_string(),
                "small".to_string(),
            ],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.15, 0.30),
                operation_time_range: (60.0, 120.0),
                memory_usage_range: (262144, 1048576), // 256KB to 1MB
                collision_rate_range: (0.10, 0.25),
                performance_rating: 4,
                memory_efficiency_rating: 9,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 262144,      // 256KB
                recommended_memory_bytes: 1048576, // 1MB
                maximum_memory_bytes: 2097152,     // 2MB
                memory_growth_rate_bytes_per_op: 0.05,
            },
        };

        // High performance template
        let high_performance_template = ConfigurationTemplate {
            name: "high_performance".to_string(),
            description: "Maximum performance configuration for high-end systems".to_string(),
            config: TranspositionConfig {
                table_size: 1048576, // 1M entries
                replacement_policy: ReplacementPolicy::DepthPreferred,
                max_age: 0,
                enable_prefetching: true,
                enable_memory_mapping: false,
                max_memory_mb: 128,
                clear_between_games: false,
                enable_statistics: true,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: false,
                bucket_count: 512,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
            category: TemplateCategory::Performance,
            tags: vec![
                "maximum".to_string(),
                "high-end".to_string(),
                "server".to_string(),
            ],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.45, 0.65),
                operation_time_range: (15.0, 35.0),
                memory_usage_range: (16777216, 134217728), // 16MB to 128MB
                collision_rate_range: (0.02, 0.08),
                performance_rating: 10,
                memory_efficiency_rating: 2,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 16777216,     // 16MB
                recommended_memory_bytes: 67108864, // 64MB
                maximum_memory_bytes: 268435456,    // 256MB
                memory_growth_rate_bytes_per_op: 0.3,
            },
        };

        // Development template
        let development_template = ConfigurationTemplate {
            name: "development".to_string(),
            description: "Configuration optimized for development and debugging".to_string(),
            config: TranspositionConfig {
                table_size: 16384, // 16K entries
                replacement_policy: ReplacementPolicy::AlwaysReplace,
                max_age: 50,
                enable_prefetching: false,
                enable_memory_mapping: false,
                max_memory_mb: 4,
                clear_between_games: true,
                enable_statistics: true,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: true,
                bucket_count: 16,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
            category: TemplateCategory::Development,
            tags: vec![
                "development".to_string(),
                "debug".to_string(),
                "testing".to_string(),
            ],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.20, 0.35),
                operation_time_range: (50.0, 100.0),
                memory_usage_range: (524288, 2097152), // 512KB to 2MB
                collision_rate_range: (0.08, 0.20),
                performance_rating: 5,
                memory_efficiency_rating: 8,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 524288,      // 512KB
                recommended_memory_bytes: 2097152, // 2MB
                maximum_memory_bytes: 4194304,     // 4MB
                memory_growth_rate_bytes_per_op: 0.15,
            },
        };

        // Production template
        let production_template = ConfigurationTemplate {
            name: "production".to_string(),
            description: "Production-ready configuration with balanced performance and reliability"
                .to_string(),
            config: TranspositionConfig {
                table_size: 262144, // 256K entries
                replacement_policy: ReplacementPolicy::DepthPreferred,
                max_age: 0,
                enable_prefetching: false,
                enable_memory_mapping: false,
                max_memory_mb: 32,
                clear_between_games: false,
                enable_statistics: true,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: false,
                bucket_count: 256,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
            category: TemplateCategory::Production,
            tags: vec![
                "production".to_string(),
                "stable".to_string(),
                "reliable".to_string(),
            ],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.30, 0.45),
                operation_time_range: (30.0, 60.0),
                memory_usage_range: (4194304, 16777216), // 4MB to 16MB
                collision_rate_range: (0.05, 0.12),
                performance_rating: 7,
                memory_efficiency_rating: 6,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 4194304,      // 4MB
                recommended_memory_bytes: 16777216, // 16MB
                maximum_memory_bytes: 33554432,     // 32MB
                memory_growth_rate_bytes_per_op: 0.12,
            },
        };

        // Add templates
        self.add_builtin_template(default_template);
        self.add_builtin_template(performance_template);
        self.add_builtin_template(memory_template);
        self.add_builtin_template(high_performance_template);
        self.add_builtin_template(development_template);
        self.add_builtin_template(production_template);
    }

    /// Add a built-in template
    fn add_builtin_template(&mut self, template: ConfigurationTemplate) {
        let name = template.name.clone();
        let metadata = TemplateMetadata {
            created_at: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            usage_count: 0,
            user_rating: None,
            author: "System".to_string(),
        };

        self.builtin_templates.insert(name.clone(), template);
        self.template_metadata.insert(name, metadata);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&ConfigurationTemplate> {
        self.builtin_templates
            .get(name)
            .or_else(|| self.custom_templates.get(name))
    }

    /// Get all templates in a category
    pub fn get_templates_by_category(
        &self,
        category: &TemplateCategory,
    ) -> Vec<&ConfigurationTemplate> {
        let mut templates = Vec::new();

        for template in self.builtin_templates.values() {
            if template.category == *category {
                templates.push(template);
            }
        }

        for template in self.custom_templates.values() {
            if template.category == *category {
                templates.push(template);
            }
        }

        templates
    }

    /// Get templates by tags
    pub fn get_templates_by_tags(&self, tags: &[String]) -> Vec<&ConfigurationTemplate> {
        let mut templates = Vec::new();

        for template in self.builtin_templates.values() {
            if tags.iter().any(|tag| template.tags.contains(tag)) {
                templates.push(template);
            }
        }

        for template in self.custom_templates.values() {
            if tags.iter().any(|tag| template.tags.contains(tag)) {
                templates.push(template);
            }
        }

        templates
    }

    /// Add a custom template
    pub fn add_custom_template(&mut self, template: ConfigurationTemplate) -> Result<(), String> {
        // Validate template
        let validator = ConfigurationValidator::new();
        let validation_result = validator.validate_template(&template);

        if !validation_result.is_valid {
            return Err(format!(
                "Template validation failed: {:?}",
                validation_result.error_message
            ));
        }

        let name = template.name.clone();
        let metadata = TemplateMetadata {
            created_at: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            usage_count: 0,
            user_rating: None,
            author: "User".to_string(),
        };

        self.custom_templates.insert(name.clone(), template);
        self.template_metadata.insert(name, metadata);

        Ok(())
    }

    /// Remove a custom template
    pub fn remove_custom_template(&mut self, name: &str) -> bool {
        if self.builtin_templates.contains_key(name) {
            false // Cannot remove built-in templates
        } else {
            let removed = self.custom_templates.remove(name).is_some();
            self.template_metadata.remove(name);
            removed
        }
    }

    /// Update template usage count
    pub fn update_template_usage(&mut self, name: &str) {
        if let Some(metadata) = self.template_metadata.get_mut(name) {
            metadata.usage_count += 1;
        }
    }

    /// Rate a template
    pub fn rate_template(&mut self, name: &str, rating: f64) -> Result<(), String> {
        if rating < 1.0 || rating > 5.0 {
            return Err("Rating must be between 1.0 and 5.0".to_string());
        }

        if let Some(metadata) = self.template_metadata.get_mut(name) {
            metadata.user_rating = Some(rating);
            Ok(())
        } else {
            Err("Template not found".to_string())
        }
    }

    /// Get template metadata
    pub fn get_template_metadata(&self, name: &str) -> Option<&TemplateMetadata> {
        self.template_metadata.get(name)
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.builtin_templates.keys().cloned());
        names.extend(self.custom_templates.keys().cloned());
        names.sort();
        names
    }

    /// Export templates to JSON
    pub fn export_templates(&self, category: Option<TemplateCategory>) -> Result<String, String> {
        let mut templates_to_export = Vec::new();

        for template in self.builtin_templates.values() {
            if category.is_none() || template.category == *category.as_ref().unwrap() {
                templates_to_export.push(template.clone());
            }
        }

        for template in self.custom_templates.values() {
            if category.is_none() || template.category == *category.as_ref().unwrap() {
                templates_to_export.push(template.clone());
            }
        }

        serde_json::to_string_pretty(&templates_to_export)
            .map_err(|e| format!("Failed to serialize templates: {}", e))
    }

    /// Import templates from JSON
    pub fn import_templates(&mut self, json: &str, overwrite: bool) -> Result<usize, String> {
        let templates: Vec<ConfigurationTemplate> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize templates: {}", e))?;

        let mut imported_count = 0;

        for template in templates {
            let name = template.name.clone();

            // Check if template already exists
            if self.builtin_templates.contains_key(&name) {
                return Err(format!(
                    "Cannot import template '{}': built-in template exists",
                    name
                ));
            }

            if self.custom_templates.contains_key(&name) && !overwrite {
                return Err(format!(
                    "Template '{}' already exists. Use overwrite=true to replace",
                    name
                ));
            }

            // Add template
            if let Err(e) = self.add_custom_template(template) {
                return Err(format!("Failed to add template '{}': {}", name, e));
            }

            imported_count += 1;
        }

        Ok(imported_count)
    }
}

impl ConfigurationValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        let mut validator = Self {
            validation_rules: Vec::new(),
            performance_benchmarks: HashMap::new(),
        };

        // Initialize validation rules
        validator.initialize_validation_rules();

        validator
    }

    /// Initialize validation rules
    fn initialize_validation_rules(&mut self) {
        // Rule 1: Table size validation
        self.validation_rules.push(ValidationRule {
            name: "table_size_validation".to_string(),
            description: "Validate table size is reasonable".to_string(),
            validator: |config| {
                if config.table_size == 0 {
                    ValidationResult {
                        is_valid: false,
                        error_message: Some("Table size cannot be zero".to_string()),
                        warning_message: None,
                        suggestion: Some("Use a minimum table size of 1024 entries".to_string()),
                    }
                } else if config.table_size > 16777216 {
                    // 16M entries
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: Some(
                            "Very large table size may cause memory issues".to_string(),
                        ),
                        suggestion: Some(
                            "Consider using a smaller table size or ensure sufficient memory"
                                .to_string(),
                        ),
                    }
                } else {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: None,
                        suggestion: None,
                    }
                }
            },
            severity: ValidationSeverity::Error,
        });

        // Rule 2: Power of two recommendation
        self.validation_rules.push(ValidationRule {
            name: "power_of_two_recommendation".to_string(),
            description: "Recommend power of two table sizes".to_string(),
            validator: |config| {
                if !config.table_size.is_power_of_two() {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: Some("Table size is not a power of two".to_string()),
                        suggestion: Some(
                            "Consider using a power of two for optimal hash performance"
                                .to_string(),
                        ),
                    }
                } else {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: None,
                        suggestion: None,
                    }
                }
            },
            severity: ValidationSeverity::Warning,
        });

        // Rule 3: Statistics recommendation for large tables
        self.validation_rules.push(ValidationRule {
            name: "statistics_recommendation".to_string(),
            description: "Recommend statistics for large tables".to_string(),
            validator: |config| {
                if config.table_size > 65536 && !config.enable_statistics {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: Some(
                            "Consider enabling statistics for large tables".to_string(),
                        ),
                        suggestion: Some(
                            "Statistics help monitor performance and identify issues".to_string(),
                        ),
                    }
                } else {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: None,
                        suggestion: None,
                    }
                }
            },
            severity: ValidationSeverity::Warning,
        });

        // Rule 4: Memory usage validation
        self.validation_rules.push(ValidationRule {
            name: "memory_usage_validation".to_string(),
            description: "Validate memory usage is reasonable".to_string(),
            validator: |config| {
                let estimated_memory = config.table_size * 16; // 16 bytes per entry
                if estimated_memory > 268435456 {
                    // 256MB
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: Some(format!(
                            "Large memory usage: {:.1} MB",
                            estimated_memory as f64 / 1024.0 / 1024.0
                        )),
                        suggestion: Some(
                            "Consider reducing table size or ensure sufficient system memory"
                                .to_string(),
                        ),
                    }
                } else {
                    ValidationResult {
                        is_valid: true,
                        error_message: None,
                        warning_message: None,
                        suggestion: None,
                    }
                }
            },
            severity: ValidationSeverity::Warning,
        });
    }

    /// Validate a configuration
    pub fn validate_configuration(&self, config: &TranspositionConfig) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.validation_rules {
            let result = (rule.validator)(config);
            results.push(result);
        }

        results
    }

    /// Validate a template
    pub fn validate_template(&self, template: &ConfigurationTemplate) -> ValidationResult {
        // Validate the configuration
        let config_results = self.validate_configuration(&template.config);

        // Check for errors
        for result in &config_results {
            if !result.is_valid {
                return ValidationResult {
                    is_valid: false,
                    error_message: result.error_message.clone(),
                    warning_message: None,
                    suggestion: result.suggestion.clone(),
                };
            }
        }

        // Validate template metadata
        if template.name.is_empty() {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Template name cannot be empty".to_string()),
                warning_message: None,
                suggestion: Some("Provide a descriptive name for the template".to_string()),
            };
        }

        if template.description.is_empty() {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Template description cannot be empty".to_string()),
                warning_message: None,
                suggestion: Some(
                    "Provide a description explaining the template's purpose".to_string(),
                ),
            };
        }

        // Check performance profile consistency
        if template.performance_profile.hit_rate_range.0
            > template.performance_profile.hit_rate_range.1
        {
            return ValidationResult {
                is_valid: false,
                error_message: Some("Invalid hit rate range".to_string()),
                warning_message: None,
                suggestion: Some(
                    "Minimum hit rate must be less than or equal to maximum hit rate".to_string(),
                ),
            };
        }

        ValidationResult {
            is_valid: true,
            error_message: None,
            warning_message: None,
            suggestion: None,
        }
    }

    /// Get validation rules
    pub fn get_validation_rules(&self) -> Vec<ValidationRule> {
        self.validation_rules.clone()
    }

    /// Add a custom validation rule
    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    /// Benchmark a configuration
    pub fn benchmark_configuration(&self, config: &TranspositionConfig) -> BenchmarkResults {
        // Simulated benchmark results based on configuration
        let base_operation_time = 50.0;
        let size_factor = (config.table_size as f64 / 65536.0).log2(); // Logarithmic scaling
        let alignment_factor = if config.enable_memory_mapping {
            0.8
        } else {
            1.0
        };
        let prefetch_factor = if config.enable_prefetching { 0.9 } else { 1.0 };

        let avg_operation_time_us =
            base_operation_time * alignment_factor * prefetch_factor * (1.0 + size_factor * 0.1);
        let hit_rate_percentage =
            (0.2 + (config.table_size as f64 / 1048576.0) * 0.3).min(0.6) * 100.0;
        let memory_usage_bytes = config.table_size * 16;
        let collision_rate_percentage =
            (0.15 - (config.table_size as f64 / 1048576.0) * 0.1).max(0.02) * 100.0;
        let throughput_ops_per_sec = 1000000.0 / avg_operation_time_us;

        BenchmarkResults {
            avg_operation_time_us,
            hit_rate_percentage,
            memory_usage_bytes: memory_usage_bytes as u64,
            collision_rate_percentage,
            throughput_ops_per_sec,
        }
    }
}

impl Default for ConfigurationTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConfigurationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = ConfigurationTemplateManager::new();

        assert!(manager.get_template("default").is_some());
        assert!(manager.get_template("performance").is_some());
        assert!(manager.get_template("memory").is_some());
    }

    #[test]
    fn test_template_categories() {
        let manager = ConfigurationTemplateManager::new();

        let performance_templates =
            manager.get_templates_by_category(&TemplateCategory::Performance);
        assert!(performance_templates.len() >= 2); // performance and high_performance

        let memory_templates = manager.get_templates_by_category(&TemplateCategory::Memory);
        assert!(memory_templates.len() >= 1); // memory
    }

    #[test]
    fn test_configuration_validation() {
        let validator = ConfigurationValidator::new();

        let valid_config = TranspositionConfig::default();
        let results = validator.validate_configuration(&valid_config);
        assert!(results.iter().all(|r| r.is_valid));

        let invalid_config = TranspositionConfig {
            table_size: 0,
            ..TranspositionConfig::default()
        };
        let results = validator.validate_configuration(&invalid_config);
        assert!(results.iter().any(|r| !r.is_valid));
    }

    #[test]
    fn test_template_validation() {
        let validator = ConfigurationValidator::new();
        let manager = ConfigurationTemplateManager::new();

        let template = manager.get_template("default").unwrap();
        let result = validator.validate_template(template);
        assert!(result.is_valid);
    }

    #[test]
    fn test_custom_template_management() {
        let mut manager = ConfigurationTemplateManager::new();

        let custom_template = ConfigurationTemplate {
            name: "custom_test".to_string(),
            description: "Test template".to_string(),
            config: TranspositionConfig::default(),
            category: TemplateCategory::Custom,
            tags: vec!["test".to_string()],
            performance_profile: PerformanceProfile {
                hit_rate_range: (0.2, 0.4),
                operation_time_range: (40.0, 80.0),
                memory_usage_range: (1000000, 4000000),
                collision_rate_range: (0.05, 0.15),
                performance_rating: 6,
                memory_efficiency_rating: 7,
            },
            memory_requirements: MemoryRequirements {
                minimum_memory_bytes: 1000000,
                recommended_memory_bytes: 4000000,
                maximum_memory_bytes: 8000000,
                memory_growth_rate_bytes_per_op: 0.1,
            },
        };

        assert!(manager.add_custom_template(custom_template).is_ok());
        assert!(manager.get_template("custom_test").is_some());
        assert!(manager.remove_custom_template("custom_test"));
        assert!(!manager.remove_custom_template("default")); // Cannot remove built-in
    }

    #[test]
    fn test_configuration_benchmarking() {
        let validator = ConfigurationValidator::new();
        let config = TranspositionConfig::default();

        let benchmark = validator.benchmark_configuration(&config);
        assert!(benchmark.avg_operation_time_us > 0.0);
        assert!(benchmark.hit_rate_percentage > 0.0);
        assert!(benchmark.memory_usage_bytes > 0);
    }
}
