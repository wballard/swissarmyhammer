//! Cost tracking integration for MCP operations
//!
//! This module provides cost tracking capabilities for MCP handlers by wrapping existing
//! handlers with cost tracking middleware. It captures API calls, token usage, and timing
//! data during MCP operations for integration with the cost tracking system.

use crate::cost::{ApiCall, ApiCallStatus, CostSessionId, CostTracker, IssueId};
use crate::mcp::tool_handlers::ToolHandlers;
use crate::mcp::types::*;
use rmcp::model::*;
use rmcp::Error as McpError;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Default token values for MCP operations
const DEFAULT_MCP_INPUT_TOKENS: u32 = 10;
const DEFAULT_MCP_OUTPUT_TOKENS: u32 = 5;

/// API call record for cost tracking
#[derive(Debug, Clone)]
pub struct ApiCallRecord {
    /// The API endpoint that was called
    pub endpoint: String,
    /// The model used for the API call
    pub model: String,
    /// When the API call started
    pub start_time: Instant,
    /// Number of input tokens used
    pub input_tokens: u32,
    /// Number of output tokens generated
    pub output_tokens: u32,
    /// Status of the API call (success/failure)
    pub status: ApiCallStatus,
    /// Error message if the call failed
    pub error_message: Option<String>,
}

/// Cost tracking wrapper for MCP handlers
///
/// This wrapper implements the decorator pattern to add cost tracking capabilities
/// to existing MCP handlers without modifying their core functionality. It intercepts
/// MCP requests and responses to extract cost information and update the cost tracker.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::cost::CostTracker;
/// use swissarmyhammer::mcp::cost_tracking::CostTrackingMcpHandler;
/// use swissarmyhammer::mcp::tool_handlers::ToolHandlers;
/// # use swissarmyhammer::issues::IssueStorage;
/// use std::sync::Arc;
/// use tokio::sync::{Mutex, RwLock};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # // Mock issue storage for example
/// # struct MockStorage;
/// # #[async_trait::async_trait]
/// # impl IssueStorage for MockStorage {
/// #     async fn create_issue(&self, _name: String, _content: String) -> swissarmyhammer::Result<swissarmyhammer::issues::Issue> { unimplemented!() }
/// #     async fn get_issue(&self, _number: u32) -> swissarmyhammer::Result<swissarmyhammer::issues::Issue> { unimplemented!() }
/// #     async fn update_issue(&self, _number: u32, _content: String) -> swissarmyhammer::Result<swissarmyhammer::issues::Issue> { unimplemented!() }
/// #     async fn mark_complete(&self, _number: u32) -> swissarmyhammer::Result<swissarmyhammer::issues::Issue> { unimplemented!() }
/// #     async fn mark_complete_with_cost(&self, _number: u32, _cost_data: swissarmyhammer::cost::IssueCostData) -> swissarmyhammer::Result<swissarmyhammer::issues::Issue> { unimplemented!() }
/// #     async fn list_issues(&self) -> swissarmyhammer::Result<Vec<swissarmyhammer::issues::Issue>> { unimplemented!() }
/// #     async fn create_issues_batch(&self, _issues: Vec<(String, String)>) -> swissarmyhammer::Result<Vec<swissarmyhammer::issues::Issue>> { unimplemented!() }
/// #     async fn get_issues_batch(&self, _numbers: Vec<u32>) -> swissarmyhammer::Result<Vec<swissarmyhammer::issues::Issue>> { unimplemented!() }
/// #     async fn update_issues_batch(&self, _updates: Vec<(u32, String)>) -> swissarmyhammer::Result<Vec<swissarmyhammer::issues::Issue>> { unimplemented!() }
/// #     async fn mark_complete_batch(&self, _numbers: Vec<u32>) -> swissarmyhammer::Result<Vec<swissarmyhammer::issues::Issue>> { unimplemented!() }
/// # }
/// #
/// let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
/// let tool_handlers = ToolHandlers::new(
///     Arc::new(RwLock::new(Box::new(MockStorage))),
///     Arc::new(Mutex::new(None))
/// );
///
/// let cost_tracking_handler = CostTrackingMcpHandler::new(
///     tool_handlers,
///     cost_tracker
/// );
/// # Ok(())
/// # }
/// ```
pub struct CostTrackingMcpHandler {
    /// The inner MCP handler being wrapped
    pub inner_handler: ToolHandlers,
    /// Shared cost tracker instance
    cost_tracker: Arc<Mutex<CostTracker>>,
    /// Current active session ID (thread-local equivalent)
    active_session: Arc<Mutex<Option<CostSessionId>>>,
}

impl CostTrackingMcpHandler {
    /// Create a new cost tracking MCP handler
    ///
    /// # Arguments
    ///
    /// * `inner_handler` - The MCP handler to wrap with cost tracking
    /// * `cost_tracker` - Shared cost tracker instance for recording API calls
    ///
    /// # Returns
    ///
    /// A new cost tracking handler that wraps the provided handler
    pub fn new(inner_handler: ToolHandlers, cost_tracker: Arc<Mutex<CostTracker>>) -> Self {
        Self {
            inner_handler,
            cost_tracker,
            active_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Start a cost tracking session for an issue
    ///
    /// This method should be called when beginning work on an issue to establish
    /// a cost tracking session. All subsequent API calls will be associated with
    /// this session until it's completed or a new session is started.
    ///
    /// # Arguments
    ///
    /// * `issue_name` - The name/identifier of the issue
    ///
    /// # Returns
    ///
    /// The session ID on success, or an error if session creation fails
    pub async fn start_session(&self, issue_name: &str) -> Result<CostSessionId, String> {
        let issue_id = IssueId::new(issue_name).map_err(|e| format!("Invalid issue ID: {e}"))?;

        let mut tracker = self.cost_tracker.lock().await;
        let session_id = tracker
            .start_session(issue_id)
            .map_err(|e| format!("Failed to start cost session: {e}"))?;

        let mut active_session = self.active_session.lock().await;
        *active_session = Some(session_id);

        info!(
            session_id = %session_id,
            issue_name = issue_name,
            "Started cost tracking session"
        );

        Ok(session_id)
    }

    /// Complete the current cost tracking session
    ///
    /// # Arguments
    ///
    /// * `status` - The final status of the session
    ///
    /// # Returns
    ///
    /// Success if the session was completed, or an error description
    pub async fn complete_session(
        &self,
        status: crate::cost::CostSessionStatus,
    ) -> Result<(), String> {
        let mut active_session = self.active_session.lock().await;

        if let Some(session_id) = *active_session {
            let mut tracker = self.cost_tracker.lock().await;
            tracker
                .complete_session(&session_id, status.clone())
                .map_err(|e| format!("Failed to complete cost session: {e}"))?;

            *active_session = None;

            info!(
                session_id = %session_id,
                ?status,
                "Completed cost tracking session"
            );

            Ok(())
        } else {
            warn!("Attempted to complete session but no active session exists");
            Err("No active cost tracking session".to_string())
        }
    }

    /// Record an API call in the current session with comprehensive error handling
    ///
    /// This method handles the complex process of recording API calls during MCP operations,
    /// including session management, duration calculation, error handling, and graceful
    /// degradation when no active session exists.
    ///
    /// # Process Flow
    /// 1. Checks for an active cost tracking session
    /// 2. Creates and completes an `ApiCall` record with timing and token data
    /// 3. Adds the call to the cost tracker with error conversion
    /// 4. Logs the successful recording with structured fields
    /// 5. Implements graceful degradation if no session is active
    ///
    /// # Graceful Degradation
    /// When no active session exists, the call is logged but not recorded as an error,
    /// allowing the system to continue functioning in environments where cost tracking
    /// is not required or temporarily unavailable.
    ///
    /// # Arguments
    /// * `record` - Complete API call record including endpoint, model, tokens, timing, and status
    ///
    /// # Returns
    /// * `Ok(())` - API call successfully recorded or gracefully handled
    /// * `Err(String)` - Failed to create API call or add to tracker (with detailed error message)
    ///
    /// # Error Handling
    /// Converts internal cost tracking errors to descriptive strings for better debugging
    /// while maintaining the async operation's reliability.
    async fn record_api_call(&self, record: ApiCallRecord) -> Result<(), String> {
        let active_session = self.active_session.lock().await;

        if let Some(session_id) = *active_session {
            drop(active_session);

            let mut api_call = ApiCall::new(&record.endpoint, &record.model)
                .map_err(|e| format!("Failed to create API call record: {e}"))?;

            // Calculate duration
            let duration = record.start_time.elapsed();
            api_call.complete(
                record.input_tokens,
                record.output_tokens,
                record.status.clone(),
                record.error_message.clone(),
            );

            let mut tracker = self.cost_tracker.lock().await;
            let call_id = tracker
                .add_api_call(&session_id, api_call)
                .map_err(|e| format!("Failed to add API call to session: {e}"))?;

            debug!(
                session_id = %session_id,
                call_id = %call_id,
                endpoint = record.endpoint,
                model = record.model,
                input_tokens = record.input_tokens,
                output_tokens = record.output_tokens,
                duration_ms = duration.as_millis(),
                status = ?record.status,
                "Recorded API call"
            );

            Ok(())
        } else {
            info!(
                endpoint = record.endpoint,
                model = record.model,
                input_tokens = record.input_tokens,
                output_tokens = record.output_tokens,
                duration_ms = record.start_time.elapsed().as_millis(),
                status = ?record.status,
                "API call made without active cost tracking session - skipping cost recording. This is normal behavior when cost tracking is disabled or between workflow sessions"
            );
            Ok(()) // Not an error - graceful degradation with better logging
        }
    }

    /// Handle issue creation with cost tracking
    pub async fn handle_issue_create(
        &self,
        request: CreateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_create with cost tracking");

        // Call the inner handler
        let result = self.inner_handler.handle_issue_create(request).await;

        // Record the operation (simulated API call for MCP operations)
        if let Err(e) = self
            .record_mcp_operation("issue_create", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle issue mark complete with cost tracking
    pub async fn handle_issue_mark_complete(
        &self,
        request: MarkCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_mark_complete with cost tracking");

        let result = self.inner_handler.handle_issue_mark_complete(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_mark_complete", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle issue all complete check with cost tracking
    pub async fn handle_issue_all_complete(
        &self,
        request: AllCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_all_complete with cost tracking");

        let result = self.inner_handler.handle_issue_all_complete(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_all_complete", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle issue update with cost tracking
    pub async fn handle_issue_update(
        &self,
        request: UpdateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_update with cost tracking");

        let result = self.inner_handler.handle_issue_update(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_update", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle current issue check with cost tracking
    pub async fn handle_issue_current(
        &self,
        request: CurrentIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_current with cost tracking");

        let result = self.inner_handler.handle_issue_current(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_current", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle issue work with cost tracking
    pub async fn handle_issue_work(
        &self,
        request: WorkIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_work with cost tracking");

        let result = self.inner_handler.handle_issue_work(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_work", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Handle issue merge with cost tracking
    pub async fn handle_issue_merge(
        &self,
        request: MergeIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        debug!("Handling issue_merge with cost tracking");

        let result = self.inner_handler.handle_issue_merge(request).await;

        if let Err(e) = self
            .record_mcp_operation("issue_merge", start_time, result.is_ok())
            .await
        {
            warn!("Failed to record MCP operation: {}", e);
        }

        result
    }

    /// Record an MCP operation as a simulated API call
    ///
    /// Since MCP operations themselves aren't direct API calls to Claude,
    /// we record them as internal operations for comprehensive tracking.
    async fn record_mcp_operation(
        &self,
        operation: &str,
        start_time: Instant,
        success: bool,
    ) -> Result<(), String> {
        let status = if success {
            ApiCallStatus::Success
        } else {
            ApiCallStatus::Failed
        };

        // Simulate token usage for MCP operations (minimal overhead)
        let input_tokens = DEFAULT_MCP_INPUT_TOKENS;
        let output_tokens = DEFAULT_MCP_OUTPUT_TOKENS;

        let record = ApiCallRecord {
            endpoint: format!("mcp://{}", operation),
            model: "mcp-internal".to_string(),
            start_time,
            input_tokens,
            output_tokens,
            status,
            error_message: None,
        };

        self.record_api_call(record).await
    }

    /// Get the current active session ID
    pub async fn get_active_session(&self) -> Option<CostSessionId> {
        *self.active_session.lock().await
    }

    /// Get session statistics for the current active session
    pub async fn get_session_stats(&self) -> Option<SessionStats> {
        let active_session = self.active_session.lock().await;

        if let Some(session_id) = *active_session {
            drop(active_session);

            let tracker = self.cost_tracker.lock().await;
            tracker
                .get_session(&session_id)
                .map(|session| SessionStats {
                    session_id,
                    issue_id: session.issue_id.clone(),
                    api_call_count: session.api_call_count(),
                    total_input_tokens: session.total_input_tokens(),
                    total_output_tokens: session.total_output_tokens(),
                    total_tokens: session.total_tokens(),
                    is_completed: session.is_completed(),
                    duration: session.total_duration,
                })
        } else {
            None
        }
    }

    /// Get session statistics by session ID
    ///
    /// This method can retrieve session statistics for any session by its ID,
    /// regardless of whether it's currently active or has been completed.
    pub async fn get_session_stats_by_id(
        &self,
        session_id: &crate::cost::CostSessionId,
    ) -> Option<SessionStats> {
        let tracker = self.cost_tracker.lock().await;
        tracker.get_session(session_id).map(|session| SessionStats {
            session_id: *session_id,
            issue_id: session.issue_id.clone(),
            api_call_count: session.api_call_count(),
            total_input_tokens: session.total_input_tokens(),
            total_output_tokens: session.total_output_tokens(),
            total_tokens: session.total_tokens(),
            is_completed: session.is_completed(),
            duration: session.total_duration,
        })
    }

    /// Record MCP operation with custom token counts (test-specific method)
    ///
    /// This method is designed for testing scenarios where realistic token counts
    /// are needed instead of the hardcoded DEFAULT_MCP_* values.
    pub async fn record_mcp_operation_with_tokens(
        &self,
        operation: &str,
        start_time: Instant,
        success: bool,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<(), String> {
        let status = if success {
            ApiCallStatus::Success
        } else {
            ApiCallStatus::Failed
        };

        let record = ApiCallRecord {
            endpoint: format!("mcp://{}", operation),
            model: "mcp-internal".to_string(),
            start_time,
            input_tokens,
            output_tokens,
            status,
            error_message: None,
        };

        self.record_api_call(record).await
    }
}

/// Token usage extractor for parsing API responses
///
/// This module provides functionality to extract token usage information
/// from various API response formats, including Claude API responses.
pub mod token_extraction {
    use serde_json::Value;
    use tracing::{debug, warn};

    /// Extracted token usage information
    #[derive(Debug, Clone, PartialEq)]
    pub struct TokenUsage {
        /// Number of input tokens consumed
        pub input_tokens: u32,
        /// Number of output tokens generated
        pub output_tokens: u32,
        /// Total tokens (input + output)
        pub total_tokens: u32,
    }

    impl TokenUsage {
        /// Create new token usage from input and output tokens
        pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
            Self {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            }
        }
    }

    /// Extract token usage from Claude API response
    ///
    /// Parses the usage field from Claude API responses to extract token counts.
    /// Supports both streaming and non-streaming response formats.
    ///
    /// # Arguments
    ///
    /// * `response_body` - The JSON response body from the API
    ///
    /// # Returns
    ///
    /// Extracted token usage or None if extraction fails
    pub fn extract_claude_token_usage(response_body: &str) -> Option<TokenUsage> {
        match serde_json::from_str::<Value>(response_body) {
            Ok(json) => {
                // Try standard Claude API response format
                if let Some(usage) = json.get("usage") {
                    let input_tokens = usage.get("input_tokens")?.as_u64()? as u32;
                    let output_tokens = usage.get("output_tokens")?.as_u64()? as u32;

                    debug!(
                        input_tokens = input_tokens,
                        output_tokens = output_tokens,
                        "Extracted token usage from Claude API response"
                    );

                    return Some(TokenUsage::new(input_tokens, output_tokens));
                }

                // Try alternative format for different API versions
                if let (Some(input), Some(output)) = (
                    json.get("input_token_count"),
                    json.get("output_token_count"),
                ) {
                    let input_tokens = input.as_u64()? as u32;
                    let output_tokens = output.as_u64()? as u32;

                    debug!(
                        input_tokens = input_tokens,
                        output_tokens = output_tokens,
                        "Extracted token usage from alternative API response format"
                    );

                    return Some(TokenUsage::new(input_tokens, output_tokens));
                }

                warn!("No token usage found in Claude API response");
                None
            }
            Err(e) => {
                warn!("Failed to parse API response JSON: {}", e);
                None
            }
        }
    }

    /// Extract token usage from response headers
    ///
    /// Some APIs provide token usage information in response headers.
    /// This function parses common header formats.
    ///
    /// # Arguments
    ///
    /// * `headers` - Map of response headers
    ///
    /// # Returns
    ///
    /// Extracted token usage or None if not found in headers
    pub fn extract_token_usage_from_headers(
        headers: &std::collections::HashMap<String, String>,
    ) -> Option<TokenUsage> {
        // Check for Anthropic-style headers
        if let (Some(input), Some(output)) = (
            headers
                .get("anthropic-input-tokens")
                .or_else(|| headers.get("x-input-tokens")),
            headers
                .get("anthropic-output-tokens")
                .or_else(|| headers.get("x-output-tokens")),
        ) {
            if let (Ok(input_tokens), Ok(output_tokens)) =
                (input.parse::<u32>(), output.parse::<u32>())
            {
                debug!(
                    input_tokens = input_tokens,
                    output_tokens = output_tokens,
                    "Extracted token usage from response headers"
                );

                return Some(TokenUsage::new(input_tokens, output_tokens));
            }
        }

        None
    }

    /// Estimate token usage when exact counts are unavailable
    ///
    /// Provides rough estimates based on text length when API doesn't
    /// return exact token counts. This is a fallback mechanism.
    ///
    /// # Arguments
    ///
    /// * `input_text` - The input text sent to the API
    /// * `output_text` - The output text received from the API
    ///
    /// # Returns
    ///
    /// Estimated token usage based on text length
    pub fn estimate_token_usage(input_text: &str, output_text: &str) -> TokenUsage {
        // Rough estimation: ~4 characters per token for English text
        let input_tokens = (input_text.len() / 4).max(1) as u32;
        let output_tokens = (output_text.len() / 4).max(1) as u32;

        debug!(
            input_tokens = input_tokens,
            output_tokens = output_tokens,
            "Estimated token usage from text length"
        );

        TokenUsage::new(input_tokens, output_tokens)
    }
}

/// Session statistics summary
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Unique identifier for the cost session
    pub session_id: CostSessionId,
    /// Associated issue identifier
    pub issue_id: IssueId,
    /// Number of API calls made in this session
    pub api_call_count: usize,
    /// Total input tokens consumed in this session
    pub total_input_tokens: u32,
    /// Total output tokens generated in this session
    pub total_output_tokens: u32,
    /// Total tokens (input + output) for this session
    pub total_tokens: u32,
    /// Whether the session has been completed
    pub is_completed: bool,
    /// Duration of the session if completed
    pub duration: Option<std::time::Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::CostSessionStatus;
    use crate::git::GitOperations;
    use crate::issues::IssueStorage;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    // Mock issue storage for testing
    struct MockIssueStorage;

    #[async_trait::async_trait]
    impl IssueStorage for MockIssueStorage {
        async fn create_issue(
            &self,
            _name: String,
            _content: String,
        ) -> crate::Result<crate::issues::Issue> {
            use chrono::Utc;

            Ok(crate::issues::Issue {
                number: crate::issues::filesystem::IssueNumber::from(1),
                name: "test-issue".to_string(),
                file_path: std::path::PathBuf::from("test.md"),
                completed: false,
                content: "test-content".to_string(),
                created_at: Utc::now(),
            })
        }

        async fn get_issue(&self, _number: u32) -> crate::Result<crate::issues::Issue> {
            use chrono::Utc;

            Ok(crate::issues::Issue {
                number: crate::issues::filesystem::IssueNumber::from(1),
                name: "test-issue".to_string(),
                file_path: std::path::PathBuf::from("test.md"),
                completed: false,
                content: "test-content".to_string(),
                created_at: Utc::now(),
            })
        }

        async fn update_issue(
            &self,
            _number: u32,
            _content: String,
        ) -> crate::Result<crate::issues::Issue> {
            use chrono::Utc;

            Ok(crate::issues::Issue {
                number: crate::issues::filesystem::IssueNumber::from(1),
                name: "test-issue".to_string(),
                file_path: std::path::PathBuf::from("test.md"),
                completed: false,
                content: _content,
                created_at: Utc::now(),
            })
        }

        async fn mark_complete(&self, _number: u32) -> crate::Result<crate::issues::Issue> {
            use chrono::Utc;

            Ok(crate::issues::Issue {
                number: crate::issues::filesystem::IssueNumber::from(1),
                name: "test-issue".to_string(),
                file_path: std::path::PathBuf::from("test.md"),
                completed: true,
                content: "test-content".to_string(),
                created_at: Utc::now(),
            })
        }

        async fn mark_complete_with_cost(
            &self,
            number: u32,
            _cost_data: crate::cost::IssueCostData,
        ) -> crate::Result<crate::issues::Issue> {
            // For mock implementation, just call mark_complete
            self.mark_complete(number).await
        }

        async fn list_issues(&self) -> crate::Result<Vec<crate::issues::Issue>> {
            Ok(vec![])
        }

        async fn create_issues_batch(
            &self,
            issues: Vec<(String, String)>,
        ) -> crate::Result<Vec<crate::issues::Issue>> {
            use chrono::Utc;

            let mut result = Vec::new();
            for (i, (name, content)) in issues.into_iter().enumerate() {
                result.push(crate::issues::Issue {
                    number: crate::issues::filesystem::IssueNumber::from(i as u32 + 1),
                    name,
                    file_path: std::path::PathBuf::from(format!("test-{}.md", i + 1)),
                    completed: false,
                    content,
                    created_at: Utc::now(),
                });
            }
            Ok(result)
        }

        async fn get_issues_batch(
            &self,
            numbers: Vec<u32>,
        ) -> crate::Result<Vec<crate::issues::Issue>> {
            use chrono::Utc;

            let mut result = Vec::new();
            for number in numbers {
                result.push(crate::issues::Issue {
                    number: crate::issues::filesystem::IssueNumber::from(number),
                    name: format!("test-issue-{}", number),
                    file_path: std::path::PathBuf::from(format!("test-{}.md", number)),
                    completed: false,
                    content: "test-content".to_string(),
                    created_at: Utc::now(),
                });
            }
            Ok(result)
        }

        async fn update_issues_batch(
            &self,
            updates: Vec<(u32, String)>,
        ) -> crate::Result<Vec<crate::issues::Issue>> {
            use chrono::Utc;

            let mut result = Vec::new();
            for (number, content) in updates {
                result.push(crate::issues::Issue {
                    number: crate::issues::filesystem::IssueNumber::from(number),
                    name: format!("test-issue-{}", number),
                    file_path: std::path::PathBuf::from(format!("test-{}.md", number)),
                    completed: false,
                    content,
                    created_at: Utc::now(),
                });
            }
            Ok(result)
        }

        async fn mark_complete_batch(
            &self,
            numbers: Vec<u32>,
        ) -> crate::Result<Vec<crate::issues::Issue>> {
            use chrono::Utc;

            let mut result = Vec::new();
            for number in numbers {
                result.push(crate::issues::Issue {
                    number: crate::issues::filesystem::IssueNumber::from(number),
                    name: format!("test-issue-{}", number),
                    file_path: std::path::PathBuf::from(format!("test-{}.md", number)),
                    completed: true,
                    content: "test-content".to_string(),
                    created_at: Utc::now(),
                });
            }
            Ok(result)
        }
    }

    async fn create_test_handler() -> CostTrackingMcpHandler {
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
        let issue_storage: Arc<RwLock<Box<dyn IssueStorage>>> =
            Arc::new(RwLock::new(Box::new(MockIssueStorage)));
        let git_ops = Arc::new(Mutex::new(None::<GitOperations>));

        let tool_handlers = ToolHandlers::new(issue_storage, git_ops);

        CostTrackingMcpHandler::new(tool_handlers, cost_tracker)
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let handler = create_test_handler().await;

        // Start a session
        let session_id = handler.start_session("test-issue").await.unwrap();
        assert!(handler.get_active_session().await.is_some());

        // Get session stats
        let stats = handler.get_session_stats().await.unwrap();
        assert_eq!(stats.session_id, session_id);
        assert_eq!(stats.api_call_count, 0);
        assert_eq!(stats.total_tokens, 0);

        // Complete the session
        handler
            .complete_session(CostSessionStatus::Completed)
            .await
            .unwrap();
        assert!(handler.get_active_session().await.is_none());
    }

    #[tokio::test]
    async fn test_mcp_operations_with_cost_tracking() {
        let handler = create_test_handler().await;

        // Start a session
        handler.start_session("test-issue").await.unwrap();

        // Perform MCP operations
        let create_request = CreateIssueRequest {
            name: IssueName::new("test-issue".to_string()).unwrap(),
            content: "Test content".to_string(),
        };

        let result = handler.handle_issue_create(create_request).await;
        assert!(result.is_ok());

        // Check that the operation was recorded
        let stats = handler.get_session_stats().await.unwrap();
        assert_eq!(stats.api_call_count, 1); // One MCP operation recorded

        handler
            .complete_session(CostSessionStatus::Completed)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_token_extraction() {
        use super::token_extraction::*;

        // Test Claude API response format
        let response_json = r#"{
            "id": "msg_123",
            "content": [{"text": "Hello world"}],
            "usage": {
                "input_tokens": 150,
                "output_tokens": 25
            }
        }"#;

        let usage = extract_claude_token_usage(response_json).unwrap();
        assert_eq!(usage.input_tokens, 150);
        assert_eq!(usage.output_tokens, 25);
        assert_eq!(usage.total_tokens, 175);

        // Test alternative format
        let alt_response_json = r#"{
            "input_token_count": 100,
            "output_token_count": 50
        }"#;

        let usage = extract_claude_token_usage(alt_response_json).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);

        // Test header extraction
        let mut headers = std::collections::HashMap::new();
        headers.insert("anthropic-input-tokens".to_string(), "200".to_string());
        headers.insert("anthropic-output-tokens".to_string(), "75".to_string());

        let usage = extract_token_usage_from_headers(&headers).unwrap();
        assert_eq!(usage.input_tokens, 200);
        assert_eq!(usage.output_tokens, 75);

        // Test estimation
        let usage = estimate_token_usage("Hello world", "Hi there");
        assert!(usage.input_tokens > 0);
        assert!(usage.output_tokens > 0);
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        let handler = create_test_handler().await;

        // Perform operations without starting a session
        let create_request = CreateIssueRequest {
            name: IssueName::new("test-issue".to_string()).unwrap(),
            content: "Test content".to_string(),
        };

        // This should succeed even without a session (graceful degradation)
        let result = handler.handle_issue_create(create_request).await;
        assert!(result.is_ok());

        // No active session should exist
        assert!(handler.get_active_session().await.is_none());
        assert!(handler.get_session_stats().await.is_none());
    }
}
