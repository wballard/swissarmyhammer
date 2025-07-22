//! Performance optimization module for cost tracking system
//!
//! This module provides comprehensive performance optimizations for the cost tracking system
//! to meet the requirement of less than 50ms overhead per API call and minimal memory usage.

pub mod benchmarks;
pub mod memory;
pub mod monitoring;
pub mod optimization;
pub mod token_optimization;

#[cfg(feature = "database")]
pub mod async_storage;

pub use benchmarks::{BenchmarkResult, BenchmarkSuite, PerformanceBenchmark};
pub use memory::{MemoryPool, ResourceManager};
pub use monitoring::{PerformanceMetrics, PerformanceMonitor};
pub use optimization::{OptimizationConfig, PerformanceOptimizer};
pub use token_optimization::{OptimizedTokenCounter, TokenCache};

#[cfg(feature = "database")]
pub use async_storage::{AsyncStorageManager, StorageConfig};

/// Performance optimization configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Memory pool configuration
    pub memory_pool_size: usize,
    /// Async batch size for operations
    pub async_batch_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    /// Connection pool size for database
    pub connection_pool_size: usize,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Cache size for token operations
    pub cache_size: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Target API call overhead in milliseconds
    pub target_api_overhead_ms: u64,
    /// Maximum memory overhead percentage
    pub max_memory_overhead_pct: f32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            memory_pool_size: 10000,
            async_batch_size: 100,
            flush_interval_ms: 1000,
            connection_pool_size: 10,
            enable_simd: cfg!(target_arch = "x86_64"),
            cache_size: 1000000,
            enable_monitoring: true,
            target_api_overhead_ms: 50,
            max_memory_overhead_pct: 5.0,
        }
    }
}

/// Performance optimization error types
#[derive(thiserror::Error, Debug)]
pub enum PerformanceError {
    /// Performance target exceeded
    #[error("Performance target exceeded: {metric} = {actual}ms, target = {target}ms")]
    TargetExceeded {
        /// The performance metric that exceeded its target
        metric: String,
        /// The actual measured value in milliseconds
        actual: u64,
        /// The target threshold in milliseconds
        target: u64,
    },
    /// Memory limit exceeded
    #[error("Memory limit exceeded: {actual}% > {limit}%")]
    MemoryLimitExceeded {
        /// The actual memory usage percentage
        actual: f32,
        /// The maximum allowed memory usage percentage
        limit: f32,
    },
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError {
        /// The configuration error message
        message: String,
    },
    /// Resource exhausted
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted {
        /// The name of the exhausted resource
        resource: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert_eq!(config.memory_pool_size, 10000);
        assert_eq!(config.async_batch_size, 100);
        assert_eq!(config.flush_interval_ms, 1000);
        assert_eq!(config.connection_pool_size, 10);
        assert_eq!(config.cache_size, 1000000);
        assert!(config.enable_monitoring);
        assert_eq!(config.target_api_overhead_ms, 50);
        assert_eq!(config.max_memory_overhead_pct, 5.0);

        // SIMD should be enabled on x86_64
        if cfg!(target_arch = "x86_64") {
            assert!(config.enable_simd);
        }
    }

    #[test]
    fn test_performance_error_display() {
        let error = PerformanceError::TargetExceeded {
            metric: "api_call_overhead".to_string(),
            actual: 75,
            target: 50,
        };
        assert!(error.to_string().contains("75ms"));
        assert!(error.to_string().contains("50ms"));

        let error = PerformanceError::MemoryLimitExceeded {
            actual: 7.5,
            limit: 5.0,
        };
        assert!(error.to_string().contains("7.5%"));
        assert!(error.to_string().contains("5%"));
    }
}
