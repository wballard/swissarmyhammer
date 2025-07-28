//! Memo update tool for MCP operations
//!
//! This module provides the UpdateMemoTool for updating memo content by ID through the MCP protocol.

use crate::mcp::memo_types::UpdateMemoRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for updating a memo's content by its ID
#[derive(Default)]
pub struct UpdateMemoTool;

impl UpdateMemoTool {
    /// Creates a new instance of the UpdateMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for UpdateMemoTool {
    fn name(&self) -> &'static str {
        "memo_update"
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
                    "description": "ULID identifier of the memo to update"
                },
                "content": {
                    "type": "string",
                    "description": "New markdown content for the memo"
                }
            },
            "required": ["id", "content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: UpdateMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_update(request).await
    }
}
