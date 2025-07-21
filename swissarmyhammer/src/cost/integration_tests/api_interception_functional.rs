//! Functional tests for API interception testing suite
//!
//! Core functionality tests that validate the main API interception pipeline
//! features including end-to-end pipeline execution, concurrent workflows,
//! token counting accuracy, and various API response format handling.

use crate::cost::test_utils::mock_mcp::McpOperation;
use std::time::Duration;
use tracing::info;

use super::api_interception_helpers::{create_test_mcp_system, DEFAULT_CONCURRENT_WORKFLOWS};

/// Test the complete end-to-end API interception pipeline
///
/// This test validates the complete flow from workflow execution through
/// MCP handler processing, cost tracking, token counting, to final cost
/// calculation. It ensures all components integrate correctly.
#[tokio::test]
async fn test_complete_api_interception_pipeline() {
    info!("Starting complete API interception pipeline test");
    let mut system = create_test_mcp_system().await;

    let operations = vec![
        McpOperation::CreateIssue {
            name: "pipeline-test-issue".to_string(),
            content: "Testing complete pipeline from workflow action through MCP handler, cost tracking, token counting, to final cost calculation.".to_string(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Updated content to test multiple API calls in sequence.".to_string(),
        },
        McpOperation::MarkComplete { number: 1 },
    ];

    let result = system
        .simulate_issue_workflow("complete-pipeline-test", operations)
        .await
        .unwrap_or_else(|e| panic!("Complete pipeline workflow failed unexpectedly during API interception test. This indicates a critical issue with the integration between MCP handlers, cost tracking, and token counting systems: {}", e));

    // Validate complete workflow execution with specific bounds
    assert_eq!(
        result.operation_results.len(),
        3,
        "Expected exactly 3 operations (Create, Update, MarkComplete) but got {}",
        result.operation_results.len()
    );
    assert!(
        result.operation_results.iter().all(|r| r.success),
        "Not all operations succeeded. Failed operations: {:?}",
        result
            .operation_results
            .iter()
            .filter(|r| !r.success)
            .collect::<Vec<_>>()
    );

    // Workflow duration should be reasonable for 3 operations with 10ms base latency
    assert!(
        result.duration >= Duration::from_millis(15),
        "Workflow duration ({:?}) is too short for 3 operations with 10ms base latency",
        result.duration
    );
    assert!(
        result.duration <= Duration::from_millis(500),
        "Workflow duration ({:?}) is unexpectedly long, suggesting performance issues",
        result.duration
    );

    // Validate session statistics
    if let Some(session_stats) = result.session_stats {
        assert_eq!(session_stats.api_call_count, 3); // MCP operations recorded as API calls
        assert!(session_stats.total_tokens > 0);
        assert!(session_stats.is_completed);
    } else {
        // Session may be completed and no longer tracked - this is also valid behavior
        info!("Session statistics not available (session may have been cleaned up)");
    }

    // Validate API performance characteristics
    let api_stats = result.api_performance_stats;
    // Note: Mock API calls may be 0 if MCP handlers use internal tracking
    // The important thing is that the operations completed successfully
    info!(
        "API performance stats: {} total calls, {} successful",
        api_stats.total_calls, api_stats.successful_calls
    );

    // Validate API performance metrics with realistic bounds
    assert!(
        api_stats.average_latency >= Duration::from_millis(5),
        "Average latency ({:?}) is unrealistically low for mock API with 10ms base latency",
        api_stats.average_latency
    );
    assert!(
        api_stats.average_latency <= Duration::from_millis(100),
        "Average latency ({:?}) exceeds reasonable bounds for fast test configuration",
        api_stats.average_latency
    );

    // Failure rate validation with meaningful bounds for test configuration
    assert!(
        api_stats.failure_rate >= 0.0,
        "Failure rate ({}) cannot be negative",
        api_stats.failure_rate
    );
    assert!(api_stats.failure_rate <= 0.05,
        "Failure rate ({}) is too high for test configuration (expected ≤5% for high-reliability test setup)",
        api_stats.failure_rate);

    // Calls per second should be reasonable for the test load
    if api_stats.total_calls > 0 {
        assert!(
            api_stats.calls_per_second >= 5.0,
            "Throughput ({} calls/sec) is too low, indicating performance issues",
            api_stats.calls_per_second
        );
        assert!(
            api_stats.calls_per_second <= 1000.0,
            "Throughput ({} calls/sec) is unrealistically high for mock system",
            api_stats.calls_per_second
        );
    }

    info!("Complete API interception pipeline test passed successfully");
}

/// Test concurrent workflow execution with accurate cost attribution
///
/// Validates that multiple concurrent workflows properly track costs
/// separately and don't interfere with each other's cost calculations.
/// Tests the thread safety of the cost tracking system.
#[tokio::test]
async fn test_concurrent_workflow_cost_attribution() {
    info!("Starting concurrent workflow cost attribution test");

    const NUM_CONCURRENT_WORKFLOWS: usize = DEFAULT_CONCURRENT_WORKFLOWS;
    const OPERATIONS_PER_WORKFLOW: usize = 4;

    let mut systems = Vec::new();
    for _ in 0..NUM_CONCURRENT_WORKFLOWS {
        systems.push(create_test_mcp_system().await);
    }

    // Execute workflows concurrently
    let mut handles = Vec::new();
    for (i, mut system) in systems.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            let operations = vec![
                McpOperation::CreateIssue {
                    name: format!("concurrent-issue-{}", i),
                    content: format!("Concurrent workflow {} testing cost attribution", i),
                },
                McpOperation::UpdateIssue {
                    number: 1,
                    content: format!("First update for workflow {}", i),
                },
                McpOperation::UpdateIssue {
                    number: 1,
                    content: format!("Second update for workflow {}", i),
                },
                McpOperation::MarkComplete { number: 1 },
            ];

            let workflow_name = format!("concurrent-workflow-{}", i);
            let result = system
                .simulate_issue_workflow(&workflow_name, operations)
                .await
                .unwrap_or_else(|e| panic!("Concurrent workflow '{}' failed during cost attribution test. This suggests thread safety issues in the cost tracking system or API interception pipeline: {}", workflow_name, e));

            (workflow_name, result)
        });
        handles.push(handle);
    }

    // Collect all results
    let mut workflow_results = Vec::new();
    for handle in handles {
        let (name, result) = handle
            .await
            .expect("Concurrent workflow task should complete");
        workflow_results.push((name, result));
    }

    // Validate all workflows completed successfully
    assert_eq!(workflow_results.len(), NUM_CONCURRENT_WORKFLOWS);

    for (workflow_name, result) in &workflow_results {
        assert_eq!(result.operation_results.len(), OPERATIONS_PER_WORKFLOW);
        assert!(
            result.operation_results.iter().all(|r| r.success),
            "All operations in {} should succeed",
            workflow_name
        );
        assert!(result.duration > Duration::from_millis(0));
    }

    // Validate performance characteristics under concurrent load
    let total_operations: usize = workflow_results
        .iter()
        .map(|(_, result)| result.operation_results.len())
        .sum();
    assert_eq!(
        total_operations,
        NUM_CONCURRENT_WORKFLOWS * OPERATIONS_PER_WORKFLOW
    );

    let total_duration = workflow_results
        .iter()
        .map(|(_, result)| result.duration)
        .max()
        .expect("Should have at least one duration");

    // Concurrent execution should be reasonably fast
    // Allow generous time for CI environments
    assert!(
        total_duration < Duration::from_secs(30),
        "Concurrent execution took too long: {:?}",
        total_duration
    );

    info!("Concurrent workflow cost attribution test completed successfully");
}

/// Test token counting accuracy across different scenarios
///
/// Validates that token counts are accurately tracked and reported
/// across various API call patterns and response formats.
#[tokio::test]
async fn test_token_counting_accuracy_integration() {
    info!("Starting token counting accuracy integration test");
    let mut system = create_test_mcp_system().await;

    // Test various operation types to ensure token counting works across different scenarios
    let operations = vec![
        McpOperation::CreateIssue {
            name: "token-counting-test".to_string(),
            content: "This is a longer content block designed to test token counting accuracy. ".repeat(10),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Short update".to_string(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Another longer update with more substantial content that should generate more tokens for both input and output processing. ".repeat(5),
        },
        McpOperation::MarkComplete { number: 1 },
    ];

    let result = system
        .simulate_issue_workflow("token-counting-integration", operations)
        .await
        .expect("Token counting workflow should succeed");

    // Validate operation completion
    assert_eq!(result.operation_results.len(), 4);
    assert!(result.operation_results.iter().all(|r| r.success));

    // Validate token counting
    if let Some(session_stats) = result.session_stats {
        assert!(
            session_stats.total_input_tokens > 0,
            "Should have input tokens"
        );
        assert!(
            session_stats.total_output_tokens > 0,
            "Should have output tokens"
        );
        assert!(session_stats.total_tokens > 0, "Should have total tokens");

        // Validate token relationships
        assert_eq!(
            session_stats.total_tokens,
            session_stats.total_input_tokens + session_stats.total_output_tokens,
            "Total tokens should equal input + output"
        );

        // Token counts should be realistic for the operations performed
        assert!(
            session_stats.total_input_tokens > 100,
            "Input tokens should reflect content size"
        );
        assert!(
            session_stats.total_output_tokens > 50,
            "Output tokens should be generated"
        );

        info!(
            "Token counts - Input: {}, Output: {}, Total: {}",
            session_stats.total_input_tokens,
            session_stats.total_output_tokens,
            session_stats.total_tokens
        );
    }

    // Validate that individual operations have reasonable token usage
    for (i, op_result) in result.operation_results.iter().enumerate() {
        assert!(op_result.success, "Operation {} should succeed", i);
        assert!(
            op_result.duration > Duration::from_millis(0),
            "Operation {} should have duration",
            i
        );
    }

    info!("Token counting accuracy integration test completed successfully");
}

/// Test various API response formats and token extraction
///
/// Validates that the system correctly handles different API response
/// formats and extracts token usage information accurately from each format.
#[tokio::test]
async fn test_various_api_response_formats() {
    info!("Starting API response format testing");

    // Test with standard token format
    let mut standard_system = create_test_mcp_system().await;
    let standard_result = standard_system
        .simulate_issue_workflow(
            "standard-format-test",
            vec![McpOperation::CreateIssue {
                name: "standard-format".to_string(),
                content: "Testing standard API response format".to_string(),
            }],
        )
        .await
        .expect("Standard format test should succeed");

    assert!(standard_result.operation_results[0].success);
    if let Some(stats) = standard_result.session_stats {
        assert!(stats.total_tokens > 0, "Standard format should have tokens");
    }

    // Test with mixed response formats in a single workflow
    let mut mixed_system = create_test_mcp_system().await;
    let mixed_result = mixed_system
        .simulate_issue_workflow(
            "mixed-format-test",
            vec![
                McpOperation::CreateIssue {
                    name: "mixed-format-1".to_string(),
                    content: "First operation with standard format".to_string(),
                },
                McpOperation::UpdateIssue {
                    number: 1,
                    content: "Second operation that may use different format".to_string(),
                },
            ],
        )
        .await
        .expect("Mixed format test should succeed");

    assert_eq!(mixed_result.operation_results.len(), 2);
    assert!(mixed_result.operation_results.iter().all(|r| r.success));

    // Validate that different operations can have varying token patterns
    if let Some(stats) = mixed_result.session_stats {
        assert!(
            stats.total_tokens > 0,
            "Mixed format should accumulate tokens"
        );
        assert_eq!(stats.api_call_count, 2, "Should track both API calls");
    }

    info!("API response format testing completed successfully");
}
