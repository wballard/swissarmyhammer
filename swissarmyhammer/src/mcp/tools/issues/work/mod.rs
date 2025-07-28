//! Issue workflow management tool for MCP operations
//!
//! This module provides the WorkIssueTool for switching to work on a specific issue.

use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::WorkIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for switching to work on an issue
#[derive(Default)]
pub struct WorkIssueTool;

impl WorkIssueTool {
    /// Creates a new instance of the WorkIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for WorkIssueTool {
    fn name(&self) -> &'static str {
        "issue_work"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "work")
            .unwrap_or("Tool description not available")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Issue name to work on"
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
        let request: WorkIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_work(request).await
    }
}
