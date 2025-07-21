//! Mock MCP system for comprehensive API interception testing
//!
//! This module provides realistic mock implementations for testing the complete
//! API interception pipeline, including Claude API responses, network conditions,
//! and various failure scenarios. It enables comprehensive testing of the integration
//! between MCP handlers, cost tracking, and token counting systems.

#![allow(missing_docs)]

use crate::cost::{CostSessionId, CostSessionStatus, CostTracker};
use crate::mcp::cost_tracking::{CostTrackingMcpHandler, SessionStats};
use crate::mcp::tool_handlers::ToolHandlers;
use crate::mcp::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, info};

/// Type-safe wrapper for input token counts to prevent primitive misuse
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputTokens(pub u32);

impl InputTokens {
    /// Create a new InputTokens instance
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    /// Get the raw token count
    pub fn count(self) -> u32 {
        self.0
    }

    /// Add two token counts together
    pub fn add(self, other: InputTokens) -> InputTokens {
        InputTokens(self.0 + other.0)
    }
}

impl From<u32> for InputTokens {
    fn from(count: u32) -> Self {
        Self::new(count)
    }
}

impl std::fmt::Display for InputTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} input tokens", self.0)
    }
}

/// Type-safe wrapper for output token counts to prevent primitive misuse
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputTokens(pub u32);

impl OutputTokens {
    /// Create a new OutputTokens instance
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    /// Get the raw token count
    pub fn count(self) -> u32 {
        self.0
    }

    /// Add two token counts together
    pub fn add(self, other: OutputTokens) -> OutputTokens {
        OutputTokens(self.0 + other.0)
    }
}

impl From<u32> for OutputTokens {
    fn from(count: u32) -> Self {
        Self::new(count)
    }
}

impl std::fmt::Display for OutputTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} output tokens", self.0)
    }
}

/// Combined token count for total calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalTokens {
    pub input: InputTokens,
    pub output: OutputTokens,
}

impl TotalTokens {
    /// Create a new TotalTokens instance
    pub fn new(input: InputTokens, output: OutputTokens) -> Self {
        Self { input, output }
    }

    /// Get the total token count
    pub fn total(self) -> u32 {
        self.input.0 + self.output.0
    }
}

impl std::fmt::Display for TotalTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total: {} ({} input + {} output)",
            self.total(),
            self.input.0,
            self.output.0
        )
    }
}

/// Response type enumeration for mock API behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseType {
    Success,
    Error,
    Timeout,
    RateLimit,
}

/// Probability thresholds for different outcome types
#[derive(Debug, Clone, Copy)]
struct OutcomeProbabilities {
    failure_threshold: u32,
    timeout_threshold: u32,
    rate_limit_threshold: u32,
}

/// Error types for mock MCP operations
#[derive(Debug, Clone)]
pub enum MockMcpError {
    /// Invalid issue name provided
    InvalidIssueName { name: String, reason: String },
    /// MCP handler operation failed
    McpHandlerError { operation: String, error: String },
    /// Failed to record API call with token counts
    ApiRecordingError { operation: String, error: String },
    /// MCP operation failed during execution
    McpOperationError {
        operation_type: String,
        error: String,
    },
    /// Session management error
    SessionError { message: String },
}

impl std::fmt::Display for MockMcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockMcpError::InvalidIssueName { name, reason } => {
                write!(f, "Invalid issue name '{}': {}", name, reason)
            }
            MockMcpError::McpHandlerError { operation, error } => {
                write!(f, "MCP handler error for '{}': {}", operation, error)
            }
            MockMcpError::ApiRecordingError { operation, error } => {
                write!(
                    f,
                    "Failed to record API call for '{}': {}",
                    operation, error
                )
            }
            MockMcpError::McpOperationError {
                operation_type,
                error,
            } => {
                write!(f, "MCP operation '{}' failed: {}", operation_type, error)
            }
            MockMcpError::SessionError { message } => {
                write!(f, "Session error: {}", message)
            }
        }
    }
}

impl std::error::Error for MockMcpError {}

/// Token generation configuration for realistic API simulation
#[derive(Debug, Clone)]
pub struct TokenGenerationConfig {
    /// Base input tokens added to requests
    pub input_tokens_base: InputTokens,
    /// Output tokens for Opus model
    pub output_tokens_opus: OutputTokens,
    /// Output tokens for Sonnet model
    pub output_tokens_sonnet: OutputTokens,
    /// Output tokens for Haiku model
    pub output_tokens_haiku: OutputTokens,
    /// Default output tokens for unknown models
    pub output_tokens_default: OutputTokens,
}

impl Default for TokenGenerationConfig {
    fn default() -> Self {
        Self {
            input_tokens_base: InputTokens::new(50),
            output_tokens_opus: OutputTokens::new(300),
            output_tokens_sonnet: OutputTokens::new(200),
            output_tokens_haiku: OutputTokens::new(150),
            output_tokens_default: OutputTokens::new(200),
        }
    }
}

/// HTTP status code wrapper type to prevent primitive misuse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatusCode(pub u16);

impl HttpStatusCode {
    /// Success status code (200)
    pub const OK: Self = Self(200);
    /// Rate limit error (429)
    pub const TOO_MANY_REQUESTS: Self = Self(429);
    /// Server error (500)
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    /// Timeout error (408)
    pub const REQUEST_TIMEOUT: Self = Self(408);

    /// Get the raw status code value
    pub fn as_u16(self) -> u16 {
        self.0
    }

    /// Check if status code represents success (200-299)
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }
}

/// Configuration for mock Claude API behavior
#[derive(Debug, Clone)]
pub struct MockClaudeApiConfig {
    /// Base latency for API responses (milliseconds)
    pub base_latency_ms: u64,
    /// Additional random latency variance (milliseconds)
    pub latency_variance_ms: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Rate of timeout errors (0.0 to 1.0)
    pub timeout_rate: f64,
    /// Rate of rate limiting errors (0.0 to 1.0)
    pub rate_limit_rate: f64,
    /// Whether to include token usage in responses
    pub include_token_usage: bool,
    /// Whether to include alternative token format
    pub use_alternative_token_format: bool,
    /// Token generation configuration
    pub token_config: TokenGenerationConfig,
}

impl Default for MockClaudeApiConfig {
    fn default() -> Self {
        Self {
            base_latency_ms: 100,
            latency_variance_ms: 50,
            success_rate: 0.95,
            timeout_rate: 0.02,
            rate_limit_rate: 0.02,
            include_token_usage: true,
            use_alternative_token_format: false,
            token_config: TokenGenerationConfig::default(),
        }
    }
}

/// Mock Claude API response data
#[derive(Debug, Clone)]
pub struct MockApiResponse {
    /// HTTP status code
    pub status_code: HttpStatusCode,
    /// Response body JSON
    pub body: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response latency
    pub latency: Duration,
    /// Whether the response succeeded
    pub success: bool,
    /// Input tokens used
    pub input_tokens: InputTokens,
    /// Output tokens generated
    pub output_tokens: OutputTokens,
}

/// Mock Claude API simulator with realistic behavior patterns
#[derive(Debug)]
pub struct MockClaudeApi {
    config: MockClaudeApiConfig,
    call_history: Vec<MockApiCall>,
    current_call_index: u32,
}

/// Record of a mock API call for testing validation
#[derive(Debug, Clone)]
pub struct MockApiCall {
    pub call_index: u32,
    pub endpoint: String,
    pub model: String,
    pub request_body: String,
    pub response: MockApiResponse,
    pub timestamp: Instant,
}

impl MockClaudeApi {
    /// Create a new mock Claude API with default configuration
    pub fn new() -> Self {
        Self::with_config(MockClaudeApiConfig::default())
    }

    /// Create a new mock Claude API with custom configuration
    pub fn with_config(config: MockClaudeApiConfig) -> Self {
        Self {
            config,
            call_history: Vec::new(),
            current_call_index: 0,
        }
    }

    /// Simulate a Claude API call with realistic response patterns
    pub async fn simulate_api_call(
        &mut self,
        endpoint: &str,
        model: &str,
        request_body: &str,
    ) -> MockApiResponse {
        let start_time = Instant::now();
        let call_index = self.current_call_index;
        self.current_call_index += 1;

        // Simulate network latency with variance
        let latency_ms =
            self.config.base_latency_ms + (call_index as u64 % self.config.latency_variance_ms);
        sleep(Duration::from_millis(latency_ms)).await;

        // Determine response outcome based on call index and configuration
        let response = self.generate_response(call_index, model, request_body);

        // Record the call for validation
        let mock_call = MockApiCall {
            call_index,
            endpoint: endpoint.to_string(),
            model: model.to_string(),
            request_body: request_body.to_string(),
            response: response.clone(),
            timestamp: start_time,
        };
        self.call_history.push(mock_call);

        debug!(
            call_index = call_index,
            endpoint = endpoint,
            model = model,
            status_code = response.status_code.as_u16(),
            success = response.success,
            latency_ms = response.latency.as_millis(),
            "Mock API call completed"
        );

        response
    }

    /// Generate a realistic API response based on configuration and probabilistic outcomes
    ///
    /// This function simulates real-world API behavior by generating different response types
    /// based on configured success rates, timeout rates, and rate limiting patterns.
    ///
    /// # Arguments
    /// * `call_index` - Sequential call number for deterministic outcome selection
    /// * `model` - AI model name for response generation and token estimation
    /// * `request_body` - Request content for realistic token count estimation
    ///
    /// # Returns
    /// A `MockApiResponse` with appropriate status code, body content, headers,
    /// latency simulation, and token usage data based on the selected outcome type.
    fn generate_response(
        &self,
        call_index: u32,
        model: &str,
        request_body: &str,
    ) -> MockApiResponse {
        let latency = Duration::from_millis(
            self.config.base_latency_ms + (call_index as u64 % self.config.latency_variance_ms),
        );

        let response_type = self.determine_response_type(call_index);

        match response_type {
            ResponseType::Timeout => self.generate_timeout_response(latency),
            ResponseType::RateLimit => self.generate_rate_limit_response(latency),
            ResponseType::Error => self.generate_error_response(call_index, latency),
            ResponseType::Success => {
                self.generate_success_response(call_index, model, request_body, latency)
            }
        }
    }

    /// Calculate outcome probability thresholds based on configuration rates
    ///
    /// # Returns
    /// A tuple of (failure_threshold, timeout_threshold, rate_limit_threshold)
    /// representing the cumulative probability boundaries for each outcome type.
    fn calculate_outcome_probabilities(&self) -> OutcomeProbabilities {
        let failure_threshold = (100.0 * (1.0 - self.config.success_rate)) as u32;
        let timeout_threshold = (100.0 * self.config.timeout_rate) as u32;
        let rate_limit_threshold = (100.0 * self.config.rate_limit_rate) as u32;

        OutcomeProbabilities {
            failure_threshold,
            timeout_threshold,
            rate_limit_threshold,
        }
    }

    /// Determine response type based on call index and configured rates
    ///
    /// Uses the call index modulo 100 to deterministically select response types:
    /// - Timeout responses: First N% based on `config.timeout_rate`
    /// - Rate limit responses: Next N% based on `config.rate_limit_rate`  
    /// - Error responses: Next N% based on failure rate (1.0 - `config.success_rate`)
    /// - Success responses: Remaining calls
    ///
    /// # Arguments
    /// * `call_index` - Sequential call number for deterministic outcome selection
    ///
    /// # Returns
    /// The `ResponseType` that should be generated for this call.
    fn determine_response_type(&self, call_index: u32) -> ResponseType {
        let probabilities = self.calculate_outcome_probabilities();
        let outcome = call_index % 100;

        if outcome < probabilities.timeout_threshold {
            ResponseType::Timeout
        } else if outcome < probabilities.timeout_threshold + probabilities.rate_limit_threshold {
            ResponseType::RateLimit
        } else if outcome < probabilities.failure_threshold {
            ResponseType::Error
        } else {
            ResponseType::Success
        }
    }

    /// Generate a successful API response with realistic token usage
    fn generate_success_response(
        &self,
        call_index: u32,
        model: &str,
        request_body: &str,
        latency: Duration,
    ) -> MockApiResponse {
        // Generate realistic token counts based on request content
        let input_tokens = self.estimate_input_tokens(request_body);
        let output_tokens = self.generate_output_tokens(call_index, model);

        let body = if self.config.include_token_usage {
            if self.config.use_alternative_token_format {
                self.generate_alternative_format_response(call_index, input_tokens, output_tokens)
            } else {
                self.generate_standard_format_response(call_index, input_tokens, output_tokens)
            }
        } else {
            self.generate_response_without_usage(call_index)
        };

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-request-id".to_string(), format!("req_{}", call_index));

        // Sometimes include token usage in headers as well
        if call_index % 10 == 0 {
            headers.insert(
                "anthropic-input-tokens".to_string(),
                input_tokens.to_string(),
            );
            headers.insert(
                "anthropic-output-tokens".to_string(),
                output_tokens.to_string(),
            );
        }

        MockApiResponse {
            status_code: HttpStatusCode::OK,
            body,
            headers,
            latency,
            success: true,
            input_tokens,
            output_tokens,
        }
    }

    /// Generate standard Claude API response format
    fn generate_standard_format_response(
        &self,
        call_index: u32,
        input_tokens: InputTokens,
        output_tokens: OutputTokens,
    ) -> String {
        self.create_json_response("standard", call_index, Some((input_tokens, output_tokens)))
    }

    /// Create standard Claude API response JSON
    fn create_standard_response_json(
        &self,
        call_index: u32,
        input_tokens: InputTokens,
        output_tokens: OutputTokens,
    ) -> String {
        let content = self.create_response_content_block(
            "This is a mock response for testing API interception",
            call_index,
        );
        let usage_block = self.create_usage_block(input_tokens, output_tokens);

        format!(
            r#"{{
                "id": "msg_mock_{}",
                "type": "message",
                "role": "assistant",
                "content": [{}],
                "model": "claude-3-sonnet-20241022",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                {}
            }}"#,
            call_index, content, usage_block
        )
    }

    /// Create alternative format response JSON
    fn create_alternative_response_json(
        &self,
        call_index: u32,
        input_tokens: InputTokens,
        output_tokens: OutputTokens,
    ) -> String {
        format!(
            r#"{{
                "message_id": "alt_mock_{}",
                "response_text": "Alternative format mock response {}",
                "input_token_count": {},
                "output_token_count": {},
                "total_tokens": {}
            }}"#,
            call_index,
            call_index,
            input_tokens.count(),
            output_tokens.count(),
            input_tokens.count() + output_tokens.count()
        )
    }

    /// Create response without usage data JSON
    fn create_no_usage_response_json(&self, call_index: u32) -> String {
        let content = self.create_simple_content_block(
            "Response without token usage data for testing estimation fallback",
        );

        format!(
            r#"{{
                "id": "msg_no_usage_{}",
                "content": [{}]
            }}"#,
            call_index, content
        )
    }

    /// Create content block with text and call index
    fn create_response_content_block(&self, text: &str, call_index: u32) -> String {
        format!(
            r#"{{
                "type": "text",
                "text": "{} Call index: {}"
            }}"#,
            text, call_index
        )
    }

    /// Create simple content block with just text
    fn create_simple_content_block(&self, text: &str) -> String {
        format!(
            r#"{{
                "text": "{}"
            }}"#,
            text
        )
    }

    /// Create usage block for token information
    fn create_usage_block(&self, input_tokens: InputTokens, output_tokens: OutputTokens) -> String {
        format!(
            r#""usage": {{
                "input_tokens": {},
                "output_tokens": {}
            }}"#,
            input_tokens, output_tokens
        )
    }

    /// Common response builder to reduce duplication
    fn create_json_response(
        &self,
        format_type: &str,
        call_index: u32,
        tokens: Option<(InputTokens, OutputTokens)>,
    ) -> String {
        match format_type {
            "standard" => {
                let (input_tokens, output_tokens) =
                    tokens.unwrap_or((InputTokens::new(0), OutputTokens::new(0)));
                self.create_standard_response_json(call_index, input_tokens, output_tokens)
            }
            "alternative" => {
                let (input_tokens, output_tokens) =
                    tokens.unwrap_or((InputTokens::new(0), OutputTokens::new(0)));
                self.create_alternative_response_json(call_index, input_tokens, output_tokens)
            }
            "no_usage" => self.create_no_usage_response_json(call_index),
            _ => format!(r#"{{"error": "Unknown format type: {}"}}"#, format_type),
        }
    }

    /// Generate alternative token format for testing flexibility
    fn generate_alternative_format_response(
        &self,
        call_index: u32,
        input_tokens: InputTokens,
        output_tokens: OutputTokens,
    ) -> String {
        self.create_json_response(
            "alternative",
            call_index,
            Some((input_tokens, output_tokens)),
        )
    }

    /// Generate response without token usage for fallback testing
    fn generate_response_without_usage(&self, call_index: u32) -> String {
        self.create_json_response("no_usage", call_index, None)
    }

    /// Common error response builder to reduce duplication
    fn create_error_response(
        &self,
        status: HttpStatusCode,
        error_type: &str,
        message: &str,
        latency: Duration,
        extra_headers: Option<HashMap<String, String>>,
    ) -> MockApiResponse {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        if let Some(extra) = extra_headers {
            headers.extend(extra);
        }

        MockApiResponse {
            status_code: status,
            body: format!(
                r#"{{"error": {{"type": "{}", "message": "{}"}}}}"#,
                error_type, message
            ),
            headers,
            latency,
            success: false,
            input_tokens: InputTokens::new(0),
            output_tokens: OutputTokens::new(0),
        }
    }

    /// Generate timeout error response
    fn generate_timeout_response(&self, latency: Duration) -> MockApiResponse {
        self.create_error_response(
            HttpStatusCode::REQUEST_TIMEOUT,
            "timeout_error",
            "Request timeout",
            latency,
            None,
        )
    }

    /// Generate rate limit error response
    fn generate_rate_limit_response(&self, latency: Duration) -> MockApiResponse {
        let mut extra_headers = HashMap::new();
        extra_headers.insert("retry-after".to_string(), "60".to_string());

        self.create_error_response(
            HttpStatusCode::TOO_MANY_REQUESTS,
            "rate_limit_error",
            "Rate limit exceeded",
            latency,
            Some(extra_headers),
        )
    }

    /// Generate generic error response
    fn generate_error_response(&self, call_index: u32, latency: Duration) -> MockApiResponse {
        let error_types = [
            "invalid_request_error",
            "authentication_error",
            "permission_error",
            "not_found_error",
            "overloaded_error",
        ];
        let error_type = error_types[call_index as usize % error_types.len()];

        self.create_error_response(
            HttpStatusCode::INTERNAL_SERVER_ERROR,
            error_type,
            "Mock error for testing",
            latency,
            None,
        )
    }

    /// Estimate input tokens based on request content (simple approximation)
    fn estimate_input_tokens(&self, request_body: &str) -> InputTokens {
        // Simple estimation: ~4 characters per token
        let estimated = ((request_body.len() / 4).max(1)
            + self.config.token_config.input_tokens_base.count() as usize)
            as u32;
        InputTokens::new(estimated)
        // Add base tokens for system prompt
    }

    /// Generate realistic output token counts based on model and call index
    fn generate_output_tokens(&self, call_index: u32, model: &str) -> OutputTokens {
        let base_tokens = match model {
            m if m.contains("claude-3-opus") => self.config.token_config.output_tokens_opus,
            m if m.contains("claude-3-sonnet") => self.config.token_config.output_tokens_sonnet,
            m if m.contains("claude-3-haiku") => self.config.token_config.output_tokens_haiku,
            _ => self.config.token_config.output_tokens_default, // Default for unknown models
        };

        // Add variance based on call index
        OutputTokens::new(base_tokens.count() + (call_index % 200))
    }

    /// Get call history for validation
    pub fn get_call_history(&self) -> &[MockApiCall] {
        &self.call_history
    }

    /// Reset call history and index
    pub fn reset(&mut self) {
        self.call_history.clear();
        self.current_call_index = 0;
    }

    /// Get performance statistics from call history
    pub fn get_performance_stats(&self) -> MockApiPerformanceStats {
        if self.call_history.is_empty() {
            return MockApiPerformanceStats::default();
        }

        let total_calls = self.call_history.len();
        let successful_calls = self
            .call_history
            .iter()
            .filter(|c| c.response.success)
            .count();

        let total_latency: Duration = self.call_history.iter().map(|c| c.response.latency).sum();
        let avg_latency = total_latency / total_calls as u32;

        let total_input_tokens: u32 = self
            .call_history
            .iter()
            .map(|c| c.response.input_tokens.count())
            .sum();
        let total_output_tokens: u32 = self
            .call_history
            .iter()
            .map(|c| c.response.output_tokens.count())
            .sum();

        MockApiPerformanceStats {
            total_calls,
            successful_calls,
            failure_rate: 1.0 - (successful_calls as f64 / total_calls as f64),
            average_latency: avg_latency,
            total_input_tokens,
            total_output_tokens,
            calls_per_second: total_calls as f64 / total_latency.as_secs_f64(),
        }
    }
}

impl Default for MockClaudeApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics from mock API calls
#[derive(Debug, Clone, Default)]
pub struct MockApiPerformanceStats {
    pub total_calls: usize,
    pub successful_calls: usize,
    pub failure_rate: f64,
    pub average_latency: Duration,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub calls_per_second: f64,
}

/// Mock issue storage for testing
pub struct MockIssueStorage {
    issues: RwLock<HashMap<u32, crate::issues::Issue>>,
    next_id: std::sync::atomic::AtomicU32,
}

impl MockIssueStorage {
    pub fn new() -> Self {
        Self {
            issues: RwLock::new(HashMap::new()),
            next_id: std::sync::atomic::AtomicU32::new(1),
        }
    }
}

impl Default for MockIssueStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::issues::IssueStorage for MockIssueStorage {
    async fn create_issue(
        &self,
        name: String,
        content: String,
    ) -> crate::Result<crate::issues::Issue> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let issue = crate::issues::Issue {
            number: crate::issues::filesystem::IssueNumber::from(id),
            name,
            file_path: std::path::PathBuf::from(format!("mock-{}.md", id)),
            completed: false,
            content,
            created_at: chrono::Utc::now(),
        };

        self.issues.write().await.insert(id, issue.clone());
        Ok(issue)
    }

    async fn get_issue(&self, number: u32) -> crate::Result<crate::issues::Issue> {
        self.issues
            .read()
            .await
            .get(&number)
            .cloned()
            .ok_or_else(|| crate::SwissArmyHammerError::IssueNotFound(number.to_string()))
    }

    async fn update_issue(
        &self,
        number: u32,
        content: String,
    ) -> crate::Result<crate::issues::Issue> {
        let mut issues = self.issues.write().await;
        if let Some(issue) = issues.get_mut(&number) {
            issue.content = content;
            Ok(issue.clone())
        } else {
            Err(crate::SwissArmyHammerError::IssueNotFound(
                number.to_string(),
            ))
        }
    }

    async fn mark_complete(&self, number: u32) -> crate::Result<crate::issues::Issue> {
        let mut issues = self.issues.write().await;
        if let Some(issue) = issues.get_mut(&number) {
            issue.completed = true;
            Ok(issue.clone())
        } else {
            Err(crate::SwissArmyHammerError::IssueNotFound(
                number.to_string(),
            ))
        }
    }

    async fn mark_complete_with_cost(&self, number: u32, _cost_data: crate::cost::IssueCostData) -> crate::Result<crate::issues::Issue> {
        // For mock implementation, just mark as complete (cost data handling would be tested separately)
        self.mark_complete(number).await
    }

    async fn list_issues(&self) -> crate::Result<Vec<crate::issues::Issue>> {
        Ok(self.issues.read().await.values().cloned().collect())
    }

    async fn create_issues_batch(
        &self,
        issues: Vec<(String, String)>,
    ) -> crate::Result<Vec<crate::issues::Issue>> {
        let mut results = Vec::new();
        for (name, content) in issues {
            results.push(self.create_issue(name, content).await?);
        }
        Ok(results)
    }

    async fn get_issues_batch(
        &self,
        numbers: Vec<u32>,
    ) -> crate::Result<Vec<crate::issues::Issue>> {
        let mut results = Vec::new();
        for number in numbers {
            results.push(self.get_issue(number).await?);
        }
        Ok(results)
    }

    async fn update_issues_batch(
        &self,
        updates: Vec<(u32, String)>,
    ) -> crate::Result<Vec<crate::issues::Issue>> {
        let mut results = Vec::new();
        for (number, content) in updates {
            results.push(self.update_issue(number, content).await?);
        }
        Ok(results)
    }

    async fn mark_complete_batch(
        &self,
        numbers: Vec<u32>,
    ) -> crate::Result<Vec<crate::issues::Issue>> {
        let mut results = Vec::new();
        for number in numbers {
            results.push(self.mark_complete(number).await?);
        }
        Ok(results)
    }
}

/// Complete mock MCP system for end-to-end testing
pub struct MockMcpSystem {
    pub api: Arc<Mutex<MockClaudeApi>>,
    pub cost_tracking_handler: CostTrackingMcpHandler,
    pub cost_tracker: Arc<Mutex<CostTracker>>,
}

impl MockMcpSystem {
    /// Create a new mock MCP system with default configuration
    pub async fn new() -> Self {
        Self::with_config(MockClaudeApiConfig::default()).await
    }

    /// Create a new mock MCP system with custom API configuration
    pub async fn with_config(api_config: MockClaudeApiConfig) -> Self {
        let api = Arc::new(Mutex::new(MockClaudeApi::with_config(api_config)));
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));

        // Create mock issue storage and tool handlers
        let issue_storage: Arc<RwLock<Box<dyn crate::issues::IssueStorage>>> =
            Arc::new(RwLock::new(Box::new(MockIssueStorage::new())));
        let git_ops = Arc::new(Mutex::new(None::<crate::git::GitOperations>));

        let tool_handlers = ToolHandlers::new(issue_storage, git_ops);
        let cost_tracking_handler =
            CostTrackingMcpHandler::new(tool_handlers, Arc::clone(&cost_tracker));

        Self {
            api,
            cost_tracking_handler,
            cost_tracker,
        }
    }

    /// Start a workflow session with cost tracking
    pub async fn start_workflow_session(
        &self,
        issue_name: &str,
    ) -> Result<CostSessionId, MockMcpError> {
        self.cost_tracking_handler
            .start_session(issue_name)
            .await
            .map_err(|e| MockMcpError::SessionError {
                message: format!("Failed to start session for '{}': {}", issue_name, e),
            })
    }

    /// Complete a workflow session
    pub async fn complete_workflow_session(
        &self,
        status: CostSessionStatus,
    ) -> Result<(), MockMcpError> {
        self.cost_tracking_handler
            .complete_session(status)
            .await
            .map_err(|e| MockMcpError::SessionError {
                message: format!("Failed to complete session: {}", e),
            })
    }

    /// Simulate a complete issue workflow with multiple MCP operations
    pub async fn simulate_issue_workflow(
        &mut self,
        issue_name: &str,
        operations: Vec<McpOperation>,
    ) -> Result<WorkflowResult, MockMcpError> {
        let start_time = Instant::now();

        // Start session
        let session_id = self.start_workflow_session(issue_name).await?;
        info!("Started workflow session for issue: {}", issue_name);

        let mut operation_results = Vec::new();

        // Execute operations
        for operation in operations {
            let result = self.execute_mcp_operation(operation).await?;
            operation_results.push(result);
        }

        // Complete session
        self.complete_workflow_session(CostSessionStatus::Completed)
            .await?;

        // Get session statistics by session ID after completion
        let session_stats = self
            .cost_tracking_handler
            .get_session_stats_by_id(&session_id)
            .await;

        Ok(WorkflowResult {
            session_id,
            duration: start_time.elapsed(),
            operation_results,
            session_stats,
            api_performance_stats: self.get_api_performance_stats().await,
        })
    }

    /// Execute a single MCP operation with mock API interaction
    async fn execute_mcp_operation(
        &mut self,
        operation: McpOperation,
    ) -> Result<McpOperationResult, MockMcpError> {
        let start_time = Instant::now();

        let result = match operation {
            McpOperation::CreateIssue { name, content } => {
                let request = CreateIssueRequest {
                    name: IssueName::new(name.clone()).map_err(|e| {
                        MockMcpError::InvalidIssueName {
                            name: name.clone(),
                            reason: e.to_string(),
                        }
                    })?,
                    content: content.clone(),
                };

                // Simulate API call
                let api_response = self
                    .simulate_api_call_for_operation(
                        "issue_create",
                        &format!("Create issue: {} with content: {}", name, content),
                    )
                    .await;

                // Execute actual MCP operation without cost tracking (avoid double recording)
                self.cost_tracking_handler
                    .inner_handler
                    .handle_issue_create(request)
                    .await
                    .map_err(|e| MockMcpError::McpHandlerError {
                        operation: format!("create issue '{}'", name),
                        error: e.to_string(),
                    })?;

                // Record the API call with realistic token counts from mock API response
                self.cost_tracking_handler
                    .record_mcp_operation_with_tokens(
                        "issue_create",
                        start_time,
                        api_response.success,
                        api_response.input_tokens.count(),
                        api_response.output_tokens.count(),
                    )
                    .await
                    .map_err(|e| MockMcpError::ApiRecordingError {
                        operation: "issue_create".to_string(),
                        error: e.to_string(),
                    })?;

                McpOperationResult {
                    operation_type: "create_issue".to_string(),
                    success: true,
                    duration: start_time.elapsed(),
                    api_response: Some(api_response),
                    error: None,
                }
            }
            McpOperation::UpdateIssue { number, content } => {
                let request = UpdateIssueRequest {
                    number: IssueNumber(number),
                    content: content.clone(),
                    append: false,
                };

                let api_response = self
                    .simulate_api_call_for_operation(
                        "issue_update",
                        &format!("Update issue {} with content: {}", number, content),
                    )
                    .await;

                // Execute actual MCP operation without cost tracking (avoid double recording)
                let mcp_result = self
                    .cost_tracking_handler
                    .inner_handler
                    .handle_issue_update(request)
                    .await;

                // Handle MCP operation result - errors are expected for some test scenarios
                let (success, error_message) = match mcp_result {
                    Ok(_) => (true, None),
                    Err(e) => (
                        false,
                        Some(
                            MockMcpError::McpOperationError {
                                operation_type: "update_issue".to_string(),
                                error: e.to_string(),
                            }
                            .to_string(),
                        ),
                    ),
                };

                // Record the API call with realistic token counts from mock API response
                // Only record successful operations, or record with success=false for failed operations
                let api_success = api_response.success && success; // Both API and MCP must succeed
                self.cost_tracking_handler
                    .record_mcp_operation_with_tokens(
                        "issue_update",
                        start_time,
                        api_success,
                        api_response.input_tokens.count(),
                        api_response.output_tokens.count(),
                    )
                    .await
                    .map_err(|e| MockMcpError::ApiRecordingError {
                        operation: "issue_update".to_string(),
                        error: e.to_string(),
                    })?;

                McpOperationResult {
                    operation_type: "update_issue".to_string(),
                    success,
                    duration: start_time.elapsed(),
                    api_response: Some(api_response),
                    error: error_message,
                }
            }
            McpOperation::MarkComplete { number } => {
                let request = MarkCompleteRequest {
                    number: IssueNumber(number),
                };

                let api_response = self
                    .simulate_api_call_for_operation(
                        "issue_mark_complete",
                        &format!("Mark issue {} complete", number),
                    )
                    .await;

                // Execute actual MCP operation without cost tracking (avoid double recording)
                let mcp_result = self
                    .cost_tracking_handler
                    .inner_handler
                    .handle_issue_mark_complete(request)
                    .await;

                // Handle MCP operation result - errors are expected for some test scenarios
                let (success, error_message) = match mcp_result {
                    Ok(_) => (true, None),
                    Err(e) => (
                        false,
                        Some(
                            MockMcpError::McpOperationError {
                                operation_type: "mark_complete".to_string(),
                                error: e.to_string(),
                            }
                            .to_string(),
                        ),
                    ),
                };

                // Record the API call with realistic token counts from mock API response
                // Only record successful operations, or record with success=false for failed operations
                let api_success = api_response.success && success; // Both API and MCP must succeed
                self.cost_tracking_handler
                    .record_mcp_operation_with_tokens(
                        "issue_mark_complete",
                        start_time,
                        api_success,
                        api_response.input_tokens.count(),
                        api_response.output_tokens.count(),
                    )
                    .await
                    .map_err(|e| MockMcpError::ApiRecordingError {
                        operation: "issue_mark_complete".to_string(),
                        error: e.to_string(),
                    })?;

                McpOperationResult {
                    operation_type: "mark_complete".to_string(),
                    success,
                    duration: start_time.elapsed(),
                    api_response: Some(api_response),
                    error: error_message,
                }
            }
        };

        Ok(result)
    }

    /// Simulate Claude API call for a given MCP operation
    async fn simulate_api_call_for_operation(
        &mut self,
        operation: &str,
        request_content: &str,
    ) -> MockApiResponse {
        let mut api = self.api.lock().await;
        api.simulate_api_call(
            &format!("https://api.anthropic.com/v1/{}", operation),
            "claude-3-sonnet-20241022",
            request_content,
        )
        .await
    }

    /// Get API performance statistics
    pub async fn get_api_performance_stats(&self) -> MockApiPerformanceStats {
        self.api.lock().await.get_performance_stats()
    }

    /// Reset the mock system state
    pub async fn reset(&self) {
        self.api.lock().await.reset();
        // Note: Cost tracker reset would require additional methods
    }
}

/// MCP operation types for workflow simulation
#[derive(Debug, Clone)]
pub enum McpOperation {
    CreateIssue { name: String, content: String },
    UpdateIssue { number: u32, content: String },
    MarkComplete { number: u32 },
}

/// Result of executing an MCP operation
#[derive(Debug, Clone)]
pub struct McpOperationResult {
    pub operation_type: String,
    pub success: bool,
    pub duration: Duration,
    pub api_response: Option<MockApiResponse>,
    pub error: Option<String>,
}

/// Complete workflow execution result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub session_id: CostSessionId,
    pub duration: Duration,
    pub operation_results: Vec<McpOperationResult>,
    pub session_stats: Option<SessionStats>,
    pub api_performance_stats: MockApiPerformanceStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_claude_api() {
        let config = MockClaudeApiConfig {
            success_rate: 1.0,
            timeout_rate: 0.0,
            rate_limit_rate: 0.0,
            ..Default::default()
        };
        let mut api = MockClaudeApi::with_config(config);

        // Test successful API call
        let response = api
            .simulate_api_call(
                "https://api.anthropic.com/v1/messages",
                "claude-3-sonnet",
                "Test request",
            )
            .await;

        assert_eq!(response.status_code, HttpStatusCode::OK);
        assert!(response.input_tokens.count() > 0);
        assert!(response.output_tokens.count() > 0);

        // Test call history
        let history = api.get_call_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].model, "claude-3-sonnet");
    }

    #[tokio::test]
    async fn test_mock_mcp_system_workflow() {
        let mut system = MockMcpSystem::new().await;

        let operations = vec![
            McpOperation::CreateIssue {
                name: "test-issue".to_string(),
                content: "Test issue content".to_string(),
            },
            McpOperation::UpdateIssue {
                number: 1,
                content: "Updated content".to_string(),
            },
            McpOperation::MarkComplete { number: 1 },
        ];

        let result = system
            .simulate_issue_workflow("integration-test", operations)
            .await;

        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        assert_eq!(workflow_result.operation_results.len(), 3);
        assert!(workflow_result.session_stats.is_some());
    }

    #[test]
    fn test_mock_api_config() {
        let config = MockClaudeApiConfig {
            success_rate: 0.8,
            timeout_rate: 0.1,
            rate_limit_rate: 0.05,
            ..Default::default()
        };

        assert_eq!(config.success_rate, 0.8);
        assert_eq!(config.timeout_rate, 0.1);
        assert_eq!(config.rate_limit_rate, 0.05);
    }
}
