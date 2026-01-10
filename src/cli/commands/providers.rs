//! Provider listing command handler.

use anyhow::Result;

use crate::config::ConfigManager;

/// Prints configured providers to stdout.
///
/// If `specific_provider` is provided, shows detailed information for that provider.
/// Otherwise, lists all configured providers with their endpoints and models.
pub fn print_providers(specific_provider: Option<&str>) -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    if config.providers.is_empty() {
        println!("No providers configured.");
        println!("Add providers to ~/.config/tl/config.toml");
        return Ok(());
    }

    let default_provider = config.tl.provider.as_deref();

    if let Some(provider_name) = specific_provider {
        // Show details for a specific provider
        if let Some(provider) = config.providers.get(provider_name) {
            let is_default = default_provider == Some(provider_name);
            println!(
                "Provider: {}{}",
                provider_name,
                if is_default { " (default)" } else { "" }
            );
            println!("  endpoint = {}", provider.endpoint);
            if provider.api_key_env.is_some() || provider.api_key.is_some() {
                let has_key = provider.get_api_key().is_some();
                println!(
                    "  api_key  = {}",
                    if has_key { "(set)" } else { "(not set)" }
                );
            }
            if provider.models.is_empty() {
                println!("  models   = (none configured)");
            } else {
                println!("  models:");
                for model in &provider.models {
                    println!("    - {model}");
                }
            }
        } else {
            anyhow::bail!("Provider '{provider_name}' not found");
        }
    } else {
        // List all providers
        println!("Configured providers:\n");
        for (name, provider) in &config.providers {
            let is_default = default_provider == Some(name.as_str());
            println!("  {}{}", name, if is_default { " (default)" } else { "" });
            println!("    endpoint: {}", provider.endpoint);
            if !provider.models.is_empty() {
                println!("    models: {}", provider.models.join(", "));
            }
        }
    }

    Ok(())
}
