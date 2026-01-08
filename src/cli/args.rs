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

    /// API endpoint URL
    #[arg(short = 'e', long)]
    pub endpoint: Option<String>,

    /// Model name
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Disable cache
    #[arg(short = 'n', long)]
    pub no_cache: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Configure tl settings
    Configure {
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
    /// List supported language codes
    Languages,
    /// Interactive chat mode for translation
    Chat {
        /// Target language code (ISO 639-1, e.g., ja, en, zh)
        #[arg(short = 't', long = "to")]
        to: Option<String>,

        /// API endpoint URL
        #[arg(short = 'e', long)]
        endpoint: Option<String>,

        /// Model name
        #[arg(short = 'm', long)]
        model: Option<String>,
    },
}
