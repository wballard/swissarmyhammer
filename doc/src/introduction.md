# SwissArmyHammer

**The MCP server for managing prompts as markdown files**

SwissArmyHammer is a powerful Model Context Protocol (MCP) server that lets you manage AI prompts as simple markdown files. It seamlessly integrates with Claude Code and other MCP-compatible tools, providing a flexible and organized way to work with AI prompts.

## What is SwissArmyHammer?

SwissArmyHammer transforms how you work with AI prompts by:

- **📁 File-based prompt management** - Store prompts as markdown files with YAML front matter
- **🔄 Live reloading** - Changes to prompt files are automatically detected and reloaded
- **🎯 Template variables** - Use `{{variable}}` syntax for dynamic prompt customization
- **⚡ MCP integration** - Works seamlessly with Claude Code and other MCP clients
- **🗂️ Organized hierarchy** - Support for built-in, user, and local prompt directories
- **🛠️ Developer-friendly** - Rich CLI with diagnostics and shell completions

## Key Features

### 🚀 Quick Setup
Get started in seconds with our one-liner installer:
```bash
curl -fsSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/dist/install.sh | bash
```

### 📝 Simple Prompt Format
Create prompts using familiar markdown with YAML front matter:
```markdown
---
title: Code Review Helper
description: Helps review code for best practices and potential issues
arguments:
  - name: code
    description: The code to review
    required: true
  - name: language
    description: Programming language
    required: false
    default: "auto-detect"
---

# Code Review

Please review the following {{language}} code:

```
{{code}}
```

Focus on:
- Code quality and readability
- Potential bugs or security issues
- Performance considerations
- Best practices adherence
```

### 🎯 Template Variables
Use template variables to make prompts dynamic and reusable:
- `{{variable}}` - Required variables
- `{{variable:default}}` - Optional variables with defaults
- Support for strings, numbers, booleans, and JSON objects

### 🔧 Built-in Diagnostics
The `doctor` command helps troubleshoot setup issues:
```bash
swissarmyhammer doctor
```

## Use Cases

SwissArmyHammer is perfect for:

- **Development Teams** - Share and standardize AI prompts across your team
- **Individual Developers** - Organize your personal prompt library
- **Content Creators** - Manage writing and editing prompts
- **Researchers** - Organize domain-specific prompts and templates
- **Students** - Build a learning-focused prompt collection

## Architecture

SwissArmyHammer follows a simple but powerful architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Claude Code   │◄──►│ SwissArmyHammer  │◄──►│ Prompt Files    │
│   (MCP Client)  │    │   (MCP Server)   │    │ (.md files)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  File Watcher    │
                       │ (Auto-reload)    │
                       └──────────────────┘
```

## Getting Started

Ready to get started? Check out our [Installation Guide](./installation.md) or jump straight to creating [Your First Prompt](./first-prompt.md).

For integration with Claude Code, see our [Claude Code Integration](./claude-code-integration.md) guide.

## Community

- **GitHub**: [github.com/wballard/swissarmyhammer](https://github.com/wballard/swissarmyhammer)
- **Issues**: Report bugs and request features
- **Discussions**: Community Q&A and sharing
- **Contributing**: See our [Contributing Guide](./contributing.md)

## License

SwissArmyHammer is open source software licensed under the MIT License. See the [License](./license.md) page for details.