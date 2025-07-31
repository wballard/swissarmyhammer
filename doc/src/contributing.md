# Contributing

Thank you for your interest in contributing to SwissArmyHammer! This guide will help you get started with contributing to the project.

## Overview

SwissArmyHammer welcomes contributions in many forms:
- **Code contributions** - Features, bug fixes, optimizations
- **Prompt contributions** - New built-in prompts
- **Documentation** - Improvements, examples, translations
- **Bug reports** - Issues and reproducible test cases
- **Feature requests** - Ideas and suggestions

## Getting Started

### Prerequisites

- Rust 1.70+ (check with `rustc --version`)
- Git
- GitHub account
- Basic familiarity with:
  - Rust programming
  - Model Context Protocol (MCP)
  - Liquid templating

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/swissarmyhammer.git
   cd swissarmyhammer
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/swissarmyhammer/swissarmyhammer.git
   ```

### Development Setup

1. Install Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install development tools:
   ```bash
   # Format checker
   rustup component add rustfmt
   
   # Linter
   rustup component add clippy
   
   # Documentation
   cargo install mdbook
   ```

3. Build the project:
   ```bash
   cargo build
   cargo test
   ```

## Development Workflow

### Branch Strategy

- `main` - Stable release branch
- `develop` - Development branch
- `feature/*` - Feature branches
- `fix/*` - Bug fix branches
- `docs/*` - Documentation branches

### Creating a Feature Branch

```bash
# Update your fork
git checkout main
git pull upstream main
git push origin main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for fixes
git checkout -b fix/issue-description
```

### Making Changes

1. **Write code** following our style guide
2. **Add tests** for new functionality
3. **Update documentation** as needed
4. **Run checks** before committing

### Running Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build documentation
cargo doc --no-deps --open

# Check everything
./scripts/check-all.sh
```

## Code Style Guide

### Rust Code

Follow Rust standard style with these additions:

```rust
// Good: Clear module organization
pub mod prompts;
pub mod template;
pub mod mcp;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// Good: Descriptive names
pub struct PromptManager {
    prompts: HashMap<String, Prompt>,
    directories: Vec<PathBuf>,
    watcher: Option<FileWatcher>,
}

// Good: Clear error handling
impl PromptManager {
    pub fn load_prompt(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read prompt file: {}", path.display()))?;
        
        let prompt = Prompt::parse(&content)
            .with_context(|| format!("Failed to parse prompt: {}", path.display()))?;
        
        self.prompts.insert(prompt.name.clone(), prompt);
        Ok(())
    }
}

// Good: Comprehensive tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_prompt() {
        let mut manager = PromptManager::new();
        let result = manager.load_prompt(Path::new("test.md"));
        assert!(result.is_ok());
    }
}
```

### Documentation

- Use `///` for public API documentation
- Include examples in doc comments
- Keep comments concise and helpful

```rust
/// Manages a collection of prompts and provides MCP server functionality.
/// 
/// # Examples
/// 
/// ```
/// use swissarmyhammer::PromptManager;
/// 
/// let mut manager = PromptManager::new();
/// manager.load_prompts()?;
/// ```
pub struct PromptManager {
    // Implementation details...
}
```

### Error Messages

Make errors helpful and actionable:

```rust
// Good
bail!("Prompt '{}' not found in directories: {:?}", name, self.directories);

// Good with context
.with_context(|| format!("Failed to parse YAML front matter in {}", path.display()))?;

// Bad
bail!("Error");
```

## Contributing Prompts

### Built-in Prompt Guidelines

1. **Location**: `builtin/prompts/`
2. **Categories**: Place in appropriate subdirectory
3. **Quality**: Must be generally useful
4. **Testing**: Include test cases

### Prompt Standards

```markdown
---
name: descriptive-name
title: Human Readable Title
description: |
  Clear description of what this prompt does.
  Include use cases and examples.
category: development
tags:
  - relevant
  - searchable
  - tags
author: your-email@example.com
version: 1.0.0
arguments:
  - name: required_arg
    description: What this argument is for
    required: true
  - name: optional_arg
    description: Optional parameter
    default: "default value"
---

# Prompt Title

Clear instructions using the arguments:
- {{required_arg}}
- {{optional_arg}}

## Section Headers

Organize the prompt logically...
```

### Testing Prompts

Add test file `builtin/prompts/tests/your-prompt.test.md`:

```yaml
name: test-your-prompt
cases:
  - name: basic usage
    arguments:
      required_arg: "test value"
    expected_contains:
      - "test value"
      - "expected output"
    expected_not_contains:
      - "error"
      
  - name: edge case
    arguments:
      required_arg: ""
      optional_arg: "custom"
    expected_error: "required_arg cannot be empty"
```

## Documentation

### Documentation Structure

```
doc/
├── src/
│   ├── SUMMARY.md      # Table of contents
│   ├── chapter-1.md    # Content files
│   └── images/         # Images and diagrams
└── book.toml          # mdbook configuration
```

### Writing Documentation

1. **Be clear and concise**
2. **Include examples**
3. **Use proper markdown**
4. **Test all code examples**

### Building Documentation

```bash
cd doc
mdbook build
mdbook serve  # Preview at http://localhost:3000
```

## Testing

### Test Organization

```
tests/
├── integration/     # Integration tests
├── fixtures/       # Test data
└── common/        # Shared test utilities
```

### Writing Tests

```rust
#[test]
fn test_prompt_loading() {
    let temp_dir = tempdir().unwrap();
    let prompt_file = temp_dir.path().join("test.md");
    
    std::fs::write(&prompt_file, r#"---
name: test-prompt
title: Test
---
Content"#).unwrap();
    
    let mut manager = PromptManager::new();
    manager.add_directory(temp_dir.path());
    manager.load_prompts().unwrap();
    
    assert!(manager.get_prompt("test-prompt").is_some());
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_prompt_loading

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration
```

## Submitting Changes

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Features
git commit -m "feat: add prompt validation API"
git commit -m "feat(mcp): implement notification support"

# Bug fixes
git commit -m "fix: correct template escaping issue"
git commit -m "fix(watcher): handle symlink changes"

# Documentation
git commit -m "docs: add prompt writing guide"
git commit -m "docs(api): document PromptManager methods"

# Performance
git commit -m "perf: optimize prompt loading"
git commit -m "perf(cache): implement LRU cache"

# Refactoring
git commit -m "refactor: simplify error handling"
git commit -m "refactor(template): extract common logic"
```

### Pull Request Process

1. **Update your branch**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Create pull request**:
   - Use clear, descriptive title
   - Reference any related issues
   - Describe what changes do
   - Include test results
   - Add screenshots if UI changes

### PR Template

```markdown
## Description
Brief description of changes

## Related Issue
Fixes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] Breaking changes documented
```

## Code Review Process

### What We Look For

1. **Correctness** - Does it work as intended?
2. **Tests** - Are changes adequately tested?
3. **Documentation** - Is it documented?
4. **Style** - Does it follow conventions?
5. **Performance** - Any performance impacts?
6. **Security** - Any security concerns?

### Review Timeline

- Initial response: 2-3 days
- Full review: Within a week
- Follow-ups: As needed

### Addressing Feedback

```bash
# Make requested changes
git add -A
git commit -m "address review feedback"

# Or amend if small change
git commit --amend

# Force push to your branch
git push -f origin feature/your-feature-name
```

## Release Process

### Version Numbering

We use [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

### Release Checklist

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`
3. Run full test suite
4. Build and test binaries
5. Update documentation
6. Create release PR
7. Tag release after merge
8. Publish to crates.io

## Community

### Getting Help

- **GitHub Issues** - Bug reports and features
- **Discussions** - Questions and ideas
- **Discord** - Real-time chat (if available)

### Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct):
- Be respectful and inclusive
- Welcome newcomers
- Focus on what's best for the community
- Show empathy towards others

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Documentation credits

## Quick Reference

### Common Commands

```bash
# Development
cargo build                 # Build project
cargo test                  # Run tests
cargo fmt                   # Format code
cargo clippy                # Lint code
cargo doc                   # Build docs

# Documentation
cd doc && mdbook serve      # Preview docs

# Prompts
cargo run -- list           # List prompts
cargo run -- doctor         # Validate prompts

# Release
cargo publish --dry-run     # Test publishing
```

### Useful Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [MCP Specification](https://modelcontextprotocol.io/)
- [Liquid Documentation](https://shopify.github.io/liquid/)
- [mdBook Documentation](https://rust-lang.github.io/mdBook/)

## Thank You!

Your contributions make SwissArmyHammer better for everyone. Whether it's fixing a typo, adding a feature, or improving documentation, every contribution is valued.

Happy contributing! 🚀