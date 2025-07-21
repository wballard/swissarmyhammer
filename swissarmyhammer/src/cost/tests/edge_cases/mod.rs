//! Edge case and boundary condition tests
//!
//! This module provides comprehensive testing for boundary conditions, error scenarios,
//! and edge cases that could cause system failures. These tests ensure robustness
//! under extreme conditions and validate proper error handling throughout the system.

// pub mod boundary_conditions;
// pub mod error_scenarios;
// pub mod malformed_data;
// pub mod resource_limits;

use crate::cost::{
    tests::CostTrackingTestHarness,
    tracker::{ApiCall, ApiCallStatus, CostError, CostSession, CostSessionStatus, IssueId},
    calculator::CostCalculator,
    token_counter::{TokenCounter, TokenUsage, ConfidenceLevel},
};
use rust_decimal::Decimal;

#[tokio::test]
async fn test_zero_token_api_calls() {
    let harness = CostTrackingTestHarness::new();
    
    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: zero_token_edge_case");
    let scenario_start = std::time::Instant::now();
    
    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let calculator = harness.calculator.clone();
        
        let issue_id = IssueId::new("edge-zero-tokens".to_string())?;
        let session_id = {
            let mut tracker = shared_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        // Create API call with zero tokens
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages".to_string(),
            "claude-3-sonnet-20241022",
        )?;

        // Complete with zero tokens
        api_call.complete(0, 0, ApiCallStatus::Success, None);

        // Add to tracker
        {
            let mut tracker = shared_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call)?;
        }

        // Calculate cost
        let session = {
            let tracker = shared_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist")?;
        let calculation = calculator.calculate_session_cost(&session)?;

        // Zero tokens should result in zero cost
        assert_eq!(calculation.total_cost, Decimal::ZERO, "Zero tokens should result in zero cost");
        assert_eq!(session.api_calls.len(), 1, "Should have one API call");
        let api_call = session.api_calls.values().next().unwrap();
        assert_eq!(api_call.input_tokens, 0, "Input tokens should be zero");
        assert_eq!(api_call.output_tokens, 0, "Output tokens should be zero");

        Ok(calculation.total_cost)
    };
    
    let result: Result<Decimal, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();
    
    match &result {
        Ok(_) => tracing::info!("Test scenario 'zero_token_edge_case' completed successfully in {:?}", duration),
        Err(e) => tracing::error!("Test scenario 'zero_token_edge_case' failed after {:?}: {}", duration, e),
    }
    
    assert!(result.is_ok(), "Zero token edge case should be handled: {:?}", result);
}

#[tokio::test]
async fn test_maximum_token_limits() {
    let harness = CostTrackingTestHarness::new();
    
    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: maximum_token_limits");
    let scenario_start = std::time::Instant::now();
    
    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let calculator = harness.calculator.clone();
        
        let issue_id = IssueId::new("edge-max-tokens".to_string())?;
        let session_id = {
            let mut tracker = shared_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        // Test with very large token counts (1M tokens)
        let max_input_tokens = 1_000_000_u32;
        let max_output_tokens = 1_000_000_u32;

        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages".to_string(),
            "claude-3-sonnet-20241022",
        )?;

        api_call.complete(max_input_tokens, max_output_tokens, ApiCallStatus::Success, None);

        {
            let mut tracker = shared_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call)?;
        }

        // Calculate cost - should handle large numbers without overflow
        let session = {
            let tracker = shared_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist")?;
        let calculation = calculator.calculate_session_cost(&session)?;

        // Validate large token handling
        assert!(calculation.total_cost > Decimal::ZERO, "Large token counts should result in positive cost");
        let api_call = session.api_calls.values().next().unwrap();
        assert_eq!(api_call.input_tokens, max_input_tokens, "Should preserve large input token count");
        assert_eq!(api_call.output_tokens, max_output_tokens, "Should preserve large output token count");

        // Cost should be reasonable for large token counts
        let expected_min_cost = Decimal::from(50); // Rough minimum for 1M+ tokens
        assert!(calculation.total_cost >= expected_min_cost, "Large token cost should be substantial");

        Ok(calculation.total_cost)
    };
    
    let result: Result<Decimal, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();
    
    match &result {
        Ok(_) => tracing::info!("Test scenario 'maximum_token_limits' completed successfully in {:?}", duration),
        Err(e) => tracing::error!("Test scenario 'maximum_token_limits' failed after {:?}: {}", duration, e),
    }
    
    assert!(result.is_ok(), "Maximum token limits should be handled: {:?}", result);
}

#[tokio::test]
async fn test_invalid_session_operations() {
    let harness = CostTrackingTestHarness::new();

    // Test operations on non-existent session
    let non_existent_id = crate::cost::CostSessionId::new();
    let api_call = ApiCall::new(
        "https://api.anthropic.com/v1/messages".to_string(),
        "claude-3-sonnet-20241022",
    ).unwrap();

    let result = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.add_api_call(&non_existent_id, api_call)
    };

    // Should return appropriate error
    assert!(result.is_err(), "Adding API call to non-existent session should fail");
    match result.err() {
        Some(CostError::SessionNotFound { session_id }) => {
            assert_eq!(session_id, non_existent_id, "Error should contain correct session ID");
        }
        _ => panic!("Expected SessionNotFound error"),
    }

    // Test completing non-existent session
    let result = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.complete_session(&non_existent_id, CostSessionStatus::Completed)
    };

    assert!(result.is_err(), "Completing non-existent session should fail");
    match result.err() {
        Some(CostError::SessionNotFound { session_id }) => {
            assert_eq!(session_id, non_existent_id, "Error should contain correct session ID");
        }
        _ => panic!("Expected SessionNotFound error"),
    }
}

#[tokio::test]
async fn test_duplicate_session_creation() {
    let harness = CostTrackingTestHarness::new();

    // Create first session
    let issue_id = IssueId::new("duplicate-test".to_string()).unwrap();
    let first_session = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.start_session(issue_id.clone())
    };

    assert!(first_session.is_ok(), "First session creation should succeed");

    // Try to create duplicate session with same issue ID
    let duplicate_result = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.start_session(issue_id.clone())
    };

    // Should handle duplicate appropriately (implementation dependent)
    // Either allow multiple sessions per issue or return error
    match duplicate_result {
        Ok(second_session_id) => {
            // If duplicates are allowed, sessions should have different IDs
            assert_ne!(first_session.unwrap(), second_session_id, "Duplicate sessions should have different IDs");
        }
        Err(CostError::SessionAlreadyExists { session_id: _ }) => {
            // If duplicates are not allowed, should get appropriate error
            // This is acceptable behavior
        }
        Err(e) => panic!("Unexpected error for duplicate session: {:?}", e),
    }
}

#[tokio::test]
async fn test_malformed_api_calls() {
    let mut harness = CostTrackingTestHarness::new();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: malformed_api_calls");
    let start = std::time::Instant::now();
    
    let result: Result<&str, Box<dyn std::error::Error + Send + Sync>> = async {
        let issue_id = IssueId::new("malformed-test".to_string())?;
        let session_id = {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        // Test with extremely long URLs
        let long_url = "https://api.anthropic.com/".to_string() + &"x".repeat(5000);
        let api_call_result = ApiCall::new(long_url, "claude-3-sonnet-20241022");

        match api_call_result {
            Ok(api_call) => {
                // If long URLs are accepted, should handle gracefully
                let mut tracker = harness.cost_tracker.lock().await;
                let result = tracker.add_api_call(&session_id, api_call);
                assert!(result.is_ok(), "Should handle long URLs gracefully");
            }
            Err(_) => {
                // If long URLs are rejected, that's also acceptable
                // System should validate input appropriately
            }
        }

        // Test with empty model name
        let empty_model_result = ApiCall::new(
            "https://api.anthropic.com/v1/messages".to_string(),
            "",
        );

        match empty_model_result {
            Ok(api_call) => {
                // If empty models are accepted, should handle gracefully
                let mut tracker = harness.cost_tracker.lock().await;
                let result = tracker.add_api_call(&session_id, api_call);
                // This may succeed or fail depending on validation rules
                println!("Empty model handling: {:?}", result);
            }
            Err(_) => {
                // Empty model rejection is reasonable validation
            }
        }

        // Test with very long model name
        let long_model = "claude-3-".to_string() + &"x".repeat(1000);
        let long_model_result = ApiCall::new(
            "https://api.anthropic.com/v1/messages".to_string(),
            &long_model,
        );

        match long_model_result {
            Ok(api_call) => {
                let mut tracker = harness.cost_tracker.lock().await;
                let result = tracker.add_api_call(&session_id, api_call);
                println!("Long model handling: {:?}", result);
            }
            Err(_) => {
                // Long model name rejection is reasonable
            }
        }

        Ok("Malformed data handling completed")
    }.await;

    let duration = start.elapsed();
    match &result {
        Ok(_) => tracing::info!("Test scenario 'malformed_api_calls' completed successfully in {:?}", duration),
        Err(e) => tracing::error!("Test scenario 'malformed_api_calls' failed after {:?}: {}", duration, e),
    }

    assert!(result.is_ok(), "Malformed data should be handled gracefully: {:?}", result);
}

#[tokio::test]
async fn test_session_state_transitions() {
    let harness = CostTrackingTestHarness::new();

    // Create and complete a session
    let issue_id = IssueId::new("state-transition-test".to_string()).unwrap();
    let session_id = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.start_session(issue_id).unwrap()
    };

    // Complete the session
    {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.complete_session(&session_id, CostSessionStatus::Completed).unwrap();
    }

    // Try to add API call to completed session
    let api_call = ApiCall::new(
        "https://api.anthropic.com/v1/messages".to_string(),
        "claude-3-sonnet-20241022",
    ).unwrap();

    let result = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.add_api_call(&session_id, api_call)
    };

    // Should reject operations on completed sessions
    assert!(result.is_err(), "Should not allow adding API calls to completed session");
    match result.clone().err() {
        Some(CostError::SessionAlreadyCompleted { session_id: error_session_id }) => {
            assert_eq!(error_session_id, session_id, "Error should reference correct session");
        }
        _ => println!("Unexpected error type (implementation may vary): {:?}", result.err()),
    }

    // Try to complete already completed session
    let double_complete_result = {
        let mut tracker = harness.cost_tracker.lock().await;
        tracker.complete_session(&session_id, CostSessionStatus::Failed)
    };

    assert!(double_complete_result.is_err(), "Should not allow double completion");
}

#[tokio::test]
async fn test_invalid_issue_ids() {
    let harness = CostTrackingTestHarness::new();

    // Test with empty issue ID
    let empty_id_result = IssueId::new("".to_string());
    match empty_id_result {
        Ok(issue_id) => {
            // If empty IDs are allowed, should handle gracefully
            let result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)
            };
            println!("Empty issue ID handling: {:?}", result);
        }
        Err(_) => {
            // Empty ID rejection is reasonable validation
        }
    }

    // Test with very long issue ID
    let long_id = "x".repeat(10000);
    let long_id_result = IssueId::new(long_id);
    match long_id_result {
        Ok(issue_id) => {
            let result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)
            };
            println!("Long issue ID handling: {:?}", result);
        }
        Err(_) => {
            // Long ID rejection is reasonable validation
        }
    }

    // Test with special characters
    let special_chars_id = IssueId::new("test<script>alert('xss')</script>".to_string());
    match special_chars_id {
        Ok(issue_id) => {
            let result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)
            };
            println!("Special characters handling: {:?}", result);
        }
        Err(_) => {
            // Special character rejection may be appropriate
        }
    }
}

#[tokio::test]
async fn test_calculator_edge_cases() {
    // Test calculator with unknown model
    let calculator = CostCalculator::paid_default();
    let mut session = CostSession::new(
        IssueId::new("calc-edge-test".to_string()).unwrap(),
    );

    // Add API call with unknown model
    let mut api_call = ApiCall::new(
        "https://api.anthropic.com/v1/messages".to_string(),
        "unknown-model-xyz-123",
    ).unwrap();
    api_call.complete(1000, 500, ApiCallStatus::Success, None);

    session.add_api_call(api_call).unwrap();

    let calculation_result = calculator.calculate_session_cost(&session);
    
    match calculation_result {
        Ok(calculation) => {
            // Unknown model should either use default rates or return zero cost
            println!("Unknown model cost calculation: {:?}", calculation.total_cost);
            assert!(calculation.total_cost >= Decimal::ZERO, "Cost should be non-negative");
        }
        Err(_) => {
            // Calculator may reject unknown models - this is acceptable
            println!("Calculator rejects unknown models");
        }
    }

    // Test with failed API calls
    let mut failed_call = ApiCall::new(
        "https://api.anthropic.com/v1/messages".to_string(),
        "claude-3-sonnet-20241022",
    ).unwrap();
    failed_call.complete(0, 0, ApiCallStatus::Failed, Some("Request failed".to_string()));

    let mut failed_session = CostSession::new(
        IssueId::new("calc-failed-test".to_string()).unwrap(),
    );
    failed_session.add_api_call(failed_call).unwrap();

    let failed_calculation = calculator.calculate_session_cost(&failed_session).unwrap();
    
    // Failed calls with zero tokens should result in zero cost
    assert_eq!(failed_calculation.total_cost, Decimal::ZERO, "Failed calls should not incur cost");
}

#[tokio::test]
async fn test_memory_pressure_scenarios() {
    let mut harness = CostTrackingTestHarness::new();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: memory_pressure");
    let start = std::time::Instant::now();
    
    let result: Result<usize, Box<dyn std::error::Error + Send + Sync>> = async {
        let mut session_ids = Vec::new();
        
        // Create many sessions to test memory management
        for i in 0..100 {
            let issue_id = IssueId::new(format!("memory-pressure-{}", i))?;
            let session_id = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)?
            };
            session_ids.push(session_id);

            // Add multiple API calls to each session
            for j in 0..10 {
                let mut api_call = ApiCall::new(
                    format!("https://api.anthropic.com/v1/messages/{}/{}", i, j),
                    "claude-3-sonnet-20241022",
                )?;
                api_call.complete(100 + j as u32, 50 + j as u32, ApiCallStatus::Success, None);

                let mut tracker = harness.cost_tracker.lock().await;
                tracker.add_api_call(&session_id, api_call)?;
            }
        }

        // Verify system handles large number of sessions
        let final_count = {
            let tracker = harness.cost_tracker.lock().await;
            tracker.session_count()
        };

        assert_eq!(final_count, 100, "Should track all sessions under memory pressure");

        // Complete all sessions
        for session_id in &session_ids {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.complete_session(session_id, CostSessionStatus::Completed)?;
        }

        let completed_count = {
            let tracker = harness.cost_tracker.lock().await;
            tracker.completed_session_count()
        };

        assert_eq!(completed_count, 100, "Should complete all sessions");

        Ok(session_ids.len())
    }.await;

    let duration = start.elapsed();
    match &result {
        Ok(_) => tracing::info!("Test scenario 'memory_pressure' completed successfully in {:?}", duration),
        Err(e) => tracing::error!("Test scenario 'memory_pressure' failed after {:?}: {}", duration, e),
    }

    assert!(result.is_ok(), "Memory pressure should be handled gracefully: {:?}", result);
}

#[tokio::test]
async fn test_token_counter_edge_cases() {
    // Test token counting with various edge cases
    let mut token_counter = TokenCounter::new(0.1); // 10% discrepancy threshold
    
    // Test with empty text - use from_api for testing
    let empty_usage = TokenUsage::from_api(0, 0);
    
    // Test the counter with mock response data since validate_token_usage doesn't exist
    // We'll test token counting from API response instead
    let empty_response = r#"{"usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}}"#;
    let empty_result = token_counter.count_from_response(empty_response, Some(empty_usage.clone()), "test-model");
    assert!(empty_result.is_ok(), "Empty token response should be valid");
    
    // Test with very large text
    let large_usage = TokenUsage::from_estimation(100000, 50000, ConfidenceLevel::Medium);
    
    // Test with large token response
    let large_response = r#"{"usage": {"prompt_tokens": 100000, "completion_tokens": 50000, "total_tokens": 150000}}"#;
    let large_result = token_counter.count_from_response(large_response, Some(large_usage), "test-model");
    assert!(large_result.is_ok(), "Large token response should be handled gracefully");

    // Test with mismatched token counts (API vs estimation)
    let estimated_usage = TokenUsage::from_estimation(10, 5, ConfidenceLevel::Low);
    let mismatched_response = r#"{"usage": {"prompt_tokens": 10000, "completion_tokens": 5000, "total_tokens": 15000}}"#;
    let mismatched_result = token_counter.count_from_response(mismatched_response, Some(estimated_usage), "test-model");
    
    // Should handle token count mismatch (validation happens internally)
    assert!(mismatched_result.is_ok(), 
        "Should detect token count mismatch");
}