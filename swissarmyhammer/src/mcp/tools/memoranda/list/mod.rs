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
    /// Preview length for memo list operations (characters)
    const MEMO_LIST_PREVIEW_LENGTH: usize = 100;

    /// Creates a new instance of the ListMemoTool
    pub fn new() -> Self {
        Self
    }

    /// Format a memo preview with consistent formatting
    fn format_memo_preview(memo: &crate::memoranda::Memo, preview_length: usize) -> String {
        format!(
            "• {} ({})\n  Created: {}\n  Updated: {}\n  Preview: {}",
            memo.title,
            memo.id,
            crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.created_at),
            crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.updated_at),
            crate::mcp::shared_utils::McpFormatter::format_preview(&memo.content, preview_length)
        )
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
        let _request: ListMemosRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        tracing::debug!("Listing all memos");

        let memo_storage = context.memo_storage.read().await;
        match memo_storage.list_memos().await {
            Ok(memos) => {
                tracing::info!("Retrieved {} memos", memos.len());
                if memos.is_empty() {
                    Ok(BaseToolImpl::create_success_response("No memos found".to_string()))
                } else {
                    let memo_list = memos
                        .iter()
                        .map(|memo| Self::format_memo_preview(memo, Self::MEMO_LIST_PREVIEW_LENGTH))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    let summary = crate::mcp::shared_utils::McpFormatter::format_list_summary("memo", memos.len(), memos.len());
                    Ok(BaseToolImpl::create_success_response(format!(
                        "{summary}:\n\n{memo_list}"
                    )))
                }
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "list memos")),
        }
    }
}
