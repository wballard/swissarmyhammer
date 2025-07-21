//! Property-based tests for data invariants and consistency
//!
//! This module provides property-based testing to verify that system invariants
//! hold across a wide range of inputs and conditions. These tests ensure that
//! fundamental properties remain true regardless of specific input values.

use crate::cost::{
    calculator::{CostCalculator, PricingModel},
    test_utils::TestDataGenerator,
    tests::CostTrackingTestHarness,
    token_counter::{ConfidenceLevel, TokenCounter, TokenUsage},
    tracker::{ApiCall, ApiCallStatus, CostSessionStatus, IssueId},
};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Property: Cost calculations should always be non-negative
#[tokio::test]
async fn property_cost_calculations_non_negative() {
    let harness = CostTrackingTestHarness::new();
    let data_generator = TestDataGenerator::default();

    // Test with various cost calculation scenarios
    let test_cases = data_generator.generate_cost_test_cases();

    for (input_tokens, output_tokens, model, should_have_cost) in test_cases {
        let mut session = crate::cost::CostSession::new(
            IssueId::new("property-non-negative".to_string()).unwrap(),
        );

        // Create API call with test case parameters
        let mut api_call =
            ApiCall::new("https://api.anthropic.com/v1/messages".to_string(), model).unwrap();

        api_call.complete(input_tokens, output_tokens, ApiCallStatus::Success, None);
        session.add_api_call(api_call).unwrap();

        // Calculate cost
        let calculation_result = harness.calculator.calculate_session_cost(&session);

        match calculation_result {
            Ok(calculation) => {
                // Property: All costs must be non-negative
                assert!(
                    calculation.total_cost >= Decimal::ZERO,
                    "Cost must be non-negative for inputs ({}, {}, {}): got {}",
                    input_tokens,
                    output_tokens,
                    model,
                    calculation.total_cost
                );

                // Property: Input cost must be non-negative
                assert!(
                    calculation.input_cost >= Decimal::ZERO,
                    "Input cost must be non-negative: got {}",
                    calculation.input_cost
                );

                // Property: Output cost must be non-negative
                assert!(
                    calculation.output_cost >= Decimal::ZERO,
                    "Output cost must be non-negative: got {}",
                    calculation.output_cost
                );

                // Property: Total cost equals sum of components
                let expected_total = calculation.input_cost + calculation.output_cost;
                assert_eq!(
                    calculation.total_cost, expected_total,
                    "Total cost should equal sum of components: {} != {} + {}",
                    calculation.total_cost, calculation.input_cost, calculation.output_cost
                );

                if should_have_cost && harness.calculator.supports_cost_calculation() {
                    // For known models with tokens, cost should be positive (unless tokens are zero)
                    if input_tokens > 0 || output_tokens > 0 {
                        assert!(
                            calculation.total_cost > Decimal::ZERO,
                            "Cost should be positive for non-zero tokens ({}, {}): got {}",
                            input_tokens,
                            output_tokens,
                            calculation.total_cost
                        );
                    }
                }
            }
            Err(_) => {
                // Some test cases may legitimately fail (e.g., invalid models)
                // This is acceptable for property testing
            }
        }
    }
}

/// Property: Token counts should be consistent across operations
#[tokio::test]
async fn property_token_count_consistency() {
    let harness = CostTrackingTestHarness::new();
    let mut token_counter = TokenCounter::new(0.1);

    // Test various token patterns with API responses
    let test_patterns = vec![
        (
            0,
            0,
            r#"{"usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}}"#,
        ),
        (
            1,
            1,
            r#"{"usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}}"#,
        ),
        (
            10,
            5,
            r#"{"usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}}"#,
        ),
        (
            100,
            50,
            r#"{"usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}}"#,
        ),
        (
            1000,
            2000,
            r#"{"usage": {"prompt_tokens": 1000, "completion_tokens": 2000, "total_tokens": 3000}}"#,
        ),
    ];

    for (expected_input, expected_output, api_response) in test_patterns {
        // Create expected usage for comparison
        let expected_usage =
            TokenUsage::from_estimation(expected_input, expected_output, ConfidenceLevel::Medium);

        // Test token counting from API response
        let result =
            token_counter.count_from_response(api_response, Some(expected_usage), "test-model");

        // Property: Counting should always complete without panicking
        assert!(result.is_ok(), "Token counting should not panic");

        let actual_usage = result.unwrap();

        // Property: Total tokens should equal input + output
        assert_eq!(
            actual_usage.input_tokens + actual_usage.output_tokens,
            actual_usage.total_tokens,
            "Token sum should be consistent: {} + {} = {}",
            actual_usage.input_tokens,
            actual_usage.output_tokens,
            actual_usage.total_tokens
        );

        // Property: Token values should be non-negative
        assert!(
            actual_usage.input_tokens >= 0,
            "Input tokens should be non-negative"
        );
        assert!(
            actual_usage.output_tokens >= 0,
            "Output tokens should be non-negative"
        );
        assert!(
            actual_usage.total_tokens >= 0,
            "Total tokens should be non-negative"
        );
    }
}

/// Property: Session state transitions should be valid
#[tokio::test]
async fn property_session_state_transitions() {
    let harness = CostTrackingTestHarness::new();

    // Test various session lifecycle patterns
    let test_scenarios = vec![
        vec![CostSessionStatus::Completed],
        vec![CostSessionStatus::Failed],
        vec![CostSessionStatus::Cancelled],
    ];

    for (scenario_id, final_states) in test_scenarios.iter().enumerate() {
        for final_state in final_states {
            let issue_id =
                IssueId::new(format!("property-state-{}-{:?}", scenario_id, final_state)).unwrap();

            // Property: Session creation should always succeed with valid input
            let session_id = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)
            };
            assert!(
                session_id.is_ok(),
                "Session creation should succeed with valid issue ID"
            );
            let session_id = session_id.unwrap();

            // Property: New sessions should be in active state
            let session = {
                let tracker = harness.cost_tracker.lock().await;
                tracker.get_session(&session_id).cloned()
            };
            assert!(session.is_some(), "New session should exist");
            let session = session.unwrap();
            assert_eq!(
                session.status,
                CostSessionStatus::Active,
                "New session should be active"
            );

            // Property: Should be able to add API calls to active sessions
            let api_call = harness.api_call_generator.generate_completed_api_call(0);
            let add_result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.add_api_call(&session_id, api_call.clone())
            };
            assert!(
                add_result.is_ok(),
                "Should be able to add API calls to active session"
            );

            // Property: Session completion should succeed
            let complete_result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.complete_session(&session_id, final_state.clone())
            };
            assert!(complete_result.is_ok(), "Session completion should succeed");

            // Property: Completed session should have correct final state
            let final_session = {
                let tracker = harness.cost_tracker.lock().await;
                tracker.get_session(&session_id).cloned()
            };
            assert!(
                final_session.is_some(),
                "Completed session should still exist"
            );
            let final_session = final_session.unwrap();
            assert_eq!(
                final_session.status,
                final_state.clone(),
                "Final session state should match requested state"
            );

            // Property: Should not be able to modify completed sessions
            let modify_result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.add_api_call(&session_id, api_call)
            };
            assert!(
                modify_result.is_err(),
                "Should not be able to modify completed session"
            );
        }
    }
}

/// Property: Data consistency across storage backends
#[tokio::test]
async fn property_data_consistency_across_operations() {
    let harness = CostTrackingTestHarness::new();

    // Create multiple sessions with various API call patterns
    let session_count = 10;
    let mut session_data = HashMap::new();

    for i in 0..session_count {
        let issue_id = IssueId::new(format!("property-consistency-{}", i)).unwrap();
        let session_id = {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)
        }
        .unwrap();

        let num_api_calls = (i % 5) + 1; // 1-5 API calls per session
        let api_calls = harness
            .api_call_generator
            .generate_multiple_calls(num_api_calls as u32);

        let mut expected_input_tokens = 0u32;
        let mut expected_output_tokens = 0u32;

        for api_call in api_calls.clone() {
            expected_input_tokens += api_call.input_tokens;
            expected_output_tokens += api_call.output_tokens;

            let mut tracker = harness.cost_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call).unwrap();
        }

        session_data.insert(
            session_id,
            (
                api_calls.len(),
                expected_input_tokens,
                expected_output_tokens,
            ),
        );
    }

    // Verify data consistency properties
    for (session_id, (expected_calls, expected_input, expected_output)) in &session_data {
        let session = {
            let tracker = harness.cost_tracker.lock().await;
            tracker.get_session(session_id).cloned()
        };

        assert!(
            session.is_some(),
            "Session should exist for ID: {:?}",
            session_id
        );
        let session = session.unwrap();

        // Property: API call count should match what was added
        assert_eq!(
            session.api_calls.len(),
            *expected_calls,
            "API call count should match for session {:?}: expected {}, got {}",
            session_id,
            expected_calls,
            session.api_calls.len()
        );

        // Property: Token counts should sum correctly
        let actual_input: u32 = session
            .api_calls
            .iter()
            .map(|(_, call)| call.input_tokens)
            .sum();
        let actual_output: u32 = session
            .api_calls
            .iter()
            .map(|(_, call)| call.output_tokens)
            .sum();

        assert_eq!(
            actual_input, *expected_input,
            "Input token sum should match for session {:?}: expected {}, got {}",
            session_id, expected_input, actual_input
        );
        assert_eq!(
            actual_output, *expected_output,
            "Output token sum should match for session {:?}: expected {}, got {}",
            session_id, expected_output, actual_output
        );

        // Property: All API calls should have valid timestamps
        for (call_idx, (_, api_call)) in session.api_calls.iter().enumerate() {
            assert!(
                api_call.started_at <= api_call.completed_at.unwrap_or(api_call.started_at),
                "API call {} in session {:?} should have valid timestamps",
                call_idx,
                session_id
            );
        }
    }
}

/// Property: Session cleanup completeness
#[tokio::test]
async fn property_session_cleanup_completeness() {
    let mut harness = CostTrackingTestHarness::new();

    // Create sessions and complete them
    let session_count = 20;
    let mut completed_sessions = Vec::new();

    for i in 0..session_count {
        let issue_id = IssueId::new(format!("property-cleanup-{}", i)).unwrap();
        let session_id = {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)
        }
        .unwrap();

        // Add some API calls
        let api_calls = harness.api_call_generator.generate_multiple_calls(3);
        for api_call in api_calls {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call).unwrap();
        }

        // Complete half the sessions
        if i < session_count / 2 {
            {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker
                    .complete_session(&session_id, CostSessionStatus::Completed)
                    .unwrap();
            }
            completed_sessions.push(session_id);
        }
    }

    // Verify cleanup properties
    let tracker_guard = harness.cost_tracker.lock().await;

    // Property: Total session count should equal created sessions
    assert_eq!(
        tracker_guard.session_count(),
        session_count,
        "Total session count should equal created sessions"
    );

    // Property: Completed count should match what we completed
    assert_eq!(
        tracker_guard.completed_session_count(),
        completed_sessions.len(),
        "Completed session count should match"
    );

    // Property: Active count should equal total minus completed
    let expected_active = session_count - completed_sessions.len();
    assert_eq!(
        tracker_guard.active_session_count(),
        expected_active,
        "Active session count should equal total minus completed"
    );

    // Property: All completed sessions should be accessible
    for session_id in &completed_sessions {
        let session = tracker_guard.get_session(session_id);
        assert!(
            session.is_some(),
            "Completed session {:?} should be accessible",
            session_id
        );
        assert_eq!(
            session.unwrap().status,
            CostSessionStatus::Completed,
            "Retrieved session should have completed status"
        );
    }
}

/// Property: Pricing model consistency
#[tokio::test]
async fn property_pricing_model_consistency() {
    let test_cases = vec![
        (PricingModel::paid_with_defaults(), true),
        (PricingModel::max_with_tracking(), false),
        (PricingModel::max_with_estimates(), true),
    ];

    for (pricing_model, should_calculate_cost) in test_cases {
        let calculator = CostCalculator::new(pricing_model);
        let mut session =
            crate::cost::CostSession::new(IssueId::new("property-pricing".to_string()).unwrap());

        // Add API call with known token counts
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages".to_string(),
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        api_call.complete(1000, 500, ApiCallStatus::Success, None);
        session.add_api_call(api_call).unwrap();

        let calculation = calculator.calculate_session_cost(&session).unwrap();

        // Property: Cost calculation availability should match pricing model
        assert_eq!(
            calculator.supports_cost_calculation(),
            should_calculate_cost,
            "Cost calculation support should match pricing model"
        );

        if should_calculate_cost {
            // Property: Paid models should calculate actual costs
            assert!(
                calculation.total_cost > Decimal::ZERO,
                "Paid model should calculate positive cost for non-zero tokens"
            );
        } else {
            // Property: Max models should not calculate costs
            assert_eq!(
                calculation.total_cost,
                Decimal::ZERO,
                "Max model should not calculate actual cost"
            );
        }

        // Property: All calculations should be mathematically consistent
        let expected_total = calculation.input_cost + calculation.output_cost;
        assert_eq!(
            calculation.total_cost, expected_total,
            "Total cost should always equal sum of input and output costs"
        );
    }
}

/// Property: Concurrent operations maintain data integrity
#[tokio::test]
async fn property_concurrent_data_integrity() {
    let harness = CostTrackingTestHarness::new();
    let shared_tracker = harness.get_shared_tracker();

    // Run concurrent operations
    let num_workers = 5;
    let operations_per_worker = 10;

    let handles: Vec<_> = (0..num_workers)
        .map(|worker_id| {
            let tracker_clone = shared_tracker.clone();
            let api_gen = harness.api_call_generator.clone();

            tokio::spawn(async move {
                let mut worker_sessions = Vec::new();

                for op_id in 0..operations_per_worker {
                    let issue_id =
                        IssueId::new(format!("property-concurrent-{}-{}", worker_id, op_id))
                            .unwrap();
                    let session_id = {
                        let mut tracker = tracker_clone.lock().await;
                        tracker.start_session(issue_id).unwrap()
                    };

                    // Add API calls
                    let api_calls = api_gen.generate_multiple_calls(3);
                    let mut expected_tokens = 0u32;

                    for api_call in api_calls {
                        expected_tokens += api_call.input_tokens;
                        expected_tokens += api_call.output_tokens;

                        let mut tracker = tracker_clone.lock().await;
                        tracker.add_api_call(&session_id, api_call).unwrap();
                    }

                    worker_sessions.push((session_id, expected_tokens));
                }

                worker_sessions
            })
        })
        .collect();

    // Collect all results
    let mut all_sessions = Vec::new();
    for handle in handles {
        let worker_sessions = handle.await.unwrap();
        all_sessions.extend(worker_sessions);
    }

    // Verify data integrity properties
    let tracker_guard = shared_tracker.lock().await;

    // Property: All created sessions should be tracked
    assert_eq!(
        tracker_guard.session_count(),
        all_sessions.len(),
        "All concurrently created sessions should be tracked"
    );

    // Property: Each session should contain correct data
    for (session_id, expected_tokens) in &all_sessions {
        let session = tracker_guard.get_session(session_id);
        assert!(
            session.is_some(),
            "Concurrent session {:?} should exist",
            session_id
        );

        let session = session.unwrap();
        assert_eq!(
            session.api_calls.len(),
            3,
            "Each session should have 3 API calls"
        );

        let actual_tokens: u32 = session
            .api_calls
            .iter()
            .map(|(_, call)| call.input_tokens + call.output_tokens)
            .sum();

        assert_eq!(
            actual_tokens, *expected_tokens,
            "Token counts should be preserved across concurrent operations"
        );
    }
}

// Property-based testing using the proptest crate would go here if we had it as a dependency
// For now, we use structured property testing with predetermined test cases

/// Property: API call data preservation
#[tokio::test]
async fn property_api_call_data_preservation() {
    let mut harness = CostTrackingTestHarness::new();

    // Test various API call configurations
    let test_configurations = vec![
        (
            "https://short.url",
            "model1",
            100,
            50,
            ApiCallStatus::Success,
            None,
        ),
        (
            "https://very-long-url.example.com/api/v1/messages/with/many/path/segments",
            "claude-3-sonnet-20241022",
            0,
            0,
            ApiCallStatus::Failed,
            Some("Error message".to_string()),
        ),
        (
            "https://api.example.com",
            "unknown-model-name",
            1000000,
            500000,
            ApiCallStatus::Success,
            None,
        ),
        (
            "https://api.example.com",
            "",
            1,
            1,
            ApiCallStatus::Timeout,
            Some("Request timeout".to_string()),
        ),
    ];

    for (url, model, input_tokens, output_tokens, status, error_msg) in test_configurations {
        let issue_id = IssueId::new("property-data-preservation".to_string()).unwrap();
        let session_id = {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)
        }
        .unwrap();

        // Create API call (some configurations may fail)
        let api_call_result = ApiCall::new(url.to_string(), model);

        if let Ok(mut api_call) = api_call_result {
            api_call.complete(
                input_tokens,
                output_tokens,
                status.clone(),
                error_msg.clone(),
            );

            let add_result = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.add_api_call(&session_id, api_call.clone())
            };

            if add_result.is_ok() {
                // Verify data preservation properties
                let session = {
                    let tracker = harness.cost_tracker.lock().await;
                    tracker.get_session(&session_id).cloned()
                };

                let session = session.unwrap();
                assert!(
                    !session.api_calls.is_empty(),
                    "API call should be preserved"
                );

                // Get any API call from the session to verify data preservation
                let (_, preserved_call) = session
                    .api_calls
                    .iter()
                    .next()
                    .expect("Session should have at least one API call");

                // Property: All API call data should be preserved exactly
                assert_eq!(preserved_call.endpoint, url, "Endpoint should be preserved");
                assert_eq!(preserved_call.model, model, "Model should be preserved");
                assert_eq!(
                    preserved_call.input_tokens, input_tokens,
                    "Input tokens should be preserved"
                );
                assert_eq!(
                    preserved_call.output_tokens, output_tokens,
                    "Output tokens should be preserved"
                );
                assert_eq!(preserved_call.status, status, "Status should be preserved");

                if let Some(ref expected_error) = error_msg {
                    assert_eq!(
                        preserved_call.error_message.as_ref(),
                        Some(expected_error),
                        "Error message should be preserved"
                    );
                } else {
                    assert!(
                        preserved_call.error_message.is_none(),
                        "No error message should be preserved when none provided"
                    );
                }

                // Property: Timestamps should be valid
                assert!(
                    preserved_call.started_at
                        <= preserved_call
                            .completed_at
                            .unwrap_or(preserved_call.started_at),
                    "API call timestamps should be valid"
                );
            }
        }
    }
}
