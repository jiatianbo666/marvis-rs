//! Output formatting and display utilities.

/// Print a section header.
pub fn print_header(text: &str) {
    println!("\n{}", text);
    println!("{}", "─".repeat(text.chars().count()));
}

/// Print a tool call in a visually distinct format.
pub fn print_tool_call(name: &str, args: &serde_json::Value) {
    println!("\n🔧 \x1b[36mTool: {}\x1b[0m", name);
    let args_str = serde_json::to_string_pretty(args).unwrap_or_default();
    if args_str.len() > 200 {
        println!("   Args: {}...", &args_str[..200]);
    } else {
        println!("   Args: {}", args_str);
    }
}

/// Print a tool result.
pub fn print_tool_result(content: &str, is_error: bool) {
    if is_error {
        println!("❌ \x1b[31mError:\x1b[0m {}", content);
    } else {
        // Truncate long results
        if content.len() > 1000 {
            println!("✅ \x1b[32mResult:\x1b[0m {}...", &content[..1000]);
            println!("   (output truncated)");
        } else {
            println!("✅ \x1b[32mResult:\x1b[0m {}", content);
        }
    }
}

/// Print a streaming text delta.
pub fn print_text_delta(delta: &str) {
    print!("{}", delta);
}

/// Print the AI response with formatting.
pub fn print_response(text: &str) {
    println!("\n🤖 \x1b[1mMarvis:\x1b[0m\n");
    println!("{}", text);
    println!();
}

/// Print a welcome banner.
pub fn print_welcome() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r#"
╔══════════════════════════════════════════╗
║        🦀  Marvis v{:<20} 🦀  ║
║     My AI Virtual Intelligent System    ║
╚══════════════════════════════════════════╝
Type /help for commands, or just ask me anything!
Type /exit to quit.
"#,
        version
    );
}

/// Print a prompt marker.
pub fn print_prompt() {
    print!("\n\x1b[32m>\x1b[0m ");
}
