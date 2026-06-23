# herdr-insight

Agent status timeline pane for [herdr](https://herdr.dev).

## Install

```bash
herdr plugin install 0x5c0f/herdr-insight
```

Or for local development:

```bash
git clone https://github.com/0x5c0f/herdr-insight.git
cd herdr-insight
cargo build --release --locked
herdr plugin link .
```

## Usage

Open the timeline pane:

```bash
herdr plugin pane open --plugin herdr-insight --entrypoint timeline
```

Or bind a key in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+t"
type = "plugin_pane"
command = "herdr-insight.timeline"
```

**Controls:** `q` quit, `↑/↓` or `j/k` scroll

## Requirements

- herdr >= 0.7.0
- Rust toolchain (for building from source)
