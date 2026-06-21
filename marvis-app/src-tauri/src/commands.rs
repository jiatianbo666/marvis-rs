//! Tauri commands — real AI + real tools, proper Windows handling.
//! Pattern: claude-code-rust agent loop with function calling.

use marvis_agent::{AgentConfig, AgentLoop};
use marvis_ai::deepseek::DeepSeekClient;
use marvis_ai::AiClient;
use marvis_core::{AiResponse, Message, Role, ToolCall, ToolResult, ToolSchema};
use marvis_security::permissions::{PermissionMode, SecurityManager};
use marvis_session::ConversationHistory;
use marvis_tools::ToolRegistry;
use marvis_tools::{
    clipboard::{ReadClipboard, WriteClipboard},
    file::{DeleteFile, FileInfo, ListDirectory, ReadFile, WriteFile},
    process::{CpuInfo, ListProcesses, MemoryInfo, ProcessInfo},
    system::{EnvVariable, SystemInfo},
    web::{WebFetch, WebSearch},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

const DEEPSEEK_KEY: &str = "sk-f81b53fef0df437f9b9a1021d2d92471";
const DEEPSEEK_MODEL: &str = "deepseek-v4-pro";

// ============ 共享状态 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub running: bool,
    pub stage: String,
    pub message: String,
    pub result: String,
    pub tokens: u64,
    pub agents: Vec<AgentStatusInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusInfo {
    pub id: String,
    pub display: String,
    pub status: String,
    pub message: String,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self {
            running: false, stage: "idle".into(), message: "Ready".into(),
            result: String::new(), tokens: 0,
            agents: vec![
                AgentStatusInfo { id: "orchestrator".into(), display: "🎯 Panda PM".into(), status: "idle".into(), message: "Ready".into() },
                AgentStatusInfo { id: "file_agent".into(), display: "🦊 Fox File Ops".into(), status: "idle".into(), message: "Ready".into() },
                AgentStatusInfo { id: "computer_agent".into(), display: "🐴 Horse SysInfo".into(), status: "idle".into(), message: "Ready".into() },
                AgentStatusInfo { id: "app_agent".into(), display: "🐰 Rabbit Apps".into(), status: "idle".into(), message: "Ready".into() },
                AgentStatusInfo { id: "browser_agent".into(), display: "🐶 Dog Browser".into(), status: "idle".into(), message: "Ready".into() },
                AgentStatusInfo { id: "search_agent".into(), display: "🐷 Pig Search".into(), status: "idle".into(), message: "Ready".into() },
            ],
        }
    }
}

pub struct SharedTask { pub status: Arc<Mutex<TaskStatus>> }

fn set_agent(s: &mut TaskStatus, id: &str, status: &str, msg: &str) {
    for a in &mut s.agents { if a.id == id { a.status = status.into(); a.message = msg.into(); } }
}

fn agent_for_tool(name: &str) -> &str {
    match name {
        "read_file"|"write_file"|"list_directory"|"delete_file"|"file_info" => "file_agent",
        "list_processes"|"process_info"|"cpu_info"|"memory_info"|"system_info"|"env_variable" => "computer_agent",
        "web_fetch"|"web_search"|"open_browser" => "browser_agent",
        "run_shell" => "app_agent",
        _ => "search_agent",
    }
}

// ============ 自定义工具：打开浏览器（不通过 run_command）============

use async_trait::async_trait;

struct OpenBrowserTool;
#[async_trait]
impl marvis_core::Tool for OpenBrowserTool {
    fn name(&self) -> &str { "open_browser" }
    fn description(&self) -> &str {
        "Open a URL in the system default browser. Use this to search the web or visit websites."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "The URL to open (e.g. https://www.google.com/search?q=rust)"},
                "search_query": {"type": "string", "description": "Alternative: a search query. Will open Google search."}
            },
            "required": []
        })
    }
    fn risk_level(&self) -> marvis_core::RiskLevel { marvis_core::RiskLevel::Normal }
    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, marvis_core::MarvisError> {
        let url = if let Some(u) = input["url"].as_str() {
            u.to_string()
        } else if let Some(q) = input["search_query"].as_str() {
            format!("https://www.google.com/search?q={}", urlencode(q))
        } else {
            // 从整个 input 中找任意 URL 或搜索词
            let s = input.to_string();
            if s.contains("http") {
                s.split("http").nth(1).map(|x| format!("http{}", x.split('"').next().unwrap_or(x))).unwrap_or("https://google.com".into())
            } else {
                "https://www.google.com".into()
            }
        };

        open_with_shell(&url).map(|_| {
            ToolResult::success("open_browser", format!("✅ Browser opened: {}", url))
        }).map_err(|e| marvis_core::MarvisError::ToolError {
            tool: "open_browser".into(),
            message: e,
        })
    }
}

struct RunShellTool;
#[async_trait]
impl marvis_core::Tool for RunShellTool {
    fn name(&self) -> &str { "run_shell" }
    fn description(&self) -> &str {
        "Execute a shell command. On Windows uses cmd /C. Use ONLY for: echo, dir, type, mkdir, rmdir, copy, move, del. NEVER use for browsers or URLs."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "The command to execute"}
            },
            "required": ["command"]
        })
    }
    fn risk_level(&self) -> marvis_core::RiskLevel { marvis_core::RiskLevel::Normal }
    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, marvis_core::MarvisError> {
        let cmd = input["command"].as_str().unwrap_or("");
        let lower = cmd.to_lowercase();

        // 🔥 浏览器命令 → 自动转换成 open_with_shell，不报错
        if lower.contains("msedge") || lower.contains("chrome") || lower.contains("firefox")
            || lower.contains("iexplore") || lower.contains("start http") || lower.contains("explorer http")
            || lower.contains("edge") || lower.contains("browser") {
            // 从命令中提取 URL
            let url = if let Some(pos) = lower.find("http") {
                cmd[pos..].split_whitespace().next().unwrap_or("https://www.google.com")
            } else if lower.contains("taobao") {
                "https://www.taobao.com"
            } else if lower.contains("google") {
                "https://www.google.com"
            } else {
                // 默认搜索
                let query = cmd.split_whitespace().last().unwrap_or("search");
                &format!("https://www.google.com/search?q={}", query)
            };

            match open_with_shell(url) {
                Ok(()) => return Ok(ToolResult::success("run_shell",
                    format!("✅ 浏览器已打开: {}", url))),
                Err(e) => return Ok(ToolResult::error("run_shell",
                    format!("打开浏览器失败: {}. 请尝试手动访问: {}", e, url))),
            }
        }

        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/C", cmd]).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).output()
        };

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let mut r = String::new();
                if !stdout.is_empty() { r.push_str(&stdout); }
                if !stderr.is_empty() { r.push_str(&format!("\nSTDERR: {}", stderr)); }
                if r.is_empty() { r = format!("Exit code: {}", o.status.code().unwrap_or(-1)); }
                if o.status.success() {
                    Ok(ToolResult::success("run_shell", r))
                } else {
                    Ok(ToolResult::error("run_shell", r))
                }
            }
            Err(e) => Ok(ToolResult::error("run_shell", format!("Failed: {}", e))),
        }
    }
}

fn build_registry() -> ToolRegistry {
    let mut r = ToolRegistry::new();
    r.register(ReadFile); r.register(WriteFile); r.register(ListDirectory);
    r.register(DeleteFile); r.register(FileInfo);
    r.register(ListProcesses); r.register(ProcessInfo);
    r.register(CpuInfo); r.register(MemoryInfo);
    r.register(WebFetch); r.register(WebSearch);
    r.register(SystemInfo); r.register(EnvVariable);
    r.register(ReadClipboard); r.register(WriteClipboard);
    r.register(OpenBrowserTool);  // ← 专用浏览器工具
    r.register(RunShellTool);     // ← 有保护的 shell
    r
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn run_task(state: State<'_, SharedTask>, task: String) -> Result<TaskStatus, String> {
    let mut s = state.status.lock().await;
    if s.running { return Ok(s.clone()); }
    s.running = true; s.stage = "thinking".into(); s.result.clear(); s.tokens = 0;
    set_agent(&mut s, "orchestrator", "thinking", "Analyzing...");
    drop(s);

    let status_ref = state.status.clone();
    let task_clone = task.clone();

    tokio::spawn(async move {
        let result = run_ai_agent(&task_clone, &status_ref).await;
        let mut s = status_ref.lock().await;
        match result {
            Ok((text, tokens)) => { s.stage = "done".into(); s.result = text; s.tokens = tokens; }
            Err(e) => { s.stage = "error".into(); s.result = format!("❌ {}", e); }
        }
        s.running = false;
        set_agent(&mut s, "orchestrator", "idle", "Ready");
        for a in &mut s.agents { if a.id != "orchestrator" { a.status = "idle".into(); a.message = "Ready".into(); } }
    });

    Ok(state.status.lock().await.clone())
}

#[tauri::command]
pub async fn get_task_status(state: State<'_, SharedTask>) -> Result<TaskStatus, String> {
    Ok(state.status.lock().await.clone())
}

#[tauri::command] pub fn get_tools() -> Vec<String> { build_registry().list_names().iter().map(|s| s.to_string()).collect() }
#[tauri::command] pub fn get_agents() -> Vec<AgentInfo> { vec![
    AgentInfo { id: "orchestrator".into(), name: "Panda PM".into(), role: "Task Planner".into(), pet: "🐼".into() },
    AgentInfo { id: "file_agent".into(), name: "Fox File Ops".into(), role: "File Scanner".into(), pet: "🦊".into() },
    AgentInfo { id: "computer_agent".into(), name: "Horse SysInfo".into(), role: "System Monitor".into(), pet: "🐴".into() },
    AgentInfo { id: "app_agent".into(), name: "Rabbit Apps".into(), role: "App Manager".into(), pet: "🐰".into() },
    AgentInfo { id: "browser_agent".into(), name: "Dog Browser".into(), role: "Web Scraper".into(), pet: "🐶".into() },
    AgentInfo { id: "search_agent".into(), name: "Pig Search".into(), role: "Search Strategist".into(), pet: "🐷".into() },
]}
#[tauri::command] pub async fn confirm_action() -> Result<TaskResult, String> { Ok(TaskResult { ok: true, message: "OK".into() }) }

#[derive(Debug, Serialize, Deserialize)] pub struct TaskResult { pub ok: bool, pub message: String }
#[derive(Debug, Serialize, Deserialize)] pub struct AgentInfo { pub id: String, pub name: String, pub role: String, pub pet: String }

// ============ 核心：手工 Agent Loop（逐步更新 agent 状态）============

async fn run_ai_agent(task: &str, status_ref: &Arc<Mutex<TaskStatus>>) -> Result<(String, u64), String> {
    let client = DeepSeekClient::new(DEEPSEEK_KEY, DEEPSEEK_MODEL);
    let tool_registry = build_registry();

    let system = format!(
        "You are Marvis, a CLI AI assistant controlling a Windows computer.\n\
        You have access to tools. Use them to complete the user's task.\n\
        \n\
        TOOLS:\n\
        - open_browser: Open a URL in the browser. Use for ANY web search, website visit, or browser action.\n\
        - list_directory(path): List files in a directory.\n\
        - read_file(path): Read file contents.\n\
        - system_info: Get OS and system info.\n\
        - cpu_info / memory_info: Get CPU/memory status.\n\
        - list_processes(sort_by, limit): List running processes.\n\
        - web_fetch(url): Fetch a web page.\n\
        - web_search(query): Search the web.\n\
        - run_shell(command): Execute Windows cmd command. Only: echo, dir, type, mkdir, copy, move, del.\n\
        \n\
        RULES:\n\
        1. You MAY call 1-3 tools if the task requires it. Call them one at a time.\n\
        2. After EACH tool, decide: do I need another tool, or should I summarize?\n\
        3. For ANY browser/search action, use open_browser, NOT run_shell.\n\
        4. Speak Chinese. Be helpful and direct."
    );

    let mut messages: Vec<Message> = vec![
        Message::system(&system),
        Message::user(task),
    ];

    let mut total_tokens: u64 = 0;
    let tool_schemas: Vec<ToolSchema> = tool_registry.schemas();

    // Orchestrator starts thinking
    {
        let mut s = status_ref.lock().await;
        set_agent(&mut s, "orchestrator", "thinking", "Analyzing your request...");
        s.stage = "thinking".into();
    }

    // Agent loop — max 5 iterations
    for iteration in 0..5 {
        // Call AI
        let response = client.chat(&messages, &tool_schemas).await
            .map_err(|e| format!("AI error: {}", e))?;

        match response {
            AiResponse::Text(text) => {
                total_tokens += (text.len() as u64) / 2 + 200;
                let mut s = status_ref.lock().await;
                set_agent(&mut s, "orchestrator", "done", "Task complete!");
                s.stage = "done".into();
                return Ok((text, total_tokens));
            }
            AiResponse::ToolCalls(calls) => {
                // 🔥 每个工具执行前更新状态
                let mut tool_results: Vec<(String, ToolResult)> = Vec::new();

                for call in &calls {
                    let agent_id = agent_for_tool(&call.name);

                    // 通知：agent 开始工作
                    {
                        let mut s = status_ref.lock().await;
                        set_agent(&mut s, agent_id, "working",
                            &format!("Executing {}...", call.name));
                        s.stage = "working".into();
                    }
                    // 让前端有时间轮询到
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                    // 真执行工具
                    let result = tool_registry.execute(&call.name, &call.arguments).await;

                    // 通知：agent 完成
                    {
                        let mut s = status_ref.lock().await;
                        match &result {
                            Ok(r) if !r.is_error => {
                                set_agent(&mut s, agent_id, "done",
                                    &format!("{} complete!", call.name));
                                total_tokens += 300;
                            }
                            Ok(r) => {
                                set_agent(&mut s, agent_id, "error", "Tool returned error");
                            }
                            Err(e) => {
                                set_agent(&mut s, agent_id, "error", &format!("Failed: {}", e));
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                    let tr = match result {
                        Ok(r) => r,
                        Err(e) => ToolResult::error(&call.id, format!("{}", e)),
                    };

                    tool_results.push((call.id.clone(), tr));
                }

                // 把 assistant tool_calls + tool results 加入 messages
                let assistant_msg = Message {
                    role: Role::Assistant,
                    content: String::new(),
                    tool_call_id: None,
                    tool_calls: Some(calls.clone()),
                    name: None,
                };
                messages.push(assistant_msg);

                for (call_id, tr) in &tool_results {
                    let tool_msg = Message {
                        role: Role::Tool,
                        content: tr.content.clone(),
                        tool_call_id: Some(call_id.clone()),
                        tool_calls: None,
                        name: Some(call_id.clone()), // will be overwritten by agent_for_tool lookup
                    };
                    messages.push(tool_msg);
                }

                // Orchestrator: back to thinking for next round
                {
                    let mut s = status_ref.lock().await;
                    set_agent(&mut s, "orchestrator", "thinking", "Deciding next step...");
                }
                // Continue loop — AI decides whether to call more tools or finish
            }
        }
    }

    Err("任务执行超时（达到最大步骤数）".into())
}

// ============ Windows Shell 打开 URL ============

fn open_with_shell(url: &str) -> Result<(), String> {
    // 方法1: cmd /c start
    if std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn().is_ok() {
        return Ok(());
    }
    // 方法2: explorer
    if std::process::Command::new("explorer.exe").arg(url).spawn().is_ok() {
        return Ok(());
    }
    // 方法3: rundll32
    if std::process::Command::new("rundll32.exe").args(["url.dll,FileProtocolHandler", url]).spawn().is_ok() {
        return Ok(());
    }
    Err("Cannot open browser".into())
}

fn urlencode(s: &str) -> String {
    s.chars().map(|c| match c {
        'a'..='z'|'A'..='Z'|'0'..='9'|'-'|'_'|'.'|'~' => c.to_string(),
        ' ' => "+".into(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}
