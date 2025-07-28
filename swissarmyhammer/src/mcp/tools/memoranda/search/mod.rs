//! Memo search tool for MCP operations
//!
//! This module provides the SearchMemoTool for searching memos by query string through the MCP protocol.

use crate::mcp::memo_types::SearchMemosRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for searching memos by query string
#[derive(Default)]
pub struct SearchMemoTool;

impl SearchMemoTool {
    /// Creates a new instance of the SearchMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for SearchMemoTool {
    fn name(&self) -> &'static str {
        "memo_search"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query string to match against memo titles and content"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: SearchMemosRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_search(request).await
    }
}
