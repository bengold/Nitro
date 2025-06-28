# Installation Guide

## Installation Speed Comparison

| Method | Time | Requirements |
|--------|------|--------------|
| Pre-built binary (planned) | ~5 seconds | None |
| Homebrew tap (planned) | ~30 seconds | Homebrew |
| Make install | ~2 minutes | Rust toolchain |
| From source | ~2-3 minutes | Rust toolchain |

## Fastest Installation (Current)

```bash
# Clone and install in under 3 minutes
git clone https://github.com/bengold/Nitro.git && \
cd nitro && \
make install
```

## Platform-Specific Instructions

### macOS

```bash
# Using Homebrew (coming soon)
brew tap bengold/Nitro
brew install nitro

# OR using Make
git clone https://github.com/bengold/Nitro.git
cd nitro
make install-system  # Installs to /usr/local/bin
```

### Linux

```bash
# One-liner (coming soon)
curl -fsSL https://raw.githubusercontent.com/bengold/Nitro/main/install.sh | bash

# OR using Make
git clone https://github.com/bengold/Nitro.git
cd nitro
make install  # Installs to ~/.local/bin
```

### Windows (WSL)

```bash
# Same as Linux instructions above
```

## Post-Installation Setup

1. **Add to PATH** (if not using system install):
   ```bash
   # For bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   
   # For zsh
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

2. **Import Homebrew taps** (if you have Homebrew):
   ```bash
   nitro homebrew import
   ```

3. **Verify installation**:
   ```bash
   nitro --version
   nitro search wget
   ```

## Uninstallation

```bash
# Remove binary
rm ~/.local/bin/nitro  # or /usr/local/bin/nitro

# Remove data (optional)
rm -rf ~/Library/Application\ Support/com.nitro.nitro  # macOS
rm -rf ~/.local/share/nitro  # Linux
```

## Troubleshooting

### "Command not found"
Make sure `~/.local/bin` is in your PATH.

### "Permission denied"
Use `chmod +x` on the binary or run `make install` instead of copying manually.

### Build fails
Ensure you have Rust 1.70+ installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```