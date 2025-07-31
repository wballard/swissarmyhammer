<div align="center">

<img src="icon.png" alt="SwissArmyHammer" width="256" height="256">

# SwissArmyHammer

**Program all the things, just by writing markdown. Really.**

📚 **[Complete Documentation & Guides](https://wballard.github.io/swissarmyhammer)** 📚

🦀 **[Rust API Documentation](https://docs.rs/swissarmyhammer)** 🦀

[![CI](https://github.com/swissarmyhammer/swissarmyhammer/workflows/CI/badge.svg)](https://github.com/swissarmyhammer/swissarmyhammer/actions)
[![License](https://img.shields.io/badge/License-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-green.svg)](https://github.com/anthropics/model-context-protocol)

[📖 Documentation](https://wballard.github.io/swissarmyhammer) • [🦀 API Docs](https://docs.rs/swissarmyhammer)

</div>

---

## ✨ What is SwissArmyHammer?

SwissArmyHammer transforms how you work with AI prompts and workflows by letting you manage them as simple markdown files.

- a command line app that uses Claude Code as a sub agent
- a powerful Model Context Protocol (MCP) server that seamlessly integrates with Claude Code
- a flexible Rust library for building prompt-based applications.

## TLDR

Follow the [Calcutron](https://github.com/swissarmyhammer/calcutron) sample to get started.

## 🎯 Key Features

- **📁 File-based Management** - Store prompts and sub agent workflows as markdown files with YAML front matter
- **🔄 Live Reloading** - Changes are automatically detected and reloaded
- **🎨 Liquid Templates** - Use Liquid templating with variables, conditionals, loops, and custom filters to make templates and workflows
- **⚡ MCP Integration** - Works seamlessly with Claude Code via Model Context Protocol
- **🗂️ Organized Hierarchy** - Built-in, user, and local prompt directories with override precedence
- **🛠️ Developer Tools** - Rich CLI with diagnostics, validation, and shell completions
- **📚 Rust Library** - Use as a dependency in your own Rust projects with comprehensive API
- **🔍 Built-in Library** - 20+ ready-to-use prompts for common development tasks
- **🔧 Workflow Engine** - Advanced state-based workflow execution with Mermaid diagrams
- **🔍 Advanced Search** - Vector search with fuzzy matching and relevance scoring
- **Git-based workflow** with automatic branch management

### Common Commands

```bash
sah --help
```

### Standard Locations

1. **Builtin** - Embedded in the SwissArmyHammer binary
   - Pre-installed prompts and workflows for common tasks
   - Always available, no setup required

2. **User** - Your personal collection
   - Prompts: `~/.swissarmyhammer/prompts/`
   - Workflows: `~/.swissarmyhammer/workflows/`
   - Shared across all your projects

3. **Local** - Project-specific files
   - Prompts: `./.swissarmyhammer/prompts/`
   - Workflows: `./.swissarmyhammer/workflows/`
   - Searched in current directory and parent directories
   - Perfect for project-specific customizations

### Example Structure

```
~/.swissarmyhammer/          # User directory
├── prompts/
│   ├── code-review.md       # Personal code review prompt
│   └── daily-standup.md     # Your daily standup template
└── workflows/
    └── release-process.md   # Your release workflow

./my-project/                # Project directory
└── .swissarmyhammer/        # Local directory
    ├── prompts/
    │   └── api-docs.md      # Project-specific API documentation prompt
    └── workflows/
        └── ci-cd.md         # Project CI/CD workflow
```

## 🚀 Quick Start

### Install

See [https://wballard.github.io/swissarmyhammer/installation.html](https://wballard.github.io/swissarmyhammer/installation.html) for detailed installation instructions.

### Configure Claude Code

Add to your Claude Code [MCP configuration](https://docs.anthropic.com/en/docs/claude-code/mcp)

```bash
claude mcp add --scope user sah sah serve
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

That's it! Your prompt is now available in Claude Code. You can use it via MCP with `/helper`.

## 📖 Documentation

- **[Installation Guide](https://wballard.github.io/swissarmyhammer/installation.html)** - All installation methods
- **[Quick Start](https://wballard.github.io/swissarmyhammer/quick-start.html)** - Get up and running
- **[Creating Prompts](https://wballard.github.io/swissarmyhammer/creating-prompts.html)** - Prompt creation guide
- **[Claude Code Integration](https://wballard.github.io/swissarmyhammer/claude-code-integration.html)** - Setup with Claude Code
- **[Built-in Prompts](https://wballard.github.io/swissarmyhammer/builtin-prompts.html)** - Ready-to-use prompts

### Development Setup

See [https://wballard.github.io/swissarmyhammer/installation.html](https://wballard.github.io/swissarmyhammer/installation.html) for development setup instructions.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and the [rmcp](https://github.com/rockerBOO/rmcp) MCP framework
- Inspired by the [Model Context Protocol](https://github.com/anthropics/model-context-protocol)
- Documentation powered by [mdBook](https://rust-lang.github.io/mdBook/)

---

<div align="center">

**[⭐ Star this repo](https://github.com/swissarmyhammer/swissarmyhammer/stargazers)** if you find SwissArmyHammer useful!

</div>
