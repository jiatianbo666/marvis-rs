//! Built-in commands for the REPL mode.

/// Process a built-in slash command. Returns Some(message) if handled, None if not a command.
pub fn handle_builtin(input: &str, tools: &[&str]) -> Option<String> {
    let trimmed = input.trim();

    if !trimmed.starts_with('/') {
        return None;
    }

    let (cmd, _args) = trimmed.split_once(' ').unwrap_or((trimmed, ""));

    match cmd {
        "/help" => Some(help_text()),
        "/tools" => Some(tools_text(tools)),
        "/exit" | "/quit" => Some("👋 Goodbye! Use Ctrl+C or type /exit to leave.".to_string()),
        "/clear" => Some("[Session cleared — history has been reset]".to_string()),
        "/history" => Some("[Type /history to see past messages — not yet persisted]".to_string()),
        "/config" => Some(config_text()),
        _ => Some(format!(
            "Unknown command: {}\nType /help for a list of available commands.",
            cmd
        )),
    }
}

fn help_text() -> String {
    r#"Marvis Built-in Commands
========================
/help       Show this help message
/tools      List available tools
/config     Show current configuration
/clear      Clear conversation history
/history    Show conversation history
/exit       Exit Marvis

You can also type natural language queries directly!
"#
    .to_string()
}

fn tools_text(tools: &[&str]) -> String {
    if tools.is_empty() {
        return "No tools are currently registered.".to_string();
    }

    let mut text = String::from("Available Tools:\n");
    for tool in tools {
        text.push_str(&format!("  • {}\n", tool));
    }
    text
}

fn config_text() -> String {
    format!(
        "Marvis v{}\nProvider: mock\nModel: mock\nPermission: normal\nStreaming: enabled",
        env!("CARGO_PKG_VERSION")
    )
}
