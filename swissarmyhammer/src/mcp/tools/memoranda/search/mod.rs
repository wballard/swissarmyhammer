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
    /// Preview length for memo search operations (characters)
    const MEMO_SEARCH_PREVIEW_LENGTH: usize = 200;

    /// Creates a new instance of the SearchMemoTool
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
        
        tracing::debug!("Searching memos with query: {}", request.query);

        // Validate search query is not empty
        crate::mcp::shared_utils::McpValidation::validate_not_empty(&request.query, "search query")
            .map_err(|e| crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "validate search query"))?;

        let memo_storage = context.memo_storage.read().await;
        match memo_storage.search_memos(&request.query).await {
            Ok(memos) => {
                tracing::info!("Search returned {} memos", memos.len());
                if memos.is_empty() {
                    Ok(BaseToolImpl::create_success_response(format!(
                        "No memos found matching query: '{}'",
                        request.query
                    )))
                } else {
                    let memo_list = memos
                        .iter()
                        .map(|memo| Self::format_memo_preview(memo, Self::MEMO_SEARCH_PREVIEW_LENGTH))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    Ok(BaseToolImpl::create_success_response(format!(
                        "Found {} memo{} matching '{}':\n\n{}",
                        memos.len(),
                        if memos.len() == 1 { "" } else { "s" },
                        request.query,
                        memo_list
                    )))
                }
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "search memos")),
        }
    }
}
