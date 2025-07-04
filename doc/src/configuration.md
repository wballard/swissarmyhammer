# Configuration

SwissArmyHammer offers flexible configuration options through configuration files, environment variables, and command-line arguments. This guide covers all configuration methods and settings.

## Configuration File

### Location

SwissArmyHammer looks for configuration files in this order:

1. `./swissarmyhammer.toml` (current directory)
2. `~/.swissarmyhammer/config.toml` (user directory)
3. `/etc/swissarmyhammer/config.toml` (system-wide)

### Format

Configuration uses TOML format:

```toml
# ~/.swissarmyhammer/config.toml

# Server configuration
[server]
host = "localhost"
port = 8080
debug = false
timeout = 30000  # milliseconds

# Prompt directories
[prompts]
directories = [
    "~/.swissarmyhammer/prompts",
    "./prompts",
    "/opt/company/prompts"
]
builtin = true
watch = true

# File watching configuration
[watch]
enabled = true
poll_interval = 1000  # milliseconds
max_depth = 5
ignore_patterns = [
    "*.tmp",
    "*.swp",
    ".git/*",
    "__pycache__/*"
]

# Logging configuration
[log]
level = "info"  # debug, info, warn, error
file = "~/.swissarmyhammer/logs/server.log"
rotate = true
max_size = "10MB"
max_age = 30  # days

# Cache configuration
[cache]
enabled = true
directory = "~/.swissarmyhammer/cache"
ttl = 3600  # seconds
max_size = "100MB"

# Template engine configuration
[template]
strict_variables = false
strict_filters = false
custom_filters_path = "~/.swissarmyhammer/filters"

# Security settings
[security]
allow_file_access = false
allow_network_access = false
sandbox_mode = true
allowed_domains = ["api.company.com", "github.com"]

# MCP specific settings
[mcp]
protocol_version = "1.0"
capabilities = ["prompts", "notifications"]
max_prompt_size = 1048576  # 1MB
compression = true
```

## Environment Variables

All configuration options can be set via environment variables:

### Naming Convention

- Prefix: `SWISSARMYHAMMER_`
- Nested values use underscores: `SECTION_KEY`
- Arrays use comma separation

### Examples

```bash
# Server settings
export SWISSARMYHAMMER_SERVER_HOST=0.0.0.0
export SWISSARMYHAMMER_SERVER_PORT=9000
export SWISSARMYHAMMER_SERVER_DEBUG=true

# Prompt directories (comma-separated)
export SWISSARMYHAMMER_PROMPTS_DIRECTORIES="/opt/prompts,~/my-prompts"
export SWISSARMYHAMMER_PROMPTS_BUILTIN=false

# Logging
export SWISSARMYHAMMER_LOG_LEVEL=debug
export SWISSARMYHAMMER_LOG_FILE=/var/log/swissarmyhammer.log

# File watching
export SWISSARMYHAMMER_WATCH_ENABLED=true
export SWISSARMYHAMMER_WATCH_POLL_INTERVAL=2000

# Security
export SWISSARMYHAMMER_SECURITY_SANDBOX_MODE=true
export SWISSARMYHAMMER_SECURITY_ALLOWED_DOMAINS="api.example.com,cdn.example.com"
```

### Precedence

Configuration precedence (highest to lowest):

1. Command-line arguments
2. Environment variables
3. Configuration files
4. Default values

## Command-Line Options

### Global Options

```bash
swissarmyhammer [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  --config <FILE>     Use specific configuration file
  --verbose          Enable verbose output
  --quiet            Suppress non-error output
  --no-color         Disable colored output
  --json             Output in JSON format
  --help             Show help information
  --version          Show version information
```

### Per-Command Configuration

Override configuration for specific commands:

```bash
# Override server settings
swissarmyhammer serve --host 0.0.0.0 --port 9000 --debug

# Override prompt directories
swissarmyhammer serve --prompts /custom/prompts --no-builtin

# Override logging
swissarmyhammer serve --log-level debug --log-file server.log
```

## Configuration Sections

### Server Configuration

Controls the MCP server behavior:

```toml
[server]
# Network binding
host = "localhost"      # IP address or hostname
port = 8080            # Port number (0 for auto-assign)

# Performance
workers = 4            # Number of worker threads
max_connections = 100  # Maximum concurrent connections
timeout = 30000        # Request timeout in milliseconds

# Debugging
debug = false          # Enable debug mode
trace = false          # Enable trace logging
metrics = true         # Enable metrics collection
```

### Prompt Configuration

Manages prompt loading and directories:

```toml
[prompts]
# Directories to load prompts from
directories = [
    "~/.swissarmyhammer/prompts",    # User prompts
    "./prompts",                      # Project prompts
    "/opt/shared/prompts"             # Shared prompts
]

# Loading behavior
builtin = true          # Include built-in prompts
watch = true            # Enable file watching
recursive = true        # Scan directories recursively
follow_symlinks = false # Follow symbolic links

# Filtering
include_patterns = ["*.md", "*.markdown"]
exclude_patterns = ["*.draft.md", "test-*"]

# Validation
strict_validation = true     # Fail on invalid prompts
required_fields = ["name", "description"]
max_file_size = "1MB"       # Maximum prompt file size
```

### File Watching

Configure file system monitoring:

```toml
[watch]
enabled = true              # Enable/disable watching
strategy = "efficient"      # efficient, aggressive, polling

# Polling strategy settings
poll_interval = 1000        # Milliseconds between polls
poll_timeout = 100          # Polling timeout

# Watch behavior
debounce = 500             # Milliseconds to wait for changes to settle
max_depth = 10             # Maximum directory depth
batch_events = true        # Batch multiple changes

# Ignore patterns
ignore_patterns = [
    "*.tmp",
    "*.swp",
    "*.bak",
    ".git/**",
    ".svn/**",
    "__pycache__/**",
    "node_modules/**"
]

# Performance
max_watches = 10000        # Maximum number of watches
event_buffer_size = 1000   # Event queue size
```

### Logging

Configure logging behavior:

```toml
[log]
# Log level: trace, debug, info, warn, error
level = "info"

# Console output
console = true
console_format = "pretty"  # pretty, json, compact
console_colors = true

# File logging
file = "~/.swissarmyhammer/logs/server.log"
file_format = "json"
rotate = true
max_size = "10MB"
max_files = 5
max_age = 30  # days

# Log filtering
include_modules = ["server", "prompts"]
exclude_modules = ["watcher"]

# Performance
buffer_size = 8192
async = true
```

### Cache Configuration

Control caching behavior:

```toml
[cache]
enabled = true
directory = "~/.swissarmyhammer/cache"

# Cache strategy
strategy = "lru"           # lru, lfu, ttl
max_entries = 1000
max_size = "100MB"

# Time-based settings
ttl = 3600                # Default TTL in seconds
refresh_ahead = 300       # Refresh cache 5 minutes before expiry

# Cache categories
[cache.prompts]
enabled = true
ttl = 7200

[cache.templates]
enabled = true
ttl = 3600

[cache.search]
enabled = false           # Disable search result caching
```

### Template Engine

Configure Liquid template processing:

```toml
[template]
# Parsing
strict_variables = false   # Error on undefined variables
strict_filters = false     # Error on undefined filters
error_mode = "warn"        # warn, error, ignore

# Custom extensions
custom_filters_path = "~/.swissarmyhammer/filters"
custom_tags_path = "~/.swissarmyhammer/tags"

# Security
allow_includes = true
include_paths = ["~/.swissarmyhammer/includes"]
max_render_depth = 10
max_iterations = 1000

# Performance
cache_templates = true
compile_cache = "~/.swissarmyhammer/template_cache"
```

### Security Settings

Control security features:

```toml
[security]
# Sandboxing
sandbox_mode = true        # Enable security sandbox
allow_file_access = false  # Allow template file access
allow_network_access = false # Allow network requests
allow_system_access = false # Allow system commands

# Network security
allowed_domains = [
    "api.company.com",
    "cdn.company.com",
    "github.com"
]
blocked_domains = [
    "malicious.site"
]

# File security
allowed_paths = [
    "~/Documents/projects",
    "/opt/shared/data"
]
blocked_paths = [
    "/etc",
    "/sys",
    "~/.ssh"
]

# Content security
max_input_size = "10MB"
max_output_size = "50MB"
sanitize_html = true
```

## Configuration Profiles

### Using Profiles

Define multiple configuration profiles:

```toml
# Default configuration
[default]
server.host = "localhost"
server.port = 8080
log.level = "info"

# Development profile
[profiles.development]
server.debug = true
log.level = "debug"
cache.enabled = false
template.strict_variables = true

# Production profile
[profiles.production]
server.host = "0.0.0.0"
server.workers = 8
log.level = "warn"
security.sandbox_mode = true

# Testing profile
[profiles.test]
server.port = 0  # Auto-assign
log.file = "/tmp/test.log"
prompts.directories = ["./test/fixtures"]
```

### Activating Profiles

```bash
# Via environment variable
export SWISSARMYHAMMER_PROFILE=production
swissarmyhammer serve

# Via command line
swissarmyhammer --profile development serve

# Multiple profiles (later overrides earlier)
swissarmyhammer --profile production --profile custom serve
```

## Advanced Configuration

### Dynamic Configuration

Load configuration from external sources:

```toml
[config]
# Load additional config from URL
remote_config = "https://config.company.com/swissarmyhammer"
remote_check_interval = 300  # seconds

# Load from environment-specific file
env_config = "/etc/swissarmyhammer/config.${ENV}.toml"

# Merge strategy
merge_strategy = "deep"  # deep, shallow, replace
```

### Hooks Configuration

Configure lifecycle hooks:

```toml
[hooks]
# Startup hooks
pre_start = [
    "~/scripts/pre-start.sh",
    "/opt/swissarmyhammer/hooks/validate.py"
]
post_start = [
    "~/scripts/notify-start.sh"
]

# Shutdown hooks
pre_stop = [
    "~/scripts/save-state.sh"
]
post_stop = [
    "~/scripts/cleanup.sh"
]

# Prompt hooks
pre_load = "~/scripts/validate-prompt.sh"
post_load = "~/scripts/index-prompt.sh"

# Error hooks
on_error = "~/scripts/error-handler.sh"

# Hook configuration
[hooks.config]
timeout = 30           # seconds
fail_on_error = false  # Continue if hook fails
environment = {
    CUSTOM_VAR = "value"
}
```

### Performance Tuning

Optimize for different scenarios:

```toml
[performance]
# Threading
thread_pool_size = 8
async_workers = 4
io_threads = 2

# Memory
max_memory = "2GB"
gc_interval = 300      # seconds
cache_pressure = 0.8   # Evict cache at 80% memory

# Network
connection_pool_size = 50
keep_alive = true
tcp_nodelay = true
socket_timeout = 30

# File I/O
read_buffer_size = 8192
write_buffer_size = 8192
use_mmap = true        # Memory-mapped files

# Optimizations
lazy_loading = true
parallel_parsing = true
compress_cache = true
```

### Monitoring Configuration

Enable monitoring and metrics:

```toml
[monitoring]
enabled = true

# Metrics collection
[monitoring.metrics]
enabled = true
interval = 60          # seconds
retention = 7          # days

# Metrics to collect
collect = [
    "cpu_usage",
    "memory_usage",
    "prompt_count",
    "request_rate",
    "error_rate",
    "cache_hit_rate"
]

# Export metrics
[monitoring.export]
format = "prometheus"  # prometheus, json, statsd
endpoint = "http://metrics.company.com:9090"
labels = {
    service = "swissarmyhammer",
    environment = "production"
}

# Health checks
[monitoring.health]
enabled = true
endpoint = "/health"
checks = [
    "server_status",
    "prompt_loading",
    "file_watcher",
    "cache_status"
]
```

## Configuration Examples

### Minimal Configuration

```toml
# Minimal working configuration
[server]
host = "localhost"
port = 8080

[prompts]
directories = ["~/.swissarmyhammer/prompts"]
```

### Development Configuration

```toml
# Development-optimized configuration
[server]
host = "localhost"
port = 8080
debug = true

[prompts]
directories = [
    "./prompts",
    "~/.swissarmyhammer/prompts"
]
watch = true

[log]
level = "debug"
console = true

[cache]
enabled = false  # Disable caching for development

[template]
strict_variables = true  # Catch template errors early
```

### Production Configuration

```toml
# Production-optimized configuration
[server]
host = "0.0.0.0"
port = 80
workers = 8
timeout = 60000

[prompts]
directories = [
    "/opt/swissarmyhammer/prompts",
    "/var/lib/swissarmyhammer/prompts"
]
builtin = true
watch = false  # Disable for performance

[log]
level = "warn"
file = "/var/log/swissarmyhammer/server.log"
rotate = true
max_size = "100MB"
max_files = 10

[cache]
enabled = true
strategy = "lru"
max_size = "1GB"

[security]
sandbox_mode = true
allow_file_access = false
allow_network_access = false

[monitoring]
enabled = true
metrics.enabled = true
health.enabled = true
```

### High-Performance Configuration

```toml
# Optimized for high load
[server]
workers = 16
max_connections = 1000
timeout = 120000

[performance]
thread_pool_size = 32
async_workers = 16
connection_pool_size = 200
lazy_loading = true
parallel_parsing = true

[cache]
enabled = true
strategy = "lfu"
max_size = "4GB"
refresh_ahead = 600

[watch]
enabled = false  # Disable for performance

[log]
level = "error"  # Minimize logging overhead
async = true
buffer_size = 65536
```

## Configuration Validation

### Validate Configuration

```bash
# Validate configuration file
swissarmyhammer config validate

# Validate specific file
swissarmyhammer config validate --file custom-config.toml

# Show effective configuration
swissarmyhammer config show

# Show configuration with sources
swissarmyhammer config show --sources
```

### Configuration Schema

```bash
# Generate configuration schema
swissarmyhammer config schema > config-schema.json

# Validate against schema
swissarmyhammer config validate --schema config-schema.json
```

## Best Practices

### 1. Use Profiles

Separate configurations for different environments:

```toml
[profiles.local]
server.debug = true

[profiles.staging]
server.host = "staging.company.com"

[profiles.production]
server.host = "0.0.0.0"
security.sandbox_mode = true
```

### 2. Secure Sensitive Data

Never store secrets in configuration files:

```toml
# Bad
api_key = "sk-1234567890abcdef"

# Good - use environment variables
api_key = "${API_KEY}"
```

### 3. Document Configuration

Add comments explaining non-obvious settings:

```toml
# Increase timeout for slow network environments
timeout = 60000  # 1 minute

# Disable caching during development to see changes immediately
[cache]
enabled = false  # TODO: Enable for production
```

### 4. Version Control

Track configuration changes:

```bash
# .gitignore
config.local.toml
config.production.toml

# Track example configuration
config.example.toml
```

### 5. Validate Changes

Always validate configuration changes:

```bash
# Before deploying
swissarmyhammer config validate --file new-config.toml

# Test with dry run
swissarmyhammer serve --config new-config.toml --dry-run
```

## Troubleshooting

### Configuration Not Loading

1. Check file exists and is readable
2. Validate TOML syntax
3. Check environment variable names
4. Review precedence order

### Performance Issues

1. Disable file watching in production
2. Tune cache settings
3. Adjust worker counts
4. Enable performance monitoring

### Security Warnings

1. Review security settings
2. Enable sandbox mode
3. Restrict file and network access
4. Update allowed domains

## Next Steps

- See [CLI Reference](./cli-reference.md) for command-line options
- Learn about [File Watching](./file-watching.md) configuration
- Explore [Troubleshooting](./troubleshooting.md) for common issues
- Read [Security](./security.md) for security best practices