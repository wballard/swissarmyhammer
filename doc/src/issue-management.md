# Issue Management Guide

SwissArmyHammer's issue management system provides a lightweight, git-friendly way to track work items directly in your repository.

## Philosophy

- **Simplicity**: Issues are just markdown files
- **Version Control**: Full git history for all issue changes
- **Branch-based Workflow**: Each issue gets its own work branch
- **AI-friendly**: Designed for LLM interaction via MCP

## MCP Tools Reference

### issue_create

Create a new issue with auto-assigned number.

**Parameters:**
- `name` (string, required): Issue name (used in filename)
- `content` (string, required): Markdown content of the issue

**Example:**
```json
{
  "tool": "issue_create",
  "arguments": {
    "name": "implement_auth",
    "content": "# Implement Authentication\n\nAdd JWT-based authentication to the API."
  }
}
```

### issue_mark_complete

Mark an issue as complete by moving it to the complete directory.

**Parameters:**
- `number` (integer, required): Issue number to complete

**Example:**
```json
{
  "tool": "issue_mark_complete",
  "arguments": {
    "number": 42
  }
}
```

### issue_all_complete

Check if all issues are completed.

**Parameters:** None

**Returns:** Boolean indicating if all issues are complete, plus statistics.

### issue_update

Update an existing issue's content.

**Parameters:**
- `number` (integer, required): Issue number to update
- `content` (string, required): New content
- `append` (boolean, optional): If true, append to existing content

### issue_current

Get the current issue based on git branch.

**Parameters:**
- `branch` (string, optional): Specific branch to check (defaults to current)

### issue_work

Switch to a work branch for an issue.

**Parameters:**
- `number` (integer, required): Issue number to work on

### issue_merge

Merge completed issue work back to main branch.

**Parameters:**
- `number` (integer, required): Issue number to merge
- `delete_branch` (boolean, optional): Delete branch after merge

## Best Practices

1. **One Issue, One Branch**: Each issue should have its own feature branch
2. **Complete Before Merge**: Mark issues complete before merging
3. **Descriptive Names**: Use clear, descriptive issue names
4. **Regular Updates**: Update issues with progress notes
5. **Clean Working Directory**: Commit changes before switching issues

## Workflow Example

```bash
# 1. Create an issue
$ swissarmyhammer mcp issue_create name="add_logging" content="Add structured logging"

# 2. Start working on it
$ swissarmyhammer mcp issue_work number=1
# Switches to branch: issue/000001_add_logging

# 3. Make changes and commit
$ git add .
$ git commit -m "Add structured logging implementation"

# 4. Update the issue with notes
$ swissarmyhammer mcp issue_update number=1 append=true content="Implemented using slog"

# 5. Mark complete when done
$ swissarmyhammer mcp issue_mark_complete number=1

# 6. Merge back to main
$ swissarmyhammer mcp issue_merge number=1
```

## Integration with AI Assistants

The issue system is designed to work seamlessly with AI assistants like Claude:

```
Human: Create an issue to track the new authentication feature