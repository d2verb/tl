use anyhow::Result;
use clap::Parser;

use tl::cli::commands::{chat, configure, translate};
use tl::cli::{Args, Command};

const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("af", "Afrikaans"),
    ("ar", "Arabic"),
    ("bg", "Bulgarian"),
    ("bn", "Bengali"),
    ("ca", "Catalan"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("fa", "Persian"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("id", "Indonesian"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("ms", "Malay"),
    ("nl", "Dutch"),
    ("no", "Norwegian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("sr", "Serbian"),
    ("sv", "Swedish"),
    ("ta", "Tamil"),
    ("th", "Thai"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("vi", "Vietnamese"),
    ("zh", "Chinese"),
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Configure { show }) => {
            if show {
                configure::run_configure_show()?;
            } else {
                configure::run_configure()?;
            }
        }
        Some(Command::Languages) => {
            print_languages();
        }
        Some(Command::Chat {
            to,
            endpoint,
            model,
        }) => {
            if let Some(ref lang) = to {
                validate_language(lang)?;
            }

            let options = chat::ChatOptions {
                to,
                endpoint,
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
                endpoint: args.endpoint,
                model: args.model,
                no_cache: args.no_cache,
            };
            translate::run_translate(options).await?;
        }
    }

    Ok(())
}
