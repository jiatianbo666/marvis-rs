# Marvis — Rust CLI AI Assistant 项目任务书

> **项目愿景**：构建一个用 Rust 编写的命令行 AI 助手，能够理解用户自然语言指令，通过调用系统工具（文件操作、进程管理、网页交互等）来实际"操控"电脑。核心思想是"让 AI 长出手脚"，安全地执行操作系统级别的操作。
>
> **项目名称**：Marvis = **M**y **A**I **V**irtual **I**ntelligent **S**ystem
>
> **参考仓库**：
> - [claude-code-rust](https://github.com/lorryjovens-hub/claude-code-rust) — Rust 重写的 Claude Code CLI，参考其架构设计、工具系统、API 抽象层
> - [marvis-office](https://github.com/Drok1015/marvis-office.git) — 多 Agent 虚拟办公室前端，参考其多 Agent 协作模式、SSE 事件系统、确认流程

---

## 目录

1. [项目架构总览](#1-项目架构总览)
2. [模块详细设计](#2-模块详细设计)
3. [开发阶段与任务分解](#3-开发阶段与任务分解)
4. [测试计划](#4-测试计划)
5. [质量保证流程](#5-质量保证流程)
6. [验收标准](#6-验收标准)

---

## 1. 项目架构总览

### 1.1 技术栈

| 层面 | 技术选择 | 说明 |
|------|---------|------|
| 语言 | Rust 2021 Edition | 稳定版 1.75+ |
| 异步运行时 | Tokio 1.x (features: `full`) | 异步 I/O、并发任务 |
| CLI 框架 | clap 4.x (features: `derive`) | 命令行参数解析 |
| TUI 框架 | ratatui 0.28 + crossterm 0.28 | 终端交互界面 |
| HTTP 客户端 | reqwest 0.12 (features: `json`, `stream`) | AI API 调用、网页抓取 |
| 序列化 | serde 1.x + serde_json 1.x | JSON 序列化 |
| 配置管理 | config 0.14 + toml 0.8 | 分层配置 |
| 错误处理 | anyhow 1.x + thiserror 2.x | 错误传播与定义 |
| 异步 trait | async-trait 0.1 | trait 中定义异步方法 |
| 终端处理 | crossterm 0.28 | 跨平台终端控制 |
| 网页抓取 | scraper 0.20 | HTML 解析 |
| 剪贴板 | arboard 3.x | 剪贴板访问 |
| 日志 | log 0.4 + env_logger 0.11 | 日志记录 |

### 1.2 Cargo 项目结构 (Workspace)

```
rust-final/
├── Cargo.toml                    # workspace root
├── README.md                     # 项目文档
├── task.md                       # 本任务书
├── config.toml                   # 默认配置文件
├── .env.example                  # 环境变量示例
├── crates/
│   ├── marvis-core/              # 核心库：类型定义、trait、错误类型
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # 通用类型 (Message, ToolCall, ToolResult)
│   │       ├── error.rs          # 错误类型定义
│   │       └── tool.rs           # Tool trait 定义
│   │
│   ├── marvis-ai/                # AI 提供商抽象层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs         # AiClient trait
│   │       ├── anthropic.rs      # Anthropic (Claude) 实现
│   │       ├── openai.rs         # OpenAI (GPT) 实现
│   │       ├── mock.rs           # Mock 实现 (测试用)
│   │       └── types.rs          # API 请求/响应类型
│   │
│   ├── marvis-tools/             # 工具系统
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs       # 工具注册中心
│   │       ├── file.rs           # 文件操作工具
│   │       ├── process.rs        # 进程管理工具
│   │       ├── web.rs            # 网页交互工具
│   │       ├── system.rs         # 系统信息工具
│   │       └── clipboard.rs      # 剪贴板工具
│   │
│   ├── marvis-agent/             # Agent 循环与任务编排
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loop.rs           # 主循环 (Agent Loop)
│   │       ├── orchestrator.rs   # 任务分解与编排
│   │       └── planner.rs        # 计划生成
│   │
│   ├── marvis-session/           # 会话管理
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── history.rs        # 对话历史
│   │       ├── context.rs        # 上下文窗口管理
│   │       └── storage.rs        # 持久化存储
│   │
│   ├── marvis-security/          # 安全与权限
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── permissions.rs    # 权限模型
│   │       └── sandbox.rs        # 沙箱隔离
│   │
│   └── marvis-cli/               # CLI 入口与 TUI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # 程序入口
│           ├── args.rs           # clap 参数定义
│           ├── repl.rs           # REPL 交互循环
│           ├── tui.rs            # 终端 UI (ratatui)
│           ├── commands.rs       # 内置命令
│           └── display.rs        # 输出格式化
│
└── tests/                        # 集成测试
    ├── integration_test.rs
    ├── fixtures/                 # 测试用固定数据
    └── common/                   # 测试辅助模块
        └── mod.rs
```

### 1.3 数据流图

```
用户输入 (自然语言)
    │
    ▼
┌──────────────────────────────────────────────────────┐
│                   marvis-cli (REPL/TUI)               │
│   - 接收用户输入                                       │
│   - 显示流式响应                                       │
│   - 权限确认交互                                       │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│                  marvis-agent (Agent Loop)            │
│   1. 构建消息上下文 (system prompt + history + user)    │
│   2. 调用 AI API                                      │
│   3. 解析响应 (文本 / 工具调用)                         │
│   4. 如为工具调用 → 安全检查 → 执行 → 收集结果 → 回到1  │
│   5. 如为文本 → 输出给用户                              │
└──────┬──────────────────────┬────────────────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐    ┌────────────────────┐
│  marvis-ai   │    │   marvis-tools     │
│  - AI 调用   │    │   - 工具注册中心    │
│  - 流式解析  │    │   - 文件操作       │
│  - 多提供商  │    │   - 进程管理       │
└──────────────┘    │   - 网页交互       │
                    │   - 系统信息       │
                    └─────────┬──────────┘
                              │
                              ▼
                    ┌────────────────────┐
                    │ marvis-security    │
                    │ - 权限检查         │
                    │ - 危险操作拦截      │
                    │ - 确认流程         │
                    └────────────────────┘
```

### 1.4 Agent Loop 核心流程伪代码

```rust
async fn agent_loop(
    client: &dyn AiClient,
    tools: &ToolRegistry,
    security: &SecurityManager,
    session: &mut Session,
    user_input: &str,
) -> Result<String> {
    // 1. 构建消息
    session.add_user_message(user_input);
    let messages = session.build_messages();

    loop {
        // 2. 调用 AI
        let response = client.chat(&messages, tools.schemas()).await?;

        match response {
            AiResponse::Text(text) => {
                // 3. 纯文本响应 → 返回
                session.add_assistant_message(&text);
                return Ok(text);
            }
            AiResponse::ToolCalls(calls) => {
                // 4. 工具调用 → 逐条处理
                session.add_tool_calls(&calls);

                let mut results = Vec::new();
                for call in &calls {
                    // 4a. 安全检查
                    security.check(&call.name, &call.args)?;

                    // 4b. 危险操作需用户确认
                    if security.needs_confirmation(&call.name, &call.args) {
                        ui.request_confirmation(&call).await?;
                    }

                    // 4c. 执行工具
                    let result = tools.execute(&call.name, &call.args).await;
                    results.push(result);
                }

                // 4d. 将工具结果加入消息
                session.add_tool_results(&results);

                // 4e. 循环回到第2步
            }
        }
    }
}
```

---

## 2. 模块详细设计

### 2.1 marvis-core — 核心类型与 trait

#### 文件：`crates/marvis-core/src/lib.rs`
```rust
// 模块声明
pub mod types;
pub mod error;
pub mod tool;

// 重新导出常用类型
pub use types::*;
pub use error::*;
pub use tool::*;
```

#### 文件：`crates/marvis-core/src/types.rs`
需要定义的类型：
- `Message` — 聊天消息 (role + content)
- `ToolCall` — AI 发起的工具调用 (id + name + arguments)
- `ToolResult` — 工具执行结果 (id + content + is_error)
- `ToolSchema` — 传给 AI 的工具定义 (name + description + parameters JSON Schema)
- `AiResponse` — AI 响应枚举 (Text / ToolCalls)
- `SessionConfig` — 会话配置
- `PermissionLevel` — 权限级别枚举 (ReadOnly / Normal / Dangerous)

#### 文件：`crates/marvis-core/src/error.rs`
使用 `thiserror` 定义错误类型：
- `MarvisError` — 顶层错误枚举
  - `AiError(String)` — AI 调用失败
  - `ToolError { tool: String, message: String }` — 工具执行失败
  - `ConfigError(String)` — 配置错误
  - `PermissionDenied { tool: String, reason: String }` — 权限拒绝
  - `SessionError(String)` — 会话错误
  - `IoError(#[from] std::io::Error)` — I/O 错误
  - `JsonError(#[from] serde_json::Error)` — JSON 错误

#### 文件：`crates/marvis-core/src/tool.rs`
定义 `Tool` trait：
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (AI 调用时匹配)
    fn name(&self) -> &str;

    /// 工具描述 (给 AI 看的)
    fn description(&self) -> &str;

    /// 输入参数 JSON Schema
    fn input_schema(&self) -> serde_json::Value;

    /// 执行工具
    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult>;

    /// 是否需要用户确认
    fn requires_confirmation(&self) -> bool { false }

    /// 风险级别
    fn risk_level(&self) -> RiskLevel { RiskLevel::Normal }
}

pub enum RiskLevel {
    ReadOnly,   // 只读操作，无需确认
    Normal,     // 普通写操作
    Dangerous,  // 危险操作，必须确认
}
```

### 2.2 marvis-ai — AI 提供商

#### 文件：`crates/marvis-ai/src/lib.rs`
```rust
pub mod client;
pub mod types;

#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(any(test, feature = "mock"))]
pub mod mock;
```

#### 文件：`crates/marvis-ai/src/client.rs`
定义 `AiClient` trait：
```rust
#[async_trait]
pub trait AiClient: Send + Sync {
    /// 发送聊天请求，返回流式或非流式响应
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<AiResponse>;

    /// 流式聊天
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolSchema],
    ) -> Result<BoxStream<'static, Result<StreamEvent>>>;
}

pub enum StreamEvent {
    TextDelta(String),
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, args_delta: String },
    ToolCallEnd { id: String },
    Done,
}
```

#### 文件：`crates/marvis-ai/src/anthropic.rs`
- 实现 Anthropic Messages API 对接
- 支持 Claude 系列模型
- 处理 `tool_use` content block 和 stop_reason
- 使用 reqwest 发送 HTTP 请求
- 支持 SSE 流式响应

#### 文件：`crates/marvis-ai/src/openai.rs`
- 实现 OpenAI Chat Completions API 对接
- 支持 GPT 系列模型
- 处理 function calling 格式
- 兼容任何 OpenAI-compatible 的 API (如 DeepSeek)

#### 文件：`crates/marvis-ai/src/mock.rs`
- Mock 实现，用于测试，不依赖真实 API
- 返回预设的响应

### 2.3 marvis-tools — 工具系统

#### 文件：`crates/marvis-tools/src/registry.rs`
- `ToolRegistry` 结构体，管理所有注册的工具
- `register(Box<dyn Tool>)` — 注册工具
- `get(name: &str) -> Option<&dyn Tool>` — 查找工具
- `schemas() -> Vec<ToolSchema>` — 获取所有工具的 schema (给 AI)
- `execute(name, args)` — 执行指定工具
- `list_names() -> Vec<&str>` — 列出所有工具名

#### 文件：`crates/marvis-tools/src/file.rs`
需要实现的文件操作工具：

| 工具名 | 功能 | 参数 | 风险级别 |
|--------|------|------|---------|
| `read_file` | 读取文件内容 | `path: String`, `offset?: usize`, `limit?: usize` | ReadOnly |
| `write_file` | 写入文件 | `path: String`, `content: String` | Normal |
| `list_directory` | 列出目录内容 | `path: String`, `recursive?: bool` | ReadOnly |
| `search_files` | 按文件名搜索 | `pattern: String`, `directory?: String` | ReadOnly |
| `delete_file` | 删除文件 | `path: String` | Dangerous |
| `move_file` | 移动/重命名文件 | `from: String`, `to: String` | Normal |
| `copy_file` | 复制文件 | `from: String`, `to: String` | Normal |
| `file_info` | 获取文件元数据 | `path: String` | ReadOnly |

#### 文件：`crates/marvis-tools/src/process.rs`
需要实现的进程管理工具：

| 工具名 | 功能 | 参数 | 风险级别 |
|--------|------|------|---------|
| `list_processes` | 列出运行中的进程 | `sort_by?: String`, `limit?: usize` | ReadOnly |
| `process_info` | 获取进程详情 | `pid: u32` | ReadOnly |
| `kill_process` | 终止进程 | `pid: u32`, `force?: bool` | Dangerous |
| `run_command` | 执行命令 | `command: String`, `args?: Vec<String>` | Normal |
| `cpu_info` | 获取 CPU 信息 | — | ReadOnly |
| `memory_info` | 获取内存信息 | — | ReadOnly |
| `disk_info` | 获取磁盘信息 | — | ReadOnly |

Windows 平台使用 `sysinfo` crate 实现进程/系统信息。执行命令通过 `std::process::Command`。

#### 文件：`crates/marvis-tools/src/web.rs`
需要实现的网页交互工具：

| 工具名 | 功能 | 参数 | 风险级别 |
|--------|------|------|---------|
| `web_fetch` | 获取网页内容 | `url: String` | ReadOnly |
| `web_search` | 搜索网页 | `query: String`, `limit?: usize` | ReadOnly |
| `download_file` | 下载文件 | `url: String`, `path: String` | Normal |

网页获取用 `reqwest` + `scraper` 提取文本内容。

#### 文件：`crates/marvis-tools/src/system.rs`
需要实现的系统信息工具：

| 工具名 | 功能 | 参数 | 风险级别 |
|--------|------|------|---------|
| `system_info` | 获取系统概览 | — | ReadOnly |
| `env_variable` | 读取环境变量 | `name: String` | ReadOnly |
| `open_url` | 在浏览器打开URL | `url: String` | Normal |
| `set_reminder` | 设置提醒 | `message: String`, `seconds: u64` | ReadOnly |

#### 文件：`crates/marvis-tools/src/clipboard.rs`
| 工具名 | 功能 | 参数 | 风险级别 |
|--------|------|------|---------|
| `read_clipboard` | 读取剪贴板 | — | ReadOnly |
| `write_clipboard` | 写入剪贴板 | `text: String` | Normal |

### 2.4 marvis-agent — Agent 循环

#### 文件：`crates/marvis-agent/src/loop.rs`
- `AgentLoop` 结构体，持有 `AiClient`、`ToolRegistry`、`SecurityManager`
- `run(&mut self, session: &mut Session, input: &str) -> Result<String>` — 主循环
- 实现完整的 Agent Loop 逻辑（见 1.4 节）
- 处理多轮工具调用
- 最大循环次数限制 (默认 10 轮，防止死循环)
- 每次循环记录日志

#### 文件：`crates/marvis-agent/src/orchestrator.rs`
- `TaskOrchestrator` — 复杂任务分解
- `decompose_task(input: &str) -> Result<Vec<SubTask>>` — 将用户请求分解为子任务
- `execute_subtasks(subtasks: Vec<SubTask>) -> Result<Vec<SubTaskResult>>` — 顺序或并行执行
- `summarize_results(results: Vec<SubTaskResult>) -> Result<String>` — 汇总结果

#### 文件：`crates/marvis-agent/src/planner.rs`
- `Plan` 结构体 — 执行计划
- `Planner` — 调用 AI 生成执行计划
- `PlanStep` — 单个步骤 (工具名 + 参数)

### 2.5 marvis-session — 会话管理

#### 文件：`crates/marvis-session/src/history.rs`
- `ConversationHistory` — 对话历史管理
- 存储 `Vec<Message>` 
- 支持迭代器遍历
- 支持序列化/反序列化 (JSON)

#### 文件：`crates/marvis-session/src/context.rs`
- `ContextManager` — 上下文窗口管理
- 估算 token 数量 (简单按字符数估算)
- 超过窗口时自动裁剪历史
- 保留 system prompt 和最近 N 条消息

#### 文件：`crates/marvis-session/src/storage.rs`
- 会话持久化到磁盘
- 保存/加载会话
- 列出历史会话

### 2.6 marvis-security — 安全与权限

#### 文件：`crates/marvis-security/src/permissions.rs`
- `PermissionLevel` 枚举 (已在 core 定义)
- `SecurityManager` 结构体
- `check_permission(tool_name, args) -> Result<()>` — 权限检查
- `needs_confirmation(tool_name, args) -> bool` — 是否需要用户确认
- `set_mode(mode: PermissionLevel)` — 设置安全模式
- 敏感关键词检测：`rm`, `delete`, `remove`, `kill`, `format`, `uninstall` 等
- 路径安全检查：禁止操作 `C:\Windows`, `/etc`, `/System` 等系统目录

#### 文件：`crates/marvis-security/src/sandbox.rs`
- `Sandbox` — 基础沙箱
- 限制文件操作范围到当前工作目录
- 限制可执行的命令
- 可选：WASM 沙箱 (后续扩展)

### 2.7 marvis-cli — 命令行界面

#### 文件：`crates/marvis-cli/src/main.rs`
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化日志
    env_logger::init();

    // 2. 解析命令行参数
    let args = Args::parse();

    // 3. 加载配置
    let config = Config::load(args.config.as_deref())?;

    // 4. 初始化各组件
    let ai_client = create_client(&config)?;
    let tool_registry = create_tool_registry()?;
    let security = SecurityManager::new(config.permission_level);
    let session = Session::new();

    // 5. 根据模式启动
    match args.command {
        Some(Command::Run { query }) => {
            // 单次执行模式
            run_single(&ai_client, &tool_registry, &security, &session, &query).await?;
        }
        Some(Command::Repl) | None => {
            // 交互 REPL 模式
            run_repl(&ai_client, &tool_registry, &security, &session).await?;
        }
    }

    Ok(())
}
```

#### 文件：`crates/marvis-cli/src/args.rs`
使用 clap derive API 定义：
```rust
#[derive(Parser)]
#[command(name = "marvis", version, about = "My AI Virtual Intelligent System")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// 配置文件路径
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// AI 模型
    #[arg(short, long, default_value = "claude-sonnet-4-6")]
    pub model: String,

    /// 权限模式
    #[arg(short, long, default_value = "normal")]
    pub permission: String,
}

#[derive(Subcommand)]
pub enum Command {
    /// 直接执行任务
    Run {
        /// 任务描述
        query: String,
    },
    /// 进入交互模式 (默认)
    Repl,
    /// 列出可用工具
    Tools,
    /// 显示配置
    Config,
}
```

#### 文件：`crates/marvis-cli/src/repl.rs`
- 使用 rustyline 或自定义 readline 实现交互循环
- 支持多行输入
- 内置命令 (`/help`, `/tools`, `/config`, `/clear`, `/history`, `/exit`)
- 显示工具调用和结果
- 确认提示

#### 文件：`crates/marvis-cli/src/tui.rs` (可选特性)
- 使用 ratatui 构建终端 UI
- 聊天面板（用户/AI 消息）
- 工具调用状态面板
- 系统信息面板

---

## 3. 开发阶段与任务分解

### 阶段 0：项目初始化 (预计时间：30 分钟)

#### 任务 0.1：创建 Cargo workspace
- [ ] 在 `rust-final/` 下创建根 `Cargo.toml`，定义 workspace members
- [ ] 创建所有 crate 目录结构和初始 `Cargo.toml`
- [ ] 创建 `src/` 目录和空的 `lib.rs` / `main.rs`
- [ ] 添加各 crate 之间的依赖关系

**验证方式**：`cargo check` 在 workspace 根目录下无错误通过

#### 任务 0.2：配置开发环境
- [ ] 创建 `.env.example` 文件
- [ ] 创建 `config.toml` 默认配置
- [ ] 创建 `.gitignore` (忽略 `target/`, `.env`, `*.log` 等)
- [ ] 运行 `cargo fmt` 确认格式化配置
- [ ] 运行 `cargo clippy` 确认 lint 配置

**验证方式**：`cargo fmt --check` 和 `cargo clippy` 无报错

---

### 阶段 1：核心基础设施 (预计时间：1-2 小时)

#### 任务 1.1：实现 marvis-core
- [ ] 实现 `types.rs` — Message, ToolCall, ToolResult, ToolSchema, AiResponse
- [ ] 实现 `error.rs` — MarvisError 枚举
- [ ] 实现 `tool.rs` — Tool trait, RiskLevel 枚举
- [ ] 编写对应单元测试

**验证方式**：
```bash
cargo test -p marvis-core
```

#### 任务 1.2：实现 marvis-session
- [ ] 实现 `history.rs` — ConversationHistory, add/serialize/deserialize
- [ ] 实现 `context.rs` — ContextManager, token 估算, 裁剪
- [ ] 实现 `storage.rs` — SessionStorage, save/load/list
- [ ] 编写单元测试

**验证方式**：
```bash
cargo test -p marvis-session
```

#### 任务 1.3：实现 marvis-security
- [ ] 实现 `permissions.rs` — SecurityManager, 权限检查, 危险操作检测
- [ ] 实现 `sandbox.rs` — 路径限制, 命令白名单
- [ ] 编写单元测试

**验证方式**：
```bash
cargo test -p marvis-security
# 验证危险操作被正确拦截
# 验证安全操作被放行
```

---

### 阶段 2：AI 层与工具层 (预计时间：2-3 小时)

#### 任务 2.1：实现 marvis-ai
- [ ] 实现 `client.rs` — AiClient trait, StreamEvent 枚举
- [ ] 实现 `types.rs` — API 请求/响应类型
- [ ] 实现 `anthropic.rs` — Anthropic Messages API 对接，支持 tool use
- [ ] 实现 `openai.rs` — OpenAI Chat Completions 对接，支持 function calling
- [ ] 实现 `mock.rs` — Mock 客户端（测试用）
- [ ] 编写单元测试

**验证方式**：
```bash
cargo test -p marvis-ai
# Mock 客户端应能正确返回模拟响应
```

#### 任务 2.2：实现 marvis-tools
- [ ] 实现 `registry.rs` — ToolRegistry
- [ ] 实现 `file.rs` — 所有文件操作工具
- [ ] 实现 `process.rs` — 所有进程管理工具
- [ ] 实现 `web.rs` — 网页工具
- [ ] 实现 `system.rs` — 系统信息工具
- [ ] 实现 `clipboard.rs` — 剪贴板工具
- [ ] 为每个工具编写单元测试

**验证方式**：
```bash
cargo test -p marvis-tools
# 测试每个工具的 execute 方法
# 测试 ToolRegistry 的注册、查找、执行功能
```

---

### 阶段 3：Agent 层 (预计时间：1-2 小时)

#### 任务 3.1：实现 marvis-agent
- [ ] 实现 `loop.rs` — AgentLoop, 主循环逻辑
- [ ] 实现 `orchestrator.rs` — TaskOrchestrator, 任务分解
- [ ] 实现 `planner.rs` — Planner, 计划生成
- [ ] 编写单元测试（使用 Mock AI 客户端）

**验证方式**：
```bash
cargo test -p marvis-agent
# 验证 Agent Loop 能正确处理文本响应
# 验证 Agent Loop 能正确处理工具调用
# 验证多轮工具调用
# 验证最大循环次数限制
```

---

### 阶段 4：CLI 界面 (预计时间：1-2 小时)

#### 任务 4.1：实现 marvis-cli
- [ ] 实现 `args.rs` — clap 参数定义
- [ ] 实现 `main.rs` — 程序入口，组件初始化
- [ ] 实现 `repl.rs` — REPL 交互循环
- [ ] 实现 `commands.rs` — 内置命令 (/help, /tools, /clear, /exit)
- [ ] 实现 `display.rs` — 输出格式化 (Markdown 渲染、语法高亮)
- [ ] 实现 `tui.rs` — 可选 TUI 模式 (ratatui)

**验证方式**：
```bash
cargo build -p marvis-cli
# 运行 `marvis --help` 查看帮助
# 运行 `marvis tools` 列出工具
# 启动 REPL 模式
```

---

### 阶段 5：集成与测试 (预计时间：1-2 小时)

#### 任务 5.1：集成测试
- [ ] 编写 `tests/integration_test.rs`
- [ ] 测试完整流程：用户输入 → AI 响应 → 工具调用 (用 Mock)
- [ ] 测试安全拦截流程
- [ ] 测试多轮对话

#### 任务 5.2：文档
- [ ] 编写 `README.md`
- [ ] 完善代码文档注释 (`///` 和 `//!`)

---

### 阶段 6：质量保证 (预计时间：30 分钟)

#### 任务 6.1：代码质量
- [ ] 运行 `cargo fmt` 格式化所有代码
- [ ] 运行 `cargo clippy` 修复所有 warning
- [ ] 运行 `cargo test` 确保所有测试通过
- [ ] 运行 `cargo doc --no-deps` 生成文档

---

## 4. 测试计划

### 4.1 单元测试清单

#### marvis-core
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_message_creation` | 创建不同类型的 Message | 正确创建 User/Assistant/ToolResult 消息 |
| `test_tool_call_serialization` | ToolCall 序列化/反序列化 | 序列化结果符合预期 JSON |
| `test_tool_result_is_error` | ToolResult 错误标记 | is_error=true 时正确标记 |
| `test_marvis_error_display` | 错误类型 Display | 各错误类型有可读的 Display 输出 |
| `test_tool_schema_generation` | 生成 ToolSchema | Schema 包含 name/description/parameters |
| `test_risk_level_ordering` | RiskLevel 比较 | Dangerous > Normal > ReadOnly |

#### marvis-ai
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_mock_client_text_response` | Mock 返回文本 | 返回预设文本响应 |
| `test_mock_client_tool_call` | Mock 返回工具调用 | 返回预设工具调用 |
| `test_anthropic_request_format` | Anthropic 请求序列化 | 格式符合 API 文档 |
| `test_anthropic_response_parse` | Anthropic 响应反序列化 | 正确解析 tool_use 和 text |
| `test_openai_request_format` | OpenAI 请求序列化 | 格式符合 API 文档 |
| `test_openai_response_parse` | OpenAI 响应反序列化 | 正确解析 function_call |
| `test_stream_event_ordering` | 流式事件顺序 | TextDelta → Done 正确顺序 |

#### marvis-tools
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_read_file_exists` | 读取存在的文件 | 返回文件内容 |
| `test_read_file_not_found` | 读取不存在的文件 | 返回错误，is_error=true |
| `test_write_and_read_file` | 写入后读取 | 读取内容与写入一致 |
| `test_list_directory` | 列出目录 | 返回目录内容列表 |
| `test_delete_file` | 删除文件 | 文件被成功删除 |
| `test_file_ops_on_system_path` | 操作系统路径 | 被沙箱拦截 |
| `test_list_processes` | 列出进程 | 返回进程列表 |
| `test_process_info` | 获取进程详情 | 返回进程信息 |
| `test_kill_nonexistent_process` | 终止不存在进程 | 返回错误 |
| `test_web_fetch` | 获取网页 | 返回 HTML 文本内容 |
| `test_web_search` | 搜索网页 | 返回搜索结果列表 |
| `test_system_info` | 获取系统信息 | 返回 OS/CPU/内存信息 |
| `test_clipboard_roundtrip` | 剪贴板读写 | 读取内容与写入一致 |
| `test_registry_register_and_get` | 注册和查找工具 | 正确注册并查找 |
| `test_registry_duplicate_register` | 重复注册 | 正确处理 (覆盖或报错) |
| `test_registry_all_schemas` | 获取所有 schema | 返回正确数量的 schema |

#### marvis-session
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_add_messages` | 添加消息 | 历史记录正确增长 |
| `test_history_serialization` | 历史序列化 | JSON 格式正确 |
| `test_history_deserialization` | 历史反序列化 | 恢复与原始一致 |
| `test_context_window_trimming` | 上下文裁剪 | 超过窗口后旧消息被裁剪 |
| `test_system_prompt_preserved` | 系统提示保留 | 裁剪后 system prompt 不变 |
| `test_session_save_load` | 会话保存加载 | 加载的会话与保存一致 |

#### marvis-security
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_read_only_allowed` | 只读操作放行 | read_file 不需要确认 |
| `test_dangerous_requires_confirmation` | 危险操作需确认 | delete_file 需要确认 |
| `test_system_path_blocked` | 系统路径拦截 | C:\Windows 操作被拒绝 |
| `test_sensitive_command_detection` | 敏感命令检测 | rm -rf 被标记为危险 |
| `test_permission_mode_switch` | 权限模式切换 | 切换后生效 |

#### marvis-agent
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_agent_loop_text_response` | 纯文本响应 | 正确返回文本 |
| `test_agent_loop_tool_call` | 单次工具调用 | 正确执行并返回结果 |
| `test_agent_loop_multi_tool` | 多次工具调用 | 正确执行多次调用 |
| `test_agent_loop_max_iterations` | 最大循环次数 | 超限后终止 |
| `test_orchestrator_decompose` | 任务分解 | 正确分解为子任务 |
| `test_planner_generates_steps` | 计划生成 | 生成合理的步骤 |

#### marvis-cli
| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_args_parse_run` | 解析 run 子命令 | 正确解析 query |
| `test_args_parse_default` | 默认参数 | 启动 REPL |
| `test_args_invalid_model` | 无效模型名 | 报错提示 |

### 4.2 集成测试

| 测试名称 | 测试内容 | 预期结果 |
|---------|---------|---------|
| `test_full_flow_text_query` | 用户问 "你好" → 文本回复 | 返回文本回复 |
| `test_full_flow_file_read` | 用户问 "读 README" → 工具调用 → 回复 | 返回文件内容摘要 |
| `test_full_flow_dangerous_blocked` | 用户问 "删系统文件" → 拦截 | 返回权限拒绝 |
| `test_full_flow_dangerous_confirmed` | 用户确认后执行危险操作 | 操作被执行 |
| `test_full_flow_multi_turn` | 多轮对话 | 上下文正确传递 |
| `test_session_persistence` | 保存会话→退出→加载→继续 | 历史完整恢复 |

### 4.3 手动功能测试场景

1. **基础对话**: 启动 `marvis repl`，输入 "Hello, what can you do?"，验证回复
2. **文件操作**: 输入 "List all files in the current directory"，验证文件列表
3. **系统信息**: 输入 "What's my CPU usage?"，验证系统信息
4. **网页抓取**: 输入 "Fetch and summarize the content of https://www.rust-lang.org"
5. **安全检查**: 尝试输入 "Delete C:\Windows\System32"，验证被拦截
6. **多轮交互**: 连续输入多条相关指令，验证上下文保持
7. **错误处理**: 输入 "Read a file that doesn't exist"，验证错误提示

---

## 5. 质量保证流程

### 5.1 每次提交前检查

```bash
# 1. 格式化
cargo fmt --all

# 2. Lint
cargo clippy --all-targets --all-features -- -D warnings

# 3. 测试
cargo test --all

# 4. 构建
cargo build --release

# 5. 文档
cargo doc --no-deps --document-private-items
```

### 5.2 Rust 核心特性体现清单

| 特性 | 体现位置 | 说明 |
|------|---------|------|
| **Ownership / Borrowing** | 全局 | 使用 `&str`, `&[Message]` 等引用避免不必要的 clone；工具执行使用 `&self`；Arc 共享状态 |
| **Struct / Enum** | marvis-core | `Message`, `ToolCall`, `ToolResult` 结构体；`AiResponse`, `RiskLevel`, `MarvisError` 枚举 |
| **Trait** | marvis-core, marvis-ai | `Tool` trait, `AiClient` trait；trait object 动态分发 |
| **泛型** | marvis-tools, marvis-agent | `ToolRegistry<T: Tool>`, 泛型约束 |
| **生命周期** | marvis-session | `ConversationHistory` 迭代器中的生命周期标注 |
| **Result/Error** | 全局 | 所有可能失败的操作返回 `Result`；`anyhow` 传播 + `thiserror` 定义 |
| **并发/异步** | marvis-agent, marvis-tools | Tokio async/await；并行工具执行；流式响应 |
| **模块化** | 全局 | Cargo workspace + mod 系统 |

### 5.3 错误处理准则

- ❌ 禁止：代码中直接使用 `unwrap()` 或 `expect()`（测试代码除外）
- ✅ 使用 `?` 传播错误
- ✅ 使用 `.context()` 添加上下文信息
- ✅ 所有工具 `execute()` 返回 `Result<ToolResult>`，执行失败时 `is_error = true`
- ✅ API 调用失败返回 `MarvisError::AiError`
- ✅ 权限拒绝返回 `MarvisError::PermissionDenied`

---

## 6. 验收标准

### 6.1 功能完整性
- [x] ~~支持 REPL 交互模式~~
- [ ] 支持单次执行模式 (`marvis run "query"`)
- [ ] 至少 5 种文件操作工具
- [ ] 至少 3 种进程/系统工具
- [ ] 至少 2 种网页工具
- [ ] AI 对话功能（支持至少一个 AI 提供商或 Mock 模式）
- [ ] 流式响应输出
- [ ] 安全检查与确认流程

### 6.2 技术要求
- [ ] 使用 Cargo workspace 管理多 crate
- [ ] 所有错误通过 Result 传播，无不必要的 unwrap/expect
- [ ] trait 抽象 AI 客户端和工具
- [ ] 泛型使用
- [ ] 生命周期标注（至少一处）
- [ ] 并发/异步 (Tokio)
- [ ] 单元测试 ≥ 20 个
- [ ] 集成测试 ≥ 5 个

### 6.3 工程质量
- [ ] `cargo fmt` 通过
- [ ] `cargo clippy` 无 warning（或全部有合理解释的 allow）
- [ ] `cargo test` 全部通过
- [ ] `cargo build --release` 成功
- [ ] README.md 完整

---

## 附录：每日开发检查清单

在每次开发会话结束时检查：

- [ ] 今天新增的代码都有对应的测试吗？
- [ ] 今天新增的函数都有文档注释吗？
- [ ] 今天修改后 `cargo fmt` 过了吗？
- [ ] 今天修改后 `cargo clippy` 过了吗？
- [ ] 今天修改后 `cargo test` 全过了吗？
- [ ] 今天修改后 `cargo build` 成功了吗？
- [ ] 有需要更新 README 的内容吗？
- [ ] 代码中有没有遗留下来的 `unwrap()` / `expect()` / `todo!()`？
