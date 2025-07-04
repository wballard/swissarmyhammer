# Installation

## Quick Install (Recommended)

### Unix-like Systems (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/swissarmyhammer/swissarmyhammer/main/install.sh | sh
```

This script will:

- Detect your platform automatically
- Download the latest release
- Install to `/usr/local/bin`
- Verify the installation

## Clone and Build

If you are installing from source:

- **Rust 1.70 or later** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

If you want to build from source or contribute to development:

```bash
# Clone the repository
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer

# Build the CLI (debug mode for development)
cargo build

# Build optimized release version
cargo build --release

# Install from the local source
cargo install --path swissarmyhammer-cli

# Or run directly without installing
cargo run --bin swissarmyhammer -- --help
```

## Future Installation Methods

Pre-built binaries and package manager support are planned for future releases:

- **macOS**: Homebrew formula
- **Linux**: DEB and RPM packages
- **Windows**: MSI installer and Chocolatey package
- **crates.io**: Published crate for `cargo install swissarmyhammer-cli`

Check the [releases page](https://github.com/wballard/swissarmyhammer/releases) for updates.

## Verification

After installation, verify that SwissArmyHammer is working correctly:

```bash
# Check version
swissarmyhammer --version

# Run diagnostics
swissarmyhammer doctor

# Show help
swissarmyhammer --help

# List available commands
swissarmyhammer list
```

The `doctor` command will check your installation and provide helpful diagnostics if anything needs attention.

## Shell Completions

Generate and install shell completions for better CLI experience:

```bash
# Bash
swissarmyhammer completion bash > ~/.local/share/bash-completion/completions/swissarmyhammer

# Zsh (add to fpath)
swissarmyhammer completion zsh > ~/.zfunc/_swissarmyhammer

# Fish
swissarmyhammer completion fish > ~/.config/fish/completions/swissarmyhammer.fish

# PowerShell
swissarmyhammer completion powershell >> $PROFILE
```

Remember to reload your shell or start a new terminal session for completions to take effect.

## Updating

To update SwissArmyHammer to the latest version:

```bash
# Update from git repository
cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli --force
```

The `--force` flag will overwrite the existing installation.

## Next Steps

Once installed, continue to the [Quick Start](./quick-start.md) guide to set up SwissArmyHammer with Claude Code and create your first prompt.

## Troubleshooting

### Common Issues

**Command not found**: Make sure `~/.cargo/bin` is in your PATH.

**Build failures**: Ensure you have Rust 1.70+ installed and try updating Rust:
```bash
rustup update
```

**Permission errors**: Don't use `sudo` with cargo install - it installs to your user directory.

For more help, check the [Troubleshooting](./troubleshooting.md) guide or run:

```bash
swissarmyhammer doctor
```