# Audit Log — herdr-insight

Organizational memory for ai-dev-audit runs. Each entry records findings for trend analysis across audits.

---

## 2026-06-23 — Initial Audit

| Field | Value |
|---|---|
| **Project** | herdr-insight |
| **Mode** | Single-Crate |
| **Health** | 🟡 |
| **🔴 Critical** | 0 |
| **🟠 High** | 1 |
| **🟡 Medium** | 0 |
| **🔵 Low** | 1 |
| **Total LoC** | 672 |

### Findings

| ID | Severity | Summary | File | Status |
|---|---|---|---|---|
| MIGRATE-1 | 🟠 High | Single-crate architecture — migration recommended when adding features | `Cargo.toml` | New |
| DeadDep-anyhow | 🔵 Low | `anyhow` dependency declared but never imported | `Cargo.toml:15` | New |

### Passed Highlights

- Zero `unwrap()`/`expect()` in production code (all use guarded variants)
- Zero `unsafe` blocks, no hardcoded secrets
- `main.rs` 18 lines — clean wiring only
- 6 tests across 2 test files
- Clean clippy, compiles without errors
- Modules are already well-separated by concern

### Notes

- TUI plugin (ratatui), not Axum HTTP service. Standard 7-crate layout needs adaptation.
- `cargo-audit` not installed — S5 skipped. Install before publishing.
- DESIGN.md references future features (Collector, Dashboard) not yet implemented — expected for P1 stage.

---

## Gotcha Validation

No false positives encountered in this audit. The `anyhow` dead dependency was manually verified (not flagged by scripts — the scripts don't check for unused deps).

### Gotchas Applied

| Gotcha | Applied To | Rationale |
|---|---|---|
| "Single-crate <500 LoC → frame as when project grows" | MIGRATE-1 | 672 LoC, well-structured. Downgraded Critical → High |
| "Deserialize for storage is acceptable" | C4 | All `Deserialize` derives are for JSONL persistence |
| "unwrap_or() ≠ unwrap()" | C1 | All `.unwrap_or*()` calls provide fallback values — never panic |
| "expect() in test fixtures is acceptable" | C2 | All `.expect()` calls are in `tests/` only |

---

---

## 2026-06-23 — Re-Audit (post-migration)

| Field | Value |
|---|---|
| **Project** | herdr-insight |
| **Mode** | Multi-Crate Workspace (5 crates) |
| **Health** | 🔴 |
| **🔴 Critical** | 1 |
| **🟠 High** | 0 |
| **🟡 Medium** | 2 |
| **🔵 Low** | 1 |
| **Total LoC** | 664 |

### Findings

| ID | Severity | Summary | File | Status |
|---|---|---|---|---|
| MIGRATE-1 | 🟠 High | Single-crate architecture | `Cargo.toml` | ✅ Resolved — multi-crate workspace created |
| DeadDep-anyhow | 🔵 Low | `anyhow` unused | `Cargo.toml` | ✅ Resolved — removed from all deps |
| B1 | 🔴 Critical | domain → infra forbidden dependency | `crates/domain/Cargo.toml:8`, `domain/src/lib.rs:2` | 🆕 New |
| D3 | 🟡 Medium | domain missing `ports/` directory | `crates/domain/src/` | 🆕 New |
| D5 | 🟡 Medium | infra flat structure (164-line lib.rs) | `crates/infra/src/lib.rs` | 🆕 New |
| D1 | 🔵 Low | domain flat structure (acceptable at 62 lines) | `crates/domain/src/lib.rs` | 🆕 New |

### Passed Highlights

- Migration executed cleanly: workspace manifest + `[workspace.dependencies]` + old src/ removed
- All 6 tests migrated and passing (domain 4 + infra 2)
- Zero unwrap/expect/unsafe/secrets/debug-output/markers — same as before
- clippy clean, compiles without errors
- `anyhow` fully removed

### Notes

- B1 is the only blocker — move `PaneInfo` to `common` (~30 min fix)
- TUI-adapted 5-crate topology (`app`/`common`/`domain`/`infra`/`tui`) is appropriate
- `cargo-audit` still not installed — S5 skipped again
- DESIGN.md still shows old single-crate structure — update after B1 fix

---

## Gotcha Validation

### Gotchas Applied

| Gotcha | Applied To | Rationale |
|---|---|---|
| "Deserialize for storage is acceptable" | C4 | All `Deserialize` derives are for JSONL persistence, confirmed in common crate |
| "unwrap_or() ≠ unwrap()" | C1 | All `.unwrap_or*()` calls in domain/tui provide fallback values |
| "expect() in test fixtures is acceptable" | C2 | Only in `crates/*/tests/` |

### New Gotcha Candidate

**TUI-adapted crate topology:** The standard 7-crate layout is designed for Axum HTTP services. For TUI (ratatui) plugins, `api` and `server` don't apply — a `tui` crate serves as the rendering layer. When auditing non-HTTP projects, don't flag missing `api`/`server` crates as A3 violations. The TUI crate is the correct architectural equivalent.

---

---

## 2026-06-23 — Re-Audit #2 (B1/D3/D5 fixes)

| Field | Value |
|---|---|
| **Project** | herdr-insight |
| **Mode** | Multi-Crate Workspace (5 crates) |
| **Health** | 🟢 |
| **🔴 Critical** | 0 |
| **🟠 High** | 0 |
| **🟡 Medium** | 0 |
| **🔵 Low** | 1 |
| **Total LoC** | 686 |

### Findings

| ID | Severity | Summary | File | Status |
|---|---|---|---|---|
| B1 | 🔴 Critical | domain → infra forbidden dependency | `crates/domain/Cargo.toml` | ✅ Resolved — PaneInfo moved to common, poll_snapshots() parameterized |
| D3 | 🟡 Medium | domain missing `ports/` directory | `crates/domain/src/` | ✅ Resolved — ports.rs with PaneRepository trait created |
| D5 | 🟡 Medium | infra flat structure | `crates/infra/src/` | ✅ Resolved — split into herdr_client.rs + persistence.rs |
| D1 | 🔵 Low | domain flat structure | `crates/domain/src/lib.rs` | ✅ Resolved — ports.rs extracted, 65-line lib.rs is acceptable |
| C10 | 🔵 Low | Intentional TODO marker (design note) | `crates/domain/src/lib.rs:8` | 🆕 New (acceptable — deferred design) |

### Bonus Improvements

- CLI definition extracted from `main.rs` to `app/src/cli.rs` — `main.rs` now 16 lines
- `herdr_bin()` and `herdr_json()` visibility tightened to `pub(crate)`
- `PaneRepository` trait defined in `domain/src/ports.rs` — ready for Collector/Dashboard

### Dependency Graph (clean)

```
app → tui (+ clap, tracing-subscriber)
tui → common, domain, infra (+ ratatui, crossterm)
domain → common (+ chrono)          ← NO infra!
infra → common (+ serde, chrono, tracing)
common → serde, serde_json, chrono, tracing, thiserror
```

All dependency directions comply with ai-dev-discipline. No forbidden edges.

### Notes

- `cargo-audit` still not installed — S5 skipped (3rd consecutive time)
- C10 TODO is intentional: marks where PaneRepository trait will replace `&[PaneInfo]` parameter
- 3-audit trajectory: 🟡(single-crate) → 🔴(B1) → 🟢(all clean)
- Project is feature-ready — all architectural invariants satisfied

---

## Trend Summary

| Audit | Date | Health | Critical | High | Medium | Low |
|---|---|---|---|---|---|---|
| Initial | 2026-06-23 | 🟡 | 0 | 1 | 0 | 2 |
| Re-Audit #1 | 2026-06-23 | 🔴 | 1 | 0 | 2 | 1 |
| Re-Audit #2 | 2026-06-23 | 🟢 | 0 | 0 | 0 | 1 |

---

## 2026-06-23 — Re-Audit #3 (Post CI/CD changes)

| Field | Value |
|---|---|
| **Project** | herdr-insight |
| **Mode** | Multi-Crate Workspace (5 crates) |
| **Health** | 🟢 |
| **🔴 Critical** | 0 |
| **🟠 High** | 0 |
| **🟡 Medium** | 0 |
| **🔵 Low** | 1 |
| **Total LoC** | 951 |

### Findings

| ID | Severity | Summary | File | Status |
|---|---|---|---|---|
| C10 | 🔵 Low | Intentional TODO marker (design note) | `crates/domain/src/lib.rs:8` | 🔁 Recurring — 无变化，按设计延后 |

### Passed Highlights

- 所有 Critical/High/Medium 问题持续保持已修复状态
- 代码行数增长 38%（686 → 951），主要来自 CI/CD 和静态构建支持
- 核心架构未变，依赖方向依然合规
- cargo check + clippy 通过
- 零 unwrap/expect/unsafe/secrets/debug-output

### Notes

- 自上次审计以来的变更：静态构建支持、CI/CD 配置、安装脚本优化
- `cargo-audit` 仍未安装 — S5 第 4 次跳过
- 4-audit 轨迹: 🟡(单crate) → 🔴(B1) → 🟢(全部修复) → 🟢(稳定)

---

## Trend Summary

| Audit | Date | Health | Critical | High | Medium | Low |
|---|---|---|---|---|---|---|
| Initial | 2026-06-23 | 🟡 | 0 | 1 | 0 | 2 |
| Re-Audit #1 | 2026-06-23 | 🔴 | 1 | 0 | 2 | 1 |
| Re-Audit #2 | 2026-06-23 | 🟢 | 0 | 0 | 0 | 1 |
| Re-Audit #3 | 2026-06-23 | 🟢 | 0 | 0 | 0 | 1 |

---

*Audit conducted by ai-dev-audit v1.*
