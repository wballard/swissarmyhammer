# Quick Start

Get up and running with SwissArmyHammer in just a few minutes.

## Prerequisites

Before you begin, make sure you have:
- SwissArmyHammer installed (see [Installation](./installation.md))
- Claude Code (or another MCP-compatible client)

## Step 1: Verify Installation

First, check that SwissArmyHammer is properly installed:

```bash
swissarmyhammer --version
```

Run the doctor command to check your setup:

```bash
swissarmyhammer doctor
```

This will check your system and provide recommendations if anything needs attention.

## Step 2: Configure Claude Code

Add SwissArmyHammer to your Claude Code MCP configuration:

### Find Your Config File

The Claude Code configuration file is located at:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

### Add the Configuration

Create or edit the configuration file with the following content:

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

If you already have other MCP servers configured, just add the `swissarmyhammer` entry to your existing `mcpServers` object.

## Step 3: Create Your Prompt Directory

Create a directory for your prompts:

```bash
mkdir -p ~/.swissarmyhammer/prompts
```

This is where you'll store your custom prompts. SwissArmyHammer will automatically watch this directory for changes.

## Step 4: Create Your First Prompt

Create a simple prompt file:

```bash
cat > ~/.swissarmyhammer/prompts/helper.md << 'EOF'
---
title: General Helper
description: A helpful assistant for various tasks
arguments:
  - name: task
    description: What you need help with
    required: true
  - name: style
    description: How to approach the task
    required: false
    default: "friendly and concise"
---

# Task Helper

Please help me with: {{task}}

Approach this in a {{style}} manner. Provide clear, actionable advice.
EOF
```

## Step 5: Test the Setup

1. **Restart Claude Code** to pick up the new MCP server configuration.

2. **Open Claude Code** and start a new conversation.

3. **Try using your prompt**: In Claude Code, you should now see SwissArmyHammer prompts available in the prompt picker.

4. **Use the built-in prompts**: SwissArmyHammer comes with several built-in prompts you can try right away:
   - `help` - Get help with using SwissArmyHammer
   - `debug-error` - Debug error messages
   - `code-review` - Review code for issues
   - `docs-readme` - Generate README files

## Step 6: Verify Everything Works

Test that SwissArmyHammer is working correctly:

```bash
# Check if Claude Code can connect (this will show server info)
swissarmyhammer serve --help

# Run diagnostics again to see the updated status
swissarmyhammer doctor
```

The doctor command should now show that Claude Code configuration is found and prompts are loading correctly.

## What's Next?

Now that you have SwissArmyHammer set up, you can:

1. **Explore built-in prompts** - See what's available out of the box
2. **Create more prompts** - Build your own prompt library
3. **Learn advanced features** - Template variables, prompt organization, etc.

### Recommended Next Steps

- [Create Your First Custom Prompt](./first-prompt.md)
- [Learn about Template Variables](./template-variables.md)
- [Explore Built-in Prompts](./builtin-prompts.md)
- [Advanced Prompt Techniques](./advanced-prompts.md)

## Troubleshooting

If something isn't working:

1. **Run the doctor**: `swissarmyhammer doctor`
2. **Check Claude Code logs**: Look for any error messages
3. **Verify file permissions**: Make sure SwissArmyHammer can read your prompt files
4. **Restart Claude Code**: Sometimes a restart is needed after configuration changes

For more detailed troubleshooting, see the [Troubleshooting](./troubleshooting.md) guide.

## Getting Help

If you need help:
- Check the [Troubleshooting](./troubleshooting.md) guide
- Look at [Examples](./examples.md) for inspiration
- Ask questions in [GitHub Discussions](https://github.com/swissarmyhammer/swissarmyhammer/discussions)
- Report bugs in [GitHub Issues](https://github.com/swissarmyhammer/swissarmyhammer/issues)