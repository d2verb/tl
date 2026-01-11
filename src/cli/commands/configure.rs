//! Configure command handler for editing default settings.

use anyhow::{Result, bail};
use inquire::{Select, Text};

use crate::config::{ConfigFile, ConfigManager, TlConfig};
use crate::style::{PRESETS, sorted_custom_keys};
use crate::translation::SUPPORTED_LANGUAGES;
use crate::ui::{Style, handle_prompt_cancellation};

/// Runs the configure command to edit default settings.
///
/// Allows the user to interactively set the default provider, model, and target language.
pub fn run_configure() -> Result<()> {
    handle_prompt_cancellation(run_configure_inner)
}

fn run_configure_inner() -> Result<()> {
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

    // Select default style (optional)
    let default_style = config.tl.style.clone();
    let style = select_style(&config, default_style.as_deref())?;

    // Update config
    config.tl = TlConfig {
        provider: Some(provider),
        model: Some(model),
        to: Some(to),
        style,
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
    println!(
        "  {}     {}",
        Style::label("style"),
        config
            .tl
            .style
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
    // split() always returns at least one element, but we use unwrap_or as fallback
    let code = selection.split(" - ").next().unwrap_or(&selection);

    Ok(code.to_string())
}

fn select_style(config: &ConfigFile, default: Option<&str>) -> Result<Option<String>> {
    // Build options: "(none)" + presets + custom styles
    let mut options: Vec<String> = vec!["(none)".to_string()];

    // Add presets
    for preset in PRESETS {
        options.push(format!("{} - {}", preset.key, preset.description));
    }

    // Add custom styles
    let custom_keys = sorted_custom_keys(&config.styles);
    for key in &custom_keys {
        let desc = config
            .styles
            .get(*key)
            .map_or("", |s| s.description.as_str());
        options.push(format!("{key} - {desc}"));
    }

    // Find default index
    let default_index = default
        .and_then(|d| {
            // Check presets
            if let Some(idx) = PRESETS.iter().position(|p| p.key == d) {
                return Some(idx + 1); // +1 for "(none)"
            }
            // Check custom styles
            if let Some(idx) = custom_keys.iter().position(|k| *k == d) {
                return Some(PRESETS.len() + 1 + idx);
            }
            None
        })
        .unwrap_or(0);

    let selection = Select::new("Default style:", options)
        .with_starting_cursor(default_index)
        .prompt()?;

    // Parse selection
    if selection == "(none)" {
        return Ok(None);
    }

    // Extract key from "key - description" format
    let key = selection.split(" - ").next().unwrap_or(&selection);
    Ok(Some(key.to_string()))
}
