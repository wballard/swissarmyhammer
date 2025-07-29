//! Next issue selection tool for MCP operations
//!
//! This module provides the NextIssueTool for getting the next issue to work on.

use crate::mcp::responses::create_success_response;
use crate::mcp::shared_utils::McpErrorHandler;
use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::NextIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for getting the next issue to work on
#[derive(Default)]
pub struct NextIssueTool;

impl NextIssueTool {
    /// Creates a new instance of the NextIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for NextIssueTool {
    fn name(&self) -> &'static str {
        "issue_next"
    }

    fn description(&self) -> &'static str {
        crate::mcp::tool_descriptions::get_tool_description("issues", "next")
            .expect("Tool description should be available")
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
        let _request: NextIssueRequest = BaseToolImpl::parse_arguments(arguments)?;

        let issue_storage = context.issue_storage.read().await;

        // Use the new get_next_issue method from storage
        match issue_storage.get_next_issue().await {
            Ok(Some(next_issue)) => Ok(create_success_response(format!(
                "Next issue: {}",
                next_issue.name.as_str()
            ))),
            Ok(None) => Ok(create_success_response(
                "No pending issues found. All issues are completed!".to_string(),
            )),
            Err(e) => Err(McpErrorHandler::handle_error(e, "get next issue")),
        }
    }
}
