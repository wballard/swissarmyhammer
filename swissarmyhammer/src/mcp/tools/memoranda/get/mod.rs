//! Memo retrieval tool for MCP operations
//!
//! This module provides the GetMemoTool for retrieving a memo by its unique ID through the MCP protocol.

use crate::mcp::memo_types::GetMemoRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for retrieving a memo by its unique ID
#[derive(Default)]
pub struct GetMemoTool;

impl GetMemoTool {
    /// Creates a new instance of the GetMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for GetMemoTool {
    fn name(&self) -> &'static str {
        "memo_get"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ULID identifier of the memo to retrieve"
                }
            },
            "required": ["id"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: GetMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_get(request).await
    }
}
