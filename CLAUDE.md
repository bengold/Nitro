# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

Nitro is a high-performance package manager written in Rust that leverages Homebrew formulae. The Phase 1 implementation provides basic CLI functionality, formula parsing, package management, and search capabilities.

## Development Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Check for compilation errors
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run the binary
./target/release/nitro --help
```

## Architecture Overview

The project follows a modular architecture:

- `src/cli/` - Command-line interface and command handlers
- `src/core/` - Core functionality (package manager, formula parser, resolver, installer)
- `src/download/` - Download manager with resume support
- `src/cache/` - Multi-level caching system
- `src/search/` - Full-text search with Tantivy
- `src/ui/` - Terminal UI components and progress reporting

## Key Conventions

1. **Error Handling**: Use `anyhow::Result` for most functions, with custom `NitroError` types for domain-specific errors
2. **Async Runtime**: Tokio is used for all async operations
3. **Progress Reporting**: All long-running operations should use the `ProgressReporter`
4. **Formula Compatibility**: Maintain compatibility with Homebrew's Ruby formula format

## Current Configuration

- Claude Code permissions are configured in `.claude/settings.local.json`
- The project directory is located at `/Users/bengold/Documents/GitHub/Nitro`
- Binary installs to `/usr/local/Cellar` by default (Homebrew-compatible layout)