# Installation Guide

SwissArmyHammer provides multiple installation methods to suit different user preferences and environments.

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

### Manual Installation

1. **Download** the appropriate binary for your platform from [GitHub Releases](https://github.com/swissarmyhammer/swissarmyhammer/releases)
2. **Extract** the archive
3. **Move** the binary to a directory in your PATH
4. **Make executable** (Unix systems): `chmod +x swissarmyhammer`

## Platform-Specific Installation

### macOS

#### Homebrew (Coming Soon)
```bash
brew install swissarmyhammer/tap/swissarmyhammer
```

#### Manual Installation
1. Download `swissarmyhammer-macos-x86_64.tar.gz` (Intel) or `swissarmyhammer-macos-arm64.tar.gz` (Apple Silicon)
2. Extract: `tar -xzf swissarmyhammer-macos-*.tar.gz`
3. Move to PATH: `sudo mv swissarmyhammer /usr/local/bin/`

### Linux

#### Manual Installation
1. Download `swissarmyhammer-linux-x86_64.tar.gz`
2. Extract: `tar -xzf swissarmyhammer-linux-x86_64.tar.gz`
3. Move to PATH: `sudo mv swissarmyhammer /usr/local/bin/`

#### Package Managers (Coming Soon)
- **Debian/Ubuntu**: `apt install swissarmyhammer`
- **Arch Linux**: `pacman -S swissarmyhammer`
- **Snap**: `snap install swissarmyhammer`

### Windows

1. Download `swissarmyhammer-windows-x86_64.zip`
2. Extract the ZIP file
3. Move `swissarmyhammer.exe` to a directory in your PATH
4. Or add the directory containing the binary to your PATH

#### Package Managers (Coming Soon)
- **Scoop**: `scoop install swissarmyhammer`
- **Winget**: `winget install swissarmyhammer`

## Development Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/swissarmyhammer/swissarmyhammer.git
cd swissarmyhammer

# Build and install
cargo install --path swissarmyhammer-cli
```

### From crates.io (Coming Soon)

```bash
cargo install swissarmyhammer-cli
```

### From Git Repository

```bash
cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli

# Ensure ~/.cargo/bin is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

## Library Usage

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { git = "https://github.com/wballard/swissarmyhammer", features = ["full"] }
```

Or once published to crates.io:

```toml
[dependencies]
swissarmyhammer = "0.1"
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

## Verification

After installation, verify everything is working:

```bash
# Check version
swissarmyhammer --version

# Run diagnostics
swissarmyhammer doctor

# List available prompts
swissarmyhammer list
```

The `doctor` command will check:
- ✅ Installation method and binary integrity
- ✅ Binary permissions and PATH availability
- ✅ Configuration files
- ✅ Prompt directories
- ✅ File system permissions

## Troubleshooting

### Command Not Found

If you get "command not found" error:

1. **Check PATH**: Ensure the installation directory is in your PATH
   ```bash
   echo $PATH
   ```

2. **Manual PATH addition**: Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.)
   ```bash
   export PATH="/usr/local/bin:$PATH"
   ```

3. **Reload shell**: `source ~/.bashrc` or restart your terminal

### Permission Denied

If you get permission errors:

```bash
# Make binary executable
chmod +x /path/to/swissarmyhammer

# Or reinstall with proper permissions
sudo mv swissarmyhammer /usr/local/bin/
sudo chmod +x /usr/local/bin/swissarmyhammer
```

### Binary Issues

If the binary won't run:

1. **Check architecture**: Ensure you downloaded the correct binary for your system
2. **Check dependencies**: Run `ldd swissarmyhammer` (Linux) to check missing libraries
3. **Antivirus**: Some antivirus software may quarantine downloaded binaries

## Uninstallation

### Homebrew
```bash
brew uninstall swissarmyhammer
```

### Manual Installation
```bash
# Remove binary
sudo rm /usr/local/bin/swissarmyhammer

# Remove configuration (optional)
rm -rf ~/.swissarmyhammer
```

### Cargo Installation
```bash
cargo uninstall swissarmyhammer-cli
```

## Build Requirements

If building from source:

- **Rust**: 1.70 or later
- **Git**: For cloning the repository
- **System dependencies**: Usually none required

## Next Steps

After installation:

1. **Run the doctor**: `swissarmyhammer doctor`
2. **Explore built-in prompts**: `swissarmyhammer list --source builtin`
3. **Create your first prompt**: `swissarmyhammer prompt create`
4. **Read the documentation**: [Full Documentation](https://github.com/swissarmyhammer/swissarmyhammer/blob/main/README.md)

## Support

- **Issues**: [GitHub Issues](https://github.com/swissarmyhammer/swissarmyhammer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/swissarmyhammer/swissarmyhammer/discussions)
- **Documentation**: [Repository Wiki](https://github.com/swissarmyhammer/swissarmyhammer/wiki)