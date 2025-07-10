# Claude Code Integration

SwissArmyHammer integrates seamlessly with [Claude Code](https://docs.anthropic.com/en/docs/claude-code) through the Model Context Protocol (MCP). This guide shows you how to set up and use SwissArmyHammer with Claude Code.

## What is MCP?

The [Model Context Protocol](https://github.com/anthropics/model-context-protocol) allows AI assistants like Claude to access external tools and data sources. SwissArmyHammer acts as an MCP server, providing Claude Code with access to your prompt library.

## Installation

### 1. Install SwissArmyHammer

See the [Installation Guide](./installation.md) for detailed instructions. The quickest method:

```bash
# Install using the install script
curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | bash
```

### 2. Verify Installation

```bash
swissarmyhammer --version
```

### 3. Test the MCP Server

```bash
swissarmyhammer serve --help
```

## Configuration

### Add to Claude Code

Configure Claude Code to use SwissArmyHammer as an MCP server:

```bash
claude mcp add --scope user swissarmyhammer swissarmyhammer serve
```

This command:
- Adds `swissarmyhammer` as an MCP server name
- Uses `swissarmyhammer serve` as the command to start the server
- Sets user scope (available for your user account)

### Alternative Configuration Methods

#### Manual Configuration

If you prefer to configure manually, add this to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "swissarmyhammer": {
      "command": "swissarmyhammer",
      "args": ["serve"]
    }
  }
}
```

#### Project-Specific Configuration

For project-specific prompts, create a local configuration:

```bash
# In your project directory
claude mcp add --scope project swissarmyhammer_local swissarmyhammer serve --prompts ./prompts
```

## Verification

### Check MCP Configuration

List your configured MCP servers:

```bash
claude mcp list
```

You should see `swissarmyhammer` in the output.

### Test the Connection

Start Claude Code and verify SwissArmyHammer is connected:

1. Open Claude Code
2. Look for SwissArmyHammer prompts in the available tools
3. Try using a built-in prompt like `help` or `plan`

### Debug Connection Issues

If SwissArmyHammer doesn't appear in Claude Code:

```bash
# Check if the server starts correctly
swissarmyhammer serve --debug

# Verify prompts are loaded
swissarmyhammer list

# Check configuration
claude mcp list
```

## Usage in Claude Code

### Available Features

Once configured, SwissArmyHammer provides these features in Claude Code:

#### 1. Prompt Library Access
All your prompts become available as tools in Claude Code:

- Built-in prompts (code review, debugging, documentation)
- User prompts (in `~/.swissarmyhammer/prompts/`)
- Local prompts (in current project directory)

#### 2. Dynamic Arguments
Prompts with arguments become interactive forms in Claude Code:

```markdown
---
name: code-review
title: Code Review
description: Reviews code for best practices, bugs, and improvements
arguments:
  - name: code
    description: Code to review
    required: true
    type_hint: string
  - name: language
    description: Programming language
    required: false
    default: auto-detect
    type_hint: string
---
```

#### 3. Live Reloading
Changes to prompt files are automatically detected and reloaded.

### Using Prompts

#### Basic Usage

1. **Select a Prompt**: Choose from available SwissArmyHammer prompts
2. **Fill Arguments**: Provide required and optional parameters
3. **Execute**: Claude runs the prompt with your arguments

#### Example Workflow

1. **Code Review**:
   - Select `code-review` prompt
   - Paste your code in the `code` field
   - Set `language` if needed
   - Execute to get detailed code analysis

2. **Debug Helper**:
   - Select `debug` prompt
   - Describe your error in the `error` field
   - Get step-by-step debugging guidance

3. **Documentation**:
   - Select `docs` prompt
   - Provide code or specifications
   - Generate comprehensive documentation

### Advanced Usage

#### Prompt Chaining

Use multiple prompts in sequence:

```markdown
1. Use `analyze-code` to understand the codebase
2. Use `plan` to create implementation strategy  
3. Use `code-review` on the new code
4. Use `docs` to generate documentation
```

#### Custom Workflows

Create project-specific prompt workflows:

```markdown
# Project prompts in ./.swissarmyhammer/prompts/

## development/
- project-setup.md - Initialize new features
- code-standards.md - Apply project coding standards
- deployment.md - Deploy to staging/production

## documentation/  
- api-docs.md - Generate API documentation
- user-guide.md - Create user-facing documentation
- changelog.md - Generate release notes
```

## Configuration Options

### Server Configuration

The `swissarmyhammer serve` command accepts several options:

```bash
swissarmyhammer serve [OPTIONS]
```

#### Common Options

- `--port <PORT>` - Port for MCP communication (default: auto)
- `--host <HOST>` - Host to bind to (default: localhost)
- `--prompts <DIR>` - Additional prompt directories
- `--builtin <BOOL>` - Include built-in prompts (default: true)
- `--watch <BOOL>` - Enable file watching (default: true)
- `--debug` - Enable debug logging

#### Examples

```bash
# Basic server
swissarmyhammer serve

# Custom prompt directory
swissarmyhammer serve --prompts /path/to/prompts

# Multiple prompt directories
swissarmyhammer serve --prompts ./prompts --prompts ~/.custom-prompts

# Debug mode
swissarmyhammer serve --debug

# Disable built-in prompts
swissarmyhammer serve --builtin false
```

### Claude Code Configuration

#### Server Arguments

Pass arguments to the SwissArmyHammer server:

```bash
# Add with custom options
claude mcp add swissarmyhammer_custom swissarmyhammer serve --prompts ./project-prompts --debug
```

#### Environment Variables

Configure through environment variables:

```bash
export SWISSARMYHAMMER_PROMPTS_DIR=/path/to/prompts
export SWISSARMYHAMMER_DEBUG=true
claude mcp add swissarmyhammer swissarmyhammer serve
```

## Prompt Organization

### Directory Structure

Organize prompts for easy discovery in Claude Code:

```
~/.swissarmyhammer/prompts/
├── development/
│   ├── code-review.md
│   ├── debug-helper.md
│   ├── refactor.md
│   └── testing.md
├── writing/
│   ├── blog-post.md
│   ├── documentation.md
│   └── email.md
├── analysis/
│   ├── data-insights.md
│   └── research.md
└── productivity/
    ├── task-planning.md
    └── meeting-notes.md
```

### Naming Conventions

Use clear, descriptive names that work well in Claude Code:

```markdown
# Good - Clear and specific
code-review-python.md
debug-javascript-async.md
documentation-api.md

# Bad - Too generic
review.md
debug.md
docs.md
```

### Categories and Tags

Use categories and tags for better organization in Claude Code:

```yaml
---
name: python-code-review
title: Python Code Review
description: Reviews Python code for PEP 8, security, and performance
category: development
tags: ["python", "code-review", "pep8", "security"]
---
```

## Troubleshooting

### Common Issues

#### SwissArmyHammer Not Available

**Symptoms**: SwissArmyHammer prompts don't appear in Claude Code

**Solutions**:
1. Verify installation: `swissarmyhammer --version`
2. Check MCP configuration: `claude mcp list`
3. Test server manually: `swissarmyhammer serve --debug`
4. Restart Claude Code

#### Connection Errors

**Symptoms**: Error messages about MCP connection

**Solutions**:
1. Check if port is available: `swissarmyhammer serve --port 8080`
2. Verify permissions: Run with `--debug` to see detailed logs
3. Check firewall settings
4. Try different host: `--host 127.0.0.1`

#### Prompts Not Loading

**Symptoms**: Some prompts missing or outdated

**Solutions**:
1. Check prompt syntax: `swissarmyhammer test <prompt-name>`
2. Verify file permissions in prompt directories
3. Check for YAML syntax errors
4. Restart the MCP server: Restart Claude Code

#### Performance Issues

**Symptoms**: Slow prompt loading or execution

**Solutions**:
1. Reduce prompt directory size
2. Disable file watching: `--watch false`
3. Use specific prompt directories: `--prompts ./specific-dir`
4. Check system resources

### Debug Mode

Enable debug mode for detailed troubleshooting:

```bash
swissarmyhammer serve --debug
```

Debug mode provides:
- Detailed logging of MCP communication
- Prompt loading information
- Error stack traces
- Performance metrics

### Logs and Diagnostics

#### Server Logs

SwissArmyHammer logs to standard output:

```bash
# Save logs to file
swissarmyhammer serve --debug > swissarmyhammer.log 2>&1
```

#### Claude Code Logs

Check Claude Code logs for MCP-related issues:

```bash
# Location varies by platform
# macOS: ~/Library/Logs/Claude Code/
# Linux: ~/.local/share/claude-code/logs/
# Windows: %APPDATA%/Claude Code/logs/
```

#### Health Check

Use the doctor command to check configuration:

```bash
swissarmyhammer doctor
```

This checks:
- Installation status
- Configuration validity
- Prompt directory accessibility
- MCP server functionality

## Best Practices

### 1. Organize Prompts Logically

Structure prompts by workflow rather than just topic:

```
prompts/
├── workflows/
│   ├── code-review-workflow.md
│   ├── feature-development.md
│   └── bug-fixing.md
├── utilities/
│   ├── format-code.md
│   ├── generate-tests.md
│   └── extract-docs.md
```

### 2. Use Descriptive Metadata

Make prompts discoverable with good metadata:

```yaml
---
name: comprehensive-code-review
title: Comprehensive Code Review
description: Deep analysis of code for security, performance, and maintainability
category: development
tags: ["security", "performance", "maintainability", "best-practices"]
keywords: ["static analysis", "code quality", "peer review"]
---
```

### 3. Test Prompts Regularly

Validate prompts before using in Claude Code:

```bash
# Test basic functionality
swissarmyhammer test code-review --code "def hello(): print('hi')"

# Test all prompts
swissarmyhammer test --all
```

### 4. Use Project-Specific Configurations

Create project-specific prompt collections:

```bash
# Per-project MCP server
cd my-project
claude mcp add project_prompts swissarmyhammer serve --prompts ./prompts
```

### 5. Keep Prompts Updated

Maintain prompt quality:

```yaml
---
name: my-prompt
version: 1.2.0
updated: 2024-03-20  # Track changes
---
```

## Examples

### Complete Setup Example

Here's a complete example of setting up SwissArmyHammer for a development project:

```bash
# 1. Install SwissArmyHammer
curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | bash

# 2. Create project prompts
mkdir -p ./.swissarmyhammer/prompts/development
cat > ./.swissarmyhammer/prompts/development/project-review.md << 'EOF'
---
name: project-code-review
title: Project Code Review
description: Reviews code according to our project standards
category: development
arguments:
  - name: code
    description: Code to review
    required: true
  - name: component
    description: Which component this code belongs to
    required: false
    default: general
---

# Project Code Review

Please review this {{component}} code according to our project standards:

```{{code}}```

Check for:
- Adherence to our coding standards
- Security best practices
- Performance considerations
- Documentation completeness
- Test coverage

Provide specific, actionable feedback.
EOF

# 3. Configure Claude Code
claude mcp add project_sah swissarmyhammer serve --prompts ./prompts

# 4. Test the setup
swissarmyhammer test project-code-review --code "print('hello')" --component "utility"

# 5. Start using in Claude Code
echo "Setup complete! Restart Claude Code to use your prompts."
```

## Next Steps

- Explore [Built-in Prompts](./builtin-prompts.md) to see what's available
- Learn [Creating Prompts](./creating-prompts.md) to build custom prompts
- Read about [Prompt Organization](./prompt-organization.md) strategies
- Check the [CLI Reference](./cli-reference.md) for all available commands
- See [Troubleshooting](./troubleshooting.md) for additional help