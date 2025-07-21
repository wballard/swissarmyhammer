//! Test helpers for API interception testing suite
//!
//! This module provides test helper functions and utilities for setting up
//! mock systems and configurations for API interception testing.

use crate::cost::test_utils::mock_mcp::{MockClaudeApiConfig, MockMcpSystem};

// Test configuration constants

/// Default base latency in milliseconds for test API responses
pub const DEFAULT_TEST_LATENCY_MS: u64 = 10;

/// Default number of concurrent workflows for testing concurrent operations
pub const DEFAULT_CONCURRENT_WORKFLOWS: usize = 5;

/// Default number of operations for performance testing scenarios
pub const DEFAULT_PERFORMANCE_OPERATIONS: usize = 20;

/// Test helper to create a comprehensive MCP system for testing
///
/// Creates a mock MCP system with optimized settings for fast, reliable testing.
/// Uses low latency and high success rates to ensure tests complete quickly
/// while still exercising the full API interception pipeline.
pub async fn create_test_mcp_system() -> MockMcpSystem {
    MockMcpSystem::with_config(MockClaudeApiConfig {
        base_latency_ms: DEFAULT_TEST_LATENCY_MS, // Faster for testing
        latency_variance_ms: 5,
        success_rate: 1.0,    // 100% success rate for token counting tests
        timeout_rate: 0.0,    // No timeouts for token counting tests
        rate_limit_rate: 0.0, // No rate limiting for token counting tests
        include_token_usage: true,
        use_alternative_token_format: false,
    })
    .await
}

/// Test helper to create alternative API configuration for error testing
///
/// Creates a mock MCP system designed to simulate challenging conditions
/// with higher error rates, timeouts, and rate limiting to test resilience
/// and recovery mechanisms in the API interception pipeline.
pub async fn create_error_prone_mcp_system() -> MockMcpSystem {
    MockMcpSystem::with_config(MockClaudeApiConfig {
        base_latency_ms: 50,
        latency_variance_ms: 25,
        success_rate: 0.7,          // Lower success rate
        timeout_rate: 0.15,         // Higher timeout rate
        rate_limit_rate: 0.10,      // Higher rate limiting
        include_token_usage: false, // No token usage for fallback testing
        use_alternative_token_format: false,
    })
    .await
}
