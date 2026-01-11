use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::paths;
use crate::ui::Style;

/// Default settings in the `[tl]` section of config.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlConfig {
    /// Default provider name.
    pub provider: Option<String>,
    /// Default model name.
    pub model: Option<String>,
    /// Default target language (ISO 639-1 code).
    pub to: Option<String>,
}

/// Configuration for a translation provider.
///
/// Each provider has an endpoint and optional API key settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// The OpenAI-compatible API endpoint URL.
    pub endpoint: String,
    /// API key stored directly in config (not recommended).
    #[serde(default)]
    pub api_key: Option<String>,
    /// Environment variable name containing the API key.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// List of available models for this provider.
    #[serde(default)]
    pub models: Vec<String>,
}

impl ProviderConfig {
    /// Gets the API key, preferring environment variable over config file.
    pub fn get_api_key(&self) -> Option<String> {
        if let Some(env_var) = &self.api_key_env
            && let Ok(key) = std::env::var(env_var)
            && !key.is_empty()
        {
            return Some(key);
        }
        self.api_key.clone()
    }

    /// Returns `true` if this provider requires an API key.
    pub const fn requires_api_key(&self) -> bool {
        self.api_key.is_some() || self.api_key_env.is_some()
    }
}

/// The complete configuration file structure.
///
/// Corresponds to `~/.config/tl/config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Default settings.
    #[serde(default)]
    pub tl: TlConfig,
    /// Provider configurations keyed by name.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Resolved configuration after merging CLI arguments and config file.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// The selected provider name.
    pub provider_name: String,
    /// The API endpoint URL.
    pub endpoint: String,
    /// The model to use for translation.
    pub model: String,
    /// The API key (if required).
    pub api_key: Option<String>,
    /// The target language code.
    pub target_language: String,
}

/// Options for resolving configuration.
///
/// Contains CLI overrides that take precedence over config file values.
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    /// Target language code override.
    pub to: Option<String>,
    /// Provider name override.
    pub provider: Option<String>,
    /// Model name override.
    pub model: Option<String>,
}

/// Resolves configuration by merging CLI options with config file settings.
///
/// CLI options take precedence over config file values.
///
/// # Errors
///
/// Returns an error if required configuration (provider, model, target language)
/// is missing or if the specified provider is not found.
pub fn resolve_config(
    options: &ResolveOptions,
    config_file: &ConfigFile,
) -> Result<ResolvedConfig> {
    // Resolve provider
    let provider_name = options
        .provider
        .as_ref()
        .or(config_file.tl.provider.as_ref())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing required configuration: 'provider'\n\n\
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
                "Provider '{provider_name}' not found\n\n\
                 No providers configured. Add providers to ~/.config/tl/config.toml"
            )
        } else {
            anyhow::anyhow!(
                "Provider '{provider_name}' not found\n\n\
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
        .as_ref()
        .or(config_file.tl.model.as_ref())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing required configuration: 'model'\n\n\
                 Please provide it via:\n  \
                 - CLI option: tl --model <name>\n  \
                 - Config file: ~/.config/tl/config.toml"
            )
        })?;

    // Warn if model is not in provider's models list
    if !provider_config.models.is_empty() && !provider_config.models.contains(&model) {
        eprintln!(
            "{} Model '{}' is not in the configured models list for '{}'\n\
             Configured models: {}\n\
             Proceeding anyway...\n",
            Style::warning("Warning:"),
            model,
            provider_name,
            provider_config.models.join(", ")
        );
    }

    // Resolve target language
    let target_language = options
        .to
        .as_ref()
        .or(config_file.tl.to.as_ref())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing required configuration: 'to' (target language)\n\n\
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
            "Provider '{provider_name}' requires an API key\n\n\
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

/// Manages loading and saving configuration files.
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new config manager.
    ///
    /// Configuration is stored at `$XDG_CONFIG_HOME/tl/config.toml`
    /// or `~/.config/tl/config.toml` if `XDG_CONFIG_HOME` is not set.
    pub fn new() -> Result<Self> {
        Ok(Self {
            config_path: paths::config_dir()?.join("config.toml"),
        })
    }

    pub const fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn load(&self) -> Result<ConfigFile> {
        let contents = fs::read_to_string(&self.config_path).with_context(|| {
            format!("Failed to read config file: {}", self.config_path.display())
        })?;

        let config_file: ConfigFile =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        Ok(config_file)
    }

    pub fn save(&self, config: &ConfigFile) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(&self.config_path, contents).with_context(|| {
            format!(
                "Failed to write config file: {}",
                self.config_path.display()
            )
        })?;

        Ok(())
    }

    pub fn load_or_default(&self) -> ConfigFile {
        self.load().unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager(temp_dir: &TempDir) -> ConfigManager {
        ConfigManager {
            config_path: temp_dir.path().join("config.toml"),
        }
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let mut providers = HashMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                endpoint: "http://localhost:11434".to_string(),
                api_key: None,
                api_key_env: None,
                models: vec!["gemma3:12b".to_string(), "llama3.2".to_string()],
            },
        );

        let config = ConfigFile {
            tl: TlConfig {
                provider: Some("ollama".to_string()),
                model: Some("gemma3:12b".to_string()),
                to: Some("ja".to_string()),
            },
            providers,
        };

        manager.save(&config).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.tl.provider, Some("ollama".to_string()));
        assert_eq!(loaded.tl.model, Some("gemma3:12b".to_string()));
        assert_eq!(loaded.tl.to, Some("ja".to_string()));
        assert!(loaded.providers.contains_key("ollama"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let result = manager.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_get_api_key_from_env() {
        // SAFETY: This test runs in isolation and only modifies a test-specific env var
        unsafe {
            std::env::set_var("TEST_API_KEY", "test-key-value");
        }

        let provider = ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: Some("fallback-key".to_string()),
            api_key_env: Some("TEST_API_KEY".to_string()),
            models: vec![],
        };

        // Environment variable takes priority
        assert_eq!(provider.get_api_key(), Some("test-key-value".to_string()));

        // SAFETY: Cleanup test env var
        unsafe {
            std::env::remove_var("TEST_API_KEY");
        }
    }

    #[test]
    fn test_provider_get_api_key_fallback() {
        // SAFETY: This test runs in isolation and only modifies a test-specific env var
        unsafe {
            std::env::remove_var("NONEXISTENT_KEY");
        }

        let provider = ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: Some("fallback-key".to_string()),
            api_key_env: Some("NONEXISTENT_KEY".to_string()),
            models: vec![],
        };

        // Falls back to api_key when env var not set
        assert_eq!(provider.get_api_key(), Some("fallback-key".to_string()));
    }

    #[test]
    fn test_provider_requires_api_key() {
        let provider_with_key = ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: Some("key".to_string()),
            api_key_env: None,
            models: vec![],
        };
        assert!(provider_with_key.requires_api_key());

        let provider_with_env = ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: None,
            api_key_env: Some("API_KEY".to_string()),
            models: vec![],
        };
        assert!(provider_with_env.requires_api_key());

        let provider_without = ProviderConfig {
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
            api_key_env: None,
            models: vec![],
        };
        assert!(!provider_without.requires_api_key());
    }

    // resolve_config tests

    fn create_test_options() -> ResolveOptions {
        ResolveOptions {
            to: Some("ja".to_string()),
            provider: Some("ollama".to_string()),
            model: Some("gemma3:12b".to_string()),
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
        let options = ResolveOptions::default();
        let config = create_test_config();

        let resolved = resolve_config(&options, &config).unwrap();

        assert_eq!(resolved.provider_name, "ollama");
        assert_eq!(resolved.model, "gemma3:12b");
        assert_eq!(resolved.target_language, "ja");
    }

    #[test]
    fn test_resolve_config_missing_provider() {
        let options = ResolveOptions {
            to: Some("ja".to_string()),
            provider: None,
            model: Some("model".to_string()),
        };
        let config = ConfigFile::default();

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
