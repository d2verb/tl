use anyhow::{Result, bail};
use futures_util::StreamExt;
use std::io::{self, Write};

use crate::cache::CacheManager;
use crate::config::{Config, ConfigManager};
use crate::input::InputReader;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::Spinner;

pub struct TranslateOptions {
    pub file: Option<String>,
    pub to: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub no_cache: bool,
}

pub async fn run_translate(options: TranslateOptions) -> Result<()> {
    let config = load_merged_config(&options)?;

    let endpoint = config.endpoint.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Missing required configuration: 'endpoint'\n\n\
             Please provide it via:\n  \
             - CLI option: tl --endpoint <url> <file>\n  \
             - Config file: Run 'tl configure' to set up configuration"
        )
    })?;

    let model = config.model.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Missing required configuration: 'model'\n\n\
             Please provide it via:\n  \
             - CLI option: tl --model <name> <file>\n  \
             - Config file: Run 'tl configure' to set up configuration"
        )
    })?;

    let to = config.to.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Missing required configuration: 'to' (target language)\n\n\
             Please provide it via:\n  \
             - CLI option: tl --to <lang> <file>\n  \
             - Config file: Run 'tl configure' to set up configuration"
        )
    })?;

    let source_text = InputReader::read(options.file.as_deref())?;

    if source_text.is_empty() {
        bail!("Error: Input is empty");
    }

    let cache_manager = CacheManager::new()?;
    let client = TranslationClient::new(endpoint.clone());

    let request = TranslationRequest {
        source_text: source_text.clone(),
        target_language: to.clone(),
        model: model.clone(),
        endpoint: endpoint.clone(),
    };

    if !options.no_cache
        && let Some(cached) = cache_manager.get(&request)?
    {
        print!("{cached}");
        io::stdout().flush()?;
        return Ok(());
    }

    let spinner = Spinner::new("Translating...");
    spinner.start();

    let mut stream = client.translate_stream(&request).await?;
    let mut full_response = String::new();
    let mut first_chunk = true;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        if first_chunk {
            spinner.stop();
            first_chunk = false;
        }

        print!("{chunk}");
        io::stdout().flush()?;
        full_response.push_str(&chunk);
    }

    if first_chunk {
        spinner.stop();
    }

    if !full_response.is_empty() {
        println!();
    }

    if !options.no_cache && !full_response.is_empty() {
        cache_manager.put(&request, &full_response)?;
    }

    Ok(())
}

fn load_merged_config(options: &TranslateOptions) -> Result<Config> {
    let manager = ConfigManager::new()?;
    let file_config = manager.load().unwrap_or_default();

    Ok(Config {
        to: options.to.clone().or(file_config.to),
        endpoint: options.endpoint.clone().or(file_config.endpoint),
        model: options.model.clone().or(file_config.model),
    })
}
