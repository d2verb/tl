//! Chat mode UI components.

use crate::ui::Style;

use super::session::SessionConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_header() {
    println!(
        "{} {} - Interactive Translation Mode",
        Style::header("tl"),
        Style::version(format!("v{VERSION}"))
    );
    println!();
}

pub fn print_goodbye() {
    println!("{}", Style::success("Goodbye!"));
}

pub fn print_config(config: &SessionConfig) {
    println!("{}", Style::header("Configuration"));
    println!(
        "  {}   {}",
        Style::label("provider"),
        Style::value(&config.provider_name)
    );
    println!(
        "  {}      {}",
        Style::label("model"),
        Style::value(&config.model)
    );
    println!(
        "  {}         {}",
        Style::label("to"),
        Style::value(&config.to)
    );
    println!(
        "  {}   {}",
        Style::label("endpoint"),
        Style::secondary(&config.endpoint)
    );
    println!();
}

pub fn print_help() {
    println!("{}", Style::header("Available commands"));
    println!(
        "  {}  {}",
        Style::command("/config"),
        Style::secondary("Show current configuration")
    );
    println!(
        "  {}    {}",
        Style::command("/help"),
        Style::secondary("Show this help")
    );
    println!(
        "  {}    {}",
        Style::command("/quit"),
        Style::secondary("Exit chat mode")
    );
    println!();
}

pub fn print_error(message: &str) {
    eprintln!("{} {message}", Style::error("Error:"));
    eprintln!();
}
