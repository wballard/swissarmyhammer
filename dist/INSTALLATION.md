# Installation Guide

SwissArmyHammer is available through multiple installation methods. Choose the one that works best for your system.

## Quick Install (Recommended)

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

#### Option 1: Direct Download

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

#### Option 2: Package Managers (Future)

We plan to add support for:
- `apt` packages for Debian/Ubuntu
- `yum`/`dnf` packages for RHEL/Fedora
- Snap packages
- Flatpak

### Windows

#### Option 1: Direct Download

```powershell
# Download the latest release
Invoke-WebRequest https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-x86_64-pc-windows-msvc.exe -OutFile swissarmyhammer.exe

# Move to a directory in your PATH (optional)
Move-Item swissarmyhammer.exe $env:USERPROFILE\bin\
```

#### Option 2: Package Managers (Future)

We plan to add support for:
- Chocolatey
- Scoop
- winget

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
# Install directly from crates.io (once published)
cargo install swissarmyhammer

# Or install from the git repository
cargo install --git https://github.com/wballard/swissarmyhammer.git
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

## Configuration

### Claude Code Integration

Add SwissArmyHammer to your Claude Code MCP configuration:

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

### Shell Completions

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

## Updating

### Manual Update

Re-run the installation method you used initially. For the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/dist/install.sh | bash
```

### Homebrew

```bash
brew update && brew upgrade swissarmyhammer
```

### Cargo

```bash
cargo install swissarmyhammer --force
```

## Troubleshooting

### Common Issues

1. **Command not found**
   - Make sure the binary is in your PATH
   - Try the full path: `~/.local/bin/swissarmyhammer`

2. **Permission denied**
   - Make sure the binary is executable: `chmod +x swissarmyhammer`
   - Check file permissions and ownership

3. **Binary won't run on older systems**
   - Try the musl variant for Linux: `swissarmyhammer-x86_64-unknown-linux-musl`
   - Check your system's minimum requirements

4. **Installation script fails**
   - Make sure you have `curl` or `wget` installed
   - Check your internet connection
   - Try downloading manually

### Getting Help

If you encounter issues:

1. Run `swissarmyhammer doctor` for diagnostics
2. Check the [GitHub Issues](https://github.com/wballard/swissarmyhammer/issues)
3. Create a new issue with:
   - Your operating system and version
   - Installation method used
   - Error messages
   - Output of `swissarmyhammer doctor`

## Uninstalling

### Remove Binary

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/swissarmyhammer

# If installed to ~/.local/bin
rm ~/.local/bin/swissarmyhammer

# If installed via Homebrew
brew uninstall swissarmyhammer

# If installed via Cargo
cargo uninstall swissarmyhammer
```

### Remove Configuration

```bash
# Remove user configuration and prompts
rm -rf ~/.swissarmyhammer

# Remove shell completions
rm ~/.local/share/bash-completion/completions/swissarmyhammer  # Bash
rm ~/.zfunc/_swissarmyhammer  # Zsh
rm ~/.config/fish/completions/swissarmyhammer.fish  # Fish
```

### Remove from Claude Code

Remove the `swissarmyhammer` entry from your Claude Code MCP configuration.