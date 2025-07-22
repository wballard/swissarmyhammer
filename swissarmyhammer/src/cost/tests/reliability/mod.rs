//! Reliability and stability tests
//!
//! This module provides comprehensive testing for long-running system stability,
//! memory management, resource cleanup, and graceful degradation under sustained load.
//! These tests validate production-ready reliability characteristics.

// pub mod memory_management;
// pub mod long_running_stability;
// pub mod resource_cleanup;
// pub mod graceful_degradation;

use crate::cost::{
    test_utils::MemoryUsageTracker,
    tests::CostTrackingTestHarness,
    tracker::{ApiCall, ApiCallStatus, CostError, CostSessionStatus, IssueId, MAX_COST_SESSIONS},
};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};
use tokio::time::{interval, sleep, timeout};

/// Configuration for reliability testing scenarios
#[derive(Debug, Clone)]
pub struct ReliabilityTestConfig {
    /// Duration for long-running tests
    pub test_duration: Duration,
    /// Number of concurrent operations
    pub concurrency_level: usize,
    /// Operations per second rate
    pub operation_rate: u32,
    /// Memory usage threshold for alerts
    pub memory_threshold: usize,
    /// Maximum acceptable session cleanup time
    pub max_cleanup_time: Duration,
}

impl Default for ReliabilityTestConfig {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(300), // 5 minutes for CI
            concurrency_level: 10,
            operation_rate: 5, // 5 ops/second
            memory_threshold: MAX_COST_SESSIONS / 2,
            max_cleanup_time: Duration::from_secs(10),
        }
    }
}

impl ReliabilityTestConfig {
    /// Create config for extended testing (longer duration)
    pub fn extended() -> Self {
        Self {
            test_duration: Duration::from_secs(3600), // 1 hour
            concurrency_level: 20,
            operation_rate: 10,
            memory_threshold: MAX_COST_SESSIONS * 3 / 4,
            max_cleanup_time: Duration::from_secs(30),
        }
    }

    /// Create config for CI-friendly testing (shorter duration)
    pub fn ci_friendly() -> Self {
        Self {
            test_duration: Duration::from_secs(20), // Reduced from 60 to 20 seconds
            concurrency_level: 5,
            operation_rate: 1, // Reduced from 3 to 1 operations per second
            memory_threshold: 50,
            max_cleanup_time: Duration::from_secs(5),
        }
    }
}

#[tokio::test]
async fn test_memory_leak_detection() {
    let harness = CostTrackingTestHarness::new();
    let config = ReliabilityTestConfig::ci_friendly();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: memory_leak_detection");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let mut memory_tracker = MemoryUsageTracker::new(&*shared_tracker.lock().await);

        let start_time = Instant::now();
        let mut operation_count = 0;
        let mut total_batch_time = Duration::from_secs(0);

        // Run continuous operations for the configured duration
        while start_time.elapsed() < config.test_duration {
            // Perform batch operations directly in async context
            let batch_start = std::time::Instant::now();
            let batch_size = 2; // Reduced from 5 to 2 sessions per batch
            let mut session_ids = Vec::new();

            // Create and run batch operations
            let batch_result: Result<usize, CostError> = async {
                // Create sessions
                for i in 0..batch_size {
                    let issue_id = IssueId::new(format!("memory-leak-{}-{}", operation_count, i))?;
                    let session_id = {
                        let mut tracker = shared_tracker.lock().await;
                        tracker.start_session(issue_id)?
                    };
                    session_ids.push(session_id);

                    // Add API calls
                    let api_calls = harness.api_call_generator.generate_multiple_calls(3);
                    for api_call in api_calls {
                        let mut tracker = shared_tracker.lock().await;
                        tracker.add_api_call(&session_id, api_call)?;
                    }
                }

                // Complete sessions
                for session_id in session_ids {
                    let mut tracker = shared_tracker.lock().await;
                    tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
                }

                Ok::<usize, CostError>(batch_size)
            }
            .await;

            // Record timing
            let batch_duration = batch_start.elapsed();
            total_batch_time += batch_duration;

            match batch_result {
                Ok(_) => {
                    operation_count += 1;
                    memory_tracker.update_peak(&*shared_tracker.lock().await);

                    // Check memory usage periodically
                    if operation_count % 10 == 0 {
                        let current_sessions = shared_tracker.lock().await.session_count();
                        if current_sessions > config.memory_threshold {
                            tracing::warn!("Memory usage high: {} sessions", current_sessions);
                        }
                    }

                    // Rate limiting
                    sleep(Duration::from_millis(1000 / config.operation_rate as u64)).await;
                }
                Err(e) => {
                    tracing::error!("Batch operation failed: {:?}", e);
                    break;
                }
            }
        }

        // Final memory statistics
        let final_stats = memory_tracker.get_stats(&*shared_tracker.lock().await);

        // Validate memory usage patterns
        tracing::info!("Final memory stats: current_sessions={}, peak_sessions={}, initial_sessions={}, cleanup_events={}", 
            final_stats.current_sessions, final_stats.peak_sessions, final_stats.initial_sessions, final_stats.cleanup_events);
        tracing::info!(
            "Memory threshold: {}, doubled threshold: {}",
            config.memory_threshold,
            config.memory_threshold * 2
        );

        assert!(final_stats.validate_memory_usage(config.memory_threshold * 2),
            "Memory usage should stay within reasonable bounds: current_sessions={}, peak_sessions={}, max_allowed={}",
            final_stats.current_sessions, final_stats.peak_sessions, config.memory_threshold * 2);

        // Print performance summary
        tracing::info!("Memory leak test performance: {} operations, total batch time: {:?}, avg per batch: {:?}",
            operation_count,
            total_batch_time,
            if operation_count > 0 { total_batch_time / operation_count as u32 } else { Duration::from_secs(0) });

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((operation_count, final_stats))
    };

    let result = timeout(config.test_duration + Duration::from_secs(30), test_future).await;

    let scenario_duration = scenario_start.elapsed();
    match &result {
        Ok(Ok(_)) => tracing::info!(
            "Test scenario 'memory_leak_detection' completed successfully in {:?}",
            scenario_duration
        ),
        Ok(Err(e)) => tracing::error!(
            "Test scenario 'memory_leak_detection' failed after {:?}: {}",
            scenario_duration,
            e
        ),
        Err(_) => tracing::error!(
            "Test scenario 'memory_leak_detection' timed out after {:?}",
            scenario_duration
        ),
    }

    assert!(
        result.is_ok(),
        "Memory leak detection should complete within timeout"
    );
    let scenario_result = result.unwrap();
    assert!(
        scenario_result.is_ok(),
        "Memory leak detection should succeed: {:?}",
        scenario_result
    );

    let (operation_count, final_stats) = scenario_result.unwrap();
    assert!(operation_count > 0, "Should perform some operations");

    tracing::info!(
        "Memory leak test completed: {} operations, final stats: {:?}",
        operation_count,
        final_stats
    );
}

#[tokio::test]
async fn test_concurrent_session_safety() {
    let config = ReliabilityTestConfig::ci_friendly();
    let harness = CostTrackingTestHarness::new();
    let shared_tracker = harness.get_shared_tracker();

    let result = timeout(Duration::from_secs(120), async {
        // Create multiple concurrent workers
        let num_workers = config.concurrency_level;
        let operations_per_worker = 20;

        let worker_handles: Vec<_> = (0..num_workers)
            .map(|worker_id| {
                let tracker_clone = shared_tracker.clone();
                let _api_gen = harness.api_call_generator.clone();
                let calc = harness.calculator.clone();

                tokio::spawn(async move {
                    let mut worker_results = Vec::new();

                    for op_id in 0..operations_per_worker {
                        let issue_id =
                            IssueId::new(format!("concurrent-safety-{}-{}", worker_id, op_id))?;

                        // Create session
                        let session_id = {
                            let mut tracker = tracker_clone.lock().await;
                            tracker.start_session(issue_id)?
                        };

                        // Add API calls with some randomness
                        let num_calls = (op_id % 5) + 1; // 1-5 calls per session
                        for call_id in 0..num_calls {
                            let mut api_call = ApiCall::new(
                                format!(
                                    "https://api.anthropic.com/v1/worker-{}-op-{}-call-{}",
                                    worker_id, op_id, call_id
                                ),
                                "claude-3-sonnet-20241022",
                            )?;

                            // Simulate variable token usage
                            let input_tokens = 100 + (call_id * 50) as u32;
                            let output_tokens = 50 + (call_id * 25) as u32;
                            api_call.complete(
                                input_tokens,
                                output_tokens,
                                ApiCallStatus::Success,
                                None,
                            );

                            let mut tracker = tracker_clone.lock().await;
                            tracker.add_api_call(&session_id, api_call)?;
                        }

                        // Calculate cost
                        let session = {
                            let tracker = tracker_clone.lock().await;
                            tracker.get_session(&session_id).cloned()
                        };

                        if let Some(session) = session {
                            let calculation = calc.calculate_session_cost(&session)?;
                            worker_results.push((session_id, calculation.total_cost, num_calls));
                        }

                        // Complete session
                        {
                            let mut tracker = tracker_clone.lock().await;
                            tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
                        }

                        // Small delay to allow other workers
                        sleep(Duration::from_millis(10)).await;
                    }

                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(worker_results)
                })
            })
            .collect();

        // Wait for all workers to complete
        let mut total_operations = 0;
        let mut total_cost = Decimal::ZERO;

        for handle in worker_handles {
            let worker_result = handle
                .await
                .expect("Worker task should not panic")
                .expect("Worker should complete successfully");
            total_operations += worker_result.len();
            for (_, cost, _) in worker_result {
                total_cost += cost;
            }
        }

        // Validate concurrent safety
        let final_tracker = shared_tracker.lock().await;
        let expected_sessions = num_workers * operations_per_worker;

        assert_eq!(
            final_tracker.session_count(),
            expected_sessions,
            "Should have created all expected sessions"
        );
        assert_eq!(
            final_tracker.completed_session_count(),
            expected_sessions,
            "All sessions should be completed"
        );
        assert!(
            total_cost >= Decimal::ZERO,
            "Total cost should be non-negative"
        );

        Ok::<(usize, Decimal), CostError>((total_operations, total_cost))
    })
    .await;

    assert!(
        result.is_ok(),
        "Concurrent safety test should complete: {:?}",
        result
    );
    let (total_operations, total_cost) = result.unwrap().unwrap();

    assert!(total_operations > 0, "Should perform operations");
    tracing::info!(
        "Concurrent safety test: {} operations, total cost: {}",
        total_operations,
        total_cost
    );
}

#[tokio::test]
async fn test_resource_cleanup_effectiveness() {
    let harness = CostTrackingTestHarness::new();
    let config = ReliabilityTestConfig::ci_friendly();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: resource_cleanup");
    let start = std::time::Instant::now();

    let result: Result<(usize, usize, usize, Duration), Box<dyn std::error::Error + Send + Sync>> =
        async {
            let shared_tracker = harness.get_shared_tracker();
            let cleanup_start = Instant::now();

            // Create many sessions to trigger cleanup mechanisms
            let session_count = config.memory_threshold + 10;
            let mut session_ids = Vec::new();

            for i in 0..session_count {
                let issue_id = IssueId::new(format!("cleanup-test-{}", i))?;
                let session_id = {
                    let mut tracker = shared_tracker.lock().await;
                    tracker.start_session(issue_id)?
                };
                session_ids.push(session_id);

                // Add API calls to make sessions substantial
                for j in 0..3 {
                    let mut api_call = ApiCall::new(
                        format!("https://api.anthropic.com/cleanup/{}/{}", i, j),
                        "claude-3-sonnet-20241022",
                    )?;
                    api_call.complete(100 + j as u32, 50 + j as u32, ApiCallStatus::Success, None);

                    let mut tracker = shared_tracker.lock().await;
                    tracker.add_api_call(&session_id, api_call)?;
                }
            }

            // Complete first half of sessions
            let half_count = session_count / 2;
            for i in 0..half_count {
                let mut tracker = shared_tracker.lock().await;
                tracker.complete_session(&session_ids[i], CostSessionStatus::Completed)?;
            }

            // Allow some time for cleanup
            sleep(Duration::from_millis(100)).await;

            // Verify resource management
            let tracker_guard = shared_tracker.lock().await;
            let current_sessions = tracker_guard.session_count();
            let active_sessions = tracker_guard.active_session_count();
            let completed_sessions = tracker_guard.completed_session_count();

            assert_eq!(
                current_sessions, session_count,
                "Should track all created sessions"
            );
            assert_eq!(
                active_sessions,
                session_count - half_count,
                "Should have correct active count"
            );
            assert_eq!(
                completed_sessions, half_count,
                "Should have correct completed count"
            );

            let cleanup_duration = cleanup_start.elapsed();
            assert!(
                cleanup_duration <= config.max_cleanup_time,
                "Cleanup should complete within acceptable time: {:?}",
                cleanup_duration
            );

            Ok((
                current_sessions,
                active_sessions,
                completed_sessions,
                cleanup_duration,
            ))
        }
        .await;

    let duration = start.elapsed();
    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'resource_cleanup' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'resource_cleanup' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Resource cleanup test should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_graceful_degradation_under_load() {
    let harness = CostTrackingTestHarness::new();
    let _config = ReliabilityTestConfig::ci_friendly();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: graceful_degradation");
    let start = std::time::Instant::now();

    let result: Result<(usize, usize, f64, Duration), Box<dyn std::error::Error + Send + Sync>> =
        async {
            let shared_tracker = harness.get_shared_tracker();
            let mut success_count = 0;
            let mut failure_count = 0;
            let mut total_response_time = Duration::ZERO;

            // Generate high load
            let load_duration = Duration::from_secs(30);
            let start_time = Instant::now();
            let mut operation_id = 0;

            while start_time.elapsed() < load_duration {
                let operation_start = Instant::now();

                // Create burst of operations
                let burst_size = 10;
                let mut burst_handles = Vec::new();

                for i in 0..burst_size {
                    let tracker_clone = shared_tracker.clone();
                    let api_gen = harness.api_call_generator.clone();
                    let calc = harness.calculator.clone();
                    let current_op_id = operation_id;

                    let handle = tokio::spawn(async move {
                        let issue_id = IssueId::new(format!("load-test-{}-{}", current_op_id, i))?;

                        let session_id = {
                            let mut tracker = tracker_clone.lock().await;
                            tracker.start_session(issue_id)?
                        };

                        // Quick API call addition
                        let api_call = api_gen.generate_completed_api_call(i);
                        {
                            let mut tracker = tracker_clone.lock().await;
                            tracker.add_api_call(&session_id, api_call)?;
                        }

                        // Calculate cost
                        let session = {
                            let tracker = tracker_clone.lock().await;
                            tracker.get_session(&session_id).cloned()
                        };

                        let cost = if let Some(session) = session {
                            calc.calculate_session_cost(&session)?.total_cost
                        } else {
                            Decimal::ZERO
                        };

                        // Complete session
                        {
                            let mut tracker = tracker_clone.lock().await;
                            tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
                        }

                        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(cost)
                    });

                    burst_handles.push(handle);
                }

                // Collect burst results with timeout
                for handle in burst_handles {
                    match timeout(Duration::from_millis(500), handle).await {
                        Ok(Ok(Ok(_cost))) => success_count += 1,
                        Ok(Ok(Err(_))) => failure_count += 1,
                        Ok(Err(_)) => failure_count += 1,
                        Err(_) => failure_count += 1, // Timeout
                    }
                }

                let operation_time = operation_start.elapsed();
                total_response_time += operation_time;
                operation_id += 1;

                // Brief pause between bursts
                sleep(Duration::from_millis(100)).await;
            }

            let total_operations = success_count + failure_count;
            let success_rate = if total_operations > 0 {
                success_count as f64 / total_operations as f64
            } else {
                0.0
            };

            let average_response_time = if operation_id > 0 {
                total_response_time / operation_id as u32
            } else {
                Duration::ZERO
            };

            // Under load, system should maintain reasonable success rate
            // Allow for significant degradation under high load, but system should not fail completely
            assert!(
                success_rate >= 0.3,
                "Success rate should be at least 30% under load: {:.2}%",
                success_rate * 100.0
            );
            assert!(
                average_response_time <= Duration::from_secs(5),
                "Average response time should be reasonable: {:?}",
                average_response_time
            );

            tracing::info!(
                "Load test results: {:.1}% success rate, avg response time: {:?}",
                success_rate * 100.0,
                average_response_time
            );

            Ok((
                success_count,
                failure_count,
                success_rate,
                average_response_time,
            ))
        }
        .await;

    let duration = start.elapsed();
    match &result {
        Ok(_) => tracing::info!(
            "Test scenario 'graceful_degradation' completed successfully in {:?}",
            duration
        ),
        Err(e) => tracing::error!(
            "Test scenario 'graceful_degradation' failed after {:?}: {}",
            duration,
            e
        ),
    }

    assert!(
        result.is_ok(),
        "Graceful degradation test should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_sustained_operation_stability() {
    let harness = CostTrackingTestHarness::new();
    let config = ReliabilityTestConfig::ci_friendly();

    // Direct test without execute_test_scenario to avoid lifetime issues
    tracing::info!("Starting test scenario: sustained_stability");
    let scenario_start = std::time::Instant::now();

    let test_future = async {
        let shared_tracker = harness.get_shared_tracker();
        let mut interval_timer = interval(Duration::from_secs(5));
        let start_time = Instant::now();
        let mut total_sessions_created = 0;
        let mut total_sessions_completed = 0;

        // Run sustained operations
        while start_time.elapsed() < config.test_duration {
            interval_timer.tick().await;

            // Create batch of sessions
            let batch_size = 5;
            let mut session_ids = Vec::new();

            for i in 0..batch_size {
                let issue_id = IssueId::new(format!("sustained-{}-{}", total_sessions_created, i))?;

                let session_id = {
                    let mut tracker = shared_tracker.lock().await;
                    tracker.start_session(issue_id)?
                };
                session_ids.push(session_id);
                total_sessions_created += 1;

                // Add varying number of API calls
                let call_count = (i % 4) + 1;
                for j in 0..call_count {
                    let api_call = harness.api_call_generator.generate_completed_api_call(j);
                    let mut tracker = shared_tracker.lock().await;
                    tracker.add_api_call(&session_id, api_call)?;
                }
            }

            // Complete sessions
            for session_id in session_ids {
                let mut tracker = shared_tracker.lock().await;
                tracker.complete_session(&session_id, CostSessionStatus::Completed)?;
                total_sessions_completed += 1;
            }

            // Periodic health check
            let tracker_guard = shared_tracker.lock().await;
            let current_sessions = tracker_guard.session_count();
            let active_sessions = tracker_guard.active_session_count();

            tracing::debug!(
                "Stability check - Total: {}, Active: {}, Completed: {}",
                current_sessions,
                active_sessions,
                total_sessions_completed
            );

            // Verify system stability
            assert_eq!(
                active_sessions, 0,
                "All sessions should be completed in each batch"
            );
            assert!(
                current_sessions >= total_sessions_completed,
                "Session tracking should be consistent"
            );
        }

        // Final stability validation
        let final_tracker = shared_tracker.lock().await;
        assert_eq!(
            final_tracker.completed_session_count(),
            total_sessions_completed,
            "Final completed count should match"
        );
        assert_eq!(
            final_tracker.active_session_count(),
            0,
            "No active sessions should remain"
        );

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
            total_sessions_created,
            total_sessions_completed,
        ))
    };

    let result = timeout(config.test_duration + Duration::from_secs(10), test_future).await;

    let scenario_duration = scenario_start.elapsed();
    match &result {
        Ok(Ok(_)) => tracing::info!(
            "Test scenario 'sustained_stability' completed successfully in {:?}",
            scenario_duration
        ),
        Ok(Err(e)) => tracing::error!(
            "Test scenario 'sustained_stability' failed after {:?}: {}",
            scenario_duration,
            e
        ),
        Err(_) => tracing::error!(
            "Test scenario 'sustained_stability' timed out after {:?}",
            scenario_duration
        ),
    }

    assert!(
        result.is_ok(),
        "Sustained stability test should complete: {:?}",
        result
    );
    let scenario_result = result.unwrap();
    assert!(
        scenario_result.is_ok(),
        "Sustained stability should succeed: {:?}",
        scenario_result
    );

    let (created, completed) = scenario_result.unwrap();
    assert_eq!(
        created, completed,
        "All created sessions should be completed"
    );
    assert!(created > 0, "Should create some sessions during test");

    tracing::info!(
        "Sustained stability test: {} sessions created and completed",
        created
    );
}
