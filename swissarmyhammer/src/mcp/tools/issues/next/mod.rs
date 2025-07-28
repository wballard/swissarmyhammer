//! Next issue selection tool for MCP operations
//!
//! This module provides the NextIssueTool for getting the next issue to work on.

use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::NextIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for getting the next issue to work on
#[derive(Default)]
pub struct NextIssueTool;

impl NextIssueTool {
    /// Creates a new instance of the NextIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for NextIssueTool {
    fn name(&self) -> &'static str {
        "issue_next"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "next")
            .unwrap_or("Tool description not available")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: NextIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_next(request).await
    }
}
