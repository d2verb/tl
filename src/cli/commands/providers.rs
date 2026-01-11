//! Provider listing command handler.

use anyhow::Result;

use crate::config::ConfigManager;
use crate::ui::Style;

/// Prints configured providers to stdout.
///
/// If `specific_provider` is provided, shows detailed information for that provider.
/// Otherwise, lists all configured providers with their endpoints and models.
pub fn print_providers(specific_provider: Option<&str>) -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    if config.providers.is_empty() {
        println!("{}", Style::warning("No providers configured."));
        println!(
            "{}",
            Style::hint("Add providers to ~/.config/tl/config.toml")
        );
        return Ok(());
    }

    let default_provider = config.tl.provider.as_deref();

    if let Some(provider_name) = specific_provider {
        // Show details for a specific provider
        if let Some(provider) = config.providers.get(provider_name) {
            let is_default = default_provider == Some(provider_name);
            println!(
                "{} {}{}",
                Style::label("Provider"),
                Style::value(provider_name),
                if is_default {
                    format!(" {}", Style::default_marker())
                } else {
                    String::new()
                }
            );
            println!(
                "  {} {}",
                Style::label("endpoint"),
                Style::secondary(&provider.endpoint)
            );
            if provider.api_key_env.is_some() || provider.api_key.is_some() {
                let has_key = provider.get_api_key().is_some();
                println!(
                    "  {}  {}",
                    Style::label("api_key"),
                    if has_key {
                        Style::success("(set)")
                    } else {
                        Style::warning("(not set)")
                    }
                );
            }
            if provider.models.is_empty() {
                println!(
                    "  {}   {}",
                    Style::label("models"),
                    Style::secondary("(none configured)")
                );
            } else {
                println!("  {}", Style::label("models"));
                for model in &provider.models {
                    println!("    {} {}", Style::secondary("-"), Style::value(model));
                }
            }
        } else {
            anyhow::bail!("Provider '{provider_name}' not found");
        }
    } else {
        // List all providers
        println!("{}", Style::header("Configured providers"));
        for (name, provider) in &config.providers {
            let is_default = default_provider == Some(name.as_str());
            println!(
                "  {}{}",
                Style::value(name),
                if is_default {
                    format!(" {}", Style::default_marker())
                } else {
                    String::new()
                }
            );
            println!(
                "    {}  {}",
                Style::label("endpoint"),
                Style::secondary(&provider.endpoint)
            );
            if !provider.models.is_empty() {
                println!(
                    "    {}    {}",
                    Style::label("models"),
                    Style::secondary(provider.models.join(", "))
                );
            }
        }
    }

    Ok(())
}
