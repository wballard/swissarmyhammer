<div align="center">

# 🔨 SwissArmyHammer

**The MCP server and Rust library for managing prompts as markdown files**

[![CI](https://github.com/wballard/swissarmyhammer/workflows/CI/badge.svg)](https://github.com/wballard/swissarmyhammer/actions)
[![Release](https://img.shields.io/github/v/release/wballard/swissarmyhammer)](https://github.com/wballard/swissarmyhammer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-green.svg)](https://github.com/anthropics/model-context-protocol)

[📖 Documentation](https://wballard.github.io/swissarmyhammer) • [🚀 Quick Start](#quick-start) • [💡 Examples](#examples) • [🤝 Contributing](#contributing)

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
- **⚡ MCP Integration** - Works seamlessly with Claude Code
- **🗂️ Organized Hierarchy** - Built-in, user, and local prompt directories
- **🛠️ Developer Tools** - Rich CLI with diagnostics and completions
- **📚 Rust Library** - Use as a dependency in your own Rust projects
- **🔍 Built-in Library** - 20+ ready-to-use prompts for common tasks
- **🎯 Custom Filters** - Domain-specific Liquid filters for code, text, and data processing

## 🚀 Quick Start

### Install

See [INSTALLATION.md](INSTALLATION.md) for detailed installation instructions.

### Configure Claude Code

Add to your Claude Code [MCP configuration](https://docs.anthropic.com/en/docs/claude-code/mcp)

```bash
claude mcp add sah_server -e  -- swissarmyhammer serve
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

See [INSTALLATION.md](INSTALLATION.md) for detailed installation instructions.

Basic usage:

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec};
use std::collections::HashMap;

// Create a prompt library
let mut library = PromptLibrary::new();

// Add prompts from a directory
library.add_directory("./prompts")?;

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

See [INSTALLATION.md](INSTALLATION.md) for development setup instructions.

## 📊 Project Status

SwissArmyHammer is actively developed and maintained. Current focus areas:

- ✅ Core MCP server functionality
- ✅ File-based prompt management  
- ✅ Template variable system
- ✅ Built-in prompt library
- ✅ CLI tools and diagnostics
- ✅ Comprehensive documentation
- ✅ Search and export/import commands
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