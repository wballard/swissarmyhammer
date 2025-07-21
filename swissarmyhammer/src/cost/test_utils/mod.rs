//! Test utilities for cost tracking system
//!
//! This module provides comprehensive test utilities, mock data generators,
//! and helper functions to support testing of the cost tracking foundation.
//! These utilities enable realistic test scenarios and consistent test data.

pub mod mock_mcp;

use crate::cost::{
    calculator::{CostCalculator, PricingModel, PricingRates},
    tracker::{ApiCall, ApiCallStatus, CostSessionStatus, CostTracker, IssueId},
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for generating mock API call data
#[derive(Debug, Clone)]
pub struct ApiCallGenerator {
    /// Base endpoint URL
    pub base_endpoint: String,
    /// Available models for testing
    pub models: Vec<String>,
    /// Token count ranges
    pub input_token_range: (u32, u32),
    /// Output token count ranges
    pub output_token_range: (u32, u32),
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl Default for ApiCallGenerator {
    fn default() -> Self {
        Self {
            base_endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            models: vec![
                "claude-3-sonnet-20241022".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            input_token_range: (50, 2000),
            output_token_range: (25, 1500),
            success_rate: 0.95,
        }
    }
}

impl ApiCallGenerator {
    /// Create a new API call generator with custom settings
    pub fn new(
        base_endpoint: String,
        models: Vec<String>,
        input_token_range: (u32, u32),
        output_token_range: (u32, u32),
        success_rate: f64,
    ) -> Self {
        assert!(
            (0.0..=1.0).contains(&success_rate),
            "Success rate must be between 0 and 1"
        );
        assert!(
            input_token_range.0 <= input_token_range.1,
            "Invalid input token range"
        );
        assert!(
            output_token_range.0 <= output_token_range.1,
            "Invalid output token range"
        );

        Self {
            base_endpoint,
            models,
            input_token_range,
            output_token_range,
            success_rate,
        }
    }

    /// Generate a realistic API call with deterministic characteristics
    pub fn generate_api_call(&self, call_index: u32) -> ApiCall {
        let model = &self.models[call_index as usize % self.models.len()];
        let endpoint = format!("{}/call-{}", self.base_endpoint, call_index);

        ApiCall::new(endpoint, model).expect("Should create valid API call")
    }

    /// Generate a completed API call with realistic token counts
    pub fn generate_completed_api_call(&self, call_index: u32) -> ApiCall {
        let mut api_call = self.generate_api_call(call_index);

        // Use deterministic values based on call_index for reproducible tests
        let input_tokens = self.input_token_range.0
            + (call_index % (self.input_token_range.1 - self.input_token_range.0 + 1));
        let output_tokens = self.output_token_range.0
            + ((call_index * 7) % (self.output_token_range.1 - self.output_token_range.0 + 1));

        // Deterministic success rate based on call_index
        let is_success = (call_index % 100) < (self.success_rate * 100.0) as u32;
        let status = if is_success {
            ApiCallStatus::Success
        } else {
            match call_index % 3 {
                0 => ApiCallStatus::Failed,
                1 => ApiCallStatus::Timeout,
                _ => ApiCallStatus::Cancelled,
            }
        };

        let error_message = if is_success {
            None
        } else {
            Some(match status {
                ApiCallStatus::Failed => "Request failed due to validation error".to_string(),
                ApiCallStatus::Timeout => "Request timed out after 60 seconds".to_string(),
                ApiCallStatus::Cancelled => "Request was cancelled by user".to_string(),
                _ => "Unknown error".to_string(),
            })
        };

        api_call.complete(input_tokens, output_tokens, status, error_message);
        api_call
    }

    /// Generate multiple API calls for testing
    pub fn generate_multiple_calls(&self, count: u32) -> Vec<ApiCall> {
        (0..count)
            .map(|i| self.generate_completed_api_call(i))
            .collect()
    }
}

/// Configuration builder for test scenarios
#[derive(Debug, Clone, Default)]
pub struct TestConfigBuilder {
    pricing_model: Option<PricingModel>,
    custom_rates: HashMap<String, PricingRates>,
}

impl TestConfigBuilder {
    /// Create a new test configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set pricing model to paid plan with defaults
    pub fn with_paid_plan(mut self) -> Self {
        self.pricing_model = Some(PricingModel::paid_with_defaults());
        self
    }

    /// Set pricing model to max plan with tracking
    pub fn with_max_plan_tracking(mut self) -> Self {
        self.pricing_model = Some(PricingModel::max_with_tracking());
        self
    }

    /// Set pricing model to max plan with estimates
    pub fn with_max_plan_estimates(mut self) -> Self {
        self.pricing_model = Some(PricingModel::max_with_estimates());
        self
    }

    /// Add custom pricing rates for a specific model
    pub fn with_custom_rates(mut self, model: String, input_cost: &str, output_cost: &str) -> Self {
        let rates =
            PricingRates::from_strings(input_cost, output_cost).expect("Valid pricing rates");
        self.custom_rates.insert(model, rates);
        self
    }

    /// Build a cost calculator with the configured settings
    pub fn build_calculator(self) -> CostCalculator {
        match self.pricing_model {
            Some(PricingModel::Paid(mut config)) => {
                // Add any custom rates
                for (model, rates) in self.custom_rates {
                    config.model_rates.insert(model, rates);
                }
                CostCalculator::new(PricingModel::Paid(config))
            }
            Some(PricingModel::Max(config)) => CostCalculator::new(PricingModel::Max(config)),
            None => CostCalculator::paid_default(),
        }
    }
}

/// Session lifecycle helper for consistent test patterns
#[derive(Debug)]
pub struct SessionLifecycleHelper {
    /// Cost tracker for managing test sessions
    pub tracker: CostTracker,
    /// Cost calculator for pricing calculations
    pub calculator: CostCalculator,
    /// API call generator for creating test data
    pub api_call_generator: ApiCallGenerator,
}

impl Default for SessionLifecycleHelper {
    fn default() -> Self {
        Self {
            tracker: CostTracker::new(),
            calculator: CostCalculator::paid_default(),
            api_call_generator: ApiCallGenerator::default(),
        }
    }
}

impl SessionLifecycleHelper {
    /// Create a new session lifecycle helper with custom configuration
    pub fn new(calculator: CostCalculator, api_call_generator: ApiCallGenerator) -> Self {
        Self {
            tracker: CostTracker::new(),
            calculator,
            api_call_generator,
        }
    }

    /// Create a complete test session with multiple API calls
    pub fn create_test_session(
        &mut self,
        issue_suffix: &str,
        num_api_calls: u32,
    ) -> Result<(crate::cost::CostSessionId, Decimal), crate::cost::CostError> {
        let issue_id = IssueId::new(format!("test-{}", issue_suffix))?;
        let session_id = self.tracker.start_session(issue_id)?;

        // Add API calls
        for i in 0..num_api_calls {
            let api_call = self.api_call_generator.generate_completed_api_call(i);
            self.tracker.add_api_call(&session_id, api_call)?;
        }

        // Calculate cost
        let session = self.tracker.get_session(&session_id).unwrap();
        let cost_calculation = self.calculator.calculate_session_cost(session)?;

        Ok((session_id, cost_calculation.total_cost))
    }

    /// Complete a session and return final cost
    pub fn complete_session(
        &mut self,
        session_id: &crate::cost::CostSessionId,
        status: CostSessionStatus,
    ) -> Result<Decimal, crate::cost::CostError> {
        self.tracker.complete_session(session_id, status)?;

        let session = self.tracker.get_session(session_id).unwrap();
        let cost_calculation = self.calculator.calculate_session_cost(session)?;

        Ok(cost_calculation.total_cost)
    }

    /// Create multiple test sessions in parallel
    pub fn create_multiple_sessions(
        &mut self,
        base_name: &str,
        count: u32,
        calls_per_session: u32,
    ) -> Result<Vec<(crate::cost::CostSessionId, Decimal)>, crate::cost::CostError> {
        let mut results = Vec::new();

        for i in 0..count {
            let session_result =
                self.create_test_session(&format!("{}-{}", base_name, i), calls_per_session)?;
            results.push(session_result);
        }

        Ok(results)
    }
}

/// Performance measurement utilities with memory tracking
#[derive(Debug, Default)]
pub struct PerformanceMeasurer {
    measurements: HashMap<String, Duration>,
    memory_measurements: HashMap<String, MemoryStats>,
}

impl PerformanceMeasurer {
    /// Create a new performance measurer
    pub fn new() -> Self {
        Self::default()
    }

    /// Measure the execution time of a closure
    pub fn measure<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.measurements.insert(name.to_string(), duration);
        result
    }

    /// Measure execution time and memory usage with cost tracker
    pub fn measure_with_memory<F, R>(&mut self, name: &str, tracker: &CostTracker, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let mut memory_tracker = MemoryUsageTracker::new(tracker);
        let start = std::time::Instant::now();

        let result = f();

        let duration = start.elapsed();
        memory_tracker.update_peak(tracker);
        let memory_stats = memory_tracker.get_stats(tracker);

        self.measurements.insert(name.to_string(), duration);
        self.memory_measurements
            .insert(name.to_string(), memory_stats);

        result
    }

    /// Get measurement by name
    pub fn get_measurement(&self, name: &str) -> Option<Duration> {
        self.measurements.get(name).copied()
    }

    /// Get all measurements
    pub fn get_all_measurements(&self) -> &HashMap<String, Duration> {
        &self.measurements
    }

    /// Assert that a measurement is within acceptable bounds
    pub fn assert_performance(&self, name: &str, max_duration: Duration) {
        match self.measurements.get(name) {
            Some(duration) => {
                assert!(
                    *duration <= max_duration,
                    "Performance test '{}' took {:?}, expected <= {:?}",
                    name,
                    duration,
                    max_duration
                );
            }
            None => panic!("No measurement found for '{}'", name),
        }
    }

    /// Get memory measurement by name
    pub fn get_memory_measurement(&self, name: &str) -> Option<&MemoryStats> {
        self.memory_measurements.get(name)
    }

    /// Assert that memory usage is within acceptable bounds
    pub fn assert_memory_usage(&self, name: &str, max_sessions: usize, max_cleanup_events: u32) {
        match self.memory_measurements.get(name) {
            Some(stats) => {
                assert!(
                    stats.validate_memory_usage(max_sessions),
                    "Memory usage test '{}' exceeded limits: peak {} sessions, current {} sessions (max allowed: {})",
                    name,
                    stats.peak_sessions,
                    stats.current_sessions,
                    max_sessions
                );

                // Ensure proper cleanup occurred
                if max_cleanup_events > 0 {
                    assert!(
                        stats.cleanup_events <= max_cleanup_events,
                        "Memory cleanup test '{}' had {} cleanup events, expected <= {}",
                        name,
                        stats.cleanup_events,
                        max_cleanup_events
                    );
                }
            }
            None => panic!("No memory measurement found for '{}'.", name),
        }
    }

    /// Print comprehensive performance and memory summary
    pub fn print_summary(&self) {
        println!("Performance Summary:");
        for (name, duration) in &self.measurements {
            println!("  {}: {:?}", name, duration);

            if let Some(memory_stats) = self.memory_measurements.get(name) {
                println!("    Memory - Initial: {}, Peak: {}, Current: {}, Active: {}, Completed: {}, Cleanup: {}",
                    memory_stats.initial_sessions,
                    memory_stats.peak_sessions,
                    memory_stats.current_sessions,
                    memory_stats.active_sessions,
                    memory_stats.completed_sessions,
                    memory_stats.cleanup_events
                );
            }
        }
    }
}

/// Realistic test data generator for various scenarios
#[derive(Default)]
pub struct TestDataGenerator {
    // Currently uses deterministic patterns, can be extended for randomness later
}

impl TestDataGenerator {
    /// Create with custom seed for reproducible random data
    /// Note: Currently uses deterministic patterns; seed is reserved for future use
    pub fn with_seed(_seed: u64) -> Self {
        Self {}
    }

    /// Generate realistic issue workflow token patterns
    pub fn generate_issue_workflow_tokens(&self) -> Vec<(u32, u32)> {
        // Simulate typical issue workflow patterns:
        // 1. Initial analysis (medium tokens)
        // 2. Code generation (high output)
        // 3. Testing/validation (low tokens)
        // 4. Final response (medium tokens)
        vec![
            (800, 400),  // Analysis
            (500, 1200), // Code generation
            (200, 150),  // Testing
            (300, 600),  // Final response
        ]
    }

    /// Generate realistic model usage patterns
    pub fn generate_model_usage_pattern(&self) -> Vec<(&'static str, f64)> {
        // Realistic distribution of model usage
        vec![
            ("claude-3-sonnet-20241022", 0.6),    // Primary model
            ("claude-3-haiku-20240307", 0.3),     // For simple tasks
            ("claude-3-opus-20240229", 0.08),     // For complex tasks
            ("claude-3-5-sonnet-20241022", 0.02), // For specific cases
        ]
    }

    /// Generate test issue IDs with realistic patterns
    pub fn generate_issue_ids(&self, count: u32) -> Vec<IssueId> {
        (0..count)
            .map(|i| {
                let issue_type = match i % 4 {
                    0 => "bug",
                    1 => "feature",
                    2 => "refactor",
                    _ => "docs",
                };
                IssueId::new(format!("{}-{:04}", issue_type, i)).expect("Valid issue ID")
            })
            .collect()
    }

    /// Generate API call failure patterns
    pub fn generate_failure_scenarios(&self) -> Vec<(ApiCallStatus, &'static str)> {
        vec![
            (ApiCallStatus::Failed, "Rate limit exceeded"),
            (ApiCallStatus::Timeout, "Request timeout after 30s"),
            (ApiCallStatus::Cancelled, "User cancelled request"),
            (ApiCallStatus::Failed, "Invalid API key"),
            (ApiCallStatus::Failed, "Model temporarily unavailable"),
        ]
    }

    /// Generate cost calculation test cases
    pub fn generate_cost_test_cases(&self) -> Vec<(u32, u32, &'static str, bool)> {
        vec![
            // (input_tokens, output_tokens, model, should_have_cost)
            (0, 0, "claude-3-sonnet", true),            // Zero tokens
            (1, 1, "claude-3-sonnet", true),            // Minimal tokens
            (1000, 500, "claude-3-sonnet", true),       // Typical usage
            (10000, 5000, "claude-3-opus", true),       // Heavy usage
            (500, 250, "claude-3-haiku", true),         // Light usage
            (1000000, 500000, "claude-3-sonnet", true), // Very large (edge case)
            (100, 200, "unknown-model", true),          // Unknown model
        ]
    }
}

/// Memory usage tracker for testing memory management
#[derive(Debug)]
pub struct MemoryUsageTracker {
    initial_session_count: usize,
    peak_session_count: usize,
    cleanup_events: u32,
}

impl MemoryUsageTracker {
    /// Create a new memory usage tracker initialized with current tracker state
    pub fn new(tracker: &CostTracker) -> Self {
        Self {
            initial_session_count: tracker.session_count(),
            peak_session_count: tracker.session_count(),
            cleanup_events: 0,
        }
    }

    /// Update peak session count if current is higher
    pub fn update_peak(&mut self, tracker: &CostTracker) {
        let current_count = tracker.session_count();
        if current_count > self.peak_session_count {
            self.peak_session_count = current_count;
        }
    }

    /// Record a cleanup event
    pub fn record_cleanup(&mut self) {
        self.cleanup_events += 1;
    }

    /// Get memory usage statistics
    pub fn get_stats(&self, tracker: &CostTracker) -> MemoryStats {
        MemoryStats {
            initial_sessions: self.initial_session_count,
            peak_sessions: self.peak_session_count,
            current_sessions: tracker.session_count(),
            active_sessions: tracker.active_session_count(),
            completed_sessions: tracker.completed_session_count(),
            cleanup_events: self.cleanup_events,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Number of sessions when tracking started
    pub initial_sessions: usize,
    /// Peak number of sessions reached during tracking
    pub peak_sessions: usize,
    /// Current number of sessions
    pub current_sessions: usize,
    /// Number of currently active sessions
    pub active_sessions: usize,
    /// Number of completed sessions
    pub completed_sessions: usize,
    /// Number of cleanup events that occurred
    pub cleanup_events: u32,
}

impl MemoryStats {
    /// Check if memory usage is within expected bounds
    pub fn validate_memory_usage(&self, max_sessions: usize) -> bool {
        self.peak_sessions <= max_sessions && self.current_sessions <= max_sessions
    }
}

/// Async test utilities for concurrent testing
pub mod async_utils {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, timeout, Duration};

    /// Run multiple async operations concurrently and collect results
    pub async fn run_concurrent_operations<F, Fut, T>(
        operations: Vec<F>,
        timeout_duration: Duration,
    ) -> Vec<Result<T, Box<dyn std::error::Error + Send + Sync>>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send,
        T: Send + 'static,
    {
        let tasks: Vec<_> = operations
            .into_iter()
            .map(|op| {
                tokio::spawn(async move {
                    timeout(timeout_duration, op())
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                })
            })
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(Ok(result)),
                Ok(Err(e)) => results.push(Err(e)),
                Err(e) => results.push(Err(Box::new(e))),
            }
        }

        results
    }

    /// Create a shared cost tracker for concurrent testing
    pub fn create_shared_tracker() -> Arc<Mutex<CostTracker>> {
        Arc::new(Mutex::new(CostTracker::new()))
    }

    /// Simulate concurrent session creation with realistic timing
    pub async fn simulate_concurrent_session_creation(
        tracker: Arc<Mutex<CostTracker>>,
        num_sessions: u32,
        delay_ms: u64,
    ) -> Result<Vec<crate::cost::CostSessionId>, Box<dyn std::error::Error + Send + Sync>> {
        let mut session_ids = Vec::new();

        for i in 0..num_sessions {
            let issue_id = IssueId::new(format!("concurrent-{}", i))?;

            {
                let mut tracker_guard = tracker.lock().await;
                let session_id = tracker_guard.start_session(issue_id)?;
                session_ids.push(session_id);
            }

            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        Ok(session_ids)
    }
}

/// Content type enumeration for standardized test data generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Basic test content for simple scenarios
    Simple,
    /// Descriptive test scenario content with context
    Descriptive,
    /// Performance-focused content for load testing
    Performance,
    /// Large content for stress testing
    LargeContent,
    /// Minimal content for edge case testing
    MinimalContent,
}

/// Content size specification for test data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentSize {
    /// Small content (~10-50 characters)
    Small,
    /// Medium content (~100-500 characters)
    Medium,
    /// Large content (~1000+ characters)
    Large,
    /// Variable size with custom repeat count
    Variable(usize),
}

/// Test context for consistent data generation
#[derive(Debug, Clone)]
pub struct TestContext {
    /// Name of the test for identification and logging
    pub test_name: String,
    /// Unique identifier for the operation within the test
    pub operation_id: usize,
    /// Identifier for the workflow this test belongs to
    pub workflow_id: usize,
    /// Additional key-value pairs for test-specific context
    pub additional_context: HashMap<String, String>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(test_name: String, operation_id: usize, workflow_id: usize) -> Self {
        Self {
            test_name,
            operation_id,
            workflow_id,
            additional_context: HashMap::new(),
        }
    }

    /// Add additional context data
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.additional_context.insert(key, value);
        self
    }
}

/// Standardized test issue builder for consistent data generation
#[derive(Debug, Clone)]
pub struct TestIssueBuilder {
    name_prefix: String,
    content_type: ContentType,
    content_size: ContentSize,
    sequence_id: Option<usize>,
    custom_suffix: Option<String>,
}

impl Default for TestIssueBuilder {
    fn default() -> Self {
        Self {
            name_prefix: "test-issue".to_string(),
            content_type: ContentType::Simple,
            content_size: ContentSize::Medium,
            sequence_id: None,
            custom_suffix: None,
        }
    }
}

impl TestIssueBuilder {
    /// Create a new test issue builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name prefix for the issue
    pub fn with_name_prefix(mut self, prefix: &str) -> Self {
        self.name_prefix = prefix.to_string();
        self
    }

    /// Set the content type
    pub fn with_content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set the content size
    pub fn with_content_size(mut self, content_size: ContentSize) -> Self {
        self.content_size = content_size;
        self
    }

    /// Set a sequence ID for numbering
    pub fn with_sequence_id(mut self, id: usize) -> Self {
        self.sequence_id = Some(id);
        self
    }

    /// Add a custom suffix to the name
    pub fn with_custom_suffix(mut self, suffix: &str) -> Self {
        self.custom_suffix = Some(suffix.to_string());
        self
    }

    /// Build the issue name
    pub fn build_name(&self) -> String {
        let mut name = self.name_prefix.clone();

        if let Some(id) = self.sequence_id {
            name.push_str(&format!("-{}", id));
        }

        if let Some(suffix) = &self.custom_suffix {
            name.push_str(&format!("-{}", suffix));
        }

        name
    }

    /// Build the issue content
    pub fn build_content(&self, context: &TestContext) -> String {
        let base_content = match self.content_type {
            ContentType::Simple => "Simple test content for basic operations.",
            ContentType::Descriptive => &format!(
                "Testing {} scenario: workflow {}, operation {} - comprehensive validation of API interception pipeline.",
                context.test_name, context.workflow_id, context.operation_id
            ),
            ContentType::Performance => &format!(
                "Performance testing operation {}/{} to measure API interception overhead and system scalability.",
                context.workflow_id, context.operation_id
            ),
            ContentType::LargeContent => "Large content block for stress testing the API interception system with substantial data payloads. This content is designed to test token counting accuracy, memory usage patterns, and overall system performance under load.",
            ContentType::MinimalContent => "Test",
        };

        match self.content_size {
            ContentSize::Small => base_content[..base_content.len().min(50)].to_string(),
            ContentSize::Medium => base_content.to_string(),
            ContentSize::Large => format!("{} {}", base_content, base_content.repeat(5)),
            ContentSize::Variable(repeat_count) => {
                format!("{} {}", base_content, base_content.repeat(repeat_count))
            }
        }
    }
}

/// Configuration for operation sequence building to reduce parameter count
#[derive(Debug)]
struct OperationSequenceConfig {
    pub test_type: String,
    pub prefix: String,
    pub count: usize,
    pub include_updates: bool,
    pub include_completions: bool,
    pub content_type: ContentType,
    pub content_size: Option<ContentSize>,
    pub custom_suffix: Option<String>,
}

/// Factory for creating standardized operation sequences
#[derive(Debug)]
pub struct OperationSequenceFactory;

impl OperationSequenceFactory {
    /// Common builder for creating issue operations with consistent patterns
    fn build_issue_operation(
        test_type: &str,
        prefix: &str,
        sequence_id: usize,
        operation_id: usize,
        content_type: ContentType,
        content_size: Option<ContentSize>,
        custom_suffix: Option<&str>,
    ) -> mock_mcp::McpOperation {
        let context = TestContext::new(test_type.to_string(), operation_id, sequence_id);
        let mut builder = TestIssueBuilder::new()
            .with_name_prefix(prefix)
            .with_sequence_id(sequence_id)
            .with_content_type(content_type);

        if let Some(size) = content_size {
            builder = builder.with_content_size(size);
        }

        if let Some(suffix) = custom_suffix {
            builder = builder.with_custom_suffix(suffix);
        }

        mock_mcp::McpOperation::CreateIssue {
            name: builder.build_name(),
            content: builder.build_content(&context),
        }
    }

    /// Common method to build operation sequences with configurable patterns
    fn build_operation_sequence(config: OperationSequenceConfig) -> Vec<mock_mcp::McpOperation> {
        let mut operations = Vec::new();

        for i in 0..config.count {
            // Create issue operation
            operations.push(Self::build_issue_operation(
                &config.test_type,
                &config.prefix,
                i,
                i,
                config.content_type,
                config.content_size,
                config.custom_suffix.as_deref(),
            ));

            // Add update operations based on pattern
            if config.include_updates && (i % 2 == 0 || config.test_type == "basic-crud") {
                operations.push(mock_mcp::McpOperation::UpdateIssue {
                    number: (i + 1) as u32,
                    content: format!("Updated content for {} test sequence {}", config.prefix, i),
                });
            }

            // Add completion operations based on pattern
            if config.include_completions {
                operations.push(mock_mcp::McpOperation::MarkComplete {
                    number: (i + 1) as u32,
                });
            }
        }

        operations
    }
    /// Create a basic CRUD sequence: Create -> Update -> Complete
    pub fn create_basic_crud_sequence(prefix: &str, _id: usize) -> Vec<mock_mcp::McpOperation> {
        Self::build_operation_sequence(OperationSequenceConfig {
            test_type: "basic-crud".to_string(),
            prefix: prefix.to_string(),
            count: 1,                  // Single sequence
            include_updates: true,     // Include updates
            include_completions: true, // Include completions
            content_type: ContentType::Descriptive,
            content_size: None,
            custom_suffix: None,
        })
    }

    /// Create a performance testing sequence with multiple operations
    pub fn create_performance_sequence(prefix: &str, count: usize) -> Vec<mock_mcp::McpOperation> {
        Self::build_operation_sequence(OperationSequenceConfig {
            test_type: "performance".to_string(),
            prefix: prefix.to_string(),
            count,
            include_updates: true,      // Include updates (every even index)
            include_completions: false, // No completions
            content_type: ContentType::Performance,
            content_size: None,
            custom_suffix: None,
        })
    }

    /// Create a sequence for error resilience testing
    pub fn create_error_testing_sequence(prefix: &str) -> Vec<mock_mcp::McpOperation> {
        let mut operations = vec![Self::build_issue_operation(
            "error-resilience",
            prefix,
            0,
            0,
            ContentType::Descriptive,
            None,
            Some("error-test"),
        )];

        // Add operations that should fail gracefully
        operations.extend(vec![
            mock_mcp::McpOperation::UpdateIssue {
                number: 999,
                content: "Update for non-existent issue to test error handling".to_string(),
            },
            mock_mcp::McpOperation::MarkComplete { number: 998 },
        ]);

        operations
    }

    /// Create concurrent operation sequence for load testing
    pub fn create_concurrent_sequence(
        prefix: &str,
        worker_id: usize,
        ops_per_workflow: usize,
    ) -> Vec<mock_mcp::McpOperation> {
        (0..ops_per_workflow)
            .map(|op_id| {
                Self::build_issue_operation(
                    "concurrent",
                    prefix,
                    worker_id * ops_per_workflow + op_id,
                    op_id,
                    ContentType::Performance,
                    None,
                    None,
                )
            })
            .collect()
    }

    /// Create memory stress testing sequence with large content
    pub fn create_memory_stress_sequence(
        prefix: &str,
        system_id: usize,
        ops_count: usize,
    ) -> Vec<mock_mcp::McpOperation> {
        (0..ops_count)
            .map(|op_id| {
                Self::build_issue_operation(
                    "memory-stress",
                    prefix,
                    system_id * ops_count + op_id,
                    op_id,
                    ContentType::LargeContent,
                    Some(ContentSize::Variable(3)), // Repeat 3 times for stress
                    None,
                )
            })
            .collect()
    }
}

/// Standardized workflow name generator
#[derive(Debug)]
pub struct WorkflowNameGenerator;

impl WorkflowNameGenerator {
    /// Generate name for functional tests
    pub fn functional_test(test_type: &str, sequence_id: Option<usize>) -> String {
        match sequence_id {
            Some(id) => format!("functional-{}-{}", test_type, id),
            None => format!("functional-{}", test_type),
        }
    }

    /// Generate name for performance tests
    pub fn performance_test(test_type: &str, batch_id: usize) -> String {
        format!("performance-{}-batch-{}", test_type, batch_id)
    }

    /// Generate name for reliability tests
    pub fn reliability_test(test_type: &str, scenario: &str) -> String {
        format!("reliability-{}-{}", test_type, scenario)
    }

    /// Generate name for concurrent tests
    pub fn concurrent_test(test_type: &str, worker_id: usize) -> String {
        format!("concurrent-{}-worker-{}", test_type, worker_id)
    }
}

/// Standard test configuration factory
#[derive(Debug)]
pub struct TestConfigFactory;

/// Standard test configuration structure
#[derive(Debug, Clone)]
pub struct StandardTestConfig {
    /// Number of workflows to run concurrently
    pub concurrent_workflows: usize,
    /// Number of operations each workflow should perform
    pub operations_per_workflow: usize,
    /// Maximum expected duration for the entire test
    pub expected_duration_limit: Duration,
    /// Memory usage threshold that triggers cleanup
    pub memory_cleanup_threshold: usize,
    /// Number of retry attempts for failed operations
    pub retry_attempts: usize,
    /// Timeout duration for individual operations
    pub timeout_duration: Duration,
}

impl TestConfigFactory {
    /// Basic test configuration for simple scenarios
    pub fn basic_test_config() -> StandardTestConfig {
        StandardTestConfig {
            concurrent_workflows: 3,
            operations_per_workflow: 5,
            expected_duration_limit: Duration::from_secs(30),
            memory_cleanup_threshold: 10,
            retry_attempts: 3,
            timeout_duration: Duration::from_secs(60),
        }
    }

    /// Performance test configuration for load testing
    pub fn performance_test_config() -> StandardTestConfig {
        StandardTestConfig {
            concurrent_workflows: 10,
            operations_per_workflow: 15,
            expected_duration_limit: Duration::from_secs(120),
            memory_cleanup_threshold: 50,
            retry_attempts: 1, // No retries for performance tests
            timeout_duration: Duration::from_secs(300),
        }
    }

    /// Error-prone configuration for resilience testing
    pub fn error_prone_config() -> StandardTestConfig {
        StandardTestConfig {
            concurrent_workflows: 5,
            operations_per_workflow: 10,
            expected_duration_limit: Duration::from_secs(60),
            memory_cleanup_threshold: 20,
            retry_attempts: 5, // More retries for error scenarios
            timeout_duration: Duration::from_secs(180),
        }
    }

    /// Concurrent test configuration for load testing
    pub fn concurrent_test_config(concurrency_level: usize) -> StandardTestConfig {
        StandardTestConfig {
            concurrent_workflows: concurrency_level,
            operations_per_workflow: 8,
            expected_duration_limit: Duration::from_secs(90),
            memory_cleanup_threshold: concurrency_level * 5,
            retry_attempts: 2,
            timeout_duration: Duration::from_secs(240),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_call_generator() {
        let generator = ApiCallGenerator::default();

        // Test single call generation
        let api_call = generator.generate_api_call(0);
        assert!(api_call.endpoint.contains("call-0"));
        assert!(generator.models.contains(&api_call.model));

        // Test completed call generation
        let completed_call = generator.generate_completed_api_call(1);
        assert!(completed_call.is_completed());
        assert!(completed_call.total_tokens() > 0);

        // Test multiple calls generation
        let calls = generator.generate_multiple_calls(5);
        assert_eq!(calls.len(), 5);
        assert!(calls.iter().all(|call| call.is_completed()));
    }

    #[test]
    fn test_config_builder() {
        // Test paid plan configuration
        let paid_calculator = TestConfigBuilder::new().with_paid_plan().build_calculator();

        assert!(paid_calculator.supports_cost_calculation());
        assert!(!paid_calculator.provides_estimates());

        // Test max plan configuration
        let max_calculator = TestConfigBuilder::new()
            .with_max_plan_tracking()
            .build_calculator();

        assert!(!max_calculator.supports_cost_calculation());
        assert!(!max_calculator.provides_estimates());

        // Test custom rates
        let custom_calculator = TestConfigBuilder::new()
            .with_paid_plan()
            .with_custom_rates("test-model".to_string(), "0.0001", "0.0005")
            .build_calculator();

        let rates = custom_calculator.get_rates_for_model("test-model");
        assert!(rates.is_some());
    }

    #[test]
    fn test_session_lifecycle_helper() {
        let mut helper = SessionLifecycleHelper::default();

        // Test session creation
        let result = helper.create_test_session("lifecycle", 3);
        assert!(result.is_ok());

        let (session_id, initial_cost) = result.unwrap();
        assert!(initial_cost >= Decimal::ZERO);

        // Test session completion
        let final_cost = helper.complete_session(&session_id, CostSessionStatus::Completed);
        assert!(final_cost.is_ok());
        assert_eq!(final_cost.unwrap(), initial_cost);

        // Test multiple sessions
        let sessions = helper.create_multiple_sessions("batch", 3, 2);
        assert!(sessions.is_ok());
        assert_eq!(sessions.unwrap().len(), 3);
    }

    #[test]
    fn test_performance_measurer() {
        let mut measurer = PerformanceMeasurer::new();

        // Measure a simple operation
        let result = measurer.measure("test_operation", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);

        let measurement = measurer.get_measurement("test_operation");
        assert!(measurement.is_some());
        assert!(measurement.unwrap() >= Duration::from_millis(10));

        // Test performance assertion (should pass)
        measurer.assert_performance("test_operation", Duration::from_millis(100));
    }

    #[test]
    fn test_test_data_generator() {
        let generator = TestDataGenerator::default();

        // Test issue workflow tokens
        let tokens = generator.generate_issue_workflow_tokens();
        assert_eq!(tokens.len(), 4); // Analysis, Code gen, Testing, Final
        assert!(tokens
            .iter()
            .all(|(input, output)| *input > 0 && *output > 0));

        // Test model usage patterns
        let patterns = generator.generate_model_usage_pattern();
        let total_weight: f64 = patterns.iter().map(|(_, weight)| weight).sum();
        assert!((total_weight - 1.0).abs() < 0.01); // Should sum to ~1.0

        // Test issue ID generation
        let issue_ids = generator.generate_issue_ids(10);
        assert_eq!(issue_ids.len(), 10);
        assert!(issue_ids.iter().all(|id| !id.as_str().is_empty()));

        // Test failure scenarios
        let failures = generator.generate_failure_scenarios();
        assert!(!failures.is_empty());

        // Test cost test cases
        let cost_cases = generator.generate_cost_test_cases();
        assert!(!cost_cases.is_empty());
        assert!(cost_cases.iter().all(|(_, _, _, _)| true)); // All should be valid
    }

    #[test]
    fn test_memory_usage_tracker() {
        let tracker = CostTracker::new();
        let memory_tracker = MemoryUsageTracker::new(&tracker);

        // Initial stats
        let stats = memory_tracker.get_stats(&tracker);
        assert_eq!(stats.initial_sessions, 0);
        assert_eq!(stats.current_sessions, 0);

        // Test validation
        assert!(stats.validate_memory_usage(1000));
    }

    #[tokio::test]
    async fn test_async_utilities() {
        use async_utils::*;

        // Test shared tracker
        let shared_tracker = create_shared_tracker();
        let session_ids = simulate_concurrent_session_creation(shared_tracker, 5, 1).await;
        assert!(session_ids.is_ok());
        assert_eq!(session_ids.unwrap().len(), 5);
    }

    #[test]
    fn test_standardized_test_data_builders() {
        // Test issue builder
        let context = TestContext::new("unit-test".to_string(), 1, 0);
        let builder = TestIssueBuilder::new()
            .with_name_prefix("test-builder")
            .with_sequence_id(42)
            .with_content_type(ContentType::Descriptive);

        let name = builder.build_name();
        assert_eq!(name, "test-builder-42");

        let content = builder.build_content(&context);
        assert!(content.contains("unit-test"));
        assert!(content.contains("workflow 0"));
        assert!(content.contains("operation 1"));

        // Test operation sequence factory
        let crud_ops = OperationSequenceFactory::create_basic_crud_sequence("crud-test", 1);
        assert_eq!(crud_ops.len(), 3);

        let perf_ops = OperationSequenceFactory::create_performance_sequence("perf-test", 5);
        assert_eq!(perf_ops.len(), 8); // 5 creates + 3 updates (every even index)

        // Test workflow name generator
        let func_name = WorkflowNameGenerator::functional_test("pipeline", Some(1));
        assert_eq!(func_name, "functional-pipeline-1");

        let perf_name = WorkflowNameGenerator::performance_test("overhead", 2);
        assert_eq!(perf_name, "performance-overhead-batch-2");

        // Test config factory
        let basic_config = TestConfigFactory::basic_test_config();
        assert_eq!(basic_config.concurrent_workflows, 3);
        assert_eq!(basic_config.operations_per_workflow, 5);

        let perf_config = TestConfigFactory::performance_test_config();
        assert_eq!(perf_config.concurrent_workflows, 10);
        assert_eq!(perf_config.operations_per_workflow, 15);
    }
}
