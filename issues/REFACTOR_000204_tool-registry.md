# REFACTOR Step 2: Implement Tool Registry Pattern

## Overview
Replace the large match statement in `call_tool` with a flexible tool registry system that can dynamically register and discover tools.

## Context
Currently, the `call_tool` method in `mcp.rs` has a large match statement with hardcoded tool names:

```rust
match request.name.as_ref() {
    "issue_create" => { /* ... */ },
    "issue_mark_complete" => { /* ... */ },
    "memo_create" => { /* ... */ },
    // ... many more cases
}
```

This approach:
- Is difficult to maintain as tools are added
- Creates tight coupling between tool definitions and the main MCP handler
- Makes testing individual tools harder
- Duplicates parameter parsing and error handling code

## Target Architecture

### Tool Registry Trait
Create a registry pattern similar to the existing `PromptLibrary`:

```rust
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    
    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self { /* ... */ }
    pub fn register<T: McpTool + 'static>(&mut self, tool: T) { /* ... */ }
    pub fn get_tool(&self, name: &str) -> Option<&dyn McpTool> { /* ... */ }
    pub fn list_tools(&self) -> Vec<Tool> { /* ... */ }
}
```

### Tool Context
Provide shared context for tool execution:

```rust
pub struct ToolContext {
    pub issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    pub git_ops: Arc<Mutex<Option<GitOperations>>>,
    pub memo_storage: Arc<RwLock<Box<dyn MemoStorage>>>,
    // Add search storage when implemented
}
```

## Tasks for This Step

### 1. Define Core Tool Traits and Registry
- Create `McpTool` trait with required methods
- Implement `ToolRegistry` struct with registration and lookup methods
- Create `ToolContext` struct for shared dependencies
- Add helper methods for common operations (parameter parsing, error handling)

### 2. Create Base Tool Implementation
Provide base implementations and utilities:

```rust
pub struct BaseToolImpl;

impl BaseToolImpl {
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(
        arguments: serde_json::Map<String, serde_json::Value>
    ) -> Result<T, McpError> {
        // Common parameter parsing logic
    }
    
    pub fn create_success_response<T: serde::Serialize>(
        content: T
    ) -> CallToolResult {
        // Common success response creation
    }
    
    pub fn create_error_response(
        error: &str,
        details: Option<String>
    ) -> CallToolResult {
        // Common error response creation
    }
}
```

### 3. Update McpServer to Use Registry
Modify the `McpServer` struct to use the tool registry:

```rust
impl McpServer {
    pub fn new(/* ... */) -> Result<Self> {
        let mut registry = ToolRegistry::new();
        
        // Register all tools
        register_issue_tools(&mut registry);
        register_memo_tools(&mut registry);
        register_search_tools(&mut registry);
        
        Ok(Self {
            // ... other fields
            tool_registry: Arc::new(registry),
            tool_context: Arc::new(ToolContext { /* ... */ }),
        })
    }
}

#[async_trait]
impl ServerHandler for McpServer {
    async fn call_tool(&self, request: CallToolRequestParam, _context: RequestContext<RoleServer>) 
        -> std::result::Result<CallToolResult, McpError> 
    {
        if let Some(tool) = self.tool_registry.get_tool(&request.name) {
            tool.execute(
                request.arguments.unwrap_or_default(),
                &self.tool_context
            ).await
        } else {
            Err(McpError::invalid_request(
                format!("Unknown tool: {}", request.name),
                None,
            ))
        }
    }
    
    async fn list_tools(&self, _request: ListToolsRequest, _context: RequestContext<RoleServer>) 
        -> std::result::Result<ListToolsResult, McpError>
    {
        Ok(ListToolsResult {
            tools: self.tool_registry.list_tools(),
            next_cursor: None,
        })
    }
}
```

### 4. Create Registration Functions
Create separate registration functions for each tool category:

```rust
pub fn register_issue_tools(registry: &mut ToolRegistry) {
    // Will be implemented in subsequent steps
}

pub fn register_memo_tools(registry: &mut ToolRegistry) {
    // Will be implemented in subsequent steps
}

pub fn register_search_tools(registry: &mut ToolRegistry) {
    // Will be implemented in subsequent steps
}
```

### 5. Comprehensive Testing
- Unit tests for `ToolRegistry` functionality
- Integration tests with mock tools
- Performance tests to ensure registry lookup is fast
- Backward compatibility tests to ensure no regressions

## Benefits of This Approach
1. **Extensibility**: New tools can be added without modifying core MCP logic
2. **Testability**: Each tool can be tested in isolation
3. **Maintainability**: Tool logic is separated from protocol handling
4. **Consistency**: Common patterns for parameter parsing and error handling
5. **Dynamic Discovery**: Tools can be discovered and documented automatically

## Proposed Solution

After analyzing the current codebase, I understand that there's already a `ToolHandlers` struct with individual handler methods for each tool. The `McpServer` has a large match statement that parses arguments and delegates to these handlers.

### Implementation Strategy

1. **Create Core Abstractions**: Define the `McpTool` trait and `ToolRegistry` as specified
2. **Leverage Existing Handlers**: Create individual tool implementations that wrap the existing `ToolHandlers` methods initially
3. **Incremental Migration**: Replace the match statement with registry lookup while maintaining backward compatibility
4. **Shared Dependencies**: Use `ToolContext` to provide access to storage and git operations

### Detailed Implementation Plan

#### Phase 1: Core Infrastructure
```rust
// Create new module: swissarmyhammer/src/mcp/tool_registry.rs
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    
    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError>;
}

pub struct ToolContext {
    pub tool_handlers: Arc<ToolHandlers>,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}
```

#### Phase 2: Individual Tool Implementations
Create wrapper implementations for each existing tool, starting with:
- `IssueCreateTool` -> wraps `ToolHandlers::handle_issue_create`
- `IssueMarkCompleteTool` -> wraps `ToolHandlers::handle_issue_mark_complete`
- `MemoCreateTool` -> wraps `ToolHandlers::handle_memo_create`
- etc.

#### Phase 3: Registry Integration
Update `McpServer` to use the registry while keeping the existing `ToolHandlers` as the implementation backend.

### Benefits of This Approach
- **Minimal Risk**: Existing handlers remain unchanged initially
- **Gradual Migration**: Can migrate one tool at a time
- **Testable**: Each tool can be tested independently
- **Extensible**: New tools can be added without touching core MCP logic

## Success Criteria
- [ ] `McpTool` trait and `ToolRegistry` implemented
- [ ] `McpServer` updated to use registry pattern
- [ ] All existing tests pass
- [ ] Registry can register and lookup tools
- [ ] Common utility functions for tools implemented
- [ ] Performance is equivalent or better than current match statement

## Next Steps
After implementing the tool registry:
1. Migrate existing issue tools to new pattern
2. Migrate existing memo tools to new pattern  
3. Add missing search tools to MCP
4. Implement markdown description build macros
5. Update CLI to use same tool implementations

## Migration Strategy
- Keep the old match statement initially as a fallback
- Gradually move tools to the registry
- Remove old implementation once all tools are migrated
- Use feature flags if needed for staged rollout