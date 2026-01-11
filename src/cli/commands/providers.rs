//! Provider management command handler.

use anyhow::{Result, bail};
use inquire::{Confirm, Select, Text};

use crate::config::{ConfigManager, ProviderConfig};
use crate::ui::{Style, handle_prompt_cancellation};

/// Reserved names that cannot be used as provider names.
const RESERVED_NAMES: &[&str] = &["add", "edit", "remove", "list"];

/// Prints all configured providers.
pub fn list_providers() -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    if config.providers.is_empty() {
        println!("{}", Style::warning("No providers configured."));
        println!(
            "{}",
            Style::hint("Run 'tl providers add' to add a provider.")
        );
        return Ok(());
    }

    let default_provider = config.tl.provider.as_deref();

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

    Ok(())
}

/// Interactively adds a new provider.
pub fn add_provider() -> Result<()> {
    handle_prompt_cancellation(add_provider_inner)
}

fn add_provider_inner() -> Result<()> {
    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Input provider name
    let name = input_provider_name(&config.providers.keys().cloned().collect::<Vec<_>>())?;

    // Input endpoint
    let endpoint = input_endpoint(None)?;

    // Input API key method
    let (api_key, api_key_env) = input_api_key_method(None, None)?;

    // Input models
    let models = input_models(None)?;

    // Create provider config
    let provider_config = ProviderConfig {
        endpoint,
        api_key,
        api_key_env,
        models,
    };

    // Add to config
    config.providers.insert(name.clone(), provider_config);

    // Save config
    manager.save(&config)?;

    println!();
    println!(
        "{} Provider '{}' added to {}",
        Style::success("✓"),
        Style::value(&name),
        Style::secondary(manager.config_path().display().to_string())
    );

    Ok(())
}

/// Interactively edits an existing provider.
pub fn edit_provider(name: &str) -> Result<()> {
    handle_prompt_cancellation(|| edit_provider_inner(name))
}

fn edit_provider_inner(name: &str) -> Result<()> {
    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Check if provider exists
    let Some(provider) = config.providers.get(name) else {
        bail!("Provider '{name}' not found");
    };

    println!(
        "{} '{}':\n",
        Style::header("Editing provider"),
        Style::value(name)
    );

    // Input endpoint
    let endpoint = input_endpoint(Some(&provider.endpoint))?;

    // Input API key method
    let (api_key, api_key_env) =
        input_api_key_method(provider.api_key.as_deref(), provider.api_key_env.as_deref())?;

    // Input models
    let models = input_models(Some(&provider.models))?;

    // Update provider config
    let provider_config = ProviderConfig {
        endpoint,
        api_key,
        api_key_env,
        models,
    };

    config.providers.insert(name.to_string(), provider_config);

    // Save config
    manager.save(&config)?;

    println!();
    println!(
        "{} Provider '{}' updated",
        Style::success("✓"),
        Style::value(name)
    );

    Ok(())
}

/// Removes a provider with confirmation.
pub fn remove_provider(name: &str) -> Result<()> {
    handle_prompt_cancellation(|| remove_provider_inner(name))
}

fn remove_provider_inner(name: &str) -> Result<()> {
    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Check if provider exists
    if !config.providers.contains_key(name) {
        bail!("Provider '{name}' not found");
    }

    // Check if this is the default provider
    if config.tl.provider.as_deref() == Some(name) {
        bail!(
            "Cannot remove '{name}' because it is the default provider.\n\n\
             Run 'tl configure' to change the default provider first."
        );
    }

    // Check if this is the last provider
    if config.providers.len() == 1 {
        println!(
            "{} This is the last configured provider.",
            Style::warning("Warning:")
        );
    }

    // Confirm removal
    let confirmed = Confirm::new(&format!(
        "Are you sure you want to remove provider '{name}'?"
    ))
    .with_default(false)
    .prompt()?;

    if !confirmed {
        println!("Cancelled.");
        return Ok(());
    }

    // Remove provider
    config.providers.remove(name);

    // Save config
    manager.save(&config)?;

    println!();
    println!(
        "{} Provider '{}' removed",
        Style::success("✓"),
        Style::value(name)
    );

    Ok(())
}

fn input_provider_name(existing_names: &[String]) -> Result<String> {
    let name = Text::new("Provider name:")
        .with_help_message("A unique name for this provider (e.g., ollama, openrouter)")
        .prompt()?;

    let name = name.trim().to_string();

    if name.is_empty() {
        bail!("Provider name cannot be empty");
    }

    if RESERVED_NAMES.contains(&name.as_str()) {
        bail!("Provider name '{name}' is reserved. Choose a different name.");
    }

    if existing_names.contains(&name) {
        bail!("Provider '{name}' already exists");
    }

    Ok(name)
}

fn input_endpoint(default: Option<&str>) -> Result<String> {
    let mut prompt = Text::new("Endpoint URL:").with_help_message("OpenAI-compatible API endpoint");

    if let Some(d) = default {
        prompt = prompt.with_default(d);
    }

    let endpoint = prompt.prompt()?;
    let endpoint = endpoint.trim().to_string();

    if endpoint.is_empty() {
        bail!("Endpoint URL cannot be empty");
    }

    // Basic URL validation
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        bail!("Endpoint must start with http:// or https://");
    }

    Ok(endpoint)
}

fn input_api_key_method(
    current_api_key: Option<&str>,
    current_api_key_env: Option<&str>,
) -> Result<(Option<String>, Option<String>)> {
    let options = vec![
        "Environment variable (recommended)",
        "Store in config file",
        "None (no auth required)",
    ];

    // Determine default selection based on current config
    let default_index = if current_api_key_env.is_some() {
        0
    } else if current_api_key.is_some() {
        1
    } else {
        2
    };

    let selection = Select::new("API key method:", options)
        .with_starting_cursor(default_index)
        .prompt()?;

    match selection {
        "Environment variable (recommended)" => {
            let mut prompt = Text::new("Environment variable name:")
                .with_help_message("e.g., OPENROUTER_API_KEY");

            if let Some(d) = current_api_key_env {
                prompt = prompt.with_default(d);
            }

            let env_var = prompt.prompt()?;
            let env_var = env_var.trim().to_string();

            if env_var.is_empty() {
                bail!("Environment variable name cannot be empty");
            }

            Ok((None, Some(env_var)))
        }
        "Store in config file" => {
            let mut prompt =
                Text::new("API key:").with_help_message("Will be stored in plain text");

            if let Some(d) = current_api_key {
                prompt = prompt.with_default(d);
            }

            let api_key = prompt.prompt()?;
            let api_key = api_key.trim().to_string();

            if api_key.is_empty() {
                bail!("API key cannot be empty");
            }

            Ok((Some(api_key), None))
        }
        "None (no auth required)" => Ok((None, None)),
        _ => unreachable!(),
    }
}

fn input_models(current: Option<&Vec<String>>) -> Result<Vec<String>> {
    let default = current.map(|m| m.join(", ")).unwrap_or_default();

    let mut prompt = Text::new("Models (comma-separated, optional):")
        .with_help_message("e.g., gpt-4o, claude-3.5-sonnet");

    if !default.is_empty() {
        prompt = prompt.with_default(&default);
    }

    let input = prompt.prompt()?;

    let models: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(models)
}
