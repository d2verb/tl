use anyhow::{Result, bail};
use futures_util::StreamExt;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::cache::CacheManager;
use crate::config::{ConfigFile, ConfigManager, ResolvedConfig};
use crate::input::InputReader;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::Spinner;

/// Write content to file atomically using a temp file and rename.
/// This prevents file corruption if interrupted (e.g., Ctrl+C).
fn atomic_write(file_path: &str, content: &str) -> Result<()> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp_path = parent.join(format!(".{file_name}.tmp"));

    // Write to temp file first
    fs::write(&temp_path, content)?;

    // Atomic rename (same filesystem)
    fs::rename(&temp_path, file_path)?;

    Ok(())
}

pub struct TranslateOptions {
    pub file: Option<String>,
    pub to: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub no_cache: bool,
    pub write: bool,
}

pub async fn run_translate(options: TranslateOptions) -> Result<()> {
    // Validate -w option requires a file
    if options.write && options.file.is_none() {
        bail!("Error: --write requires a file argument (cannot write to stdin)");
    }

    let manager = ConfigManager::new()?;
    let config_file = manager.load_or_default();
    let resolved = resolve_config(&options, &config_file)?;

    let source_text = InputReader::read(options.file.as_deref())?;

    if source_text.is_empty() {
        bail!("Error: Input is empty");
    }

    let cache_manager = CacheManager::new()?;
    let client = TranslationClient::new(resolved.endpoint.clone(), resolved.api_key.clone());

    let request = TranslationRequest {
        source_text: source_text.clone(),
        target_language: resolved.target_language.clone(),
        model: resolved.model.clone(),
        endpoint: resolved.endpoint.clone(),
    };

    if !options.no_cache
        && let Some(cached) = cache_manager.get(&request)?
    {
        if options.write {
            if let Some(ref file_path) = options.file {
                atomic_write(file_path, &cached)?;
            }
        } else {
            print!("{cached}");
            io::stdout().flush()?;
        }
        return Ok(());
    }

    let spinner_msg = if options.write {
        format!(
            "Translating {}...",
            options.file.as_deref().unwrap_or("file")
        )
    } else {
        "Translating...".to_string()
    };
    let spinner = Spinner::new(&spinner_msg);

    let mut stream = client.translate_stream(&request).await?;
    let mut full_response = String::new();
    let mut spinner_active = true;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        // When streaming to stdout, stop spinner on first chunk to show output
        // When writing to file, keep spinner until completion
        if spinner_active && !options.write {
            spinner.stop();
            spinner_active = false;
        }

        if !options.write {
            print!("{chunk}");
            io::stdout().flush()?;
        }
        full_response.push_str(&chunk);
    }

    if spinner_active {
        spinner.stop();
    }

    if !options.write && !full_response.is_empty() {
        println!();
    }

    if !options.no_cache && !full_response.is_empty() {
        cache_manager.put(&request, &full_response)?;
    }

    // Write to file if -w is specified
    if options.write
        && !full_response.is_empty()
        && let Some(ref file_path) = options.file
    {
        atomic_write(file_path, &full_response)?;
    }

    Ok(())
}

pub fn resolve_config(
    options: &TranslateOptions,
    config_file: &ConfigFile,
) -> Result<ResolvedConfig> {
    // Resolve provider
    let provider_name = options
        .provider
        .clone()
        .or_else(|| config_file.tl.provider.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Error: Missing required configuration: 'provider'\n\n\
                 Please provide it via:\n  \
                 - CLI option: tl --provider <name>\n  \
                 - Config file: ~/.config/tl/config.toml"
            )
        })?;

    // Get provider config
    let provider_config = config_file.providers.get(&provider_name).ok_or_else(|| {
        let available: Vec<_> = config_file.providers.keys().collect();
        if available.is_empty() {
            anyhow::anyhow!(
                "Error: Provider '{provider_name}' not found\n\n\
                 No providers configured. Add providers to ~/.config/tl/config.toml"
            )
        } else {
            anyhow::anyhow!(
                "Error: Provider '{provider_name}' not found\n\n\
                 Available providers:\n  \
                 - {}\n\n\
                 Add providers to ~/.config/tl/config.toml",
                available
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n  - ")
            )
        }
    })?;

    // Resolve model
    let model = options
        .model
        .clone()
        .or_else(|| config_file.tl.model.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Error: Missing required configuration: 'model'\n\n\
                 Please provide it via:\n  \
                 - CLI option: tl --model <name>\n  \
                 - Config file: ~/.config/tl/config.toml"
            )
        })?;

    // Warn if model is not in provider's models list
    if !provider_config.models.is_empty() && !provider_config.models.contains(&model) {
        eprintln!(
            "Warning: Model '{}' is not in the configured models list for '{}'\n\
             Configured models: {}\n\
             Proceeding anyway...\n",
            model,
            provider_name,
            provider_config.models.join(", ")
        );
    }

    // Resolve target language
    let target_language = options
        .to
        .clone()
        .or_else(|| config_file.tl.to.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Error: Missing required configuration: 'to' (target language)\n\n\
                 Please provide it via:\n  \
                 - CLI option: tl --to <lang>\n  \
                 - Config file: ~/.config/tl/config.toml"
            )
        })?;

    // Get API key
    let api_key = provider_config.get_api_key();

    // Check if API key is required but missing
    if provider_config.requires_api_key() && api_key.is_none() {
        let env_var = provider_config.api_key_env.as_deref().unwrap_or("API_KEY");
        bail!(
            "Error: Provider '{provider_name}' requires an API key\n\n\
             Set the {env_var} environment variable:\n  \
             export {env_var}=\"your-api-key\"\n\n\
             Or set api_key in ~/.config/tl/config.toml"
        );
    }

    Ok(ResolvedConfig {
        provider_name,
        endpoint: provider_config.endpoint.clone(),
        model,
        api_key,
        target_language,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProviderConfig, TlConfig};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_options() -> TranslateOptions {
        TranslateOptions {
            file: None,
            to: Some("ja".to_string()),
            provider: Some("ollama".to_string()),
            model: Some("gemma3:12b".to_string()),
            no_cache: false,
            write: false,
        }
    }

    fn create_test_config() -> ConfigFile {
        let mut providers = HashMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                endpoint: "http://localhost:11434".to_string(),
                api_key: None,
                api_key_env: None,
                models: vec!["gemma3:12b".to_string()],
            },
        );
        providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                endpoint: "https://openrouter.ai/api".to_string(),
                api_key: None,
                api_key_env: Some("TL_TEST_NONEXISTENT_API_KEY".to_string()),
                models: vec!["gpt-4o".to_string()],
            },
        );

        ConfigFile {
            tl: TlConfig {
                provider: Some("ollama".to_string()),
                model: Some("gemma3:12b".to_string()),
                to: Some("ja".to_string()),
            },
            providers,
        }
    }

    // atomic_write tests

    #[test]
    fn test_atomic_write_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        atomic_write(file_path_str, "Hello, World!").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        fs::write(&file_path, "Original content").unwrap();
        atomic_write(file_path_str, "New content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "New content");
    }

    #[test]
    fn test_atomic_write_no_temp_file_remains() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        atomic_write(file_path_str, "content").unwrap();

        // Temp file should not exist after successful write
        let temp_path = temp_dir.path().join(".test.txt.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_atomic_write_unicode_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let content = "こんにちは世界！🌍";
        atomic_write(file_path_str, content).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    // resolve_config tests

    #[test]
    fn test_resolve_config_with_cli_options() {
        let options = create_test_options();
        let config = create_test_config();

        let resolved = resolve_config(&options, &config).unwrap();

        assert_eq!(resolved.provider_name, "ollama");
        assert_eq!(resolved.endpoint, "http://localhost:11434");
        assert_eq!(resolved.model, "gemma3:12b");
        assert_eq!(resolved.target_language, "ja");
        assert!(resolved.api_key.is_none());
    }

    #[test]
    fn test_resolve_config_cli_overrides_file() {
        let mut options = create_test_options();
        options.to = Some("en".to_string());
        options.model = Some("llama3".to_string());

        let config = create_test_config();

        let resolved = resolve_config(&options, &config).unwrap();

        assert_eq!(resolved.target_language, "en");
        assert_eq!(resolved.model, "llama3");
    }

    #[test]
    fn test_resolve_config_falls_back_to_file() {
        let options = TranslateOptions {
            file: None,
            to: None,
            provider: None,
            model: None,
            no_cache: false,
            write: false,
        };
        let config = create_test_config();

        let resolved = resolve_config(&options, &config).unwrap();

        assert_eq!(resolved.provider_name, "ollama");
        assert_eq!(resolved.model, "gemma3:12b");
        assert_eq!(resolved.target_language, "ja");
    }

    #[test]
    fn test_resolve_config_missing_provider() {
        let options = TranslateOptions {
            file: None,
            to: Some("ja".to_string()),
            provider: None,
            model: Some("model".to_string()),
            no_cache: false,
            write: false,
        };
        let config = ConfigFile {
            tl: TlConfig {
                provider: None,
                model: None,
                to: None,
            },
            providers: HashMap::new(),
        };

        let result = resolve_config(&options, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("provider"));
    }

    #[test]
    fn test_resolve_config_provider_not_found() {
        let mut options = create_test_options();
        options.provider = Some("nonexistent".to_string());

        let config = create_test_config();

        let result = resolve_config(&options, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_resolve_config_missing_model() {
        let mut options = create_test_options();
        options.model = None;

        let mut config = create_test_config();
        config.tl.model = None;

        let result = resolve_config(&options, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("model"));
    }

    #[test]
    fn test_resolve_config_missing_target_language() {
        let mut options = create_test_options();
        options.to = None;

        let mut config = create_test_config();
        config.tl.to = None;

        let result = resolve_config(&options, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("to"));
    }

    #[test]
    fn test_resolve_config_api_key_required_but_missing() {
        let mut options = create_test_options();
        options.provider = Some("openrouter".to_string());

        let config = create_test_config();

        let result = resolve_config(&options, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }
}
