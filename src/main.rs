use anyhow::Result;
use clap::Parser;

use tl_cli::cli::commands::{chat, providers, translate};
use tl_cli::cli::{Args, Command};
use tl_cli::translation::{print_languages, validate_language};
use tl_cli::ui::Style;

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {err}", Style::error("Error:"));
        std::process::exit(1);
    }
}

#[tokio::main]
async fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Languages) => {
            print_languages();
        }
        Some(Command::Providers { provider }) => {
            providers::print_providers(provider.as_deref())?;
        }
        Some(Command::Chat {
            to,
            provider,
            model,
        }) => {
            if let Some(ref lang) = to {
                validate_language(lang)?;
            }

            let options = chat::ChatOptions {
                to,
                provider,
                model,
            };
            chat::run_chat(options).await?;
        }
        None => {
            if let Some(ref lang) = args.to {
                validate_language(lang)?;
            }

            let options = translate::TranslateOptions {
                file: args.file,
                to: args.to,
                provider: args.provider,
                model: args.model,
                no_cache: args.no_cache,
                write: args.write,
            };
            translate::run_translate(options).await?;
        }
    }

    Ok(())
}
