# 🦀 Marvis Office — AI Agent Desktop App

> **My AI Virtual Intelligent System** — 用 Rust 构建的桌面 AI 助手，多 Agent 协作，真实操控你的电脑。

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.96+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-2.x-blue?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/React-19-61dafb?logo=react" alt="React">
  <img src="https://img.shields.io/badge/tests-119-green" alt="Tests">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

## 项目愿景

Marvis Office 是一个**真实可用的 AI 桌面应用**。你输入自然语言任务，AI 分析需求、调度多
个专业 Agent 协作、调用系统工具实际执行——打开浏览器、读取文件、查看进程、搜索网页等。
核心理念：**让 AI 真正长出手脚**，安全地操控电脑。

## 项目背景

本项目为 Rust 课程期末大作业，展示 Rust 工程化开发的完整能力：
- Workspace 模块化架构（7 个 crate）
- 所有权/借用、Trait/泛型/生命周期的合理使用
- 异步编程（Tokio）、错误处理（anyhow + thiserror）
- 单元测试覆盖（119 个测试）
- Tauri 桌面应用打包

## 界面预览

```
┌──────────────────────────────────────────────────────────┐
│  🏢 Marvis Office                                        │
│                                                          │
│  ┌──────────────────────────────┬──────────────────────┐ │
│  │        🏢 Office 场景        │  🏢 Marvis Office    │ │
│  │                              │                      │ │
│  │  🪴     🪟      🪟     🕐   │  [输入任务...]       │ │
│  │                              │  [▶ Start] [🎲 Demo] │ │
│  │  🖥️        🖥️        🖥️     │  ● Rust Backend      │ │
│  │  🐼🎯      🦊📂     🐴💻   │                      │ │
│  │  Panda PM  Fox File Horse   │  🤖 AI Response      │ │
│  │                              │  ┌────────────────┐  │ │
│  │  🖥️        🖥️        🖥️     │  │ 真实的AI回复... │  │ │
│  │  🐰📱      🐶🌐     🐷🔍   │  └────────────────┘  │ │
│  │  Rabbit    Dog      Pig     │                      │ │
│  │                              │  📊 Token Bar        │ │
│  │  ☕ Coffee  🏋️ Gym  🚻 Rest │  📝 Activity Log     │ │
│  └──────────────────────────────┴──────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

## 系统架构

```
marvis-tauri.exe (Tauri 桌面应用)
├── src-tauri/ (Rust 后端)
│   ├── DeepSeekClient  ←── DeepSeek API (真 AI)
│   ├── ToolRegistry    ←── 18 个系统工具
│   ├── AgentLoop       ←── AI ↔ 工具协调
│   └── SecurityManager ←── 权限检查
│
├── src/ (React 前端)
│   ├── Office.tsx      ←── 办公场景 + 动画
│   ├── AgentAvatar.tsx ←── 宠物头像组件
│   ├── TokenBar.tsx    ←── Token 消耗显示
│   └── TaskLog.tsx     ←── 任务日志
│
└── crates/ (7 个 Rust 库)
    ├── marvis-core     ←── 核心类型与 Trait
    ├── marvis-ai       ←── AI 客户端 (DeepSeek/Qwen/Mock)
    ├── marvis-tools    ←── 工具系统 (18 个工具)
    ├── marvis-agent    ←── Agent 循环
    ├── marvis-session  ←── 会话管理
    ├── marvis-security ←── 安全与权限
    └── marvis-cli      ←── 命令行入口
```

## 数据流

```
用户输入自然语言
  │
  ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  React 前端   │───▶│  Tauri IPC   │───▶│  Rust 后端    │
│  (轮询状态)   │◀───│  (invoke)    │◀───│  (AgentLoop)  │
└──────────────┘    └──────────────┘    └──────┬───────┘
                                               │
                          ┌────────────────────┼────────────────────┐
                          ▼                    ▼                    ▼
                   ┌──────────┐        ┌──────────┐        ┌──────────┐
                   │ DeepSeek │        │  Tool     │        │  Windows │
                   │   API    │        │ Registry  │        │  Shell   │
                   │ (真 AI)  │        │ (18工具)  │        │ (cmd/C)  │
                   └──────────┘        └──────────┘        └──────────┘
```

## 快速开始

### 环境要求

| 工具 | 版本要求 | 说明 |
|------|---------|------|
| Rust | 1.75+（推荐 1.96+）| `rustup` 安装 |
| Node.js | 18+ | 前端构建 |
| npm | 9+ | 包管理 |
| Windows | 10/11 | 桌面应用平台 |

### 编译运行（桌面 App）

```bash
# 1. 克隆项目
git clone <repo-url>
cd rust-final

# 2. 安装前端依赖
cd marvis-app
npm install

# 3. 配置 API Key
# 编辑 marvis-app/src-tauri/src/commands.rs 中的 DEEPSEEK_KEY
# 或者在项目根目录创建 .env 文件

# 4. 构建桌面应用
npx tauri build

# 5. 运行
./target/x86_64-pc-windows-gnu/release/marvis-tauri.exe
```

### 编译运行（CLI 工具）

```bash
# 构建 CLI 版本
cargo build --release -p marvis-cli

# 使用 Mock AI（无需 API Key）
cargo run --release -p marvis-cli -- repl

# 使用 DeepSeek 真实 AI
cargo run --release -p marvis-cli -- -P deepseek -m deepseek-v4-pro repl

# 单次执行
cargo run --release -p marvis-cli -- run "查看系统信息"
```

### 配置 AI 提供商

在 `marvis-app/src-tauri/src/commands.rs` 中修改：

```rust
const DEEPSEEK_KEY: &str = "your-deepseek-api-key";
const DEEPSEEK_MODEL: &str = "deepseek-v4-pro";
```

支持任意 OpenAI 兼容 API（DeepSeek、Qwen、OpenAI 等）。

## 可用工具（18 个）

| 类别 | 工具名 | 功能 | 风险级别 |
|------|--------|------|---------|
| 📂 文件 | `read_file` | 读取文件内容 | ReadOnly |
| | `write_file` | 写入文件 | Normal |
| | `list_directory` | 列出目录内容 | ReadOnly |
| | `delete_file` | 删除文件 | ⚠️ Dangerous |
| | `file_info` | 文件元数据 | ReadOnly |
| 💻 系统 | `system_info` | 系统概览 | ReadOnly |
| | `cpu_info` | CPU 详情 | ReadOnly |
| | `memory_info` | 内存使用 | ReadOnly |
| | `list_processes` | 进程列表 | ReadOnly |
| | `process_info` | 进程详情 | ReadOnly |
| | `env_variable` | 读取环境变量 | ReadOnly |
| 🌐 网络 | `web_fetch` | 获取网页内容 | ReadOnly |
| | `web_search` | 搜索引擎 | ReadOnly |
| | `open_browser` | 打开 URL/搜索 | Normal |
| 🖥️ 终端 | `run_shell` | 执行命令* | Normal |
| 📋 剪贴板 | `read_clipboard` | 读取剪贴板 | ReadOnly |
| | `write_clipboard` | 写入剪贴板 | Normal |

*\*run_shell 仅允许安全命令（echo, dir, type, mkdir, copy, move, del），浏览器命令自动转由 open_browser 处理*

## 多 Agent 系统

| Agent | 角色 | 负责工具 |
|-------|------|---------|
| 🐼 **Panda PM** | 任务规划 & 协调 | AI 推理、任务分解、结果汇总 |
| 🦊 **Fox File Ops** | 文件专家 | read_file, write_file, list_directory, delete_file, file_info |
| 🐴 **Horse SysInfo** | 系统监控 | system_info, cpu_info, memory_info, list_processes, process_info |
| 🐶 **Dog Browser** | 网络交互 | web_fetch, web_search, open_browser |
| 🐷 **Pig Search** | 搜索策略 | web_search, 搜索词提取 |
| 🐰 **Rabbit Apps** | 应用管理 | run_shell, read_clipboard, write_clipboard |

## Rust 核心特性体现

| 特性 | 体现位置 | 说明 |
|------|---------|------|
| **Ownership / Borrowing** | 全局 | `&dyn AiClient`, `Arc<dyn Tool>`, 借用传递 |
| **Struct / Enum** | marvis-core | `Message`, `AiResponse`, `RiskLevel`, `MarvisError` |
| **Trait** | marvis-core/ai | `Tool` trait, `AiClient` trait, 动态分发 |
| **泛型** | marvis-tools | `impl Tool + 'static` |
| **生命周期** | marvis-session | `HistoryIterator<'a>` |
| **Result / ?** | 全局 | `anyhow` + `thiserror`, 零 `unwrap()`（测试除外） |
| **async/await** | marvis-agent/tools | Tokio, `#[async_trait]` |
| **Workspace** | 根 Cargo.toml | 7 crate + Tauri app |

## 测试

```bash
# 运行全部测试（119 个）
cargo test --all -- --test-threads=1

# 按 crate 测试
cargo test -p marvis-core    # 25 测试：核心类型、错误处理、Trait
cargo test -p marvis-tools   # 39 测试：全部 18 个工具
cargo test -p marvis-agent   # 15 测试：Agent 循环、编排器、计划器
cargo test -p marvis-ai      #  5 测试：Mock 客户端、流式响应
cargo test -p marvis-session # 19 测试：历史、上下文、存储
cargo test -p marvis-security # 16 测试：权限、沙箱

# 单线程运行（避免 sysinfo 并发问题）
cargo test --all -- --test-threads=1
```

## 代码质量

```bash
# 格式化
cargo fmt --all

# Lint
cargo clippy --all-targets

# 文档
cargo doc --no-deps --open
```

## 错误处理

项目严格遵循 Rust 错误处理最佳实践：

- ❌ **禁止** `unwrap()` / `expect()`（仅测试允许）
- ✅ `Result<T, MarvisError>` 统一错误类型
- ✅ `?` 操作符传播错误
- ✅ `anyhow::Context` 添加上下文信息
- ✅ 工具执行失败返回 `ToolResult { is_error: true }`，不崩溃

## 项目结构

```
rust-final/
├── Cargo.toml                    # Workspace 根
├── README.md                     # 本文件
├── config.toml                   # 默认配置
├── .env.example                  # 环境变量示例
├── crates/                       # Rust 库
│   ├── marvis-core/              # 核心类型、Trait、错误
│   ├── marvis-ai/                # AI 客户端 (DeepSeek/Qwen/Mock)
│   ├── marvis-tools/             # 18 个系统工具
│   ├── marvis-agent/             # Agent 循环、编排器
│   ├── marvis-session/           # 会话历史、上下文管理
│   ├── marvis-security/          # 权限、沙箱
│   └── marvis-cli/               # 命令行入口
└── marvis-app/                   # Tauri 桌面应用
    ├── src-tauri/                # Rust 后端
    │   ├── Cargo.toml
    │   ├── tauri.conf.json
    │   ├── capabilities/
    │   └── src/
    │       ├── main.rs           # 桌面入口
    │       ├── lib.rs            # Tauri Builder
    │       └── commands.rs       # AI + 工具调度
    ├── src/                      # React 前端
    │   ├── App.tsx               # 主组件 + 轮询逻辑
    │   ├── components/
    │   │   ├── Office.tsx        # 办公场景
    │   │   ├── AgentAvatar.tsx   # 宠物头像
    │   │   ├── TokenBar.tsx      # Token 显示
    │   │   └── TaskLog.tsx       # 任务日志
    │   ├── hooks/
    │   │   └── useAgentEvents.ts # 事件监听
    │   └── data/
    │       └── agents.ts         # Agent 定义
    ├── index.html
    ├── package.json
    └── vite.config.ts
```

## 技术栈

| 层面 | 技术 | 用途 |
|------|------|------|
| 🦀 **后端语言** | Rust 2021 Edition | 全部核心逻辑 |
| ⚡ **异步运行时** | Tokio 1.x | 异步 I/O、并发 |
| 🖥️ **桌面框架** | Tauri 2.x | 原生窗口、系统集成 |
| 🎨 **前端框架** | React 19 + TypeScript | 图形界面 |
| 🏗️ **前端构建** | Vite 6 | 打包 |
| 🤖 **AI API** | DeepSeek (OpenAI 兼容) | 智能推理 |
| 📡 **HTTP** | reqwest 0.12 | API 请求 |
| 🔧 **系统信息** | sysinfo 0.32 | CPU/内存/进程 |
| 🕸️ **网页解析** | scraper 0.20 | HTML 提取 |
| 📋 **CLI** | clap 4.x | 命令行解析 |
| 📝 **日志** | env_logger 0.11 | 日志 |
| ✅ **测试** | cargo test | 119 个单元测试 |

## 依赖说明

```toml
# 核心依赖
tokio = { version = "1", features = ["full"] }    # 异步运行时
clap = { version = "4", features = ["derive"] }    # CLI
serde = { version = "1", features = ["derive"] }   # 序列化
serde_json = "1"                                    # JSON
reqwest = { version = "0.12" }                     # HTTP 客户端
sysinfo = "0.32"                                    # 系统信息
scraper = "0.20"                                    # HTML 解析
async-trait = "0.1"                                 # 异步 Trait
anyhow = "1"                                        # 错误处理
thiserror = "2"                                     # 自定义错误

# Tauri 桌面
tauri = { version = "2" }                          # 桌面框架
tauri-build = { version = "2" }                    # 构建工具
tauri-plugin-shell = "2"                           # Shell 插件

# 前端
react = "^19.0.0"                                   # UI 框架
@tauri-apps/api = "^2.2.0"                         # Tauri 前端 API
vite = "^6.0.0"                                     # 构建工具
typescript = "^5.6.0"                               # 类型检查
```

## License

MIT License

## 参考

- [claude-code-rust](https://github.com/lorryjovens-hub/claude-code-rust) — Rust 版 Claude Code 架构参考
- [marvis-office](https://github.com/Drok1015/marvis-office.git) — 多 Agent 可视化概念参考
