//! Command-line argument parsing using clap derive API.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Marvis — My AI Virtual Intelligent System.
///
/// A Rust-powered CLI AI assistant that understands natural language
/// and can control your computer through system tools.
#[derive(Parser, Debug)]
#[command(
    name = "marvis",
    version = env!("CARGO_PKG_VERSION"),
    about = "My AI Virtual Intelligent System — a Rust CLI AI assistant",
    long_about = "Marvis is an AI assistant that can understand natural language \
                  instructions and execute system tools to help you control your computer."
)]
pub struct Args {
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// AI model to use
    #[arg(long, default_value = "deepseek-v4-pro")]
    pub model: String,

    /// AI provider: deepseek, qwen, or mock
    #[arg(short = 'P', long, default_value = "mock")]
    pub provider: String,

    /// Permission mode: readonly, normal, dangerous
    #[arg(short, long, default_value = "normal")]
    pub permission: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommand (if any)
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a single task and exit
    #[command(name = "run")]
    Run {
        /// The task description in natural language
        #[arg(required = true)]
        query: Vec<String>,
    },

    /// Start interactive REPL mode (default)
    #[command(name = "repl")]
    Repl,

    /// List all available tools
    #[command(name = "tools")]
    Tools,

    /// Show current configuration
    #[command(name = "config")]
    ShowConfig,

    /// List saved sessions
    #[command(name = "sessions")]
    Sessions,
}
