# Installation Instructions

## CLI Installation

### From Git Repository (requires Rust)

```bash
cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli

# Ensure ~/.cargo/bin is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

## Library Installation

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { git = "https://github.com/wballard/swissarmyhammer", features = ["full"] }
```

## Development Setup

```bash
# Clone the repository
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer

# Build the workspace (library + CLI)
cargo build

# Run tests
cargo test

# Run the CLI in development mode
cargo run --bin swissarmyhammer -- serve

# Build optimized release version
cargo build --release
```