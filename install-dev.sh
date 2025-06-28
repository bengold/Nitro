#!/bin/bash

# Quick installation script for development/testing
# This installs Nitro in the current directory for testing

set -e

echo "🚀 Installing Nitro (Development Mode)"
echo "===================================="
echo

# Build the project
echo "Building Nitro..."
cargo build --release

# Create a wrapper script that sets up the environment
cat > nitro-dev << 'EOF'
#!/bin/bash
# Nitro development wrapper

# Set the binary path
NITRO_BIN="$(dirname "$0")/target/release/nitro"

# Check if binary exists
if [ ! -f "$NITRO_BIN" ]; then
    echo "Error: Nitro binary not found at $NITRO_BIN"
    echo "Run: cargo build --release"
    exit 1
fi

# Run Nitro with all arguments
exec "$NITRO_BIN" "$@"
EOF

chmod +x nitro-dev

echo
echo "✅ Nitro installed successfully!"
echo
echo "To use Nitro:"
echo "  ./nitro-dev --help"
echo
echo "To test with existing Homebrew:"
echo "  ./nitro-dev tap list"
echo "  ./nitro-dev search wget"
echo
echo "To install globally:"
echo "  cargo install --path ."
echo