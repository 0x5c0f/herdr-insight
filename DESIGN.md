# herdr-insight — 设计规格

## 概述

`herdr-insight` 是一个 herdr 插件，提供 Agent 状态时间线面板。

单 Rust 二进制，Pane-as-daemon 模式 — 无独立后台进程。

## Rust 技术栈

| 组件 | Crate | 用途 |
|------|-------|------|
| CLI 参数 | `clap` 4.x | 子命令分发（`crates/app/src/cli.rs`） |
| TUI 渲染 | `ratatui` 0.30 + `crossterm` 0.29 | timeline 面板 |
| 序列化 | `serde` + `serde_json` | JSON API 响应、数据持久化 |
| 错误处理 | `thiserror` | 统一错误类型 |
| 日志 | `tracing` | 插件日志（herdr 捕获） |

## 工作区结构（多 crate）

```
herdr-insight/
├── Cargo.toml              # [workspace] 根
├── herdr-plugin.toml
├── crates/
│   ├── common/             # 共享类型：AgentState, AgentSnapshot, StateTransition, InsightError
│   ├── infra/              # 基础设施：herdr_client (list_all_panes), persistence (JSONL 读写)
│   ├── domain/             # 业务逻辑：poller (poll_snapshots, detect_transitions)
│   ├── tui/                # TUI 面板：timeline (mod, state, render)
│   └── app/                # 二进制入口：main.rs
└── target/release/herdr-insight
```

**依赖方向：** `app → tui → {domain, infra} → common`。domain 仅依赖 common，不依赖 infra。tui 作为组合根协调 domain（业务逻辑）和 infra（数据源、持久化）。

## herdr-plugin.toml 清单

```toml
id = "herdr-insight"
name = "Herdr Insight"
version = "0.1.0"
min_herdr_version = "0.7.0"
description = "Agent status timeline pane for herdr"
platforms = ["linux", "macos"]

[[build]]
command = ["cargo", "build", "--release", "--locked"]

[[panes]]
id = "timeline"
title = "Agent Timeline"
placement = "split"
command = ["./target/release/herdr-insight", "timeline"]
```

## Timeline 面板

**ratatui TUI 面板**，在插件 pane 中渲染。轮询间隔 2 秒。

**数据来源：** `herdr pane list` → JSON 解析 → 状态转移检测 → JSONL 持久化

**布局：**
```
┌─ Herdr Insight · Timeline ────────────────────────┐
│                                                    │
│  16:20 ● claude         working ──── 23min         │
│  16:18 ▲ codex          blocked ──  5min          │
│  16:15 ○ claude         done                      │
│                                                    │
│  q quit  ↑↓ scroll                                │
```

**键位：** `q`/`Esc` 退出，`↑/↓` 或 `j/k` 滚动

**数据持久化：** JSONL 格式，`$HERDR_PLUGIN_STATE_DIR/data/timeline.jsonl`，7 天滚动清理。

## 错误处理

- 无 `unwrap()`/`expect()`，全部使用 `InsightResult`
- herdr CLI 不可用时 graceful fallback
- JSON 解析失败时跳过损坏条目

## 测试

| 层 | 文件 | 内容 |
|-----|------|------|
| 单元 | `crates/domain/tests/` | 状态转移检测 |
| 单元 | `crates/infra/tests/` | JSONL 读写、滚动清理 |
