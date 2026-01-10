use super::session::SessionConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_header() {
    println!("tl v{VERSION} - Interactive Translation Mode");
    println!();
}

pub fn print_goodbye() {
    println!("Goodbye!");
}

pub fn print_config(config: &SessionConfig) {
    println!("Current configuration:");
    println!("  provider = {}", config.provider_name);
    println!("  model    = {}", config.model);
    println!("  to       = {}", config.to);
    println!("  endpoint = {}", config.endpoint);
    println!();
}

pub fn print_help() {
    println!("Available commands:");
    println!("  /config  Show current configuration");
    println!("  /help    Show this help");
    println!("  /quit    Exit chat mode");
    println!();
}

pub fn print_error(message: &str) {
    eprintln!("Error: {message}");
    eprintln!();
}
