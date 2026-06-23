# herdr-insight Makefile
# Plugin maintenance: build, install, link, uninstall, update, test, lint

PLUGIN_ID := herdr-insight
PLUGIN_DIR := $(shell pwd)
CONFIG_DIR := $(shell herdr plugin config-dir $(PLUGIN_ID) 2>/dev/null || echo "$$HOME/.config/herdr/plugins/config/$(PLUGIN_ID)")

.PHONY: build link unlink install uninstall update clean test lint check help

## Build release binary
build:
	cargo build --release --locked

## Link plugin to herdr (local development)
link: build
	herdr plugin link $(PLUGIN_DIR)

## Unlink plugin from herdr
unlink:
	herdr plugin unlink $(PLUGIN_ID)

## Install plugin from GitHub
install:
	herdr plugin install 0x5c0f/herdr-insight

## Uninstall plugin
uninstall:
	herdr plugin uninstall $(PLUGIN_ID)

## Update plugin (reinstall from GitHub)
update:
	herdr plugin unlink $(PLUGIN_ID) 2>/dev/null || true
	herdr plugin install 0x5c0f/herdr-insight

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
		echo '[timeline.columns]' > "$(CONFIG_DIR)/config.toml"; \
		echo 'time = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'state = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'agent = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'pane = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'status = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'duration = true' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'session = false' >> "$(CONFIG_DIR)/config.toml"; \
		echo 'output = false' >> "$(CONFIG_DIR)/config.toml"; \
		echo "created $(CONFIG_DIR)/config.toml"; \
	else \
		echo "config already exists at $(CONFIG_DIR)/config.toml"; \
	fi

## Show available targets
help:
	@echo "herdr-insight Makefile"
	@echo ""
	@echo "Build & Install:"
	@echo "  make build         Build release binary"
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
