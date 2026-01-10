use anyhow::Result;
use futures_util::StreamExt;
use inquire::Text;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use std::io::{self, Write};

use super::command::{Input, SlashCommand, SlashCommandCompleter, parse_input};
use super::ui;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::Spinner;

/// Configuration for a chat session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// The provider name.
    pub provider_name: String,
    /// The API endpoint URL.
    pub endpoint: String,
    /// The model to use.
    pub model: String,
    /// The API key (if required).
    pub api_key: Option<String>,
    /// The target language code.
    pub to: String,
}

impl SessionConfig {
    /// Creates a new session configuration.
    pub const fn new(
        provider_name: String,
        endpoint: String,
        model: String,
        api_key: Option<String>,
        to: String,
    ) -> Self {
        Self {
            provider_name,
            endpoint,
            model,
            api_key,
            to,
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
        let client = TranslationClient::new(config.endpoint.clone(), config.api_key.clone());
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

    fn handle_command(&self, cmd: SlashCommand) -> bool {
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
            SlashCommand::Unknown(cmd) => {
                ui::print_error(&format!("Unknown command: /{cmd}"));
                true
            }
        }
    }

    async fn translate_and_print(&self, text: &str) -> Result<()> {
        let request = TranslationRequest {
            source_text: text.to_string(),
            target_language: self.config.to.clone(),
            model: self.config.model.clone(),
            endpoint: self.config.endpoint.clone(),
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
        let config = SessionConfig::new(
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
            "gemma3:12b".to_string(),
            None,
            "ja".to_string(),
        );

        assert_eq!(config.provider_name, "ollama");
        assert_eq!(config.endpoint, "http://localhost:11434");
        assert_eq!(config.model, "gemma3:12b");
        assert!(config.api_key.is_none());
        assert_eq!(config.to, "ja");
    }
}
