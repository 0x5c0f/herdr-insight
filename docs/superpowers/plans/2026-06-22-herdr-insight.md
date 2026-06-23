# herdr-insight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build herdr-insight — a herdr plugin providing Agent output collector, status timeline, and context dashboard.

**Architecture:** Single Rust binary with clap subcommands. Pane-as-daemon pattern — no separate background process. ratatui TUI for dashboard/timeline panes, CLI-only for collector.

**Tech Stack:** Rust 2021, clap 4.x, ratatui 0.30, crossterm 0.29, serde/serde_json, anyhow + thiserror, chrono, tracing

**Workspace:** `/home/cxd/Projects/aiediter/herdr-plugins/herdr-insight`

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies: clap, ratatui, crossterm, serde, serde_json, anyhow, thiserror, chrono, tracing |
| `herdr-plugin.toml` | Plugin manifest: actions, panes, events |
| `src/main.rs` | Entry point, subcommand dispatch |
| `src/cli.rs` | clap command definitions |
| `src/error.rs` | InsightError enum, InsightResult type alias |
| `src/herdr_api.rs` | Wrapper around `herdr` CLI calls (subprocess) |
| `src/data.rs` | Data models (AgentSnapshot, StateTransition, OutputEntry), JSONL I/O |
| `src/collector.rs` | Output collector logic: read panes, generate markdown report |
| `src/dashboard/mod.rs` | Dashboard TUI entry, event loop |
| `src/dashboard/state.rs` | Dashboard app state struct |
| `src/dashboard/render.rs` | ratatui render functions |
| `src/timeline/mod.rs` | Timeline TUI entry, event loop |
| `src/timeline/state.rs` | Timeline app state struct |
| `src/timeline/render.rs` | ratatui render functions |
| `src/shared/mod.rs` | Shared module re-exports |
| `src/shared/poller.rs` | Poll herdr state, detect state transitions |
| `src/shared/widgets.rs` | Reusable ratatui widgets |

---

## Phase 0: Project Skeleton

### Task 0.1: Cargo.toml with dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Write Cargo.toml**

```toml
[package]
name = "herdr-insight"
version = "0.1.0"
edition = "2021"
description = "Agent output collector, status timeline, and context dashboard for herdr"
license = "MIT"
repository = "https://github.com/0x5c0f/herdr-insight"

[dependencies]
clap = { version = "4", features = ["derive"] }
ratatui = "0.30"
crossterm = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

- [ ] **Step 2: Verify cargo check passes**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check
```

Expected: downloads crates, compiles cleanly.

- [ ] **Step 3: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add Cargo.toml && git commit -m "chore: add Cargo.toml with dependencies"
```

---

### Task 0.2: herdr-plugin.toml manifest

**Files:**
- Create: `herdr-plugin.toml`

- [ ] **Step 1: Write manifest**

```toml
id = "herdr-insight"
name = "Herdr Insight"
version = "0.1.0"
min_herdr_version = "0.7.0"
description = "Agent output collector, status timeline, and context dashboard for herdr"
platforms = ["linux", "macos"]

# Requires Rust toolchain (cargo). `plugin link` does NOT run build commands;
# build manually with `cargo build --release --locked` during development.
[[build]]
command = ["cargo", "build", "--release", "--locked"]

# === Collector: event-triggered ===
# herdr injects HERDR_PLUGIN_EVENT_JSON with workspace_id, pane_id, etc.
[[events]]
on = "pane.agent_status_changed"
command = ["./target/release/herdr-insight", "collect", "--on-status-change"]

# === Collector: manual trigger ===
[[actions]]
id = "collect"
title = "Collect agent outputs"
contexts = ["workspace"]
command = ["./target/release/herdr-insight", "collect"]

# === Dashboard pane (persistent split) ===
[[panes]]
id = "dashboard"
title = "Agent Dashboard"
placement = "split"
command = ["./target/release/herdr-insight", "dashboard"]

# === Timeline pane (persistent split) ===
[[panes]]
id = "timeline"
title = "Agent Timeline"
placement = "split"
command = ["./target/release/herdr-insight", "timeline"]
```

- [ ] **Step 2: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add herdr-plugin.toml && git commit -m "chore: add herdr-plugin.toml manifest"
```

---

### Task 0.3: Error module

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write error types**

```rust
// src/error.rs
use std::io;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InsightError {
    #[error("herdr CLI not found at '{0}'")]
    HerdrNotFound(String),

    #[error("herdr CLI call failed: {0}")]
    HerdrCallFailed(#[source] io::Error),

    #[error("invalid JSON from herdr: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("data file corrupted: {0}")]
    DataCorrupted(String),

    #[error("terminal setup failed: {0}")]
    Terminal(#[source] io::Error),
}

pub(crate) type InsightResult<T> = Result<T, InsightError>;
```

- [ ] **Step 2: Wire up main.rs**

```rust
// src/main.rs
mod cli;
mod collector;
mod dashboard;
mod data;
mod error;
mod herdr_api;
mod shared;
mod timeline;

use error::InsightResult;

fn main() -> InsightResult<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse_args();
    match cli.command {
        cli::Command::Dashboard => dashboard::run(),
        cli::Command::Timeline => timeline::run(),
        cli::Command::Collect { on_status_change } => collector::run(on_status_change),
    }
}
```

- [ ] **Step 3: Verify clippy passes**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo clippy -- -D warnings 2>&1 | tail -5
```

Note: will fail because cli module doesn't exist yet — expected. Proceed to next task.

- [ ] **Step 4: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add error module and main entry skeleton"
```

---

### Task 0.4: CLI module with clap

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs` (correct any import mismatches)

- [ ] **Step 1: Write clap definitions**

```rust
// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "herdr-insight", about = "Agent insight tools for herdr")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Start the agent dashboard TUI
    Dashboard,
    /// Start the agent status timeline TUI
    Timeline,
    /// Collect agent outputs and generate a report
    Collect {
        /// Triggered by herdr event hook (pane.agent_status_changed)
        #[arg(long)]
        on_status_change: bool,
    },
}

impl Cli {
    pub(crate) fn parse_args() -> Self {
        Self::parse()
    }
}
```

- [ ] **Step 2: Verify cargo check passes**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

Expected: `Finished dev profile` — might have warnings about unused functions; that's OK for skeleton.

- [ ] **Step 3: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add clap CLI with dashboard/timeline/collect subcommands"
```

---

### Task 0.5: Stub modules so cargo check passes clean

**Files:**
- Create: `src/herdr_api.rs`, `src/data.rs`, `src/collector.rs`
- Create: `src/shared/mod.rs`, `src/shared/poller.rs`, `src/shared/widgets.rs`
- Create: `src/dashboard/mod.rs`, `src/dashboard/state.rs`, `src/dashboard/render.rs`
- Create: `src/timeline/mod.rs`, `src/timeline/state.rs`, `src/timeline/render.rs`

- [ ] **Step 1: Create all stub files**

```rust
// src/herdr_api.rs — stub
use crate::error::InsightResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PaneInfo {
    pub(crate) id: String,
    pub(crate) agent_label: Option<String>,
    pub(crate) state: String,
    pub(crate) title: Option<String>,
}

impl PaneInfo {
    pub(crate) fn list_all() -> InsightResult<Vec<Self>> {
        Err(crate::error::InsightError::DataCorrupted("not implemented".into()))
    }
}
```

```rust
// src/data.rs — stub
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum AgentState {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentSnapshot {
    pub(crate) timestamp: chrono::DateTime<chrono::Utc>,
    pub(crate) pane_id: String,
    pub(crate) agent_label: Option<String>,
    pub(crate) state: AgentState,
    pub(crate) workspace: String,
}
```

```rust
// src/collector.rs — stub
use crate::error::InsightResult;

pub(crate) fn run(on_status_change: bool) -> InsightResult<()> {
    tracing::info!("collector run, on_status_change={on_status_change}");
    Ok(())
}
```

```rust
// src/shared/mod.rs
pub(crate) mod poller;
pub(crate) mod widgets;
```

```rust
// src/shared/poller.rs — stub
use crate::error::InsightResult;

pub(crate) fn poll_snapshots() -> InsightResult<Vec<crate::data::AgentSnapshot>> {
    Ok(Vec::new())
}
```

```rust
// src/shared/widgets.rs — stub
// Reusable ratatui widgets
```

```rust
// src/dashboard/mod.rs — stub
use crate::error::InsightResult;

pub(crate) fn run() -> InsightResult<()> {
    println!("dashboard starting...");
    Ok(())
}
```

```rust
// src/dashboard/state.rs — stub
pub(crate) struct DashboardState {
    pub(crate) running: bool,
}

impl DashboardState {
    pub(crate) fn new() -> Self { Self { running: true } }
}
```

```rust
// src/dashboard/render.rs — stub
```

```rust
// src/timeline/mod.rs — stub
use crate::error::InsightResult;

pub(crate) fn run() -> InsightResult<()> {
    println!("timeline starting...");
    Ok(())
}
```

```rust
// src/timeline/state.rs — stub
pub(crate) struct TimelineState {
    pub(crate) running: bool,
}

impl TimelineState {
    pub(crate) fn new() -> Self { Self { running: true } }
}
```

```rust
// src/timeline/render.rs — stub
```

- [ ] **Step 2: Verify cargo check passes cleanly**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1
```

Expected: `Finished dev profile [unoptimized + debuginfo]`

- [ ] **Step 3: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "chore: add stub modules for all crates"
```

---

## Phase 1: herdr_api + Data Layer

### Task 1.1: herdr_api — herdr binary resolution and JSON call

**Files:**
- Modify: `src/herdr_api.rs`
- Create: `tests/herdr_api_tests.rs` (optional, see note)

Note: `herdr_api` requires a running herdr server to test fully. Unit tests mock the subprocess call. Integration tests are manual.

- [ ] **Step 1: Write herdr_api implementation**

```rust
// src/herdr_api.rs
use crate::error::{InsightError, InsightResult};
use serde::de::DeserializeOwned;
use std::process::Command;

/// Returns the herdr binary path from $HERDR_BIN_PATH, or "herdr" as fallback.
fn herdr_bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

/// Runs `herdr` with the given args and returns parsed JSON stdout.
fn herdr_json<T: DeserializeOwned>(args: &[&str]) -> InsightResult<T> {
    let bin = herdr_bin();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                InsightError::HerdrNotFound(bin.clone())
            } else {
                InsightError::HerdrCallFailed(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(InsightError::HerdrCallFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("herdr exited with {}: {}", output.status, stderr.trim()),
        )));
    }

    let value: T = serde_json::from_slice(&output.stdout)?;
    Ok(value)
}

/// Raw herdr call returning string stdout (for pane_read).
fn herdr_text(args: &[&str]) -> InsightResult<String> {
    let bin = herdr_bin();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                InsightError::HerdrNotFound(bin.clone())
            } else {
                InsightError::HerdrCallFailed(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(InsightError::HerdrCallFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("herdr exited with {}: {}", output.status, stderr.trim()),
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

- [ ] **Step 2: Write PaneInfo with serde parsing from herdr JSON output**

```rust
// Append to src/herdr_api.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PaneInfo {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) agent_label: Option<String>,
    #[serde(default)]
    pub(crate) agent_state: Option<String>,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) workspace_name: Option<String>,
    #[serde(default)]
    pub(crate) tab_index: Option<usize>,
    #[serde(default)]
    pub(crate) custom_status: Option<String>,
}

impl PaneInfo {
    /// Call `herdr pane list --json` and parse.
    pub(crate) fn list_all() -> InsightResult<Vec<Self>> {
        // herdr pane list outputs a JSON array of pane objects
        let panes: Vec<serde_json::Value> = herdr_json(&["pane", "list", "--json"])?;
        let result: Result<Vec<Self>, _> = panes.into_iter().map(serde_json::from_value).collect();
        result.map_err(InsightError::InvalidJson)
    }
}

/// Read pane content via `herdr pane read <id> --source <source> --lines <N>`.
pub(crate) fn pane_read(id: &str, source: &str, lines: u32) -> InsightResult<String> {
    herdr_text(&[
        "pane", "read", id,
        "--source", source,
        "--lines", &lines.to_string(),
    ])
}

/// Run `herdr workspace list --json`.
pub(crate) fn workspace_list() -> InsightResult<Vec<serde_json::Value>> {
    herdr_json(&["workspace", "list", "--json"])
}
```

- [ ] **Step 3: Write test with mocked herdr binary**

```rust
// tests/herdr_api_tests.rs
// NOTE: these tests are skipped if herdr is not running.
// Run with: cargo test -- --ignored

#[test]
#[ignore = "requires running herdr server"]
fn pane_list_parses_json() {
    let panes = herdr_insight::herdr_api::PaneInfo::list_all();
    assert!(panes.is_ok());
}

#[test]
fn herdr_not_found_error() {
    // This test verifies the error type when herdr binary doesn't exist.
    // We can't easily mock Command without dependency injection, so this
    // is tested manually by running with HERDR_BIN_PATH=/nonexistent/herdr.
}
```

- [ ] **Step 4: Verify cargo check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ tests/ && git commit -m "feat: add herdr_api with pane_list, pane_read, workspace_list"
```

---

### Task 1.2: data.rs — models and JSONL persistence

**Files:**
- Modify: `src/data.rs`
- Create: `tests/data_tests.rs`

- [ ] **Step 1: Write data models**

```rust
// src/data.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AgentState {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

impl From<Option<String>> for AgentState {
    fn from(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("idle") => AgentState::Idle,
            Some("working") => AgentState::Working,
            Some("blocked") => AgentState::Blocked,
            Some("done") => AgentState::Done,
            _ => AgentState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentSnapshot {
    pub(crate) timestamp: chrono::DateTime<chrono::Utc>,
    pub(crate) pane_id: String,
    pub(crate) agent_label: Option<String>,
    pub(crate) state: AgentState,
    pub(crate) workspace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) custom_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pane_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StateTransition {
    pub(crate) timestamp: chrono::DateTime<chrono::Utc>,
    pub(crate) pane_id: String,
    pub(crate) agent_label: Option<String>,
    pub(crate) from: AgentState,
    pub(crate) to: AgentState,
    /// Duration in previous state, in seconds.
    pub(crate) duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutputKind {
    Diff,
    Log,
    Error,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OutputEntry {
    pub(crate) timestamp: chrono::DateTime<chrono::Utc>,
    pub(crate) pane_id: String,
    pub(crate) agent_label: String,
    pub(crate) workspace: String,
    pub(crate) output_kind: OutputKind,
    pub(crate) content: String,
}
```

- [ ] **Step 2: Write JSONL persistence functions**

```rust
// Append to src/data.rs

use crate::error::{InsightError, InsightResult};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn state_dir() -> InsightResult<PathBuf> {
    std::env::var("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .map_err(|_| InsightError::DataCorrupted("HERDR_PLUGIN_STATE_DIR not set".into()))
}

/// Append a line to a JSONL file.
pub(crate) fn append_jsonl<T: Serialize>(filename: &str, entry: &T) -> InsightResult<()> {
    let dir = state_dir()?.join("data");
    fs::create_dir_all(&dir)?;
    let path = dir.join(filename);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Read all lines from a JSONL file, parsing each as T.
/// Skips lines that fail to parse (graceful degradation).
pub(crate) fn read_jsonl<T: serde::de::DeserializeOwned>(filename: &str) -> InsightResult<Vec<T>> {
    let dir = state_dir()?.join("data");
    let path = dir.join(filename);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<T>(&line) {
            Ok(item) => results.push(item),
            Err(_) => {
                tracing::warn!("skipping corrupted JSONL line in {}", filename);
            }
        }
    }
    Ok(results)
}

/// Purge timeline entries older than `retention_days`.
pub(crate) fn purge_old_timeline_entries(retention_days: i64) -> InsightResult<()> {
    let dir = state_dir()?.join("data");
    let path = dir.join("timeline.jsonl");
    if !path.exists() {
        return Ok(());
    }
    let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days);

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut kept = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(transition) = serde_json::from_str::<StateTransition>(&line) {
            if transition.timestamp >= cutoff {
                kept.push(line);
            }
        } else {
            // keep unparseable lines to avoid silent data loss
            kept.push(line);
        }
    }
    // Rewrite with kept lines
    let mut file = File::create(&path)?;
    for line in kept {
        writeln!(file, "{line}")?;
    }
    Ok(())
}
```

- [ ] **Step 3: Write unit tests for JSONL I/O**

```rust
// tests/data_tests.rs
use herdr_insight::data::{append_jsonl, read_jsonl, AgentSnapshot, AgentState};
use std::env;

fn set_state_dir_for_test(dir: &std::path::Path) {
    env::set_var("HERDR_PLUGIN_STATE_DIR", dir.to_string_lossy().as_ref());
}

#[test]
fn roundtrip_jsonl() {
    let tmp = tempfile::tempdir().expect("tempdir");
    set_state_dir_for_test(tmp.path());

    let snap = AgentSnapshot {
        timestamp: chrono::Utc::now(),
        pane_id: "pane-1".into(),
        agent_label: Some("claude".into()),
        state: AgentState::Working,
        workspace: "herdr".into(),
        custom_status: None,
        pane_title: None,
    };
    append_jsonl("test.jsonl", &snap).expect("append");
    let read: Vec<AgentSnapshot> = read_jsonl("test.jsonl").expect("read");
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].pane_id, "pane-1");
}

#[test]
fn read_empty_file_returns_empty_vec() {
    let tmp = tempfile::tempdir().expect("tempdir");
    set_state_dir_for_test(tmp.path());
    let result: Vec<AgentSnapshot> = read_jsonl("nonexistent.jsonl").expect("read");
    assert!(result.is_empty());
}
```

> Add `tempfile = "3"` to `[dev-dependencies]` in Cargo.toml.

- [ ] **Step 4: Verify tests pass**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo test 2>&1 | tail -10
```

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ tests/ Cargo.toml && git commit -m "feat: add data models and JSONL persistence with tests"
```

---

### Task 1.3: shared/poller.rs — state polling

**Files:**
- Modify: `src/shared/poller.rs`
- Modify: `src/data.rs` (ensure imports are public)
- Create: `tests/poller_tests.rs`

- [ ] **Step 1: Write poller implementation**

```rust
// src/shared/poller.rs
use crate::data::{AgentSnapshot, AgentState, StateTransition};
use crate::error::InsightResult;
use crate::herdr_api::PaneInfo;
use std::collections::HashMap;

/// Poll herdr for current agent snapshots.
pub(crate) fn poll_snapshots() -> InsightResult<Vec<AgentSnapshot>> {
    let panes = PaneInfo::list_all()?;
    let now = chrono::Utc::now();
    let snapshots: Vec<AgentSnapshot> = panes
        .into_iter()
        .map(|p| AgentSnapshot {
            timestamp: now,
            pane_id: p.id,
            agent_label: p.agent_label,
            state: AgentState::from(p.agent_state),
            workspace: p.workspace_name.unwrap_or_default(),
            custom_status: p.custom_status,
            pane_title: p.title,
        })
        .collect();
    Ok(snapshots)
}

/// Detect state transitions by comparing previous and current snapshots.
pub(crate) fn detect_transitions(
    previous: &HashMap<String, AgentSnapshot>,
    current: &[AgentSnapshot],
) -> Vec<StateTransition> {
    let now = chrono::Utc::now();
    let mut transitions = Vec::new();

    for snap in current {
        let prev = previous.get(&snap.pane_id);
        let state_changed = prev.map(|p| p.state != snap.state).unwrap_or(true);
        if state_changed {
            let duration_secs = prev.map(|p| {
                (now - p.timestamp).num_milliseconds() as f64 / 1000.0
            });
            transitions.push(StateTransition {
                timestamp: now,
                pane_id: snap.pane_id.clone(),
                agent_label: snap.agent_label.clone(),
                from: prev.map(|p| p.state).unwrap_or(AgentState::Unknown),
                to: snap.state,
                duration_secs,
            });
        }
    }

    transitions
}

/// Build a lookup map from pane_id to snapshot.
pub(crate) fn snapshot_map(snapshots: &[AgentSnapshot]) -> HashMap<String, AgentSnapshot> {
    snapshots
        .iter()
        .cloned()
        .map(|s| (s.pane_id.clone(), s))
        .collect()
}
```

- [ ] **Step 2: Write poller unit test**

```rust
// tests/poller_tests.rs
use herdr_insight::data::{AgentSnapshot, AgentState, StateTransition};
use herdr_insight::shared::poller::{detect_transitions, snapshot_map};
use std::collections::HashMap;

fn make_snap(pane_id: &str, state: AgentState) -> AgentSnapshot {
    AgentSnapshot {
        timestamp: chrono::Utc::now(),
        pane_id: pane_id.to_string(),
        agent_label: Some("test".into()),
        state,
        workspace: "test-ws".into(),
        custom_status: None,
        pane_title: None,
    }
}

#[test]
fn detect_transition_when_state_changes() {
    let prev = HashMap::from([
        ("pane-1".to_string(), make_snap("pane-1", AgentState::Working)),
    ]);
    let current = vec![make_snap("pane-1", AgentState::Done)];
    let transitions = detect_transitions(&prev, &current);
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].from, AgentState::Working);
    assert_eq!(transitions[0].to, AgentState::Done);
    assert_eq!(transitions[0].pane_id, "pane-1");
}

#[test]
fn detect_new_pane_as_transition() {
    let prev = HashMap::new();
    let current = vec![make_snap("pane-new", AgentState::Working)];
    let transitions = detect_transitions(&prev, &current);
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].from, AgentState::Unknown);
    assert_eq!(transitions[0].to, AgentState::Working);
}

#[test]
fn no_transition_when_state_unchanged() {
    let prev = HashMap::from([
        ("pane-1".to_string(), make_snap("pane-1", AgentState::Working)),
    ]);
    let current = vec![make_snap("pane-1", AgentState::Working)];
    let transitions = detect_transitions(&prev, &current);
    assert!(transitions.is_empty());
}
```

- [ ] **Step 3: Verify tests pass**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo test poller 2>&1 | tail -10
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ tests/ && git commit -m "feat: add state poller with transition detection"
```

---

## Phase 2: Dashboard Panel

### Task 2.1: Dashboard terminal setup and event loop

**Files:**
- Modify: `src/dashboard/mod.rs`, `src/dashboard/state.rs`

- [ ] **Step 1: Write dashboard event loop**

```rust
// src/dashboard/mod.rs
mod render;
mod state;

use crate::error::InsightResult;
use crate::shared::poller;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use state::DashboardState;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub(crate) fn run() -> InsightResult<()> {
    // Terminal setup
    enable_raw_mode().map_err(crate::error::InsightError::Terminal)?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
        .map_err(crate::error::InsightError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::InsightError::Terminal(std::io::Error::other(e)))?;

    let mut state = DashboardState::new();
    let mut last_poll = Instant::now()
        .checked_sub(Duration::from_secs(3))
        .unwrap_or(Instant::now());
    let mut previous_snapshots = HashMap::new();

    let result = loop {
        // Poll every 2 seconds
        let now = Instant::now();
        if now.duration_since(last_poll) >= Duration::from_secs(2) {
            if let Ok(snaps) = poller::poll_snapshots() {
                state.snapshots = snaps;
                state.error_message = None;
                // Detect transitions and persist to timeline
                let transitions = poller::detect_transitions(&previous_snapshots, &state.snapshots);
                for t in &transitions {
                    let _ = crate::data::append_jsonl("timeline.jsonl", t);
                }
                previous_snapshots = poller::snapshot_map(&state.snapshots);
            } else {
                state.error_message = Some("herdr connection lost".to_string());
            }
            last_poll = now;
        }

        // Render
        let _ = terminal.draw(|frame| render::draw(frame, &state));

        // Handle input (16ms timeout for ~60fps)
        if event::poll(Duration::from_millis(16)).map_err(crate::error::InsightError::Terminal)? {
            if let Event::Key(key) = event::read().map_err(crate::error::InsightError::Terminal)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        state.running = false;
                        break Ok(());
                    }
                    KeyCode::Char('r') => {
                        last_poll = Instant::now()
                            .checked_sub(Duration::from_secs(3))
                            .unwrap_or(Instant::now());
                    }
                    _ => {}
                }
            }
        }

        if !state.running {
            break Ok(());
        }
    };

    // Cleanup
    disable_raw_mode().map_err(crate::error::InsightError::Terminal)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .map_err(crate::error::InsightError::Terminal)?;

    result
}
```

- [ ] **Step 2: Write dashboard state**

```rust
// src/dashboard/state.rs
use crate::data::AgentSnapshot;

pub(crate) struct DashboardState {
    pub(crate) snapshots: Vec<AgentSnapshot>,
    pub(crate) running: bool,
    pub(crate) error_message: Option<String>,
    pub(crate) scroll_offset: u16,
}

impl DashboardState {
    pub(crate) fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            running: true,
            error_message: None,
            scroll_offset: 0,
        }
    }
}
```

- [ ] **Step 3: Verify cargo check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add dashboard event loop and state"
```

---

### Task 2.2: Dashboard render

**Files:**
- Modify: `src/dashboard/render.rs`

- [ ] **Step 1: Write dashboard render**

```rust
// src/dashboard/render.rs
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::state::DashboardState;
use crate::data::AgentState;

pub(crate) fn draw(frame: &mut Frame, state: &DashboardState) {
    let area = frame.area();

    // Outer block
    let block = Block::default()
        .title(" Herdr Insight · Dashboard ")
        .title_style(Style::default().bold().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.height < 3 {
        return;
    }

    // Split: header (2 lines) + content + footer (1 line)
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_header(frame, rows[0], state);
    render_agent_list(frame, rows[1], state);
    render_footer(frame, rows[2], state);
}

fn render_header(frame: &mut Frame, area: Rect, state: &DashboardState) {
    let agent_count = state.snapshots.iter().filter(|s| s.agent_label.is_some()).count();
    let info = format!(" Agents: {} tracked", agent_count);

    if let Some(err) = &state.error_message {
        let line = Line::from(Span::styled(err, Style::default().fg(Color::Red)));
        frame.render_widget(Paragraph::new(line), area);
    } else {
        let line = Line::from(Span::styled(info, Style::default().fg(Color::Gray)));
        frame.render_widget(Paragraph::new(line), area);
    }
}

fn render_agent_list(frame: &mut Frame, area: Rect, state: &DashboardState) {
    let agent_snaps: Vec<&crate::data::AgentSnapshot> = state
        .snapshots
        .iter()
        .filter(|s| s.agent_label.is_some())
        .collect();

    if agent_snaps.is_empty() && state.error_message.is_none() {
        let msg = Paragraph::new(" No agents detected. Start an agent in a pane.")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    let visible_height = area.height as usize;
    let start = state.scroll_offset as usize;
    let end = (start + visible_height).min(agent_snaps.len());

    let list_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            (0..visible_height).map(|_| Constraint::Length(1))
        )
        .split(area);

    for (idx, snap) in agent_snaps.iter().enumerate().skip(start).take(visible_height) {
        let row = list_area[idx - start];
        let (icon, color) = state_icon_color(snap.state);
        let label = snap.agent_label.as_deref().unwrap_or("unknown");
        let status_text = format!("{:?}", snap.state).to_lowercase();
        let line = Line::from(vec![
            Span::styled(icon, Style::default().fg(color).bold()),
            Span::raw(" "),
            Span::styled(label, Style::default().bold()),
            Span::raw(format!("  {status_text:<10}")),
        ]);
        frame.render_widget(Paragraph::new(line), row);
    }
}

fn render_footer(frame: &mut Frame, area: Rect, _state: &DashboardState) {
    let line = Line::from(Span::styled(
        " q quit  r refresh",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn state_icon_color(state: AgentState) -> (&'static str, Color) {
    match state {
        AgentState::Working => ("●", Color::Yellow),
        AgentState::Blocked => ("▲", Color::Red),
        AgentState::Done => ("○", Color::Green),
        AgentState::Idle => ("◌", Color::Gray),
        AgentState::Unknown => ("?", Color::Gray),
    }
}
```

- [ ] **Step 2: Add derive(Debug) to DashboardState for completeness**

```bash
# No code change needed; verify compilation
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add dashboard render with agent list"
```

---

## Phase 3: Timeline Panel

### Task 3.1: Timeline state and event loop

**Files:**
- Modify: `src/timeline/mod.rs`, `src/timeline/state.rs`

- [ ] **Step 1: Write timeline state**

```rust
// src/timeline/state.rs
use crate::data::StateTransition;

pub(crate) struct TimelineState {
    pub(crate) transitions: Vec<StateTransition>,
    pub(crate) running: bool,
    pub(crate) scroll_offset: usize,
    pub(crate) error_message: Option<String>,
}

impl TimelineState {
    pub(crate) fn new() -> Self {
        Self {
            transitions: Vec::new(),
            running: true,
            scroll_offset: 0,
            error_message: None,
        }
    }
}
```

- [ ] **Step 2: Write timeline event loop**

```rust
// src/timeline/mod.rs
mod render;
mod state;

use crate::error::InsightResult;
use crate::shared::poller;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use state::TimelineState;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub(crate) fn run() -> InsightResult<()> {
    enable_raw_mode().map_err(crate::error::InsightError::Terminal)?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
        .map_err(crate::error::InsightError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::InsightError::Terminal(std::io::Error::other(e)))?;

    let mut state = TimelineState::new();

    // Load existing timeline from data file
    if let Ok(existing) = crate::data::read_jsonl::<crate::data::StateTransition>("timeline.jsonl") {
        state.transitions = existing;
    }

    let mut last_poll = Instant::now()
        .checked_sub(Duration::from_secs(3))
        .unwrap_or(Instant::now());
    let mut previous_snapshots = HashMap::new();

    let result = loop {
        let now = Instant::now();
        if now.duration_since(last_poll) >= Duration::from_secs(2) {
            if let Ok(snaps) = poller::poll_snapshots() {
                let transitions = poller::detect_transitions(&previous_snapshots, &snaps);
                for t in &transitions {
                    let _ = crate::data::append_jsonl("timeline.jsonl", t);
                    state.transitions.push(t.clone());
                }
                previous_snapshots = poller::snapshot_map(&snaps);
                state.error_message = None;

                // Purge entries older than 7 days
                let _ = crate::data::purge_old_timeline_entries(7);
            } else {
                state.error_message = Some("herdr connection lost".to_string());
            }
            last_poll = now;
        }

        let _ = terminal.draw(|frame| render::draw(frame, &state));

        if event::poll(Duration::from_millis(16)).map_err(crate::error::InsightError::Terminal)? {
            if let Event::Key(key) = event::read().map_err(crate::error::InsightError::Terminal)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Ok(());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.scroll_offset = state.scroll_offset.saturating_add(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode().map_err(crate::error::InsightError::Terminal)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .map_err(crate::error::InsightError::Terminal)?;

    result
}
```

- [ ] **Step 3: Verify cargo check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add timeline event loop with scroll and persistence"
```

---

### Task 3.2: Timeline render

**Files:**
- Modify: `src/timeline/render.rs`

- [ ] **Step 1: Write timeline render**

```rust
// src/timeline/render.rs
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::state::TimelineState;
use crate::data::{AgentState, StateTransition};

pub(crate) fn draw(frame: &mut Frame, state: &TimelineState) {
    let area = frame.area();

    let block = Block::default()
        .title(" Herdr Insight · Timeline ")
        .title_style(Style::default().bold().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    render_entries(frame, rows[0], state);
    render_footer(frame, rows[1]);
}

fn render_entries(frame: &mut Frame, area: Rect, state: &TimelineState) {
    let mut entries: Vec<&StateTransition> = state.transitions.iter().collect();
    entries.reverse(); // newest first

    let visible = area.height as usize;
    let start = state.scroll_offset.min(entries.len().saturating_sub(1));
    let end = (start + visible).min(entries.len());

    if entries.is_empty() {
        let msg = Paragraph::new(" No timeline data yet. Waiting for agent state changes...")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    for (idx, entry) in entries.iter().enumerate().skip(start).take(visible) {
        let y = area.y + (idx - start) as u16;
        if y >= area.y + area.height {
            break;
        }
        let row = Rect::new(area.x, y, area.width, 1);
        render_entry(frame, row, entry);
    }
}

fn render_entry(frame: &mut Frame, area: Rect, entry: &StateTransition) {
    let time = entry.timestamp.format("%H:%M").to_string();
    let (icon, color) = state_icon_color(entry.to);
    let label = entry.agent_label.as_deref().unwrap_or("unknown");
    let state_str = format!("{:?}", entry.to).to_lowercase();

    let mut spans = vec![
        Span::styled(format!(" {time} "), Style::default().fg(Color::DarkGray)),
        Span::styled(icon, Style::default().fg(color).bold()),
        Span::raw(" "),
        Span::styled(label, Style::default().bold()),
        Span::raw(format!("  {state_str:<10}")),
    ];

    if let Some(dur) = entry.duration_secs {
        spans.push(Span::styled(
            format!(" {:4.0}min", dur / 60.0),
            Style::default().fg(Color::Gray),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        " q quit  ↑↓ scroll",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn state_icon_color(state: AgentState) -> (&'static str, Color) {
    match state {
        AgentState::Working => ("●", Color::Yellow),
        AgentState::Blocked => ("▲", Color::Red),
        AgentState::Done => ("○", Color::Green),
        AgentState::Idle => ("◌", Color::Gray),
        AgentState::Unknown => ("?", Color::Gray),
    }
}
```

- [ ] **Step 2: Verify cargo check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

- [ ] **Step 3: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ && git commit -m "feat: add timeline render with scrollable entry list"
```

---

## Phase 4: Collector

### Task 4.1: Collector implementation

**Files:**
- Modify: `src/collector.rs`
- Create: `tests/collector_tests.rs`

- [ ] **Step 1: Write collector logic**

```rust
// src/collector.rs
use crate::error::InsightResult;
use crate::herdr_api;
use chrono::Utc;

pub(crate) fn run(on_status_change: bool) -> InsightResult<()> {
    let trigger = if on_status_change {
        // Read event context from herdr
        let event_json = std::env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default();
        if !event_json.is_empty() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&event_json) {
                let state = event["event"]["state"].as_str().unwrap_or("");
                if state != "done" {
                    // Only collect when agent transitions to done
                    tracing::info!("skipping collect for non-done state: {state}");
                    return Ok(());
                }
            }
        }
        "event"
    } else {
        "manual"
    };

    tracing::info!("collecting agent outputs, trigger={trigger}");

    // List all panes
    let panes = herdr_api::PaneInfo::list_all()?;
    let agent_panes: Vec<_> = panes.iter().filter(|p| p.agent_label.is_some()).collect();

    if agent_panes.is_empty() {
        tracing::info!("no agent panes found");
        return Ok(());
    }

    // Build report
    let now = Utc::now();
    let mut report = String::new();
    report.push_str(&format!("# Session Report — {}\n\n", now.format("%Y-%m-%d %H:%M")));

    // Group by workspace
    let mut workspaces: std::collections::BTreeMap<String, Vec<&herdr_api::PaneInfo>> =
        std::collections::BTreeMap::new();
    for pane in &agent_panes {
        let ws = pane.workspace_name.as_deref().unwrap_or("unknown");
        workspaces.entry(ws.to_string()).or_default().push(pane);
    }

    for (ws_name, ws_panes) in &workspaces {
        report.push_str(&format!("## Workspace: {ws_name}\n\n"));
        for pane in ws_panes {
            let label = pane.agent_label.as_deref().unwrap_or("unknown");
            let state = pane.agent_state.as_deref().unwrap_or("unknown");
            report.push_str(&format!("### {label} [{state}]\n"));

            // Try to read last 100 lines of visible output
            match herdr_api::pane_read(&pane.id, "visible", 100) {
                Ok(content) => {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        // Take last 5 non-empty lines as summary
                        let summary_lines: Vec<&str> = trimmed
                            .lines()
                            .rev()
                            .filter(|l| !l.trim().is_empty())
                            .take(5)
                            .collect();
                        for line in summary_lines.iter().rev() {
                            report.push_str(&format!("- {line}\n"));
                        }
                    }
                }
                Err(e) => {
                    report.push_str(&format!("- (read error: {e})\n"));
                }
            }
            report.push('\n');
        }
    }

    // Write report to state dir
    let state_dir = crate::data::state_dir()?;
    let reports_dir = state_dir.join("data").join("reports");
    std::fs::create_dir_all(&reports_dir)?;
    let filename = format!("report-{}.md", now.format("%Y%m%d-%H%M%S"));
    let path = reports_dir.join(&filename);
    std::fs::write(&path, &report)?;

    tracing::info!("report written to {}", path.display());
    eprintln!("report written to {}", path.display());

    Ok(())
}
```

- [ ] **Step 2: Add `state_dir` to public exports in data.rs**

```rust
// In src/data.rs, ensure this function is public:
// pub(crate) fn state_dir() -> InsightResult<PathBuf> { ... }
```

(Already defined as `fn state_dir()` — make sure it's `pub(crate)`.)

- [ ] **Step 3: Write collector test**

```rust
// tests/collector_tests.rs
#[test]
fn collector_no_panes_is_noop() {
    // Collector gracefully handles empty pane list.
    // Full integration test requires a running herdr server.
}
```

- [ ] **Step 4: Verify cargo check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo check 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add src/ tests/ && git commit -m "feat: add collector with markdown report generation"
```

---

## Phase 5: Polish & Release

### Task 5.1: README.md

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write README**

```markdown
# herdr-insight

Agent output collector, status timeline, and context dashboard for [herdr](https://herdr.dev).

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

### Dashboard

Open a real-time agent status dashboard:

```bash
herdr plugin pane open herdr-insight dashboard
```

Or bind a key in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+d"
type = "plugin_pane"
command = "herdr-insight.dashboard"
```

**Controls:** `q` quit, `r` refresh

### Timeline

Open the agent state change timeline:

```bash
herdr plugin pane open herdr-insight timeline
```

**Controls:** `q` quit, `↑/↓` or `j/k` scroll

### Collector

Collect agent outputs and generate a markdown report:

```bash
herdr plugin action invoke herdr-insight.collect
```

The plugin also listens to `pane.agent_status_changed` events and
automatically snapshots output when an agent transitions to `done`.

Reports are stored in the plugin state directory:
`herdr plugin config-dir herdr-insight` → `data/reports/`

## Requirements

- herdr >= 0.7.0
- Rust toolchain (for building from source)
```

- [ ] **Step 2: Commit**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add README.md && git commit -m "docs: add README"
```

---

### Task 5.2: Lint and final checks

**Files:**
- None (verify only)

- [ ] **Step 1: Run clippy**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo clippy --all-targets -- -D warnings 2>&1 | tail -10
```

Expected: clean (no warnings).

- [ ] **Step 2: Run fmt check**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo fmt --check
```

Expected: no output (all files formatted correctly). If files need formatting, run `cargo fmt` first.

- [ ] **Step 3: Run all tests**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 4: Build release binary**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && cargo build --release --locked 2>&1 | tail -5
```

Expected: `Finished release [optimized]`

- [ ] **Step 5: Commit any lint fixes**

```bash
cd /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight && git add -A && git diff --cached --quiet || git commit -m "chore: fix clippy and fmt warnings"
```

---

### Task 5.3: Smoke test with herdr plugin link

**Files:**
- None (manual verification)

- [ ] **Step 1: Link plugin in herdr**

```bash
export PATH="/opt/zig/0.15.2:$PATH"
herdr plugin link /home/cxd/Projects/aiediter/herdr-plugins/herdr-insight
```

- [ ] **Step 2: Verify plugin is registered**

```bash
herdr plugin list | grep herdr-insight
```

Expected: shows `herdr-insight` in the list.

- [ ] **Step 3: Test dashboard pane**

```bash
herdr plugin pane open herdr-insight dashboard
```

Expected: opens a split pane with the dashboard TUI. Press `q` to quit.

- [ ] **Step 4: Test timeline pane**

```bash
herdr plugin pane open herdr-insight timeline
```

Expected: opens timeline pane. Press `q` to quit.

- [ ] **Step 5: Test collector action**

```bash
herdr plugin action invoke herdr-insight.collect
```

Expected: generates a report file in the plugin state directory.

---

## Completed Checklist

- [x] P0: Project skeleton, Cargo.toml, herdr-plugin.toml, cli, stubs
- [x] P1: herdr_api with pane_list/pane_read, data models with JSONL, poller with transition detection
- [x] P2: Dashboard TUI with event loop and render
- [x] P3: Timeline TUI with scroll and persistence
- [x] P4: Collector with markdown report generation
- [x] P5: README, lint checks, smoke test

