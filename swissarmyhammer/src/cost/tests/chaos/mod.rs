//! Chaos engineering tests with failure injection
//!
//! This module provides chaos engineering tests that inject various types of failures
//! to validate system resilience, error handling, and recovery mechanisms under
//! adverse conditions. These tests ensure the system degrades gracefully.

// pub mod failure_injection;
// pub mod network_partitions;
// pub mod resource_exhaustion;
// pub mod random_failures;

use crate::cost::{
    tests::CostTrackingTestHarness,
    tracker::{ApiCall, ApiCallStatus, CostSessionStatus, IssueId},
};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};

/// Configuration for chaos testing scenarios
#[derive(Debug, Clone)]
pub struct ChaosTestConfig {
    /// Probability of operation failure (0.0 to 1.0)
    pub failure_rate: f64,
    /// Duration to run chaos tests
    pub chaos_duration: Duration,
    /// Number of concurrent chaos operations
    pub chaos_concurrency: usize,
    /// Minimum successful operations ratio to pass test
    pub min_success_rate: f64,
    /// Maximum acceptable error recovery time
    pub max_recovery_time: Duration,
}

impl Default for ChaosTestConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.3, // 30% failure rate
            chaos_duration: Duration::from_secs(60),
            chaos_concurrency: 8,
            min_success_rate: 0.4, // 40% operations should still succeed
            max_recovery_time: Duration::from_secs(5),
        }
    }
}

/// Failure injection coordinator for orchestrating chaos scenarios
#[derive(Debug)]
pub struct FailureInjector {
    network_failures: AtomicBool,
    storage_failures: AtomicBool,
    memory_pressure: AtomicBool,
    random_delays: AtomicBool,
    failure_count: AtomicU32,
    success_count: AtomicU32,
}

impl Default for FailureInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl FailureInjector {
    /// Creates a new failure injector with all failure types disabled
    pub fn new() -> Self {
        Self {
            network_failures: AtomicBool::new(false),
            storage_failures: AtomicBool::new(false),
            memory_pressure: AtomicBool::new(false),
            random_delays: AtomicBool::new(false),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
        }
    }

    /// Enables network failure injection for subsequent operations
    pub fn enable_network_failures(&self) {
        self.network_failures.store(true, Ordering::Relaxed);
    }

    /// Enables storage failure injection for subsequent operations
    pub fn enable_storage_failures(&self) {
        self.storage_failures.store(true, Ordering::Relaxed);
    }

    /// Enables memory pressure simulation for subsequent operations
    pub fn enable_memory_pressure(&self) {
        self.memory_pressure.store(true, Ordering::Relaxed);
    }

    /// Enables random delays for subsequent operations
    pub fn enable_random_delays(&self) {
        self.random_delays.store(true, Ordering::Relaxed);
    }

    /// Disables all failure injection types
    pub fn disable_all_failures(&self) {
        self.network_failures.store(false, Ordering::Relaxed);
        self.storage_failures.store(false, Ordering::Relaxed);
        self.memory_pressure.store(false, Ordering::Relaxed);
        self.random_delays.store(false, Ordering::Relaxed);
    }

    /// Attempts to inject a failure based on enabled failure types and operation ID
    pub async fn maybe_inject_failure(&self, operation_id: u32) -> Result<(), ChaosError> {
        // Inject random delays if enabled
        if self.random_delays.load(Ordering::Relaxed) {
            let delay_ms = (operation_id % 1000) + 50; // 50-1050ms delays
            sleep(Duration::from_millis(delay_ms as u64)).await;
        }

        // Inject network failures
        if self.network_failures.load(Ordering::Relaxed) && operation_id % 3 == 0 {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            return Err(ChaosError::NetworkFailure(
                "Simulated network partition".to_string(),
            ));
        }

        // Inject storage failures
        if self.storage_failures.load(Ordering::Relaxed) && operation_id % 5 == 0 {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            return Err(ChaosError::StorageFailure(
                "Simulated storage backend failure".to_string(),
            ));
        }

        // Inject memory pressure
        if self.memory_pressure.load(Ordering::Relaxed) && operation_id % 7 == 0 {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            return Err(ChaosError::ResourceExhaustion(
                "Simulated memory exhaustion".to_string(),
            ));
        }

        self.success_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Returns statistics about failure injection (successes, failures, success_rate)
    pub fn get_stats(&self) -> (u32, u32, f64) {
        let failures = self.failure_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        let total = failures + successes;
        let success_rate = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };
        (successes, failures, success_rate)
    }
}

/// Errors that can be injected during chaos testing
#[derive(Debug, thiserror::Error)]
pub enum ChaosError {
    /// Network-related failure simulation
    #[error("Network failure: {0}")]
    NetworkFailure(String),
    /// Storage-related failure simulation
    #[error("Storage failure: {0}")]
    StorageFailure(String),
    /// Resource exhaustion simulation (memory, CPU, etc.)
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    /// Random failure for unpredictable chaos testing
    #[error("Random chaos: {0}")]
    RandomFailure(String),
}

#[tokio::test]
async fn test_random_api_call_failures() {
    let harness = CostTrackingTestHarness::with_config()
        .with_failure_injection()
        .build();

    let config = ChaosTestConfig::default();
    let failure_injector = Arc::new(FailureInjector::new());
    failure_injector.enable_network_failures();
    failure_injector.enable_random_delays();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: random_api_failures");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let injector = failure_injector.clone();

        let start_time = Instant::now();
        let mut operation_id = 0u32;
        let mut successful_sessions = 0;
        let mut failed_sessions = 0;

        while start_time.elapsed() < config.chaos_duration {
            // Inject failures before operation
            if let Err(chaos_error) = injector.maybe_inject_failure(operation_id).await {
                match chaos_error {
                    ChaosError::NetworkFailure(_) => {
                        // Simulate network failure - skip this operation
                        operation_id += 1;
                        continue;
                    }
                    _ => {
                        warn!("Chaos injection: {:?}", chaos_error);
                    }
                }
            }

            // Attempt to create session and API calls despite chaos
            let issue_id = IssueId::new(format!("chaos-random-{}", operation_id));

            match issue_id {
                Ok(issue_id) => {
                    let session_result = {
                        let mut tracker = shared_tracker.lock().await;
                        tracker.start_session(issue_id)
                    };

                    match session_result {
                        Ok(session_id) => {
                            // Try to add API calls with potential failures
                            let mut call_successes = 0;
                            let mut _call_failures = 0;

                            for call_id in 0..3 {
                                // Inject more chaos for API calls
                                if let Err(_) =
                                    injector.maybe_inject_failure(operation_id + call_id).await
                                {
                                    _call_failures += 1;
                                    continue;
                                }

                                let api_call = ApiCall::new(
                                    format!(
                                        "https://api.anthropic.com/chaos/{}/{}",
                                        operation_id, call_id
                                    ),
                                    "claude-3-sonnet-20241022",
                                );

                                match api_call {
                                    Ok(mut api_call) => {
                                        // Simulate varying success/failure based on chaos
                                        let status = if call_id % 4 == 0 {
                                            ApiCallStatus::Failed
                                        } else {
                                            ApiCallStatus::Success
                                        };

                                        let (input_tokens, output_tokens) =
                                            if status == ApiCallStatus::Success {
                                                (100 + call_id * 50, 50 + call_id * 25)
                                            } else {
                                                (0, 0) // Failed calls have no tokens
                                            };

                                        api_call.complete(
                                            input_tokens,
                                            output_tokens,
                                            status,
                                            None,
                                        );

                                        let add_result = {
                                            let mut tracker = shared_tracker.lock().await;
                                            tracker.add_api_call(&session_id, api_call)
                                        };

                                        match add_result {
                                            Ok(_) => call_successes += 1,
                                            Err(_) => _call_failures += 1,
                                        }
                                    }
                                    Err(_) => _call_failures += 1,
                                }
                            }

                            // Complete session based on call results
                            let session_status = if call_successes > 0 {
                                successful_sessions += 1;
                                CostSessionStatus::Completed
                            } else {
                                failed_sessions += 1;
                                CostSessionStatus::Failed
                            };

                            let mut tracker = shared_tracker.lock().await;
                            let _ = tracker.complete_session(&session_id, session_status);
                        }
                        Err(_) => {
                            failed_sessions += 1;
                        }
                    }
                }
                Err(_) => {
                    failed_sessions += 1;
                }
            }

            operation_id += 1;

            // Brief pause between operations
            sleep(Duration::from_millis(50)).await;
        }

        let (injector_successes, injector_failures, injector_success_rate) = injector.get_stats();

        info!(
            "Chaos test results - Sessions: {} successful, {} failed",
            successful_sessions, failed_sessions
        );
        info!(
            "Injector stats: {} successes, {} failures, {:.2}% success rate",
            injector_successes,
            injector_failures,
            injector_success_rate * 100.0
        );

        let total_sessions = successful_sessions + failed_sessions;
        let session_success_rate = if total_sessions > 0 {
            successful_sessions as f64 / total_sessions as f64
        } else {
            0.0
        };

        // System should maintain minimum success rate despite chaos
        assert!(
            session_success_rate >= config.min_success_rate,
            "Session success rate {:.2}% should be above minimum {:.2}%",
            session_success_rate * 100.0,
            config.min_success_rate * 100.0
        );

        Ok((successful_sessions, failed_sessions, session_success_rate))
    };

    let result = timeout(config.chaos_duration + Duration::from_secs(30), test_future).await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(Ok(_)) => tracing::info!(
            "Test scenario 'random_api_failures' completed successfully in {:?}",
            duration
        ),
        Ok(Err(e)) => tracing::error!(
            "Test scenario 'random_api_failures' failed after {:?}: {}",
            duration,
            e
        ),
        Err(_) => tracing::error!(
            "Test scenario 'random_api_failures' timed out after {:?}",
            duration
        ),
    }

    assert!(
        result.is_ok(),
        "Random failure chaos test should complete: {:?}",
        result
    );
    let scenario_result: Result<(i32, i32, f64), Box<dyn std::error::Error + Send + Sync>> =
        result.unwrap();
    assert!(
        scenario_result.is_ok(),
        "Random failures should be handled gracefully: {:?}",
        scenario_result
    );
}

#[tokio::test]
async fn test_storage_backend_interruptions() {
    let harness = CostTrackingTestHarness::new();
    let _config = ChaosTestConfig::default();
    let failure_injector = Arc::new(FailureInjector::new());
    failure_injector.enable_storage_failures();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: storage_interruptions");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let injector = failure_injector.clone();

        let mut successful_operations = 0;
        let mut failed_operations = 0;

        // Create sessions and simulate storage failures
        for i in 0..50 {
            // Inject storage failures periodically
            if let Err(ChaosError::StorageFailure(_)) = injector.maybe_inject_failure(i).await {
                failed_operations += 1;
                continue; // Skip this operation due to storage failure
            }

            let issue_id = IssueId::new(format!("storage-chaos-{}", i)).unwrap();

            // Try to create session - may fail due to simulated storage issues
            let session_result = {
                let mut tracker = shared_tracker.lock().await;
                tracker.start_session(issue_id)
            };

            match session_result {
                Ok(session_id) => {
                    // Add API calls - may also fail
                    let api_calls = harness.api_call_generator.generate_multiple_calls(2);
                    let mut calls_added = 0;

                    for (call_idx, api_call) in api_calls.into_iter().enumerate() {
                        // More storage failure chances for API call additions
                        if let Err(_) = injector
                            .maybe_inject_failure(i * 10 + call_idx as u32)
                            .await
                        {
                            continue;
                        }

                        let add_result = {
                            let mut tracker = shared_tracker.lock().await;
                            tracker.add_api_call(&session_id, api_call)
                        };

                        if add_result.is_ok() {
                            calls_added += 1;
                        }
                    }

                    // Try to complete session
                    if calls_added > 0 {
                        let complete_result = {
                            let mut tracker = shared_tracker.lock().await;
                            tracker.complete_session(&session_id, CostSessionStatus::Completed)
                        };

                        if complete_result.is_ok() {
                            successful_operations += 1;
                        } else {
                            failed_operations += 1;
                        }
                    } else {
                        failed_operations += 1;
                    }
                }
                Err(_) => {
                    failed_operations += 1;
                }
            }

            // Small delay between operations
            sleep(Duration::from_millis(20)).await;
        }

        let total_ops = successful_operations + failed_operations;
        let success_rate = if total_ops > 0 {
            successful_operations as f64 / total_ops as f64
        } else {
            0.0
        };

        // Even with storage interruptions, some operations should succeed
        assert!(
            success_rate > 0.0,
            "Some operations should succeed despite storage failures"
        );

        // System should not crash or become unresponsive
        let final_tracker = shared_tracker.lock().await;
        let session_count = final_tracker.session_count();

        // Verify system is still responsive
        assert!(
            session_count >= successful_operations,
            "System should maintain session tracking despite storage failures"
        );

        info!(
            "Storage chaos results: {} successful, {} failed, {:.1}% success rate",
            successful_operations,
            failed_operations,
            success_rate * 100.0
        );

        Ok((successful_operations, failed_operations, success_rate))
    };

    let result: Result<_, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'storage_interruptions' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'storage_interruptions' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Storage interruption test should complete: {:?}",
        result
    );
}

#[tokio::test]
async fn test_concurrent_failures_under_load() {
    let harness = CostTrackingTestHarness::new();
    let config = ChaosTestConfig::default();
    let failure_injector = Arc::new(FailureInjector::new());
    failure_injector.enable_network_failures();
    failure_injector.enable_storage_failures();
    failure_injector.enable_memory_pressure();

    let shared_tracker = harness.get_shared_tracker();

    let result = timeout(Duration::from_secs(90), async {
        // Launch multiple concurrent workers with different failure patterns
        let num_workers = config.chaos_concurrency;
        let worker_handles: Vec<_> = (0..num_workers)
            .map(|worker_id| {
                let tracker_clone = shared_tracker.clone();
                let injector_clone = failure_injector.clone();
                let _api_gen = harness.api_call_generator.clone();
                let _calc = harness.calculator.clone();

                tokio::spawn(async move {
                    let mut worker_successes = 0;
                    let mut worker_failures = 0;

                    for op_id in 0..20 {
                        let global_op_id = worker_id as u32 * 100 + op_id;

                        // Each worker experiences different failure patterns
                        if let Err(chaos_error) =
                            injector_clone.maybe_inject_failure(global_op_id).await
                        {
                            match chaos_error {
                                ChaosError::NetworkFailure(_) => {
                                    worker_failures += 1;
                                    continue; // Network down, skip operation
                                }
                                ChaosError::ResourceExhaustion(_) => {
                                    // Memory pressure - try with smaller operation
                                    sleep(Duration::from_millis(100)).await; // Brief backoff
                                }
                                ChaosError::StorageFailure(_) => {
                                    worker_failures += 1;
                                    continue; // Storage unavailable
                                }
                                _ => {}
                            }
                        }

                        // Attempt operation despite potential failures
                        let issue_id =
                            IssueId::new(format!("concurrent-chaos-{}-{}", worker_id, op_id));

                        match issue_id {
                            Ok(issue_id) => {
                                let session_result = {
                                    let mut tracker = tracker_clone.lock().await;
                                    tracker.start_session(issue_id)
                                };

                                match session_result {
                                    Ok(session_id) => {
                                        // Try to add minimal API call under pressure
                                        let mut api_call = ApiCall::new(
                                            format!(
                                                "https://api.anthropic.com/concurrent/{}/{}",
                                                worker_id, op_id
                                            ),
                                            "claude-3-sonnet-20241022",
                                        )?;

                                        api_call.complete(50, 25, ApiCallStatus::Success, None);

                                        let add_result = {
                                            let mut tracker = tracker_clone.lock().await;
                                            tracker.add_api_call(&session_id, api_call)
                                        };

                                        if add_result.is_ok() {
                                            // Try to complete
                                            let complete_result = {
                                                let mut tracker = tracker_clone.lock().await;
                                                tracker.complete_session(
                                                    &session_id,
                                                    CostSessionStatus::Completed,
                                                )
                                            };

                                            if complete_result.is_ok() {
                                                worker_successes += 1;
                                            } else {
                                                worker_failures += 1;
                                            }
                                        } else {
                                            worker_failures += 1;
                                        }
                                    }
                                    Err(_) => worker_failures += 1,
                                }
                            }
                            Err(_) => worker_failures += 1,
                        }

                        // Rate limiting to avoid overwhelming the system
                        sleep(Duration::from_millis(50)).await;
                    }

                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
                        worker_successes,
                        worker_failures,
                    ))
                })
            })
            .collect();

        // Collect results from all workers
        let mut total_successes = 0;
        let mut total_failures = 0;

        for handle in worker_handles {
            match handle.await {
                Ok(Ok((successes, failures))) => {
                    total_successes += successes;
                    total_failures += failures;
                }
                Ok(Err(_)) | Err(_) => {
                    total_failures += 1; // Worker itself failed
                }
            }
        }

        let total_operations = total_successes + total_failures;
        let success_rate = if total_operations > 0 {
            total_successes as f64 / total_operations as f64
        } else {
            0.0
        };

        // Under concurrent chaos, system should maintain some level of success
        assert!(
            success_rate >= config.min_success_rate * 0.5, // Reduced expectation under extreme load
            "Success rate {:.2}% should be reasonable under concurrent failures",
            success_rate * 100.0
        );

        // System should remain responsive
        let final_tracker = shared_tracker.lock().await;
        let session_count = final_tracker.session_count();
        assert!(
            session_count > 0,
            "System should have processed some sessions despite chaos"
        );

        let (injector_successes, injector_failures, injector_rate) = failure_injector.get_stats();
        info!(
            "Concurrent chaos test: {} successes, {} failures ({:.1}% success rate)",
            total_successes,
            total_failures,
            success_rate * 100.0
        );
        info!(
            "Injector contributed: {} successes, {} failures ({:.1}% success rate)",
            injector_successes,
            injector_failures,
            injector_rate * 100.0
        );

        Ok::<(i32, i32, f64), Box<dyn std::error::Error + Send + Sync>>((
            total_successes,
            total_failures,
            success_rate,
        ))
    })
    .await;

    assert!(
        result.is_ok(),
        "Concurrent failure test should complete: {:?}",
        result
    );
    let (successes, _failures, _rate) = result.unwrap().unwrap();
    assert!(
        successes > 0,
        "Should have some successful operations under chaos"
    );
}

#[tokio::test]
async fn test_recovery_after_failures() {
    let harness = CostTrackingTestHarness::new();
    let config = ChaosTestConfig::default();
    let failure_injector = Arc::new(FailureInjector::new());

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: failure_recovery");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();

        // Phase 1: Inject severe failures
        failure_injector.enable_network_failures();
        failure_injector.enable_storage_failures();
        failure_injector.enable_memory_pressure();

        info!("Starting failure injection phase");
        let failure_start = Instant::now();
        let mut operations_during_failure = 0;
        let mut _failures_during_failure = 0;

        // Run operations under heavy failure conditions
        while failure_start.elapsed() < Duration::from_secs(15) {
            if let Err(_) = failure_injector
                .maybe_inject_failure(operations_during_failure)
                .await
            {
                _failures_during_failure += 1;
                operations_during_failure += 1;
                continue;
            }

            let issue_id = IssueId::new(format!("recovery-failure-{}", operations_during_failure));

            match issue_id {
                Ok(issue_id) => {
                    let session_result = {
                        let mut tracker = shared_tracker.lock().await;
                        tracker.start_session(issue_id)
                    };

                    if session_result.is_ok() {
                        // Minimal success during failure phase
                    } else {
                        _failures_during_failure += 1;
                    }
                }
                Err(_) => _failures_during_failure += 1,
            }

            operations_during_failure += 1;
            sleep(Duration::from_millis(100)).await;
        }

        // Phase 2: Stop failures and measure recovery
        info!("Starting recovery phase");
        failure_injector.disable_all_failures();
        let recovery_start = Instant::now();

        let mut recovery_successes = 0;
        let mut recovery_failures = 0;

        // Test system recovery
        for i in 0..20 {
            let issue_id = IssueId::new(format!("recovery-{}", i)).unwrap();
            let session_id = {
                let mut tracker = shared_tracker.lock().await;
                tracker.start_session(issue_id)
            };

            match session_id {
                Ok(session_id) => {
                    // Add API calls during recovery
                    let api_calls = harness.api_call_generator.generate_multiple_calls(2);
                    let mut calls_successful = 0;

                    for api_call in api_calls {
                        let add_result = {
                            let mut tracker = shared_tracker.lock().await;
                            tracker.add_api_call(&session_id, api_call)
                        };

                        if add_result.is_ok() {
                            calls_successful += 1;
                        }
                    }

                    if calls_successful > 0 {
                        let complete_result = {
                            let mut tracker = shared_tracker.lock().await;
                            tracker.complete_session(&session_id, CostSessionStatus::Completed)
                        };

                        if complete_result.is_ok() {
                            recovery_successes += 1;
                        } else {
                            recovery_failures += 1;
                        }
                    } else {
                        recovery_failures += 1;
                    }
                }
                Err(_) => recovery_failures += 1,
            }

            sleep(Duration::from_millis(50)).await;
        }

        let recovery_time = recovery_start.elapsed();
        let recovery_success_rate = if recovery_successes + recovery_failures > 0 {
            recovery_successes as f64 / (recovery_successes + recovery_failures) as f64
        } else {
            0.0
        };

        info!(
            "Recovery results: {} successes, {} failures in {:?} ({:.1}% success rate)",
            recovery_successes,
            recovery_failures,
            recovery_time,
            recovery_success_rate * 100.0
        );

        // Recovery should be relatively quick and effective
        assert!(
            recovery_time <= config.max_recovery_time,
            "Recovery time {:?} should be within acceptable limit {:?}",
            recovery_time,
            config.max_recovery_time
        );

        assert!(
            recovery_success_rate >= 0.8,
            "Recovery success rate {:.2}% should be high after failures stop",
            recovery_success_rate * 100.0
        );

        // System should be functional after recovery
        let final_tracker = shared_tracker.lock().await;
        let total_sessions = final_tracker.session_count();
        assert!(
            total_sessions >= recovery_successes,
            "System should track all successful recovery sessions"
        );

        Ok((
            operations_during_failure,
            recovery_successes,
            recovery_time,
            recovery_success_rate,
        ))
    };

    let result: Result<_, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'failure_recovery' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'failure_recovery' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Failure recovery test should complete: {:?}",
        result
    );
}

#[tokio::test]
async fn test_resource_limit_violations() {
    let harness = CostTrackingTestHarness::new();
    let failure_injector = Arc::new(FailureInjector::new());
    failure_injector.enable_memory_pressure();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: resource_limits");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();

        // Try to exceed system limits to test graceful handling
        let excessive_sessions = 200; // More than typical limits
        let mut successful_creations = 0;
        let mut resource_violations = 0;

        for i in 0..excessive_sessions {
            // Inject memory pressure periodically
            if let Err(ChaosError::ResourceExhaustion(_)) =
                failure_injector.maybe_inject_failure(i).await
            {
                resource_violations += 1;
                continue;
            }

            let issue_id = IssueId::new(format!("resource-limit-{}", i));

            match issue_id {
                Ok(issue_id) => {
                    let session_result = {
                        let mut tracker = shared_tracker.lock().await;
                        tracker.start_session(issue_id)
                    };

                    match session_result {
                        Ok(session_id) => {
                            // Add substantial API calls to increase memory pressure
                            for j in 0..5 {
                                let mut api_call = ApiCall::new(
                                    format!("https://api.anthropic.com/limit/{}/{}", i, j),
                                    "claude-3-sonnet-20241022",
                                )?;

                                // Large token counts to simulate memory usage
                                api_call.complete(1000, 500, ApiCallStatus::Success, None);

                                let mut tracker = shared_tracker.lock().await;
                                let add_result = tracker.add_api_call(&session_id, api_call);

                                if add_result.is_err() {
                                    resource_violations += 1;
                                    break;
                                }
                            }

                            successful_creations += 1;
                        }
                        Err(_) => resource_violations += 1,
                    }
                }
                Err(_) => resource_violations += 1,
            }

            // Brief pause to allow system to manage resources
            if i % 10 == 0 {
                sleep(Duration::from_millis(50)).await;
            }
        }

        // System should handle resource pressure gracefully
        assert!(
            successful_creations > 0,
            "Should create some sessions even under resource pressure"
        );
        assert!(
            resource_violations > 0,
            "Should encounter some resource violations as expected"
        );

        let violation_rate = resource_violations as f64 / excessive_sessions as f64;
        assert!(
            violation_rate < 0.9,
            "Violation rate {:.1}% should not be excessive",
            violation_rate * 100.0
        );

        // System should remain responsive
        let final_tracker = shared_tracker.lock().await;
        let session_count = final_tracker.session_count();
        assert!(
            session_count >= successful_creations / 2,
            "System should maintain reasonable number of sessions despite resource pressure"
        );

        info!(
            "Resource limit test: {} successful creations, {} violations ({:.1}% violation rate)",
            successful_creations,
            resource_violations,
            violation_rate * 100.0
        );

        Ok((successful_creations, resource_violations, violation_rate))
    };

    let result: Result<_, Box<dyn std::error::Error + Send + Sync>> = test_future.await;
    let duration = scenario_start.elapsed();

    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'resource_limits' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'resource_limits' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Resource limit test should complete: {:?}",
        result
    );
}
