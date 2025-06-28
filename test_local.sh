#!/bin/bash

# Test Nitro in a local environment without needing root permissions

echo "=== Setting up local test environment ==="

# Create a local test directory structure
export NITRO_TEST_DIR="$PWD/test_env"
mkdir -p "$NITRO_TEST_DIR"/{Cellar,bin,taps}

# Create a wrapper script that sets up the environment
cat > "$NITRO_TEST_DIR/nitro_test" << 'EOF'
#!/bin/bash
# Override the installation prefix to use our test directory
export NITRO_PREFIX="$NITRO_TEST_DIR"
exec "$PWD/target/release/nitro" "$@"
EOF

chmod +x "$NITRO_TEST_DIR/nitro_test"
NITRO="$NITRO_TEST_DIR/nitro_test"

echo "Test environment created at: $NITRO_TEST_DIR"
echo

echo "=== Running Nitro Tests ==="
echo

echo "1. List packages (empty):"
$NITRO list
echo

echo "2. List taps:"
$NITRO tap list
echo

echo "3. Search for a package:"
$NITRO search python
echo

echo "4. Get info about a package:"
$NITRO info python || echo "Package not found (expected if tap not synced)"
echo

echo "5. Test invalid command:"
$NITRO invalid-command 2>&1 | head -5
echo

echo "=== Advanced Tests (if you have git and internet) ==="
echo

echo "6. Try to add a tap (requires git):"
if command -v git &> /dev/null; then
    $NITRO tap add homebrew/cask || echo "Tap operation failed"
    $NITRO tap list
else
    echo "Git not found, skipping tap test"
fi

echo
echo "=== Test Complete ==="
echo "To test installation, you would need to:"
echo "1. Either run with sudo for /usr/local/Cellar access"
echo "2. Or modify the installer.rs to use a custom prefix"
echo "3. Have a synced Homebrew tap with formulae"