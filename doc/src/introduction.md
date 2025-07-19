# SwissArmyHammer

**The MCP server for managing prompts as markdown files**

SwissArmyHammer solves the problem of AI prompt management by providing a comprehensive solution that treats prompts as first-class citizens in your development workflow. Unlike scattered prompt files or hard-coded templates, SwissArmyHammer offers a structured, versioned, and collaborative approach to prompt engineering.

## The Problem

Developers and AI practitioners face several challenges when working with AI prompts:

- **Scattered Management**: Prompts are often stored in various locations, making them hard to find and maintain
- **No Version Control**: Changes to prompts are difficult to track and rollback
- **Limited Reusability**: Prompts are often duplicated or slightly modified across projects
- **Poor Collaboration**: Team members can't easily share and improve prompts together
- **No Standardization**: Different projects use different prompt formats and structures

## The Solution

SwissArmyHammer addresses these challenges by providing:

**Unified Prompt Management**: Store all prompts as markdown files with YAML front matter in a hierarchical structure that supports built-in, user, and project-specific prompts.

**Version Control Integration**: Since prompts are plain markdown files, they work seamlessly with Git and other version control systems.

**Template Engine**: Powerful Liquid templating with custom filters allows for dynamic, reusable prompts that can be customized for different contexts.

**MCP Protocol Support**: Native integration with Claude Code and other MCP-compatible tools means prompts are available where you need them.

**Developer-Friendly Tools**: Rich CLI with validation, search, and diagnostic capabilities ensures prompt quality and discoverability.

## How SwissArmyHammer Works

SwissArmyHammer transforms your prompt workflow through:

- **📁 File-based prompt management** - Store prompts as markdown files with YAML front matter
- **🔄 Live reloading** - Changes to prompt files are automatically detected and reloaded
- **🎯 Template variables** - Use `{{variable}}` syntax for dynamic prompt customization
- **⚡ MCP integration** - Works seamlessly with Claude Code and other MCP clients
- **🗂️ Organized hierarchy** - Support for built-in, user, and local prompt directories
- **🛠️ Developer-friendly** - Rich CLI with diagnostics and shell completions

## Quick Start

### Installation
```bash
cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli
```

### Basic Usage
1. **Create a prompt directory**: `mkdir ~/.swissarmyhammer/prompts`
2. **Configure Claude Code**: Add SwissArmyHammer to your MCP configuration
3. **Create your first prompt**: Use the simple markdown + YAML format
4. **Start using prompts**: Available immediately in Claude Code

### Key Benefits

- **🔧 Zero Configuration**: Works out of the box with sensible defaults
- **📱 Cross-Platform**: Runs on macOS, Linux, and Windows
- **🔄 Real-Time Updates**: File changes are automatically detected and reloaded
- **🎯 Type Safe**: Rust implementation provides reliability and performance
- **🌐 Community Driven**: Open source with active development and contributions

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

## Who Should Use SwissArmyHammer?

### Development Teams
- **Standardize prompts** across projects and team members
- **Version control** prompt changes with Git integration
- **Code review** prompt modifications like any other code
- **Share libraries** of tested, proven prompts

### Individual Developers
- **Organize personal prompts** in a structured hierarchy
- **Reuse prompts** across different projects and contexts
- **Build expertise** through curated prompt collections
- **Integrate seamlessly** with existing development workflows

### Content Creators & Researchers
- **Manage specialized prompts** for specific domains
- **Create template libraries** for common content types
- **Collaborate effectively** on prompt development
- **Maintain quality** through validation and testing

### Students & Educators
- **Learn prompt engineering** through structured examples
- **Build knowledge bases** of educational prompts
- **Share resources** with classmates and colleagues
- **Track progress** through versioned prompt evolution

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

## Next Steps

1. **[Install SwissArmyHammer](./installation.md)** - Get up and running quickly
2. **[Create Your First Prompt](./first-prompt.md)** - Learn the basics
3. **[Integrate with Claude Code](./claude-code-integration.md)** - Connect to your AI assistant
4. **[Explore Advanced Features](./advanced-prompts.md)** - Unlock the full potential

## Why Choose SwissArmyHammer?

**Proven Architecture**: Built on well-tested technologies like Rust, Liquid templating, and the MCP protocol.

**Active Development**: Regular updates, bug fixes, and new features based on community feedback.

**Comprehensive Documentation**: Detailed guides, examples, and API reference to get you productive quickly.

**Open Source**: MIT licensed with a welcoming community for contributions and feedback.

## Join the Community

- **[GitHub Repository](https://github.com/wballard/swissarmyhammer)** - Source code, issues, and discussions
- **[Contributing Guide](./contributing.md)** - How to contribute to the project
- **[Issue Tracker](https://github.com/wballard/swissarmyhammer/issues)** - Report bugs and request features
- **[Discussions](https://github.com/wballard/swissarmyhammer/discussions)** - Community Q&A and sharing

## License

SwissArmyHammer is open source software licensed under the MIT License. See the [License](./license.md) page for details.