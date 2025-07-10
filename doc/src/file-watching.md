# File Watching

SwissArmyHammer includes a powerful file watching system that automatically detects and reloads prompt changes without restarting the server.

## How It Works

The file watcher monitors your prompt directories for changes and automatically:

1. **Detects** new, modified, or deleted prompt files
2. **Validates** changed files for syntax errors
3. **Reloads** prompts into memory
4. **Notifies** connected clients of updates
5. **Maintains** state during the reload process

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ File System │────>│ File Watcher │────>│ Prompt Cache │
└─────────────┘     └──────────────┘     └──────────────┘
                            │                     │
                            ▼                     ▼
                    ┌──────────────┐     ┌──────────────┐
                    │  Validator   │     │ MCP Clients  │
                    └──────────────┘     └──────────────┘
```

## Configuration

### Basic Settings

Configure file watching in your `config.yaml`:

```yaml
watch:
  # Enable/disable file watching
  enabled: true
  
  # Check interval in milliseconds
  interval: 1000
  
  # Debounce delay to batch rapid changes
  debounce: 500
  
  # Maximum files to process per cycle
  batch_size: 100
```

### Advanced Options

```yaml
watch:
  # File patterns to watch
  patterns:
    - "**/*.md"
    - "**/*.markdown"
    - "**/prompts.yaml"
  
  # Patterns to ignore
  ignore:
    - "**/node_modules/**"
    - "**/.git/**"
    - "**/target/**"
    - "**/*.swp"
    - "**/*~"
    - "**/.DS_Store"
  
  # Watch strategy
  strategy: efficient  # efficient, aggressive, polling
  
  # Platform-specific settings
  platform:
    # macOS FSEvents
    macos:
      use_fsevents: true
      latency: 0.1
    
    # Linux inotify
    linux:
      use_inotify: true
      max_watches: 8192
    
    # Windows
    windows:
      use_polling: false
      poll_interval: 1000
```

## Watch Strategies

### Efficient (Default)

Best for most use cases:

```yaml
watch:
  strategy: efficient
  # Uses native OS file watching APIs
  # Low CPU usage
  # May have slight delay on some systems
```

### Aggressive

For development with frequent changes:

```yaml
watch:
  strategy: aggressive
  interval: 100      # Check every 100ms
  debounce: 50       # Minimal debounce
  # Higher CPU usage
  # Near-instant updates
```

### Polling

Fallback for compatibility:

```yaml
watch:
  strategy: polling
  interval: 2000     # Poll every 2 seconds
  # Works everywhere
  # Higher CPU usage
  # Slower updates
```

## File Events

### Supported Events

The watcher handles these file system events:

1. **Created** - New prompt files added
2. **Modified** - Existing files changed
3. **Deleted** - Files removed
4. **Renamed** - Files moved or renamed
5. **Metadata** - Permission or timestamp changes

### Event Processing

```yaml
# Event processing configuration
watch:
  events:
    # Process creation events
    create:
      enabled: true
      validate: true
      
    # Process modification events
    modify:
      enabled: true
      validate: true
      reload_delay: 100  # ms
      
    # Process deletion events
    delete:
      enabled: true
      cleanup_cache: true
      
    # Process rename events
    rename:
      enabled: true
      track_moves: true
```

## Validation

### Automatic Validation

Files are validated before reload:

```yaml
watch:
  validation:
    # Enable validation
    enabled: true
    
    # Validation rules
    rules:
      # Check YAML front matter
      yaml_syntax: true
      
      # Validate required fields
      required_fields:
        - name
        - title
        - description
      
      # Check template syntax
      template_syntax: true
      
      # Maximum file size
      max_size: 1MB
    
    # What to do on validation failure
    on_failure: warn  # warn, ignore, stop
```

### Validation Errors

When validation fails:

```
[WARN] Validation failed for prompts/invalid.md:
  - Line 5: Invalid YAML syntax
  - Missing required field: 'title'
  - Template error: Unclosed tag '{% if'
  
File will not be loaded. Fix errors and save again.
```

## Performance

### Optimization Tips

1. **Exclude unnecessary paths**:
   ```yaml
   watch:
     ignore:
       - "**/backup/**"
       - "**/archive/**"
       - "**/*.log"
   ```

2. **Tune intervals for your workflow**:
   ```yaml
   # For active development
   watch:
     interval: 500
     debounce: 250
   
   # For production
   watch:
     interval: 5000
     debounce: 2000
   ```

3. **Limit watch scope**:
   ```yaml
   watch:
     # Only watch specific directories
     directories:
       - ./.swissarmyhammer/prompts
       - ~/.swissarmyhammer/prompts
     # Don't watch subdirectories
     recursive: false
   ```

### Resource Usage

Monitor watcher resource usage:

```bash
# Check watcher status
swissarmyhammer doctor --watch

# Show watcher statistics
swissarmyhammer status --verbose

# Output:
File Watcher Status:
  Strategy: efficient
  Files watched: 156
  Directories: 12
  CPU usage: 0.1%
  Memory: 2.4MB
  Events processed: 1,234
  Last reload: 2 minutes ago
```

## Debugging

### Enable Debug Logging

```yaml
logging:
  modules:
    swissarmyhammer::watcher: debug
```

### Common Issues

1. **Changes not detected**:
   ```bash
   # Check if watching is enabled
   swissarmyhammer config get watch.enabled
   
   # Test file watching
   swissarmyhammer test --watch
   ```

2. **High CPU usage**:
   ```yaml
   # Increase intervals
   watch:
     interval: 2000
     debounce: 1000
   
   # Use efficient strategy
   watch:
     strategy: efficient
   ```

3. **Too many open files**:
   ```bash
   # Linux: Increase inotify watches
   echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
   sudo sysctl -p
   
   # macOS: Usually not an issue with FSEvents
   
   # Windows: Use polling fallback
   ```

## Platform-Specific Notes

### macOS

Uses FSEvents for efficient watching:

```yaml
watch:
  platform:
    macos:
      use_fsevents: true
      # FSEvents latency in seconds
      latency: 0.1
      # Ignore events older than
      ignore_older_than: 10  # seconds
```

### Linux

Uses inotify with automatic limits:

```yaml
watch:
  platform:
    linux:
      use_inotify: true
      # Will warn if approaching limits
      warn_threshold: 0.8
      # Fallback to polling if needed
      auto_fallback: true
```

### Windows

Uses ReadDirectoryChangesW:

```yaml
watch:
  platform:
    windows:
      # Buffer size for changes
      buffer_size: 65536
      # Watch subtree
      watch_subtree: true
      # Notification filters
      filters:
        - file_name
        - last_write
        - size
```

## Integration

### Client Notifications

Clients are notified of changes:

```javascript
// MCP client receives notification
client.on('prompt.changed', (event) => {
  console.log(`Prompt ${event.name} was ${event.type}`);
  // Refresh UI, clear caches, etc.
});
```

### Hooks

Run commands on file changes:

```yaml
watch:
  hooks:
    # Before processing changes
    pre_reload:
      - echo "Reloading prompts..."
    
    # After successful reload
    post_reload:
      - ./scripts/notify-team.sh
      - ./scripts/update-index.sh
    
    # On reload failure
    on_error:
      - ./scripts/alert-admin.sh
```

### API Access

Query watcher status via API:

```bash
# Get watcher status
curl http://localhost:3333/api/watcher/status

# Get recent events
curl http://localhost:3333/api/watcher/events

# Trigger manual reload
curl -X POST http://localhost:3333/api/watcher/reload
```

## Best Practices

### Development

1. **Use aggressive watching** for immediate feedback
2. **Enable validation** to catch errors early
3. **Watch only active directories** to reduce overhead
4. **Use debug logging** to troubleshoot issues

### Production

1. **Use efficient strategy** for lower resource usage
2. **Increase intervals** to reduce CPU load
3. **Disable watching** if prompts rarely change
4. **Monitor resource usage** regularly

### Large Projects

1. **Exclude build directories** and dependencies
2. **Use specific patterns** instead of wildcards
3. **Consider splitting** prompts across multiple directories
4. **Implement caching** to reduce reload impact

## Manual Control

### CLI Commands

Control file watching manually:

```bash
# Pause file watching
swissarmyhammer watch pause

# Resume file watching
swissarmyhammer watch resume

# Force reload all prompts
swissarmyhammer watch reload

# Show watch status
swissarmyhammer watch status
```

### Environment Variables

Override watch settings:

```bash
# Disable watching
export SWISSARMYHAMMER_WATCH_ENABLED=false

# Change interval
export SWISSARMYHAMMER_WATCH_INTERVAL=5000

# Force polling strategy
export SWISSARMYHAMMER_WATCH_STRATEGY=polling
```

## Troubleshooting

### Diagnostic Commands

```bash
# Run watcher diagnostics
swissarmyhammer doctor --watch

# Test file detection
echo "test" >> prompts/test.md
swissarmyhammer watch test

# Monitor events in real-time
swissarmyhammer watch monitor
```

### Common Solutions

1. **Linux: Increase inotify limits**
2. **macOS: Grant full disk access**
3. **Windows: Run as administrator**
4. **All: Check file permissions**
5. **All: Verify ignore patterns**

## Next Steps

- Configure watching in [Configuration](./configuration.md)
- Learn about [Prompt Organization](./prompt-organization.md)
- Understand [Prompt Overrides](./prompt-overrides.md)
- Read [Troubleshooting](./troubleshooting.md) for more help