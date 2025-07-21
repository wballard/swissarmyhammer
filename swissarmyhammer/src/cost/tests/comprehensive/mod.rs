//! Comprehensive end-to-end system tests
//!
//! This module provides comprehensive integration testing for the complete
//! cost tracking system, validating end-to-end workflows and multi-component
//! interactions under realistic conditions.

// pub mod system_integration;
// pub mod workflow_validation;
// pub mod cross_component_consistency;

use crate::cost::{
    calculator::PricingModel,
    tests::CostTrackingTestHarness,
    tracker::{ApiCallStatus, CostSessionStatus, IssueId},
};
use rust_decimal::Decimal;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_complete_system_workflow_happy_path() {
    let harness = CostTrackingTestHarness::new();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: complete_workflow_happy_path");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let calculator = harness.calculator.clone();
        let api_generator = harness.api_call_generator.clone();

        // Create a test session with realistic workflow
        let issue_id = IssueId::new("comprehensive-happy-001".to_string())?;
        let session_id = {
            let mut tracker = shared_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        // Simulate typical workflow: analysis -> code gen -> testing -> completion
        let api_calls = api_generator.generate_multiple_calls(4);

        for api_call in api_calls {
            let mut tracker = shared_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call)?;
        }

        // Complete the session
        {
            let mut tracker = shared_tracker.lock().await;
            tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
        }

        // Calculate final cost
        let session = {
            let tracker = shared_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist")?;
        let calculation = calculator.calculate_session_cost(&session)?;

        // Validate results
        assert!(
            calculation.total_cost > Decimal::ZERO,
            "Total cost should be positive"
        );
        assert_eq!(session.api_calls.len(), 4, "Should have 4 API calls");
        assert_eq!(
            session.status,
            CostSessionStatus::Completed,
            "Session should be completed"
        );

        Ok(calculation.total_cost)
    };

    let result: Result<Decimal, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'complete_workflow_happy_path' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'complete_workflow_happy_path' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Happy path workflow should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_complete_system_workflow_with_failures() {
    let harness = CostTrackingTestHarness::with_config()
        .with_failure_injection()
        .build();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: complete_workflow_with_failures");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let calculator = harness.calculator.clone();
        let api_generator = harness.api_call_generator.clone();

        let issue_id = IssueId::new("comprehensive-failures-001".to_string())?;
        let session_id = {
            let mut tracker = shared_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        // Generate API calls with some failures
        let api_calls = api_generator.generate_multiple_calls(6);
        let mut success_count = 0;
        let mut failure_count = 0;

        for api_call in api_calls {
            let mut tracker = shared_tracker.lock().await;
            match tracker.add_api_call(&session_id, api_call.clone()) {
                Ok(_) => {
                    if api_call.status == ApiCallStatus::Success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                }
                Err(_) => failure_count += 1,
            }
        }

        // Complete session with appropriate status
        let final_status = if success_count > failure_count {
            CostSessionStatus::Completed
        } else {
            CostSessionStatus::Failed
        };

        {
            let mut tracker = shared_tracker.lock().await;
            tracker.complete_session(&session_id, final_status)?;
        }

        // Validate that system handles failures gracefully
        let session = {
            let tracker = shared_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist even with failures")?;

        // Should be able to calculate cost even with some failures
        let calculation = calculator.calculate_session_cost(&session)?;

        assert!(
            calculation.total_cost >= Decimal::ZERO,
            "Cost should be non-negative with failures"
        );
        assert!(
            session.api_calls.len() == 6,
            "Should track all API calls including failures"
        );

        Ok((success_count, failure_count, calculation.total_cost))
    };

    let result: Result<(i32, i32, Decimal), Box<dyn std::error::Error + Send + Sync>> =
        test_future.await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'complete_workflow_with_failures' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'complete_workflow_with_failures' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "System should handle failures gracefully: {:?}",
        result
    );

    let (success_count, failure_count, total_cost) = result.unwrap();
    assert!(
        success_count > 0 || failure_count > 0,
        "Should have some API call results"
    );
}

#[tokio::test]
async fn test_multiple_concurrent_workflows() {
    let mut harness = CostTrackingTestHarness::new();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: concurrent_workflows");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let calculator = harness.calculator.clone();

        // Create multiple concurrent workflows
        let num_workflows = 5;
        let mut handles = Vec::new();

        for i in 0..num_workflows {
            let tracker_clone = shared_tracker.clone();
            let calc_clone = calculator.clone();
            let api_gen = harness.api_call_generator.clone();

            let handle = tokio::spawn(async move {
                let issue_id = IssueId::new(format!("concurrent-workflow-{}", i))?;
                let session_id = {
                    let mut tracker = tracker_clone.lock().await;
                    tracker.start_session(issue_id)?
                };

                // Add API calls for each workflow
                let api_calls = api_gen.generate_multiple_calls(3);
                for api_call in api_calls {
                    let mut tracker = tracker_clone.lock().await;
                    tracker.add_api_call(&session_id, api_call)?;
                }

                // Complete the workflow
                {
                    let mut tracker = tracker_clone.lock().await;
                    tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
                }

                // Calculate cost
                let session = {
                    let tracker = tracker_clone.lock().await;
                    tracker.get_session(&session_id).cloned()
                };

                let session = session.ok_or("Session should exist")?;
                let calculation = calc_clone.calculate_session_cost(&session)?;

                Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
                    session_id,
                    calculation.total_cost,
                ))
            });

            handles.push(handle);
        }

        // Wait for all workflows to complete
        let mut total_cost = Decimal::ZERO;
        let mut session_count = 0;

        for handle in handles {
            let result = handle.await??;
            total_cost += result.1;
            session_count += 1;
        }

        // Validate concurrent execution results
        assert_eq!(
            session_count, num_workflows,
            "All workflows should complete"
        );
        assert!(total_cost > Decimal::ZERO, "Total cost should be positive");

        // Verify tracker state
        let tracker = shared_tracker.lock().await;
        assert_eq!(
            tracker.session_count(),
            num_workflows,
            "Should have all sessions"
        );
        assert_eq!(
            tracker.completed_session_count(),
            num_workflows,
            "All sessions should be completed"
        );

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((session_count, total_cost))
    };

    let result = timeout(Duration::from_secs(30), test_future).await;

    let scenario_duration = scenario_start.elapsed();
    match &result {
        Ok(Ok(_)) => tracing::info!(
            "Test scenario 'concurrent_workflows' completed successfully in {:?}",
            scenario_duration
        ),
        Ok(Err(e)) => tracing::error!(
            "Test scenario 'concurrent_workflows' failed after {:?}: {}",
            scenario_duration,
            e
        ),
        Err(_) => tracing::error!(
            "Test scenario 'concurrent_workflows' timed out after {:?}",
            scenario_duration
        ),
    }

    assert!(
        result.is_ok(),
        "Concurrent workflows should complete within timeout"
    );
    let scenario_result = result.unwrap();
    assert!(
        scenario_result.is_ok(),
        "Concurrent workflows should succeed: {:?}",
        scenario_result
    );
}

#[tokio::test]
async fn test_system_resource_management() {
    let mut harness = CostTrackingTestHarness::new();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: resource_management");
    let start = std::time::Instant::now();

    let result: Result<usize, Box<dyn std::error::Error + Send + Sync>> = async {
        // Create many sessions to test resource cleanup
        let num_sessions = 20;
        let mut session_ids = Vec::new();

        // Create sessions
        for i in 0..num_sessions {
            let issue_id = IssueId::new(format!("resource-test-{}", i))?;
            let session_id = {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.start_session(issue_id)?
            };
            session_ids.push(session_id);
        }

        // Add API calls to each session
        for session_id in &session_ids {
            let api_calls = harness.api_call_generator.generate_multiple_calls(2);
            for api_call in api_calls {
                let mut tracker = harness.cost_tracker.lock().await;
                tracker.add_api_call(session_id, api_call)?;
            }
        }

        // Complete half the sessions
        let half_count = num_sessions / 2;
        for i in 0..half_count {
            let mut tracker = harness.cost_tracker.lock().await;
            tracker.complete_session(&session_ids[i], CostSessionStatus::Completed)?;
        }

        // Verify resource management
        let tracker = harness.cost_tracker.lock().await;
        assert_eq!(
            tracker.session_count(),
            num_sessions,
            "Should track all sessions"
        );
        assert_eq!(
            tracker.active_session_count(),
            num_sessions - half_count,
            "Should have correct active count"
        );
        assert_eq!(
            tracker.completed_session_count(),
            half_count,
            "Should have correct completed count"
        );

        // Test cleanup behavior (implementation dependent)
        let total_api_calls: usize = session_ids
            .iter()
            .filter_map(|id| tracker.get_session(id))
            .map(|session| session.api_calls.len())
            .sum();

        assert_eq!(
            total_api_calls,
            num_sessions * 2,
            "Should maintain all API call records"
        );

        Ok(num_sessions)
    }
    .await;

    let duration = start.elapsed();
    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'resource_management' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'resource_management' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Resource management should work correctly: {:?}",
        result
    );
}

#[tokio::test]
async fn test_pricing_model_integration() {
    // Test with paid plan
    let mut paid_harness =
        CostTrackingTestHarness::with_pricing_model(PricingModel::paid_with_defaults());

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: paid_plan_integration");
    let start = std::time::Instant::now();

    let paid_result: Result<Decimal, Box<dyn std::error::Error + Send + Sync>> = async {
        let issue_id = IssueId::new("pricing-paid-001".to_string())?;
        let session_id = {
            let mut tracker = paid_harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        let api_calls = paid_harness.api_call_generator.generate_multiple_calls(3);
        for api_call in api_calls {
            let mut tracker = paid_harness.cost_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call)?;
        }

        let session = {
            let tracker = paid_harness.cost_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist")?;
        let calculation = paid_harness.calculator.calculate_session_cost(&session)?;

        assert!(
            calculation.total_cost > Decimal::ZERO,
            "Paid plan should calculate actual cost"
        );
        assert!(
            paid_harness.calculator.supports_cost_calculation(),
            "Paid plan should support cost calculation"
        );

        Ok(calculation.total_cost)
    }
    .await;

    let duration = start.elapsed();
    match &paid_result {
        Ok(_) => tracing::info!(
            "Test scenario 'paid_plan_integration' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'paid_plan_integration' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        paid_result.is_ok(),
        "Paid plan integration should work: {:?}",
        paid_result
    );

    // Test with max plan
    let mut max_harness =
        CostTrackingTestHarness::with_pricing_model(PricingModel::max_with_tracking());

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: max_plan_integration");
    let start = std::time::Instant::now();

    let max_result: Result<usize, Box<dyn std::error::Error + Send + Sync>> = async {
        let issue_id = IssueId::new("pricing-max-001".to_string())?;
        let session_id = {
            let mut tracker = max_harness.cost_tracker.lock().await;
            tracker.start_session(issue_id)?
        };

        let api_calls = max_harness.api_call_generator.generate_multiple_calls(3);
        for api_call in api_calls {
            let mut tracker = max_harness.cost_tracker.lock().await;
            tracker.add_api_call(&session_id, api_call)?;
        }

        let session = {
            let tracker = max_harness.cost_tracker.lock().await;
            tracker.get_session(&session_id).cloned()
        };

        let session = session.ok_or("Session should exist")?;
        let calculation = max_harness.calculator.calculate_session_cost(&session)?;

        assert_eq!(
            calculation.total_cost,
            Decimal::ZERO,
            "Max plan should not calculate cost"
        );
        assert!(
            !max_harness.calculator.supports_cost_calculation(),
            "Max plan should not support cost calculation"
        );

        Ok(session.api_calls.len())
    }
    .await;

    let duration = start.elapsed();
    match &max_result {
        Ok(_) => tracing::info!(
            "Test scenario 'max_plan_integration' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'max_plan_integration' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        max_result.is_ok(),
        "Max plan integration should work: {:?}",
        max_result
    );
}
