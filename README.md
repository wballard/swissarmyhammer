<div align="center">

# 🔨 SwissArmyHammer

**The MCP server for managing prompts as markdown files**

[![CI](https://github.com/wballard/swissarmyhammer/workflows/CI/badge.svg)](https://github.com/wballard/swissarmyhammer/actions)
[![Release](https://img.shields.io/github/v/release/wballard/swissarmyhammer)](https://github.com/wballard/swissarmyhammer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-green.svg)](https://github.com/anthropics/model-context-protocol)

[📖 Documentation](https://wballard.github.io/swissarmyhammer) • [🚀 Quick Start](#quick-start) • [💡 Examples](#examples) • [🤝 Contributing](#contributing)

</div>

---

## ✨ What is SwissArmyHammer?

SwissArmyHammer transforms how you work with AI prompts by letting you manage them as simple markdown files. It's a powerful Model Context Protocol (MCP) server that seamlessly integrates with Claude Code and other MCP-compatible tools.

```markdown
---
title: Code Review Helper
description: Reviews code for best practices and issues
arguments:
  - name: code
    description: The code to review
    required: true
---

# Code Review

Please review this code for:
- Best practices
- Potential bugs
- Performance issues

```{{code}}```
```

## 🎯 Key Features

- **📁 File-based Management** - Store prompts as markdown files with YAML front matter
- **🔄 Live Reloading** - Changes are automatically detected and reloaded
- **🎨 Template Variables** - Use `{{variable}}` syntax for dynamic prompts
- **⚡ MCP Integration** - Works seamlessly with Claude Code
- **🗂️ Organized Hierarchy** - Built-in, user, and local prompt directories
- **🛠️ Developer Tools** - Rich CLI with diagnostics and completions
- **🔍 Built-in Library** - 20+ ready-to-use prompts for common tasks

## 🚀 Quick Start

### Install

```bash
# Install from Git repository (requires Rust)
cargo install --git https://github.com/wballard/swissarmyhammer.git

```

### Configure Claude Code

Add to your Claude Code MCP configuration:

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

### Create Your First Prompt

```bash
mkdir -p ~/.swissarmyhammer/prompts
cat > ~/.swissarmyhammer/prompts/helper.md << 'EOF'
---
title: Task Helper
description: Helps with various tasks
arguments:
  - name: task
    description: What you need help with
    required: true
---

Please help me with: {{task}}

Provide clear, actionable advice.
EOF
```

That's it! Your prompt is now available in Claude Code.

## 💡 Examples

### Debug Assistant
```markdown
---
title: Debug Helper
description: Helps debug code and error messages
arguments:
  - name: error
    description: The error message or code issue
    required: true
  - name: language
    description: Programming language
    default: "auto-detect"
---

Help me debug this {{language}} issue:

{{error}}

Please provide:
1. Likely causes
2. Step-by-step debugging approach
3. Potential solutions
```

### Documentation Generator
```markdown
---
title: API Documentation
description: Generates API documentation from code
arguments:
  - name: code
    description: The API code to document
    required: true
  - name: format
    description: Documentation format
    default: "markdown"
---

Generate {{format}} documentation for this API:

```
{{code}}
```

Include endpoints, parameters, responses, and examples.
```

## 🛠️ CLI Commands

```bash
# Run as MCP server
swissarmyhammer serve

# Check configuration and setup
swissarmyhammer doctor

# Generate shell completions
swissarmyhammer completion bash > ~/.bash_completion.d/swissarmyhammer

# Show help
swissarmyhammer --help
```

## 📖 Documentation

- **[Installation Guide](https://wballard.github.io/swissarmyhammer/installation.html)** - All installation methods
- **[Quick Start](https://wballard.github.io/swissarmyhammer/quick-start.html)** - Get up and running
- **[Creating Prompts](https://wballard.github.io/swissarmyhammer/creating-prompts.html)** - Prompt creation guide
- **[Claude Code Integration](https://wballard.github.io/swissarmyhammer/claude-code-integration.html)** - Setup with Claude Code
- **[Built-in Prompts](https://wballard.github.io/swissarmyhammer/builtin-prompts.html)** - Ready-to-use prompts

## 🏗️ Architecture

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

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer

# Install dependencies
cargo build

# Run tests
cargo test

# Run the development server
cargo run -- serve
```

## 📊 Project Status

SwissArmyHammer is actively developed and maintained. Current focus areas:

- ✅ Core MCP server functionality
- ✅ File-based prompt management  
- ✅ Template variable system
- ✅ Built-in prompt library
- ✅ CLI tools and diagnostics
- ✅ Comprehensive documentation
- 🚧 Package manager distributions
- 🚧 Advanced template features
- 🚧 Plugin system

## 🌟 Why SwissArmyHammer?

- **Simple**: Plain markdown files, no complex databases
- **Powerful**: Rich template system with live reloading
- **Organized**: Hierarchical prompt management
- **Integrated**: First-class MCP protocol support
- **Developer-friendly**: Great CLI tools and diagnostics
- **Open**: MIT licensed, community-driven

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and the [rmcp](https://github.com/rockerBOO/rmcp) MCP framework
- Inspired by the [Model Context Protocol](https://github.com/anthropics/model-context-protocol)
- Documentation powered by [mdBook](https://rust-lang.github.io/mdBook/)

---

<div align="center">

**[⭐ Star this repo](https://github.com/wballard/swissarmyhammer/stargazers)** if you find SwissArmyHammer useful!

</div>