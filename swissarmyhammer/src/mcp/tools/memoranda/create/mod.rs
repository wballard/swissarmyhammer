//! Memo creation tool for MCP operations
//!
//! This module provides the CreateMemoTool for creating new memos through the MCP protocol.

use crate::mcp::memo_types::CreateMemoRequest;
use crate::mcp::shared_utils::McpErrorHandler;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for creating new memos
#[derive(Default)]
pub struct CreateMemoTool;

impl CreateMemoTool {
    /// Creates a new instance of the CreateMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for CreateMemoTool {
    fn name(&self) -> &'static str {
        "memo_create"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title of the memo"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content of the memo"
                }
            },
            "required": ["title", "content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        tracing::debug!("Creating memo with title: {}", request.title);

        // Note: Both title and content can be empty - storage layer supports this
        
        let memo_storage = context.memo_storage.write().await;
        match memo_storage
            .create_memo(request.title, request.content)
            .await
        {
            Ok(memo) => {
                tracing::info!("Created memo {}", memo.id);
                Ok(BaseToolImpl::create_success_response(format!(
                    "Successfully created memo '{}' with ID: {}\n\nTitle: {}\nContent: {}",
                    memo.title, memo.id, memo.title, memo.content
                )))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "create memo")),
        }
    }
}
