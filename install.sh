#!/bin/bash
set -e

# Nitro Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/yourusername/nitro/main/install.sh | bash

NITRO_VERSION="${NITRO_VERSION:-latest}"
INSTALL_DIR="${NITRO_INSTALL_DIR:-$HOME/.nitro}"
BIN_DIR="$INSTALL_DIR/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        darwin) OS="darwin" ;;
        linux) OS="linux" ;;
        *) echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
    esac
    
    case "$ARCH" in
        x86_64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
    esac
    
    PLATFORM="${OS}-${ARCH}"
}

# Download pre-built binary
download_binary() {
    echo -e "${BLUE}Downloading Nitro ${NITRO_VERSION} for ${PLATFORM}...${NC}"
    
    if [ "$NITRO_VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/yourusername/nitro/releases/latest/download/nitro-${PLATFORM}.tar.gz"
    else
        DOWNLOAD_URL="https://github.com/yourusername/nitro/releases/download/${NITRO_VERSION}/nitro-${PLATFORM}.tar.gz"
    fi
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    # Download and extract
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$TEMP_DIR"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$DOWNLOAD_URL" | tar -xz -C "$TEMP_DIR"
    else
        echo -e "${RED}Neither curl nor wget found. Please install one of them.${NC}"
        exit 1
    fi
    
    # Install binary
    mkdir -p "$BIN_DIR"
    mv "$TEMP_DIR/nitro" "$BIN_DIR/"
    chmod +x "$BIN_DIR/nitro"
}

# Build from source (fallback)
build_from_source() {
    echo -e "${YELLOW}Pre-built binary not available. Building from source...${NC}"
    
    # Check for Rust
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}Rust is required to build from source.${NC}"
        echo "Install Rust with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Clone and build
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    git clone https://github.com/yourusername/nitro.git "$TEMP_DIR"
    cd "$TEMP_DIR"
    
    echo -e "${BLUE}Building Nitro (this may take a few minutes)...${NC}"
    cargo build --release
    
    # Install binary
    mkdir -p "$BIN_DIR"
    mv target/release/nitro "$BIN_DIR/"
}

# Setup shell integration
setup_shell() {
    echo -e "${BLUE}Setting up shell integration...${NC}"
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    
    case "$SHELL_NAME" in
        bash)
            RC_FILE="$HOME/.bashrc"
            [ -f "$HOME/.bash_profile" ] && RC_FILE="$HOME/.bash_profile"
            ;;
        zsh)
            RC_FILE="$HOME/.zshrc"
            ;;
        fish)
            RC_FILE="$HOME/.config/fish/config.fish"
            ;;
        *)
            echo -e "${YELLOW}Unknown shell: $SHELL_NAME${NC}"
            echo "Please add $BIN_DIR to your PATH manually"
            return
            ;;
    esac
    
    # Add to PATH if not already there
    if ! grep -q "nitro/bin" "$RC_FILE" 2>/dev/null; then
        echo "" >> "$RC_FILE"
        echo "# Nitro package manager" >> "$RC_FILE"
        if [ "$SHELL_NAME" = "fish" ]; then
            echo "set -gx PATH $BIN_DIR \$PATH" >> "$RC_FILE"
        else
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$RC_FILE"
        fi
        echo -e "${GREEN}Added Nitro to PATH in $RC_FILE${NC}"
    fi
}

# Main installation
main() {
    echo -e "${BLUE}Installing Nitro Package Manager${NC}"
    echo "=============================="
    
    detect_platform
    
    # Try to download pre-built binary first
    if ! download_binary 2>/dev/null; then
        build_from_source
    fi
    
    setup_shell
    
    # Create initial config
    "$BIN_DIR/nitro" --version >/dev/null 2>&1 || true
    
    echo ""
    echo -e "${GREEN}✅ Nitro installed successfully!${NC}"
    echo ""
    echo "To get started:"
    echo "  1. Reload your shell: source ~/.bashrc (or ~/.zshrc)"
    echo "  2. Import Homebrew taps: nitro homebrew import"
    echo "  3. Search for packages: nitro search <package>"
    echo "  4. Install packages: nitro install <package>"
    echo ""
    echo "For more information: nitro --help"
}

# Run main installation
main "$@"