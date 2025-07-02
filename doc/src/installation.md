# Installation

SwissArmyHammer is available through multiple installation methods. Choose the one that works best for your system.

## Quick Install (Recommended)

### Cargo Install from Git

```bash
cargo install --git https://github.com/wballard/swissarmyhammer.git
```

This requires Rust to be installed on your system. If you don't have Rust, install it from [rustup.rs](https://rustup.rs/).

### One-liner install script

```bash
curl -fsSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/dist/install.sh | bash
```

This script will:
- Detect your platform automatically
- Download the appropriate binary
- Install it to `~/.local/bin` (or `$INSTALL_DIR` if set)
- Add the binary to your PATH
- Verify the installation

## Platform-Specific Installation

### macOS

#### Option 1: Homebrew (Recommended)

```bash
# Add the tap (once the formula is published)
brew tap wballard/swissarmyhammer
brew install swissarmyhammer
```

#### Option 2: Direct Download

```bash
# Intel Macs
curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-x86_64-apple-darwin -o swissarmyhammer
chmod +x swissarmyhammer
sudo mv swissarmyhammer /usr/local/bin/

# Apple Silicon Macs
curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-aarch64-apple-darwin -o swissarmyhammer
chmod +x swissarmyhammer
sudo mv swissarmyhammer /usr/local/bin/
```

### Linux

#### Direct Download

```bash
# x86_64 (most common)
curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-x86_64-unknown-linux-gnu -o swissarmyhammer
chmod +x swissarmyhammer
sudo mv swissarmyhammer /usr/local/bin/

# x86_64 (static binary, works on any Linux distribution)
curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-x86_64-unknown-linux-musl -o swissarmyhammer
chmod +x swissarmyhammer
sudo mv swissarmyhammer /usr/local/bin/

# ARM64 (for ARM-based servers/systems)
curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-aarch64-unknown-linux-gnu -o swissarmyhammer
chmod +x swissarmyhammer
sudo mv swissarmyhammer /usr/local/bin/
```

### Windows

#### Direct Download

```powershell
# Download the latest release
Invoke-WebRequest https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-x86_64-pc-windows-msvc.exe -OutFile swissarmyhammer.exe

# Move to a directory in your PATH (optional)
Move-Item swissarmyhammer.exe $env:USERPROFILE\bin\
```

## Install from Source

### Prerequisites

- Rust 1.70 or later
- Git

### Build and Install

```bash
# Clone the repository
git clone https://github.com/wballard/swissarmyhammer.git
cd swissarmyhammer

# Build the release binary
cargo build --release

# Install to ~/.cargo/bin (make sure it's in your PATH)
cargo install --path .

# Or copy the binary manually
cp target/release/swissarmyhammer /usr/local/bin/
```

### Using Cargo

```bash
# Install from the git repository (recommended)
cargo install --git https://github.com/wballard/swissarmyhammer.git

# Install directly from crates.io (once published)
cargo install swissarmyhammer
```

## Verification

After installation, verify that SwissArmyHammer is working correctly:

```bash
# Check version
swissarmyhammer --version

# Run diagnostics
swissarmyhammer doctor

# Test basic functionality
swissarmyhammer --help
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

## Next Steps

Once installed, continue to the [Quick Start](./quick-start.md) guide to set up SwissArmyHammer with Claude Code and create your first prompt.

## Troubleshooting

If you encounter any issues during installation, check the [Troubleshooting](./troubleshooting.md) guide or run:

```bash
swissarmyhammer doctor
```

This will diagnose common setup problems and provide solutions.