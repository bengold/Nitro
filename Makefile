.PHONY: install build release clean test help

# Default installation directory
PREFIX ?= $(HOME)/.local

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: build ## Build and install Nitro to PREFIX/bin (default: ~/.local/bin)
	@echo "Installing Nitro to $(PREFIX)/bin"
	@mkdir -p $(PREFIX)/bin
	@cp target/release/nitro $(PREFIX)/bin/
	@chmod +x $(PREFIX)/bin/nitro
	@echo "✅ Nitro installed successfully!"
	@echo ""
	@echo "Make sure $(PREFIX)/bin is in your PATH"
	@echo "Run 'nitro --help' to get started"

build: ## Build release version
	@echo "Building Nitro..."
	@cargo build --release

dev: ## Build and run development version
	@cargo build
	@./target/debug/nitro

test: ## Run tests
	@cargo test

clean: ## Clean build artifacts
	@cargo clean

release: ## Create a new release (requires VERSION)
ifndef VERSION
	$(error VERSION is not set. Use: make release VERSION=0.1.0)
endif
	@echo "Creating release v$(VERSION)..."
	@git tag -a v$(VERSION) -m "Release v$(VERSION)"
	@echo "Tagged v$(VERSION). Push with: git push origin v$(VERSION)"

# Quick installation commands
install-bash: install ## Install and add to .bashrc
	@grep -q "$(PREFIX)/bin" ~/.bashrc || echo 'export PATH="$$HOME/.local/bin:$$PATH"' >> ~/.bashrc
	@echo "✅ Added to ~/.bashrc. Run: source ~/.bashrc"

install-zsh: install ## Install and add to .zshrc
	@grep -q "$(PREFIX)/bin" ~/.zshrc || echo 'export PATH="$$HOME/.local/bin:$$PATH"' >> ~/.zshrc
	@echo "✅ Added to ~/.zshrc. Run: source ~/.zshrc"

install-system: build ## Install system-wide (requires sudo)
	@echo "Installing Nitro to /usr/local/bin (requires sudo)"
	@sudo cp target/release/nitro /usr/local/bin/
	@sudo chmod +x /usr/local/bin/nitro
	@echo "✅ Nitro installed system-wide!"