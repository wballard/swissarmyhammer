//! Issue branch merging tool for MCP operations
//!
//! This module provides the MergeIssueTool for merging issue work branches.

use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::MergeIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for merging an issue work branch
#[derive(Default)]
pub struct MergeIssueTool;

impl MergeIssueTool {
    /// Creates a new instance of the MergeIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for MergeIssueTool {
    fn name(&self) -> &'static str {
        "issue_merge"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Issue name to merge"
                },
                "delete_branch": {
                    "type": "boolean",
                    "description": "Whether to delete the branch after merging",
                    "default": false
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: MergeIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_merge(request).await
    }
}
