# MCP Memoranda Tools

SwissArmyHammer's memoranda system provides MCP tools for memo management through the Model Context Protocol. These tools enable AI assistants to create, read, update, delete, and search text memos with structured metadata and timestamps.

## Overview

The memoranda MCP tools provide a complete note-taking system with:

- **ULID-based Identifiers**: Unique, sortable memo identifiers
- **Structured Storage**: Filesystem-based storage with atomic operations
- **Full-text Search**: Search across memo titles and content
- **Metadata Tracking**: Automatic creation and update timestamps
- **AI Context Support**: Formatted output for AI consumption

```
┌─────────────┐    MCP Tools    ┌─────────────────────┐
│   Claude    │◄───────────────►│ SwissArmyHammer     │
│  Assistant  │   JSON-RPC 2.0  │ Memoranda System    │
└─────────────┘                 └─────────────────────┘
                                         │
                                         ▼
                                ┌─────────────────────┐
                                │  .swissarmyhammer   │
                                │      /memos/        │
                                │   [ULID].json      │
                                └─────────────────────┘
```

## Available Tools

| Tool Name | Description | Purpose |
|-----------|-------------|---------|
| `memo_create` | Create a new memo | Add new structured notes |
| `memo_get` | Get a specific memo | Retrieve memo by ID |
| `memo_update` | Update memo content | Modify existing memo |
| `memo_delete` | Delete a memo | Remove memo from storage |
| `memo_list` | List all memos | Browse available memos |
| `memo_search` | Search memos | Find memos by content |
| `memo_get_all_context` | Get all memo context | Retrieve all memos for AI |

## Tool Details

### memo_create

Creates a new memo with title and content.

**Request Schema:**
```json
{
  "title": "string (required) - Brief title or subject",
  "content": "string (required) - Main memo content/body"
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_create",
    "arguments": {
      "title": "Meeting Notes",
      "content": "# Team Meeting 2024-01-15\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22"
    }
  },
  "id": 1
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Successfully created memo 'Meeting Notes' with ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV\n\nTitle: Meeting Notes\nContent: # Team Meeting 2024-01-15\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22"
      }
    ]
  },
  "id": 1
}
```

**Use Cases:**
- Creating meeting notes
- Saving code snippets
- Recording project decisions
- Storing research findings

### memo_get

Retrieves a specific memo by its ULID identifier.

**Request Schema:**
```json
{
  "id": "string (required) - ULID identifier of the memo"
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_get",
    "arguments": {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
    }
  },
  "id": 2
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Memo found:\n\nID: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nTitle: Meeting Notes\nCreated: 2024-01-15 14:30:00 UTC\nUpdated: 2024-01-15 14:30:00 UTC\n\nContent:\n# Team Meeting 2024-01-15\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22"
      }
    ]
  },
  "id": 2
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid memo ID format: invalid-id",
    "data": null
  },
  "id": 2
}
```

**Use Cases:**
- Retrieving specific memo by ID
- Verifying memo exists
- Getting full memo details

### memo_update

Updates the content of an existing memo by ID.

**Request Schema:**
```json
{
  "id": "string (required) - ULID identifier of memo to update",
  "content": "string (required) - New content to replace existing content"
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_update",
    "arguments": {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "content": "# Team Meeting 2024-01-15 (Updated)\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22\n- Action items added to project board"
    }
  },
  "id": 3
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Successfully updated memo:\n\nID: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nTitle: Meeting Notes\nUpdated: 2024-01-15 16:45:00 UTC\n\nContent:\n# Team Meeting 2024-01-15 (Updated)\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22\n- Action items added to project board"
      }
    ]
  },
  "id": 3
}
```

**Use Cases:**
- Updating meeting notes with action items
- Correcting typos or errors
- Adding new information to existing memos
- Appending follow-up notes

### memo_delete

Permanently deletes a memo by its ID.

**Request Schema:**
```json
{
  "id": "string (required) - ULID identifier of memo to delete"
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_delete",
    "arguments": {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
    }
  },
  "id": 4
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Successfully deleted memo with ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV"
      }
    ]
  },
  "id": 4
}
```

**Use Cases:**
- Removing outdated memos
- Cleaning up duplicate entries
- Deleting sensitive information
- Maintaining storage hygiene

### memo_list

Lists all available memos with previews.

**Request Schema:**
```json
{
  // No parameters required
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_list",
    "arguments": {}
  },
  "id": 5
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Found 2 memos:\n\n• Meeting Notes (01ARZ3NDEKTSV4RRFFQ69G5FAV)\n  Created: 2024-01-15 14:30\n  Updated: 2024-01-15 16:45\n  Preview: # Team Meeting 2024-01-15 (Updated)\n\n- Discussed Q1 roadmap\n- Assigned tasks for...\n\n• Project Ideas (01BRZ3NDEKTSV4RRFFQ69G5FAW)\n  Created: 2024-01-14 09:15\n  Updated: 2024-01-14 09:15\n  Preview: ## New Features\n\n1. Dark mode toggle\n2. Export functionality\n3. Advanced search with..."
      }
    ]
  },
  "id": 5
}
```

**Empty Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "No memos found"
      }
    ]
  },
  "id": 5
}
```

**Use Cases:**
- Browsing available memos
- Getting overview of memo collection
- Finding recently created/updated memos
- Discovering forgotten notes

### memo_search

Searches memos by query string across titles and content.

**Request Schema:**
```json
{
  "query": "string (required) - Search query to match against titles and content"
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_search",
    "arguments": {
      "query": "meeting roadmap"
    }
  },
  "id": 6
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Found 1 memo matching 'meeting roadmap':\n\n• Meeting Notes (01ARZ3NDEKTSV4RRFFQ69G5FAV)\n  Created: 2024-01-15 14:30\n  Updated: 2024-01-15 16:45\n  Preview: # Team Meeting 2024-01-15 (Updated)\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22\n- Action items added to project board..."
      }
    ]
  },
  "id": 6
}
```

**No Results Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "No memos found matching query: 'nonexistent'"
      }
    ]
  },
  "id": 6
}
```

**Use Cases:**
- Finding memos by keywords
- Locating specific information
- Discovering related memos
- Content-based memo retrieval

### memo_get_all_context

Retrieves all memo content formatted for AI consumption.

**Request Schema:**
```json
{
  // No parameters required
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memo_get_all_context",
    "arguments": {}
  },
  "id": 7
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "All memo context (2 memos):\n\n=== Meeting Notes (ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV) ===\nCreated: 2024-01-15 14:30\nUpdated: 2024-01-15 16:45\n\n# Team Meeting 2024-01-15 (Updated)\n\n- Discussed Q1 roadmap\n- Assigned tasks for sprint\n- Next meeting: 2024-01-22\n- Action items added to project board\n\n================================================================================\n\n=== Project Ideas (ID: 01BRZ3NDEKTSV4RRFFQ69G5FAW) ===\nCreated: 2024-01-14 09:15\nUpdated: 2024-01-14 09:15\n\n## New Features\n\n1. Dark mode toggle\n2. Export functionality\n3. Advanced search with filters\n4. Collaborative editing"
      }
    ]
  },
  "id": 7
}
```

**Use Cases:**
- Providing all memo context to AI
- Analyzing memo collection
- Context-aware responses
- Knowledge base queries

## Integration Guide for AI Assistants

### Basic Workflow

1. **Create Memo**:
   ```json
   memo_create → "Meeting Notes" + content → Returns ULID
   ```

2. **Retrieve Memo**:
   ```json
   memo_get → ULID → Returns full memo details
   ```

3. **Search Memos**:
   ```json
   memo_search → "keywords" → Returns matching memos
   ```

4. **Update Memo**:
   ```json
   memo_update → ULID + new content → Returns updated memo
   ```

### Context Management

For AI assistants requiring context:

```javascript
// Get all memos for context
const allContext = await callTool('memo_get_all_context', {});

// Search for relevant memos
const relevantMemos = await callTool('memo_search', { 
  query: userQuery 
});

// Create new memo from conversation
await callTool('memo_create', {
  title: `Discussion: ${topic}`,
  content: conversationSummary
});
```

### Best Practices

1. **Use Descriptive Titles**: Help with search and organization
   ```json
   { "title": "API Design Meeting 2024-01-15" }
   ```

2. **Structure Content**: Use markdown for better readability
   ```markdown
   # Meeting Title
   
   ## Attendees
   - Person A
   - Person B
   
   ## Agenda
   1. Topic 1
   2. Topic 2
   
   ## Action Items
   - [ ] Task 1
   - [ ] Task 2
   ```

3. **Search Effectively**: Use specific keywords
   ```json
   { "query": "API authentication security" }
   ```

4. **Context Awareness**: Use `memo_get_all_context` sparingly
   - Good for initial context loading
   - Expensive for large memo collections
   - Consider `memo_search` for specific queries

## Performance Considerations

### Response Times

| Tool | Typical Response Time | Factors |
|------|----------------------|---------|
| `memo_create` | 10-50ms | File I/O, validation |
| `memo_get` | 5-20ms | File lookup, ULID parsing |
| `memo_update` | 15-60ms | File I/O, timestamp update |
| `memo_delete` | 10-30ms | File deletion |
| `memo_list` | 20-200ms | Collection size, preview generation |
| `memo_search` | 50-500ms | Content size, query complexity |
| `memo_get_all_context` | 100ms-2s | Collection size, formatting |

### Limitations

- **File System Storage**: Performance scales with disk I/O
- **No Pagination**: `memo_list` returns all memos
- **Basic Search**: Simple string matching (no advanced queries)
- **Memory Usage**: Large collections held in memory for search

### Optimization Tips

1. **Batch Operations**: Create multiple memos in sequence rather than individually
2. **Search First**: Use `memo_search` before `memo_list` for specific needs
3. **Context Caching**: Cache `memo_get_all_context` results when possible
4. **Selective Updates**: Update only when content actually changes

## Error Handling

### Common Errors

| Error Code | Error Type | Description | Resolution |
|------------|------------|-------------|-----------|
| -32602 | Invalid Params | Invalid ULID format | Use valid 26-character ULID |
| -32602 | Invalid Params | Memo not found | Verify memo exists with `memo_list` |
| -32603 | Internal Error | Storage failure | Check file permissions, disk space |
| -32603 | Internal Error | Serialization error | Verify content is valid UTF-8 |

### Example Error Responses

**Invalid ULID:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid memo ID format: invalid-id",
    "data": null
  },
  "id": 1
}
```

**Memo Not Found:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Memo not found: 01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "data": null
  },
  "id": 2
}
```

**Storage Error:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Failed to create memo: Permission denied",
    "data": null
  },
  "id": 3
}
```

### Error Recovery Strategies

1. **Retry with Backoff**: For transient storage errors
2. **Validate Input**: Check ULID format before operations
3. **Graceful Degradation**: Fall back to basic operations on errors
4. **User Feedback**: Provide clear error messages to users

## Security Considerations

### Input Validation

- **ULID Validation**: All memo IDs validated against ULID format
- **Content Sanitization**: Content stored as-is (markdown safe)
- **Size Limits**: No built-in limits (consider application-level limits)

### Access Control

- **File System Permissions**: Memos stored in `.swissarmyhammer/memos/`
- **Process Isolation**: MCP server runs with user permissions
- **No Network Access**: Local storage only

### Data Protection

- **Plain Text Storage**: Memos stored unencrypted
- **Atomic Operations**: File operations are atomic
- **Backup Strategy**: Consider regular backups of memo directory

## Testing MCP Memoranda Tools

### Manual Testing with Claude Code

```bash
# Start SwissArmyHammer MCP server
swissarmyhammer serve --transport stdio

# Test via Claude Code integration
# Tools will be available in Claude Code interface
```

### Automated Testing

```rust
#[test]
async fn test_memo_create_tool() {
    let handlers = setup_test_handlers().await;
    
    let request = CreateMemoRequest {
        title: "Test Memo".to_string(),
        content: "Test content".to_string(),
    };
    
    let result = handlers.handle_memo_create(request).await;
    assert!(result.is_ok());
}
```

### Integration Testing

```javascript
// Test tool sequence
async function testMemoWorkflow() {
    // Create memo
    const created = await callTool('memo_create', {
        title: 'Test Memo',
        content: 'Test content'
    });
    
    const memoId = extractMemoId(created.content[0].text);
    
    // Verify creation
    const retrieved = await callTool('memo_get', { id: memoId });
    assert(retrieved.content[0].text.includes('Test Memo'));
    
    // Update memo
    await callTool('memo_update', {
        id: memoId,
        content: 'Updated content'
    });
    
    // Verify update
    const updated = await callTool('memo_get', { id: memoId });
    assert(updated.content[0].text.includes('Updated content'));
    
    // Delete memo
    await callTool('memo_delete', { id: memoId });
    
    // Verify deletion
    try {
        await callTool('memo_get', { id: memoId });
        assert(false, 'Memo should be deleted');
    } catch (error) {
        assert(error.code === -32602);
    }
}
```

## Next Steps

- Review [CLI Reference](./cli-memoranda.md) for command-line usage
- See [Getting Started Guide](../examples/memoranda-quickstart.md) for tutorials
- Check [Advanced Usage Examples](../examples/memoranda-advanced.md) for patterns
- Visit [Troubleshooting](./troubleshooting.md) for common issues