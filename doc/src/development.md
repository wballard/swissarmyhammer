# Development Setup

This guide covers setting up a development environment for working on SwissArmyHammer.

## Prerequisites

### Required Tools

- **Rust** 1.70 or later
- **Git** 2.0 or later
- **Cargo** (comes with Rust)
- **A code editor** (VS Code recommended)

### Optional Tools

- **Docker** - For testing container builds
- **mdBook** - For documentation development
- **Node.js** - For web-based tooling
- **Python** - For utility scripts

## Environment Setup

### Installing Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the installation prompts, then:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Setting Up the Repository

```bash
# Clone the repository
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer

# Install development dependencies
cargo install cargo-watch cargo-edit cargo-outdated

# Install formatting and linting tools
rustup component add rustfmt clippy

# Install documentation tools
cargo install mdbook mdbook-linkcheck mdbook-mermaid
```

### VS Code Setup

Install recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates",
    "tamasfe.even-better-toml",
    "streetsidesoftware.code-spell-checker",
    "yzhang.markdown-all-in-one"
  ]
}
```

Settings for `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Project Structure

```
swissarmyhammer/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library entry point
│   ├── cli/              # CLI commands
│   ├── mcp/              # MCP server implementation
│   ├── prompts/          # Prompt management
│   ├── template/         # Template engine
│   └── utils/            # Utilities
├── tests/
│   ├── integration/      # Integration tests
│   └── fixtures/         # Test data
├── doc/
│   └── src/              # Documentation source
├── benches/              # Benchmarks
└── Cargo.toml           # Project manifest
```

## Building the Project

### Development Build

```bash
# Quick build (debug mode)
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- serve

# Watch for changes and rebuild
cargo watch -x build
```

### Release Build

```bash
# Optimized build
cargo build --release

# Run release binary
./target/release/swissarmyhammer --version

# Build with all features
cargo build --release --all-features
```

### Cross-Compilation

```bash
# Install cross-compilation tools
cargo install cross

# Build for different targets
cross build --target x86_64-pc-windows-gnu
cross build --target aarch64-apple-darwin
cross build --target x86_64-unknown-linux-musl
```

## Development Workflow

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_prompt_loading

# With output
cargo test -- --show-output

# With specific features
cargo test --features "experimental"
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Strict linting
cargo clippy -- -D warnings

# Check for security issues
cargo audit

# Update dependencies
cargo update
cargo outdated
```

### Documentation

```bash
# Build API documentation
cargo doc --no-deps --open

# Build user documentation
cd doc
mdbook build
mdbook serve

# Check documentation examples
cargo test --doc
```

## Debugging

### VS Code Debug Configuration

`.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": ["build", "--bin=swissarmyhammer"],
        "filter": {
          "name": "swissarmyhammer",
          "kind": "bin"
        }
      },
      "args": ["serve", "--debug"],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "RUST_BACKTRACE": "1"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tests",
      "cargo": {
        "args": ["test", "--no-run"],
        "filter": {
          "name": "swissarmyhammer",
          "kind": "lib"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Command Line Debugging

```bash
# Enable debug logging
export RUST_LOG=swissarmyhammer=debug

# Enable backtrace
export RUST_BACKTRACE=1

# Run with debugging
cargo run -- serve --debug

# Use GDB
rust-gdb target/debug/swissarmyhammer

# Use LLDB
rust-lldb target/debug/swissarmyhammer
```

### Logging

Add debug logging to your code:

```rust
use log::{debug, info, warn, error};

fn process_prompt(name: &str) -> Result<()> {
    debug!("Processing prompt: {}", name);
    
    if let Some(prompt) = self.get_prompt(name) {
        info!("Found prompt: {}", prompt.title);
        Ok(())
    } else {
        error!("Prompt not found: {}", name);
        Err(anyhow!("Prompt not found"))
    }
}
```

## Performance Profiling

### Benchmarking

Create benchmarks in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use swissarmyhammer::PromptManager;

fn bench_prompt_loading(c: &mut Criterion) {
    c.bench_function("load 100 prompts", |b| {
        b.iter(|| {
            let manager = PromptManager::new();
            manager.load_prompts()
        });
    });
}

criterion_group!(benches, bench_prompt_loading);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench

# Compare benchmarks
cargo bench -- --save-baseline before
# Make changes
cargo bench -- --baseline before
```

### CPU Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin swissarmyhammer -- serve

# Using perf (Linux)
perf record --call-graph=dwarf cargo run --release -- serve
perf report
```

### Memory Profiling

```bash
# Install valgrind (Linux/macOS)
# macOS: brew install valgrind
# Linux: apt-get install valgrind

# Run with valgrind
valgrind --leak-check=full \
         --show-leak-kinds=all \
         target/debug/swissarmyhammer serve

# Use heaptrack (Linux)
heaptrack cargo run -- serve
heaptrack_gui heaptrack.*.gz
```

## Testing Strategies

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_parsing() {
        let content = r#"---
name: test
title: Test Prompt
---
Content"#;
        
        let prompt = Prompt::parse(content).unwrap();
        assert_eq!(prompt.name, "test");
        assert_eq!(prompt.title, "Test Prompt");
    }
}
```

### Integration Testing

In `tests/integration/`:

```rust
use swissarmyhammer::PromptManager;
use tempfile::tempdir;

#[test]
fn test_full_workflow() {
    let temp_dir = tempdir().unwrap();
    
    // Create test prompts
    std::fs::write(
        temp_dir.path().join("test.md"),
        "---\nname: test\n---\nContent"
    ).unwrap();
    
    // Test loading
    let mut manager = PromptManager::new();
    manager.add_directory(temp_dir.path());
    manager.load_prompts().unwrap();
    
    // Test retrieval
    assert!(manager.get_prompt("test").is_some());
}
```

### Property Testing

Using `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_prompt_name_validation(name in "[a-z][a-z0-9-]*") {
        assert!(is_valid_prompt_name(&name));
    }
}
```

## Common Development Tasks

### Adding a New Command

1. Create command module in `src/cli/`:
   ```rust
   // src/cli/new_command.rs
   use clap::Args;
   
   #[derive(Args)]
   pub struct NewCommand {
       #[arg(short, long)]
       option: String,
   }
   
   impl NewCommand {
       pub fn run(&self) -> Result<()> {
           // Implementation
           Ok(())
       }
   }
   ```

2. Add to CLI enum:
   ```rust
   // src/cli/mod.rs
   #[derive(Subcommand)]
   pub enum Commands {
       NewCommand(NewCommand),
       // ...
   }
   ```

### Adding a Feature

1. Define feature in `Cargo.toml`:
   ```toml
   [features]
   experimental = ["dep:experimental-lib"]
   ```

2. Conditionally compile code:
   ```rust
   #[cfg(feature = "experimental")]
   pub mod experimental {
       // Experimental features
   }
   ```

### Updating Dependencies

```bash
# Check outdated dependencies
cargo outdated

# Update specific dependency
cargo update -p serde

# Update all dependencies
cargo update

# Edit dependency version
cargo upgrade serde --version 1.0.150
```

## Troubleshooting

### Common Issues

#### Compilation Errors

```bash
# Clean build artifacts
cargo clean

# Check for missing dependencies
cargo check

# Verify toolchain
rustup show
```

#### Test Failures

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run tests serially
cargo test -- --test-threads=1

# Skip slow tests
cargo test --lib
```

#### Performance Issues

```bash
# Build with debug symbols in release
cargo build --release --features debug

# Check binary size
cargo bloat --release

# Analyze dependencies
cargo tree --duplicates
```

## CI/CD Integration

### GitHub Actions

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
    
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Cache
      uses: Swatinem/rust-cache@v2
    
    - name: Check
      run: cargo check --all-features
    
    - name: Test
      run: cargo test --all-features
    
    - name: Clippy
      run: cargo clippy -- -D warnings
    
    - name: Format
      run: cargo fmt -- --check
```

### Pre-commit Hooks

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: fmt
        name: Format
        entry: cargo fmt -- --check
        language: system
        types: [rust]
        pass_filenames: false
      
      - id: clippy
        name: Clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
      
      - id: test
        name: Test
        entry: cargo test
        language: system
        types: [rust]
        pass_filenames: false
```

## Next Steps

- Read [Contributing](./contributing.md) for contribution guidelines
- Check [Testing](./testing.md) for detailed testing practices
- See [Release Process](./release-process.md) for release procedures
- Review [Architecture](./architecture.md) for system design