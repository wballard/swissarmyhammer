# Quick Start Guide

Get up and running with SwissArmyHammer in minutes!

## Installation

### As a CLI Tool (MCP Server)

```bash
# Install from source (requires Rust)
cargo install --git https://github.com/wballard/swissarmyhammer.git

# Or clone and build
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer
cargo install --path swissarmyhammer-cli
```

### As a Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = "0.1"
```

## Your First Prompt (CLI)

1. Create a prompt file:

```bash
mkdir -p ~/.swissarmyhammer/prompts
cat > ~/.swissarmyhammer/prompts/hello.md << 'EOF'
---
title: Hello World
description: A simple greeting prompt
arguments:
  - name: name
    description: Name to greet
    required: true
---

Hello {{ name }}! Welcome to SwissArmyHammer.
EOF
```

2. Configure Claude Code:

```bash
claude mcp add sah_server -e -- swissarmyhammer serve
```

3. Use in Claude Code - your prompt "Hello World" is now available!

## Your First Prompt (Library)

```rust
use swissarmyhammer::{PromptLibrary, Prompt};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a library
    let mut library = PromptLibrary::new();
    
    // Add a prompt programmatically
    let prompt = Prompt::new("greeting", "Hello {{ name }}!")
        .with_description("A simple greeting");
    library.add(prompt)?;
    
    // Render the prompt
    let greeting = library.get("greeting")?;
    let mut args = HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    
    println!("{}", greeting.render(&args)?);
    // Output: Hello World!
    
    Ok(())
}
```

## Common Use Cases

### Code Review Assistant

```markdown
---
title: Code Review
description: Comprehensive code review
arguments:
  - name: code
    description: Code to review
    required: true
  - name: language
    description: Programming language
    default: "auto"
---

Please review this {{ language }} code:

```{{ language }}
{{ code }}
```

Check for:
- Best practices
- Potential bugs
- Performance issues
- Security concerns
```

### Documentation Generator

```markdown
---
title: Generate Docs
description: Create documentation from code
arguments:
  - name: code
    description: Code to document
    required: true
  - name: style
    description: Documentation style
    default: "markdown"
---

Generate {{ style }} documentation for:

{{ code | dedent }}

Include:
- Function descriptions
- Parameters and returns
- Usage examples
- Edge cases
```

### Test Writer

```markdown
---
title: Write Tests
description: Generate test cases
arguments:
  - name: code
    description: Code to test
    required: true
  - name: framework
    description: Test framework
    default: "pytest"
---

Write {{ framework }} tests for:

{{ code }}

Include:
- Happy path tests
- Edge cases
- Error handling
- Performance tests if applicable
```

## Using Custom Filters

SwissArmyHammer includes powerful custom filters:

```markdown
---
title: Code Analysis
description: Analyze code structure
arguments:
  - name: code
    description: Code to analyze
    required: true
---

## Code Analysis

**Metrics:**
- Lines: {{ code | count_lines }}
- Estimated tokens: {{ code | count_tokens }}

**Functions found:**
{{ code | extract_functions | bullet_list }}

**Formatted code:**
{{ code | format_lang: "python" }}
```

## Directory Structure

SwissArmyHammer looks for prompts in these locations (in order):

1. **Built-in prompts**: Included with SwissArmyHammer
2. **User prompts**: `~/.swissarmyhammer/prompts/`
3. **Local prompts**: `./.swissarmyhammer/prompts/`

Local prompts override user prompts, which override built-in prompts.

## Next Steps

- Explore [Built-in Prompts](builtin-prompts.md)
- Learn about [Creating Prompts](creating-prompts.md)
- Discover [Custom Filters](custom-filters.md)
- Set up [Claude Code Integration](claude-code.md)
- Build with the [Rust Library](library-usage.md)

## Tips

1. **Use descriptive names**: `code-review-security` instead of `cr2`
2. **Provide good descriptions**: Help users understand what each prompt does
3. **Set sensible defaults**: Make prompts easier to use
4. **Use categories**: Organize prompts with the `category` field
5. **Test your prompts**: Use the CLI test command

## Troubleshooting

### Prompt not found

```bash
# Check available prompts
swissarmyhammer list

# Verify prompt file location
ls ~/.swissarmyhammer/prompts/
```

### Template errors

```bash
# Validate prompt syntax
swissarmyhammer validate ~/.swissarmyhammer/prompts/my-prompt.md

# Test rendering
swissarmyhammer test my-prompt
```

### Claude Code not detecting prompts

```bash
# Run diagnostics
swissarmyhammer doctor

# Restart Claude Code after configuration changes
```

## Getting Help

- Run `swissarmyhammer --help` for CLI options
- Check the [documentation](https://github.com/wballard/swissarmyhammer/docs)
- File issues on [GitHub](https://github.com/wballard/swissarmyhammer/issues)