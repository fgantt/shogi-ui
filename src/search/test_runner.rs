//! Test runner for comprehensive transposition table testing
//!
//! This module provides a test runner that can execute the comprehensive
//! test suite and generate detailed reports.

use crate::search::comprehensive_tests::ComprehensiveTestResults;
use crate::search::*;
use std::time::Instant;

/// Test runner for comprehensive testing
pub struct TestRunner {
    /// Test suite instance
    test_suite: ComprehensiveTestSuite,
    /// Execution configuration
    config: TestRunnerConfig,
}

/// Test runner configuration
#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    /// Whether to run all tests or just specific categories
    pub test_categories: Vec<TestCategory>,
    /// Whether to generate detailed reports
    pub detailed_reporting: bool,
    /// Whether to stop on first failure
    pub stop_on_failure: bool,
    /// Output format for reports
    pub output_format: OutputFormat,
}

/// Test categories
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    UnitTests,
    IntegrationTests,
    PerformanceBenchmarks,
    StressTests,
    MemoryLeakTests,
    RegressionTests,
    PositionValidation,
}

/// Output formats for reports
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Console,
    Json,
    Html,
    Csv,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        Self {
            test_suite: ComprehensiveTestSuite::new(),
            config: TestRunnerConfig::default(),
        }
    }

    /// Create a test runner with custom configuration
    pub fn with_config(config: TestRunnerConfig) -> Self {
        Self {
            test_suite: ComprehensiveTestSuite::new(),
            config,
        }
    }

    /// Run the comprehensive test suite
    pub fn run_tests(&mut self) -> TestExecutionResult {
        println!("🚀 Starting Comprehensive Test Runner...");
        let start_time = Instant::now();

        let results = if self.config.test_categories.is_empty() {
            // Run all tests
            self.test_suite.run_all_tests().clone()
        } else {
            // Run specific test categories
            self.run_selected_tests().clone()
        };

        let execution_time = start_time.elapsed();

        // Generate report
        let report = self.generate_report(&results, execution_time);

        TestExecutionResult {
            results: results.clone(),
            execution_time,
            report,
            success: self.is_successful(&results),
        }
    }

    /// Run selected test categories
    fn run_selected_tests(&mut self) -> &ComprehensiveTestResults {
        println!("🎯 Running selected test categories...");

        for category in &self.config.test_categories {
            match category {
                TestCategory::UnitTests => {
                    println!("📋 Running Unit Tests...");
                    self.test_suite.run_unit_tests();
                }
                TestCategory::IntegrationTests => {
                    println!("🔗 Running Integration Tests...");
                    self.test_suite.run_integration_tests();
                }
                TestCategory::PerformanceBenchmarks => {
                    println!("⚡ Running Performance Benchmarks...");
                    self.test_suite.run_performance_benchmarks();
                }
                TestCategory::StressTests => {
                    println!("💪 Running Stress Tests...");
                    self.test_suite.run_stress_tests();
                }
                TestCategory::MemoryLeakTests => {
                    println!("🧠 Running Memory Leak Tests...");
                    self.test_suite.run_memory_leak_tests();
                }
                TestCategory::RegressionTests => {
                    println!("🔄 Running Regression Tests...");
                    self.test_suite.run_regression_tests();
                }
                TestCategory::PositionValidation => {
                    println!("🎯 Running Position Validation...");
                    self.test_suite.run_position_validation();
                }
            }

            if self.config.stop_on_failure && !self.is_category_successful(category) {
                println!("⚠️  Stopping on failure in category: {:?}", category);
                break;
            }
        }

        &self.test_suite.results
    }

    /// Check if a test category was successful
    fn is_category_successful(&self, category: &TestCategory) -> bool {
        match category {
            TestCategory::UnitTests => {
                self.test_suite.results.unit_tests.tests_run == 0
                    || self.test_suite.results.unit_tests.tests_passed
                        == self.test_suite.results.unit_tests.tests_run
            }
            TestCategory::IntegrationTests => {
                self.test_suite.results.integration_tests.tests_run == 0
                    || self.test_suite.results.integration_tests.tests_passed
                        == self.test_suite.results.integration_tests.tests_run
            }
            TestCategory::PerformanceBenchmarks => {
                self.test_suite.results.performance_tests.benchmarks_run == 0
                    || self.test_suite.results.performance_tests.benchmarks_passed
                        == self.test_suite.results.performance_tests.benchmarks_run * 2
            }
            TestCategory::StressTests => {
                self.test_suite.results.stress_tests.tests_run == 0
                    || self.test_suite.results.stress_tests.tests_passed
                        == self.test_suite.results.stress_tests.tests_run
            }
            TestCategory::MemoryLeakTests => {
                self.test_suite.results.memory_tests.tests_run == 0
                    || self.test_suite.results.memory_tests.tests_passed
                        == self.test_suite.results.memory_tests.tests_run
            }
            TestCategory::RegressionTests => {
                self.test_suite.results.regression_tests.tests_run == 0
                    || self.test_suite.results.regression_tests.tests_passed
                        == self.test_suite.results.regression_tests.tests_run
            }
            TestCategory::PositionValidation => {
                self.test_suite.results.position_validation.positions_tested == 0
                    || self.test_suite.results.position_validation.positions_passed
                        == self.test_suite.results.position_validation.positions_tested
            }
        }
    }

    /// Check if overall test execution was successful
    fn is_successful(&self, results: &ComprehensiveTestResults) -> bool {
        let total_tests = results.unit_tests.tests_run
            + results.integration_tests.tests_run
            + results.performance_tests.benchmarks_run * 2
            + results.stress_tests.tests_run
            + results.memory_tests.tests_run
            + results.regression_tests.tests_run
            + results.position_validation.positions_tested;

        let total_passed = results.unit_tests.tests_passed
            + results.integration_tests.tests_passed
            + results.performance_tests.benchmarks_passed
            + results.stress_tests.tests_passed
            + results.memory_tests.tests_passed
            + results.regression_tests.tests_passed
            + results.position_validation.positions_passed;

        total_tests > 0 && total_passed == total_tests
    }

    /// Generate detailed test report
    fn generate_report(
        &self,
        results: &ComprehensiveTestResults,
        execution_time: std::time::Duration,
    ) -> String {
        match self.config.output_format {
            OutputFormat::Console => self.generate_console_report(results, execution_time),
            OutputFormat::Json => self.generate_json_report(results, execution_time),
            OutputFormat::Html => self.generate_html_report(results, execution_time),
            OutputFormat::Csv => self.generate_csv_report(results, execution_time),
        }
    }

    /// Generate console report
    fn generate_console_report(
        &self,
        results: &ComprehensiveTestResults,
        execution_time: std::time::Duration,
    ) -> String {
        let mut report = String::new();

        report.push_str(&format!("\n📊 COMPREHENSIVE TEST SUITE REPORT\n"));
        report.push_str(&format!(
            "Execution Time: {:.2}s\n",
            execution_time.as_secs_f64()
        ));
        report.push_str(&format!("=====================================\n\n"));

        // Unit Tests
        report.push_str(&format!(
            "📋 Unit Tests: {}/{} passed",
            results.unit_tests.tests_passed, results.unit_tests.tests_run
        ));
        if !results.unit_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.unit_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Integration Tests
        report.push_str(&format!(
            "🔗 Integration Tests: {}/{} passed",
            results.integration_tests.tests_passed, results.integration_tests.tests_run
        ));
        if !results.integration_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.integration_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Performance Tests
        report.push_str(&format!(
            "⚡ Performance Tests: {}/{} passed",
            results.performance_tests.benchmarks_passed,
            results.performance_tests.benchmarks_run * 2
        ));
        if !results.performance_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.performance_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Stress Tests
        report.push_str(&format!(
            "💪 Stress Tests: {}/{} passed",
            results.stress_tests.tests_passed, results.stress_tests.tests_run
        ));
        if !results.stress_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.stress_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Memory Tests
        report.push_str(&format!(
            "🧠 Memory Tests: {}/{} passed",
            results.memory_tests.tests_passed, results.memory_tests.tests_run
        ));
        if !results.memory_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.memory_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Regression Tests
        report.push_str(&format!(
            "🔄 Regression Tests: {}/{} passed",
            results.regression_tests.tests_passed, results.regression_tests.tests_run
        ));
        if !results.regression_tests.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.regression_tests.failures.len()
            ));
        }
        report.push_str("\n");

        // Position Validation
        report.push_str(&format!(
            "🎯 Position Validation: {}/{} passed",
            results.position_validation.positions_passed,
            results.position_validation.positions_tested
        ));
        if !results.position_validation.failures.is_empty() {
            report.push_str(&format!(
                " ❌ {} failures",
                results.position_validation.failures.len()
            ));
        }
        report.push_str("\n");

        // Overall Results
        let total_tests = results.unit_tests.tests_run
            + results.integration_tests.tests_run
            + results.performance_tests.benchmarks_run * 2
            + results.stress_tests.tests_run
            + results.memory_tests.tests_run
            + results.regression_tests.tests_run
            + results.position_validation.positions_tested;

        let total_passed = results.unit_tests.tests_passed
            + results.integration_tests.tests_passed
            + results.performance_tests.benchmarks_passed
            + results.stress_tests.tests_passed
            + results.memory_tests.tests_passed
            + results.regression_tests.tests_passed
            + results.position_validation.positions_passed;

        let success_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        report.push_str(&format!(
            "\n🎯 OVERALL RESULTS: {}/{} tests passed ({:.1}%)\n",
            total_passed, total_tests, success_rate
        ));

        if total_passed == total_tests {
            report.push_str(
                "🎉 ALL TESTS PASSED! Transposition table enhancements are ready for production.\n",
            );
        } else {
            report.push_str("⚠️  Some tests failed. Please review the failures above.\n");
        }

        // Performance Summary
        if results.performance_tests.benchmarks_run > 0 {
            report.push_str(&format!("\n📈 PERFORMANCE SUMMARY:\n"));
            report.push_str(&format!(
                "Average Operation Time: {:.2}μs\n",
                results.performance_tests.avg_operation_time_us
            ));
            report.push_str(&format!(
                "Hit Rate: {:.1}%\n",
                results.performance_tests.hit_rate * 100.0
            ));
            report.push_str(&format!(
                "Speed Improvement: {:.1}%\n",
                results.performance_tests.speed_improvement
            ));
        }

        report
    }

    /// Generate JSON report
    fn generate_json_report(
        &self,
        _results: &ComprehensiveTestResults,
        _execution_time: std::time::Duration,
    ) -> String {
        // TODO: Implement JSON report generation
        "{\"report\": \"JSON report not yet implemented\"}".to_string()
    }

    /// Generate HTML report
    fn generate_html_report(
        &self,
        _results: &ComprehensiveTestResults,
        _execution_time: std::time::Duration,
    ) -> String {
        // TODO: Implement HTML report generation
        "<html><body><h1>HTML report not yet implemented</h1></body></html>".to_string()
    }

    /// Generate CSV report
    fn generate_csv_report(
        &self,
        _results: &ComprehensiveTestResults,
        _execution_time: std::time::Duration,
    ) -> String {
        // TODO: Implement CSV report generation
        "category,tests_run,tests_passed,success_rate\nCSV report not yet implemented".to_string()
    }
}

/// Test execution result
#[derive(Debug)]
pub struct TestExecutionResult {
    /// Test results
    pub results: ComprehensiveTestResults,
    /// Execution time
    pub execution_time: std::time::Duration,
    /// Generated report
    pub report: String,
    /// Whether execution was successful
    pub success: bool,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        Self {
            test_categories: Vec::new(), // Empty means run all
            detailed_reporting: true,
            stop_on_failure: false,
            output_format: OutputFormat::Console,
        }
    }
}

/// Convenience function to run all tests
pub fn run_all_tests() -> TestExecutionResult {
    let mut runner = TestRunner::new();
    runner.run_tests()
}

/// Convenience function to run specific test categories
pub fn run_test_categories(categories: Vec<TestCategory>) -> TestExecutionResult {
    let config = TestRunnerConfig {
        test_categories: categories,
        ..Default::default()
    };
    let mut runner = TestRunner::with_config(config);
    runner.run_tests()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new();
        assert_eq!(runner.config.test_categories.len(), 0);
        assert_eq!(runner.config.output_format, OutputFormat::Console);
    }

    #[test]
    fn test_runner_with_config() {
        let config = TestRunnerConfig {
            test_categories: vec![TestCategory::UnitTests, TestCategory::IntegrationTests],
            detailed_reporting: false,
            stop_on_failure: true,
            output_format: OutputFormat::Json,
        };

        let runner = TestRunner::with_config(config.clone());
        assert_eq!(runner.config.test_categories.len(), 2);
        assert_eq!(runner.config.detailed_reporting, false);
        assert_eq!(runner.config.stop_on_failure, true);
        assert_eq!(runner.config.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_test_category_equality() {
        assert_eq!(TestCategory::UnitTests, TestCategory::UnitTests);
        assert_ne!(TestCategory::UnitTests, TestCategory::IntegrationTests);
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Console, OutputFormat::Console);
        assert_ne!(OutputFormat::Console, OutputFormat::Json);
    }
}
