use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::paths;

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
}
