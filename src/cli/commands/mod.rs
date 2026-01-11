//! Subcommand implementations.

use anyhow::Result;

use crate::config::{ConfigFile, ConfigManager};

/// Chat mode command handler.
pub mod chat;

/// Configure command handler.
pub mod configure;

/// Provider management command handler.
pub mod providers;

/// Style management command handler.
pub mod styles;

/// Translation command handler.
pub mod translate;

/// Loads the configuration file.
///
/// Returns defaults if the config file doesn't exist.
/// Fails if the config file exists but is invalid or unreadable.
pub fn load_config() -> Result<(ConfigManager, ConfigFile)> {
    let manager = ConfigManager::new()?;
    let config = manager.load_or_default()?;
    Ok((manager, config))
}
