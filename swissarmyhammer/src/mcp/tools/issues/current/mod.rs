//! Current issue retrieval tool for MCP operations
//!
//! This module provides the CurrentIssueTool for getting the current issue being worked on.

use crate::config::Config;
use crate::mcp::responses::{create_error_response, create_success_response};
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::CurrentIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for getting the current issue being worked on
#[derive(Default)]
pub struct CurrentIssueTool;

impl CurrentIssueTool {
    /// Creates a new instance of the CurrentIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for CurrentIssueTool {
    fn name(&self) -> &'static str {
        "issue_current"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "current")
            .unwrap_or("Tool description not available")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "branch": {
                    "type": ["string", "null"],
                    "description": "Which branch to check (optional, defaults to current)"
                }
            },
            "required": []
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let _request: CurrentIssueRequest = BaseToolImpl::parse_arguments(arguments)?;

        let git_ops = context.git_ops.lock().await;
        match git_ops.as_ref() {
            Some(ops) => match ops.current_branch() {
                Ok(branch) => {
                    let config = Config::global();
                    if let Some(issue_name) = branch.strip_prefix(&config.issue_branch_prefix) {
                        Ok(create_success_response(format!(
                            "Currently working on issue: {issue_name}"
                        )))
                    } else {
                        Ok(create_success_response(format!(
                            "Not on an issue branch. Current branch: {branch}"
                        )))
                    }
                }
                Err(e) => Ok(create_error_response(format!(
                    "Failed to get current branch: {e}"
                ))),
            },
            None => Ok(create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }
}
