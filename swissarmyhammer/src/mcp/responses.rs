//! Response creation utilities for MCP operations

use rmcp::model::*;

/// Create a success response for MCP tool calls
pub fn create_success_response(message: String) -> CallToolResult {
    CallToolResult {
        content: vec![Annotated::new(
            RawContent::Text(RawTextContent { text: message }),
            None,
        )],
        is_error: Some(false),
    }
}

/// Create an error response for MCP tool calls
pub fn create_error_response(message: String) -> CallToolResult {
    CallToolResult {
        content: vec![Annotated::new(
            RawContent::Text(RawTextContent { text: message }),
            None,
        )],
        is_error: Some(true),
    }
}