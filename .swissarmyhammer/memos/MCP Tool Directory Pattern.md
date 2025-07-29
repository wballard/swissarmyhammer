# MCP Tool Directory Pattern

## Overview

The Swiss Army Hammer MCP tools follow a consistent directory organization pattern that improves maintainability and discoverability.

## Directory Structure

```
src/mcp/tools/
├── <noun>/
│   ├── <verb>/
│   │   ├── mod.rs         # Tool implementation
│   │   └── description.md # Tool description
│   └── mod.rs            # Module exports
```

## Pattern Details

### Noun-Based Organization
Tools are grouped by the resource they operate on:
- `issues/` - Issue management tools
- `memoranda/` - Memo management tools  
- `search/` - Search functionality tools

### Verb-Based Submodules
Each action on a resource gets its own submodule:
- `issues/create/` - Create new issues
- `issues/work/` - Start working on an issue
- `issues/merge/` - Merge issue branches
- `memoranda/get/` - Retrieve memos
- `memoranda/update/` - Update existing memos

### Separated Descriptions
Each tool has a `description.md` file that contains the help text shown to users. This separation:
- Keeps implementation code clean
- Makes descriptions easy to update
- Allows markdown formatting in descriptions
- Centralizes user-facing documentation

## Benefits

1. **Discoverability**: Easy to find all operations for a resource
2. **Consistency**: Predictable location for new tools
3. **Modularity**: Each tool is self-contained
4. **Documentation**: Descriptions are first-class citizens
5. **Maintenance**: Clear separation of concerns

## Example

```
src/mcp/tools/
├── issues/
│   ├── create/
│   │   ├── mod.rs
│   │   └── description.md
│   ├── work/
│   │   ├── mod.rs
│   │   └── description.md
│   └── mod.rs
```

This pattern scales well as new resources and operations are added to the MCP server.