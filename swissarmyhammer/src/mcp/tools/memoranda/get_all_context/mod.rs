//! Memo get all context tool for MCP operations
//!
//! This module provides the GetAllContextMemoTool for retrieving all memo content formatted for AI context consumption.

use crate::mcp::memo_types::GetAllContextRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for getting all memo content formatted for AI context consumption
#[derive(Default)]
pub struct GetAllContextMemoTool;

impl GetAllContextMemoTool {
    /// Creates a new instance of the GetAllContextMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for GetAllContextMemoTool {
    fn name(&self) -> &'static str {
        "memo_get_all_context"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
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
        let request: GetAllContextRequest = BaseToolImpl::parse_arguments(arguments)?;
        context
            .tool_handlers
            .handle_memo_get_all_context(request)
            .await
    }
}
