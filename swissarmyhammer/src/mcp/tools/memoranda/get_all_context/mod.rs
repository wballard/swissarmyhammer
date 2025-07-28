//! Memo get all context tool for MCP operations
//!
//! This module provides the GetAllContextMemoTool for retrieving all memo content formatted for AI context consumption.

use crate::mcp::memo_types::GetAllContextRequest;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for getting all memo content formatted for AI context consumption
#[derive(Default)]
pub struct GetAllContextMemoTool;

impl GetAllContextMemoTool {
    /// Creates a new instance of the GetAllContextMemoTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for GetAllContextMemoTool {
    fn name(&self) -> &'static str {
        "memo_get_all_context"
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
        let _request: GetAllContextRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        tracing::debug!("Getting all memo context");

        let memo_storage = context.memo_storage.read().await;
        match memo_storage.list_memos().await {
            Ok(memos) => {
                tracing::info!("Retrieved {} memos for context", memos.len());
                if memos.is_empty() {
                    Ok(BaseToolImpl::create_success_response("No memos available".to_string()))
                } else {
                    // Sort memos by updated_at descending (most recent first)
                    let mut sorted_memos = memos;
                    sorted_memos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                    let context = sorted_memos
                        .iter()
                        .map(|memo| {
                            format!(
                                "=== {} (ID: {}) ===\nCreated: {}\nUpdated: {}\n\n{}",
                                memo.title,
                                memo.id,
                                crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.created_at),
                                crate::mcp::shared_utils::McpFormatter::format_timestamp(memo.updated_at),
                                memo.content
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(&format!("\n\n{}\n\n", "=".repeat(80)));

                    let memo_count = sorted_memos.len();
                    let plural_suffix = if memo_count == 1 { "" } else { "s" };
                    Ok(BaseToolImpl::create_success_response(format!(
                        "All memo context ({memo_count} memo{plural_suffix}):\n\n{context}"
                    )))
                }
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(e, "get memo context")),
        }
    }
}
