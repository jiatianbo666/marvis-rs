//! Interactive REPL (Read-Eval-Print Loop) for Marvis.

use crate::commands::handle_builtin;
use crate::display;
use log::info;
use marvis_agent::AgentLoop;
use marvis_security::SecurityManager;
use marvis_session::ConversationHistory;
use marvis_tools::ToolRegistry;
use std::io::{self, Write};

/// Run the interactive REPL.
pub async fn run_repl(
    agent: &AgentLoop<'_>,
    tools: &ToolRegistry,
    _security: &SecurityManager,
) -> anyhow::Result<()> {
    display::print_welcome();

    let mut history = ConversationHistory::new();
    let stdin = io::stdin();

    loop {
        display::print_prompt();
        io::stdout().flush()?;

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF
                println!("\nGoodbye! 👋");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        // Check for built-in commands
        let tool_names: Vec<&str> = tools.list_names();
        if let Some(response) = handle_builtin(&input, &tool_names) {
            if input == "/exit" || input == "/quit" {
                println!("{}", response);
                break;
            }
            if input == "/clear" {
                history.clear();
                println!("{}", response);
                continue;
            }
            println!("{}", response);
            continue;
        }

        // Process with AI agent
        info!("Processing user input: {}", input);
        println!(); // Blank line before response

        match agent.run(&mut history, &input).await {
            Ok(response) => {
                display::print_response(&response);
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
    }

    Ok(())
}
