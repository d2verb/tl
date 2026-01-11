//! Styles command handler for managing translation styles.

use anyhow::{Result, bail};
use inquire::{Confirm, Editor, Text};

use crate::config::{ConfigManager, CustomStyle};
use crate::style::{PRESETS, get_preset, is_preset, sorted_custom_keys, validate_custom_key};
use crate::ui::{Style, handle_prompt_cancellation};

/// Lists all available styles (presets and custom).
pub fn list_styles() -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    // Print preset styles
    println!("{}", Style::header("Preset styles"));
    for preset in PRESETS {
        println!(
            "  {}  {}",
            Style::value(format!("{:10}", preset.key)),
            Style::secondary(preset.description)
        );
    }

    // Print custom styles if any
    if !config.styles.is_empty() {
        println!();
        println!("{}", Style::header("Custom styles"));
        for key in sorted_custom_keys(&config.styles) {
            let description = config
                .styles
                .get(key)
                .map_or("", |s| s.description.as_str());
            println!(
                "  {}  {}",
                Style::value(format!("{key:10}")),
                Style::secondary(description)
            );
        }
    }

    Ok(())
}

/// Shows details of a style (description and prompt).
pub fn show_style(name: &str) -> Result<()> {
    // Check preset first
    if let Some(preset) = get_preset(name) {
        println!("{}", Style::header("Preset style"));
        println!();
        println!("  {}  {}", Style::label("Name:"), Style::value(preset.key));
        println!(
            "  {}  {}",
            Style::label("Desc:"),
            Style::secondary(preset.description)
        );
        println!();
        println!("{}", Style::label("Prompt:"));
        println!("{}", preset.prompt);
        return Ok(());
    }

    // Check custom styles
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    let custom = config
        .styles
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Style '{name}' not found"))?;

    println!("{}", Style::header("Custom style"));
    println!();
    println!("{}  {}", Style::label("Name:"), Style::value(name));
    println!(
        "{}  {}",
        Style::label("Desc:"),
        Style::secondary(&custom.description)
    );
    println!();
    println!("{}", Style::label("Prompt:"));
    println!("{}", custom.prompt);

    Ok(())
}

/// Adds a new custom style interactively.
pub fn add_style() -> Result<()> {
    handle_prompt_cancellation(add_style_inner)
}

fn add_style_inner() -> Result<()> {
    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Get style name
    let name = Text::new("Style name:")
        .with_help_message("Alphanumeric and underscores only (e.g., my_style)")
        .prompt()?;

    let name = name.trim().to_string();

    // Validate name
    validate_custom_key(&name).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Check if already exists
    if config.styles.contains_key(&name) {
        bail!("Style '{name}' already exists. Use 'tl styles edit {name}' to modify it.");
    }

    // Get style description (short, for display)
    let description = Text::new("Description:")
        .with_help_message("Short description for display (e.g., \"Ojisan-style texting\")")
        .prompt()?;

    let description = description.trim().to_string();

    if description.is_empty() {
        bail!("Description cannot be empty");
    }

    // Get style prompt (instructions for LLM) using editor
    let prompt = Editor::new("Prompt (opens editor):")
        .with_help_message("Instructions for the LLM. Save and close editor when done.")
        .with_predefined_text(
            "# Enter the prompt for the LLM below.\n# Lines starting with # are ignored.\n\n",
        )
        .prompt()?;

    let prompt = filter_comment_lines(&prompt);

    if prompt.is_empty() {
        bail!("Prompt cannot be empty");
    }

    // Save
    config.styles.insert(
        name.clone(),
        CustomStyle {
            description,
            prompt,
        },
    );
    manager.save(&config)?;

    println!();
    println!(
        "{} Style '{}' added",
        Style::success("✓"),
        Style::value(&name)
    );

    Ok(())
}

/// Filters out comment lines (starting with #) and trims the result.
fn filter_comment_lines(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Edits an existing custom style.
pub fn edit_style(name: &str) -> Result<()> {
    handle_prompt_cancellation(|| edit_style_inner(name))
}

fn edit_style_inner(name: &str) -> Result<()> {
    // Check if it's a preset
    if is_preset(name) {
        bail!("Cannot edit preset style '{name}'. Preset styles are immutable.");
    }

    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Check if exists
    let current = config.styles.get(name).cloned().ok_or_else(|| {
        anyhow::anyhow!("Style '{name}' not found. Use 'tl styles add' to create it.")
    })?;

    println!(
        "{} '{}':",
        Style::header("Editing style"),
        Style::value(name)
    );
    println!();

    // Get new description
    let description = Text::new("Description:")
        .with_default(&current.description)
        .prompt()?;

    let description = description.trim().to_string();

    if description.is_empty() {
        bail!("Description cannot be empty");
    }

    // Get new prompt using editor
    let prompt = Editor::new("Prompt (opens editor):")
        .with_help_message("Edit the prompt for the LLM. Save and close editor when done.")
        .with_predefined_text(&current.prompt)
        .prompt()?;

    let prompt = prompt.trim().to_string();

    if prompt.is_empty() {
        bail!("Prompt cannot be empty");
    }

    // Save
    config.styles.insert(
        name.to_string(),
        CustomStyle {
            description,
            prompt,
        },
    );
    manager.save(&config)?;

    println!();
    println!(
        "{} Style '{}' updated",
        Style::success("✓"),
        Style::value(name)
    );

    Ok(())
}

/// Removes a custom style.
pub fn remove_style(name: &str) -> Result<()> {
    handle_prompt_cancellation(|| remove_style_inner(name))
}

fn remove_style_inner(name: &str) -> Result<()> {
    // Check if it's a preset
    if is_preset(name) {
        bail!("Cannot remove preset style '{name}'. Preset styles are immutable.");
    }

    let manager = ConfigManager::new()?;
    let mut config = manager.load_or_default();

    // Check if exists
    if !config.styles.contains_key(name) {
        bail!("Style '{name}' not found");
    }

    // Confirm removal
    let confirm = Confirm::new(&format!("Remove style '{name}'?"))
        .with_default(false)
        .prompt()?;

    if !confirm {
        println!("Cancelled");
        return Ok(());
    }

    // Warn if it's the default style
    if config.tl.style.as_deref() == Some(name) {
        println!(
            "{} This is your default style. You may want to run 'tl configure' to set a new default.",
            Style::warning("Warning:")
        );
    }

    // Remove
    config.styles.remove(name);
    manager.save(&config)?;

    println!();
    println!(
        "{} Style '{}' removed",
        Style::success("✓"),
        Style::value(name)
    );

    Ok(())
}
