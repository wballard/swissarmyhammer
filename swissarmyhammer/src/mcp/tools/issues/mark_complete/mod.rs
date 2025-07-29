//! Issue completion tool for MCP operations
//!
//! This module provides the MarkCompleteIssueTool for marking issues as complete through the MCP protocol.

use crate::mcp::responses::create_mark_complete_response;
use crate::mcp::shared_utils::{McpErrorHandler, McpValidation};
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::MarkCompleteRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for marking issues as complete
#[derive(Default)]
pub struct MarkCompleteIssueTool;

impl MarkCompleteIssueTool {
    /// Creates a new instance of the MarkCompleteIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for MarkCompleteIssueTool {
    fn name(&self) -> &'static str {
        "issue_mark_complete"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "mark_complete")
            .expect("Tool description should be available")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Issue name to mark as complete"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: MarkCompleteRequest = BaseToolImpl::parse_arguments(arguments)?;

        // Validate issue name is not empty
        McpValidation::validate_not_empty(request.name.as_str(), "issue name")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue name"))?;

        let issue_storage = context.issue_storage.write().await;
        match issue_storage.mark_complete(request.name.as_str()).await {
            Ok(issue) => {
                // After successfully marking issue complete, switch back to main branch
                // if we're currently on the issue branch and commit the changes
                let git_ops_guard = context.git_ops.lock().await;
                if let Some(git_ops) = git_ops_guard.as_ref() {
                    let expected_branch = format!("issue/{}", issue.name);
                    match git_ops.current_branch() {
                        Ok(current_branch) => {
                            // If we're on the issue branch, switch back to main
                            if current_branch == expected_branch {
                                match git_ops.main_branch() {
                                    Ok(main_branch) => {
                                        if let Err(e) = git_ops.checkout_branch(&main_branch) {
                                            tracing::warn!(
                                                "Failed to switch back to main branch after completing issue {}: {}",
                                                issue.name, e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to determine main branch after completing issue {}: {}",
                                            issue.name, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to get current branch after completing issue {}: {}",
                                issue.name,
                                e
                            );
                        }
                    }
                }

                Ok(create_mark_complete_response(&issue))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "mark issue complete")),
        }
    }
}
