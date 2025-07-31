# CLI MCP Integration: Refactor Memo Commands

## Overview

Refactor the CLI memo commands in `swissarmyhammer-cli/src/memo.rs` to use MCP tools directly instead of duplicating business logic, following the same pattern established for issue commands.

## Problem Statement

The `memo.rs` module contains significant code duplication that mirrors functionality already implemented in MCP memoranda tools. The CLI implements its own business logic instead of calling the existing, tested MCP tools.

## Goals

1. Replace all memo command implementations with direct MCP tool calls
2. Maintain identical CLI behavior and output formatting
3. Reduce code duplication and maintenance burden
4. Leverage existing MCP memoranda infrastructure

## MCP Tools Mapping

| CLI Function | MCP Tool | Tool Arguments |
|-------------|----------|----------------|
| `create_memo()` | `memo_create` | `title`, `content` |
| `list_memos()` | `memo_list` | None |
| `get_memo()` | `memo_get` | `id` |
| `update_memo()` | `memo_update` | `id`, `content` |
| `delete_memo()` | `memo_delete` | `id` |
| `search_memos()` | `memo_search` | `query` |
| `get_context()` | `memo_get_all_context` | None |

## Tasks

### 1. Refactor Core Command Functions

Transform each function from direct storage access to MCP tool calls:

**Before (create_memo example):**
```rust
async fn create_memo(
    storage: MarkdownMemoStorage,
    title: String,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_input(content)?;
    let memo = storage.create_memo(title, content).await?;
    
    println!("{} Created memo: {}", "✅".green(), memo.title.bold());
    println!("🆔 ID: {}", memo.id.as_str().blue());
    // ... more formatting
    Ok(())
}
```

**After:**
```rust
async fn create_memo(
    context: &CliToolContext,
    title: String,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_input(content)?;
    
    let args = context.create_arguments(vec![
        ("title", json!(title)),
        ("content", json!(content)),
    ]);
    
    let result = context.execute_tool("memo_create", args).await?;
    
    // Extract memo details from MCP response and format for CLI
    let response_data = response_formatting::extract_success_data(&result)?;
    if let Some(memo_data) = response_data.as_object() {
        println!("{} Created memo: {}", 
            "✅".green(), 
            memo_data.get("title").unwrap().as_str().unwrap().bold()
        );
        println!("🆔 ID: {}", 
            memo_data.get("id").unwrap().as_str().unwrap().blue()
        );
        // ... more formatting logic
    }
    
    Ok(())
}
```

### 2. Update Main Handler Function

Replace direct storage instantiation with tool context:

```rust
pub async fn handle_memo_command(command: MemoCommands) -> Result<(), Box<dyn std::error::Error>> {
    let context = CliToolContext::new().await?;

    match command {
        MemoCommands::Create { title, content } => {
            create_memo(&context, title, content).await?;
        }
        MemoCommands::List => {
            list_memos(&context).await?;
        }
        MemoCommands::Get { id } => {
            get_memo(&context, &id).await?;
        }
        MemoCommands::Update { id, content } => {
            update_memo(&context, &id, content).await?;
        }
        MemoCommands::Delete { id } => {
            delete_memo(&context, &id).await?;
        }
        MemoCommands::Search { query } => {
            search_memos(&context, &query).await?;
        }
        MemoCommands::Context => {
            get_context(&context).await?;
        }
    }

    Ok(())
}
```

### 3. Handle Response Formatting

Create specialized response formatting for different memo operations:

```rust
mod memo_response_formatting {
    use rmcp::model::CallToolResult;
    use colored::*;
    use serde_json::Value;
    
    pub fn format_memo_list(result: &CallToolResult) -> Result<(), Box<dyn std::error::Error>> {
        let memos = response_formatting::extract_success_data(result)?;
        
        if let Some(memo_array) = memos.as_array() {
            if memo_array.is_empty() {
                println!("{} No memos found", "ℹ️".blue());
                return Ok(());
            }

            println!(
                "{} Found {} memo{}",
                "📝".green(),
                memo_array.len().to_string().bold(),
                if memo_array.len() == 1 { "" } else { "s" }
            );
            
            // Sort and display memos with CLI formatting
            for memo in memo_array {
                format_memo_summary(memo);
            }
        }
        
        Ok(())
    }
    
    pub fn format_memo_summary(memo: &Value) {
        if let Some(memo_obj) = memo.as_object() {
            println!("{} {}", "🆔".dimmed(), 
                memo_obj.get("id").unwrap().as_str().unwrap().blue());
            println!("{} {}", "📄".dimmed(), 
                memo_obj.get("title").unwrap().as_str().unwrap().bold());
            // ... additional formatting
        }
    }
    
    pub fn format_search_results(result: &CallToolResult) -> Result<(), Box<dyn std::error::Error>> {
        // Handle search-specific formatting
        Ok(())
    }
}
```

### 4. Preserve CLI-Specific Utilities

Keep the following CLI-specific functions that don't have MCP equivalents:
- `get_content_input()` - Handles stdin input
- `format_content_preview()` - CLI-specific text truncation
- Constants like `DEFAULT_LIST_PREVIEW_LENGTH`

### 5. Update Integration with MCP Response Format

Ensure the CLI properly handles the response format from MCP tools:

```rust
// Handle MCP tool responses and extract meaningful data
fn extract_memo_from_response(result: &CallToolResult) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match result {
        CallToolResult::Success { content } => {
            // Parse the content and extract memo data
            // Handle different content types (text, JSON, etc.)
        }
        CallToolResult::Error { error, .. } => {
            Err(format!("MCP tool error: {}", error).into())
        }
    }
}
```

## Implementation Approach

### Phase 1: Simple CRUD Operations
Start with straightforward one-to-one mappings:
- `get_memo()` → `memo_get`
- `delete_memo()` → `memo_delete`
- `get_context()` → `memo_get_all_context`

### Phase 2: Operations with Formatting
Handle operations that require significant CLI formatting:
- `create_memo()` → `memo_create` + CLI formatting
- `update_memo()` → `memo_update` + CLI formatting
- `list_memos()` → `memo_list` + CLI table formatting

### Phase 3: Complex Operations
Handle operations with advanced logic:
- `search_memos()` → `memo_search` + result formatting

## Acceptance Criteria

- [ ] All memo commands use MCP tools instead of direct storage access
- [ ] CLI behavior and output formatting remain identical to current implementation
- [ ] All existing CLI functionality preserved (stdin support, preview formatting, etc.)
- [ ] Code reduction: `memo.rs` should be significantly smaller
- [ ] No direct use of `MarkdownMemoStorage` in CLI commands
- [ ] Integration tests verify MCP tool execution
- [ ] Error handling maintains existing user experience
- [ ] All edge cases handled (empty results, malformed data, etc.)

## Testing Strategy

1. **Behavioral Testing**: Compare before/after CLI output for all commands
2. **Error Case Testing**: Ensure error messages remain user-friendly
3. **Integration Testing**: Verify MCP tool calls execute correctly
4. **Edge Case Testing**: Handle empty lists, invalid IDs, etc.

## Expected Changes

- Modified: `swissarmyhammer-cli/src/memo.rs` (significant reduction in size)
- Modified: `swissarmyhammer-cli/tests/memo_*.rs` (test updates)
- New: Memo-specific response formatting utilities
- New: Integration tests for memo MCP tool calls

## Dependencies

- Requires: CLI_000220_project-setup (CliToolContext implementation)
- Requires: CLI_000221_refactor-issue-commands (establishes patterns)
- Requires: All memoranda MCP tools to be stable and tested

## Risk Mitigation

1. **Response Format Changes**: Ensure MCP tool responses contain all data needed for CLI formatting
2. **Performance**: Monitor any impact from additional abstraction layer
3. **Error Messages**: Ensure MCP errors are properly translated to CLI-friendly messages

## Follow-up Issues

Success here, combined with CLI_000221, establishes the pattern for refactoring remaining CLI modules like `search.rs`.