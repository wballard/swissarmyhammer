# REFACTOR Step 3: Migrate Issue Tools to New Organization

## Overview
Move all issue-related MCP tools from the large match statement to the new tool registry pattern under `./mcp/tools/issues/`.

## Context
Currently, issue tools are handled in the main `call_tool` match statement:
- `issue_create`
- `issue_mark_complete` 
- `issue_all_complete`
- `issue_update`
- `issue_current`
- `issue_work`
- `issue_merge`
- `issue_next`

Each tool needs to be:
1. Moved to its own module under `./mcp/tools/issues/`
2. Converted to implement the `McpTool` trait
3. Given a markdown description file
4. Registered with the tool registry

## Target Structure
```
swissarmyhammer/src/mcp/tools/issues/
├── mod.rs                    # Module registration and exports
├── create/
│   ├── mod.rs               # CreateIssueTool implementation
│   └── description.md       # Tool description for MCP
├── mark_complete/
│   ├── mod.rs               # MarkCompleteIssueTool implementation  
│   └── description.md
├── all_complete/
│   ├── mod.rs               # AllCompleteIssueTool implementation
│   └── description.md
├── update/
│   ├── mod.rs               # UpdateIssueTool implementation
│   └── description.md
├── current/
│   ├── mod.rs               # CurrentIssueTool implementation
│   └── description.md
├── work/
│   ├── mod.rs               # WorkIssueTool implementation
│   └── description.md
├── merge/
│   ├── mod.rs               # MergeIssueTool implementation
│   └── description.md
└── next/
    ├── mod.rs               # NextIssueTool implementation
    └── description.md
```

## Tasks for This Step

### 1. Create Issue Tools Module Structure
Set up the directory structure and base module files:

```rust
// swissarmyhammer/src/mcp/tools/issues/mod.rs
pub mod create;
pub mod mark_complete;
pub mod all_complete;
pub mod update;
pub mod current;
pub mod work;
pub mod merge;
pub mod next;

use crate::mcp::tools::ToolRegistry;

pub fn register_issue_tools(registry: &mut ToolRegistry) {
    registry.register(create::CreateIssueTool::new());
    registry.register(mark_complete::MarkCompleteIssueTool::new());
    registry.register(all_complete::AllCompleteIssueTool::new());
    registry.register(update::UpdateIssueTool::new());
    registry.register(current::CurrentIssueTool::new());
    registry.register(work::WorkIssueTool::new());
    registry.register(merge::MergeIssueTool::new());
    registry.register(next::NextIssueTool::new());
}
```

### 2. Implement Individual Issue Tools

For each tool, create the implementation following the pattern:

```rust
// Example: swissarmyhammer/src/mcp/tools/issues/create/mod.rs
use crate::mcp::tools::{McpTool, ToolContext, BaseToolImpl};
use crate::mcp::types::{CreateIssueRequest, IssueName};
use crate::mcp::responses::create_issue_response;
use async_trait::async_trait;
use rmcp::model::*;
use rmcp::Error as McpError;

pub struct CreateIssueTool;

impl CreateIssueTool {
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
        serde_json::to_value(schemars::schema_for!(CreateIssueRequest))
            .expect("Failed to generate schema")
    }
    
    async fn execute(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        context: &ToolContext,
    ) -> std::result::Result<CallToolResult, McpError> {
        let request: CreateIssueRequest = BaseToolImpl::parse_arguments(arguments)?;
        
        let mut issue_storage = context.issue_storage.write().await;
        let next_number = issue_storage.get_next_issue_number().await
            .map_err(|e| McpError::internal_error(format!("Failed to get next issue number: {e}"), None))?;
        
        let issue_name = match &request.name {
            Some(name) => IssueName(format!("{:06}_{}", next_number, name.0)),
            None => IssueName(format!("{:06}", next_number)),
        };
        
        let issue = Issue {
            name: issue_name.clone(),
            content: request.content,
        };
        
        issue_storage.create_issue(issue).await
            .map_err(|e| McpError::internal_error(format!("Failed to create issue: {e}"), None))?;
        
        Ok(create_issue_response(&issue_name))
    }
}
```

### 3. Create Markdown Descriptions

For each tool, create a comprehensive markdown description:

```markdown
<!-- swissarmyhammer/src/mcp/tools/issues/create/description.md -->
# Create Issue

Create a new issue with auto-assigned number. Issues are markdown files stored in ./issues directory for tracking work items.

## Parameters

- `content` (required): Markdown content of the issue
- `name` (optional): Name of the issue (will be used in filename)
  - When provided, creates files like `000123_name.md`
  - When omitted, creates files like `000123.md`

## Examples

Create a named issue:
```json
{
  "name": "feature_name",
  "content": "# Implement new feature\n\nDetails..."
}
```

Create a nameless issue:
```json
{
  "content": "# Quick fix needed\n\nDetails..."
}
```

## Returns

Returns the created issue name and confirmation message.
```

### 4. Migrate Logic from Existing Handlers

Extract the logic from the current MCP server methods and move it to the individual tool implementations. Ensure:
- All error handling is preserved
- All validation logic is maintained
- Response formatting matches exactly
- Any shared utilities are moved to common modules

### 5. Update Build System

Modify `build.rs` to include the markdown description files similar to how builtin prompts are handled:

```rust
// Add to build.rs
fn collect_tool_descriptions() -> Result<(), Box<dyn std::error::Error>> {
    let tool_dirs = [
        "src/mcp/tools/issues",
        "src/mcp/tools/memoranda", 
        "src/mcp/tools/search",
    ];
    
    for dir in &tool_dirs {
        let dir_path = Path::new(dir);
        if dir_path.exists() {
            collect_descriptions_from_dir(dir_path)?;
        }
    }
    
    Ok(())
}
```

### 6. Comprehensive Testing

For each migrated tool:
- Unit tests for the tool implementation
- Integration tests with the tool registry
- Backward compatibility tests ensuring identical behavior
- Error handling tests for edge cases

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::test_utils::*;
    
    #[tokio::test]
    async fn test_create_issue_tool() {
        let tool = CreateIssueTool::new();
        let context = create_test_tool_context().await;
        
        let arguments = serde_json::json!({
            "content": "Test issue content",
            "name": "test_issue"
        });
        
        let result = tool.execute(
            arguments.as_object().unwrap().clone(),
            &context
        ).await;
        
        assert!(result.is_ok());
        // Additional assertions...
    }
}
```

## Success Criteria
- [ ] All 8 issue tools migrated to new structure
- [ ] Each tool has its own module with description.md
- [ ] All tools registered with the tool registry
- [ ] Build system includes markdown descriptions
- [ ] All existing tests pass
- [ ] New unit tests for each tool
- [ ] No behavioral changes - exact same functionality

## Integration Points
- Tools use existing `IssueStorage` and `GitOperations` through `ToolContext`
- Response formatting uses existing helper functions from `mcp/responses.rs`
- Type definitions remain in `mcp/types.rs` for now
- Error handling uses existing patterns from `mcp/error_handling.rs`

## Next Steps
After completing issue tool migration:
1. Migrate memoranda tools to new pattern
2. Add missing search tools
3. Update CLI to use same tool implementations
4. Remove old implementation from main match statement
5. Clean up duplicate code across the codebase

## Risk Mitigation
- Maintain parallel implementation until fully tested
- Use comprehensive integration tests
- Test with real MCP clients to ensure protocol compatibility
- Keep detailed logs of any behavioral changes