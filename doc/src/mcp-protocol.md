# MCP Protocol

SwissArmyHammer implements the Model Context Protocol (MCP) to provide prompts to AI assistants like Claude. This guide covers the protocol details and implementation specifics.

## Overview

The Model Context Protocol (MCP) is a standardized protocol for communication between AI assistants and context providers. SwissArmyHammer acts as an MCP server, providing prompt templates as resources and tools.

```
┌─────────────┐         MCP          ┌──────────────────┐
│   Claude    │◄────────────────────►│ SwissArmyHammer  │
│  (Client)   │    JSON-RPC 2.0      │    (Server)      │
└─────────────┘                      └──────────────────┘
```

## Protocol Basics

### Transport

MCP uses JSON-RPC 2.0 over various transports:

- **stdio** - Standard input/output (default for Claude Code)
- **HTTP** - REST API endpoints
- **WebSocket** - Persistent connections

### Message Format

All messages follow JSON-RPC 2.0 format:

```json
{
  "jsonrpc": "2.0",
  "method": "prompts/list",
  "params": {},
  "id": 1
}
```

Response format:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "prompts": [...]
  },
  "id": 1
}
```

## MCP Methods

### Initialize

Establishes connection and capabilities:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "prompts": {},
      "resources": {}
    },
    "clientInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "prompts": {
        "listChanged": true
      },
      "resources": {
        "subscribe": true,
        "listChanged": true
      }
    },
    "serverInfo": {
      "name": "swissarmyhammer",
      "version": "0.1.0"
    }
  },
  "id": 1
}
```

### List Prompts

Get available prompts:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "prompts/list",
  "params": {
    "cursor": null
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "prompts": [
      {
        "id": "code-review",
        "name": "Code Review",
        "description": "Reviews code for best practices and issues",
        "arguments": [
          {
            "name": "code",
            "description": "The code to review",
            "required": true
          },
          {
            "name": "language",
            "description": "Programming language",
            "required": false
          }
        ]
      }
    ],
    "nextCursor": null
  },
  "id": 2
}
```

### Get Prompt

Retrieve a specific prompt with arguments:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "promptId": "code-review",
    "arguments": {
      "code": "def add(a, b):\n    return a + b",
      "language": "python"
    }
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "Please review this python code:\n\n```python\ndef add(a, b):\n    return a + b\n```\n\nFocus on:\n- Code quality\n- Best practices\n- Potential issues"
        }
      }
    ]
  },
  "id": 3
}
```

### List Resources

Get available resources (prompt source files):

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "resources/list",
  "params": {
    "cursor": null
  },
  "id": 4
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "resources": [
      {
        "uri": "prompt://code-review",
        "name": "code-review.md",
        "description": "Code review prompt source",
        "mimeType": "text/markdown"
      }
    ],
    "nextCursor": null
  },
  "id": 4
}
```

### Read Resource

Get resource content:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "prompt://code-review"
  },
  "id": 5
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "contents": [
      {
        "uri": "prompt://code-review",
        "mimeType": "text/markdown",
        "text": "---\nname: code-review\ntitle: Code Review\n---\n\n# Code Review\n..."
      }
    ]
  },
  "id": 5
}
```

## Notifications

### Prompt List Changed

Sent when prompts are added/removed/modified:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/prompts/list_changed",
  "params": {}
}
```

### Resource List Changed

Sent when resources change:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/resources/list_changed",
  "params": {}
}
```

## Error Handling

MCP defines standard error codes:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "Missing required argument 'code'"
    }
  },
  "id": 6
}
```

Standard error codes:
- `-32700` - Parse error
- `-32600` - Invalid request
- `-32601` - Method not found
- `-32602` - Invalid params
- `-32603` - Internal error

Custom error codes:
- `1001` - Prompt not found
- `1002` - Invalid prompt format
- `1003` - Template render error

## SwissArmyHammer Extensions

### Pagination

For large prompt collections:

```json
// Request with pagination
{
  "jsonrpc": "2.0",
  "method": "prompts/list",
  "params": {
    "cursor": "eyJvZmZzZXQiOjUwfQ==",
    "limit": 50
  },
  "id": 7
}
```

### Filtering

Filter prompts by criteria:

```json
// Request with filters
{
  "jsonrpc": "2.0",
  "method": "prompts/list",
  "params": {
    "filter": {
      "category": "development",
      "tags": ["python", "testing"]
    }
  },
  "id": 8
}
```

### Metadata

Extended prompt metadata:

```json
{
  "id": "code-review",
  "name": "Code Review",
  "description": "Reviews code for best practices",
  "metadata": {
    "author": "SwissArmyHammer Team",
    "version": "1.0.0",
    "category": "development",
    "tags": ["code", "review", "quality"],
    "lastModified": "2024-01-15T10:30:00Z"
  }
}
```

## Implementation Details

### Server Lifecycle

1. **Initialization**
   ```rust
   // Server startup sequence
   let server = MCPServer::new();
   server.load_prompts()?;
   server.start_file_watcher()?;
   server.listen(transport)?;
   ```

2. **Request Handling**
   ```rust
   match request.method.as_str() {
       "initialize" => handle_initialize(params),
       "prompts/list" => handle_list_prompts(params),
       "prompts/get" => handle_get_prompt(params),
       "resources/list" => handle_list_resources(params),
       "resources/read" => handle_read_resource(params),
       _ => Err(MethodNotFound),
   }
   ```

3. **Change Detection**
   ```rust
   // File watcher triggers notifications
   watcher.on_change(|event| {
       server.reload_prompts();
       server.notify_clients("prompts/list_changed");
   });
   ```

### Transport Implementations

#### stdio Transport

Default for Claude Code integration:

```rust
// Read from stdin, write to stdout
let stdin = io::stdin();
let stdout = io::stdout();

loop {
    let request = read_json_rpc(&mut stdin)?;
    let response = server.handle_request(request)?;
    write_json_rpc(&mut stdout, response)?;
}
```

#### HTTP Transport

For web integrations:

```rust
// HTTP endpoint handler
async fn handle_mcp(Json(request): Json<MCPRequest>) -> Json<MCPResponse> {
    let response = server.handle_request(request).await;
    Json(response)
}
```

#### WebSocket Transport

For real-time updates:

```rust
// WebSocket handler
async fn handle_websocket(ws: WebSocket, server: Arc<MCPServer>) {
    let (tx, rx) = ws.split();
    
    // Handle incoming messages
    rx.for_each(|msg| async {
        if let Ok(request) = parse_json_rpc(msg) {
            let response = server.handle_request(request).await;
            tx.send(serialize_json_rpc(response)).await;
        }
    }).await;
}
```

## Security Considerations

### Authentication

MCP doesn't specify authentication, but SwissArmyHammer supports:

```json
// With API key
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "authentication": {
      "type": "bearer",
      "token": "sk-..."
    }
  },
  "id": 1
}
```

### Rate Limiting

Prevent abuse:

```rust
// Rate limit configuration
rate_limiter: {
    requests_per_minute: 100,
    burst_size: 20,
    per_method: {
        "prompts/get": 50,
        "resources/read": 30
    }
}
```

### Input Validation

All inputs are validated:

```rust
// Validate prompt arguments
fn validate_arguments(args: &HashMap<String, Value>) -> Result<()> {
    // Check required fields
    // Validate data types
    // Sanitize inputs
    // Check size limits
}
```

## Performance Optimization

### Caching

Responses are cached for efficiency:

```rust
// Cache configuration
cache: {
    prompts_list: {
        ttl: 300,  // 5 minutes
        max_size: 1000
    },
    rendered_prompts: {
        ttl: 3600,  // 1 hour
        max_size: 10000
    }
}
```

### Streaming

Large responses support streaming:

```json
// Streaming response
{
  "jsonrpc": "2.0",
  "result": {
    "stream": true,
    "chunks": [
      {"index": 0, "data": "First chunk..."},
      {"index": 1, "data": "Second chunk..."},
      {"index": 2, "data": "Final chunk", "final": true}
    ]
  },
  "id": 9
}
```

## Testing MCP

### Manual Testing

Test with curl:

```bash
# Test initialize
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | \
  swissarmyhammer serve --transport stdio

# Test prompts list
curl -X POST http://localhost:3333/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"prompts/list","params":{},"id":2}'
```

### Automated Testing

```rust
#[test]
async fn test_mcp_protocol() {
    let server = MCPServer::new_test();
    
    // Test initialize
    let response = server.handle_request(json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {},
        "id": 1
    })).await;
    
    assert_eq!(response["result"]["serverInfo"]["name"], "swissarmyhammer");
}
```

## Debugging

### Enable Debug Logging

```yaml
logging:
  modules:
    swissarmyhammer::mcp: debug
```

### Request/Response Logging

```rust
// Log all MCP traffic
middleware: {
    log_requests: true,
    log_responses: true,
    log_errors: true,
    pretty_print: true
}
```

### Protocol Inspector

```bash
# Inspect MCP traffic
swissarmyhammer mcp-inspector --port 3334

# Connect through inspector
export MCP_PROXY=http://localhost:3334
```

## Best Practices

1. **Always validate inputs** - Never trust client data
2. **Handle errors gracefully** - Return proper error codes
3. **Implement timeouts** - Prevent hanging requests
4. **Cache when possible** - Reduce computation
5. **Log important events** - Aid debugging
6. **Version your changes** - Maintain compatibility
7. **Document extensions** - Help client implementers

## Next Steps

- Implement [Claude Code Integration](./claude-code-integration.md)
- Review [API Reference](./api-reference.md) for details
- See [Troubleshooting](./troubleshooting.md) for common issues
- Check [Examples](./examples.md) for implementation patterns