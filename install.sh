#!/bin/bash
# SwissArmyHammer Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/swissarmyhammer/swissarmyhammer/main/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="swissarmyhammer/swissarmyhammer"
BINARY_NAME="swissarmyhammer"
INSTALL_DIR="/usr/local/bin"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case "$os" in
        Linux*)
            case "$arch" in
                x86_64) PLATFORM="linux-x86_64" ;;
                *) print_error "Unsupported architecture: $arch. Only x86_64 is supported on Linux." ;;
            esac
            ARCHIVE_EXT="tar.gz"
            ;;
        Darwin*)
            case "$arch" in
                x86_64) PLATFORM="macos-x86_64" ;;
                arm64) PLATFORM="macos-arm64" ;;
                *) print_error "Unsupported architecture: $arch. Only x86_64 and arm64 are supported on macOS." ;;
            esac
            ARCHIVE_EXT="tar.gz"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            print_error "Windows installation via this script is not supported. Please download the Windows binary from GitHub releases."
            ;;
        *)
            print_error "Unsupported operating system: $os"
            ;;
    esac
    
    print_status "Detected platform: $PLATFORM"
}

# Get the latest release version
get_latest_version() {
    print_status "Fetching latest release information..."
    
    if command -v curl >/dev/null 2>&1; then
        LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        LATEST_VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        print_error "Neither curl nor wget is available. Please install one of them and try again."
    fi
    
    if [ -z "$LATEST_VERSION" ]; then
        print_error "Failed to fetch the latest version information."
    fi
    
    print_status "Latest version: $LATEST_VERSION"
}

# Download and install the binary
install_binary() {
    local archive_name="${BINARY_NAME}-${PLATFORM}.${ARCHIVE_EXT}"
    local download_url="https://github.com/$REPO/releases/download/$LATEST_VERSION/$archive_name"
    local temp_dir=$(mktemp -d)
    
    print_status "Downloading $archive_name..."
    
    # Download the archive
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$download_url" -o "$temp_dir/$archive_name"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$download_url" -O "$temp_dir/$archive_name"
    else
        print_error "Neither curl nor wget is available."
    fi
    
    # Extract the archive
    print_status "Extracting archive..."
    if [ "$ARCHIVE_EXT" = "tar.gz" ]; then
        tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"
    else
        print_error "Unsupported archive format: $ARCHIVE_EXT"
    fi
    
    # Install the binary
    print_status "Installing $BINARY_NAME to $INSTALL_DIR..."
    
    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        cp "$temp_dir/$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        print_status "Administrator privileges required for installation to $INSTALL_DIR"
        sudo cp "$temp_dir/$BINARY_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    # Clean up
    rm -rf "$temp_dir"
    
    print_success "$BINARY_NAME installed successfully!"
}

# Verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local version=$($BINARY_NAME --version 2>/dev/null || echo "unknown")
        print_success "$BINARY_NAME is installed and available in PATH"
        print_status "Version: $version"
        
        # Run doctor command if available
        print_status "Running diagnostic check..."
        if $BINARY_NAME doctor >/dev/null 2>&1; then
            print_success "Installation verified successfully!"
        else
            print_warning "Installation completed but doctor command reported issues. Run '$BINARY_NAME doctor' for details."
        fi
    else
        print_error "$BINARY_NAME is not available in PATH. Installation may have failed."
    fi
}

# Print usage instructions
print_usage() {
    echo
    print_success "Installation complete! Here's how to get started:"
    echo
    echo "  1. List available prompts:"
    echo "     $BINARY_NAME list"
    echo
    echo "  2. Search for prompts:"
    echo "     $BINARY_NAME search <query>"
    echo
    echo "  3. Use a prompt:"
    echo "     $BINARY_NAME prompt <name>"
    echo
    echo "  4. Get help:"
    echo "     $BINARY_NAME help"
    echo
    echo "  5. Run diagnostics:"
    echo "     $BINARY_NAME doctor"
    echo
    print_status "For more information, visit: https://github.com/$REPO"
}

# Main installation flow
main() {
    echo "SwissArmyHammer Installation Script"
    echo "==================================="
    echo
    
    # Check dependencies
    if ! command -v tar >/dev/null 2>&1; then
        print_error "tar is required but not installed."
    fi
    
    detect_platform
    get_latest_version
    install_binary
    verify_installation
    print_usage
}

# Run the installation
main "$@"