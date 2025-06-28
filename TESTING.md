# Testing Nitro Package Manager

## Build and Run

```bash
# Build the project
cargo build --release

# Run the binary
./target/release/nitro --help
```

## Basic Testing

### 1. Check Version and Help
```bash
./target/release/nitro --version
./target/release/nitro --help
./target/release/nitro install --help
```

### 2. List Commands (Safe to run)
```bash
# List installed packages (empty initially)
./target/release/nitro list

# List configured taps
./target/release/nitro tap list

# Search for packages
./target/release/nitro search wget
./target/release/nitro search python
```

### 3. Info Command
```bash
# Get info about a package (requires tap data)
./target/release/nitro info wget
```

## Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_formula_parser
```

## Integration Testing

Use the provided test scripts:

```bash
# Basic functionality test
./test_nitro.sh

# Local environment test (no root needed)
./test_local.sh
```

## Manual Testing with Real Packages

⚠️ **Note**: Installing packages requires:
1. Write permissions to `/usr/local/Cellar` (may need sudo)
2. Git installed for tap management
3. Internet connection for downloading formulae

```bash
# Add a tap (if not already added)
sudo ./target/release/nitro tap add homebrew/core

# Update tap data
sudo ./target/release/nitro tap update

# Search for a package
./target/release/nitro search wget

# Install a simple package
sudo ./target/release/nitro install wget

# List installed packages
./target/release/nitro list

# Uninstall a package
sudo ./target/release/nitro uninstall wget
```

## Test Coverage Areas

### ✅ Implemented and Testable:
- CLI argument parsing
- Formula parsing (Ruby DSL)
- Dependency resolution
- Search functionality
- Tap management
- Progress reporting
- Error handling

### ⚠️ Requires Environment Setup:
- Package installation (needs write permissions)
- Source compilation (needs build tools)
- Binary downloads (not yet implemented)
- Tap synchronization (needs git and internet)

### 🚧 Future Testing:
- Binary package installation
- Delta updates
- Custom CDN support
- Parallel installations

## Common Issues

1. **Permission Denied**: Most operations that modify the system require sudo
2. **Package Not Found**: Taps need to be added and synced first
3. **Git Not Found**: Tap operations require git to be installed
4. **No Internet**: Can't download formulae or packages without internet

## Development Testing

For development, you can modify the installer to use a local directory:
1. Change `/usr/local` to a user-writable directory in `src/core/installer.rs`
2. Rebuild and test without sudo