#!/bin/bash
set -e

# swissarmyhammer installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/dist/install.sh | bash

REPO="wballard/swissarmyhammer"
BINARY_NAME="swissarmyhammer"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform and architecture
detect_platform() {
    local os
    local arch
    
    case "$(uname -s)" in
        Darwin*)
            os="apple-darwin"
            ;;
        Linux*)
            os="unknown-linux-gnu"
            # Check if we should use musl instead
            if command -v ldd >/dev/null 2>&1 && ldd --version 2>&1 | grep -q musl; then
                os="unknown-linux-musl"
            fi
            ;;
        CYGWIN*|MINGW*|MSYS*)
            os="pc-windows-msvc"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${arch}-${os}"
}

# Get latest release version from GitHub
get_latest_version() {
    local version
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    if [ -z "$version" ]; then
        log_error "Failed to get latest version"
        exit 1
    fi
    
    echo "$version"
}

# Download and install binary
install_binary() {
    local platform="$1"
    local version="$2"
    local binary_name="${BINARY_NAME}"
    
    # Add .exe extension for Windows
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="${BINARY_NAME}.exe"
    fi
    
    local asset_name="${BINARY_NAME}-${platform}"
    if [[ "$platform" == *"windows"* ]]; then
        asset_name="${asset_name}.exe"
    fi
    
    local download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"
    local temp_file="/tmp/${asset_name}"
    
    log_info "Downloading ${asset_name} from ${download_url}"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "${temp_file}" "${download_url}"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "${temp_file}" "${download_url}"
    else
        log_error "Neither curl nor wget is available"
        exit 1
    fi
    
    # Create install directory if it doesn't exist
    mkdir -p "${INSTALL_DIR}"
    
    # Make binary executable and move to install directory
    chmod +x "${temp_file}"
    mv "${temp_file}" "${INSTALL_DIR}/${binary_name}"
    
    log_success "Installed ${binary_name} to ${INSTALL_DIR}/${binary_name}"
}

# Add install directory to PATH if needed
update_path() {
    local shell_rc
    
    # Determine shell config file
    case "$SHELL" in
        */bash)
            if [ -f "$HOME/.bashrc" ]; then
                shell_rc="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_rc="$HOME/.bash_profile"
            else
                shell_rc="$HOME/.bashrc"
            fi
            ;;
        */zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        */fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            log_warning "Unknown shell: $SHELL. You may need to manually add ${INSTALL_DIR} to your PATH."
            return
            ;;
    esac
    
    # Check if directory is already in PATH
    if echo "$PATH" | grep -q "${INSTALL_DIR}"; then
        log_info "Install directory is already in PATH"
        return
    fi
    
    # Add to PATH in shell config
    if [ -f "$shell_rc" ] && grep -q "${INSTALL_DIR}" "$shell_rc"; then
        log_info "Install directory already configured in ${shell_rc}"
    else
        echo "" >> "$shell_rc"
        echo "# Added by swissarmyhammer installer" >> "$shell_rc"
        if [[ "$SHELL" == */fish ]]; then
            echo "set -gx PATH ${INSTALL_DIR} \$PATH" >> "$shell_rc"
        else
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$shell_rc"
        fi
        log_success "Added ${INSTALL_DIR} to PATH in ${shell_rc}"
        log_warning "Please restart your shell or run: source ${shell_rc}"
    fi
}

# Verify installation
verify_installation() {
    if command -v "${BINARY_NAME}" >/dev/null 2>&1; then
        local installed_version
        installed_version=$("${BINARY_NAME}" --version 2>/dev/null | head -n1 || echo "unknown")
        log_success "Installation verified: ${installed_version}"
        
        log_info "Running doctor command to check setup..."
        if "${BINARY_NAME}" doctor >/dev/null 2>&1; then
            log_success "Doctor check passed"
        else
            log_warning "Doctor check had warnings. Run '${BINARY_NAME} doctor' for details."
        fi
    elif [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        local installed_version
        installed_version=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null | head -n1 || echo "unknown")
        log_success "Installation verified: ${installed_version}"
        log_info "Binary installed at ${INSTALL_DIR}/${BINARY_NAME}"
        log_warning "Please add ${INSTALL_DIR} to your PATH to use '${BINARY_NAME}' command"
    else
        log_error "Installation verification failed"
        exit 1
    fi
}

# Show next steps
show_next_steps() {
    echo
    log_info "🔨 swissarmyhammer installation complete!"
    echo
    echo "Next steps:"
    echo "1. Run 'swissarmyhammer doctor' to check your setup"
    echo "2. Add swissarmyhammer to your Claude Code MCP configuration:"
    echo '   {'
    echo '     "mcpServers": {'
    echo '       "swissarmyhammer": {'
    echo '         "command": "swissarmyhammer",'
    echo '         "args": ["serve"]'
    echo '       }'
    echo '     }'
    echo '   }'
    echo "3. Create prompts in ~/.swissarmyhammer/prompts/"
    echo
    echo "For more information, visit: https://github.com/${REPO}"
}

# Main installation flow
main() {
    log_info "🔨 Installing swissarmyhammer..."
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: ${platform}"
    
    # Get latest version
    local version
    version=$(get_latest_version)
    log_info "Latest version: ${version}"
    
    # Install binary
    install_binary "$platform" "$version"
    
    # Update PATH
    update_path
    
    # Verify installation
    verify_installation
    
    # Show next steps
    show_next_steps
}

# Check for required tools
if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
    log_error "Neither curl nor wget is available. Please install one of them."
    exit 1
fi

# Run main installation
main "$@"