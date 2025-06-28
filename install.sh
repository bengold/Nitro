#!/bin/bash

set -e

# Nitro Package Manager Installation Script
# This script installs Nitro alongside Homebrew for enhanced performance

NITRO_VERSION="${NITRO_VERSION:-latest}"
INSTALL_DIR="${NITRO_INSTALL_DIR:-$HOME/.nitro}"
BIN_DIR="${NITRO_BIN_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}==>${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Check system requirements
check_requirements() {
    info "Checking system requirements..."
    
    # Check OS
    if [[ "$OSTYPE" != "darwin"* ]] && [[ "$OSTYPE" != "linux"* ]]; then
        error "Nitro currently only supports macOS and Linux"
    fi
    
    # Check for curl or wget
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        error "Either curl or wget is required for installation"
    fi
    
    # Check for git (optional but recommended)
    if ! command -v git &> /dev/null; then
        warn "Git is not installed. Some features may not work properly."
    fi
    
    success "System requirements met"
}

# Detect Homebrew installation
detect_homebrew() {
    info "Detecting Homebrew installation..."
    
    if command -v brew &> /dev/null; then
        HOMEBREW_PREFIX=$(brew --prefix)
        success "Homebrew found at: $HOMEBREW_PREFIX"
        export HOMEBREW_PREFIX
    else
        warn "Homebrew not found. Nitro will work but with limited functionality."
        warn "Install Homebrew from https://brew.sh for full features."
    fi
}

# Download Nitro binary
download_nitro() {
    info "Downloading Nitro..."
    
    # Detect architecture
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac
    
    # Detect OS
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    
    # For now, since we don't have pre-built binaries, we'll build from source
    # In the future, this would download pre-built binaries
    
    if command -v cargo &> /dev/null; then
        info "Rust toolchain detected. Building from source..."
        build_from_source
    else
        error "Pre-built binaries not yet available. Please install Rust to build from source."
        echo "Visit https://rustup.rs to install Rust"
    fi
}

# Build from source
build_from_source() {
    info "Building Nitro from source..."
    
    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    # Clone repository
    if command -v git &> /dev/null; then
        git clone https://github.com/nitro-pm/nitro.git
        cd nitro
    else
        error "Git is required to build from source"
    fi
    
    # Build release binary
    cargo build --release
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR/bin"
    
    # Copy binary
    cp target/release/nitro "$INSTALL_DIR/bin/"
    chmod +x "$INSTALL_DIR/bin/nitro"
    
    # Clean up
    cd /
    rm -rf "$TMP_DIR"
    
    success "Nitro built and installed successfully"
}

# Setup shell integration
setup_shell() {
    info "Setting up shell integration..."
    
    # Create bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"
    
    # Create symlink
    ln -sf "$INSTALL_DIR/bin/nitro" "$BIN_DIR/nitro"
    
    # Detect shell and add to PATH
    SHELL_NAME=$(basename "$SHELL")
    
    case "$SHELL_NAME" in
        bash)
            RC_FILE="$HOME/.bashrc"
            ;;
        zsh)
            RC_FILE="$HOME/.zshrc"
            ;;
        fish)
            RC_FILE="$HOME/.config/fish/config.fish"
            ;;
        *)
            RC_FILE="$HOME/.profile"
            ;;
    esac
    
    # Add PATH export if not already present
    if ! grep -q "export PATH=\"\$HOME/.local/bin:\$PATH\"" "$RC_FILE" 2>/dev/null; then
        echo "" >> "$RC_FILE"
        echo "# Nitro Package Manager" >> "$RC_FILE"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$RC_FILE"
        
        # Add Homebrew compatibility
        if [ -n "$HOMEBREW_PREFIX" ]; then
            echo "export HOMEBREW_PREFIX=\"$HOMEBREW_PREFIX\"" >> "$RC_FILE"
        fi
    fi
    
    success "Shell integration configured"
}

# Post-installation setup
post_install() {
    info "Running post-installation setup..."
    
    # Import existing Homebrew taps if available
    if [ -n "$HOMEBREW_PREFIX" ] && [ -d "$HOMEBREW_PREFIX/Homebrew/Library/Taps" ]; then
        info "Importing Homebrew taps..."
        "$INSTALL_DIR/bin/nitro" tap list &> /dev/null || true
    fi
    
    success "Post-installation complete"
}

# Main installation
main() {
    echo "🚀 Nitro Package Manager Installer"
    echo "=================================="
    echo
    
    check_requirements
    detect_homebrew
    download_nitro
    setup_shell
    post_install
    
    echo
    success "Nitro installation complete!"
    echo
    echo "To get started:"
    echo "  1. Reload your shell configuration:"
    echo "     source $RC_FILE"
    echo "  2. Verify installation:"
    echo "     nitro --version"
    echo "  3. See available commands:"
    echo "     nitro --help"
    echo
    
    if [ -z "$HOMEBREW_PREFIX" ]; then
        echo "For full functionality, install Homebrew:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    fi
}

# Run main installation
main "$@"