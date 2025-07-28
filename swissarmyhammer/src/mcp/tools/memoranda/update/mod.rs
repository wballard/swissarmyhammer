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
        
        tracing::debug!("Updating memo with ID: {}", request.id);

        // Validate memo content using shared validation
        crate::mcp::shared_utils::McpValidation::validate_not_empty(&request.content, "memo content")
            .map_err(|e| crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "validate memo content"))?;

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
        match memo_storage.update_memo(&memo_id, request.content).await {
            Ok(memo) => {
                tracing::info!("Updated memo {}", memo.id);
                Ok(BaseToolImpl::create_success_response(format!(
                    "Successfully updated memo:\n\nID: {}\nTitle: {}\nUpdated: {}\n\nContent:\n{}",
                    memo.id,
                    memo.title,
                    crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.updated_at),
                    memo.content
                )))
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "update memo")),
        }
    }
}
