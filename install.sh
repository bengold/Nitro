#!/bin/bash

# Nitro installer script for macOS

set -e

echo "Installing Nitro package manager..."

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This installer is for macOS only"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project in release mode
echo "Building Nitro..."
cargo build --release

# Create the installation directory if it doesn't exist
sudo mkdir -p /usr/local/bin

# Copy the binary to /usr/local/bin
echo "Installing Nitro to /usr/local/bin..."
sudo cp target/release/nitro /usr/local/bin/

# Make sure it's executable
sudo chmod +x /usr/local/bin/nitro

# Verify installation
if command -v nitro &> /dev/null; then
    echo "✅ Nitro has been successfully installed!"
    echo ""
    nitro --version
    echo ""
    echo "You can now use 'nitro' from anywhere in your terminal."
    echo "Try 'nitro --help' to see available commands."
else
    echo "❌ Installation failed. Please check the error messages above."
    exit 1
fi