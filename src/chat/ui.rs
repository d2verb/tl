use super::session::SessionConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ANSI color codes
const GREEN_BOLD: &str = "\x1B[1;32m";
const RESET: &str = "\x1B[0m";

pub fn prompt() -> String {
    format!("{GREEN_BOLD}>{RESET} ")
}

pub fn print_header() {
    println!();
    println!("tl v{VERSION} - Interactive Translation Mode");
    println!("Type text to translate, or use /commands. Press Ctrl+C to exit.");
    println!();
}

pub fn print_goodbye() {
    println!("Goodbye!");
}

pub fn print_config(config: &SessionConfig) {
    println!("Current configuration:");
    println!("  to       = {}", config.to);
    println!("  endpoint = {}", config.endpoint);
    println!("  model    = {}", config.model);
    println!();
}

pub fn print_set_success(key: &str, value: &str) {
    println!("Set '{key}' to '{value}'");
    println!();
}

pub fn print_help() {
    println!("Available commands:");
    println!("  /config             Show current configuration");
    println!("  /set <key> <value>  Set configuration (session only)");
    println!("  /clear              Clear screen");
    println!("  /help               Show this help");
    println!("  /quit               Exit chat mode");
    println!();
}

pub fn print_error(message: &str) {
    eprintln!("Error: {message}");
    eprintln!();
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}
