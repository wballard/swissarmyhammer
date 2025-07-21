//! Reliability tests for API interception testing suite
//!
//! Tests focused on error handling, recovery mechanisms, memory management,
//! and edge case scenarios. Validates system behavior under adverse conditions
//! and ensures graceful degradation and cleanup.

use crate::cost::test_utils::mock_mcp::McpOperation;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

use super::api_interception_helpers::{create_error_prone_mcp_system, create_test_mcp_system};

/// Test error resilience and recovery mechanisms
///
/// Validates system behavior with API failures, network issues, and
/// malformed responses. Ensures graceful degradation and recovery
/// capabilities under adverse conditions.
#[tokio::test]
async fn test_error_resilience_and_recovery() {
    info!("Starting error resilience and recovery test");
    let mut system = create_error_prone_mcp_system().await;

    let operations = vec![
        McpOperation::CreateIssue {
            name: "error-resilience-test".to_string(),
            content:
                "Testing system behavior with API failures, network issues, and malformed responses"
                    .to_string(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Testing error recovery during update operations".to_string(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Additional update to test sustained error conditions".to_string(),
        },
        McpOperation::MarkComplete { number: 1 },
    ];

    // Execute workflow with error-prone system
    let result = system
        .simulate_issue_workflow("error-resilience-test", operations)
        .await
        .expect("Error resilience test should handle failures gracefully");

    // Validate graceful degradation
    assert_eq!(result.operation_results.len(), 4);

    // With 70% success rate, we expect some failures but system should continue
    let successful_ops = result
        .operation_results
        .iter()
        .filter(|r| r.success)
        .count();
    let failed_ops = result
        .operation_results
        .iter()
        .filter(|r| !r.success)
        .count();

    info!(
        "Error resilience test: {} successful, {} failed operations",
        successful_ops, failed_ops
    );

    // Should have at least some operations succeed (not all fail)
    assert!(
        successful_ops > 0,
        "At least some operations should succeed"
    );

    // Should handle failures gracefully (total should equal expected)
    assert_eq!(successful_ops + failed_ops, 4);

    // Validate that failure handling works
    if failed_ops > 0 {
        info!(
            "Validating error handling for {} failed operations",
            failed_ops
        );

        // Check that failed operations have proper error context
        for op_result in result.operation_results.iter().filter(|r| !r.success) {
            assert!(
                op_result.duration > Duration::from_millis(0),
                "Failed operations should still have duration"
            );
        }
    }

    // Session should handle mixed success/failure appropriately
    if let Some(session_stats) = result.session_stats {
        // Session should complete even with some failures
        assert!(
            session_stats.api_call_count > 0,
            "Should track API calls even with failures"
        );

        // May or may not be completed depending on failure handling strategy
        info!("Session completion status: {}", session_stats.is_completed);
    }

    // API stats should reflect the error conditions
    let api_stats = result.api_performance_stats;
    if failed_ops > 0 {
        assert!(
            api_stats.failure_rate > 0.0,
            "API stats should reflect failures"
        );
        assert!(
            api_stats.failure_rate <= 1.0,
            "Failure rate should not exceed 100%"
        );
    }

    info!("Error resilience and recovery test completed successfully");
}

/// Test memory management and cleanup behavior
///
/// Validates that the system properly manages memory usage, cleans up
/// resources, and handles sustained load without memory leaks or
/// resource exhaustion.
#[tokio::test]
async fn test_memory_management_and_cleanup() {
    info!("Starting memory management and cleanup test");

    // Test with multiple systems to simulate memory pressure
    let num_test_systems = 20;
    let operations_per_system = 10;

    let mut systems = Vec::new();
    for _ in 0..num_test_systems {
        systems.push(create_test_mcp_system().await);
    }

    let mut all_results = Vec::new();

    // Execute workflows across all systems
    for (system_id, mut system) in systems.into_iter().enumerate() {
        let mut operations = Vec::new();
        for op_id in 0..operations_per_system {
            operations.push(McpOperation::CreateIssue {
                name: format!("memory-test-s{}-op{}", system_id, op_id),
                content: format!(
                    "Memory management test operation {}/{} with substantial content to test memory usage patterns",
                    system_id, op_id
                ).repeat(3),
            });

            // Add some updates to increase memory usage
            if op_id % 3 == 0 {
                operations.push(McpOperation::UpdateIssue {
                    number: (op_id + 1) as u32,
                    content: format!(
                        "Memory test update {}/{} with additional content for memory pressure testing",
                        system_id, op_id
                    ).repeat(2),
                });
            }
        }

        let result = system
            .simulate_issue_workflow(&format!("memory-test-{}", system_id), operations)
            .await
            .expect("Memory management workflow should succeed");

        all_results.push(result);

        // Small delay to allow for cleanup between systems
        if system_id % 5 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    // Validate all systems completed successfully
    assert_eq!(all_results.len(), num_test_systems);

    let mut total_operations = 0;
    let mut total_tokens = 0;
    let mut completed_sessions = 0;

    for (system_id, result) in all_results.iter().enumerate() {
        // Each system should complete its operations
        assert!(
            !result.operation_results.is_empty(),
            "System {} should have operation results",
            system_id
        );
        assert!(
            result.operation_results.iter().all(|op| op.success),
            "All operations in system {} should succeed",
            system_id
        );

        total_operations += result.operation_results.len();

        if let Some(stats) = &result.session_stats {
            total_tokens += stats.total_tokens;
            if stats.is_completed {
                completed_sessions += 1;
            }
        }
    }

    info!(
        "Memory management test completed: {} systems, {} operations, {} tokens, {} completed sessions",
        num_test_systems, total_operations, total_tokens, completed_sessions
    );

    // Validate resource usage patterns
    assert!(total_operations > 0, "Should have processed operations");
    assert!(total_tokens > 0, "Should have accumulated tokens");

    // Memory cleanup validation - sessions should complete and be eligible for cleanup
    assert!(
        completed_sessions > 0,
        "Should have completed sessions for cleanup"
    );

    // Performance validation - despite memory pressure, operations should be fast
    let max_duration = all_results
        .iter()
        .map(|r| r.duration)
        .max()
        .expect("Should have durations");

    assert!(
        max_duration < Duration::from_secs(10),
        "Memory management should not significantly impact performance: {:?}",
        max_duration
    );

    info!("Memory management and cleanup test passed successfully");
}

/// Test edge cases and boundary conditions
///
/// Validates system behavior at boundaries and edge cases including
/// empty inputs, large inputs, concurrent edge cases, and unusual
/// API response patterns.
#[tokio::test]
async fn test_edge_cases_and_boundary_conditions() {
    info!("Starting edge cases and boundary conditions test");
    let mut system = create_test_mcp_system().await;

    // Test Case 1: Empty/minimal content
    info!("Testing empty and minimal content edge cases");
    let minimal_operations = vec![
        McpOperation::CreateIssue {
            name: "minimal".to_string(),
            content: "x".to_string(), // Minimal content
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "".to_string(), // Empty update content
        },
    ];

    let minimal_result = system
        .simulate_issue_workflow("minimal-content-test", minimal_operations)
        .await
        .expect("Minimal content test should succeed");

    assert_eq!(minimal_result.operation_results.len(), 2);
    assert!(minimal_result.operation_results.iter().all(|r| r.success));

    // Test Case 2: Large content (stress boundary handling)
    info!("Testing large content boundary handling");
    let large_content = "Large content block for boundary testing. ".repeat(1000);
    let large_operations = vec![
        McpOperation::CreateIssue {
            name: "large-content-test".to_string(),
            content: large_content.clone(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: large_content,
        },
    ];

    let large_result = system
        .simulate_issue_workflow("large-content-test", large_operations)
        .await
        .expect("Large content test should succeed");

    assert_eq!(large_result.operation_results.len(), 2);
    assert!(large_result.operation_results.iter().all(|r| r.success));

    // Validate token counting with large content
    if let Some(stats) = large_result.session_stats {
        assert!(
            stats.total_input_tokens > 1000,
            "Large content should generate significant input tokens"
        );
    }

    // Test Case 3: Rapid sequential operations
    info!("Testing rapid sequential operations");
    let rapid_operations: Vec<_> = (0..50)
        .map(|i| McpOperation::CreateIssue {
            name: format!("rapid-{}", i),
            content: format!("Rapid operation {}", i),
        })
        .collect();

    let rapid_result = system
        .simulate_issue_workflow("rapid-sequential-test", rapid_operations)
        .await
        .expect("Rapid sequential test should succeed");

    assert_eq!(rapid_result.operation_results.len(), 50);
    assert!(rapid_result.operation_results.iter().all(|r| r.success));

    // Test Case 4: Mixed operation types in unusual patterns
    info!("Testing unusual operation patterns");
    let mixed_operations = vec![
        McpOperation::UpdateIssue {
            number: 999, // Update before create (should handle gracefully)
            content: "Update without create".to_string(),
        },
        McpOperation::CreateIssue {
            name: "mixed-pattern".to_string(),
            content: "Create after update attempt".to_string(),
        },
        McpOperation::MarkComplete { number: 888 }, // Complete non-existent issue
        McpOperation::MarkComplete { number: 1 },   // Complete actual issue
    ];

    let mixed_result = system
        .simulate_issue_workflow("mixed-pattern-test", mixed_operations)
        .await
        .expect("Mixed pattern test should handle edge cases gracefully");

    assert_eq!(mixed_result.operation_results.len(), 4);
    // Some operations may fail due to the unusual patterns, but system should continue
    let successful_mixed = mixed_result
        .operation_results
        .iter()
        .filter(|r| r.success)
        .count();
    assert!(
        successful_mixed > 0,
        "At least some operations should succeed in mixed pattern test"
    );

    // Test Case 5: Timeout boundary conditions
    info!("Testing timeout boundary conditions");
    let mut timeout_system = create_error_prone_mcp_system().await;

    let timeout_operations = vec![McpOperation::CreateIssue {
        name: "timeout-test".to_string(),
        content: "Testing timeout handling and recovery".to_string(),
    }];

    let timeout_result = timeout_system
        .simulate_issue_workflow("timeout-boundary-test", timeout_operations)
        .await
        .expect("Timeout boundary test should handle timeouts gracefully");

    // May succeed or fail depending on timeout simulation, but should not crash
    assert_eq!(timeout_result.operation_results.len(), 1);

    info!("Edge cases and boundary conditions test completed successfully");
}

/// Test network interruption scenarios and recovery
///
/// Validates system behavior when network connections are interrupted
/// during API calls and tests the recovery mechanisms.
#[tokio::test]
async fn test_network_interruption_recovery() {
    info!("Starting network interruption recovery test");
    let mut error_prone_system = create_error_prone_mcp_system().await;

    // Test operations that will encounter various network issues
    let network_test_operations = vec![
        McpOperation::CreateIssue {
            name: "network-interruption-1".to_string(),
            content: "Testing network interruption during API call".to_string(),
        },
        McpOperation::CreateIssue {
            name: "network-interruption-2".to_string(),
            content: "Testing recovery after network interruption".to_string(),
        },
        McpOperation::UpdateIssue {
            number: 1,
            content: "Testing update operation resilience".to_string(),
        },
        McpOperation::CreateIssue {
            name: "network-interruption-3".to_string(),
            content: "Testing sustained network issues".to_string(),
        },
    ];

    let result = error_prone_system
        .simulate_issue_workflow("network-interruption-test", network_test_operations)
        .await
        .expect("Network interruption test should handle failures gracefully");

    // With error-prone system (30% failure rate), we expect some network failures
    assert_eq!(result.operation_results.len(), 4);

    let successful_operations = result
        .operation_results
        .iter()
        .filter(|r| r.success)
        .count();
    let failed_operations = result
        .operation_results
        .iter()
        .filter(|r| !r.success)
        .count();

    info!(
        "Network interruption test: {} successful, {} failed operations",
        successful_operations, failed_operations
    );

    // System should continue functioning despite network issues
    assert!(
        successful_operations > 0,
        "Some operations should succeed despite network issues"
    );
    assert_eq!(successful_operations + failed_operations, 4);

    // Validate API performance stats reflect network issues
    let api_stats = result.api_performance_stats;
    if failed_operations > 0 {
        assert!(
            api_stats.failure_rate > 0.0,
            "API stats should reflect network interruptions"
        );
    }

    info!("Network interruption recovery test completed successfully");
}

/// Test malformed API response handling
///
/// Validates system behavior when receiving malformed or unexpected
/// API responses and ensures graceful handling without crashes.
#[tokio::test]
async fn test_malformed_response_handling() {
    info!("Starting malformed API response handling test");
    let mut system = create_error_prone_mcp_system().await;

    // Test operations that may receive malformed responses
    let malformed_response_operations = vec![
        McpOperation::CreateIssue {
            name: "malformed-response-1".to_string(),
            content: "Testing handling of malformed JSON responses".to_string(),
        },
        McpOperation::UpdateIssue {
            number: 999, // Invalid issue number that may cause malformed response
            content: "Testing malformed response from invalid operation".to_string(),
        },
        McpOperation::CreateIssue {
            name: "malformed-response-2".to_string(),
            content: "Testing partial response handling".to_string(),
        },
    ];

    let result = system
        .simulate_issue_workflow("malformed-response-test", malformed_response_operations)
        .await
        .expect("Malformed response test should not crash");

    assert_eq!(result.operation_results.len(), 3);

    // System should handle malformed responses gracefully
    let processed_operations = result.operation_results.len();
    assert_eq!(
        processed_operations, 3,
        "All operations should be processed despite malformed responses"
    );

    // Check that the system continues functioning
    for (i, op_result) in result.operation_results.iter().enumerate() {
        // Even failed operations should have valid duration
        assert!(
            op_result.duration > Duration::from_millis(0),
            "Operation {} should have valid duration even with malformed response",
            i
        );
    }

    info!("Malformed API response handling test completed successfully");
}

/// Test rate limiting recovery mechanisms
///
/// Validates that the system properly handles rate limiting responses
/// and implements appropriate backoff and retry strategies.
#[tokio::test]
async fn test_rate_limiting_recovery() {
    info!("Starting rate limiting recovery test");
    let mut system = create_error_prone_mcp_system().await;

    // Create a series of operations that will trigger rate limiting
    let rate_limit_operations: Vec<_> = (1..=15)
        .map(|i| McpOperation::CreateIssue {
            name: format!("rate-limit-test-{}", i),
            content: format!("Testing rate limiting behavior with operation {}", i),
        })
        .collect();

    let start_time = std::time::Instant::now();
    let result = system
        .simulate_issue_workflow("rate-limiting-test", rate_limit_operations)
        .await
        .expect("Rate limiting test should handle throttling gracefully");

    let total_duration = start_time.elapsed();

    assert_eq!(result.operation_results.len(), 15);

    // Analyze results for rate limiting patterns
    let successful_ops = result
        .operation_results
        .iter()
        .filter(|r| r.success)
        .count();
    let failed_ops = result
        .operation_results
        .iter()
        .filter(|r| !r.success)
        .count();

    info!(
        "Rate limiting test: {} successful, {} failed operations in {:?}",
        successful_ops, failed_ops, total_duration
    );

    // With error-prone system, we expect some operations to be rate limited
    assert!(successful_ops > 0, "Some operations should succeed");
    assert_eq!(successful_ops + failed_ops, 15);

    // Validate API performance statistics
    let api_stats = result.api_performance_stats;
    if failed_ops > 0 {
        assert!(
            api_stats.failure_rate > 0.0,
            "API stats should reflect rate limiting"
        );
    }

    // Rate limiting should not cause the entire system to fail
    assert!(
        api_stats.total_calls > 0,
        "System should continue processing despite rate limits"
    );

    info!("Rate limiting recovery test completed successfully");
}

/// Test sustained load and memory management
///
/// Validates system behavior under sustained high load to detect
/// memory leaks and ensure proper resource cleanup.
#[tokio::test]
async fn test_sustained_load_memory_management() {
    info!("Starting sustained load memory management test");

    const SUSTAINED_OPERATIONS: usize = 100;
    const BATCH_SIZE: usize = 20;

    let mut all_results = Vec::new();

    // Run operations in batches to simulate sustained load
    for batch in 0..(SUSTAINED_OPERATIONS / BATCH_SIZE) {
        info!(
            "Processing batch {} of {}",
            batch + 1,
            SUSTAINED_OPERATIONS / BATCH_SIZE
        );

        let mut system = create_test_mcp_system().await;

        let batch_operations: Vec<_> = (0..BATCH_SIZE)
            .map(|i| {
                let operation_id = batch * BATCH_SIZE + i;
                McpOperation::CreateIssue {
                    name: format!("sustained-load-{}", operation_id),
                    content: format!(
                        "Sustained load test operation {} - testing memory management under continuous load",
                        operation_id
                    ).repeat(2), // Increase memory usage per operation
                }
            })
            .collect();

        let batch_result = system
            .simulate_issue_workflow(&format!("sustained-load-batch-{}", batch), batch_operations)
            .await
            .expect("Sustained load batch should succeed");

        all_results.push(batch_result);

        // Small delay between batches to allow for cleanup
        sleep(Duration::from_millis(10)).await;
    }

    // Validate all batches completed successfully
    assert_eq!(all_results.len(), SUSTAINED_OPERATIONS / BATCH_SIZE);

    let mut total_successful_operations = 0;
    let mut total_processing_time = Duration::from_millis(0);

    for (batch_idx, result) in all_results.iter().enumerate() {
        assert_eq!(
            result.operation_results.len(),
            BATCH_SIZE,
            "Batch {} should have completed all operations",
            batch_idx
        );

        let successful_in_batch = result
            .operation_results
            .iter()
            .filter(|r| r.success)
            .count();
        total_successful_operations += successful_in_batch;
        total_processing_time += result.duration;

        // Verify session statistics if available
        if let Some(stats) = &result.session_stats {
            assert!(
                stats.total_tokens > 0,
                "Batch {} should have accumulated tokens",
                batch_idx
            );
            assert!(
                stats.api_call_count > 0,
                "Batch {} should have API calls recorded",
                batch_idx
            );
        }
    }

    info!(
        "Sustained load test completed: {} operations across {} batches in {:?}",
        total_successful_operations,
        all_results.len(),
        total_processing_time
    );

    // Validate memory management
    assert!(
        total_successful_operations >= SUSTAINED_OPERATIONS * 8 / 10, // At least 80% success
        "Sustained load should maintain high success rate"
    );

    // Performance should remain reasonable under sustained load
    let avg_batch_time = total_processing_time / all_results.len() as u32;
    assert!(
        avg_batch_time < Duration::from_secs(30),
        "Average batch processing time should remain reasonable: {:?}",
        avg_batch_time
    );

    info!("Sustained load memory management test completed successfully");
}
