//! Memo deletion tool for MCP operations
//!
//! This module provides the DeleteMemoTool for deleting memos by their unique ID through the MCP protocol.

use crate::mcp::memo_types::DeleteMemoRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for deleting a memo by its unique ID
#[derive(Default)]
pub struct DeleteMemoTool;

impl DeleteMemoTool {
    /// Creates a new instance of the DeleteMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for DeleteMemoTool {
    fn name(&self) -> &'static str {
        "memo_delete"
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
                    "description": "ULID identifier of the memo to delete"
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
        let request: DeleteMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        tracing::debug!("Deleting memo with ID: {}", request.id);

        let memo_id = match crate::memoranda::MemoId::from_string(request.id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(McpError::invalid_params(
                    format!("Invalid memo ID format: {}", request.id),
                    None,
                ))
            }
        };

        let memo_storage = context.memo_storage.write().await;
        match memo_storage.delete_memo(&memo_id).await {
            Ok(()) => {
                tracing::info!("Deleted memo {}", request.id);
                Ok(BaseToolImpl::create_success_response(format!(
                    "Successfully deleted memo with ID: {}",
                    request.id
                )))
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "delete memo")),
        }
    }
}
