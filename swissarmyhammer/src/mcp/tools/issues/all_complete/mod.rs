//! Issue completion checking tool for MCP operations
//!
//! This module provides the AllCompleteIssueTool for checking if all issues are completed.

use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::AllCompleteRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for checking if all issues are complete
#[derive(Default)]
pub struct AllCompleteIssueTool;

impl AllCompleteIssueTool {
    /// Creates a new instance of the AllCompleteIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for AllCompleteIssueTool {
    fn name(&self) -> &'static str {
        "issue_all_complete"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "all_complete")
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
        let request: AllCompleteRequest = BaseToolImpl::parse_arguments(arguments)?;
        context
            .tool_handlers
            .handle_issue_all_complete(request)
            .await
    }
}
