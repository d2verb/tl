use clap::{Parser, Subcommand};

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

    /// Disable cache
    #[arg(short = 'n', long)]
    pub no_cache: bool,

    /// Overwrite the input file with the translated content
    #[arg(short = 'w', long)]
    pub write: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List supported language codes
    Languages,
    /// List configured providers and their models
    Providers {
        /// Show models for a specific provider
        provider: Option<String>,
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
    },
}
