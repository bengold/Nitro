# Nitro Package Manager

A high-performance package manager written in Rust that leverages Homebrew formulae while providing significant performance improvements through parallel operations, intelligent caching, and pre-built binaries.

## Building

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