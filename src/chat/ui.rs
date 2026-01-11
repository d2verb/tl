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
        "  {}      {}",
        Style::label("style"),
        config
            .style_name
            .as_deref()
            .map_or_else(|| Style::secondary("(none)"), Style::value)
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
    println!(
        "  {}     {}",
        Style::command("/set"),
        Style::secondary("Set option (style, to, model)")
    );
    println!();
    println!("{}", Style::header("Set examples"));
    println!(
        "  {}  {}",
        Style::command("/set style casual"),
        Style::secondary("Use casual translation style")
    );
    println!(
        "  {}         {}",
        Style::command("/set to ja"),
        Style::secondary("Set target language to Japanese")
    );
    println!(
        "  {}  {}",
        Style::command("/set model gpt-4o"),
        Style::secondary("Switch to a different model")
    );
    println!(
        "  {}      {}",
        Style::command("/set style"),
        Style::secondary("Clear style (no style)")
    );
    println!();
}

pub fn print_error(message: &str) {
    eprintln!("{} {message}", Style::error("Error:"));
    eprintln!();
}
