/// Chat session configuration (modifiable during session)
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub to: String,
    pub endpoint: String,
    pub model: String,
}

impl SessionConfig {
    pub const fn new(to: String, endpoint: String, model: String) -> Self {
        Self {
            to,
            endpoint,
            model,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "to" => self.to = value.to_string(),
            "endpoint" => self.endpoint = value.to_string(),
            "model" => self.model = value.to_string(),
            _ => return Err(format!("Unknown configuration key: '{key}'")),
        }
        Ok(())
    }
}

use anyhow::Result;
use futures_util::StreamExt;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::io::{self, Write};

use super::command::{Input, SlashCommand, parse_input};
use super::ui;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::Spinner;

pub struct ChatSession {
    config: SessionConfig,
    client: TranslationClient,
}

impl ChatSession {
    pub fn new(config: SessionConfig) -> Self {
        let client = TranslationClient::new(config.endpoint.clone());
        Self { config, client }
    }

    pub async fn run(&mut self) -> Result<()> {
        ui::print_header();

        let mut rl = DefaultEditor::new()?;

        loop {
            let input = rl.readline(&ui::prompt());

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
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
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
            SlashCommand::Set { key, value } => {
                match self.config.set(&key, &value) {
                    Ok(()) => {
                        if key == "endpoint" {
                            self.client = TranslationClient::new(self.config.endpoint.clone());
                        }
                        ui::print_set_success(&key, &value);
                    }
                    Err(e) => ui::print_error(&e),
                }
                true
            }
            SlashCommand::Clear => {
                ui::clear_screen();
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
        spinner.start();

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
