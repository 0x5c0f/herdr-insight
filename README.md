# herdr-insight

Agent status timeline pane for [herdr](https://herdr.dev).

Real-time visualization of agent state transitions across all workspaces. Track when agents start working, get blocked, finish tasks, or go idle.

## Features

- Real-time agent status timeline (working/blocked/done/idle)
- Configurable columns (show/hide any column)
- Session ID tracking
- Last output preview
- Horizontal split (bottom) and vertical split (right) modes
- 7-day timeline data retention
- Multi-workspace agent tracking

## Install

### From GitHub (recommended)

```bash
herdr plugin install 0x5c0f/herdr-insight --yes
```

### From GitHub with Makefile

```bash
git clone https://github.com/0x5c0f/herdr-insight.git
cd herdr-insight
make install
```

### Local development

```bash
git clone https://github.com/0x5c0f/herdr-insight.git
cd herdr-insight
make link
```

### Pre-compiled binary

```bash
# Download latest release
make download

# Or install from GitHub (auto-detects: cargo build or download)
make install
```

## Usage

### Keybindings

Add to `~/.config/herdr/config.toml`:

```toml
# Horizontal split (bottom)
[[keys.command]]
key = "prefix+t"
type = "plugin_action"
command = "herdr-insight.open-timeline-down"

# Vertical split (right)
[[keys.command]]
key = "prefix+shift+t"
type = "plugin_action"
command = "herdr-insight.open-timeline-right"
```

### CLI commands

```bash
# Open timeline (default split direction)
herdr plugin pane open --plugin herdr-insight --entrypoint timeline

# Open with specific direction
herdr plugin pane open --plugin herdr-insight --entrypoint timeline --direction down
herdr plugin pane open --plugin herdr-insight --entrypoint timeline --direction right

# Invoke actions directly
herdr plugin action invoke herdr-insight.open-timeline-down
herdr plugin action invoke herdr-insight.open-timeline-right
```

### Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Close timeline |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |

## Configuration

Create config file at `~/.config/herdr/plugins/config/herdr-insight/config.toml`:

```bash
make config-init
```

### Column visibility

```toml
[timeline.columns]
time = true        # Timestamp
state = true       # Status icon (●▲○◌)
agent = true       # Agent name
pane = true        # Pane ID [wX:pY]
status = true      # Status text (idle/working/blocked/done)
duration = true    # Duration in previous state
session = false    # Session ID (default: off)
output = false     # Last output preview (default: off, has performance cost)
```

### Default values

| Column | Default | Description |
|--------|---------|-------------|
| `time` | `true` | Show timestamp |
| `state` | `true` | Show status icon |
| `agent` | `true` | Show agent name |
| `pane` | `true` | Show pane ID |
| `status` | `true` | Show status text |
| `duration` | `true` | Show duration |
| `session` | `false` | Show session ID |
| `output` | `false` | Show last output preview |

## Build from source

### Requirements

- Rust toolchain (cargo)
- herdr >= 0.7.0

### Build commands

```bash
make build          # Build release binary (cargo or download)
make build-static   # Build static binary (musl, Linux only)
make release        # Build release tarballs
make release-upload # Upload to GitHub Releases
```

### Makefile targets

```
make help           # Show all available targets
make build          # Build release binary
make download       # Download pre-compiled binary
make build-static   # Build static binary (musl)
make release        # Build release tarballs
make release-upload Upload release to GitHub
make link           # Build and link to herdr (local dev)
make unlink         # Unlink from herdr
make install        # Install from GitHub
make uninstall      # Uninstall plugin
make update         # Reinstall from GitHub
make test           # Run tests
make lint           # Run rustfmt + clippy
make check          # Run lint + test
make clean          # Clean build artifacts
make open           # Open timeline panel
make info           # Show plugin registration info
make config         # Show config directory path
make state          # Show state directory path
make logs           # Show plugin logs
make config-init    # Create default config file
```

## Architecture

Multi-crate Rust workspace:

```
herdr-insight/
├── crates/
│   ├── common/     # Shared types (AgentState, AgentSnapshot, etc.)
│   ├── domain/     # Business logic (poller, transitions)
│   ├── infra/      # Infrastructure (herdr CLI, persistence)
│   ├── tui/        # Timeline TUI (render, state, config)
│   └── app/        # Binary entry point
└── herdr-plugin.toml
```

## License

MIT
