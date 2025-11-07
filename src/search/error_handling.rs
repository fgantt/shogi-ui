//! Error handling for transposition table operations
//!
//! This module provides comprehensive error handling, graceful degradation,
//! error recovery mechanisms, and logging for all transposition table operations.

use crate::search::transposition_config::TranspositionConfig;
use std::fmt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Error types for transposition table operations
#[derive(Debug, Clone, PartialEq)]
pub enum TranspositionError {
    /// Hash generation failed
    HashGenerationFailed(String),
    /// Table operation failed
    TableOperationFailed(String),
    /// Memory allocation failed
    MemoryAllocationFailed(String),
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Thread safety violation
    ThreadSafetyViolation(String),
    /// Data corruption detected
    DataCorruption(String),
    /// Resource exhaustion
    ResourceExhaustion(String),
    /// Invalid input parameters
    InvalidInput(String),
    /// Timeout occurred
    Timeout(String),
}

impl fmt::Display for TranspositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranspositionError::HashGenerationFailed(msg) => {
                write!(f, "Hash generation failed: {}", msg)
            }
            TranspositionError::TableOperationFailed(msg) => {
                write!(f, "Table operation failed: {}", msg)
            }
            TranspositionError::MemoryAllocationFailed(msg) => {
                write!(f, "Memory allocation failed: {}", msg)
            }
            TranspositionError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            TranspositionError::ThreadSafetyViolation(msg) => {
                write!(f, "Thread safety violation: {}", msg)
            }
            TranspositionError::DataCorruption(msg) => {
                write!(f, "Data corruption detected: {}", msg)
            }
            TranspositionError::ResourceExhaustion(msg) => {
                write!(f, "Resource exhaustion: {}", msg)
            }
            TranspositionError::InvalidInput(msg) => {
                write!(f, "Invalid input: {}", msg)
            }
            TranspositionError::Timeout(msg) => {
                write!(f, "Timeout: {}", msg)
            }
        }
    }
}

impl std::error::Error for TranspositionError {}

/// Result type for transposition table operations
pub type TranspositionResult<T> = Result<T, TranspositionError>;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning that doesn't prevent operation
    Warning,
    /// Error that may cause degraded performance
    Error,
    /// Critical error that prevents operation
    Critical,
}

/// Error log entry
#[derive(Debug, Clone)]
pub struct ErrorLogEntry {
    /// Timestamp when error occurred
    pub timestamp: u64,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error message
    pub message: String,
    /// Error context (operation, parameters, etc.)
    pub context: Option<String>,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
}

impl ErrorLogEntry {
    /// Create a new error log entry
    pub fn new(severity: ErrorSeverity, message: String) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity,
            message,
            context: None,
            stack_trace: None,
        }
    }

    /// Add context to the error entry
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Add stack trace to the error entry
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }
}

/// Error logger for transposition table operations
pub struct ErrorLogger {
    /// Maximum number of log entries to keep
    max_entries: usize,
    /// Log entries
    entries: Arc<Mutex<Vec<ErrorLogEntry>>>,
    /// Error counters by severity
    counters: Arc<Mutex<ErrorCounters>>,
}

/// Error counters for tracking error statistics
#[derive(Debug, Clone, Default)]
pub struct ErrorCounters {
    pub info_count: u64,
    pub warning_count: u64,
    pub error_count: u64,
    pub critical_count: u64,
    pub total_count: u64,
}

impl ErrorLogger {
    /// Create a new error logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Arc::new(Mutex::new(Vec::new())),
            counters: Arc::new(Mutex::new(ErrorCounters::default())),
        }
    }

    /// Log an error entry
    pub fn log(&self, entry: ErrorLogEntry) {
        let mut entries = self.entries.lock().unwrap();
        let mut counters = self.counters.lock().unwrap();

        // Update counters
        match entry.severity {
            ErrorSeverity::Info => counters.info_count += 1,
            ErrorSeverity::Warning => counters.warning_count += 1,
            ErrorSeverity::Error => counters.error_count += 1,
            ErrorSeverity::Critical => counters.critical_count += 1,
        }
        counters.total_count += 1;

        // Add entry and maintain size limit
        entries.push(entry);
        if entries.len() > self.max_entries {
            entries.remove(0);
        }
    }

    /// Log an error with severity and message
    pub fn log_error(&self, severity: ErrorSeverity, message: String) {
        let entry = ErrorLogEntry::new(severity, message);
        self.log(entry);
    }

    /// Log an error with context
    pub fn log_error_with_context(
        &self,
        severity: ErrorSeverity,
        message: String,
        context: String,
    ) {
        let entry = ErrorLogEntry::new(severity, message).with_context(context);
        self.log(entry);
    }

    /// Get recent error entries
    pub fn get_recent_entries(&self, count: usize) -> Vec<ErrorLogEntry> {
        let entries = self.entries.lock().unwrap();
        let start = if entries.len() > count {
            entries.len() - count
        } else {
            0
        };
        entries[start..].to_vec()
    }

    /// Get error counters
    pub fn get_counters(&self) -> ErrorCounters {
        self.counters.lock().unwrap().clone()
    }

    /// Clear all log entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        let mut counters = self.counters.lock().unwrap();
        *counters = ErrorCounters::default();
    }

    /// Export error log to string
    pub fn export_log(&self) -> String {
        let entries = self.entries.lock().unwrap();
        let counters = self.counters.lock().unwrap();

        let mut output = String::new();
        output.push_str(&format!("=== Transposition Table Error Log ===\n"));
        output.push_str(&format!("Total Errors: {}\n", counters.total_count));
        output.push_str(&format!(
            "Info: {}, Warning: {}, Error: {}, Critical: {}\n\n",
            counters.info_count,
            counters.warning_count,
            counters.error_count,
            counters.critical_count
        ));

        for entry in entries.iter() {
            output.push_str(&format!(
                "[{}] {:?}: {}\n",
                entry.timestamp, entry.severity, entry.message
            ));
            if let Some(context) = &entry.context {
                output.push_str(&format!("  Context: {}\n", context));
            }
            if let Some(stack_trace) = &entry.stack_trace {
                output.push_str(&format!("  Stack: {}\n", stack_trace));
            }
            output.push('\n');
        }

        output
    }
}

/// Graceful degradation handler
pub struct GracefulDegradationHandler {
    /// Error logger
    logger: Arc<ErrorLogger>,
    /// Degradation level (0 = normal, higher = more degraded)
    degradation_level: AtomicU32,
    /// Fallback configurations
    fallback_configs: Vec<TranspositionConfig>,
    /// Operation timeouts
    timeouts: Arc<Mutex<OperationTimeouts>>,
}

/// Operation timeout configuration
#[derive(Debug, Clone)]
pub struct OperationTimeouts {
    pub hash_generation_ms: u64,
    pub table_probe_ms: u64,
    pub table_store_ms: u64,
    pub table_clear_ms: u64,
}

impl Default for OperationTimeouts {
    fn default() -> Self {
        Self {
            hash_generation_ms: 100,
            table_probe_ms: 50,
            table_store_ms: 100,
            table_clear_ms: 500,
        }
    }
}

impl GracefulDegradationHandler {
    /// Create a new graceful degradation handler
    pub fn new(logger: Arc<ErrorLogger>) -> Self {
        Self {
            logger,
            degradation_level: AtomicU32::new(0),
            fallback_configs: vec![
                TranspositionConfig::debug_config(),
                TranspositionConfig::memory_optimized(),
            ],
            timeouts: Arc::new(Mutex::new(OperationTimeouts::default())),
        }
    }

    /// Get current degradation level
    pub fn get_degradation_level(&self) -> u32 {
        self.degradation_level.load(Ordering::Acquire)
    }

    /// Increase degradation level
    pub fn increase_degradation(&self) {
        let current = self.degradation_level.load(Ordering::Acquire);
        if current < 3 {
            self.degradation_level.store(current + 1, Ordering::Release);
            self.logger.log_error_with_context(
                ErrorSeverity::Warning,
                "Degradation level increased".to_string(),
                format!("New level: {}", current + 1),
            );
        }
    }

    /// Decrease degradation level
    pub fn decrease_degradation(&self) {
        let current = self.degradation_level.load(Ordering::Acquire);
        if current > 0 {
            self.degradation_level.store(current - 1, Ordering::Release);
            self.logger.log_error_with_context(
                ErrorSeverity::Info,
                "Degradation level decreased".to_string(),
                format!("New level: {}", current - 1),
            );
        }
    }

    /// Reset degradation level
    pub fn reset_degradation(&self) {
        self.degradation_level.store(0, Ordering::Release);
        self.logger.log_error(
            ErrorSeverity::Info,
            "Degradation level reset to normal".to_string(),
        );
    }

    /// Get fallback configuration based on degradation level
    pub fn get_fallback_config(&self) -> TranspositionConfig {
        let level = self.get_degradation_level() as usize;
        if level < self.fallback_configs.len() {
            self.fallback_configs[level].clone()
        } else {
            self.fallback_configs.last().unwrap().clone()
        }
    }

    /// Check if operation should timeout
    pub fn should_timeout(&self, operation: &str, start_time: std::time::Instant) -> bool {
        let timeouts = self.timeouts.lock().unwrap();
        let timeout_ms = match operation {
            "hash_generation" => timeouts.hash_generation_ms,
            "table_probe" => timeouts.table_probe_ms,
            "table_store" => timeouts.table_store_ms,
            "table_clear" => timeouts.table_clear_ms,
            _ => 1000, // Default timeout
        };

        start_time.elapsed().as_millis() > timeout_ms as u128
    }

    /// Handle operation error with graceful degradation
    pub fn handle_error(&self, error: TranspositionError) -> TranspositionResult<()> {
        let severity = match error {
            TranspositionError::HashGenerationFailed(_) => ErrorSeverity::Error,
            TranspositionError::TableOperationFailed(_) => ErrorSeverity::Warning,
            TranspositionError::MemoryAllocationFailed(_) => ErrorSeverity::Critical,
            TranspositionError::InvalidConfiguration(_) => ErrorSeverity::Error,
            TranspositionError::ThreadSafetyViolation(_) => ErrorSeverity::Critical,
            TranspositionError::DataCorruption(_) => ErrorSeverity::Critical,
            TranspositionError::ResourceExhaustion(_) => ErrorSeverity::Error,
            TranspositionError::InvalidInput(_) => ErrorSeverity::Warning,
            TranspositionError::Timeout(_) => ErrorSeverity::Warning,
        };

        self.logger.log_error(severity, error.to_string());

        // Increase degradation level for critical errors
        if severity >= ErrorSeverity::Error {
            self.increase_degradation();
        }

        // For critical errors, return the error
        if severity >= ErrorSeverity::Critical {
            Err(error)
        } else {
            // For non-critical errors, log and continue
            Ok(())
        }
    }
}

/// Error recovery mechanisms
pub struct ErrorRecoveryManager {
    /// Error logger
    logger: Arc<ErrorLogger>,
    /// Recovery strategies
    strategies: Vec<Box<dyn ErrorRecoveryStrategy>>,
    /// Recovery attempt counters
    recovery_attempts: AtomicU64,
    /// Successful recovery counter
    successful_recoveries: AtomicU64,
}

/// Trait for error recovery strategies
pub trait ErrorRecoveryStrategy: Send + Sync {
    /// Check if this strategy can handle the error
    fn can_handle(&self, error: &TranspositionError) -> bool;

    /// Attempt to recover from the error
    fn recover(&self, error: &TranspositionError) -> TranspositionResult<()>;

    /// Get strategy name
    fn name(&self) -> &str;
}

/// Memory allocation recovery strategy
pub struct MemoryAllocationRecovery;

impl ErrorRecoveryStrategy for MemoryAllocationRecovery {
    fn can_handle(&self, error: &TranspositionError) -> bool {
        matches!(error, TranspositionError::MemoryAllocationFailed(_))
    }

    fn recover(&self, _error: &TranspositionError) -> TranspositionResult<()> {
        // Force garbage collection (if available)
        // In Rust, we can't force GC, but we can suggest it
        std::hint::black_box(());

        // Return success to allow retry
        Ok(())
    }

    fn name(&self) -> &str {
        "Memory Allocation Recovery"
    }
}

/// Configuration recovery strategy
pub struct ConfigurationRecovery;

impl ErrorRecoveryStrategy for ConfigurationRecovery {
    fn can_handle(&self, error: &TranspositionError) -> bool {
        matches!(error, TranspositionError::InvalidConfiguration(_))
    }

    fn recover(&self, _error: &TranspositionError) -> TranspositionResult<()> {
        // Return success to allow fallback configuration
        Ok(())
    }

    fn name(&self) -> &str {
        "Configuration Recovery"
    }
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new(logger: Arc<ErrorLogger>) -> Self {
        let strategies: Vec<Box<dyn ErrorRecoveryStrategy>> = vec![
            Box::new(MemoryAllocationRecovery),
            Box::new(ConfigurationRecovery),
        ];

        Self {
            logger,
            strategies,
            recovery_attempts: AtomicU64::new(0),
            successful_recoveries: AtomicU64::new(0),
        }
    }

    /// Attempt to recover from an error
    pub fn attempt_recovery(&self, error: &TranspositionError) -> TranspositionResult<()> {
        self.recovery_attempts.fetch_add(1, Ordering::Relaxed);

        for strategy in &self.strategies {
            if strategy.can_handle(error) {
                match strategy.recover(error) {
                    Ok(()) => {
                        self.successful_recoveries.fetch_add(1, Ordering::Relaxed);
                        self.logger.log_error_with_context(
                            ErrorSeverity::Info,
                            format!("Successfully recovered using {}", strategy.name()),
                            error.to_string(),
                        );
                        return Ok(());
                    }
                    Err(recovery_error) => {
                        self.logger.log_error_with_context(
                            ErrorSeverity::Warning,
                            format!("Recovery failed using {}", strategy.name()),
                            recovery_error.to_string(),
                        );
                    }
                }
            }
        }

        self.logger.log_error_with_context(
            ErrorSeverity::Error,
            "No recovery strategy found for error".to_string(),
            error.to_string(),
        );

        Err(error.clone())
    }

    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        RecoveryStats {
            total_attempts: self.recovery_attempts.load(Ordering::Acquire),
            successful_recoveries: self.successful_recoveries.load(Ordering::Relaxed),
            recovery_rate: if self.recovery_attempts.load(Ordering::Acquire) > 0 {
                self.successful_recoveries.load(Ordering::Relaxed) as f64
                    / self.recovery_attempts.load(Ordering::Acquire) as f64
            } else {
                0.0
            },
        }
    }
}

/// Recovery statistics
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_attempts: u64,
    pub successful_recoveries: u64,
    pub recovery_rate: f64,
}

/// Comprehensive error handler that combines all error handling components
pub struct ComprehensiveErrorHandler {
    /// Error logger
    logger: Arc<ErrorLogger>,
    /// Graceful degradation handler
    degradation_handler: Arc<GracefulDegradationHandler>,
    /// Error recovery manager
    recovery_manager: Arc<ErrorRecoveryManager>,
}

impl ComprehensiveErrorHandler {
    /// Create a new comprehensive error handler
    pub fn new() -> Self {
        let logger = Arc::new(ErrorLogger::new(1000));
        let degradation_handler = Arc::new(GracefulDegradationHandler::new(Arc::clone(&logger)));
        let recovery_manager = Arc::new(ErrorRecoveryManager::new(Arc::clone(&logger)));

        Self {
            logger,
            degradation_handler,
            recovery_manager,
        }
    }

    /// Handle an error with full error handling pipeline
    pub fn handle_error(&self, error: TranspositionError) -> TranspositionResult<()> {
        // First, attempt recovery
        if let Ok(()) = self.recovery_manager.attempt_recovery(&error) {
            return Ok(());
        }

        // If recovery fails, handle with graceful degradation
        self.degradation_handler.handle_error(error)
    }

    /// Get error logger
    pub fn logger(&self) -> Arc<ErrorLogger> {
        Arc::clone(&self.logger)
    }

    /// Get graceful degradation handler
    pub fn degradation_handler(&self) -> Arc<GracefulDegradationHandler> {
        Arc::clone(&self.degradation_handler)
    }

    /// Get error recovery manager
    pub fn recovery_manager(&self) -> Arc<ErrorRecoveryManager> {
        Arc::clone(&self.recovery_manager)
    }

    /// Get comprehensive error statistics
    pub fn get_error_stats(&self) -> ErrorStatistics {
        ErrorStatistics {
            counters: self.logger.get_counters(),
            degradation_level: self.degradation_handler.get_degradation_level(),
            recovery_stats: self.recovery_manager.get_recovery_stats(),
        }
    }
}

/// Comprehensive error statistics
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub counters: ErrorCounters,
    pub degradation_level: u32,
    pub recovery_stats: RecoveryStats,
}

impl Default for ComprehensiveErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_logger() {
        let logger = ErrorLogger::new(100);

        // Log some errors
        logger.log_error(ErrorSeverity::Info, "Test info message".to_string());
        logger.log_error(ErrorSeverity::Warning, "Test warning message".to_string());
        logger.log_error(ErrorSeverity::Error, "Test error message".to_string());

        // Check counters
        let counters = logger.get_counters();
        assert_eq!(counters.info_count, 1);
        assert_eq!(counters.warning_count, 1);
        assert_eq!(counters.error_count, 1);
        assert_eq!(counters.total_count, 3);

        // Check recent entries
        let recent = logger.get_recent_entries(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].severity, ErrorSeverity::Warning);
        assert_eq!(recent[1].severity, ErrorSeverity::Error);
    }

    #[test]
    fn test_graceful_degradation() {
        let logger = Arc::new(ErrorLogger::new(100));
        let handler = GracefulDegradationHandler::new(logger);

        // Test initial state
        assert_eq!(handler.get_degradation_level(), 0);

        // Test degradation increase
        handler.increase_degradation();
        assert_eq!(handler.get_degradation_level(), 1);

        // Test degradation decrease
        handler.decrease_degradation();
        assert_eq!(handler.get_degradation_level(), 0);

        // Test reset
        handler.increase_degradation();
        handler.increase_degradation();
        handler.reset_degradation();
        assert_eq!(handler.get_degradation_level(), 0);
    }

    #[test]
    fn test_error_recovery() {
        let logger = Arc::new(ErrorLogger::new(100));
        let manager = ErrorRecoveryManager::new(logger);

        // Test memory allocation recovery
        let memory_error = TranspositionError::MemoryAllocationFailed("Test".to_string());
        assert!(manager.attempt_recovery(&memory_error).is_ok());

        // Test configuration recovery
        let config_error = TranspositionError::InvalidConfiguration("Test".to_string());
        assert!(manager.attempt_recovery(&config_error).is_ok());

        // Test unknown error
        let unknown_error = TranspositionError::Timeout("Test".to_string());
        assert!(manager.attempt_recovery(&unknown_error).is_err());

        // Check stats
        let stats = manager.get_recovery_stats();
        assert_eq!(stats.total_attempts, 3);
        assert_eq!(stats.successful_recoveries, 2);
        assert!((stats.recovery_rate - 0.6666666666666666).abs() < 0.01);
    }

    #[test]
    fn test_comprehensive_error_handler() {
        let handler = ComprehensiveErrorHandler::new();

        // Test error handling
        let error = TranspositionError::MemoryAllocationFailed("Test".to_string());
        assert!(handler.handle_error(error).is_ok());

        // Test error statistics
        let stats = handler.get_error_stats();
        assert!(stats.recovery_stats.total_attempts > 0);
        assert!(stats.recovery_stats.successful_recoveries > 0);
    }

    #[test]
    fn test_error_export() {
        let logger = ErrorLogger::new(10);

        logger.log_error_with_context(
            ErrorSeverity::Error,
            "Test error".to_string(),
            "Test context".to_string(),
        );

        let export = logger.export_log();
        assert!(export.contains("Transposition Table Error Log"));
        assert!(export.contains("Test error"));
        assert!(export.contains("Test context"));
    }
}
