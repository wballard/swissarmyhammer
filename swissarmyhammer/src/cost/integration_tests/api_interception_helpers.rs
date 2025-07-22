//! Test helpers for API interception testing suite
//!
//! This module provides test helper functions and utilities for setting up
//! mock systems and configurations for API interception testing.

use crate::cost::test_utils::mock_mcp::{MockClaudeApiConfig, MockMcpSystem};

// Test configuration constants

/// Default base latency in milliseconds for test API responses
///
/// This value represents the minimum artificial delay added to mock API calls
/// to simulate network latency while keeping tests fast. The actual latency
/// will vary based on the variance configuration in each test scenario.
///
/// # Value: 10ms
/// This is fast enough to keep tests running quickly while still providing
/// realistic timing for latency-sensitive test validations.
///
/// # Usage
/// Used by [`create_test_mcp_system`] as the base latency for standard testing.
/// Performance tests may override this value for specific timing requirements.
pub const DEFAULT_TEST_LATENCY_MS: u64 = 10;

/// Default number of concurrent workflows for testing concurrent operations
///
/// This constant defines the standard concurrency level used across integration
/// tests to validate that the API interception system can handle multiple
/// simultaneous workflows without conflicts or resource contention.
///
/// # Value: 5 concurrent workflows
/// This level provides meaningful concurrency testing without overwhelming
/// the test environment or causing resource exhaustion on CI systems.
///
/// # Usage Examples
/// - Testing concurrent session management
/// - Validating thread safety of cost tracking
/// - Ensuring proper cleanup under concurrent load
/// - Performance baseline for concurrent operations
///
/// # Notes
/// Individual tests may use different concurrency levels based on their
/// specific requirements, but this constant provides a consistent baseline
/// across the test suite.
pub const DEFAULT_CONCURRENT_WORKFLOWS: usize = 5;

/// Default number of operations for performance testing scenarios
///
/// This constant defines the standard operation count used in performance
/// tests to measure API interception overhead, throughput, and scalability.
/// The value is chosen to provide meaningful performance data while keeping
/// test execution time reasonable.
///
/// # Value: 20 operations
/// This count generates sufficient load to measure performance characteristics
/// while maintaining fast test execution. Each operation typically includes
/// both API calls and cost tracking activities.
///
/// # Performance Metrics
/// With 20 operations, tests can measure:
/// - Average latency per operation
/// - Throughput (operations per second)
/// - Memory usage patterns
/// - Token counting accuracy at scale
/// - Cost calculation performance
///
/// # Usage in Tests
/// Used primarily in `api_interception_performance.rs` for:
/// - Baseline performance measurements
/// - Regression detection
/// - Scalability validation
/// - Memory leak detection
///
/// # Scaling Considerations
/// Performance tests may multiply this value for stress testing scenarios,
/// but this constant provides the standard baseline for consistent
/// performance measurements across test runs.
pub const DEFAULT_PERFORMANCE_OPERATIONS: usize = 20;

/// Creates a comprehensive mock MCP system optimized for testing
///
/// This function creates a fully configured mock MCP system with settings optimized
/// for reliable, fast-executing tests. The system includes cost tracking, token counting,
/// and API interception capabilities while minimizing artificial delays and failures
/// that could make tests flaky.
///
/// # Purpose
/// - Provides a consistent test environment for functional tests
/// - Ensures high success rates to test the "happy path" scenarios
/// - Minimizes latency to keep tests fast and deterministic
/// - Enables comprehensive validation of the API interception pipeline
///
/// # Configuration Details
/// - **Latency**: 10ms base with 5ms variance for fast test execution
/// - **Success Rate**: 100% to ensure consistent test results
/// - **Timeout Rate**: 0% to avoid flaky test failures
/// - **Rate Limiting**: Disabled for predictable test behavior
/// - **Token Usage**: Enabled with standard format for token counting validation
///
/// # Returns
/// A fully configured `MockMcpSystem` ready for testing API interception workflows.
///
/// # Usage Example
/// ```rust
/// use crate::cost::integration_tests::api_interception_helpers::create_test_mcp_system;
/// use crate::cost::test_utils::mock_mcp::McpOperation;
///
/// #[tokio::test]
/// async fn test_basic_workflow() {
///     let mut system = create_test_mcp_system().await;
///     
///     let operations = vec![
///         McpOperation::CreateIssue {
///             name: "test-issue".to_string(),
///             content: "Test content".to_string(),
///         },
///     ];
///     
///     let result = system
///         .simulate_issue_workflow("basic-test", operations)
///         .await;
///     
///     assert!(result.is_ok());
///     let workflow_result = result.unwrap();
///     assert!(workflow_result.operation_results.iter().all(|r| r.success));
/// }
/// ```
///
/// # See Also
/// - [`create_error_prone_mcp_system`] for testing error conditions
/// - [`MockMcpSystem::with_config`] for custom configuration
/// - [`MockClaudeApiConfig`] for configuration options
pub async fn create_test_mcp_system() -> MockMcpSystem {
    MockMcpSystem::with_config(MockClaudeApiConfig {
        base_latency_ms: DEFAULT_TEST_LATENCY_MS, // Faster for testing
        latency_variance_ms: 5,
        success_rate: 1.0,    // 100% success rate for token counting tests
        timeout_rate: 0.0,    // No timeouts for token counting tests
        rate_limit_rate: 0.0, // No rate limiting for token counting tests
        include_token_usage: true,
        use_alternative_token_format: false,
        token_config: crate::cost::test_utils::mock_mcp::TokenGenerationConfig::default(),
    })
    .await
}

/// Creates a mock MCP system configured for error resilience testing
///
/// This function creates a mock MCP system with intentionally challenging conditions
/// to test error handling, resilience, and recovery mechanisms. The system is designed
/// to simulate real-world API failures, network issues, and rate limiting scenarios
/// that the API interception pipeline must handle gracefully.
///
/// # Purpose
/// - Tests error handling and recovery mechanisms
/// - Validates graceful degradation under challenging conditions
/// - Simulates real-world API reliability issues
/// - Tests fallback behaviors when token usage data is unavailable
/// - Validates retry logic and timeout handling
///
/// # Configuration Details
/// - **Latency**: 50ms base with 25ms variance to simulate network delays
/// - **Success Rate**: 70% to test error handling paths
/// - **Timeout Rate**: 15% to test timeout recovery mechanisms
/// - **Rate Limiting**: 10% to test rate limit handling
/// - **Token Usage**: Disabled to test estimation fallback mechanisms
///
/// # Returns
/// A `MockMcpSystem` configured to simulate challenging API conditions.
///
/// # Usage Example
/// ```rust
/// use crate::cost::integration_tests::api_interception_helpers::create_error_prone_mcp_system;
/// use crate::cost::test_utils::mock_mcp::McpOperation;
///
/// #[tokio::test]
/// async fn test_error_resilience() {
///     let mut system = create_error_prone_mcp_system().await;
///     
///     let operations = vec![
///         McpOperation::CreateIssue {
///             name: "resilience-test".to_string(),
///             content: "Testing error handling".to_string(),
///         },
///         McpOperation::UpdateIssue {
///             number: 1,
///             content: "Update with potential failures".to_string(),
///         },
///     ];
///     
///     let result = system
///         .simulate_issue_workflow("error-resilience-test", operations)
///         .await;
///     
///     // Even with errors, the system should handle them gracefully
///     assert!(result.is_ok());
///     let workflow_result = result.unwrap();
///     
///     // Check that the system properly recorded both successes and failures
///     let stats = workflow_result.session_stats.unwrap();
///     assert!(stats.api_call_count > 0);
/// }
/// ```
///
/// # Testing Scenarios
/// This configuration is particularly useful for testing:
/// - **API timeout handling**: How the system responds to slow or unresponsive APIs
/// - **Rate limit recovery**: Proper handling of HTTP 429 responses
/// - **Partial failure scenarios**: Mixed success/failure outcomes in batch operations
/// - **Token estimation fallback**: Behavior when token usage data is unavailable
/// - **Session completion**: Ensuring sessions complete even with partial failures
///
/// # See Also
/// - [`create_test_mcp_system`] for standard testing with high reliability
/// - [`MockClaudeApiConfig`] for understanding configuration options
/// - Error handling tests in `api_interception_reliability.rs`
pub async fn create_error_prone_mcp_system() -> MockMcpSystem {
    MockMcpSystem::with_config(MockClaudeApiConfig {
        base_latency_ms: 50,
        latency_variance_ms: 25,
        success_rate: 0.7,          // Lower success rate
        timeout_rate: 0.15,         // Higher timeout rate
        rate_limit_rate: 0.10,      // Higher rate limiting
        include_token_usage: false, // No token usage for fallback testing
        use_alternative_token_format: false,
        token_config: crate::cost::test_utils::mock_mcp::TokenGenerationConfig::default(),
    })
    .await
}
