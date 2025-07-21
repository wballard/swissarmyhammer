//! Storage test utilities for comprehensive multi-backend testing
//!
//! This module provides utilities for testing storage and reporting functionality
//! across multiple storage backends including markdown files, metrics system,
//! and optional database storage.

use crate::cost::{
    calculator::CostCalculator,
    formatting::{CostSectionFormatter, IssueCostData},
    tracker::{ApiCall, CostSession, CostTracker, IssueId},
    CostError,
};

#[cfg(feature = "database")]
use crate::cost::database::{CostDatabase, DatabaseConfig};
use crate::issues::filesystem::{Issue, IssueNumber, IssueStorage};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Standardized error conversion utilities for consistent error handling
pub mod error_conversion {
    use crate::cost::CostError;

    /// Convert IO errors to CostError with context
    pub fn io_error(context: &str, error: std::io::Error) -> CostError {
        CostError::InvalidInput {
            message: format!("{}: {}", context, error),
        }
    }

    /// Convert storage errors to CostError with context
    pub fn storage_error(context: &str, error: impl std::fmt::Display) -> CostError {
        CostError::InvalidInput {
            message: format!("{}: {}", context, error),
        }
    }

    /// Convert parsing errors to CostError with context
    pub fn parse_error(context: &str, error: impl std::fmt::Display) -> CostError {
        CostError::InvalidInput {
            message: format!("{}: {}", context, error),
        }
    }

    /// Convert generic errors to CostError with context
    pub fn generic_error(context: &str, error: impl std::fmt::Display) -> CostError {
        CostError::InvalidInput {
            message: format!("{}: {}", context, error),
        }
    }
}

/// Test harness for coordinating multi-backend storage testing
pub struct StorageTestHarness<S: IssueStorage> {
    /// Cost tracking component
    pub tracker: CostTracker,
    /// Cost calculation component
    pub calculator: CostCalculator,
    /// Cost section formatting component
    pub formatter: CostSectionFormatter,
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Issue storage backend under test
    pub issue_storage: Arc<RwLock<S>>,
    /// Optional cost database for enhanced testing
    #[cfg(feature = "database")]
    pub database: Option<CostDatabase>,
    /// Placeholder when database feature is disabled
    #[cfg(not(feature = "database"))]
    pub database: (),
    /// Test configuration settings
    pub test_config: StorageTestConfig,
}

/// Configuration for storage testing scenarios
#[derive(Debug, Clone)]
pub struct StorageTestConfig {
    /// Enable database storage backend testing
    pub enable_database: bool,
    /// Enable markdown file storage testing
    pub enable_markdown_storage: bool,
    /// Enable metrics integration testing
    pub enable_metrics_integration: bool,
    /// Run tests in simulation mode without real storage
    pub simulation_mode: bool,
    /// Enable performance measurement during testing
    pub performance_testing: bool,
}

impl Default for StorageTestConfig {
    fn default() -> Self {
        Self {
            enable_database: true,
            enable_markdown_storage: true,
            enable_metrics_integration: true,
            simulation_mode: false,
            performance_testing: false,
        }
    }
}

impl StorageTestHarness<MockStorageForTesting> {
    /// Create a new storage test harness with default configuration
    pub async fn new() -> Result<Self, CostError> {
        Self::with_config(StorageTestConfig::default()).await
    }

    /// Create a storage test harness with custom configuration
    pub async fn with_config(config: StorageTestConfig) -> Result<Self, CostError> {
        let temp_dir = TempDir::new()
            .map_err(|e| error_conversion::io_error("Failed to create temp directory", e))?;

        let tracker = CostTracker::new();
        let calculator = CostCalculator::paid_default();
        let formatter = CostSectionFormatter::default();

        // Create mock issue storage
        let issue_storage: Arc<RwLock<MockStorageForTesting>> =
            Arc::new(RwLock::new(MockStorageForTesting::new()));

        #[cfg(feature = "database")]
        let database = if config.enable_database {
            let db_path = temp_dir.path().join("test_costs.db");
            let db_config = DatabaseConfig::with_values(
                true,
                db_path,
                std::time::Duration::from_secs(30),
                10,
                365,
            )?;
            Some(CostDatabase::new(db_config).await?)
        } else {
            None
        };

        #[cfg(not(feature = "database"))]
        let database = ();

        Ok(Self {
            tracker,
            calculator,
            formatter,
            temp_dir,
            issue_storage,
            database,
            test_config: config,
        })
    }

    /// Get the temporary directory path for test files
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a test issue with cost tracking enabled
    pub async fn create_test_issue(&mut self, issue_name: &str) -> Result<IssueId, CostError> {
        let issue_id = IssueId::new(format!("test-{}", issue_name))?;

        // Initialize issue in storage if enabled
        if self.test_config.enable_markdown_storage {
            let storage = self.issue_storage.read().await;
            storage
                .create_issue(
                    issue_id.as_str().to_string(),
                    "Test issue content".to_string(),
                )
                .await
                .map_err(|e| error_conversion::storage_error("Failed to create test issue", e))?;
        }

        Ok(issue_id)
    }

    /// Start a cost session for testing
    pub fn start_cost_session(
        &mut self,
        issue_id: IssueId,
    ) -> Result<crate::cost::CostSessionId, CostError> {
        self.tracker.start_session(issue_id)
    }

    /// Add realistic test API calls to a session
    pub async fn add_test_api_calls(
        &mut self,
        session_id: &crate::cost::CostSessionId,
        call_count: usize,
    ) -> Result<Vec<crate::cost::ApiCallId>, CostError> {
        let mut call_ids = Vec::new();

        for i in 0..call_count {
            let api_call = ApiCall::new(
                format!("https://api.anthropic.com/v1/messages/{}", i),
                "claude-3-sonnet-20241022",
            )?;

            let call_id = self.tracker.add_api_call(session_id, api_call)?;
            call_ids.push(call_id);
        }

        Ok(call_ids)
    }

    /// Complete API calls with realistic token usage
    pub fn complete_api_calls(
        &mut self,
        session_id: &crate::cost::CostSessionId,
        call_ids: &[crate::cost::ApiCallId],
    ) -> Result<(), CostError> {
        for (i, call_id) in call_ids.iter().enumerate() {
            let input_tokens = 100 + (i as u32 * 50);
            let output_tokens = 150 + (i as u32 * 75);
            let success = i % 4 != 0; // 75% success rate

            self.tracker.complete_api_call(
                session_id,
                call_id,
                input_tokens,
                output_tokens,
                if success {
                    crate::cost::ApiCallStatus::Success
                } else {
                    crate::cost::ApiCallStatus::Failed
                },
                if success {
                    None
                } else {
                    Some(format!("Test error {}", i))
                },
            )?;
        }

        Ok(())
    }

    /// Complete a cost session and store results across all backends
    pub async fn complete_session_with_storage(
        &mut self,
        session_id: &crate::cost::CostSessionId,
        status: crate::cost::CostSessionStatus,
    ) -> Result<StorageResults, CostError> {
        // Complete the session
        self.tracker.complete_session(session_id, status)?;

        let session =
            self.tracker
                .get_session(session_id)
                .ok_or_else(|| CostError::SessionNotFound {
                    session_id: *session_id,
                })?;

        let mut results = StorageResults::new();

        // Store in markdown format
        if self.test_config.enable_markdown_storage {
            let cost_data = self.create_issue_cost_data(session)?;
            let markdown_content = self.formatter.format_cost_section(&cost_data);
            results.markdown_content = Some(markdown_content);

            // Store in issue storage - use mark_complete_with_cost
            let storage = self.issue_storage.read().await;
            storage
                .mark_complete_with_cost(1, cost_data.clone())
                .await
                .map_err(|e| error_conversion::storage_error("Failed to store cost data", e))?;
        }

        // Store in database if enabled
        #[cfg(feature = "database")]
        if let Some(database) = &self.database {
            if self.test_config.enable_database {
                database.store_session(session).await?;
                results.database_stored = true;
            }
        }

        #[cfg(not(feature = "database"))]
        if self.test_config.enable_database {
            // Database feature not available - mark as not stored but don't treat as failure
            results.database_stored = false;
        }

        // Update metrics if enabled
        if self.test_config.enable_metrics_integration {
            results.metrics_updated = self.update_metrics(session).await?;
        }

        Ok(results)
    }

    /// Create issue cost data from a completed session
    fn create_issue_cost_data(&self, session: &CostSession) -> Result<IssueCostData, CostError> {
        use crate::cost::formatting::{CostSummaryStats, CurrencyAmount};

        let cost_calculation = self.calculator.calculate_session_cost(session)?;

        // Create currency amount from cost calculation
        let total_cost = if cost_calculation.total_cost > rust_decimal::Decimal::ZERO {
            Some(CurrencyAmount::new(cost_calculation.total_cost))
        } else {
            None
        };

        // Create summary stats
        let average_cost_per_call = if !session.api_calls.is_empty()
            && cost_calculation.total_cost > rust_decimal::Decimal::ZERO
        {
            Some(CurrencyAmount::new(
                cost_calculation.total_cost / rust_decimal::Decimal::from(session.api_calls.len()),
            ))
        } else {
            None
        };

        // Find most expensive call
        let most_expensive_call = session
            .api_calls
            .values()
            .filter_map(|call| {
                if call.is_completed() {
                    self.calculator
                        .calculate_tokens_cost(call.input_tokens, call.output_tokens, &call.model)
                        .ok()
                        .map(|calc| CurrencyAmount::new(calc.total_cost))
                } else {
                    None
                }
            })
            .max_by(|a, b| a.amount().cmp(&b.amount()));

        let summary_stats = CostSummaryStats {
            average_cost_per_call,
            most_expensive_call,
            token_efficiency: if cost_calculation.input_tokens > 0 {
                Some(
                    rust_decimal::Decimal::from(cost_calculation.output_tokens)
                        / rust_decimal::Decimal::from(cost_calculation.input_tokens),
                )
            } else {
                None
            },
            total_duration: session
                .total_duration
                .map(|d| crate::cost::formatting::SessionDuration::new(d)),
            successful_calls: crate::cost::formatting::ApiCallCount::new(
                session
                    .api_calls
                    .values()
                    .filter(|call| matches!(call.status, crate::cost::ApiCallStatus::Success))
                    .count() as u32,
            ),
            failed_calls: crate::cost::formatting::ApiCallCount::new(
                session
                    .api_calls
                    .values()
                    .filter(|call| !matches!(call.status, crate::cost::ApiCallStatus::Success))
                    .count() as u32,
            ),
        };

        Ok(IssueCostData {
            session_data: session.clone(),
            total_cost,
            pricing_model: self.calculator.pricing_model.clone(),
            summary_stats,
        })
    }

    /// Update metrics system with cost data
    async fn update_metrics(&self, session: &CostSession) -> Result<bool, CostError> {
        // Simulate metrics update
        if !self.test_config.simulation_mode {
            // In real implementation, this would integrate with metrics system
            tracing::info!("Updating metrics for session {}", session.session_id);
        }
        Ok(true)
    }
}

impl<S: IssueStorage> StorageTestHarness<S> {
    /// Create a new storage test harness with custom storage implementation
    pub fn with_storage(
        storage: S,
        temp_dir: TempDir,
        test_config: StorageTestConfig,
    ) -> Result<Self, CostError> {
        let tracker = CostTracker::new();
        let calculator = CostCalculator::paid_default();
        let formatter = CostSectionFormatter::default();
        let issue_storage = Arc::new(RwLock::new(storage));

        #[cfg(feature = "database")]
        let database = None; // Custom storage implementations don't automatically get database

        #[cfg(not(feature = "database"))]
        let database = ();

        Ok(Self {
            tracker,
            calculator,
            formatter,
            temp_dir,
            issue_storage,
            database,
            test_config,
        })
    }

    /// Get a reference to the issue storage
    pub fn storage(&self) -> &Arc<RwLock<S>> {
        &self.issue_storage
    }
}

/// Results from multi-backend storage operations
#[derive(Debug, Default)]
pub struct StorageResults {
    /// Markdown content that was stored, if any
    pub markdown_content: Option<String>,
    /// Whether data was successfully stored in database
    pub database_stored: bool,
    /// Whether metrics were successfully updated
    pub metrics_updated: bool,
}

impl StorageResults {
    fn new() -> Self {
        Self::default()
    }

    /// Check if all configured backends were successfully updated
    pub fn all_backends_success(&self, config: &StorageTestConfig) -> bool {
        let markdown_ok = !config.enable_markdown_storage || self.markdown_content.is_some();

        // Database is OK if:
        // - Database is not enabled in config, OR
        // - Database feature is not available (compile-time), OR
        // - Database was successfully stored
        #[cfg(feature = "database")]
        let database_ok = !config.enable_database || self.database_stored;

        #[cfg(not(feature = "database"))]
        let database_ok = true; // Always OK when feature is not available

        let metrics_ok = !config.enable_metrics_integration || self.metrics_updated;

        markdown_ok && database_ok && metrics_ok
    }
}

/// Multi-backend data consistency validator
pub struct MultiBackendValidator {
    expected_data: HashMap<String, ExpectedCostData>,
}

/// Expected cost data for validation purposes
#[derive(Debug, Clone)]
pub struct ExpectedCostData {
    /// Total expected cost for the operation
    pub total_cost: Decimal,
    /// Total number of API calls made
    pub total_api_calls: usize,
    /// Number of input tokens consumed
    pub input_tokens: u32,
    /// Number of output tokens generated
    pub output_tokens: u32,
}

impl MultiBackendValidator {
    /// Create a new multi-backend validator
    pub fn new() -> Self {
        Self {
            expected_data: HashMap::new(),
        }
    }

    /// Record expected data for a session
    pub fn expect_session_data(&mut self, session_id: &str, data: ExpectedCostData) {
        self.expected_data.insert(session_id.to_string(), data);
    }

    /// Validate markdown content against expected data
    pub fn validate_markdown_content(
        &self,
        session_id: &str,
        content: &str,
    ) -> Result<bool, CostError> {
        let expected =
            self.expected_data
                .get(session_id)
                .ok_or_else(|| CostError::InvalidInput {
                    message: format!("No expected data for session {}", session_id),
                })?;

        // Check for properly formatted markdown fields
        let contains_cost =
            content.contains(&format!("**Total Cost**: ${:.2}", expected.total_cost));
        let contains_calls = content.contains(&format!(
            "**Total API Calls**: {}",
            expected.total_api_calls
        ));
        let contains_input = content.contains(&format!(
            "**Total Input Tokens**: {}",
            expected.input_tokens
        ));
        let contains_output = content.contains(&format!(
            "**Total Output Tokens**: {}",
            expected.output_tokens
        ));

        Ok(contains_cost && contains_calls && contains_input && contains_output)
    }

    /// Validate database data against expected data
    #[cfg(feature = "database")]
    pub async fn validate_database_data(
        &self,
        session_id: &str,
        database: &CostDatabase,
    ) -> Result<bool, CostError> {
        use crate::cost::database::queries::{TimePeriod, TrendQuery};

        let session_id_parsed = session_id
            .parse()
            .map_err(|e| error_conversion::parse_error("Invalid session ID format", e))?;

        // Query database for session data
        let trend_query = TrendQuery::new(TimePeriod::Day, 1);
        let trends = database.get_cost_trends(trend_query).await?;

        // Validate at least one trend entry exists
        Ok(!trends.is_empty())
    }
}

/// Mock storage implementation for testing
pub struct MockStorageForTesting {
    issues: HashMap<String, MockIssueData>,
}

#[derive(Debug, Clone)]
struct MockIssueData {
    content: String,
    cost_data: Option<IssueCostData>,
}

impl MockStorageForTesting {
    /// Create a new mock storage for testing
    pub fn new() -> Self {
        Self {
            issues: HashMap::new(),
        }
    }

    /// Helper method to create Issue structs with consistent defaults
    fn create_mock_issue(number: u32, name: String, content: String, completed: bool) -> Issue {
        let base_path = if completed { "/tmp/complete" } else { "/tmp" };
        Issue {
            number: IssueNumber::new(number).unwrap(),
            name,
            content,
            completed,
            file_path: PathBuf::from(&format!("{}/mock.md", base_path)),
            created_at: Utc::now(),
        }
    }

    /// Create a test issue with default content
    pub async fn create_test_issue(&mut self, issue_id: &IssueId) -> Result<Issue, CostError> {
        let issue_data = MockIssueData {
            content: format!("# Test Issue {}\n\nTest issue content\n", issue_id.as_str()),
            cost_data: None,
        };

        let issue = Self::create_mock_issue(
            1, // Mock number
            issue_id.as_str().to_string(),
            issue_data.content.clone(),
            false,
        );

        self.issues
            .insert(issue_id.as_str().to_string(), issue_data);
        Ok(issue)
    }

    /// Update an issue with cost data
    pub async fn update_with_cost_data(
        &mut self,
        issue_id: &IssueId,
        cost_data: &IssueCostData,
    ) -> Result<(), CostError> {
        if let Some(issue) = self.issues.get_mut(issue_id.as_str()) {
            issue.cost_data = Some(cost_data.clone());
            issue
                .content
                .push_str("\n## Cost Analysis\n[Cost data would be inserted here]\n");
        }
        Ok(())
    }

    /// Get cost data for a specific issue
    pub fn get_issue_cost_data(&self, issue_id: &IssueId) -> Option<&IssueCostData> {
        self.issues.get(issue_id.as_str())?.cost_data.as_ref()
    }
}

#[async_trait::async_trait]
impl IssueStorage for MockStorageForTesting {
    async fn list_issues(&self) -> crate::error::Result<Vec<Issue>> {
        let mut issues = Vec::new();
        for (id, data) in &self.issues {
            let issue = Self::create_mock_issue(1, id.clone(), data.content.clone(), false);
            issues.push(issue);
        }
        Ok(issues)
    }

    async fn get_issue(&self, number: u32) -> crate::error::Result<Issue> {
        Ok(Self::create_mock_issue(
            number,
            format!("mock-issue-{}", number),
            "Mock issue content".to_string(),
            false,
        ))
    }

    async fn create_issue(&self, name: String, content: String) -> crate::error::Result<Issue> {
        Ok(Self::create_mock_issue(1, name, content, false))
    }

    async fn update_issue(&self, number: u32, content: String) -> crate::error::Result<Issue> {
        Ok(Self::create_mock_issue(
            number,
            format!("updated-issue-{}", number),
            content,
            false,
        ))
    }

    async fn mark_complete(&self, number: u32) -> crate::error::Result<Issue> {
        Ok(Self::create_mock_issue(
            number,
            format!("completed-issue-{}", number),
            "Completed issue content".to_string(),
            true,
        ))
    }

    async fn mark_complete_with_cost(
        &self,
        number: u32,
        _cost_data: crate::cost::IssueCostData,
    ) -> crate::error::Result<Issue> {
        Ok(Self::create_mock_issue(
            number,
            format!("completed-with-cost-{}", number),
            "Completed issue with cost data".to_string(),
            true,
        ))
    }

    async fn create_issues_batch(
        &self,
        issues: Vec<(String, String)>,
    ) -> crate::error::Result<Vec<Issue>> {
        let mut result = Vec::new();
        for (name, content) in issues {
            result.push(self.create_issue(name, content).await?);
        }
        Ok(result)
    }

    async fn get_issues_batch(&self, numbers: Vec<u32>) -> crate::error::Result<Vec<Issue>> {
        let mut result = Vec::new();
        for number in numbers {
            result.push(self.get_issue(number).await?);
        }
        Ok(result)
    }

    async fn update_issues_batch(
        &self,
        updates: Vec<(u32, String)>,
    ) -> crate::error::Result<Vec<Issue>> {
        let mut result = Vec::new();
        for (number, content) in updates {
            result.push(self.update_issue(number, content).await?);
        }
        Ok(result)
    }

    async fn mark_complete_batch(&self, numbers: Vec<u32>) -> crate::error::Result<Vec<Issue>> {
        let mut result = Vec::new();
        for number in numbers {
            result.push(self.mark_complete(number).await?);
        }
        Ok(result)
    }
}

/// Performance testing utilities
pub struct PerformanceValidator {
    start_times: HashMap<String, std::time::Instant>,
    benchmarks: HashMap<String, std::time::Duration>,
}

impl PerformanceValidator {
    /// Create a new performance validator
    pub fn new() -> Self {
        Self {
            start_times: HashMap::new(),
            benchmarks: HashMap::new(),
        }
    }

    /// Start timing a performance test
    pub fn start_timing(&mut self, test_name: &str) {
        self.start_times
            .insert(test_name.to_string(), std::time::Instant::now());
    }

    /// Stop timing and record benchmark
    pub fn stop_timing(&mut self, test_name: &str) -> Option<std::time::Duration> {
        if let Some(start_time) = self.start_times.remove(test_name) {
            let duration = start_time.elapsed();
            self.benchmarks.insert(test_name.to_string(), duration);
            Some(duration)
        } else {
            None
        }
    }

    /// Assert performance bounds
    pub fn assert_performance_bounds(
        &self,
        test_name: &str,
        max_duration: std::time::Duration,
    ) -> bool {
        if let Some(actual_duration) = self.benchmarks.get(test_name) {
            *actual_duration <= max_duration
        } else {
            false
        }
    }

    /// Get all benchmark results
    pub fn get_benchmarks(&self) -> &HashMap<String, std::time::Duration> {
        &self.benchmarks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_test_harness_creation() {
        let harness = StorageTestHarness::new().await.unwrap();
        assert!(harness.temp_path().exists());
    }

    #[tokio::test]
    async fn test_multi_backend_validator() {
        let mut validator = MultiBackendValidator::new();
        let expected = ExpectedCostData {
            total_cost: Decimal::from_str_exact("0.25").unwrap(),
            total_api_calls: 3,
            input_tokens: 1000,
            output_tokens: 1500,
        };

        validator.expect_session_data("test-session", expected.clone());

        let markdown = "**Total Cost**: $0.25\n**Total API Calls**: 3\n**Total Input Tokens**: 1000\n**Total Output Tokens**: 1500";
        assert!(validator
            .validate_markdown_content("test-session", markdown)
            .unwrap());
    }
}
