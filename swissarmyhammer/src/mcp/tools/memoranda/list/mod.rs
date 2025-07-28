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
                    Ok(BaseToolImpl::create_success_response(
                        "No memos found".to_string(),
                    ))
                } else {
                    let memo_list = memos
                        .iter()
                        .map(|memo| Self::format_memo_preview(memo, Self::MEMO_LIST_PREVIEW_LENGTH))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    let summary = crate::mcp::shared_utils::McpFormatter::format_list_summary(
                        "memo",
                        memos.len(),
                        memos.len(),
                    );
                    Ok(BaseToolImpl::create_success_response(format!(
                        "{summary}:\n\n{memo_list}"
                    )))
                }
            }
            Err(e) => Err(crate::mcp::shared_utils::McpErrorHandler::handle_error(
                e,
                "list memos",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitOperations;
    use crate::issues::IssueStorage;
    use crate::mcp::tool_handlers::ToolHandlers;
    use crate::mcp::tool_registry::ToolContext;
    use crate::memoranda::{mock_storage::MockMemoStorage, MemoStorage};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    async fn create_test_context() -> ToolContext {
        let issue_storage: Arc<RwLock<Box<dyn IssueStorage>>> = Arc::new(RwLock::new(Box::new(
            crate::issues::FileSystemIssueStorage::new(PathBuf::from("./test_issues")).unwrap(),
        )));
        let git_ops: Arc<Mutex<Option<GitOperations>>> = Arc::new(Mutex::new(None));
        let memo_storage: Arc<RwLock<Box<dyn MemoStorage>>> =
            Arc::new(RwLock::new(Box::new(MockMemoStorage::new())));

        let tool_handlers = Arc::new(ToolHandlers::new(
            issue_storage.clone(),
            git_ops.clone(),
            memo_storage.clone(),
        ));

        ToolContext::new(tool_handlers, issue_storage, git_ops, memo_storage)
    }

    #[test]
    fn test_list_memo_tool_new() {
        let tool = ListMemoTool::new();
        assert_eq!(tool.name(), "memo_list");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_list_memo_tool_schema() {
        let tool = ListMemoTool::new();
        let schema = tool.schema();

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], serde_json::json!({}));
        assert_eq!(schema["required"], serde_json::json!([]));
    }

    #[test]
    fn test_format_memo_preview() {
        use crate::memoranda::{Memo, MemoId};
        use chrono::Utc;

        let memo = Memo {
            id: MemoId::new(),
            title: "Test Memo".to_string(),
            content: "This is a long piece of content that should be truncated in the preview to show only the first part".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let preview = ListMemoTool::format_memo_preview(&memo, 50);
        assert!(preview.contains("Test Memo"));
        assert!(preview.contains("Created:"));
        assert!(preview.contains("Updated:"));
        assert!(preview.contains("Preview:"));
    }

    #[tokio::test]
    async fn test_list_memo_tool_execute_empty_list() {
        let tool = ListMemoTool::new();
        let context = create_test_context().await;

        let arguments = serde_json::Map::new();

        let result = tool.execute(arguments, &context).await;
        assert!(result.is_ok());

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_list_memo_tool_execute_with_memos() {
        let tool = ListMemoTool::new();
        let context = create_test_context().await;

        // Create some test memos
        let memo_storage = context.memo_storage.write().await;
        memo_storage
            .create_memo("First Memo".to_string(), "First content".to_string())
            .await
            .unwrap();
        memo_storage
            .create_memo("Second Memo".to_string(), "Second content".to_string())
            .await
            .unwrap();
        drop(memo_storage); // Release the lock

        let arguments = serde_json::Map::new();

        let result = tool.execute(arguments, &context).await;
        assert!(result.is_ok());

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_list_memo_tool_execute_with_invalid_arguments() {
        let tool = ListMemoTool::new();
        let context = create_test_context().await;

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "invalid_field".to_string(),
            serde_json::Value::String("invalid".to_string()),
        );

        let result = tool.execute(arguments, &context).await;
        // Should succeed because the schema allows extra fields and the parsing ignores unknown fields
        assert!(result.is_ok());
    }
}
