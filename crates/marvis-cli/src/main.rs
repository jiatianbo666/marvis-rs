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
            let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| String::new());
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
