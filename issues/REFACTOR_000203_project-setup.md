# REFACTOR Step 1: Project Setup and Planning

## Overview
Set up the foundation for refactoring MCP tools organization according to the `refactor_tools.md` specification.

## Context
The specification requires:
1. Moving all MCP code into `./mcp/` module folder (currently split between `mcp.rs` and `mcp/`)
2. Organizing tools by noun/verb pattern: `./mcp/tools/memoranda`, `./mcp/tools/issues`, `./mcp/tools/search`
3. Creating individual tool modules with markdown descriptions
4. Building a tool registry to replace the large match statement
5. Making CLI commands use the same tools as MCP server

## Current State Analysis
```
swissarmyhammer/src/
├── mcp.rs                     # Large match statement, needs to move to mcp/mod.rs
├── mcp/                       # Existing submodules
│   ├── tool_handlers.rs       # Needs to be reorganized
│   ├── types.rs              # Issue and memo types
│   ├── memo_types.rs         # Memo-specific types
│   ├── utils.rs              # Utilities
│   └── ...
├── semantic/                  # Search functionality (should be renamed to search)
└── issues/                    # Issue storage logic
```

## Tasks for This Step

### 1. Create New Directory Structure
Create the target directory structure for the refactored tools:

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
        └── query/
```

### 2. Document Current Tool Inventory
Create comprehensive documentation of existing tools:

**Issue Tools:**
- `issue_create` → `issues/create`
- `issue_mark_complete` → `issues/mark_complete` 
- `issue_all_complete` → `issues/all_complete`
- `issue_update` → `issues/update`
- `issue_current` → `issues/current`
- `issue_work` → `issues/work`
- `issue_merge` → `issues/merge`
- `issue_next` → `issues/next`

**Memoranda Tools:**
- `memo_create` → `memoranda/create`
- `memo_get` → `memoranda/get`
- `memo_update` → `memoranda/update`
- `memo_delete` → `memoranda/delete`
- `memo_list` → `memoranda/list`
- `memo_search` → `memoranda/search`
- `memo_get_all_context` → `memoranda/get_all_context`

**Missing Search Tools (from CLI):**
- `search_index` → `search/index` (needs to be added to MCP)
- `search_query` → `search/query` (needs to be added to MCP)

### 3. Research Build Macro Implementation
Research how to implement the build macro pattern similar to builtin prompts for markdown descriptions. This will allow tool descriptions to be maintained as standalone markdown files that get compiled into the binary.

### 4. Set Up Testing Framework
Ensure comprehensive tests exist for all current MCP tools before refactoring begins. This includes:
- Unit tests for each tool handler method
- Integration tests for MCP protocol communication
- CLI integration tests

## Success Criteria
- [ ] New directory structure created but empty
- [ ] All existing MCP tools documented with mapping to new organization
- [ ] Research completed on build macro implementation
- [ ] Testing framework verified and any missing tests identified
- [ ] No breaking changes - all existing functionality still works

## Next Steps
After completing this setup step, the plan will proceed with:
1. Creating tool registry pattern
2. Moving individual tools to new structure  
3. Implementing build macros for markdown descriptions
4. Updating CLI to use the same tools
5. Cleaning up duplicate code

## Risk Mitigation
- Keep all changes backwards compatible until migration is complete
- Maintain extensive test coverage throughout refactoring
- Use feature flags if needed to switch between old and new implementations