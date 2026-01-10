use anyhow::Result;
use clap::Parser;

use tl_cli::cli::commands::{chat, translate};
use tl_cli::cli::{Args, Command};
use tl_cli::config::ConfigManager;

const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("af", "Afrikaans"),
    ("am", "Amharic"),
    ("ar", "Arabic"),
    ("az", "Azerbaijani"),
    ("be", "Belarusian"),
    ("bg", "Bulgarian"),
    ("bn", "Bengali"),
    ("bs", "Bosnian"),
    ("ca", "Catalan"),
    ("cs", "Czech"),
    ("cy", "Welsh"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("eu", "Basque"),
    ("fa", "Persian"),
    ("fi", "Finnish"),
    ("fil", "Filipino"),
    ("fr", "French"),
    ("ga", "Irish"),
    ("gl", "Galician"),
    ("gu", "Gujarati"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("hy", "Armenian"),
    ("id", "Indonesian"),
    ("is", "Icelandic"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ka", "Georgian"),
    ("kk", "Kazakh"),
    ("km", "Khmer"),
    ("kn", "Kannada"),
    ("ko", "Korean"),
    ("la", "Latin"),
    ("lo", "Lao"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("mk", "Macedonian"),
    ("ml", "Malayalam"),
    ("mn", "Mongolian"),
    ("mr", "Marathi"),
    ("ms", "Malay"),
    ("mt", "Maltese"),
    ("my", "Myanmar (Burmese)"),
    ("ne", "Nepali"),
    ("nl", "Dutch"),
    ("no", "Norwegian"),
    ("pa", "Punjabi"),
    ("pl", "Polish"),
    ("ps", "Pashto"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("si", "Sinhala"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("sq", "Albanian"),
    ("sr", "Serbian"),
    ("sv", "Swedish"),
    ("sw", "Swahili"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("th", "Thai"),
    ("tl", "Tagalog"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("uz", "Uzbek"),
    ("vi", "Vietnamese"),
    ("zh", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
];

fn print_languages() {
    println!("Supported language codes (ISO 639-1):\n");
    for (code, name) in SUPPORTED_LANGUAGES {
        println!("  {code:4} - {name}");
    }
}

fn validate_language(lang: &str) -> Result<()> {
    if SUPPORTED_LANGUAGES.iter().any(|(code, _)| *code == lang) {
        Ok(())
    } else {
        anyhow::bail!(
            "Error: Invalid language code: '{lang}'\n\n\
             Valid language codes (ISO 639-1): ja, en, zh, ko, fr, de, es, ...\n\
             Run 'tl languages' to see all supported codes."
        )
    }
}

fn print_providers(specific_provider: Option<&str>) -> Result<()> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default();

    if config.providers.is_empty() {
        println!("No providers configured.");
        println!("Add providers to ~/.config/tl/config.toml");
        return Ok(());
    }

    let default_provider = config.tl.provider.as_deref();

    if let Some(provider_name) = specific_provider {
        // Show details for a specific provider
        if let Some(provider) = config.providers.get(provider_name) {
            let is_default = default_provider == Some(provider_name);
            println!(
                "Provider: {}{}",
                provider_name,
                if is_default { " (default)" } else { "" }
            );
            println!("  endpoint = {}", provider.endpoint);
            if provider.api_key_env.is_some() || provider.api_key.is_some() {
                let has_key = provider.get_api_key().is_some();
                println!(
                    "  api_key  = {}",
                    if has_key { "(set)" } else { "(not set)" }
                );
            }
            if provider.models.is_empty() {
                println!("  models   = (none configured)");
            } else {
                println!("  models:");
                for model in &provider.models {
                    println!("    - {model}");
                }
            }
        } else {
            anyhow::bail!("Provider '{provider_name}' not found");
        }
    } else {
        // List all providers
        println!("Configured providers:");
        for (name, provider) in &config.providers {
            let is_default = default_provider == Some(name.as_str());
            println!("  {}{}", name, if is_default { " (default)" } else { "" });
            println!("    endpoint: {}", provider.endpoint);
            if !provider.models.is_empty() {
                println!("    models: {}", provider.models.join(", "));
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Languages) => {
            print_languages();
        }
        Some(Command::Providers { provider }) => {
            print_providers(provider.as_deref())?;
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
