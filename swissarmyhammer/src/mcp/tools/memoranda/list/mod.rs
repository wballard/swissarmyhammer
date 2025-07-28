//! Memo listing tool for MCP operations
//!
//! This module provides the ListMemoTool for listing all memos through the MCP protocol.

use crate::mcp::memo_types::ListMemosRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for listing all memos
#[derive(Default)]
pub struct ListMemoTool;

impl ListMemoTool {
    /// Creates a new instance of the ListMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for ListMemoTool {
    fn name(&self) -> &'static str {
        "memo_list"
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
        let request: ListMemosRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_list(request).await
    }
}
