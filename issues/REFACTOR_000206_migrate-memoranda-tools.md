# REFACTOR Step 4: Migrate Memoranda Tools to New Organization

## Overview
Move all memoranda-related MCP tools from the current tool handlers to the new tool registry pattern under `./mcp/tools/memoranda/`.

## Context
Currently, memoranda tools are handled by the `ToolHandlers` struct in `tool_handlers.rs`:
- `memo_create`
- `memo_get`
- `memo_update`
- `memo_delete`
- `memo_list`
- `memo_search`
- `memo_get_all_context`

These tools are already partially separated from the main match statement but need to be converted to the new `McpTool` trait pattern and organized with markdown descriptions.

## Target Structure
```
swissarmyhammer/src/mcp/tools/memoranda/
├── mod.rs                    # Module registration and exports
├── create/
│   ├── mod.rs               # CreateMemoTool implementation
│   └── description.md       # Tool description for MCP
├── get/
│   ├── mod.rs               # GetMemoTool implementation
│   └── description.md
├── update/
│   ├── mod.rs               # UpdateMemoTool implementation
│   └── description.md
├── delete/
│   ├── mod.rs               # DeleteMemoTool implementation
│   └── description.md
├── list/
│   ├── mod.rs               # ListMemosTool implementation
│   └── description.md
├── search/
│   ├── mod.rs               # SearchMemosTool implementation
│   └── description.md
└── get_all_context/
    ├── mod.rs               # GetAllContextTool implementation
    └── description.md
```

## Tasks for This Step

### 1. Create Memoranda Tools Module Structure

Set up the directory structure and base module files:

```rust
// swissarmyhammer/src/mcp/tools/memoranda/mod.rs
pub mod create;
pub mod get;
pub mod update;
pub mod delete;
pub mod list;
pub mod search;
pub mod get_all_context;

use crate::mcp::tools::ToolRegistry;

pub fn register_memoranda_tools(registry: &mut ToolRegistry) {
    registry.register(create::CreateMemoTool::new());
    registry.register(get::GetMemoTool::new());
    registry.register(update::UpdateMemoTool::new());
    registry.register(delete::DeleteMemoTool::new());
    registry.register(list::ListMemosTool::new());
    registry.register(search::SearchMemosTool::new());
    registry.register(get_all_context::GetAllContextTool::new());
}
```

### 2. Implement Individual Memoranda Tools

Extract logic from `ToolHandlers` methods and convert to individual tool implementations:

```rust
// Example: swissarmyhammer/src/mcp/tools/memoranda/create/mod.rs
use crate::mcp::tools::{McpTool, ToolContext, BaseToolImpl};
use crate::mcp::memo_types::CreateMemoRequest;
use crate::memoranda::MemoId;
use async_trait::async_trait;
use rmcp::model::*;
use rmcp::Error as McpError;
use std::collections::HashMap;

pub struct CreateMemoTool;

impl CreateMemoTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTool for CreateMemoTool {
    fn name(&self) -> &'static str {
        "memo_create"
    }
    
    fn description(&self) -> &'static str {
        include_str!("description.md")
    }
    
    fn schema(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(CreateMemoRequest))
            .expect("Failed to generate schema")
    }
    
    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateMemoRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        let mut memo_storage = context.memo_storage.write().await;
        let memo = memo_storage.create_memo(request.title, request.content).await
            .map_err(|e| McpError::internal_error(format!("Failed to create memo: {e}"), None))?;
        
        let mut response_content = HashMap::new();
        response_content.insert("id".to_string(), serde_json::Value::String(memo.id.to_string()));
        response_content.insert("title".to_string(), serde_json::Value::String(memo.title));
        response_content.insert("created_at".to_string(), serde_json::Value::String(memo.created_at.to_rfc3339()));
        response_content.insert("message".to_string(), serde_json::Value::String("Memo created successfully".to_string()));
        
        Ok(BaseToolImpl::create_success_response(response_content))
    }
}
```

### 3. Create Comprehensive Markdown Descriptions

For each memoranda tool, create detailed markdown descriptions:

```markdown
<!-- swissarmyhammer/src/mcp/tools/memoranda/create/description.md -->
# Create Memo

Create a new memo with the given title and content. Returns the created memo with its unique ID.

## Parameters

- `title` (required): Title of the memo
- `content` (required): Markdown content of the memo

## Examples

Create a memo with title and content:
```json
{
  "title": "Meeting Notes",
  "content": "# Team Meeting\n\nDiscussed project roadmap..."
}
```

## Returns

Returns the created memo information:
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "title": "Meeting Notes", 
  "created_at": "2025-07-27T10:30:00Z",
  "message": "Memo created successfully"
}
```

## Storage

Memos are stored in `./.swissarmyhammer/memos` in the local repository root.
```

### 4. Handle Preview Formatting

Migrate the preview formatting logic from the current `ToolHandlers`:

```rust
// Add to memoranda tools common utilities
const MEMO_LIST_PREVIEW_LENGTH: usize = 100;
const MEMO_SEARCH_PREVIEW_LENGTH: usize = 200;

fn format_memo_preview(content: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        content.to_string()
    } else {
        format!("{}...", &content[..max_length])
    }
}

fn create_memo_summary(memo: &crate::memoranda::Memo, preview_length: usize) -> serde_json::Value {
    serde_json::json!({
        "id": memo.id.to_string(),
        "title": memo.title,
        "preview": format_memo_preview(&memo.content, preview_length),
        "created_at": memo.created_at.to_rfc3339(),
        "updated_at": memo.updated_at.to_rfc3339()
    })
}
```

### 5. Implement All Seven Memoranda Tools

Each tool needs individual implementation:

#### create - CreateMemoTool
- Validates title and content
- Creates memo via storage backend
- Returns memo ID and metadata

#### get - GetMemoTool  
- Validates memo ID format
- Retrieves memo from storage
- Returns full memo content

#### update - UpdateMemoTool
- Validates memo ID and new content
- Updates memo via storage backend
- Returns updated memo metadata

#### delete - DeleteMemoTool
- Validates memo ID format
- Deletes memo from storage
- Returns confirmation message

#### list - ListMemosTool
- Retrieves all memos from storage
- Formats with preview content
- Returns paginated list

#### search - SearchMemosTool
- Performs full-text search across memos
- Formats results with longer previews
- Returns ranked search results

#### get_all_context - GetAllContextTool
- Retrieves all memos formatted for AI context
- Sorts by most recent first
- Returns comprehensive context data

### 6. Update Tool Context

Ensure `ToolContext` provides necessary access to memo storage:

```rust
pub struct ToolContext {
    pub issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    pub git_ops: Arc<Mutex<Option<GitOperations>>>,
    pub memo_storage: Arc<RwLock<Box<dyn MemoStorage>>>,
}
```

### 7. Comprehensive Testing

For each memoranda tool:
- Unit tests for tool implementation
- Integration tests with different storage backends
- Error handling tests (invalid IDs, storage failures)
- Preview formatting tests
- Search functionality tests

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::test_utils::*;
    use crate::memoranda::mock_storage::MockMemoStorage;
    
    #[tokio::test]
    async fn test_create_memo_tool() {
        let tool = CreateMemoTool::new();
        let context = create_test_tool_context_with_memo_storage(
            Box::new(MockMemoStorage::new())
        ).await;
        
        let arguments = serde_json::json!({
            "title": "Test Memo",
            "content": "This is test content"
        });
        
        let result = tool.execute(
            arguments.as_object().unwrap().clone(),
            &context
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        // Verify response structure and content
    }
    
    #[tokio::test]
    async fn test_search_memo_tool() {
        // Test search functionality with mock data
    }
}
```

## Success Criteria
- [ ] All 7 memoranda tools migrated to new structure
- [ ] Each tool has its own module with description.md
- [ ] All tools registered with the tool registry
- [ ] Preview formatting logic preserved
- [ ] Search functionality fully working
- [ ] All existing tests pass
- [ ] New unit tests for each tool
- [ ] No behavioral changes - exact same functionality

## Integration Points
- Tools use existing `MemoStorage` trait through `ToolContext`
- ULID generation and validation preserved
- Preview length constants maintained
- Error handling patterns consistent with existing code
- Response formatting matches current tool handlers

## Migration from ToolHandlers

The current `ToolHandlers` struct in `tool_handlers.rs` implements these methods:
- `handle_memo_create`
- `handle_memo_get`
- `handle_memo_update`
- `handle_memo_delete`
- `handle_memo_list`
- `handle_memo_search`
- `handle_memo_get_all_context`

These will be directly migrated to individual tool implementations, preserving all logic and error handling.

## Next Steps
After completing memoranda tool migration:
1. Add missing search tools (index/query) to MCP
2. Update CLI to use same tool implementations
3. Remove old ToolHandlers implementation
4. Clean up duplicate code
5. Implement build macros for tool descriptions

## Risk Mitigation
- Test all memo operations thoroughly before removing old implementation
- Verify ULID handling and storage compatibility
- Ensure search functionality works with existing memo data
- Test with different storage backends (filesystem, mock)
- Validate response formatting matches exactly