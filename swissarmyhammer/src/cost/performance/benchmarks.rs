//! Comprehensive benchmarking framework for cost tracking performance
//!
//! This module provides benchmarking capabilities to measure API call overhead,
//! memory usage, storage performance, and aggregation speed.

use crate::cost::{ApiCall, ApiCallStatus, CostTracker, IssueId, TokenCounter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Result of a performance benchmark
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Number of operations performed
    pub operations: u64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Average operation time in microseconds
    pub avg_operation_time_us: f64,
    /// Memory usage in bytes (if measured)
    pub memory_usage_bytes: Option<usize>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(
        name: String,
        duration: Duration,
        operations: u64,
        memory_usage_bytes: Option<usize>,
    ) -> Self {
        let ops_per_second = if duration.as_secs_f64() > 0.0 {
            operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        let avg_operation_time_us = if operations > 0 {
            duration.as_micros() as f64 / operations as f64
        } else {
            0.0
        };

        Self {
            name,
            duration,
            operations,
            ops_per_second,
            avg_operation_time_us,
            memory_usage_bytes,
            metrics: HashMap::new(),
        }
    }

    /// Add a custom metric to the result
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.metrics.insert(name, value);
    }

    /// Check if the result meets performance target
    pub fn meets_target(&self, target_ms: u64) -> bool {
        self.avg_operation_time_us <= (target_ms as f64 * 1000.0)
    }
}

/// Individual performance benchmark
pub trait PerformanceBenchmark {
    /// Name of the benchmark
    fn name(&self) -> &str;

    /// Run the benchmark with specified number of operations
    fn run(&self, operations: u64) -> BenchmarkResult;

    /// Setup phase before benchmark (optional)
    fn setup(&self) {}

    /// Cleanup phase after benchmark (optional)
    fn cleanup(&self) {}
}

/// API call overhead benchmark
pub struct ApiCallOverheadBenchmark {
    /// Cost tracker instance for measuring API call overhead
    pub tracker: CostTracker,
    /// Token counter for processing API responses
    pub token_counter: TokenCounter,
}

impl Default for ApiCallOverheadBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiCallOverheadBenchmark {
    /// Create a new API call overhead benchmark
    pub fn new() -> Self {
        Self {
            tracker: CostTracker::new(),
            token_counter: TokenCounter::default(),
        }
    }

    fn create_test_api_call(&self) -> ApiCall {
        ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap()
    }
}

impl PerformanceBenchmark for ApiCallOverheadBenchmark {
    fn name(&self) -> &str {
        "api_call_overhead"
    }

    fn run(&self, operations: u64) -> BenchmarkResult {
        let mut tracker = self.tracker.clone();
        const MAX_CALLS_PER_SESSION: u64 = 400; // Leave some buffer below the 500 limit

        let start = Instant::now();

        let mut total_tokens = 0u64;
        let mut total_api_calls = 0u64;

        for batch in 0..((operations + MAX_CALLS_PER_SESSION - 1) / MAX_CALLS_PER_SESSION) {
            let issue_id = IssueId::new(format!("benchmark-issue-{}", batch)).unwrap();
            let session_id = tracker.start_session(issue_id).unwrap();

            let batch_start = batch * MAX_CALLS_PER_SESSION;
            let batch_end = std::cmp::min(batch_start + MAX_CALLS_PER_SESSION, operations);

            for i in batch_start..batch_end {
                // Simulate API call tracking overhead
                let mut api_call = self.create_test_api_call();
                api_call.complete(
                    100 + (i % 50) as u32,  // Variable input tokens
                    200 + (i % 100) as u32, // Variable output tokens
                    ApiCallStatus::Success,
                    None,
                );

                let _call_id = tracker.add_api_call(&session_id, api_call).unwrap();
            }

            let session = tracker.get_session(&session_id).unwrap();
            total_tokens += session.total_tokens() as u64;
            total_api_calls += session.api_call_count() as u64;
        }

        let duration = start.elapsed();

        let mut result = BenchmarkResult::new(
            self.name().to_string(),
            duration,
            operations,
            None, // TODO: Add memory measurement
        );

        result.add_metric("total_tokens".to_string(), total_tokens as f64);
        result.add_metric("api_calls".to_string(), total_api_calls as f64);

        result
    }
}

/// Token counting performance benchmark
pub struct TokenCountingBenchmark {
    /// Token counter for measuring token counting performance
    pub counter: TokenCounter,
}

impl Default for TokenCountingBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenCountingBenchmark {
    /// Create a new token counting benchmark
    pub fn new() -> Self {
        Self {
            counter: TokenCounter::default(),
        }
    }

    fn create_test_response(&self, size_multiplier: u64) -> String {
        let base_response = r#"{"id":"msg_123","content":[{"text":"Hello world"}],"usage":{"input_tokens":150,"output_tokens":25}}"#;
        // Simulate variable response sizes
        format!(
            "{}{}",
            base_response,
            "x".repeat((size_multiplier % 100) as usize)
        )
    }
}

impl PerformanceBenchmark for TokenCountingBenchmark {
    fn name(&self) -> &str {
        "token_counting"
    }

    fn run(&self, operations: u64) -> BenchmarkResult {
        // Create a new token counter since it doesn't implement Clone
        let mut counter = TokenCounter::default();
        let start = Instant::now();

        let mut total_tokens = 0u64;

        for i in 0..operations {
            let response = self.create_test_response(i);
            if let Ok(usage) =
                counter.count_from_response(&response, None, "claude-3-sonnet-20241022")
            {
                total_tokens += usage.total_tokens as u64;
            }
        }

        let duration = start.elapsed();

        let mut result = BenchmarkResult::new(self.name().to_string(), duration, operations, None);

        result.add_metric("total_tokens_processed".to_string(), total_tokens as f64);
        result.add_metric(
            "avg_tokens_per_response".to_string(),
            total_tokens as f64 / operations as f64,
        );

        result
    }
}

/// Memory usage benchmark
pub struct MemoryUsageBenchmark {
    /// Cost tracker instance for measuring memory usage
    pub tracker: CostTracker,
}

impl Default for MemoryUsageBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryUsageBenchmark {
    /// Create a new memory usage benchmark
    pub fn new() -> Self {
        Self {
            tracker: CostTracker::new(),
        }
    }

    fn estimate_memory_usage(&self, tracker: &CostTracker) -> usize {
        // Basic estimation - this could be more sophisticated with actual memory profiling
        let sessions = tracker.get_all_sessions();
        let mut total_size = std::mem::size_of_val(sessions);

        for (session_id, session) in sessions {
            total_size += std::mem::size_of_val(session_id);
            total_size += std::mem::size_of_val(session);
            total_size +=
                session.api_calls.capacity() * std::mem::size_of::<crate::cost::ApiCall>();
        }

        total_size
    }
}

impl PerformanceBenchmark for MemoryUsageBenchmark {
    fn name(&self) -> &str {
        "memory_usage"
    }

    fn run(&self, operations: u64) -> BenchmarkResult {
        let mut tracker = self.tracker.clone();
        let start = Instant::now();

        let initial_memory = self.estimate_memory_usage(&tracker);

        // Create multiple sessions and API calls to test memory usage
        // Keep API calls per session under limit
        const API_CALLS_PER_SESSION: u64 = 10;

        for i in 0..operations {
            let issue_id = IssueId::new(format!("benchmark-issue-{}", i)).unwrap();
            let session_id = tracker.start_session(issue_id).unwrap();

            // Add API calls per session (keeping under the 500 limit)
            for j in 0..API_CALLS_PER_SESSION {
                let mut api_call = ApiCall::new(
                    "https://api.anthropic.com/v1/messages",
                    "claude-3-sonnet-20241022",
                )
                .unwrap();
                api_call.complete(100 + j as u32, 200 + j as u32, ApiCallStatus::Success, None);
                tracker.add_api_call(&session_id, api_call).unwrap();
            }
        }

        let final_memory = self.estimate_memory_usage(&tracker);
        let duration = start.elapsed();

        let mut result = BenchmarkResult::new(
            self.name().to_string(),
            duration,
            operations,
            Some(final_memory),
        );

        result.add_metric("initial_memory_bytes".to_string(), initial_memory as f64);
        result.add_metric("final_memory_bytes".to_string(), final_memory as f64);
        result.add_metric(
            "memory_growth_bytes".to_string(),
            (final_memory - initial_memory) as f64,
        );
        result.add_metric(
            "memory_per_operation".to_string(),
            (final_memory - initial_memory) as f64 / operations as f64,
        );

        result
    }
}

/// Aggregation performance benchmark
pub struct AggregationBenchmark {
    /// Cost tracker instance for measuring aggregation performance
    pub tracker: CostTracker,
}

impl Default for AggregationBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl AggregationBenchmark {
    /// Create a new aggregation benchmark
    pub fn new() -> Self {
        Self {
            tracker: CostTracker::new(),
        }
    }

    fn setup_test_data(&self, tracker: &mut CostTracker, sessions: u64) {
        const API_CALLS_PER_SESSION: u32 = 20; // Keep under the 500 limit

        for i in 0..sessions {
            let issue_id = IssueId::new(format!("aggregate-issue-{}", i)).unwrap();
            let session_id = tracker.start_session(issue_id).unwrap();

            for j in 0..API_CALLS_PER_SESSION {
                let mut api_call = ApiCall::new(
                    "https://api.anthropic.com/v1/messages",
                    if j % 3 == 0 {
                        "claude-3-opus"
                    } else {
                        "claude-3-sonnet"
                    },
                )
                .unwrap();
                api_call.complete(100 + j * 10, 200 + j * 5, ApiCallStatus::Success, None);
                tracker.add_api_call(&session_id, api_call).unwrap();
            }

            tracker
                .complete_session(&session_id, crate::cost::CostSessionStatus::Completed)
                .unwrap();
        }
    }

    fn calculate_aggregated_metrics(&self, tracker: &CostTracker) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        let mut total_tokens = 0u64;
        let mut total_cost = 0.0f64;
        let mut session_count = 0usize;

        for session in tracker.get_all_sessions().values() {
            total_tokens += session.total_tokens() as u64;
            session_count += 1;

            // Simple cost estimation for aggregation benchmark
            total_cost += (session.total_input_tokens() as f64 * 0.000015)
                + (session.total_output_tokens() as f64 * 0.000075);
        }

        metrics.insert("total_tokens".to_string(), total_tokens as f64);
        metrics.insert("total_cost".to_string(), total_cost);
        metrics.insert("session_count".to_string(), session_count as f64);
        metrics.insert(
            "avg_tokens_per_session".to_string(),
            if session_count > 0 {
                total_tokens as f64 / session_count as f64
            } else {
                0.0
            },
        );

        metrics
    }
}

impl PerformanceBenchmark for AggregationBenchmark {
    fn name(&self) -> &str {
        "aggregation_performance"
    }

    fn run(&self, operations: u64) -> BenchmarkResult {
        let mut tracker = self.tracker.clone();

        // Setup test data
        self.setup_test_data(&mut tracker, operations);

        let start = Instant::now();

        // Perform aggregation operations
        let _metrics = self.calculate_aggregated_metrics(&tracker);

        let duration = start.elapsed();

        let mut result = BenchmarkResult::new(self.name().to_string(), duration, operations, None);

        result.add_metric("sessions_processed".to_string(), operations as f64);

        result
    }
}

/// Comprehensive benchmark suite
pub struct BenchmarkSuite {
    benchmarks: Vec<Box<dyn PerformanceBenchmark>>,
    target_ms: u64,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite with performance targets
    pub fn new(target_ms: u64) -> Self {
        let benchmarks: Vec<Box<dyn PerformanceBenchmark>> = vec![
            Box::new(ApiCallOverheadBenchmark::new()),
            Box::new(TokenCountingBenchmark::new()),
            Box::new(MemoryUsageBenchmark::new()),
            Box::new(AggregationBenchmark::new()),
        ];

        Self {
            benchmarks,
            target_ms,
        }
    }

    /// Run all benchmarks with specified operations count
    pub fn run_all(&self, operations: u64) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        for benchmark in &self.benchmarks {
            println!("Running benchmark: {}", benchmark.name());

            benchmark.setup();
            let result = benchmark.run(operations);
            benchmark.cleanup();

            println!("  Duration: {:?}", result.duration);
            println!("  Ops/sec: {:.2}", result.ops_per_second);
            println!("  Avg time: {:.2}μs", result.avg_operation_time_us);
            println!("  Meets target: {}", result.meets_target(self.target_ms));

            results.push(result);
        }

        results
    }

    /// Check if all benchmarks meet performance targets
    pub fn all_meet_targets(&self, results: &[BenchmarkResult]) -> bool {
        results.iter().all(|r| r.meets_target(self.target_ms))
    }

    /// Generate performance report
    pub fn generate_report(&self, results: &[BenchmarkResult]) -> String {
        let mut report = String::new();
        report.push_str("Performance Benchmark Report\n");
        report.push_str(&format!("Target: {}ms per operation\n\n", self.target_ms));

        for result in results {
            report.push_str(&format!("Benchmark: {}\n", result.name));
            report.push_str(&format!("  Operations: {}\n", result.operations));
            report.push_str(&format!("  Duration: {:?}\n", result.duration));
            report.push_str(&format!("  Ops/sec: {:.2}\n", result.ops_per_second));
            report.push_str(&format!(
                "  Avg time: {:.2}μs\n",
                result.avg_operation_time_us
            ));
            report.push_str(&format!(
                "  Target: {}\n",
                if result.meets_target(self.target_ms) {
                    "✓ PASS"
                } else {
                    "✗ FAIL"
                }
            ));

            if let Some(memory) = result.memory_usage_bytes {
                report.push_str(&format!("  Memory: {} bytes\n", memory));
            }

            for (metric, value) in &result.metrics {
                report.push_str(&format!("  {}: {:.2}\n", metric, value));
            }

            report.push('\n');
        }

        report.push_str(&format!(
            "Overall: {}\n",
            if self.all_meet_targets(results) {
                "✓ ALL PASS"
            } else {
                "✗ SOME FAIL"
            }
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult::new(
            "test_benchmark".to_string(),
            Duration::from_millis(100),
            1000,
            Some(4096),
        );

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.operations, 1000);
        assert_eq!(result.ops_per_second, 10000.0); // 1000 ops in 0.1 sec
        assert_eq!(result.avg_operation_time_us, 100.0); // 100ms = 100,000μs / 1000 ops = 100μs
        assert_eq!(result.memory_usage_bytes, Some(4096));
    }

    #[test]
    fn test_benchmark_result_meets_target() {
        let result =
            BenchmarkResult::new("test".to_string(), Duration::from_millis(40), 1000, None);

        assert!(result.meets_target(50)); // 40μs < 50ms (50,000μs)
        assert!(result.meets_target(30)); // 40μs < 30ms (30,000μs)
                                          // Test with a target that should fail
        assert!(!result.meets_target(0)); // 40μs > 0ms (0μs)
    }

    #[test]
    fn test_api_call_overhead_benchmark() {
        let benchmark = ApiCallOverheadBenchmark::new();
        assert_eq!(benchmark.name(), "api_call_overhead");

        let result = benchmark.run(100);
        assert_eq!(result.operations, 100);
        assert!(result.duration.as_micros() > 0); // Use microseconds for more precision
        assert!(result.metrics.contains_key("total_tokens"));
        assert!(result.metrics.contains_key("api_calls"));
    }

    #[test]
    fn test_token_counting_benchmark() {
        let benchmark = TokenCountingBenchmark::new();
        assert_eq!(benchmark.name(), "token_counting");

        let result = benchmark.run(50);
        assert_eq!(result.operations, 50);
        assert!(result.metrics.contains_key("total_tokens_processed"));
        assert!(result.metrics.contains_key("avg_tokens_per_response"));
    }

    #[test]
    fn test_memory_usage_benchmark() {
        let benchmark = MemoryUsageBenchmark::new();
        assert_eq!(benchmark.name(), "memory_usage");

        let result = benchmark.run(10); // Smaller number for memory test
        assert_eq!(result.operations, 10);
        assert!(result.memory_usage_bytes.is_some());
        assert!(result.metrics.contains_key("memory_growth_bytes"));
        assert!(result.metrics.contains_key("memory_per_operation"));
    }

    #[test]
    fn test_aggregation_benchmark() {
        let benchmark = AggregationBenchmark::new();
        assert_eq!(benchmark.name(), "aggregation_performance");

        let result = benchmark.run(5); // Small number for aggregation test
        assert_eq!(result.operations, 5);
        assert!(result.metrics.contains_key("sessions_processed"));
    }

    #[test]
    fn test_benchmark_suite() {
        let suite = BenchmarkSuite::new(50);
        let results = suite.run_all(10); // Small operations count for testing

        assert_eq!(results.len(), 4); // Should have all 4 benchmarks

        let report = suite.generate_report(&results);
        assert!(report.contains("Performance Benchmark Report"));
        assert!(report.contains("Target: 50ms"));
        assert!(report.contains("api_call_overhead"));
        assert!(report.contains("token_counting"));
        assert!(report.contains("memory_usage"));
        assert!(report.contains("aggregation_performance"));
    }

    #[test]
    fn test_benchmark_result_metrics() {
        let mut result =
            BenchmarkResult::new("test".to_string(), Duration::from_millis(100), 1000, None);

        result.add_metric("custom_metric".to_string(), 42.0);
        assert_eq!(result.metrics.get("custom_metric"), Some(&42.0));
    }
}
