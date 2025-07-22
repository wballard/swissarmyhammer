# Memoranda CLI Reference

SwissArmyHammer's memoranda system provides command-line tools for managing structured text memos with automatic timestamping, unique identifiers, and full-text search capabilities.

## Overview

The memoranda CLI enables you to:

- **Create** structured memos with titles and content
- **List** and browse all stored memos with previews
- **Search** memos by keywords across titles and content
- **Retrieve** specific memos by their unique identifiers
- **Update** memo content while preserving metadata
- **Delete** memos when no longer needed
- **Export** all memos as formatted context for AI assistants

All memos are stored in `./.swissarmyhammer/memos/` with ULID identifiers for chronological ordering.

## Basic Usage

```bash
swissarmyhammer memo [SUBCOMMAND] [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| [`create`](#create) | Create a new memo with title and content |
| [`list`](#list) | List all memos with previews |
| [`get`](#get) | Retrieve a specific memo by ID |
| [`update`](#update) | Update memo content by ID |
| [`delete`](#delete) | Delete a memo by ID |
| [`search`](#search) | Search memos by query string |
| [`context`](#context) | Get all memo context for AI consumption |

---

## create

Creates a new memo with a title and content.

### Usage

```bash
swissarmyhammer memo create <TITLE> [OPTIONS]
```

### Arguments

- `<TITLE>` - Brief title or subject for the memo (required)

### Options

- `-c, --content <CONTENT>` - Memo content (optional)
  - If omitted, prompts for interactive input
  - Use `-c -` to read from stdin
  - Use `-c "content text"` for direct input

### Examples

#### Interactive Content Input
```bash
# Create memo with interactive content input
swissarmyhammer memo create "Meeting Notes"
# Prompts: Enter memo content, press Ctrl+D when finished
```

#### Direct Content Input
```bash
# Create memo with inline content
swissarmyhammer memo create "Daily Standup" -c "- Completed user auth\n- Working on API endpoints\n- Next: Database schema"
```

#### Stdin Content Input
```bash
# Create memo from file or piped content
cat meeting_notes.md | swissarmyhammer memo create "Team Meeting" -c -

# Or from heredoc
swissarmyhammer memo create "Project Notes" -c - << EOF
# Project Planning Session

## Attendees
- Alice, Bob, Charlie

## Decisions
- Use React for frontend
- PostgreSQL for database
- Deploy on AWS
EOF
```

#### Markdown Content
```bash
swissarmyhammer memo create "Code Review Checklist" -c "# Code Review Checklist

## Security
- [ ] Input validation
- [ ] Authentication checks
- [ ] No hardcoded secrets

## Performance
- [ ] Database queries optimized
- [ ] Caching implemented
- [ ] Memory usage acceptable

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Edge cases covered"
```

### Output

```
✅ Created memo: Meeting Notes
🆔 ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV
📅 Created: 2024-01-15 14:30:25 UTC
```

---

## list

Lists all available memos with metadata and content previews.

### Usage

```bash
swissarmyhammer memo list
```

### Options

None.

### Examples

```bash
# List all memos
swissarmyhammer memo list
```

### Output

```
📝 Found 3 memos

🆔 01ARZ3NDEKTSV4RRFFQ69G5FAV
📄 Meeting Notes
📅 2024-01-15 14:30:25 UTC
💬 # Team Meeting 2024-01-15\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint...

🆔 01BRZ3NDEKTSV4RRFFQ69G5FAW
📄 Project Ideas
📅 2024-01-14 09:15:42 UTC
💬 ## New Features\n\n1. Dark mode toggle\n2. Export functionality\n3. Advanced search...

🆔 01CRZ3NDEKTSV4RRFFQ69G5FAX
📄 Code Snippets
📅 2024-01-13 16:22:18 UTC
💬 ```rust\nfn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1...
```

**Empty Collection:**
```
ℹ️ No memos found
```

### Content Preview

- Shows first 100 characters of memo content
- Newlines replaced with spaces for compact display
- Truncated content indicated with "..."
- Memos sorted by creation time (newest first)

---

## get

Retrieves and displays a specific memo by its ULID identifier.

### Usage

```bash
swissarmyhammer memo get <ID>
```

### Arguments

- `<ID>` - ULID identifier of the memo to retrieve (required)

### Examples

```bash
# Get memo by ID
swissarmyhammer memo get 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Output

```
📝 Memo: Meeting Notes
🆔 ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV
📅 Created: 2024-01-15 14:30:25 UTC
🔄 Updated: 2024-01-15 14:30:25 UTC

Content:
# Team Meeting 2024-01-15

## Attendees
- Alice (PM)
- Bob (Engineering)
- Charlie (Design)

## Agenda Items
1. Q1 Roadmap Review
2. Sprint Planning
3. Technical Debt Discussion

## Action Items
- [ ] Alice: Schedule follow-up with stakeholders
- [ ] Bob: Research database migration options
- [ ] Charlie: Create wireframes for new features

## Next Meeting
- Date: 2024-01-22
- Time: 2:00 PM UTC
- Location: Conference Room B
```

### Error Handling

**Invalid ULID format:**
```bash
swissarmyhammer memo get invalid-id
# Error: Invalid memo ID format: 'invalid-id'. Expected a valid ULID...
```

**Memo not found:**
```bash
swissarmyhammer memo get 01XYZ3NDEKTSV4RRFFQ69G5FAV
# Error: Memo not found with ID: 01XYZ3NDEKTSV4RRFFQ69G5FAV
```

---

## update

Updates the content of an existing memo while preserving its title and metadata.

### Usage

```bash
swissarmyhammer memo update <ID> [OPTIONS]
```

### Arguments

- `<ID>` - ULID identifier of the memo to update (required)

### Options

- `-c, --content <CONTENT>` - New content for the memo (optional)
  - If omitted, prompts for interactive input
  - Use `-c -` to read from stdin
  - Use `-c "content text"` for direct input

### Examples

#### Interactive Update
```bash
# Update memo with interactive content input
swissarmyhammer memo update 01ARZ3NDEKTSV4RRFFQ69G5FAV
# Prompts: Enter memo content, press Ctrl+D when finished
```

#### Direct Update
```bash
# Update memo with inline content
swissarmyhammer memo update 01ARZ3NDEKTSV4RRFFQ69G5FAV -c "# Updated Meeting Notes

Added action items and next steps from today's discussion."
```

#### Stdin Update
```bash
# Update memo from file
cat updated_notes.md | swissarmyhammer memo update 01ARZ3NDEKTSV4RRFFQ69G5FAV -c -
```

### Output

```
✅ Updated memo: Meeting Notes
🆔 ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV
🔄 Updated: 2024-01-15 16:45:30 UTC

Content:
# Updated Meeting Notes

Added action items and next steps from today's discussion.
```

### Behavior

- **Title preserved** - Only content is updated, title remains unchanged
- **Updated timestamp** - `updated_at` field refreshed to current time
- **Created timestamp** - `created_at` field remains unchanged
- **ID preserved** - ULID identifier never changes

---

## delete

Permanently deletes a memo from storage.

### Usage

```bash
swissarmyhammer memo delete <ID>
```

### Arguments

- `<ID>` - ULID identifier of the memo to delete (required)

### Examples

```bash
# Delete memo by ID
swissarmyhammer memo delete 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Output

```
🗑️ Deleted memo: Meeting Notes
🆔 ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Important Notes

⚠️ **Warning**: Deletion is permanent and cannot be undone. The memo file is immediately removed from the filesystem.

**Best Practices:**
- Verify the memo ID before deletion using `get` command
- Consider archiving important memos instead of deleting
- Use search to ensure you're deleting the correct memo

---

## search

Searches memos by query string across titles and content with advanced highlighting and relevance scoring.

### Usage

```bash
swissarmyhammer memo search <QUERY>
```

### Arguments

- `<QUERY>` - Search query to match against memo titles and content (required)

### Examples

#### Basic Search
```bash
# Search for memos containing "meeting"
swissarmyhammer memo search "meeting"
```

#### Multi-word Search
```bash
# Search for multiple keywords
swissarmyhammer memo search "project roadmap timeline"
```

#### Empty Query
```bash
# Empty query returns all memos
swissarmyhammer memo search ""
```

### Output

#### Successful Search
```
🔍 Found 2 memos matching 'meeting'

🆔 01ARZ3NDEKTSV4RRFFQ69G5FAV
📄 Meeting Notes
📅 2024-01-15 14:30:25 UTC
⭐ 95.5% relevance
💬 # Team **Meeting** 2024-01-15\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next **meeting**: 2024-01-22...

🆔 01DRZ3NDEKTSV4RRFFQ69G5FAY
📄 Sprint Planning
📅 2024-01-10 11:20:15 UTC
⭐ 78.2% relevance
💬 Planning **meeting** for next sprint. Need to review backlog and assign story points...
```

#### No Results
```
ℹ️ No memos found matching 'nonexistent'
```

### Search Features

- **Case-insensitive** - Search matches regardless of case
- **Partial matching** - Finds substrings within words
- **Title and content** - Searches across both memo titles and content
- **Relevance scoring** - Results sorted by relevance (0-100%)
- **Highlighted matches** - Query terms highlighted in results
- **Advanced search engine** - Uses advanced search when available, falls back to basic search
- **Result previews** - Shows 150 characters of content around matches

### Search Tips

- Use specific keywords for better results
- Combine related terms to narrow results
- Search for unique identifiers or project names
- Use common words to find broader categories

---

## context

Exports all memo content formatted for AI assistant consumption.

### Usage

```bash
swissarmyhammer memo context
```

### Options

None.

### Examples

```bash
# Get all memo context
swissarmyhammer memo context
```

### Output

```
📄 All memo context (2 memos)

## Meeting Notes (ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV)

Created: 2024-01-15 14:30:25 UTC
Updated: 2024-01-15 16:45:30 UTC

# Team Meeting 2024-01-15

## Attendees
- Alice (PM)
- Bob (Engineering)
- Charlie (Design)

## Action Items
- [ ] Schedule follow-up meeting
- [ ] Research database options
- [ ] Create wireframes

===

## Project Ideas (ID: 01BRZ3NDEKTSV4RRFFQ69G5FAW)

Created: 2024-01-14 09:15:42 UTC
Updated: 2024-01-14 09:15:42 UTC

## New Features

1. Dark mode toggle
2. Export functionality  
3. Advanced search with filters
4. Collaborative editing

===
```

### Output Format

- **Sorted by updated time** - Most recently updated memos first
- **Full content** - Complete memo content without truncation
- **Metadata included** - Creation and update timestamps
- **Clear separators** - `===` between memos for easy parsing
- **AI-friendly format** - Optimized for AI assistant consumption

### Use Cases

- **AI Context Loading** - Provide memo knowledge base to AI assistants
- **Backup Creation** - Export all memos for archival
- **Content Analysis** - Analyze memo collection patterns
- **Documentation Generation** - Convert memos to documentation format

---

## Common Workflows

### Daily Note-Taking

```bash
# Morning: Create daily journal
swissarmyhammer memo create "Daily Journal $(date +%Y-%m-%d)" -c "# Goals for today:
- Review PR #123
- Complete API documentation
- Team standup at 10am"

# Throughout the day: Search for related memos
swissarmyhammer memo search "API documentation"

# Evening: Update with accomplishments
swissarmyhammer memo update $MEMO_ID -c "# Daily Journal $(date +%Y-%m-%d)

## Completed:
- ✅ Reviewed PR #123 - approved with minor comments
- ✅ Completed API documentation for auth endpoints
- ✅ Team standup - discussed upcoming sprint

## Tomorrow:
- Implement user profile endpoints
- Review database migration scripts"
```

### Meeting Notes Management

```bash
# Before meeting: Create template
swissarmyhammer memo create "Weekly Team Meeting $(date +%Y-%m-%d)" -c "# Weekly Team Meeting

## Attendees
- 

## Agenda
1. Sprint progress review
2. Blockers discussion
3. Next week planning

## Action Items
- [ ] 

## Next Meeting
- Date: 
- Topics: "

# During meeting: Get memo ID for quick updates
MEMO_ID=$(swissarmyhammer memo list | grep "Weekly Team Meeting" | grep -o '01[A-Z0-9]\{25\}' | head -1)

# After meeting: Update with notes
swissarmyhammer memo update $MEMO_ID -c "$(cat meeting_notes_final.md)"
```

### Project Documentation

```bash
# Create project overview
swissarmyhammer memo create "Project Alpha Overview" -c "# Project Alpha

## Objectives
- Improve user onboarding flow
- Reduce support tickets by 30%
- Increase user retention

## Tech Stack
- Frontend: React + TypeScript
- Backend: Node.js + Express
- Database: PostgreSQL
- Deployment: Docker + AWS

## Team
- PM: Alice
- Engineering: Bob, Carol
- Design: David
- QA: Eve"

# Document decisions
swissarmyhammer memo create "Architecture Decisions" -c "# Architecture Decision Records

## ADR-001: Database Choice
**Date**: $(date +%Y-%m-%d)
**Status**: Accepted
**Decision**: Use PostgreSQL for primary database
**Rationale**: Better JSON support, mature ecosystem"

# Search related documentation
swissarmyhammer memo search "project alpha database"
```

### Code Snippet Collection

```bash
# Save useful code snippets
swissarmyhammer memo create "Rust Error Handling Patterns" -c '# Rust Error Handling

## Custom Error Types
```rust
#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Parse(String),
    NotFound,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MyError::Io(err) => write!(f, "IO error: {}", err),
            MyError::Parse(msg) => write!(f, "Parse error: {}", msg),
            MyError::NotFound => write!(f, "Not found"),
        }
    }
}

impl std::error::Error for MyError {}
```

## Result Helpers
```rust
type Result<T> = std::result::Result<T, MyError>;

fn might_fail() -> Result<String> {
    Ok("success".to_string())
}
```'

# Search for code examples
swissarmyhammer memo search "rust error"
swissarmyhammer memo search "```rust"
```

## Storage and File Management

### Storage Location

Memos are stored in:
```
./.swissarmyhammer/memos/
├── 01ARZ3NDEKTSV4RRFFQ69G5FAV.json
├── 01BRZ3NDEKTSV4RRFFQ69G5FAW.json
└── 01CRZ3NDEKTSV4RRFFQ69G5FAX.json
```

### File Format

Each memo is stored as a JSON file:

```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "title": "Meeting Notes",
  "content": "# Team Meeting 2024-01-15\n\n- Discussed roadmap...",
  "created_at": "2024-01-15T14:30:25.123456789Z",
  "updated_at": "2024-01-15T16:45:30.987654321Z"
}
```

### Backup and Restore

#### Backup Memos
```bash
# Copy memo directory
cp -r .swissarmyhammer/memos/ backup/memos-$(date +%Y%m%d)/

# Or create archive
tar -czf memos-backup-$(date +%Y%m%d).tar.gz .swissarmyhammer/memos/

# Export as context for external storage
swissarmyhammer memo context > all-memos-$(date +%Y%m%d).md
```

#### Restore Memos
```bash
# Restore from backup directory
cp -r backup/memos-20240115/ .swissarmyhammer/memos/

# Or extract from archive
tar -xzf memos-backup-20240115.tar.gz
```

### Migration Between Projects

```bash
# Export from project A
cd /path/to/project-a
swissarmyhammer memo context > project-a-memos.md

# In project B, manually recreate important memos
cd /path/to/project-b
swissarmyhammer memo create "Imported from Project A" -c - < project-a-memos.md
```

## Performance Considerations

### Response Times

| Command | Typical Time | Notes |
|---------|--------------|--------|
| `create` | 10-50ms | File I/O dependent |
| `list` | 50-200ms | Scales with memo count |
| `get` | 5-20ms | Single file read |
| `update` | 15-60ms | File write + timestamp |
| `delete` | 10-30ms | File deletion |
| `search` | 100-500ms | Content size dependent |
| `context` | 200ms-2s | All memos loaded |

### Optimization Tips

1. **Large Collections** - Consider archiving old memos
2. **Search Performance** - Use specific keywords rather than broad terms
3. **Context Export** - Run sparingly for large memo collections
4. **File System** - Use SSD storage for better I/O performance

### Limitations

- **No pagination** - `list` and `context` load all memos
- **Memory usage** - Large memos consume proportional memory
- **Search complexity** - Basic string matching only
- **Concurrent access** - No locking mechanism for simultaneous operations

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Error: Permission denied (os error 13)
# Solution: Check directory permissions
ls -la .swissarmyhammer/
chmod 755 .swissarmyhammer/
chmod 644 .swissarmyhammer/memos/*
```

#### Invalid ULID Format
```bash
# Error: Invalid memo ID format
# Solution: Use complete 26-character ULID
swissarmyhammer memo get 01ARZ3NDEKTSV4RRFFQ69G5FAV  # ✅ Correct
swissarmyhammer memo get 01ARZ3                        # ❌ Too short
```

#### Storage Directory Missing
```bash
# Error: No such file or directory
# Solution: Directory created automatically, but check parent permissions
mkdir -p .swissarmyhammer/memos
```

#### Large Content Issues
```bash
# For very large content, use file input instead of command line
# Command line arguments have length limits
cat large_document.md | swissarmyhammer memo create "Large Doc" -c -
```

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug swissarmyhammer memo list
```

## Integration with Other Tools

### Shell Scripts

```bash
#!/bin/bash
# daily-standup.sh - Create daily standup note

DATE=$(date +%Y-%m-%d)
TITLE="Daily Standup $DATE"

swissarmyhammer memo create "$TITLE" -c "# Daily Standup $DATE

## Yesterday
- 

## Today  
- 

## Blockers
- 

## Notes
- "

echo "Created daily standup note for $DATE"
```

### Git Hooks

```bash
#!/bin/bash
# post-commit hook - Create memo for significant commits

COMMIT_MSG=$(git log -1 --pretty=%B)
COMMIT_HASH=$(git rev-parse --short HEAD)

if [[ "$COMMIT_MSG" == *"BREAKING CHANGE"* || "$COMMIT_MSG" == *"feat:"* ]]; then
    swissarmyhammer memo create "Git Commit $COMMIT_HASH" -c "# Significant Commit

**Hash**: $COMMIT_HASH
**Message**: $COMMIT_MSG

**Changes**:
$(git show --stat $COMMIT_HASH)

**Files Modified**:
$(git show --name-only $COMMIT_HASH)"
fi
```

### Editor Integration

#### Vim/Neovim

```vim
" Add to .vimrc or init.vim
command! MemoCreate :!swissarmyhammer memo create <q-args>
command! MemoList :!swissarmyhammer memo list
command! MemoSearch :!swissarmyhammer memo search <q-args>

" Quick memo creation from current buffer
nnoremap <leader>mc :MemoCreate 
nnoremap <leader>ml :MemoList<CR>
nnoremap <leader>ms :MemoSearch 
```

#### VS Code

Create tasks in `.vscode/tasks.json`:

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Create Memo",
            "type": "shell",
            "command": "swissarmyhammer",
            "args": ["memo", "create", "${input:memoTitle}"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always"
            }
        }
    ],
    "inputs": [
        {
            "id": "memoTitle",
            "description": "Memo title",
            "default": "Quick Note",
            "type": "promptString"
        }
    ]
}
```

## See Also

- [MCP Memoranda Tools](./mcp-memoranda.md) - MCP integration for AI assistants
- [Getting Started Guide](../examples/memoranda-quickstart.md) - Step-by-step tutorial
- [Advanced Usage Examples](../examples/memoranda-advanced.md) - Complex workflows
- [API Reference](./api-reference.md) - Programmatic usage
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions