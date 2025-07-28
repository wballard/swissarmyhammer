//! Issue completion tool for MCP operations
//!
//! This module provides the MarkCompleteIssueTool for marking issues as complete through the MCP protocol.

use crate::mcp::tool_registry::{McpTool, ToolContext, BaseToolImpl};
use crate::mcp::types::MarkCompleteRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for marking issues as complete
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
        include_str!("description.md")
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
        context
            .tool_handlers
            .handle_issue_mark_complete(request)
            .await
    }
}