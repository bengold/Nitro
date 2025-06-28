#!/bin/bash
set -e

# Quick local installation script for Nitro
# This builds and installs Nitro from the current directory

echo "🚀 Installing Nitro from local source..."
echo "====================================="

# Check for Rust
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Rust is required. Install it with:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build release version
echo "📦 Building Nitro (this may take a few minutes)..."
cargo build --release

# Install to user's local bin
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "📁 Installing to $INSTALL_DIR/nitro"
cp target/release/nitro "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/nitro"

# Check if PATH needs updating
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add it to your shell configuration:"
    echo "  For bash: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
    echo "  For zsh:  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
    echo ""
fi

echo "✅ Nitro installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Make sure $INSTALL_DIR is in your PATH"
echo "  2. Run: nitro homebrew import"
echo "  3. Try: nitro search wget"