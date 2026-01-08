use anyhow::Result;
use rustyline::DefaultEditor;

use crate::config::{Config, ConfigManager};

fn prompt_with_default(rl: &mut DefaultEditor, prompt: &str, default: &str) -> Result<String> {
    let full_prompt = if default.is_empty() {
        format!("{prompt}: ")
    } else {
        format!("{prompt} [{default}]: ")
    };

    let line = rl.readline(&full_prompt)?;
    let line = line.trim();

    if line.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(line.to_string())
    }
}

pub fn run_configure() -> Result<()> {
    let manager = ConfigManager::new()?;
    let existing_config = manager.load().ok();

    println!("tl Configuration");
    println!("{}", "─".repeat(16));

    let default_to = existing_config
        .as_ref()
        .and_then(|c| c.to.clone())
        .unwrap_or_default();
    let default_endpoint = existing_config
        .as_ref()
        .and_then(|c| c.endpoint.clone())
        .unwrap_or_default();
    let default_model = existing_config
        .as_ref()
        .and_then(|c| c.model.clone())
        .unwrap_or_default();

    let mut rl = DefaultEditor::new()?;

    let to = prompt_with_default(&mut rl, "Target language (to)", &default_to)?;
    let endpoint = prompt_with_default(&mut rl, "API endpoint", &default_endpoint)?;
    let model = prompt_with_default(&mut rl, "Model name", &default_model)?;

    let config = Config {
        to: if to.is_empty() { None } else { Some(to) },
        endpoint: if endpoint.is_empty() {
            None
        } else {
            Some(endpoint)
        },
        model: if model.is_empty() { None } else { Some(model) },
    };

    manager.save(&config)?;

    println!();
    println!("Configuration saved to {}", manager.config_path().display());

    Ok(())
}

pub fn run_configure_show() -> Result<()> {
    let manager = ConfigManager::new()?;

    if let Ok(config) = manager.load() {
        println!("Configuration file: {}", manager.config_path().display());
        println!();
        println!("to       = {}", config.to.as_deref().unwrap_or("(not set)"));
        println!(
            "endpoint = {}",
            config.endpoint.as_deref().unwrap_or("(not set)")
        );
        println!(
            "model    = {}",
            config.model.as_deref().unwrap_or("(not set)")
        );
    } else {
        println!("No configuration file found.");
        println!("Run 'tl configure' to create one.");
    }

    Ok(())
}
