//! Issue creation tool for MCP operations
//!
//! This module provides the CreateIssueTool for creating new issues through the MCP protocol.

#[cfg(not(test))]
use crate::common::rate_limiter::get_rate_limiter;
use crate::mcp::responses::create_issue_response;
use crate::mcp::shared_utils::{McpErrorHandler, McpValidation};
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::CreateIssueRequest;
use crate::mcp::utils::validate_issue_name;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for creating new issues
#[derive(Default)]
pub struct CreateIssueTool;

impl CreateIssueTool {
    /// Creates a new instance of the CreateIssueTool
    pub fn new() -> Self {
        Self
    }

    /// Check rate limit for an MCP operation
    ///
    /// Applies rate limiting to prevent DoS attacks. Uses client identification
    /// based on request context when available, falls back to a default client ID.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation being performed
    /// * `cost` - Token cost of the operation (default: 1, expensive: 2-5)
    /// * `client_id` - Optional client identifier (falls back to "unknown")
    ///
    /// # Returns
    ///
    /// * `Result<(), McpError>` - Ok if operation is allowed, error if rate limited
    fn check_rate_limit(
        &self,
        _operation: &str,
        _cost: u32,
        _client_id: Option<&str>,
    ) -> std::result::Result<(), McpError> {
        // Skip rate limiting in test environment
        #[cfg(test)]
        {
            Ok(())
        }

        #[cfg(not(test))]
        {
            let client = _client_id.unwrap_or("unknown");
            let rate_limiter = get_rate_limiter();

            rate_limiter
                .check_rate_limit(client, _operation, _cost)
                .map_err(|e| {
                    tracing::warn!(
                        "Rate limit exceeded for client '{}', operation '{}': {}",
                        client,
                        _operation,
                        e
                    );
                    McpError::invalid_params(e.to_string(), None)
                })
        }
    }
}

#[async_trait]
impl McpTool for CreateIssueTool {
    fn name(&self) -> &'static str {
        "issue_create"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "create")
            .unwrap_or("Tool description not available")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": ["string", "null"],
                    "description": "Name of the issue (optional for nameless issues)"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content of the issue"
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateIssueRequest = BaseToolImpl::parse_arguments(arguments)?;

        // Apply rate limiting for issue creation
        self.check_rate_limit("issue_create", 1, None)?;

        tracing::debug!("Creating issue: {:?}", request.name);

        // Validate issue name using shared validation logic, or use empty string for nameless issues
        let validated_name = match &request.name {
            Some(name) => {
                McpValidation::validate_not_empty(name.as_str(), "issue name")
                    .map_err(|e| McpErrorHandler::handle_error(e, "validate issue name"))?;
                validate_issue_name(name.as_str())?
            }
            None => String::new(), // Empty name for nameless issues - skip validation
        };

        // Validate content is not empty
        McpValidation::validate_not_empty(&request.content, "issue content")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue content"))?;

        let issue_storage = context.issue_storage.write().await;
        match issue_storage
            .create_issue(validated_name, request.content)
            .await
        {
            Ok(issue) => {
                tracing::info!("Created issue {}", issue.name);
                Ok(create_issue_response(&issue))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "create issue")),
        }
    }
}
