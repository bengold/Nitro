# Nitro Package Manager

A high-performance package manager written in Rust that leverages Homebrew formulae while providing significant performance improvements through parallel operations, intelligent caching, and pre-built binaries.

## Quick Installation

### Option 1: Using Make (Fastest)
```bash
git clone https://github.com/yourusername/nitro.git
cd nitro
make install         # Installs to ~/.local/bin
# OR
make install-system  # Installs to /usr/local/bin (requires sudo)
```

### Option 2: One-liner (Coming Soon)
```bash
curl -fsSL https://raw.githubusercontent.com/yourusername/nitro/main/install.sh | bash
```

### Option 3: From Source

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Usage

```bash
# Install a package
nitro install wget

# Search for packages
nitro search python

# List installed packages
nitro list

# Update packages
nitro update

# Add a tap
nitro tap add homebrew/core

# Get help
nitro --help
```

## Features

- **Homebrew Compatibility**: Works with existing Homebrew formulae
- **Parallel Operations**: Download and install multiple packages concurrently
- **Binary Packages**: Skip compilation with pre-built binaries when available
- **Smart Caching**: Multi-level caching for faster operations
- **Rich Terminal UI**: Beautiful progress bars and interactive features
- **Fast Search**: Full-text search with fuzzy matching support

## Development Status

This is Phase 1 of the Nitro implementation, providing:
- Basic CLI structure
- Homebrew formula parsing
- Package installation framework
- Dependency resolution
- Search functionality
- Progress reporting

Next phases will add binary package support, performance optimizations, and advanced features.