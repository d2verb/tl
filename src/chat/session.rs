use anyhow::Result;
use futures_util::StreamExt;
use inquire::Text;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use std::collections::HashMap;
use std::io::{self, Write};

use super::command::{Input, SlashCommand, SlashCommandCompleter, parse_input};
use super::ui;
use crate::config::{CustomStyle, ResolvedConfig};
use crate::style;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::{Spinner, Style};

/// Configuration for a chat session.
///
/// Wraps [`ResolvedConfig`] and adds chat-specific fields.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// The resolved configuration (provider, model, language, style).
    pub resolved: ResolvedConfig,
    /// Available custom styles (cached from config file).
    pub custom_styles: HashMap<String, CustomStyle>,
}

impl SessionConfig {
    /// Creates a new session configuration.
    #[allow(clippy::missing_const_for_fn)] // HashMap can't be used in const context
    pub fn new(resolved: ResolvedConfig, custom_styles: HashMap<String, CustomStyle>) -> Self {
        Self {
            resolved,
            custom_styles,
        }
    }
}

/// An interactive chat session for translation.
///
/// Provides a REPL-style interface for translating text interactively.
pub struct ChatSession {
    config: SessionConfig,
    client: TranslationClient,
}

impl ChatSession {
    /// Creates a new chat session with the given configuration.
    pub fn new(config: SessionConfig) -> Self {
        let client = TranslationClient::new(
            config.resolved.endpoint.clone(),
            config.resolved.api_key.clone(),
        );
        Self { config, client }
    }

    pub async fn run(&mut self) -> Result<()> {
        ui::print_header();

        let prompt_style = Styled::new("❯")
            .with_fg(Color::LightBlue)
            .with_attr(Attributes::BOLD);
        let mut render_config = RenderConfig::default()
            .with_prompt_prefix(prompt_style)
            .with_answered_prompt_prefix(prompt_style);

        // Non-highlighted suggestions: gray
        render_config.option = StyleSheet::new().with_fg(Color::Grey);
        // Highlighted suggestion: purple
        render_config.selected_option = Some(StyleSheet::new().with_fg(Color::DarkMagenta));

        loop {
            let input = Text::new("")
                .with_render_config(render_config)
                .with_autocomplete(SlashCommandCompleter)
                .with_help_message("Type text to translate, /help for commands, Ctrl+C to quit")
                .prompt();

            match input {
                Ok(line) => match parse_input(&line) {
                    Input::Empty => {}
                    Input::Command(cmd) => {
                        if !self.handle_command(cmd) {
                            break;
                        }
                    }
                    Input::Text(text) => {
                        self.translate_and_print(&text).await?;
                    }
                },
                Err(
                    inquire::InquireError::OperationCanceled
                    | inquire::InquireError::OperationInterrupted,
                ) => {
                    println!(); // Clear line before goodbye message
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        ui::print_goodbye();
        Ok(())
    }

    fn handle_command(&mut self, cmd: SlashCommand) -> bool {
        match cmd {
            SlashCommand::Config => {
                ui::print_config(&self.config);
                true
            }
            SlashCommand::Help => {
                ui::print_help();
                true
            }
            SlashCommand::Quit => false,
            SlashCommand::Set { key, value } => {
                self.handle_set(&key, value.as_deref());
                true
            }
            SlashCommand::Unknown(cmd) => {
                ui::print_error(&format!("Unknown command: /{cmd}"));
                true
            }
        }
    }

    fn handle_set(&mut self, key: &str, value: Option<&str>) {
        match key {
            "style" => self.set_style(value),
            "to" => self.set_to(value),
            "model" => self.set_model(value),
            "" => {
                println!("Usage: /set <key> <value>");
                println!("Keys: style, to, model");
            }
            _ => {
                ui::print_error(&format!("Unknown setting: {key}"));
                println!("Available: style, to, model");
            }
        }
    }

    fn set_style(&mut self, value: Option<&str>) {
        let Some(key) = value else {
            // Clear style
            self.config.resolved.style_name = None;
            self.config.resolved.style_prompt = None;
            println!("{} Style cleared", Style::success("✓"));
            return;
        };

        // Resolve style using cached custom_styles
        let resolved = match style::resolve_style(key, &self.config.custom_styles) {
            Ok(r) => r,
            Err(e) => {
                ui::print_error(&e.to_string());
                return;
            }
        };

        self.config.resolved.style_name = Some(key.to_string());
        self.config.resolved.style_prompt = Some(resolved.prompt().to_string());
        println!(
            "{} Style set to {}\n",
            Style::success("✓"),
            Style::value(key)
        );
    }

    fn set_to(&mut self, value: Option<&str>) {
        match value {
            None => {
                ui::print_error("Usage: /set to <language>");
            }
            Some(lang) => {
                self.config.resolved.target_language = lang.to_string();
                println!(
                    "{} Target language set to {}",
                    Style::success("✓"),
                    Style::value(lang)
                );
            }
        }
    }

    fn set_model(&mut self, value: Option<&str>) {
        match value {
            None => {
                ui::print_error("Usage: /set model <name>");
            }
            Some(model) => {
                self.config.resolved.model = model.to_string();
                println!(
                    "{} Model set to {}",
                    Style::success("✓"),
                    Style::value(model)
                );
            }
        }
    }

    async fn translate_and_print(&self, text: &str) -> Result<()> {
        let request = TranslationRequest {
            source_text: text.to_string(),
            target_language: self.config.resolved.target_language.clone(),
            model: self.config.resolved.model.clone(),
            endpoint: self.config.resolved.endpoint.clone(),
            style: self.config.resolved.style_prompt.clone(),
        };

        let spinner = Spinner::new("Translating...");

        let mut stream = self.client.translate_stream(&request).await?;
        let mut first_chunk = true;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;

            if first_chunk {
                spinner.stop();
                first_chunk = false;
            }

            print!("{chunk}");
            io::stdout().flush()?;
        }

        if first_chunk {
            spinner.stop();
        }

        println!();
        println!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_new() {
        let mut custom_styles = HashMap::new();
        custom_styles.insert(
            "my_style".to_string(),
            CustomStyle {
                description: "My description".to_string(),
                prompt: "My custom prompt".to_string(),
            },
        );

        let resolved = ResolvedConfig {
            provider_name: "ollama".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            model: "gemma3:12b".to_string(),
            api_key: None,
            target_language: "ja".to_string(),
            style_name: Some("casual".to_string()),
            style_prompt: Some("Use a casual tone.".to_string()),
        };

        let config = SessionConfig::new(resolved, custom_styles);

        assert_eq!(config.resolved.provider_name, "ollama");
        assert_eq!(config.resolved.endpoint, "http://localhost:11434");
        assert_eq!(config.resolved.model, "gemma3:12b");
        assert!(config.resolved.api_key.is_none());
        assert_eq!(config.resolved.target_language, "ja");
        assert_eq!(config.resolved.style_name, Some("casual".to_string()));
        assert_eq!(
            config.resolved.style_prompt,
            Some("Use a casual tone.".to_string())
        );
        assert_eq!(
            config.custom_styles.get("my_style").map(|s| &s.prompt),
            Some(&"My custom prompt".to_string())
        );
    }
}
