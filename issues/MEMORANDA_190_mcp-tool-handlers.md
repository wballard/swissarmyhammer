# Implement Memoranda MCP Tool Handlers

## Overview
Add memoranda tool handlers to the existing MCP server, implementing all 7 core API tools from the original memoranda specification.

## Tasks

### 1. Add MemoStorage to ToolHandlers
- Modify `swissarmyhammer/src/mcp/tool_handlers.rs` to include `MemoStorage`
- Add memo_storage field to `ToolHandlers` struct
- Update constructor to accept memo storage instance

### 2. Implement MCP Tool Methods
Add these handler methods to ToolHandlers:
- `handle_memo_create(CreateMemoRequest) -> CallToolResult`
- `handle_memo_update(UpdateMemoRequest) -> CallToolResult`
- `handle_memo_get(GetMemoRequest) -> CallToolResult`
- `handle_memo_delete(DeleteMemoRequest) -> CallToolResult`
- `handle_memo_list() -> CallToolResult`
- `handle_memo_search(SearchMemosRequest) -> CallToolResult`
- `handle_memo_get_all_context() -> CallToolResult`

### 3. MCP Type Definitions
- Create `swissarmyhammer/src/mcp/memo_types.rs` for MCP request/response types
- Add to existing MCP types structure
- Ensure proper serde derives for JSON compatibility

### 4. Error Response Handling
- Create helper functions for memo error responses
- Follow existing patterns from issue handlers
- Proper error message formatting for AI consumption

### 5. Integration with Main MCP Server
- Update `swissarmyhammer/src/mcp.rs` to register memoranda tools
- Add tool definitions to MCP server initialization
- Update tool routing to handle memo_* tool names

## Implementation Notes
- Follow exact patterns from existing issue handlers
- Use async/await throughout
- Comprehensive error handling with user-friendly messages
- Response format should be optimized for AI consumption

## Acceptance Criteria
- [ ] All 7 memoranda MCP tools implemented
- [ ] Proper error handling and responses
- [ ] MCP server updated to include memoranda tools
- [ ] Response format compatible with MCP protocol
- [ ] Integration tests showing MCP tools work end-to-end