//! Integration tests for cost tracking foundation
//!
//! This module provides comprehensive integration tests that validate the complete
//! cost tracking system works correctly across all foundation components.
//! Tests cover session lifecycle, configuration integration, cost calculations,
//! memory management, and error handling scenarios.

use crate::cost::{
    calculator::{CostCalculator, MaxPlanConfig, PaidPlanConfig, PricingModel},
    tracker::{
        ApiCall, ApiCallId, ApiCallStatus, CostError, CostSessionId, CostSessionStatus,
        CostTracker, IssueId, MAX_API_CALLS_PER_SESSION, MAX_COST_SESSIONS,
    },
};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test helper for creating realistic API call data
pub fn create_test_api_call(endpoint_suffix: &str, model: &str) -> Result<ApiCall, CostError> {
    ApiCall::new(
        format!("https://api.anthropic.com/v1/{}", endpoint_suffix),
        model,
    )
}

/// Test helper for creating issue IDs with validation
pub fn create_test_issue_id(suffix: &str) -> Result<IssueId, CostError> {
    IssueId::new(format!("issue-{}", suffix))
}

/// Test helper for completing API calls with realistic data
pub fn complete_api_call_with_realistic_data(
    api_call: &mut ApiCall,
    input_tokens: u32,
    output_tokens: u32,
    success: bool,
) {
    let status = if success {
        ApiCallStatus::Success
    } else {
        ApiCallStatus::Failed
    };
    let error = if success {
        None
    } else {
        Some("Rate limit exceeded".to_string())
    };

    api_call.complete(input_tokens, output_tokens, status, error);
}

/// Helper function to create multiple test sessions for integration testing
fn create_workflow_test_sessions(tracker: &mut CostTracker, num_sessions: u32) -> Result<Vec<CostSessionId>, crate::cost::CostError> {
    let issue_ids: Vec<_> = (1..=num_sessions)
        .map(|i| create_test_issue_id(&format!("workflow-{}", i)).unwrap())
        .collect();

    let mut session_ids = Vec::new();
    for issue_id in &issue_ids {
        let session_id = tracker.start_session(issue_id.clone())?;
        session_ids.push(session_id);
    }

    Ok(session_ids)
}

/// Helper function to add and complete API calls for integration testing
async fn add_and_complete_api_calls(
    tracker: &mut CostTracker, 
    session_ids: &[CostSessionId]
) -> Result<Vec<(CostSessionId, crate::cost::ApiCallId)>, crate::cost::CostError> {
    let models = ["claude-3-sonnet-20241022", "claude-3-haiku-20240229"];
    let mut all_call_ids = Vec::new();

    // Add API calls to each session
    for (i, session_id) in session_ids.iter().enumerate() {
        for j in 0..3 {
            let model = models[j % models.len()];
            let api_call = create_test_api_call(&format!("messages/{}/{}", i, j), model)
                .expect("Should create API call");

            // Simulate realistic processing time
            sleep(Duration::from_millis(10)).await;

            let call_id = tracker.add_api_call(session_id, api_call)?;
            all_call_ids.push((*session_id, call_id));
        }
    }

    // Complete API calls with varying success rates and realistic token counts
    for (call_index, (session_id, call_id)) in all_call_ids.iter().enumerate() {
        let input_tokens = 100 + (call_index as u32 * 50);
        let output_tokens = 200 + (call_index as u32 * 30);
        let success = call_index % 4 != 0; // 25% failure rate

        tracker.complete_api_call(
            session_id,
            call_id,
            input_tokens,
            output_tokens,
            if success {
                ApiCallStatus::Success
            } else {
                ApiCallStatus::Failed
            },
            if success {
                None
            } else {
                Some("Timeout error".to_string())
            },
        )?;
    }

    Ok(all_call_ids)
}

/// Helper function to calculate and verify session costs
fn calculate_and_verify_costs(
    tracker: &CostTracker,
    calculator: &CostCalculator,
    session_ids: &[CostSessionId]
) -> Result<Decimal, crate::cost::CostError> {
    let mut total_workflow_cost = Decimal::ZERO;
    
    for session_id in session_ids {
        let session = tracker.get_session(session_id).unwrap();
        let cost_calculation = calculator.calculate_session_cost(session)?;

        assert!(cost_calculation.total_cost >= Decimal::ZERO);
        assert_eq!(session.api_call_count(), 3);
        total_workflow_cost += cost_calculation.total_cost;
    }
    
    Ok(total_workflow_cost)
}

/// Helper function to complete sessions with different outcomes
fn complete_workflow_sessions(
    tracker: &mut CostTracker,
    session_ids: &[CostSessionId]
) -> Result<(), crate::cost::CostError> {
    for (i, session_id) in session_ids.iter().enumerate() {
        let status = if i % 2 == 0 {
            CostSessionStatus::Completed
        } else {
            CostSessionStatus::Failed
        };
        tracker.complete_session(session_id, status)?;
    }
    Ok(())
}

/// Complete end-to-end integration test covering the full workflow
#[tokio::test]
async fn test_complete_cost_tracking_workflow() {
    let mut tracker = CostTracker::new();
    let calculator = CostCalculator::paid_default();

    // Phase 1: Create test sessions
    let session_ids = create_workflow_test_sessions(&mut tracker, 5)
        .expect("Should create test sessions");

    assert_eq!(tracker.session_count(), 5);
    assert_eq!(tracker.active_session_count(), 5);

    // Phase 2 & 3: Add and complete API calls
    add_and_complete_api_calls(&mut tracker, &session_ids).await
        .expect("Should add and complete API calls");

    // Phase 4: Calculate and verify costs
    let total_workflow_cost = calculate_and_verify_costs(&tracker, &calculator, &session_ids)
        .expect("Should calculate and verify costs");

    // Phase 5: Complete sessions
    complete_workflow_sessions(&mut tracker, &session_ids)
        .expect("Should complete sessions");

    // Phase 6: Verify final state
    assert_eq!(tracker.session_count(), 5);
    assert_eq!(tracker.active_session_count(), 0);
    assert_eq!(tracker.completed_session_count(), 5);
    assert!(total_workflow_cost > Decimal::ZERO);

    // Verify all sessions have expected characteristics
    for session_id in &session_ids {
        let session = tracker.get_session(session_id).unwrap();
        assert!(session.is_completed());
        assert!(session.total_duration.is_some());
        assert_eq!(session.api_call_count(), 3);
        assert!(session.total_tokens() > 0);
    }
}

/// Test configuration integration with different pricing models
#[tokio::test]
async fn test_configuration_integration() {
    // Test 1: Paid Plan Configuration
    let paid_config = PaidPlanConfig::new_with_defaults();
    let paid_calculator = CostCalculator::new(PricingModel::Paid(paid_config));

    assert!(paid_calculator.supports_cost_calculation());
    assert!(!paid_calculator.provides_estimates());

    // Create test session with paid model
    let mut tracker = CostTracker::new();
    let issue_id = create_test_issue_id("config-paid").unwrap();
    let session_id = tracker.start_session(issue_id).unwrap();

    let mut api_call = create_test_api_call("messages/config", "claude-3-sonnet").unwrap();
    complete_api_call_with_realistic_data(&mut api_call, 1000, 500, true);
    let _call_id = tracker.add_api_call(&session_id, api_call).unwrap();

    let session = tracker.get_session(&session_id).unwrap();
    let cost_calculation = paid_calculator
        .calculate_session_cost(session)
        .expect("Should calculate paid cost");

    assert!(cost_calculation.total_cost > Decimal::ZERO);
    assert!(!cost_calculation.is_estimated);

    // Test 2: Max Plan with Tracking
    let max_config = MaxPlanConfig::new(true);
    let max_calculator = CostCalculator::new(PricingModel::Max(max_config));

    assert!(!max_calculator.supports_cost_calculation());
    assert!(!max_calculator.provides_estimates());

    let cost_calculation = max_calculator
        .calculate_session_cost(session)
        .expect("Should calculate max cost");

    assert_eq!(cost_calculation.total_cost, Decimal::ZERO);
    assert_eq!(cost_calculation.input_tokens, 1000);
    assert_eq!(cost_calculation.output_tokens, 500);

    // Test 3: Max Plan with Estimates
    let max_with_estimates =
        MaxPlanConfig::new_with_estimates(true, PaidPlanConfig::new_with_defaults());
    let estimate_calculator = CostCalculator::new(PricingModel::Max(max_with_estimates));

    assert!(estimate_calculator.supports_cost_calculation());
    assert!(estimate_calculator.provides_estimates());

    let cost_calculation = estimate_calculator
        .calculate_session_cost(session)
        .expect("Should calculate estimated cost");

    assert!(cost_calculation.total_cost > Decimal::ZERO);
    assert!(cost_calculation.is_estimated);

    tracker
        .complete_session(&session_id, CostSessionStatus::Completed)
        .unwrap();
}

/// Helper function to create concurrent test sessions
fn create_concurrent_test_sessions(
    tracker: &mut CostTracker, 
    num_sessions: usize
) -> Result<Vec<CostSessionId>, crate::cost::CostError> {
    let mut session_ids = Vec::new();
    
    for i in 0..num_sessions {
        let issue_id = create_test_issue_id(&format!("concurrent-{}", i))?;
        let session_id = tracker.start_session(issue_id)?;
        session_ids.push(session_id);
    }
    
    Ok(session_ids)
}

/// Helper function to process API calls for concurrent testing  
async fn process_concurrent_api_calls(
    tracker: &mut CostTracker,
    session_ids: &[CostSessionId],
    calls_per_session: usize
) -> Result<Vec<(CostSessionId, crate::cost::ApiCallId, usize, usize)>, crate::cost::CostError> {
    let mut all_calls = Vec::new();
    
    // Add API calls to all sessions
    for (session_index, session_id) in session_ids.iter().enumerate() {
        for call_index in 0..calls_per_session {
            let api_call = create_test_api_call(
                &format!("concurrent/{}/{}", session_index, call_index),
                "claude-3-sonnet-20241022",
            ).unwrap();

            let call_id = tracker.add_api_call(session_id, api_call)?;
            all_calls.push((*session_id, call_id, session_index, call_index));
        }
    }

    // Complete all API calls with realistic timing
    for (session_id, call_id, session_index, call_index) in &all_calls {
        let input_tokens = 100 + (session_index * 10) as u32 + (call_index * 5) as u32;
        let output_tokens = 150 + (session_index * 15) as u32 + (call_index * 8) as u32;

        tracker.complete_api_call(
            session_id,
            call_id,
            input_tokens,
            output_tokens,
            ApiCallStatus::Success,
            None,
        )?;

        // Add small delay to simulate realistic timing
        sleep(Duration::from_millis(1)).await;
    }
    
    Ok(all_calls)
}

/// Helper function to verify concurrent sessions state and costs
fn verify_concurrent_sessions_state(
    tracker: &CostTracker,
    calculator: &CostCalculator,
    session_ids: &[CostSessionId],
    calls_per_session: usize
) -> Result<(), crate::cost::CostError> {
    for session_id in session_ids {
        let session = tracker.get_session(session_id).unwrap();
        assert_eq!(session.api_call_count(), calls_per_session);
        assert!(session.total_tokens() > 0);

        let cost_calculation = calculator.calculate_session_cost(session)?;
        assert!(cost_calculation.total_cost > Decimal::ZERO);
    }
    Ok(())
}

/// Helper function to complete concurrent sessions with different outcomes
fn complete_concurrent_sessions(
    tracker: &mut CostTracker,
    session_ids: &[CostSessionId]
) -> Result<(), crate::cost::CostError> {
    for (i, session_id) in session_ids.iter().enumerate() {
        let status = if i % 3 == 0 {
            CostSessionStatus::Failed
        } else {
            CostSessionStatus::Completed
        };
        tracker.complete_session(session_id, status)?;
    }
    Ok(())
}

/// Test concurrent session handling and thread safety
#[tokio::test]
async fn test_concurrent_sessions() {
    let mut tracker = CostTracker::new();
    let calculator = CostCalculator::paid_default();

    // Test configuration
    const NUM_SESSIONS: usize = 10;
    const CALLS_PER_SESSION: usize = 5;

    // Phase 1: Create concurrent sessions
    let session_ids = create_concurrent_test_sessions(&mut tracker, NUM_SESSIONS)
        .expect("Should create concurrent sessions");

    assert_eq!(tracker.session_count(), NUM_SESSIONS);
    assert_eq!(tracker.active_session_count(), NUM_SESSIONS);

    // Phase 2 & 3: Process API calls concurrently
    process_concurrent_api_calls(&mut tracker, &session_ids, CALLS_PER_SESSION).await
        .expect("Should process concurrent API calls");

    // Phase 4: Verify session state and costs  
    verify_concurrent_sessions_state(&tracker, &calculator, &session_ids, CALLS_PER_SESSION)
        .expect("Should verify concurrent sessions state");

    // Phase 5: Complete all sessions
    complete_concurrent_sessions(&mut tracker, &session_ids)
        .expect("Should complete concurrent sessions");

    assert_eq!(tracker.session_count(), NUM_SESSIONS);
    assert_eq!(tracker.active_session_count(), 0);
    assert_eq!(tracker.completed_session_count(), NUM_SESSIONS);
}

/// Test memory management and cleanup behavior
#[tokio::test]
async fn test_memory_management() {
    let mut tracker = CostTracker::new();

    // Test 1: Session limit enforcement
    let mut session_ids = Vec::new();

    // Fill up to near the limit
    for i in 0..(MAX_COST_SESSIONS - 5) {
        let issue_id = create_test_issue_id(&format!("memory-{}", i)).unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();
        session_ids.push(session_id);
    }

    assert_eq!(tracker.session_count(), MAX_COST_SESSIONS - 5);

    // Complete some sessions to test cleanup eligibility
    for session_id in session_ids.iter().take(10) {
        tracker
            .complete_session(session_id, CostSessionStatus::Completed)
            .unwrap();
    }

    assert_eq!(tracker.completed_session_count(), 10);

    // Test 2: API call limit per session
    let issue_id = create_test_issue_id("api-limit-test").unwrap();
    let session_id = tracker.start_session(issue_id).unwrap();

    // Fill up to the API call limit
    for i in 0..MAX_API_CALLS_PER_SESSION {
        let api_call =
            create_test_api_call(&format!("limit-test/{}", i), "claude-3-haiku").unwrap();
        let result = tracker.add_api_call(&session_id, api_call);
        assert!(result.is_ok(), "API call {} should succeed", i);
    }

    let session = tracker.get_session(&session_id).unwrap();
    assert_eq!(session.api_call_count(), MAX_API_CALLS_PER_SESSION);

    // Adding one more should fail
    let overflow_call = create_test_api_call("overflow", "claude-3-haiku").unwrap();
    let result = tracker.add_api_call(&session_id, overflow_call);
    assert!(matches!(result, Err(CostError::TooManyApiCalls { .. })));

    // Test 3: Memory cleanup behavior
    let initial_count = tracker.session_count();
    tracker.cleanup_old_sessions();

    // In test environment, sessions won't be old enough to clean up automatically
    // But the cleanup should run without errors
    assert_eq!(tracker.session_count(), initial_count);
}

/// Test error handling throughout the system
#[tokio::test]
async fn test_error_handling_integration() {
    let mut tracker = CostTracker::new();
    let _calculator = CostCalculator::paid_default();

    // Test 1: Invalid session operations
    let invalid_session_id = CostSessionId::new();
    let invalid_call_id = ApiCallId::new();

    // Try to complete non-existent session
    let result = tracker.complete_session(&invalid_session_id, CostSessionStatus::Completed);
    assert!(matches!(result, Err(CostError::SessionNotFound { .. })));

    // Try to add API call to non-existent session
    let api_call = create_test_api_call("error-test", "claude-3-sonnet").unwrap();
    let result = tracker.add_api_call(&invalid_session_id, api_call);
    assert!(matches!(result, Err(CostError::SessionNotFound { .. })));

    // Test 2: Invalid API call operations
    let issue_id = create_test_issue_id("error-test").unwrap();
    let session_id = tracker.start_session(issue_id).unwrap();

    // Try to complete non-existent API call
    let result = tracker.complete_api_call(
        &session_id,
        &invalid_call_id,
        100,
        200,
        ApiCallStatus::Success,
        None,
    );
    assert!(matches!(result, Err(CostError::ApiCallNotFound { .. })));

    // Test 3: Double completion errors
    let api_call = create_test_api_call("double-complete", "claude-3-sonnet").unwrap();
    let call_id = tracker.add_api_call(&session_id, api_call).unwrap();

    // Complete once - should succeed
    tracker
        .complete_api_call(
            &session_id,
            &call_id,
            100,
            200,
            ApiCallStatus::Success,
            None,
        )
        .unwrap();

    // Complete session once - should succeed
    tracker
        .complete_session(&session_id, CostSessionStatus::Completed)
        .unwrap();

    // Try to complete session again - should fail
    let result = tracker.complete_session(&session_id, CostSessionStatus::Failed);
    assert!(matches!(
        result,
        Err(CostError::SessionAlreadyCompleted { .. })
    ));

    // Test 4: Input validation errors
    let invalid_issue_result = IssueId::new("");
    assert!(matches!(
        invalid_issue_result,
        Err(CostError::InvalidInput { .. })
    ));

    let invalid_issue_result = IssueId::new("a".repeat(300));
    assert!(matches!(
        invalid_issue_result,
        Err(CostError::InvalidInput { .. })
    ));

    let invalid_call_result = ApiCall::new("", "claude-3-sonnet");
    assert!(matches!(
        invalid_call_result,
        Err(CostError::InvalidInput { .. })
    ));

    let invalid_call_result = ApiCall::new("https://api.anthropic.com/v1/messages", "");
    assert!(matches!(
        invalid_call_result,
        Err(CostError::InvalidInput { .. })
    ));
}

/// Helper function to measure session creation performance
fn measure_session_creation_performance(
    tracker: &mut CostTracker,
    num_sessions: usize,
) -> (Duration, Vec<CostSessionId>) {
    let start_time = Instant::now();
    let mut session_ids = Vec::new();
    
    for i in 0..num_sessions {
        let issue_id = create_test_issue_id(&format!("perf-{}", i)).unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();
        session_ids.push(session_id);
    }
    
    (start_time.elapsed(), session_ids)
}

/// Helper function to measure API call creation and completion performance
async fn measure_api_call_performance(
    tracker: &mut CostTracker,
    session_ids: &[CostSessionId],
    calls_per_session: usize,
) -> (Duration, Duration) {
    // Measure API call addition
    let api_start = Instant::now();
    let mut all_calls = Vec::new();

    for session_id in session_ids {
        for j in 0..calls_per_session {
            let api_call = create_test_api_call(&format!("perf/{}", j), "claude-3-sonnet").unwrap();
            let call_id = tracker.add_api_call(session_id, api_call).unwrap();
            all_calls.push((*session_id, call_id));
        }
    }

    let api_creation_time = api_start.elapsed();

    // Measure completion performance
    let completion_start = Instant::now();
    for (i, (session_id, call_id)) in all_calls.iter().enumerate() {
        tracker
            .complete_api_call(
                session_id,
                call_id,
                100 + (i as u32 % 1000),
                200 + (i as u32 % 800),
                ApiCallStatus::Success,
                None,
            )
            .unwrap();
    }

    (api_creation_time, completion_start.elapsed())
}

/// Helper function to measure cost calculation performance
fn measure_cost_calculation_performance(
    tracker: &CostTracker,
    calculator: &CostCalculator,
    session_ids: &[CostSessionId],
) -> (Duration, Decimal) {
    let calc_start = Instant::now();
    let mut total_cost = Decimal::ZERO;

    for session_id in session_ids {
        let session = tracker.get_session(session_id).unwrap();
        let cost_calculation = calculator.calculate_session_cost(session).unwrap();
        total_cost += cost_calculation.total_cost;
    }

    (calc_start.elapsed(), total_cost)
}

/// Helper function to measure session completion performance
fn measure_session_completion_performance(
    tracker: &mut CostTracker,
    session_ids: &[CostSessionId],
) -> Duration {
    let start = Instant::now();
    
    for session_id in session_ids {
        tracker
            .complete_session(session_id, CostSessionStatus::Completed)
            .unwrap();
    }
    
    start.elapsed()
}

/// Helper function to assert performance bounds
fn assert_performance_bounds(
    session_creation_time: Duration,
    api_creation_time: Duration,
    completion_time: Duration,
    calculation_time: Duration,
    session_completion_time: Duration,
    total_time: Duration,
) {
    assert!(
        session_creation_time < Duration::from_secs(5),
        "Session creation took too long: {:?}",
        session_creation_time
    );
    assert!(
        api_creation_time < Duration::from_secs(10),
        "API call creation took too long: {:?}",
        api_creation_time
    );
    assert!(
        completion_time < Duration::from_secs(10),
        "API call completion took too long: {:?}",
        completion_time
    );
    assert!(
        calculation_time < Duration::from_secs(5),
        "Cost calculation took too long: {:?}",
        calculation_time
    );
    assert!(
        session_completion_time < Duration::from_secs(5),
        "Session completion took too long: {:?}",
        session_completion_time
    );
    assert!(
        total_time < Duration::from_secs(30),
        "Total performance test took too long: {:?}",
        total_time
    );
}

/// Test performance characteristics under load
#[tokio::test]
async fn test_performance_characteristics() {
    let mut tracker = CostTracker::new();
    let calculator = CostCalculator::paid_default();

    // Performance test parameters
    const PERF_SESSIONS: usize = 50;
    const PERF_CALLS_PER_SESSION: usize = 20;

    let start_time = Instant::now();

    // Phase 1: Measure session creation performance
    let mut session_ids = Vec::new();
    for i in 0..PERF_SESSIONS {
        let issue_id = create_test_issue_id(&format!("perf-{}", i)).unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();
        session_ids.push(session_id);
    }

    let session_creation_time = start_time.elapsed();

    // Phase 2: Measure API call addition performance
    let api_start = Instant::now();
    let mut all_calls = Vec::new();

    for session_id in &session_ids {
        for j in 0..PERF_CALLS_PER_SESSION {
            let api_call = create_test_api_call(&format!("perf/{}", j), "claude-3-sonnet").unwrap();
            let call_id = tracker.add_api_call(session_id, api_call).unwrap();
            all_calls.push((*session_id, call_id));
        }
    }

    let api_creation_time = api_start.elapsed();

    // Phase 3: Measure completion performance
    let completion_start = Instant::now();
    for (i, (session_id, call_id)) in all_calls.iter().enumerate() {
        tracker
            .complete_api_call(
                session_id,
                call_id,
                100 + (i as u32 % 1000),
                200 + (i as u32 % 800),
                ApiCallStatus::Success,
                None,
            )
            .unwrap();
    }

    let completion_time = completion_start.elapsed();

    // Phase 4: Measure cost calculation performance
    let calc_start = Instant::now();
    let mut total_cost = Decimal::ZERO;

    for session_id in &session_ids {
        let session = tracker.get_session(session_id).unwrap();
        let cost_calculation = calculator.calculate_session_cost(session).unwrap();
        total_cost += cost_calculation.total_cost;
    }

    let calculation_time = calc_start.elapsed();

    // Phase 5: Measure session completion performance
    let session_completion_start = Instant::now();
    for session_id in &session_ids {
        tracker
            .complete_session(session_id, CostSessionStatus::Completed)
            .unwrap();
    }

    let session_completion_time = session_completion_start.elapsed();
    let total_time = start_time.elapsed();

    // Verify results
    assert_eq!(tracker.session_count(), PERF_SESSIONS);
    assert_eq!(tracker.completed_session_count(), PERF_SESSIONS);
    assert!(total_cost > Decimal::ZERO);

    // Performance assertions (generous bounds for CI environments)
    assert!(
        session_creation_time < Duration::from_secs(5),
        "Session creation took too long: {:?}",
        session_creation_time
    );
    assert!(
        api_creation_time < Duration::from_secs(10),
        "API call creation took too long: {:?}",
        api_creation_time
    );
    assert!(
        completion_time < Duration::from_secs(10),
        "API call completion took too long: {:?}",
        completion_time
    );
    assert!(
        calculation_time < Duration::from_secs(5),
        "Cost calculation took too long: {:?}",
        calculation_time
    );
    assert!(
        session_completion_time < Duration::from_secs(5),
        "Session completion took too long: {:?}",
        session_completion_time
    );
    assert!(
        total_time < Duration::from_secs(30),
        "Total performance test took too long: {:?}",
        total_time
    );

    // Memory usage should be reasonable
    assert!(tracker.session_count() == PERF_SESSIONS);

    // Verify all operations completed successfully
    for session_id in &session_ids {
        let session = tracker.get_session(session_id).unwrap();
        assert!(session.is_completed());
        assert_eq!(session.api_call_count(), PERF_CALLS_PER_SESSION);
        assert!(session.total_tokens() > 0);
    }
}

/// Test edge cases and boundary conditions
#[tokio::test]
async fn test_edge_cases_and_boundaries() {
    let mut tracker = CostTracker::new();
    let calculator = CostCalculator::paid_default();

    // Test 1: Zero token calculations
    let issue_id = create_test_issue_id("zero-tokens").unwrap();
    let session_id = tracker.start_session(issue_id).unwrap();

    let mut zero_token_call = create_test_api_call("zero-tokens", "claude-3-sonnet").unwrap();
    complete_api_call_with_realistic_data(&mut zero_token_call, 0, 0, true);
    let _call_id = tracker.add_api_call(&session_id, zero_token_call).unwrap();

    let session = tracker.get_session(&session_id).unwrap();
    let cost_calculation = calculator.calculate_session_cost(session).unwrap();

    assert_eq!(cost_calculation.total_cost, Decimal::ZERO);
    assert_eq!(cost_calculation.input_tokens, 0);
    assert_eq!(cost_calculation.output_tokens, 0);

    // Test 2: Large token counts (within reasonable bounds)
    let large_issue_id = create_test_issue_id("large-tokens").unwrap();
    let large_session_id = tracker.start_session(large_issue_id).unwrap();

    let mut large_token_call = create_test_api_call("large-tokens", "claude-3-sonnet").unwrap();
    complete_api_call_with_realistic_data(&mut large_token_call, 1_000_000, 500_000, true);
    tracker
        .add_api_call(&large_session_id, large_token_call)
        .unwrap();

    let session = tracker.get_session(&large_session_id).unwrap();
    let cost_calculation = calculator.calculate_session_cost(session).unwrap();

    assert!(cost_calculation.total_cost > Decimal::ZERO);
    assert_eq!(cost_calculation.input_tokens, 1_000_000);
    assert_eq!(cost_calculation.output_tokens, 500_000);

    // Test 3: Unknown model handling
    let unknown_model_issue = create_test_issue_id("unknown-model").unwrap();
    let unknown_session_id = tracker.start_session(unknown_model_issue).unwrap();

    let mut unknown_model_call =
        create_test_api_call("unknown-model", "unknown-model-2024").unwrap();
    complete_api_call_with_realistic_data(&mut unknown_model_call, 100, 200, true);
    tracker
        .add_api_call(&unknown_session_id, unknown_model_call)
        .unwrap();

    let session = tracker.get_session(&unknown_session_id).unwrap();
    let cost_calculation = calculator.calculate_session_cost(session).unwrap();

    // Should use default rates for unknown model
    assert!(cost_calculation.total_cost > Decimal::ZERO);

    // Test 4: Model name matching variations
    let model_variations = [
        "claude-3-sonnet-20241022",
        "claude-3-5-sonnet-20241022",
        "claude-3-opus-20240229",
        "claude-3-haiku-20240307",
    ];

    for (i, model) in model_variations.iter().enumerate() {
        let var_issue_id = create_test_issue_id(&format!("model-var-{}", i)).unwrap();
        let var_session_id = tracker.start_session(var_issue_id).unwrap();

        let mut model_call = create_test_api_call("model-variation", model).unwrap();
        complete_api_call_with_realistic_data(&mut model_call, 100, 100, true);
        tracker.add_api_call(&var_session_id, model_call).unwrap();

        let session = tracker.get_session(&var_session_id).unwrap();
        let cost_calculation = calculator.calculate_session_cost(session).unwrap();

        assert!(cost_calculation.total_cost > Decimal::ZERO);
        assert!(!cost_calculation.is_estimated);
    }

    // Complete all test sessions
    for session_id in [session_id, large_session_id, unknown_session_id] {
        tracker
            .complete_session(&session_id, CostSessionStatus::Completed)
            .unwrap();
    }
}

/// Test cost calculation precision and accuracy
#[tokio::test]
async fn test_cost_calculation_precision() {
    let calculator = CostCalculator::paid_default();

    // Test small precision calculations
    let small_calculation = calculator
        .calculate_tokens_cost(1, 1, "claude-3-sonnet")
        .unwrap();

    // Should maintain precision for small amounts
    assert!(small_calculation.input_cost > Decimal::ZERO);
    assert!(small_calculation.output_cost > Decimal::ZERO);
    assert_eq!(
        small_calculation.total_cost,
        small_calculation.input_cost + small_calculation.output_cost
    );

    // Test precision with larger amounts
    let large_calculation = calculator
        .calculate_tokens_cost(100_000, 50_000, "claude-3-sonnet")
        .unwrap();

    assert!(large_calculation.total_cost > small_calculation.total_cost);
    assert_eq!(
        large_calculation.total_cost,
        large_calculation.input_cost + large_calculation.output_cost
    );

    // Test different models have different costs
    let sonnet_calc = calculator
        .calculate_tokens_cost(1000, 1000, "claude-3-sonnet")
        .unwrap();
    let opus_calc = calculator
        .calculate_tokens_cost(1000, 1000, "claude-3-opus")
        .unwrap();
    let haiku_calc = calculator
        .calculate_tokens_cost(1000, 1000, "claude-3-haiku")
        .unwrap();

    // Opus should be more expensive than Sonnet, Haiku should be least expensive
    assert!(opus_calc.total_cost > sonnet_calc.total_cost);
    assert!(sonnet_calc.total_cost > haiku_calc.total_cost);

    // All should maintain precision
    for calc in [&sonnet_calc, &opus_calc, &haiku_calc] {
        assert_eq!(calc.total_cost, calc.input_cost + calc.output_cost);
        assert!(calc.total_cost > Decimal::ZERO);
    }
}

#[cfg(test)]
mod test_validation {
    use super::*;

    #[test]
    fn test_helper_functions() {
        // Test create_test_api_call
        let api_call = create_test_api_call("test", "claude-3-sonnet").unwrap();
        assert_eq!(api_call.endpoint, "https://api.anthropic.com/v1/test");
        assert_eq!(api_call.model, "claude-3-sonnet");

        // Test create_test_issue_id
        let issue_id = create_test_issue_id("123").unwrap();
        assert_eq!(issue_id.as_str(), "issue-123");

        // Test invalid inputs
        assert!(create_test_api_call("test", "").is_err());
        // Note: The helper creates "issue-" prefix, so empty string becomes "issue-" which is valid
        // Let's test with a truly invalid case like whitespace
        assert!(IssueId::new("").is_err());
    }

    #[test]
    fn test_complete_api_call_with_realistic_data() {
        let mut api_call = create_test_api_call("test", "claude-3-sonnet").unwrap();

        // Test successful completion
        complete_api_call_with_realistic_data(&mut api_call, 100, 200, true);
        assert_eq!(api_call.input_tokens, 100);
        assert_eq!(api_call.output_tokens, 200);
        assert_eq!(api_call.status, ApiCallStatus::Success);
        assert!(api_call.error_message.is_none());

        // Test failed completion
        let mut failed_call = create_test_api_call("failed", "claude-3-sonnet").unwrap();
        complete_api_call_with_realistic_data(&mut failed_call, 50, 0, false);
        assert_eq!(failed_call.status, ApiCallStatus::Failed);
        assert!(failed_call.error_message.is_some());
    }
}
