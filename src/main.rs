use anyhow::Result;
use clap::Parser;

use tl_cli::cli::commands::{chat, configure, providers, styles, translate};
use tl_cli::cli::{Args, Command, ProvidersCommand, StylesCommand};
use tl_cli::output::{self, OutputConfig};
use tl_cli::translation::{print_languages, validate_language};
use tl_cli::ui::Style;

fn main() {
    let args = Args::parse();

    // Initialize output configuration from CLI flags
    output::init(OutputConfig {
        quiet: args.quiet,
        no_color: args.no_color || std::env::var("NO_COLOR").is_ok(),
    });

    if let Err(err) = run(args) {
        eprintln!("{} {err}", Style::error("Error:"));
        let exit_code = classify_error(&err);
        std::process::exit(exit_code);
    }
}

/// Find `std::io::Error` in the error chain.
///
/// This is needed because anyhow errors often wrap the original `io::Error`
/// with context messages, so we need to traverse the chain to find it.
fn find_io_error(err: &anyhow::Error) -> Option<&std::io::Error> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<std::io::Error>())
}

/// Classify an error and return the appropriate exit code.
///
/// Uses BSD-style exit codes from `sysexits.h` via the `exitcode` crate.
/// These are cross-platform compatible (Windows, macOS, Linux).
fn classify_error(err: &anyhow::Error) -> exitcode::ExitCode {
    let err_str = err.to_string().to_lowercase();
    let chain_str: String = err
        .chain()
        .map(|e| e.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    // Check for file not found using ErrorKind first (cross-platform)
    // This handles both Unix ("No such file or directory") and
    // Windows ("The system cannot find the file specified") messages
    if let Some(io_err) = find_io_error(err)
        && io_err.kind() == std::io::ErrorKind::NotFound
    {
        return exitcode::NOINPUT;
    }

    // Check for file not found / input errors (NOINPUT = 66)
    // String-based fallback for wrapped errors or context messages
    if chain_str.contains("no such file")
        || chain_str.contains("failed to read")
        || chain_str.contains("failed to open")
        || (err_str.contains("input") && err_str.contains("empty"))
    {
        return exitcode::NOINPUT;
    }

    // Check for I/O errors - general file operations (IOERR = 74)
    if err.downcast_ref::<std::io::Error>().is_some()
        || chain_str.contains("permission denied")
        || chain_str.contains("failed to write")
        || chain_str.contains("failed to create")
    {
        return exitcode::IOERR;
    }

    // Check for authentication/permission errors (NOPERM = 77)
    if err_str.contains("api key")
        || err_str.contains("api_key")
        || err_str.contains("unauthorized")
        || err_str.contains("authentication")
        || err_str.contains("401")
        || chain_str.contains("api key")
    {
        return exitcode::NOPERM;
    }

    // Check for network/service unavailable errors (UNAVAILABLE = 69)
    if err.downcast_ref::<reqwest::Error>().is_some()
        || err_str.contains("connection")
        || err_str.contains("timeout")
        || err_str.contains("network")
        || err_str.contains("dns")
        || err_str.contains("failed to connect")
        || chain_str.contains("connection refused")
    {
        return exitcode::UNAVAILABLE;
    }

    // Check for configuration errors (CONFIG = 78)
    if chain_str.contains("config")
        || err_str.contains("provider")
        || err_str.contains("not configured")
    {
        return exitcode::CONFIG;
    }

    // Check for usage errors - invalid arguments, bad input (USAGE = 64)
    if err_str.contains("invalid")
        || err_str.contains("missing")
        || err_str.contains("not found")
        || err_str.contains("required")
        || err_str.contains("unsupported language")
    {
        return exitcode::USAGE;
    }

    // Default to software/internal error (SOFTWARE = 70)
    exitcode::SOFTWARE
}

#[tokio::main]
async fn run(args: Args) -> Result<()> {
    match args.command {
        Some(Command::Languages) => {
            print_languages();
        }
        Some(Command::Providers { command }) => match command {
            None => {
                providers::list_providers()?;
            }
            Some(ProvidersCommand::Add) => {
                providers::add_provider()?;
            }
            Some(ProvidersCommand::Edit { name }) => {
                providers::edit_provider(&name)?;
            }
            Some(ProvidersCommand::Remove { name }) => {
                providers::remove_provider(&name)?;
            }
        },
        Some(Command::Styles { command }) => match command {
            None => {
                styles::list_styles()?;
            }
            Some(StylesCommand::Add) => {
                styles::add_style()?;
            }
            Some(StylesCommand::Show { name }) => {
                styles::show_style(&name)?;
            }
            Some(StylesCommand::Edit { name }) => {
                styles::edit_style(&name)?;
            }
            Some(StylesCommand::Remove { name }) => {
                styles::remove_style(&name)?;
            }
        },
        Some(Command::Configure) => {
            configure::run_configure()?;
        }
        Some(Command::Chat {
            to,
            provider,
            model,
            style,
        }) => {
            if let Some(ref lang) = to {
                validate_language(lang)?;
            }

            let options = chat::ChatOptions {
                to,
                provider,
                model,
                style,
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
                style: args.style,
                no_cache: args.no_cache,
                write: args.write,
            };
            translate::run_translate(options).await?;
        }
    }

    Ok(())
}
