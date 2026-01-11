//! CLI argument definitions using clap.

use clap::{Parser, Subcommand};

/// Command-line arguments for the `tl` CLI.
#[derive(Parser, Debug)]
#[command(name = "tl")]
#[command(about = "AI-powered translation CLI tool")]
#[command(version)]
pub struct Args {
    /// File to translate (reads from stdin if not provided)
    pub file: Option<String>,

    /// Target language code (ISO 639-1, e.g., ja, en, zh)
    #[arg(short = 't', long = "to")]
    pub to: Option<String>,

    /// Provider name (e.g., ollama, openrouter)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// Model name
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Translation style (e.g., casual, formal, literal, natural)
    #[arg(short = 's', long)]
    pub style: Option<String>,

    /// Disable cache
    #[arg(short = 'n', long)]
    pub no_cache: bool,

    /// Overwrite the input file with the translated content
    #[arg(short = 'w', long)]
    pub write: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// List supported language codes
    Languages,
    /// Manage providers (list all if no subcommand given)
    Providers {
        #[command(subcommand)]
        command: Option<ProvidersCommand>,
    },
    /// Manage translation styles (list all if no subcommand given)
    Styles {
        #[command(subcommand)]
        command: Option<StylesCommand>,
    },
    /// Interactive chat mode for translation
    Chat {
        /// Target language code (ISO 639-1, e.g., ja, en, zh)
        #[arg(short = 't', long = "to")]
        to: Option<String>,

        /// Provider name (e.g., ollama, openrouter)
        #[arg(short = 'p', long)]
        provider: Option<String>,

        /// Model name
        #[arg(short = 'm', long)]
        model: Option<String>,

        /// Translation style (e.g., casual, formal, literal, natural)
        #[arg(short = 's', long)]
        style: Option<String>,
    },
    /// Configure default settings
    Configure,
}

/// Subcommands for provider management.
#[derive(Subcommand, Debug)]
pub enum ProvidersCommand {
    /// Add a new provider
    Add,
    /// Edit an existing provider
    Edit {
        /// Provider name to edit
        name: String,
    },
    /// Remove a provider
    Remove {
        /// Provider name to remove
        name: String,
    },
}

/// Subcommands for style management.
#[derive(Subcommand, Debug)]
pub enum StylesCommand {
    /// Add a new custom style
    Add,
    /// Show details of a style (description and prompt)
    Show {
        /// Style name to show
        name: String,
    },
    /// Edit an existing custom style
    Edit {
        /// Style name to edit
        name: String,
    },
    /// Remove a custom style
    Remove {
        /// Style name to remove
        name: String,
    },
}
