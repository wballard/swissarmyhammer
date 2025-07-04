# serve Command

The `serve` command starts SwissArmyHammer as a Model Context Protocol (MCP) server, making your prompts available to Claude Code and other MCP clients.

## Usage

```bash
swissarmyhammer serve [OPTIONS]
```

## Overview

The serve command:
- Starts an MCP server that provides access to your prompt library
- Loads prompts from various directories (built-in, user, local)
- Watches for file changes and automatically reloads prompts
- Provides real-time access to prompts for Claude Code integration

## Options

### `--port <PORT>`
- **Description**: Port number for MCP communication
- **Default**: Automatically assigned by the system
- **Example**: `--port 8080`

```bash
swissarmyhammer serve --port 8080
```

### `--host <HOST>`
- **Description**: Host address to bind the server to
- **Default**: `localhost`
- **Example**: `--host 127.0.0.1`

```bash
swissarmyhammer serve --host 127.0.0.1
```

### `--prompts <DIRECTORY>`
- **Description**: Additional directories to load prompts from
- **Default**: Standard locations (`~/.swissarmyhammer/prompts`, `./prompts`)
- **Repeatable**: Can be used multiple times
- **Example**: `--prompts ./custom-prompts`

```bash
# Single custom directory
swissarmyhammer serve --prompts ./project-prompts

# Multiple directories
swissarmyhammer serve --prompts ./prompts --prompts ~/.custom-prompts
```

### `--builtin <BOOLEAN>`
- **Description**: Include built-in prompts in the library
- **Default**: `true`
- **Values**: `true`, `false`
- **Example**: `--builtin false`

```bash
# Disable built-in prompts
swissarmyhammer serve --builtin false

# Explicitly enable built-in prompts
swissarmyhammer serve --builtin true
```

### `--watch <BOOLEAN>`
- **Description**: Enable file watching for automatic prompt reloading
- **Default**: `true`
- **Values**: `true`, `false`
- **Example**: `--watch false`

```bash
# Disable file watching (for performance)
swissarmyhammer serve --watch false
```

### `--debug`
- **Description**: Enable debug logging for troubleshooting
- **Default**: Disabled
- **Output**: Detailed logs to stdout

```bash
swissarmyhammer serve --debug
```

### `--config <FILE>`
- **Description**: Path to configuration file
- **Default**: `~/.swissarmyhammer/config.toml`
- **Example**: `--config ./custom-config.toml`

```bash
swissarmyhammer serve --config ./project-config.toml
```

## Examples

### Basic Server

Start a basic MCP server with default settings:

```bash
swissarmyhammer serve
```

This loads:
- Built-in prompts
- User prompts from `~/.swissarmyhammer/prompts/`
- Local prompts from `./prompts/` (if exists)
- Enables file watching

### Development Server

For development with debug logging:

```bash
swissarmyhammer serve --debug
```

Output includes:
- Prompt loading details
- MCP protocol messages
- File watching events
- Error stack traces

### Custom Prompt Directory

Serve prompts from a specific directory:

```bash
swissarmyhammer serve --prompts /path/to/my/prompts
```

### Multiple Directories

Load prompts from multiple locations:

```bash
swissarmyhammer serve \
  --prompts ./project-prompts \
  --prompts ~/.shared-prompts \
  --prompts /team/common-prompts
```

### Project-Only Prompts

Serve only local project prompts (no built-in or user prompts):

```bash
swissarmyhammer serve \
  --prompts ./prompts \
  --builtin false
```

### Performance-Optimized

For large prompt collections, disable file watching:

```bash
swissarmyhammer serve \
  --watch false \
  --prompts ./large-prompt-collection
```

### Custom Port and Host

Specify network settings:

```bash
swissarmyhammer serve \
  --host 0.0.0.0 \
  --port 9000
```

## Prompt Loading Order

SwissArmyHammer loads prompts in this order:

1. **Built-in prompts** (if `--builtin true`)
   - Located in the binary
   - Categories: development, writing, analysis, etc.

2. **User prompts** (always loaded)
   - Location: `~/.swissarmyhammer/prompts/`
   - Your personal prompt library

3. **Custom directories** (from `--prompts` flags)
   - Processed in order specified
   - Can override earlier prompts with same name

4. **Local prompts** (always checked)
   - Location: `./prompts/` in current directory
   - Project-specific prompts

### Prompt Override Behavior

When prompts have the same name:
- **Later sources override earlier ones**
- **Local prompts have highest priority**
- **Built-in prompts have lowest priority**

Example hierarchy:
```
./prompts/code-review.md          (highest priority)
~/.custom/code-review.md          (from --prompts ~/.custom)
~/.swissarmyhammer/prompts/code-review.md  (user prompts)
built-in:code-review              (lowest priority)
```

## File Watching

When file watching is enabled (`--watch true`), the server automatically:

### Detects Changes
- New prompt files added
- Existing prompt files modified
- Prompt files deleted
- Directory structure changes

### Reloads Prompts
- Parses updated files
- Validates YAML front matter
- Updates the prompt library
- Notifies connected MCP clients

### Handles Errors
- Invalid YAML syntax
- Missing required fields
- Template compilation errors
- Logs errors without stopping the server

### Performance Considerations

File watching uses system resources:
- **Memory**: Stores file metadata
- **CPU**: Processes file system events
- **Disk I/O**: Reads modified files

For large prompt collections (1000+ files), consider:
```bash
# Disable watching for better performance
swissarmyhammer serve --watch false
```

## MCP Protocol Details

### Server Capabilities

SwissArmyHammer advertises these MCP capabilities:

```json
{
  "capabilities": {
    "prompts": {
      "listChanged": true
    },
    "tools": {
      "listChanged": false
    }
  }
}
```

### Prompt Exposure

Each prompt becomes an MCP prompt with:
- **Name**: From prompt's `name` field
- **Description**: From prompt's `description` field
- **Arguments**: From prompt's `arguments` array

### Example MCP Prompt

A SwissArmyHammer prompt:
```yaml
---
name: code-review
title: Code Review Assistant
description: Reviews code for best practices and issues
arguments:
  - name: code
    description: Code to review
    required: true
  - name: language
    description: Programming language
    required: false
    default: auto-detect
---
```

Becomes this MCP prompt:
```json
{
  "name": "code-review",
  "description": "Reviews code for best practices and issues",
  "arguments": [
    {
      "name": "code",
      "description": "Code to review",
      "required": true
    },
    {
      "name": "language", 
      "description": "Programming language",
      "required": false
    }
  ]
}
```

## Integration with Claude Code

### Configuration

Add SwissArmyHammer to Claude Code's MCP configuration:

```bash
claude mcp add swissarmyhammer swissarmyhammer serve
```

### Custom Configuration

Add with specific options:

```bash
claude mcp add project_sah swissarmyhammer serve --prompts ./project-prompts --debug
```

### Multiple Servers

Run different SwissArmyHammer instances:

```bash
# Global prompts
claude mcp add sah_global swissarmyhammer serve

# Project-specific prompts  
claude mcp add sah_project swissarmyhammer serve --prompts ./prompts --builtin false
```

## Logging and Output

### Standard Output

Normal operation logs:
```
2024-03-20T10:30:00Z INFO SwissArmyHammer MCP Server starting
2024-03-20T10:30:00Z INFO Loaded 25 prompts from 3 directories
2024-03-20T10:30:00Z INFO Server listening on localhost:8080
2024-03-20T10:30:00Z INFO MCP client connected
```

### Debug Output

With `--debug` flag:
```
2024-03-20T10:30:00Z DEBUG Loading prompts from: ~/.swissarmyhammer/prompts
2024-03-20T10:30:00Z DEBUG Found prompt file: code-review.md
2024-03-20T10:30:00Z DEBUG Parsed prompt: code-review (Code Review Assistant)
2024-03-20T10:30:00Z DEBUG MCP request: prompts/list
2024-03-20T10:30:00Z DEBUG MCP response: 25 prompts returned
```

### Error Handling

The server continues running even with errors:
```
2024-03-20T10:30:00Z ERROR Failed to parse prompt: invalid-prompt.md
2024-03-20T10:30:00Z ERROR   YAML error: missing required field 'description'
2024-03-20T10:30:00Z INFO  Continuing with 24 valid prompts
```

## Troubleshooting

### Server Won't Start

**Check port availability:**
```bash
# Try a specific port
swissarmyhammer serve --port 8080

# Check if port is in use
lsof -i :8080  # macOS/Linux
netstat -an | findstr 8080  # Windows
```

**Check permissions:**
```bash
# Run with debug to see detailed errors
swissarmyhammer serve --debug
```

### Prompts Not Loading

**Verify directories exist:**
```bash
# Check default directories
ls -la ~/.swissarmyhammer/prompts
ls -la ./prompts

# Check custom directories
ls -la /path/to/custom/prompts
```

**Validate prompt syntax:**
```bash
# Test individual prompts
swissarmyhammer test prompt-name

# Validate all prompts
swissarmyhammer doctor
```

### Performance Issues

**Large prompt collections:**
```bash
# Disable file watching
swissarmyhammer serve --watch false

# Limit to specific directories
swissarmyhammer serve --prompts ./essential-prompts --builtin false
```

**Memory usage:**
```bash
# Monitor memory usage
top -p $(pgrep swissarmyhammer)  # Linux
top | grep swissarmyhammer       # macOS
```

### Connection Issues

**MCP client can't connect:**
```bash
# Check server is running
ps aux | grep swissarmyhammer

# Test with different host/port
swissarmyhammer serve --host 127.0.0.1 --port 8080

# Check firewall settings
```

**Debug MCP communication:**
```bash
# Enable debug logging
swissarmyhammer serve --debug

# Save logs to file
swissarmyhammer serve --debug > server.log 2>&1
```

## Configuration File

Create a configuration file for persistent settings:

```toml
# ~/.swissarmyhammer/config.toml

[server]
host = "localhost"
port = 8080
debug = false

[prompts]
builtin = true
watch = true
directories = [
    "~/.swissarmyhammer/prompts",
    "./prompts",
    "/team/shared-prompts"
]
```

Use with:
```bash
swissarmyhammer serve --config ~/.swissarmyhammer/config.toml
```

## Environment Variables

Configure through environment variables:

```bash
export SWISSARMYHAMMER_HOST=localhost
export SWISSARMYHAMMER_PORT=8080
export SWISSARMYHAMMER_DEBUG=true
export SWISSARMYHAMMER_PROMPTS_DIR=/custom/prompts

swissarmyhammer serve
```

## Best Practices

### 1. Use Consistent Directory Structure

```
~/.swissarmyhammer/prompts/
├── development/
├── writing/
├── analysis/
└── productivity/
```

### 2. Enable Debug During Development

```bash
swissarmyhammer serve --debug
```

### 3. Use Project-Specific Servers

```bash
# In each project
claude mcp add project_sah swissarmyhammer serve --prompts ./prompts
```

### 4. Monitor Performance

```bash
# For large collections
swissarmyhammer serve --watch false --debug
```

### 5. Version Control Integration

```bash
# .gitignore
.swissarmyhammer/cache/
.swissarmyhammer/logs/

# Keep prompts in version control
git add prompts/
```

## Next Steps

- Learn about [Claude Code Integration](./claude-code-integration.md) setup
- Explore [Configuration](./configuration.md) options
- See [Troubleshooting](./troubleshooting.md) for common issues
- Check [Built-in Prompts](./builtin-prompts.md) reference