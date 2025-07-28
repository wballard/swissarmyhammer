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
        
        tracing::debug!("Getting memo with ID: {}", request.id);

        let memo_id = match crate::memoranda::MemoId::from_string(request.id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(McpError::invalid_params(
                    format!("Invalid memo ID format: {}", request.id),
                    None,
                ))
            }
        };

        let memo_storage = context.memo_storage.read().await;
        match memo_storage.get_memo(&memo_id).await {
            Ok(memo) => {
                tracing::info!("Retrieved memo {}", memo.id);
                Ok(BaseToolImpl::create_success_response(format!(
                    "Memo found:\n\nID: {}\nTitle: {}\nCreated: {}\nUpdated: {}\n\nContent:\n{}",
                    memo.id,
                    memo.title,
                    crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.created_at),
                    crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.updated_at),
                    memo.content
                )))
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "get memo")),
        }
    }
}
