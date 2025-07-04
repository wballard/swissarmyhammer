# doctor Command

The `doctor` command performs comprehensive health checks on your SwissArmyHammer installation and configuration. It identifies issues and provides recommendations for optimal operation.

## Usage

```bash
swissarmyhammer doctor [OPTIONS]
```

## Overview

The doctor command checks:
- Installation integrity and version compatibility
- Configuration file validity
- Prompt directory accessibility and structure
- Prompt file syntax and metadata
- MCP server functionality
- System dependencies and environment

## Options

### `--verbose`
- **Description**: Enable detailed output with additional diagnostic information
- **Default**: Disabled
- **Example**: Shows file paths, configuration details, and system information

```bash
swissarmyhammer doctor --verbose
```

### `--json`
- **Description**: Output results in JSON format for programmatic use
- **Default**: Human-readable text output
- **Example**: Useful for scripts and automation

```bash
swissarmyhammer doctor --json
```

### `--fix`
- **Description**: Automatically fix issues when possible
- **Default**: Report-only mode
- **Example**: Creates missing directories, fixes permissions

```bash
swissarmyhammer doctor --fix
```

### `--check <CATEGORY>`
- **Description**: Run specific check categories only
- **Values**: `installation`, `config`, `prompts`, `mcp`, `system`
- **Repeatable**: Can specify multiple categories

```bash
# Check only prompt-related issues
swissarmyhammer doctor --check prompts

# Check multiple categories
swissarmyhammer doctor --check config --check prompts
```

## Check Categories

### Installation Checks

Verifies SwissArmyHammer installation:

#### Binary Location
- Checks if `swissarmyhammer` is in PATH
- Verifies executable permissions
- Confirms version compatibility

#### Dependencies
- Validates system requirements
- Checks for required libraries
- Verifies runtime dependencies

#### Example Output
```
✓ SwissArmyHammer binary found: /usr/local/bin/swissarmyhammer
✓ Version: 0.1.0 (latest)
✓ Executable permissions: OK
✓ System dependencies: All present
```

### Configuration Checks

Validates configuration files and settings:

#### Configuration File
- Checks for valid TOML syntax
- Validates configuration schema
- Identifies deprecated settings

#### Directory Structure
- Verifies default directories exist
- Checks directory permissions
- Validates custom prompt directories

#### Environment Variables
- Lists relevant environment variables
- Checks for conflicts or inconsistencies
- Validates variable values

#### Example Output
```
✓ Configuration file: ~/.swissarmyhammer/config.toml
✓ Configuration syntax: Valid TOML
✓ Default directories: Created and accessible
⚠ Custom directory not found: /nonexistent/prompts
✓ Environment variables: No conflicts
```

### Prompt Checks

Analyzes prompt files and library structure:

#### Directory Scanning
- Scans all configured prompt directories
- Counts prompt files by category
- Identifies orphaned or miscategorized files

#### File Validation
- Validates YAML front matter syntax
- Checks required fields presence
- Verifies argument specifications

#### Content Analysis
- Validates Liquid template syntax
- Checks for common template errors
- Identifies missing or broken references

#### Duplicate Detection
- Finds prompts with identical names
- Shows override hierarchy
- Warns about potential conflicts

#### Example Output
```
✓ Prompt directories: 3 found, all accessible
✓ Prompt files: 47 total, 45 valid
✗ Invalid prompts: 2 files with errors
  - debug-helper.md: Missing required field 'description'
  - code-review.md: Invalid YAML syntax on line 8
⚠ Duplicate names: 1 conflict found
  - 'help' defined in both builtin and ~/.swissarmyhammer/prompts/
✓ Template syntax: All valid
```

### MCP Checks

Tests Model Context Protocol functionality:

#### Server Startup
- Attempts to start MCP server
- Tests port binding
- Verifies server responds to requests

#### Protocol Compliance
- Validates MCP protocol responses
- Checks capability advertisements
- Tests prompt exposure format

#### Integration Status
- Checks Claude Code configuration
- Tests end-to-end connectivity
- Validates prompt accessibility

#### Example Output
```
✓ MCP server startup: Success on port 8080
✓ Protocol compliance: All tests passed
✓ Prompt exposure: 45 prompts available
⚠ Claude Code integration: Not configured
  Run: claude mcp add swissarmyhammer swissarmyhammer serve
```

### System Checks

Analyzes system environment and performance:

#### Operating System
- Identifies OS and version
- Checks compatibility
- Validates system requirements

#### File System
- Tests file watching capabilities
- Checks disk space availability
- Validates permissions

#### Performance
- Measures prompt loading time
- Tests file watching responsiveness
- Checks memory usage patterns

#### Example Output
```
✓ Operating system: macOS 14.0 (supported)
✓ File system: APFS with file watching support
✓ Disk space: 15.2 GB available
✓ Performance: Prompt loading < 100ms
⚠ Memory usage: High with 1000+ prompts (consider --watch false)
```

## Common Issues and Solutions

### Installation Issues

#### SwissArmyHammer Not Found
```
✗ SwissArmyHammer binary: Not found in PATH
```

**Solutions:**
- Install SwissArmyHammer: `curl -sSL https://install.sh | bash`
- Add to PATH: `export PATH="$HOME/.local/bin:$PATH"`
- Verify installation: `which swissarmyhammer`

#### Permission Denied
```
✗ Executable permissions: Permission denied
```

**Solutions:**
```bash
# Fix permissions
chmod +x $(which swissarmyhammer)

# Or reinstall
curl -sSL https://install.sh | bash
```

### Configuration Issues

#### Invalid Configuration File
```
✗ Configuration syntax: Invalid TOML at line 15
```

**Solutions:**
- Validate TOML syntax online
- Check for missing quotes or brackets
- Reset to defaults: `swissarmyhammer doctor --fix`

#### Missing Directories
```
⚠ Prompt directory not accessible: /custom/prompts
```

**Solutions:**
```bash
# Create missing directory
mkdir -p /custom/prompts

# Fix automatically
swissarmyhammer doctor --fix
```

### Prompt Issues

#### Invalid YAML Front Matter
```
✗ Invalid prompts: 3 files with YAML errors
  - code-review.md: missing required field 'name'
```

**Solutions:**
- Add missing required fields
- Validate YAML syntax
- Use `swissarmyhammer test <prompt>` for detailed errors

#### Duplicate Prompt Names
```
⚠ Duplicate names: 'help' defined in multiple locations
```

**Solutions:**
- Rename one of the conflicting prompts
- Use different directories for different contexts
- Check prompt override hierarchy

#### Template Syntax Errors
```
✗ Template errors: 2 prompts with Liquid syntax issues
  - debug.md: Unknown filter 'unknownfilter'
```

**Solutions:**
- Fix Liquid template syntax
- Check available filters: see [Custom Filters](./custom-filters.md)
- Test templates: `swissarmyhammer test <prompt>`

### MCP Issues

#### Server Won't Start
```
✗ MCP server startup: Failed to bind to port 8080
```

**Solutions:**
```bash
# Try different port
swissarmyhammer serve --port 8081

# Check what's using the port
lsof -i :8080  # macOS/Linux
netstat -an | findstr 8080  # Windows
```

#### Claude Code Not Configured
```
⚠ Claude Code integration: Not configured
```

**Solutions:**
```bash
# Add to Claude Code
claude mcp add swissarmyhammer swissarmyhammer serve

# Verify configuration
claude mcp list
```

### Performance Issues

#### Slow Prompt Loading
```
⚠ Performance: Prompt loading > 1000ms
```

**Solutions:**
- Reduce prompt directory size
- Disable file watching: `--watch false`
- Use SSDs for prompt storage
- Split large libraries into categories

#### High Memory Usage
```
⚠ Memory usage: 2.1 GB with file watching enabled
```

**Solutions:**
```bash
# Disable file watching
swissarmyhammer serve --watch false

# Limit prompt directories
swissarmyhammer serve --prompts ./essential-prompts
```

## Automated Fixes

With the `--fix` flag, doctor can automatically resolve:

### Directory Issues
- Creates missing prompt directories
- Sets appropriate permissions
- Creates default configuration file

### Configuration Issues
- Repairs malformed TOML files
- Sets missing default values
- Removes deprecated settings

### Permission Issues
- Fixes file and directory permissions
- Makes binaries executable
- Sets appropriate ownership

### Example Auto-Fix
```bash
swissarmyhammer doctor --fix

# Output:
Fixed: Created missing directory ~/.swissarmyhammer/prompts
Fixed: Set executable permission on swissarmyhammer binary
Fixed: Created default configuration file
Warning: Could not fix invalid YAML in code-review.md (manual intervention required)
```

## Output Formats

### Human-Readable (Default)
```
SwissArmyHammer Doctor Report
============================

Installation Checks:
✓ Binary found and executable
✓ Version 0.1.0 (latest)
✓ Dependencies satisfied

Configuration Checks:
✓ Configuration file valid
⚠ Custom directory not found: /tmp/prompts

Prompt Checks:
✓ 45 prompts loaded successfully
✗ 2 prompts with errors (see details below)

MCP Checks:
✓ Server starts successfully
✓ Protocol compliance verified

System Checks:
✓ OS compatibility confirmed
⚠ High memory usage detected

Summary: 3 warnings, 1 error found
```

### JSON Format
```json
{
  "timestamp": "2024-03-20T10:30:00Z",
  "version": "0.1.0",
  "checks": {
    "installation": {
      "status": "passed",
      "details": [
        {
          "check": "binary_found",
          "status": "passed",
          "message": "Binary found at /usr/local/bin/swissarmyhammer"
        }
      ]
    },
    "configuration": {
      "status": "warning",
      "details": [
        {
          "check": "custom_directory",
          "status": "warning", 
          "message": "Directory not found: /tmp/prompts",
          "fixable": true
        }
      ]
    }
  },
  "summary": {
    "total_checks": 15,
    "passed": 12,
    "warnings": 2,
    "errors": 1
  }
}
```

## Integration with CI/CD

Use doctor in automated workflows:

### GitHub Actions
```yaml
- name: Check SwissArmyHammer Health
  run: |
    swissarmyhammer doctor --json > health-report.json
    if [ $(jq '.summary.errors' health-report.json) -gt 0 ]; then
      echo "Health check failed"
      exit 1
    fi
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
swissarmyhammer doctor --check prompts
if [ $? -ne 0 ]; then
  echo "Prompt validation failed. Fix issues before committing."
  exit 1
fi
```

### Development Script
```bash
#!/bin/bash
# dev-setup.sh
echo "Setting up development environment..."
swissarmyhammer doctor --fix --verbose
echo "Health check complete. Run 'swissarmyhammer serve' to start."
```

## Best Practices

### Regular Health Checks
```bash
# Weekly health check
swissarmyhammer doctor --verbose

# Before important deployments
swissarmyhammer doctor --check mcp --check prompts
```

### Monitoring Integration
```bash
# Check and alert on issues
swissarmyhammer doctor --json | jq -r '.summary.errors' | \
  xargs -I {} sh -c 'if [ {} -gt 0 ]; then echo "Alert: SwissArmyHammer errors detected"; fi'
```

### Development Workflow
```bash
# After making prompt changes
swissarmyhammer doctor --check prompts --fix

# Before committing
swissarmyhammer doctor --check prompts
```

## Troubleshooting Doctor Issues

### Doctor Command Not Found
```bash
# Verify installation
which swissarmyhammer

# Reinstall if needed
curl -sSL https://install.sh | bash
```

### Doctor Hangs or Crashes
```bash
# Run with timeout
timeout 30s swissarmyhammer doctor --verbose

# Check specific categories
swissarmyhammer doctor --check installation
```

### False Positives
```bash
# Skip problematic checks
swissarmyhammer doctor --check config --check prompts

# Use verbose mode for details
swissarmyhammer doctor --verbose
```

## Next Steps

- Fix any issues identified by doctor
- Set up regular health monitoring
- Configure automated fixes where appropriate
- See [Troubleshooting](./troubleshooting.md) for detailed problem resolution
- Check [Configuration](./configuration.md) for advanced settings