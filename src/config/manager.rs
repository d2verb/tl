use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub to: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    tl: Config,
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("tl");

        Ok(Self {
            config_path: config_dir.join("config.toml"),
        })
    }

    pub const fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn load(&self) -> Result<Config> {
        let contents = fs::read_to_string(&self.config_path).with_context(|| {
            format!("Failed to read config file: {}", self.config_path.display())
        })?;

        let config_file: ConfigFile =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        Ok(config_file.tl)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let config_file = ConfigFile { tl: config.clone() };
        let contents =
            toml::to_string_pretty(&config_file).context("Failed to serialize config")?;

        fs::write(&self.config_path, contents).with_context(|| {
            format!(
                "Failed to write config file: {}",
                self.config_path.display()
            )
        })?;

        Ok(())
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

        let config = Config {
            to: Some("ja".to_string()),
            endpoint: Some("http://localhost:11434".to_string()),
            model: Some("gpt-oss:20b".to_string()),
        };

        manager.save(&config).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.to, Some("ja".to_string()));
        assert_eq!(loaded.endpoint, Some("http://localhost:11434".to_string()));
        assert_eq!(loaded.model, Some("gpt-oss:20b".to_string()));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let result = manager.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_save_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let config = Config {
            to: Some("en".to_string()),
            endpoint: None,
            model: None,
        };

        manager.save(&config).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.to, Some("en".to_string()));
        assert_eq!(loaded.endpoint, None);
        assert_eq!(loaded.model, None);
    }
}
