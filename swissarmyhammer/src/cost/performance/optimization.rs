//! Main performance optimization coordinator
//!
//! This module coordinates all performance optimization components and provides
//! the main interface for optimized cost tracking operations.

use crate::cost::performance::{
    memory::{ResourceLimits, ResourceManager},
    monitoring::{MonitoringConfig, PerformanceMetrics},
    token_optimization::{OptimizedTokenCounter, TokenOptimizationConfig},
    PerformanceConfig, PerformanceError,
};
use crate::cost::{
    ApiCall, ApiCallId, ApiCallStatus, CostError, CostSession, CostSessionId, CostTracker, IssueId,
    TokenUsage,
};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Configuration for performance optimization
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// General performance configuration
    pub performance: PerformanceConfig,
    /// Token optimization configuration  
    pub token_optimization: TokenOptimizationConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            performance: PerformanceConfig::default(),
            token_optimization: TokenOptimizationConfig::default(),
            monitoring: MonitoringConfig::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Main performance optimizer that coordinates all optimization components
pub struct PerformanceOptimizer {
    /// Configuration
    config: OptimizationConfig,
    /// Resource manager for memory optimization
    resource_manager: Arc<ResourceManager>,
    /// Optimized token counter
    token_counter: Arc<OptimizedTokenCounter>,
    /// Performance metrics and monitoring
    metrics: Arc<PerformanceMetrics>,
    /// Base cost tracker (wrapped for optimization)
    base_tracker: Arc<RwLock<CostTracker>>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: OptimizationConfig) -> Result<Self, PerformanceError> {
        let resource_manager = Arc::new(
            ResourceManager::new(config.resource_limits.clone()).map_err(|e| {
                PerformanceError::ConfigError {
                    message: format!("Failed to create resource manager: {}", e),
                }
            })?,
        );

        let token_counter = Arc::new(OptimizedTokenCounter::new(
            config.token_optimization.clone(),
        ));

        let metrics = Arc::new(PerformanceMetrics::new(config.monitoring.clone()));

        let base_tracker = Arc::new(RwLock::new(CostTracker::new()));

        Ok(Self {
            config,
            resource_manager,
            token_counter,
            metrics,
            base_tracker,
        })
    }

    /// Start a new optimized cost session
    pub fn start_session(&self, issue_id: IssueId) -> Result<CostSessionId, CostError> {
        let start_time = Instant::now();

        // Use base tracker for session management
        let session_id = {
            let mut tracker = self.base_tracker.write().unwrap();
            tracker.start_session(issue_id)?
        };

        // Record performance metrics
        let duration = start_time.elapsed();
        self.metrics.record_api_call(duration, true);

        // Check performance target
        if duration.as_millis() > self.config.performance.target_api_overhead_ms as u128 {
            tracing::warn!(
                duration_ms = duration.as_millis(),
                target_ms = self.config.performance.target_api_overhead_ms,
                "Session start exceeded performance target"
            );
        }

        Ok(session_id)
    }

    /// Add an optimized API call to a session
    pub fn add_api_call(
        &self,
        session_id: &CostSessionId,
        endpoint: &str,
        model: &str,
    ) -> Result<ApiCallId, CostError> {
        let start_time = Instant::now();

        // Create API call using optimized memory management
        let api_call = {
            // Use resource manager to optimize string allocation
            let interned_endpoint = self.resource_manager.intern_string(endpoint);
            let interned_model = self.resource_manager.intern_string(model);

            ApiCall::new(interned_endpoint.as_ref(), interned_model.as_ref())?
        };

        // Add to tracker
        let call_id = {
            let mut tracker = self.base_tracker.write().unwrap();
            tracker.add_api_call(session_id, api_call)?
        };

        // Record performance metrics
        let duration = start_time.elapsed();
        self.metrics.record_api_call(duration, true);

        Ok(call_id)
    }

    /// Complete an API call with optimized token counting
    pub fn complete_api_call_with_response(
        &self,
        session_id: &CostSessionId,
        call_id: &ApiCallId,
        response_body: &str,
        status: ApiCallStatus,
        error_message: Option<String>,
    ) -> Result<TokenUsage, CostError> {
        let start_time = Instant::now();

        // Use optimized token counter
        let token_usage = {
            let count_start = Instant::now();
            let usage = self.token_counter.count_from_response(response_body)?;
            let count_duration = count_start.elapsed();

            // Record token counting performance
            self.metrics
                .record_token_counting(count_duration, true, usage.total_tokens);

            usage
        };

        // Complete the API call
        {
            let mut tracker = self.base_tracker.write().unwrap();
            tracker.complete_api_call(
                session_id,
                call_id,
                token_usage.input_tokens,
                token_usage.output_tokens,
                status,
                error_message,
            )?;
        }

        // Record overall performance
        let total_duration = start_time.elapsed();
        self.metrics.record_api_call(total_duration, true);

        // Validate performance target
        if total_duration.as_millis() > self.config.performance.target_api_overhead_ms as u128 {
            return Err(CostError::SerializationError {
                message: format!(
                    "API call completion exceeded performance target: {}ms > {}ms",
                    total_duration.as_millis(),
                    self.config.performance.target_api_overhead_ms
                ),
            });
        }

        Ok(token_usage)
    }

    /// Complete a cost session
    pub fn complete_session(
        &self,
        session_id: &CostSessionId,
        status: crate::cost::CostSessionStatus,
    ) -> Result<(), CostError> {
        let start_time = Instant::now();

        // Complete using base tracker
        {
            let mut tracker = self.base_tracker.write().unwrap();
            tracker.complete_session(session_id, status)?;
        }

        // Record performance
        let duration = start_time.elapsed();
        self.metrics.record_api_call(duration, true);

        Ok(())
    }

    /// Get session with optimized access
    pub fn get_session(&self, session_id: &CostSessionId) -> Option<CostSession> {
        let start_time = Instant::now();

        let session = {
            let tracker = self.base_tracker.read().unwrap();
            tracker.get_session(session_id).cloned()
        };

        // Record performance
        let duration = start_time.elapsed();
        self.metrics.record_api_call(duration, session.is_some());

        session
    }

    /// Get all sessions with optimized iteration
    pub fn get_all_sessions(&self) -> std::collections::HashMap<CostSessionId, CostSession> {
        let tracker = self.base_tracker.read().unwrap();
        tracker.get_all_sessions().clone()
    }

    /// Run performance validation
    pub fn validate_performance(&self) -> Result<PerformanceValidationResult, PerformanceError> {
        // Check if performance targets are being met
        self.metrics.check_targets()?;

        // Get resource statistics
        let resource_stats = self.resource_manager.get_resource_stats();
        let token_stats = self.token_counter.get_stats();
        let performance_report = self.metrics.get_report();

        // Calculate memory usage percentage
        let estimated_memory_usage = resource_stats.buffer_pool_stats.total_created * 1024; // Rough estimate
        let memory_usage_pct = (estimated_memory_usage as f32
            / self.config.resource_limits.max_memory_bytes as f32)
            * 100.0;

        if memory_usage_pct > self.config.performance.max_memory_overhead_pct {
            return Err(PerformanceError::MemoryLimitExceeded {
                actual: memory_usage_pct,
                limit: self.config.performance.max_memory_overhead_pct,
            });
        }

        Ok(PerformanceValidationResult {
            targets_met: true,
            memory_usage_pct,
            cache_hit_rate_pct: token_stats.cache_stats.hit_rate(),
            resource_stats,
            performance_report,
        })
    }

    /// Clean up resources and optimize memory usage
    pub fn cleanup(&self) {
        // Cleanup token counter caches
        self.token_counter.cleanup();

        // Cleanup old sessions from base tracker
        {
            let mut tracker = self.base_tracker.write().unwrap();
            tracker.cleanup_old_sessions();
        }

        tracing::info!("Performance optimizer cleanup completed");
    }

    /// Get comprehensive performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            resource_stats: self.resource_manager.get_resource_stats(),
            token_stats: self.token_counter.get_stats(),
            monitoring_report: self.metrics.get_report(),
            session_count: {
                let tracker = self.base_tracker.read().unwrap();
                tracker.session_count()
            },
            active_session_count: {
                let tracker = self.base_tracker.read().unwrap();
                tracker.active_session_count()
            },
        }
    }
}

/// Result of performance validation
#[derive(Debug, Clone)]
pub struct PerformanceValidationResult {
    /// Whether all performance targets are met
    pub targets_met: bool,
    /// Current memory usage percentage
    pub memory_usage_pct: f32,
    /// Cache hit rate percentage
    pub cache_hit_rate_pct: f64,
    /// Resource usage statistics
    pub resource_stats: crate::cost::performance::memory::ResourceStats,
    /// Performance monitoring report
    pub performance_report: String,
}

/// Comprehensive performance statistics
#[derive(Debug)]
pub struct PerformanceStats {
    /// Resource management statistics
    pub resource_stats: crate::cost::performance::memory::ResourceStats,
    /// Token optimization statistics
    pub token_stats: crate::cost::performance::token_optimization::OptimizedTokenStats,
    /// Performance monitoring report
    pub monitoring_report: String,
    /// Total session count
    pub session_count: usize,
    /// Active session count
    pub active_session_count: usize,
}

/// Builder for creating optimized performance configuration
pub struct PerformanceConfigBuilder {
    config: OptimizationConfig,
}

impl PerformanceConfigBuilder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self {
            config: OptimizationConfig::default(),
        }
    }

    /// Set API call overhead target
    pub fn api_overhead_target_ms(mut self, target_ms: u64) -> Self {
        self.config.performance.target_api_overhead_ms = target_ms;
        self.config.monitoring.alert_thresholds.api_overhead_ms = target_ms;
        self
    }

    /// Set memory usage limit
    pub fn memory_limit_pct(mut self, limit_pct: f32) -> Self {
        self.config.performance.max_memory_overhead_pct = limit_pct;
        self.config.monitoring.alert_thresholds.memory_usage_pct = limit_pct;
        self
    }

    /// Enable/disable SIMD optimizations
    pub fn simd_enabled(mut self, enabled: bool) -> Self {
        self.config.performance.enable_simd = enabled;
        self.config.token_optimization.enable_simd = enabled;
        self
    }

    /// Set cache size for token operations
    pub fn token_cache_size(mut self, size: usize) -> Self {
        self.config.performance.cache_size = size;
        self.config.token_optimization.cache_size = size;
        self
    }

    /// Set memory pool size
    pub fn memory_pool_size(mut self, size: usize) -> Self {
        self.config.performance.memory_pool_size = size;
        self.config.token_optimization.buffer_pool_size = size;
        self
    }

    /// Build the configuration
    pub fn build(self) -> OptimizationConfig {
        self.config
    }
}

impl Default for PerformanceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_builder() {
        let config = PerformanceConfigBuilder::new()
            .api_overhead_target_ms(30)
            .memory_limit_pct(3.0)
            .simd_enabled(false)
            .token_cache_size(5000)
            .memory_pool_size(500)
            .build();

        assert_eq!(config.performance.target_api_overhead_ms, 30);
        assert_eq!(config.performance.max_memory_overhead_pct, 3.0);
        assert!(!config.performance.enable_simd);
        assert_eq!(config.performance.cache_size, 5000);
        assert_eq!(config.performance.memory_pool_size, 500);
    }

    #[test]
    fn test_optimization_config_default() {
        let config = OptimizationConfig::default();

        assert_eq!(config.performance.target_api_overhead_ms, 50);
        assert_eq!(config.performance.max_memory_overhead_pct, 5.0);
        assert_eq!(config.token_optimization.cache_size, 10000);
        assert_eq!(config.monitoring.max_measurements, 10000);
    }

    #[test]
    fn test_performance_optimizer_creation() {
        let config = OptimizationConfig::default();
        let optimizer = PerformanceOptimizer::new(config);

        assert!(optimizer.is_ok());

        let optimizer = optimizer.unwrap();
        let stats = optimizer.get_performance_stats();

        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.active_session_count, 0);
    }

    #[test]
    fn test_optimized_session_lifecycle() {
        let config = OptimizationConfig::default();
        let optimizer = PerformanceOptimizer::new(config).unwrap();

        // Start session
        let issue_id = IssueId::new("test-issue").unwrap();
        let session_id = optimizer.start_session(issue_id).unwrap();

        // Add API call
        let call_id = optimizer
            .add_api_call(
                &session_id,
                "https://api.anthropic.com/v1/messages",
                "claude-3-sonnet-20241022",
            )
            .unwrap();

        // Complete API call
        let response = r#"{"usage":{"input_tokens":100,"output_tokens":50}}"#;
        let token_usage = optimizer
            .complete_api_call_with_response(
                &session_id,
                &call_id,
                response,
                ApiCallStatus::Success,
                None,
            )
            .unwrap();

        assert_eq!(token_usage.input_tokens, 100);
        assert_eq!(token_usage.output_tokens, 50);

        // Complete session
        optimizer
            .complete_session(&session_id, crate::cost::CostSessionStatus::Completed)
            .unwrap();

        // Verify session exists
        let session = optimizer.get_session(&session_id);
        assert!(session.is_some());

        let session = session.unwrap();
        assert!(session.is_completed());
        assert_eq!(session.api_call_count(), 1);
        assert_eq!(session.total_tokens(), 150);
    }

    #[test]
    fn test_performance_validation() {
        let config = PerformanceConfigBuilder::new()
            .api_overhead_target_ms(100) // Generous target for testing
            .memory_limit_pct(50.0) // Generous limit for testing
            .build();

        let optimizer = PerformanceOptimizer::new(config).unwrap();

        // Add some activity
        let issue_id = IssueId::new("test-issue").unwrap();
        let session_id = optimizer.start_session(issue_id).unwrap();
        let call_id = optimizer
            .add_api_call(
                &session_id,
                "https://api.anthropic.com/v1/messages",
                "claude-3-sonnet",
            )
            .unwrap();

        let response = r#"{"usage":{"input_tokens":50,"output_tokens":25}}"#;
        optimizer
            .complete_api_call_with_response(
                &session_id,
                &call_id,
                response,
                ApiCallStatus::Success,
                None,
            )
            .unwrap();

        // Validate performance
        let validation_result = optimizer.validate_performance().unwrap();
        assert!(validation_result.targets_met);
        assert!(validation_result.memory_usage_pct >= 0.0);
        assert!(validation_result.cache_hit_rate_pct >= 0.0);
        assert!(!validation_result.performance_report.is_empty());
    }

    #[test]
    fn test_performance_cleanup() {
        let config = OptimizationConfig::default();
        let optimizer = PerformanceOptimizer::new(config).unwrap();

        // Add some data
        let issue_id = IssueId::new("cleanup-test").unwrap();
        let session_id = optimizer.start_session(issue_id).unwrap();

        // Cleanup should not panic
        optimizer.cleanup();

        // Session should still exist after cleanup
        let session = optimizer.get_session(&session_id);
        assert!(session.is_some());
    }

    #[test]
    fn test_performance_stats() {
        let config = OptimizationConfig::default();
        let optimizer = PerformanceOptimizer::new(config).unwrap();

        let stats = optimizer.get_performance_stats();

        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.active_session_count, 0);
        assert!(!stats.monitoring_report.is_empty());
        assert!(stats.resource_stats.buffer_pool_stats.total_created > 0);
    }
}
