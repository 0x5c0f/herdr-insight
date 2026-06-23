# herdr-insight

[English](README.md) | [中文](README_zh-CN.md)

[herdr](https://herdr.dev) 的 Agent 状态时间线面板。

实时可视化所有工作区中的 Agent 任务。追踪 Agent 何时开始工作、被阻塞或进入空闲状态。

## 功能特性

- 实时 Agent 任务时间线（working/blocked 状态）
- 任务聚合 — 每个 Agent 一条记录，自动去重
- 可配置列（显示/隐藏任意列）
- 会话 ID 追踪
- 横向分屏（底部）和纵向分屏（右侧）模式
- 7 天时间线数据保留
- 多工作区 Agent 追踪

## 安装

### 从 GitHub 安装（推荐）

```bash
herdr plugin install 0x5c0f/herdr-insight --yes
```

### 使用 Makefile 从 GitHub 安装

```bash
git clone https://github.com/0x5c0f/herdr-insight.git
cd herdr-insight
make install
```

### 本地开发

```bash
git clone https://github.com/0x5c0f/herdr-insight.git
cd herdr-insight
make link
```

### 预编译二进制

```bash
# 下载最新版本
make download

# 或从 GitHub 安装（自动检测：cargo 编译或下载）
make install
```

## 使用方法

### 快捷键绑定

在 `~/.config/herdr/config.toml` 中添加：

```toml
# 横向分屏（底部）
[[keys.command]]
key = "prefix+t"
type = "plugin_action"
command = "herdr-insight.open-timeline-down"

# 纵向分屏（右侧）
[[keys.command]]
key = "prefix+shift+t"
type = "plugin_action"
command = "herdr-insight.open-timeline-right"
```

### CLI 命令

```bash
# 打开时间线（默认分屏方向）
herdr plugin pane open --plugin herdr-insight --entrypoint timeline

# 指定方向打开
herdr plugin pane open --plugin herdr-insight --entrypoint timeline --direction down
herdr plugin pane open --plugin herdr-insight --entrypoint timeline --direction right

# 直接调用 action
herdr plugin action invoke herdr-insight.open-timeline-down
herdr plugin action invoke herdr-insight.open-timeline-right
```

### 操作说明

| 按键 | 功能 |
|------|------|
| `q` / `Esc` | 关闭时间线 |
| `↑` / `k` | 向上滚动（更新的任务） |
| `↓` / `j` | 向下滚动（更早的任务） |

## 配置

在 `~/.config/herdr/plugins/config/herdr-insight/config.toml` 创建配置文件：

```bash
make config-init
```

### 列可见性

```toml
[columns]
time = true        # 时间戳
state = true       # 状态图标（● 工作中 / ▲ 被阻塞）
agent = true       # Agent 名称
pane = true        # 窗格 ID [wX:pY]
status = true      # 状态文本（working/blocked）
duration = true    # 当前任务持续时间
session = true     # 完整会话 ID
```

### 默认值

| 列 | 默认值 | 说明 |
|----|--------|------|
| `time` | `true` | 显示时间戳 |
| `state` | `true` | 显示状态图标 |
| `agent` | `true` | 显示 Agent 名称 |
| `pane` | `true` | 显示窗格 ID |
| `status` | `true` | 显示状态文本 |
| `duration` | `true` | 显示当前任务持续时间 |
| `session` | `true` | 显示完整会话 ID |

## 从源码构建

### 环境要求

- Rust 工具链（cargo）
- herdr >= 0.7.0

### 构建命令

```bash
make build          # 构建 release 二进制（cargo 或下载）
make build-static   # 构建静态链接二进制（musl，仅 Linux）
make release        # 构建发布包
make release-upload # 上传到 GitHub Releases
```

### Makefile 目标

```
make help           # 显示所有可用目标
make build          # 构建 release 二进制
make download       # 下载预编译二进制
make build-static   # 构建静态链接二进制（musl）
make release        # 构建发布包
make release-upload # 上传到 GitHub Releases
make link           # 构建并链接到 herdr（本地开发）
make unlink         # 从 herdr 取消链接
make install        # 从 GitHub 安装
make uninstall      # 卸载插件
make update         # 从 GitHub 重新安装
make test           # 运行测试
make lint           # 运行 rustfmt + clippy
make check          # 运行 lint + test
make clean          # 清理构建产物
make open           # 打开时间线面板
make info           # 显示插件注册信息
make config         # 显示配置目录路径
make state          # 显示状态目录路径
make logs           # 显示插件日志
make config-init    # 创建默认配置文件
```

## 架构

多 crate Rust 工作区：

```
herdr-insight/
├── crates/
│   ├── common/     # 共享类型（AgentState, AgentSnapshot 等）
│   ├── domain/     # 业务逻辑（轮询器、状态转移）
│   ├── infra/      # 基础设施（herdr CLI、持久化）
│   ├── tui/        # 时间线 TUI（渲染、状态、配置）
│   └── app/        # 二进制入口点
└── herdr-plugin.toml
```

## 许可证

MIT
