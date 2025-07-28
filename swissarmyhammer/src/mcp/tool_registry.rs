//! Tool registry for MCP operations
//!
//! This module provides a registry pattern for managing MCP tools, replacing
//! the large match statement with a flexible, extensible system.

use super::memo_types::{
    CreateMemoRequest, DeleteMemoRequest, GetAllContextRequest, GetMemoRequest, ListMemosRequest,
    SearchMemosRequest, UpdateMemoRequest,
};
use super::tool_handlers::ToolHandlers;
use rmcp::model::{Annotated, CallToolResult, RawContent, RawTextContent, Tool};
use rmcp::Error as McpError;
use std::collections::HashMap;
use std::sync::Arc;

/// Context shared by all tools during execution
#[derive(Clone)]
pub struct ToolContext {
    /// The tool handlers instance containing the business logic
    pub tool_handlers: Arc<ToolHandlers>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(tool_handlers: Arc<ToolHandlers>) -> Self {
        Self { tool_handlers }
    }
}

/// Trait defining the interface for all MCP tools
#[async_trait::async_trait]
pub trait McpTool: Send + Sync {
    /// Get the tool's name
    fn name(&self) -> &'static str;

    /// Get the tool's description
    fn description(&self) -> &'static str;

    /// Get the tool's JSON schema for arguments
    fn schema(&self) -> serde_json::Value;

    /// Execute the tool with the given arguments and context
    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError>;
}

/// Registry for managing MCP tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool in the registry
    pub fn register<T: McpTool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Box::new(tool));
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn McpTool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }

    /// List all registered tool names
    pub fn list_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all registered tools as Tool objects for MCP list_tools response
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|tool| {
                let schema = tool.schema();
                let schema_map = if let serde_json::Value::Object(map) = schema {
                    map
                } else {
                    serde_json::Map::new()
                };

                Tool {
                    name: tool.name().into(),
                    description: Some(tool.description().into()),
                    input_schema: std::sync::Arc::new(schema_map),
                    annotations: None,
                }
            })
            .collect()
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

/// Base implementation providing common utility methods for MCP tools
pub struct BaseToolImpl;

impl BaseToolImpl {
    /// Parse tool arguments from a JSON map into a typed struct
    ///
    /// # Arguments
    ///
    /// * `arguments` - The JSON map of arguments from the MCP request
    ///
    /// # Returns
    ///
    /// * `Result<T, McpError>` - The parsed arguments or an error
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> std::result::Result<T, McpError> {
        serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_request(format!("Invalid arguments: {e}"), None))
    }

    /// Create a success response with serializable content
    ///
    /// # Arguments
    ///
    /// * `content` - The content to include in the response
    ///
    /// # Returns
    ///
    /// * `CallToolResult` - A success response
    pub fn create_success_response<T: Into<String>>(content: T) -> CallToolResult {
        CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent {
                    text: content.into(),
                }),
                None,
            )],
            is_error: Some(false),
        }
    }

    /// Create an error response with the given error message
    ///
    /// # Arguments
    ///
    /// * `error` - The error message
    /// * `details` - Optional additional details
    ///
    /// # Returns
    ///
    /// * `CallToolResult` - An error response
    pub fn create_error_response<T: Into<String>>(
        error: T,
        details: Option<String>,
    ) -> CallToolResult {
        let error_text = match details {
            Some(details) => format!("{}: {}", error.into(), details),
            None => error.into(),
        };

        CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent { text: error_text }),
                None,
            )],
            is_error: Some(true),
        }
    }
}

/// Tool registration functions for organizing tools by category
/// Register all issue-related tools with the registry
pub fn register_issue_tools(registry: &mut ToolRegistry) {
    use crate::mcp::tools::issues;
    issues::register_issue_tools(registry);
}

/// Register all memo-related tools with the registry
pub fn register_memo_tools(registry: &mut ToolRegistry) {
    registry.register(MemoCreateTool);
    registry.register(MemoListTool);
    registry.register(MemoGetAllContextTool);
    registry.register(MemoGetTool);
    registry.register(MemoUpdateTool);
    registry.register(MemoDeleteTool);
    registry.register(MemoSearchTool);
}

/// Tool for creating new memos
pub struct MemoCreateTool;

#[async_trait::async_trait]
impl McpTool for MemoCreateTool {
    fn name(&self) -> &'static str {
        "memo_create"
    }

    fn description(&self) -> &'static str {
        "Create a new memo with the given title and content"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title of the memo"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content of the memo"
                }
            },
            "required": ["title", "content"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_create(request).await
    }
}

/// Tool for listing all memos
pub struct MemoListTool;

#[async_trait::async_trait]
impl McpTool for MemoListTool {
    fn name(&self) -> &'static str {
        "memo_list"
    }

    fn description(&self) -> &'static str {
        "List all available memos with their titles, IDs, and content previews"
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
        let request: ListMemosRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_list(request).await
    }
}

/// Tool for getting all memo context
pub struct MemoGetAllContextTool;

#[async_trait::async_trait]
impl McpTool for MemoGetAllContextTool {
    fn name(&self) -> &'static str {
        "memo_get_all_context"
    }

    fn description(&self) -> &'static str {
        "Get all memo content formatted for AI context consumption"
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
        let request: GetAllContextRequest = BaseToolImpl::parse_arguments(arguments)?;
        context
            .tool_handlers
            .handle_memo_get_all_context(request)
            .await
    }
}

/// Tool for getting a memo by ID
pub struct MemoGetTool;

#[async_trait::async_trait]
impl McpTool for MemoGetTool {
    fn name(&self) -> &'static str {
        "memo_get"
    }

    fn description(&self) -> &'static str {
        "Retrieve a memo by its unique ID"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ULID identifier of the memo to retrieve"
                }
            },
            "required": ["id"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: GetMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_get(request).await
    }
}

/// Tool for updating a memo's content
pub struct MemoUpdateTool;

#[async_trait::async_trait]
impl McpTool for MemoUpdateTool {
    fn name(&self) -> &'static str {
        "memo_update"
    }

    fn description(&self) -> &'static str {
        "Update a memo's content by its ID"
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
        context.tool_handlers.handle_memo_update(request).await
    }
}

/// Tool for deleting a memo
pub struct MemoDeleteTool;

#[async_trait::async_trait]
impl McpTool for MemoDeleteTool {
    fn name(&self) -> &'static str {
        "memo_delete"
    }

    fn description(&self) -> &'static str {
        "Delete a memo by its unique ID"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ULID identifier of the memo to delete"
                }
            },
            "required": ["id"]
        })
    }

    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: DeleteMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        context.tool_handlers.handle_memo_delete(request).await
    }
}

/// Tool for searching memos
pub struct MemoSearchTool;

#[async_trait::async_trait]
impl McpTool for MemoSearchTool {
    fn name(&self) -> &'static str {
        "memo_search"
    }

    fn description(&self) -> &'static str {
        "Search memos by query string (searches both title and content)"
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
        context.tool_handlers.handle_memo_search(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{Annotated, RawContent, RawTextContent};

    /// Mock tool for testing
    struct MockTool {
        name: &'static str,
        description: &'static str,
    }

    #[async_trait::async_trait]
    impl McpTool for MockTool {
        fn name(&self) -> &'static str {
            self.name
        }

        fn description(&self) -> &'static str {
            self.description
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
            _arguments: serde_json::Map<String, serde_json::Value>,
            _context: &ToolContext,
        ) -> std::result::Result<CallToolResult, McpError> {
            Ok(CallToolResult {
                content: vec![Annotated::new(
                    RawContent::Text(RawTextContent {
                        text: format!("Mock tool {} executed", self.name),
                    }),
                    None,
                )],
                is_error: Some(false),
            })
        }
    }

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tool = MockTool {
            name: "test_tool",
            description: "A test tool",
        };

        registry.register(tool);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.get_tool("test_tool").is_some());
        assert!(registry.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_tool_lookup() {
        let mut registry = ToolRegistry::new();
        let tool = MockTool {
            name: "lookup_test",
            description: "A lookup test tool",
        };

        registry.register(tool);

        let retrieved_tool = registry.get_tool("lookup_test").unwrap();
        assert_eq!(retrieved_tool.name(), "lookup_test");
        assert_eq!(retrieved_tool.description(), "A lookup test tool");
    }

    #[test]
    fn test_multiple_tool_registration() {
        let mut registry = ToolRegistry::new();

        let tool1 = MockTool {
            name: "tool1",
            description: "First tool",
        };
        let tool2 = MockTool {
            name: "tool2",
            description: "Second tool",
        };

        registry.register(tool1);
        registry.register(tool2);

        assert_eq!(registry.len(), 2);
        assert!(registry.get_tool("tool1").is_some());
        assert!(registry.get_tool("tool2").is_some());

        let tool_names = registry.list_tool_names();
        assert!(tool_names.contains(&"tool1".to_string()));
        assert!(tool_names.contains(&"tool2".to_string()));
    }

    #[tokio::test]
    async fn test_tool_execution() {
        use crate::git::GitOperations;
        use crate::issues::IssueStorage;
        use crate::memoranda::{mock_storage::MockMemoStorage, MemoStorage};
        use std::path::PathBuf;
        use tokio::sync::{Mutex, RwLock};

        // Create mock storage and handlers for context
        let issue_storage: Arc<RwLock<Box<dyn IssueStorage>>> = Arc::new(RwLock::new(Box::new(
            crate::issues::FileSystemIssueStorage::new(PathBuf::from("./test_issues")).unwrap(),
        )));
        let git_ops: Arc<Mutex<Option<GitOperations>>> = Arc::new(Mutex::new(None));
        let memo_storage: Arc<RwLock<Box<dyn MemoStorage>>> =
            Arc::new(RwLock::new(Box::new(MockMemoStorage::new())));

        let tool_handlers = Arc::new(ToolHandlers::new(issue_storage, git_ops, memo_storage));
        let context = ToolContext::new(tool_handlers);

        let tool = MockTool {
            name: "exec_test",
            description: "Execution test tool",
        };

        let result = tool.execute(serde_json::Map::new(), &context).await;
        assert!(result.is_ok());

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());
    }

    #[test]
    fn test_base_tool_impl_parse_arguments() {
        use serde::Deserialize;

        #[derive(Deserialize, PartialEq, Debug)]
        struct TestArgs {
            name: String,
            count: Option<i32>,
        }

        let mut args = serde_json::Map::new();
        args.insert(
            "name".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        args.insert(
            "count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );

        let parsed: TestArgs = BaseToolImpl::parse_arguments(args).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.count, Some(42));
    }

    #[test]
    fn test_base_tool_impl_parse_arguments_error() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestArgs {
            #[serde(rename = "required_field")]
            _required_field: String,
        }

        let args = serde_json::Map::new(); // Missing required field

        let result: std::result::Result<TestArgs, McpError> = BaseToolImpl::parse_arguments(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_base_tool_impl_create_success_response() {
        let response = BaseToolImpl::create_success_response("Success message");

        assert_eq!(response.is_error, Some(false));
        assert_eq!(response.content.len(), 1);

        if let RawContent::Text(text_content) = &response.content[0].raw {
            assert_eq!(text_content.text, "Success message");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_base_tool_impl_create_error_response() {
        let response = BaseToolImpl::create_error_response("Error message", None);

        assert_eq!(response.is_error, Some(true));
        assert_eq!(response.content.len(), 1);

        if let RawContent::Text(text_content) = &response.content[0].raw {
            assert_eq!(text_content.text, "Error message");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_base_tool_impl_create_error_response_with_details() {
        let response = BaseToolImpl::create_error_response(
            "Error message",
            Some("Additional details".to_string()),
        );

        assert_eq!(response.is_error, Some(true));
        assert_eq!(response.content.len(), 1);

        if let RawContent::Text(text_content) = &response.content[0].raw {
            assert_eq!(text_content.text, "Error message: Additional details");
        } else {
            panic!("Expected text content");
        }
    }
}
