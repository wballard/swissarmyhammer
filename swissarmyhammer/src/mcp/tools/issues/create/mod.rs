//! Issue creation tool for MCP operations
//!
//! This module provides the CreateIssueTool for creating new issues through the MCP protocol.

use crate::mcp::tool_registry::{BaseToolImpl, McpTool, ToolContext};
use crate::mcp::types::CreateIssueRequest;
use async_trait::async_trait;
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;

/// Tool for creating new issues
#[derive(Default)]
pub struct CreateIssueTool;

impl CreateIssueTool {
    /// Creates a new instance of the CreateIssueTool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for CreateIssueTool {
    fn name(&self) -> &'static str {
        "issue_create"
    }

    fn description(&self) -> &'static str {
        include_str!("description.md")
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": ["string", "null"],
                    "description": "Name of the issue (optional for nameless issues)"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content of the issue"
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_issue_create(request).await
    }
}
