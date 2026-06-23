//! # marvis-cli
//!
//! Entry point for the Marvis CLI AI assistant.
//!
//! Marvis understands natural language and can execute system tools
//! like file operations, process management, and web interactions.

mod args;
mod commands;
mod display;
mod repl;

use args::{Args, Command};
use clap::Parser;
use log::LevelFilter;
use marvis_agent::AgentLoop;
use marvis_ai::mock::MockClient;
use marvis_security::permissions::{PermissionMode, SecurityManager};
use marvis_session::ConversationHistory;
use marvis_tools::registry::ToolRegistry;
use marvis_tools::{
    clipboard::{ReadClipboard, WriteClipboard},
    file::{DeleteFile, FileInfo, ListDirectory, ReadFile, WriteFile},
    process::{CpuInfo, ListProcesses, MemoryInfo, ProcessInfo},
    system::{EnvVariable, RunCommand, RunShell, SystemInfo},
    web::{OpenBrowser, WebFetch, WebSearch},
};

fn setup_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // File tools
    registry.register(ReadFile);
    registry.register(WriteFile);
    registry.register(ListDirectory);
    registry.register(DeleteFile);
    registry.register(FileInfo);

    // Process tools
    registry.register(ListProcesses);
    registry.register(ProcessInfo);
    registry.register(CpuInfo);
    registry.register(MemoryInfo);

    // Web tools
    registry.register(WebFetch);
    registry.register(WebSearch);
    registry.register(OpenBrowser);

    // System tools
    registry.register(SystemInfo);
    registry.register(EnvVariable);
    registry.register(RunCommand);
    registry.register(RunShell);

    // Clipboard tools
    registry.register(ReadClipboard);
    registry.register(WriteClipboard);

    registry
}

/// Search for an API key in env var first, then .env files (multi-path).
fn get_api_key(var_name: &str) -> String {
    // 1. Check environment variable
    if let Ok(key) = std::env::var(var_name) {
        let key = key.trim().trim_matches('"').trim_matches('\'').to_string();
        if !key.is_empty() && key != "your-deepseek-api-key" {
            return key;
        }
    }
    // 2. Search .env in multiple locations
    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let mut p = dir.to_path_buf();
            for _ in 0..6 {
                search_dirs.push(p.clone());
                if let Some(parent) = p.parent() { p = parent.to_path_buf(); } else { break; }
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut p = cwd;
        for _ in 0..4 {
            search_dirs.push(p.clone());
            if let Some(parent) = p.parent() { p = parent.to_path_buf(); } else { break; }
        }
    }
    for dir in &search_dirs {
        let env_path = dir.join(".env");
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
                let prefix = format!("{}=", var_name);
                if let Some(val) = trimmed.strip_prefix(&prefix) {
                    let val = val.trim().trim_matches('"').trim_matches('\'');
                    if !val.is_empty() && val != "your-deepseek-api-key" {
                        return val.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn parse_permission_mode(mode: &str) -> PermissionMode {
    match mode.to_lowercase().as_str() {
        "readonly" | "read-only" | "ro" => PermissionMode::ReadOnly,
        "dangerous" | "full" | "all" => PermissionMode::Dangerous,
        _ => PermissionMode::Normal,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_level(log_level)
        .init();

    // Set up components
    let tool_registry = setup_tools();
    let permission_mode = parse_permission_mode(&args.permission);
    let security = SecurityManager::new(permission_mode);

    // Create AI client based on provider
    let ai_client: Box<dyn marvis_ai::AiClient> = match args.provider.as_str() {
        "deepseek" => {
            let api_key = get_api_key("DEEPSEEK_API_KEY");
            if api_key.is_empty() {
                log::warn!("DEEPSEEK_API_KEY not set, falling back to mock");
                Box::new(MockClient::new())
            } else {
                Box::new(marvis_ai::deepseek::DeepSeekClient::new(
                    api_key,
                    &args.model,
                ))
            }
        }
        "qwen" => {
            let api_key = std::env::var("QWEN_API_KEY").unwrap_or_else(|_| String::new());
            if api_key.is_empty() {
                log::warn!("QWEN_API_KEY not set, falling back to mock");
                Box::new(MockClient::new())
            } else {
                Box::new(marvis_ai::qwen::QwenClient::new(
                    api_key,
                    "Qwen/Qwen3.5-35B-A3B",
                ))
            }
        }
        _ => {
            log::info!("Using mock AI client (no API calls)");
            Box::new(MockClient::new())
        }
    };

    let agent = AgentLoop::new(ai_client.as_ref(), &tool_registry, &security);

    // Execute based on command
    match args.command {
        Some(Command::Run { query }) => {
            // Single-shot mode
            let query_str = query.join(" ");
            let mut history = ConversationHistory::new();
            match agent.run(&mut history, &query_str).await {
                Ok(response) => {
                    println!("{}", response);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Command::Repl) | None => {
            // Interactive REPL mode
            repl::run_repl(&agent, &tool_registry, &security).await?;
        }
        Some(Command::Tools) => {
            println!("Available tools ({}):", tool_registry.len());
            for name in tool_registry.list_names() {
                println!("  • {}", name);
            }
        }
        Some(Command::ShowConfig) => {
            println!("Marvis Configuration:");
            println!("  Provider: {}", args.provider);
            println!("  Model: {}", args.model);
            println!("  Permission mode: {}", args.permission);
            println!("  Tools registered: {}", tool_registry.len());
            println!("  Streaming: enabled");
        }
        Some(Command::Sessions) => {
            println!("Session management is not yet implemented.");
            println!("Sessions will be stored in .marvis/sessions/");
        }
    }

    Ok(())
}
