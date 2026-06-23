# herdr-insight Makefile
# Plugin maintenance: build, install, link, uninstall, update, test, lint, release

PLUGIN_ID := herdr-insight
PLUGIN_DIR := $(shell pwd)
CONFIG_DIR := $(shell herdr plugin config-dir $(PLUGIN_ID) 2>/dev/null || echo "$$HOME/.config/herdr/plugins/config/$(PLUGIN_ID)")
VERSION := $(shell grep '^version' crates/app/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
GITHUB_REPO := 0x5c0f/herdr-insight

# Detect platform
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH := $(shell uname -m | sed 's/x86_64/x86_64/' | sed 's/aarch64/arm64/')
ASSET_NAME := herdr-insight-$(OS)-$(ARCH)

.PHONY: build build-static link unlink install uninstall update clean test lint check release release-upload help

## Build release binary (local cargo or download pre-compiled)
build:
	@if command -v cargo >/dev/null 2>&1; then \
		echo "Building with cargo..."; \
		cargo build --release --locked; \
	else \
		echo "Cargo not found, downloading pre-compiled binary..."; \
		$(MAKE) download; \
	fi

## Download pre-compiled binary from GitHub releases
download:
	@echo "Downloading $(ASSET_NAME) v$(VERSION)..."
	@curl -L -o herdr-insight \
		"https://github.com/$(GITHUB_REPO)/releases/download/v$(VERSION)/$(ASSET_NAME)"
	@chmod +x herdr-insight
	@echo "Downloaded: herdr-insight"

## Build static binary (musl, for Linux distribution)
build-static:
	@echo "Building static binary with musl..."
	rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
	cargo build --release --locked --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/herdr-insight ./herdr-insight-static
	@echo "Static binary: herdr-insight-static"

## Build release tarballs for all platforms
release: build-static
	@echo "Creating release tarballs..."
	@mkdir -p dist
	tar -czf dist/$(ASSET_NAME).tar.gz -C target/release herdr-insight
	tar -czf dist/herdr-insight-linux-x86_64-static.tar.gz herdr-insight-static
	@echo "Release artifacts in dist/"

## Upload release to GitHub (requires gh CLI)
release-upload:
	@echo "Uploading release v$(VERSION)..."
	gh release create v$(VERSION) dist/*.tar.gz \
		--repo $(GITHUB_REPO) \
		--title "v$(VERSION)" \
		--notes "Release v$(VERSION)"

## Link plugin to herdr (local development, requires server)
link: build
	@herdr status 2>&1 | grep -q "status: running" || { echo "Error: herdr server not running. Start with: herdr server"; exit 1; }
	herdr plugin link $(PLUGIN_DIR)

## Unlink plugin from herdr
unlink:
	herdr plugin unlink $(PLUGIN_ID)

## Install plugin from GitHub
install:
	herdr plugin install $(GITHUB_REPO) --yes

## Uninstall plugin
uninstall:
	herdr plugin uninstall $(PLUGIN_ID)

## Update plugin (reinstall from GitHub)
update:
	herdr plugin unlink $(PLUGIN_ID) 2>/dev/null || true
	herdr plugin install $(GITHUB_REPO) --yes

## Run tests
test:
	cargo test --workspace

## Run lint checks
lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

## Run full checks
check: lint test

## Clean build artifacts
clean:
	cargo clean
	rm -f herdr-insight herdr-insight-static
	rm -rf dist

## Open timeline panel
open:
	herdr plugin pane open --plugin $(PLUGIN_ID) --entrypoint timeline

## Print plugin info
info:
	herdr plugin list 2>&1 | grep $(PLUGIN_ID) || echo "plugin not linked/installed"

## Print config directory
config:
	@echo $(CONFIG_DIR)

## Print plugin state directory
state:
	herdr plugin config-dir $(PLUGIN_ID) 2>/dev/null || echo "run 'herdr plugin config-dir $(PLUGIN_ID)'"

## Show plugin logs
logs:
	herdr plugin log list --plugin $(PLUGIN_ID)

## Create default config file
config-init:
	@mkdir -p "$(CONFIG_DIR)"
	@if [ ! -f "$(CONFIG_DIR)/config.toml" ]; then \
		echo '[columns]' > "$(CONFIG_DIR)/config.toml"; \
		echo 'time = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'state = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'agent = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'pane = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'status = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'duration = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'session = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo "created $(CONFIG_DIR)/config.toml"; \
	else \
		echo "config already exists at $(CONFIG_DIR)/config.toml"; \
	fi

## Show available targets
help:
	@echo "herdr-insight Makefile v$(VERSION)"
	@echo ""
	@echo "Build & Install:"
	@echo "  make build         Build release binary (cargo or download)"
	@echo "  make download      Download pre-compiled binary"
	@echo "  make build-static  Build static binary (musl)"
	@echo "  make release       Build release tarballs"
	@echo "  make release-upload Upload release to GitHub"
	@echo "  make link          Build and link to herdr (local dev)"
	@echo "  make unlink        Unlink from herdr"
	@echo "  make install       Install from GitHub"
	@echo "  make uninstall     Uninstall plugin"
	@echo "  make update        Reinstall from GitHub"
	@echo ""
	@echo "Test & Lint:"
	@echo "  make test          Run tests"
	@echo "  make lint          Run rustfmt + clippy"
	@echo "  make check         Run lint + test"
	@echo "  make clean         Clean build artifacts"
	@echo ""
	@echo "Usage:"
	@echo "  make open          Open timeline panel"
	@echo "  make info          Show plugin registration info"
	@echo "  make config        Show config directory path"
	@echo "  make state         Show state directory path"
	@echo "  make logs          Show plugin logs"
	@echo "  make config-init   Create default config file"
