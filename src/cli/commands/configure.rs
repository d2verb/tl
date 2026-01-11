//! Configure command handler for editing default settings.

use anyhow::{Result, bail};
use inquire::{Select, Text};

use crate::config::{ConfigFile, ConfigManager, TlConfig};
use crate::translation::SUPPORTED_LANGUAGES;
use crate::ui::Style;

/// Runs the configure command to edit default settings.
///
/// Allows the user to interactively set the default provider, model, and target language.
pub fn run_configure() -> Result<()> {
    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Check if at least one provider is configured
    if config.providers.is_empty() {
        bail!(
            "No providers configured.\n\n\
             Run 'tl providers add' to add a provider first."
        );
    }

    // Display current defaults
    print_current_defaults(&config);

    // Get provider names for selection
    let provider_names: Vec<String> = config.providers.keys().cloned().collect();

    // Select default provider
    let default_provider = config.tl.provider.clone();
    let provider = select_provider(&provider_names, default_provider.as_deref())?;

    // Get models for the selected provider
    let provider_config = config.providers.get(&provider);
    let available_models: Vec<String> = provider_config
        .map(|p| p.models.clone())
        .unwrap_or_default();

    // Select default model
    let default_model = config.tl.model.clone();
    let model = select_model(&available_models, default_model.as_deref())?;

    // Select default target language
    let default_to = config.tl.to.clone();
    let to = select_target_language(default_to.as_deref())?;

    // Update config
    config.tl = TlConfig {
        provider: Some(provider),
        model: Some(model),
        to: Some(to),
    };

    // Save config
    manager.save(&config)?;

    println!();
    println!(
        "{} Configuration saved to {}",
        Style::success("✓"),
        Style::secondary(manager.config_path().display().to_string())
    );

    Ok(())
}

fn print_current_defaults(config: &ConfigFile) {
    println!("{}", Style::header("Current defaults"));
    println!(
        "  {}  {}",
        Style::label("provider"),
        config
            .tl
            .provider
            .as_deref()
            .map_or_else(|| Style::secondary("(not set)"), Style::value)
    );
    println!(
        "  {}     {}",
        Style::label("model"),
        config
            .tl
            .model
            .as_deref()
            .map_or_else(|| Style::secondary("(not set)"), Style::value)
    );
    println!(
        "  {}        {}",
        Style::label("to"),
        config
            .tl
            .to
            .as_deref()
            .map_or_else(|| Style::secondary("(not set)"), Style::value)
    );
    println!();
}

fn select_provider(providers: &[String], default: Option<&str>) -> Result<String> {
    let default_index = default
        .and_then(|d| providers.iter().position(|p| p == d))
        .unwrap_or(0);

    let selection = Select::new("Default provider:", providers.to_vec())
        .with_starting_cursor(default_index)
        .prompt()?;

    Ok(selection)
}

fn select_model(available_models: &[String], default: Option<&str>) -> Result<String> {
    if available_models.is_empty() {
        // No models configured, fall back to text input
        let mut prompt = Text::new("Default model:").with_help_message("Enter the model name");

        if let Some(d) = default {
            prompt = prompt.with_default(d);
        }

        let model = prompt.prompt()?;

        if model.trim().is_empty() {
            bail!("Model name cannot be empty");
        }

        Ok(model.trim().to_string())
    } else {
        // Models available, use selection
        let default_index = default
            .and_then(|d| available_models.iter().position(|m| m == d))
            .unwrap_or(0);

        let selection = Select::new("Default model:", available_models.to_vec())
            .with_starting_cursor(default_index)
            .prompt()?;

        Ok(selection)
    }
}

fn select_target_language(default: Option<&str>) -> Result<String> {
    // Build options with format "code - Name"
    let options: Vec<String> = SUPPORTED_LANGUAGES
        .iter()
        .map(|(code, name)| format!("{code} - {name}"))
        .collect();

    let default_index = default
        .and_then(|d| SUPPORTED_LANGUAGES.iter().position(|(code, _)| *code == d))
        .unwrap_or(0);

    let selection = Select::new("Default target language:", options)
        .with_starting_cursor(default_index)
        .prompt()?;

    // Extract code from "code - Name" format
    let code = selection
        .split(" - ")
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?;

    Ok(code.to_string())
}
