//! Issue updating tool for MCP operations
//!
//! This module provides the UpdateIssueTool for updating existing issue content.

use crate::mcp::tool_registry::{McpTool, ToolContext, BaseToolImpl};
use crate::mcp::types::UpdateIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for updating issue content
pub struct UpdateIssueTool;

impl UpdateIssueTool {
    /// Creates a new instance of the UpdateIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for UpdateIssueTool {
    fn name(&self) -> &'static str {
        "issue_update"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Issue name to update"
                },
                "content": {
                    "type": "string",
                    "description": "New markdown content for the issue"
                },
                "append": {
                    "type": "boolean",
                    "description": "If true, append to existing content instead of replacing",
                    "default": false
                }
            },
            "required": ["name", "content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: UpdateIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_update(request).await
    }
}