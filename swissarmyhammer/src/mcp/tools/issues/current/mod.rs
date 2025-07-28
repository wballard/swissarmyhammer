//! Current issue retrieval tool for MCP operations
//!
//! This module provides the CurrentIssueTool for getting the current issue being worked on.

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
        include_str!("description.md")
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
        let request: CurrentIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_current(request).await
    }
}
