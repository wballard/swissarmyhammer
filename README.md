<div align="center">

<img src="icon.svg" alt="SwissArmyHammer" width="256" height="256">

# SwissArmyHammer

**The MCP server and Rust library for managing prompts as markdown files**

📚 **[Complete Documentation & Guides](https://wballard.github.io/swissarmyhammer)** 📚

🦀 **[Rust API Documentation](https://docs.rs/swissarmyhammer)** 🦀

[![CI](https://github.com/wballard/swissarmyhammer/workflows/CI/badge.svg)](https://github.com/wballard/swissarmyhammer/actions)
[![Release](https://img.shields.io/github/v/release/wballard/swissarmyhammer)](https://github.com/wballard/swissarmyhammer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-green.svg)](https://github.com/anthropics/model-context-protocol)

[📖 Documentation](https://wballard.github.io/swissarmyhammer) • [🦀 API Docs](https://docs.rs/swissarmyhammer) • [🚀 Quick Start](#quick-start) • [💡 Examples](#examples)

</div>

---

## ✨ What is SwissArmyHammer?

SwissArmyHammer transforms how you work with AI prompts by letting you manage them as simple markdown files. It's both a powerful Model Context Protocol (MCP) server that seamlessly integrates with Claude Code and a flexible Rust library for building prompt-based applications.

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
- **🎨 Liquid Templates** - Use Liquid templating with variables, conditionals, loops, and custom filters
- **⚡ MCP Integration** - Works seamlessly with Claude Code via Model Context Protocol
- **🗂️ Organized Hierarchy** - Built-in, user, and local prompt directories with override precedence
- **🛠️ Developer Tools** - Rich CLI with diagnostics, validation, and shell completions
- **📚 Rust Library** - Use as a dependency in your own Rust projects with comprehensive API
- **🔍 Built-in Library** - 20+ ready-to-use prompts for common development tasks
- **🎯 Custom Filters** - Domain-specific Liquid filters for code, text, and data processing
- **🔧 Workflow Engine** - Advanced state-based workflow execution with Mermaid diagrams
- **🔍 Advanced Search** - Full-text search with fuzzy matching and relevance scoring

## 📝 Issue Management

SwissArmyHammer includes a comprehensive issue management system that integrates with git workflows and provides both MCP tools and CLI commands for managing project issues.

### Features

#### 🎯 Core Features
- **Git-based Workflow**: Each issue can have its own work branch
- **MCP Integration**: Full integration with Model Context Protocol for AI assistants
- **CLI Tools**: Complete command-line interface for issue management
- **Markdown Storage**: Issues stored as markdown files in your repository
- **Automatic Numbering**: Sequential 6-digit issue numbering (000001, 000002, etc.)
- **Completion Tracking**: Issues move from active to completed state

#### 📋 Issue Lifecycle
1. **Create** - Create new issues with `issue_create` or `swissarmyhammer issue create`
2. **Work** - Switch to issue work branch with `issue_work`
3. **Update** - Update issue content with `issue_update`
4. **Complete** - Mark issues complete with `issue_mark_complete`
5. **Merge** - Merge completed work with `issue_merge`

#### 🔧 MCP Tools
- `issue_create` - Create new issues
- `issue_mark_complete` - Mark issues as complete
- `issue_all_complete` - Check if all issues are completed
- `issue_update` - Update issue content
- `issue_current` - Get current issue from git branch
- `issue_work` - Start working on an issue
- `issue_merge` - Merge completed issue work

#### 💻 CLI Commands
- `swissarmyhammer issue create` - Create new issues
- `swissarmyhammer issue list` - List all issues
- `swissarmyhammer issue show` - Show issue details
- `swissarmyhammer issue update` - Update issue content
- `swissarmyhammer issue complete` - Mark issues complete
- `swissarmyhammer issue work` - Start working on an issue
- `swissarmyhammer issue merge` - Merge completed work
- `swissarmyhammer issue current` - Show current issue
- `swissarmyhammer issue status` - Show project status

### Quick Start

#### 1. Create Your First Issue
```bash
# Using CLI
swissarmyhammer issue create "implement_auth" --content "Add JWT authentication"

# Using MCP (via Claude Code)
> Create an issue to implement JWT authentication
```

#### 2. Start Working
```bash
# Switch to issue work branch
swissarmyhammer issue work 1

# Check current issue
swissarmyhammer issue current
```

#### 3. Update Progress
```bash
# Update with progress notes
swissarmyhammer issue update 1 --content "Authentication implemented" --append

# Mark complete when done
swissarmyhammer issue complete 1
```

#### 4. Merge to Main
```bash
# Merge completed work
swissarmyhammer issue merge 1
```

### Directory Structure

```
./issues/
├── 000001_implement_auth.md      # Active issue
├── 000002_fix_bug.md            # Active issue
└── complete/
    └── 000003_add_tests.md      # Completed issue
```

### Best Practices

1. **Use Descriptive Names**: Issue names become part of branch names
2. **Regular Updates**: Keep issues updated with progress notes
3. **Complete Before Merge**: Always mark issues complete before merging
4. **Clean Branches**: Use default branch deletion after merge
5. **Atomic Commits**: Make focused commits in issue branches

### Troubleshooting

#### Common Issues

**"Not in a git repository"**
- Solution: Initialize git repository (`git init`)

**"Uncommitted changes"**
- Solution: Commit changes (`git add . && git commit`) or stash (`git stash`)

**"Issue not found"**
- Solution: Check issue number with `swissarmyhammer issue list`

**"Branch already exists"**
- Solution: Switch to existing branch or delete old branch

#### Getting Help

Use `swissarmyhammer issue --help` for detailed command help.

## 📂 Directory Structure

SwissArmyHammer uses a hierarchical system for organizing prompts and workflows. Files are loaded from three standard locations, with later sources overriding earlier ones:

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

### Precedence

When files have the same name in multiple locations:
- Local overrides User
- User overrides Builtin

This allows you to customize built-in prompts for your needs while keeping the originals intact.

## 🚀 Quick Start

### Install

See [https://wballard.github.io/swissarmyhammer/installation.html](https://wballard.github.io/swissarmyhammer/installation.html) for detailed installation instructions.

### Configure Claude Code

Add to your Claude Code [MCP configuration](https://docs.anthropic.com/en/docs/claude-code/mcp)

```bash
claude mcp add --scope user swissarmyhammer swissarmyhammer serve
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

### Use as a Rust Library

See [https://wballard.github.io/swissarmyhammer/installation.html](https://wballard.github.io/swissarmyhammer/installation.html) for detailed installation instructions.

Basic usage:

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec};
use std::collections::HashMap;

// Create a prompt library
let mut library = PromptLibrary::new();

// Add prompts from a directory
library.add_directory("./.swissarmyhammer/prompts")?;

// Get and render a prompt
let prompt = library.get("code-review")?;

let mut args = HashMap::new();
args.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());

let rendered = prompt.render(&args)?;
println!("{}", rendered);
```

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
  - name: include_examples
    description: Include code examples
    default: "true"
---

Help me debug this {{ language | capitalize }} issue:

{{ error }}

{% if language == "python" %}
Focus on common Python issues like:
- Indentation errors
- Import problems
- Type mismatches
{% elsif language == "javascript" %}
Focus on common JavaScript issues like:
- Undefined variables
- Async/await problems
- Scoping issues
{% endif %}

Please provide:
1. Likely causes
2. Step-by-step debugging approach
3. Potential solutions
{% if include_examples == "true" %}
4. Code examples showing the fix
{% endif %}
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

`{{code}}`

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

### Development Setup

See [https://wballard.github.io/swissarmyhammer/installation.html](https://wballard.github.io/swissarmyhammer/installation.html) for development setup instructions.

## 📊 Project Status

SwissArmyHammer is actively developed and maintained. Current focus areas:

- ✅ Core MCP server functionality
- ✅ File-based prompt management  
- ✅ Template variable system
- ✅ Built-in prompt library
- ✅ CLI tools and diagnostics
- ✅ Comprehensive documentation
- ✅ Search commands
- ✅ Rust library with full API
- 🚧 Pre-built binary releases
- 🚧 Package manager distributions
- 🚧 Advanced template features

## 🌟 Why SwissArmyHammer?

- **Simple**: Plain markdown files, no complex databases
- **Powerful**: Rich template system with live reloading
- **Organized**: Hierarchical prompt management
- **Integrated**: First-class MCP protocol support
- **Developer-friendly**: Great CLI tools and diagnostics
- **Open**: MIT licensed, community-driven

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and the [rmcp](https://github.com/rockerBOO/rmcp) MCP framework
- Inspired by the [Model Context Protocol](https://github.com/anthropics/model-context-protocol)
- Documentation powered by [mdBook](https://rust-lang.github.io/mdBook/)

---

<div align="center">

**[⭐ Star this repo](https://github.com/wballard/swissarmyhammer/stargazers)** if you find SwissArmyHammer useful!

</div>
