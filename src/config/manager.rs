use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// [tl] セクションの設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub to: Option<String>,
}

/// プロバイダー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub endpoint: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

impl ProviderConfig {
    /// API Key を取得（環境変数優先）
    pub fn get_api_key(&self) -> Option<String> {
        if let Some(env_var) = &self.api_key_env
            && let Ok(key) = std::env::var(env_var)
            && !key.is_empty()
        {
            return Some(key);
        }
        self.api_key.clone()
    }

    /// API Key が必要かどうか
    pub const fn requires_api_key(&self) -> bool {
        self.api_key.is_some() || self.api_key_env.is_some()
    }
}

/// 設定ファイル全体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub tl: TlConfig,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// 解決済みの設定（CLI引数とconfigファイルをマージ済み）
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub provider_name: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub target_language: String,
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .context("Failed to determine home directory")?
            .join(".config")
            .join("tl");

        Ok(Self {
            config_path: config_dir.join("config.toml"),
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
