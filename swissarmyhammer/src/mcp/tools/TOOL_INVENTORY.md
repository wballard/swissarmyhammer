# MCP Tool Inventory and Mapping

This document provides a comprehensive inventory of all MCP tools in the current system and their planned mapping to the new noun/verb directory structure.

## Current Tool Distribution

### Issue Tools (in mcp.rs)
Currently implemented in large match statement in `mcp.rs` around lines 2040-2110.

| Current Tool Name | New Location | Current Handler | Description |
|-------------------|---------------|-----------------|-------------|
| `issue_create` | `issues/create/` | `handle_issue_create()` | Create new issue with auto-assigned number |
| `issue_mark_complete` | `issues/mark_complete/` | `handle_issue_mark_complete()` | Mark issue as completed |
| `issue_all_complete` | `issues/all_complete/` | `handle_issue_all_complete()` | Check if all issues are completed |
| `issue_update` | `issues/update/` | `handle_issue_update()` | Update existing issue content |
| `issue_current` | `issues/current/` | `handle_issue_current()` | Get current issue being worked on |
| `issue_work` | `issues/work/` | `handle_issue_work()` | Switch to work on specific issue |
| `issue_merge` | `issues/merge/` | `handle_issue_merge()` | Merge issue work branch |
| `issue_next` | `issues/next/` | `handle_issue_next()` | Get next issue to work on |

**Implementation Notes:**
- All use shared types from `mcp/types.rs`: `CreateIssueRequest`, `MarkCompleteRequest`, etc.
- Integrate with git workflow for branch management
- Store issues as markdown files in `./issues/` directory
- Auto-assign sequential issue numbers

### Memoranda Tools (in mcp/tool_handlers.rs)
Currently implemented in `ToolHandlers` struct in `mcp/tool_handlers.rs` around lines 600-900.

| Current Tool Name | New Location | Current Handler | Description |
|-------------------|---------------|-----------------|-------------|
| `memo_create` | `memoranda/create/` | `handle_memo_create()` | Create new memo with title and content |
| `memo_get` | `memoranda/get/` | `handle_memo_get()` | Retrieve memo by ULID |
| `memo_update` | `memoranda/update/` | `handle_memo_update()` | Update existing memo content |
| `memo_delete` | `memoranda/delete/` | `handle_memo_delete()` | Delete memo by ULID |
| `memo_list` | `memoranda/list/` | `handle_memo_list()` | List all memos with metadata |
| `memo_search` | `memoranda/search/` | `handle_memo_search()` | Search memos by content |
| `memo_get_all_context` | `memoranda/get_all_context/` | `handle_memo_get_all_context()` | Get all memos for AI context |

**Implementation Notes:**
- Use shared types from `mcp/memo_types.rs`: `CreateMemoRequest`, `GetMemoRequest`, etc.
- Use ULID for unique identifiers
- Support markdown content format
- Storage abstraction through `MemoStorage` trait

### Search Tools (CLI-only, needs MCP integration)
Currently implemented in CLI only via `swissarmyhammer-cli/src/search.rs` and semantic search modules.

| CLI Command | New MCP Tool Name | New Location | Current Implementation | Description |
|-------------|-------------------|---------------|------------------------|-------------|
| `search index` | `search_index` | `search/index/` | `run_semantic_index()` | Build and maintain search index |
| `search query` | `search_query` | `search/query/` | `run_semantic_query()` | Query indexed files semantically |

**Implementation Notes:**
- Currently CLI-only through `SearchCommands` enum
- Uses semantic search with vector embeddings
- Integrates with `semantic/` module: `FileIndexer`, `SemanticSearcher`, `VectorStorage`
- Needs MCP protocol integration (request/response types, error handling)

## File Structure Analysis

### Current Structure
```
swissarmyhammer/src/
├── mcp.rs                     # 4268+ lines, issue tools + main server logic
├── mcp/
│   ├── tool_handlers.rs       # Memoranda tools in ToolHandlers struct
│   ├── types.rs              # Issue request/response types
│   ├── memo_types.rs         # Memoranda request/response types
│   ├── utils.rs              # Shared utilities
│   └── ...
├── semantic/                  # Search functionality (7 modules)
│   ├── embedding.rs
│   ├── indexer.rs
│   ├── searcher.rs
│   └── ...
└── swissarmyhammer-cli/src/
    └── search.rs             # CLI search commands
```

### Target Structure (Created but Empty)
```
swissarmyhammer/src/mcp/
├── mod.rs                     # Main MCP module (move from mcp.rs)
└── tools/
    ├── mod.rs                 # Tool registry
    ├── memoranda/
    │   ├── mod.rs
    │   ├── create/
    │   │   ├── mod.rs
    │   │   └── description.md
    │   ├── get/
    │   ├── update/
    │   ├── delete/
    │   ├── list/
    │   ├── search/
    │   └── get_all_context/
    ├── issues/
    │   ├── mod.rs
    │   ├── create/
    │   │   ├── mod.rs
    │   │   └── description.md
    │   ├── mark_complete/
    │   ├── all_complete/
    │   ├── update/
    │   ├── current/
    │   ├── work/
    │   ├── merge/
    │   └── next/
    └── search/
        ├── mod.rs
        ├── index/
        │   └── mod.rs
        └── query/
            └── mod.rs
```

## Migration Priority

1. **High Priority**: Memoranda tools (well-isolated in tool_handlers.rs)
2. **High Priority**: Issue tools (large but self-contained handlers)
3. **Medium Priority**: Search tools (requires MCP integration design)

## Breaking Changes Risk Assessment

- **Low Risk**: Directory structure creation (done, no imports changed)
- **Medium Risk**: Tool handler extraction (can maintain compatibility)
- **High Risk**: Large mcp.rs refactoring (needs careful incremental approach)

## Test Coverage Assessment

### Existing Tests
- Issue tools: Comprehensive unit tests in mcp.rs (~50+ test functions)
- Memoranda tools: Integration tests in swissarmyhammer/tests/mcp_memoranda_tests.rs
- Search tools: CLI integration tests in swissarmyhammer-cli/tests/search_cli_test.rs

### Test Requirements for Migration
- Maintain all existing test functionality
- Add tests for new tool registry pattern
- Ensure MCP protocol compatibility
- Performance regression tests for large mcp.rs refactoring

## Next Steps

1. **REFACTOR_000204**: Create tool registry pattern
2. **REFACTOR_000205**: Migrate issue tools
3. **REFACTOR_000206**: Migrate memoranda tools  
4. **REFACTOR_000207**: Add search tools to MCP
5. **REFACTOR_000208**: Implement build macros for descriptions