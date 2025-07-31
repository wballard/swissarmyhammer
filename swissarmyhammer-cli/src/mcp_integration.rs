//! Integration layer for calling MCP tools from CLI commands
//!
//! This module provides utilities for CLI commands to call MCP tools directly,
//! eliminating code duplication between CLI and MCP implementations.

use serde_json::Map;
use swissarmyhammer::mcp::tool_registry::{ToolContext, ToolRegistry};
use swissarmyhammer::mcp::{register_issue_tools, register_memo_tools, register_search_tools};
use rmcp::model::CallToolResult;
use rmcp::Error as McpError;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// CLI-specific tool context that can create and execute MCP tools
pub struct CliToolContext {
    tool_registry: ToolRegistry,
    tool_context: ToolContext,
}

impl CliToolContext {
    /// Create a new CLI tool context with all necessary storage backends
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize storage backends
        let current_dir = std::env::current_dir()?;
        let issues_dir = current_dir.join("issues");
        
        // Create issue storage
        let issue_storage: Arc<RwLock<Box<dyn swissarmyhammer::issues::IssueStorage>>> = 
            Arc::new(RwLock::new(Box::new(
                swissarmyhammer::issues::FileSystemIssueStorage::new(issues_dir)?
            )));

        // Initialize git operations if available
        let git_ops: Arc<Mutex<Option<swissarmyhammer::git::GitOperations>>> = 
            Arc::new(Mutex::new(
                swissarmyhammer::git::GitOperations::new().ok()
            ));

        // Create memo storage
        let memo_storage: Arc<RwLock<Box<dyn swissarmyhammer::memoranda::MemoStorage>>> = 
            Arc::new(RwLock::new(Box::new(
                swissarmyhammer::memoranda::storage::FileSystemMemoStorage::new(current_dir)
            )));

        // Create tool handlers for backward compatibility
        let tool_handlers = Arc::new(
            swissarmyhammer::mcp::tool_handlers::ToolHandlers::new(memo_storage.clone())
        );

        // Create rate limiter
        let rate_limiter: Arc<dyn swissarmyhammer::common::rate_limiter::RateLimitChecker> = 
            Arc::new(swissarmyhammer::common::rate_limiter::MockRateLimiter);

        // Create tool context
        let tool_context = ToolContext::new(
            tool_handlers,
            issue_storage,
            git_ops,
            memo_storage,
            rate_limiter,
        );

        // Create and populate tool registry
        let mut tool_registry = ToolRegistry::new();
        register_issue_tools(&mut tool_registry);
        register_memo_tools(&mut tool_registry);
        register_search_tools(&mut tool_registry);

        Ok(Self {
            tool_registry,
            tool_context,
        })
    }

    /// Execute an MCP tool with the given arguments
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(tool) = self.tool_registry.get_tool(tool_name) {
            tool.execute(arguments, &self.tool_context).await
        } else {
            Err(McpError::internal_error(
                format!("Tool not found: {tool_name}"),
                None,
            ))
        }
    }

    /// Helper to convert CLI arguments to MCP tool arguments
    pub fn create_arguments(&self, pairs: Vec<(&str, serde_json::Value)>) -> Map<String, serde_json::Value> {
        let mut args = Map::new();
        for (key, value) in pairs {
            args.insert(key.to_string(), value);
        }
        args
    }

    /// Get the list of available tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tool_registry.list_tool_names()
    }

    /// Check if a tool exists
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tool_registry.get_tool(tool_name).is_some()
    }
}

/// Utilities for formatting MCP responses for CLI display
pub mod response_formatting {
    use rmcp::model::{CallToolResult, RawContent};
    use colored::*;
    use serde_json::Value;

    /// Extract and format success message from MCP response
    pub fn format_success_response(result: &CallToolResult) -> String {
        if result.is_error.unwrap_or(false) {
            format_error_response(result)
        } else {
            extract_text_content(result)
                .unwrap_or_else(|| "Operation completed successfully".to_string())
                .green()
                .to_string()
        }
    }

    /// Extract and format error message from MCP response
    pub fn format_error_response(result: &CallToolResult) -> String {
        extract_text_content(result)
            .unwrap_or_else(|| "An unknown error occurred".to_string())
            .red()
            .to_string()
    }

    /// Extract text content from CallToolResult
    fn extract_text_content(result: &CallToolResult) -> Option<String> {
        result.content.first().and_then(|content| {
            match &content.raw {
                RawContent::Text(text_content) => Some(text_content.text.clone()),
                _ => None,
            }
        })
    }

    /// Format structured data as CLI table
    pub fn format_as_table(data: &Value) -> String {
        match data {
            Value::Array(items) => {
                if items.is_empty() {
                    return "No items found".to_string();
                }

                // For simple arrays, just list items
                if items.iter().all(|item| matches!(item, Value::String(_))) {
                    return items
                        .iter()
                        .filter_map(|item| item.as_str())
                        .map(|s| format!("• {s}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                }

                // For complex objects, try to create a simple table representation
                format_object_array(items)
            }
            Value::Object(obj) => {
                format_single_object(obj)
            }
            Value::String(s) => s.clone(),
            _ => serde_json::to_string_pretty(data).unwrap_or_else(|_| "Invalid data".to_string()),
        }
    }

    fn format_object_array(items: &[Value]) -> String {
        if items.is_empty() {
            return "No items found".to_string();
        }

        // Get common keys from first object
        if let Value::Object(first_obj) = &items[0] {
            let keys: Vec<&String> = first_obj.keys().collect();
            
            if keys.is_empty() {
                return "Empty objects".to_string();
            }

            let mut result = String::new();
            
            // Header
            let header_keys: Vec<String> = keys.iter().map(|k| (*k).clone()).collect();
            result.push_str(&header_keys.join(" | "));
            result.push('\n');
            result.push_str(&"-".repeat(result.len() - 1));
            result.push('\n');

            // Rows
            for item in items {
                if let Value::Object(obj) = item {
                    let row: Vec<String> = keys
                        .iter()
                        .map(|key| {
                            obj.get(*key)
                                .map(format_value_for_table)
                                .unwrap_or_else(|| "-".to_string())
                        })
                        .collect();
                    result.push_str(&row.join(" | "));
                    result.push('\n');
                }
            }

            result
        } else {
            // Non-object array
            items
                .iter()
                .map(|item| format!("• {}", format_value_for_table(item)))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn format_single_object(obj: &serde_json::Map<String, Value>) -> String {
        obj.iter()
            .map(|(key, value)| format!("{}: {}", key.bold(), format_value_for_table(value)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_value_for_table(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => format!("[{} items]", arr.len()),
            Value::Object(obj) => format!("{{{} fields}}", obj.len()),
        }
    }

    /// Format CLI-friendly output with optional colors
    pub fn format_cli_output(message: &str, success: bool, use_colors: bool) -> String {
        if use_colors {
            if success {
                message.green().to_string()
            } else {
                message.red().to_string()
            }
        } else {
            message.to_string()
        }
    }

    /// Create a formatted status message
    pub fn create_status_message(operation: &str, success: bool, details: Option<&str>) -> String {
        let status_icon = if success { "✅" } else { "❌" };
        let status_text = if success { "SUCCESS" } else { "ERROR" };
        
        let mut message = format!("{status_icon} {status_text} {operation}");
        
        if let Some(details) = details {
            message.push_str(&format!("\n{details}"));
        }
        
        message
    }
}

/// CLI-specific result type
pub type CliResult<T> = Result<T, CliError>;

/// CLI error type that includes both error information and suggested exit code
#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub exit_code: i32,
}

impl CliError {
    /// Create a new CLI error with a message and exit code
    pub fn new(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }
}

impl From<McpError> for CliError {
    fn from(error: McpError) -> Self {
        CliError::new(format!("MCP error: {error}"), 1)
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliError {}

/// Helper trait for CLI commands to easily call MCP tools
pub trait McpToolRunner {
    /// Execute an MCP tool and return a CLI-formatted result
    fn run_mcp_tool(
        &self,
        context: &CliToolContext,
        tool_name: &str,
        arguments: Map<String, serde_json::Value>,
    ) -> impl std::future::Future<Output = CliResult<String>> + Send;
}

/// Default implementation for any type
impl<T> McpToolRunner for T {
    fn run_mcp_tool(
        &self,
        context: &CliToolContext,
        tool_name: &str,
        arguments: Map<String, serde_json::Value>,
    ) -> impl std::future::Future<Output = CliResult<String>> + Send {
        let future = context.execute_tool(tool_name, arguments);
        async move {
            let result = future.await?;
            
            let output = if result.is_error.unwrap_or(false) {
                response_formatting::format_error_response(&result)
            } else {
                response_formatting::format_success_response(&result)
            };
            
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cli_tool_context_creation() {
        let result = CliToolContext::new().await;
        assert!(result.is_ok(), "Failed to create CliToolContext: {:?}", result.err());
        
        let context = result.unwrap();
        assert!(!context.list_tools().is_empty(), "No tools registered");
    }

    #[tokio::test]
    async fn test_tool_existence_check() {
        let context = CliToolContext::new().await.unwrap();
        
        // Check for known tools
        assert!(context.has_tool("issue_create"), "issue_create tool should exist");
        assert!(context.has_tool("memo_create"), "memo_create tool should exist");
        assert!(!context.has_tool("nonexistent_tool"), "nonexistent tool should not exist");
    }

    #[test]
    fn test_create_arguments() {
        let context = CliToolContext {
            tool_registry: ToolRegistry::new(),
            tool_context: create_mock_tool_context(),
        };
        
        let args = context.create_arguments(vec![
            ("name", json!("test")),
            ("count", json!(42)),
        ]);
        
        assert_eq!(args.get("name"), Some(&json!("test")));
        assert_eq!(args.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_response_formatting() {
        use rmcp::model::{Annotated, RawContent, RawTextContent};
        
        let success_result = CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent {
                    text: "Operation successful".to_string(),
                }),
                None,
            )],
            is_error: Some(false),
        };
        
        let formatted = response_formatting::format_success_response(&success_result);
        assert!(formatted.contains("Operation successful"));
    }

    #[test]
    fn test_format_as_table() {
        let data = json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);
        
        let table = response_formatting::format_as_table(&data);
        assert!(table.contains("name"));
        assert!(table.contains("Alice"));
        assert!(table.contains("Bob"));
    }

    // Helper function for tests
    fn create_mock_tool_context() -> ToolContext {
        use std::path::PathBuf;
        
        let issue_storage: Arc<RwLock<Box<dyn swissarmyhammer::issues::IssueStorage>>> = 
            Arc::new(RwLock::new(Box::new(
                swissarmyhammer::issues::FileSystemIssueStorage::new(PathBuf::from("./test_issues")).unwrap()
            )));
        
        let git_ops: Arc<Mutex<Option<swissarmyhammer::git::GitOperations>>> = 
            Arc::new(Mutex::new(None));
        
        let memo_storage: Arc<RwLock<Box<dyn swissarmyhammer::memoranda::MemoStorage>>> = 
            Arc::new(RwLock::new(Box::new(
                swissarmyhammer::memoranda::storage::FileSystemMemoStorage::new(PathBuf::from("./test_issues"))
            )));
        
        let tool_handlers = Arc::new(
            swissarmyhammer::mcp::tool_handlers::ToolHandlers::new(memo_storage.clone())
        );
        
        let rate_limiter: Arc<dyn swissarmyhammer::common::rate_limiter::RateLimitChecker> = 
            Arc::new(swissarmyhammer::common::rate_limiter::MockRateLimiter);
        
        ToolContext::new(
            tool_handlers,
            issue_storage,
            git_ops,
            memo_storage,
            rate_limiter,
        )
    }
}