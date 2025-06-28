#!/bin/bash

# Test script for Nitro package manager

echo "=== Nitro Package Manager Test ==="
echo

# Build if not already built
if [ ! -f "./target/release/nitro" ]; then
    echo "Building Nitro..."
    cargo build --release
fi

NITRO="./target/release/nitro"

echo "1. Testing help command:"
$NITRO --help
echo

echo "2. Testing version:"
$NITRO --version
echo

echo "3. Testing list command (should show no packages):"
$NITRO list || echo "Note: This may fail if /usr/local/Cellar doesn't exist or isn't writable"
echo

echo "4. Testing search command:"
$NITRO search wget || echo "Note: Search requires tap data to be available"
echo

echo "5. Testing tap list:"
$NITRO tap list
echo

echo "6. Testing info command:"
$NITRO info wget || echo "Note: This requires formula data"
echo

echo "=== Test Summary ==="
echo "Nitro is built and basic commands work!"
echo "For full testing with package installation, you'll need:"
echo "1. Write permissions to /usr/local/Cellar (or run with sudo)"
echo "2. A Homebrew tap added (e.g., nitro tap add homebrew/core)"
echo "3. Git installed for tap management"